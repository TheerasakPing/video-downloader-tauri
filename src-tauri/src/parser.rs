use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesInfo {
    pub series_id: i32,
    pub title: String,
    pub total_episodes: i32,
    pub poster_url: Option<String>,
    pub episode_urls: HashMap<i32, String>,
}

pub struct RongyokParser {
    client: Client,
}

impl RongyokParser {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Extract series_id from URL
    pub fn parse_series_url(url: &str) -> Option<i32> {
        // Format 1: ?series_id=XXX
        let re1 = Regex::new(r"series_id=(\d+)").ok()?;
        if let Some(caps) = re1.captures(url) {
            return caps.get(1)?.as_str().parse().ok();
        }

        // Format 2: /series/XXX/
        let re2 = Regex::new(r"/series/(\d+)").ok()?;
        if let Some(caps) = re2.captures(url) {
            return caps.get(1)?.as_str().parse().ok();
        }

        None
    }

    /// Fetch series information
    pub async fn get_series_info(&self, series_id: i32, original_url: Option<&str>) -> Result<SeriesInfo, String> {
        let (url, domain) = Self::construct_series_url(series_id, original_url);
        let is_thongyok = url.contains("thongyok.com");

        let response = self
            .client
            .get(&url)
            .header("Accept", "text/html,application/xhtml+xml")
            .header("Accept-Language", "th,en-US;q=0.9,en;q=0.8")
            .header("Referer", &domain)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        // Extract episode URLs from HTML (doesn't need document)
        let mut episode_urls = self.extract_all_episode_urls(&html);

        // Parse document in a block to ensure it's dropped before any await
        let (title, poster_url, total_episodes_from_doc) = {
            let document = Html::parse_document(&html);

            // Get title
            let title_selector = Selector::parse("title").unwrap();
            let title = document
                .select(&title_selector)
                .next()
                .map(|el| el.text().collect::<String>())
                .unwrap_or_else(|| format!("Series {}", series_id));

            // Clean title - remove " - ตอนที่ X" suffix and site name
            let title_re = Regex::new(r"\s*-\s*ตอนที่\s*\d+.*$").unwrap();
            let title = title_re.replace(&title, "").to_string();
            let title_re2 = Regex::new(r"\s*(Thongyok|Rongyok).*$").unwrap();
            let title = title_re2.replace(&title, "").to_string().trim().to_string();

            // Get poster URL
            let og_image_selector = Selector::parse("meta[property='og:image']").unwrap();
            let poster_url = document
                .select(&og_image_selector)
                .next()
                .and_then(|el| el.value().attr("content"))
                .map(|s| s.to_string());

            // Extract total episodes from document
            let total_eps = self.extract_total_episodes(&document, &html);

            (title, poster_url, total_eps)
        }; // document is dropped here

        // For thongyok.com, video URLs are on individual episode pages
        if is_thongyok && episode_urls.is_empty() {
            episode_urls = self.fetch_thongyok_episode_urls(series_id, total_episodes_from_doc, &domain).await?;
        }

        let total_episodes = if episode_urls.is_empty() {
            total_episodes_from_doc
        } else {
            episode_urls.keys().max().copied().unwrap_or(1)
        };

        Ok(SeriesInfo {
            series_id,
            title,
            total_episodes,
            poster_url,
            episode_urls,
        })
    }

    /// Fetch episode URLs from thongyok.com individual episode pages
    async fn fetch_thongyok_episode_urls(&self, series_id: i32, total_episodes: i32, domain: &str) -> Result<HashMap<i32, String>, String> {
        let mut episode_urls = HashMap::new();

        for ep in 1..=total_episodes {
            let watch_url = format!("https://thongyok.com/watch/{}/{}", series_id, ep);

            match self.client
                .get(&watch_url)
                .header("Accept", "text/html,application/xhtml+xml")
                .header("Accept-Language", "th,en-US;q=0.9,en;q=0.8")
                .header("Referer", domain)
                .send()
                .await
            {
                Ok(response) => {
                    if let Ok(html) = response.text().await {
                        // Extract video URL from episode page
                        if let Some(video_url) = self.extract_video_url_from_page(&html) {
                            episode_urls.insert(ep, video_url);
                        }
                    }
                }
                Err(_) => continue, // Skip failed requests
            }
        }

        Ok(episode_urls)
    }

    /// Extract a single video URL from an episode page
    fn extract_video_url_from_page(&self, html: &str) -> Option<String> {
        // Pattern: Discord CDN URL
        let pattern = Regex::new(
            r#"https?://cdn\.discordapp\.com/attachments/\d+/\d+/\d+\.mp4\?[^"'\s<>]+"#
        ).ok()?;

        if let Some(m) = pattern.find(html) {
            let mut url = m.as_str().to_string();
            url = url.replace("&amp;", "&");
            return Some(url);
        }

        None
    }

    /// Helper to construct URL and domain
    fn construct_series_url(series_id: i32, original_url: Option<&str>) -> (String, String) {
        let url = if let Some(orig) = original_url {
            if orig.contains("thongyok.com") {
                orig.to_string()
            } else {
                format!("https://rongyok.com/watch/?series_id={}", series_id)
            }
        } else {
            format!("https://rongyok.com/watch/?series_id={}", series_id)
        };

        let domain = if url.contains("thongyok.com") {
            "https://thongyok.com/"
        } else {
            "https://rongyok.com/"
        };

        (url, domain.to_string())
    }

