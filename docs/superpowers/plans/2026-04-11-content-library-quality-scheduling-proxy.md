# Content Library, Quality Selection, Scheduling & Proxy — Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add persistent content library, HLS quality selection, time-based scheduling with speed profiles, and proxy/retry support to the Rongyok Video Downloader.

**Architecture:** Phase 1 adds SQLite-backed library storage and HLS quality parsing. Phase 2 adds a background scheduler task, proxy propagation via `Arc<RwLock<ProxyConfig>>`, and retry logic. All features integrate through the existing Tauri command + event system.

**Tech Stack:** Rust (rusqlite, tokio watch channels, reqwest proxy), TypeScript/React (new components, hooks), SQLite

**Spec:** `docs/superpowers/specs/2026-04-10-content-library-quality-scheduling-proxy-design.md`

---

## File Structure

### New Files

| File | Responsibility |
|------|---------------|
| `src-tauri/src/library.rs` | SQLite storage, library CRUD Tauri commands |
| `src-tauri/src/scheduler.rs` | Background tokio task for schedule checking |
| `src-tauri/src/proxy.rs` | `ProxyConfig`, `RetryConfig`, `build_client()`, proxy Tauri commands |
| `src/components/LibraryPanel.tsx` | Library grid view with filters/sort/search |
| `src/components/LibraryCard.tsx` | Individual series card in library grid |
| `src/components/SeriesDetail.tsx` | Series detail view with episode status |
| `src/components/QualitySelector.tsx` | Quality dropdown selector |
| `src/hooks/useLibrary.ts` | Library state management hook |

### Modified Files

| File | Changes |
|------|---------|
| `src-tauri/Cargo.toml` | Add rusqlite dependency |
| `src-tauri/src/lib.rs` | Register new commands, extend AppState, auto-save to library, quality threading |
| `src-tauri/src/downloader.rs` | Extract `parse_master_playlist()`, add quality param, watch channels, retry logic |
| `src-tauri/src/parser.rs` | Replace `Client` with `Arc<RwLock<ProxyConfig>>` |
| `src-tauri/src/baanjeen_parser.rs` | Same proxy pattern |
| `src-tauri/src/hsck_parser.rs` | Same proxy pattern |
| `src-tauri/src/njav_parser.rs` | Same proxy pattern |
| `src-tauri/src/njavtv_parser.rs` | Same proxy pattern |
| `src-tauri/src/titan_parser.rs` | Same proxy pattern |
| `src-tauri/src/chrome_detector.rs` | Add `--proxy-server` arg to Chrome launch |
| `src/App.tsx` | Add library tab, integrate QualitySelector, library auto-save |
| `src/types/index.ts` | Add new types, extend SeriesInfo |
| `src/components/SettingsPanel.tsx` | Add schedule, proxy, retry settings |
| `src/hooks/useSettings.ts` | Add new settings fields |
| `src/components/index.ts` | Export new components |

---

## Phase 1: Content Library & Quality Selection

### Task 1: Add rusqlite dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add rusqlite to Cargo.toml**

Add under `[dependencies]`:

```toml
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = "0.4"
```

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: Compiles with new dependency

- [ ] **Step 3: Commit**

```
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "chore: add rusqlite dependency for content library"
```

---

### Task 2: Create library.rs — SQLite schema + CRUD operations

**Files:**
- Create: `src-tauri/src/library.rs`

- [ ] **Step 1: Create library.rs with schema and core types**

```rust
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;

// --- Data types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEntry {
    pub id: i64,
    pub parser_series_id: i32,
    pub title: String,
    pub source: String,
    pub source_url: Option<String>,
    pub poster_path: Option<String>,
    pub total_episodes: i32,
    pub date_added: String,
    pub last_downloaded: Option<String>,
    pub completed_count: i32,  // computed from episodes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEpisode {
    pub id: i64,
    pub library_id: i64,
    pub episode_number: i32,
    pub video_url: Option<String>,
    pub file_path: Option<String>,
    pub quality: Option<String>,
    pub file_size: Option<i64>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesDetail {
    pub entry: LibraryEntry,
    pub episodes: Vec<LibraryEpisode>,
    pub can_refetch: bool,
}

// --- Database manager ---

pub struct LibraryDb {
    conn: Mutex<Connection>,
    poster_dir: PathBuf,
}

impl LibraryDb {
    pub fn new(app_data_dir: &std::path::Path) -> Result<Self, String> {
        let db_path = app_data_dir.join("library.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open library DB: {}", e))?;

        let poster_dir = app_data_dir.join("library_posters");
        std::fs::create_dir_all(&poster_dir).ok();

        let db = Self {
            conn: Mutex::new(conn),
            poster_dir,
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY);
             CREATE TABLE IF NOT EXISTS library (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 parser_series_id INTEGER NOT NULL DEFAULT 0,
                 title TEXT NOT NULL,
                 source TEXT NOT NULL,
                 source_url TEXT NOT NULL DEFAULT '',
                 poster_path TEXT,
                 total_episodes INTEGER DEFAULT 1,
                 date_added TEXT NOT NULL,
                 last_downloaded TEXT,
                 metadata TEXT,
                 UNIQUE(source, source_url)
             );
             CREATE TABLE IF NOT EXISTS library_episodes (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 library_id INTEGER NOT NULL,
                 episode_number INTEGER NOT NULL,
                 video_url TEXT,
                 file_path TEXT,
                 quality TEXT,
                 file_size INTEGER,
                 status TEXT DEFAULT 'pending',
                 FOREIGN KEY (library_id) REFERENCES library(id) ON DELETE CASCADE,
                 UNIQUE(library_id, episode_number)
             );"
        ).map_err(|e| format!("Schema init failed: {}", e))?;

        // Insert schema version if not exists
        conn.execute(
            "INSERT OR IGNORE INTO schema_version (version) VALUES (1)",
            [],
        ).map_err(|e| format!("Schema version insert failed: {}", e))?;

        Ok(())
    }

    // --- CRUD methods will be added in next steps ---
}
```

