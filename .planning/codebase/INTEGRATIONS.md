# External Integrations

**Analysis Date:** 2026-01-29

## APIs & External Services

**Video Hosting/Streaming:**
- `xn--82c7abb4jua0l.com` (Rongyok) - Primary source for video content scraping.
- `baanjeen` - Secondary source for video content scraping.
- `baiwarp.com` / `play.baiwarp.com` - Embedded video players targeted for URL extraction.
- `media.vdohls.com` - CDN/Streaming endpoint for video assets.

**Update Service:**
- GitHub Releases - Used as the endpoint for the Tauri updater (`src-tauri/tauri.conf.json`).
  - Endpoint: `https://github.com/TheerasakPing/video-downloader-tauri/releases/latest/download/latest.json`

## Data Storage

**Databases:**
- None - The application is stateless regarding databases; it operates directly on the filesystem.

**File Storage:**
- Local Filesystem - Downloads are saved to user-defined directories.
- Temporary Directory - Used for intermediate video chunks before merging.

**Caching:**
- Local App Cache - Used for temporary storage and update artifacts (`src-tauri/src/lib.rs`).

## Authentication & Identity

**Auth Provider:**
- None - The targeted sites appear to be public or handled via session/cookie extraction (though no complex auth logic was found in the core parsers).

## Monitoring & Observability

**Error Tracking:**
- None - Standard logging to console/stderr.

**Logs:**
- Tauri Events - Progress and errors are emitted from Rust to the Frontend via `Emitter`.

## CI/CD & Deployment

**Hosting:**
- GitHub Releases - Distribution of binaries.

**CI Pipeline:**
- Not detected (likely standard GitHub Actions for Tauri).

## Environment Configuration

**Required env vars:**
- `TMPDIR` - Set internally on macOS to redirect temporary files to the app cache.

**Secrets location:**
- Not applicable - No API keys or secrets found in the codebase. Public scraping methodology.

## Webhooks & Callbacks

**Incoming:**
- None.

**Outgoing:**
- None.

## Tools & Binaries

**Multimedia:**
- `ffmpeg` - Required for HLS stream downloading and merging TS segments into MP4.
- `ffprobe` - Used for validating video files and extracting duration.

---

*Integration audit: 2026-01-29*
