# Repository Guidelines

## Project Structure & Module Organization
Estoa is a Cargo workspace defined in `Cargo.toml`, with each component living under `crates/`. Core runtime logic belongs in domain crates such as `crdts`, `codec`, and `storage`; CLI entry points sit in `cli/src`, and reusable testing helpers live in `proptest` and `proptest-macros`. Use `src/` for library code and `tests/` for integration or property suites; keep shared fixtures in crate-specific modules instead of the workspace root.

## Build, Test, and Development Commands
- `cargo clippy --all --all-targets --all-features` to check builds every time.
- `cargo fmt --all` to format all the files, which you should do every time.
- `cargo nextest run` uses the optional Nextest runner installed in the dev shell for faster iteration.
Enter the Nix development shell with `nix develop` to load the pinned toolchain and helper utilities.

## Coding Style & Naming Conventions
Formatting is governed by `rustfmt.toml`, enforcing 4-space indentation, 80-character width, and grouped imports; run `cargo fmt --all` or `nix fmt` before pushing. Favor `snake_case` modules, `CamelCase` types, and concise enums that mirror CRDT concepts. Co-locate feature-specific code inside the owning crate and expose only intentional APIs via minimal `pub use` statements.

## Testing Guidelines
Prefer focused unit tests inside `src/` modules (`mod tests { ... }`) and broader behaviour checks in `crates/*/tests/`. Property tests should use the `#[proptest]` macro from `estoa-proptest-macros` to cover randomised scenarios; seed determinism via `rng()` helpers when debugging failures. When adding new CRDT operations, pair unit coverage with a regression property asserting convergence.

## Commit & Pull Request Guidelines
Follow the existing `type(scope): summary` pattern, e.g. `feat(estoa-proptest): add random generator`, and compose imperative, present-tense summaries under 72 characters. Each pull request should link to any relevant issue, note behavioural changes, and describe the validation steps (`cargo test --workspace`, formatters run) in the description. Include screenshots or protocol traces when UI or networking behaviour changes, and keep PRs scoped to a single feature or fix.
