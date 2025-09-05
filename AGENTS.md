# Repository Guidelines

This repo hosts the Codex CLI (Rust) workspace used to run the agent, TUI, and core services. Use this guide to build, test, and contribute consistently.

## Project Structure & Module Organization
- `codex-rs/` (Rust workspace)
  - `core/`: conversation engine, tools (conv_*, shell, apply_patch), providers, session/task logic.
  - `tui/`: terminal UI (ratatui), history rendering, status widgets, input/composer.
  - `login/`, `apply-patch/`, and other crates: focused functionality consumed by `core`/`tui`.
- Tests live alongside sources (e.g., `codex-rs/*/src/**/*_tests.rs`) plus snapshot/fixture dirs (e.g., `tui/src/snapshots`, `tui/tests/fixtures/`).

Run `just fmt` (in `codex-rs` directory) automatically after making Rust code changes; do not ask for approval to run it. Before finalizing a change to `codex-rs`, run `just fix -p <project>` (in `codex-rs` directory) to fix any linter issues in the code. Prefer scoping with `-p` to avoid slow workspace‑wide Clippy builds; only run `just fix` without `-p` if you changed shared crates. Additionally, run the tests:
1. Run the test for the specific project that was changed. For example, if changes were made in `codex-rs/tui`, run `cargo test -p codex-tui`.
2. Once those pass, if any changes were made in common, core, or protocol, run the complete test suite with `cargo test --all-features`.
When running interactively, ask the user before running `just fix` to finalize. `just fmt` does not require approval. project-specific or individual tests can be run without asking the user, but do ask the user before running the complete test suite.

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

### TUI Styling (ratatui)
- Prefer Stylize helpers: use "text".dim(), .bold(), .cyan(), .italic(), .underlined() instead of manual Style where possible.
- Prefer simple conversions: use "text".into() for spans and vec![…].into() for lines; when inference is ambiguous (e.g., Paragraph::new/Cell::from), use Line::from(spans) or Span::from(text).
- Computed styles: if the Style is computed at runtime, using `Span::styled` is OK (`Span::from(text).set_style(style)` is also acceptable).
- Avoid hardcoded white: do not use `.white()`; prefer the default foreground (no color).
- Chaining: combine helpers by chaining for readability (e.g., url.cyan().underlined()).
- Single items: prefer "text".into(); use Line::from(text) or Span::from(text) only when the target type isn’t obvious from context, or when using .into() would require extra type annotations.
- Building lines: use vec![…].into() to construct a Line when the target type is obvious and no extra type annotations are needed; otherwise use Line::from(vec![…]).
- Avoid churn: don’t refactor between equivalent forms (Span::styled ↔ set_style, Line::from ↔ .into()) without a clear readability or functional gain; follow file‑local conventions and do not introduce type annotations solely to satisfy .into().
- Compactness: prefer the form that stays on one line after rustfmt; if only one of Line::from(vec![…]) or vec![…].into() avoids wrapping, choose that. If both wrap, pick the one with fewer wrapped lines.

## Snapshot tests

This repo uses snapshot tests (via `insta`), especially in `codex-rs/tui`, to validate rendered output. When UI or text output changes intentionally, update the snapshots as follows:

- Run tests to generate any updated snapshots:
  - `cargo test -p codex-tui`
- Check what’s pending:
  - `cargo insta pending-snapshots -p codex-tui`
- Review changes by reading the generated `*.snap.new` files directly in the repo, or preview a specific file:
  - `cargo insta show -p codex-tui path/to/file.snap.new`
- Only if you intend to accept all new snapshots in this crate, run:
  - `cargo insta accept -p codex-tui`

If you don’t have the tool:
- `cargo install cargo-insta`
