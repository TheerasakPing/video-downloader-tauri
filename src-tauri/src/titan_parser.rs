use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

        // Extract title
        let document = Html::parse_document(&html);
        let title_selector = Selector::parse("title").unwrap();
        let title = document
            .select(&title_selector)
            .next()
            .map(|t| t.text().collect::<String>())
            .unwrap_or_else(|| "Unknown Video".to_string())
            .trim()
            .to_string();

        let mut episode_urls = HashMap::new();

        // Method 1: DPlayer data-config
        // <div class="dplayer" data-config='{...}'>
        let dplayer_selector = Selector::parse("div.dplayer").unwrap();
        let mut ep_count = 1;
        
        for element in document.select(&dplayer_selector) {
            if let Some(config_str) = element.value().attr("data-config") {
                // Determine if this is a repeat or new logic needed? 
                // Usually 51cg1 puts multiple DPlayer divs for multiple Parts.
                
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

        // Method 2: Regex Fallback (Only if no DPlayer found or separate method?)
        // If DPlayer found something, we assume it covers it. If not, try regex.
        if episode_urls.is_empty() {
             eprintln!("[Titan] DPlayer method failed/empty, trying regex...");
             let re = Regex::new(r#"["'](https?://[^"']*?\.m3u8[^"']*?)["']"#).unwrap();
             for cap in re.captures_iter(&html) {
                 if let Some(m) = cap.get(1) {
                     let url = m.as_str().replace("\\/", "/");
                     // Avoid duplicates if regex catches same thing? 
                     // For now simple add.
                     eprintln!("[Titan] Regex URL found (Ep {}): {}", ep_count, url);
                     episode_urls.insert(ep_count, url);
                     ep_count += 1;
                 }
             }
        }

        // Method 3: Iframe check (omitted for now)

        if episode_urls.is_empty() {
            eprintln!("[Titan] No video found.");
        }

        let total_episodes = episode_urls.len() as i32;

        Ok(TitanSeriesInfo {
            url: series_url.to_string(),
            title,
            total_episodes,
            poster_url: None, // Can be improved later
            episode_urls,
        })
    }
}
