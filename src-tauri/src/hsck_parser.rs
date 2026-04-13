/// Parser สำหรับ hsck123.com และ domain สำรอง
///
/// โครงสร้างของเว็บ:
/// - หน้ารายการ: `/?type=TYPE&p=PAGE` หรือ `/`
/// - หน้าวิดีโอ: `/view/?id=XXXXXXXX`
/// - Video URL ฝังใน: `<img id="video_img" src="M3U8_URL" alt="POSTER_URL">`
/// - Title อยู่ใน `<title>TITLE - 黄色仓库 - hsck123.com</title>`
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::proxy;
use crate::{SearchResult, SiteCategory};

#[derive(Debug, Clone)]
pub struct HsckVideoInfo {
    #[allow(dead_code)]
    pub id: String,
    pub title: String,
    pub video_url: String,
    pub poster_url: Option<String>,
}

pub struct HsckParser {
    proxy_config: Arc<RwLock<proxy::ProxyConfig>>,
}

impl HsckParser {
    pub fn new(proxy_config: Arc<RwLock<proxy::ProxyConfig>>) -> Self {
        Self { proxy_config }
    }

    fn client(&self) -> Client {
        proxy::build_client(&self.proxy_config.read().unwrap())
    }

    /// ตรวจสอบว่า URL เป็น hsck123.com หรือ domain ที่กำหนด
    pub fn is_hsck_url(url: &str, domain: &str) -> bool {
        url.contains(domain)
            || url.contains("hsck123.com")
            || url.contains("cctv12306.com")
    }

    /// ดึง video ID จาก URL `/view/?id=XXXXXXXX`
    pub fn parse_video_id(url: &str) -> Option<String> {
        let re = Regex::new(r"[?&]id=([a-zA-Z0-9]+)").ok()?;
        re.captures(url)?.get(1).map(|m| m.as_str().to_string())
    }

    /// สร้าง URL สำหรับหน้าวิดีโอจาก domain + id
    #[allow(dead_code)]
    fn make_video_url(domain: &str, id: &str) -> String {
        format!("https://{}/view/?id={}", domain, id)
    }

