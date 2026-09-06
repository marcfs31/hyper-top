# Changelog

All notable changes to this project will be documented in this file.

The format is based on Semantic Versioning (SemVer), with the project version kept in sync with `Cargo.toml`.

## [0.2.0] - 2026-09-06
### Added
- Full-featured TUI process dashboard with richer system health summaries.
- Search/filter support for processes by name, command, and PID.
- Expanded sort modes including CPU, memory, PID, name, and threads.
- Process kill confirmation flow and pause/resume controls.
- Help modal and richer metadata display.
- Additional process metadata: command line, status, thread count, parent PID, swap usage.

### Changed
- Upgraded the app model and telemetry pipeline to carry richer process data.
- Improved input handling and process selection behavior.
- Bumped the package version to `0.2.0` to reflect the full-featured release state.

## [0.1.0] - 2026-09-06
### Added
- Initial `hyper-top` project scaffold.
- Basic process monitor UI, CPU/RAM summaries, and telemetry refresh loop.
- Initial sort/filter behavior and kill support.
