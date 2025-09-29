// src/db.rs
use rusqlite::Connection;
use std::path::Path;

pub fn init() -> rusqlite::Result<()> {
    let db_path = super::config::database_url();
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                Some(format!("Failed to create directory: {}", e))
            )
        })?;
    }
    
    let init = !Path::new(&db_path).exists();
    let conn = Connection::open(&db_path)?;
    if init {
        conn.execute_batch(
            "CREATE TABLE guild_affiliates (
                guild_id TEXT NOT NULL,
                region TEXT NOT NULL,
                tracking_tag TEXT NOT NULL,
                PRIMARY KEY (guild_id, region)
            );
            CREATE TABLE guild_settings (
                guild_id TEXT PRIMARY KEY,
                footer_text TEXT NOT NULL
            );
            CREATE TABLE link_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                guild_id TEXT NOT NULL,
                region TEXT NOT NULL,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
            );"
        )?;
    }
    Ok(())
}

pub fn with_connection<F, T>(f: F) -> rusqlite::Result<T>
where
    F: FnOnce(&Connection) -> rusqlite::Result<T>,
{
    let conn = Connection::open(&super::config::database_url())?;
    f(&conn)
}