# Contributing

Read the root [CONTRIBUTING.md](../../CONTRIBUTING.md) first: it holds the provenance rule (no code, asset or text taken from another product) and the license of contributions.

## Setup

Requires a Rust stable toolchain (`rustup`) and, for `git2`, a C toolchain (MSVC Build Tools on Windows, a system C compiler on Linux/macOS).

```
cargo build --workspace
cargo test --workspace
```

**Picking the project up cold?** Follow the reading order at the top of the root `CLAUDE.md`.

## Workflow

1. Pick a task from `where-we-are.md` "What to pick up next"; `../roadmap.md` says which phase it belongs to.
2. If it touches `GitBackend`: write the contract test in `gitpoulpi-testkit` first (red), implement `Git2Backend`, then `SystemGitBackend` (green), then wire the frontend surface. See `testing.md`.
3. `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` must all pass before opening a PR.
4. Update `docs/architecture/git-backend-trait.md` if an operation's native-vs-fallback status changed.

## PR checklist

- [ ] `cargo fmt --all --check` clean
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] `cargo test --workspace` passes (Linux, and Windows with `pwsh scripts/check.ps1`: CI is disabled, see `CLAUDE.md`)
- [ ] New `GitBackend` operations have a contract test covering both backends
- [ ] Docs updated if the change affects the capability matrix, config schema, or CLI/MCP surface
- [ ] A TUI change that alters what's on screen was checked against a real
      screenshot, not just a text read - see `visual-verification.md`, and note
      that the naive capture approach photographs the wrong window
- [ ] A TUI fix has a PTY test (`testing.md`)
- [ ] Any new test was *falsified* (delete the fix, confirm the
      test fails) - vacuous tests have shipped here more than once

## Branch / commit conventions

Conventional-commit-style prefixes (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`).
