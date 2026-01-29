# Architecture

**Analysis Date:** 2026-01-29

## Pattern Overview

**Overall:** Tauri (Rust Backend + React Frontend)

**Key Characteristics:**
- **Asynchronous Execution:** Heavy use of Rust's `tokio` for concurrent downloads and I/O.
- **Event-Driven Communication:** Frontend and Backend communicate via Tauri's `invoke` (commands) and `emit/listen` (real-time progress events).
- **Service-Oriented Logic:** Core logic (parsing, downloading, detection) is encapsulated in dedicated Rust modules.

## Layers

**Frontend (Presentation):**
- Purpose: User interface and application control
- Location: `src/`
- Contains: React components, hooks for state management (settings, history, logger), and Tauri API calls.
- Depends on: `@tauri-apps/api`, `lucide-react`, `vite`.
- Used by: End user.

**Backend (Core Logic):**
- Purpose: Heavy lifting - networking, file system, external process management.
- Location: `src-tauri/src/`
- Contains: Tauri command handlers, video parsers, download manager, and Chrome detector.
- Depends on: `reqwest`, `headless_chrome`, `tokio`, `serde`, `scraper`.
- Used by: Frontend via Tauri bridge.

**Integration Layer:**
- Purpose: Interaction with system tools and external services.
- Location: `src-tauri/src/downloader.rs`, `src-tauri/src/chrome_detector.rs`
- Contains: FFmpeg process management, Headless Chrome orchestration.
- Depends on: System FFmpeg, `headless_chrome` crate.
- Used by: Backend Core Logic.

## Data Flow

**Video Fetching Flow:**

1. User enters URL in `src/App.tsx`.
2. Frontend calls `fetch_series` command in `src-tauri/src/lib.rs`.
3. Backend chooses parser (`BaanJeenParser` or `RongyokParser`).
4. If static parsing fails, `ChromeVideoDetector` launches headless Chrome to find video URLs.
5. `UnifiedSeriesInfo` is returned to Frontend.

**Download Flow:**

1. User selects episodes and clicks Download in `src/App.tsx`.
2. Frontend calls `start_download` command.
3. Backend creates `VideoDownloader` instances and spawns `tokio` tasks.
4. `VideoDownloader` handles HTTP range requests (direct) or FFmpeg streams (HLS).
5. Progress events (`download-progress`) are emitted to Frontend in real-time.
6. Optional: FFmpeg merges downloaded files upon completion.

**State Management:**
- **Backend State:** Managed via Tauri's `State` managed object in `AppState` struct (`src-tauri/src/lib.rs`), protected by `Mutex` for thread safety.
- **Frontend State:** Managed via React `useState` and custom hooks (e.g., `useSettings`, `useHistory`).

## Key Abstractions

**Parsers:**
- Purpose: Abstracting the extraction of series/episode data from specific websites.
- Examples: `src-tauri/src/parser.rs` (`RongyokParser`), `src-tauri/src/baanjeen_parser.rs` (`BaanJeenParser`).
- Pattern: Strategy-like selection based on URL.

**Downloader:**
- Purpose: Unified interface for different download methods (Direct vs HLS).
- Examples: `src-tauri/src/downloader.rs` (`VideoDownloader`).
- Pattern: Handler that delegates to `reqwest` or `ffmpeg` based on URL content.

## Entry Points

**Backend Entry Point:**
- Location: `src-tauri/src/main.rs`
- Triggers: OS application launch.
- Responsibilities: Initializes Tauri builder, sets up managed state, registers command handlers, and starts the event loop.

**Frontend Entry Point:**
- Location: `src/main.tsx`
- Triggers: Tauri Webview initialization.
- Responsibilities: Renders React root, provides internationalization context, and mounts the main `App` component.

## Error Handling

**Strategy:** Results-based error propagation from Rust to TypeScript.

**Patterns:**
- **Rust `Result<T, String>`:** Tauri commands return Result types which are converted to Promise rejections in JS.
- **UI Logs:** `useLogger` hook in `src/App.tsx` captures and displays errors/warnings in a dedicated panel.

## Cross-Cutting Concerns

**Logging:** Centralized `log-info` event emission from Rust to Frontend logger.
**Validation:** `downloader.rs` uses `ffprobe` to validate file integrity after download.
**Authentication:** Not explicitly required for current targets (uses Browser User-Agents).

---

*Architecture analysis: 2026-01-29*
