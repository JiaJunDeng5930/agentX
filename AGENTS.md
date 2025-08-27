# Repository Guidelines

This repo hosts the Codex CLI (Rust) workspace used to run the agent, TUI, and core services. Use this guide to build, test, and contribute consistently.

## Project Structure & Module Organization
- `codex-rs/` (Rust workspace)
  - `core/`: conversation engine, tools (conv_*, shell, apply_patch), providers, session/task logic.
  - `tui/`: terminal UI (ratatui), history rendering, status widgets, input/composer.
  - `login/`, `apply-patch/`, and other crates: focused functionality consumed by `core`/`tui`.
- Tests live alongside sources (e.g., `codex-rs/*/src/**/*_tests.rs`) plus snapshot/fixture dirs (e.g., `tui/src/snapshots`, `tui/tests/fixtures/`).

Before finalizing a change to `codex-rs`, run `just fmt` (in `codex-rs` directory) to format the code and `just fix -p <project>` (in `codex-rs` directory) to fix any linter issues in the code. Additionally, run the tests:
1. Run the test for the specific project that was changed. For example, if changes were made in `codex-rs/tui`, run `cargo test -p codex-tui`.
2. Once those pass, if any changes were made in common, core, or protocol, run the complete test suite with `cargo test --all-features`.
When running interactively, ask the user before running these commands to finalize.

## Coding Style & Naming Conventions
- Rust style with rustfmt (4-space indentation, trailing commas, stable config). Run `just fmt` before commits.
- Naming: `snake_case` for functions/vars/modules; `UpperCamelCase` for types/traits; `SCREAMING_SNAKE_CASE` for consts.
- Prefer small, focused modules; keep public surface minimal (`pub(crate)` where possible).
- Clippy (if enabled): `cargo clippy --workspace --all-features -D warnings`.

## Testing Guidelines
- Use Rust’s built-in test harness. Co-locate unit tests next to impls; add integration tests under each crate’s `tests/` where appropriate.
- Snapshot/fixture tests exist in TUI (e.g., `tui/src/snapshots`, `tui/tests/fixtures/`). Update snapshots intentionally and review diffs.
- Keep tests deterministic (no network, no time-based flakes). Prefer fixture logs and controlled inputs.

## Commit & Pull Request Guidelines
- Commit style: Conventional Commits (e.g., `feat(tui): …`, `fix(core): …`, `docs: …`, `chore: …`). Scope with crate names where helpful.
- Keep commits small and descriptive; explain “why” in the body if not obvious.
- PRs: include a clear summary, rationale, and testing notes (commands run, screenshots/logs if UI or behavior changed). Link issues when applicable.

## Security & Configuration Tips
- Do not commit secrets, tokens, or local paths outside the workspace. Tests must not rely on external network access.
- Shell/tooling runs can be sandboxed/approved at runtime; keep defaults safe and document when escalating permissions is required.

## Architecture Notes
- Core event loop streams model responses; tool calls must be paired with tool outputs in the same turn. Preserve this invariant when refactoring (see conv_* tools and auto-compact logic). 
