# Open time: measuring and what was fixed

Owner complaints: opening a repository is slow, and opening a worktree in a
new tab (double-click a Worktrees row -> `tabs::open_worktree_in_tab` ->
`SwitcherAction::Open`) is very slow.

## How a repo / worktree tab opens

`SwitcherAction::Open` only records the tab and sets `switch_to`; the outer
loop (`lib.rs::run_windows`) ends the window and starts `run_in_repo` for the
new `RepoRef`. **The daemon is reused**: one embedded daemon per *session*
serves every repository (worktrees included, each keyed by its own root), so
a worktree tab costs one TCP reconnect, not a daemon spawn. The window paints
a "Loading repository..." skeleton immediately (`still_loading`), then the
link's first `RefreshOutcome` fills it in. The hosting worker is its own
thread; the avatar service likewise.

So the perceived wait is: link connect + `Hello` + **the first refresh**
(status, log, refs, stashes, worktrees, conflicts, remotes, rebase_state).
That refresh also runs every 2 s, and on every `RepoChanged` event.

## Measuring

```text
# per-query and end-to-end numbers, this checkout by default
cargo test -p gitpoulpi-tui --lib --release open_time_bench -- --ignored --nocapture
# other repository
$env:GITPOULPI_BENCH_REPO = 'C:\path\to\repo'
# trace a real window (file, never the terminal)
$env:GITPOULPI_TIMING = 1; gitpoulpi-tui ; Get-Content $env:TEMP\gitpoulpi-timing.log
```

`GITPOULPI_TIMING` (existing switch, `gitpoulpi-ipc::timing`) now also logs
`tui-open` lines: window opening, daemon link started, setup done, first frame
drawn, first refresh applied (each `+ms` since the window began opening), and
one `refresh ... all eight queries in` line per refresh, and a `tui-stall`
line for every frame-loop phase (`tick`, `draw`, `input`, `apply_outcomes`)
that held the input thread 40 ms or more. (`input` at ~40 ms while something
animates is the drain's frame-paced poll, not a stall.)

`open_time_bench.rs` also holds a loose regression guard,
`first_outcome_on_a_small_fixture_is_prompt` (10 s ceiling, runs in the normal
suite).

## Hot spots found (this repo, Windows, libgit2 backend)

| step | cost (unloaded machine) | why |
|---|---|---|
| `do_refresh` = 8 requests awaited **one after another** | sum of all = ~740 ms | client awaited each reply before sending the next, although the daemon serves reads concurrently |
| `conflicts` | 100-300 ms | `git diff --diff-filter=U` process spawn, on every refresh, even with no conflict |
| `rebase_state` | 60-160 ms | `git rev-parse --absolute-git-dir` spawn to find a directory libgit2 already knows |
| `worktrees` | 30-300 ms, grows with worktree count | one `Repository::open` per worktree, in sequence (this repo has many agent worktrees) |
| `status` | 150-430 ms | libgit2 status walk (untracked recursion) - unchanged |
| `log` (5000, all refs) | 130-400 ms, about the same for 200 | TIME+TOPOLOGICAL revwalk visits the whole history whatever the limit - unchanged |
| hosting first request | off-thread already | but started immediately, competing with the first refresh |

Not a cost: repo discovery (`.git` checks, a few file reads), daemon
spawn for a worktree tab (reused), settings resolution, image/avatar caches
(lazy), `HostingUi::new` (thread + small file reads).

## Fixes

1. `daemon_link::do_refresh`: the eight requests go out together
   (`tokio::join!`); wall time is the slowest query, not the sum.
2. `Git2Backend::conflicts` / `rebase_state`: answered natively when the index
   has no unmerged entries / no `rebase-merge`/`rebase-apply` dir (the exact
   test the system backend makes first); anything else still goes to `git`.
3. `reads::worktrees`: the per-worktree branch lookups run on scoped threads.
4. Hosting network requests wait 1.5 s after a window opens (`HOSTING_OPEN_DELAY`),
   so the first screen is not competing with them.
5. `tui-open` tracing (above).

## 2026-10-06: the Windows open "freeze"

Maintainer report: the repository shows quickly, then the first fetch and the
issues/MR refresh block the interface. Traced with `tui-stall` on Windows:
neither the fetch nor the hosting refresh runs on the input thread (both
were already off it). Two real causes:

1. **9-12 s before the first frame, on the input thread**: the "Open in"
   detection (`tab_dir::detected`) asked `on_path` about ~40 programs, each
   probing every `PATH` directory x every `PATHEXT` extension -- tens of
   thousands of file checks, slow under Defender. Now `repo_menu::path_lookup`
   lists each `PATH` directory once and answers from a set. First frame:
   ~0.2 s.
2. **Any mutation during a fetch froze every pane for the whole fetch**: a
   fetch/push held the daemon's per-repo lock *shared*; a stage (or checkout,
   commit, ...) then waited as a writer, and because the lock is fair every
   read after it -- refreshes, commit files, diffs, from every client --
   queued behind that writer. Open-time fetches take 5-20 s here. Transfers
   now hold no repo lock, only a per-repo transfer lock that orders them
   among themselves and with the exclusive network operations (`server.rs`
   `TransferLocks`; `tests/shared_transfers.rs`).

## Numbers

Same process, interleaved, median of 5 (`refresh_shape_ab` in the bench), the
old shape being serial queries with `git`-spawning conflicts/rebase_state:

| | before | after |
|---|---|---|
| one full refresh (this repo, ~500 commits, several agent worktrees) | 742 ms | 168 ms |

End to end, link start to first `RefreshOutcome` (unloaded machine):

| | before | after |
|---|---|---|
| first open, daemon spawned | 1.5 s | 0.38 s |
| second open, daemon warm | 0.88 s | 0.58 s (noisy; 0.3-0.6 s across runs) |
| worktree tab, same daemon | not measured | 0.38 s / 0.31 s |

The "before" end-to-end figures were taken on an idle machine, the "after"
ones with the machine only lightly loaded; the in-process A/B row is the
trustworthy comparison. Under load (other builds running) everything scales
5-10x, which is likely part of the maintainer's experience too.

## Not done / ideas

- Repositories with very long history: the revwalk visits all of it before the
  first page is returned. A first-paint page (small limit, then the full walk)
  would need a non-topological cheap walk or a persistent commit-graph cache;
  left out because it changes ordering guarantees the graph lanes rely on.
- `status` could skip untracked recursion on first paint and fill it in
  afterwards; not attempted (behavior change in the WIP row).
- The window is rebuilt on each tab switch (`run_windows`); keeping the last
  `RefreshOutcome` per tab for an instant stale-while-revalidate paint is the
  next lever if 0.3 s is still too long.

## Regression guard

`cargo run -p xtask -- perf` (see `scripts/perf/README.md`) measures every operation's time and `git` spawn count against budgets and fails on a regression. The spawn-count tests (`tests/spawn_budget.rs` in `gitpoulpi-git2-backend` and `gitpoulpi-daemon`) run in the normal suite.
