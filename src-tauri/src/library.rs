use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

// --- Data types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEntry {
    pub id: i64,
    pub parser_series_id: i32,
    pub title: String,
    pub source: String,
    pub source_url: Option<String>,
    pub poster_path: Option<String>,
    pub total_episodes: i32,
    pub date_added: String,
    pub last_downloaded: Option<String>,
    pub completed_count: i32,
    pub favorite: bool,
    pub tags: Vec<LibraryTag>,
    pub watched_count: Option<i32>,
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryEpisode {
    pub id: i64,
    pub library_id: i64,
    pub episode_number: i32,
    pub video_url: Option<String>,
    pub file_path: Option<String>,
    pub quality: Option<String>,
    pub file_size: Option<i64>,
    pub status: String,
    pub watched: bool,
    pub watched_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchProgress {
    pub library_id: i64,
    pub episode_number: i32,
    pub position_seconds: f64,
    pub duration_seconds: f64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesDetail {
    pub entry: LibraryEntry,
    pub episodes: Vec<LibraryEpisode>,
    pub can_refetch: bool,
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesMetadata {
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStats {
    pub total_series: i32,
    pub total_episodes: i32,
    pub completed_episodes: i32,
    pub total_size_bytes: i64,
    pub by_source: Vec<SourceStat>,
    pub by_status: StatusStat,
    pub by_month: Vec<MonthStat>,
    pub favorite_count: i32,
    pub tag_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceStat {
    pub source: String,
    pub series_count: i32,
    pub episode_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusStat {
    pub complete: i32,
    pub in_progress: i32,
    pub not_started: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthStat {
    pub month: String,
    pub count: i32,
}

// --- Database manager ---

pub struct LibraryDb {
    conn: Mutex<Connection>,
    poster_dir: PathBuf,
}

impl LibraryDb {
    pub fn new(app_data_dir: &std::path::Path) -> Result<Self, String> {
        let db_path = app_data_dir.join("library.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open library DB: {}", e))?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

        let poster_dir = app_data_dir.join("library_posters");
        std::fs::create_dir_all(&poster_dir).ok();

        let db = Self {
            conn: Mutex::new(conn),
            poster_dir,
        };
        db.init_schema()?;
        db.run_migrations()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY);
             CREATE TABLE IF NOT EXISTS library (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 parser_series_id INTEGER NOT NULL DEFAULT 0,
                 title TEXT NOT NULL,
                 source TEXT NOT NULL,
                 source_url TEXT NOT NULL DEFAULT '',
                 poster_path TEXT,
                 total_episodes INTEGER DEFAULT 1,
                 date_added TEXT NOT NULL,
                 last_downloaded TEXT,
                 metadata TEXT,
                 UNIQUE(source, source_url)
             );
             CREATE TABLE IF NOT EXISTS library_episodes (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 library_id INTEGER NOT NULL,
                 episode_number INTEGER NOT NULL,
                 video_url TEXT,
                 file_path TEXT,
                 quality TEXT,
                 file_size INTEGER,
                 status TEXT DEFAULT 'pending',
                 FOREIGN KEY (library_id) REFERENCES library(id) ON DELETE CASCADE,
                 UNIQUE(library_id, episode_number)
             );"
        ).map_err(|e| format!("Schema init failed: {}", e))?;

        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_library_title ON library(title);
             CREATE INDEX IF NOT EXISTS idx_library_date ON library(date_added);
             CREATE INDEX IF NOT EXISTS idx_library_episodes ON library_episodes(library_id);"
        ).map_err(|e| format!("Index creation failed: {}", e))?;

        conn.execute(
            "INSERT OR IGNORE INTO schema_version (version) VALUES (1)",
            [],
        ).map_err(|e| format!("Schema version insert failed: {}", e))?;

        Ok(())
    }

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

        if current_version < 3 {
            Self::migration_3_add_watch_progress(&conn)?;
            conn.execute("INSERT OR IGNORE INTO schema_version (version) VALUES (3)", [])
                .map_err(|e| format!("Migration version update failed: {}", e))?;
        }

        if current_version < 4 {
            Self::migration_4_add_metadata_columns(&conn)?;
            conn.execute("INSERT OR IGNORE INTO schema_version (version) VALUES (4)", [])
                .map_err(|e| format!("Migration version update failed: {}", e))?;
        }

        Ok(())
    }

    fn migration_2_add_favorites_and_tags(conn: &Connection) -> Result<(), String> {
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

    fn migration_3_add_watch_progress(conn: &Connection) -> Result<(), String> {
        let has_watched: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('library_episodes') WHERE name='watched'",
            [], |row| row.get(0),
        ).map_err(|e| e.to_string())?;

        if !has_watched {
            conn.execute("ALTER TABLE library_episodes ADD COLUMN watched INTEGER NOT NULL DEFAULT 0", [])
                .map_err(|e| format!("Migration 3 (watched column) failed: {}", e))?;
            conn.execute("ALTER TABLE library_episodes ADD COLUMN watched_at TEXT", [])
                .map_err(|e| format!("Migration 3 (watched_at column) failed: {}", e))?;
        }

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS watch_progress (
                library_id INTEGER NOT NULL REFERENCES library(id) ON DELETE CASCADE,
                episode_number INTEGER NOT NULL,
                position_seconds REAL NOT NULL DEFAULT 0,
                duration_seconds REAL NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (library_id, episode_number)
            );"
        ).map_err(|e| format!("Migration 3 (watch_progress table) failed: {}", e))?;

        Ok(())
    }

    fn migration_4_add_metadata_columns(conn: &Connection) -> Result<(), String> {
        let has_description: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('library') WHERE name='description'",
            [], |row| row.get(0),
        ).map_err(|e| e.to_string())?;

        if !has_description {
            conn.execute_batch(
                "ALTER TABLE library ADD COLUMN description TEXT;
                 ALTER TABLE library ADD COLUMN rating REAL;
                 ALTER TABLE library ADD COLUMN year INTEGER;
                 ALTER TABLE library ADD COLUMN genre TEXT;
                 ALTER TABLE library ADD COLUMN duration TEXT;"
            ).map_err(|e| format!("Migration 4 (metadata columns) failed: {}", e))?;
        }

        Ok(())
    }

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

    pub fn mark_episode_watched(&self, library_id: i64, episode_number: i32) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE library_episodes SET watched = 1, watched_at = ?1 WHERE library_id = ?2 AND episode_number = ?3",
            params![now, library_id, episode_number],
        ).map_err(|e| format!("Mark episode watched failed: {}", e))?;
        Ok(())
    }

    pub fn mark_episode_unwatched(&self, library_id: i64, episode_number: i32) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE library_episodes SET watched = 0, watched_at = NULL WHERE library_id = ?1 AND episode_number = ?2",
            params![library_id, episode_number],
        ).map_err(|e| format!("Mark episode unwatched failed: {}", e))?;
        Ok(())
    }

    pub fn update_watch_progress(&self, library_id: i64, episode_number: i32, position_seconds: f64, duration_seconds: f64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO watch_progress (library_id, episode_number, position_seconds, duration_seconds, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(library_id, episode_number) DO UPDATE SET
                position_seconds = excluded.position_seconds,
                duration_seconds = excluded.duration_seconds,
                updated_at = excluded.updated_at",
            params![library_id, episode_number, position_seconds, duration_seconds, now],
        ).map_err(|e| format!("Update watch progress failed: {}", e))?;
        Ok(())
    }

    pub fn get_watch_progress(&self, library_id: i64, episode_number: i32) -> Result<Option<WatchProgress>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let progress = conn.query_row(
            "SELECT library_id, episode_number, position_seconds, duration_seconds, updated_at
             FROM watch_progress WHERE library_id = ?1 AND episode_number = ?2",
            params![library_id, episode_number],
            |row| Ok(WatchProgress {
                library_id: row.get(0)?,
                episode_number: row.get(1)?,
                position_seconds: row.get(2)?,
                duration_seconds: row.get(3)?,
                updated_at: row.get(4)?,
            }),
        ).ok();
        Ok(progress)
    }

    pub fn get_episode_file_path(&self, library_id: i64, episode_number: i32) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let path: Option<String> = conn.query_row(
            "SELECT file_path FROM library_episodes WHERE library_id = ?1 AND episode_number = ?2",
            params![library_id, episode_number],
            |row| row.get(0),
        ).unwrap_or(None);

        if let Some(ref p) = path {
            if !std::path::Path::new(p).exists() {
                return Ok(None);
            }
        }
        Ok(path)
    }

    pub fn save_series(
        &self,
        title: &str,
        source: &str,
        source_url: Option<&str>,
        poster_data: Option<&[u8]>,
        total_episodes: i32,
        parser_series_id: i32,
        episode_urls: &std::collections::HashMap<i32, String>,
        metadata: Option<&str>,
        series_meta: Option<&SeriesMetadata>,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();

        let source_url_val = source_url.unwrap_or("");

        let (description, rating, year, genre, duration) = match series_meta {
            Some(m) => (
                m.description.as_deref(),
                m.rating,
                m.year,
                m.genre.as_deref(),
                m.duration.as_deref(),
            ),
            None => (None, None, None, None, None),
        };

        conn.execute(
            "INSERT INTO library (parser_series_id, title, source, source_url, total_episodes, date_added, metadata,
                                  description, rating, year, genre, duration)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(source, source_url) DO UPDATE SET
                title = excluded.title,
                total_episodes = excluded.total_episodes,
                metadata = excluded.metadata,
                description = excluded.description,
                rating = excluded.rating,
                year = excluded.year,
                genre = excluded.genre,
                duration = excluded.duration",
            params![parser_series_id, title, source, source_url_val, total_episodes, now, metadata,
                    description, rating, year, genre, duration],
        ).map_err(|e| format!("Save library failed: {}", e))?;

        let library_id = conn.last_insert_rowid();

        // Save poster if provided
        if let Some(data) = poster_data {
            let poster_path = self.poster_dir.join(format!("{}.jpg", library_id));
            std::fs::write(&poster_path, data).ok();
            conn.execute(
                "UPDATE library SET poster_path = ?1 WHERE id = ?2",
                params![poster_path.to_str(), library_id],
            ).ok();
        }

        // Upsert episodes
        for (ep_num, url) in episode_urls {
            conn.execute(
                "INSERT INTO library_episodes (library_id, episode_number, video_url)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(library_id, episode_number) DO UPDATE SET
                    video_url = excluded.video_url",
                params![library_id, ep_num, url],
            ).map_err(|e| format!("Save episode {} failed: {}", ep_num, e))?;
        }

        Ok(library_id)
    }

    pub fn update_episode_status(
        &self,
        library_id: i64,
        episode_number: i32,
        status: &str,
        file_path: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE library_episodes SET status = ?1, file_path = ?2 WHERE library_id = ?3 AND episode_number = ?4",
            params![status, file_path, library_id, episode_number],
        ).map_err(|e| format!("Update episode status failed: {}", e))?;

        if status == "completed" {
            conn.execute(
                "UPDATE library SET last_downloaded = datetime('now') WHERE id = ?1",
                params![library_id],
            ).map_err(|e| format!("Update last_downloaded failed: {}", e))?;
        }

        Ok(())
    }

    pub fn get_library(&self, query: Option<LibraryQuery>) -> Result<Vec<LibraryEntry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let q = query.unwrap_or_default();

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

        match q.status.as_deref() {
            Some("complete") => where_clauses.push("ec.completed_count = ec.total_episodes".to_string()),
            Some("in_progress") => where_clauses.push("ec.completed_count > 0 AND ec.completed_count < ec.total_episodes".to_string()),
            Some("not_started") => where_clauses.push("ec.completed_count = 0".to_string()),
            _ => {}
        }

        let where_sql = if where_clauses.is_empty() { String::new() } else { format!("WHERE {}", where_clauses.join(" AND ")) };

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
                SELECT l.*, COUNT(CASE WHEN e.status = 'completed' THEN 1 END) as completed_count,
                       COUNT(CASE WHEN e.watched = 1 THEN 1 END) as watched_count
                FROM library l
                LEFT JOIN library_episodes e ON e.library_id = l.id
                GROUP BY l.id
            )
            SELECT ec.id, ec.parser_series_id, ec.title, ec.source, ec.source_url,
                   ec.poster_path, ec.total_episodes, ec.date_added, ec.last_downloaded,
                   ec.completed_count, ec.favorite, ec.watched_count,
                   ec.description, ec.rating, ec.year, ec.genre, ec.duration
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
                watched_count: row.get(11)?,
                description: row.get(12)?,
                rating: row.get(13)?,
                year: row.get(14)?,
                genre: row.get(15)?,
                duration: row.get(16)?,
            })
        }).map_err(|e| format!("Map library entries failed: {}", e))?
        .filter_map(|e| e.ok())
        .collect();

        let ids: Vec<i64> = entries.iter().map(|e| e.id).collect();
        let tags_map = Self::get_tags_for_entries(&conn, &ids)?;

        let entries = entries.into_iter().map(|mut e| {
            e.tags = tags_map.get(&e.id).cloned().unwrap_or_default();
            e
        }).collect();

        Ok(entries)
    }

    pub fn get_series_detail(&self, library_id: i64) -> Result<SeriesDetail, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let entry: LibraryEntry = conn.query_row(
            "SELECT l.id, l.parser_series_id, l.title, l.source, l.source_url,
                    l.poster_path, l.total_episodes, l.date_added, l.last_downloaded,
                    COUNT(CASE WHEN e.status = 'completed' THEN 1 END),
                    l.favorite,
                    COUNT(CASE WHEN e.watched = 1 THEN 1 END) as watched_count,
                    l.description, l.rating, l.year, l.genre, l.duration
             FROM library l LEFT JOIN library_episodes e ON e.library_id = l.id
             WHERE l.id = ?1 GROUP BY l.id",
            params![library_id],
            |row| Ok(LibraryEntry {
                id: row.get(0)?, parser_series_id: row.get(1)?, title: row.get(2)?,
                source: row.get(3)?, source_url: row.get(4)?, poster_path: row.get(5)?,
                total_episodes: row.get(6)?, date_added: row.get(7)?, last_downloaded: row.get(8)?,
                completed_count: row.get(9)?,
                favorite: row.get::<_, i32>(10)? != 0,
                tags: Vec::new(),
                watched_count: row.get(11)?,
                description: row.get(12)?, rating: row.get(13)?, year: row.get(14)?,
                genre: row.get(15)?, duration: row.get(16)?,
            }),
        ).map_err(|e| format!("Series not found: {}", e))?;

        let tags_map = Self::get_tags_for_entries(&conn, &[library_id])?;
        let entry = LibraryEntry {
            tags: tags_map.get(&library_id).cloned().unwrap_or_default(),
            ..entry
        };

        let mut stmt = conn.prepare(
            "SELECT id, library_id, episode_number, video_url, file_path, quality, file_size, status, watched, watched_at
             FROM library_episodes WHERE library_id = ?1 ORDER BY episode_number"
        ).map_err(|e| e.to_string())?;

        let episodes: Vec<LibraryEpisode> = stmt.query_map(params![library_id], |row| {
            Ok(LibraryEpisode {
                id: row.get(0)?, library_id: row.get(1)?, episode_number: row.get(2)?,
                video_url: row.get(3)?, file_path: row.get(4)?, quality: row.get(5)?,
                file_size: row.get(6)?, status: row.get(7)?,
                watched: row.get::<_, i32>(8)? != 0,
                watched_at: row.get(9)?,
            })
        }).map_err(|e| e.to_string())?.filter_map(|e| e.ok()).collect();

        let source_url = entry.source_url.as_deref().unwrap_or("");
        let can_refetch = !source_url.contains(".m3u8") && !source_url.contains(".mp4") && !source_url.is_empty();

        Ok(SeriesDetail { entry, episodes, can_refetch })
    }

    pub fn remove_series(&self, library_id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let poster_path: Option<String> = conn.query_row(
            "SELECT poster_path FROM library WHERE id = ?1", params![library_id],
            |row| row.get(0),
        ).unwrap_or(None);

        if let Some(path) = poster_path {
            std::fs::remove_file(&path).ok();
        }

        // Delete episodes BEFORE the library entry (explicit, safer than CASCADE)
        conn.execute("DELETE FROM library_episodes WHERE library_id = ?1", params![library_id])
            .map_err(|e| format!("Delete episodes failed: {}", e))?;
        conn.execute("DELETE FROM library WHERE id = ?1", params![library_id])
            .map_err(|e| format!("Delete failed: {}", e))?;
        Ok(())
    }

    // DEPRECATED: Use get_library(Some(LibraryQuery { search: Some(query), ..Default::default() }))
    pub fn search_library(&self, query: &str) -> Result<Vec<LibraryEntry>, String> {
        self.get_library(Some(LibraryQuery { search: Some(query.to_string()), ..Default::default() }))
    }

    pub fn get_library_stats(&self) -> Result<LibraryStats, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // Total series
        let total_series: i32 = conn.query_row(
            "SELECT COUNT(*) FROM library",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        // Total and completed episodes
        let (total_episodes, completed_episodes): (i32, i32) = conn.query_row(
            "SELECT COUNT(*), SUM(CASE WHEN e.status = 'completed' THEN 1 ELSE 0 END)
             FROM library_episodes e",
            [],
            |row| Ok((row.get(0)?, row.get(1).unwrap_or(0))),
        ).unwrap_or((0, 0));

        // Total size bytes
        let total_size_bytes: i64 = conn.query_row(
            "SELECT COALESCE(SUM(file_size), 0) FROM library_episodes",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        // By source
        let mut by_source_stmt = conn.prepare(
            "SELECT l.source, COUNT(DISTINCT l.id), COUNT(e.id)
             FROM library l
             LEFT JOIN library_episodes e ON e.library_id = l.id
             GROUP BY l.source"
        ).map_err(|e| e.to_string())?;

        let by_source: Vec<SourceStat> = by_source_stmt.query_map([], |row| {
            Ok(SourceStat {
                source: row.get(0)?,
                series_count: row.get(1)?,
                episode_count: row.get(2)?,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

        // By status
        let (complete, in_progress, not_started): (i32, i32, i32) = conn.query_row(
            "SELECT
                SUM(CASE WHEN completed_count = total_episodes THEN 1 ELSE 0 END),
                SUM(CASE WHEN completed_count > 0 AND completed_count < total_episodes THEN 1 ELSE 0 END),
                SUM(CASE WHEN completed_count = 0 THEN 1 ELSE 0 END)
             FROM (
                 SELECT l.id,
                        l.total_episodes,
                        COUNT(CASE WHEN e.status = 'completed' THEN 1 END) as completed_count
                 FROM library l
                 LEFT JOIN library_episodes e ON e.library_id = l.id
                 GROUP BY l.id
             )",
            [],
            |row| Ok((row.get(0).unwrap_or(0), row.get(1).unwrap_or(0), row.get(2).unwrap_or(0))),
        ).unwrap_or((0, 0, 0));

        let by_status = StatusStat {
            complete,
            in_progress,
            not_started,
        };

        // By month (last 6 months)
        let mut by_month_stmt = conn.prepare(
            "SELECT strftime('%Y-%m', date_added) as month, COUNT(*) as count
             FROM library
             WHERE date_added >= datetime('now', '-6 months')
             GROUP BY month
             ORDER BY month DESC"
        ).map_err(|e| e.to_string())?;

        let by_month: Vec<MonthStat> = by_month_stmt.query_map([], |row| {
            Ok(MonthStat {
                month: row.get(0)?,
                count: row.get(1)?,
            })
        }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect::<Vec<_>>().into_iter().rev().collect();

        // Favorite count
        let favorite_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM library WHERE favorite = 1",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        // Tag count
        let tag_count: i32 = conn.query_row(
            "SELECT COUNT(*) FROM library_tags",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        Ok(LibraryStats {
            total_series,
            total_episodes,
            completed_episodes,
            total_size_bytes,
            by_source,
            by_status,
            by_month,
            favorite_count,
            tag_count,
        })
    }

    // Phase 5: Import & Export
    pub fn export_to_json(&self) -> Result<String, String> {
        let entries = self.get_library(None)?;
        let mut export_data = Vec::new();
        for entry in entries {
            let detail = self.get_series_detail(entry.id).ok();
            export_data.push(serde_json::json!({
                "entry": entry,
                "detail": detail,
            }));
        }
        serde_json::to_string_pretty(&export_data).map_err(|e| e.to_string())
    }

    pub fn import_from_json(&self, json_data: &str) -> Result<i32, String> {
        let import_data: Vec<serde_json::Value> = serde_json::from_str(json_data)
            .map_err(|e| format!("Invalid JSON: {}", e))?;
        let mut count = 0;
        for item in import_data {
            // Extract entry fields
            let title = item["entry"]["title"].as_str().unwrap_or("");
            let source = item["entry"]["source"].as_str().unwrap_or("");
            let source_url = item["entry"]["sourceUrl"].as_str();
            let total_episodes = item["entry"]["totalEpisodes"].as_i64().unwrap_or(1) as i32;
            let parser_series_id = item["entry"]["parserSeriesId"].as_i64().unwrap_or(0) as i32;

            let mut episode_urls = std::collections::HashMap::new();
            if let Some(detail) = item["detail"].as_object() {
                if let Some(episodes) = detail["episodes"].as_array() {
                    for ep in episodes {
                        if let (Some(num), Some(url)) = (ep["episodeNumber"].as_i64(), ep["videoUrl"].as_str()) {
                            episode_urls.insert(num as i32, url.to_string());
                        }
                    }
                }
            }

            if !title.is_empty() {
                self.save_series(title, source, source_url, None, total_episodes, parser_series_id, &episode_urls, None, None)?;
                count += 1;
            }
        }
        Ok(count)
    }

    /// Find duplicate series entries based on normalized title comparison
    /// Returns groups of entries with similar titles from different sources
    pub fn find_duplicates(&self) -> Result<Vec<(LibraryEntry, Vec<LibraryEntry>)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // Get all library entries with their source info
        let mut stmt = conn.prepare(
            "SELECT l.id, l.parser_series_id, l.title, l.source, l.source_url, l.poster_path,
                    l.total_episodes, l.date_added, l.last_downloaded,
                    COUNT(CASE WHEN e.status = 'completed' THEN 1 END) as completed_count,
                    l.favorite,
                    COUNT(CASE WHEN e.watched = 1 THEN 1 END) as watched_count,
                    l.description, l.rating, l.year, l.genre, l.duration
             FROM library l
             LEFT JOIN library_episodes e ON e.library_id = l.id
             GROUP BY l.id
             ORDER BY LOWER(REPLACE(REPLACE(l.title, ' ', ''), '-', ''))"
        ).map_err(|e| e.to_string())?;

        let entries: Vec<LibraryEntry> = stmt.query_map([], |row| {
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
                watched_count: row.get(11)?,
                description: row.get(12)?,
                rating: row.get(13)?,
                year: row.get(14)?,
                genre: row.get(15)?,
                duration: row.get(16)?,
            })
        }).map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .collect();

        // Get tags for all entries
        let ids: Vec<i64> = entries.iter().map(|e| e.id).collect();
        let tags_map = Self::get_tags_for_entries(&conn, &ids)?;

        let entries_with_tags: Vec<LibraryEntry> = entries.into_iter().map(|mut e| -> LibraryEntry {
            e.tags = tags_map.get(&e.id).cloned().unwrap_or_default();
            e
        }).collect();

        // Group by normalized title (remove spaces, hyphens, convert to lowercase)
        let mut groups: std::collections::HashMap<String, Vec<LibraryEntry>> = std::collections::HashMap::new();

        for entry in entries_with_tags {
            let normalized = entry.title
                .to_lowercase()
                .replace(' ', "")
                .replace('-', "")
                .replace('_', "");

            groups.entry(normalized).or_default().push(entry);
        }

        // Find groups with entries from different sources
        let duplicates: Vec<(LibraryEntry, Vec<LibraryEntry>)> = groups.into_values()
            .filter(|g| g.len() > 1)
            .filter_map(|mut group| {
                // Check if entries are from different sources
                let sources: std::collections::HashSet<&str> = group.iter()
                    .map(|e| e.source.as_str())
                    .collect();

                if sources.len() > 1 {
                    let primary = group.remove(0);
                    Some((primary, group))
                } else {
                    None
                }
            })
            .collect();

        Ok(duplicates)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    static TEST_DB_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    fn test_db() -> LibraryDb {
        // Use timestamp + atomic counter for guaranteed uniqueness across parallel tests
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEST_DB_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("vdt_test_library_{}_{}", ts, counter));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).ok();
        LibraryDb::new(&dir).expect("Failed to create test DB")
    }

    /// Helper: create a series with N episodes and return the library ID
    fn add_series(db: &LibraryDb, title: &str, source: &str, ep_count: i32) -> i64 {
        let mut episodes = HashMap::new();
        for i in 1..=ep_count {
            episodes.insert(i, format!("https://example.com/{}/ep{}.m3u8", source, i));
        }
        db.save_series(title, source, Some(&format!("https://{}/{}", source, title)), None, ep_count, 0, &episodes, None, None).unwrap()
    }

    // ─── Basic CRUD ───────────────────────────────

    #[test]
    fn test_save_and_get_series() {
        let db = test_db();
        let mut episodes = HashMap::new();
        episodes.insert(1, "https://example.com/ep1.m3u8".to_string());
        episodes.insert(2, "https://example.com/ep2.m3u8".to_string());

        let id = db.save_series("Test Series", "rongyok", Some("https://rongyok.com/123"), None, 2, 123, &episodes, None, None).unwrap();
        assert!(id > 0);

        let entries = db.get_library(None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Test Series");
        assert_eq!(entries[0].source, "rongyok");
    }

    #[test]
    fn test_save_series_upsert() {
        let db = test_db();
        let mut episodes = HashMap::new();
        episodes.insert(1, "https://example.com/ep1.m3u8".to_string());

        let id1 = db.save_series("Dup", "rongyok", Some("https://rongyok.com/dup"), None, 1, 0, &episodes, None, None).unwrap();
        let id2 = db.save_series("Dup Updated", "rongyok", Some("https://rongyok.com/dup"), None, 3, 0, &episodes, None, None).unwrap();

        // ON CONFLICT updates the existing row
        assert_eq!(id1, id2);
        let entries = db.get_library(None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Dup Updated");
        assert_eq!(entries[0].total_episodes, 3);
    }

    #[test]
    fn test_remove_series() {
        let db = test_db();
        let id = add_series(&db, "To Delete", "rongyok", 2);
        assert_eq!(db.get_library(None).unwrap().len(), 1);

        db.remove_series(id).unwrap();
        assert_eq!(db.get_library(None).unwrap().len(), 0);
    }

    #[test]
    fn test_remove_series_cascades_episodes() {
        let db = test_db();
        let id = add_series(&db, "Cascade Test", "rongyok", 3);
        let detail = db.get_series_detail(id).unwrap();
        assert_eq!(detail.episodes.len(), 3);

        db.remove_series(id).unwrap();
        // Episodes should be gone too
        let detail_result = db.get_series_detail(id);
        assert!(detail_result.is_err());
    }

    #[test]
    fn test_get_series_detail() {
        let db = test_db();
        let id = add_series(&db, "Detail Test", "rongyok", 2);
        let detail = db.get_series_detail(id).unwrap();

        assert_eq!(detail.entry.title, "Detail Test");
        assert_eq!(detail.episodes.len(), 2);
        assert_eq!(detail.episodes[0].episode_number, 1);
        assert_eq!(detail.episodes[1].episode_number, 2);
        // can_refetch depends on source_url not being a direct media URL
        assert!(detail.can_refetch);
    }

    #[test]
    fn test_series_detail_can_refetch_false_for_m3u8() {
        let db = test_db();
        let mut episodes = HashMap::new();
        episodes.insert(1, "https://example.com/ep1.m3u8".to_string());
        let id = db.save_series("Direct URL", "rongyok", Some("https://example.com/video.m3u8"), None, 1, 0, &episodes, None, None).unwrap();

        let detail = db.get_series_detail(id).unwrap();
        assert!(!detail.can_refetch);
    }

    // ─── Tag operations ───────────────────────────

    #[test]
    fn test_tags() {
        let db = test_db();
        let tag_id = db.create_tag("Action").unwrap();
        assert!(tag_id > 0);

        let tags = db.get_tags().unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "Action");
    }

    #[test]
    fn test_create_tag_empty_name_fails() {
        let db = test_db();
        let result = db.create_tag("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_create_tag_whitespace_only_fails() {
        let db = test_db();
        let result = db.create_tag("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_tag_duplicate_fails() {
        let db = test_db();
        db.create_tag("Drama").unwrap();
        let result = db.create_tag("Drama");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_create_tag_trims_whitespace() {
        let db = test_db();
        let id1 = db.create_tag("  Thriller  ").unwrap();
        assert!(id1 > 0);
        let tags = db.get_tags().unwrap();
        assert_eq!(tags[0].name, "Thriller");
    }

    #[test]
    fn test_delete_tag() {
        let db = test_db();
        let tag_id = db.create_tag("Comedy").unwrap();
        assert_eq!(db.get_tags().unwrap().len(), 1);

        db.delete_tag(tag_id).unwrap();
        assert_eq!(db.get_tags().unwrap().len(), 0);
    }

    #[test]
    fn test_assign_and_unassign_tag() {
        let db = test_db();
        let series_id = add_series(&db, "Tagged Series", "rongyok", 1);
        let tag_id = db.create_tag("Sci-Fi").unwrap();

        db.assign_tag(series_id, tag_id).unwrap();

        let detail = db.get_series_detail(series_id).unwrap();
        assert_eq!(detail.entry.tags.len(), 1);
        assert_eq!(detail.entry.tags[0].name, "Sci-Fi");

        db.unassign_tag(series_id, tag_id).unwrap();
        let detail = db.get_series_detail(series_id).unwrap();
        assert!(detail.entry.tags.is_empty());
    }

    #[test]
    fn test_tag_usage_count() {
        let db = test_db();
        let id1 = add_series(&db, "Series A", "rongyok", 1);
        let id2 = add_series(&db, "Series B", "titan", 1);
        let tag_id = db.create_tag("Popular").unwrap();

        db.assign_tag(id1, tag_id).unwrap();
        db.assign_tag(id2, tag_id).unwrap();

        let tags = db.get_tags().unwrap();
        assert_eq!(tags[0].usage_count, 2);
    }

    #[test]
    fn test_assign_tag_idempotent() {
        let db = test_db();
        let id = add_series(&db, "Idempotent", "rongyok", 1);
        let tag_id = db.create_tag("Tag1").unwrap();

        db.assign_tag(id, tag_id).unwrap();
        db.assign_tag(id, tag_id).unwrap(); // Should not fail (OR IGNORE)

        let detail = db.get_series_detail(id).unwrap();
        assert_eq!(detail.entry.tags.len(), 1);
    }

    // ─── Favorite operations ──────────────────────

    #[test]
    fn test_toggle_favorite() {
        let db = test_db();
        let mut episodes = HashMap::new();
        episodes.insert(1, "https://example.com/ep1.m3u8".to_string());
        let id = db.save_series("Fav Test", "rongyok", None, None, 1, 0, &episodes, None, None).unwrap();

        let fav = db.toggle_favorite(id).unwrap();
        assert!(fav);
        let fav2 = db.toggle_favorite(id).unwrap();
        assert!(!fav2);
    }

    #[test]
    fn test_toggle_favorite_reflected_in_library() {
        let db = test_db();
        let id = add_series(&db, "Fav Check", "rongyok", 1);

        db.toggle_favorite(id).unwrap();
        let entries = db.get_library(None).unwrap();
        assert!(entries[0].favorite);

        db.toggle_favorite(id).unwrap();
        let entries = db.get_library(None).unwrap();
        assert!(!entries[0].favorite);
    }

    // ─── Query builder ────────────────────────────

    #[test]
    fn test_query_filter_by_source() {
        let db = test_db();
        add_series(&db, "Rongyok A", "rongyok", 1);
        add_series(&db, "Titan B", "titan", 1);
        add_series(&db, "Rongyok C", "rongyok", 2);

        let query = LibraryQuery { source: Some("rongyok".to_string()), ..Default::default() };
        let results = db.get_library(Some(query)).unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|e| e.source == "rongyok"));
    }

    #[test]
    fn test_query_filter_by_favorite() {
        let db = test_db();
        let id1 = add_series(&db, "Regular", "rongyok", 1);
        let _id2 = add_series(&db, "Favorited", "rongyok", 1);
        db.toggle_favorite(id1).unwrap();

        let query = LibraryQuery { favorite_only: Some(true), ..Default::default() };
        let results = db.get_library(Some(query)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Regular");
    }

    #[test]
    fn test_query_search() {
        let db = test_db();
        add_series(&db, "Naruto Shippuden", "rongyok", 1);
        add_series(&db, "One Piece", "titan", 1);
        add_series(&db, "Naruto Classic", "rongyok", 1);

        let query = LibraryQuery { search: Some("naruto".to_string()), ..Default::default() };
        let results = db.get_library(Some(query)).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_query_search_case_insensitive() {
        let db = test_db();
        add_series(&db, "UPPERCASE Title", "rongyok", 1);

        let query = LibraryQuery { search: Some("uppercase".to_string()), ..Default::default() };
        let results = db.get_library(Some(query)).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_query_filter_by_tag() {
        let db = test_db();
        let id1 = add_series(&db, "Tagged", "rongyok", 1);
        let _id2 = add_series(&db, "Untagged", "titan", 1);
        let tag_id = db.create_tag("Anime").unwrap();
        db.assign_tag(id1, tag_id).unwrap();

        let query = LibraryQuery { tag_id: Some(tag_id), ..Default::default() };
        let results = db.get_library(Some(query)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Tagged");
    }

    #[test]
    fn test_query_filter_by_status_not_started() {
        let db = test_db();
        let _id1 = add_series(&db, "New Series", "rongyok", 2);
        let id2 = add_series(&db, "Old Series", "titan", 1);
        // Mark episode of Old Series as completed
        db.update_episode_status(id2, 1, "completed", None).unwrap();

        let query = LibraryQuery { status: Some("not_started".to_string()), ..Default::default() };
        let results = db.get_library(Some(query)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "New Series");
    }

    #[test]
    fn test_query_filter_by_status_complete() {
        let db = test_db();
        let id1 = add_series(&db, "Complete", "rongyok", 1);
        let _id2 = add_series(&db, "Incomplete", "titan", 2);
        db.update_episode_status(id1, 1, "completed", None).unwrap();

        let query = LibraryQuery { status: Some("complete".to_string()), ..Default::default() };
        let results = db.get_library(Some(query)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Complete");
    }

    #[test]
    fn test_query_sort_by_title() {
        let db = test_db();
        add_series(&db, "Zebra", "rongyok", 1);
        add_series(&db, "Apple", "titan", 1);
        add_series(&db, "Mango", "rongyok", 1);

        let query = LibraryQuery { sort: Some("title".to_string()), order: Some("asc".to_string()), ..Default::default() };
        let results = db.get_library(Some(query)).unwrap();
        assert_eq!(results[0].title, "Apple");
        assert_eq!(results[1].title, "Mango");
        assert_eq!(results[2].title, "Zebra");
    }

    #[test]
    fn test_query_combined_filters() {
        let db = test_db();
        let id1 = add_series(&db, "Rongyok Action", "rongyok", 1);
        let _id2 = add_series(&db, "Titan Action", "titan", 1);
        let _id3 = add_series(&db, "Rongyok Drama", "rongyok", 1);
        let tag_id = db.create_tag("Action").unwrap();
        db.assign_tag(id1, tag_id).unwrap();

        let query = LibraryQuery {
            source: Some("rongyok".to_string()),
            tag_id: Some(tag_id),
            ..Default::default()
        };
        let results = db.get_library(Some(query)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rongyok Action");
    }

    // ─── Episode management ───────────────────────

    #[test]
    fn test_update_episode_status() {
        let db = test_db();
        let id = add_series(&db, "Status Test", "rongyok", 1);

        db.update_episode_status(id, 1, "completed", Some("/path/to/file.mp4")).unwrap();

        let detail = db.get_series_detail(id).unwrap();
        assert_eq!(detail.episodes[0].status, "completed");
        assert_eq!(detail.episodes[0].file_path, Some("/path/to/file.mp4".to_string()));
    }

    #[test]
    fn test_update_episode_status_completed_updates_last_downloaded() {
        let db = test_db();
        let id = add_series(&db, "Timestamp Test", "rongyok", 1);

        let before = db.get_series_detail(id).unwrap();
        assert!(before.entry.last_downloaded.is_none());

        db.update_episode_status(id, 1, "completed", None).unwrap();

        let after = db.get_series_detail(id).unwrap();
        assert!(after.entry.last_downloaded.is_some());
    }

    // ─── Watch progress ───────────────────────────

    #[test]
    fn test_watch_progress_crud() {
        let db = test_db();
        let id = add_series(&db, "Progress Test", "rongyok", 1);

        // Initially no progress
        let progress = db.get_watch_progress(id, 1).unwrap();
        assert!(progress.is_none());

        // Set progress
        db.update_watch_progress(id, 1, 45.5, 120.0).unwrap();
        let progress = db.get_watch_progress(id, 1).unwrap().unwrap();
        assert_eq!(progress.position_seconds, 45.5);
        assert_eq!(progress.duration_seconds, 120.0);

        // Update progress (upsert)
        db.update_watch_progress(id, 1, 90.0, 120.0).unwrap();
        let progress = db.get_watch_progress(id, 1).unwrap().unwrap();
        assert_eq!(progress.position_seconds, 90.0);
    }

    #[test]
    fn test_mark_episode_watched_unwatched() {
        let db = test_db();
        let id = add_series(&db, "Watched Test", "rongyok", 2);

        let detail = db.get_series_detail(id).unwrap();
        assert!(!detail.episodes[0].watched);
        assert!(detail.episodes[0].watched_at.is_none());

        db.mark_episode_watched(id, 1).unwrap();
        let detail = db.get_series_detail(id).unwrap();
        assert!(detail.episodes[0].watched);
        assert!(detail.episodes[0].watched_at.is_some());

        db.mark_episode_unwatched(id, 1).unwrap();
        let detail = db.get_series_detail(id).unwrap();
        assert!(!detail.episodes[0].watched);
        assert!(detail.episodes[0].watched_at.is_none());
    }

    // ─── Library stats ────────────────────────────

    #[test]
    fn test_library_stats() {
        let db = test_db();
        let stats = db.get_library_stats().unwrap();
        assert_eq!(stats.total_series, 0);
    }

    #[test]
    fn test_library_stats_with_data() {
        let db = test_db();
        let id1 = add_series(&db, "Stats A", "rongyok", 2);
        let _id2 = add_series(&db, "Stats B", "titan", 1);
        db.update_episode_status(id1, 1, "completed", None).unwrap();
        db.toggle_favorite(id1).unwrap();
        db.create_tag("Tag1").unwrap();

        let stats = db.get_library_stats().unwrap();
        assert_eq!(stats.total_series, 2);
        assert_eq!(stats.total_episodes, 3);
        assert_eq!(stats.completed_episodes, 1);
        assert_eq!(stats.favorite_count, 1);
        assert_eq!(stats.tag_count, 1);
        assert_eq!(stats.by_status.in_progress, 1); // id1: 1 of 2 completed
        assert_eq!(stats.by_status.not_started, 1); // id2: 0 completed
    }

    // ─── Import / Export ──────────────────────────

    #[test]
    fn test_export_import_roundtrip() {
        let db = test_db();
        let tag_id = db.create_tag("Exported").unwrap();
        let id = add_series(&db, "Export Me", "rongyok", 2);
        db.assign_tag(id, tag_id).unwrap();

        let json = db.export_to_json().unwrap();
        assert!(!json.is_empty());
        assert!(json.contains("Export Me"));

        // Import into fresh DB
        let db2 = test_db();
        let count = db2.import_from_json(&json).unwrap();
        assert_eq!(count, 1);

        let entries = db2.get_library(None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Export Me");
    }

    #[test]
    fn test_import_invalid_json_fails() {
        let db = test_db();
        let result = db.import_from_json("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_empty_array() {
        let db = test_db();
        let count = db.import_from_json("[]").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_import_skips_empty_titles() {
        let db = test_db();
        let json = r#"[{"entry":{"title":"","source":"rongyok","totalEpisodes":1}}]"#;
        let count = db.import_from_json(json).unwrap();
        assert_eq!(count, 0);
    }

    // ─── Duplicate detection ──────────────────────

    #[test]
    fn test_find_duplicates_cross_source() {
        let db = test_db();
        add_series(&db, "Same Title", "rongyok", 1);
        add_series(&db, "Same Title", "titan", 1);

        let dupes = db.find_duplicates().unwrap();
        assert_eq!(dupes.len(), 1);
        let (primary, others) = &dupes[0];
        assert_eq!(primary.title, "Same Title");
        assert_eq!(others.len(), 1);
    }

    #[test]
    fn test_find_duplicates_normalized() {
        let db = test_db();
        add_series(&db, "My-Awesome Series", "rongyok", 1);
        add_series(&db, "my awesome series", "titan", 1);

        let dupes = db.find_duplicates().unwrap();
        assert_eq!(dupes.len(), 1);
    }

    #[test]
    fn test_find_duplicates_no_false_positives() {
        let db = test_db();
        add_series(&db, "Series A", "rongyok", 1);
        add_series(&db, "Series B", "rongyok", 1);

        let dupes = db.find_duplicates().unwrap();
        assert!(dupes.is_empty());
    }

    #[test]
    fn test_find_duplicates_same_source_not_flagged() {
        let db = test_db();
        // Same title, same source — this is a UNIQUE conflict, handled by upsert
        // but if somehow two different entries exist with same source, they won't
        // be flagged because find_duplicates filters for different sources
        let _id = add_series(&db, "Unique Source", "rongyok", 1);

        let dupes = db.find_duplicates().unwrap();
        assert!(dupes.is_empty());
    }

    // ─── Migration idempotency ────────────────────

    #[test]
    fn test_migrations_idempotent() {
        let db = test_db();
        // Opening the same DB again should not fail on duplicate migrations
        let dir = {
            let conn = db.conn.lock().unwrap();
            let db_path = conn.path().map(std::path::Path::new).unwrap();
            let dir = db_path.parent().unwrap().to_path_buf();
            drop(conn);
            dir
        };

        let _db2 = LibraryDb::new(&dir).expect("Re-opening DB with existing migrations should succeed");
    }

    // ─── Search (deprecated wrapper) ─────────────

    #[test]
    fn test_search_library() {
        let db = test_db();
        add_series(&db, "Hunter x Hunter", "rongyok", 1);
        add_series(&db, "Naruto", "titan", 1);

        let results = db.search_library("hunter").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Hunter x Hunter");
    }

    // ─── Metadata Tests ──────────────────────────────

    #[test]
    fn test_metadata_serialization() {
        let meta = SeriesMetadata {
            description: Some("An action anime".to_string()),
            rating: Some(8.5),
            year: Some(2024),
            genre: Some("Action".to_string()),
            duration: Some("24 min".to_string()),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let back: SeriesMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(back.description.as_deref(), Some("An action anime"));
        assert_eq!(back.rating, Some(8.5));
        assert_eq!(back.year, Some(2024));
        assert_eq!(back.genre.as_deref(), Some("Action"));
        assert_eq!(back.duration.as_deref(), Some("24 min"));
    }

    #[test]
    fn test_metadata_default_all_none() {
        let meta = SeriesMetadata::default();
        assert!(meta.description.is_none());
        assert!(meta.rating.is_none());
        assert!(meta.year.is_none());
        assert!(meta.genre.is_none());
        assert!(meta.duration.is_none());
    }

    #[test]
    fn test_save_series_with_metadata() {
        let db = test_db();
        let mut episodes = HashMap::new();
        episodes.insert(1, "https://example.com/ep1.m3u8".to_string());

        let meta = SeriesMetadata {
            description: Some("A thrilling series".to_string()),
            rating: Some(9.1),
            year: Some(2023),
            genre: Some("Drama".to_string()),
            duration: Some("45 min".to_string()),
        };

        let id = db.save_series(
            "Meta Test", "rongyok", Some("https://rongyok.com/meta"), None,
            1, 0, &episodes, None, Some(&meta),
        ).unwrap();

        let detail = db.get_series_detail(id).unwrap();
        assert_eq!(detail.entry.description.as_deref(), Some("A thrilling series"));
        assert_eq!(detail.entry.rating, Some(9.1));
        assert_eq!(detail.entry.year, Some(2023));
        assert_eq!(detail.entry.genre.as_deref(), Some("Drama"));
        assert_eq!(detail.entry.duration.as_deref(), Some("45 min"));
    }

    #[test]
    fn test_save_series_without_metadata_fields_are_none() {
        let db = test_db();
        let id = add_series(&db, "No Meta", "rongyok", 1);

        let detail = db.get_series_detail(id).unwrap();
        assert!(detail.entry.description.is_none());
        assert!(detail.entry.rating.is_none());
        assert!(detail.entry.year.is_none());
        assert!(detail.entry.genre.is_none());
        assert!(detail.entry.duration.is_none());
    }

    #[test]
    fn test_metadata_returned_in_library_list() {
        let db = test_db();
        let mut episodes = HashMap::new();
        episodes.insert(1, "https://example.com/ep1.m3u8".to_string());

        let meta = SeriesMetadata {
            description: Some("List test".to_string()),
            rating: Some(7.0),
            year: Some(2025),
            genre: Some("Comedy".to_string()),
            duration: Some("30 min".to_string()),
        };

        db.save_series(
            "Meta List", "rongyok", Some("https://rongyok.com/mlist"), None,
            1, 0, &episodes, None, Some(&meta),
        ).unwrap();

        let entries = db.get_library(None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].description.as_deref(), Some("List test"));
        assert_eq!(entries[0].rating, Some(7.0));
        assert_eq!(entries[0].year, Some(2025));
        assert_eq!(entries[0].genre.as_deref(), Some("Comedy"));
    }

    #[test]
    fn test_metadata_upsert_updates_fields() {
        let db = test_db();
        let mut episodes = HashMap::new();
        episodes.insert(1, "https://example.com/ep1.m3u8".to_string());

        let meta1 = SeriesMetadata {
            description: Some("Original".to_string()),
            rating: Some(5.0),
            year: Some(2020),
            genre: Some("Horror".to_string()),
            duration: Some("60 min".to_string()),
        };

        let id = db.save_series(
            "Upsert Meta", "rongyok", Some("https://rongyok.com/upsert"), None,
            1, 0, &episodes, None, Some(&meta1),
        ).unwrap();

        let meta2 = SeriesMetadata {
            description: Some("Updated description".to_string()),
            rating: Some(8.0),
            year: Some(2024),
            genre: Some("Thriller".to_string()),
            duration: Some("45 min".to_string()),
        };

        let id2 = db.save_series(
            "Upsert Meta Updated", "rongyok", Some("https://rongyok.com/upsert"), None,
            1, 0, &episodes, None, Some(&meta2),
        ).unwrap();

        assert_eq!(id, id2); // Same row due to ON CONFLICT

        let detail = db.get_series_detail(id).unwrap();
        assert_eq!(detail.entry.description.as_deref(), Some("Updated description"));
        assert_eq!(detail.entry.rating, Some(8.0));
        assert_eq!(detail.entry.year, Some(2024));
        assert_eq!(detail.entry.genre.as_deref(), Some("Thriller"));
    }

    #[test]
    fn test_migration_adds_metadata_columns() {
        let db = test_db();
        // Verify migration ran by checking that metadata fields work
        let mut episodes = HashMap::new();
        episodes.insert(1, "https://example.com/ep1.m3u8".to_string());

        // This should work without error if migration ran
        let meta = SeriesMetadata {
            description: Some("Migration test".to_string()),
            ..Default::default()
        };
        let id = db.save_series("Migration", "rongyok", None, None, 1, 0, &episodes, None, Some(&meta)).unwrap();
        let detail = db.get_series_detail(id).unwrap();
        assert_eq!(detail.entry.description.as_deref(), Some("Migration test"));
    }
}
