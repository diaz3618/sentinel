# Architecture

← [Back to README](../README.md) | [Usage Guide](USAGE.md) | [Tuning Guide](TUNING.md)

## Overview

Workspace crates:
- `core`: config, meminfo, process discovery, policy, reserve management, PSI monitoring, cgroup awareness
- `daemon`: main loop, signal handling, logging wiring, dual-threshold PSI+meminfo decisioning
- `cli`: read-only views, simulate mode, config helpers, pretty printing

Key modules in `core`:
- `config`: loads TOML, supports reload on SIGHUP (daemon)
- `mem`: read /proc/meminfo, compute percentages
- `psi`: read /proc/pressure/memory, parse PSI metrics (some/full avg10/avg60/avg300)
- `cgroups`: parse /proc/[pid]/cgroup, identify systemd slices, protect critical units
- `procinfo`: list processes with badness scoring (RSS, oom_score_adj, cgroup priority)
- `policy`: dual-threshold model (meminfo + PSI), staged actions and rate-limiting
- `actions`: side-effect adapters (signals, nice/ionice), behind traits for testing
- `reserve`: balloon memory management

## Decision Engine

Sentinel uses a **dual-threshold model** combining traditional memory metrics with modern pressure stall information:

1. **Memory Headroom** (reserve balloon): Maintains available memory buffer
2. **PSI Monitoring** (kernel ≥4.20): Detects stalls before severe OOM
3. **Cgroup Awareness**: Prioritizes user workloads over system services

### PSI Integration

Pressure Stall Information (PSI) provides early warning of resource contention:
- `some_avg10`: Percentage of time some tasks were stalled (10s window)
- `full_avg10`: Percentage of time all tasks were stalled (10s window)
- Triggers actions **before** memory exhaustion, not after

### Cgroup-Aware Targeting

Process selection respects systemd slice hierarchy:
- **User slice** (highest priority): Desktop apps, user sessions
- **Machine slice**: VMs and containers
- **System slice** (protected): Critical services (sshd, sentinel, etc.)
- **Init scope** (never touched): PID 1 and essential init processes

Badness score = RSS% + oom_score_adj + cgroup_priority
