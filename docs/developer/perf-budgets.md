# Perf budgets

CI is disabled, so these are a local pre-merge check. Run both on a release build:

```
cargo run --release -p xtask -- perf --enforce          # daemon, both backends
cargo build --release -p gitpoulpi-tui
cargo run --release -p xtask -- perf --tui --enforce    # the real TUI in a pty, merge editor included
```

Plain `perf --tui` stays report-only (prints the ratios, exit 0); `--enforce` exits 1 on any `OVER`. `perf` has always failed on a regression (`--no-budget` turns that off; `--enforce` is the explicit spelling). The fixture (2,000 files, 5,000 commits) is built under `target/perf/fixture`; delete `target/perf` when done. Both tasks use isolated config dirs and never touch `~/.config/gitpoulpi`.

## Daemon budgets (`xtask perf`, `budget()` in `xtask/src/perf.rs`)

Spawn counts are exact. Time is `mult * floor + slack`, where `floor` is the median of `git --version` on the machine (0.9 ms here), so a slow or antivirus-laden Windows box scales the budget. Since P7 the slack is about 2x the median of three release runs, never under 10 ms (a loaded dev box must not flake a sub-millisecond op).

Baseline: median of 3 runs, release, 2026-10-02, Linux, load average about 5 (not quiet, 32 threads so mostly idle), floor 0.9 ms. Budget is the resulting limit at that floor (ms).

| operation | git2 base | git2 budget | system base | system budget |
|---|---|---|---|---|
| status (clean) | 3.7 | 10 | 4.3 | 14 |
| log 5000 | 43.5 | 80 | 41.1 | 89 |
| log 200 | 18.1 | 40 | 15.8 | 45 |
| log 5000, cold, `log.use_mailmap` off / on / on with a 300-entry `.mailmap` (S16; three asks with different limits so none is cached, median; 2026-10-22, release, load average 13) | 45-49 / 45-49 / 45-53 | 88 | 42-66 / 42-55 / 42-63 | 88 |
| refs | 0.7 | 10 | 5.1 | 14 |
| commit stats x60 (cold) | 26.7 | 55 | 16.5 | 40 |
| commit stats x60 (warm) | 0.1 | 10 | 0.2 | 10 |
| diff files / 1 file (big commit) | 3.0 / 1.0 | 10 | 5.8 / 3.2 | 15 |
| diff files / 1 file (200 dirty) | 2.6 / 1.1 | 10 | 3.5 / 2.3 | 14 |
| stage / unstage file | 6.9 / 4.8 | 15 | 9.8 / 5.6 | 22 |
| stage / unstage all | 7.9 / 5.9 | 18 | 6.8 / 7.6 | 23 |
| stage / unstage file (undo off) | 3.2 / 2.0 | 12 | 6.6 / 3.5 | 17 / 14 |
| stage / unstage all (undo off) | 5.3 / 3.3 | 12 | 4.6 / 4.9 | 15 |
| discard 1 file (undo off) | 3.1 | 13 | 7.6 | 17 |
| discard 50 files | 16.4 | 40 | 22.3 | 45 |
| branch create / delete (journal) | 1.7 / 1.6 | 10 | 2.9 / 5.4 | 14 / 15 |
| journal state | 0.4 | 10 | 1.8 | 14 |
| undo / redo (branch create) | 2.3 / 2.4 | 10 | 6.8 / 6.8 | 18 / 17 |
| undo (branch delete) | 1.3 | 10 | 7.8 | 19 |
| commit count (cold, 5,000 commits) | 3.9-22.7 | 39 (1 spawn) | 3.5-4.5 | 39 (1 spawn) |
| commit count (warm) | 0.4-0.7 | reported | 3-5 (not cached, see below) | reported |

### Outside-change snapshot (`outside_rows` in `xtask/src/perf.rs`, `tests/spawn_budget_outside.rs`)

