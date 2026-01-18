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
    pub async fn get_series_info(&self, series_id: i32) -> Result<SeriesInfo, String> {
        let url = format!("https://rongyok.com/watch/?series_id={}", series_id);

        let response = self
            .client
            .get(&url)
            .header("Accept", "text/html,application/xhtml+xml")
            .header("Accept-Language", "th,en-US;q=0.9,en;q=0.8")
            .header("Referer", "https://rongyok.com/")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let document = Html::parse_document(&html);

        // Get title
        let title_selector = Selector::parse("title").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_else(|| format!("Series {}", series_id));

        // Clean title - remove " - ตอนที่ X" suffix
        let title_re = Regex::new(r"\s*-\s*ตอนที่\s*\d+.*$").unwrap();
        let title = title_re.replace(&title, "").to_string();

        // Get poster URL
        let og_image_selector = Selector::parse("meta[property='og:image']").unwrap();
        let poster_url = document
            .select(&og_image_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_string());

        // Extract episode URLs
        let episode_urls = self.extract_all_episode_urls(&html);
        let total_episodes = if episode_urls.is_empty() {
            self.extract_total_episodes(&document, &html)
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
