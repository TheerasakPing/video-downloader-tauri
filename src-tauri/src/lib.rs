mod baanjeen_parser;
mod chrome_detector;
mod downloader;
mod hsck_parser;
mod njavtv_parser;
mod parser;
mod titan_parser;
mod utils;

use baanjeen_parser::BaanJeenParser;
use chrome_detector::ChromeVideoDetector;
use downloader::{check_ffmpeg, merge_videos_with_progress, DownloadConfig, DownloadResult, DownloadState, VideoDownloader};
use hsck_parser::HsckParser;
use njavtv_parser::NjavtvParser;
use parser::RongyokParser;
use serde::{Deserialize, Serialize};
use titan_parser::{TitanParser, HlsKeyInfo};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::fs;
use tauri::{AppHandle, Emitter, State, Manager};
use utils::{expand_path, sanitize_filename};

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
}

impl DomainSettings {
    fn default_hsck_domain() -> String {
        "hsck123.com".to_string()
    }
    fn default_njavtv_domain() -> String {
        "njavtv.com".to_string()
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

// App state
struct AppState {
    rongyok_parser: RongyokParser,
    baanjeen_parser: BaanJeenParser,
    titan_parser: TitanParser,
    hsck_parser: HsckParser,
    njavtv_parser: NjavtvParser,
    chrome_detector: Mutex<ChromeVideoDetector>,
    downloader: Mutex<Option<VideoDownloader>>,
    current_series: Mutex<Option<UnifiedSeriesInfo>>,
    download_states: Mutex<HashMap<i32, Arc<DownloadState>>>,
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

        *state.current_series.lock().unwrap() = Some(series_info.clone());
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
            let mut detector = state.chrome_detector.lock().unwrap();

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
        let mut detector = state.chrome_detector.lock().unwrap();

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
    } else if TitanParser::is_titan_url(&url, &settings.titan_domain) {
        // Use Titan Parser
        let titan_info = state.titan_parser.get_series_info(&url, &settings.titan_domain).await?;
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
    *state.current_series.lock().unwrap() = Some(series_info.clone());

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

    let mut detector = state.chrome_detector.lock().unwrap();
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

    // คำนวณ output dir จริง (เพิ่ม subfolder ตาม source ถ้า group_by_source=true)
    let effective_output_dir = if request.group_by_source {
        let source_folder = match series.source.as_str() {
            "baanjeen" => "baanjeen",
            "hsck" => "hsck",
            "njavtv" => "njavtv",
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
    *state.downloader.lock().unwrap() = Some(VideoDownloader::with_config(&effective_output_dir, config));

    let mut results = Vec::new();
    let mut successful_files = Vec::new();

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

            // Pass HLS key info if available (Titan encrypted streams)
            let hls_key_info = series.episode_keys.get(episode).cloned();

            // Get source_url as referer for hotlink protection bypass
            // Clone to extend lifetime for async block
            let referer = series.source_url.clone();

            let cookies = series.cookies.clone();

            let app = app_handle.clone();
            let dl = VideoDownloader::with_config(
                &effective_output_dir,
                DownloadConfig {
                    speed_limit_kbps: request.speed_limit,
                    file_naming: request.file_naming.clone(),
                    series_title: request.series_title.clone(),
                }
            );
            let ep = *episode;

            // Create download state for this episode
            let download_state = Arc::new(DownloadState::new());
            {
                let mut states = state.download_states.lock().unwrap();
                states.insert(ep, download_state.clone());
            }

            let handle = tokio::spawn(async move {
                dl.download_episode(ep, &video_url, hls_key_info, referer.as_deref(), &cookies, &app, Some(download_state)).await
            });
            handles.push((ep, handle));
        }

        // Wait for all in this chunk to complete
        for (ep, handle) in handles {
            match handle.await {
                Ok(result) => {
                    // Remove from download states when done
                    {
                        let mut states = state.download_states.lock().unwrap();
                        states.remove(&ep);
                    }

                    if result.success {
                        if let Some(ref path) = result.file_path {
                            successful_files.push(path.clone());
                        }
                    }
                    let _ = app_handle.emit("download-result", &result);
                    results.push(result);
                }
                Err(e) => {
                    // Remove from download states on error
                    {
                        let mut states = state.download_states.lock().unwrap();
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

    // Debug: emit info about what we're about to do
    let files_count = successful_files.len();
    let ffmpeg_available = check_ffmpeg();
    let merge_info = format!(
        "Merge check: auto_merge={}, files={}, ffmpeg={}",
        request.auto_merge,
        files_count,
        ffmpeg_available
    );
    let _ = app_handle.emit("log-info", merge_info);

    // Debug: log the files list
    if !successful_files.is_empty() {
        let _ = app_handle.emit("log-info", format!("Files to merge: {:?}", &successful_files[..successful_files.len().min(5)]));
    }

    // Merge if requested
    let should_merge = request.auto_merge && files_count > 0 && ffmpeg_available;
    let _ = app_handle.emit("log-info", format!("Should merge: {}", should_merge));

    if should_merge {
        let _ = app_handle.emit("log-info", format!("Series title: {}", series.title));
        let output_filename = sanitize_filename(&series.title);
        let _ = app_handle.emit("log-info", format!("Output filename: {}", output_filename));
        let expanded_output_dir = expand_path(&effective_output_dir);
        let _ = app_handle.emit("log-info", format!("Expanded dir: {:?}", expanded_output_dir));
        let output_path = expanded_output_dir.join(format!("{}.mp4", output_filename));
        let output_path_str = output_path.to_string_lossy().to_string();

        let _ = app_handle.emit("log-info", format!("Starting merge to: {}", output_path_str));
        let _ = app_handle.emit("merge-started", ());

        if successful_files.len() == 1 {
            // Just rename/copy the single file
            let _ = app_handle.emit("log-info", "Single file - renaming...".to_string());

            // Check if source file exists
            let source = std::fs::canonicalize(&successful_files[0])
                .map_err(|e| format!("Cannot find source file: {}", e))
                .unwrap_or(std::path::PathBuf::from(&successful_files[0]));

            match std::fs::rename(&source, &output_path) {
                Ok(_) => {
                    let _ = app_handle.emit("merge-complete", output_path_str);
                }
                Err(e) => {
                    let _ = app_handle.emit("log-info", format!("Rename failed: {}, trying copy...", e));
                    // Try copy if rename fails (cross-device)
                    match std::fs::copy(&source, &output_path) {
                        Ok(_) => {
                            std::fs::remove_file(&source).ok();
                            let _ = app_handle.emit("merge-complete", output_path_str.clone());
                        }
                        Err(e) => {
                            let _ = app_handle.emit("merge-error", format!("Failed to rename: {}", e));
                        }
                    }
                }
            }
        } else {
            // Merge multiple files
            let _ = app_handle.emit("log-info", format!("Merging {} files with FFmpeg...", successful_files.len()));

            // Sort files by episode number before merging
            let mut sorted_files = successful_files.clone();
            sorted_files.sort();

            match merge_videos_with_progress(sorted_files.clone(), &output_path_str, Some(&app_handle)) {
                Ok(_) => {
                    let _ = app_handle.emit("log-info", "Merge complete, deleting individual files...".to_string());
                    // Delete individual files after successful merge
                    for file in &sorted_files {
                        std::fs::remove_file(file).ok();
                    }
                    let _ = app_handle.emit("merge-complete", output_path_str);
                }
                Err(e) => {
                    let _ = app_handle.emit("merge-error", e);
                }
            }
        }
    } else if request.auto_merge && !ffmpeg_available {
        let _ = app_handle.emit("merge-error", "FFmpeg not found - cannot merge videos".to_string());
    } else {
        let _ = app_handle.emit("log-info", format!("Merge skipped: auto_merge={}, files={}", request.auto_merge, files_count));
    }

    Ok(results)
}

#[tauri::command]
async fn pause_download(episode: i32, state: State<'_, AppState>) -> Result<(), String> {
    let states = state.download_states.lock().unwrap();
    if let Some(download_state) = states.get(&episode) {
        download_state.is_paused.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    } else {
        Err(format!("No active download for episode {}", episode))
    }
}

#[tauri::command]
async fn resume_download(episode: i32, state: State<'_, AppState>) -> Result<(), String> {
    let states = state.download_states.lock().unwrap();
    if let Some(download_state) = states.get(&episode) {
        download_state.is_paused.store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    } else {
        Err(format!("No active download for episode {}", episode))
    }
}

#[tauri::command]
async fn cancel_download(episode: i32, state: State<'_, AppState>) -> Result<(), String> {
    let states = state.download_states.lock().unwrap();
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
    if let Some(series) = state.current_series.lock().unwrap().as_ref() {
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
    *state.current_series.lock().unwrap() = Some(series);
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

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState {
            rongyok_parser: RongyokParser::new(),
            baanjeen_parser: BaanJeenParser::new(),
            titan_parser: TitanParser::new(),
            hsck_parser: HsckParser::new(),
            njavtv_parser: NjavtvParser::new(),
            chrome_detector: Mutex::new(ChromeVideoDetector::new().unwrap_or_else(|e| {
                eprintln!("Warning: Chrome detector initialization failed: {}", e);
                ChromeVideoDetector::new().unwrap()
            })),
            downloader: Mutex::new(None),
            current_series: Mutex::new(None),
            download_states: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            get_domain_settings,
            save_domain_settings,
            fetch_series,
            update_series_state,
            check_ffmpeg_available,
            auto_detect_video_url,
            start_download,
            pause_download,
            resume_download,
            cancel_download,
            get_episode_url,
            open_folder,
            list_files,
            delete_files,
            play_file,
            set_taskbar_progress,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
