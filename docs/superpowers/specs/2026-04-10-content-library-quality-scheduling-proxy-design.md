# Feature Design: Content Library, Quality Selection, Scheduling & Proxy

**Date:** 2026-04-10
**Status:** Draft
**Scope:** 4 features delivered in 2 phases
**Phases:**
- Phase 1 — Content Library & Details + Quality Selection
- Phase 2 — Scheduling & Speed Control + Proxy & Fallback

---

## Overview

Enhance Rongyok Video Downloader with four major features: a persistent content library with series metadata and detail views, video quality selection from HLS master playlists, time-based download scheduling with speed profiles, and proxy/mirror server support with automatic fallback.

### Current State

The app supports 6 site parsers (rongyok, njav, njavtv, hsck, titan, baanjeen) with a download engine handling HLS, direct, and encrypted Titan streams. Series info exists only in memory during a session. Downloads always select the highest available quality. Speed limiting exists for direct downloads only (chunk-level sleep). No proxy or retry mechanism exists.

---

## Phase 1

### 1. Content Library & Details

#### 1.1 Library Tab

Add a new `library` tab to the existing tab system.

```
Tab system (current):  download | files | history | settings | logs
Tab system (new):      download | library | files | history | settings | logs
```

The `TabType` union in `App.tsx:86` extends to include `"library"`.

#### 1.2 Library View

Grid of series cards showing:
- Poster image
- Series title
- Source site badge (rongyok, njav, titan, etc.)
- Episode count and download status
- Date added

Features:
- **Filter** by source site, download status (complete/partial/not downloaded)
- **Sort** by date added, title, episode count
- **Search** by series title (client-side fuzzy match)
- **Quick actions** per card: re-download, play last episode, open folder, delete

#### 1.3 Series Detail View

Clicking a library card opens a detail panel (replaces grid, with back button):
- Large poster with metadata (title, source, total episodes, date added)
- Episode grid showing download status per episode (downloaded, partial, not downloaded)
- Actions: download missing episodes, play episode, open containing folder
- Episode-level file paths stored for direct access

#### 1.4 Data Model

**Backend (SQLite via rusqlite):**

```sql
CREATE TABLE library (
    series_id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    source TEXT NOT NULL,           -- "rongyok", "njav", "titan", etc.
    source_url TEXT,
    poster_url TEXT,                -- base64 data URL or cached file path
    total_episodes INTEGER DEFAULT 1,
    date_added TEXT NOT NULL,       -- ISO 8601
    last_downloaded TEXT,
    metadata TEXT                   -- JSON: extra site-specific data
);

CREATE TABLE library_episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_id INTEGER NOT NULL,
    episode_number INTEGER NOT NULL,
    video_url TEXT,                 -- original video URL
    file_path TEXT,                 -- local file path after download
    quality TEXT,                   -- "1080p", "720p", etc.
    file_size INTEGER,
    status TEXT DEFAULT 'pending',  -- pending, downloading, completed, failed
    FOREIGN KEY (series_id) REFERENCES library(series_id) ON DELETE CASCADE,
    UNIQUE(series_id, episode_number)
);
```

**New Tauri commands:**

| Command | Input | Output | Purpose |
|---------|-------|--------|---------|
| `save_to_library` | `UnifiedSeriesInfo` | `()` | Save or update series in library |
| `get_library` | filter, sort, page | `Vec<LibraryEntry>` | List library entries |
| `get_series_detail` | `series_id` | `SeriesDetail` | Full detail with episode status |
| `remove_from_library` | `series_id` | `()` | Remove series and episodes |
| `update_episode_status` | `series_id`, ep, status, path | `()` | Update download status |
| `search_library` | query string | `Vec<LibraryEntry>` | Search by title |

**Data flow:**

```
fetch_series() → UnifiedSeriesInfo
      ↓
   save_to_library() → SQLite
      ↓
   Library tab reads via get_library()
      ↓
   Click series → get_series_detail() → detail view
      ↓
   Download → update_episode_status() on completion
```

**Frontend components to add:**

