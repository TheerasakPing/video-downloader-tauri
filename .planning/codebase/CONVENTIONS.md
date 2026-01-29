# Coding Conventions

**Analysis Date:** 2026-01-29

## Naming Patterns

**Files:**
- **Frontend (TSX/TS):** PascalCase for components (`src/components/Button.tsx`), camelCase for hooks (`src/hooks/useI18n.tsx`) and general logic files.
- **Backend (Rust):** snake_case for modules and file names (`src-tauri/src/baanjeen_parser.rs`).

**Functions:**
- **Frontend:** camelCase for component functions and hooks (`useI18n`, `fetch_series`).
- **Backend:** snake_case for functions and methods (`get_series_info`, `is_baanjeen_url`).

**Variables:**
- **Frontend:** camelCase for variables and state (`selectedEpisodes`, `ffmpegAvailable`).
- **Backend:** snake_case for local variables and fields (`series_id`, `poster_url`).

**Types:**
- **Frontend:** PascalCase for Interfaces and Types (`SeriesInfo`, `DownloadProgress`).
- **Backend:** PascalCase for Structs and Enums (`UnifiedSeriesInfo`, `DownloadState`).

## Code Style

**Formatting:**
- **Frontend:** Standard Prettier-like formatting (though no config file found, the code follows consistent indentation).
- **Backend:** Standard Rust formatting (`rustfmt`).

**Linting:**
- **Frontend:** TypeScript is used for type safety. No ESLint/Biome config found in the root, but `tsconfig.json` enforces strictness.
- **Backend:** `clippy` and standard compiler warnings.

## Import Organization

**Order:**
1. React hooks and core libraries (`import { useState, ... } from "react"`)
2. External packages (`import { Loader2 } from "lucide-react"`)
3. Internal hooks/types (`import { useI18n } from "./hooks/useI18n"`)
4. Styles (`import "./index.css"`)

**Path Aliases:**
- Not explicitly detected in `tsconfig.json`. Relative paths are used (e.g., `import { LogEntry } from "../types"`).

## Error Handling

**Patterns:**
- **Frontend:** `try...catch` blocks for async operations and parsing (`src/hooks/useSettings.ts`). Uses a custom logger hook (`src/hooks/useLogger.ts`) to display errors to users.
- **Backend:** Uses `Result<T, String>` for tauri commands and `Result<T, E>` for internal logic. Extensively uses the `?` operator and `map_err` to convert errors to strings for frontend consumption (`src-tauri/src/lib.rs`). `anyhow` and `thiserror` are listed in `Cargo.toml`.

## Logging

**Framework:** Custom `useLogger` hook for UI logs; `eprintln!` for backend console logs.

**Patterns:**
- Frontend logs are categorized by levels: `info`, `success`, `warning`, `error`.
- Backend uses prefixed `eprintln!` for debugging (e.g., `eprintln!("[BaanJeen] Fetching page: {}")`).

## Comments

**When to Comment:**
- High-level logic explanation (e.g., explaining hybrid parsing in `src-tauri/src/lib.rs`).
- Regex pattern explanations.
- TODOs for future improvements.

**JSDoc/TSDoc:**
- Minimal usage in frontend.
- Rust uses triple-slash `///` for documentation comments on public methods (`src-tauri/src/baanjeen_parser.rs`).

## Function Design

**Size:** Components in `src/components/` are generally small and focused. `src/App.tsx` is large (~800 lines) and acts as the main orchestrator.

**Parameters:** Prefers destructured props in React components. Rust functions use explicit types and often take references (`&str`, `&AppHandle`).

**Return Values:** React hooks return objects/arrays of state and actions. Rust commands return `Result` types compatible with Tauri's IPC.

## Module Design

**Exports:** Named exports for components and hooks. `src-tauri/src/lib.rs` uses `mod` to organize backend logic.

**Barrel Files:** Not used; imports are direct from file paths.

---

*Convention analysis: 2026-01-29*
