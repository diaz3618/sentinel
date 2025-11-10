# Sentinel Tuning Guide

← [Back to README](../README.md) | [Usage Guide](USAGE.md) | [Architecture](ARCHITECTURE.md)

This guide covers advanced configuration for different workloads and system configurations.

**Contents:**
- [System Requirements](#system-requirements)
- [Memory Thresholds](#memory-thresholds)
- [Swap Configuration](#swap-configuration)
- [Workload-Specific Tuning](#workload-specific-tuning)
- [Protected Units](#protected-units)
- [Troubleshooting](#troubleshooting)

## System Requirements

- **Linux kernel ≥4.20** for PSI support (recommended)
- **cgroup v2** for full slice awareness (Ubuntu 22.04+, Fedora 31+)
- **systemd** for service management
- **Root privileges** for daemon operation

## Memory Thresholds

### Traditional Thresholds (`soft_threshold_pct`, `hard_threshold_pct`)

Based on `/proc/meminfo` MemAvailable percentage:

- **Soft threshold**: Release reserve balloon, begin monitoring
- **Hard threshold**: Take action on processes

**Recommendations:**
- With swap: `soft=15%`, `hard=5%`
- Without swap: `soft=20%`, `hard=10%`
- Low-memory systems (<4GB): increase both by 5%

### PSI Thresholds (`psi_soft_pct`, `psi_hard_pct`)

Based on pressure stall percentage (avg10 window):

- **PSI soft**: Some tasks experiencing memory stalls
- **PSI hard**: Severe stalls indicating imminent OOM

**Recommendations:**
- With swap: `psi_soft=10.0`, `psi_hard=30.0`
- Without swap: `psi_soft=8.0`, `psi_hard=20.0` (act faster)
- Desktop/interactive: `psi_soft=15.0`, `psi_hard=40.0` (more tolerant)

## Swap Configuration

### With Swap Available

Advantages:
- More time to react before OOM
- Better buffer for bursts
- Can use moderate thresholds

Settings:
```toml
reserve_mb = 512
soft_threshold_pct = 15
hard_threshold_pct = 5
psi_soft_pct = 10.0
psi_hard_pct = 30.0
```

### Without Swap (Common in Containers/VMs)

Challenges:
- Faster progression to OOM
- No buffer for memory spikes
- Need aggressive monitoring

Settings:
```toml
reserve_mb = 1024          # Larger reserve
soft_threshold_pct = 20    # Earlier detection
hard_threshold_pct = 10    # More headroom
psi_soft_pct = 8.0         # React to pressure faster
psi_hard_pct = 20.0        # Lower tolerance
scan_interval_sec = 1      # Check more frequently
```

### With zram

zram provides compressed swap in RAM:
- Behaves like swap but uses CPU for compression
- Better than no swap, not as good as disk swap
- Can use "with swap" settings but monitor CPU

## Workload-Specific Tuning

### Desktop/Workstation

Goals: Protect interactivity, allow recovery

```toml
reserve_mb = 512
soft_threshold_pct = 20
hard_threshold_pct = 10
mode = "slow"
psi_enabled = true
psi_soft_pct = 15.0
psi_hard_pct = 40.0
exclude_names = ["Xorg", "gdm", "sshd", "systemd", "sentinel"]
protected_units = ["display-manager.service", "sshd.service"]
```

### Server

Goals: Protect critical services, predictable behavior

```toml
reserve_mb = 1024
soft_threshold_pct = 15
hard_threshold_pct = 5
mode = "slow"
max_actions_per_min = 2
protected_units = [
    "sshd.service",
    "nginx.service",
    "postgresql.service",
    "docker.service"
]
```

### Container/VM Host

Goals: Protect infrastructure, allow container management

```toml
reserve_mb = 2048
soft_threshold_pct = 15
hard_threshold_pct = 8
exclude_names = ["dockerd", "containerd", "kubelet"]
protected_units = [
    "docker.service",
    "containerd.service",
    "kubelet.service"
]
```

## Operating Modes

### `mode = "watch"`
- Monitor only, no actions
- Use for: testing, observability, dry runs

### `mode = "slow"`
- Send SIGSTOP to pause processes
- Use for: general protection, allowing recovery
- **Recommended for most setups**

### `mode = "kill"`
- Send SIGKILL immediately
- Use for: aggressive protection, containers
- Risk: data loss, incomplete transactions

### `mode = "hybrid"`
- SIGSTOP first, SIGKILL if pressure persists
- Use for: balanced approach
- Best for: production systems with monitoring

## Protected Units

Always protect:
- `sshd.service` / `ssh.service` - Remote access
- `sentinel.service` - Self-protection
- `systemd` - Init system

Workload-specific protection:
- Databases: `postgresql.service`, `mysql.service`
- Web servers: `nginx.service`, `apache2.service`
- Container runtime: `docker.service`, `containerd.service`
- Display: `gdm.service`, `lightdm.service`

## Rate Limiting

`max_actions_per_min` prevents thrashing:

- Too low: Slow to respond to leaks
- Too high: May kill too many processes

**Recommendations:**
- Stable workloads: `2-3`
- Bursty workloads: `4-5`
- Fast leaks: `5-10` (with monitoring)

## Checking PSI Availability

```bash
# Check if PSI is available
cat /proc/pressure/memory

# Example output:
# some avg10=0.00 avg60=0.05 avg300=0.10 total=12345
# full avg10=0.00 avg60=0.00 avg300=0.00 total=1234
```

If not available:
- Kernel < 4.20
- Set `psi_enabled = false` in config
- Sentinel will fall back to meminfo-only mode

## Cgroup v2 Detection

```bash
# Check cgroup version
mount | grep cgroup

# cgroup2 (unified hierarchy):
# cgroup2 on /sys/fs/cgroup type cgroup2

# Legacy cgroup v1:
# cgroup on /sys/fs/cgroup/... type cgroup
```

For cgroup v1: Slice detection still works but may be less accurate.

## Troubleshooting

### Sentinel not detecting pressure

- Check PSI values: `cat /proc/pressure/memory`
- Verify thresholds aren't too high
- Check `journalctl -u sentinel -f` for logs

### Protected process killed

- Add to `protected_units` in config
- Or add to `exclude_names` for name-based exclusion
- Reload config: `sudo systemctl reload sentinel` or `kill -HUP <pid>`

### Too aggressive killing

- Increase thresholds
- Lower `max_actions_per_min`
- Change to `mode = "slow"` instead of kill
- Increase `reserve_mb` for more buffer

### Not aggressive enough

- Decrease thresholds (especially PSI)
- Increase `max_actions_per_min`
- Reduce `scan_interval_sec`
- Check if swap is masking issues

## Monitoring

Check daemon logs:
```bash
journalctl -u sentinel -f
```

Key log fields:
- `avail_pct`: Current available memory percentage
- `psi_avg10`: Current PSI pressure (10s window)
- `badness`: Process badness score
- `slice`: Cgroup slice of target process

## Best Practices

1. **Start with defaults** - They work for most systems
2. **Use provided profiles** - See `packaging/*.toml` examples
3. **Enable PSI** - Kernel ≥4.20 provides better detection
4. **Protect critical services** - Always protect SSH and infrastructure
5. **Monitor first** - Use `mode = "watch"` initially
6. **Tune iteratively** - Adjust based on logs and behavior
7. **Test with stress** - Use `stress-ng` to validate settings
