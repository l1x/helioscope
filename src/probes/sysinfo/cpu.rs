use sysinfo::System;
use tracing::info;

pub fn probe_cpu(sys: &System) {
    info!("Starting CPU probe");

    let core_count = sys.cpus().len();
    info!("Detected {} CPU cores", core_count);

    for (idx, cpu) in sys.cpus().iter().enumerate() {
        info!(
            core = idx,
            name = cpu.name(),
            frequency_mhz = cpu.frequency(),
            "CPU: "
        );
    }
}
