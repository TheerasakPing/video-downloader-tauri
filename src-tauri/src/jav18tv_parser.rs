/// Parser for 18jav.tv
///
/// Site structure:
/// - Video page: `https://18jav.tv/videos/{slug}` (e.g., zyfe6e)
/// - Likely Cloudflare protected
/// - Metadata in og:tags (og:title, og:image)
/// - Video embedded via iframe or direct player
///
/// Strategy (hybrid, same as javwow):
/// 1. HTTP fetch metadata (may work without CF challenge)
/// 2. If blocked -> WebView bypass -> extract metadata + iframe/video URL
/// 3. Chrome detector as last resort

use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::sync::{Arc, RwLock};
use crate::proxy;
use crate::{SearchResult, SiteCategory};

#[derive(Debug, Clone)]
pub struct Jav18tvSeriesInfo {
    pub url: String,
    pub title: String,
    pub total_episodes: i32,
    pub poster_url: Option<String>,
    /// Embed iframe URL if found
    pub embed_url: Option<String>,
    /// Direct video URL if found in page source
    pub direct_video_url: Option<String>,
}

pub struct Jav18tvParser {
    proxy_config: Arc<RwLock<proxy::ProxyConfig>>,
}

impl Jav18tvParser {
    pub fn new(proxy_config: Arc<RwLock<proxy::ProxyConfig>>) -> Self {
        Self { proxy_config }
    }

    fn client(&self) -> Client {
        proxy::build_client(&self.proxy_config.read().unwrap())
    }

    /// Check if URL belongs to 18jav.tv or configured domain
    pub fn is_jav18tv_url(url: &str, domain: &str) -> bool {
        url.contains(domain) || url.contains("18jav.tv")
    }

