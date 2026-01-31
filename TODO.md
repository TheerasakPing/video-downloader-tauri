# Project Tasks

## BaanJeen Module Implementation

- [x] Create `baanjeen_parser.rs` for static site parsing
- [x] Integrate parser into `lib.rs`
- [x] Implement `chrome_detector.rs` for dynamic video detection
- [x] Add "Hybrid Mode" to fallback to Chrome detection if static parsing fails
- [x] Fix compilation errors (RequestInterceptor, private types)
- [x] Add Progress Bar UI in `App.tsx` and event emission in Rust
- [x] Improve Chrome Detector:
    - [x] Add aggressive play button clicking
    - [x] Add polling loop
    - [x] Add iframe direct scanning
    - [x] Add `PerformanceObserver` for network capturing
    - [x] Add Source Code Inspection (asset/medias extraction)
    - [x] Refactor path expansion to `utils.rs`
- [x] Improve Downloader:
    - [x] Support `.m3u8` via FFmpeg
    - [x] Add User-Agent and Referer headers
    - [x] Add reconnection flags
- [x] Documentation:
    - [x] Create STATUS.md
    - [x] Update TODO.md

## Pending / Future Improvements
- [ ] Add support for `blob:` video downloads (requires complex interception)
- [ ] Add queue management for multiple concurrent Chrome detections
- [ ] Add settings UI to toggle Headless mode (useful for debugging)
