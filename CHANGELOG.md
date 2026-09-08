# Changelog

All notable changes to this project will be documented in this file.

The format is based on Semantic Versioning (SemVer), with the project version kept in sync with `Cargo.toml`.

## [0.10.0] - 2026-09-08
### Added
- Live network throughput monitoring: an aggregate upload/download rate panel in the dashboard's Charts row, plus combined cumulative data total since launch.
- Per-interface network breakdown (name, download rate, upload rate) in the Process Details panel.

## [0.9.3] - 2026-09-06
### Fixed
- Mouse-wheel scrolling now scrolls process details when the Details panel is focused.
- Details scrolling is clamped to the available content so it cannot overshoot into blank space.

## [0.9.2] - 2026-09-06
### Added
- Scrollable process details with arrow keys, `j`/`k`, PageUp/PageDown, and Home/End when the Details panel is focused.

## [0.9.1] - 2026-09-06
### Fixed
- Aligned the process number header with the right-aligned process numbers in the PID column.
- Expanded `.gitignore` for Rust build output, packaging artifacts, editor metadata, local secrets, and coverage files.

## [0.9.0] - 2026-09-06
### Added
- Five modern color palettes: Tokyo Night, Catppuccin, Nord, Dracula, and Gruvbox.

### Changed
- Refined the existing Default, Solarized, and Midnight palettes with improved contrast and accent colors.
- The `T` shortcut now cycles through all eight themes.

## [0.8.2] - 2026-09-06
### Changed
- Added a clearly separated process number beside each PID in the Processes table.
- Bumped the package version to `0.8.2`.

## [0.8.1] - 2026-09-06
### Added
- Hierarchical process tree mode toggled with `e`, with parent-first navigation and visual indentation.

### Fixed
- Replaced ambiguous unavailable parent UID values with `N/A`.
- Help dialog navigation now visibly scrolls with Up/Down, PageUp/PageDown, Home, and End.

## [0.8.0] - 2026-09-06
### Added
- Dedicated input handling module with focused shortcut and interaction tests.

### Changed
- Reduced `main.rs` to application startup, terminal lifecycle, and telemetry orchestration.
- Preserved responsive process navigation, filtering, sorting, focus, limits, help, and expanded-view controls.
- Bumped the package version to `0.8.0` for the architecture and maintainability milestone.

## [0.7.0] - 2026-09-06
### Added
- Automated GitHub release packaging for Linux, macOS, and Windows installation bundles.
- One-click archive generation for each supported platform via a reusable release packaging script.
- A `--version`/`-V` flag so the installed binary reports its release version directly.

### Changed
- Updated the release process to publish OS-specific install artifacts with every version tag.
- Improved installability and distribution workflows for users looking for a packaged binary instead of a source-only install.
- Bumped the package version to `0.7.0` to reflect the release-packaging milestone.

## [0.6.1] - 2026-09-06
### Added
- Desktop launchers for Linux, macOS, and Windows to make the app easy to start from the OS shell or desktop environment.
- Platform-specific launch wrappers alongside the generic terminal launcher assets.

### Changed
- Improved the install-friendly developer experience by providing OS-native entry points.
- Bumped the package version to `0.6.1` to reflect the cross-platform launcher milestone.

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
