// helioscope-collector/src/store/reader.rs

use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use time::OffsetDateTime;
use tracing::{debug, info};

use crate::store::errors::StoreError;
use crate::store::queries::MetricDataPoint;

/// Shared connection pool for read operations
pub struct ReaderPool {
    pool: SqlitePool,
}

impl ReaderPool {
    /// Create a new reader pool for the current date's database
    pub async fn new(data_dir: &str) -> Result<Self, StoreError> {
        let date = get_current_date();
        let db_path = format!("{}/helioscope_{}.db", data_dir, date);
        let db_url = format!("sqlite://{}", db_path);

        info!("Initializing reader pool for: {}", db_url);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        info!("Reader pool initialized with 5 connections");

        Ok(Self { pool })
    }

    /// Query metrics for a specific node within a time range
    pub async fn query_node_metrics(
        &self,
        node_id: &str,
        metric_pattern: &str,
        hours: u32,
    ) -> Result<Vec<MetricDataPoint>, StoreError> {
        let now = OffsetDateTime::now_utc();
        let start_time = now - time::Duration::hours(hours as i64);

        let start_str = format_timestamp(start_time);
        let now_str = format_timestamp(now);

        debug!(
            "Querying metrics for node {} with pattern '{}' from {} to {}",
            node_id, metric_pattern, start_str, now_str
        );

        let rows = sqlx::query(
            r#"
            SELECT node_id, timestamp, probe_type, probe_name, probe_value
            FROM probe_data
            WHERE node_id = ?1
              AND probe_name LIKE ?2
              AND timestamp >= ?3
              AND timestamp <= ?4
            ORDER BY timestamp ASC
            "#,
        )
        .bind(node_id)
        .bind(metric_pattern)
        .bind(&start_str)
        .bind(&now_str)
        .fetch_all(&self.pool)
        .await?;

        let result: Vec<MetricDataPoint> = rows
            .into_iter()
            .map(|row| MetricDataPoint {
                node_id: row.get("node_id"),
                timestamp: row.get("timestamp"),
                probe_type: row.get("probe_type"),
                probe_name: row.get("probe_name"),
                probe_value: row.get("probe_value"),
            })
            .collect();

        debug!("Found {} data points", result.len());

        Ok(result)
    }

    /// Query all distinct node IDs in the database
    pub async fn query_all_node_ids(&self) -> Result<Vec<String>, StoreError> {
        debug!("Querying all distinct node IDs");

        let rows = sqlx::query(
            r#"
            SELECT DISTINCT node_id
            FROM probe_data
            ORDER BY node_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let node_ids: Vec<String> = rows.into_iter().map(|row| row.get("node_id")).collect();

        debug!("Found {} unique nodes", node_ids.len());

        Ok(node_ids)
    }

    /// Query the latest metrics for a specific node
    pub async fn query_latest_node_metrics(
        &self,
        node_id: &str,
    ) -> Result<Vec<MetricDataPoint>, StoreError> {
        debug!("Querying latest metrics for node {}", node_id);

        let rows = sqlx::query(
            r#"
            SELECT node_id, timestamp, probe_type, probe_name, probe_value
            FROM probe_data
            WHERE node_id = ?1
              AND timestamp = (
                  SELECT MAX(timestamp)
                  FROM probe_data
                  WHERE node_id = ?1
              )
            "#,
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;

        let result: Vec<MetricDataPoint> = rows
            .into_iter()
            .map(|row| MetricDataPoint {
                node_id: row.get("node_id"),
                timestamp: row.get("timestamp"),
                probe_type: row.get("probe_type"),
                probe_name: row.get("probe_name"),
                probe_value: row.get("probe_value"),
            })
            .collect();

        debug!("Found {} latest data points", result.len());

        Ok(result)
    }

    /// Close the connection pool gracefully
    pub async fn close(self) {
        self.pool.close().await;
        info!("Reader pool closed");
    }
}

/// Get current date in YYYY-MM-DD format for database filename
fn get_current_date() -> String {
    let now = OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}",
        now.year(),
        now.month() as u8,
        now.day()
    )
}

/// Format timestamp in ISO 8601 format for SQLite queries
fn format_timestamp(dt: OffsetDateTime) -> String {
    dt.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}
