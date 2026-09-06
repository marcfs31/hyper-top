# hyper-top

`hyper-top` is a terminal-first system monitor and process dashboard built with Rust, `ratatui`, and `sysinfo`.

It provides:
- live CPU, memory, swap, uptime, and load-average metrics
- searchable and sortable process tables
- direct process termination support
- customizable refresh cadence, themes, and visible columns
- persistent configuration support via JSON or TOML files

## Quick start

From the project root:

```bash
cargo run
```

Or install it as a command globally:

```bash
cargo install --path . --locked
hyper-top --help
```

## Developer runner

To run the project using the included helper script:

```bash
./scripts/dev.sh
```

## Sample config

A sample config is included at:

```bash
./hyper-top.toml.example
```

You can load it with:

```bash
cargo run -- --config ./hyper-top.toml.example
```

or after installation:

```bash
hyper-top --config ./hyper-top.toml.example
```

## Example config file

```toml
refresh_interval_ms = 900
max_processes = 80
theme = "midnight"
show_full_command = true

[[sort_profiles]]
name = "Memory"
sort_mode = "memory"

[[filter_presets]]
name = "postgres"
query = "postgres"

visible_columns = ["pid", "name", "cpu", "memory", "threads"]
```

## Keyboard controls

- Tab: move focus
- j / k or Up / Down: select process
- /: filter by process name or PID
- s: cycle sort mode
- c / m / n / p / t: direct sort by CPU, memory, name, PID, threads
- x: terminate the selected process
- Space: pause/resume telemetry
- r: cycle refresh rate
- T: cycle theme
- [: decrease visible process count
- ]: increase visible process count
- ?: help
- q: quit

## Install as a terminal app

The project is ready for local installation via Cargo:

```bash
cargo install --path . --locked --force
```

This installs a `hyper-top` command on your PATH.

To also expose a desktop-launch entry, use the provided launcher file:

```bash
./dist/hyper-top.desktop
```

A desktop launcher file is included at `dist/hyper-top.desktop` for a terminal-based app entry. If your desktop environment supports `.desktop` launchers, copy it to a suitable location such as `~/.local/share/applications/` and update the `Exec` target if needed.

## Notes

- The app expects a real terminal and does not work like a GUI window in a non-terminal environment.
- The config file can be JSON or TOML; TOML is the easiest for manual editing.
- If a config file is not found, the app falls back to built-in defaults.
