# Library Organization Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add tags, favorites, configurable sorting/filtering, and episode-level actions to the content library.

**Architecture:** Extend SQLite with migration system (version 2), add `favorite` column + `library_tags`/`library_tag_map` tables, implement query builder with CTE for computed fields, separate query for tags loading. Frontend gets toolbar with sort/filter controls, tag chips, favorite stars on cards, and interactive episode boxes in SeriesDetail.

**Tech Stack:** Rust (rusqlite, serde, tauri), React 19, TypeScript, Tailwind CSS, lucide-react icons

**Spec:** `docs/superpowers/specs/2026-04-11-library-organization-design.md`

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src-tauri/src/library.rs` | Modify | Add migration runner, new structs, tag CRUD, query builder, fix update_episode_status, update get_series_detail |
| `src-tauri/src/lib.rs` | Modify | Add 7 new Tauri commands, modify cmd_get_library, deprecate cmd_search_library |
| `src/types/index.ts` | Modify | Add LibraryTag, LibraryQuery; extend LibraryEntry with favorite + tags |
| `src/hooks/useLibrary.ts` | Modify | Replace search with LibraryQuery, add tag/favorite/episode methods |
| `src/components/LibraryCard.tsx` | Modify | Add favorite star overlay, tag badges below title |
| `src/components/LibraryPanel.tsx` | Modify | Add toolbar row, tag filter chips, wire new query params |
| `src/components/SeriesDetail.tsx` | Modify | Add tag chips with add/remove, episode click actions, lastDownloaded display |

---

## Task 1: Database Migration System

**Files:**
- Modify: `src-tauri/src/library.rs`

- [ ] **Step 1: Add `PRAGMA foreign_keys=ON` to connection setup**

In `LibraryDb::new()`, modify the existing WAL pragma line to include foreign keys:

```rust
// Line 58 — replace:
conn.execute_batch("PRAGMA journal_mode=WAL;")
// With:
conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
```

- [ ] **Step 2: Add `run_migrations()` method after `init_schema()`**

After the `init_schema()` method (after line 110), add:

```rust
fn run_migrations(&self) -> Result<(), String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;

    let current_version: i32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version", [],
        |row| row.get(0),
    ).unwrap_or(0);

    if current_version < 2 {
        Self::migration_2_add_favorites_and_tags(&conn)?;
        conn.execute("INSERT OR IGNORE INTO schema_version (version) VALUES (2)", [])
            .map_err(|e| format!("Migration version update failed: {}", e))?;
    }

    Ok(())
}

fn migration_2_add_favorites_and_tags(conn: &Connection) -> Result<(), String> {
    // Idempotent: check if column exists before ALTER
    let has_favorite: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM pragma_table_info('library') WHERE name='favorite'",
        [], |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    if !has_favorite {
        conn.execute("ALTER TABLE library ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0", [])
            .map_err(|e| format!("Migration 2 (favorite column) failed: {}", e))?;
    }

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS library_tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        );
        CREATE TABLE IF NOT EXISTS library_tag_map (
            library_id INTEGER NOT NULL REFERENCES library(id) ON DELETE CASCADE,
            tag_id INTEGER NOT NULL REFERENCES library_tags(id) ON DELETE CASCADE,
            PRIMARY KEY (library_id, tag_id)
        );"
    ).map_err(|e| format!("Migration 2 (tag tables) failed: {}", e))?;

    Ok(())
}
```

- [ ] **Step 3: Call `run_migrations()` after `init_schema()` in `new()`**

In `LibraryDb::new()`, after `db.init_schema()?;` (line 69), add:

```rust
db.run_migrations()?;
```

- [ ] **Step 4: Verify with `cargo check`**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/library.rs
git commit -m "feat(library): add migration system with favorites and tags schema"
```

---

## Task 2: New Rust Structs + Modified LibraryEntry

**Files:**
- Modify: `src-tauri/src/library.rs`

- [ ] **Step 1: Add `LibraryTag` and `LibraryQuery` structs after imports**

