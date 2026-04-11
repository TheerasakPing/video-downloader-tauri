use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::proxy;
use crate::{SearchResult, SiteCategory};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaanJeenSeriesInfo {
    pub url: String,
    pub title: String,
    pub total_episodes: i32,
    pub poster_url: Option<String>,
    pub episode_urls: HashMap<i32, String>,
    pub iframe_urls: Vec<String>,
}

pub struct BaanJeenParser {
    proxy_config: Arc<RwLock<proxy::ProxyConfig>>,
}

impl BaanJeenParser {
    pub fn new(proxy_config: Arc<RwLock<proxy::ProxyConfig>>) -> Self {
        Self { proxy_config }
    }

    fn client(&self) -> Client {
        proxy::build_client(&self.proxy_config.read().unwrap())
    }

    /// Check if URL is from บ้านจีน.com or configured dynamic domain
    pub fn is_baanjeen_url(url: &str, domain: &str) -> bool {
        url.contains(domain) || url.contains("xn--82c7abb4jua0l.com") || url.contains("บ้านจีน.com")
    }

    /// Extract series slug from URL
    /// Example: https://xn--82c7abb4jua0l.com/some-series-name/ -> "some-series-name"
    #[allow(dead_code)]
    pub fn parse_series_url(url: &str) -> Option<String> {
        // Match pattern: domain/slug/ or domain/slug
        let re = Regex::new(r"https?://[^/]+/([^/]+)/?$").ok()?;
        if let Some(caps) = re.captures(url) {
            return Some(caps.get(1)?.as_str().to_string());
        }
        None
    }

