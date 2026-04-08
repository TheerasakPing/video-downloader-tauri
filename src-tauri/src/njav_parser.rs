use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NjavSeriesInfo {
    pub url: String,
    pub title: String,
    pub total_episodes: i32,
    pub poster_url: Option<String>,
    /// Video code extracted from URL (e.g. "snos-034")
    pub video_code: String,
    /// If single-episode page, contains the page URL for Chrome detection
    pub direct_page_url: Option<String>,
    /// Direct javxx.com player URL (bypasses Cloudflare on njav.org)
    pub javxx_url: Option<String>,
}

pub struct NjavParser {
    client: Client,
}

impl NjavParser {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Check if URL belongs to njav.org
    pub fn is_njav_url(url: &str, _domain: &str) -> bool {
        url.contains("njav.org")
    }

    /// Extract series info from njav.org page
    ///
    /// njav.org structure:
    ///   /snos-034/  → Cloudflare blocked (403). Must use javxx.com instead.
    ///   javxx.com is a React SPA that embeds surrit.store video player via iframe.
    ///   URL pattern: https://javxx.com/en/v/{code}
    ///
    /// Strategy: Skip Cloudflare-protected njav.org entirely.
    /// Go directly to javxx.com which hosts the actual video player.
    /// Chrome detector then handles the SPA rendering and surrit.store iframe.
    pub async fn get_series_info(
        &self,
        series_url: &str,
        _domain: &str,
    ) -> Result<NjavSeriesInfo, String> {
        eprintln!("[Njav] get_series_info called for: {}", series_url);

        // Extract video code from URL (e.g., "snos-034" from https://njav.org/snos-034/)
        let video_code = self.extract_video_code(series_url);
        let video_code_lower = video_code.to_lowercase();

        // Build list of candidate URL suffixes to try on javxx.com
        // Some videos are under "uncensored-leaked" or other categories
        let candidates = vec![
            video_code_lower.clone(),
            format!("{}-uncensored-leaked", video_code_lower),
            format!("{}-uncensored", video_code_lower),
        ];

        let mut title = video_code.to_uppercase();
        let mut poster_url: Option<String> = None;
        let mut found_javxx_url: Option<String> = None;

        for suffix in &candidates {
            let url = format!("https://javxx.com/en/v/{}", suffix);
            eprintln!("[Njav] Trying javxx.com: {}", url);

            if let Ok(html) = self.try_fetch(&url).await {
                if !html.contains("404 Not Found") && html.contains("WatchPlayer") {
                    eprintln!("[Njav] Found video at: {}", url);
                    found_javxx_url = Some(url);

                    let document = Html::parse_document(&html);
                    if let Some(t) = self.extract_title_from_html(&document, &html) {
                        let cleaned = t
                            .replace("JAVxx - Watch ", "")
                            .replace("JAV Online in HD", "")
                            .replace(" | Free Japan JAV MissAV", "")
                            .replace(" | Free Japan AV MissAV", "")
                            .trim()
                            .to_string();
                        if !cleaned.is_empty() {
                            title = cleaned;
                        }
                    }
                    poster_url = self.extract_poster_from_javxx(&html);
                    break;
                } else {
                    eprintln!("[Njav] javxx.com returned 404 for: {}", suffix);
                }
            }
        }

        // If none of the candidates worked, still use the original code as fallback
        let javxx_url = found_javxx_url
            .unwrap_or_else(|| format!("https://javxx.com/en/v/{}", video_code_lower));

        Ok(NjavSeriesInfo {
            url: series_url.to_string(),
            title,
            total_episodes: 1,
            poster_url,
            video_code,
            direct_page_url: Some(series_url.to_string()),
            javxx_url: Some(javxx_url),
        })
    }

    /// Fetch page HTML
    async fn try_fetch(&self, url: &str) -> Result<String, String> {
        let response = self
            .client
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

        eprintln!("[Njav] HTML length: {} bytes", html.len());
        Ok(html)
    }

    /// Extract video code from URL (e.g., "snos-034")
    fn extract_video_code(&self, url: &str) -> String {
        // Extract last path segment: https://njav.org/snos-034/ → snos-034
        // Be robust against trailing characters like colons or spaces
        let cleaned = url.trim_end_matches(|c: char| !c.is_alphanumeric());
        
        cleaned.split('/')
            .filter(|s| !s.is_empty())
            .last()
            .unwrap_or("unknown")
            .to_string()
    }

