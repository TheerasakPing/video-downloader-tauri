# Phase 2: Library Organization — Design Specification

**Goal:** Add tags, favorites, configurable sorting/filtering, and episode-level actions to the content library so users can organize and manage their downloaded content.

**Architecture:** Extend the existing SQLite database with migration system, add `favorite` column and `library_tags`/`library_tag_map` tables, modify backend queries to accept sort/filter params, enhance LibraryPanel UI with toolbar and tag chips, make SeriesDetail episodes interactive.

**Sites covered:** All — library features are source-agnostic.

---

## 1. Database Schema Changes

### 1.1 Migration System

The existing `schema_version` table (created in `init_schema()`) already tracks the current schema version. Extend the existing migration approach: on `LibraryDb::new()`, check the current version in `schema_version` and apply pending migrations in order. Each migration is a SQL string executed in a transaction. Migrations must be **idempotent** — safe to re-run if the app crashes mid-migration.

### 1.2 Migration 2: Favorites + Tags

```sql
-- Idempotent: only add column if it doesn't exist
-- Use a conditional check since SQLite lacks IF NOT EXISTS for ALTER TABLE
-- Implementation: SELECT COUNT(*) FROM pragma_table_info('library') WHERE name='favorite'
-- If count is 0, run: ALTER TABLE library ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0

-- Tags table (IF NOT EXISTS is safe for CREATE TABLE)
CREATE TABLE IF NOT EXISTS library_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);

-- Many-to-many mapping
CREATE TABLE IF NOT EXISTS library_tag_map (
    library_id INTEGER NOT NULL REFERENCES library(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES library_tags(id) ON DELETE CASCADE,
    PRIMARY KEY (library_id, tag_id)
);
```

**Why a column for favorite but a table for tags:** Favorite is a simple boolean per series — a column is the simplest representation. Tags are many-to-many (a series can have multiple tags, a tag can be on multiple series) — a join table is the correct relational model.

**Foreign key enforcement:** SQLite foreign keys are disabled by default. Modify the existing WAL pragma line in `LibraryDb::new()` (currently at `library.rs:58`) to include foreign key enforcement:

```rust
// BEFORE (library.rs:58):
conn.execute_batch("PRAGMA journal_mode=WAL;")

// AFTER:
conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
```

The existing `ON DELETE CASCADE` on `library_episodes` will also become active, which is correct behavior — deleting a library entry should cascade to episodes and tag mappings. The explicit episode deletion in `remove_series()` becomes redundant but harmless (acts as a safety net).

**Migration implementation pattern:** Each migration is a method that checks prerequisites and applies changes idempotently:

```rust
fn run_migrations(conn: &Connection) -> Result<(), String> {
    let current_version: i32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_version", [],
        |row| row.get(0),
    ).unwrap_or(0);

    if current_version < 2 {
        migration_2_add_favorites_and_tags(conn)?;
        conn.execute("INSERT OR IGNORE INTO schema_version (version) VALUES (2)", [])
            .map_err(|e| e.to_string())?;
    }
    // future migrations: if current_version < 3 { ... }
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
        "CREATE TABLE IF NOT EXISTS library_tags (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL UNIQUE);
         CREATE TABLE IF NOT EXISTS library_tag_map (library_id INTEGER NOT NULL REFERENCES library(id) ON DELETE CASCADE, tag_id INTEGER NOT NULL REFERENCES library_tags(id) ON DELETE CASCADE, PRIMARY KEY (library_id, tag_id));"
    ).map_err(|e| format!("Migration 2 (tag tables) failed: {}", e))?;

    Ok(())
}
```

---

## 2. Backend Rust Changes

### 2.1 New/Modified Rust Structs

