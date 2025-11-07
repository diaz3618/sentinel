

# Badges
![License](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/rust-1.77%2B-orange)
![Docs](https://img.shields.io/badge/docs-available-brightgreen)


# Sentinel: Linux Memory Guard Daemon & CLI

Sentinel is a Rust daemon and CLI tool for preemptively protecting Linux VMs from memory exhaustion. It monitors system memory, maintains a configurable reserve balloon, and takes staged actions before the system hits OOM.

This exists because I got tired of abusing my VMs and getting locked out until it resolved itself or I force restart it (not ideal). Sentinel keeps a memory reserve so critical services like SSH stay available even if the system runs out of RAM. Simple and effective 👍.

## Features
- **Daemon (`sentinel`)**: Monitors `/proc/meminfo`, maintains reserve, classifies pressure, and acts before OOM (slow, stop, kill, cgroup integration).
- **CLI (`sentinelctl`)**: Status, top processes, simulate actions, config get/set, logs, reserve control, cgroup slice inspection.
- **Config**: TOML config at `/etc/memsentinel.toml` (see `packaging/sentinel.example.toml`). Live reload on SIGHUP.
- **Systemd**: Example unit and slice files in `packaging/systemd/` for secure deployment.
- **Tests**: Unit and integration tests for all core logic and CLI rendering.

## Usage

### Dev (testing)
```bash
# Run CLI in dev mode
cargo run --bin sentinelctl status
cargo run --bin sentinelctl top --limit 5
cargo run --bin sentinelctl simulate soft --dry-run
cargo run --bin sentinelctl config get reserve_mb
cargo run --bin sentinelctl reserve hold

# Run daemon in dev mode
cargo run --bin sentinel
```

### Production
```bash
# Build release binaries
cargo build --release --workspace

# Run CLI
./target/release/sentinelctl status
./target/release/sentinelctl top --limit 5
./target/release/sentinelctl simulate soft --dry-run
./target/release/sentinelctl config get reserve_mb
./target/release/sentinelctl reserve hold

# Run daemon
./target/release/sentinel
```

## CLI Commands
- `status` — Show current memory, reserve, thresholds, and pressure state
- `top` — List top RSS processes (with exclusions)
- `simulate` — Show what actions would be taken at soft/hard threshold
- `config` — Get/set config keys (TOML)
- `logs` — Stream recent actions (journald, stub)
- `reserve` — Hold/release/rebuild reserve balloon
- `slices` — Inspect cgroup v2 slices (stub)

## Systemd Integration
- See `packaging/systemd/sentinel.service` and `sentinel.slice` for secure deployment
- Hardening: `NoNewPrivileges`, `ProtectSystem=strict`, `OOMScoreAdjust=-1000`, etc.

## Documentation
- [UX Guidelines](docs/UX.md)
- [Architecture](docs/ARCHITECTURE.md)

## Example Config
See `packaging/sentinel.example.toml` for all options and defaults.

## Contributing

Not much to contribute to, but if you have an idea or find a bug, go for it.

## License

This project is licensed under the MIT License.

*Do whatever you want with it*