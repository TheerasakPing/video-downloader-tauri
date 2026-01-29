# Testing Patterns

**Analysis Date:** 2026-01-29

## Test Framework

**Runner:**
- **Frontend:** Playwright (v1.57.0)
- **Backend:** Native Rust test runner (`cargo test`)

**Assertion Library:**
- **Frontend:** Playwright's built-in assertions.
- **Backend:** Standard Rust `assert_eq!`, `assert!`.

**Run Commands:**
```bash
npm run test           # Run Playwright tests
npm run test:ui        # Playwright UI mode
cargo test             # Run Rust unit tests
```

## Test File Organization

**Location:**
- **Frontend:** `./tests` directory for Playwright E2E tests.
- **Backend:** In-file test modules using `#[cfg(test)]`.

**Naming:**
- **Frontend:** `*.spec.ts` (Configured in `playwright.config.ts`).
- **Backend:** Tests are contained within the source files they test.

**Structure:**
```
[root]/
├── tests/              # Playwright E2E tests
src-tauri/src/
├── parser.rs           # Contains mod tests { ... }
└── baanjeen_parser.rs  # Contains mod tests { ... }
```

## Test Structure

**Suite Organization:**
```typescript
// Playwright (Frontend)
import { test, expect } from '@playwright/test';
test('test name', async ({ page }) => {
  await page.goto('/');
  // ...
});

// Rust (Backend)
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_function_name() {
        assert_eq!(...);
    }
}
```

**Patterns:**
- **Unit Testing:** Focused on URL parsing and logic in Rust (`src-tauri/src/parser.rs`).
- **E2E Testing:** Playwright is configured to test the full application flow at `http://localhost:1420`.

## Mocking

**Framework:**
- **Frontend:** Playwright's network intercepting/mocking capabilities.
- **Backend:** Not explicitly detected; tests use static inputs.

**Patterns:**
- Rust tests use hardcoded URL strings to verify parser logic without making network requests in unit tests.

**What to Mock:**
- Network responses for series metadata and video URLs.
- Tauri IPC calls (if testing frontend in isolation).

**What NOT to Mock:**
- Utility functions (e.g., `sanitize_filename`).
- URL parsing regex logic.

## Fixtures and Factories

**Test Data:**
- Simple string literals for URLs and expected IDs.

**Location:**
- Embedded directly in test functions.

## Coverage

**Requirements:** None enforced.

**View Coverage:**
```bash
# Rust coverage (requires cargo-tarpaulin or similar)
cargo tarpaulin
```

## Test Types

**Unit Tests:**
- URL parsing logic in `src-tauri/src/parser.rs` and `src-tauri/src/baanjeen_parser.rs`.

**Integration Tests:**
- Not explicitly separated in the codebase.

**E2E Tests:**
- Playwright tests in the `tests/` directory (config present, though directory was empty in initial scan, likely used for manual E2E).

## Common Patterns

**Async Testing:**
- Playwright tests use `async/await`.
- Rust tests observed so far are synchronous but `tokio::test` would be used for async Rust.

**Error Testing:**
- Verifying that invalid URLs return `None` or expected error strings.

---

*Testing analysis: 2026-01-29*
