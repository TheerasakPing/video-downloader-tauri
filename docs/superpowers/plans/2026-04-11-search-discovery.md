# Phase 1: Search & Discovery Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add cross-site search and category browsing so users can discover and download content from within the app.

**Architecture:** Per-parser `search`/`browse`/`list_categories` methods in Rust, 3 new Tauri commands aggregating results in parallel, new "Browse" tab in React frontend with search + categories + detail-to-download flow.

**Tech Stack:** Rust (scraper, reqwest, futures), TypeScript/React (Tailwind CSS, lucide-react), Tauri v2

**Spec:** `docs/superpowers/specs/2026-04-11-search-discovery-design.md`

---

## File Structure

| File | Responsibility | Status |
|------|---------------|--------|
| `src-tauri/src/lib.rs` | Shared types (SearchResult, SiteCategory, SearchResponse) + 3 Tauri commands | Modify |
| `src-tauri/src/hsck_parser.rs` | search + list_categories (already has browse via list_videos) | Modify |
| `src-tauri/src/parser.rs` | search + list_categories + browse for RongyokParser | Modify |
| `src-tauri/src/baanjeen_parser.rs` | search + list_categories + browse for BaanJeenParser | Modify |
| `src-tauri/src/titan_parser.rs` | search + list_categories + browse for TitanParser | Modify |
| `src/types/index.ts` | TypeScript type definitions | Modify |
| `src/components/BrowseCard.tsx` | Search result card | Create |
| `src/components/BrowseDetail.tsx` | Detail view with load+download | Create |
| `src/components/BrowsePanel.tsx` | Main browse tab with search+grid | Create |
| `src/components/index.ts` | Export new components | Modify |
| `src/App.tsx` | Add browse tab | Modify |

---

### Task 1: Backend Types in lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs:103-121` (after UnifiedSeriesInfo)

- [ ] **Step 1: Add SearchResult, SiteCategory, SearchResponse types**