    /// ดึงข้อมูลวิดีโอหน้าเดียวจาก URL
    pub async fn get_video_info(
        &self,
        url: &str,
        _domain: &str,
    ) -> Result<HsckVideoInfo, String> {
        let id = Self::parse_video_id(url)
            .unwrap_or_else(|| "unknown".to_string());

        eprintln!("[HSCK] Fetching video page: {}", url);

        let response = self
            .client()
            .get(url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        eprintln!("[HSCK] HTML length: {} bytes", html.len());

        let (title, video_url, poster_url) = Self::extract_video_data(&html)?;

        eprintln!("[HSCK] Title: {}", title);
        eprintln!("[HSCK] Video URL: {}", video_url);

        Ok(HsckVideoInfo {
            id,
            title,
            video_url,
            poster_url,
        })
    }

    /// แยก title, video_url, poster_url จาก HTML ของหน้าวิดีโอ
    fn extract_video_data(html: &str) -> Result<(String, String, Option<String>), String> {
        let document = Html::parse_document(html);

        // ดึง video URL และ poster จาก <img id="video_img" src="M3U8" alt="POSTER">
        // NOTE: เว็บนี้ใช้ src เป็น video URL และ alt เป็น poster URL (สลับกัน)
        let img_selector = Selector::parse("img#video_img").unwrap();
        let (video_url, poster_url) = if let Some(img) = document.select(&img_selector).next() {
            let src = img.value().attr("src").unwrap_or("").to_string();
            let alt = img.value().attr("alt").map(|s| s.to_string());
            (src, alt)
        } else {
            // Fallback: ค้นหา m3u8 URL จาก HTML ตรงๆ
            let m3u8_re = Regex::new(r#"https?://[^\s"'<>]+\.m3u8[^\s"'<>]*"#).unwrap();
            let video_url = m3u8_re
                .find(html)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            (video_url, None)
        };

        if video_url.is_empty() {
            return Err("ไม่พบ Video URL ในหน้านี้".to_string());
        }

        // ดึง title จาก <title> tag
        // Format: "TITLE - 黄色仓库 - hsck123.com"
        let title = Self::extract_title(&document, html);

        Ok((title, video_url, poster_url))
    }

    /// ดึง title จาก title tag โดยตัด suffix ที่ไม่ต้องการ
    fn extract_title(document: &Html, _html: &str) -> String {
        let title_selector = Selector::parse("title").unwrap();
        if let Some(title_el) = document.select(&title_selector).next() {
            let raw = title_el.text().collect::<String>();
            // ตัด suffix: " - 黄色仓库 - hsck123.com" หรือ pattern คล้ายกัน
            let re = Regex::new(r"\s*-\s*黄色仓库.*$").unwrap();
            let cleaned = re.replace(&raw, "").trim().to_string();
            if !cleaned.is_empty() {
                return cleaned;
            }
            // ถ้าไม่ match ให้ตัดแค่ส่วนหลัง " - "
            if let Some(idx) = raw.rfind(" - ") {
                let truncated = &raw[..idx];
                if !truncated.is_empty() {
                    return truncated.trim().to_string();
                }
            }
            return raw.trim().to_string();
        }
        "Unknown Video".to_string()
    }

    /// ดึงรายการวิดีโอจากหน้า list (homepage หรือ category)
    /// Returns: Vec<(id, title, poster_url)>
    pub async fn list_videos(
        &self,
        domain: &str,
        page: u32,
        category: Option<&str>,
    ) -> Result<Vec<(String, String, Option<String>)>, String> {
        let url = match category {
            Some(cat) => format!("https://{}/?type={}&p={}", domain, cat, page),
            None => format!("https://{}/?p={}", domain, page),
        };

        eprintln!("[HSCK] Fetching list page: {}", url);

        let response = self
            .client()
            .get(&url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let document = Html::parse_document(&html);

        // ดึงรายการจาก <a href="/view/?id=XXXXXXXX" title="TITLE">
        let link_selector = Selector::parse(r#"a[href*="/view/?id="]"#).unwrap();
        let img_selector_inner = Selector::parse("img").unwrap();

        let mut videos: Vec<(String, String, Option<String>)> = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for link in document.select(&link_selector) {
            let href = link.value().attr("href").unwrap_or("");
            let id = match Self::parse_video_id(href) {
                Some(id) => id,
                None => continue,
            };

            if seen_ids.contains(&id) {
                continue;
            }
            seen_ids.insert(id.clone());

            // ดึง title จาก title attribute หรือ text content
            let title = link
                .value()
                .attr("title")
                .map(|s| s.to_string())
                .unwrap_or_else(|| link.text().collect::<String>().trim().to_string());

            if title.is_empty() {
                continue;
            }

            // ดึง poster จาก img ลูก (data-original หรือ src)
            let poster = link.select(&img_selector_inner).next().and_then(|img| {
                img.value()
                    .attr("data-original")
                    .or_else(|| img.value().attr("src"))
                    .map(|s| s.to_string())
            });

            videos.push((id, title, poster));
        }

        eprintln!("[HSCK] Found {} videos on list page", videos.len());
        Ok(videos)
    }

    pub async fn search(&self, query: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
        let url = format!("https://{}?s={}&p={}", domain, query, page);
        let resp = self.client().get(&url).send().await.map_err(|e| e.to_string())?;
        let html_text = resp.text().await.map_err(|e| e.to_string())?;
        let document = Html::parse_document(&html_text);

        let mut results = Vec::new();
        let link_selector = Selector::parse(r#"a[href*="/view/?id="]"#).unwrap();

        for el in document.select(&link_selector) {
            let href = el.value().attr("href").unwrap_or("").to_string();
            let title = el.value().attr("title").unwrap_or("").to_string();
            if title.is_empty() || href.is_empty() { continue; }

            let poster = el.select(&Selector::parse("img").unwrap())
                .next()
                .and_then(|img| img.value().attr("data-original").or_else(|| img.value().attr("src")).map(|s| s.to_string()));

            let full_url = if href.starts_with("http") {
                href
            } else {
                format!("https://{}{}", domain, if href.starts_with('/') { "" } else { "/" }) + &href
            };

            results.push(SearchResult {
                title,
                poster_url: poster,
                url: full_url,
                source: "hsck".to_string(),
                total_episodes: None,
                description: None,
                rating: None,
                year: None,
                genre: None,
                duration: None,
            });
        }

        // Deduplicate by URL
        let mut seen = std::collections::HashSet::new();
        results.retain(|r| seen.insert(r.url.clone()));
        Ok(results)
    }

    pub fn list_categories(&self, _domain: &str) -> Vec<SiteCategory> {
        vec![
            SiteCategory { id: "latest".into(), label: "Latest".into(), source: "hsck".into() },
            SiteCategory { id: "hot".into(), label: "Hot".into(), source: "hsck".into() },
            SiteCategory { id: "chinese".into(), label: "Chinese".into(), source: "hsck".into() },
            SiteCategory { id: "japanese".into(), label: "Japanese".into(), source: "hsck".into() },
            SiteCategory { id: "korean".into(), label: "Korean".into(), source: "hsck".into() },
        ]
    }

    pub async fn browse(&self, category: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
        let cat = if category == "latest" { None } else { Some(category) };
        let items = self.list_videos(domain, page as u32, cat).await?;
        Ok(items.into_iter().map(|(id, title, poster)| {
            SearchResult {
                title,
                poster_url: poster,
                url: format!("https://{}/view/?id={}", domain, id),
                source: "hsck".to_string(),
                total_episodes: None,
                description: None,
                rating: None,
                year: None,
                genre: None,
                duration: None,
            }
        }).collect())
    }

    /// ดึงข้อมูล series จาก URL (สำหรับ UnifiedSeriesInfo)
    /// hsck123.com มีแค่วิดีโอเดี่ยว ไม่มี series ดังนั้น total_episodes = 1
    pub async fn get_series_info(
        &self,
        url: &str,
        domain: &str,
    ) -> Result<HsckSeriesInfo, String> {
        let info = self.get_video_info(url, domain).await?;
        let mut episode_urls = HashMap::new();
        episode_urls.insert(1i32, info.video_url);

        Ok(HsckSeriesInfo {
            title: info.title,
            total_episodes: 1,
            poster_url: info.poster_url,
            episode_urls,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HsckSeriesInfo {
    pub title: String,
    pub total_episodes: i32,
    pub poster_url: Option<String>,
    pub episode_urls: HashMap<i32, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hsck_url() {
        assert!(HsckParser::is_hsck_url("https://hsck123.com/view/?id=uo0ggiik", "hsck123.com"));
        assert!(HsckParser::is_hsck_url("https://cctv12306.com/view/?id=abc123", "hsck123.com"));
        assert!(!HsckParser::is_hsck_url("https://rongyok.com/watch/?series_id=1004", "hsck123.com"));
    }

    #[test]
    fn test_parse_video_id() {
        assert_eq!(
            HsckParser::parse_video_id("https://hsck123.com/view/?id=uo0ggiik"),
            Some("uo0ggiik".to_string())
        );
        assert_eq!(
            HsckParser::parse_video_id("https://cctv12306.com/view/?id=muydrhsx"),
            Some("muydrhsx".to_string())
        );
        assert_eq!(
            HsckParser::parse_video_id("https://hsck123.com/"),
            None
        );
    }

    #[test]
    fn test_extract_video_data_from_html() {
        let html = r#"
            <html>
            <head><title>测试视频 - 黄色仓库 - hsck123.com</title></head>
            <body>
              <img id="video_img" alt="https://aliyun.cctv05.com/i/b1000415.jpg"
                   src="https://t0.97img.com/b1000415/a.m3u8">
            </body>
            </html>
        "#;

        let (title, video_url, poster_url) = HsckParser::extract_video_data(html).unwrap();
        assert_eq!(title, "测试视频");
        assert_eq!(video_url, "https://t0.97img.com/b1000415/a.m3u8");
        assert_eq!(poster_url, Some("https://aliyun.cctv05.com/i/b1000415.jpg".to_string()));
    }
}
