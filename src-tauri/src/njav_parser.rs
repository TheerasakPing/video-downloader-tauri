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
    ///   /snos-034/  → WordPress page with advanced_iframe → missav.guide → 302 → javxx.com
    ///   javxx.com is a React SPA that embeds surrit.store video player via iframe
    ///
    /// We pre-resolve the iframe chain server-side (njav → missav → javxx) so that
    /// Chrome detector receives the final javxx.com URL instead of the njav.org page.
    /// Chrome detector then handles the SPA rendering and surrit.store iframe.
    pub async fn get_series_info(
        &self,
        series_url: &str,
        _domain: &str,
    ) -> Result<NjavSeriesInfo, String> {
        eprintln!("[Njav] get_series_info called for: {}", series_url);

        // Extract video code from URL (e.g., "snos-034" from https://njav.org/snos-034/)
        let video_code = self.extract_video_code(series_url);

        // Try to fetch the page HTML for title/poster AND iframe resolution
        let mut title = video_code.to_uppercase();
        let mut poster_url = None;
        let mut direct_page_url = Some(series_url.to_string());

        // Phase 1: Synchronous extraction from HTML (no awaits while Html is alive)
        let iframe_src: Option<String> = if let Ok(html) = self.try_fetch(series_url).await {
            let document = Html::parse_document(&html);

            // Extract title from page
            if let Some(t) = self.extract_title_from_html(&document, &html) {
                title = t;
            }

            // Extract poster
            poster_url = self.extract_poster(&document);

            // Extract iframe src for later resolution (after dropping document)
            self.extract_advanced_iframe_src(&html)
        } else {
            None
        };

        // Phase 2: Async redirect resolution (document is dropped, safe to await)
        if let Some(ref src) = iframe_src {
            eprintln!("[Njav] Found iframe URL: {}", src);
            match self.resolve_redirect(src).await {
                Some(resolved_url) => {
                    eprintln!("[Njav] Resolved video page: {}", resolved_url);
                    direct_page_url = Some(resolved_url);
                }
                None => {
                    eprintln!(
                        "[Njav] Could not resolve iframe redirect, falling back to original URL"
                    );
                }
            }
        } else {
            eprintln!("[Njav] No advanced_iframe found in HTML, using original URL");
        }

        eprintln!(
            "[Njav] Video code: {}, Title: {}, Direct URL: {:?}",
            video_code, title, direct_page_url
        );

        Ok(NjavSeriesInfo {
            url: series_url.to_string(),
            title,
            total_episodes: 1,
            poster_url,
            video_code,
            direct_page_url,
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
        url.trim_end_matches('/')
            .split('/')
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
}