The daemon diffs the repository's refs against the last snapshot it kept (`.git/gitpoulpi/refs-snapshot.json`) to record what a shell or an AI agent did (the outside-operations design). It reads refs, HEAD, the stash list (only when `refs/stash` exists) and the worktree list (only when linked worktrees exist), then the reflog of each changed branch for the reason. A fingerprint of the ref storage (HEAD, `packed-refs`, the directories of `refs/`, the stash reflog) skips the read when nothing moved; it is not trusted for 3 s after the newest file time it saw (coarse file system clocks), so a poll right after a change is always a real read. Only requests that can move a ref, HEAD or the stash, and the two journal reads, sync before and adopt after.

Baseline: release, 2026-10-14, Linux, load average 5-8, fixture as above (2,000 files, 5,000 commits), median of one run; ms.

| operation | git2 base | git2 budget | system base | system budget |
|---|---|---|---|---|
| outside snapshot read (nothing moved, fingerprint not yet trusted) | 1.6-2.2 | 10 | 4.1-5.1 (2 spawns) | 17 |
| outside change recorded (1 ref moved by plain git; read + reflog + pin + journal write) | 10.8-11.8 | 25 | 12.4-14.8 (2 spawns) | 30 |
| branch create (journal + outside; its own move adopted) | 18.6-22.3 | 30 | 21.3-28.6 (3 spawns) | 35 |

Spawns on the system backend (`tests/spawn_budget_outside.rs`, exact): a ref-moving mutation adds **2** (the snapshot read that adopts its own moves); a poll of a quiet repository adds **0**; libgit2 adds none. The journal rows of this table were measured with `journal_outside` off (the harness switches it on for the three rows above only), so they are the baselines they always were.

Journal rows at depth 100 (2026-10-14, release, Linux, load average 2-3, git2; the harness journal is about 1 MB because the staging and discard rows before it leave entries that list every file): before the fix branch create / delete 17.1 / 17.3 ms, undo / redo (branch create) 11.5 / 11.5, undo (branch delete) 9.4, journal state 1.2, outside change recorded 9.9, branch create (journal + outside) 18.1. Cause: every journal write re-serialised all entries with serde_json (7 ms per MB, two writes per operation: write-ahead and outcome), so an operation cost grew with the bytes held in the history, not with the operation. `journal_store::render` now keeps each entry's JSON next to the entry it came from and reuses it while the entry is unchanged (the write is `{version, next_id, undo, redo, retired, pin_ns}` stitched from cached pieces, byte for byte what `serde_json::to_vec` gives); an operation serialises the one or two entries it touched. After: 4.3 / 4.5, 5.0 / 4.8, 3.0, 1.3, 3.6-4.1, 4.7-5.3 (three runs). What is left is cloning the cached journal on each load (0.4 ms per MB). Guard: `journal_store::tests::a_save_serialises_only_the_entries_that_changed` (100 entries; a save that changes 1-2 of them serialises 1-2, deterministic, no clock). The 10 ms budget of the journal rows stays: it now holds with margin, and re-serialising the history would put them back near 17 ms. The harness fixture's `.git/gitpoulpi/journal.json` grows across runs; delete `target/perf/fixture/.git/gitpoulpi` (and any leftover `perf-undo` branch) before a run that needs a clean baseline. Measure with `--release`: a dev build is 4x slower on serde and reads 70 ms.

### Conflict prevention (`perf --search`, "conflict check" rows)

`PredictConflicts` (docs/architecture/session-and-state.md "Conflict prevention") runs in the daemon, never on the render thread, debounced 2 s and cached by `(head oid, base oid)`. Release, 2026-10-20, Linux, the search fixture (`perf --search --commits 50000`; a branch forked 100 commits back editing `f.txt`, which every later commit edits, against `main`), median of 3, ms:

| measurement | git2 | system | budget |
|---|---|---|---|
| the merge itself (`predict_merge_conflicts`) | 0.6-1.2 | 1.6-1.9 (1 spawn) | 150 (enforced) |
| request, cold (base resolution + merge), Base branches = `main` | 3.1 | 9.4 | reported |
| request, same HEAD and base (cached) | 0.8-1.7 | 5.7-8.2 | reported |
| request, default `**`, 101 branches, new HEAD | 113-128 | 151-172 | reported |
| request, default `**`, same tips (distance memo) | 1.1 | 5.7 | reported |