After the existing `SeriesDetail` struct (after line 43), add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryTag {
    pub id: i64,
    pub name: String,
    pub usage_count: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryQuery {
    pub sort: Option<String>,
    pub order: Option<String>,
    pub source: Option<String>,
    pub status: Option<String>,
    pub tag_id: Option<i64>,
    pub favorite_only: Option<bool>,
    pub search: Option<String>,
}
```

- [ ] **Step 2: Add `favorite` and `tags` fields to `LibraryEntry`**

Modify the `LibraryEntry` struct to add two new fields:

```rust
pub struct LibraryEntry {
    // ... existing fields ...
    pub completed_count: i32,
    pub favorite: bool,
    pub tags: Vec<LibraryTag>,
}
```

- [ ] **Step 3: Update all `LibraryEntry` construction sites**

Update `get_library()`, `get_series_detail()`, and `search_library()` query maps to include:
- `favorite: row.get::<_, i32>(N)? != 0` (at the new column index after completed_count)
- `tags: Vec::new()` (populated later by `get_tags_for_entries`)

Update SELECT statements to include `l.favorite` column.

- [ ] **Step 4: Verify with `cargo check`**

Run: `cd src-tauri && cargo check`
Expected: Compiles (will need to update query maps in get_library, get_series_detail, search_library)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/library.rs
git commit -m "feat(library): add LibraryTag, LibraryQuery structs; extend LibraryEntry"
```

---

## Task 3: Tag CRUD Methods

**Files:**
- Modify: `src-tauri/src/library.rs`

- [ ] **Step 1: Add tag methods to `impl LibraryDb`**

Add these methods after `run_migrations()`:

```rust
pub fn get_tags(&self) -> Result<Vec<LibraryTag>, String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, COUNT(tmap.library_id) as usage_count
         FROM library_tags t
         LEFT JOIN library_tag_map tmap ON tmap.tag_id = t.id
         GROUP BY t.id ORDER BY t.name"
    ).map_err(|e| e.to_string())?;

    let tags: Vec<LibraryTag> = stmt.query_map([], |row| {
        Ok(LibraryTag { id: row.get(0)?, name: row.get(1)?, usage_count: row.get(2)? })
    }).map_err(|e| e.to_string())?.filter_map(|t| t.ok()).collect();

    Ok(tags)
}

pub fn create_tag(&self, name: &str) -> Result<i64, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Tag name cannot be empty".to_string());
    }
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO library_tags (name) VALUES (?1)", params![trimmed])
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint") {
                "Tag already exists".to_string()
            } else {
                format!("Create tag failed: {}", e)
            }
        })?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_tag(&self, tag_id: i64) -> Result<(), String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    // CASCADE on library_tag_map handles mapping cleanup
    conn.execute("DELETE FROM library_tags WHERE id = ?1", params![tag_id])
        .map_err(|e| format!("Delete tag failed: {}", e))?;
    Ok(())
}

pub fn assign_tag(&self, library_id: i64, tag_id: i64) -> Result<(), String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR IGNORE INTO library_tag_map (library_id, tag_id) VALUES (?1, ?2)",
        params![library_id, tag_id],
    ).map_err(|e| format!("Assign tag failed: {}", e))?;
    Ok(())
}

pub fn unassign_tag(&self, library_id: i64, tag_id: i64) -> Result<(), String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM library_tag_map WHERE library_id = ?1 AND tag_id = ?2",
        params![library_id, tag_id],
    ).map_err(|e| format!("Unassign tag failed: {}", e))?;
    Ok(())
}
```

- [ ] **Step 2: Add `get_tags_for_entries` private helper**

```rust
fn get_tags_for_entries(conn: &Connection, ids: &[i64]) -> Result<std::collections::HashMap<i64, Vec<LibraryTag>>, String> {
    if ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let placeholders: Vec<String> = ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
    let sql = format!(
        "SELECT tmap.library_id, t.id, t.name, COUNT(*) OVER (PARTITION BY t.id) as usage_count
         FROM library_tag_map tmap
         JOIN library_tags t ON t.id = tmap.tag_id
         WHERE tmap.library_id IN ({})",
        placeholders.join(",")
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let params: Vec<&dyn rusqlite::ToSql> = ids.iter().map(|id| id as &dyn rusqlite::ToSql).collect();
    let mut map: std::collections::HashMap<i64, Vec<LibraryTag>> = std::collections::HashMap::new();
    for row in stmt.query_map(params.as_slice(), |row| {
        Ok((row.get::<_, i64>(0)?, LibraryTag { id: row.get(1)?, name: row.get(2)?, usage_count: row.get(3)? }))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()) {
        map.entry(row.0).or_default().push(row.1);
    }
    Ok(map)
}
```

- [ ] **Step 3: Verify with `cargo check`**

Run: `cd src-tauri && cargo check`

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/library.rs
git commit -m "feat(library): add tag CRUD methods and get_tags_for_entries helper"
```

---

## Task 4: Favorites Toggle + Episode File Path

**Files:**
- Modify: `src-tauri/src/library.rs`

- [ ] **Step 1: Add `toggle_favorite` and `get_episode_file_path` methods**

```rust
pub fn toggle_favorite(&self, library_id: i64) -> Result<bool, String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    let current: bool = conn.query_row(
        "SELECT favorite FROM library WHERE id = ?1", params![library_id],
        |row| row.get::<_, i32>(0).map(|v| v != 0),
    ).map_err(|e| format!("Toggle favorite failed: {}", e))?;

    let new_val = !current;
    conn.execute(
        "UPDATE library SET favorite = ?1 WHERE id = ?2",
        params![new_val as i32, library_id],
    ).map_err(|e| format!("Toggle favorite failed: {}", e))?;

    Ok(new_val)
}

