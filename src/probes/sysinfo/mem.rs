use sysinfo::System;
use tracing::info;

pub fn probe_memory(sys: &System) {
    info!("Starting memory probe");

    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let total_swap = sys.total_swap();
    let used_swap = sys.used_swap();

    let memory_usage_percent = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };

    let swap_usage_percent = if total_swap > 0 {
        (used_swap as f64 / total_swap as f64) * 100.0
    } else {
        0.0
    };

    info!(
        total_memory_bytes = total_memory,
        used_memory_bytes = used_memory,
        memory_usage_percent = format!("{:.1}", memory_usage_percent),
        "MEM: "
    );

    info!(
        total_swap_bytes = total_swap,
        used_swap_bytes = used_swap,
        swap_usage_percent = format!("{:.1}", swap_usage_percent),
        "SWAP: "
    );
}