    /// Extract title from HTML
    fn extract_title_from_html(&self, document: &Html, html: &str) -> Option<String> {
        // Try og:title first (usually contains the video code + description)
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

        // Try <title> tag
        if let Ok(sel) = Selector::parse("title") {
            if let Some(el) = document.select(&sel).next() {
                let text = el.text().collect::<String>();
                let clean = text
                    .split(" - ")
                    .next()
                    .unwrap_or(&text)
                    .trim()
                    .to_string();
                if !clean.is_empty() {
                    return Some(clean);
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

        // Try entry-title class (common in WordPress)
        if let Ok(sel) = Selector::parse(".entry-title") {
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


    /// Extract the advanced_iframe src from njav.org HTML.
    /// njav.org uses the WordPress "Advanced iFrame" plugin which embeds
    /// the video page (e.g., missav.guide/snos-034) in an iframe.
    fn extract_advanced_iframe_src(&self, html: &str) -> Option<String> {
        let document = Html::parse_document(html);

        // Try id="advanced_iframe" first (njav.org uses WordPress Advanced iFrame plugin)
        if let Ok(sel) = Selector::parse("iframe#advanced_iframe") {
            if let Some(el) = document.select(&sel).next() {
                if let Some(src) = el.value().attr("src") {
                    if src.starts_with("http") {
                        return Some(src.to_string());
                    }
                }
            }
        }

        // Fallback: any iframe with missav/javxx/surrit in src
        if let Ok(sel) = Selector::parse("iframe[src]") {
            for el in document.select(&sel) {
                if let Some(src) = el.value().attr("src") {
                    if src.contains("missav") || src.contains("javxx") || src.contains("surrit") {
                        return Some(src.to_string());
                    }
                }
            }
        }

        None
    }

    /// Follow a redirect (e.g., missav.guide/snos-034 → javxx.com/th/v/snos-034-uncensored-leaked)
    /// Returns the final URL after following all redirects.
    async fn resolve_redirect(&self, url: &str) -> Option<String> {
        let response = self.client.get(url).send().await.ok()?;

        let final_url = response.url().to_string();

        eprintln!("[Njav] Redirect {} → {}", url, final_url);

        // Accept known video page domains
        if final_url.contains("javxx.com")
            || final_url.contains("missav")
            || final_url.contains("surrit")
        {
            Some(final_url)
        } else {
            eprintln!("[Njav] Unexpected redirect target: {}", final_url);
            // Return it anyway as a fallback — the domain might change
            Some(final_url)
        }
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

        // Look for img with the video code in src (WordPress upload pattern)
        if let Ok(sel) = Selector::parse("img[src*='uploads']") {
            for el in document.select(&sel) {
                if let Some(src) = el.value().attr("src") {
                    // Usually the first uploaded image matching the code is the poster
                    if src.contains("uploads") && !src.contains("banner") && !src.contains("register") {
                        return Some(src.to_string());
                    }
                }
            }
        }

        None
    }

    /// Extract poster from javxx.com HTML
    /// javxx.com has: cover="https://icdn.javxx.com/img2/.../cover.webp"
    /// and also: <meta property="og:image" content="https://icdn.javxx.com/...">
    fn extract_poster_from_javxx(&self, html: &str) -> Option<String> {
        // Try cover="..." attribute on WatchPlayer tag
        let cover_re = Regex::new(r#"cover="(https://icdn\.javxx\.com/[^"]+)"#).ok()?;
        if let Some(captures) = cover_re.captures(html) {
            if let Some(m) = captures.get(1) {
                return Some(m.as_str().to_string());
            }
        }

        // Try og:image meta tag
        let og_re = Regex::new(r#"<meta\s+property="og:image"\s+content="([^"]+)"#).ok()?;
        if let Some(captures) = og_re.captures(html) {
            if let Some(m) = captures.get(1) {
                let url = m.as_str().to_string();
                if url.contains("icdn") || url.contains("cover") {
                    return Some(url);
                }
            }
        }

        // Try twitter:image
        let tw_re = Regex::new(r#"<meta\s+name="twitter:image"\s+content="([^"]+)"#).ok()?;
        if let Some(captures) = tw_re.captures(html) {
            if let Some(m) = captures.get(1) {
                let url = m.as_str().to_string();
                if url.contains("icdn") || url.contains("cover") {
                    return Some(url);
                }
            }
        }

        None
    }
}
