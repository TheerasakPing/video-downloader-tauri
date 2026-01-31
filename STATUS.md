# Project Status: Rongyok Video Downloader (BaanJeen Module)

**Date:** January 29, 2026
**Module:** `baanjeen_parser.rs` & `chrome_detector.rs`
**Target Site:** `https://xn--82c7abb4jua0l.com` (บ้านจีน.com)

## 🎯 Objective
Enable automatic video detection and downloading from บ้านจีน.com, which uses complex dynamic loading, iframes, and HLS streaming (.m3u8).

## 🚧 Challenges
1.  **Dynamic Loading:** Video player and URLs are not in the initial HTML source. They load via JavaScript after user interaction.
2.  **Iframe Encapsulation:** The video player is often nested inside an iframe (e.g., `play.baiwarp.com`), hiding network requests from the main page context.
3.  **Click-to-Play:** Video URLs (`.m3u8`) often generate only *after* the user clicks the "Play" button or Poster image.
4.  **Anti-Bot / Headers:** The server checks for specific `User-Agent` and `Referer` headers.

## 🛠️ Implemented Solution: "Hybrid God Mode"

We have implemented a multi-layered detection system that mimics the logic of a successful Playwright/Python script.

### Layer 1: Static Parsing (`baanjeen_parser.rs`)
-   **URL Detection:** Identifies `xn--82c7abb4jua0l.com` URLs.
-   **Metadata Extraction:** Extracts Title, Poster, and Episode list from the static HTML.
-   **Iframe Discovery:** Finds all `iframe` sources on the page to be used as fallback targets.

### Layer 2: Chrome Detector (`chrome_detector.rs`)
This is the core "God Mode" engine using `headless_chrome`.

1.  **Iframe Direct Scanning (New!):**
    -   If scanning the main page fails, the system automatically iterates through all discovered `iframe` URLs.
    -   It navigates *directly* to the iframe player page, bypassing the main site's clutter and nesting.

2.  **Performance Observer Injection (New!):**
    -   Instead of relying on external network event listeners (which can be flaky with cross-origin iframes), we **inject JavaScript** into the page.
    -   This script uses the `PerformanceObserver` API to monitor the browser's own resource timing buffer in real-time.
    -   It captures any resource containing `.m3u8`, `.mp4`, or `video` and reports it back to Rust.

3.  **Source Code Inspection (Python Script Parity):**
    -   **The Secret Weapon:** We implemented the exact logic from the provided Python script.
    -   The detector scans the page's rendered HTML for specific JavaScript configuration variables: `"asset": "..."` and `"medias": ...`.
    -   If found, it **reconstructs the master playlist URL** directly (`https://{asset}/hls/{mediaId}/master.m3u8`), bypassing the need for network interception entirely.

4.  **Aggressive Interaction:**
    -   Automatically clicks specific elements: `.jw-poster`, `button[aria-label='Play']`, `iframe`, and the center of the screen.
    -   Hides `navigator.webdriver` to avoid basic bot detection.

### Layer 3: Robust Downloader (`downloader.rs`)
-   **FFmpeg HLS Support:** Automatically switches to `ffmpeg` for `.m3u8` URLs.
-   **Browser Mimicry:** Sends correct `User-Agent` and `Referer` (https://google.com) headers to satisfy server checks.
-   **Network Resilience:** Added `-reconnect 1 -reconnect_at_eof 1 -reconnect_streamed 1 -reconnect_delay_max 2` flags to handle connection drops during download.

## ✅ Current Status
-   **Logic:** 100% Match with working Python script.
-   **Flow:** Main Page -> (Fail) -> Iframe Scan -> (Success via Config/Network) -> Download.
-   **UI:** Shows "Detection Progress" bar with detailed status steps.

## 📋 Next Steps
-   Monitor reliability of the `PerformanceObserver` injection.
-   If specific videos still fail, verify if they use a new player type not covered by the "asset/medias" regex.