pub fn get_episode_file_path(&self, library_id: i64, episode_number: i32) -> Result<Option<String>, String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    let path: Option<String> = conn.query_row(
        "SELECT file_path FROM library_episodes WHERE library_id = ?1 AND episode_number = ?2",
        params![library_id, episode_number],
        |row| row.get(0),
    ).unwrap_or(None);

    // Verify file exists
    if let Some(ref p) = path {
        if !std::path::Path::new(p).exists() {
            return Ok(None);
        }
    }
    Ok(path)
}
```

- [ ] **Step 2: Verify with `cargo check`**

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/library.rs
git commit -m "feat(library): add toggle_favorite and get_episode_file_path"
```

---

## Task 5: Query Builder — Modified `get_library()` with Filters

**Files:**
- Modify: `src-tauri/src/library.rs`

- [ ] **Step 1: Replace existing `get_library()` with query-building version**

Replace the existing `get_library()` method (lines 179-209) with:

```rust
pub fn get_library(&self, query: Option<LibraryQuery>) -> Result<Vec<LibraryEntry>, String> {
    let conn = self.conn.lock().map_err(|e| e.to_string())?;
    let q = query.unwrap_or_default();

    // Build dynamic SQL
    let mut where_clauses: Vec<String> = Vec::new();
    let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref source) = q.source {
        where_clauses.push(format!("ec.source = ?{}", param_values.len() + 1));
        param_values.push(Box::new(source.clone()));
    }
    if q.favorite_only.unwrap_or(false) {
        where_clauses.push("ec.favorite = 1".to_string());
    }
    if let Some(ref search) = q.search {
        let pattern = format!("%{}%", search);
        where_clauses.push(format!("LOWER(ec.title) LIKE LOWER(?{})", param_values.len() + 1));
        param_values.push(Box::new(pattern));
    }
    if let Some(tag_id) = q.tag_id {
        where_clauses.push(format!(
            "ec.id IN (SELECT library_id FROM library_tag_map WHERE tag_id = ?{})",
            param_values.len() + 1
        ));
        param_values.push(Box::new(tag_id));
    }

    // Status filter on computed completed_count
    match q.status.as_deref() {
        Some("complete") => where_clauses.push("ec.completed_count = ec.total_episodes".to_string()),
        Some("in_progress") => where_clauses.push("ec.completed_count > 0 AND ec.completed_count < ec.total_episodes".to_string()),
        Some("not_started") => where_clauses.push("ec.completed_count = 0".to_string()),
        _ => {}
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_clauses.join(" AND "))
    };

    // Sort column mapping (whitelist)
    let order_col = match q.sort.as_deref() {
        Some("title") => "LOWER(ec.title)",
        Some("source") => "ec.source",
        Some("last_downloaded") => "ec.last_downloaded DESC NULLS LAST",
        Some("progress") => "(ec.completed_count * 1.0 / ec.total_episodes) DESC",
        _ => "ec.date_added",
    };
    let order_dir = match q.order.as_deref() {
        Some("asc") => "ASC",
        Some("desc") => "DESC",
        _ if q.sort.as_deref() == Some("progress") || q.sort.as_deref() == Some("last_downloaded") => "",
        _ => "DESC",
    };
    let order_sql = format!("ORDER BY {} {}", order_col, order_dir).trim_end().to_string();

    let sql = format!(
        "WITH entry_counts AS (
            SELECT l.*, COUNT(CASE WHEN e.status = 'completed' THEN 1 END) as completed_count
            FROM library l
            LEFT JOIN library_episodes e ON e.library_id = l.id
            GROUP BY l.id
        )
        SELECT ec.id, ec.parser_series_id, ec.title, ec.source, ec.source_url,
               ec.poster_path, ec.total_episodes, ec.date_added, ec.last_downloaded,
               ec.completed_count, ec.favorite
        FROM entry_counts ec
        {where_sql}
        {order_sql}"
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("Query library failed: {}", e))?;
    let params: Vec<&dyn rusqlite::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

    let entries: Vec<LibraryEntry> = stmt.query_map(params.as_slice(), |row| {
        Ok(LibraryEntry {
            id: row.get(0)?,
            parser_series_id: row.get(1)?,
            title: row.get(2)?,
            source: row.get(3)?,
            source_url: row.get(4)?,
            poster_path: row.get(5)?,
            total_episodes: row.get(6)?,
            date_added: row.get(7)?,
            last_downloaded: row.get(8)?,
            completed_count: row.get(9)?,
            favorite: row.get::<_, i32>(10)? != 0,
            tags: Vec::new(),
        })
    }).map_err(|e| format!("Map library entries failed: {}", e))?
    .filter_map(|e| e.ok())
    .collect();

    // Load tags for all entries via separate query
    let ids: Vec<i64> = entries.iter().map(|e| e.id).collect();
    let tags_map = Self::get_tags_for_entries(&conn, &ids)?;

    let entries = entries.into_iter().map(|mut e| {
        e.tags = tags_map.get(&e.id).cloned().unwrap_or_default();
        e
    }).collect();

    Ok(entries)
}
```

