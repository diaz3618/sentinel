
# Sentinel CLI UX Guide

## Output
- Keep it short and clear. Show summaries by default; use flags for details.
- Colors: 
    - `green` = healthy
    - `yellow` = warning
    - `red` = critical
- Falls back gracefully if the terminal can't show color, or the user disables it.
- ASCII tables are default. Unicode tables show up if the terminal supports it or with `--unicode`.
- Symbols:
    - `●` (healthy)
    - `▲` (warning)
    - `✖` (critical).
    - If Unicode isn't available, use `OK`, `WARN`, `CRIT`.

## Table Style
- All tables use comfy-table.
- Borders are single-line for readability and simplicity.
- Numbers are right-aligned. Long names get cut off with `...`.
- Columns:
    - PID
    - NAME
    - RSS
    - NICE
    - IO-CLASS
    - CGROUP (when relevant).

## Examples

### Status

| State     | MemAvailable | Total     | Used      |
|-----------|--------------|-----------|-----------|
| ● Healthy | 48%          | 31.33 MiB | 16.42 MiB |

### Top

| PID     | NAME    | RSS      |
|---------|---------|----------|
| 2312854 | code    | 1.10 GiB |
| ...     | ...     | ...      |

### Simulate

	Simulating soft threshold response (dry-run)

## Flags
- `--color auto|always|never`, `--no-color`
- `--unicode` / `--no-unicode` (default: auto)
- `-v/--verbose`, `-q/--quiet`
- Paging: auto-pipe to pager if output is long and TTY, disable with `--no-pager`
