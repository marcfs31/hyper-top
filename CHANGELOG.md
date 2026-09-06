# Changelog

All notable changes to this project will be documented in this file.

The format is based on Semantic Versioning (SemVer), with the project version kept in sync with `Cargo.toml`.

## [0.6.0] - 2026-09-06
### Added
- Installation and launcher support for running the app as a terminal command or desktop launch entry.
- Sample config file and local developer runner scripts for easier setup and reuse.
- Project documentation covering installation, customization, and quick-start usage.

### Changed
- Expanded the project’s operational readiness for local installs and repeated usage.
- Bumped the package version to `0.6.0` to reflect the installability and developer experience milestone.

## [0.5.0] - 2026-09-06
### Added
- Configuration-driven flexibility for telemetry refresh cadence, view density, and theme selection.
- Runtime customization controls for refresh rate and process display count via keyboard shortcuts.
- More polished status metadata that surfaces the active theme and tuning settings directly in the interface.

### Changed
- The app now exposes a reusable `AppConfig` model for customizing runtime behavior without changing core logic.
- Improved UI affordances for tuning the live monitoring experience while preserving the existing workflow.
- Bumped the package version to `0.5.0` to reflect the customization milestone.

## [0.4.0] - 2026-09-06
### Added
- Comprehensive automated regression tests covering process selection, filtering, sorting, focus toggling, and process metadata handling.
- Dedicated telemetry validation checks for common process status states.

### Changed
- Hardened selection and metadata flows to maintain reliable behavior during process list updates.
- Improved test coverage and reliability around sorting and filtering interactions.
- Bumped the package version to `0.4.0` to reflect the stability and testability milestone.

## [0.3.0] - 2026-09-06
### Added
- New automated tests for filtering, sorting, selection boundaries, and focus cycling.
- Extra guardrails around process metadata visibility and selection state.

### Changed
- Tightened the reliability of app state transitions between filter, sort, and focus modes.
- Bumped the package version to `0.3.0` to reflect the testing and reliability improvements.

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
