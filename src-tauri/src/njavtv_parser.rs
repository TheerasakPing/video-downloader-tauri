use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::proxy;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NjavtvSeriesInfo {
    pub url: String,
    pub title: String,
    pub total_episodes: i32,
    pub poster_url: Option<String>,
    /// episode_number → page URL (not video URL yet — requires Chrome detection)
    pub episode_page_urls: HashMap<i32, String>,
    /// If single-episode page, contains the page URL for Chrome detection
    pub direct_page_url: Option<String>,
}

#[allow(dead_code)]
pub struct NjavtvParser {
    #[allow(dead_code)]
    proxy_config: Arc<RwLock<proxy::ProxyConfig>>,
}

#[allow(dead_code)]
impl NjavtvParser {
    pub fn new(proxy_config: Arc<RwLock<proxy::ProxyConfig>>) -> Self {
        Self { proxy_config }
    }

    fn client(&self) -> Client {
        proxy::build_client(&self.proxy_config.read().unwrap())
    }

    /// Check if URL belongs to njavtv.com or configured njavtv domain
    pub fn is_njavtv_url(url: &str, domain: &str) -> bool {
        url.contains(domain) || url.contains("njavtv.com")
    }

    /// Extract series info from njavtv.com page
    /// 
    /// njavtv.com structure:
    ///   /dm18/th/cus-376  → single video page (Cloudflare protected)
    ///   /dm18/th/          → list page
    ///
    /// Since Cloudflare blocks simple HTTP, we return the URL itself for
    /// Chrome-based detection. The title is extracted from the URL if HTML
    /// is unavailable.
    pub async fn get_series_info(
        &self,
        series_url: &str,
        _domain: &str,
    ) -> Result<NjavtvSeriesInfo, String> {
        eprintln!("[NjavTV] get_series_info called for: {}", series_url);

        // SKIP HTTP fetch — NjavTV is always Cloudflare protected
        // Go directly to Chrome detection to avoid 30s timeout hang
        eprintln!("[NjavTV] Skipping HTTP fetch (always Cloudflare), using Chrome detection");
        
        let title = self.extract_title_from_url(series_url);
        Ok(NjavtvSeriesInfo {
            url: series_url.to_string(),
            title,
            total_episodes: 1,
            poster_url: None,
            episode_page_urls: {
                let mut m = HashMap::new();
                m.insert(1, series_url.to_string());
                m
            },
            direct_page_url: Some(series_url.to_string()),
        })
    }

    /// Try to fetch page HTML (may fail with Cloudflare)
    async fn try_fetch(&self, url: &str) -> Result<String, String> {
        let response = self
            .client()
            .get(url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "th,en-US;q=0.9,en;q=0.8")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        eprintln!("[NjavTV] HTML length: {} bytes", html.len());
        Ok(html)
    }

    /// Parse HTML when Cloudflare is bypassed
    fn parse_html(&self, base_url: &str, html: &str) -> NjavtvSeriesInfo {
        let document = Html::parse_document(html);

        // Extract title
        let title = self.extract_title_from_html(&document, html)
            .unwrap_or_else(|| self.extract_title_from_url(base_url));

        // Extract poster
        let poster_url = self.extract_poster(&document);

        // Try to find episode list links
        let episode_page_urls = self.extract_episode_links(&document, base_url);

        if episode_page_urls.is_empty() {
            // Single episode page
            NjavtvSeriesInfo {
                url: base_url.to_string(),
                title,
                total_episodes: 1,
                poster_url,
                episode_page_urls: {
                    let mut m = HashMap::new();
                    m.insert(1, base_url.to_string());
                    m
                },
                direct_page_url: Some(base_url.to_string()),
            }
        } else {
            let total = episode_page_urls.len() as i32;
            NjavtvSeriesInfo {
                url: base_url.to_string(),
                title,
                total_episodes: total,
                poster_url,
                episode_page_urls,
                direct_page_url: None,
            }
        }
    }

    /// Extract title from HTML
    fn extract_title_from_html(&self, document: &Html, html: &str) -> Option<String> {
        // Try <title> tag
        if let Ok(sel) = Selector::parse("title") {
            if let Some(el) = document.select(&sel).next() {
                let text = el.text().collect::<String>();
                let clean = text
                    .split(" - ").next()
                    .unwrap_or(&text)
                    .trim()
                    .to_string();
                if !clean.is_empty() && !clean.contains("njavtv") {
                    return Some(clean);
                }
            }
        }

        // Try og:title
        if let Ok(sel) = Selector::parse("meta[property='og:title']") {
            if let Some(el) = document.select(&sel).next() {
                if let Some(content) = el.value().attr("content") {
                    let clean = content.trim().to_string();
                    if !clean.is_empty() {
                        return Some(clean);
                    }
                }
            }
        }

        // Try h1
        if let Ok(sel) = Selector::parse("h1") {
            if let Some(el) = document.select(&sel).next() {
                let text = el.text().collect::<String>().trim().to_string();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }

        // Regex fallback for title in JS/JSON
        let re = Regex::new(r#""title"\s*:\s*"([^"]+)""#).ok()?;
        re.captures(html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Derive a readable title from the URL slug
    fn extract_title_from_url(&self, url: &str) -> String {
        url.split('/')
            .filter(|s| !s.is_empty())
            .last()
            .map(|slug| {
                slug.split('-')
                    .map(|w| {
                        let mut chars = w.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_else(|| "NjavTV Video".to_string())
    }

    /// Extract poster/thumbnail URL
    fn extract_poster(&self, document: &Html) -> Option<String> {
        // og:image
        if let Ok(sel) = Selector::parse("meta[property='og:image']") {
            if let Some(el) = document.select(&sel).next() {
                if let Some(content) = el.value().attr("content") {
                    return Some(content.to_string());
                }
            }
        }
        None
    }

    /// Extract episode links (for series list pages)
    fn extract_episode_links(&self, document: &Html, base_url: &str) -> HashMap<i32, String> {
        let mut episodes = HashMap::new();

        // Common selectors for episode links
        let selectors = [
            "a.episode-link",
            ".episode-list a",
            ".episodes a",
            "a[href*='/th/']",
            ".ep-list a",
        ];

        for sel_str in &selectors {
            if let Ok(sel) = Selector::parse(sel_str) {
                let links: Vec<_> = document.select(&sel).collect();
                if !links.is_empty() {
                    // Parse episode number from URL or text
                    for (i, el) in links.iter().enumerate() {
                        let href = el.value().attr("href")
                            .map(|h| {
                                if h.starts_with("http") {
                                    h.to_string()
                                } else {
                                    // Construct absolute URL
                                    let base = base_url.split('/').take(3).collect::<Vec<_>>().join("/");
                                    format!("{}{}", base, h)
                                }
                            });

                        if let Some(url) = href {
                            let ep_num = self.extract_ep_number_from_url(&url)
                                .unwrap_or((i + 1) as i32);
                            episodes.insert(ep_num, url);
                        }
                    }
                    break;
                }
            }
        }

        episodes
    }

    /// Extract episode number from URL pattern like /ep-5 or /episode-5
    fn extract_ep_number_from_url(&self, url: &str) -> Option<i32> {
        let re = Regex::new(r"(?:ep|episode|e)[-_]?(\d+)").ok()?;
        re.captures(url)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }
}
