# Copilot instructions

## Project overview
This repository contains `hyper-top`, a Rust TUI system monitor built with `ratatui` and `sysinfo`.

## Engineering standards
- Follow the repository’s existing architecture and conventions.
- Prefer surgical fixes over broad rewrites.
- Add focused tests for changed behavior.
- Keep code clear, robust, and easy to maintain.
- Handle failures and edge cases intentionally.

## Release and versioning
- Follow semantic versioning (SemVer).
- Keep package version in `Cargo.toml` consistent with the project release.
- Update `CHANGELOG.md` for user-visible changes.

## Validation
- Run `cargo fmt` before finishing.
- Run the narrowest relevant test command, typically `cargo test` for this repo.
- Do not claim results that were not executed.

## AI agent rules
- Read the target file and nearby context before making code changes.
- State the behavior to fix or implement before editing.
- Do not add unrelated refactors or scope expansion.
- Preserve user changes and avoid destructive edits.
- Favor correctness, performance, and maintainability over cleverness.
