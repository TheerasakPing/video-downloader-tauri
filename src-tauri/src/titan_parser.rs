use futures_util::{stream, StreamExt};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HlsKeyInfo {
    /// Base64-encoded AES-128 key
    pub key_b64: String,
    /// IV as hex string (32 hex chars = 16 bytes)
    pub iv_hex: String,
    /// Base HLS URL up to the episode folder (e.g. https://hls.357ms.com/series_360/ep_1)
    pub hls_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TitanSeriesInfo {
    pub url: String,
    pub title: String,
    pub total_episodes: i32,
    pub poster_url: Option<String>,
    pub episode_urls: HashMap<i32, String>,
    /// AES-128-CBC key info per episode (only present when API fetch succeeded)
    pub episode_keys: HashMap<i32, HlsKeyInfo>,
}

/// Decoded API response from /api/v1/hls/config/{series_id}/{ep_num}
#[derive(Debug, Deserialize)]
struct HlsApiData {
    stream_url: String,
    #[serde(default)]
    hls_key_b64: String,
    #[serde(default)]
    hls_iv: String,
}

#[derive(Debug, Deserialize)]
struct HlsApiResponse {
    status: String,
    data: Option<HlsApiData>,
}

pub struct TitanParser {
    client: Client,
}

impl TitanParser {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Check if URL is from 357ms.com or dynamic domain
    /// `domains` can be a comma-separated list
    pub fn is_titan_url(url: &str, domains: &str) -> bool {
        if url.contains("51cg") || url.contains("357ms") {
            return true;
        }

        domains
            .split(',')
            .map(|d| d.trim())
            .filter(|d| !d.is_empty())
            .any(|d| url.contains(d))
    }

    /// XOR-decode the API response.
    /// Format: first 2 bytes = decimal key (e.g. "70"), rest = XOR-encoded payload.
    fn xor_decode(raw: &str) -> Option<String> {
        if raw.len() < 3 {
            return None;
        }
        let key_str = &raw[..2];
        let key: u8 = key_str.parse().ok()?;
        let decoded: String = raw[2..]
            .chars()
            .map(|c| char::from_u32((c as u32) ^ (key as u32)).unwrap_or(c))
            .collect();
        Some(decoded)
    }

    /// Extract the base URL (scheme + host) from a full URL.
    fn base_url(url: &str) -> String {
        // e.g. "https://www.357ms.com/series/360" -> "https://www.357ms.com"
        if let Some(pos) = url.find("://") {
            let after = &url[pos + 3..];
            if let Some(slash) = after.find('/') {
                return format!("https://{}", &after[..slash]);
            }
            return format!("https://{}", after);
        }
        url.to_string()
    }

    /// Extract series_id from URL path
    /// Supports: /series/360, /archives/252482, or falls back to last path segment
    fn extract_series_id_from_url(url: &str) -> Option<i32> {
        // Try /series/{id} pattern first
        if let Ok(re) = Regex::new(r"/series/(\d+)") {
            if let Some(caps) = re.captures(url) {
                if let Some(m) = caps.get(1) {
                    if let Ok(id) = m.as_str().parse::<i32>() {
                        return Some(id);
                    }
                }
            }
        }
        
        // Try /archives/{id} pattern (single video posts)
        if let Ok(re) = Regex::new(r"/archives/(\d+)") {
            if let Some(caps) = re.captures(url) {
                if let Some(m) = caps.get(1) {
                    if let Ok(id) = m.as_str().parse::<i32>() {
                        return Some(id);
                    }
                }
            }
        }
        
        None
    }
    
    /// Check if this is an archive page (single video post)
    pub fn is_archive_url(url: &str) -> bool {
        url.contains("/archives/")
    }

    /// Fetch series information from a Titan network URL.
    /// Supports:
    ///   - Series listing page: https://www.357ms.com/series/360
    ///   - Episode watch page:  https://www.357ms.com/watch/26037
    pub async fn get_series_info(
        &self,
        series_url: &str,
        _domain: &str,
    ) -> Result<TitanSeriesInfo, String> {
        eprintln!("[Titan] Fetching page: {}", series_url);

        let response = self
            .client
            .get(series_url)
            .header("Referer", "https://www.357ms.com/")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let final_url = response.url().to_string();
        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let base = Self::base_url(&final_url);

        // Parse synchronously, then do async work after
        let (title, poster_url_raw, ep_links, series_id_from_url) = {
            let document = Html::parse_document(&html);

            // Extract title
            let title_selector = Selector::parse("title").unwrap();
            let raw_title = document
                .select(&title_selector)
                .next()
                .map(|t| t.text().collect::<String>())
                .unwrap_or_else(|| "Unknown Video".to_string())
                .trim()
                .to_owned();

            // Clean site suffix from title  
            let title = raw_title
                .split(" | ")
                .next()
                .unwrap_or(&raw_title)
                .trim()
                .to_string();

            // ── Poster ──────────────────────────────────────────────
            let mut poster_url_raw = None;

            let og_image_selector = Selector::parse("meta[property='og:image']").unwrap();
            if let Some(el) = document.select(&og_image_selector).next() {
                poster_url_raw = el.value().attr("content").map(|s| s.to_string());
            }

            // ── Episode Links ────────────────────────────────────────
            // Matches: <a class="ep-card" href="/watch/26037">
            // Also matches the canonical "start watching" link
            let ep_selector = Selector::parse("a[href*='/watch/']").unwrap();
            let mut seen = std::collections::HashSet::new();
            let mut ep_links: Vec<(i32, String)> = Vec::new();

            // Regex to extract EP number from link text or data
            let ep_num_re = Regex::new(r"EP[.\s]*(\d+)").unwrap();

            for el in document.select(&ep_selector) {
                let href = match el.value().attr("href") {
                    Some(h) => h.to_string(),
                    None => continue,
                };

                // Deduplicate
                if seen.contains(&href) {
                    continue;
                }
                seen.insert(href.clone());

                // Full URL
                let full_url = if href.starts_with("http") {
                    href.clone()
                } else {
                    format!("{}{}", base, href)
                };

                // Try to extract ep number from link text
                let text = el.text().collect::<String>();
                let ep_num = ep_num_re
                    .captures(&text.to_uppercase())
                    .and_then(|c| c.get(1))
                    .and_then(|m| m.as_str().parse::<i32>().ok());

                if let Some(n) = ep_num {
                    ep_links.push((n, full_url));
                }
            }

            // Sort by episode number
            ep_links.sort_by_key(|(n, _)| *n);

            eprintln!("[Titan] Found {} episode links", ep_links.len());
            let series_id_from_url = Self::extract_series_id_from_url(&final_url);
            eprintln!("[Titan] Series ID from URL: {:?}", series_id_from_url);

            (title, poster_url_raw, ep_links, series_id_from_url)
        }; // document dropped here

        // ── Initialize episode maps ──────────────────────────────────
        let mut episode_urls: HashMap<i32, String> = HashMap::new();
        let mut episode_keys: HashMap<i32, HlsKeyInfo> = HashMap::new();

        // ── Handle archive pages (single video posts) differently ─────
        // Archive pages embed videos directly in HTML (dplayer data-config JSON or static m3u8)
        if Self::is_archive_url(&final_url) {
            eprintln!("[Titan] Archive page detected - extracting embedded video URLs");
            
            let mut found_urls: Vec<String> = Vec::new();
            let mut seen_urls = std::collections::HashSet::new();

            // ── Strategy 1: JSON "url" field inside data-config (dplayer) ───────────────
            // Handles: data-config='{"video":{"url":"https:\/\/hls.example.com\/...m3u8?auth_key=..."}}'
            // The URL field uses escaped slashes \/ which curl/html delivers as \/
            let re_json_url = Regex::new(r#""url"\s*:\s*"(https?:[^"]*?\.m3u8[^"]*?)""#).unwrap();
            for cap in re_json_url.captures_iter(&html) {
                if let Some(m) = cap.get(1) {
                    let url = m.as_str().replace("\\/", "/");
                    if seen_urls.insert(url.clone()) {
                        eprintln!("[Titan] dplayer JSON URL: {}", &url[..url.len().min(80)]);
                        found_urls.push(url);
                    }
                }
            }

            // ── Strategy 2: Classic quoted m3u8 URL (fallback for other embed styles) ──
            if found_urls.is_empty() {
                let re_quoted = Regex::new(r#"["'](https?://[^"']*?\.m3u8[^"']*?)["']"#).unwrap();
                for cap in re_quoted.captures_iter(&html) {
                    if let Some(m) = cap.get(1) {
                        let url = m.as_str().replace("\\/", "/");
                        if seen_urls.insert(url.clone()) {
                            found_urls.push(url);
                        }
                    }
                }
            }

            // ── Strategy 3: Broad m3u8 anywhere in the page ──────────────────────────
            if found_urls.is_empty() {
                let re_broad = Regex::new(r#"(https?:[^\s"'<>]*?\.m3u8[^\s"'<>]*)"#).unwrap();
                for cap in re_broad.captures_iter(&html) {
                    if let Some(m) = cap.get(1) {
                        let url = m.as_str().replace("\\/", "/");
                        if seen_urls.insert(url.clone()) {
                            found_urls.push(url);
                        }
                    }
                }
            }

            if !found_urls.is_empty() {
                eprintln!("[Titan] Archive: found {} video URL(s)", found_urls.len());
                for (idx, video_url) in found_urls.iter().enumerate() {
                    episode_urls.insert(idx as i32 + 1, video_url.clone());
                }
            } else {
                eprintln!("[Titan] Archive page has no static m3u8, marking for Playwright detection");
            }
        } else if !ep_links.is_empty() {
            // ── Fetch HLS stream URLs via API ────────────────────────────
            // Use concurrent fetch (5 at a time) to speed up large series.
            const CONCURRENT_FETCHES: usize = 5;
            
            if let Some(series_id) = series_id_from_url {
                eprintln!(
                    "[Titan] Fetching {} episode URLs concurrently (batch={})",
                    ep_links.len(),
                    CONCURRENT_FETCHES
                );
                let self_arc = Arc::new(self.client.clone());
                let base_arc = Arc::new(base.clone());

                let results: Vec<(i32, Option<HlsApiData>)> = stream::iter(ep_links.iter().cloned())
                    .map(|(ep_num, ep_watch_url)| {
                        let client = self_arc.clone();
                        let b = base_arc.clone();
                        async move {
                            let api_url = format!(
                                "{}/api/v1/hls/config/{}/{}",
                                b, series_id, ep_num
                            );
                            let result: Option<HlsApiData> = async {
                                let raw = client
                                    .get(&api_url)
                                    .header("Referer", &ep_watch_url)
                                    .send()
                                    .await
                                    .ok()?
                                    .text()
                                    .await
                                    .ok()?;
                                let decoded = Self::xor_decode(&raw)?;
                                eprintln!("[Titan] EP {} API decoded: {}", ep_num, &decoded[..decoded.len().min(120)]);
                                let parsed: HlsApiResponse =
                                    serde_json::from_str(&decoded).ok()?;
                                if parsed.status != "success" {
                                    return None;
                                }
                                parsed.data
                            }
                            .await;
                            (ep_num, result)
                        }
                    })
                    .buffer_unordered(CONCURRENT_FETCHES)
                    .collect()
                    .await;

                for (ep_num, api_data_opt) in results {
                    match api_data_opt {
                        Some(api_data) => {
                            eprintln!("[Titan] EP {} -> {}", ep_num, api_data.stream_url);
                            // Store encryption key if present
                            if !api_data.hls_key_b64.is_empty() && !api_data.hls_iv.is_empty() {
                                // hls_base_url = stream_url minus the filename
                                let hls_base_url = api_data.stream_url
                                    .rsplit_once('/')
                                    .map(|(base, _)| base.to_string())
                                    .unwrap_or_else(|| api_data.stream_url.clone());
                                episode_keys.insert(ep_num, HlsKeyInfo {
                                    key_b64: api_data.hls_key_b64.clone(),
                                    iv_hex: api_data.hls_iv.clone(),
                                    hls_base_url,
                                });
                            }
                            episode_urls.insert(ep_num, api_data.stream_url);
                        }
                        None => {
                            eprintln!("[Titan] EP {} -> API failed, using pattern fallback", ep_num);
                            let fallback = format!(
                                "https://hls.357ms.com/series_{}/ep_{}/master.m3u8",
                                series_id, ep_num
                            );
                            episode_urls.insert(ep_num, fallback);
                        }
                    }
                }
            } else {
                // No series_id in URL — regex fallback
                eprintln!("[Titan] No series_id found, falling back to regex...");
                let re = Regex::new(r#"["'](https?://[^"']*?\.m3u8[^"']*?)["']"#).unwrap();
                let mut ep_count = 1;
                for cap in re.captures_iter(&html) {
                    if let Some(m) = cap.get(1) {
                        let url = m.as_str().replace("\\/", "/");
                        episode_urls.insert(ep_count, url);
                        ep_count += 1;
                    }
                }
            }
        } else {
            // No ep-card links (direct watch page) — regex fallback
            eprintln!("[Titan] No ep-card links - trying regex m3u8 extraction...");
            let re = Regex::new(r#"["'](https?://[^"']*?\.m3u8[^"']*?)["']"#).unwrap();
            let mut ep_count = 1;
            for cap in re.captures_iter(&html) {
                if let Some(m) = cap.get(1) {
                    let url = m.as_str().replace("\\/", "/");
                    eprintln!("[Titan] Regex URL (Ep {}): {}", ep_count, url);
                    episode_urls.insert(ep_count, url);
                    ep_count += 1;
                }
            }
        }


        let total_episodes = episode_urls.len() as i32;
        eprintln!("[Titan] Total episodes with URLs: {}", total_episodes);

        // ── Fetch poster as base64 data URL ──────────────────────────
        let poster_data_url = if let Some(ref url) = poster_url_raw {
            eprintln!("[Titan] Fetching poster: {}", url);
            self.fetch_image_as_data_url(url).await
        } else {
            None
        };

        eprintln!("[Titan] Episodes with encryption keys: {}", episode_keys.len());

        Ok(TitanSeriesInfo {
            url: series_url.to_string(),
            title,
            total_episodes,
            poster_url: poster_data_url,
            episode_urls,
            episode_keys,
        })
    }

    /// Fetch an image URL and return it as a base64 data URL.
    async fn fetch_image_as_data_url(&self, image_url: &str) -> Option<String> {
        let final_url = if image_url.starts_with("//") {
            format!("https:{}", image_url)
        } else {
            image_url.to_string()
        };

        let response = self
            .client
            .get(&final_url)
            .header("Accept", "image/*")
            .send()
            .await
            .ok()?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("image/jpeg")
            .to_string();

        let bytes = response.bytes().await.ok()?;

        use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
        let base64_data = BASE64.encode(&bytes);

        Some(format!("data:{};base64,{}", content_type, base64_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_titan_url() {
        let domains = "51cg1.com, 51cm.com, titan51.net";

        assert!(TitanParser::is_titan_url("https://51cm.com/video/123", domains));
        assert!(TitanParser::is_titan_url("https://titan51.net/video/123", domains));
        assert!(TitanParser::is_titan_url("https://51cg.com/video/123", domains));
        assert!(TitanParser::is_titan_url("https://357ms.com/series/360", domains));
        assert!(TitanParser::is_titan_url("https://www.357ms.com/series/360", domains));

        assert!(!TitanParser::is_titan_url("https://youtube.com/video/123", domains));
    }

    #[test]
    fn test_xor_decode() {
        // key=70, encoded char '^' (ord=94), 94 XOR 70 = 24... just test round-trip logic
        let key: u8 = 70;
        let test = "hello";
        let encoded: String = std::iter::once(format!("{:02}", key))
            .chain(test.chars().map(|c| {
                char::from_u32((c as u32) ^ (key as u32))
                    .unwrap_or(c)
                    .to_string()
            }))
            .collect();
        let decoded = TitanParser::xor_decode(&encoded).unwrap();
        assert_eq!(decoded, test);
    }

    #[test]
    fn test_base_url_extraction() {
        assert_eq!(
            TitanParser::base_url("https://www.357ms.com/series/360"),
            "https://www.357ms.com"
        );
        assert_eq!(
            TitanParser::base_url("https://51cg1.com/video/123"),
            "https://51cg1.com"
        );
    }

    #[test]
    fn test_series_id_extraction() {
        assert_eq!(
            TitanParser::extract_series_id_from_url("https://www.357ms.com/series/360"),
            Some(360)
        );
        assert_eq!(
            TitanParser::extract_series_id_from_url("https://www.357ms.com/watch/26037"),
            None
        );
    }
}
