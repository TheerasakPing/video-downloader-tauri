# Codebase Concerns

**Analysis Date:** 2026-01-29

## Tech Debt

**Giant Central Component:**
- Issue: `App.tsx` is a "God Component" handling state for downloads, UI tabs, logs, settings, history, and updates.
- Files: `src/App.tsx` (1303 lines)
- Impact: Extremely difficult to maintain, test, or modify without side effects.
- Fix approach: Refactor into specialized providers or separate feature components.

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

**Placeholder URLs:**
- Issue: Example URLs still contain `XXX` placeholders which might be confusing for users or indicate incomplete logic.
- Files: `src/App.tsx:978`, `src/hooks/useI18n.tsx:26`, `src-tauri/src/parser.rs:34`
- Symptoms: User sees `series_id=XXX` in placeholder text.
- Trigger: Default state of the download input.

## Security Considerations

**External Process Execution (FFmpeg):**
- Issue: High reliance on external FFmpeg binary for HLS streams and merging.
- Files: `src-tauri/src/downloader.rs`
- Risk: Command injection if filename sanitization or URL validation is bypassed (though current use of `.args()` mitigates this).
- Current mitigation: `sanitize_filename` is used for output paths.
- Recommendations: Strict validation of the `video_url` before passing to FFmpeg.

**Browser Automation (Chrome Detector):**
- Issue: Uses headless browser logic to detect video URLs.
- Files: `src-tauri/src/chrome_detector.rs`
- Risk: Processing untrusted URLs in a browser environment can expose the system to web-based exploits.
- Recommendations: Ensure the headless browser instance is strictly sandboxed and has a timeout.

## Performance Bottlenecks

**Large State Object in Frontend:**
- Issue: React state updates for every progress event (percentage, speed) cause frequent re-renders of the massive `App.tsx`.
- Files: `src/App.tsx`
- Problem: UI lag during high-speed downloads or when many episodes are in the queue.
- Cause: Progress events are emitted frequently from Rust and caught by a single state in the root.
- Improvement path: Use a specialized store (like Zustand) or context to isolate progress updates from the rest of the UI.

## Fragile Areas

**Site-Specific Parsers:**
- Files: `src-tauri/src/parser.rs`, `src-tauri/src/baanjeen_parser.rs`
- Why fragile: URL parsing and metadata extraction rely on specific string patterns and HTML structures of target sites.
- Safe modification: Any changes to the target websites' layout or URL structure will break these modules.
- Test coverage: Needs robust unit tests with various URL formats.

**Hybrid Detection Logic:**
- Files: `src-tauri/src/lib.rs:94`
- Why fragile: Complex fallback logic that switches between static parsing and browser-based detection.
- Safe modification: Difficult to verify all paths (main URL vs iframes) without live site testing.

## Test Coverage Gaps

**Rust Backend:**
- What's not tested: Core logic for `VideoDownloader` and parsers.
- Files: `src-tauri/src/*.rs`
- Risk: Regressions in download logic or parsing failure go unnoticed until runtime.
- Priority: High

---

*Concerns audit: 2026-01-29*
