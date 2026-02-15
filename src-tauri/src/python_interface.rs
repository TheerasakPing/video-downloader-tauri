use std::collections::HashMap;
use std::process::Command;
use crate::UnifiedSeriesInfo;
use serde::Deserialize;
use std::path::Path;

// Helper to call python script
fn call_python_script(mode: &str, url: &str) -> Result<String, String> {
    // Try to find the script. Assuming we are in project root or src-tauri
    let script_locations = [
        "scripts/357ms_extractor.py",
        "../scripts/357ms_extractor.py",
        "/Volumes/Data/code_project/_rongyok_video_downloader_rust/scripts/357ms_extractor.py"
    ];

    let script_path = script_locations.iter()
        .find(|p| Path::new(p).exists())
        .ok_or_else(|| "Could not find 357ms_extractor.py".to_string())?;

    let output = Command::new("python3")
        .arg(script_path)
        .arg(mode)
        .arg(url)
        .output()
        .map_err(|e| format!("Failed to execute python: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Python script failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

pub fn fetch_357ms_series(url: &str) -> Result<UnifiedSeriesInfo, String> {
    eprintln!("[357ms] Fetching series info via Python for: {}", url);
    let json_output = call_python_script("series", url)?;

    #[derive(Deserialize)]
    struct PyEpisode {
        number: i32,
        title: String,
        url: String,
    }

    #[derive(Deserialize)]
    struct PySeriesInfo {
        title: String,
        cover_url: Option<String>,
        episodes: Vec<PyEpisode>,
    }

    let info: PySeriesInfo = serde_json::from_str(&json_output)
        .map_err(|e| format!("Failed to parse Python output: {}", e))?;

    let mut episode_urls = HashMap::new();
    for ep in info.episodes {
        episode_urls.insert(ep.number, ep.url);
    }

    eprintln!("[357ms] Found {} episodes for {}", episode_urls.len(), info.title);

    Ok(UnifiedSeriesInfo {
        series_id: 0,
        title: info.title,
        total_episodes: episode_urls.len() as i32,
        poster_url: info.cover_url,
        episode_urls,
        source: "357ms".to_string(),
    })
}

// Return type: (m3u8_url, headers, key_hex)
pub fn extract_357ms_video(url: &str) -> Result<(String, HashMap<String, String>, Option<String>), String> {
    eprintln!("[357ms] Extracting video via Python for: {}", url);
    let json_output = call_python_script("video", url)?;

    #[derive(Deserialize)]
    struct PyVideoInfo {
        m3u8_url: Option<String>,
        headers: HashMap<String, String>,
        key_hex: Option<String>,
        error: Option<String>,
    }

    let info: PyVideoInfo = serde_json::from_str(&json_output)
        .map_err(|e| format!("Failed to parse Python output: {}", e))?;

    if let Some(err) = info.error {
        return Err(format!("Extractor returned error: {}", err));
    }

    let m3u8 = info.m3u8_url.ok_or("No m3u8_url found".to_string())?;
    
    // Ensure User-Agent is present
    let mut headers = info.headers;
    if !headers.contains_key("User-Agent") {
        headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string());
    }

    Ok((m3u8, headers, info.key_hex))
}