- [ ] **Step 2: Add save_to_library method**

Add to `impl LibraryDb`:

```rust
pub fn save_series(
    &self,
    title: &str,
    source: &str,
    source_url: Option<&str>,
    poster_data: Option<&[u8]>,
    total_episodes: i32,
    parser_series_id: i32,
    episode_urls: &std::collections::HashMap<i32, String>,
    metadata: Option<&str>,
) -> Result<i64, String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    // Upsert library entry
    conn.execute(
        "INSERT INTO library (parser_series_id, title, source, source_url, total_episodes, date_added, metadata)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(source, source_url) DO UPDATE SET
            title = excluded.title,
            total_episodes = excluded.total_episodes,
            metadata = excluded.metadata",
        params![parser_series_id, title, source, source_url, total_episodes, now, metadata],
    ).map_err(|e| format!("Save library failed: {}", e))?;

    let library_id = conn.last_insert_rowid();

    // Save poster if provided
    if let Some(data) = poster_data {
        let poster_path = self.poster_dir.join(format!("{}.jpg", library_id));
        std::fs::write(&poster_path, data).ok();
        conn.execute(
            "UPDATE library SET poster_path = ?1 WHERE id = ?2",
            params![poster_path.to_str(), library_id],
        ).ok();
    }

    // Upsert episodes
    for (ep_num, url) in episode_urls {
        conn.execute(
            "INSERT INTO library_episodes (library_id, episode_number, video_url)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(library_id, episode_number) DO UPDATE SET
                video_url = excluded.video_url",
            params![library_id, ep_num, url],
        ).map_err(|e| format!("Save episode {} failed: {}", ep_num, e))?;
    }

    Ok(library_id)
}

pub fn update_episode_status(
    &self,
    library_id: i64,
    episode_number: i32,
    status: &str,
    file_path: Option<&str>,
) -> Result<(), String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE library_episodes SET status = ?1, file_path = ?2 WHERE library_id = ?3 AND episode_number = ?4",
        params![status, file_path, library_id, episode_number],
    ).map_err(|e| format!("Update episode status failed: {}", e))?;
    Ok(())
}

pub fn get_library(&self) -> Result<Vec<LibraryEntry>, String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT l.id, l.parser_series_id, l.title, l.source, l.source_url,
                l.poster_path, l.total_episodes, l.date_added, l.last_downloaded,
                COUNT(CASE WHEN e.status = 'completed' THEN 1 END) as completed_count
         FROM library l
         LEFT JOIN library_episodes e ON e.library_id = l.id
         GROUP BY l.id
         ORDER BY l.date_added DESC"
    ).map_err(|e| format!("Query library failed: {}", e))?;

    let entries = stmt.query_map([], |row| {
        Ok(LibraryEntry {
            id: row.get(0)?,
            parser_series_id: row.get(1)?,
            title: row.get(2)?,
            source: row.get(3)?,
            source_url: row.get(4)?,
            poster_path: row.get(5)?,
            total_episodes: row.get(6)?,
            date_added: row.get(7)?,
            last_downloaded: row.get(8)?,
            completed_count: row.get(9)?,
        })
    }).map_err(|e| format!("Map library entries failed: {}", e))?
    .filter_map(|e| e.ok())
    .collect();

    Ok(entries)
}

pub fn get_series_detail(&self, library_id: i64) -> Result<SeriesDetail, String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;

    let entry: LibraryEntry = conn.query_row(
        "SELECT l.id, l.parser_series_id, l.title, l.source, l.source_url,
                l.poster_path, l.total_episodes, l.date_added, l.last_downloaded,
                COUNT(CASE WHEN e.status = 'completed' THEN 1 END)
         FROM library l LEFT JOIN library_episodes e ON e.library_id = l.id
         WHERE l.id = ?1 GROUP BY l.id",
        params![library_id],
        |row| Ok(LibraryEntry {
            id: row.get(0)?, parser_series_id: row.get(1)?, title: row.get(2)?,
            source: row.get(3)?, source_url: row.get(4)?, poster_path: row.get(5)?,
            total_episodes: row.get(6)?, date_added: row.get(7)?, last_downloaded: row.get(8)?,
            completed_count: row.get(9)?,
        }),
    ).map_err(|e| format!("Series not found: {}", e))?;

    let mut stmt = conn.prepare(
        "SELECT id, library_id, episode_number, video_url, file_path, quality, file_size, status
         FROM library_episodes WHERE library_id = ?1 ORDER BY episode_number"
    ).map_err(|e| e.to_string())?;

    let episodes: Vec<LibraryEpisode> = stmt.query_map(params![library_id], |row| {
        Ok(LibraryEpisode {
            id: row.get(0)?, library_id: row.get(1)?, episode_number: row.get(2)?,
            video_url: row.get(3)?, file_path: row.get(4)?, quality: row.get(5)?,
            file_size: row.get(6)?, status: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?.filter_map(|e| e.ok()).collect();

    let source_url = entry.source_url.as_deref().unwrap_or("");
    let can_refetch = !source_url.contains(".m3u8") && !source_url.contains(".mp4") && !source_url.is_empty();

    Ok(SeriesDetail { entry, episodes, can_refetch })
}

pub fn remove_series(&self, library_id: i64) -> Result<(), String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    // Get poster path before deleting
    let poster_path: Option<String> = conn.query_row(
        "SELECT poster_path FROM library WHERE id = ?1", params![library_id],
        |row| row.get(0),
    ).unwrap_or(None);

    if let Some(path) = poster_path {
        std::fs::remove_file(&path).ok();
    }

    conn.execute("DELETE FROM library WHERE id = ?1", params![library_id])
        .map_err(|e| format!("Delete failed: {}", e))?;
    Ok(())
}

pub fn search_library(&self, query: &str) -> Result<Vec<LibraryEntry>, String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT l.id, l.parser_series_id, l.title, l.source, l.source_url,
                l.poster_path, l.total_episodes, l.date_added, l.last_downloaded,
                COUNT(CASE WHEN e.status = 'completed' THEN 1 END)
         FROM library l LEFT JOIN library_episodes e ON e.library_id = l.id
         WHERE l.title LIKE ?1
         GROUP BY l.id ORDER BY l.date_added DESC"
    ).map_err(|e| e.to_string())?;

    stmt.query_map(params![pattern], |row| {
        Ok(LibraryEntry {
            id: row.get(0)?, parser_series_id: row.get(1)?, title: row.get(2)?,
            source: row.get(3)?, source_url: row.get(4)?, poster_path: row.get(5)?,
            total_episodes: row.get(6)?, date_added: row.get(7)?, last_downloaded: row.get(8)?,
            completed_count: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?.filter_map(|e| e.ok()).collect::<Vec<_>>();
    Ok(entries)
}
```

