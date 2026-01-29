# Testing Patterns

**Analysis Date:** 2026-01-29

## Test Framework

**Runner:**
- Frontend: Playwright `^1.57.0` (configured in `playwright.config.ts`)
- Backend: Cargo (Built-in Rust test runner)

**Assertion Library:**
- Frontend: Playwright built-in assertions (`expect(page).to...`).
- Backend: Standard Rust `assert!`, `assert_eq!`.

**Run Commands:**
```bash
npm run test           # Run Playwright tests
npm run test:ui        # Playwright UI mode
cargo test             # Run Rust unit tests
```

## Test File Organization

**Location:**
- Frontend: `tests/` directory for E2E and integration tests.
- Backend: Co-located in source files using `#[cfg(test)]` modules (e.g., `src-tauri/src/parser.rs`).

**Naming:**
- Frontend: `*.spec.ts` or `*.test.ts` (configured in `playwright.config.ts`).
- Backend: `test_` prefix for test functions within source files.

## Test Structure

**Suite Organization:**
```rust
// Rust example from src-tauri/src/parser.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_series_url() {
        assert_eq!(RongyokParser::parse_series_url("..."), Some(1004));
    }
}
```

**Patterns:**
- Unit tests for pure logic like URL parsing and regex matching.
- E2E tests via Playwright (targeting `http://localhost:1420`).

## Mocking

**Framework:**
- Frontend: Playwright network interception/mocking.
- Backend: No explicit mocking framework; tests use static inputs/expected values.

**Patterns:**
- Rust tests use hardcoded URL strings and expected IDs to verify parser logic without network requests.

## Fixtures and Factories

**Test Data:**
- Hardcoded URLs and expected metadata within test functions.

**Location:**
- Defined inline within `#[test]` functions or as local variables.

## Coverage

**Requirements:**
- None enforced.

**View Coverage:**
- Not currently configured in `package.json` or `Cargo.toml`.

## Test Types

**Unit Tests:**
- Focus on `RongyokParser` and `BaanJeenParser` logic in `src-tauri/src/parser.rs` and `src-tauri/src/baanjeen_parser.rs`.

**Integration Tests:**
- Playwright is configured for full-app integration/E2E testing.

**E2E Tests:**
- Playwright tests targeting the Tauri development server.

## Common Patterns

**Async Testing:**
- Playwright tests are async (`test('...', async ({ page }) => { ... })`).
- Rust tests found are synchronous (parsing logic).

**Error Testing:**
- Verifying that invalid URL formats return `None` or expected errors.

---

*Testing analysis: 2026-01-29*
