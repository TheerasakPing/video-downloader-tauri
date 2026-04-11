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
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = chrono::Utc::now().to_rfc3339();

        let source_url_val = source_url.unwrap_or("");

        conn.execute(
            "INSERT INTO library (parser_series_id, title, source, source_url, total_episodes, date_added, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(source, source_url) DO UPDATE SET
                title = excluded.title,
                total_episodes = excluded.total_episodes,
                metadata = excluded.metadata",
            params![parser_series_id, title, source, source_url_val, total_episodes, now, metadata],
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
                    l.favorite
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
            }),
        ).map_err(|e| format!("Series not found: {}", e))?;

        let tags_map = Self::get_tags_for_entries(&conn, &[library_id])?;
        let entry = LibraryEntry {
            tags: tags_map.get(&library_id).cloned().unwrap_or_default(),
            ..entry
        };

        let mut stmt = conn.prepare(
            "SELECT id, library_id, episode_number, video_url, file_path, quality, file_size, status
             FROM library_episodes WHERE library_id = ?1 ORDER BY episode_number"
        ).map_err(|e| e.to_string())?;

        let episodes: Vec<LibraryEpisode> = stmt.query_map(params![library_id], |row| {
            Ok(LibraryEpisode {
                id: row.get(0)?, library_id: row.get(1)?, episode_number: row.get(2)?,
                video_url: row.get(3)?, file_path: row.get(4)?, quality: row.get(5)?,
                file_size: row.get(6)?, status: row.get(7)?,
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
}