Note: Also add `chrono` to Cargo.toml if not already present.

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles with library module

- [ ] **Step 4: Commit**

```
git add src-tauri/src/library.rs src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add SQLite library storage module with CRUD operations"
```

---

### Task 3: Register library commands in lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs` (add `mod library;` at top, extend `AppState`, add Tauri commands, register)

- [ ] **Step 1: Add module declaration and AppState field**

At top of `lib.rs`, add:
```rust
mod library;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
```

Extend `AppState` struct (around line 117):
```rust
struct AppState {
    // ... existing fields ...
    library_db: library::LibraryDb,
    current_library_id: Mutex<Option<i64>>,
}
```

- [ ] **Step 2: Add Tauri command wrappers**

Add these commands before the `run()` function:

```rust
#[tauri::command]
fn cmd_save_to_library(
    state: State<'_, AppState>,
    title: String, source: String, source_url: Option<String>,
    poster_url: Option<String>, total_episodes: i32, parser_series_id: i32,
    episode_urls: HashMap<i32, String>, metadata: Option<String>,
) -> Result<i64, String> {
    // Download poster if it's a remote URL, or decode if base64
    let poster_data = if let Some(ref url) = poster_url {
        if url.starts_with("data:") {
            // Extract base64 portion
            url.split(',').nth(1)
                .and_then(|b| base64::engine::general_purpose::STANDARD.decode(b).ok())
        } else if url.starts_with("http") {
            // Will be fetched asynchronously — skip for now, store URL
            None
        } else {
            None
        }
    } else {
        None
    };

    state.library_db.save_series(
        &title, &source, source_url.as_deref(),
        poster_data.as_deref(), total_episodes, parser_series_id,
        &episode_urls, metadata.as_deref(),
    )
}

#[tauri::command]
fn cmd_get_library(state: State<'_, AppState>) -> Result<Vec<library::LibraryEntry>, String> {
    state.library_db.get_library()
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

#[tauri::command]
fn cmd_search_library(state: State<'_, AppState>, query: String) -> Result<Vec<library::LibraryEntry>, String> {
    state.library_db.search_library(&query)
}
```

- [ ] **Step 3: Initialize LibraryDb in run() function**

In the `run()` function where `AppState` is constructed, add:

```rust
let app_data_dir = app_handle.path().app_data_dir()
    .expect("Failed to get app data dir");
let library_db = library::LibraryDb::new(&app_data_dir)
    .expect("Failed to initialize library database");
```

Pass `library_db` and `current_library_id: Mutex::new(None)` into the `AppState` initialization.

- [ ] **Step 4: Register commands**

Add to the `.invoke_handler()` macro:

```rust
cmd_save_to_library, cmd_get_library, cmd_get_series_detail,
cmd_remove_from_library, cmd_update_episode_status, cmd_search_library,
cmd_refetch_series,
```

Also add the `refetch_series` command (required by spec):

```rust
#[tauri::command]
async fn cmd_refetch_series(
    library_id: i64,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<library::SeriesDetail, String> {
    // Get stored source_url from library
    let detail = state.library_db.get_series_detail(library_id)?;
    let source_url = detail.entry.source_url
        .ok_or_else(|| "No source URL stored for this series".to_string())?;

    if !detail.can_refetch {
        return Err("This series cannot be re-fetched (direct video URL)".to_string());
    }

    // Re-invoke fetch_series with stored URL
    let new_info = fetch_series(source_url, app_handle, state).await?;

    // Update library with fresh data
    state.library_db.save_series(
        &new_info.title, &new_info.source, Some(&source_url),
        None, new_info.total_episodes, new_info.series_id,
        &new_info.episode_urls, None,
    ).ok();

    // Return updated detail
    state.library_db.get_series_detail(library_id)
}
```

- [ ] **Step 5: Auto-save to library after fetch_series**

At the end of `fetch_series()` (where `UnifiedSeriesInfo` is returned), add auto-save call. **Note:** `fetch_series()` has two return paths — the early return for direct video URLs (~line 173) and the main return at the end (~line 424). The auto-save must go before both:

```rust
// Auto-save to library (place before BOTH return statements in fetch_series)
let lib_id = state.library_db.save_series(
    &series_info.title, &series_info.source,
    Some(&url),  // source_url is the input URL
    None, series_info.total_episodes, series_info.series_id,
    &series_info.episode_urls, None,
).ok();
*state.current_library_id.lock().unwrap() = lib_id;
```

For the direct video URL path (~line 173), add the same auto-save before `return Ok(series_info)`.

- [ ] **Step 6: Update episode status after download completes**

In the `download-result` event handling section of `start_download()`, after each episode completes:

```rust
if let Some(lib_id) = *state.current_library_id.lock().unwrap() {
    state.library_db.update_episode_status(
        lib_id, ep, if result.success { "completed" } else { "failed" },
        result.file_path.as_deref(),
    ).ok();
}
```

