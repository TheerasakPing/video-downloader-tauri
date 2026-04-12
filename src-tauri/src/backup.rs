// Backup Module - Database backup and restore functionality
use std::path::PathBuf;
use serde_json::{Map, Value};

/// Backup Manager for creating and restoring database backups
/// Exports all SQLite databases to JSON format for easy backup/restore
pub struct BackupManager {
    data_dir: PathBuf,
}

impl BackupManager {
    /// Create a new BackupManager with the specified data directory
    pub fn new(data_dir: &std::path::Path) -> Self {
        Self { data_dir: data_dir.to_path_buf() }
    }

    /// Create a backup of all database files to JSON format
    /// Returns the path to the created backup file
    pub fn create_backup(&self, output_path: &str) -> Result<String, String> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = if output_path.is_empty() {
            format!("backup_{}.json", timestamp)
        } else {
            output_path.to_string()
        };

        // Collect all .db files in data directory
        let db_files: Vec<PathBuf> = std::fs::read_dir(&self.data_dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "db").unwrap_or(false))
            .collect();

        if db_files.is_empty() {
            return Err("No database files found to backup".to_string());
        }

        // Create a JSON backup of all databases
        let mut backup_data: Map<String, Value> = Map::new();

        for db_path in &db_files {
            let db_name = db_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let conn = rusqlite::Connection::open(db_path)
                .map_err(|e| format!("Failed to open {}: {}", db_name, e))?;

            // Get all tables (excluding sqlite system tables) - collect immediately to fix lifetime issue
            let tables: Vec<String> = {
                let mut stmt = conn.prepare(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
                ).map_err(|e| format!("Query failed for {}: {}", db_name, e))?;

                let mut tables_vec = Vec::new();
                let rows = stmt.query_map([], |row| row.get::<_, String>(0))
                    .map_err(|e| format!("Query failed for {}: {}", db_name, e))?;

                for row in rows {
                    if let Ok(table_name) = row {
                        tables_vec.push(table_name);
                    }
                }
                tables_vec
            };

            let mut db_export: Map<String, Value> = Map::new();

            for table in &tables {
                // Get all rows with a simplified approach
                let mut stmt = conn.prepare(&format!("SELECT * FROM {}", table))
                    .map_err(|e| format!("Select failed for {}.{}: {}", db_name, table, e))?;

                let _column_count = stmt.column_count();
                let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

                let mut rows: Vec<Value> = Vec::new();

                let row_results = stmt.query_map([], |row| {
                    let mut map = Map::new();
                    for (i, col_name) in column_names.iter().enumerate() {
                        // Try to get value as String first, then other types
                        let value: Value = match row.get::<_, Option<String>>(i) {
                            Ok(Some(v)) => Value::String(v),
                            Ok(None) => Value::Null,
                            Err(_) => match row.get::<_, Option<i64>>(i) {
                                Ok(Some(v)) => Value::Number(v.into()),
                                Ok(None) => Value::Null,
                                Err(_) => match row.get::<_, Option<f64>>(i) {
                                    Ok(Some(v)) => {
                                        if v.fract() == 0.0 {
                                            Value::Number(serde_json::Number::from(v as i64))
                                        } else {
                                            serde_json::Number::from_f64(v).map(Value::Number).unwrap_or(Value::Null)
                                        }
                                    },
                                    Ok(None) => Value::Null,
                                    Err(_) => Value::Null,
                                },
                            },
                        };
                        map.insert(col_name.clone(), value);
                    }
                    Ok(Value::Object(map))
                })
                .map_err(|e| format!("Row mapping failed for {}.{}: {}", db_name, table, e))?;

                for row in row_results {
                    if let Ok(r) = row {
                        rows.push(r);
                    }
                }

                db_export.insert(table.clone(), Value::Array(rows));
            }

            backup_data.insert(db_name, Value::Object(db_export));
        }

        // Add metadata
        let metadata = serde_json::json!({
            "version": "1.0",
            "created_at": chrono::Utc::now().to_rfc3339(),
            "databases": db_files.iter().map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string()).collect::<Vec<_>>()
        });
        backup_data.insert("_metadata".to_string(), metadata);

        // Write to file
        let json = serde_json::to_string_pretty(&backup_data)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;
        std::fs::write(&backup_file, json)
            .map_err(|e| format!("Failed to write backup file: {}", e))?;

        Ok(backup_file)
    }

    /// Restore databases from a JSON backup file
    /// Returns the number of rows restored
    pub fn restore_backup(&self, backup_path: &str) -> Result<i32, String> {
        let json = std::fs::read_to_string(backup_path)
            .map_err(|e| format!("Failed to read backup file: {}", e))?;

        let backup_data: Map<String, Value> = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse backup JSON: {}", e))?;

        // Check for metadata
        if let Some(meta) = backup_data.get("_metadata") {
            eprintln!("Restoring backup created at: {:?}", meta.get("created_at"));
        }

        let mut restored_rows = 0i32;

        for (db_name, tables_data) in &backup_data {
            if db_name == "_metadata" {
                continue;
            }

            let db_path = self.data_dir.join(db_name);

            // Open database (creates if doesn't exist)
            let conn = rusqlite::Connection::open(&db_path)
                .map_err(|e| format!("Failed to open/create {}: {}", db_name, e))?;

            if let Value::Object(tables) = tables_data {
                for (table_name, rows) in tables {
                    if let Value::Array(row_list) = rows {
                        // Clear existing data
                        conn.execute(&format!("DELETE FROM {}", table_name), [])
                            .map_err(|e| format!("Failed to clear {}.{}: {}", db_name, table_name, e))?;

                        // Insert rows
                        for row in row_list {
                            if let Value::Object(cols) = row {
                                if cols.is_empty() {
                                    continue;
                                }

                                let columns: Vec<&str> = cols.keys().map(|k| k.as_str()).collect();
                                let placeholders: Vec<String> = (0..columns.len())
                                    .map(|i| format!("?{}", i + 1))
                                    .collect();

                                let sql = format!(
                                    "INSERT OR REPLACE INTO {} ({}) VALUES ({})",
                                    table_name,
                                    columns.join(", "),
                                    placeholders.join(", ")
                                );

                                // Build parameters dynamically
                                let mut param_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
                                for col in &columns {
                                    let value = cols.get(*col).unwrap_or(&Value::Null);
                                    let param: Box<dyn rusqlite::ToSql> = match value {
                                        Value::String(s) => Box::new(s.clone()),
                                        Value::Number(n) => {
                                            if let Some(i) = n.as_i64() {
                                                Box::new(i)
                                            } else if let Some(f) = n.as_f64() {
                                                Box::new(f)
                                            } else {
                                                Box::new(0i64)
                                            }
                                        },
                                        Value::Bool(b) => Box::new(if *b { 1i32 } else { 0i32 }),
                                        Value::Null => Box::new(Option::<String>::None),
                                        _ => Box::new(value.to_string()),
                                    };
                                    param_values.push(param);
                                }

                                let param_refs: Vec<&dyn rusqlite::ToSql> = param_values.iter()
                                    .map(|p| p.as_ref())
                                    .collect();

                                match conn.execute(&sql, param_refs.as_slice()) {
                                    Ok(_) => restored_rows += 1,
                                    Err(e) => {
                                        eprintln!("Warning: Failed to restore row in {}.{}: {}", db_name, table_name, e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(restored_rows)
    }

    /// Get the data directory path
    #[allow(dead_code)]
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a unique temp directory for test isolation
    fn test_dir() -> std::path::PathBuf {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("vdt_test_backup_{}", unique_id));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    fn test_manager(dir: &std::path::Path) -> BackupManager {
        BackupManager::new(dir)
    }

    /// Seed a test .db file with a table and rows
    fn seed_test_db(dir: &std::path::Path, db_name: &str) {
        let db_path = dir.join(db_name);
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL, value INTEGER);
             INSERT INTO items (name, value) VALUES ('alpha', 10);
             INSERT INTO items (name, value) VALUES ('beta', 20);"
        ).unwrap();
    }

    #[test]
    fn test_backup_manager_creation() {
        let temp_dir = std::env::temp_dir();
        let manager = BackupManager::new(&temp_dir);
        assert_eq!(manager.data_dir(), &temp_dir);
    }

    #[test]
    fn test_create_backup_no_db_files() {
        let dir = test_dir();
        let manager = test_manager(&dir);
        // Empty directory — no .db files
        let result = manager.create_backup("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No database files"));
    }

    #[test]
    fn test_create_backup_with_data() {
        let dir = test_dir();
        let manager = test_manager(&dir);
        seed_test_db(&dir, "test_backup.db");

        let output = dir.join("backup_output.json");
        let result = manager.create_backup(output.to_str().unwrap());
        assert!(result.is_ok());

        // Verify the backup file exists and contains valid JSON
        let json_str = std::fs::read_to_string(output).unwrap();
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        // Should contain the database name as a key
        assert!(json.get("test_backup.db").is_some());
        // Should contain metadata
        assert!(json.get("_metadata").is_some());
        assert_eq!(json["_metadata"]["version"], "1.0");

        // Verify table data
        let items = &json["test_backup.db"]["items"];
        assert!(items.is_array());
        assert_eq!(items.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_create_backup_default_filename() {
        let dir = test_dir();
        let manager = test_manager(&dir);
        seed_test_db(&dir, "sample.db");

        // Empty string → auto-generated filename with timestamp
        let result = manager.create_backup("");
        assert!(result.is_ok());
        let filename = result.unwrap();
        assert!(filename.starts_with("backup_"));
        assert!(filename.ends_with(".json"));
    }

    #[test]
    fn test_restore_backup_roundtrip() {
        let dir = test_dir();
        let manager = test_manager(&dir);
        seed_test_db(&dir, "roundtrip.db");

        // Create backup
        let backup_path = dir.join("backup.json");
        manager.create_backup(backup_path.to_str().unwrap()).unwrap();

        // Clear data from the table (schema stays intact — restore only writes rows)
        let db_path = dir.join("roundtrip.db");
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute("DELETE FROM items", []).unwrap();
        }
        // Verify table is empty
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            let count: i32 = conn.query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0)).unwrap();
            assert_eq!(count, 0);
        }

        // Restore
        let count = manager.restore_backup(backup_path.to_str().unwrap()).unwrap();
        assert_eq!(count, 2); // 2 rows restored

        // Verify data is back
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        let row_count: i32 = conn.query_row("SELECT COUNT(*) FROM items", [], |r| r.get(0)).unwrap();
        assert_eq!(row_count, 2);
    }

    #[test]
    fn test_restore_backup_invalid_json() {
        let dir = test_dir();
        let manager = test_manager(&dir);
        let bad_path = dir.join("bad.json");
        std::fs::write(&bad_path, "not valid json!!!").unwrap();

        let result = manager.restore_backup(bad_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("parse"));
    }

    #[test]
    fn test_restore_backup_missing_file() {
        let dir = test_dir();
        let manager = test_manager(&dir);
        let result = manager.restore_backup("/nonexistent/path/backup.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_restore_backup_preserves_values() {
        let dir = test_dir();
        let manager = test_manager(&dir);
        seed_test_db(&dir, "values_test.db");

        let backup_path = dir.join("values_backup.json");
        manager.create_backup(backup_path.to_str().unwrap()).unwrap();

        // Clear the table
        let db_path = dir.join("values_test.db");
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute("DELETE FROM items", []).unwrap();
        }

        // Restore and verify values
        manager.restore_backup(backup_path.to_str().unwrap()).unwrap();

        let conn = rusqlite::Connection::open(&db_path).unwrap();
        let (name, value): (String, i32) = conn.query_row(
            "SELECT name, value FROM items WHERE id = 1", [], |r| Ok((r.get(0)?, r.get(1)?))
        ).unwrap();
        assert_eq!(name, "alpha");
        assert_eq!(value, 10);
    }

    #[test]
    fn test_backup_multiple_databases() {
        let dir = test_dir();
        let manager = test_manager(&dir);
        seed_test_db(&dir, "db_one.db");

        // Create a second database
        let db2_path = dir.join("db_two.db");
        {
            let conn = rusqlite::Connection::open(&db2_path).unwrap();
            conn.execute_batch(
                "CREATE TABLE logs (id INTEGER PRIMARY KEY, msg TEXT);
                 INSERT INTO logs (msg) VALUES ('hello');"
            ).unwrap();
        }

        let backup_path = dir.join("multi_backup.json");
        manager.create_backup(backup_path.to_str().unwrap()).unwrap();

        let json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&backup_path).unwrap()
        ).unwrap();

        assert!(json.get("db_one.db").is_some());
        assert!(json.get("db_two.db").is_some());

        // Metadata should list both databases
        let dbs = json["_metadata"]["databases"].as_array().unwrap();
        assert_eq!(dbs.len(), 2);
    }
}