| Component | Purpose |
|-----------|---------|
| `LibraryPanel` | Main library grid view with filters |
| `LibraryCard` | Individual series card in grid |
| `SeriesDetail` | Detail view with episode status |
| `LibrarySearch` | Search bar with filter dropdowns |

**Integration with existing download flow:**
- After `fetch_series()` succeeds, automatically call `save_to_library()`
- After each episode download completes (in `download-result` handler), call `update_episode_status()`
- History panel remains separate (tracks download sessions); library tracks content

#### 1.5 Storage

SQLite database stored in Tauri app data directory (`app_data_dir/library.db`).

Dependency: `rusqlite` with `bundled` feature in `Cargo.toml`.

---

### 2. Quality Selection

#### 2.1 Overview

Allow users to select video quality from HLS master playlists before downloading. Currently, `download_hls_manual()` always selects the highest bandwidth variant.

#### 2.2 Quality Options API

**New Rust struct:**

```rust
#[derive(Serialize, Deserialize)]
struct QualityOption {
    resolution: String,       // "1920x1080"
    bandwidth: u64,           // 5000000
    label: String,            // "1080p (5.0 Mbps)"
    stream_url: String,       // sub-playlist URL for this quality
}

#[derive(Serialize, Deserialize)]
struct QualityInfo {
    qualities: Vec<QualityOption>,
    default_index: usize,     // index of recommended quality
}
```

**New Tauri command:**

```rust
#[tauri::command]
async fn get_quality_options(url: String) -> Result<QualityInfo, String>
```

This command:
1. Fetches the URL
2. If it's an HLS master playlist (contains `#EXT-X-STREAM-INF`), parses all variants
3. Returns sorted quality options (highest first)
4. For non-HLS URLs, returns a single option: `{"resolution": "original", ...}`

#### 2.3 Backend Changes

**In `downloader.rs`:**

- Extract variant parsing logic from `download_hls_manual()` into a shared function: `parse_master_playlist(html: &str) -> Vec<QualityOption>`
- `download_hls_manual()` and `download_hls_stream()` accept an optional `preferred_quality: Option<String>` parameter (resolution string like "1920x1080")
- When `preferred_quality` is set, match against parsed variants instead of always selecting max bandwidth
- When `None`, use current behavior (highest quality)

**In `lib.rs`:**

- `DownloadRequest` gains `preferred_quality: Option<String>` field
- `start_download` passes quality preference through to `VideoDownloader`

#### 2.4 Frontend Changes

**New component: `QualitySelector`**

- Appears after series is fetched, before download starts
- Dropdown showing available qualities (fetched lazily from first episode URL)
- "Apply to all episodes" toggle
- Options: "Best available (default)", "1080p", "720p", "480p", etc.
- Falls back to "Best available" if HLS parsing fails or URL is direct MP4

**Quality selector placement in download flow:**

```
Fetch series → Episode selector → Quality selector → Download button
                                     ^^^^
                                     NEW
```

**Default behavior:** "Best available" — identical to current behavior. Users who don't care about quality never see a difference.

#### 2.5 Settings Integration

Add to `Settings` in `useSettings.ts`:

```ts
defaultQuality: string;  // "best" | "1080p" | "720p" | "480p" | "ask"
```

- `"best"` — always highest quality (current default)
- `"ask"` — show quality selector each time
- Specific resolution — always use that quality when available

---

## Phase 2

### 3. Scheduling & Speed Control

#### 3.1 Time-Based Scheduling

**Data structure:**

```rust
#[derive(Serialize, Deserialize, Clone)]
struct ScheduleConfig {
    enabled: bool,
    active_start: Option<String>,   // "22:00" (local time)
    active_end: Option<String>,     // "08:00" (local time)
    speed_during_active: u64,       // KB/s, 0 = unlimited
    speed_outside_active: u64,      // KB/s, 0 = unlimited
    auto_pause: bool,               // pause downloads outside active hours
    auto_resume: bool,              // auto-resume when entering active hours
}
```

**Backend:**

- Background tokio task checks schedule every 30 seconds
- When transitioning between active/inactive:
  - If `auto_pause`: sends pause signal to all active downloads
  - If `auto_resume`: sends resume signal to paused downloads
