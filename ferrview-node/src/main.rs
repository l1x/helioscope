use argh::FromArgs;
use std::time::Duration;
use sysinfo::{Components, Disks, Networks, System};
use tracing::{debug, error, info};
use tracing_subscriber::{EnvFilter, fmt::time::UtcTime};

use crate::{
    client::http::HttpClient,
    client::retry::send_with_retry,
    config::Config,
    probes::{
        procfs,
        sysinfo::{cpu, disk, mem, network, statik, temp},
    },
    utils::timestamp::get_utc_formatter,
};

mod client;
mod config;
mod probes;
mod utils;

fn default_config_file() -> String {
    String::from("ferrview-node.toml")
}

#[derive(FromArgs, Debug)]
#[argh(description = "Helioscope metrics collection node")]
#[argh(help_triggers("-h", "--help", "help"))]
pub struct Argz {
    /// config file location
    #[argh(option, default = "default_config_file()")]
    config_file: String,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    let timer = UtcTime::new(get_utc_formatter());
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_timer(timer)
        .init();

    info!("Starting ferrview-node");

    let argz: Argz = argh::from_env();
    debug!("Args: {:?}", argz);
    info!("Config file: {}", argz.config_file);

    let config = Config::load(&argz.config_file).expect("Failed to load configuration");

    debug!("Config: {:?}", config);
    info!("Node ID: {}", config.node_id);
    info!("Collector address: {}", config.metrics_collector_addr);
    info!("Collection interval: {}s", config.collection_interval_secs);

    // Initialize HTTP client
    let client = HttpClient::new(&config.metrics_collector_addr);

    // Initialize system info
    let mut sys = System::new_all();
    let mut disks = Disks::new_with_refreshed_list();
    let mut components = Components::new_with_refreshed_list();
    let mut networks = Networks::new_with_refreshed_list();

    // Perform initial CPU refresh with delay for accurate first reading
    info!("Performing initial CPU refresh for accurate readings");
    sys.refresh_cpu_usage();
    tokio::time::sleep(Duration::from_millis(200)).await;
    sys.refresh_cpu_usage();

    let interval = Duration::from_secs(config.collection_interval_secs);

    info!("Starting collection loop");

    loop {
        // Refresh system information
        sys.refresh_all();

        let mut all_data = Vec::new();

        // Collect all enabled probes

        if config.probes.sysinfo.static_info {
            let static_info = statik::probe_static_info(&config.node_id);
            debug!("Collected {} static info metrics", static_info.len());
            all_data.extend(static_info);
        }

        if config.probes.sysinfo.cpu {
            let cpu_data = cpu::probe_cpu(&sys, &config.node_id);
            debug!("Collected {} CPU metrics", cpu_data.len());
            all_data.extend(cpu_data);
        }

        if config.probes.sysinfo.memory {
            let mem_data = mem::probe_memory(&sys, &config.node_id);
            debug!("Collected {} memory metrics", mem_data.len());
            all_data.extend(mem_data);
        }

        if config.probes.sysinfo.disk {
            let disk_data = disk::probe_disks(&mut disks, &config.node_id);
            debug!("Collected {} disk metrics", disk_data.len());
            all_data.extend(disk_data);
        }

        if config.probes.sysinfo.temperature {
            let temp_data = temp::probe_temperature(&mut components, &config.node_id);
            debug!("Collected {} temperature metrics", temp_data.len());
            all_data.extend(temp_data);
        }

        if config.probes.sysinfo.network {
            let network_data = network::probe_networks(&mut networks, &config.node_id);
            debug!("Collected {} network metrics", network_data.len());
            all_data.extend(network_data);
        }

        if config.probes.procfs.forks {
            match procfs::forks::probe_forks(&config.node_id) {
                Ok(forks_data) => {
                    debug!("Collected {} fork metrics", forks_data.len());
                    all_data.extend(forks_data);
                }
                Err(e) => error!("Failed to collect fork metrics: {}", e),
            }
        }

        info!("Collected {} total metrics", all_data.len());

        // Send batch with retry
        if !all_data.is_empty() {
            let data_clone = all_data.clone();
            match send_with_retry(|| client.send_batch(data_clone.clone()), 3).await {
                Ok(_) => info!("Batch sent successfully"),
                Err(e) => error!("Failed to send batch after retries: {}", e),
            }
        } else {
            info!("No metrics collected, skipping send");
        }

        // Wait for next collection interval
        debug!("Sleeping for {:?}", interval);
        tokio::time::sleep(interval).await;
    }
}
