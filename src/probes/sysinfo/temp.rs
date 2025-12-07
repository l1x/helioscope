use sysinfo::Components;
use tracing::info;

pub fn probe_temperature() {
    info!("Starting temperature probe");

    let components = Components::new_with_refreshed_list();
    let component_count = components.len();

    info!("Detected {} temperature sensors", component_count);

    if component_count == 0 {
        info!("No temperature sensors available");
        return;
    }

    for component in &components {
        info!(
            label = component.label(),
            temperature_celsius = component.temperature(),
            max_celsius = component.max().map(|v| format!("{:.1}", v)).as_deref(),
            critical_celsius = component.critical().map(|v| format!("{:.1}", v)).as_deref(),
            "TEMP: "
        );
    }
}
