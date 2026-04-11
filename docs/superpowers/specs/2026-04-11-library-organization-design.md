# Phase 2: Library Organization — Design Specification

**Goal:** Add tags, favorites, configurable sorting/filtering, and episode-level actions to the content library so users can organize and manage their downloaded content.

**Architecture:** Extend the existing SQLite database with migration system, add `favorite` column and `library_tags`/`library_tag_map` tables, modify backend queries to accept sort/filter params, enhance LibraryPanel UI with toolbar and tag chips, make SeriesDetail episodes interactive.

**Sites covered:** All — library features are source-agnostic.

---

## 1. Database Schema Changes

### 1.1 Migration System

Add a `schema_migrations` table and a migration runner that applies pending migrations in order on app startup:

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY
);
```

The runner checks the max version in `schema_migrations` and applies any migration files with a higher version number. Each migration is a SQL string executed in a transaction.

### 1.2 Migration 2: Favorites + Tags

```sql
-- Add favorite column to library
ALTER TABLE library ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0;

-- Tags table
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

---

## 2. Backend Rust Changes

### 2.1 LibraryQuery Struct

Replace the hardcoded `ORDER BY date_added DESC` in `get_library()` with a parameterized query:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
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

### 2.2 New Methods on LibraryDb

| Method | Purpose |
|--------|---------|
| `run_migrations()` | Apply pending schema migrations |
| `get_library(query: LibraryQuery)` | Return filtered/sorted library entries with tags and completed count |
| `get_tags()` | Return all tags with usage count |
| `create_tag(name: &str)` | Create a new tag |
| `delete_tag(tag_id: i64)` | Delete tag and all its mappings |
| `assign_tag(library_id: i64, tag_id: i64)` | Add tag to series |
| `unassign_tag(library_id: i64, tag_id: i64)` | Remove tag from series |
| `toggle_favorite(library_id: i64)` | Flip favorite boolean |
| `get_episode_file_path(library_id: i64, episode_number: i32)` | Get file path for opening |

### 2.3 Fix `update_episode_status()`

After setting episode status, also update `last_downloaded` on the parent library row when the episode status is `'completed'`:

```sql
UPDATE library SET last_downloaded = datetime('now') WHERE id = ?
```

### 2.4 Query Building

`get_library()` builds SQL dynamically based on `LibraryQuery`:
- **Sort**: Whitelist of allowed sort columns, map to actual column names. Default: `date_added DESC`.
- **Source**: `WHERE source = ?` (existing behavior)
- **Status**: JOIN with episodes to compute completion status, filter by the computed value
- **Tag**: JOIN with `library_tag_map` WHERE `tag_id = ?`
- **Favorite**: `WHERE favorite = 1`
- **Search**: `WHERE title LIKE ?` (case-insensitive via `LOWER()`)

### 2.5 Tauri Commands

Add these new commands (modify existing `cmd_get_library` to accept `LibraryQuery`):

| Command | Parameters | Returns |
|---------|-----------|---------|
| `cmd_get_library` | `query: LibraryQuery` | `Vec<LibraryEntry>` (modified — now takes query params) |
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

- `LibraryEntry`: Add `favorite: boolean`, `tags: LibraryTag[]`
- `LibraryQuery` replaces the current separate search string parameter

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
- **Tag not found**: Return error from `assign_tag`/`unassign_tag`
- **Episode file missing**: `cmd_open_episode` returns error if `file_path` is NULL or file doesn't exist
- **Migration failure**: Log error, app continues with current schema (migrations are additive only)

---

## 7. File Changes Summary

### Backend (Rust)
| File | Change |
|------|--------|
| `src-tauri/src/library.rs` | Add migration runner, LibraryQuery struct, tag CRUD, modified get_library with query params, fix update_episode_status |
| `src-tauri/src/lib.rs` | Add/modify 8 Tauri commands for library query, tags, favorites, episode open |

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
