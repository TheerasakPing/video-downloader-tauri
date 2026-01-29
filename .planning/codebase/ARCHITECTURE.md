# Architecture

**Analysis Date:** 2026-01-29

## Pattern Overview

**Overall:** Tauri-based Desktop Application (Rust Backend + React Frontend)

**Key Characteristics:**
- **Polyglot:** Combines Rust for performance-critical tasks (downloading, parsing, headless browser control) and TypeScript/React for the UI.
- **Event-Driven:** Uses Tauri's event system (`emit`/`listen`) for real-time progress updates from the Rust backend to the React frontend.
- **Asynchronous:** Heavily relies on `tokio` in Rust for concurrent downloads and `async/await` in TypeScript for responsive UI.

## Layers

**Frontend (Presentation):**
- Purpose: User interface and application state management.
- Location: `src/`
- Contains: React components, hooks, and types.
- Depends on: Tauri API (@tauri-apps/api), Lucide React (icons), Tailwind CSS (styling).
- Used by: End user.

**Tauri Bridge (API):**
- Purpose: Facilitates communication between Frontend and Backend.
- Location: `src-tauri/src/lib.rs`
- Contains: `#[tauri::command]` functions that are invoked from the frontend.
- Depends on: Backend logic modules.
- Used by: Frontend via `invoke()`.

**Backend (Business Logic):**
- Purpose: Core functionality like web scraping, video downloading, and file management.
- Location: `src-tauri/src/`
- Contains: `downloader.rs` (logic), `parser.rs` (Rongyok), `baanjeen_parser.rs` (BaanJeen), `chrome_detector.rs` (Headless browser).
- Depends on: `reqwest` (HTTP), `tokio` (Async), `headless_chrome`, `scraper` (HTML parsing), `ffmpeg` (external tool).
- Used by: Tauri Bridge.

## Data Flow

**Download Flow:**

1. **User Input:** User enters a URL in `App.tsx`.
2. **Fetch Info:** Frontend invokes `fetch_series` command. Backend (`parser.rs` or `baanjeen_parser.rs`) scrapes the site and returns `UnifiedSeriesInfo`.
3. **Selection:** User selects episodes and clicks "Download".
4. **Start Download:** Frontend invokes `start_download` with a `DownloadRequest`.
5. **Execution:** Backend (`downloader.rs`) spawns `tokio` tasks for each episode, using `reqwest` for direct downloads or `ffmpeg` for HLS streams.
6. **Progress Updates:** Backend emits `download-progress` events. Frontend (`App.tsx`) listens and updates state.
7. **Completion/Merge:** Once episodes are finished, if `autoMerge` is enabled, backend invokes `ffmpeg` to concatenate files and emits `merge-complete`.

**State Management:**
- **Frontend:** Managed via React `useState`, `useEffect`, and custom hooks (`useSettings`, `useHistory`, etc.).
- **Backend:** Managed via an `AppState` struct stored in Tauri's managed state, using `Mutex` for thread-safe access to shared components (parsers, downloader, detector).

## Key Abstractions

**VideoDownloader:**
- Purpose: Handles the actual file transfer and progress tracking.
- Examples: `src-tauri/src/downloader.rs`
- Pattern: Command pattern (encapsulates a download task).

**SeriesParser (Trait-like):**
- Purpose: Common interface (though not an explicit Rust trait here, they follow similar patterns) for different video source sites.
- Examples: `src-tauri/src/parser.rs`, `src-tauri/src/baanjeen_parser.rs`.

**UnifiedSeriesInfo:**
- Purpose: A shared data structure that normalizes series data from different sources for the frontend.
- Examples: Defined in `src-tauri/src/lib.rs`.

## Entry Points

**Backend Entry Point:**
- Location: `src-tauri/src/main.rs`
- Triggers: Application launch.
- Responsibilities: Calls `tauri_app_lib::run()` which initializes the Tauri app and sets up managed state and command handlers.

**Frontend Entry Point:**
- Location: `src/main.tsx`
- Triggers: Webview initialization.
- Responsibilities: Renders the React `App` component into the DOM.

## Error Handling

**Strategy:** Result-based error propagation in Rust, caught and displayed as notifications/logs in the Frontend.

**Patterns:**
- **Rust `Result<T, String>`:** Command functions return `Result` which Tauri automatically translates to Promise resolve/reject in JS.
- **Frontend Logger:** A custom `useLogger` hook catches errors and displays them in a dedicated log panel.

## Cross-Cutting Concerns

**Logging:** Backend uses `eprintln!` and emits `log-info` events to the frontend's `LogPanel`.
**Validation:** `downloader.rs` uses `ffprobe` to validate the integrity of downloaded video files.
**Authentication:** Not explicitly required for the target sites, but `reqwest` clients use specific User-Agents to mimic browsers.

---

*Architecture analysis: 2026-01-29*
