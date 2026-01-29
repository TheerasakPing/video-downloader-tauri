# Codebase Concerns

**Analysis Date:** 2026-01-29

## Tech Debt

**Giant Central Component:**
- Issue: `App.tsx` is a "God Component" handling state for downloads, UI tabs, logs, settings, history, and updates.
- Files: `src/App.tsx` (1303 lines)
- Impact: Extremely difficult to maintain, test, or modify without side effects.
- Fix approach: Refactor into specialized providers or separate feature components.

**Chrome Detector Reliability:**
- Issue: The `ChromeVideoDetector` relies on hardcoded timeouts (`std::thread::sleep`) and aggressive clicking of common selectors. This is fragile as site layouts change or network speeds vary.
- Files: `src-tauri/src/chrome_detector.rs`
- Impact: Detection may fail on slower connections or if the site changes its player DOM structure.
- Fix approach: Replace fixed sleeps with `wait_until` logic for specific elements or network conditions using `headless_chrome` capabilities.

**Headless Chrome Lifecycle:**
- Issue: The browser instance is managed via `Arc<Browser>` in a `Mutex` but the initialization and cleanup logic is complex, including manual retries and restarts on "connection closed" errors.
- Files: `src-tauri/src/chrome_detector.rs`, `src-tauri/src/lib.rs`
- Impact: Potential for orphaned browser processes or memory leaks if cleanup fails or if multiple concurrent requests conflict.
- Fix approach: Implement a more robust singleton or pool pattern for the browser instance with explicit health checks.

**Dynamic Script Injection:**
- Issue: The system injects a large `PerformanceObserver` script and a click script into the browser context. This is hard to maintain and debug.
- Files: `src-tauri/src/chrome_detector.rs`
- Impact: JavaScript errors in the injected scripts are hard to capture; changes to the browser environment (e.g., CSP) might block injection.
- Fix approach: Move JavaScript logic to separate `.js` files in resources or use `headless_chrome`'s native network interception if possible.

**Duplicated Path Expansion Logic:**
- Issue: Logic to expand `~/` to home directory is implemented manually in multiple places.
- Files: `src-tauri/src/lib.rs`, `src-tauri/src/downloader.rs`
- Impact: Inconsistent path handling if one implementation is updated and not the other.
- Fix approach: Create a shared utility function in a `utils` or `filesystem` module.

**Type Safety Bypasses:**
- Issue: Use of `@ts-ignore` for dynamic setting updates.
- Files: `src/App.tsx:165`
- Impact: Potential runtime errors if setting keys change or are invalid.
- Fix approach: Use proper TypeScript index signatures or a type-safe settings mapping.

## Known Bugs

**FFmpeg Not Found on Windows/Linux Release:**
- Issue: `get_ffmpeg_command` tries to find `ffmpeg` in various locations but may fail if the binary is not correctly bundled as a sidecar or in the `Resources` folder.
- Files: `src-tauri/src/downloader.rs`
- Symptoms: "FFmpeg is required for .m3u8 downloads but was not found" error.
- Workaround: Install `ffmpeg` system-wide or manually place it in the app directory.

**BaanJeen URL Parsing Complexity:**
- Issue: Static parsing often fails due to dynamic loading, requiring the "Hybrid God Mode" fallback.
- Files: `src-tauri/src/baanjeen_parser.rs`, `src-tauri/src/lib.rs`
- Symptoms: First fetch attempt often returns 0 episodes, necessitating a second, slower Chrome-based detection.
- Workaround: Use the "Hybrid Mode" automatically (already implemented but slow).

## Security Considerations

**Unsafe Launch Arguments:**
- Risk: `--no-sandbox` and `--disable-dev-shm-usage` are used to launch Chrome. While often necessary in Docker/constrained environments, it reduces browser isolation.
- Files: `src-tauri/src/chrome_detector.rs`
- Current mitigation: Headless mode is active; the browser only visits user-provided URLs.
- Recommendations: Investigate if sandbox can be enabled on target platforms (macOS/Windows) while only disabling it on Linux if needed.

**Arbitrary Code Execution in Browser:**
- Risk: The application navigates to user-provided URLs in a full browser engine. Malicious sites could potentially exploit the browser.
- Files: `src-tauri/src/chrome_detector.rs`
- Current mitigation: Injected scripts are limited in scope.
- Recommendations: Ensure the browser process has minimal permissions.

## Performance Bottlenecks

**Large State Object in Frontend:**
- Issue: React state updates for every progress event (percentage, speed) cause frequent re-renders of the massive `App.tsx`.
- Files: `src/App.tsx`
- Problem: UI lag during high-speed downloads or when many episodes are in the queue.
- Cause: Progress events are emitted frequently from Rust and caught by a single state in the root.
- Improvement path: Use a specialized store (like Zustand) or context to isolate progress updates from the rest of the UI.

**Serial Episode Download Chunks:**
- Problem: Downloads are processed in chunks of `concurrent_downloads` size, but the outer loop is serial.
- Files: `src-tauri/src/lib.rs` (in `start_download`)
- Cause: Chunks are awaited before starting the next set.
- Improvement path: Use a proper task queue (e.g., `tokio::sync::mpsc`) to maintain a constant number of concurrent downloads.

**Duplicate Image Fetching:**
- Problem: `fetch_image_as_data_url` is called during every metadata fetch, downloading and base64-encoding the poster even if it's already cached.
- Files: `src-tauri/src/baanjeen_parser.rs`, `src-tauri/src/parser.rs`
- Cause: Poster is fetched as part of the unified info object.
- Improvement path: Only fetch poster on demand or check if it's already in the frontend cache.

## Fragile Areas

**Regex-Based URL Extraction:**
- Files: `src-tauri/src/chrome_detector.rs`, `src-tauri/src/baanjeen_parser.rs`, `src-tauri/src/parser.rs`
- Why fragile: Relies on matching Discord CDN or `.m3u8` patterns in HTML/JS strings. Changes in URL formats or site obfuscation will break this.
- Safe modification: Add comprehensive tests for all known URL formats.
- Test coverage: Minimal tests for URL parsing in `parser.rs`.

**FFmpeg Progress Parsing:**
- Files: `src-tauri/src/downloader.rs`
- Why fragile: Parses stderr strings (`time=XX:XX:XX.XX`) from FFmpeg output. Output format may vary between FFmpeg versions.
- Safe modification: Use `-progress` flag with FFmpeg to get machine-readable output.
- Test coverage: No unit tests for `parse_ffmpeg_time`.

## Scaling Limits

**Concurrent Chrome Instances:**
- Current capacity: Single browser instance (Mutex protected).
- Limit: Sequential detection for multiple iframes or concurrent fetch requests.
- Scaling path: Implement a pool of browser tabs or separate instances.

## Missing Critical Features

**Blob URL Support:**
- Problem: `blob:` video URLs cannot be downloaded directly via `reqwest` or `ffmpeg`.
- Blocks: Downloading from sites that use `blob:` URLs for their players.
- Priority: Medium (noted in `TODO.md`).

## Test Coverage Gaps

**Rust Backend Testing:**
- What's not tested: Core logic for `VideoDownloader` and parsers.
- Files: `src-tauri/src/*.rs`
- Risk: Regressions in download logic or parsing failure go unnoticed until runtime.
- Priority: High.

---

*Concerns audit: 2026-01-29*