- [ ] **Step 2: Update `search_library()` as deprecated wrapper**

```rust
// DEPRECATED: Use get_library(Some(LibraryQuery { search: Some(query), ..Default::default() }))
pub fn search_library(&self, query: &str) -> Result<Vec<LibraryEntry>, String> {
    self.get_library(Some(LibraryQuery { search: Some(query.to_string()), ..Default::default() }))
}
```

- [ ] **Step 3: Update `get_series_detail()` to include favorite + tags**

Modify `get_series_detail()` to:
1. Add `l.favorite` to the SELECT
2. Read it as `favorite: row.get::<_, i32>(10)? != 0`
3. After constructing the entry, call `get_tags_for_entries(&conn, &[library_id])`
4. Attach tags to the entry

- [ ] **Step 4: Verify with `cargo check`**

Run: `cd src-tauri && cargo check`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/library.rs
git commit -m "feat(library): implement query builder with CTE, filters, sort, tags loading"
```

---

## Task 6: Fix `update_episode_status`

**Files:**
- Modify: `src-tauri/src/library.rs`

- [ ] **Step 1: Add `last_downloaded` update on episode completion**

After the existing UPDATE for episode status, add:

```rust
// In update_episode_status(), after the existing conn.execute UPDATE:
if status == "completed" {
    conn.execute(
        "UPDATE library SET last_downloaded = datetime('now') WHERE id = ?1",
        params![library_id],
    ).map_err(|e| format!("Update last_downloaded failed: {}", e))?;
}
```

- [ ] **Step 2: Verify with `cargo check`**

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/library.rs
git commit -m "fix(library): update last_downloaded when episode completes"
```

---

## Task 7: Tauri Commands

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Modify `cmd_get_library` to accept `Option<LibraryQuery>`**

```rust
#[tauri::command]
fn cmd_get_library(state: State<'_, AppState>, query: Option<library::LibraryQuery>) -> Result<Vec<library::LibraryEntry>, String> {
    state.library_db.get_library(query)
}
```

- [ ] **Step 1b: Add `// DEPRECATED` comment on `cmd_search_library` in lib.rs**

The existing `cmd_search_library` (line 982) already calls `search_library(&query)`, which after Task 5 becomes the deprecated wrapper. Add a `// DEPRECATED: use cmd_get_library(Some(LibraryQuery { search: Some(query), ..Default::default() }))` comment above it.

- [ ] **Step 2: Add 7 new commands before `pub fn run()`**