The merge costs milliseconds on 50,000 commits (it reads three trees, not the history). What costs is finding the base: one graph walk per candidate branch (`ahead_behind`: about 1 ms each on libgit2, a spawn each on the system backend), capped at 300 candidates (`MAX_BASE_CANDIDATES`), which is why a new HEAD with the default `**` over 101 branches takes 110-170 ms. That is paid once per HEAD change, off the render thread, and the daemon keeps the distances per `(tip, base)` so a repeat request (a fetch that moved one branch, Settings closed) re-walks only what moved. The deterministic guards are `tests/conflict_prevention.rs` (one merge per `(head, base)`, a debounced burst costs one merge).

Operations without a row (stashes, worktrees, remotes, lfs, conflicts, "diff full", the "log 5000 (poll, not modified)" row and the 200-dirty mutations) are reported, never budgeted. Note the poll row measures about 36 ms in this harness: it re-reads the whole log over one connection, not the cached 0.5 ms path of the TUI poll, so it is informational only.

Commit total for the status line (`count_commits`, the `commit count` rows): git2 delegates to system git, so both backends are one `git rev-list --count` spawn when cold and the daemon's `ReadCache` answers the repeat with no spawn while the refs stand still. "Warm" is reported, not budgeted: in this harness the refs are mutated between the git2 and system passes, inside the cache's 1 s racy window, so a warm read sometimes spawns; the deterministic guard is `cache::tests::the_commit_total_is_cached_until_a_ref_moves`. The 50,000-commit figure is measured by `perf --search` (below): see "commit total".

### Workspace search (`xtask perf --search`, `xtask/src/perf_search.rs`)

```
cargo run --release -p xtask -- perf --search      # 50,000 commits x 10 repositories, both backends
GITPOULPI_PERF_DIR=/tmp/gp-perf ...                # fixtures elsewhere (a few hundred MB; default target/perf)
```

A 50k-commit repository (one `git fast-import`, 3 s) cloned locally nine times (hard-linked objects), each searched through an in-process daemon over IPC. Budget, the maintainer's ask: a **warm query under 150 ms for 10 repositories of 50k commits, so under 15 ms per repository** (median over six query shapes: verbatim, a symbol, a prefixed one, a fuzzy one, a miss). Absolute milliseconds, not scaled by the spawn floor: a warm query spawns nothing. `--enforce` is the default here too; `--no-budget` only prints. Release builds only (a dev build is several times slower).

Baseline: release, 2026-10-14, Linux, load average 3-6, 50,000 commits each:

| measurement | git2 | system | budget |
|---|---|---|---|
| commit total, 50,000 commits (`count_commits`, no daemon cache; median of 3) | 11.6 ms | 9.5 ms | <= 60 ms |
| commit total, 50,000 commits, `log.commit_count = "libgit2"` (in-process revwalk, no spawn; median of 3, machine under load) | 259 ms | 234 ms | reported (the point is zero spawns: the setting is for machines where each spawn is slowed by antivirus scanning) |
| cold index build of one repository (median) | 293 ms | 245 ms | reported (off the request: the window gets the refs at once, `partial`) |
| warm query, one repository (median / slowest sample) | 4.4 / 11.8 ms | 4.1 / 13.5 ms | <= 15 ms |
| warm query over 10 repositories, one after another | 45.8 ms | 41.0 ms | <= 150 ms |
| incremental update after 1 new commit (waits for it) | 17 ms | 21 ms | reported |

Where the time goes: one stat walk of the ref files for the fingerprint, then a scan of the lower-cased subjects (`fuzzy::score_lower`, an ASCII byte path for the in-order tier, no allocation per commit) and a partial selection of the best 50 instead of a sort. The commit index is built by a background thread that holds no repository lock, so a mutation is never queued behind it (`search.rs`); `commit_messages` on the system backend is three spawns (`for-each-ref`, `rev-parse HEAD`, `log --stdin`), four when it checks a known tip for a rewrite (`rev-list`).

