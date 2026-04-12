/// Parser สำหรับ javwow.com
///
/// โครงสร้างของเว็บ:
/// - หน้าวิดีโอ: `https://javwow.com/START-383/` (slug-based)
/// - มี Cloudflare protection → ต้องใช้ Chrome detector
/// - Metadata อยู่ใน og:tags (og:title, og:image, og:url)
/// - Video ฝังผ่าน iframe → `onlysubthai.com/v/{videoId}?sid=5917&t=hls`
/// - Stream จริงเป็น HLS (.m3u8) ที่โหลดจากใน iframe
///
/// วิธีการ:
/// 1. ลอง HTTP fetch ก่อน (อาจผ่าน Cloudflare ได้)
/// 2. ถ้า fail → ใช้ Chrome detector ที่ lib.rs (เหมือน NjavTV pattern)
/// 3. Extract metadata จาก og:tags
/// 4. Video URL ต้อง detect ผ่าน Chrome (intercept .m3u8 requests)

use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::sync::{Arc, RwLock};
use crate::proxy;
use crate::{SearchResult, SiteCategory};

/// Series info extracted from javwow.com page
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct JavwowSeriesInfo {
    pub url: String,
    pub title: String,
    pub total_episodes: i32,
    pub poster_url: Option<String>,
    /// If HTML was fetched, contains the embed iframe URL (onlysubthai.com)
    pub embed_url: Option<String>,
}

pub struct JavwowParser {
    proxy_config: Arc<RwLock<proxy::ProxyConfig>>,
}

impl JavwowParser {
    pub fn new(proxy_config: Arc<RwLock<proxy::ProxyConfig>>) -> Self {
        Self { proxy_config }
    }

    fn client(&self) -> Client {
        proxy::build_client(&self.proxy_config.read().unwrap())
    }

    /// Check if URL belongs to javwow.com
    pub fn is_javwow_url(url: &str, domain: &str) -> bool {
        url.contains(domain) || url.contains("javwow.com")
    }