- [ ] **Step 7: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles with all new commands

- [ ] **Step 8: Commit**

```
git add src-tauri/src/lib.rs
git commit -m "feat: register library Tauri commands and auto-save on fetch/download"
```

---

### Task 4: Extend frontend types

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: Extend SeriesInfo and add new types**

Replace entire file content:

```typescript
export interface SeriesInfo {
  seriesId: number;
  title: string;
  totalEpisodes: number;
  url?: string;
  posterUrl?: string;
  episodeUrls: Record<number, string>;
  source: string;
  sourceUrl?: string;
  episodeKeys?: Record<number, HlsKeyInfo>;
  cookies?: [string, string][];
}

export interface HlsKeyInfo {
  key: string;
  iv: string;
}

export interface EpisodeInfo {
  episodeNumber: number;
  title: string;
  videoUrl: string;
}

export interface DownloadProgress {
  episode: number;
  downloaded: number;
  total: number;
  speed: number;
  percentage: number;
}

export interface DownloadState {
  isDownloading: boolean;
  isPaused: boolean;
  currentEpisode: number;
  completedEpisodes: number[];
  failedEpisodes: number[];
  totalSelected: number;
}

export type LogLevel = "info" | "success" | "warning" | "error";

export interface LogEntry {
  id: string;
  timestamp: Date;
  level: LogLevel;
  message: string;
}

export interface DomainSettings {
  titanDomain: string;
  baanjeenDomain: string;
  rongyokDomain: string;
  hsckDomain?: string;
  njavtvDomain?: string;
}

// --- New: Library types ---

export interface LibraryEntry {
  id: number;
  parserSeriesId: number;
  title: string;
  source: string;
  sourceUrl?: string;
  posterPath?: string;
  totalEpisodes: number;
  dateAdded: string;
  lastDownloaded?: string;
  completedCount: number;
}

export interface LibraryEpisode {
  id: number;
  libraryId: number;
  episodeNumber: number;
  videoUrl?: string;
  filePath?: string;
  quality?: string;
  fileSize?: number;
  status: string;
}

export interface SeriesDetail {
  entry: LibraryEntry;
  episodes: LibraryEpisode[];
  canRefetch: boolean;
}

// --- New: Quality types ---

export interface QualityOption {
  resolution: string;
  bandwidth: number;
  label: string;
  streamUrl: string;
}

export interface QualityInfo {
  qualities: QualityOption[];
  defaultIndex: number;
}

// --- New: Phase 2 types ---

export interface ScheduleConfig {
  enabled: boolean;
  activeStart?: string;
  activeEnd?: string;
  speedDuringActive: number;
  speedOutsideActive: number;
  autoPause: boolean;
  autoResume: boolean;
}

export interface ProxyConfig {
  proxyType: 'Http' | 'Socks5' | 'Direct';
  host: string;
  port: number;
  username?: string;
  password?: string;
}

export interface RetryConfig {
  maxRetries: number;
  retryDelayMs: number;
  fallbackUrls: string[];
  autoRetry: boolean;
  skipFailedSegments: boolean;
}
```

- [ ] **Step 2: Commit**

```
git add src/types/index.ts
git commit -m "feat: extend frontend types for library, quality, scheduling, and proxy"
```

---

### Task 5: Extract parse_master_playlist in downloader.rs

**Files:**
- Modify: `src-tauri/src/downloader.rs`

- [ ] **Step 1: Add QualityOption struct and parse function**