```rust
#[tauri::command]
fn cmd_get_tags(state: State<'_, AppState>) -> Result<Vec<library::LibraryTag>, String> {
    state.library_db.get_tags()
}

#[tauri::command]
fn cmd_create_tag(state: State<'_, AppState>, name: String) -> Result<i64, String> {
    state.library_db.create_tag(&name)
}

#[tauri::command]
fn cmd_delete_tag(state: State<'_, AppState>, tag_id: i64) -> Result<(), String> {
    state.library_db.delete_tag(tag_id)
}

#[tauri::command]
fn cmd_assign_tag(state: State<'_, AppState>, library_id: i64, tag_id: i64) -> Result<(), String> {
    state.library_db.assign_tag(library_id, tag_id)
}

#[tauri::command]
fn cmd_unassign_tag(state: State<'_, AppState>, library_id: i64, tag_id: i64) -> Result<(), String> {
    state.library_db.unassign_tag(library_id, tag_id)
}

#[tauri::command]
fn cmd_toggle_favorite(state: State<'_, AppState>, library_id: i64) -> Result<bool, String> {
    state.library_db.toggle_favorite(library_id)
}

#[tauri::command]
fn cmd_open_episode(state: State<'_, AppState>, library_id: i64, episode_number: i32) -> Result<(), String> {
    let path = state.library_db.get_episode_file_path(library_id, episode_number)?;
    let path = path.ok_or("Episode file not found")?;

    #[cfg(target_os = "macos")]
    { std::process::Command::new("open").arg(&path).spawn().map_err(|e| e.to_string())?; }
    #[cfg(target_os = "windows")]
    { std::process::Command::new("cmd").args(["/c", "start", "", &path]).spawn().map_err(|e| e.to_string())?; }
    #[cfg(target_os = "linux")]
    { std::process::Command::new("xdg-open").arg(&path).spawn().map_err(|e| e.to_string())?; }
    Ok(())
}
```

- [ ] **Step 3: Register all new commands in `invoke_handler`**

Add to the `tauri::generate_handler![]` list:
```rust
cmd_get_tags,
cmd_create_tag,
cmd_delete_tag,
cmd_assign_tag,
cmd_unassign_tag,
cmd_toggle_favorite,
cmd_open_episode,
```

- [ ] **Step 4: Verify with `cargo check`**

Run: `cd src-tauri && cargo check`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add 7 Tauri commands for tags, favorites, episode open"
```

---

## Task 8: Frontend Types

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: Add `LibraryTag` and `LibraryQuery` interfaces**

After the `SeriesDetail` interface (around line 90), add:

```typescript
export interface LibraryTag {
  id: number;
  name: string;
  usageCount: number;
}

export interface LibraryQuery {
  sort?: string;
  order?: string;
  source?: string;
  status?: string;
  tagId?: number;
  favoriteOnly?: boolean;
  search?: string;
}
```

- [ ] **Step 2: Extend `LibraryEntry` with `favorite` and `tags`**

Add to the `LibraryEntry` interface:

```typescript
export interface LibraryEntry {
  // ... existing fields ...
  completedCount: number;
  favorite: boolean;
  tags: LibraryTag[];
}
```

- [ ] **Step 3: Verify with `npx tsc --noEmit`**

Run: `cd /Volumes/Data/workspace/git/video-downloader-tauri && npx tsc --noEmit`
Expected: No type errors

- [ ] **Step 4: Commit**

```bash
git add src/types/index.ts
git commit -m "feat: add LibraryTag, LibraryQuery types; extend LibraryEntry"
```

---

## Task 9: Update `useLibrary` Hook

**Files:**
- Modify: `src/hooks/useLibrary.ts`

- [ ] **Step 1: Replace the hook with query-based version**

Replace the entire file content with:

```typescript
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LibraryEntry, LibraryTag, LibraryQuery, SeriesDetail } from '../types';

