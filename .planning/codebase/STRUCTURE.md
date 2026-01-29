# Codebase Structure

**Analysis Date:** 2026-01-29

## Directory Layout

```
_rongyok_video_downloader_rust/
├── src/                # Frontend (React + TypeScript)
│   ├── assets/         # Images, fonts, etc.
│   ├── components/     # UI Components (Button, ProgressBar, etc.)
│   ├── hooks/          # Custom React Hooks (useLogger, useSettings, etc.)
│   ├── types/          # TypeScript interface definitions
│   ├── App.tsx         # Main application container
│   ├── main.tsx        # React entry point
│   └── index.css       # Global styles (Tailwind)
├── src-tauri/          # Backend (Rust + Tauri config)
│   ├── icons/          # App icons for different platforms
│   ├── src/            # Rust source code
│   │   ├── baanjeen_parser.rs   # Scraper for BaanJeen site
│   │   ├── chrome_detector.rs   # Headless browser detection logic
│   │   ├── downloader.rs        # Core download/merge logic
│   │   ├── lib.rs               # Tauri command definitions & app setup
│   │   ├── main.rs              # App entry point
│   │   └── parser.rs            # Scraper for Rongyok site
│   ├── Cargo.toml      # Rust dependencies
│   └── tauri.conf.json # Tauri configuration
├── tests/              # E2E tests (Playwright)
├── public/             # Static assets for the frontend
├── dist/               # Build output (frontend)
└── package.json        # Frontend dependencies & scripts
```

## Directory Purposes

**src/components/:**
- Purpose: Atomic and composite UI components.
- Contains: React components used across the app.
- Key files: `SeriesCard.tsx`, `EpisodeSelector.tsx`, `LogPanel.tsx`.

**src/hooks/:**
- Purpose: Logic encapsulation for React.
- Contains: Hooks for settings, history, logging, and interaction with Tauri events.
- Key files: `useLogger.ts`, `useSettings.ts`, `useHistory.ts`.

**src-tauri/src/:**
- Purpose: Backend logic and system integration.
- Contains: Rust modules for networking, parsing, and file I/O.
- Key files: `lib.rs` (the hub), `downloader.rs` (the engine).

## Key File Locations

**Entry Points:**
- `src/main.tsx`: Frontend React root.
- `src-tauri/src/main.rs`: Backend process root.

**Configuration:**
- `src-tauri/tauri.conf.json`: Tauri application settings (permissions, window, bundle).
- `package.json`: Node.js dependencies and build scripts.
- `Cargo.toml`: Rust dependencies and metadata.

**Core Logic:**
- `src-tauri/src/downloader.rs`: Manages `reqwest` and `ffmpeg` processes.
- `src-tauri/src/chrome_detector.rs`: Manages `headless_chrome` for URL extraction.

**Testing:**
- `tests/`: Contains Playwright test suites for end-to-end verification.

## Naming Conventions

**Files:**
- Frontend: PascalCase for components (`SeriesCard.tsx`), camelCase for hooks (`useLogger.ts`).
- Backend: snake_case for Rust modules (`chrome_detector.rs`).

**Directories:**
- Mostly kebab-case or snake_case for system directories, but frontend follows standard React folder patterns.

## Where to Add New Code

**New Feature:**
- Frontend Logic: `src/hooks/`
- UI: `src/components/`
- Backend Command: `src-tauri/src/lib.rs`
- Core Engine: New module in `src-tauri/src/`

**New Component/Module:**
- Component: `src/components/` (remember to export from `index.ts` if applicable).
- Rust Utility: `src-tauri/src/` as a new module.

**Utilities:**
- Shared helpers (Frontend): `src/utils/` (if created) or inside specific hooks.
- Shared helpers (Backend): New module or inside `lib.rs`.

## Special Directories

**src-tauri/binaries/:**
- Purpose: Holds external binaries like `ffmpeg` (if bundled as sidecars).
- Generated: No
- Committed: Yes

**dist/:**
- Purpose: Compiled frontend assets.
- Generated: Yes (by `npm run build`)
- Committed: No

---

*Structure analysis: 2026-01-29*