Add near top of file (after imports):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityOption {
    pub resolution: String,
    pub bandwidth: u64,
    pub label: String,
    pub stream_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityInfo {
    pub qualities: Vec<QualityOption>,
    pub default_index: usize,
}

/// Parse HLS master playlist to extract quality variants.
/// Returns variants sorted by bandwidth (highest first).
pub fn parse_master_playlist(master_text: &str, master_url: &str) -> Vec<QualityOption> {
    let base_url = master_url.rfind('/').map(|i| &master_url[..=i]).unwrap_or(master_url);
    let mut variants: Vec<QualityOption> = Vec::new();
    let mut current_bandwidth: i64 = -1;
    let mut current_resolution: Option<String> = None;

    for line in master_text.lines() {
        let line = line.trim();
        if line.starts_with("#EXT-X-STREAM-INF:") {
            current_bandwidth = line.split(',')
                .find_map(|part| {
                    part.trim().strip_prefix("BANDWIDTH=")
                        .and_then(|v| v.parse::<i64>().ok())
                })
                .unwrap_or(0);
            current_resolution = line.split(',')
                .find_map(|part| {
                    part.trim().strip_prefix("RESOLUTION=")
                        .map(|v| v.to_string())
                });
        } else if !line.is_empty() && !line.starts_with('#') && current_bandwidth >= 0 {
            let sub_url = if line.starts_with("http") {
                line.to_string()
            } else {
                format!("{}/{}", base_url.trim_end_matches('/'), line.trim_start_matches('/'))
            };
            let resolution = current_resolution.clone().unwrap_or_else(|| "unknown".to_string());
            let bandwidth = current_bandwidth as u64;
            let height = resolution.split('x').nth(1).unwrap_or("?");
            let label = format!("{}p ({:.1} Mbps)", height, bandwidth as f64 / 1_000_000.0);
            variants.push(QualityOption {
                resolution,
                bandwidth,
                label,
                stream_url: sub_url,
            });
            current_bandwidth = -1;
            current_resolution = None;
        }
    }

    variants.sort_by(|a, b| b.bandwidth.cmp(&a.bandwidth));
    variants
}
```

- [ ] **Step 2: Refactor download_hls_manual to use parse_master_playlist**

Replace lines 711-743 of `download_hls_manual()` (the variant selection block) with:

```rust
// 2. Parse quality variants from master playlist
let variants = parse_master_playlist(&master_text, master_url);

// Select sub-playlist: use preferred quality if specified, else highest bandwidth
let sub_playlist_url = if let Some(ref quality) = preferred_quality {
    variants.iter()
        .find(|v| v.resolution == *quality)
        .map(|v| v.stream_url.clone())
        .or_else(|| {
            // Fallback to nearest lower quality
            let target = quality.split('x').nth(1)
                .and_then(|h| h.parse::<i32>().ok())
                .unwrap_or(0);
            variants.iter()
                .filter(|v| {
                    v.resolution.split('x').nth(1)
                        .and_then(|h| h.parse::<i32>().ok())
                        .unwrap_or(0) <= target
                })
                .map(|v| v.stream_url.clone())
                .next()
        })
        .unwrap_or_else(|| variants.first().map(|v| v.stream_url.clone()).unwrap_or_else(|| master_url.to_string()))
} else {
    // Current behavior: highest bandwidth
    variants.first().map(|v| v.stream_url.clone()).unwrap_or_else(|| master_url.to_string())
};
```

Note: This requires adding `preferred_quality: Option<String>` parameter to `download_hls_manual()` signature.

- [ ] **Step 3: Add preferred_quality to download_episode routing**

Modify `download_episode()` signature (line 114) to accept `preferred_quality: Option<String>`:

```rust
pub async fn download_episode(
    &self,
    episode: i32,
    video_url: &str,
    hls_key_info: Option<crate::titan_parser::HlsKeyInfo>,
    referer: Option<&str>,
    cookies: &[(String, String)],
    app_handle: &AppHandle,
    download_state: Option<Arc<DownloadState>>,
    preferred_quality: Option<String>,  // NEW
) -> DownloadResult {
```

Pass `preferred_quality` through to `download_hls_manual()`. For `download_hls_stream()`, when quality is specified, pre-resolve master playlist and pass sub-playlist URL:

```rust
} else if video_url.contains(".m3u8") {
    let effective_url = if let Some(ref quality) = preferred_quality {
        // Pre-resolve to specific quality sub-playlist
        let client = Client::builder().user_agent("Mozilla/5.0").build().unwrap_or(Client::new());
        if let Ok(resp) = client.get(video_url).send().await {
            if let Ok(text) = resp.text().await {
                let variants = parse_master_playlist(&text, video_url);
                variants.iter()
                    .find(|v| v.resolution == *quality)
                    .map(|v| v.stream_url.clone())
                    .unwrap_or(video_url.to_string())
            } else { video_url.to_string() }
        } else { video_url.to_string() }
    } else { video_url.to_string() };

    let result = self.download_hls_stream(episode, &effective_url, output_path.to_str().unwrap(), referer, cookies, app_handle, download_state.clone()).await;
    // ... existing fallback to manual ...
```

- [ ] **Step 4: Add get_quality_options Tauri command in lib.rs**

```rust
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
        // Not an HLS master playlist — single quality
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
```

Register in `.invoke_handler()`.

- [ ] **Step 5: Thread preferred_quality through start_download**

Add `preferred_quality: Option<String>` to `DownloadRequest` struct (line 132). In `start_download()`, pass it to each `download_episode()` call.

- [ ] **Step 6: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles with quality selection support

- [ ] **Step 7: Commit**

```
git add src-tauri/src/downloader.rs src-tauri/src/lib.rs
git commit -m "feat: extract HLS quality parser and add quality selection to download flow"
```

---

### Task 6: Create QualitySelector component

**Files:**
- Create: `src/components/QualitySelector.tsx`
- Modify: `src/App.tsx`
- Modify: `src/hooks/useSettings.ts`

- [ ] **Step 1: Create QualitySelector.tsx**

```tsx
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { QualityInfo, QualityOption } from '../types';
import { Monitor, ChevronDown } from 'lucide-react';

interface QualitySelectorProps {
  episodeUrl: string | undefined;
  onSelect: (quality: string | null) => void;
  defaultQuality: string;
}

export default function QualitySelector({ episodeUrl, onSelect, defaultQuality }: QualitySelectorProps) {
  const [qualityInfo, setQualityInfo] = useState<QualityInfo | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [expanded, setExpanded] = useState(false);

  useEffect(() => {
    if (!episodeUrl || defaultQuality !== 'ask') {
      return;
    }
    setLoading(true);
    invoke<QualityInfo>('get_quality_options', { url: episodeUrl })
      .then(info => {
        setQualityInfo(info);
        if (info.qualities.length > 0) {
          setSelected(null); // null = best available
        }
      })
      .catch(() => setQualityInfo(null))
      .finally(() => setLoading(false));
  }, [episodeUrl, defaultQuality]);

  const handleSelect = (quality: string | null) => {
    setSelected(quality);
    onSelect(quality);
    setExpanded(false);
  };

  if (defaultQuality === 'best' || !qualityInfo || qualityInfo.qualities.length <= 1) {
    return null;
  }

  return (
    <div className="flex items-center gap-2">
      <Monitor size={16} className="text-[var(--accent)]" />
      <div className="relative">
        <button
          onClick={() => setExpanded(!expanded)}
          className="flex items-center gap-1 px-3 py-1.5 rounded-lg bg-[var(--card)] border border-[var(--border)] text-sm hover:border-[var(--accent)] transition-colors"
        >
          {selected
            ? qualityInfo.qualities.find(q => q.resolution === selected)?.label || 'Best'
            : 'Best available'}
          <ChevronDown size={14} />
        </button>
        {expanded && (
          <div className="absolute top-full mt-1 left-0 z-50 bg-[var(--card)] border border-[var(--border)] rounded-lg shadow-xl min-w-[200px]">
            <button
              onClick={() => handleSelect(null)}
              className={`w-full text-left px-3 py-2 text-sm hover:bg-[var(--accent)]/10 ${!selected ? 'text-[var(--accent)]' : ''}`}
            >
              Best available (recommended)
            </button>
            {qualityInfo.qualities.map((q, i) => (
              <button
                key={i}
                onClick={() => handleSelect(q.resolution)}
                className={`w-full text-left px-3 py-2 text-sm hover:bg-[var(--accent)]/10 ${selected === q.resolution ? 'text-[var(--accent)]' : ''}`}
              >
                {q.label}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Integrate in App.tsx download tab**

In the download section (between EpisodeSelector and the Download button area), add:

```tsx
<QualitySelector
  episodeUrl={seriesInfo ? Object.values(seriesInfo.episodeUrls)[0] : undefined}
  onSelect={(q) => setSelectedQuality(q)}
  defaultQuality={settings.defaultQuality || 'best'}
/>
```

Add state: `const [selectedQuality, setSelectedQuality] = useState<string | null>(null);`

Pass `selectedQuality` to `start_download` invoke as `preferredQuality`.

- [ ] **Step 3: Add defaultQuality to useSettings**

Add to the `Settings` interface in `useSettings.ts`:
```ts
defaultQuality: string;  // "best" | "1080p" | "720p" | "480p" | "ask"
```

Add to defaults: `defaultQuality: 'best'`

Add a quality selector section in `SettingsPanel.tsx`.

- [ ] **Step 4: Export new component**

Add to `src/components/index.ts`:
```ts
export { default as QualitySelector } from './QualitySelector';
```

- [ ] **Step 5: Verify build**

Run: `npm run build`
Expected: Frontend compiles with QualitySelector

- [ ] **Step 6: Commit**

```
git add src/components/QualitySelector.tsx src/App.tsx src/hooks/useSettings.ts src/components/index.ts
git commit -m "feat: add QualitySelector component with lazy HLS quality fetching"
```

---

### Task 7: Create Library components + tab

**Files:**
- Create: `src/hooks/useLibrary.ts`
- Create: `src/components/LibraryPanel.tsx`
- Create: `src/components/LibraryCard.tsx`
- Create: `src/components/SeriesDetail.tsx`
- Modify: `src/App.tsx` (add tab)
- Modify: `src/components/index.ts`

- [ ] **Step 1: Create useLibrary.ts hook**

```tsx
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LibraryEntry, SeriesDetail } from '../types';

export function useLibrary() {
  const [entries, setEntries] = useState<LibraryEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [detail, setDetail] = useState<SeriesDetail | null>(null);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<LibraryEntry[]>('cmd_get_library');
      setEntries(result);
    } catch (e) {
      console.error('Failed to load library:', e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const search = useCallback(async (query: string) => {
    if (!query.trim()) { refresh(); return; }
    try {
      const result = await invoke<LibraryEntry[]>('cmd_search_library', { query });
      setEntries(result);
    } catch (e) {
      console.error('Search failed:', e);
    }
  }, [refresh]);

  const loadDetail = useCallback(async (id: number) => {
    try {
      const result = await invoke<SeriesDetail>('cmd_get_series_detail', { libraryId: id });
      setDetail(result);
    } catch (e) {
      console.error('Load detail failed:', e);
    }
  }, []);

  const remove = useCallback(async (id: number) => {
    try {
      await invoke('cmd_remove_from_library', { libraryId: id });
      setEntries(prev => prev.filter(e => e.id !== id));
      if (detail?.entry.id === id) setDetail(null);
    } catch (e) {
      console.error('Remove failed:', e);
    }
  }, [detail]);

  const closeDetail = useCallback(() => setDetail(null), []);

  return { entries, loading, detail, refresh, search, loadDetail, remove, closeDetail };
}
```

- [ ] **Step 2: Create LibraryCard.tsx**

Minimal card component displaying poster, title, source badge, episode progress. Clickable to open detail.

- [ ] **Step 3: Create SeriesDetail.tsx**

Detail panel with back button, large poster, metadata, episode grid with status indicators (completed/failed/pending).

- [ ] **Step 4: Create LibraryPanel.tsx**

Grid of LibraryCards with search bar, source filter dropdown, and sort selector. When a card is clicked, shows SeriesDetail.

- [ ] **Step 5: Add library tab to App.tsx**

Update `TabType` (line 86):
```ts
type TabType = "download" | "library" | "files" | "history" | "settings" | "logs";
```

Add tab button in the header tabs config, and add the library tab content:
```tsx
{activeTab === "library" && <LibraryPanel />}
```

- [ ] **Step 6: Export new components**

Add all new components to `src/components/index.ts`.

- [ ] **Step 7: Verify build**

Run: `npm run build`
Expected: Frontend compiles with library tab

- [ ] **Step 8: Commit**

```
git add src/hooks/useLibrary.ts src/components/LibraryPanel.tsx src/components/LibraryCard.tsx src/components/SeriesDetail.tsx src/App.tsx src/components/index.ts
git commit -m "feat: add Content Library tab with search, filter, and series detail view"
```

---

## Phase 2: Scheduling & Speed Control + Proxy & Fallback

### Task 8: Create proxy.rs — ProxyConfig, RetryConfig, build_client()

**Files:**
- Create: `src-tauri/src/proxy.rs`
- Modify: `src-tauri/Cargo.toml` (if socks5 feature needed)

- [ ] **Step 1: Create proxy.rs**

```rust
use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyType {
    Http,
    Socks5,
    Direct,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            proxy_type: ProxyType::Direct,
            host: String::new(),
            port: 0,
            username: None,
            password: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryConfig {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub fallback_urls: Vec<String>,
    pub auto_retry: bool,
    pub skip_failed_segments: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 2000,
            fallback_urls: vec![],
            auto_retry: true,
            skip_failed_segments: false,
        }
    }
}

/// Build a reqwest::Client with the given proxy configuration.
pub fn build_client(proxy_config: &ProxyConfig) -> Client {
    let mut builder = Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36");

    match proxy_config.proxy_type {
        ProxyType::Direct => {},
        ProxyType::Http => {
            let url = format!("http://{}:{}", proxy_config.host, proxy_config.port);
            if let Ok(proxy) = Proxy::all(&url) {
                builder = builder.proxy(proxy);
            }
        },
        ProxyType::Socks5 => {
            let url = format!("socks5://{}:{}", proxy_config.host, proxy_config.port);
            if let Ok(proxy) = Proxy::all(&url) {
                builder = builder.proxy(proxy);
            }
        },
    }

    builder.build().unwrap_or_else(|_| Client::new())
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: Commit**

```
git add src-tauri/src/proxy.rs
git commit -m "feat: add ProxyConfig, RetryConfig, and shared build_client helper"
```

---

### Task 9: Migrate parsers to use proxy pattern

**Files:**
- Modify: `src-tauri/src/parser.rs`
- Modify: `src-tauri/src/baanjeen_parser.rs`
- Modify: `src-tauri/src/hsck_parser.rs`
- Modify: `src-tauri/src/njav_parser.rs`
- Modify: `src-tauri/src/njavtv_parser.rs`
- Modify: `src-tauri/src/titan_parser.rs`
- Modify: `src-tauri/src/chrome_detector.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add mod proxy; to lib.rs**

```rust
mod proxy;
```

- [ ] **Step 2: Change parser.rs — RongyokParser**

Replace `client: Client` field with `proxy_config: Arc<RwLock<proxy::ProxyConfig>>`:

```rust
pub struct RongyokParser {
    proxy_config: Arc<RwLock<proxy::ProxyConfig>>,
}

impl RongyokParser {
    fn client(&self) -> Client {
        proxy::build_client(&self.proxy_config.read().unwrap())
    }
    // ... rest of methods use self.client() instead of &self.client
}
```

Apply the same pattern to all 5 other parsers.

- [ ] **Step 3: Update AppState construction**

Initialize parsers with shared proxy config:

```rust
let proxy_config = Arc::new(RwLock::new(proxy::ProxyConfig::default()));
// ... in AppState:
rongyok_parser: RongyokParser { proxy_config: proxy_config.clone() },
baanjeen_parser: BaanJeenParser { proxy_config: proxy_config.clone() },
// ... etc
```

- [ ] **Step 4: Add proxy config commands**

```rust
#[tauri::command]
fn cmd_get_proxy_config(state: State<'_, AppState>) -> Result<proxy::ProxyConfig, String> {
    // Read from AppState proxy_config
}

#[tauri::command]
fn cmd_save_proxy_config(state: State<'_, AppState>, config: proxy::ProxyConfig) -> Result<(), String> {
    // Write to AppState proxy_config (propagates to all parsers automatically)
}

#[tauri::command]
async fn cmd_test_proxy_connection(config: proxy::ProxyConfig) -> Result<bool, String> {
    let client = proxy::build_client(&config);
    match client.get("https://www.google.com").send().await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(e) => Err(format!("Connection failed: {}", e)),
    }
}
```

- [ ] **Step 5: Add proxy args to Chrome detector**

In `chrome_detector.rs`, when building `LaunchOptions`, add:

```rust
if proxy_config.proxy_type != ProxyType::Direct {
    let proxy_arg = format!("--proxy-server={}://{}:{}",
        match proxy_config.proxy_type { Http => "http", Socks5 => "socks5", Direct => "" },
        proxy_config.host, proxy_config.port);
    args.push(proxy_arg);
}
```

Pass proxy_config reference to the detector's methods.

- [ ] **Step 6: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 7: Commit**

```
git add src-tauri/src/parser.rs src-tauri/src/baanjeen_parser.rs src-tauri/src/hsck_parser.rs src-tauri/src/njav_parser.rs src-tauri/src/njavtv_parser.rs src-tauri/src/titan_parser.rs src-tauri/src/chrome_detector.rs src-tauri/src/lib.rs
git commit -m "feat: migrate all parsers to shared proxy config pattern"
```

---

### Task 10: Add retry logic to downloader

**Files:**
- Modify: `src-tauri/src/downloader.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add retry wrapper function**

Add a generic retry helper in downloader.rs:

```rust
async fn retry_request<F, Fut, T>(
    max_retries: u32,
    retry_delay_ms: u64,
    f: F,
) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut last_error = String::new();
    for attempt in 0..=max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = e.clone();
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_millis(retry_delay_ms)).await;
                }
            }
        }
    }
    Err(last_error)
}
```

- [ ] **Step 2: Wrap segment downloads in retry loops**

In `download_hls_manual()`, wrap the segment download request:

```rust
let segment_data = retry_request(retry_config.max_retries, retry_config.retry_delay_ms, || {
    let client = seg_client.clone();
    let url = seg_url.clone();
    async move {
        let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            resp.bytes().await.map_err(|e| e.to_string()).map(|b| b.to_vec())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }
}).await;
```

Same pattern for `download_titan_hls()` segment fetches and `download_direct_file()`.

- [ ] **Step 3: Add retry_config to DownloadConfig**

Add field to `DownloadConfig`:
```rust
pub retry_config: Option<crate::proxy::RetryConfig>,
```

Thread from `start_download()` through `DownloadRequest`.

- [ ] **Step 4: Emit retry events**

Before each retry, emit:
```rust
let _ = app_handle.emit("retry-attempt", serde_json::json!({
    "episode": episode,
    "attempt": attempt + 1,
    "maxRetries": max_retries,
    "error": &last_error
}));
```

- [ ] **Step 5: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 6: Commit**

```
git add src-tauri/src/downloader.rs src-tauri/src/lib.rs
git commit -m "feat: add retry logic with configurable retries and fallback"
```

---

### Task 11: Create scheduler.rs

**Files:**
- Create: `src-tauri/src/scheduler.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create scheduler.rs**

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::watch;
use tokio::time::{self, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleConfig {
    pub enabled: bool,
    pub active_start: Option<String>,   // "HH:MM" local time
    pub active_end: Option<String>,     // "HH:MM" local time
    pub speed_during_active: u64,       // KB/s, 0 = unlimited
    pub speed_outside_active: u64,      // KB/s, 0 = unlimited
    pub auto_pause: bool,
    pub auto_resume: bool,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active_start: None,
            active_end: None,
            speed_during_active: 0,
            speed_outside_active: 0,
            auto_pause: false,
            auto_resume: false,
        }
    }
}

fn parse_time(time_str: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 2 {
        let h: u32 = parts[0].parse().ok()?;
        let m: u32 = parts[1].parse().ok()?;
        if h < 24 && m < 60 { return Some((h, m)); }
    }
    None
}

fn is_active_now(config: &ScheduleConfig) -> bool {
    let start = config.active_start.as_deref().and_then(parse_time);
    let end = config.active_end.as_deref().and_then(parse_time);
    let (start_h, start_m) = match start { Some(t) => t, None => return true };
    let (end_h, end_m) = match end { Some(t) => t, None => return true };

    let now = chrono::Local::now();
    let now_mins = now.hour() * 60 + now.minute();
    let start_mins = start_h * 60 + start_m;
    let end_mins = end_h * 60 + end_m;

    if start_mins > end_mins {
        // Crosses midnight
        now_mins >= start_mins || now_mins < end_mins
    } else {
        now_mins >= start_mins && now_mins < end_mins
    }
}

pub fn start_scheduler(
    config: Arc<std::sync::Mutex<ScheduleConfig>>,
    speed_sender: Arc<watch::Sender<u64>>,
    app_handle: tauri::AppHandle,
) {
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let cfg = config.lock().unwrap().clone();
            if !cfg.enabled { continue; }

            let active = is_active_now(&cfg);
            let speed_limit = if active { cfg.speed_during_active } else { cfg.speed_outside_active };

            let _ = speed_sender.send(speed_limit);
            let _ = app_handle.emit("schedule-status", serde_json::json!({
                "active": active,
                "currentSpeedLimit": speed_limit,
            }));
        }
    });
}
```

- [ ] **Step 2: Add DownloadState watch channels**

Modify `DownloadState` in `downloader.rs`:

```rust
pub struct DownloadState {
    pub is_paused: Arc<std::sync::atomic::AtomicBool>,
    pub is_cancelled: Arc<std::sync::atomic::AtomicBool>,
    pub speed_tx: Arc<tokio::sync::watch::Sender<u64>>,
    pub pause_tx: tokio::sync::watch::Sender<bool>,  // Store sender too
    pub pause_rx: tokio::sync::watch::Receiver<bool>,
}
```

Update `DownloadState::new()`:
```rust
impl DownloadState {
    pub fn new() -> Self {
        let (speed_tx, _) = tokio::sync::watch::channel(0u64);
        let (pause_tx, pause_rx) = tokio::sync::watch::channel(false);
        Self {
            is_paused: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            is_cancelled: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            speed_tx: Arc::new(speed_tx),
            pause_tx,
            pause_rx,
        }
    }
}
```

Update `pause_download`/`resume_download` commands in `lib.rs` to send through `pause_tx`:
```rust
// In pause_download:
state.download_states.lock().unwrap().get(&series_id)
    .map(|ds| { ds.is_paused.store(true, SeqCst); ds.pause_tx.send(true).ok(); });