Add after the `UnifiedSeriesInfo` struct (after line 121), before `struct AppState`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub title: String,
    pub poster_url: Option<String>,
    pub url: String,
    pub source: String,
    pub total_episodes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteCategory {
    pub id: String,
    pub label: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub source: String,
    pub page: i32,
    pub has_more: bool,
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors (types are defined but not yet used — may get unused warnings, that's fine)

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add SearchResult, SiteCategory, SearchResponse types"
```

---

### Task 2: HSCK Parser — search + list_categories

**Files:**
- Modify: `src-tauri/src/hsck_parser.rs` (add after line 222, after `list_videos`)

**Context:** HSCK already has `list_videos(domain, page, category)` at lines 151-222 that returns `Vec<(String, String, Option<String>)>`. It uses scraper selectors `a[href*="/view/?id="]` for video links, `img[data-original]` or `img[src]` for posters. We need to add `search` (using `?s=query`) and `list_categories` (returning hardcoded category list).

- [ ] **Step 1: Add imports for SearchResult and SiteCategory**

At top of `hsck_parser.rs`, add after existing imports:
```rust
use crate::{SearchResult, SiteCategory};
```

- [ ] **Step 2: Add `search` method**

Add inside `impl HsckParser` block (after `list_videos`, around line 222):

```rust
pub async fn search(&self, query: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
    let url = format!("https://{}?s={}&p={}", domain, query, page);
    let resp = self.client().get(&url).send().await.map_err(|e| e.to_string())?;
    let html_text = resp.text().await.map_err(|e| e.to_string())?;
    let document = Html::parse_document(&html_text);

    let mut results = Vec::new();
    let link_selector = Selector::parse("a[href*=\"/view/?id=\"]").unwrap();

    for el in document.select(&link_selector) {
        let href = el.value().attr("href").unwrap_or("").to_string();
        let title = el.value().attr("title").unwrap_or("").to_string();
        if title.is_empty() || href.is_empty() { continue; }

        let poster = el.select(&Selector::parse("img").unwrap())
            .next()
            .and_then(|img| img.value().attr("data-original").or_else(|| img.value().attr("src")).map(|s| s.to_string()));

        let full_url = if href.starts_with("http") { href } else { format!("https://{}{}", domain, if href.starts_with('/') { "" } else { "/" }) + &href };

        results.push(SearchResult {
            title,
            poster_url: poster,
            url: full_url,
            source: "hsck".to_string(),
            total_episodes: None,
        });
    }

    // Deduplicate by URL
    let mut seen = std::collections::HashSet::new();
    results.retain(|r| seen.insert(r.url.clone()));
    Ok(results)
}

pub fn list_categories(&self, _domain: &str) -> Vec<SiteCategory> {
    vec![
        SiteCategory { id: "latest".into(), label: "Latest".into(), source: "hsck".into() },
        SiteCategory { id: "hot".into(), label: "Hot".into(), source: "hsck".into() },
        SiteCategory { id: "chinese".into(), label: "Chinese".into(), source: "hsck".into() },
        SiteCategory { id: "japanese".into(), label: "Japanese".into(), source: "hsck".into() },
        SiteCategory { id: "korean".into(), label: "Korean".into(), source: "hsck".into() },
    ]
}

pub async fn browse(&self, category: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
    let cat = if category == "latest" { None } else { Some(category) };
    let items = self.list_videos(domain, page as u32, cat).await?;
    Ok(items.into_iter().map(|(id, title, poster)| {
        SearchResult {
            title,
            poster_url: poster,
            url: format!("https://{}/view/?id={}", domain, id),
            source: "hsck".to_string(),
            total_episodes: None,
        }
    }).collect())
}
```

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/hsck_parser.rs
git commit -m "feat: add search, list_categories, browse to HsckParser"
```

---

### Task 3: Rongyok Parser — search + list_categories + browse

**Files:**
- Modify: `src-tauri/src/parser.rs` (add at end of `impl RongyokParser`, before line 330)

**Context:** RongyokParser uses `scraper` crate, has `client()` method at line 29, `get_series_info` at line 51. Site is WordPress-based. Search URL pattern: `https://{domain}/?s={query}`.

- [ ] **Step 1: Add imports**

At top of `parser.rs`, add:
```rust
use crate::{SearchResult, SiteCategory};
```

- [ ] **Step 2: Add search, list_categories, browse methods**

Add at end of `impl RongyokParser` (before the closing `}`):

```rust
pub async fn search(&self, query: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
    let url = format!("https://{}?s={}&paged={}", domain, query, page);
    let resp = self.client().get(&url).send().await.map_err(|e| e.to_string())?;
    let html_text = resp.text().await.map_err(|e| e.to_string())?;
    self.parse_listing_html(&html_text, domain)
}

pub fn list_categories(&self, _domain: &str) -> Vec<SiteCategory> {
    vec![
        SiteCategory { id: "latest".into(), label: "Latest".into(), source: "rongyok".into() },
        SiteCategory { id: "popular".into(), label: "Popular".into(), source: "rongyok".into() },
    ]
}

pub async fn browse(&self, category: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
    let url = match category {
        "latest" => format!("https://{}?paged={}", domain, page),
        "popular" => format!("https://{}/popular/?paged={}", domain, page),
        _ => format!("https://{}?paged={}", domain, page),
    };
    let resp = self.client().get(&url).send().await.map_err(|e| e.to_string())?;
    let html_text = resp.text().await.map_err(|e| e.to_string())?;
    self.parse_listing_html(&html_text, domain)
}

fn parse_listing_html(&self, html_text: &str, domain: &str) -> Result<Vec<SearchResult>, String> {
    let document = Html::parse_document(html_text);
    let mut results = Vec::new();

    let heading_sel = Selector::parse("h2 a, h3 a, .entry-title a, .post-title a").unwrap();
    let link_sel = Selector::parse("a").unwrap();
    let img_sel = Selector::parse("img").unwrap();

    // Try common WordPress selectors for post/article containers
    let selectors = ["article", ".post-item", ".entry-item", ".type-post", ".post"];
    for sel_str in &selectors {
        if let Ok(selector) = Selector::parse(sel_str) {
            for el in document.select(&selector) {
                let link = el.select(&link_sel).next()
                    .and_then(|a| a.value().attr("href").map(|h| h.to_string()));
                let title = el.select(&heading_sel).next()
                    .map(|a| a.text().collect::<String>().trim().to_string())
                    .or_else(|| el.select(&link_sel).next()
                        .and_then(|a| a.value().attr("title").map(|t| t.to_string())));
                let poster = el.select(&img_sel).next()
                    .and_then(|img| img.value().attr("src").or_else(|| img.value().attr("data-src")).map(|s| s.to_string()));

                if let (Some(url), Some(title)) = (link, title) {
                    if title.is_empty() || url.contains("wp-admin") || url.contains("wp-login") { continue; }
                    results.push(SearchResult {
                        title,
                        poster_url: poster,
                        url,
                        source: "rongyok".to_string(),
                        total_episodes: None,
                    });
                }
            }
        }
        if !results.is_empty() { break; }
    }
    Ok(results)
}
```

**IMPORTANT:** The selector in `parse_listing_html` is a best-effort guess. The implementer MUST inspect the actual HTML at `rongyok.com/?s=test` and adjust the CSS selectors to match the real page structure. The pattern of trying multiple selectors handles some variation.

- [ ] **Step 3: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/parser.rs
git commit -m "feat: add search, list_categories, browse to RongyokParser"
```

---

### Task 4: BaanJeen Parser — search + list_categories + browse

**Files:**
- Modify: `src-tauri/src/baanjeen_parser.rs` (add at end of `impl BaanJeenParser`, around line 415)

**Context:** BaanJeenParser uses `scraper`, has `client()` at line 30. WordPress-based. Domain is `xn--82c7abb4jua0l.com`.

- [ ] **Step 1: Add imports and methods**

Add at top:
```rust
use crate::{SearchResult, SiteCategory};
```

Add at end of `impl BaanJeenParser`:
```rust
pub async fn search(&self, query: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
    let url = format!("https://{}?s={}&paged={}", domain, query, page);
    let resp = self.client().get(&url).send().await.map_err(|e| e.to_string())?;
    let html_text = resp.text().await.map_err(|e| e.to_string())?;
    self.parse_listing_html(&html_text, domain, "baanjeen")
}

pub fn list_categories(&self, _domain: &str) -> Vec<SiteCategory> {
    vec![
        SiteCategory { id: "latest".into(), label: "Latest".into(), source: "baanjeen".into() },
        SiteCategory { id: "chinese-series".into(), label: "Chinese Series".into(), source: "baanjeen".into() },
        SiteCategory { id: "chinese-movie".into(), label: "Chinese Movies".into(), source: "baanjeen".into() },
    ]
}

pub async fn browse(&self, category: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
    let url = match category {
        "latest" => format!("https://{}?paged={}", domain, page),
        other => format!("https://{}/category/{}/?paged={}", domain, other, page),
    };
    let resp = self.client().get(&url).send().await.map_err(|e| e.to_string())?;
    let html_text = resp.text().await.map_err(|e| e.to_string())?;
    self.parse_listing_html(&html_text, domain, "baanjeen")
}

fn parse_listing_html(&self, html_text: &str, domain: &str, source: &str) -> Result<Vec<SearchResult>, String> {
    let document = Html::parse_document(html_text);
    let mut results = Vec::new();
    let article_sel = Selector::parse("article, .post-item, .entry-item").unwrap();

    for el in document.select(&article_sel) {
        let link_sel = Selector::parse("a").unwrap();
        let link = el.select(&link_sel).next().and_then(|a| a.value().attr("href")).map(|h| h.to_string());
        let title = el.select(&Selector::parse("h2 a, h3 a, .entry-title a").unwrap()).next()
            .map(|a| a.text().collect::<String>().trim().to_string())
            .or_else(|| el.select(&link_sel).next().and_then(|a| a.value().attr("title")).map(|t| t.to_string()));
        let poster = el.select(&Selector::parse("img").unwrap()).next()
            .and_then(|img| img.value().attr("src").or_else(|| img.value().attr("data-src")).map(|s| s.to_string()));

        if let (Some(url), Some(title)) = (link, title) {
            if title.is_empty() || url.contains("wp-admin") { continue; }
            results.push(SearchResult {
                title,
                poster_url: poster,
                url,
                source: source.to_string(),
                total_episodes: None,
            });
        }
    }
    Ok(results)
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/baanjeen_parser.rs
git commit -m "feat: add search, list_categories, browse to BaanJeenParser"
```

---

### Task 5: Titan Parser — search + list_categories + browse

**Files:**
- Modify: `src-tauri/src/titan_parser.rs` (add at end of `impl TitanParser`, around line 466)

**Context:** TitanParser uses `scraper`, has `client()` at line 58. Domain is `357ms.com`/`51cg1.com`. WordPress-based with `/archives/` pattern.

- [ ] **Step 1: Add imports and methods**

Add at top:
```rust
use crate::{SearchResult, SiteCategory};
```

Add at end of `impl TitanParser`:
```rust
pub async fn search(&self, query: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
    let url = format!("https://{}?s={}&paged={}", domain, query, page);
    let resp = self.client().get(&url).send().await.map_err(|e| e.to_string())?;
    let html_text = resp.text().await.map_err(|e| e.to_string())?;
    self.parse_listing_html(&html_text, domain)
}

pub fn list_categories(&self, _domain: &str) -> Vec<SiteCategory> {
    vec![
        SiteCategory { id: "latest".into(), label: "Latest".into(), source: "titan".into() },
        SiteCategory { id: "archives".into(), label: "Archives".into(), source: "titan".into() },
    ]
}

pub async fn browse(&self, category: &str, page: i32, domain: &str) -> Result<Vec<SearchResult>, String> {
    let url = match category {
        "latest" => format!("https://{}?paged={}", domain, page),
        "archives" => format!("https://{}/archives/?paged={}", domain, page),
        other => format!("https://{}/tag/{}/?paged={}", domain, other, page),
    };
    let resp = self.client().get(&url).send().await.map_err(|e| e.to_string())?;
    let html_text = resp.text().await.map_err(|e| e.to_string())?;
    self.parse_listing_html(&html_text, domain)
}

fn parse_listing_html(&self, html_text: &str, domain: &str) -> Result<Vec<SearchResult>, String> {
    let document = Html::parse_document(html_text);
    let mut results = Vec::new();
    let article_sel = Selector::parse("article, .post-item, .entry-item").unwrap();

    for el in document.select(&article_sel) {
        let link_sel = Selector::parse("a").unwrap();
        let link = el.select(&link_sel).next().and_then(|a| a.value().attr("href")).map(|h| h.to_string());
        let title = el.select(&Selector::parse("h2 a, h3 a, .entry-title a").unwrap()).next()
            .map(|a| a.text().collect::<String>().trim().to_string())
            .or_else(|| el.select(&link_sel).next().and_then(|a| a.value().attr("title")).map(|t| t.to_string()));
        let poster = el.select(&Selector::parse("img").unwrap()).next()
            .and_then(|img| img.value().attr("src").or_else(|| img.value().attr("data-src")).map(|s| s.to_string()));

        if let (Some(url), Some(title)) = (link, title) {
            if title.is_empty() || url.contains("wp-admin") { continue; }
            results.push(SearchResult {
                title,
                poster_url: poster,
                url,
                source: "titan".to_string(),
                total_episodes: None,
            });
        }
    }
    Ok(results)
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/titan_parser.rs
git commit -m "feat: add search, list_categories, browse to TitanParser"
```

---

### Task 6: Tauri Commands — search_sites, get_browse_categories, browse_category

**Files:**
- Modify: `src-tauri/src/lib.rs` (add commands after existing commands, around line 1049; register in invoke_handler around line 1139)

- [ ] **Step 1: Add search_sites command**

Add after `get_quality_options` function (after line ~1050), before `pub fn run()`:

```rust
#[tauri::command]
async fn search_sites(
    query: String,
    page: i32,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<SearchResponse>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    let settings = get_domain_settings(app_handle);
    let futures: Vec<std::pin::Pin<Box<dyn std::future::Future<Output = Option<SearchResponse>> + Send>>> = vec![
        Box::pin(async {
            match state.rongyok_parser.search(&query, page, &settings.rongyok_domain).await {
                Ok(results) => {
                    let has_more = results.len() >= 20;
                    Some(SearchResponse { results, source: "rongyok".into(), page, has_more })
                }
                Err(_) => None,
            }
        }),
        Box::pin(async {
            match state.baanjeen_parser.search(&query, page, &settings.baanjeen_domain).await {
                Ok(results) => {
                    let has_more = results.len() >= 20;
                    Some(SearchResponse { results, source: "baanjeen".into(), page, has_more })
                }
                Err(_) => None,
            }
        }),
        Box::pin(async {
            match state.titan_parser.search(&query, page, &settings.titan_domain).await {
                Ok(results) => {
                    let has_more = results.len() >= 20;
                    Some(SearchResponse { results, source: "titan".into(), page, has_more })
                }
                Err(_) => None,
            }
        }),
        Box::pin(async {
            match state.hsck_parser.search(&query, page, &settings.hsck_domain).await {
                Ok(results) => {
                    let has_more = results.len() >= 20;
                    Some(SearchResponse { results, source: "hsck".into(), page, has_more })
                }
                Err(_) => None,
            }
        }),
    ];

    let responses: Vec<SearchResponse> = futures_util::future::join_all(futures)
        .await
        .into_iter()
        .flatten()
        .collect();

    Ok(responses)
}

#[tauri::command]
fn get_browse_categories(state: State<'_, AppState>) -> Result<Vec<SiteCategory>, String> {
    let mut categories = Vec::new();
    // Use default domains for category listing — categories are static
    categories.extend(state.rongyok_parser.list_categories("rongyok.com"));
    categories.extend(state.baanjeen_parser.list_categories("xn--82c7abb4jua0l.com"));
    categories.extend(state.titan_parser.list_categories("51cg1.com"));
    categories.extend(state.hsck_parser.list_categories("hsck123.com"));
    Ok(categories)
}

#[tauri::command]
async fn browse_category(
    source: String,
    category: String,
    page: i32,
    state: State<'_, AppState>,
    app_handle: AppHandle,
) -> Result<SearchResponse, String> {
    let settings = get_domain_settings(app_handle);
    let results = match source.as_str() {
        "rongyok" => state.rongyok_parser.browse(&category, page, &settings.rongyok_domain).await?,
        "baanjeen" => state.baanjeen_parser.browse(&category, page, &settings.baanjeen_domain).await?,
        "titan" => state.titan_parser.browse(&category, page, &settings.titan_domain).await?,
        "hsck" => state.hsck_parser.browse(&category, page, &settings.hsck_domain).await?,
        _ => return Err(format!("Unknown source: {}", source)),
    };
    let has_more = results.len() >= 20;
    Ok(SearchResponse { results, source, page, has_more })
}
```

- [ ] **Step 2: Add `futures_util` import at top of lib.rs**

Add at the top imports section:
```rust
use futures_util::future;
```

Note: `futures-util = "0.3"` is already in `Cargo.toml` — no Cargo.toml change needed.

- [ ] **Step 3: Register commands in invoke_handler**

Add these 3 commands to the `invoke_handler` macro (before the closing `])`):

```rust
search_sites,
get_browse_categories,
browse_category,
```

- [ ] **Step 4: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: add search_sites, get_browse_categories, browse_category commands"
```

---

### Task 7: Frontend TypeScript Types

**Files:**
- Modify: `src/types/index.ts` (add after existing types, after line ~135)

- [ ] **Step 1: Add types**

Add at end of `src/types/index.ts`:

```typescript
export interface SearchResult {
  title: string;
  posterUrl?: string;
  url: string;
  source: string;
  totalEpisodes?: number;
}

export interface SiteCategory {
  id: string;
  label: string;
  source: string;
}

export interface SearchResponse {
  results: SearchResult[];
  source: string;
  page: number;
  hasMore: boolean;
}
```

- [ ] **Step 2: Commit**

```bash
git add src/types/index.ts
git commit -m "feat: add SearchResult, SiteCategory, SearchResponse TypeScript types"
```

---

### Task 8: BrowseCard + BrowseDetail Components

**Files:**
- Create: `src/components/BrowseCard.tsx`
- Create: `src/components/BrowseDetail.tsx`

- [ ] **Step 1: Create BrowseCard.tsx**

```tsx
import React from "react";
import { Image as ImageIcon } from "lucide-react";
import type { SearchResult } from "../types";

const SOURCE_COLORS: Record<string, string> = {
  rongyok: "bg-violet-500/20 text-violet-300",
  baanjeen: "bg-blue-500/20 text-blue-300",
  titan: "bg-amber-500/20 text-amber-300",
  hsck: "bg-emerald-500/20 text-emerald-300",
};

interface BrowseCardProps {
  result: SearchResult;
  onClick: (result: SearchResult) => void;
}

export default function BrowseCard({ result, onClick }: BrowseCardProps) {
  return (
    <div
      className="group cursor-pointer rounded-lg border border-slate-700/50 bg-slate-800/30 overflow-hidden hover:border-slate-600 hover:bg-slate-800/60 transition-all"
      onClick={() => onClick(result)}
    >
      <div className="aspect-[2/3] bg-slate-900 relative overflow-hidden">
        {result.posterUrl ? (
          <img
            src={result.posterUrl}
            alt={result.title}
            className="w-full h-full object-cover group-hover:scale-105 transition-transform duration-300"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center">
            <ImageIcon size={24} className="text-slate-600" />
          </div>
        )}
        <span
          className={`absolute top-1.5 left-1.5 px-1.5 py-0.5 rounded text-[10px] font-bold ${
            SOURCE_COLORS[result.source] || "bg-slate-500/20 text-slate-300"
          }`}
        >
          {result.source}
        </span>
        {result.totalEpisodes && (
          <span className="absolute top-1.5 right-1.5 px-1.5 py-0.5 rounded text-[10px] bg-black/60 text-white">
            EP {result.totalEpisodes}
          </span>
        )}
      </div>
      <div className="p-2">
        <p className="text-xs text-slate-200 line-clamp-2 leading-tight">
          {result.title}
        </p>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Create BrowseDetail.tsx**

```tsx
import React, { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeft, Download, Loader2, AlertCircle } from "lucide-react";
import { Button, EpisodeSelector } from "./";
import QualitySelector from "./QualitySelector";
import type { SearchResult, SeriesInfo } from "../types";

interface BrowseDetailProps {
  result: SearchResult;
  onBack: () => void;
  settings: { outputDir: string; autoMerge: boolean; concurrentDownloads: number; speedLimit: number; fileNaming: string; groupBySource: boolean; defaultQuality: string };
  ffmpegAvailable: boolean;
}

export default function BrowseDetail({ result, onBack, settings, ffmpegAvailable }: BrowseDetailProps) {
  const [series, setSeries] = useState<SeriesInfo | null>(null);
  const [selectedEpisodes, setSelectedEpisodes] = useState<Set<number>>(new Set());
  const [selectedQuality, setSelectedQuality] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadSeries = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const info = await invoke<SeriesInfo>("fetch_series", { url: result.url });
      setSeries(info);
      setSelectedEpisodes(new Set(Array.from({ length: info.totalEpisodes }, (_, i) => i + 1)));
    } catch (e) {
      setError(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  const handleDownload = async () => {
    if (!series || selectedEpisodes.size === 0) return;
    try {
      await invoke("update_series_state", { series });
      const episodes = Array.from(selectedEpisodes).sort((a, b) => a - b);
      await invoke("start_download", {
        request: {
          seriesId: series.seriesId,
          episodes,
          outputDir: settings.outputDir,
          autoMerge: settings.autoMerge && ffmpegAvailable,
          concurrentDownloads: settings.concurrentDownloads,
          speedLimit: settings.speedLimit,
          fileNaming: settings.fileNaming,
          seriesTitle: series.title,
          groupBySource: settings.groupBySource,
          preferredQuality: selectedQuality,
        },
      });
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="p-3 space-y-4">
      <button onClick={onBack} className="flex items-center gap-1.5 text-xs text-slate-400 hover:text-white">
        <ArrowLeft size={14} /> Back to browse
      </button>

      <div className="flex gap-4">
        {result.posterUrl && (
          <img src={result.posterUrl} alt={result.title} className="w-32 h-48 object-cover rounded-lg" />
        )}
        <div className="flex-1 space-y-2">
          <h2 className="text-sm font-bold text-white">{result.title}</h2>
          <p className="text-xs text-slate-400">Source: {result.source}</p>
          {result.totalEpisodes && <p className="text-xs text-slate-400">Episodes: {result.totalEpisodes}</p>}

          {!series && !isLoading && !error && (
            <Button onClick={loadSeries} leftIcon={<Download size={14} />} size="sm">
              Load Series
            </Button>
          )}
        </div>
      </div>

      {isLoading && (
        <div className="flex items-center justify-center py-8 gap-2 text-slate-400">
          <Loader2 size={20} className="animate-spin" /> Loading series info...
        </div>
      )}

      {error && (
        <div className="flex items-center gap-2 p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-xs text-red-300">
          <AlertCircle size={14} />
          <span>Could not load series: {error}</span>
          <Button onClick={loadSeries} size="sm" variant="ghost" className="ml-auto">Retry</Button>
        </div>
      )}

      {series && (
        <div className="space-y-3">
          <EpisodeSelector
            totalEpisodes={series.totalEpisodes}
            selectedEpisodes={selectedEpisodes}
            onToggle={(ep) => {
              setSelectedEpisodes((prev) => {
                const next = new Set(prev);
                next.has(ep) ? next.delete(ep) : next.add(ep);
                return next;
              });
            }}
            onSelectAll={() => setSelectedEpisodes(new Set(Array.from({ length: series.totalEpisodes }, (_, i) => i + 1)))}
            onDeselectAll={() => setSelectedEpisodes(new Set())}
          />
          <QualitySelector
            episodeUrl={Object.values(series.episodeUrls)[0]}
            onSelect={(q) => setSelectedQuality(q)}
            defaultQuality={settings.defaultQuality || "best"}
          />
          <Button onClick={handleDownload} leftIcon={<Download size={14} />} variant="success" disabled={selectedEpisodes.size === 0}>
            Download ({selectedEpisodes.size})
          </Button>
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 3: Verify frontend compiles**

Run: `npx tsc --noEmit`
Expected: No type errors (QualitySelector is imported directly from `./QualitySelector`)

- [ ] **Step 4: Commit**

```bash
git add src/components/BrowseCard.tsx src/components/BrowseDetail.tsx
git commit -m "feat: add BrowseCard and BrowseDetail components"
```

---

### Task 9: BrowsePanel Component

**Files:**
- Create: `src/components/BrowsePanel.tsx`

- [ ] **Step 1: Create BrowsePanel.tsx**

```tsx
import React, { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Search, Loader2, Compass, X } from "lucide-react";
import { Button } from "./";
import BrowseCard from "./BrowseCard";
import BrowseDetail from "./BrowseDetail";
import type { SearchResult, SiteCategory, SearchResponse } from "../types";

const SOURCE_FILTERS = [
  { id: "", label: "All" },
  { id: "rongyok", label: "Rongyok" },
  { id: "baanjeen", label: "BaanJeen" },
  { id: "titan", label: "Titan" },
  { id: "hsck", label: "Hsck" },
];

const SOURCE_COLORS: Record<string, string> = {
  rongyok: "border-violet-500/30 text-violet-300",
  baanjeen: "border-blue-500/30 text-blue-300",
  titan: "border-amber-500/30 text-amber-300",
  hsck: "border-emerald-500/30 text-emerald-300",
};

interface BrowsePanelProps {
  settings: any;
  ffmpegAvailable: boolean;
}

export default function BrowsePanel({ settings, ffmpegAvailable }: BrowsePanelProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResponse[]>([]);
  const [categories, setCategories] = useState<SiteCategory[]>([]);
  const [selectedSource, setSelectedSource] = useState("");
  const [activeCategory, setActiveCategory] = useState<{ source: string; id: string } | null>(null);
  const [detail, setDetail] = useState<SearchResult | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);

  useEffect(() => {
    invoke<SiteCategory[]>("get_browse_categories").then(setCategories).catch(() => {});
  }, []);

  const doSearch = useCallback(async (q: string, page: number = 1) => {
    if (!q.trim()) return;
    setIsLoading(true);
    try {
      const responses = await invoke<SearchResponse[]>("search_sites", { query: q, page });
      if (page === 1) {
        setResults(responses);
      } else {
        setResults((prev) => {
          const merged = [...prev];
          for (const resp of responses) {
            const existing = merged.find((r) => r.source === resp.source);
            if (existing) {
              existing.results = [...existing.results, ...resp.results];
              existing.hasMore = resp.hasMore;
            } else {
              merged.push(resp);
            }
          }
          return merged;
        });
      }
      setCurrentPage(page);
    } catch (e) {
      console.error("Search failed:", e);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const doBrowse = useCallback(async (source: string, category: string, page: number = 1) => {
    setIsLoading(true);
    setActiveCategory({ source, id: category });
    try {
      const response = await invoke<SearchResponse>("browse_category", { source, category, page });
      setResults([response]);
      setCurrentPage(page);
    } catch (e) {
      console.error("Browse failed:", e);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const allResults = results
    .filter((r) => !selectedSource || r.source === selectedSource)
    .flatMap((r) => r.results);

  const hasMore = results.some((r) => (!selectedSource || r.source === selectedSource) && r.hasMore);

  if (detail) {
    return (
      <div className="h-full overflow-y-auto custom-scrollbar">
        <BrowseDetail
          result={detail}
          onBack={() => setDetail(null)}
          settings={settings}
          ffmpegAvailable={ffmpegAvailable}
        />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Search Bar */}
      <div className="p-3 space-y-2 border-b border-slate-700/50 bg-slate-900/50">
        <div className="flex gap-2">
          <div className="flex-1 relative">
            <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-400" />
            <input
              type="text"
              placeholder="Search across all sites..."
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => e.key === "Enter" && doSearch(query)}
              className="w-full pl-8 pr-3 py-1.5 text-xs bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500/50"
            />
          </div>
          <Button size="sm" onClick={() => doSearch(query)} isLoading={isLoading}>
            <Search size={14} />
          </Button>
        </div>

        {/* Source Filters */}
        <div className="flex gap-1 overflow-x-auto pb-1">
          {SOURCE_FILTERS.map((f) => (
            <button
              key={f.id}
              onClick={() => setSelectedSource(f.id)}
              className={`px-2 py-0.5 text-[10px] font-bold rounded-full whitespace-nowrap transition-all ${
                selectedSource === f.id
                  ? "bg-cyan-500/20 text-cyan-300 border border-cyan-500/40"
                  : "bg-slate-800 text-slate-400 border border-slate-700 hover:text-white"
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>

        {/* Category Pills */}
        {categories.length > 0 && (
          <div className="flex gap-1 overflow-x-auto pb-1">
            {categories.map((cat) => (
              <button
                key={`${cat.source}-${cat.id}`}
                onClick={() => doBrowse(cat.source, cat.id)}
                className={`px-2 py-0.5 text-[10px] rounded-full whitespace-nowrap border transition-all ${
                  activeCategory?.source === cat.source && activeCategory?.id === cat.id
                    ? `bg-slate-700 text-white ${SOURCE_COLORS[cat.source] || "border-slate-600"}`
                    : "bg-slate-800/50 text-slate-400 border-slate-700 hover:text-white"
                }`}
              >
                <span className="opacity-60">{cat.source}:</span> {cat.label}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Results Grid */}
      <div className="flex-1 overflow-y-auto custom-scrollbar p-3">
        {allResults.length === 0 && !isLoading && (
          <div className="h-full flex flex-col items-center justify-center text-slate-500 opacity-40">
            <Compass size={40} className="mb-3" />
            <p className="text-sm">Search or browse categories to discover content</p>
          </div>
        )}

        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-2">
          {allResults.map((result, i) => (
            <BrowseCard key={`${result.source}-${i}`} result={result} onClick={setDetail} />
          ))}
        </div>

        {isLoading && (
          <div className="flex justify-center py-4">
            <Loader2 size={20} className="animate-spin text-cyan-400" />
          </div>
        )}

        {hasMore && !isLoading && (
          <div className="flex justify-center py-3">
            <Button size="sm" variant="ghost" onClick={() => {
              if (activeCategory) {
                doBrowse(activeCategory.source, activeCategory.id, currentPage + 1);
              } else {
                doSearch(query, currentPage + 1);
              }
            }}>
              Load more
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Verify frontend compiles**

Run: `npx tsc --noEmit`
Expected: No type errors

- [ ] **Step 3: Commit**

```bash
git add src/components/BrowsePanel.tsx
git commit -m "feat: add BrowsePanel component with search and category browsing"
```

---

### Task 10: App.tsx Integration + Component Exports

**Files:**
- Modify: `src/components/index.ts` (add 3 exports)
- Modify: `src/App.tsx` (add browse tab)

- [ ] **Step 1: Add exports to components/index.ts**

Add after the last export:
```typescript
export { default as BrowsePanel } from "./BrowsePanel";
export { default as BrowseCard } from "./BrowseCard";
export { default as BrowseDetail } from "./BrowseDetail";
```

- [ ] **Step 2: Add browse tab to App.tsx**

In `src/App.tsx`:

a) Add `Compass` to the lucide-react import (line 1-27):
```typescript
import { ..., Compass } from "lucide-react";
```

b) Add `BrowsePanel` to component imports (lines 28-44):
```typescript
import { ..., BrowsePanel } from "./components";
```

c) Update `TabType` (line 89):
```typescript
type TabType = "download" | "library" | "browse" | "files" | "history" | "settings" | "logs";
```

d) Update `tabs` array (line 250):
```typescript
const tabs: TabType[] = ["download", "library", "browse", "files", "history", "settings", "logs"];
```

e) Add browse tab to `tabsConfig` array (insert after the library entry, before files):
```typescript
{
  id: "browse",
  label: "Browse",
  icon: (
    <Compass
      size={16}
      className="drop-shadow-[0_0_4px_rgba(52,211,153,0.6)]"
    />
  ),
  glowColor: "emerald",
  activeClass: "bg-emerald-500/20 text-emerald-300 border border-emerald-500/40",
},
```

f) Add browse tab rendering (insert after the library tab block, before files tab):
```tsx
{activeTab === "browse" && (
  <div className="page-transition animate-fade-in h-full overflow-hidden">
    <BrowsePanel settings={settings} ffmpegAvailable={ffmpegAvailable} />
  </div>
)}
```

- [ ] **Step 3: Verify frontend compiles**

Run: `npx tsc --noEmit`
Expected: No type errors

- [ ] **Step 4: Full build test**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add src/components/index.ts src/App.tsx
git commit -m "feat: integrate Browse tab into app with search and category browsing"
```

---

### Task 11: Manual Testing & Selector Verification

**Files:** None (testing only)

- [ ] **Step 1: Launch the app**

Run: `npm run tauri dev`

- [ ] **Step 2: Test Browse tab appears**

Click the "Browse" tab in the header. Verify the search bar and category pills appear.

- [ ] **Step 3: Test search**

Type a keyword (e.g., "test") and press Enter. Check that results appear from at least one site. If a site returns 0 results, its HTML selectors in `parse_listing_html` need adjustment — inspect the actual HTML and fix the CSS selectors.

- [ ] **Step 4: Test category browsing**

Click a category pill. Verify results load for that category.

- [ ] **Step 5: Test detail view**

Click a result card. Verify the detail view loads with poster and "Load Series" button. Click "Load Series" and verify episodes appear.

- [ ] **Step 6: Fix selectors if needed**

If search returns no results for a site, use a browser to inspect the search results page HTML at `https://{domain}/?s=test` and update the CSS selectors in the parser's `parse_listing_html` method to match the actual page structure.

- [ ] **Step 7: Commit any selector fixes**

```bash
git add -A
git commit -m "fix: update HTML selectors for accurate search results parsing"
```
