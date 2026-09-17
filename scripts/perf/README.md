# Performance scripts

Measurement tools from the 2026-10-01 perf audit (the perf design). Python 3, no dependencies.

- `mkfix.py [dir]`: builds the 2k-file / 5k-commit fixture repository `fix/` and its bare remote `remote.git/`.
  - Build it **outside** any temp directory: on Windows, antivirus scanning makes `%TEMP%` 2-5x slower.
- `bench.py <git2|system> <repo> [label]`: IPC-level timing of every operation against a standalone daemon. It reports the op's own time and the time until the refresh lands. Flags: `--reads-only`, `--tui-sim`.
- `cli.py`: the time of `gitpoulpi` CLI calls with the embedded daemon.
- `spawn.py`: the cost of spawning git (shim vs direct binary on Windows).
- `net.py`: fetch, pull and push timing against the local bare remote.
- `rawgit.py`: the same operations with raw git, as a floor.

## Regression guard (`xtask perf`)

`cargo run -p xtask -- perf [--repo <path>] [--backend git2|system|both] [--no-budget] [--rebuild]` is the cross-platform gate (plan S9; no Python needed). It builds a 2k-file / 5k-commit fixture in `target/perf/fixture` (outside the temp dir), runs an in-process daemon per backend, times each operation over `gitpoulpi-ipc`, counts the `git` processes it spawned (`gitpoulpi_cli_backend::total_spawn_count`), prints the table and a "prev" column, writes `target/perf/<date>.json`, and exits 1 when an operation exceeds its budget. Spawn budgets are exact counts; time budgets are `mult * floor + slack` where the floor is the median `git --version` time on the machine. `--repo` measures reads only and never mutates. The budget table is `budget()` in `xtask/src/perf.rs`. Use `--release`-built numbers when comparing absolute times.

The deterministic half runs in the normal test suite: `crates/gitpoulpi-git2-backend/tests/spawn_budget.rs` (per backend op) and `crates/gitpoulpi-daemon/tests/spawn_budget.rs` (per daemon request), using `gitpoulpi_testkit::SpawnBudget`.

The binary is `target/release/gitpoulpi[.exe]`, or the `GITPOULPI_EXE` environment variable. Generated output (`cfg*/`, `tmp-*/`, `fix/`, `remote.git/`, `*.log`) is gitignored.

The numbers are noisy (process spawn and antivirus cost). Compare ratios between runs on the same machine, never absolute values across machines.