export function useLibrary() {
  const [entries, setEntries] = useState<LibraryEntry[]>([]);
  const [tags, setTags] = useState<LibraryTag[]>([]);
  const [loading, setLoading] = useState(false);
  const [detail, setDetail] = useState<SeriesDetail | null>(null);
  const [query, setQuery] = useState<LibraryQuery>({});

  const refresh = useCallback(async (q?: LibraryQuery) => {
    setLoading(true);
    try {
      const result = await invoke<LibraryEntry[]>('cmd_get_library', { query: q ?? query });
      setEntries(result);
    } catch (e) {
      console.error('Failed to load library:', e);
    } finally {
      setLoading(false);
    }
  }, [query]);

  const loadTags = useCallback(async () => {
    try {
      const result = await invoke<LibraryTag[]>('cmd_get_tags');
      setTags(result);
    } catch (e) {
      console.error('Failed to load tags:', e);
    }
  }, []);

  const updateQuery = useCallback((updates: Partial<LibraryQuery>) => {
    setQuery(prev => {
      const next = { ...prev, ...updates };
      refresh(next);
      return next;
    });
  }, [refresh]);

  useEffect(() => { refresh(); loadTags(); }, [refresh, loadTags]);

  const loadDetail = useCallback(async (id: number) => {
    try {
      const result = await invoke<SeriesDetail>('cmd_get_series_detail', { libraryId: id });
      setDetail(result);
    } catch (e) {
      console.error('Load detail failed:', e);
    }
  }, []);

  const remove = useCallback(async (id: number) => {
    try {
      await invoke('cmd_remove_from_library', { libraryId: id });
      setEntries(prev => prev.filter(e => e.id !== id));
      if (detail?.entry.id === id) setDetail(null);
    } catch (e) {
      console.error('Remove failed:', e);
    }
  }, [detail]);

  const toggleFavorite = useCallback(async (id: number) => {
    try {
      const newState = await invoke<boolean>('cmd_toggle_favorite', { libraryId: id });
      setEntries(prev => prev.map(e => e.id === id ? { ...e, favorite: newState } : e));
      if (detail?.entry.id === id) {
        setDetail(d => d ? { ...d, entry: { ...d.entry, favorite: newState } } : null);
      }
    } catch (e) {
      console.error('Toggle favorite failed:', e);
    }
  }, [detail]);

  const createTag = useCallback(async (name: string) => {
    const id = await invoke<number>('cmd_create_tag', { name });
    await loadTags();
    return id;
  }, [loadTags]);

  const deleteTag = useCallback(async (tagId: number) => {
    await invoke('cmd_delete_tag', { tagId });
    await loadTags();
    await refresh();
  }, [loadTags, refresh]);

  const assignTag = useCallback(async (libraryId: number, tagId: number) => {
    await invoke('cmd_assign_tag', { libraryId, tagId });
    await refresh();
    if (detail?.entry.id === libraryId) await loadDetail(libraryId);
  }, [refresh, detail, loadDetail]);

  const unassignTag = useCallback(async (libraryId: number, tagId: number) => {
    await invoke('cmd_unassign_tag', { libraryId, tagId });
    await refresh();
    if (detail?.entry.id === libraryId) await loadDetail(libraryId);
  }, [refresh, detail, loadDetail]);

  const openEpisode = useCallback(async (libraryId: number, episodeNumber: number) => {
    try {
      await invoke('cmd_open_episode', { libraryId, episodeNumber });
    } catch (e) {
      console.error('Open episode failed:', e);
    }
  }, []);

  const closeDetail = useCallback(() => setDetail(null), []);

  const search = useCallback((q: string) => {
    updateQuery({ search: q || undefined });
  }, [updateQuery]);

  return {
    entries, tags, loading, detail, query,
    refresh, search, updateQuery, loadTags,
    loadDetail, remove, closeDetail,
    toggleFavorite, createTag, deleteTag, assignTag, unassignTag, openEpisode,
  };
}
```

- [ ] **Step 2: Verify with `npx tsc --noEmit`**

- [ ] **Step 3: Commit**

```bash
git add src/hooks/useLibrary.ts
git commit -m "feat: update useLibrary hook with query, tag, favorite, episode methods"
```

---

## Task 10: LibraryCard — Favorite Star + Tag Badges

**Files:**
- Modify: `src/components/LibraryCard.tsx`

- [ ] **Step 1: Add favorite star overlay and tag badges**

Update the component to add:
- Star icon in top-right corner of poster (filled gold if favorited, outline if not)
- Click star calls `onToggleFavorite` with `e.stopPropagation()`
- Tag badges below title (max 2 visible, "+N more" overflow)

Add `onToggleFavorite` to props interface:
```typescript
interface LibraryCardProps {
  entry: LibraryEntry;
  onClick: (id: number) => void;
  onRemove: (id: number) => void;
  onToggleFavorite: (id: number) => void;
}
```

Add star icon import: `import { Play, Trash2, Star } from 'lucide-react';`

In the poster `div`, after the status badge, add:
```tsx
{/* Favorite star */}
<button
  onClick={(e) => { e.stopPropagation(); onToggleFavorite(entry.id); }}
  className="absolute bottom-2 right-2 p-1 rounded-full bg-black/40 hover:bg-black/60 transition-all"
>
  <Star
    size={16}
    className={entry.favorite ? 'text-yellow-400 fill-yellow-400' : 'text-white/60'}
  />
