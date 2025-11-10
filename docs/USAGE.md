# Sentinel Usage Guide

← [Back to README](../README.md) | [Tuning Guide](TUNING.md) | [Architecture](ARCHITECTURE.md)

## Quick start

1. Install:
   ```bash
   sudo ./install.sh
   ```

2. Configure:
   ```bash
   sentinelctl config init
   ```

3. Start:
   ```bash
   sudo systemctl start sentinel
   ```

4. Check status:
   ```bash
   sentinelctl status
   ```

That's it. Sentinel is now protecting your system.

## Daemon operations

### Using systemd (recommended)

```bash
# Start service
sudo systemctl start sentinel

# Enable on boot
sudo systemctl enable sentinel

# Check status
sudo systemctl status sentinel

# View logs
journalctl -u sentinel -f

# Reload config (SIGHUP)
sudo systemctl reload sentinel
```

### Manual operation

```bash
# Run in foreground
sudo sentinel

# Run in background
sudo sentinel --silent

# Stop daemon
sudo sentinel --stop
```
sudo sentinel --stop
```

Reload config (if running in foreground):
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
## CLI commands

### Check system status
```bash
sentinelctl status
sentinelctl status --json          # Machine-readable output
sentinelctl status --watch         # Live monitoring (refreshes every 2s)
```

### View memory hogs
```bash
sentinelctl top
sentinelctl top --limit 20         # Show more processes
sentinelctl top --json             # JSON output
```

### Simulate pressure response
```bash
sentinelctl simulate soft --dry-run
sentinelctl simulate hard --explain    # Show scoring details
```

### Reserve management
```bash
sentinelctl reserve hold           # Hold reserve balloon
sentinelctl reserve release        # Release reserve
sentinelctl reserve rebuild        # Release and re-hold
```

### Configuration
```bash
sentinelctl config init            # Interactive wizard
sentinelctl config get reserve_mb  # Get specific value
```

## Configuration file

Location: `/etc/memsentinel.toml`

### Basic settings

```toml
reserve_mb = 512                # Memory reserve size
soft_threshold_pct = 15         # Release reserve at this level
hard_threshold_pct = 5          # Start killing processes
mode = "slow"                   # slow/kill/hybrid/watch
scan_interval_sec = 2           # Check frequency
max_actions_per_min = 4         # Rate limiting
```

### PSI settings (Linux 4.20+)

```toml
psi_enabled = true
psi_soft_pct = 10.0             # Soft pressure threshold
psi_hard_pct = 30.0             # Hard pressure threshold
```

### Protection

```toml
# Processes to never kill (by name)
exclude_names = ["sshd", "systemd", "sentinel"]

# Systemd units to protect
protected_units = [
    "sshd.service",
    "sentinel.service",
    "nginx.service",
]
```

### Operating modes

- **watch**: Monitor only, no actions
- **slow**: Send SIGSTOP to pause processes
- **kill**: Send SIGKILL immediately
- **hybrid**: SIGSTOP first, then SIGKILL if needed

## Common scenarios

### Workstation with swap
Use `packaging/workstation.toml`:
- Higher thresholds (more tolerant)
- Protects display manager
- Moderate PSI settings

### Server without swap
Use `packaging/no-swap.toml`:
- Lower thresholds (act faster)
- Larger reserve
- Aggressive PSI settings

### Container host
Use `packaging/container-host.toml`:
- Protects Docker/containerd
- Large reserve
- Allows killing containers

See [TUNING.md](TUNING.md) for detailed explanations.

## Troubleshooting

### Daemon not starting
```bash
# Check logs
journalctl -u sentinel -n 50

# Verify config
sentinelctl config get

# Check if PSI available
cat /proc/pressure/memory
```

### Process still getting killed
- Add to `protected_units` or `exclude_names`
- Reload config: `sudo systemctl reload sentinel`
- Check logs for badness scores

### Too aggressive
- Increase thresholds
- Lower `max_actions_per_min`
- Change mode from `kill` to `slow`