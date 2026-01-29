# External Integrations

**Analysis Date:** 2026-01-29

## APIs & External Services

**Video Hosting & Content Sources:**
- `rongyok.com` / `thongyok.com` - Targeted for series metadata and video URL extraction (`src-tauri/src/parser.rs`)
- `xn--82c7abb4jua0l.com` (บ้านจีน.com) - Targeted for series metadata and video URL extraction (`src-tauri/src/baanjeen_parser.rs`)
- Embedded players (e.g., `baiwarp.com`) - Targeted for HLS stream URL detection via headless browser

**Software Updates:**
- GitHub Releases - Endpoint for Tauri's auto-updater plugin
  - Endpoint: `https://github.com/TheerasakPing/video-downloader-tauri/releases/latest/download/latest.json`
  - Implementation: `src-tauri/tauri.conf.json`

## Data Storage

**Databases:**
- None active - The application is stateless; `my-database.db` exists in the root but is not referenced in the active Rust source code.

**File Storage:**
- Local Filesystem - Downloads are saved to user-selected directories.
- Implementation: `src-tauri/src/downloader.rs`

**Caching:**
- Local App Cache - Used for temporary file storage during downloads and update artifacts.
- Implementation: `src-tauri/src/lib.rs` (TMPDIR redirection on macOS)

## Authentication & Identity

**Auth Provider:**
- None - Accesses public content.
- Implementation: Mimics browser sessions using `User-Agent` and `Referer` headers in HTTP requests (`src-tauri/src/downloader.rs`).

## Monitoring & Observability

**Error Tracking:**
- None - Relies on local application logs.

**Logs:**
- Tauri Events - Rust backend emits `log-info`, `download-progress`, and `merge-progress` events to the React frontend for UI feedback.
- Implementation: `src-tauri/src/lib.rs` and `src-tauri/src/downloader.rs`

## CI/CD & Deployment

**Hosting:**
- GitHub Releases - Distribution of platform-specific binaries.

**CI Pipeline:**
- GitHub Actions - Workflow defined in `.github/workflows/release.yml`.

## Environment Configuration

**Required env vars:**
- `TMPDIR` (macOS) - Dynamically set at runtime to the application's cache directory to avoid cross-device link issues.

**Secrets location:**
- Not applicable - No cloud secrets or API keys are embedded in the client application.

## Webhooks & Callbacks

**Incoming:**
- None

**Outgoing:**
- None

## Multimedia Tools

**Processing Engine:**
- `ffmpeg` - External binary used for HLS stream capturing and segment merging.
- `ffprobe` - External binary used for video validation and duration extraction.
- Implementation: Binaries are located in `src-tauri/resources/bin/` and invoked as sidecars/external processes.

---

*Integration audit: 2026-01-29*