**LibraryTag** — new struct for tag representation:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryTag {
    pub id: i64,
    pub name: String,
    pub usage_count: i32,
}
```

**LibraryEntry** — modified to include `favorite` and `tags`:

```rust
// Add these fields to the existing LibraryEntry struct in library.rs:
pub favorite: bool,               // 0 or 1 from DB
pub tags: Vec<LibraryTag>,        // populated via separate query after main SELECT
```

**Tags loading approach:** Tags are loaded via a **separate query** (not JOIN) for clarity. After fetching library entries, run one additional query to fetch all tags for the returned entry IDs:

```sql
SELECT tmap.library_id, t.id, t.name, COUNT(*) OVER (PARTITION BY t.id) as usage_count
FROM library_tag_map tmap
JOIN library_tags t ON t.id = tmap.tag_id
WHERE tmap.library_id IN (?, ?, ...)  -- entry IDs from main query
```

This avoids N+1 queries while keeping the main query simple. Tags are grouped by `library_id` in Rust and attached to each `LibraryEntry`.

**LibraryQuery** — replaces hardcoded `ORDER BY date_added DESC`:

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryQuery {
    pub sort: Option<String>,        // "date_added" | "title" | "progress" | "source" | "last_downloaded"
    pub order: Option<String>,       // "asc" | "desc"
    pub source: Option<String>,      // filter by source (rongyok, baanjeen, etc.)
    pub status: Option<String>,      // "all" | "complete" | "in_progress" | "not_started"
    pub tag_id: Option<i64>,         // filter by tag
    pub favorite_only: Option<bool>, // only favorites
    pub search: Option<String>,      // text search on title (case-insensitive)
}
```

Note: `Default` derive enables `LibraryQuery::default()` for the deprecation wrapper on `cmd_search_library` and for constructing queries incrementally in the frontend.

### 2.2 New Methods on LibraryDb

| Method | Purpose |
|--------|---------|
| `run_migrations()` | Apply pending schema migrations |
| `get_library(query: Option<LibraryQuery>)` | Return filtered/sorted library entries with tags and completed count. `None` = default sort |
| `get_tags()` | Return all tags with usage count. SQL: `SELECT t.id, t.name, COUNT(tmap.library_id) as usage_count FROM library_tags t LEFT JOIN library_tag_map tmap ON tmap.tag_id = t.id GROUP BY t.id ORDER BY t.name` |
| `get_tags_for_entries(ids: &[i64])` | Private helper — load tags for specific library entries (called internally by `get_library`). See Section 2.1 for SQL. |
| `create_tag(name: &str)` | Create a new tag |
| `delete_tag(tag_id: i64)` | Delete tag and all its mappings |
| `assign_tag(library_id: i64, tag_id: i64)` | Add tag to series |
| `unassign_tag(library_id: i64, tag_id: i64)` | Remove tag from series |
| `toggle_favorite(library_id: i64)` | Flip favorite boolean |
| `get_episode_file_path(library_id: i64, episode_number: i32)` | Get file path for opening |

**Note on `remove_series()`:** With `PRAGMA foreign_keys = ON` enabled (Section 1.2), `ON DELETE CASCADE` on `library_tag_map.library_id` will automatically remove tag mappings when a library entry is deleted. The existing explicit episode deletion in `remove_series()` can remain as a safety net — the CASCADE will be a no-op for already-deleted rows.

### 2.3 Fix `update_episode_status()`

After setting episode status, also update `last_downloaded` on the parent library row when the episode status is `'completed'`:

```sql
UPDATE library SET last_downloaded = datetime('now') WHERE id = ?
```

### 2.4 Query Building

`get_library(query: Option<LibraryQuery>)` builds SQL dynamically. When `query` is `None`, returns all entries sorted by `date_added DESC` (backward-compatible with existing behavior).

**Sort column mapping** (whitelist — unknown values fall back to `date_added`):

| Query `sort` value | SQL ORDER BY |
|---------------------|--------------|
| `"date_added"` | `l.date_added` (default) |
| `"title"` | `LOWER(l.title)` |
| `"source"` | `l.source` |
| `"last_downloaded"` | `l.last_downloaded DESC NULLS LAST` |
| `"progress"` | `completed_count * 1.0 / l.total_episodes DESC` (computed, not a column) |

**Note on `progress` sort:** Since `completed_count` is computed via `COUNT(CASE WHEN e.status = 'completed' ...)` in the existing query, the ORDER BY clause references this alias directly. The division handles the "what % complete" semantics.

**Filter clauses** (applied to the base query):

The base query uses a CTE to compute `completed_count` once, then filters on it. **Tags are NOT joined in this query** — they are loaded separately (see Section 2.1 tags loading approach).