</button>
```

After the title, add tag badges:
```tsx
{entry.tags.length > 0 && (
  <div className="flex flex-wrap gap-1 mt-1">
    {entry.tags.slice(0, 2).map(tag => (
      <span key={tag.id} className="text-[10px] px-1.5 py-0.5 rounded bg-[var(--accent)]/10 text-[var(--accent)] truncate max-w-[80px]">
        {tag.name}
      </span>
    ))}
    {entry.tags.length > 2 && (
      <span className="text-[10px] text-[var(--text)] opacity-40">+{entry.tags.length - 2}</span>
    )}
  </div>
)}
```

- [ ] **Step 2: Verify with `npx tsc --noEmit`**

- [ ] **Step 3: Commit**

```bash
git add src/components/LibraryCard.tsx
git commit -m "feat: add favorite star and tag badges to LibraryCard"
```

---

## Task 11: LibraryPanel — Toolbar + Tag Filter Chips

**Files:**
- Modify: `src/components/LibraryPanel.tsx`

- [ ] **Step 1: Add toolbar row with sort, order, favorite, status controls**

Between the search bar and the library grid, add a toolbar:

```tsx
{/* Toolbar */}
<div className="flex items-center gap-2 flex-wrap">
  {/* Sort */}
  <select
    value={query.sort || 'date_added'}
    onChange={(e) => updateQuery({ sort: e.target.value })}
    className="px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-xs"
  >
    <option value="date_added">Date Added</option>
    <option value="title">Title</option>
    <option value="progress">Progress</option>
    <option value="source">Source</option>
    <option value="last_downloaded">Last Downloaded</option>
  </select>

  {/* Order toggle */}
  <button
    onClick={() => updateQuery({ order: query.order === 'asc' ? 'desc' : 'asc' })}
    className="px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-xs"
    title={query.order === 'asc' ? 'Ascending' : 'Descending'}
  >
    {query.order === 'asc' ? '↑' : '↓'}
  </button>

  {/* Favorite filter */}
  <button
    onClick={() => updateQuery({ favoriteOnly: query.favoriteOnly ? undefined : true })}
    className={`px-2 py-1 rounded text-xs border ${query.favoriteOnly ? 'bg-yellow-500/20 border-yellow-500/40 text-yellow-400' : 'bg-[var(--card)] border-[var(--border)] text-[var(--text)]'}`}
  >
    ★ Favorites
  </button>

  {/* Status filter */}
  <select
    value={query.status || 'all'}
    onChange={(e) => updateQuery({ status: e.target.value === 'all' ? undefined : e.target.value })}
    className="px-2 py-1 rounded bg-[var(--card)] border border-[var(--border)] text-[var(--text)] text-xs"
  >
    <option value="all">All status</option>
    <option value="complete">Complete</option>
    <option value="in_progress">In Progress</option>
    <option value="not_started">Not Started</option>
  </select>
</div>
```

- [ ] **Step 2: Add tag filter chips row**

Below toolbar, add horizontal scrollable tag chips (same pattern as BrowsePanel source filters):

```tsx
{/* Tag filter chips */}
{tags.length > 0 && (
  <div className="flex gap-2 overflow-x-auto pb-1">
    <button
      onClick={() => updateQuery({ tagId: undefined })}
      className={`px-3 py-1 rounded-full text-xs whitespace-nowrap ${!query.tagId ? 'bg-[var(--accent)] text-white' : 'bg-[var(--card)] text-[var(--text)] border border-[var(--border)]'}`}
    >
      All
    </button>
    {tags.map(tag => (
      <button
        key={tag.id}
        onClick={() => updateQuery({ tagId: query.tagId === tag.id ? undefined : tag.id })}
        className={`px-3 py-1 rounded-full text-xs whitespace-nowrap ${query.tagId === tag.id ? 'bg-[var(--accent)] text-white' : 'bg-[var(--card)] text-[var(--text)] border border-[var(--border)]'}`}
      >
        {tag.name} ({tag.usageCount})
      </button>
    ))}
    <button
      onClick={() => {
        const name = prompt('Tag name:');
        if (name?.trim()) createTag(name.trim());
      }}
      className="px-3 py-1 rounded-full text-xs whitespace-nowrap bg-[var(--card)] text-[var(--accent)] border border-dashed border-[var(--accent)]/40"
    >
      + New Tag
    </button>
  </div>
)}
```

- [ ] **Step 3: Update hook destructuring and remove client-side filtering**

Replace the hook destructuring to include new methods:
```typescript
const { entries, tags, loading, detail, query, search, updateQuery, loadDetail, remove, closeDetail, toggleFavorite, createTag } = useLibrary();
```

Remove `filterSource` state and the client-side `filtered` variable (lines 11, 27-29). The source filter dropdown (lines 60-69) should call `updateQuery({ source: e.target.value === 'all' ? undefined : e.target.value })` instead of `setFilterSource`. Use `entries` directly instead of `filtered` in the grid.

Pass `onToggleFavorite={toggleFavorite}` to each `LibraryCard`.

- [ ] **Step 4: Verify with `npx tsc --noEmit`**

- [ ] **Step 5: Commit**

```bash
git add src/components/LibraryPanel.tsx
git commit -m "feat: add toolbar, tag chips, sort/filter controls to LibraryPanel"
```

---

## Task 12: SeriesDetail — Tag Management + Episode Actions

**Files:**
- Modify: `src/components/SeriesDetail.tsx`

- [ ] **Step 1: Add tag management to meta row**

Update props to accept tag methods:
```typescript
interface SeriesDetailProps {
  detail: SeriesDetailType;
  onBack: () => void;
  onRemove: (id: number) => void;
  onRefetch: (id: number) => void;
  onToggleFavorite: (id: number) => void;
  tags: LibraryTag[];
  onAssignTag: (libraryId: number, tagId: number) => void;
  onUnassignTag: (libraryId: number, tagId: number) => void;
  onCreateTag: (name: string) => Promise<number>;
  onOpenEpisode: (libraryId: number, episodeNumber: number) => void;
}
```

Add tag chips in the meta info section:
```tsx
{/* Tags */}
<div className="flex items-center gap-2 flex-wrap mt-2">
  {entry.tags.map(tag => (
    <span key={tag.id} className="flex items-center gap-1 px-2 py-0.5 rounded-full text-xs bg-[var(--accent)]/10 text-[var(--accent)]">
      {tag.name}
      <button onClick={() => onUnassignTag(entry.id, tag.id)} className="hover:text-red-400">×</button>
    </span>
  ))}
  <select
    onChange={(e) => {
      if (e.target.value) { onAssignTag(entry.id, Number(e.target.value)); e.target.value = ''; }
    }}
    className="text-xs bg-transparent text-[var(--text)] opacity-60"
    defaultValue=""
  >
    <option value="" disabled>+ Add tag</option>
    {tags.filter(t => !entry.tags.some(et => et.id === t.id)).map(tag => (
      <option key={tag.id} value={tag.id}>{tag.name}</option>
    ))}
  </select>
