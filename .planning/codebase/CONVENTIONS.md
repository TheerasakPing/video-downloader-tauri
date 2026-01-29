# Coding Conventions

**Analysis Date:** 2026-01-29

## Naming Patterns

**Files:**
- React Components: `PascalCase.tsx` (e.g., `src/components/Button.tsx`)
- Hooks: `camelCase.ts` with `use` prefix (e.g., `src/hooks/useLogger.ts`)
- Rust Modules: `snake_case.rs` (e.g., `src-tauri/src/downloader.rs`)
- TypeScript Types: `index.ts` (e.g., `src/types/index.ts`)

**Functions:**
- TypeScript: `camelCase` (e.g., `addLog`, `fetchSeries`)
- Rust: `snake_case` (e.g., `fetch_series`, `detect_video_url`)

**Variables:**
- TypeScript: `camelCase` (e.g., `baseStyles`, `isLoading`)
- Rust: `snake_case` (e.g., `series_info`, `video_url`)

**Types:**
- TypeScript Interfaces/Types: `PascalCase` (e.g., `ButtonProps`, `SeriesInfo`)
- Rust Structs/Enums: `PascalCase` (e.g., `UnifiedSeriesInfo`, `DownloadRequest`)

## Code Style

**Formatting:**
- Frontend: Prettier/ESLint (standard TypeScript formatting observed in `src/App.tsx`).
- Backend: `rustfmt` standard style.

**Linting:**
- Frontend: TypeScript compiler (`tsc`) used in build script. `tsconfig.json` defines strictness.
- Backend: Standard Rust compiler lints and `clippy`.

## Import Organization

**Order:**
1. React/Framework imports (`import { useState } from "react"`)
2. External library imports (`import { Loader2 } from "lucide-react"`)
3. Local component/hook imports (`import { useLogger } from "../hooks/useLogger"`)
4. Type/Interface imports (`import { LogEntry } from "../types"`)

**Path Aliases:**
- Not explicitly configured in `tsconfig.json`, uses relative paths (e.g., `../hooks/`).

## Error Handling

**Patterns:**
- TypeScript: Uses `try...catch` for async operations. Errors are logged via `useLogger` hook for UI display.
- Rust:
    - Commands return `Result<T, String>` to bridge with JavaScript.
    - Internal logic uses `thiserror` for custom error types and `anyhow` for flexible error handling.
    - Extensive use of the `?` operator and `map_err(|e| e.to_string())` for IPC compatibility.

## Logging

**Framework:**
- Frontend: Custom hook `src/hooks/useLogger.ts` which manages a local log state displayed in `LogPanel.tsx`.
- Backend: Emits events back to frontend using `app_handle.emit("log-info", ...)` to show logs in the UI.

**Patterns:**
- Logs have levels: `info`, `success`, `warning`, `error`.
- Backend logs critical steps (parsing start, download progress, merge status).

## Comments

**When to Comment:**
- Complex logic (e.g., Chrome detection hybrid mode in `src-tauri/src/lib.rs`).
- Module declarations and helper function purposes.

**JSDoc/TSDoc:**
- Minimal usage; types are primarily relied upon for documentation. Rust uses `///` for documentation comments.

## Function Design

**Size:**
- React components are generally modular. `src/App.tsx` acts as the primary container.
- Rust functions in `src-tauri/src/lib.rs` coordinate multiple steps and can be longer.

**Parameters:**
- Frontend: Uses object destructuring for props.
- Rust: Uses `State` and `AppHandle` for Tauri context, plus request structs for command arguments.

## Module Design

**Exports:**
- TypeScript: Named exports for components and hooks. Barrel files (e.g., `src/components/index.ts`) used for cleaner imports.
- Rust: `pub` visibility for shared structs and modules.

---

*Convention analysis: 2026-01-29*
