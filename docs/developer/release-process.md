# Release process

The jobs, secrets and variables of the release workflow, the dry run and the first winget submission are in [releasing.md](releasing.md).

Not yet defined - no releases exist. This will cover version bumping across the workspace, `xtask`-driven packaging (Windows installer/winget, Linux deb/rpm/AUR or static binary, Homebrew best-effort), and changelog generation. That is Phase 3, "Hardening & performance" - see `../roadmap.md`.

## What is already fixed

- **Official build**: `scripts/build-opti.sh` / `.ps1` (the `dist` profile, everything on: the default `full` features). Output in `target/dist/`.
- **Before tagging**: `cargo run -p xtask -- features` (the full feature matrix; about as long as a full test run) must pass, on top of `pwsh scripts/check.ps1 -Full` (fmt, clippy, fast and PTY test tiers; Windows tests are local, not in CI). Day to day, `cargo run -p xtask -- features --quick` (or `pwsh scripts/check.ps1 -Features`) is enough.
- **Custom builds are not release artifacts.** `build-custom` writes to `target/custom/`, never over `target/dist/`; a release must never be built from a feature subset. Each custom build ships a `BUILD-FEATURES.txt` and reports its features in `--version` and `daemon status`, so a support report shows which build it came from.
- **Sizes** of the official and the slimmed builds are tracked in `binary-size.md`; re-measure there after a large dependency change.
