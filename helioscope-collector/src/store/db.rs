// helioscope-collector/src/store/db.rs
use sqlx::Connection;
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection};
use std::path::Path;
use std::str::FromStr;
use tracing::{debug, info};

use crate::store::errors::StoreError;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS probe_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    probe_type TEXT NOT NULL,
    probe_name TEXT NOT NULL,
    probe_value TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_probe_data_node_timestamp
    ON probe_data(node_id, timestamp);

CREATE INDEX IF NOT EXISTS idx_probe_data_probe_type
    ON probe_data(probe_type);
"#;

pub struct Database {
    conn: SqliteConnection,
}

impl Database {
    /// Initialize a new database connection for a specific date
    /// Creates: data/helioscope_2024-12-08.db
    pub async fn new_for_date(data_dir: &str, date: &str) -> Result<Self, StoreError> {
        std::fs::create_dir_all(data_dir)?;

        let db_filename = format!("helioscope_{}.db", date);
        let db_path = Path::new(data_dir).join(&db_filename);
        let db_url = format!("sqlite://{}", db_path.display());

        info!("Initializing database at: {}", db_url);

        let options = SqliteConnectOptions::from_str(&db_url)?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let mut conn = SqliteConnection::connect_with(&options).await?;

        debug!("Running database migrations");
        sqlx::query(SCHEMA).execute(&mut conn).await?;

        info!("Database initialized successfully: {}", db_filename);

        Ok(Self { conn })
    }

    /// Get mutable reference to the connection
    pub fn conn(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }

    /// Close the database connection
    pub async fn close(self) -> Result<(), StoreError> {
        self.conn.close().await?;
        Ok(())
    }
}