```sql
WITH entry_counts AS (
    SELECT l.*, COUNT(CASE WHEN e.status = 'completed' THEN 1 END) as completed_count
    FROM library l
    LEFT JOIN library_episodes e ON e.library_id = l.id
    GROUP BY l.id
)
SELECT ec.*
FROM entry_counts ec
WHERE 1=1
-- dynamic filters appended here:
-- source:      AND ec.source = ?
-- favorite:    AND ec.favorite = 1
-- search:      AND LOWER(ec.title) LIKE LOWER(? || '%')
-- status:      AND [status condition on ec.completed_count / ec.total_episodes]
-- tag:         AND ec.id IN (SELECT library_id FROM library_tag_map WHERE tag_id = ?)
ORDER BY [sort column] [ASC|DESC]
```

**Tag filter:** Uses a subquery `AND ec.id IN (SELECT library_id FROM library_tag_map WHERE tag_id = ?)` instead of JOIN — this avoids row duplication and keeps the query clean. Filtering by tag means "show only items that HAVE this tag". Untagged items are excluded. After fetching entries, load their tags via the separate query documented in Section 2.1. Filtered entries still show ALL their tags (not just the filtered tag) in the UI.

**Status filter conditions:**
- `"complete"`: `completed_count = total_episodes`
- `"in_progress"`: `completed_count > 0 AND completed_count < total_episodes`
- `"not_started"`: `completed_count = 0`

**Search:** Backend wraps the pattern (`format!("%{}%", query)`), frontend passes the raw search string. Note: this changes existing behavior from case-sensitive to case-insensitive — an intentional UX improvement.

**Dynamic SQL building:** Use rusqlite's parameterized queries with string interpolation only for known-safe fragments (column names from whitelist, WHERE clause structure). User input (search text, source name) is always passed as `?` parameters to prevent SQL injection.

### 2.5 Tauri Commands

Modify existing `cmd_get_library` to accept an optional query parameter (backward-compatible — `None` returns all entries with default sort). Tauri's command system handles `Option<T>` parameters natively: if the frontend doesn't pass the argument, it arrives as `None`.

**Deprecate** `cmd_search_library` since `cmd_get_library(query: { search: "..." })` subsumes it. Keep `cmd_search_library` as a thin wrapper during the transition:

```rust
#[tauri::command]
fn cmd_search_library(state: State<'_, AppState>, query: String) -> Result<Vec<LibraryEntry>, String> {
    // Deprecated: use cmd_get_library(Some(LibraryQuery { search: Some(query), ..Default::default() }))
    state.library_db.get_library(Some(LibraryQuery { search: Some(query), ..Default::default() }))
}
```

The backend `search_library()` method in `library.rs` is also deprecated — `get_library(Some(query))` replaces it. Mark both the method and the command with `// DEPRECATED` comments for future removal.

| Command | Parameters | Returns |
|---------|-----------|---------|
| `cmd_get_library` | `query: Option<LibraryQuery>` | `Vec<LibraryEntry>` (modified — optional query, backward-compatible) |
| `cmd_get_tags` | — | `Vec<LibraryTag>` |
| `cmd_create_tag` | `name: String` | `i64` (tag id) |
| `cmd_delete_tag` | `tagId: i64` | `()` |
| `cmd_assign_tag` | `libraryId: i64, tagId: i64` | `()` |
| `cmd_unassign_tag` | `libraryId: i64, tagId: i64` | `()` |
| `cmd_toggle_favorite` | `libraryId: i64` | `bool` (new favorite state) |
| `cmd_open_episode` | `libraryId: i64, episodeNumber: i32` | `()` (opens file with system player) |

---

## 3. Frontend Types

### 3.1 New TypeScript Types

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

### 3.2 Modified Types

- `LibraryEntry`: Add `favorite: boolean`, `tags: LibraryTag[]` (mirrors backend Rust struct changes in Section 2.1)
- `LibraryQuery` replaces the current separate search string parameter (used by `cmd_get_library`)

---

## 4. Frontend UI

### 4.1 LibraryPanel Toolbar

Add a toolbar row above the card grid (below the existing search bar):

- **Sort dropdown**: `<select>` with options: Date Added (default), Title, Progress, Source, Last Downloaded
- **Order toggle button**: Ascending/Descending icon toggle
- **Favorite filter**: Star icon toggle — when active, only show favorited items
- **Status filter dropdown**: All / Complete / In Progress / Not Started

### 4.2 Tag Filter Section

