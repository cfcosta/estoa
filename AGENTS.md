# Repository Guidelines

## Project Structure & Module Organization
Estoa is a Cargo workspace defined in `Cargo.toml`, with each crate under `crates/`. Core runtime code belongs in domain crates (`crdts`, `codec`, `storage`), while CLI entry points live in `cli/src`. Keep library code in each crate’s `src/`, integration or property tests in `tests/`, and share test fixtures through crate-local modules rather than the workspace root. When adding new features, co-locate modules by domain and expose only the minimal public surface via targeted `pub use` exports.

## Development Environment
Enter the pinned toolchain with `nix develop` to load Rust, cargo-nextest, and helper scripts. Use this shell for consistency across platforms. Run editors or language servers inside the shell so they inherit the same toolchain and workspace metadata.

## Build, Test, and Development Commands
- `cargo clippy --all --all-targets --all-features`: lint every crate and target before submitting changes.
- `cargo fmt --all`: format according to `rustfmt.toml` (4-space indent, 80-column width, grouped imports).
- `cargo nextest run`: execute the full test suite quickly; fall back to `cargo test --workspace` if Nextest is unavailable.

## Coding Style & Naming Conventions
Follow Rust defaults: `snake_case` for modules and functions, `CamelCase` for types, concise enums mirroring CRDT operations. Prefer explicit module imports, keeping top-level re-exports lean. Avoid introducing Unicode unless already present. Document non-obvious logic with brief comments instead of line-by-line narration.

## Testing Guidelines
Place focused unit tests inline via `#[cfg(test)] mod tests` and broader behaviour checks in `crates/*/tests/`. Property-based suites should use the `#[proptest]` macro from `estoa-proptest-macros` and seed deterministically via provided RNG helpers when debugging failures. Always run `cargo nextest run` (or `cargo test --workspace`) before opening a pull request.

## Commit & Pull Request Guidelines
Write commit messages as `type(scope): summary`, keeping the imperative summary under 72 characters. Group work into narrowly scoped commits that reflect CRDT or storage boundaries. Pull requests must link related issues, describe behavioural changes, list validation commands (e.g., `cargo fmt --all`, `cargo clippy --all --all-targets --all-features`, `cargo nextest run`), and include screenshots or traces for UI or protocol changes. Keep PRs focused on a single feature or fix and note any follow-up work explicitly.