    /// Fetch series information from configured Baanjeen domain
    pub async fn get_series_info(&self, series_url: &str, domain: &str) -> Result<BaanJeenSeriesInfo, String> {
        eprintln!("[BaanJeen] Fetching page: {} (Domain config: {})", series_url, domain);

        // Fetch the main page
        let response = self
            .client()
            .get(series_url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "th,en-US;q=0.9,en;q=0.8")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        eprintln!("[BaanJeen] HTML length: {} bytes", html.len());

        // Extract all data from document in a synchronous block (before any await)
        let (title, poster_url, iframe_sources, episode_links, script_sources) = {
            let document = Html::parse_document(&html);

            // Extract title
            let title = self.extract_title(&document, &html);

            // Extract poster URL
            let poster_url = self.extract_poster(&document);

            // Extract iframe sources
            let iframe_selector = Selector::parse("iframe").unwrap();
            let iframe_sources: Vec<String> = document
                .select(&iframe_selector)
                .filter_map(|iframe| {
                    iframe.value().attr("src").or_else(|| iframe.value().attr("data-lazy-src"))
                        .map(|s| s.to_string())
                })
                .collect();

            // Extract episode links
            let link_selector = Selector::parse("a[href*='ตอนที่'], a[href*='ep-'], a[href*='episode']").unwrap();
            let episode_links: Vec<(String, String)> = document
                .select(&link_selector)
                .map(|link| {
                    let href = link.value().attr("href").unwrap_or("").to_string();
                    let text = link.text().collect::<String>();
                    (text, href)
                })
                .collect();

            // Extract external JavaScript sources
            let script_selector = Selector::parse("script[src]").unwrap();
            let script_sources: Vec<String> = document
                .select(&script_selector)
                .filter_map(|script| script.value().attr("src").map(|s| s.to_string()))
                .collect();

            (title, poster_url, iframe_sources, episode_links, script_sources)
        }; // document is dropped here, before any await

        eprintln!("[BaanJeen] Found {} iframe sources", iframe_sources.len());
        eprintln!("[BaanJeen] Found {} script sources", script_sources.len());

        // Now process the extracted data (async operations are safe here)
        let mut episode_urls = HashMap::new();

        // Method 1: Check iframe sources for video URLs
        for src in &iframe_sources {
            if let Some(video_url) = self.extract_video_from_iframe(src).await {
                eprintln!("[BaanJeen] Found video in iframe: {}", video_url);
                episode_urls.insert(1, video_url);
                let total_episodes = 1;
                let poster_data_url = if let Some(ref url) = poster_url {
                    self.fetch_image_as_data_url(url).await
                } else {
                    None
                };

                return Ok(BaanJeenSeriesInfo {
                    url: series_url.to_string(),
                    title,
                    total_episodes,
                    poster_url: poster_data_url,
                    episode_urls,
                    iframe_urls: iframe_sources,
                });
            }
        }

        // Method 2: Look for video URLs in main HTML
        eprintln!("[BaanJeen] Searching for video URLs in HTML...");
        if let Some(video_url) = self.search_video_urls(&html) {
            eprintln!("[BaanJeen] Found video URL in HTML: {}", video_url);
            episode_urls.insert(1, video_url);
            let total_episodes = 1;
            let poster_data_url = if let Some(ref url) = poster_url {
                self.fetch_image_as_data_url(url).await
            } else {
                None
            };

            return Ok(BaanJeenSeriesInfo {
                url: series_url.to_string(),
                title,
                total_episodes,
                poster_url: poster_data_url,
                episode_urls,
                iframe_urls: iframe_sources,
            });
        }

        // Method 3: Download and search external JavaScript files
        eprintln!("[BaanJeen] Searching in {} external JS files...", script_sources.len());
        for script_src in script_sources {
            // Make script URL absolute
            let script_url = if script_src.starts_with("http") {
                script_src.clone()
            } else if script_src.starts_with("//") {
                format!("https:{}", script_src)
            } else if script_src.starts_with("/") {
                format!("https://{}{}", domain, script_src)
            } else {
                format!("https://{}/{}", domain, script_src)
            };

            eprintln!("[BaanJeen] Fetching JS: {}", script_url);

            match self.client().get(&script_url).send().await {
                Ok(response) => {
                    if let Ok(js_content) = response.text().await {
                        if let Some(video_url) = self.search_video_urls(&js_content) {
                            eprintln!("[BaanJeen] Found video URL in JS file: {}", video_url);
                            episode_urls.insert(1, video_url);
                            let total_episodes = 1;
                            let poster_data_url = if let Some(ref url) = poster_url {
                                self.fetch_image_as_data_url(url).await
                            } else {
                                None
                            };

                            return Ok(BaanJeenSeriesInfo {
                                url: series_url.to_string(),
                                title,
                                total_episodes,
                                poster_url: poster_data_url,
                                episode_urls,
                                iframe_urls: iframe_sources,
                            });
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[BaanJeen] Failed to fetch JS file: {}", e);
                }
            }
        }

        // Method 4: Process episode links
        for (index, (text, href)) in episode_links.iter().enumerate() {
            let ep_num = self.extract_episode_number(text, href).unwrap_or((index + 1) as i32);
            episode_urls.insert(ep_num, href.clone());
        }

        // Even if no video URLs found, still return series info with title and poster
        if episode_urls.is_empty() {
            eprintln!("[BaanJeen] No video URLs found with automatic parsing");
            eprintln!("[BaanJeen] Returning series info without video URLs");

            // Return series info with empty episode URLs but with title and poster
            let total_episodes = 0; // Indicate no episodes found

            // Convert poster to data URL if available
            let poster_data_url = if let Some(ref url) = poster_url {
                self.fetch_image_as_data_url(url).await
            } else {
                None
            };

            return Ok(BaanJeenSeriesInfo {
                url: series_url.to_string(),
                title: if title.is_empty() {
                    "ไม่พบชื่อเรื่อง".to_string()
                } else {
                    title
                },
                total_episodes,
                poster_url: poster_data_url,
                episode_urls: HashMap::new(), // Empty - no video URLs found
                iframe_urls: iframe_sources,
            });
        }

        let total_episodes = episode_urls.keys().max().copied().unwrap_or(1);

        // Convert poster to data URL if available
        let poster_data_url = if let Some(ref url) = poster_url {
            self.fetch_image_as_data_url(url).await
        } else {
            None
        };

        Ok(BaanJeenSeriesInfo {
            url: series_url.to_string(),
            title,
            total_episodes,
            poster_url: poster_data_url,
            episode_urls,
            iframe_urls: iframe_sources,
        })
    }

    /// Search for video URLs in text content (HTML or JavaScript)
    fn search_video_urls(&self, content: &str) -> Option<String> {
        // Support both normal and escaped URLs
        let video_patterns = vec![
            // HLS streaming URLs (.m3u8) - with optional escaping
            Regex::new(r#"https?:(?:\\/\\/|//)[^"'\\s<>\\]+\.m3u8[^"'\\s<>\\]*"#).unwrap(),
            // Discord CDN URLs - with optional escaping
            Regex::new(r#"https?:(?:\\/\\/|//)cdn\.discordapp\.com(?:\\/|/)attachments(?:\\/|/)\d+(?:\\/|/)\d+(?:\\/|/)[^"'\\s<>\\]+"#).unwrap(),
            // Direct MP4 URLs - with optional escaping
            Regex::new(r#"https?:(?:\\/\\/|//)[^"'\\s<>\\]+\.mp4[^"'\\s<>\\]*"#).unwrap(),
        ];

        for pattern in video_patterns {
            if let Some(m) = pattern.find(content) {
                let mut url = m.as_str().to_string();
                // Clean up escaped URLs
                url = url.replace("\\/", "/");
                url = url.replace("\\u0026", "&");
                url = url.replace("&amp;", "&");

                eprintln!("[BaanJeen] Found video URL: {}", url);
                return Some(url);
            }
        }

        None
    }

    /// Extract title from page
    fn extract_title(&self, document: &Html, _html: &str) -> String {
        // Try og:title first
        let og_title_selector = Selector::parse("meta[property='og:title']").unwrap();
        if let Some(og_title) = document.select(&og_title_selector).next() {
            if let Some(content) = og_title.value().attr("content") {
                let title = content.to_string();
                // Clean up common suffixes
                let re = Regex::new(r"\s*ละครสั้นจีนโรงหยก\s*$").unwrap();
                return re.replace(&title, "").trim().to_string();
            }
        }

        // Fallback to <title> tag
        let title_selector = Selector::parse("title").unwrap();
        if let Some(title_el) = document.select(&title_selector).next() {
            let title = title_el.text().collect::<String>();
            let re = Regex::new(r"\s*ละครสั้นจีนโรงหยก\s*$").unwrap();
            return re.replace(&title, "").trim().to_string();
        }

        "Unknown Series".to_string()
    }

    /// Extract poster image URL
    fn extract_poster(&self, document: &Html) -> Option<String> {
        // Try og:image
        let og_image_selector = Selector::parse("meta[property='og:image']").unwrap();
        if let Some(og_image) = document.select(&og_image_selector).next() {
            if let Some(content) = og_image.value().attr("content") {
                return Some(content.to_string());
            }
        }

        // Try featured image in content
        let img_selector = Selector::parse("article img, .entry-content img").unwrap();
        if let Some(img) = document.select(&img_selector).next() {
            if let Some(src) = img.value().attr("src").or_else(|| img.value().attr("data-lazy-src")) {
                return Some(src.to_string());
            }
        }

        None
    }

    /// Extract video URL from iframe source
    async fn extract_video_from_iframe(&self, iframe_src: &str) -> Option<String> {
        // If iframe src is already a direct video URL
        if iframe_src.ends_with(".mp4") || iframe_src.contains("cdn.discordapp.com") {
            return Some(iframe_src.to_string());
        }

        // Otherwise, fetch the iframe page to find the video
        match self.client().get(iframe_src).send().await {
            Ok(response) => {
                if let Ok(html) = response.text().await {
                    // Look for video source in iframe content
                    let patterns = vec![
                        Regex::new(r#"https?://cdn\.discordapp\.com/attachments/\d+/\d+/[^"'\\s<>]+"#).unwrap(),
                        Regex::new(r#"https?://[^"'\\s<>]+\.mp4[^"'\\s<>]*"#).unwrap(),
                    ];

                    for pattern in patterns {
                        if let Some(m) = pattern.find(&html) {
                            return Some(m.as_str().to_string());
                        }
                    }
                }
            }
            Err(_) => {}
        }

        None
    }

    /// Extract episode number from text or URL
    fn extract_episode_number(&self, text: &str, url: &str) -> Option<i32> {
        // Try Thai pattern: ตอนที่ X
        let re_thai = Regex::new(r"ตอนที่\s*(\d+)").unwrap();
        if let Some(caps) = re_thai.captures(text) {
            if let Ok(num) = caps.get(1)?.as_str().parse::<i32>() {
                return Some(num);
            }
        }

        // Try English patterns: ep-X, episode-X
        let re_en = Regex::new(r"(?i)ep(?:isode)?[-_]?(\d+)").unwrap();
        if let Some(caps) = re_en.captures(url) {
            if let Ok(num) = caps.get(1)?.as_str().parse::<i32>() {
                return Some(num);
            }
        }

        None
    }

    /// Fetch an image and convert it to a base64 data URL
    async fn fetch_image_as_data_url(&self, image_url: &str) -> Option<String> {
        let response = self
            .client()
            .get(image_url)
            .header("Accept", "image/*")
            .send()
            .await
            .ok()?;

        // Get content type
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();

        // Get image bytes
        let bytes = response.bytes().await.ok()?;

        // Convert to base64
        let base64_data = BASE64.encode(&bytes);

        // Return as data URL
        Some(format!("data:{};base64,{}", content_type, base64_data))
    }

    pub async fn search(&self, query: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
        let url = format!("https://{}?s={}&paged={}", domain, query, page);
        let resp = self.client().get(&url).send().await.map_err(|e| e.to_string())?;
        let html_text = resp.text().await.map_err(|e| e.to_string())?;
        self.parse_listing_html(&html_text, domain, "baanjeen")
    }

    pub fn list_categories(&self, _domain: &str) -> Vec<SiteCategory> {
        vec![
            SiteCategory { id: "latest".into(), label: "Latest".into(), source: "baanjeen".into() },
            SiteCategory { id: "chinese-series".into(), label: "Chinese Series".into(), source: "baanjeen".into() },
            SiteCategory { id: "chinese-movie".into(), label: "Chinese Movies".into(), source: "baanjeen".into() },
        ]
    }

    pub async fn browse(&self, category: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
        let url = match category {
            "latest" => format!("https://{}?paged={}", domain, page),
            other => format!("https://{}/category/{}/?paged={}", domain, other, page),
        };
        let resp = self.client().get(&url).send().await.map_err(|e| e.to_string())?;
        let html_text = resp.text().await.map_err(|e| e.to_string())?;
        self.parse_listing_html(&html_text, domain, "baanjeen")
    }

    fn parse_listing_html(&self, html_text: &str, _domain: &str, source: &str) -> Result<Vec<SearchResult>, String> {
        let document = Html::parse_document(html_text);
        let mut results = Vec::new();
        let article_sel = Selector::parse("article, .post-item, .entry-item").unwrap();

        for el in document.select(&article_sel) {
            let link_sel = Selector::parse("a").unwrap();
            let link = el.select(&link_sel).next().and_then(|a| a.value().attr("href")).map(|h| h.to_string());
            let title = el.select(&Selector::parse("h2 a, h3 a, .entry-title a").unwrap()).next()
                .map(|a| a.text().collect::<String>().trim().to_string())
                .or_else(|| el.select(&link_sel).next().and_then(|a| a.value().attr("title")).map(|t| t.to_string()));
            let poster = el.select(&Selector::parse("img").unwrap()).next()
                .and_then(|img| img.value().attr("src").or_else(|| img.value().attr("data-src")).map(|s| s.to_string()));

            if let (Some(url), Some(title)) = (link, title) {
                if title.is_empty() || url.contains("wp-admin") { continue; }
                results.push(SearchResult {
                    title,
                    poster_url: poster,
                    url,
                    source: source.to_string(),
                    total_episodes: None,
                });
            }
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_baanjeen_url() {
        assert!(BaanJeenParser::is_baanjeen_url("https://xn--82c7abb4jua0l.com/some-series/", "xn--82c7abb4jua0l.com"));
        assert!(BaanJeenParser::is_baanjeen_url("https://บ้านจีน.com/some-series/", "xn--82c7abb4jua0l.com"));
        assert!(!BaanJeenParser::is_baanjeen_url("https://rongyok.com/watch/?series_id=1004", "xn--82c7abb4jua0l.com"));
        assert!(BaanJeenParser::is_baanjeen_url("https://bjn-new-domain.com/some-series/", "bjn-new-domain.com"));
    }

    #[test]
    fn test_parse_series_url() {
        assert_eq!(
            BaanJeenParser::parse_series_url("https://xn--82c7abb4jua0l.com/some-series-name/"),
            Some("some-series-name".to_string())
        );
        assert_eq!(
            BaanJeenParser::parse_series_url("https://xn--82c7abb4jua0l.com/%e0%b8%ab%e0%b8%a5%e0%b8%b1%e0%b8%81%e0%b8%aa%e0%b8%b9%e0%b8%95%e0%b8%a3/"),
            Some("%e0%b8%ab%e0%b8%a5%e0%b8%b1%e0%b8%81%e0%b8%aa%e0%b8%b9%e0%b8%95%e0%b8%a3".to_string())
        );
    }
}