Below the toolbar, a horizontal scrollable row of tag chips (same pattern as BrowsePanel's source filters):
- "All" chip (no tag filter) + one chip per tag showing tag name and count
- Active tag chip is highlighted
- "+ New Tag" button at the end that creates a tag via inline input

### 4.3 LibraryCard Favorite Star

- Star icon in top-right corner of the card (overlapping the poster)
- Click toggles favorite state (calls `cmd_toggle_favorite`)
- Favorited cards show a filled gold star; unfavorited show an empty outline star
- No card click conflict — star click stops propagation

### 4.4 LibraryCard Tag Badges

- Small tag badges below the title (max 2 visible, "+N more" if more)
- Tags use muted colors to avoid visual noise

### 4.5 Tag Management in SeriesDetail

- In the detail view meta row, show assigned tags as removable chips
- "Add tag" button opens a dropdown of existing tags (or creates new inline)
- Remove tag by clicking X on the chip

### 4.6 Episode Actions in SeriesDetail

Episode boxes become interactive based on status:
- **Completed** (`completed`): Click → invoke `cmd_open_episode` to open file in system player. Show a play icon overlay on hover.
- **Failed** (`failed`): Click → retry download for that single episode. Show a retry icon overlay on hover.
- **Downloading** (`downloading`): Show animated progress indicator. Not clickable.
- **Pending** (`pending`): Click → start download for that single episode. Show a download icon overlay on hover.

Also display `lastDownloaded` date in the detail meta row (if available).

### 4.7 Batch Tag Assignment (Future Consideration)

Not in initial scope but the schema supports it. Mention as out-of-scope to avoid over-building.

---

## 5. Data Flow

```
User opens Library tab → cmd_get_library(defaultQuery) → shows all entries sorted by date

User clicks sort dropdown → rebuild LibraryQuery → cmd_get_library(query) → refresh grid

User clicks tag chip → set tagId in query → cmd_get_library(query) → filtered grid

User clicks favorite star on card → cmd_toggle_favorite(id) → update local state

User clicks episode (completed) → cmd_open_episode(id, ep) → opens file with system player

User clicks episode (failed) → retry single episode download → update status on complete
```

---

## 6. Error Handling

- **Invalid sort column**: Whitelist on backend — ignore unknown sort values, fall back to `date_added`
- **Tag name collision**: `UNIQUE` constraint on `library_tags.name` — return error "Tag already exists"
- **Tag name empty**: `create_tag` trims whitespace and rejects empty strings — return error "Tag name cannot be empty"
- **Tag not found**: Return error from `assign_tag`/`unassign_tag`
- **Episode file missing**: `cmd_open_episode` returns error if `file_path` is NULL or file doesn't exist on disk
- **Migration failure**: Log error, app continues with current schema (migrations are additive only)

**Canonical episode status values:** `pending` (default), `downloading`, `completed`, `failed`. These are the only valid values for `library_episodes.status`. The frontend must use these exact strings when calling `cmd_update_episode_status`.

---

## 7. File Changes Summary

### Backend (Rust)
| File | Change |
|------|--------|
| `src-tauri/src/library.rs` | Add `LibraryTag`, `LibraryQuery` structs; add `favorite`/`tags` fields to `LibraryEntry`; add migration runner, tag CRUD, modified `get_library(Option<LibraryQuery>)`, private `get_tags_for_entries()`, fix `update_episode_status`; deprecate `search_library()` method |
| `src-tauri/src/lib.rs` | Modify `cmd_get_library` to accept `Option<LibraryQuery>`; add 7 new Tauri commands for tags, favorites, episode open |

### Frontend (React/TypeScript)
| File | Change |
|------|--------|
| `src/types/index.ts` | Add LibraryTag, LibraryQuery types; extend LibraryEntry |
| `src/hooks/useLibrary.ts` | Update to use LibraryQuery params, add tag/favorite methods |
| `src/components/LibraryPanel.tsx` | Add toolbar (sort, filter, favorite toggle), tag filter chips |
| `src/components/LibraryCard.tsx` | Add favorite star, tag badges |
| `src/components/SeriesDetail.tsx` | Add tag management, episode click actions, last_downloaded display |

---

## 8. Out of Scope

- Batch multi-select operations
- Hierarchical folder collections
- Watch status tracking (watched/unwatched)
- Library statistics dashboard
- Drag-and-drop tag assignment
- Tag colors/customization
- Import/export of tag configuration
