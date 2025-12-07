use helioscope_common::ProbeDataPoint;
use sysinfo::System;
use time::OffsetDateTime;
use tracing::{debug, info};

pub fn probe_cpu(sys: &System, node_id: &str) -> Vec<ProbeDataPoint> {
    info!("Starting CPU probe");

    let mut data_points = Vec::new();
    let timestamp = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap();

    let core_count = sys.cpus().len();
    info!("Detected {} CPU cores", core_count);

    // Add core count metric
    data_points.push(ProbeDataPoint {
        node_id: node_id.to_string(),
        timestamp: timestamp.clone(),
        probe_type: "sysinfo".to_string(),
        probe_name: "cpu_core_count".to_string(),
        probe_value: core_count.to_string(),
    });

    // Add per-core metrics
    for (idx, cpu) in sys.cpus().iter().enumerate() {
        debug!(
            core = idx,
            name = cpu.name(),
            frequency_mhz = cpu.frequency(),
            usage_percent = cpu.cpu_usage(),
            "CPU: "
        );

        // Frequency
        data_points.push(ProbeDataPoint {
            node_id: node_id.to_string(),
            timestamp: timestamp.clone(),
            probe_type: "sysinfo".to_string(),
            probe_name: format!("cpu_core_{}_frequency_mhz", idx),
            probe_value: cpu.frequency().to_string(),
        });

        // Usage percentage
        data_points.push(ProbeDataPoint {
            node_id: node_id.to_string(),
            timestamp: timestamp.clone(),
            probe_type: "sysinfo".to_string(),
            probe_name: format!("cpu_core_{}_usage_percent", idx),
            probe_value: format!("{:.1}", cpu.cpu_usage()),
        });
    }

    data_points
}
