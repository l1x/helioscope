# Helioscope

A lightweight system monitoring tool written in Rust.

## Overview

Helioscope is a modular system monitoring utility designed to collect and display system information through configurable probes. It uses the `sysinfo` crate to gather comprehensive system data and provides structured output via structured logging.

## Features

- **CPU Monitoring**: Core count, frequency, and individual core information
- **Memory Analysis**: RAM usage, swap usage, and percentage calculations
- **Temperature Sensing**: Hardware temperature readings with critical thresholds
- **Configurable Probes**: Enable/disable specific monitoring modules via TOML configuration
- **Structured Logging**: JSON-formatted output with timestamps for easy parsing
- **Lightweight**: Minimal dependencies and optimized binary size

## Installation

### Prerequisites

- Rust toolchain (stable edition 2024 or later)
- Cargo package manager

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd helioscope

# Build the project
cargo build --release

# The binary will be available at target/release/helioscope
```

## Configuration

Helioscope uses a TOML configuration file (`helioscope.toml`) to control which probes are active:

```toml
node_id = "uuid or similar (string)"
metrics_collector_addr = "hostname:port (ip address is ok too)"

[probes.sysinfo]
static_info = true   # System static information
cpu = true           # CPU core information
memory = true        # RAM and swap usage
disk = true          # Disk information
network = true       # Network interface data
temperature = true   # Hardware temperature sensors

#[probes.some_other_probe]
```

## Usage

### Basic Usage

```bash
# Run with default configuration
./helioscope

# Or using cargo
cargo run
```

### Output Format

Helioscope outputs structured JSON logs with UTC timestamps:

```
2024-01-01T12:00:00Z INFO Starting helioscope
2024-01-01T12:00:00Z INFO Starting CPU probe
2024-01-01T12:00:00Z INFO Detected 8 CPU cores
```

### Example Output

```
2024-01-01T12:00:00Z INFO Starting memory probe
2024-01-01T12:00:00Z INFO Memory information total_memory_bytes=17179869184 used_memory_bytes=8589934592 memory_usage_percent="50.0"
2024-01-01T12:00:00Z INFO Swap information total_swap_bytes=4294967296 used_swap_bytes=1073741824 swap_usage_percent="25.0"
```

## Project Structure

```
helioscope/
├── src/
│   ├── main.rs              # Main application entry point
│   ├── probes/
│   │   ├── mod.rs           # Probe module declarations
│   │   └── sysinfo/
│   │       ├── mod.rs       # Sysinfo probe module
│   │       ├── cpu.rs       # CPU monitoring implementation
│   │       ├── mem.rs       # Memory monitoring implementation
│   │       └── temp.rs      # Temperature monitoring implementation
├── Cargo.toml              # Rust dependencies and metadata
├── helioscope.toml         # Configuration file
└── LICENSE                 # AGPL v3 license
```

## Dependencies

- `sysinfo`: System information gathering
- `serde`: Configuration serialization/deserialization
- `toml`: Configuration file parsing
- `tracing`: Structured logging framework
- `time`: Timestamp formatting

## Development

### Building for Development

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Performance

The release build is optimized for minimal binary size with:

- Thin LTO (Link Time Optimization)
- Panic abort for smaller binaries
- Symbol stripping with separate debug info
- Single codegen unit for optimal optimization

## Extending Helioscope

### Adding New Probes

1. Create a new module in `src/probes/`
2. Implement probe functions following the existing pattern
3. Add configuration options in `Config` struct
4. Register the probe in the main execution flow

### Example Probe Structure

```rust
pub fn probe_example(sys: &System) {
    use tracing::info;
    info!("Starting example probe");
    // Probe implementation
}
```

## License

Helioscope is licensed under the GNU Affero General Public License v3.0 (AGPL-3.0). See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for:

- New probe implementations
- Configuration enhancements
- Performance improvements
- Documentation updates

## Roadmap

### Client

- [ ] Disk usage monitoring
- [ ] Network interface statistics
- [ ] Process monitoring
- [ ] GPU information
- [ ] Battery status (for laptops)
- [ ] Sending

### Collector

- [ ] Collector API (probably HTTP)
- [ ] Storing metrics as a time series
- [ ] Cloud backup for every hour (configurable)

## Support

For issues, questions, or feature requests, please open an issue on the project repository.
