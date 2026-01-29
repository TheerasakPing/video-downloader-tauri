# Technology Stack

**Analysis Date:** 2026-01-29

## Languages

**Primary:**
- Rust 2021 edition - Core application logic, downloader, and Tauri backend.
- TypeScript - Frontend logic and UI components.

**Secondary:**
- Python 3 - Auxiliary scripts for video extraction (e.g., `src-tauri/download_video.py`).

## Runtime

**Environment:**
- Tauri v2 - Desktop application framework.
- Node.js - Frontend build environment.

**Package Manager:**
- npm - Managed via `package.json`.
- Cargo - Managed via `src-tauri/Cargo.toml`.
- Lockfile: `package-lock.json` (implied), `src-tauri/Cargo.lock` (present).

## Frameworks

**Core:**
- React 19 - Frontend UI library.
- Vite 7 - Frontend build tool and dev server.
- Tailwind CSS 4 - Styling framework.

**Testing:**
- Playwright - End-to-end and UI testing (`package.json`).

**Build/Dev:**
- Tauri CLI v2 - Application packaging and development.

## Key Dependencies

**Critical:**
- `tauri` (v2) - Main framework for cross-platform desktop apps.
- `reqwest` (v0.12) - Async HTTP client for Rust used in direct downloads.
- `tokio` (v1) - Async runtime for Rust.
- `headless_chrome` (v1.0) - Used for automated video URL detection in Rust.
- `scraper` (v0.22) - HTML parsing for extracting video links.

**Infrastructure:**
- `ffmpeg` / `ffprobe` - External binaries used for merging video chunks and downloading HLS streams.
- `tauri-plugin-updater` - Handles application updates.
- `tauri-plugin-dialog` - Native system dialogs.

## Configuration

**Environment:**
- Configured via `src-tauri/tauri.conf.json`.
- Versioning and metadata in `package.json` and `Cargo.toml`.

**Build:**
- `vite.config.ts` - Vite configuration.
- `tsconfig.json` - TypeScript configuration.
- `src-tauri/tauri.conf.json` - Tauri specific build and bundle settings.

## Platform Requirements

**Development:**
- Rust toolchain.
- Node.js & npm.
- System dependencies for Tauri (WebView2 on Windows, WebKitGTK on Linux).

**Production:**
- Windows, macOS, Linux (Tauri targets).
- Requires `ffmpeg` available in path or bundled in resources.

---

*Stack analysis: 2026-01-29*
