# AGENTS.md

This repository is a Rust TUI monitoring tool. All contributors, including AI assistants, must follow the same standards expected of a professional engineering team.

## Project intent
- Build reliable, maintainable software that is easy to reason about.
- Prefer small, targeted changes over broad rewrites.
- Keep the app interactive, resilient, and testable.

## SemVer policy
- Use semantic versioning for every release.
- Keep the package version aligned with the project state in `Cargo.toml`.
- Record notable user-visible changes in `CHANGELOG.md`.
- Current version: `0.2.0`.

## Required workflow
1. Read the relevant files and tests before editing.
2. Define the intended behavior and acceptance criteria.
3. Keep the fix or feature scoped to the root cause.
4. Add or update focused tests where behavior changes.
5. Run the smallest validation command that checks the changed behavior.
6. Review the diff for regressions before finishing.

## Professional standards
- Keep code readable and idiomatic.
- Handle edge cases and errors explicitly.
- Prefer existing architecture and APIs over new abstractions.
- Preserve unrelated user work and avoid broad refactors.
- Use minimal, justified abstractions; do not add complexity for speculation.
- Never claim validation that was not actually run.

## Rust-specific expectations
- Format code with `cargo fmt` before final validation.
- Prefer compile-safe, idiomatic Rust patterns.
- Avoid unchecked `unwrap` or panic-prone logic in user-facing paths unless unavoidable and justified.
- Keep tests focused and deterministic.

## Delivery expectations
- Provide concise status updates.
- Keep commits logically scoped and explain the intent in the commit message.
- Ensure the final state is working, tested, and ready for review.
