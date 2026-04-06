---
session: ses_29e5
updated: 2026-04-06T08:01:45.897Z
---

# Session Summary

## Goal
Fix end-to-end NjavTV video downloads by passing Chrome-detected authentication cookies through the entire detection → download pipeline so that CDN requests for obfuscated `.jpeg` HLS segments succeed.

## Constraints & Preferences
- User directive: "ทำจนกว่าจะได้" (keep working until it succeeds)
- Test URL: `https://njavtv.com/th/dass-812-uncensored-leak`
- Cookie storage pattern: `Vec<(String, String)>` tuples passed through the chain
- Fallback downloader (`download_hls_manual`) is the primary execution path for NjavTV due to FFmpeg rejecting disguised segment extensions

## Progress
### Done
- [x] Added `extract_cookies()` method to `ChromeVideoDetector` to evaluate `document.cookie` and store in `self.last_cookies`
- [x] Called `self.extract_cookies(tab)` in NjavTV detection path (`chrome_detector.rs` ~line 253) right after finding `window.hls.url`
- [x] Added `cookies: Vec<(String, String)>` field to `UnifiedSeriesInfo` struct in `lib.rs`
- [x] Updated all 6 `UnifiedSeriesInfo` instantiations in `fetch_series` to include the `cookies` field
- [x] Modified `start_download` in `lib.rs` to pass `series.cookies.clone()` to `download_episode`
- [x] Updated `download_episode` and `download_hls_stream` signatures in `downloader.rs` to accept `cookies` parameter
- [x] Added Cookie header construction to FFmpeg `-headers` argument in `download_hls_stream`
- [x] Updated `download_hls_manual` signature to accept `cookies: &[(String, String)]` parameter
- [x] Added Cookie header injection to the reqwest `seg_client` builder in `download_hls_manual`
- [x] Verified call site at `downloader.rs:157` correctly passes `cookies` to `download_hls_manual`

### In Progress
- [ ] Compile verification (`cargo build --release`)
- [ ] End-to-end testing with the target NjavTV URL

### Blocked
- (none)

## Key Decisions
- **Cookie format as `Vec<(String, String)>`**: Chosen over `HashMap` to preserve insertion order (some CDNs validate order) and align with Chrome's native cookie array format
- **FFmpeg Cookie header via `-headers`**: FFmpeg accepts `\r\n`-separated headers; cookies are formatted as `Cookie: k1=v1; k2=v2\r\n` and appended
- **Reqwest cookies via `default_headers`**: For `download_hls_manual`, cookies are added to the `ClientBuilder` so every HTTP segment request automatically includes the `Cookie` header
- **Single extraction point**: `extract_cookies()` is called once per episode detection after the m3u8 URL is resolved, avoiding redundant JS evaluations

## Next Steps
1. Run `cd src-tauri && cargo build --release` to verify compilation
2. Launch the Tauri app and test with `https://njavtv.com/th/dass-812-uncensored-leak`
3. Monitor console logs for `[ManualHLS]` and `Cookie:` header presence to confirm authentication bypass
4. If download still fails, inspect raw HTTP responses to verify cookie validity and adjust header formatting if needed

## Critical Context
- **Root Cause**: NjavTV CDN returns fake JPEG images instead of video segments when requests lack valid session cookies
- **Data Flow**: `fetch_series` → `detector.get_last_cookies()` → `UnifiedSeriesInfo.cookies` → `start_download` → `download_episode(cookies)` → `download_hls_manual(cookies)`
- **`download_hls_manual` Role**: Fallback downloader that manually fetches each segment via reqwest, concatenates them into a `.ts` file, and converts to `.mp4` via FFmpeg. This is the critical path for NjavTV.
- **Pre-existing LSP Warning**: `lib.rs:783` has an `unsafe function` warning for `std::env::set_var` — unrelated to current changes

## File Operations
### Read
- `/Volumes/Data/_workspace/_git/video-downloader-tauri/src-tauri/src/downloader.rs`
- `/Volumes/Data/_workspace/_git/video-downloader-tauri/src-tauri/src/lib.rs`
- `/Volumes/Data/_workspace/_git/video-downloader-tauri/src-tauri/src/chrome_detector.rs`

### Modified
- `/Volumes/Data/_workspace/_git/video-downloader-tauri/src-tauri/src/downloader.rs` — Updated `download_hls_manual` signature to accept `cookies: &[(String, String)]`; injected Cookie header into `seg_client` default headers
- `/Volumes/Data/_workspace/_git/video-downloader-tauri/src-tauri/src/lib.rs` — Added `cookies` field to `UnifiedSeriesInfo`, threaded cookies through `start_download` → `download_episode`
- `/Volumes/Data/_workspace/_git/video-downloader-tauri/src-tauri/src/chrome_detector.rs` — Added `extract_cookies()` method and integrated it into the NjavTV detection flow