    /// Extract slug from URL (e.g., "START-383" from "/start-383/")
    fn extract_slug(url: &str) -> Option<String> {
        let re = Regex::new(r"javwow\.com/([a-zA-Z0-9_-]+)/?").ok()?;
        re.captures(url)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Try to fetch page HTML (may be blocked by Cloudflare)
    async fn try_fetch_html(&self, url: &str) -> Result<String, String> {
        let response = self
            .client()
            .get(url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "th,en-US;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status();
        if status.as_u16() == 403 {
            return Err("Cloudflare blocked (403)".to_string());
        }

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        // Check if Cloudflare challenge page
        if html.contains("cf-browser-verification") || html.contains("challenge-platform") {
            return Err("Cloudflare challenge page detected".to_string());
        }

        Ok(html)
    }

    /// Extract series info from HTML
    fn parse_html(&self, url: &str, html: &str) -> Result<JavwowSeriesInfo, String> {
        let document = Html::parse_document(html);

        // Extract title from og:title or <title>
        let title = self.extract_title(&document)
            .unwrap_or_else(|| {
                Self::extract_slug(url)
                    .map(|s| s.to_uppercase())
                    .unwrap_or_else(|| "JavWow Video".to_string())
            });

        // Extract poster from og:image
        let poster_url = self.extract_poster(&document);

        // Extract embed iframe URL (onlysubthai.com)
        let embed_url = self.extract_embed_url(&document, html);

        Ok(JavwowSeriesInfo {
            url: url.to_string(),
            title,
            total_episodes: 1, // javwow pages are single video
            poster_url,
            embed_url,
        })
    }

    /// Extract title from page
    fn extract_title(&self, document: &Html) -> Option<String> {
        // Try og:title first (most reliable)
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

        // Try <title> tag — format: "TITLE - JavWow" or similar
        if let Ok(sel) = Selector::parse("title") {
            if let Some(el) = document.select(&sel).next() {
                let text = el.text().collect::<String>();
                let clean = text
                    .split(" - ").next()
                    .unwrap_or(&text)
                    .trim()
                    .to_string();
                if !clean.is_empty() && !clean.contains("JavWow") && !clean.contains("javwow") {
                    return Some(clean);
                }
            }
        }

        // Try h1 or .entry-title
        for sel_str in &["h1.entry-title", "h1", ".video-title"] {
            if let Ok(sel) = Selector::parse(sel_str) {
                if let Some(el) = document.select(&sel).next() {
                    let text = el.text().collect::<String>().trim().to_string();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
            }
        }

        None
    }

    /// Extract poster/thumbnail URL from og:image or other sources
    fn extract_poster(&self, document: &Html) -> Option<String> {
        // og:image is the most reliable source
        if let Ok(sel) = Selector::parse("meta[property='og:image']") {
            if let Some(el) = document.select(&sel).next() {
                if let Some(content) = el.value().attr("content") {
                    if !content.is_empty() {
                        return Some(content.to_string());
                    }
                }
            }
        }

        // Fallback: twitter:image
        if let Ok(sel) = Selector::parse("meta[name='twitter:image']") {
            if let Some(el) = document.select(&sel).next() {
                if let Some(content) = el.value().attr("content") {
                    if !content.is_empty() {
                        return Some(content.to_string());
                    }
                }
            }
        }

        // Fallback: first prominent image
        if let Ok(sel) = Selector::parse("img.wp-post-image, img.attachment-full") {
            if let Some(el) = document.select(&sel).next() {
                if let Some(src) = el.value().attr("src").or_else(|| el.value().attr("data-src")) {
                    if !src.is_empty() {
                        return Some(src.to_string());
                    }
                }
            }
        }

        None
    }

    /// Extract embed iframe URL from page (onlysubthai.com)
    fn extract_embed_url(&self, document: &Html, html: &str) -> Option<String> {
        // Look for iframe with onlysubthai.com
        if let Ok(sel) = Selector::parse("iframe") {
            for iframe in document.select(&sel) {
                if let Some(src) = iframe.value().attr("src") {
                    if src.contains("onlysubthai.com") || src.contains("onlysubthai") {
                        let url = if src.starts_with("//") {
                            format!("https:{}", src)
                        } else if src.starts_with("/") {
                            // Relative URL — shouldn't happen for embed but handle it
                            return None;
                        } else {
                            src.to_string()
                        };
                        return Some(url);
                    }
                }
            }
        }

        // Fallback: regex search for onlysubthai URL in raw HTML
        let re = Regex::new(r#"(https?://[^"'\s<>]*onlysubthai\.com[^"'\s<>]*)"#).ok()?;
        re.captures(html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract video ID from onlysubthai embed URL
    /// e.g., `https://onlysubthai.com/v/abcdef?sid=5917&t=hls` → "abcdef"
    #[allow(dead_code)]
    fn extract_video_id(embed_url: &str) -> Option<String> {
        let re = Regex::new(r"onlysubthai\.com/v/([a-zA-Z0-9_-]+)").ok()?;
        re.captures(embed_url)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Get series info from URL
    /// Tries HTTP fetch first; falls back to URL-derived info for Chrome detection
    pub async fn get_series_info(
        &self,
        url: &str,
        _domain: &str,
    ) -> Result<JavwowSeriesInfo, String> {
        eprintln!("[JavWow] get_series_info called for: {}", url);

        // Try HTTP fetch first (may work without Cloudflare challenge)
        match self.try_fetch_html(url).await {
            Ok(html) => {
                eprintln!("[JavWow] HTTP fetch succeeded, HTML length: {} bytes", html.len());
                self.parse_html(url, &html)
            }
            Err(e) => {
                eprintln!("[JavWow] HTTP fetch failed: {}, using URL-based info", e);
                // Fall back to URL-derived info; Chrome detector will handle video detection
                let slug = Self::extract_slug(url)
                    .unwrap_or_else(|| "Unknown".to_string());

                Ok(JavwowSeriesInfo {
                    url: url.to_string(),
                    title: slug.to_uppercase(),
                    total_episodes: 1,
                    poster_url: None,
                    embed_url: None,
                })
            }
        }
    }

    /// Search for videos on javwow.com
    pub async fn search(&self, query: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
        let url = format!("https://{}/?s={}&paged={}", domain, query, page);

        eprintln!("[JavWow] Search URL: {}", url);

        let html = match self.try_fetch_html(&url).await {
            Ok(html) => html,
            Err(_) => return Ok(Vec::new()), // Cloudflare blocked — return empty
        };

        let document = Html::parse_document(&html);
        let mut results = Vec::new();

        // WordPress search results — look for post entries with links
        let selectors = [
            r#"article a[href*="javwow.com"]"#,
            ".post-item a",
            ".search-results a",
            "h2.entry-title a",
            "h3.entry-title a",
            ".post-title a",
        ];

        let mut seen = std::collections::HashSet::new();

        for sel_str in &selectors {
            if let Ok(sel) = Selector::parse(sel_str) {
                for el in document.select(&sel) {
                    let href = el.value().attr("href").unwrap_or("").to_string();
                    if href.is_empty() || seen.contains(&href) { continue; }

                    let title = el.text().collect::<String>().trim().to_string();
                    if title.is_empty() { continue; }

                    seen.insert(href.clone());

                    // Try to extract poster from nearby img
                    let poster = el.select(&Selector::parse("img").unwrap())
                        .next()
                        .and_then(|img| {
                            img.value().attr("data-src")
                                .or_else(|| img.value().attr("src"))
                                .map(|s| s.to_string())
                        });

                    results.push(SearchResult {
                        title,
                        poster_url: poster,
                        url: href,
                        source: "javwow".to_string(),
                        total_episodes: None,
                        description: None,
                        rating: None,
                        year: None,
                        genre: None,
                        duration: None,
                    });
                }
            }
            if !results.is_empty() { break; }
        }

        // Deduplicate
        let mut dedup = std::collections::HashSet::new();
        results.retain(|r| dedup.insert(r.url.clone()));

        eprintln!("[JavWow] Search found {} results", results.len());
        Ok(results)
    }

    /// Browse categories
    pub async fn browse(&self, category: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
        let url = if category == "latest" {
            format!("https://{}/page/{}/", domain, page)
        } else {
            format!("https://{}/category/{}/page/{}/", domain, category, page)
        };

        let html = match self.try_fetch_html(&url).await {
            Ok(html) => html,
            Err(_) => return Ok(Vec::new()),
        };

        self.parse_listing(&html, domain)
    }

    /// Parse listing page HTML into search results
    fn parse_listing(&self, html: &str, _domain: &str) -> Result<Vec<SearchResult>, String> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // WordPress post listings
        let link_sels = [
            r#"article a[href*="javwow.com"]"#,
            ".post-item a",
            "h2.entry-title a",
            "h3.entry-title a",
            ".post-title a",
        ];

        let mut seen = std::collections::HashSet::new();

        for sel_str in &link_sels {
            if let Ok(sel) = Selector::parse(sel_str) {
                for el in document.select(&sel) {
                    let href = el.value().attr("href").unwrap_or("").to_string();
                    if href.is_empty() || seen.contains(&href) { continue; }

                    // Skip category/tag/archive links
                    if href.contains("/category/") || href.contains("/tag/")
                        || href.contains("/page/") || href.contains("/?s=") { continue; }

                    let title = el.text().collect::<String>().trim().to_string();
                    if title.is_empty() { continue; }

                    seen.insert(href.clone());

                    // Extract poster from parent article or nearby img
                    let poster = el.select(&Selector::parse("img").unwrap())
                        .next()
                        .and_then(|img| {
                            img.value().attr("data-src")
                                .or_else(|| img.value().attr("src"))
                                .map(|s| s.to_string())
                        });

                    results.push(SearchResult {
                        title,
                        poster_url: poster,
                        url: href,
                        source: "javwow".to_string(),
                        total_episodes: None,
                        description: None,
                        rating: None,
                        year: None,
                        genre: None,
                        duration: None,
                    });
                }
            }
            if !results.is_empty() { break; }
        }

        let mut dedup = std::collections::HashSet::new();
        results.retain(|r| dedup.insert(r.url.clone()));

        Ok(results)
    }

    /// List available browse categories
    pub fn list_categories(&self, _domain: &str) -> Vec<SiteCategory> {
        vec![
            SiteCategory { id: "latest".into(), label: "Latest".into(), source: "javwow".into() },
            SiteCategory { id: "censored".into(), label: "Censored".into(), source: "javwow".into() },
            SiteCategory { id: "uncensored".into(), label: "Uncensored".into(), source: "javwow".into() },
            SiteCategory { id: "subthai".into(), label: "Sub-Thai".into(), source: "javwow".into() },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_parser() -> JavwowParser {
        JavwowParser::new(Arc::new(RwLock::new(proxy::ProxyConfig::default())))
    }

    #[test]
    fn test_is_javwow_url() {
        assert!(JavwowParser::is_javwow_url("https://javwow.com/start-383/", "javwow.com"));
        assert!(JavwowParser::is_javwow_url("https://javwow.com/ABC-123", "javwow.com"));
        assert!(JavwowParser::is_javwow_url("https://custom-jav.com/start-383/", "custom-jav.com"));
        assert!(!JavwowParser::is_javwow_url("https://rongyok.com/watch/?series_id=1004", "javwow.com"));
        assert!(!JavwowParser::is_javwow_url("https://hsck123.com/view/?id=abc", "javwow.com"));
    }

    #[test]
    fn test_extract_slug() {
        assert_eq!(JavwowParser::extract_slug("https://javwow.com/start-383/"), Some("start-383".to_string()));
        assert_eq!(JavwowParser::extract_slug("https://javwow.com/ABC-123"), Some("ABC-123".to_string()));
        assert_eq!(JavwowParser::extract_slug("https://javwow.com/"), None);
        assert_eq!(JavwowParser::extract_slug("https://other.com/page"), None);
    }

    #[test]
    fn test_extract_video_id() {
        assert_eq!(
            JavwowParser::extract_video_id("https://onlysubthai.com/v/abcdef?sid=5917&t=hls"),
            Some("abcdef".to_string())
        );
        assert_eq!(
            JavwowParser::extract_video_id("https://onlysubthai.com/v/xyz123"),
            Some("xyz123".to_string())
        );
        assert_eq!(JavwowParser::extract_video_id("https://other.com/page"), None);
    }

    #[test]
    fn test_parse_html_with_og_tags() {
        let parser = make_parser();
        let html = r#"
            <html>
            <head>
                <meta property="og:title" content="START-383 Thai Subtitle" />
                <meta property="og:image" content="https://javwow.com/wp-content/uploads/2025/11/START-383.jpg" />
                <meta property="og:url" content="https://javwow.com/start-383/" />
            </head>
            <body>
                <iframe src="https://onlysubthai.com/v/abc123?sid=5917&t=hls"></iframe>
            </body>
            </html>
        "#;

        let info = parser.parse_html("https://javwow.com/start-383/", html).unwrap();
        assert_eq!(info.title, "START-383 Thai Subtitle");
        assert_eq!(info.poster_url, Some("https://javwow.com/wp-content/uploads/2025/11/START-383.jpg".to_string()));
        assert_eq!(info.embed_url, Some("https://onlysubthai.com/v/abc123?sid=5917&t=hls".to_string()));
        assert_eq!(info.total_episodes, 1);
    }

    #[test]
    fn test_parse_html_title_from_tag() {
        let parser = make_parser();
        let html = r#"
            <html>
            <head><title>START-383 SubThai - JavWow</title></head>
            <body></body>
            </html>
        "#;

        let info = parser.parse_html("https://javwow.com/start-383/", html).unwrap();
        assert_eq!(info.title, "START-383 SubThai");
    }

    #[test]
    fn test_parse_html_no_title_falls_back_to_slug() {
        let parser = make_parser();
        let html = r#"<html><head></head><body></body></html>"#;

        let info = parser.parse_html("https://javwow.com/start-383/", html).unwrap();
        assert_eq!(info.title, "START-383");
    }

    #[test]
    fn test_extract_embed_url_from_iframe() {
        let parser = make_parser();
        let html = r#"
            <html><body>
                <iframe src="//onlysubthai.com/v/test123?sid=5917&t=hls" width="100%" height="400"></iframe>
            </body></html>
        "#;

        let embed = parser.extract_embed_url(&Html::parse_document(html), html);
        assert_eq!(embed, Some("https://onlysubthai.com/v/test123?sid=5917&t=hls".to_string()));
    }

    #[test]
    fn test_extract_embed_url_from_regex_fallback() {
        let parser = make_parser();
        let html = r#"
            <html><body>
                <script>var embed = "https://onlysubthai.com/v/hidden456?t=hls";</script>
            </body></html>
        "#;

        let embed = parser.extract_embed_url(&Html::parse_document(html), html);
        assert_eq!(embed, Some("https://onlysubthai.com/v/hidden456?t=hls".to_string()));
    }
}