- Dynamically adjusts `speed_limit_kbps` on active downloads
- Emits `schedule-status` event: `{ active: bool, next_transition: String, current_speed_limit: u64 }`

**Frontend:**

- Schedule settings section in `SettingsPanel`
- Schedule status badge in header (shows active/inactive + next transition time)
- Speed graph overlay showing current schedule state

**Persistence:** Stored in SQLite alongside library data, or as a separate JSON config file in app data directory.

#### 3.2 Speed Control Improvements

**Direct downloads (existing):**
- Already supports speed limiting via chunk-level sleep
- No changes needed

**HLS via FFmpeg (`download_hls_stream`):**
- Add FFmpeg `-rate_limit` flag when speed limit is set
- Formula: `-rate_limit {speed_limit_kbps * 1024}` bytes per second
- This is a FFmpeg-native feature, no manual chunking needed

**HLS manual (`download_hls_manual`):**
- Add per-segment download delay based on speed limit
- Similar approach to direct download: measure elapsed time, sleep if ahead of pace

**Scheduling integration:**
- `DownloadConfig` gains `schedule: Option<ScheduleConfig>`
- Speed limit is dynamically adjustable (not just at download start)
- New Tauri command: `update_speed_limit(kbps: u64)` — updates running downloads

#### 3.3 Pause/Resume Improvements

Replace busy-wait pause loop with `tokio::sync::watch` channel:

```rust
struct DownloadState {
    is_paused: Arc<AtomicBool>,
    is_cancelled: Arc<AtomicBool>,
    speed_limit_kbps: Arc<watch::Sender<u64>>,  // NEW: dynamic speed control
    pause_signal: watch::Receiver<bool>,          // NEW: event-driven pause
}
```

Benefits:
- No polling overhead during pause
- Dynamic speed limit changes propagate instantly
- Cleaner cancellation path

---

### 4. Proxy & Fallback Servers

#### 4.1 Proxy Support

**Data structure:**

```rust
#[derive(Serialize, Deserialize, Clone)]
struct ProxyConfig {
    proxy_type: ProxyType,   // Http, Socks5, Direct
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
enum ProxyType {
    Http,
    Socks5,
    Direct,   // no proxy
}
```

**Integration points — all reqwest::Client builders must use proxy:**

| Location | File | Current | Change |
|----------|------|---------|--------|
| RongyokParser::new() | parser.rs | `Client::builder()` | Read proxy from AppState |
| BaanJeenParser | baanjeen_parser.rs | `Client::builder()` | Same |
| HsckParser | hsck_parser.rs | `Client::builder()` | Same |
| NjavParser | njav_parser.rs | `Client::builder()` | Same |
| NjavtvParser | njavtv_parser.rs | `Client::builder()` | Same |
| TitanParser | titan_parser.rs | `Client::builder()` | Same |
| VideoDownloader | downloader.rs | per-request client | Same |
| ChromeVideoDetector | chrome_detector.rs | LaunchOptions | Add `--proxy-server` arg |

**Implementation approach:**
1. Store `ProxyConfig` in `AppState` behind `RwLock`
2. Create a helper function `build_client(proxy: &ProxyConfig) -> reqwest::Client` used by all parsers
3. Parsers receive proxy config at construction or via method parameter
4. Chrome detector adds `--proxy-server={type}://{host}:{port}` to launch args

**New Tauri commands:**

| Command | Purpose |
|---------|---------|
| `get_proxy_config` | Read current proxy settings |
| `save_proxy_config` | Persist proxy settings |
| `test_proxy_connection` | Test connectivity through configured proxy |

**Frontend:**
- Proxy settings section in `SettingsPanel`
- Connection test button with status indicator
- Proxy type dropdown (HTTP/SOCKS5/Direct)
- Host/port/credentials fields

#### 4.2 Fallback & Retry

**Data structure:**

```rust
#[derive(Serialize, Deserialize, Clone)]
struct RetryConfig {
    max_retries: u32,               // per-segment retry count (default: 3)
    retry_delay_ms: u64,            // delay between retries (default: 2000)
    fallback_urls: Vec<String>,     // mirror URLs to try on failure
    auto_retry: bool,               // enable automatic retry (default: true)
    skip_failed_segments: bool,     // for HLS: skip bad segments (default: false)
}
```

