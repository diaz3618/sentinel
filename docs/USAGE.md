# Sentinel Documentation

## Overview
Sentinel is a Rust-based daemon and CLI tool for protecting Linux VMs from memory exhaustion. It monitors system memory, maintains a configurable reserve, and takes staged actions before the system hits Out-Of-Memory (OOM).

## Components
- **Daemon (`sentinel`)**: Runs in the background, monitors memory, and acts to prevent OOM.
- **CLI (`sentinelctl`)**: Provides status, top processes, config management, logs, reserve control, and cgroup slice inspection.
- **Config**: TOML file at `/etc/memsentinel.toml` (see example below).

## Installation
### Quick Install
```bash
sudo ./install.sh
```
This script installs Rust, dependencies, builds binaries, copies config, and sets up systemd service.

**Reinstalling**: The script detects existing installations and offers a clean reinstall option that:
- Stops and disables the service
- Removes all binaries and systemd files
- Optionally removes the config file
- Cleans build artifacts
- Performs a fresh installation

### Manual Install
Build and deploy binaries manually if needed. See README for details.

## Configuration
### Creating Configuration File
Use the interactive wizard to create your config:
```bash
sentinelctl config init
```

Example interactive session:
```
🔧 Sentinel Configuration Wizard
================================

📊 Memory Reserve Configuration
Reserve memory (MB) [512]: 1024

⚠️  Threshold Configuration
Soft threshold (% available) [15]: 20
Hard threshold (% available) [5]: 10

🎯 Action Mode
  - kill: Terminate processes aggressively
  - slow: Pause processes with SIGSTOP
  - hybrid: Use both strategies
Mode (kill/slow/hybrid) [hybrid]: hybrid

⏱️  Monitoring Configuration
Scan interval (seconds) [2]: 5
Max actions per minute [4]: 3

🛡️  Protected Processes
Current: ["sshd", "systemd", "sentinel"]
Add more protected process names (comma-separated, or press Enter to skip): nginx,postgres

✅ Configuration written to memsentinel.toml
💡 Run: sudo mv memsentinel.toml /etc/memsentinel.toml
```

Alternatively, copy the example config:
```bash
sudo cp packaging/sentinel.example.toml /etc/memsentinel.toml
```

### Configuration Options
Example config (`packaging/sentinel.example.toml`):
```toml
reserve_mb = 512
soft_threshold_pct = 15
hard_threshold_pct = 5
mode = "hybrid"                 # "kill" | "slow" | "hybrid"
scan_interval_sec = 2
exclude_names = ["sshd", "systemd", "sentinel"]
max_actions_per_min = 4

[cli]
color = "auto"                  # auto|always|never
unicode = "auto"                # auto|always|never
table_max_width = 120
```

## Usage
### Daemon
Start the daemon:
```bash
sudo sentinel
```
Reload config:
```bash
sudo kill -SIGHUP $(pidof sentinel)
```

### CLI
Create config interactively:
```bash
sentinelctl config init
```
Show status:
```bash
sentinelctl status
```
Show top memory processes:
```bash
sentinelctl top --limit 10
```
Simulate actions:
```bash
sentinelctl simulate soft --dry-run
```
Get/set config:
```bash
sentinelctl config get reserve_mb
sentinelctl config set reserve_mb 1024
```
Show logs:
```bash
sentinelctl logs --since "1h" --follow
```
Control reserve:
```bash
sentinelctl reserve hold
sentinelctl reserve release
sentinelctl reserve rebuild
```
Inspect cgroup slices:
```bash
sentinelctl slices --tree
```

## Systemd Integration
Example unit and slice files are in `packaging/systemd/`. Enable and start the service:
```bash
sudo systemctl enable sentinel
sudo systemctl start sentinel
```

## Examples
### Protect SSH from OOM
Configure `exclude_names = ["sshd"]` to ensure SSH is not killed under memory pressure.

### Custom Reserve
Set `reserve_mb = 1024` in config to keep 1GB always available.

### Hybrid Mode
Set `mode = "hybrid"` to use both slow and kill actions based on pressure.

## Troubleshooting
- Check logs with `sentinelctl logs`.
- Ensure config is valid TOML.
- Use `sentinelctl status` to verify daemon is running.

## Contributing
See `docs/ARCHITECTURE.md` and `docs/UX.md` for design and UX details.

## License
MIT
