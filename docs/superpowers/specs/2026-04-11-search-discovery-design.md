# Phase 1: Search & Discovery — Design Specification

**Goal:** Add cross-site search and category browsing to discover content across all supported sites from within the app.

**Architecture:** Per-parser search/browse methods, parallel aggregation in the Rust backend, new "Browse" tab in the frontend with search + category browsing + detail-to-download flow.

**Sites covered:** rongyok, baanjeen, titan, hsck (skipping Cloudflare-protected njavtv/njav — too slow for search).

---

## 1. Backend Data Types

### 1.1 SearchResult

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub title: String,
    pub poster_url: Option<String>,
    pub url: String,
    pub source: String,
    pub episode_count: Option<i32>,
}
```

A single search/browse result from any site. The `url` field is the full link to the series/video page on the source site, ready to pass to `fetch_series`.

### 1.2 SiteCategory

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteCategory {
    pub id: String,
    pub label: String,
    pub source: String,
}
```

A browseable category from a site (e.g., "latest", "trending", "chinese"). The `id` is used in browse requests; `label` is displayed in the UI.

### 1.3 SearchResponse

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub source: String,
    pub page: i32,
    pub has_more: bool,
}
```

Results from one site. `search_sites` returns a `Vec<SearchResponse>` so the frontend can group results by source.

---

## 2. Parser Extensions

Each parser that supports search adds three methods. Parsers that don't support search (njav, njavtv) are simply skipped.

### 2.1 Method Signatures

```rust
async fn search(&self, query: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String>;
fn list_categories(&self, domain: &str) -> Vec<SiteCategory>;
async fn browse(&self, category: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String>;
```

### 2.2 Per-Site Implementation Notes

#### RongyokParser (rongyok.com, thongyok.com)
- **Search URL:** `https://{domain}/?s={query}` — standard WordPress search
- **Categories:** latest, popular (via homepage pagination `?paged={n}`)
- **HTML parsing:** scraper for `.post-item` or similar WordPress article containers, extract title, poster from `<img>`, link from `<a>`
- **episode_count:** Parse from series page or skip

#### BaanJeenParser (บ้านจีน.com)
- **Search URL:** `https://{domain}/?s={query}` — WordPress search
- **Categories:** Extract category links from homepage nav (Chinese drama categories)
- **HTML parsing:** scraper for article/post containers, similar to rongyok

#### TitanParser (357ms.com, 51cg1.com)
- **Search URL:** `https://{domain}/?s={query}` — WordPress search
- **Categories:** Archive tags, latest
- **HTML parsing:** scraper for post containers, extract title/poster/link. May need to handle the site's redirect patterns

#### HsckParser (hsck123.com)
- **Search URL:** `https://{domain}/?s={query}` — search parameter
- **Categories:** Already has `list_videos` with category support — extend to return `SiteCategory` list
- **HTML parsing:** Already implemented in `list_videos` — reuse selectors for search results

---

## 3. Tauri Commands

### 3.1 search_sites

```rust
#[tauri::command]
async fn search_sites(
    query: String,
    page: i32,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResponse>, String>
```

Dispatches search to all 4 parsers in parallel using `futures::join_all`. Returns successful results only — if a parser fails, its results are omitted (never causes total failure).

### 3.2 get_browse_categories

```rust
#[tauri::command]
fn get_browse_categories(
    state: State<'_, AppState>,
) -> Result<Vec<SiteCategory>, String>
```

Returns static category lists from each parser. No network request needed — categories are hardcoded per-site based on known site structure.

### 3.3 browse_category

```rust
#[tauri::command]
async fn browse_category(
    source: String,
    category: String,
    page: i32,
    state: State<'_, AppState>,
) -> Result<SearchResponse, String>
```

Browse a single site's category with pagination. Routes to the correct parser based on `source` string.

---

## 4. Frontend Design

### 4.1 New Tab

Add `"browse"` to `TabType` union and `tabs` array, positioned between "library" and "files". Uses a `Compass` or `Globe` icon from lucide-react with emerald/teal glow color.

### 4.2 BrowsePanel Component

Main component for the Browse tab:

- **Search bar** at top with `Search` icon, debounced input (300ms), searches on Enter or button click
- **Source filter chips** below search: "All" / "Rongyok" / "BaanJeen" / "Titan" / "Hsck" — filters displayed results by source
- **Category pills row** below filters: horizontal scrollable row of category buttons, fetched from `get_browse_categories`, grouped by source with source prefix labels
- **Results grid**: `grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5`, shows BrowseCard components
- **Pagination**: "Load more" button at bottom when `has_more` is true for any source
- **Loading state**: Skeleton cards or spinner while searching
- **Empty state**: "Search or browse categories to discover content" with search icon

### 4.3 BrowseCard Component

Card for each result in the grid:

- Poster image (or placeholder `ImageIcon` if none)
- Title (truncated to 2 lines)
- Source badge (colored dot + site name, matching LibraryCard pattern)
- Episode count badge if available
- Click handler → navigates to detail view

### 4.4 BrowseDetail Component

Detail view when a result is clicked:

- Back button → returns to browse grid
- Poster + title + source info + episode count
- "Load Series" button → calls existing `fetch_series` command with the result URL
- Loading spinner while fetching
- Once loaded: shows episode selector + download button (reuses EpisodeSelector and quality/download controls from Download tab)
- Error state: "Could not load series details"

### 4.5 State Management

No new hook needed. BrowsePanel uses local `useState` for:
- `query: string` — search input
- `results: SearchResponse[]` — search results grouped by source
- `selectedSource: string | null` — active source filter
- `detail: SearchResult | null` — selected result for detail view
- `isLoading: boolean` — loading state
- `categories: SiteCategory[]` — cached category list

### 4.6 Data Flow

```
User types query → invoke("search_sites", { query, page: 1 })
  → Backend dispatches to 4 parsers in parallel
  → Returns Vec<SearchResponse>
  → Frontend displays results grouped by source

User clicks category → invoke("browse_category", { source, category, page: 1 })
  → Backend routes to correct parser
  → Returns SearchResponse
  → Frontend replaces grid with category results

User clicks result card → setDetail(result)
  → Shows BrowseDetail
  → User clicks "Load Series" → invoke("fetch_series", { url })
  → Shows episodes → User selects + downloads (existing flow)
```

---

## 5. File Changes Summary

### Backend (Rust)
| File | Change |
|------|--------|
| `src-tauri/src/parser.rs` | Add `search`, `list_categories`, `browse` methods to RongyokParser |
| `src-tauri/src/baanjeen_parser.rs` | Add `search`, `list_categories`, `browse` methods |
| `src-tauri/src/titan_parser.rs` | Add `search`, `list_categories`, `browse` methods |
| `src-tauri/src/hsck_parser.rs` | Add `search`, `list_categories` methods; refactor `list_videos` into `browse` |
| `src-tauri/src/lib.rs` | Add `SearchResult`, `SiteCategory`, `SearchResponse` types; add 3 Tauri commands; register commands |

### Frontend (React/TypeScript)
| File | Change |
|------|--------|
| `src/types/index.ts` | Add `SearchResult`, `SiteCategory`, `SearchResponse` types |
| `src/components/BrowsePanel.tsx` | New — main browse tab with search + categories + grid |
| `src/components/BrowseCard.tsx` | New — result card component |
| `src/components/BrowseDetail.tsx` | New — detail view with load + download |
| `src/components/index.ts` | Export new components |
| `src/App.tsx` | Add "browse" tab to TabType and tabsConfig |

---

## 6. Error Handling

- **Parser search failure:** Silently omit failed parser results. Frontend shows whatever succeeded.
- **No results:** Show "No results found" empty state with suggestion to try different keywords.
- **Network error:** Show warning toast, allow retry.
- **fetch_series failure from detail:** Show error in BrowseDetail with retry button.

---

## 7. Out of Scope

- Search on njavtv/njav (Cloudflare-protected — too slow)
- Caching search results (can add later if needed)
- Search suggestions/autocomplete
- Advanced filters (date, quality, etc.)
- Sorting results