**Retry flow:**

```
Download segment/episode
  → On failure:
    1. Wait retry_delay_ms
    2. Retry same URL (up to max_retries times)
       → On persistent failure:
         3. Try each fallback_urls entry in order
            → On all fail:
              4. If skip_failed_segments && HLS: log warning, continue
              5. Else: report error for this episode
```

**Integration in `downloader.rs`:**

- `download_hls_manual()`: wrap segment download in retry loop
- `download_direct_file()`: wrap entire download in retry loop with Range header resume
- `download_titan_hls()`: wrap segment fetch in retry loop
- Fallback URLs: replace base URL domain with mirror domain

**Retry events emitted to frontend:**

```
retry-attempt { episode, attempt, max_retries, error }
fallback-used { episode, original_url, fallback_url }
```

**Frontend:**
- Retry settings section in `SettingsPanel`
- Retry status in download progress (shows "Retrying... (2/3)")
- Mirror URL management (add/remove/reorder)

---

## Architecture Decisions

### Storage: SQLite

Chosen over JSON files for:
- Efficient querying (filter by source, sort by date)
- Concurrent read/write safety
- Future scalability (bookmarks, playlists, watch history)
- rusqlite with bundled feature (no external SQLite dependency)

### Parser Proxy Propagation

Rather than modifying each parser individually, create a shared `build_client()` helper:
- Single point of change for proxy logic
- All parsers use the same client construction path
- Easy to add future client configuration (custom headers, timeouts)

### Quality Selection: Lazy Fetch

Quality options are fetched on-demand (when user opens quality selector) rather than during `fetch_series()`:
- Avoids N extra requests for N episodes
- Most users won't change quality, so no wasted work
- First episode URL is used as representative sample

### Scheduling: Background Tokio Task

A lightweight background task checks schedule state every 30 seconds:
- Minimal CPU overhead
- Immediate response to schedule transitions
- Decoupled from download logic (schedule can change independently)

---

## Files to Create/Modify

### Phase 1

**New files:**
- `src-tauri/src/library.rs` — SQLite library storage + Tauri commands
- `src/components/LibraryPanel.tsx` — Library grid view
- `src/components/LibraryCard.tsx` — Individual series card
- `src/components/SeriesDetail.tsx` — Series detail view
- `src/components/QualitySelector.tsx` — Quality dropdown
- `src/hooks/useLibrary.ts` — Library state management hook

**Modified files:**
- `src-tauri/Cargo.toml` — Add rusqlite dependency
- `src-tauri/src/lib.rs` — Register new commands, add AppState fields, auto-save to library
- `src-tauri/src/downloader.rs` — Extract playlist parser, add quality parameter
- `src/App.tsx` — Add library tab, integrate QualitySelector
- `src/types/index.ts` — Add LibraryEntry, SeriesDetail, QualityOption types

### Phase 2

**New files:**
- `src-tauri/src/scheduler.rs` — Background schedule task
- `src-tauri/src/proxy.rs` — Proxy config management + client builder helper

**Modified files:**
- `src-tauri/src/lib.rs` — Register schedule/proxy commands, AppState additions
- `src-tauri/src/downloader.rs` — Add retry logic, dynamic speed, pause improvements
- `src-tauri/src/parser.rs` — Use shared client builder
- `src-tauri/src/titan_parser.rs` — Use shared client builder
- `src-tauri/src/baanjeen_parser.rs` — Use shared client builder
- `src-tauri/src/hsck_parser.rs` — Use shared client builder
- `src-tauri/src/njav_parser.rs` — Use shared client builder
- `src-tauri/src/njavtv_parser.rs` — Use shared client builder
- `src-tauri/src/chrome_detector.rs` — Add proxy args
- `src/components/SettingsPanel.tsx` — Add schedule, proxy, retry settings sections
- `src/hooks/useSettings.ts` — Add schedule/proxy/retry settings
- `src/types/index.ts` — Add ScheduleConfig, ProxyConfig, RetryConfig types