    /// Extract all episode URLs from JavaScript
    fn extract_all_episode_urls(&self, html: &str) -> HashMap<i32, String> {
        let mut episode_urls = HashMap::new();

        // Pattern 1: Discord CDN with numeric filename (e.g., 1.mp4, 2.mp4)
        let pattern1 = Regex::new(
            r#"https?:(?:\\/\\/|//)cdn\.discordapp\.com(?:\\/|/)attachments(?:\\/|/)(\d+)(?:\\/|/)(\d+)(?:\\/|/)(\d+)\.mp4\?[^"'<>\s\\]+"#
        ).unwrap();

        for caps in pattern1.captures_iter(html) {
            if let (Some(full_match), Some(ep_num)) = (caps.get(0), caps.get(3)) {
                if let Ok(ep) = ep_num.as_str().parse::<i32>() {
                    let mut url = full_match.as_str().to_string();
                    url = url.replace("\\/", "/");
                    url = url.replace("\\u0026", "&");
                    url = url.replace("&amp;", "&");
                    episode_urls.insert(ep, url);
                }
            }
        }

        // Pattern 2: Discord CDN with EP prefix (e.g., EP01.mp4, EP02.mp4)
        let pattern2 = Regex::new(
            r#"(?i)https?:(?:\\/\\/|//)cdn\.discordapp\.com(?:\\/|/)attachments(?:\\/|/)(\d+)(?:\\/|/)(\d+)(?:\\/|/)EP(\d+)\.mp4\?[^"'<>\s\\]+"#
        ).unwrap();

        for caps in pattern2.captures_iter(html) {
            if let (Some(full_match), Some(ep_num)) = (caps.get(0), caps.get(3)) {
                if let Ok(ep) = ep_num.as_str().parse::<i32>() {
                    let mut url = full_match.as_str().to_string();
                    url = url.replace("\\/", "/");
                    url = url.replace("\\u0026", "&");
                    url = url.replace("&amp;", "&");
                    episode_urls.entry(ep).or_insert(url);
                }
            }
        }

        // Pattern 3: Generic video_url in JSON
        let pattern3 = Regex::new(r#""video_url"\s*:\s*"(https?:[^"]+\.mp4[^"]*)""#).unwrap();

        for caps in pattern3.captures_iter(html) {
            if let Some(url_match) = caps.get(1) {
                let mut url = url_match.as_str().to_string();
                url = url.replace("\\/", "/");
                url = url.replace("\\u0026", "&");

                // Try to extract episode number from URL
                let ep_re = Regex::new(r#"[/\\](?:EP)?(\d+)\.mp4"#).unwrap();
                if let Some(ep_caps) = ep_re.captures(&url) {
                    if let Ok(ep) = ep_caps.get(1).unwrap().as_str().parse::<i32>() {
                        episode_urls.entry(ep).or_insert(url);
                    }
                }
            }
        }

        episode_urls
    }

    /// Extract total episode count
    fn extract_total_episodes(&self, document: &Html, html: &str) -> i32 {
        // Method 1: Look for pattern in description "XX ตอน"
        let desc_selector = Selector::parse("meta[name='description']").unwrap();
        if let Some(desc_el) = document.select(&desc_selector).next() {
            if let Some(content) = desc_el.value().attr("content") {
                let re = Regex::new(r"(\d+)\s*ตอน").unwrap();
                if let Some(caps) = re.captures(content) {
                    if let Ok(count) = caps.get(1).unwrap().as_str().parse::<i32>() {
                        return count;
                    }
                }
            }
        }

        // Method 2: Count episode mentions
        let re = Regex::new(r"ตอนที่\s*(\d+)").unwrap();
        let max_ep = re
            .captures_iter(html)
            .filter_map(|caps| caps.get(1)?.as_str().parse::<i32>().ok())
            .max()
            .unwrap_or(1);

        max_ep
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_series_url() {
        // Rongyok formats
        assert_eq!(RongyokParser::parse_series_url("https://rongyok.com/watch/?series_id=1004"), Some(1004));
        assert_eq!(RongyokParser::parse_series_url("https://rongyok.com/series/1004/title"), Some(1004));

        // Thongyok format
        assert_eq!(RongyokParser::parse_series_url("https://thongyok.com/series/1004/%E0%B8%84%E0%B8%B7%E0%B8%99..."), Some(1004));

        // Invalid
        assert_eq!(RongyokParser::parse_series_url("https://rongyok.com/invalid"), None);
    }

    #[test]
    fn test_construct_series_url() {
        // Case 1: Rongyok URL (standard)
        let (url, domain) = RongyokParser::construct_series_url(1004, Some("https://rongyok.com/watch/?series_id=1004"));
        assert_eq!(url, "https://rongyok.com/watch/?series_id=1004");
        assert_eq!(domain, "https://rongyok.com/");

        // Case 2: Thongyok URL
        let th_url = "https://thongyok.com/series/1004/test-title";
        let (url, domain) = RongyokParser::construct_series_url(1004, Some(th_url));
        assert_eq!(url, th_url);
        assert_eq!(domain, "https://thongyok.com/");

        // Case 3: None (fallback)
        let (url, domain) = RongyokParser::construct_series_url(1004, None);
        assert_eq!(url, "https://rongyok.com/watch/?series_id=1004");
        assert_eq!(domain, "https://rongyok.com/");

        // Case 4: Other domain/format treated as Rongyok if not explicit Thongyok
        let (url, domain) = RongyokParser::construct_series_url(1004, Some("https://other.com/1004"));
        // Current implementation falls back to Rongyok format if not containing "thongyok.com"
        assert_eq!(url, "https://rongyok.com/watch/?series_id=1004");
        assert_eq!(domain, "https://rongyok.com/");
    }
}
