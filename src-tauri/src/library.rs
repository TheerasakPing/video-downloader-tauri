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
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

        let poster_dir = app_data_dir.join("library_posters");
        std::fs::create_dir_all(&poster_dir).ok();

        let db = Self {
            conn: Mutex::new(conn),
            poster_dir,
        };
        db.init_schema()?;
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
        Ok(())
    }

    pub fn get_library(&self) -> Result<Vec<LibraryEntry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT l.id, l.parser_series_id, l.title, l.source, l.source_url,
                    l.poster_path, l.total_episodes, l.date_added, l.last_downloaded,
                    COUNT(CASE WHEN e.status = 'completed' THEN 1 END) as completed_count
             FROM library l
             LEFT JOIN library_episodes e ON e.library_id = l.id
             GROUP BY l.id
             ORDER BY l.date_added DESC"
        ).map_err(|e| format!("Query library failed: {}", e))?;

        let entries = stmt.query_map([], |row| {
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
            })
        }).map_err(|e| format!("Map library entries failed: {}", e))?
        .filter_map(|e| e.ok())
        .collect();

        Ok(entries)
    }

    pub fn get_series_detail(&self, library_id: i64) -> Result<SeriesDetail, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let entry: LibraryEntry = conn.query_row(
            "SELECT l.id, l.parser_series_id, l.title, l.source, l.source_url,
                    l.poster_path, l.total_episodes, l.date_added, l.last_downloaded,
                    COUNT(CASE WHEN e.status = 'completed' THEN 1 END)
             FROM library l LEFT JOIN library_episodes e ON e.library_id = l.id
             WHERE l.id = ?1 GROUP BY l.id",
            params![library_id],
            |row| Ok(LibraryEntry {
                id: row.get(0)?, parser_series_id: row.get(1)?, title: row.get(2)?,
                source: row.get(3)?, source_url: row.get(4)?, poster_path: row.get(5)?,
                total_episodes: row.get(6)?, date_added: row.get(7)?, last_downloaded: row.get(8)?,
                completed_count: row.get(9)?,
            }),
        ).map_err(|e| format!("Series not found: {}", e))?;

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

    pub fn search_library(&self, query: &str) -> Result<Vec<LibraryEntry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT l.id, l.parser_series_id, l.title, l.source, l.source_url,
                    l.poster_path, l.total_episodes, l.date_added, l.last_downloaded,
                    COUNT(CASE WHEN e.status = 'completed' THEN 1 END)
             FROM library l LEFT JOIN library_episodes e ON e.library_id = l.id
             WHERE l.title LIKE ?1
             GROUP BY l.id ORDER BY l.date_added DESC"
        ).map_err(|e| e.to_string())?;

        let entries: Vec<LibraryEntry> = stmt.query_map(params![pattern], |row| {
            Ok(LibraryEntry {
                id: row.get(0)?, parser_series_id: row.get(1)?, title: row.get(2)?,
                source: row.get(3)?, source_url: row.get(4)?, poster_path: row.get(5)?,
                total_episodes: row.get(6)?, date_added: row.get(7)?, last_downloaded: row.get(8)?,
                completed_count: row.get(9)?,
            })
        }).map_err(|e| e.to_string())?.filter_map(|e| e.ok()).collect();

        Ok(entries)
    }
}
