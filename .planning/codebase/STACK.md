# Technology Stack

**Analysis Date:** 2026-01-29

## Languages

**Primary:**
- Rust 2021 - Core application logic, downloader, and Tauri backend (`src-tauri/src/`)
- TypeScript 5.8 - Frontend logic and UI components (`src/`)

**Secondary:**
- Python - Auxiliary scripts for video extraction (`src-tauri/download_video.py`, `src-tauri/test_detection.py`)
- CSS (Tailwind 4.1) - UI Styling

## Runtime

**Environment:**
- Tauri 2.0 - Desktop application framework
- Node.js - Frontend build environment

**Package Manager:**
- npm - Managed via `package.json`
- Cargo - Managed via `src-tauri/Cargo.toml`
- Lockfiles: `package-lock.json` and `src-tauri/Cargo.lock` present

## Frameworks

**Core:**
- React 19.1 - Frontend UI library
- Vite 7.0 - Frontend build tool and dev server
- Tailwind CSS 4.1 - Styling framework

**Testing:**
- Playwright 1.57 - End-to-end and UI testing (`tests/`, `playwright.config.ts`)

**Build/Dev:**
- Tauri CLI 2.0 - Application packaging and development

## Key Dependencies

**Critical:**
- `tauri` (v2) - Main framework for cross-platform desktop apps
- `reqwest` (v0.12) - Async HTTP client for Rust used in direct downloads (`src-tauri/src/downloader.rs`)
- `tokio` (v1) - Async runtime for Rust
- `headless_chrome` (v1.0) - Automated video URL detection (`src-tauri/src/chrome_detector.rs`)
- `scraper` (v0.22) - HTML parsing for extracting video links (`src-tauri/src/parser.rs`, `src-tauri/src/baanjeen_parser.rs`)

**Infrastructure:**
- `ffmpeg` / `ffprobe` - External binaries used for merging video chunks and downloading HLS streams (`src-tauri/resources/bin/`)
- `tauri-plugin-updater` - Application update mechanism
- `tauri-plugin-dialog` - Native system dialogs
- `tauri-plugin-opener` - Open URLs/folders in default applications

## Configuration

**Environment:**
- `src-tauri/tauri.conf.json` - Main Tauri application configuration
- `package.json` - Frontend dependencies and scripts
- `src-tauri/Cargo.toml` - Rust dependencies and metadata

**Build:**
- `vite.config.ts` - Vite configuration
- `tsconfig.json` - TypeScript configuration
- `src-tauri/build.rs` - Rust build script for Tauri

## Platform Requirements

**Development:**
- Rust toolchain
- Node.js & npm
- Platform-specific Tauri dependencies (WebView2 on Windows, WebKitGTK on Linux)

**Production:**
- Windows, macOS, Linux (Tauri supported platforms)
- Bundled `ffmpeg` and `ffprobe` binaries in `src-tauri/resources/bin/`

---

*Stack analysis: 2026-01-29*
