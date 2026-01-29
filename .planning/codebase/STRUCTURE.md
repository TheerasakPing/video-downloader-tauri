# Codebase Structure

**Analysis Date:** 2026-01-29

## Directory Layout

```
_rongyok_video_downloader_rust/
├── src/                # Frontend (React + TypeScript)
│   ├── assets/         # UI assets (icons, styles)
│   ├── components/     # React UI components
│   ├── hooks/          # Custom React hooks (logic & state)
│   ├── types/          # TypeScript interface definitions
│   ├── App.tsx         # Main application container
│   └── main.tsx        # React entry point
├── src-tauri/          # Backend (Rust + Tauri)
│   ├── src/            # Rust source files
│   │   ├── baanjeen_parser.rs # Website-specific scraper
│   │   ├── chrome_detector.rs # Headless Chrome automation
│   │   ├── downloader.rs      # Download and FFmpeg engine
│   │   ├── lib.rs             # Tauri command handlers & app setup
│   │   ├── main.rs            # Application entry point
│   │   └── parser.rs          # Website-specific scraper
│   ├── tauri.conf.json # Tauri configuration
│   └── Cargo.toml      # Rust package manifest
├── package.json        # Frontend dependencies & build scripts
└── tsconfig.json       # TypeScript configuration
```

## Directory Purposes

**src/components/:**
- Purpose: Atomic and composite UI components.
- Contains: Reusable React components like `Button`, `ProgressBar`, `SeriesCard`.
- Key files: `src/components/index.ts` (barrel file).

**src/hooks/:**
- Purpose: Logic encapsulation and state management.
- Contains: Hooks for logger, settings, history, and system integration.
- Key files: `src/hooks/useLogger.ts`, `src/hooks/useSettings.ts`.

**src-tauri/src/:**
- Purpose: Core application backend.
- Contains: Rust modules for networking, scraping, and process management.
- Key files: `src-tauri/src/lib.rs` (bridge), `src-tauri/src/downloader.rs` (engine).

## Key File Locations

**Entry Points:**
- `src/main.tsx`: React application entry.
- `src-tauri/src/main.rs`: Rust process entry.

**Configuration:**
- `src-tauri/tauri.conf.json`: Main Tauri configuration.
- `package.json`: Node.js/Vite configuration.

**Core Logic:**
- `src-tauri/src/downloader.rs`: Download management.
- `src-tauri/src/chrome_detector.rs`: Headless browser control.

**Testing:**
- `src-tauri/src/baanjeen_parser.rs`: Contains unit tests for parsing.

## Naming Conventions

**Files:**
- Frontend: PascalCase for components (`SeriesCard.tsx`), camelCase for hooks (`useSettings.ts`).
- Backend: snake_case for Rust modules (`chrome_detector.rs`).

**Directories:**
- All lowercase, usually kebab-case or simple names (`components`, `src-tauri`).

## Where to Add New Code

**New Feature:**
- Add command to `src-tauri/src/lib.rs`.
- Implement core logic in new or existing module in `src-tauri/src/`.
- Add UI state/interaction in `src/App.tsx` or new hook in `src/hooks/`.

**New Component/Module:**
- Create in `src/components/`.
- Export via `src/components/index.ts`.

**Utilities:**
- Rust helpers in `src-tauri/src/lib.rs` or dedicated module.
- Frontend helpers in a new `src/utils/` (if needed) or relevant hooks.

## Special Directories

**src-tauri/binaries/:**
- Purpose: Location for external sidecar binaries (e.g., ffmpeg).
- Generated: No.
- Committed: Yes.

---

*Structure analysis: 2026-01-29*
