use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitanSeriesInfo {
    pub url: String,
    pub title: String,
    pub total_episodes: i32,
    pub poster_url: Option<String>,
    pub episode_urls: HashMap<i32, String>,
}

pub struct TitanParser {
    client: Client,
}

impl TitanParser {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Check if URL is from 51cg1.com
    pub fn is_titan_url(url: &str) -> bool {
        url.contains("51cg1.com") || url.contains("51cg")
    }

    /// Fetch series information from 51cg1.com
    pub async fn get_series_info(&self, series_url: &str) -> Result<TitanSeriesInfo, String> {
        eprintln!("[Titan] Fetching page: {}", series_url);

        let response = self
            .client
            .get(series_url)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        // Extract data in a synchronous block to ensure Html is dropped before await
        let (title, episode_urls, poster_url_string) = {
            let document = Html::parse_document(&html);
            
            // Extract title
            let title_selector = Selector::parse("title").unwrap();
            let title = document
                .select(&title_selector)
                .next()
                .map(|t| t.text().collect::<String>())
                .unwrap_or_else(|| "Unknown Video".to_string())
                .trim()
                .to_string();

            let mut episode_urls = HashMap::new();
            let mut ep_count = 1;

            // Method 1: DPlayer data-config
            let dplayer_selector = Selector::parse("div.dplayer").unwrap();
            for element in document.select(&dplayer_selector) {
                if let Some(config_str) = element.value().attr("data-config") {
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(config_str) {
                         if let Some(url_val) = json_val.get("video").and_then(|v| v.get("url")) {
                            if let Some(url_str) = url_val.as_str() {
                                let clean_url = url_str.replace("\\/", "/");
                                eprintln!("[Titan] DPlayer URL found (Ep {}): {}", ep_count, clean_url);
                                episode_urls.insert(ep_count, clean_url);
                                ep_count += 1;
                            }
                        }
                    }
                }
            }

            // Method 2: Regex Fallback
            if episode_urls.is_empty() {
                 eprintln!("[Titan] DPlayer method failed/empty, trying regex...");
                 let re = Regex::new(r#"["'](https?://[^"']*?\.m3u8[^"']*?)["']"#).unwrap();
                 for cap in re.captures_iter(&html) {
                     if let Some(m) = cap.get(1) {
                         let url = m.as_str().replace("\\/", "/");
                         eprintln!("[Titan] Regex URL found (Ep {}): {}", ep_count, url);
                         episode_urls.insert(ep_count, url);
                         ep_count += 1;
                     }
                 }
            }
            
            if episode_urls.is_empty() {
                eprintln!("[Titan] No video found.");
            }

            // Poster extraction
            eprintln!("[Titan] Extracting poster...");
            let mut poster_url = None;

            // Try to get poster from DPlayer config
            for element in document.select(&dplayer_selector) {
                if let Some(config_str) = element.value().attr("data-config") {
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(config_str) {
                        if let Some(pic_val) = json_val.get("video").and_then(|v| v.get("pic")) {
                            if let Some(pic_str) = pic_val.as_str() {
                                let clean_pic = pic_str.replace("\\/", "/");
                                eprintln!("[Titan] Found poster in DPlayer: {}", clean_pic);
                                poster_url = Some(clean_pic);
                                break;
                            }
                        }
                    }
                }
            }

            // Fallback to og:image
            if poster_url.is_none() {
                let og_image_selector = Selector::parse("meta[property='og:image']").unwrap();
                if let Some(og_image) = document.select(&og_image_selector).next() {
                    if let Some(content) = og_image.value().attr("content") {
                        eprintln!("[Titan] Found poster in og:image: {}", content);
                        poster_url = Some(content.to_string());
                    }
                }
            }
            
            (title, episode_urls, poster_url)
        }; // document is dropped here

        let total_episodes = episode_urls.len() as i32;
        eprintln!("[Titan] Series info ready. Episodes: {}", total_episodes);

        // Convert poster to data URL if available (Async call is now safe)
        let poster_data_url = if let Some(ref url) = poster_url_string {
            eprintln!("[Titan] Fetching poster image: {}", url);
            match self.fetch_image_as_data_url(url).await {
                Some(data) => {
                    eprintln!("[Titan] Poster fetched successfully (len: {})", data.len());
                    Some(data)
                }
                None => {
                    eprintln!("[Titan] Failed to fetch poster (timeout or error)");
                    None
                }
            }
        } else {
            eprintln!("[Titan] No poster URL found");
            None
        };

        Ok(TitanSeriesInfo {
            url: series_url.to_string(),
            title,
            total_episodes,
            poster_url: poster_data_url, 
            episode_urls,
        })
    }

    /// Fetch an image and convert it to a base64 data URL
    async fn fetch_image_as_data_url(&self, image_url: &str) -> Option<String> {
        eprintln!("[Titan] fetch_image_as_data_url start: {}", image_url);
        
        // Handle relative URLs
        let final_url = if image_url.starts_with("//") {
            format!("https:{}", image_url)
        } else {
            image_url.to_string()
        };

        eprintln!("[Titan] Sending image request...");
        let response = match self.client
            .get(&final_url)
            .header("Accept", "image/*")
            .send()
            .await {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("[Titan] Image request failed: {}", e);
                    return None;
                }
            };
            
        eprintln!("[Titan] Reading image bytes...");
        // Get content type
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();

        // Get image bytes
        let bytes = match response.bytes().await {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[Titan] Failed to read image bytes: {}", e);
                return None;
            }
        };

        eprintln!("[Titan] Encoding base64...");
        // Convert to base64
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
        let base64_data = BASE64.encode(&bytes);

        eprintln!("[Titan] Image processed.");
        // Return as data URL
        Some(format!("data:{};base64,{}", content_type, base64_data))
    }
}
