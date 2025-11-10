sentinel - Linux Memory Guard
=============================

[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.77%2B-orange)](https://www.rust-lang.org/)

**A preemptive OOM guard that keeps your SSH session alive.**

Sentinel monitors system memory and intervenes before the kernel's OOM killer freezes your machine. It maintains a memory reserve balloon so critical services like SSH stay responsive when RAM runs out.

Why does this exist? I got tired of my VMs becoming completely unresponsive during memory pressure, having to wait forever or force-restart them. Sentinel solves this by keeping a reserve and taking early action.

**Quick links:** [Installation](#installation) | [Documentation](docs/USAGE.md) | [Configuration](#configuration)

## What does it do

Sentinel checks available memory and swap (plus PSI metrics on modern kernels) multiple times per second. When pressure builds, it:

1. **Soft pressure** (≤15% available): Releases the reserve balloon
2. **Hard pressure** (≤5% available): Selects and terminates the worst offender

The "worst offender" is chosen using a badness score that combines:
- Memory usage (RSS)
- Kernel's OOM score adjustment
- Cgroup priority (prefers killing user apps over system services)

This runs in userspace, not as a kernel module. It's written in Rust with no runtime dependencies.

## Features

- Monitors `/proc/meminfo` and PSI (Pressure Stall Information) on Linux 4.20+
- Maintains a configurable memory reserve (default 512 MB)
- Respects systemd cgroup slices — protects SSH, databases, web servers
- Multiple operating modes: watch, slow, kill, hybrid
- Live config reload (SIGHUP)
- CLI tool for status, top processes, simulation, and config management
- Hardened systemd unit with minimal capabilities

## Requirements

- Linux 4.20+ recommended (for PSI support; works on older kernels without PSI)
- systemd with cgroup v2 (optional, for full slice awareness)
- Rust 1.77+ (for building from source)

## Installation

#### Quick install (recommended)

```bash
sudo ./install.sh
```

The install script will:
- Build release binaries
- Install to `/usr/local/bin`
- Copy config to `/etc/memsentinel.toml`
- Set up systemd service (if available)
- Detect and offer clean reinstall if already present

To uninstall:
```bash
sudo ./uninstall.sh
```

#### Manual build

```bash
cargo build --release --workspace
sudo cp target/release/{sentinel,sentinelctl} /usr/local/bin/
sudo cp memsentinel.toml /etc/
```

## Configuration

Create config interactively:
```bash
sentinelctl config init
```

Or edit `/etc/memsentinel.toml` directly. Here are the key settings:

```toml
reserve_mb = 512              # Memory to keep reserved
soft_threshold_pct = 15       # Release reserve at this level
hard_threshold_pct = 5        # Start killing processes
mode = "slow"                 # slow/kill/hybrid/watch

# PSI thresholds (Linux 4.20+)
psi_enabled = true
psi_soft_pct = 10.0
psi_hard_pct = 30.0

# Protected services (never kill these)
protected_units = [
    "sshd.service",
    "sentinel.service",
]
```

**Sample configs** for different workloads are in `packaging/`:
- `workstation.toml` — Desktop with swap
- `server.toml` — Conservative server settings  
- `no-swap.toml` — Aggressive thresholds for containers
- `container-host.toml` — Docker/k8s host protection

See the [tuning guide](docs/TUNING.md) for detailed explanations.

## Usage

Start the daemon:
```bash
sudo systemctl start sentinel
```

Or run in foreground for testing:
```bash
sudo sentinel
```

Check status:
```bash
sentinelctl status
```

Output:
```
Sentinel — Status
+-----------+--------------+-----------+-----------+
| State     | MemAvailable | Total     | Used      |
+==================================================+
| ● Healthy | 25%          | 31.33 GiB | 23.55 GiB |
+-----------+--------------+-----------+-----------+

PSI Memory Pressure:
  some avg10: 0.00%  avg60: 0.00%  avg300: 0.00%
  full avg10: 0.00%  avg60: 0.00%  avg300: 0.00%
```

More commands:
```bash
sentinelctl top --limit 10        # Show memory hogs
sentinelctl status --json         # Machine-readable output
sentinelctl status --watch        # Live monitoring
sentinelctl simulate soft         # Preview actions
sentinelctl reserve hold          # Manually hold reserve
```

## How it compares

**vs kernel OOM killer**: Acts earlier, keeps system responsive, protects SSH

**vs other userspace OOM daemons**: Unique reserve balloon approach + PSI + cgroup awareness

## Documentation

- **[Usage guide](docs/USAGE.md)** — Examples and workflows
- **[Tuning guide](docs/TUNING.md)** — PSI tuning, swap scenarios, workload configs
- **[Architecture](docs/ARCHITECTURE.md)** — How it works internally

## Why "available" memory and not "free"?

On Linux, "free" memory is normally close to zero because the kernel uses unused RAM for disk caches. These caches can be dropped instantly when needed.

The "available" memory metric (from `/proc/meminfo` MemAvailable) accounts for this. It's the amount of memory that can be allocated without swapping.

Sentinel monitors MemAvailable, not MemFree, for this reason.

## Acknowledgements

Sentinel's design was inspired by ideas from the broader Linux memory management ecosystem:

- [earlyoom](https://github.com/rfjakob/earlyoom) — Simple, effective early OOM daemon
- [systemd-oomd](https://www.freedesktop.org/software/systemd/man/systemd-oomd.service.html) — PSI-based OOM prevention integrated with systemd
- [nohang](https://github.com/hakavlad/nohang) — Configurable OOM handler with diagnostics
- [psi-notify](https://github.com/cdown/psi-notify) — PSI-based user notifications
- [memavaild](https://github.com/hakavlad/memavaild) — Memory headroom management
- [PSI documentation](https://docs.kernel.org/accounting/psi.html) — Kernel pressure stall information


## Contributing

Contributions, ideas, and bug reports welcome.

## License

MIT — see [LICENSE](LICENSE) for details.