    /// Extract slug from URL (e.g., "zyfe6e" from "/videos/zyfe6e")
    fn extract_slug(url: &str) -> Option<String> {
        let re = Regex::new(r"18jav\.tv/videos/([a-zA-Z0-9_-]+)/?").ok()?;
        re.captures(url)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Try to fetch page HTML (may be blocked by Cloudflare)
    async fn try_fetch_html(&self, url: &str) -> Result<String, String> {
        let response = self
            .client()
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9,th;q=0.8")
            // Don't set Accept-Encoding; let reqwest handle compression via its built-in gzip/deflate support
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Cache-Control", "max-age=0")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
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

        // Check for Cloudflare challenge
        if html.contains("cf-browser-verification") || html.contains("challenge-platform")
            || html.contains("Just a moment") || html.contains("Checking your browser")
        {
            return Err("Cloudflare challenge page detected".to_string());
        }

        Ok(html)
    }

    /// Extract series info from HTML
    fn parse_html(&self, url: &str, html: &str) -> Result<Jav18tvSeriesInfo, String> {
        let document = Html::parse_document(html);

        let title = self.extract_title(&document)
            .unwrap_or_else(|| {
                Self::extract_slug(url)
                    .map(|s| s.to_uppercase())
                    .unwrap_or_else(|| "18JAV Video".to_string())
            });

        let poster_url = self.extract_poster(&document);
        let embed_url = self.extract_embed_url(&document, html);
        let direct_video_url = self.extract_direct_video_url(html);
        Ok(Jav18tvSeriesInfo {
            url: url.to_string(),
            title,
            total_episodes: 1,
            poster_url,
            embed_url,
            direct_video_url,
        })
    }

    fn extract_title(&self, document: &Html) -> Option<String> {
        // og:title
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

        // <title> tag
        if let Ok(sel) = Selector::parse("title") {
            if let Some(el) = document.select(&sel).next() {
                let text = el.text().collect::<String>();
                let clean = text
                    .split(" - ").next()
                    .unwrap_or(&text)
                    .trim()
                    .to_string();
                if !clean.is_empty() && !clean.contains("18jav") && !clean.contains("18 JAV") {
                    return Some(clean);
                }
            }
        }

        // h1 or video title selectors
        for sel_str in &["h1", "h1.entry-title", ".video-title", "[class*='title']"] {
            if let Ok(sel) = Selector::parse(sel_str) {
                if let Some(el) = document.select(&sel).next() {
                    let text = el.text().collect::<String>().trim().to_string();
                    if !text.is_empty() && text.len() < 200 {
                        return Some(text);
                    }
                }
            }
        }

        None
    }

    fn extract_poster(&self, document: &Html) -> Option<String> {
        // og:image
        if let Ok(sel) = Selector::parse("meta[property='og:image']") {
            if let Some(el) = document.select(&sel).next() {
                if let Some(content) = el.value().attr("content") {
                    if !content.is_empty() {
                        return Some(content.to_string());
                    }
                }
            }
        }

        // twitter:image
        if let Ok(sel) = Selector::parse("meta[name='twitter:image']") {
            if let Some(el) = document.select(&sel).next() {
                if let Some(content) = el.value().attr("content") {
                    if !content.is_empty() {
                        return Some(content.to_string());
                    }
                }
            }
        }

        // video poster attribute
        if let Ok(sel) = Selector::parse("video[poster]") {
            if let Some(el) = document.select(&sel).next() {
                if let Some(poster) = el.value().attr("poster") {
                    if !poster.is_empty() {
                        return Some(poster.to_string());
                    }
                }
            }
        }

        // First prominent image
        for sel_str in &["img.wp-post-image", "img.attachment-full", ".video-thumbnail img", "img[src*='thumb']"] {
            if let Ok(sel) = Selector::parse(sel_str) {
                if let Some(el) = document.select(&sel).next() {
                    if let Some(src) = el.value().attr("src").or_else(|| el.value().attr("data-src")) {
                        if !src.is_empty() {
                            return Some(src.to_string());
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract embed iframe URL
    fn extract_embed_url(&self, document: &Html, html: &str) -> Option<String> {
        // Look for iframes with video player URLs
        if let Ok(sel) = Selector::parse("iframe") {
            for iframe in document.select(&sel) {
                if let Some(src) = iframe.value().attr("src")
                    .or_else(|| iframe.value().attr("data-src"))
                    .or_else(|| iframe.value().attr("data-lazy-src"))
                {
                    if src.contains("/embed/") || src.contains("/v/") || src.contains("player")
                        || src.contains("m3u8") || src.contains(".mp4")
                        || src.contains("stream") || src.contains("video")
                    {
                        let url = if src.starts_with("//") {
                            format!("https:{}", src)
                        } else if src.starts_with("/") {
                            format!("https://18jav.tv{}", src)
                        } else {
                            src.to_string()
                        };
                        return Some(url);
                    }
                }
            }
        }

        // Check og:video meta tag
        for prop in &["og:video", "og:video:url", "og:video:secure_url"] {
            if let Ok(sel) = Selector::parse(&format!("meta[property='{}']", prop)) {
                if let Some(el) = document.select(&sel).next() {
                    if let Some(content) = el.value().attr("content") {
                        if !content.is_empty() {
                            return Some(content.to_string());
                        }
                    }
                }
            }
        }

        // Regex fallback
        let re = Regex::new(r#"(https?://[^"'\s<>]*(?:embed|player|video)[^"'\s<>]*)"#).ok()?;
        re.captures(html)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Try to extract a direct video URL from the page source
    fn extract_direct_video_url(&self, html: &str) -> Option<String> {
        // Pattern 1: var hlsUrl = "..." (18jav.tv specific)
        if let Ok(re) = Regex::new(r#"var\s+hlsUrl\s*=\s*["']([^"']+)["']"#) {
            if let Some(caps) = re.captures(html) {
                if let Some(m) = caps.get(1) {
                    let url = m.as_str().to_string();
                    if url.starts_with("http") {
                        return Some(url);
                    }
                }
            }
        }

        // Pattern 2: generic m3u8 URLs
        if let Ok(re) = Regex::new(r#"(https?://[^"'\s<>]+\.m3u8[^"'\s<>]*)"#) {
            if let Some(caps) = re.captures(html) {
                if let Some(m) = caps.get(1) {
                    let url = m.as_str().to_string();
                    if !url.contains(".css") && !url.contains(".js") {
                        return Some(url);
                    }
                }
            }
        }

        // Pattern 3: og:video meta tag (mp4)
        if let Ok(re) = Regex::new(r#"<meta\s+property=["']og:video["'][^>]+content=["']([^"']+)["']"#) {
            if let Some(caps) = re.captures(html) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().to_string());
                }
            }
        }

        // Pattern 4: mp4 URLs (skip preview/thumbnails)
        if let Ok(re) = Regex::new(r#"(https?://[^"'\s<>]+\.mp4[^"'\s<>]*)"#) {
            if let Some(caps) = re.captures(html) {
                if let Some(m) = caps.get(1) {
                    let url = m.as_str().to_string();
                    if !url.contains("preview") && !url.contains(".css") && !url.contains(".js") {
                        return Some(url);
                    }
                }
            }
        }

        None
    }

    /// Get series info from URL
    pub async fn get_series_info(
        &self,
        url: &str,
        _domain: &str,
    ) -> Result<Jav18tvSeriesInfo, String> {
        eprintln!("[18JAV] get_series_info called for: {}", url);

        match self.try_fetch_html(url).await {
            Ok(html) => {
                eprintln!("[18JAV] HTTP fetch succeeded, HTML length: {} bytes", html.len());
                self.parse_html(url, &html)
            }
            Err(e) => {
                eprintln!("[18JAV] HTTP fetch failed: {}, using URL-based info", e);
                let slug = Self::extract_slug(url)
                    .unwrap_or_else(|| "Unknown".to_string());

                Ok(Jav18tvSeriesInfo {
                    url: url.to_string(),
                    title: slug.to_uppercase(),
                    total_episodes: 1,
                    poster_url: None,
                    embed_url: None,
                    direct_video_url: None,
                })
            }
        }
    }

    /// Search for videos on 18jav.tv
    pub async fn search(&self, query: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
        let url = format!("https://{}/?s={}&page={}", domain, query, page);
        eprintln!("[18JAV] Search URL: {}", url);

        let html = match self.try_fetch_html(&url).await {
            Ok(html) => html,
            Err(_) => return Ok(Vec::new()),
        };

        self.parse_listing(&html, domain)
    }

    /// Browse categories
    pub async fn browse(&self, category: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
        let url = match category {
            "latest" => format!("https://{}/page/{}/", domain, page),
            other => format!("https://{}/category/{}/page/{}/", domain, other, page),
        };

        let html = match self.try_fetch_html(&url).await {
            Ok(html) => html,
            Err(_) => return Ok(Vec::new()),
        };

        self.parse_listing(&html, domain)
    }

    fn parse_listing(&self, html: &str, domain: &str) -> Result<Vec<SearchResult>, String> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let link_sels = [
            format!(r#"a[href*="{}/videos/"]"#, domain),
            r#"a[href*="/videos/"]"#.to_string(),
            "article a".to_string(),
            ".post-item a".to_string(),
            "h2.entry-title a".to_string(),
            "h3.entry-title a".to_string(),
            ".post-title a".to_string(),
            ".video-item a".to_string(),
        ];

        for sel_str in &link_sels {
            if let Ok(sel) = Selector::parse(sel_str) {
                for el in document.select(&sel) {
                    let href = el.value().attr("href").unwrap_or("").to_string();
                    if href.is_empty() || seen.contains(&href) { continue; }
                    if href.contains("/page/") || href.contains("/?s=")
                        || href.contains("/category/") || href.contains("/tag/") { continue; }

                    let title = el.text().collect::<String>().trim().to_string();
                    if title.is_empty() { continue; }

                    seen.insert(href.clone());

                    let poster = el.select(&Selector::parse("img").unwrap())
                        .next()
                        .and_then(|img| {
                            img.value().attr("data-src")
                                .or_else(|| img.value().attr("data-lazy-src"))
                                .or_else(|| img.value().attr("src"))
                                .map(|s| s.to_string())
                        });

                    results.push(SearchResult {
                        title,
                        poster_url: poster,
                        url: href,
                        source: "jav18tv".to_string(),
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

        eprintln!("[18JAV] Found {} listing results", results.len());
        Ok(results)
    }

    pub fn list_categories(&self, _domain: &str) -> Vec<SiteCategory> {
        vec![
            SiteCategory { id: "latest".into(), label: "Latest".into(), source: "jav18tv".into() },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_parser() -> Jav18tvParser {
        Jav18tvParser::new(Arc::new(RwLock::new(proxy::ProxyConfig::default())))
    }

    #[test]
    fn test_is_jav18tv_url() {
        assert!(Jav18tvParser::is_jav18tv_url("https://18jav.tv/videos/zyfe6e", "18jav.tv"));
        assert!(Jav18tvParser::is_jav18tv_url("https://18jav.tv/videos/abc123", "18jav.tv"));
        assert!(Jav18tvParser::is_jav18tv_url("https://custom-18jav.com/videos/test", "custom-18jav.com"));
        assert!(!Jav18tvParser::is_jav18tv_url("https://rongyok.com/watch/?series_id=1004", "18jav.tv"));
    }

    #[test]
    fn test_extract_slug() {
        assert_eq!(Jav18tvParser::extract_slug("https://18jav.tv/videos/zyfe6e"), Some("zyfe6e".to_string()));
        assert_eq!(Jav18tvParser::extract_slug("https://18jav.tv/videos/abc-123/"), Some("abc-123".to_string()));
        assert_eq!(Jav18tvParser::extract_slug("https://18jav.tv/"), None);
    }

    #[test]
    fn test_parse_html_with_og_tags() {
        let parser = make_parser();
        let html = r#"
            <html>
            <head>
                <meta property="og:title" content="Test Video Title" />
                <meta property="og:image" content="https://18jav.tv/images/thumb123.jpg" />
            </head>
            <body>
                <iframe src="https://player.example.com/embed/abc123"></iframe>
            </body>
            </html>
        "#;

        let info = parser.parse_html("https://18jav.tv/videos/zyfe6e", html).unwrap();
        assert_eq!(info.title, "Test Video Title");
        assert_eq!(info.poster_url, Some("https://18jav.tv/images/thumb123.jpg".to_string()));
        assert!(info.embed_url.is_some());
    }

    #[test]
    fn test_parse_html_no_title_falls_back_to_slug() {
        let parser = make_parser();
        let html = r#"<html><head></head><body></body></html>"#;

        let info = parser.parse_html("https://18jav.tv/videos/zyfe6e", html).unwrap();
        assert_eq!(info.title, "ZYFE6E");
    }
}
