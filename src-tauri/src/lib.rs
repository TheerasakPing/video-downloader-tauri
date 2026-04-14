mod baanjeen_parser;
mod backup;
mod chrome_detector;
mod downloader;
mod hsck_parser;
mod javwow_parser;
mod jav18tv_parser;
mod njav_parser;
mod njavtv_parser;
mod parser;
mod titan_parser;
mod library;
mod notifications;
mod proxy;
mod queue_db;
mod scheduler;
mod webhook;

mod utils;

use baanjeen_parser::BaanJeenParser;
use chrome_detector::ChromeVideoDetector;
use downloader::{check_ffmpeg, DownloadConfig, DownloadResult, DownloadState, VideoDownloader};
use hsck_parser::HsckParser;
use javwow_parser::JavwowParser;
use jav18tv_parser::Jav18tvParser;
use njav_parser::NjavParser;
use njavtv_parser::NjavtvParser;
use parser::RongyokParser;
use regex::Regex;
use serde::{Deserialize, Serialize};
use base64::Engine;
use library::LibraryDb;
use notifications::NotificationDb;
use queue_db::QueueDb;
use titan_parser::{TitanParser, TitanSeriesInfo, HlsKeyInfo};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

/// Extension trait for safe Mutex locking that recovers from poison.
trait SafeMutexLock<T> {
    fn safe_lock(&self) -> std::sync::MutexGuard<'_, T>;
}

impl<T> SafeMutexLock<T> for Mutex<T> {
    fn safe_lock(&self) -> std::sync::MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|e| {
            eprintln!("Warning: Mutex poisoned, recovering...");
            e.into_inner()
        })
    }
}

/// Decode a data URL (e.g. `data:image/jpeg;base64,...`) into raw bytes.
/// Returns `None` if the string is not a valid data URL.
fn decode_data_url(data_url: &str) -> Option<Vec<u8>> {
    if !data_url.starts_with("data:") {
        return None;
    }
    let base64_part = data_url.split(',').nth(1)?;
    base64::engine::general_purpose::STANDARD.decode(base64_part).ok()
}

fn derive_title_from_url(url: &str, fallback: &str) -> String {
    let slug = url
        .split('/')
        .filter(|s| !s.is_empty())
        .next_back()
        .and_then(|s| s.split('?').next())
        .unwrap_or(fallback)
        .trim_matches('/')
        .trim();

    if slug.is_empty() {
        return fallback.to_string();
    }

    let title = slug
        .replace('-', " ")
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if title.is_empty() {
        fallback.to_string()
    } else {
        title.to_uppercase()
    }
}

fn decode_packed_js_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    '\\' => out.push('\\'),
                    '\'' => out.push('\''),
                    '"' => out.push('"'),
                    '/' => out.push('/'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    other => {
                        out.push('\\');
                        out.push(other);
                    }
                }
            } else {
                out.push(ch);
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn encode_base_n(mut value: usize, radix: usize) -> String {
    const ALPHABET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    if value == 0 {
        return "0".to_string();
    }
    let mut buf = Vec::new();
    while value > 0 {
        buf.push(ALPHABET[value % radix] as char);
        value /= radix;
    }
    buf.iter().rev().collect()
}

fn unpack_packer_script(html: &str) -> Option<String> {
    let re = Regex::new(
        r#"eval\(function\(p,a,c,k,e,d\)\{.*?\}\('((?:\\.|[^'])*)',(\d+),(\d+),'((?:\\.|[^'])*)'\.split\('\|'\),\d+,\{\}\)\)"#,
    ).ok()?;
    let caps = re.captures(html)?;

    let payload = decode_packed_js_string(caps.get(1)?.as_str());
    let radix: usize = caps.get(2)?.as_str().parse().ok()?;
    let count: usize = caps.get(3)?.as_str().parse().ok()?;
    let symtab_raw = decode_packed_js_string(caps.get(4)?.as_str());
    let symtab: Vec<&str> = symtab_raw.split('|').collect();

    let token_re = Regex::new(r"\b\w+\b").ok()?;
    let unpacked = token_re.replace_all(&payload, |caps: &regex::Captures| {
        let word = caps.get(0).map(|m| m.as_str()).unwrap_or_default();
        if let Some(idx) = (0..count).find(|i| encode_base_n(*i, radix) == word) {
            let replacement = symtab.get(idx).copied().unwrap_or("");
            if replacement.is_empty() {
                word.to_string()
            } else {
                replacement.to_string()
            }
        } else {
            word.to_string()
        }
    });

    Some(unpacked.into_owned())
}

fn extract_avkuy_stream_from_player_html(html: &str) -> Option<(String, Option<String>)> {
    let unpacked = unpack_packer_script(html)?;

    let file_re = Regex::new(r#"["']file["']\s*:\s*["']([^"']+)["']"#).ok()?;
    let image_re = Regex::new(r#"["']image["']\s*:\s*["']([^"']+)["']"#).ok()?;

    let file = file_re
        .captures(&unpacked)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())?;

    let image = image_re
        .captures(&unpacked)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    Some((file, image))
}

async fn fetch_avkuy_stream_from_player_page(
    player_url: &str,
    source_url: &str,
    cookies: &[(String, String)],
) -> Option<(String, Option<String>)> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()
        .ok()?;

    let cookie_header = cookies
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("; ");

    let mut req = client
        .get(player_url)
        .header("Referer", source_url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8");

    if !cookie_header.is_empty() {
        req = req.header("Cookie", cookie_header);
    }

    let html = req.send().await.ok()?.text().await.ok()?;
    extract_avkuy_stream_from_player_html(&html)
}
use tauri::{AppHandle, Emitter, State, Manager};
use utils::{expand_path, sanitize_filename};

fn parse_cookie_string(cookie_str: &str) -> Vec<(String, String)> {
    cookie_str
        .split(';')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.trim().to_string();
            let val = parts.next()?.trim().to_string();
            if key.is_empty() { None } else { Some((key, val)) }
        })
        .collect()
}


#[derive(Debug, Default, Clone)]
struct WebviewExtractResult {
    title: Option<String>,
    poster_url: Option<String>,
    iframe_url: Option<String>,
    video_url: Option<String>,
    cookies: Vec<(String, String)>,
}