// In resume_download:
state.download_states.lock().unwrap().get(&series_id)
    .map(|ds| { ds.is_paused.store(false, SeqCst); ds.pause_tx.send(false).ok(); });
```

Update pause loops in download methods to use `pause_rx.changed().await`.

- [ ] **Step 3: Register scheduler in lib.rs**

Add `mod scheduler;`, start scheduler in `run()`, add schedule config commands.

- [ ] **Step 4: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 5: Commit**

```
git add src-tauri/src/scheduler.rs src-tauri/src/downloader.rs src-tauri/src/lib.rs
git commit -m "feat: add background scheduler with dynamic speed control"
```

---

### Task 12: Frontend settings for Phase 2 features

**Files:**
- Modify: `src/components/SettingsPanel.tsx`
- Modify: `src/hooks/useSettings.ts`

- [ ] **Step 1: Add Phase 2 settings fields to useSettings**

Add to `Settings` interface:
```ts
scheduleConfig: ScheduleConfig;
proxyConfig: ProxyConfig;
retryConfig: RetryConfig;
```

Add defaults.

- [ ] **Step 2: Add schedule settings section in SettingsPanel**

Time pickers for active_start/active_end, speed inputs, toggle switches for auto_pause/auto_resume.

- [ ] **Step 3: Add proxy settings section**

Proxy type dropdown, host/port/credentials inputs, test connection button.

- [ ] **Step 4: Add retry settings section**

Max retries slider, retry delay input, mirror URL list with add/remove.

- [ ] **Step 5: Verify build**

Run: `npm run build`

- [ ] **Step 6: Commit**

```
git add src/components/SettingsPanel.tsx src/hooks/useSettings.ts
git commit -m "feat: add settings UI for scheduling, proxy, and retry configuration"
```

---

### Task 13: Integration test — full build verification

**Files:**
- None (verification only)

- [ ] **Step 1: Full Rust build check**

Run: `cd src-tauri && cargo check`
Expected: No errors

- [ ] **Step 2: Frontend build check**

Run: `npm run build`
Expected: No errors

- [ ] **Step 3: Dev mode smoke test**

Run: `npm run tauri dev`
Expected: App launches, library tab visible, settings panel shows new sections

- [ ] **Step 4: Final commit**

```
git commit --allow-empty -m "chore: mark Phase 1 + Phase 2 implementation complete"
```
