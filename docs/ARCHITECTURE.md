# Architecture

Workspace crates:
- `core`: config, meminfo, process discovery, policy, reserve management (traits for side-effects)
- `daemon`: main loop, signal handling, logging wiring
- `cli`: read-only views, simulate mode, config helpers, pretty printing

Key modules in `core`:
- `config`: loads TOML, supports reload on SIGHUP (daemon)
- `mem`: read /proc/meminfo, compute percentages
- `proc`: list processes (RSS, name, pid), exclusions
- `policy`: staged actions and rate-limiting (pure logic; no side effects)
- `actions`: side-effect adapters (signals, nice/ionice), behind traits for testing
- `reserve`: balloon memory management