</div>
```

- [ ] **Step 2: Add favorite toggle in header**

Add a star button next to the title:
```tsx
<button
  onClick={() => onToggleFavorite(entry.id)}
  className="p-1 rounded hover:bg-[var(--card)] transition-colors"
  title={entry.favorite ? 'Remove from favorites' : 'Add to favorites'}
>
  <Star size={18} className={entry.favorite ? 'text-yellow-400 fill-yellow-400' : 'text-[var(--text)] opacity-40'} />
</button>
```

- [ ] **Step 3: Make episode boxes interactive**

Replace the static episode box with clickable version:

```tsx
{episodes.map((ep) => {
  const isClickable = ep.status === 'completed' || ep.status === 'failed' || ep.status === 'pending';
  const handleClick = () => {
    if (ep.status === 'completed' && ep.filePath) {
      onOpenEpisode(entry.id, ep.episodeNumber);
    }
    // failed/pending: could trigger re-download in future
  };

  return (
    <div
      key={ep.id}
      onClick={isClickable ? handleClick : undefined}
      className={`p-3 rounded-lg border text-center text-sm transition-all ${getStatusColor(ep.status)} ${isClickable ? 'cursor-pointer hover:scale-105 hover:shadow-md' : ''}`}
    >
      <div className="font-medium">Ep {ep.episodeNumber}</div>
      <div className="text-xs mt-1 capitalize">{ep.status}</div>
      {ep.status === 'completed' && (
        <Play size={12} className="mx-auto mt-1 opacity-40" />
      )}
    </div>
  );
})}
```

- [ ] **Step 4: Add `lastDownloaded` display**

In the meta row, add:
```tsx
{entry.lastDownloaded && (
  <span>Last download: {new Date(entry.lastDownloaded).toLocaleDateString()}</span>
)}
```

- [ ] **Step 5: Verify with `npx tsc --noEmit`**

- [ ] **Step 6: Commit**

```bash
git add src/components/SeriesDetail.tsx
git commit -m "feat: add tag management, episode actions, favorite toggle to SeriesDetail"
```

---

## Task 13: Integration Build Check

- [ ] **Step 1: Run full Rust build**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 2: Run full TypeScript check**

```bash
cd /Volumes/Data/workspace/git/video-downloader-tauri && npx tsc --noEmit
```

- [ ] **Step 3: Run frontend dev build**

```bash
npm run build
```

- [ ] **Step 4: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix: integration fixes for library organization feature"
```
