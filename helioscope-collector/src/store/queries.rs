// helioscope-collector/src/store/queries.rs

use sqlx::SqliteConnection;
use time::OffsetDateTime;
use tracing::debug;

use crate::store::errors::StoreError;

/// A single metric data point from the database
#[derive(Debug, Clone)]
pub struct MetricDataPoint {
    pub node_id: String,
    pub timestamp: String,
    pub probe_type: String,
    pub probe_name: String,
    pub probe_value: String,
}

/// Query metrics for a specific node within a time range
pub async fn query_node_metrics(
    conn: &mut SqliteConnection,
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

    let rows = sqlx::query_as::<_, MetricRow>(
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
    .fetch_all(conn)
    .await?;

    let result: Vec<MetricDataPoint> = rows
        .into_iter()
        .map(|row| MetricDataPoint {
            node_id: row.node_id,
            timestamp: row.timestamp,
            probe_type: row.probe_type,
            probe_name: row.probe_name,
            probe_value: row.probe_value,
        })
        .collect();

    debug!("Found {} data points", result.len());

    Ok(result)
}

/// Query all distinct node IDs in the database
pub async fn query_all_node_ids(conn: &mut SqliteConnection) -> Result<Vec<String>, StoreError> {
    debug!("Querying all distinct node IDs");

    let rows = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT DISTINCT node_id
        FROM probe_data
        ORDER BY node_id
        "#,
    )
    .fetch_all(conn)
    .await?;

    let node_ids: Vec<String> = rows.into_iter().map(|(id,)| id).collect();

    debug!("Found {} unique nodes", node_ids.len());

    Ok(node_ids)
}

/// Query the latest metrics for a specific node
pub async fn query_latest_node_metrics(
    conn: &mut SqliteConnection,
    node_id: &str,
) -> Result<Vec<MetricDataPoint>, StoreError> {
    debug!("Querying latest metrics for node {}", node_id);

    let rows = sqlx::query_as::<_, MetricRow>(
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
    .fetch_all(conn)
    .await?;

    let result: Vec<MetricDataPoint> = rows
        .into_iter()
        .map(|row| MetricDataPoint {
            node_id: row.node_id,
            timestamp: row.timestamp,
            probe_type: row.probe_type,
            probe_name: row.probe_name,
            probe_value: row.probe_value,
        })
        .collect();

    debug!("Found {} latest data points", result.len());

    Ok(result)
}

/// Query metrics for all nodes within a time range
pub async fn query_all_nodes_metrics(
    conn: &mut SqliteConnection,
    metric_pattern: &str,
    hours: u32,
) -> Result<Vec<MetricDataPoint>, StoreError> {
    let now = OffsetDateTime::now_utc();
    let start_time = now - time::Duration::hours(hours as i64);

    let start_str = format_timestamp(start_time);
    let now_str = format_timestamp(now);

    debug!(
        "Querying metrics for all nodes with pattern '{}' from {} to {}",
        metric_pattern, start_str, now_str
    );

    let rows = sqlx::query_as::<_, MetricRow>(
        r#"
        SELECT node_id, timestamp, probe_type, probe_name, probe_value
        FROM probe_data
        WHERE probe_name LIKE ?1
          AND timestamp >= ?2
          AND timestamp <= ?3
        ORDER BY node_id, timestamp ASC
        "#,
    )
    .bind(metric_pattern)
    .bind(&start_str)
    .bind(&now_str)
    .fetch_all(conn)
    .await?;

    let result: Vec<MetricDataPoint> = rows
        .into_iter()
        .map(|row| MetricDataPoint {
            node_id: row.node_id,
            timestamp: row.timestamp,
            probe_type: row.probe_type,
            probe_name: row.probe_name,
            probe_value: row.probe_value,
        })
        .collect();

    debug!("Found {} data points across all nodes", result.len());

    Ok(result)
}

/// Helper struct for sqlx query mapping
#[derive(sqlx::FromRow)]
struct MetricRow {
    node_id: String,
    timestamp: String,
    probe_type: String,
    probe_name: String,
    probe_value: String,
}

/// Format timestamp in ISO 8601 format for SQLite queries
fn format_timestamp(dt: OffsetDateTime) -> String {
    dt.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| String::from("1970-01-01T00:00:00Z"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_format_timestamp() {
        let dt = datetime!(2024-12-08 10:30:00 UTC);
        let formatted = format_timestamp(dt);
        assert!(formatted.starts_with("2024-12-08"));
        assert!(formatted.contains("10:30:00"));
    }

    #[test]
    fn test_metric_data_point_clone() {
        let point = MetricDataPoint {
            node_id: "test".to_string(),
            timestamp: "2024-12-08T10:00:00Z".to_string(),
            probe_type: "sysinfo".to_string(),
            probe_name: "cpu_usage".to_string(),
            probe_value: "50.0".to_string(),
        };

        let cloned = point.clone();
        assert_eq!(point.node_id, cloned.node_id);
        assert_eq!(point.probe_value, cloned.probe_value);
    }
}