The window's share (S2, `workspace_search/tests.rs`, `a_keystroke_over_ten_repositories_costs_the_window_a_few_milliseconds`): ten repositories answering forty hits each plus ten hosting caches of 300 items, merged, grouped and painted per keystroke: **3.8 ms in a release build, 12 ms in a dev build**; the test caps it at 60 ms (dev). The path from a keystroke to results is the 60 ms debounce, the daemon's warm query (41-46 ms over ten repositories, sequential in the CLI, concurrent in the window, one request per repository on the shared connection) and this slice, so the first results show well under the 150 ms budget. The hosting caches are read once, on a thread, when the popup opens; nothing on the render thread touches a disk or a socket.

## The Radar scan (`radar::worker`, guards in `radar/worker/tests.rs`, `repo_probe` tests)

A Radar round is dominated by the calls it makes one after the other, so the budgets count calls and overlap; one wall clock guard has a wide margin. Measured with the hosting fake of `radar/worker/tests.rs` (`cargo test -p gitpoulpi-tui --lib radar_scan_timings -- --ignored --nocapture`: every list call waits a timer, a client costs a sleep standing for the credential-helper subprocess that reads the token), 16 GitHub repositories, four asked at a time, 2026-10-27, Linux, debug build, median of 1 (the numbers are timers, not CPU):

| scenario | before | after |
|---|---|---|
| 100 ms per call, 50 ms per token read: first fetched row / list complete | 707 ms / 2829 ms | 254 ms / 862 ms |
| 100 ms per call, free tokens: first / complete | 506 ms / 2026 ms | 203 ms / 811 ms |
| 4 repositories, 100 ms per call, 50 ms per token: first / complete | 707 ms / 708 ms | 252 ms / 253 ms |
| clients built (token reads) for 16 repositories | 64 | 1 |
| list calls in flight at once | 4 (one per repository) | 12 (three per repository) |

Where the time went (each is a trap if reverted):

- **Calls inside a repository were serial.** `fetch_snapshot` asked the pull requests, then the issues, and `fetch_extras` the mentioned requests, the checks and the mentioned issues: five round trips in a row on GitHub. The pull request and issue lists now go out together (`tokio::join!` in `fetch_snapshot`, over `pulls_part` / `issues_part`), then the three extras together: two round trips. The limit of four repositories stays (it is in the user docs), so at most 12 calls are in flight, 3 per repository.
- **A token was read per client**, and a repository builds up to four: a `git credential fill` subprocess each (64 for 16 repositories; a heavy helper such as Git Credential Manager costs hundreds of ms each). `RadarFactory` keeps tokens per profile for the length of a round (`TokenMemo`, cleared by `ClientFactory::new_round`, so a refresh after a reconnect reads the new token). A failed lookup is never kept.
- **`config.toml` was parsed twice per repository** by `FsPrepare` in the cache phase that precedes every network call; `PerRound` reads it once per round (`Prepare::begin_round`).
- **The ETag cache was cleared whole at 64 entries** (`conditional.rs`): the Radar asks about five lists per repository, so a scope of 13 or more repositories wiped its own validators on every round and every refresh paid full bodies. It now holds 512 and evicts the entry untouched the longest.
- **The WIP probe (`repo_probe::start`) ran one `git status` at a time**; it now runs four (`WORKERS`), in list order. 16 `git status` of this repository: 64-85 ms one at a time, 20 ms with four at once (warm disk; a cold disk or a large work tree widens the gap).

Not changed, noted: `ReqwestTransport` sets `pool_max_idle_per_host(0)` (a kept-alive connection belongs to the runtime that opened it, and each Radar thread has its own), so every call pays a TCP and TLS handshake. Sharing one runtime across the Radar threads would remove it; that is a bigger change than this slice. Rows already appear progressively (cached rows first, then each answer as it lands) and nothing here runs on the render thread.

Guards: `one_repositorys_lists_are_asked_together` (peak 3 for one repository, five calls), `a_round_reads_each_token_once_and_the_next_round_reads_it_again`, `a_token_that_failed_to_load_is_asked_for_again`, `what_is_read_once_per_round_is_read_once_until_the_next_round`, `a_sixteen_repository_scan_waits_two_round_trips_per_wave` (40 ms calls, 20 ms tokens: first row under 250 ms, list complete under 600 ms; the old shape needed about 1120 ms), `conditional::tests::a_full_cache_drops_its_oldest_entry_not_all_of_them`, `repo_probe::tests::the_probe_runs_several_repositories_at_once_and_never_more_than_asked`.