/// AVKuy-specific extraction via real Tauri WebView (better Cloudflare compatibility than CDP).
/// Returns best-effort metadata (title/poster/iframe) and direct video URL when detected.
async fn fetch_via_webview(
    app_handle: &AppHandle,
    url: &str,
) -> WebviewExtractResult {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    const LABEL: &str = "avkuy-helper";

    let mut result = WebviewExtractResult::default();

    let log_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("com.rongyok.downloader")
        .join("webview_debug.log");
    let log_to_file = |msg: &str| {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log_path) {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let _ = writeln!(f, "[{}] [AVKUY] {}", ts, msg);
        }
        eprintln!("[AVKUY WebView] {}", msg);
    };

    if let Some(w) = app_handle.get_webview_window(LABEL) {
        let _ = w.close();
        std::thread::sleep(std::time::Duration::from_millis(300));
    }

    let parsed_url = match url.parse() {
        Ok(u) => u,
        Err(e) => {
            log_to_file(&format!("URL parse error: {}", e));
            return result;
        }
    };

    let window = match WebviewWindowBuilder::new(
        app_handle,
        LABEL,
        WebviewUrl::External(parsed_url),
    )
    .title("avkuy-helper")
    .inner_size(1024.0, 768.0)
    .position(-32000.0, -32000.0)
    .visible(true)
    .decorations(false)
    .build()
    {
        Ok(w) => w,
        Err(e) => {
            log_to_file(&format!("Failed to build window: {}", e));
            return result;
        }
    };

    let meta_js = r#"
        (function() {
            const title = (document.title || '').toLowerCase();
            const bodyText = (document.body?.innerText || '').toLowerCase();
            const isCf =
                title.includes('just a moment') ||
                title.includes('attention required') ||
                bodyText.includes('checking your browser') ||
                document.getElementById('challenge-running') !== null ||
                document.getElementById('challenge-form') !== null ||
                document.querySelector('iframe[src*="challenges.cloudflare.com"]') !== null;

            if (isCf) {
                window.location.hash = 'ak_status=cf';
                return;
            }

            const get = (sel, attr = 'content') => {
                const el = document.querySelector(sel);
                if (!el) return '';
                if (attr === 'text') return (el.textContent || '').trim();
                return (el.getAttribute(attr) || '').trim();
            };

            let iframe = '';
            const frames = document.querySelectorAll('iframe');
            for (let i = 0; i < frames.length; i++) {
                const src =
                    frames[i].src ||
                    frames[i].getAttribute('data-src') ||
                    frames[i].getAttribute('data-lazy-src') ||
                    '';
                if (!src) continue;
                if (src.includes('/v/')) {
                    iframe = src.startsWith('//') ? 'https:' + src : src;
                    break;
                }
            }
            if (!iframe) {
                const html = document.documentElement.outerHTML;
                const m = html.match(/(https?:\/\/[^"'\s<>]*\/v\/[^"'\s<>]*)/i);
                if (m) iframe = m[1];
            }

            const payload = {
                title:
                    get("meta[property='og:title']") ||
                    get("h1", 'text') ||
                    document.title ||
                    '',
                poster:
                    get("meta[property='og:image']") ||
                    get("meta[name='twitter:image']") ||
                    get("video", 'poster') ||
                    '',
                iframe,
                cookies: document.cookie,
            };
            window.location.hash = 'ak_meta=' + encodeURIComponent(JSON.stringify(payload));
        })()
    "#;

    // Wait until metadata/iframe is visible (Cloudflare may take time)
    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
    for attempt in 0..20 {
        let _ = window.eval(meta_js);
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

        if let Ok(cur) = window.url() {
            let hash = cur.fragment().unwrap_or("").to_string();
            if let Some(encoded) = hash.strip_prefix("ak_meta=") {
                if let Ok(decoded) = urlencoding::decode(encoded) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&decoded) {
                        result.title = json
                            .get("title")
                            .and_then(|v| v.as_str())
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty());
                        result.poster_url = json
                            .get("poster")
                            .and_then(|v| v.as_str())
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty());
                        result.iframe_url = json
                            .get("iframe")
                            .and_then(|v| v.as_str())
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty());
                        // Extract cookies from main page
                        if let Some(cookie_str) = json.get("cookies").and_then(|v| v.as_str()) {
                            result.cookies = parse_cookie_string(cookie_str);
                        }
                        break;
                    }
                }
            } else if hash == "ak_status=cf" {
                let _ = app_handle.emit("log-info", format!("Waiting AVKuy Cloudflare... ({}/20)", attempt + 1));
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    // If iframe was found, open it in fresh window and detect stream URL
    if let Some(iframe_url) = result.iframe_url.clone() {
        let _ = window.close();
        std::thread::sleep(std::time::Duration::from_millis(400));

        let iframe_parsed = match iframe_url.parse() {
            Ok(u) => u,
            Err(_) => return result,
        };

        let window2 = match WebviewWindowBuilder::new(
            app_handle,
            LABEL,
            WebviewUrl::External(iframe_parsed),
        )
        .title("avkuy-player")
        .inner_size(1024.0, 768.0)
        .position(-32000.0, -32000.0)
        .visible(true)
        .decorations(false)
        .build()
        {
            Ok(w) => w,
            Err(_) => return result,
        };

        let observer_js = r#"
            (function() {
                if (performance.setResourceTimingBufferSize) {
                    performance.setResourceTimingBufferSize(5000);
                }
                if (!window.__FOUND_URLS) {
                    window.__FOUND_URLS = [];
                    const ok = (u) => u.includes('.m3u8') || u.includes('.mp4');
                    performance.getEntriesByType('resource').forEach(e => { if (ok(e.name)) window.__FOUND_URLS.push(e.name); });
                    try {
                        const o = new PerformanceObserver((list) => {
                            list.getEntries().forEach(e => { if (ok(e.name)) window.__FOUND_URLS.push(e.name); });
                        });
                        o.observe({ entryTypes: ['resource'] });
                    } catch(_) {}
                }
            })()
        "#;

        let click_js = r#"
            (function() {
                const selectors = ['button','video','img','[class*="play"]','.jw-poster','.jw-display-icon-container','iframe'];
                selectors.forEach(sel => {
                    document.querySelectorAll(sel).forEach(el => { try { el.click(); } catch(_) {} });
                });
                document.querySelectorAll('video').forEach(v => { try { v.play(); } catch(_) {} });
                try {
                    const el = document.elementFromPoint(window.innerWidth / 2, window.innerHeight / 2);
                    if (el) el.click();
                } catch(_) {}
            })()
        "#;

        let check_js = r#"
            (function() {
                if (window.__FOUND_URLS && window.__FOUND_URLS.length > 0) {
                    const m3u8 = window.__FOUND_URLS.filter(u => u.includes('.m3u8'));
                    const mp4 = window.__FOUND_URLS.filter(u => u.includes('.mp4'));
                    if (m3u8.length > 0) {
                        window.location.hash = 'ak_video=' + encodeURIComponent(m3u8[m3u8.length - 1]);
                        return;
                    }
                    if (mp4.length > 0) {
                        window.location.hash = 'ak_video=' + encodeURIComponent(mp4[0]);
                        return;
                    }
                }
                const html = document.documentElement.outerHTML;
                const m = html.match(/["'](https?:[^"']+\.m3u8[^"']*)["']/);
                if (m) window.location.hash = 'ak_video=' + encodeURIComponent(m[1]);
            })()
        "#;

        tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
        let _ = window2.eval(observer_js);

        for attempt in 0..14 {
            let _ = window2.eval(click_js);
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let _ = window2.eval(check_js);
            tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

            if let Ok(cur) = window2.url() {
                let hash = cur.fragment().unwrap_or("").to_string();
                if let Some(encoded) = hash.strip_prefix("ak_video=") {
                    if let Ok(decoded) = urlencoding::decode(encoded) {
                        let v = decoded.to_string();
                        if v.contains(".m3u8") || v.contains(".mp4") {
                            result.video_url = Some(v);
                            break;
                        }
                    }
                }
            }

            let _ = app_handle.emit("log-info", format!("AVKuy player scan {}/14...", attempt + 1));
        }

        // Extract cookies from player page
        let cookie_js = "window.location.hash = 'ak_cookies=' + encodeURIComponent(document.cookie);";
        let _ = window2.eval(cookie_js);
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        if let Ok(cur) = window2.url() {
            let hash = cur.fragment().unwrap_or("").to_string();
            if let Some(encoded) = hash.strip_prefix("ak_cookies=") {
                if let Ok(decoded) = urlencoding::decode(encoded) {
                    let player_cookies = parse_cookie_string(&decoded);
                    // Merge with main page cookies (player cookies take precedence)
                    for (k, v) in player_cookies {
                        if let Some(pos) = result.cookies.iter().position(|(ck, _)| ck == &k) {
                            result.cookies[pos] = (k, v);
                        } else {
                            result.cookies.push((k, v));
                        }
                    }
                }
            }
        }

        let _ = window2.close();
    } else {
        let _ = window.close();
    }

    result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainSettings {
    pub titan_domain: String,
    pub baanjeen_domain: String,
    pub rongyok_domain: String,
    #[serde(default = "DomainSettings::default_hsck_domain")]
    pub hsck_domain: String,
    #[serde(default = "DomainSettings::default_njavtv_domain")]
    pub njavtv_domain: String,
    #[serde(default = "DomainSettings::default_njav_domain")]
    pub njav_domain: String,
    #[serde(default = "DomainSettings::default_javwow_domain")]
    pub javwow_domain: String,
    #[serde(default = "DomainSettings::default_avkuy_domain")]
    pub avkuy_domain: String,
    #[serde(default = "DomainSettings::default_jav18tv_domain")]
    pub jav18tv_domain: String,
}

impl DomainSettings {
    fn default_hsck_domain() -> String {
        "hsck123.com".to_string()
    }
    fn default_njavtv_domain() -> String {
        "njavtv.com".to_string()
    }
    fn default_njav_domain() -> String {
        "njav.org".to_string()
    }
    fn default_javwow_domain() -> String {
        "javwow.com".to_string()
    }
    fn default_avkuy_domain() -> String {
        "www2.avkuy.com".to_string()
    }
    fn default_jav18tv_domain() -> String {
        "18jav.tv".to_string()
    }
}

impl Default for DomainSettings {
    fn default() -> Self {
        Self {
            titan_domain: "51cg1.com".to_string(),
            baanjeen_domain: "xn--82c7abb4jua0l.com".to_string(),
            rongyok_domain: "rongyok.com".to_string(),
            hsck_domain: "hsck123.com".to_string(),
            njavtv_domain: "njavtv.com".to_string(),
            njav_domain: "njav.org".to_string(),
            javwow_domain: "javwow.com".to_string(),
            avkuy_domain: "www2.avkuy.com".to_string(),
            jav18tv_domain: "18jav.tv".to_string(),
        }
    }
}

fn get_settings_path(app_handle: &AppHandle) -> std::path::PathBuf {
    let mut path = app_handle.path().app_data_dir().unwrap_or_default();
    path.push("domain_settings.json");
    path
}

#[tauri::command]
fn get_domain_settings(app_handle: AppHandle) -> DomainSettings {
    let path = get_settings_path(&app_handle);
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(settings) = serde_json::from_str(&content) {
            return settings;
        }
    }
    DomainSettings::default()
}

#[tauri::command]
fn save_domain_settings(settings: DomainSettings, app_handle: AppHandle) -> Result<(), String> {
    let path = get_settings_path(&app_handle);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}


// Unified series info that works with both parsers
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedSeriesInfo {
    pub series_id: i32,
    pub title: String,
    pub total_episodes: i32,
    pub poster_url: Option<String>,
    pub episode_urls: HashMap<i32, String>,
    pub source: String, // "rongyok", "baanjeen", "titan"
    /// Original page URL to use as Referer for hotlink protection bypass
    #[serde(default)]
    pub source_url: Option<String>,
    /// AES-128 key info per episode — only populated for Titan (357ms.com) encrypted HLS
    #[serde(default)]
    pub episode_keys: HashMap<i32, HlsKeyInfo>,
    /// Auth cookies from Chrome detector — required for NjavTV CDN access
    #[serde(default)]
    pub cookies: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub title: String,
    pub poster_url: Option<String>,
    pub url: String,
    pub source: String,
    pub total_episodes: Option<i32>,
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteCategory {
    pub id: String,
    pub label: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub source: String,
    pub page: i32,
    pub has_more: bool,
}

// App state
struct AppState {
    rongyok_parser: RongyokParser,
    baanjeen_parser: BaanJeenParser,
    titan_parser: TitanParser,
    hsck_parser: HsckParser,
    javwow_parser: JavwowParser,
    jav18tv_parser: Jav18tvParser,
    njavtv_parser: NjavtvParser,
    njav_parser: NjavParser,
    chrome_detector: Mutex<ChromeVideoDetector>,
    downloader: Mutex<Option<VideoDownloader>>,
    current_series: Mutex<Option<UnifiedSeriesInfo>>,
    download_states: Mutex<HashMap<i32, Arc<DownloadState>>>,
    library_db: LibraryDb,
    notification_db: NotificationDb,
    queue_db: QueueDb,
    schedule_db: scheduler::ScheduleDb,
    current_library_id: Mutex<Option<i64>>,
    schedule_config: Arc<Mutex<scheduler::ScheduleConfig>>,
    webhook_config: Arc<Mutex<webhook::WebhookConfig>>,
    backup_manager: backup::BackupManager,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DownloadRequest {
    series_id: i32,
    episodes: Vec<i32>,
    output_dir: String,
    auto_merge: bool,
    concurrent_downloads: i32,
    speed_limit: i32,  // KB/s, 0 = unlimited
    file_naming: String, // "ep_001", "episode_1", "title_ep1"
    series_title: String,
    #[serde(default)]
    group_by_source: bool, // สร้าง subfolder ตามชื่อเว็บ
    #[serde(default)]
    preferred_quality: Option<String>,
    #[serde(default)]
    retry_config: Option<crate::proxy::RetryConfig>,
}

// Commands

#[tauri::command]
async fn fetch_series(url: String, app_handle: AppHandle, state: State<'_, AppState>) -> Result<UnifiedSeriesInfo, String> {
    // Check if URL is a direct video file
    if url.contains(".m3u8") || url.contains(".mp4") {
        let mut episode_urls = HashMap::new();
        episode_urls.insert(1, url.clone());

        // Extract filename from URL as title
        let title = url.split('/').last()
            .and_then(|s| s.split('?').next())
            .unwrap_or("Direct Video")
            .to_string();

        let series_info = UnifiedSeriesInfo {
            series_id: 0,
            title,
            total_episodes: 1,
            poster_url: None,
            episode_urls,
            source_url: None,
            episode_keys: Default::default(),
            cookies: Vec::new(),
            source: "direct".to_string(),
        };

        *state.current_series.safe_lock() = Some(series_info.clone());

        // Auto-save to library
        let lib_id = state.library_db.save_series(
            &series_info.title, &series_info.source,
            Some(&url), None, series_info.total_episodes, series_info.series_id,
            &series_info.episode_urls, None, None,
        ).ok();
        *state.current_library_id.safe_lock() = lib_id;

        return Ok(series_info);
    }

    // Check which parser to use based on URL
    let settings = get_domain_settings(app_handle.clone());
    
    let series_info = if BaanJeenParser::is_baanjeen_url(&url, &settings.baanjeen_domain) {
        // Use BaanJeen parser
        let mut baanjeen_info = state.baanjeen_parser.get_series_info(&url, &settings.baanjeen_domain).await?;

        // HYBRID MODE: If no episodes found with static parsing, try Chrome detector
        if baanjeen_info.episode_urls.is_empty() {
            eprintln!("[Hybrid] Static parsing failed, trying Chrome detector on MAIN URL...");
            let _ = app_handle.emit("log-info", "Static parsing failed, activating Chrome detector...".to_string());

            // Acquire lock and run detection
            let mut detector = state.chrome_detector.safe_lock();

            // 1. Try Main URL
            let mut found_url = detector.detect_video_url(&url, Some(&app_handle)).ok().flatten();

            // 2. If not found, try Iframes
            if found_url.is_none() && !baanjeen_info.iframe_urls.is_empty() {
                eprintln!("[Hybrid] Main URL failed, trying {} iframes...", baanjeen_info.iframe_urls.len());
                let _ = app_handle.emit("log-info", format!("Main URL failed, scanning {} iframes...", baanjeen_info.iframe_urls.len()));

                for (i, iframe_url) in baanjeen_info.iframe_urls.iter().enumerate() {
                    let _ = app_handle.emit("log-info", format!("Scanning iframe {}/{}...", i+1, baanjeen_info.iframe_urls.len()));
                    // Ensure iframe URL is absolute
                    let target_url = if iframe_url.starts_with("//") {
                        format!("https:{}", iframe_url)
                    } else if iframe_url.starts_with("/") {
                        format!("https://{}{}", settings.baanjeen_domain, iframe_url)
                    } else {
                        iframe_url.clone()
                    };

                    if let Ok(Some(video_url)) = detector.detect_video_url(&target_url, Some(&app_handle)) {
                        found_url = Some(video_url);
                        break;
                    }
                }
            }

            // Process result
            if let Some(video_url) = found_url {
                eprintln!("[Hybrid] Chrome detector found URL: {}", video_url);
                let _ = app_handle.emit("log-info", format!("Chrome detector found video: {}", video_url));
                baanjeen_info.episode_urls.insert(1, video_url);
                baanjeen_info.total_episodes = 1;
            } else {
                eprintln!("[Hybrid] Chrome detector found no video");
                let _ = app_handle.emit("log-info", "Chrome detector found no video after checking main page and iframes".to_string());
            }
        }

        UnifiedSeriesInfo {
            series_id: 0, // BaanJeen doesn't use numeric IDs
            title: baanjeen_info.title,
            total_episodes: baanjeen_info.total_episodes,
            poster_url: baanjeen_info.poster_url,
            episode_urls: baanjeen_info.episode_urls,
            source_url: None,
            episode_keys: Default::default(),
            cookies: Vec::new(),
            source: "baanjeen".to_string(),
        }
    } else if NjavtvParser::is_njavtv_url(&url, &settings.njavtv_domain) {
        // Use NjavTV Parser (Cloudflare protected — uses Chrome detector)
        let _ = app_handle.emit("log-info", "Detected njavtv.com — using Chrome detector...".to_string());
        let njavtv_info = state.njavtv_parser.get_series_info(&url, &settings.njavtv_domain).await?;

        // Always use Chrome detector since njavtv.com is Cloudflare protected
        let _ = app_handle.emit("log-info", "Launching Chrome to bypass Cloudflare...".to_string());
        let mut detector = state.chrome_detector.safe_lock();

        let mut episode_urls: HashMap<i32, String> = HashMap::new();

        if njavtv_info.total_episodes == 1 {
            // Single episode — detect from the page URL directly
            let page_url = njavtv_info.direct_page_url
                .as_deref()
                .unwrap_or(&url);

            let _ = app_handle.emit("log-info", format!("Detecting video from: {}", page_url));
            match detector.detect_video_url(page_url, Some(&app_handle)) {
                Ok(Some(video_url)) => {
                    let _ = app_handle.emit("log-info", format!("Found video URL: {}", video_url));
                    episode_urls.insert(1, video_url);
                }
                Ok(None) => {
                    let _ = app_handle.emit("log-info", "Chrome detector found no video on njavtv page".to_string());
                    return Err("Could not find video URL on njavtv.com page. The video may require login or is region-blocked.".to_string());
                }
                Err(e) => {
                    return Err(format!("Chrome detection failed: {}", e));
                }
            }
        } else {
            // Multi-episode: detect each episode page
            for (ep, page_url) in &njavtv_info.episode_page_urls {
                let _ = app_handle.emit("log-info", format!("Detecting ep {} from: {}", ep, page_url));
                if let Ok(Some(video_url)) = detector.detect_video_url(page_url, Some(&app_handle)) {
                    episode_urls.insert(*ep, video_url);
                }
            }
            if episode_urls.is_empty() {
                return Err("Could not find any video URLs on njavtv.com".to_string());
            }
        }

        let total = episode_urls.len() as i32;
        let cookies = detector.get_last_cookies().to_vec();
        UnifiedSeriesInfo {
            series_id: 0,
            title: njavtv_info.title,
            total_episodes: total,
            poster_url: njavtv_info.poster_url,
            episode_urls,
            source_url: Some(url.clone()),
            episode_keys: Default::default(),
            cookies,
            source: "njavtv".to_string(),
        }
    } else if NjavParser::is_njav_url(&url, &settings.njav_domain) {
        // Use Njav Parser (njav.org — nested iframes via javxx.com → surrit.store)
        let _ = app_handle.emit("log-info", "Detected njav.org — using Chrome detector...".to_string());
        let njav_info = state.njav_parser.get_series_info(&url, &settings.njav_domain).await?;

        // Always use Chrome detector since video is loaded through nested iframes
        let _ = app_handle.emit("log-info", "Launching Chrome to detect video from njav.org iframes...".to_string());
        let mut detector = state.chrome_detector.safe_lock();

        let page_url = njav_info.direct_page_url
            .as_deref()
            .unwrap_or(&url);

        let _ = app_handle.emit("log-info", format!("Detecting video from: {}", page_url));
        let mut episode_urls: HashMap<i32, String> = HashMap::new();

        match detector.detect_video_url(page_url, Some(&app_handle)) {
            Ok(Some(video_url)) => {
                let _ = app_handle.emit("log-info", format!("Found video URL: {}", video_url));
                episode_urls.insert(1, video_url);
            }
            Ok(None) => {
                let _ = app_handle.emit("log-info", "Chrome detector found no video on njav.org page".to_string());
                return Err("Could not find video URL on njav.org page. The video may be region-blocked.".to_string());
            }
            Err(e) => {
                return Err(format!("Chrome detection failed: {}", e));
            }
        }

        let cookies = detector.get_last_cookies().to_vec();
        UnifiedSeriesInfo {
            series_id: 0,
            title: njav_info.title,
            total_episodes: 1,
            poster_url: njav_info.poster_url,
            episode_urls,
            source_url: Some(url.clone()),
            episode_keys: Default::default(),
            cookies,
            source: "njav".to_string(),
        }
    } else if HsckParser::is_hsck_url(&url, &settings.hsck_domain) {
        // Use HSCK Parser (hsck123.com และ domain สำรอง)
        let hsck_info = state.hsck_parser.get_series_info(&url, &settings.hsck_domain).await?;
        UnifiedSeriesInfo {
            series_id: 0,
            title: hsck_info.title,
            total_episodes: hsck_info.total_episodes,
            poster_url: hsck_info.poster_url,
            episode_urls: hsck_info.episode_urls,
            source_url: None,
            episode_keys: Default::default(),
            cookies: Vec::new(),
            source: "hsck".to_string(),
        }
    } else if JavwowParser::is_javwow_url(&url, &settings.javwow_domain) {
        // JavWow: WebView for Cloudflare bypass + metadata, then packer unpack for stream URL.
        // Chrome detector only used as last resort when WebView approach fails.
        let _ = app_handle.emit("log-info", "Detected javwow — using WebView extractor...".to_string());
        let javwow_info = state.javwow_parser.get_series_info(&url, &settings.javwow_domain).await?;
        let mut detected_title: Option<String> = Some(javwow_info.title.clone()).filter(|t| !t.is_empty());
        let mut detected_poster = javwow_info.poster_url.clone();
        let mut cookies: Vec<(String, String)> = Vec::new();
        let mut episode_urls: HashMap<i32, String> = HashMap::new();
        let mut embed_url = javwow_info.embed_url.clone();

        // Try packer unpack if we already have embed URL from HTTP fetch
        if let Some(ref eu) = embed_url {
            let _ = app_handle.emit("log-info", format!("Found javwow embed URL via HTTP: {}", eu));
            if let Some((stream_url, player_poster)) =
                fetch_avkuy_stream_from_player_page(eu, &url, &cookies).await
            {
                let _ = app_handle.emit("log-info", format!("Found javwow stream: {}", stream_url));
                episode_urls.insert(1, stream_url);
                if detected_poster.is_none() {
                    detected_poster = player_poster;
                }
            }
        }

        // If HTTP failed (Cloudflare blocked), use WebView approach
        if episode_urls.is_empty() {
            let webview_result = fetch_via_webview(&app_handle, &url).await;
            if detected_title.is_none() {
                detected_title = webview_result.title.clone();
            }
            if detected_poster.is_none() {
                detected_poster = webview_result.poster_url.clone();
            }
            cookies = webview_result.cookies.clone();

            if let Some(video_url) = webview_result.video_url {
                let _ = app_handle.emit("log-info", format!("Found javwow video via WebView: {}", video_url));
                episode_urls.insert(1, video_url);
            }

            // Try packer unpack with WebView embed URL and cookies
            if episode_urls.is_empty() {
                if let Some(ref eu) = webview_result.iframe_url {
                    embed_url = Some(eu.clone());
                    if !cookies.is_empty() {
                        let _ = app_handle.emit("log-info", "Trying javwow player HTML extraction (WebView cookies)...".to_string());
                        if let Some((stream_url, player_poster)) =
                            fetch_avkuy_stream_from_player_page(eu, &url, &cookies).await
                        {
                            let _ = app_handle.emit("log-info", format!("Found javwow stream via player HTML: {}", stream_url));
                            episode_urls.insert(1, stream_url);
                            if detected_poster.is_none() {
                                detected_poster = player_poster;
                            }
                        }
                    }
                }
            }
        }

        // Chrome detector as last resort only if WebView approach failed
        if episode_urls.is_empty() {
            let _ = app_handle.emit("log-info", "WebView approach failed, falling back to Chrome detector...".to_string());
            {
                let mut detector = state.chrome_detector.safe_lock();
                match detector.detect_video_url(&url, Some(&app_handle)) {
                    Ok(Some(video_url)) => {
                        let _ = app_handle.emit("log-info", format!("Found video URL: {}", video_url));
                        episode_urls.insert(1, video_url);
                    }
                    Ok(None) => {
                        let _ = app_handle.emit("log-info", "Chrome detector found no video on javwow page".to_string());
                    }
                    Err(e) => {
                        return Err(format!("Chrome detection failed: {}", e));
                    }
                }
                if detected_title.is_none() {
                    detected_title = detector.get_last_title().map(|s| s.to_string());
                }
                if detected_poster.is_none() {
                    detected_poster = detector.get_last_poster_url().map(|s| s.to_string());
                }
                cookies = detector.get_last_cookies().to_vec();
            } // detector lock dropped here

            // Try packer unpack with Chrome cookies
            if episode_urls.is_empty() {
                if let Some(ref eu) = embed_url {
                    let _ = app_handle.emit("log-info", "Trying javwow player HTML extraction (Chrome cookies)...".to_string());
                    if let Some((stream_url, player_poster)) =
                        fetch_avkuy_stream_from_player_page(eu, &url, &cookies).await
                    {
                        episode_urls.insert(1, stream_url);
                        if detected_poster.is_none() {
                            detected_poster = player_poster;
                        }
                    }
                }
            }
        }

        if episode_urls.is_empty() {
            return Err("Could not find video URL on javwow.com page. The video may be region-blocked or require login.".to_string());
        }

        UnifiedSeriesInfo {
            series_id: 0,
            title: detected_title.unwrap_or_else(|| derive_title_from_url(&url, "JavWow Video")),
            total_episodes: 1,
            poster_url: detected_poster,
            episode_urls,
            source_url: Some(url.clone()),
            episode_keys: Default::default(),
            cookies,
            source: "javwow".to_string(),
        }
    } else if Jav18tvParser::is_jav18tv_url(&url, &settings.jav18tv_domain) {
        // 18jav.tv: WebView for Cloudflare bypass + metadata extraction.
        let _ = app_handle.emit("log-info", "Detected 18jav.tv - using WebView extractor...".to_string());
        let jav18tv_info = state.jav18tv_parser.get_series_info(&url, &settings.jav18tv_domain).await?;
        let mut detected_title: Option<String> = Some(jav18tv_info.title.clone()).filter(|t| !t.is_empty());
        let mut detected_poster = jav18tv_info.poster_url.clone();
        let mut cookies: Vec<(String, String)> = Vec::new();
        let mut episode_urls: HashMap<i32, String> = HashMap::new();

        // If we got a direct video URL from HTML parsing, use it
        if let Some(ref video_url) = jav18tv_info.direct_video_url {
            let _ = app_handle.emit("log-info", format!("Found 18jav direct video URL: {}", video_url));
            episode_urls.insert(1, video_url.clone());
        }

        // If we got an embed URL, try to extract stream from it
        if episode_urls.is_empty() {
            if let Some(ref embed) = jav18tv_info.embed_url {
                let _ = app_handle.emit("log-info", format!("Found 18jav embed URL: {}", embed));
                if let Some((stream_url, player_poster)) =
                    fetch_avkuy_stream_from_player_page(embed, &url, &cookies).await
                {
                    let _ = app_handle.emit("log-info", format!("Found 18jav stream via embed: {}", stream_url));
                    episode_urls.insert(1, stream_url);
                    if detected_poster.is_none() {
                        detected_poster = player_poster;
                    }
                }
            }
        }

        // If HTTP failed (Cloudflare), use WebView approach
        if episode_urls.is_empty() {
            let webview_result = fetch_via_webview(&app_handle, &url).await;
            if detected_title.is_none() {
                detected_title = webview_result.title.clone();
            }
            if detected_poster.is_none() {
                detected_poster = webview_result.poster_url.clone();
            }
            cookies = webview_result.cookies.clone();

            if let Some(video_url) = webview_result.video_url {
                let _ = app_handle.emit("log-info", format!("Found 18jav video via WebView: {}", video_url));
                episode_urls.insert(1, video_url);
            }

            // Try embed URL with WebView cookies
            if episode_urls.is_empty() {
                if let Some(ref eu) = webview_result.iframe_url {
                    if !cookies.is_empty() {
                        let _ = app_handle.emit("log-info", "Trying 18jav player HTML extraction (WebView cookies)...".to_string());
                        if let Some((stream_url, player_poster)) =
                            fetch_avkuy_stream_from_player_page(eu, &url, &cookies).await
                        {
                            episode_urls.insert(1, stream_url);
                            if detected_poster.is_none() {
                                detected_poster = player_poster;
                            }
                        }
                    }
                }
            }
        }

        // Chrome detector as last resort
        if episode_urls.is_empty() {
            let _ = app_handle.emit("log-info", "Falling back to Chrome detector for 18jav...".to_string());
            {
                let mut detector = state.chrome_detector.safe_lock();
                match detector.detect_video_url(&url, Some(&app_handle)) {
                    Ok(Some(video_url)) => {
                        let _ = app_handle.emit("log-info", format!("Found video URL: {}", video_url));
                        episode_urls.insert(1, video_url);
                    }
                    Ok(None) => {
                        let _ = app_handle.emit("log-info", "Chrome detector found no video on 18jav page".to_string());
                    }
                    Err(e) => {
                        return Err(format!("Chrome detection failed: {}", e));
                    }
                }
                if detected_title.is_none() {
                    detected_title = detector.get_last_title().map(|s| s.to_string());
                }
                if detected_poster.is_none() {
                    detected_poster = detector.get_last_poster_url().map(|s| s.to_string());
                }
                cookies = detector.get_last_cookies().to_vec();
            }
        }

        if episode_urls.is_empty() {
            return Err("Could not find video URL on 18jav.tv page. The video may be region-blocked or require login.".to_string());
        }

        UnifiedSeriesInfo {
            series_id: 0,
            title: detected_title.unwrap_or_else(|| derive_title_from_url(&url, "18JAV Video")),
            total_episodes: 1,
            poster_url: detected_poster,
            episode_urls,
            source_url: Some(url.clone()),
            episode_keys: Default::default(),
            cookies,
            source: "jav18tv".to_string(),
        }
    } else if settings
        .avkuy_domain
        .split(',')
        .map(|d| d.trim())
        .any(|d| !d.is_empty() && url.contains(d))
        || url.contains("avkuy.com")
        || url.contains("av-kuy.com")
    {
        // AVKuy: WebView for Cloudflare bypass + metadata, then HTTP packer unpack.
        // Chrome detector only used as last resort when WebView cookies are insufficient.
        let _ = app_handle.emit("log-info", "Detected avkuy — using WebView extractor...".to_string());
        let webview_result = fetch_via_webview(&app_handle, &url).await;
        let mut episode_urls: HashMap<i32, String> = HashMap::new();
        let mut detected_title = webview_result.title.clone();
        let mut detected_poster = webview_result.poster_url.clone();
        let mut cookies = webview_result.cookies.clone();
        let iframe_url = webview_result.iframe_url.clone();

        if let Some(video_url) = webview_result.video_url {
            let _ = app_handle.emit("log-info", format!("Found AVKuy video via WebView: {}", video_url));
            episode_urls.insert(1, video_url);
        }

        // Try packer unpack with WebView cookies first (avoids heavy Chrome)
        if episode_urls.is_empty() {
            if let Some(player_url) = iframe_url.as_deref() {
                if !cookies.is_empty() {
                    let _ = app_handle.emit("log-info", "Trying AVKuy player HTML extraction (WebView cookies)...".to_string());
                    if let Some((stream_url, player_poster)) =
                        fetch_avkuy_stream_from_player_page(player_url, &url, &cookies).await
                    {
                        let _ = app_handle.emit("log-info", format!("Found AVKuy stream via player HTML: {}", stream_url));
                        episode_urls.insert(1, stream_url);
                        if detected_poster.is_none() {
                            detected_poster = player_poster;
                        }
                    }
                }
            }
        }

        // Chrome detector as last resort only if WebView approach failed
        if episode_urls.is_empty() {
            let _ = app_handle.emit("log-info", "WebView cookies not enough, falling back to Chrome detector...".to_string());
            // Scope Chrome detector lock to avoid holding MutexGuard across await
            {
                let mut detector = state.chrome_detector.safe_lock();

                let detect_target = iframe_url.as_deref().unwrap_or(&url);
                let _ = app_handle.emit("log-info", format!("Detecting video from: {}", detect_target));
                match detector.detect_video_url(detect_target, Some(&app_handle)) {
                    Ok(Some(video_url)) => {
                        let _ = app_handle.emit("log-info", format!("Found video URL: {}", video_url));
                        episode_urls.insert(1, video_url);
                    }
                    Ok(None) => {
                        let _ = app_handle.emit("log-info", "Chrome detector found no video on AVKuy player page".to_string());
                    }
                    Err(e) => {
                        return Err(format!("Chrome detection failed: {}", e));
                    }
                }

                if detected_title.is_none() {
                    detected_title = detector.get_last_title().map(|s| s.to_string());
                }
                if detected_poster.is_none() {
                    detected_poster = detector.get_last_poster_url().map(|s| s.to_string());
                }
                cookies = detector.get_last_cookies().to_vec();
            } // detector lock dropped here

            // Try packer unpack with Chrome cookies (no MutexGuard held)
            if episode_urls.is_empty() {
                if let Some(player_url) = iframe_url.as_deref() {
                    let _ = app_handle.emit("log-info", "Trying AVKuy player HTML extraction (Chrome cookies)...".to_string());
                    if let Some((stream_url, player_poster)) =
                        fetch_avkuy_stream_from_player_page(player_url, &url, &cookies).await
                    {
                        let _ = app_handle.emit("log-info", format!("Found AVKuy stream via player HTML: {}", stream_url));
                        episode_urls.insert(1, stream_url);
                        if detected_poster.is_none() {
                            detected_poster = player_poster;
                        }
                    }
                }
            }
        }

        if episode_urls.is_empty() {
            return Err("Could not find video URL on avkuy page. The video may be region-blocked or require login.".to_string());
        }

        UnifiedSeriesInfo {
            series_id: 0,
            title: detected_title.unwrap_or_else(|| derive_title_from_url(&url, "AVKUY Video")),
            total_episodes: 1,
            poster_url: detected_poster,
            episode_urls,
            source_url: Some(url.clone()),
            episode_keys: Default::default(),
            cookies,
            source: "avkuy".to_string(),
        }
    } else if TitanParser::is_titan_url(&url, &settings.titan_domain) {
        // Use Titan Parser
        let mut titan_info = state.titan_parser.get_series_info(&url, &settings.titan_domain).await?;
        
        // If Titan found 0 episodes on an archive page, try Playwright dynamic detection
        if titan_info.total_episodes == 0 && TitanParser::is_archive_url(&url) {
            eprintln!("[Titan] No episodes found via static analysis, trying Playwright detection...");
            
            // Use Chrome detector to find video URL (lock is released immediately after)
            let detected_url = {
                let mut detector = state.chrome_detector.safe_lock();
                detector.detect_video_url(&url, Some(&app_handle))?
            }; // Lock released here
            
            if let Some(video_url) = detected_url {
                eprintln!("[Titan+Playwright] Found video URL: {}", video_url);
                
                // Create episode map with detected URL
                let mut episode_urls = HashMap::new();
                episode_urls.insert(1, video_url.clone());
                
                titan_info = TitanSeriesInfo {
                    url: url.clone(),
                    title: if titan_info.title.is_empty() || titan_info.title == "Unknown Video" {
                        "Archive Video".to_string()
                    } else {
                        titan_info.title
                    },
                    total_episodes: 1,
                    poster_url: titan_info.poster_url,
                    episode_urls,
                    episode_keys: HashMap::new(),
                };
            } else {
                eprintln!("[Titan+Playwright] No video URL detected");
            }
        }
        
        UnifiedSeriesInfo {
            series_id: 0,
            title: titan_info.title,
            total_episodes: titan_info.total_episodes,
            poster_url: titan_info.poster_url,
            episode_urls: titan_info.episode_urls,
            source_url: None,
            episode_keys: titan_info.episode_keys,
            cookies: Vec::new(),
            source: "titan".to_string(),
        }
    } else {
        // Use Rongyok parser (also passing domain for parsing checks if needed)
        let series_id = RongyokParser::parse_series_url(&url, &settings.rongyok_domain).ok_or("Invalid URL format")?;
        let rongyok_info = state.rongyok_parser.get_series_info(series_id, Some(&url), &settings.rongyok_domain).await?;
        UnifiedSeriesInfo {
            series_id: rongyok_info.series_id,
            title: rongyok_info.title,
            total_episodes: rongyok_info.total_episodes,
            poster_url: rongyok_info.poster_url,
            episode_urls: rongyok_info.episode_urls,
            source_url: None,
            episode_keys: Default::default(),
            cookies: Vec::new(),
            source: "rongyok".to_string(),
        }
    };

    let series_info = series_info;

    // Store in state
    *state.current_series.safe_lock() = Some(series_info.clone());

    // Auto-save to library (with poster extracted from data URL)
    let poster_data = series_info.poster_url.as_ref().and_then(|u| decode_data_url(u));
    let lib_id = state.library_db.save_series(
        &series_info.title, &series_info.source,
        Some(&url), poster_data.as_deref(), series_info.total_episodes, series_info.series_id,
        &series_info.episode_urls, None, None,
    ).ok();
    *state.current_library_id.safe_lock() = lib_id;

    Ok(series_info)
}

#[tauri::command]
fn check_ffmpeg_available() -> bool {
    check_ffmpeg()
}

#[tauri::command]
fn set_taskbar_progress(app_handle: AppHandle, progress: i32) {
    use tauri::Manager;
    if let Some(_window) = app_handle.get_webview_window("main") {
        // TODO: Implement Taskbar Progress for Tauri v2
        // API changed: window.set_progress_bar requires specific struct
        // For now, logging to console
        if progress >= 0 && progress <= 100 {
            // println!("Taskbar Progress: {}%", progress);
        }
    }
}

#[tauri::command]
async fn auto_detect_video_url(url: String, app_handle: AppHandle, state: State<'_, AppState>) -> Result<Option<String>, String> {
    eprintln!("[AutoDetect] Starting auto-detection for: {}", url);

    let settings = get_domain_settings(app_handle.clone());
    let is_avkuy = settings
        .avkuy_domain
        .split(',')
        .map(|d| d.trim())
        .any(|d| !d.is_empty() && url.contains(d))
        || url.contains("avkuy.com")
        || url.contains("av-kuy.com");

    if is_avkuy {
        let detected = fetch_via_webview(&app_handle, &url).await;
        if let Some(video_url) = detected.video_url {
            return Ok(Some(video_url));
        }
    }

    let mut detector = state.chrome_detector.safe_lock();
    let video_url = detector.detect_video_url(&url, Some(&app_handle))?;

    Ok(video_url)
}

#[tauri::command]
async fn start_download(
    request: DownloadRequest,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<DownloadResult>, String> {
    let series = state
        .current_series
        .lock()
        .unwrap()
        .clone()
        .ok_or("No series loaded")?;
    let _ = app_handle.emit("log-info", format!("start_download called: {} episodes for {}", request.episodes.len(), series.title));

    // คำนวณ output dir จริง (เพิ่ม subfolder ตาม source ถ้า group_by_source=true)
    let effective_output_dir = if request.group_by_source {
        let source_folder = match series.source.as_str() {
            "baanjeen" => "baanjeen",
            "hsck" => "hsck",
            "njavtv" => "njavtv",
            "njav" => "njav",
            "javwow" => "javwow",
            "jav18tv" => "jav18tv",
            "avkuy" => "avkuy",
            "titan" => "titan",
            "direct" => "direct",
            _ => "rongyok", // default
        };
        format!("{}/{}", request.output_dir.trim_end_matches('/'), source_folder)
    } else {
        request.output_dir.clone()
    };

    // Create downloader with config
    let config = DownloadConfig {
        speed_limit_kbps: request.speed_limit,
        file_naming: request.file_naming.clone(),
        series_title: request.series_title.clone(),
    };
    let _downloader = VideoDownloader::with_config(&effective_output_dir, config.clone());
    *state.downloader.safe_lock() = Some(VideoDownloader::with_config(&effective_output_dir, config.clone()));

    let mut results = Vec::new();
    let mut successful_files = Vec::new();

    // Shared downloader instance (avoid creating new HTTP client per episode)
    let shared_dl = Arc::new(VideoDownloader::with_config(&effective_output_dir, config));

    // Concurrent downloads using chunks
    let concurrent = request.concurrent_downloads.max(1) as usize;

    for chunk in request.episodes.chunks(concurrent) {
        let mut handles = Vec::new();

        for episode in chunk {
            let video_url = series
                .episode_urls
                .get(episode)
                .ok_or(format!("No URL for episode {}", episode))?
                .clone();

            let hls_key_info = series.episode_keys.get(episode).cloned();
            let referer = series.source_url.clone();
            let cookies = series.cookies.clone();
            let app = app_handle.clone();
            let dl = Arc::clone(&shared_dl);
            let ep = *episode;
            let preferred_quality = request.preferred_quality.clone();

            // Create download state for this episode
            let download_state = Arc::new(DownloadState::new());
            {
                let mut states = state.download_states.safe_lock();
                states.insert(ep, download_state.clone());
            }

            let handle = tokio::spawn(async move {
                dl.download_episode(ep, &video_url, hls_key_info, referer.as_deref(), &cookies, &app, Some(download_state), preferred_quality).await
            });
            handles.push((ep, handle));
        }
        // Wait for all in this chunk to complete
        for (ep, handle) in handles {
            match handle.await {
                Ok(result) => {
                    // Remove from download states when done
                    {
                        let mut states = state.download_states.safe_lock();
                        states.remove(&ep);
                    }

                    if result.success {
                        if let Some(ref path) = result.file_path {
                            successful_files.push(path.clone());
                        }
                    }
                    let _ = app_handle.emit("download-result", &result);

                    // Update library episode status
                    if let Some(lib_id) = *state.current_library_id.safe_lock() {
                        state.library_db.update_episode_status(
                            lib_id, ep, if result.success { "completed" } else { "failed" },
                            result.file_path.as_deref(),
                        ).ok();

                        // Send webhook notification
                        let webhook_config = state.webhook_config.safe_lock().clone();
                        if webhook_config.enabled {
                            let event_type = if result.success { "download_complete" } else { "download_failed" };
                            let title = if result.success {
                                format!("Download Complete: Episode {}", ep)
                            } else {
                                format!("Download Failed: Episode {}", ep)
                            };
                            let message = if result.success {
                                format!("Episode {} of {} downloaded successfully", ep, series.title)
                            } else {
                                format!("Episode {} of {} failed: {}", ep, series.title, result.error.as_ref().map(|s| s.as_str()).unwrap_or("Unknown error"))
                            };

                            tauri::async_runtime::spawn(async move {
                                let _ = webhook::send_webhook(&webhook_config, event_type, &title, &message).await;
                            });
                        }
                    }

                    results.push(result);
                }
                Err(e) => {
                    // Remove from download states on error
                    {
                        let mut states = state.download_states.safe_lock();
                        states.remove(&ep);
                    }
                    let result = DownloadResult {
                        episode: ep,
                        success: false,
                        file_path: None,
                        error: Some(format!("Task failed: {}", e)),
                    };
                    results.push(result);
                }
            }
        }
    }

    // Return download results immediately so the next batch item can start.
    // Merge runs as a background task so downloads are not blocked.
    let files_count = successful_files.len();
    let should_merge = request.auto_merge && files_count > 0 && check_ffmpeg();

    if should_merge {
        let files_to_merge = successful_files.clone();
        let series_title = series.title.clone();
        let merge_app_handle = app_handle.clone();
        let output_dir = effective_output_dir.clone();

        // Emit merge-started right away so the UI shows the merge section
        let _ = merge_app_handle.emit("merge-started", ());

        tokio::spawn(async move {
            let output_filename = sanitize_filename(&series_title);
            let expanded_output_dir = expand_path(&output_dir);
            let output_path = expanded_output_dir.join(format!("{}.mp4", output_filename));
            let output_path_str = output_path.to_string_lossy().to_string();

            let _ = merge_app_handle.emit("log-info", format!("Background merge: {} files -> {}", files_to_merge.len(), output_path_str));

            if files_to_merge.len() == 1 {
                let source = std::path::PathBuf::from(&files_to_merge[0]);
                match std::fs::rename(&source, &output_path) {
                    Ok(_) => {
                        let _ = merge_app_handle.emit("merge-complete", output_path_str);
                    }
                    Err(e) => {
                        match std::fs::copy(&source, &output_path) {
                            Ok(_) => {
                                std::fs::remove_file(&source).ok();
                                let _ = merge_app_handle.emit("merge-complete", output_path_str);
                            }
                            Err(e2) => {
                                let _ = merge_app_handle.emit("merge-error", format!("Failed rename/copy: {} / {}", e, e2));
                            }
                        }
                    }
                }
            } else {
                let mut sorted_files = files_to_merge.clone();
                sorted_files.sort();
                let files_for_cleanup = sorted_files.clone();
                let path_for_emit = output_path_str.clone();
                let handle_for_emit = merge_app_handle.clone();
                // merge_videos_with_progress is sync/CPU-bound; run on blocking thread
                let merge_result = tokio::task::spawn_blocking(move || {
                    downloader::merge_videos_with_progress(sorted_files, &output_path_str, Some(&merge_app_handle))
                }).await;
                match merge_result {
                    Ok(Ok(_)) => {
                        for file in &files_for_cleanup {
                            std::fs::remove_file(file).ok();
                        }
                        let _ = handle_for_emit.emit("merge-complete", path_for_emit);
                    }
                    Ok(Err(e)) => {
                        let _ = handle_for_emit.emit("merge-error", e);
                    }
                    Err(e) => {
                        let _ = handle_for_emit.emit("merge-error", format!("Merge task panicked: {}", e));
                    }
                }
            }
        });
    }

    Ok(results)
}

#[tauri::command]
async fn pause_download(episode: i32, state: State<'_, AppState>) -> Result<(), String> {
    let states = state.download_states.safe_lock();
    if let Some(download_state) = states.get(&episode) {
        download_state.is_paused.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    } else {
        Err(format!("No active download for episode {}", episode))
    }
}

#[tauri::command]
async fn resume_download(episode: i32, state: State<'_, AppState>) -> Result<(), String> {
    let states = state.download_states.safe_lock();
    if let Some(download_state) = states.get(&episode) {
        download_state.is_paused.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    } else {
        Err(format!("No active download for episode {}", episode))
    }
}

#[tauri::command]
async fn cancel_download(episode: i32, state: State<'_, AppState>) -> Result<(), String> {
    let states = state.download_states.safe_lock();
    if let Some(download_state) = states.get(&episode) {
        download_state.is_cancelled.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    } else {
        Err(format!("No active download for episode {}", episode))
    }
}

#[tauri::command]
async fn get_episode_url(
    series_id: i32,
    episode: i32,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<String, String> {
    // Try from cached series first
    if let Some(series) = state.current_series.safe_lock().as_ref() {
        if series.series_id == series_id {
            if let Some(url) = series.episode_urls.get(&episode) {
                return Ok(url.clone());
            }
        }
    }

    // Fetch fresh
    let settings = get_domain_settings(app_handle.clone());
    let series_info = state.rongyok_parser.get_series_info(series_id, None, &settings.rongyok_domain).await?;
    series_info
        .episode_urls
        .get(&episode)
        .cloned()
        .ok_or(format!("No URL for episode {}", episode))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSeriesResponse {
    success: bool,
}

#[tauri::command]
async fn update_series_state(
    series: UnifiedSeriesInfo,
    state: State<'_, AppState>,
) -> Result<UpdateSeriesResponse, String> {
    *state.current_series.safe_lock() = Some(series);
    Ok(UpdateSeriesResponse { success: true })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileInfo {
    name: String,
    path: String,
    size: u64,
    is_episode: bool,
    is_merged: bool,
}

#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    let expanded_path = expand_path(&path);
    let path_str = expanded_path.to_string_lossy().to_string();

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path_str)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn list_files(dir: String) -> Result<Vec<FileInfo>, String> {
    let expanded_path = expand_path(&dir);
    if !expanded_path.exists() {
        // Try to create the directory
        std::fs::create_dir_all(&expanded_path).ok();
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let entries = std::fs::read_dir(&expanded_path).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_path = entry.path();

        if file_path.is_file() {
            if let Some(ext) = file_path.extension() {
                if ext == "mp4" || ext == "ts" || ext == "mkv" {
                    let metadata = std::fs::metadata(&file_path).map_err(|e| e.to_string())?;
                    let name = file_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("")
                        .to_string();

                    // Check if it's an episode file (ep_XXX pattern) or merged file
                    let is_episode = name.starts_with("ep_") || name.contains("_ep");
                    let is_merged = !is_episode && name.ends_with(".mp4");

                    files.push(FileInfo {
                        name,
                        path: file_path.to_string_lossy().to_string(),
                        size: metadata.len(),
                        is_episode,
                        is_merged,
                    });
                }
            }
        }
    }

    // Sort by name
    files.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(files)
}

#[tauri::command]
async fn delete_files(paths: Vec<String>) -> Result<u32, String> {
    let mut deleted = 0;
    for path in paths {
        if std::fs::remove_file(&path).is_ok() {
            deleted += 1;
        }
    }
    Ok(deleted)
}

#[tauri::command]
async fn play_file(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn cmd_save_to_library(
    state: State<'_, AppState>,
    title: String, source: String, source_url: Option<String>,
    poster_url: Option<String>, total_episodes: i32, parser_series_id: i32,
    episode_urls: HashMap<i32, String>, metadata: Option<String>,
) -> Result<i64, String> {
    // Download poster if base64 data URL
    let poster_data = if let Some(ref url) = poster_url {
        if url.starts_with("data:") {
            url.split(',').nth(1)
                .and_then(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
        } else {
            None
        }
    } else {
        None
    };

    state.library_db.save_series(
        &title, &source, source_url.as_deref(),
        poster_data.as_deref(), total_episodes, parser_series_id,
        &episode_urls, metadata.as_deref(), None,
    )
}

#[tauri::command]
fn cmd_get_library(state: State<'_, AppState>, query: Option<library::LibraryQuery>) -> Result<Vec<library::LibraryEntry>, String> {
    state.library_db.get_library(query)
}

#[tauri::command]
fn cmd_get_series_detail(state: State<'_, AppState>, library_id: i64) -> Result<library::SeriesDetail, String> {
    state.library_db.get_series_detail(library_id)
}

#[tauri::command]
fn cmd_remove_from_library(state: State<'_, AppState>, library_id: i64) -> Result<(), String> {
    state.library_db.remove_series(library_id)
}

#[tauri::command]
fn cmd_update_episode_status(
    state: State<'_, AppState>, library_id: i64, episode_number: i32,
    status: String, file_path: Option<String>,
) -> Result<(), String> {
    state.library_db.update_episode_status(library_id, episode_number, &status, file_path.as_deref())
}

// DEPRECATED: use cmd_get_library(Some(LibraryQuery { search: Some(query), ..Default::default() }))
#[tauri::command]
fn cmd_search_library(state: State<'_, AppState>, query: String) -> Result<Vec<library::LibraryEntry>, String> {
    state.library_db.search_library(&query)
}

#[tauri::command]
fn cmd_get_tags(state: State<'_, AppState>) -> Result<Vec<library::LibraryTag>, String> {
    state.library_db.get_tags()
}

#[tauri::command]
fn cmd_create_tag(state: State<'_, AppState>, name: String) -> Result<i64, String> {
    state.library_db.create_tag(&name)
}

#[tauri::command]
fn cmd_delete_tag(state: State<'_, AppState>, tag_id: i64) -> Result<(), String> {
    state.library_db.delete_tag(tag_id)
}

#[tauri::command]
fn cmd_assign_tag(state: State<'_, AppState>, library_id: i64, tag_id: i64) -> Result<(), String> {
    state.library_db.assign_tag(library_id, tag_id)
}

#[tauri::command]
fn cmd_unassign_tag(state: State<'_, AppState>, library_id: i64, tag_id: i64) -> Result<(), String> {
    state.library_db.unassign_tag(library_id, tag_id)
}

#[tauri::command]
fn cmd_toggle_favorite(state: State<'_, AppState>, library_id: i64) -> Result<bool, String> {
    state.library_db.toggle_favorite(library_id)
}

#[tauri::command]
fn cmd_open_episode(state: State<'_, AppState>, library_id: i64, episode_number: i32) -> Result<(), String> {
    let path = state.library_db.get_episode_file_path(library_id, episode_number)?;
    let path = path.ok_or("Episode file not found")?;

    #[cfg(target_os = "macos")]
    { std::process::Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?; }
    #[cfg(target_os = "windows")]
    { std::process::Command::new("cmd").args(["/c", "start", "", &path]).spawn().map_err(|e| e.to_string())?; }
    #[cfg(target_os = "linux")]
    { std::process::Command::new("xdg-open").arg(&path).spawn().map_err(|e| e.to_string())?; }
    Ok(())
}

#[tauri::command]
async fn cmd_refetch_series(
    library_id: i64,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<library::SeriesDetail, String> {
    let detail = state.library_db.get_series_detail(library_id)?;
    let source_url = detail.entry.source_url.clone()
        .ok_or_else(|| "No source URL stored for this series".to_string())?;

    if !detail.can_refetch {
        return Err("This series cannot be re-fetched (direct video URL)".to_string());
    }

    // Re-invoke fetch_series with stored URL
    let new_info = fetch_series(source_url, app_handle, state.clone()).await?;

    // Update library with fresh data (including poster)
    let poster_data = new_info.poster_url.as_ref().and_then(|u| decode_data_url(u));
    state.library_db.save_series(
        &new_info.title, &new_info.source, Some(&new_info.source_url.unwrap_or_default()),
        poster_data.as_deref(), new_info.total_episodes, new_info.series_id,
        &new_info.episode_urls, None, None,
    ).ok();

    state.library_db.get_series_detail(library_id)
}

#[tauri::command]
fn cmd_get_proxy_config(state: State<'_, AppState>) -> Result<proxy::ProxyConfig, String> {
    let config = state.rongyok_parser.proxy_config.read()
        .unwrap_or_else(|e| {
            eprintln!("Warning: RwLock poisoned, recovering...");
            e.into_inner()
        });
    Ok(config.clone())
}

#[tauri::command]
fn cmd_save_proxy_config(state: State<'_, AppState>, config: proxy::ProxyConfig) -> Result<(), String> {
    let mut pc = state.rongyok_parser.proxy_config.write()
        .unwrap_or_else(|e| {
            eprintln!("Warning: RwLock poisoned, recovering...");
            e.into_inner()
        });
    *pc = config;
    Ok(())
}

#[tauri::command]
async fn cmd_test_proxy_connection(config: proxy::ProxyConfig) -> Result<bool, String> {
    let client = proxy::build_client(&config);
    match client.get("https://www.google.com").send().await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(e) => Err(format!("Connection failed: {}", e)),
    }
}

#[tauri::command]
fn cmd_get_schedule_config(state: State<'_, AppState>) -> Result<scheduler::ScheduleConfig, String> {
    let cfg = state.schedule_config.safe_lock();
    Ok(cfg.clone())
}

#[tauri::command]
fn cmd_save_schedule_config(state: State<'_, AppState>, config: scheduler::ScheduleConfig) -> Result<(), String> {
    let mut cfg = state.schedule_config.safe_lock();
    *cfg = config;
    Ok(())
}

#[tauri::command]
async fn get_quality_options(url: String) -> Result<crate::downloader::QualityInfo, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0")
        .build().map_err(|e| e.to_string())?;

    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let text = resp.text().await.map_err(|e| e.to_string())?;

    if text.contains("#EXT-X-STREAM-INF") {
        let variants = crate::downloader::parse_master_playlist(&text, &url);
        if variants.is_empty() {
            return Ok(crate::downloader::QualityInfo { qualities: vec![], default_index: 0 });
        }
        Ok(crate::downloader::QualityInfo {
            default_index: 0,
            qualities: variants,
        })
    } else {
        Ok(crate::downloader::QualityInfo {
            qualities: vec![crate::downloader::QualityOption {
                resolution: "original".to_string(),
                bandwidth: 0,
                label: "Original quality".to_string(),
                stream_url: url,
            }],
            default_index: 0,
        })
    }
}

#[tauri::command]
async fn search_sites(
    query: String,
    page: i32,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResponse>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let settings = get_domain_settings(app_handle);
    let futures: Vec<std::pin::Pin<Box<dyn std::future::Future<Output = Option<SearchResponse>> + Send>>> = vec![
        Box::pin(async {
            match state.rongyok_parser.search(&query, page, &settings.rongyok_domain).await {
                Ok(results) => {
                    let has_more = results.len() >= 20;
                    Some(SearchResponse { results, source: "rongyok".into(), page, has_more })
                }
                Err(_) => None,
            }
        }),
        Box::pin(async {
            match state.baanjeen_parser.search(&query, page, &settings.baanjeen_domain).await {
                Ok(results) => {
                    let has_more = results.len() >= 20;
                    Some(SearchResponse { results, source: "baanjeen".into(), page, has_more })
                }
                Err(_) => None,
            }
        }),
        Box::pin(async {
            match state.titan_parser.search(&query, page, &settings.titan_domain).await {
                Ok(results) => {
                    let has_more = results.len() >= 20;
                    Some(SearchResponse { results, source: "titan".into(), page, has_more })
                }
                Err(_) => None,
            }
        }),
        Box::pin(async {
            match state.hsck_parser.search(&query, page, &settings.hsck_domain).await {
                Ok(results) => {
                    let has_more = results.len() >= 20;
                    Some(SearchResponse { results, source: "hsck".into(), page, has_more })
                }
                Err(_) => None,
            }
        }),
        Box::pin(async {
            match state.javwow_parser.search(&query, page, &settings.javwow_domain).await {
                Ok(results) => {
                    let has_more = results.len() >= 20;
                    Some(SearchResponse { results, source: "javwow".into(), page, has_more })
                }
                Err(_) => None,
            }
        }),
        Box::pin(async {
            match state.jav18tv_parser.search(&query, page, &settings.jav18tv_domain).await {
                Ok(results) => {
                    let has_more = results.len() >= 20;
                    Some(SearchResponse { results, source: "jav18tv".into(), page, has_more })
                }
                Err(_) => None,
            }
        }),
    ];

    let responses: Vec<SearchResponse> = futures_util::future::join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect();

    Ok(responses)
}

#[tauri::command]
fn get_browse_categories(state: State<'_, AppState>) -> Result<Vec<SiteCategory>, String> {
    let mut categories = Vec::new();
    categories.extend(state.rongyok_parser.list_categories("rongyok.com"));
    categories.extend(state.baanjeen_parser.list_categories("xn--82c7abb4jua0l.com"));
    categories.extend(state.titan_parser.list_categories("51cg1.com"));
    categories.extend(state.hsck_parser.list_categories("hsck123.com"));
    categories.extend(state.javwow_parser.list_categories("javwow.com"));
    categories.extend(state.jav18tv_parser.list_categories("18jav.tv"));
    Ok(categories)
}

#[tauri::command]
async fn browse_category(
    source: String,
    category: String,
    page: i32,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<SearchResponse, String> {
    let settings = get_domain_settings(app_handle);
    let results = match source.as_str() {
        "rongyok" => state.rongyok_parser.browse(&category, page, &settings.rongyok_domain).await?,
        "baanjeen" => state.baanjeen_parser.browse(&category, page, &settings.baanjeen_domain).await?,
        "titan" => state.titan_parser.browse(&category, page, &settings.titan_domain).await?,
        "hsck" => state.hsck_parser.browse(&category, page, &settings.hsck_domain).await?,
        "javwow" => state.javwow_parser.browse(&category, page, &settings.javwow_domain).await?,
        "jav18tv" => state.jav18tv_parser.browse(&category, page, &settings.jav18tv_domain).await?,
        _ => return Err(format!("Unknown source: {}", source)),
    };
    let has_more = results.len() >= 20;
    Ok(SearchResponse { results, source, page, has_more })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Set TMPDIR to app cache directory to avoid cross-device link errors during updates
    #[cfg(target_os = "macos")]
    {
        if let Some(cache_dir) = dirs::cache_dir() {
            let app_tmp = cache_dir.join("com.rongyok.downloader").join("tmp");
            std::fs::create_dir_all(&app_tmp).ok();
            std::env::set_var("TMPDIR", &app_tmp);
        }
    }

    // Initialize app data directory (computed once, used by all subsystems)
    let app_data_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("com.rongyok.downloader");
    std::fs::create_dir_all(&app_data_dir)
        .expect("Failed to create application data directory");

    let library_db = LibraryDb::new(&app_data_dir)
        .expect("Failed to initialize library database");

    let notification_db = NotificationDb::new(&app_data_dir)
        .expect("Failed to initialize notification database");

    let queue_db = QueueDb::new(&app_data_dir)
        .expect("Failed to initialize queue database");

    let proxy_config = Arc::new(RwLock::new(proxy::ProxyConfig::default()));
    let schedule_config = Arc::new(Mutex::new(scheduler::ScheduleConfig::default()));
    let webhook_config = Arc::new(Mutex::new(webhook::WebhookConfig::default()));

    let schedule_db = scheduler::ScheduleDb::new(&app_data_dir)
        .expect("Failed to initialize schedule database");

    let backup_mgr = backup::BackupManager::new(&app_data_dir);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState {
            rongyok_parser: RongyokParser::new(proxy_config.clone()),
            baanjeen_parser: BaanJeenParser::new(proxy_config.clone()),
            titan_parser: TitanParser::new(proxy_config.clone()),
            hsck_parser: HsckParser::new(proxy_config.clone()),
            javwow_parser: JavwowParser::new(proxy_config.clone()),
            jav18tv_parser: Jav18tvParser::new(proxy_config.clone()),
            njavtv_parser: NjavtvParser::new(proxy_config.clone()),
            njav_parser: NjavParser::new(proxy_config.clone()),
            chrome_detector: Mutex::new(
                ChromeVideoDetector::new()
                    .expect("Chrome detector initialization failed. Please ensure Chrome/Chromium is installed.")
            ),
            downloader: Mutex::new(None),
            current_series: Mutex::new(None),
            download_states: Mutex::new(HashMap::new()),
            library_db,
            notification_db,
            queue_db,
            schedule_db,
            current_library_id: Mutex::new(None),
            schedule_config,
            webhook_config,
            backup_manager: backup_mgr,
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<AppState>();
            let sched_config = state.schedule_config.clone();
            scheduler::start_scheduler(sched_config, handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_domain_settings,
            save_domain_settings,
            fetch_series,
            update_series_state,
            check_ffmpeg_available,
            auto_detect_video_url,
            start_download,
            get_quality_options,
            pause_download,
            resume_download,
            cancel_download,
            get_episode_url,
            open_folder,
            list_files,
            delete_files,
            play_file,
            set_taskbar_progress,
            cmd_save_to_library,
            cmd_get_library,
            cmd_get_series_detail,
            cmd_remove_from_library,
            cmd_update_episode_status,
            cmd_search_library,
            cmd_get_tags,
            cmd_create_tag,
            cmd_delete_tag,
            cmd_assign_tag,
            cmd_unassign_tag,
            cmd_toggle_favorite,
            cmd_open_episode,
            cmd_refetch_series,
            cmd_get_proxy_config,
            cmd_save_proxy_config,
            cmd_test_proxy_connection,
            cmd_get_schedule_config,
            cmd_save_schedule_config,
            search_sites,
            get_browse_categories,
            browse_category,
            // Phase 3: Watch Progress
            cmd_get_library_stats,
            cmd_mark_episode_watched,
            cmd_mark_episode_unwatched,
            cmd_update_watch_progress,
            cmd_get_watch_progress,
            // Phase 3: Notifications
            cmd_get_notifications,
            cmd_mark_notification_read,
            cmd_mark_all_read,
            cmd_get_unread_count,
            cmd_clear_notifications,
            cmd_log_notification,
            // Phase 3: Queue Persistence
            cmd_persist_queue_item,
            cmd_get_persistent_queue,
            cmd_update_queue_item,
            cmd_remove_queue_item,
            cmd_clear_queue_completed,
            cmd_restore_queue,
            cmd_get_queue_stats,
            // Phase 3: Scheduler
            scheduler::cmd_create_schedule,
            scheduler::cmd_get_schedules,
            scheduler::cmd_update_schedule,
            scheduler::cmd_toggle_schedule,
            scheduler::cmd_delete_schedule,
            scheduler::cmd_get_due_schedules,
            scheduler::cmd_mark_schedule_run,
            // Phase 7: Webhooks
            webhook::cmd_get_webhook_config,
            webhook::cmd_save_webhook_config,
            webhook::cmd_test_webhook,
            webhook::cmd_check_new_episodes,
            webhook::cmd_send_test_notification,
            // Phase 5: Import & Export
            cmd_export_library,
            cmd_import_library,
            cmd_batch_parse_urls,
            // Phase 8: Backup & Duplicate Detection
            cmd_create_backup,
            cmd_restore_backup,
            cmd_find_duplicates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// --- Phase 3: Wrapper commands accessing DBs through AppState ---

// Watch Progress & Library Stats
#[tauri::command]
fn cmd_get_library_stats(state: State<'_, AppState>) -> Result<library::LibraryStats, String> {
    state.library_db.get_library_stats()
}

#[tauri::command]
fn cmd_mark_episode_watched(state: State<'_, AppState>, library_id: i64, episode_number: i32) -> Result<(), String> {
    state.library_db.mark_episode_watched(library_id, episode_number)
}

#[tauri::command]
fn cmd_mark_episode_unwatched(state: State<'_, AppState>, library_id: i64, episode_number: i32) -> Result<(), String> {
    state.library_db.mark_episode_unwatched(library_id, episode_number)
}

#[tauri::command]
fn cmd_update_watch_progress(state: State<'_, AppState>, library_id: i64, episode_number: i32, position_seconds: f64, duration_seconds: f64) -> Result<(), String> {
    state.library_db.update_watch_progress(library_id, episode_number, position_seconds, duration_seconds)
}

#[tauri::command]
fn cmd_get_watch_progress(state: State<'_, AppState>, library_id: i64, episode_number: i32) -> Result<Option<library::WatchProgress>, String> {
    state.library_db.get_watch_progress(library_id, episode_number)
}

// Notifications
#[tauri::command]
fn cmd_get_notifications(state: State<'_, AppState>, limit: i32, unread_only: bool) -> Result<Vec<notifications::NotificationEntry>, String> {
    state.notification_db.get_notifications(limit, unread_only)
}

#[tauri::command]
fn cmd_mark_notification_read(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.notification_db.mark_read(id)
}

#[tauri::command]
fn cmd_mark_all_read(state: State<'_, AppState>) -> Result<(), String> {
    state.notification_db.mark_all_read()
}

#[tauri::command]
fn cmd_get_unread_count(state: State<'_, AppState>) -> Result<i32, String> {
    state.notification_db.get_unread_count()
}

#[tauri::command]
fn cmd_clear_notifications(state: State<'_, AppState>, days: i32) -> Result<usize, String> {
    state.notification_db.clear_old(days)
}

#[tauri::command]
fn cmd_log_notification(state: State<'_, AppState>, category: String, title: String, message: String, action_type: Option<String>, action_data: Option<String>) -> Result<i64, String> {
    state.notification_db.log_notification(&category, &title, &message, action_type.as_deref(), action_data.as_deref())
}

// Queue Persistence
#[tauri::command]
fn cmd_persist_queue_item(state: State<'_, AppState>, url: String) -> Result<i64, String> {
    state.queue_db.add_item(&url)
}

#[tauri::command]
fn cmd_get_persistent_queue(state: State<'_, AppState>) -> Result<Vec<queue_db::PersistentQueueItem>, String> {
    state.queue_db.get_all()
}

#[tauri::command]
fn cmd_update_queue_item(state: State<'_, AppState>, id: i64, status: String, series_info: Option<String>, error: Option<String>) -> Result<(), String> {
    state.queue_db.update_status(id, &status, series_info.as_deref(), error.as_deref())
}

#[tauri::command]
fn cmd_remove_queue_item(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.queue_db.remove_item(id)
}

#[tauri::command]
fn cmd_clear_queue_completed(state: State<'_, AppState>) -> Result<usize, String> {
    state.queue_db.clear_completed()
}

#[tauri::command]
fn cmd_restore_queue(state: State<'_, AppState>) -> Result<Vec<queue_db::PersistentQueueItem>, String> {
    state.queue_db.get_pending()
}

#[tauri::command]
fn cmd_get_queue_stats(state: State<'_, AppState>) -> Result<queue_db::QueueStats, String> {
    state.queue_db.get_stats()
}

// Phase 5: Import & Export
#[tauri::command]
fn cmd_export_library(state: State<'_, AppState>) -> Result<String, String> {
    state.library_db.export_to_json()
}

#[tauri::command]
fn cmd_import_library(state: State<'_, AppState>, json_data: String) -> Result<i32, String> {
    state.library_db.import_from_json(&json_data)
}

#[tauri::command]
async fn cmd_batch_parse_urls(urls: String, app_handle: AppHandle, state: State<'_, AppState>) -> Result<Vec<UnifiedSeriesInfo>, String> {
    let url_list: Vec<&str> = urls.lines().filter(|l| !l.trim().is_empty()).collect();
    let mut results = Vec::new();
    for url in url_list {
        match fetch_series(url.to_string(), app_handle.clone(), state.clone()).await {
            Ok(info) => results.push(info),
            Err(_) => continue,
        }
    }
    Ok(results)
}


// Phase 8: Backup & Duplicate Detection
#[tauri::command]
fn cmd_create_backup(state: State<'_, AppState>, output_path: String) -> Result<String, String> {
    state.backup_manager.create_backup(&output_path)
}

#[tauri::command]
fn cmd_restore_backup(state: State<'_, AppState>, backup_path: String) -> Result<i32, String> {
    state.backup_manager.restore_backup(&backup_path)
}

#[tauri::command]
fn cmd_find_duplicates(state: State<'_, AppState>) -> Result<Vec<(library::LibraryEntry, Vec<library::LibraryEntry>)>, String> {
    state.library_db.find_duplicates()
}
