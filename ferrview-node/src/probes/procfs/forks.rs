use ferrview_common::ProbeDataPoint;
use std::io;
use tracing::info;

/// Probe the fork rate by reading /proc/stat
///
/// This probe reads the cumulative number of forks (processes created) since boot
/// from /proc/stat. The value is a monotonically increasing counter.
///
/// The collector can calculate the fork rate by computing the derivative of this value.
#[cfg(target_os = "linux")]
pub fn probe_forks(node_id: &str) -> Result<Vec<ProbeDataPoint>, io::Error> {
    use crate::utils::timestamp::get_utc_timestamp;
    use std::fs;
    use tracing::warn;

    info!("Starting forks probe");

    let timestamp = get_utc_timestamp();
    let mut data_points = Vec::new();

    // Read /proc/stat
    let stat_content = fs::read_to_string("/proc/stat")?;

    // Find the "processes" line and extract the fork count
    for line in stat_content.lines() {
        if line.starts_with("processes ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let fork_count = parts[1];
                info!("Fork count: {}", fork_count);

                data_points.push(ProbeDataPoint {
                    node_id: node_id.to_string(),
                    timestamp: timestamp.clone(),
                    probe_type: "procfs".to_string(),
                    probe_name: "forks_total".to_string(),
                    probe_value: fork_count.to_string(),
                });

                info!("Collected {} fork metrics", data_points.len());
                return Ok(data_points);
            }
        }
    }

    warn!("Could not find 'processes' line in /proc/stat");
    Ok(data_points)
}

/// Non-Linux platforms return empty data
#[cfg(not(target_os = "linux"))]
pub fn probe_forks(_node_id: &str) -> Result<Vec<ProbeDataPoint>, io::Error> {
    info!("Forks probe not supported on this platform");
    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn test_probe_forks_reads_procstat() {
        let result = probe_forks("test-node");
        assert!(result.is_ok());
        let data_points = result.unwrap();

        // Should have exactly one data point
        assert_eq!(data_points.len(), 1);

        let dp = &data_points[0];
        assert_eq!(dp.node_id, "test-node");
        assert_eq!(dp.probe_type, "procfs");
        assert_eq!(dp.probe_name, "forks_total");

        // Value should be a valid positive number
        let value: u64 = dp.probe_value.parse().expect("fork count should be a number");
        assert!(value > 0, "fork count should be positive");
    }

    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_probe_forks_returns_empty_on_non_linux() {
        let result = probe_forks("test-node");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
