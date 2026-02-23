use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use regex::Regex;
use scraper::{Html, Selector};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ms357ApiConfig {
    pub status: String,
    pub data: Ms357ApiData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ms357ApiData {
    pub id: i32,
    pub stream_url: String,
    pub sub_url: Option<String>,
    pub sub_xor_key: u8,
    pub hls_key_b64: String,
    pub hls_iv: String,
    pub cover_url: Option<String>,
}

pub struct Ms357Parser {
    client: Client,
}

impl Ms357Parser {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub fn is_ms357_url(url: &str) -> bool {
        url.contains("357ms.com") || url.contains("357ms") 
    }

    pub fn extract_series_id(url: &str) -> Option<i32> {
        let re1 = Regex::new(r"series[/_]id=(\d+)").ok()?;
        if let Some(caps) = re1.captures(url) {
            return caps.get(1)?.as_str().parse().ok();
        }

        let re2 = Regex::new(r"/series/(\d+)").ok()?;
        if let Some(caps) = re2.captures(url) {
            return caps.get(1)?.as_str().parse().ok();
        }

        None
    }

    pub async fn fetch_api_config(&self, series_id: i32, ep_number: i32) -> Result<Ms357ApiData, String> {
        let url = format!("https://www.357ms.com/api/v1/hls/config/{}/{}", series_id, ep_number);
        eprintln!("[Ms357] Fetching API config: {}", url);

        let response = self.client
            .get(&url)
            .header("Referer", "https://www.357ms.com/")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API returned error status: {}", response.status()));
        }

        let encrypted_text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
        if encrypted_text.len() < 2 {
            return Err("Invalid API response format".to_string());
        }

        // Decrypt: First two characters are the XOR key as an integer string (e.g., "55")
        // Wait, Node.js code did: parseInt(data.substring(0, 2), 10)
        let key_str = &encrypted_text[0..2];
        let xor_key = key_str.parse::<u8>().map_err(|_| "Invalid XOR key".to_string())?;

        let ciphertext = &encrypted_text[2..];
        let mut decrypted = String::with_capacity(ciphertext.len());
        for c in ciphertext.chars() {
            let dc = (c as u32 ^ xor_key as u32) as u8 as char;
            decrypted.push(dc);
        }

        let config: Ms357ApiConfig = serde_json::from_str(&decrypted)
            .map_err(|e| format!("Failed to parse decrypted config: {}", e))?;

        if config.status != "success" {
            return Err(format!("API returned non-success status: {}", config.status));
        }

        Ok(config.data)
    }

    pub async fn get_series_info(&self, url: &str) -> Result<crate::UnifiedSeriesInfo, String> {
        let series_id = Self::extract_series_id(url).ok_or("Could not extract series ID")?;
        eprintln!("[Ms357] Detected series ID: {}", series_id);

        // Fetch episode 1 to get config
        let ep1_config = self.fetch_api_config(series_id, 1).await?;

        // Try to fetch series webpage for title and total episodes
        let mut title = format!("Series {}", series_id);
        let mut total_episodes = 1;
        let mut parsed_cover_url: Option<String> = None;
        
        let mut episode_urls = HashMap::new();
        // Pack series_id + config into JSON format for downloader to use
        // We will mock `episode_urls` with JSON payloads
        
        // We can find total episodes by parsing the HTML or assuming an arbitrary max and stopping when API 404s
        // Let's scrape the main page to get total episodes
        if let Ok(res) = self.client.get(url).send().await {
            if let Ok(html) = res.text().await {
                let document = Html::parse_document(&html);
                if let Ok(sel) = Selector::parse("title") {
                    if let Some(el) = document.select(&sel).next() {
                        title = el.text().collect::<String>().replace(" - 357ms", "").trim().to_string();
                    }
                }
                
                if let Ok(sel) = Selector::parse("meta[property=\"og:image\"]") {
                    if let Some(el) = document.select(&sel).next() {
                        if let Some(content) = el.value().attr("content") {
                            parsed_cover_url = Some(content.to_string());
                        }
                    }
                }
                
                // Extract total episodes using multiple robust regexes
            let mut max_ep = 1;
            
            // Regex 1: <span class="ep-number">EP.30</span>
            let ep_re1 = Regex::new(r#"<span[^>]*class=["'][^"']*ep-number[^"']*["'][^>]*>EP\.?(\d+)</span>"#).unwrap();
            for cap in ep_re1.captures_iter(&html) {
                if let Some(m) = cap.get(1) {
                    if let Ok(ep) = m.as_str().parse::<i32>() {
                        max_ep = max_ep.max(ep);
                    }
                }
            }

            // Regex 2: >30 ตอน< (Thai string for X episodes)
            let ep_re2 = Regex::new(r#">(\d+)\s*ตอน<"#).unwrap();
            for cap in ep_re2.captures_iter(&html) {
                if let Some(m) = cap.get(1) {
                    if let Ok(ep) = m.as_str().parse::<i32>() {
                        max_ep = max_ep.max(ep);
                    }
                }
            }

            // Fallback Regex 3: original attempts just in case
            let ep_re3 = Regex::new(r"episodes?\b[^>]*>(\d+)").unwrap();
            for cap in ep_re3.captures_iter(&html) {
                if let Some(m) = cap.get(1) {
                    if let Ok(ep) = m.as_str().parse::<i32>() {
                        max_ep = max_ep.max(ep);
                    }
                }
            }
                
                // For 357ms, usually episodes are in the DOM, let's just populate 1 to max_ep
                // If we can't find it, default to 1 (or prompt user).
                // Usually we can just populate dummy URLs because the downloader will fetch the config dynamically.
                total_episodes = max_ep.max(1);
            }
        }

        // Just populate MS357 pseudo-urls
        // We'll construct a special JSON string or URI: ms357://{series_id}/{ep_number}
        for ep in 1..=total_episodes {
            episode_urls.insert(ep, format!("ms357://{}/{}", series_id, ep));
        }
        
        // Convert cover URL to base64 if possible
        let mut poster_data_url = None;
        
        // Try to use parsed cover URL from HTML first, fallback to config cover URL
        let cover_url_to_fetch = parsed_cover_url.or_else(|| ep1_config.cover_url.clone());
        
        if let Some(cover) = cover_url_to_fetch {
             let cover_url = if cover.starts_with("//") {
                 format!("https:{}", cover)
             } else {
                 cover
             };
             
             if let Ok(res) = self.client.get(&cover_url).send().await {
                 let content_type = res.headers()
                     .get("content-type")
                     .and_then(|v| v.to_str().ok())
                     .unwrap_or("image/jpeg")
                     .to_string();
                     
                 if let Ok(bytes) = res.bytes().await {
                     let b64 = BASE64.encode(&bytes);
                     poster_data_url = Some(format!("data:{};base64,{}", content_type, b64));
                 }
             }
        }

        Ok(crate::UnifiedSeriesInfo {
            series_id,
            title,
            total_episodes,
            poster_url: poster_data_url,
            episode_urls,
            source: "ms357".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_api() {
        let parser = Ms357Parser::new();
        let res = parser.fetch_api_config(221, 1).await;
        println!("{:?}", res);
    }
}
