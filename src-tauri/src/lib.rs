mod downloader;
mod parser;

use downloader::{check_ffmpeg, merge_videos, sanitize_filename, DownloadConfig, DownloadResult, VideoDownloader};
use parser::{RongyokParser, SeriesInfo};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

// Helper function to expand ~ to home directory
fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

// App state
struct AppState {
    parser: RongyokParser,
    downloader: Mutex<Option<VideoDownloader>>,
    current_series: Mutex<Option<SeriesInfo>>,
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
}

// Commands

#[tauri::command]
async fn fetch_series(url: String, state: State<'_, AppState>) -> Result<SeriesInfo, String> {
    let series_id = RongyokParser::parse_series_url(&url).ok_or("Invalid URL format")?;

    let series_info = state.parser.get_series_info(series_id).await?;

    // Store in state
    *state.current_series.lock().unwrap() = Some(series_info.clone());

    Ok(series_info)
}

#[tauri::command]
fn check_ffmpeg_available() -> bool {
    check_ffmpeg()
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

    // Create downloader with config
    let config = DownloadConfig {
        speed_limit_kbps: request.speed_limit,
        file_naming: request.file_naming.clone(),
        series_title: request.series_title.clone(),
    };
    let _downloader = VideoDownloader::with_config(&request.output_dir, config.clone());
    *state.downloader.lock().unwrap() = Some(VideoDownloader::with_config(&request.output_dir, config));

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

            let app = app_handle.clone();
            let dl = VideoDownloader::with_config(
                &request.output_dir,
                DownloadConfig {
                    speed_limit_kbps: request.speed_limit,
                    file_naming: request.file_naming.clone(),
                    series_title: request.series_title.clone(),
                }
            );
            let ep = *episode;

            let handle = tokio::spawn(async move {
                dl.download_episode(ep, &video_url, &app).await
            });
            handles.push(handle);
        }

        // Wait for all in this chunk to complete
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    if result.success {
                        if let Some(ref path) = result.file_path {
                            successful_files.push(path.clone());
                        }
                    }
                    let _ = app_handle.emit("download-result", &result);
                    results.push(result);
                }
                Err(e) => {
                    let result = DownloadResult {
                        episode: 0,
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
        let expanded_output_dir = expand_path(&request.output_dir);
        let _ = app_handle.emit("log-info", format!("Expanded dir: {:?}", expanded_output_dir));
        let output_path = expanded_output_dir.join(format!("{}.mp4", output_filename));
        let output_path_str = output_path.to_string_lossy().to_string();

        let _ = app_handle.emit("log-info", format!("Starting merge to: {}", output_path_str));
        let _ = app_handle.emit("merge-started", ());

        if successful_files.len() == 1 {
            // Just rename/copy the single file
            let _ = app_handle.emit("log-info", "Single file - renaming...".to_string());
            match std::fs::rename(&successful_files[0], &output_path) {
                Ok(_) => {
                    let _ = app_handle.emit("merge-complete", output_path_str);
                }
                Err(e) => {
                    let _ = app_handle.emit("log-info", format!("Rename failed: {}, trying copy...", e));
                    // Try copy if rename fails (cross-device)
                    match std::fs::copy(&successful_files[0], &output_path) {
                        Ok(_) => {
                            std::fs::remove_file(&successful_files[0]).ok();
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

            match merge_videos(sorted_files.clone(), &output_path_str) {
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
async fn get_episode_url(
    series_id: i32,
    episode: i32,
    state: State<'_, AppState>,
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
    let series_info = state.parser.get_series_info(series_id).await?;
    series_info
        .episode_urls
        .get(&episode)
        .cloned()
        .ok_or(format!("No URL for episode {}", episode))
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
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState {
            parser: RongyokParser::new(),
            downloader: Mutex::new(None),
            current_series: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            fetch_series,
            check_ffmpeg_available,
            start_download,
            get_episode_url,
            open_folder,
            list_files,
            delete_files,
            play_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