## History rewrite content engine (`xtask perf --rewrite`, `crates/gitpoulpi-git2-backend/tests/rewrite_content_perf.rs`)

Release build, Linux, SSD, warm cache. The fixture is `import_content_history`: a linear history whose every commit adds two text files holding a fake token (twice as many unique text blobs as commits) and rewrites a key file under `secrets/`, plus 20 branches and 20 tags. Targets are the plan's table (Windows, Defender on); the test asserts the preview times, other sizes are interpolated from the 10k column.

| run (preview; apply in brackets) | 10k commits | target | 100k commits | target |
|---|---|---|---|---|
| remove `secrets/` (no blob read) | 0.78 s (0.06 s) | < 3 s | 13.8 s (0.64 s) | < 30 s |
| replace text, 20k / 200k unique blobs | 1.6 s (0.24 s) | < 20 s | 26.3 s (2.8 s) | < 4 min |

Spawns: preview + apply of a content run is 8 without a remote and at most 10 with one (the metadata run's budget plus one `status` and one `read-tree`), whatever the history size (`spawn_budget_rewrite.rs`). The time is trees and blobs read once through one `cat-file --batch` and written to the temp pack as they are made; the 100k remove run is superlinear against 10k because the root and `src/` trees grow with the history.

## TUI budgets (`xtask perf --tui`, `xtask/src/perf_tui.rs`)

Measured over a 10 s idle window with the Log open, then a held Down x300 at 100 Hz, then 40 key-latency samples. Baseline: three runs, release, load average 2.4-5.

| check | baseline | budget |
|---|---|---|
| idle CPU (UI thread, else process) | 0.1-0.2 % (process 0.6-0.7 %) | <= 2 % |
| idle terminal bytes/s | 0 | 0 |
| idle terminal writes/s | 0 | 0 |
| held Down x300 wall time | 3.00-3.01 s (the keys alone take 3.0 s to send) | <= 3.3 s |
| held key CPU (UI thread, else process) | 20 % (process 41 %) | <= 45 % |
| key -> first byte p90 | 2.2-2.5 ms (4.5 ms seen on a loaded box) | <= 6 ms |
| start -> repository shown, clean / with a leftover port file | 0.3 s / 0.3 s (Linux, debug) | <= 1 s each |

**Startup rows** (`perf --tui`, also alone with `--startup-only`): spawn to the `tui-open first refresh applied` line of the timing log (`GITPOULPI_TIMING`, with `TMPDIR`/`TMP`/`TEMP` pointed at a private dir so the log is this run's), best of two starts, in a fresh config dir. Two rows: `start -> repository shown (clean)` and `(leftover)`, where the config dir holds a `.port` file naming a closed port and no recorded process, what an older build left on every exit. Budget <= 1 s each. Linux, debug build, 2000x5000 fixture, 2026-10-28: 0.28 s clean, 0.31 s leftover (the leftover case is instant to refuse on Linux; the row exists for Windows, where before the fix it was clean + 2.2 s: connect to the dead port, ~2 s to be refused). A leftover file with a dead recorded process is deleted without any connect (unit tests in `gitpoulpi-ipc` `discovery`).

**Idle is 0 bytes.** `quiet_backend::QuietBackend` (the TUI's ratatui backend) drops an empty frame diff and cursor commands that restate the known state, so a poll tick that changes nothing writes nothing (before: a 24-byte `ESC[39m ESC[49m ESC[59m ESC[0m ESC[?25l` about every 2 s). The Kitty synchronized-update bracket is held back until a frame writes something. "Settled" in the measurement means no byte for 6 s: a status message lives at least 3 s and its removal is one legitimate repaint, which would otherwise land in the window.

## Merge conflict editor budgets (`perf --tui`, `xtask/src/perf_merge.rs`, slice SP)

`perf --tui` also opens the merge editor, in the real release TUI at 200x60, on two generated fixtures (`target/perf/merge-10000-50`, `target/perf/merge-100000-3`; built by `merge_fixture::build_big`, the same generator as `xtask merge-fixture --kind big --lines N --conflicts K`): a 10,000-line Rust file with about 50 conflicts and a 100,000-line one with 3. `--merge-only` runs just these, `--no-merge` skips them. Held keys run at 100 Hz (typing at 50 Hz); the figure is UI-thread CPU per key, so it is the frame cost, whatever the machine's key rate.

| check | before SP (10k / 100k) | after SP (10k / 100k) | budget (10k / 100k) |
|---|---|---|---|
| open: first byte (the "Opening" status) | 1 / 1 ms | 1-2 / 1 ms | <= 150 ms |
| open: editor on screen | 71 / 367 ms | 97-166 / 400-610 ms (view built on a worker; the daemon read and the IPC dominate; 1.2 s seen under load) | <= 600 / 2000 ms |
| idle bytes after settle and after the run | 0 / 0 | 0 / 0 | 0 |
| A/B held Down, UI ms per key | 10.4 / 10.5 | 0.8 / 1.4 | <= 4.5 (the Log's 45 % at 100 Hz) |
| A/B held PageDown | 10.8 / 10.7 | 1.0 / 1.9 | <= 4.5 |
| A/B scroll right (`L`) | 0.9 / 3.9 | 0.9 / 1.0 | <= 4.5 |
| Output held Down | 1.0 / 10.2 | 0.9 / 0.9 | <= 4.5 |
| Output held PageDown | 0.8 / 10.4 | 1.0 / 1.1 | <= 4.5 |
| Output typing (50 Hz) | 2.2 / 20.2 | 1.6 / 3.0 | <= 9 |
| Full file toggle, the first one | 1.5 / 1.5 (170 ms in a later run on the 100k file) | 2 / 2 (the side diffs are computed on the build worker) | <= 250 ms |
| pick / unpick, key to last byte | 12 / 31 ms | 3 / 4 ms | <= 250 ms |
| one typed char, key to last byte | 2.5 / 21 ms | 1-3 / 8 ms | <= 150 ms |

Baseline: release, 2026-10-03, Linux, load average 6-13 (other builds running), a range over a few runs (the figures are 10 ms CPU ticks / 150 keys, so read them to about 0.2 ms; a loaded box doubles them). What the pass changed, in the order of their cost:

- **Syntax highlighting was the frame cost.** syntect costs 0.13 to 0.2 ms a line on the Rust grammar, and the old code parsed every line from the top down to the furthest one shown, 20 ms of every frame while it caught up: a held key on a 100k file never stopped burning CPU. Now (`Highlight::advance_window`) a jump of more than 400 lines is crossed with a cold window starting 40 lines above the view (right unless a string or comment opened above it runs into it), kept while the view moves down from it; frames of a held key (`MergeView::input_busy`: two inputs within 150 ms) parse nothing, except the Output caret's line, and the colouring is filled in when the stream stops. An edit keeps the old colours below it (`stale`) until they are parsed again, and an edit inside the window re-parses from its 8-line checkpoint.
- **`origin_of_line` compared a whole gap with its original once per painted row** (2.4 MB a row on the 100k file): the gap check, the conflict statuses and the picked-lines' letters are cached on the Output's version and the model's generation (`Derived` in `model.rs`).
- **Typing rescanned the Output**: the `Doc` line index is patched by the edit instead of rebuilt, and the common prefix and suffix of the old and new text are found by block compares.
- **Opening a file with 3,000+ lines builds its model, alignment and view on a worker** (`merge_flow.rs`), with the grammars warmed and the Full file diffs and the widest line (the sideways scroll's clamp, which was a pass over the whole file per key) computed there; the UI shows the "Opening" status and the spinner meanwhile.

Already fine and left alone: the three panes' paint walks only build the visible rows (since S5), `side_changes` is computed once per document, and the alignment is computed once at open.

Rerunning on a different machine: the daemon budgets scale with the spawn floor, the TUI ones do not. On Windows, treat a first failure as a prompt to re-baseline here, not as proof of a regression.
