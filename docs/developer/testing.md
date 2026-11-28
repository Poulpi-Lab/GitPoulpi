# Testing

## Tiers

The suite is split so the slow end-to-end tests can be run apart. `cargo run -p xtask -- test [--tier fast|pty|full] [--jobs N] [--serial] [--no-fail-fast] [--no-run] [--dry-run]`:

| Tier | What | Command it runs |
|---|---|---|
| `fast` (default) | everything except the PTY targets: unit, integration, contract, doc tests | `cargo test --workspace --lib --bins --test <each non-pty target>`, then `--doc` |
| `pty` | only the integration targets named `pty_*` (the `pty_hosting` directory target and `gitpoulpi-cli`'s `pty_*` included) | `cargo test --workspace --test <each pty_ target>` |
| `full` | both | `cargo test --workspace` |

**Parallel runner.** `cargo test` runs the test binaries one after another, which is the wall clock on Windows: a process spawn costs ~100 ms there (~1 ms on Linux) and the tests spawn `git` constantly, so a binary mostly waits (the contract binary: 600 s on Windows, 6 s on Linux). So `xtask test` builds once (`cargo test --no-run --message-format=json`), then runs the test executables itself, `--jobs N` at a time (default: three quarters of the CPUs, measured faster than all of them on the 8-CPU Windows VM; `GITPOULPI_TEST_JOBS` overrides; at most half that many PTY binaries at once, `GITPOULPI_PTY_JOBS`, and PTY binaries only start once every non-PTY binary has started, so the long contract suite is the only thing overlapping them; `--tier full` is one pool for both, `scripts/check.ps1 -Full` uses it), each from its package directory with output captured. Doc tests run as one `cargo test --doc` job beside them. Output: one progress line per finished binary, then a summary sorted by name (pass/fail, duration), the ten slowest, and the full output of every failing binary. The durations of the last run (`target/xtask-test-durations.tsv`) order the next queue longest-first. `--serial` (and `--no-run`) use plain `cargo test`, the old path, to compare or bisect. PTY binaries run 2 tests at a time each (`GITPOULPI_PTY_TEST_THREADS`): libtest's one thread per CPU times N parallel binaries starved the terminals and about 10 of 250 PTY binaries failed on timing waits. Each PTY test isolates its config dir and daemon (own temp dir, port 0), so binaries share no state. Non-PTY binaries are capped too (`RUST_TEST_THREADS` = max(4, CPUs / jobs), `GITPOULPI_TEST_THREADS` overrides; cap 2 was measured slower than no cap on 20 CPUs because the long binaries are the critical path, cap 4 to 8 slightly faster). **Serial group:** binaries listed in `SERIAL` (`xtask/src/test_runner.rs`; today `pty_idle`, which measures a quiet window) are held back, then run one at a time on an otherwise idle machine after the pool has drained; add a binary there only if it fundamentally needs an unloaded machine, as it lengthens the wall clock. Keep `TMP`/`TEMP` overridable: on Windows point them at a folder the antivirus does not scan. **Temp isolation:** each binary gets its own `gpx-<pid>-<n>` folder under that temp dir as `TMPDIR`/`TMP`/`TEMP`, removed when it ends (left-overs older than 6 h are swept at the next run), so a test that leaks (a forgotten `TempDir`, a TUI killed before it cleans its copies) cannot fill the temp dir; the summary lists the binaries that left entries behind (`temp leaks:`), fix the big ones at the source. A test that builds a huge path or environment value from the temp dir must budget bytes, not entries: the folder name is longer than the system temp dir. `scripts/check.ps1` also sets `CARGO_PROFILE_DEV_DEBUG=0` (no debug info in gate builds; backtraces keep function names, lose line numbers).

**Run-alone rule.** A test that passes alone but fails under load runs alone, not beside the others. `xtask/run-alone.txt` lists them: `<crate>::<binary>` (a whole binary, which keeps its own thread count) or `<crate>::<binary>::<test name>` (one test, exact name from `-- --list`, run with one test thread); `#` starts a comment or a reason. The pool leaves them out (single tests via exact `--skip`), and after it drains each entry runs by itself on an idle machine. Safety net: every test that failed in the pool is rerun alone with an exact filter. If it passes, the run does not fail, the summary counts it (`N saved by the alone rerun`) and prints a block `passed only when run alone (add to xtask/run-alone.txt):` with the exact lines. Add those lines (with a reason) when the same test shows up twice; a test that fails alone too is a real failure, fix it. Do not use the list to hide a race: first find the cause (patterns below). `--dry-run` prints the alone phase; `--failed` honours the list.

**Iterating on failures.** Every parallel `xtask test` run writes `target/xtask-test-failures.tsv` (atomically): one row per failing binary (package, target, kind) with the test names parsed from libtest's `test <name> ... FAILED` lines; a binary that failed without a parseable name (crash, timeout, a build error) has no names and reruns whole. A green run leaves an empty file, so "last run green" differs from "no run yet" (no file). `xtask test --failed` (`scripts/check.ps1 -Failed` adds fmt + clippy) builds only those targets and runs each with its failing tests as `--exact` filters, then rewrites the file with its own result, so repeating it converges to nothing. The loop: full tier, `--failed` until green, then the full tier again before a release tag (a targeted rerun proves the fixes, not that nothing else broke; `--failed` says so when it is green). A `--failed` run never stops at the first failing binary. Fix one cause at a time and rerun only the failures that fix should cure: `--failed --only PATTERN` (repeatable; `check.ps1 -Failed -Only a,b`) picks the recorded failures whose target or test name contains a pattern, and the others stay in the file for the next fix. Rerunning failures a fix cannot touch only costs time. `check.ps1 -Failed` skips clippy to keep that loop short (add `-Clippy` when the fix is about a lint); the full tiers always run it.

Cargo cannot exclude one test target, only select them, so the xtask lists the targets with `cargo metadata` (kind `test`, name prefix `pty_`) and spells the selection out: a new test file lands in the right tier with no list to edit. All tiers use the same flags, so they share one build. Name a new PTY test `pty_*` or it runs in the fast tier. `--dry-run` prints the commands.

`scripts/check.ps1`: fmt + clippy + fast tier by default, `-Pty` adds the PTY tier, `-Full` is all of it, `-Quick` is fmt + clippy. The last lines list the tiers that ran and the ones SKIPPED.

CI (Ubuntu) runs the full suite. **Windows tests are local only**: the Windows runner hits the job timeout compiling the test targets, so its CI job runs fmt and clippy. Run `pwsh scripts/check.ps1` before pushing and `pwsh scripts/check.ps1 -Full` before a release tag.

## Isolation from the machine's real user state

A test never reads or writes the developer's real config, data dir, home or credential helper (the integration token store shells to `git credential`, i.e. the OS keychain or Credential Manager).
- **PTY children** (`xtask::pty::Terminal`, the single choke point): whenever the child gets a `GITPOULPI_CONFIG_DIR`, `HOME`, `USERPROFILE`, `APPDATA` and `LOCALAPPDATA` become subdirectories of it, `GIT_CONFIG_NOSYSTEM=1`, and `GIT_CONFIG_GLOBAL` is a file in it whose helper is `store --file <dir>/git-credentials` (`pty::isolated_user_env`). A test may still override any of these with `.env(..)`. `pty_hosting`'s Jira connect test asserts the token lands in that file.
- **First-open column fit:** a repository with no saved Log widths is fitted to its content once. `common::launch` sets `GITPOULPI_NO_LOG_AUTOFIT=1` so tests about other things start from automatic widths; `pty_log_column_fit_new_repo` sets it to `0`.
- **Daemon, CLI and MCP test binaries**: `gitpoulpi_testkit::isolate_binary!();` once at module level (it runs before `main`, single-threaded: `set_var` against concurrent `getenv` is undefined behavior) sets the same variables for the whole process, with a per-process throwaway dir. Add it to every new test binary of those crates; `crates/gitpoulpi-testkit/tests/isolate.rs` checks the mechanism. Do not `set_var` inside tests: set once (`Once`), pass the value in the request (`"autostash": "never"` in `flow.rs`), or write the config file of the isolated dir.
- Unit tests that need a token store use `InMemoryTokens`.

## Fixture repos

`gitpoulpi_testkit::FixtureRepoBuilder` builds a real temp repo by shelling to `git` (testkit must not use a backend it tests; the one libgit2 shortcut is below). `git init` is not spawned per fixture: one real `git init` per machine writes a skeleton (`<tmp>/gitpoulpi-testkit-init-v1/{plain,bare}`) that is copied (`init_skeleton`), which cut the contract binary's spawns from 20240 to 19013. `commit*`/`commit_removal` do not spawn `git add` + `git commit` either: `add_and_commit` stages the raw bytes and commits through libgit2 (`git2`, a testkit dependency only for this), with the CLI's message cleanup, identity, fixed dates and reflog line; a unit test pins that the commit ids, reflog, `status` and `diff-files` equal the CLI's. It steps aside for the real commands as soon as the CLI's own behaviour matters: a `.gitattributes`, an LFS filter in the config, reftable, or a `.config()` call (also forced everywhere by `GITPOULPI_FIXTURE_GIT_CLI=1`, to compare). Together with the skeleton: contract binary 20240 -> 16358 `git` spawns, fixture-built 10135 -> 6251. It cuts itself off from the user's global/system git config, so a fixture is identical on every machine; repo-local keys go through `.config()` (how the hostile-config test turns on `diff.mnemonicPrefix` and friends). `conflict_fixture.rs` builds conflicted states; `fake_agent.rs` stands in for an AI agent CLI.

## Contract tests - the backbone

`gitpoulpi_testkit::contract` (`src/contract/*.rs`) holds ~200 `check_*` assertions over `&dyn GitBackend`, run against **both** backends from `crates/gitpoulpi-git2-backend/tests/contract.rs`.

- The list lives in one place, the `for_each_contract_check!` macro in `contract/mod.rs`; the test crate generates one `#[test]` per check per backend so cargo runs them in parallel (as two sequential tests the binary took 140 s locally / 22 s on Windows CI, because every check builds a fixture and spawns `git`). An unlisted `check_*` fails the `every_check_is_listed` guard. A check that depends on another does not belong here.
- Network ops run against local bare origins (`with_origin()`), never a real host.
- Backend-specific tests exist in `crates/gitpoulpi-cli-backend/tests/`; prefer the contract suite - a behaviour asserted on one backend only is one the two may disagree about.
- **Spawn budgets** (`testkit/src/spawn_budget.rs`, used by the backend and daemon `tests/spawn_budget.rs`): how many `git` processes an operation may spawn. A failure is a perf regression, not a flaky test (`perf-budgets.md`).

### Older git

The minimum git version is 2.34 (Ubuntu 22.04 ships it; Apple's Command Line Tools ship 2.39). Product code that needs a newer git detects it (`process::git_at_least`, which parses `git --version` once and caches the result) and takes another path, or answers `Unsupported`; it never assumes a version. To test this, build an older git into a temp prefix (`make prefix=/tmp/old-git NO_OPENSSL=1 NO_CURL=1 NO_EXPAT=1 NO_TCLTK=1 NO_GETTEXT=1 NO_PERL=1 NO_PYTHON=1 install` from a git 2.34.x source tarball), put its `bin` first on `PATH` and run `cargo test -p gitpoulpi-cli-backend -p gitpoulpi-git2-backend -p gitpoulpi-daemon` (the contract suite and the spawn budgets). Such a git has no HTTP support, so the credential-helper test in `gitpoulpi-cli-backend/tests/credentials.rs` is expected to fail there. A check whose *oracle* needs a newer git gates itself with `gitpoulpi_testkit::system_git_at_least`; product code is never skipped.

## LFS tests

Offline, and they skip cleanly (`gitpoulpi-testkit/src/lfs.rs`, the `with_lfs*` builders):
- Pointer-only checks need no binary (`commit_pointer_text`).
- Checks needing `git-lfs` start with `require_git_lfs!("name")`: prints `SKIPPED` without it, but **panics** when `GITPOULPI_REQUIRE_LFS=1` (set by `scripts/check.ps1` when `git lfs version` works, and by `ci.yml`), so a machine that should have git-lfs cannot pass silently.
- `.with_lfs_origin()`: a committed `.lfsconfig` pointing at the bare origin as a `file:///C:/...` URL (forward slashes, `%20`), so git-lfs's standalone file adapter transfers without a server. It adds one commit: call it before the commits a check asserts on.
- Locks need HTTP: `LfsLockServer::start()` (127.0.0.1:0, in-memory, owner `tester`); also set `credential.helper = ""` so Git Credential Manager never prompts.
- Keep LFS checks few: each spawns several processes, slow on Windows.

## TUI tests

Three kinds:
1. **Buffer tests** (`ratatui::TestBackend`) in inline `#[cfg(test)]` modules: assert specific cells, columns and styles. Shared fixtures in `test_support.rs`.
2. **`insta` goldens** for the themes only (`crates/gitpoulpi-tui/src/theme/snapshot_tests.rs`, one per preset; update with `INSTA_UPDATE=always`). Elsewhere targeted assertions are preferred: they say why they exist, and the UI is redesigned too often for snapshot churn.
3. **PTY tests** (`crates/gitpoulpi-tui/tests/pty_*.rs`): the built `gitpoulpi-tui` binary in a pty (`xtask::pty`, `portable-pty` + `vt100`), real key and SGR-mouse bytes in, the painted cell grid (chars and colours) out. The only tests covering the whole chain (crossterm parser, event loop, hit targets, daemon, real `git`). **Every fix gets one.** No window or compositor, so they run in parallel; `cargo run -p xtask -- tui <script>` drives a screen by hand (`visual-verification.md`).

Rules:
- **Falsify every test** you write for a fix: remove the fix, see red, restore. Three tests in this project once passed while asserting nothing.
- **A buffer test cannot see how a font renders a glyph** (emoji presentation, the graph's glyph metrics): for anything visual, take a screenshot (`visual-verification.md`).
- Prefer rules that are compile errors: `palette/tests/catalog.rs` `every_mouse_affordance_maps_to_a_palette_command` is an exhaustive `match` over the toolbar/menu/diff-bar enums, so a new mouse affordance does not compile until the palette reaches it.
- Locate things in a PTY test by their painted text, never by a hard-coded column.
- **PTY tests run on every OS.** No `#![cfg(unix)]` on a `pty_*` file: a gate is a platform losing end-to-end coverage. `tests/common` has the portable pieces: `fake_program` / `fake_recorder` / `fake_failing` (an executable `/bin/sh` script on unix, a `.cmd` on Windows; a program the *app* starts), `write_sh_script` (a script only `git` starts: a hook, `git-lfs`, `gpg.program`; Git for Windows runs it through its own `sh`, so one file serves both), `toml_path` (a path inside a TOML `"..."` string), `path_with_first`, `Launch::scratch_home` (`HOME`/`XDG_*` on unix, `USERPROFILE`/`APPDATA`/`LOCALAPPDATA` on Windows) with `alias_file`, `release_archive` (`.zip` on Windows) and `updater_stub` (a real `.exe` built with `rustc` on Windows: the app copies the stub to `gitpoulpi-updater.exe` and starts it in a new console, so the test reads its recorded files instead of the screen). A test that really is one OS only gets `#[cfg_attr(windows, ignore = "<precise reason>")]` (or a `cfg` on that test), never the whole file, and a Windows twin when that is cheap (`pty_launcher`'s Start menu test).
- PTY tests are **load-sensitive when written carelessly**: rerun a failure alone before calling it a regression, then reproduce it under load (below) before "fixing" it. A timing claim you did not measure is not a finding (a "flaky" test once spread through three docs before anyone ran it: it passed twelve runs out of twelve). Every flake the load stress found had a named cause; none was fixed by a longer timeout.
- **Reproduce under load, on purpose.** Run several copies of a test binary at once (`for i in $(seq 40); do target/debug/deps/pty_x-HASH <filter> > log.$i & done; wait`): a 5-10 % failure rate at 40 copies is a real race. Run the touched `pty_*` binaries in 4-6 parallel lanes while a full `cargo test --workspace` runs for the stress version. `GITPOULPI_PTY_TIME_SCALE=3` multiplies every `wait_*` deadline (it only bounds a *failing* wait; a passing one returns at once) for a machine that is simply slow.

### Waiting in PTY tests: the patterns that survived load

The cause of every flake of this kind was one of these. The first rule covers most of them.

1. **The predicate is the whole post-condition.** A terminal has no frame boundaries: the title is drawn before the rows, the rows before the scrollbar, the highlight after the text. `wait_for("the help", |s| s.contains("Keybindings"))` followed by `assert!(s.contains("User docs"))` passes on a half-painted frame and fails under load. Everything you are about to assert or click goes *into* the wait (the marks on both scrollbar tracks, the count in the status line, the thread under its line, the colour of the keyword), and the value you use afterwards comes from the screen the wait returned. A first read of `term.screen()` right after a launch or a click is the same bug. After a `resize`, the rows are on the old frame too: wait for something only the new layout draws (a pane starting past a column).
2. **A predicate that is already true proves nothing.** `s.contains("Ctrl+N") && s.contains("focus")` held before the key was read (the title carried the session name, the footer lists `Ctrl+N`), `s.contains(" 1 ")` matched the ordinary page, `s.contains("1") && s.contains("3")` matched everything. Wait for something only the new state draws (`<badge>=focus that sub-pane` for the navigation prefix).
3. **Keys that read the last painted frame need that frame first.** The `Ctrl+N` badge digit resolves against the badges of the last *drawn* frame; a pty delivers `C-n` and `1` before the repaint, so the digit finds none. The same shape: a `d` sent with the key that opens a confirmation box is read by the screen underneath. Send the follow-up only after the screen shows what it acts on. (This is by design for the badges: the glyph on screen is what you press.)
4. **No quiet-period waits for correctness.** `wait_for_stable(800ms)` between two keys let an unread Escape meet the next `Ctrl+N` as `Alt+Ctrl+N` on a busy machine. Wait for the visible consequence of the key (the prefix hint appears, then goes back to `?: help`). `wait_for_stable` is for measuring (a byte count over a settled screen), not for ordering.
5. **A target can move after its first paint.** A commit's details, a list filled by the daemon, a board refreshed after a file write arrive in stages and push rows around. `wait_for_settled_text(needle, 150ms, ..)` returns a position that held still; `Terminal::act_until(what, attempt, timeout, action, predicate)` repeats an **idempotent** gesture (open a menu, double-click a branch, a right-click) with the target located afresh each time until the screen answers. Never use it for a toggle, a mutation or anything a slow success would repeat. And locate again only what is unambiguous: after a click that opens a diff, the file name is also in the diff's header.
6. **A double-click is two events the TUI times itself** (`DOUBLE_CLICK_WINDOW`, 400 ms, measured at handling): a stalled TUI reads the pair as two single clicks. Repeat it with `act_until`.
7. **Disk before journal.** The daemon changes the files or HEAD, *then* journals the operation, and a restore or a checkout reports nothing on screen when done. A Ctrl+Z or an Undo click the moment the file changed can find nothing to undo. Press it again until the file is back (only when the test journals nothing else), as `pty_file_pane` and `pty_hosting/providers` do.
8. **A modal that is still on its way eats the next input.** The "Checkout failed" dialog arrives independently of the banner it accompanies; a click while it is up is swallowed. Wait for it and dismiss it (`pty_autostash_banner`).
9. **The wait on the disk is not the wait on the screen.** After a note moved on disk the board re-reads; a press at the pre-refresh position hits nothing. Wait for the screen to show the move (`pty_hosting/board`).
10. **Own `wait_until` helpers** (polling the disk) must use `xtask::pty::scaled(STEP)` for the deadline so the time scale applies, and name what they wait for in the panic message.
11. **Waiting for a phrase that wraps.** A dialog line that carries a measured number (`Estimated time: about 46 ms`) wraps at a different word when the number gets longer, so `contains("Estimated time: about")` held with 5 ms and never with 46 ms under load. Wait for single words or for text that starts a line.
12. **A key after a closing popup.** A wait that the *pane under* a checklist has already changed does not prove the checklist closed: the unread Escape then meets the next `Ctrl+P` as `Alt+Ctrl+P` and the typed text lands in the sidebar filter. Put "the popup is gone" into the wait.
13. **Time budgets measure CPU time, not the wall clock.** `palette::tests::french::matching_a_keystroke...` failed 62 runs of 80 on a best-of-five wall-clock minimum because the thread simply waited for a core; it reads `CLOCK_THREAD_CPUTIME_ID` now.
14. **A key right after `Esc` can be eaten by it.** `send_keys("Escape")` then `send_keys("C-z")` reached the TUI in one read as `Alt+Ctrl+Z`: the field kept the caret and every later `Ctrl+Z` was the field's own undo (`pty_commit_now`, failing 10 runs of 10 on two cores). Order the test so that no key follows an `Esc` before the screen shows its consequence, or leave the field by a click. A status-line message left by an earlier step also makes "the reason was reported" true before the key under test was read: wait for a change only that key draws (the right pane switching to the Status).
15. **The "What is sent" dialog is off in the harness.** Every AI action stops at `ai_sent_dialog` unless `[ai.sent_preview]` says otherwise, so `tests/common` sets `GITPOULPI_AI_SENT_PREVIEW_<ACTION>=false` for all ten actions; `pty_ai_sent_dialog` turns one back on with `.env(..)` (the environment wins over `config.toml`, so "Don't show again" is asserted in the file). The staging guard needs no such switch: ordinary files pass it; a test that stages a `.env`, a key or a token expects the `pty_stage_sensitive` question.

### Unit tests and process-wide settings

`live_settings` holds process-wide statics (`MINIMAP`, `BLAME_*`, the date patterns, the Diff view prefs); `diff_recode`'s store (the remembered encoding per path) is thread-local in `cfg(test)` for the same reason (a test that remembered Windows-1252 for `src/a.rs` turned the bar of another test's render from `Auto` to `Windows-1252`). Unit tests run in parallel in one process, so a test that turned the minimap on changed the width of every other test's render (`a_comment_in_split_goes_on_the_side...`, `the_blame_gutter_stays_fixed...` failed only under load). In `cfg(test)` those cells are **thread-local** (`published!`/`cell!` in `live_settings.rs`): each test thread has its own copy, defaulting to the production default. A new published setting goes through `published!` too; a global the test must share across threads needs a lock instead. A time-based behaviour (a held key within `BUSY`, 150 ms) is made independent of the CPU by stamping the clock (`last_input`) in the test rather than hoping the loop is fast.

### Config files: `config.toml` and `tui.toml`

A test that resolves `Settings` from a scratch `config.toml` also reads a `tui.toml` next to it, so seed both or neither. `crates/gitpoulpi-config/tests/split_read_path.rs` and `split_writers.rs` cover the read and write sides (no `tui.toml` changes nothing; once it exists silence means default; an interface write creates it as a copy and never touches `config.toml`), `pty_tui_toml.rs` the end to end path through the terminal UI and `config set`. `docs_coverage.rs` fails when a key is documented under the wrong file in `docs/user/config-reference.md` according to `KEY_HOME`; adding or moving a key means classifying it there and placing its row. The `desktop.*` keys are deliberately excused from the documentation check until their wave (`desktop_keys_are_excused_until_the_documentation_wave`).

### Dev-profile cost

A debug build is 10-50x slower on tight loops. The syntax highlighter (syntect and its regex stack) is built with `opt-level = 3` in dev and test (`[profile.dev.package.*]` in the root `Cargo.toml`): `pty_editor_highlight_idle` (40,000 lines) took 50 s of its 60 s budget before and takes about 8 s now. When a PTY test is slow even alone, profile the dependency before raising the timeout.
- **ConPTY trap:** Windows' ConPTY writes its own preamble bytes; the driver once counted them as the child's output and asserted against a blank grid. "Bytes arrived" is not "the child drew": ask the grid (`xtask/src/pty/mod.rs`, `Terminal::bytes_drawn`). `xtask/tests/pty_driver.rs` (against the trivial `xtask emit` child) tells driver failures from TUI failures; keep it running on Windows.
- Tests use temp config dirs only, never the maintainer's config.

### The fake hosting service (`xtask::mock_host`)

Host-driven flows have their own, stateful one: `xtask::mock_host::flow::start(FlowHostOptions)` is a GitLab-shaped API (issues, branches, merge requests) over a real bare repository (the fixture's `remote_path()`), with a scripted **bot** that makes `{type}/{iid}-{slug}` and a request `bot_delay` after an issue is created; `crates/gitpoulpi-cli/tests/flow_issue.rs` and `crates/gitpoulpi-tui/tests/pty_flow_issue.rs` are the models. `GITPOULPI_FLOW_POLL_MS` fixes the wait's pause (default 1 s growing to 10 s) so a test does not sleep seconds.

A loopback server (`cargo run -p xtask -- mock-host [--port N] [--bare-root DIR]`, prints its port, serves until killed; `xtask::mock_host::start` in tests) answering as GitHub Enterprise (`/api/v3`, profile `kind = "github_enterprise"`) and GitLab self-managed (`/api/v4`, `kind = "gitlab_self_managed"`) at the same `http://127.0.0.1:PORT`. Each side has its own signed-in user and fictional owners (the user, organisations, a GitLab sub-group), public and private repositories, real `per_page`/`page` paging. A token containing `revoked` gets a 401. With `--bare-root` (`seed_bare_repos` in tests) every repository also exists as a tiny bare repo and the advertised clone URL is its `file://` path, so a clone really happens with no network. `pty_clone_hosted.rs` is the model (grouped sections per owner, per-source separation, search, a real clone, a 401). Facts the Clone list taught: the list is a **six-row window** (reach later sections by search), and no avatar or fork marker is drawn (the mock's `fork` field is there for when one is). The GitLab side also answers `/api/v4/projects/<path>/merge_requests` and `/issues` with a few fictional items (the signed-in user is `mara`; ages are relative to now; a project whose path contains `conflict` gets the second set), which the `radar` screenshot shot links two fixture repositories to. Add fixture owners or routes in `xtask/src/mock_host.rs`; the older, wider GitLab mock of `pty_hosting/mock.rs` (PRs, issues, Jira...) shares its HTTP loop (`xtask::mock_host::http::serve`) rather than duplicating it.

### The real-server bench (`scripts/test-bench.sh`)

Local Docker containers the integrations are checked against by hand, and by the `#[ignore]` live tests. Never part of the gate and
never reachable from outside: every port binds to `127.0.0.1`, data lives in named volumes `gitpoulpi-*`.

```
scripts/test-bench.sh up [name...]      # create what is missing, start what is stopped (never recreates)
scripts/test-bench.sh seed [name...]    # admin user, token, sample content; safe to run again
scripts/test-bench.sh status [name...]  # container state, HTTP answer, whether a token file exists
scripts/test-bench.sh down [name...]    # stop the containers; the volumes stay
```

| Name | Container | Port | Seeded with |
|---|---|---|---|
| `gitlab` | `gitpoulpi-gitlab` (`gitlab/gitlab-ce`, 5 GB, minutes to boot) | 8929, ssh 2224 | a root token, project `gitpoulpi-sandbox` |
| `gitea` | `gitpoulpi-gitea` | 3000 | admin `poulpi`, a token, repository `poulpi/gitpoulpi-sandbox` |
| `forgejo` | `gitpoulpi-forgejo` (Forgejo 11) | 3001 | the same |
| `redmine` | `gitpoulpi-redmine` (SQLite) | 3002 | default data, REST API on, admin key, project `bench` (member `admin`), version `v0.1`, three issues |
| `gerrit` | `gitpoulpi-gerrit` (3.14, "become any account" auth) | 8081, ssh 29418 | a token for `admin` (REST), project `gitpoulpi-sandbox`, one change pushed to `refs/for/master` |

Secrets: tokens go to `$GITPOULPI_BENCH_DIR` (default `~/.cache/gitpoulpi-bench`, mode 600, file `<name>.token`, plus `gerrit.user`
and a generated `password`); set `GITPOULPI_BENCH_PASSWORD` to choose the GitLab root / Gitea admin password instead.
`GITPOULPI_BENCH_USER` names the Gitea/Forgejo admin (default `poulpi`). Nothing is printed. Point GitPoulpi at a bench only in a
scratch environment (`HOME`, `GITPOULPI_CONFIG_DIR`, `XDG_*`, `GIT_CONFIG_GLOBAL` inside a throwaway directory, `GITPOULPI_DESKTOP_NO_HOOKS=1`),
with the token in `GITPOULPI_TOKEN_<ID>`; the tokens of a Gerrit profile go in with `--account <user>`.

`scripts/e2e-review.sh SCRATCH_DIR up github|gitlab|gitea|forgejo` drives the review view by hand against the bench in such a scratch environment (private headless sway, keys and clicks through kitty; see the script's header).

### Live provider tests: one master switch

A test that reaches a real provider (the internet or this bench) is `#[ignore = "live: set GITPOULPI_LIVE_TESTS=1 and run with --ignored"]` **and** starts with `if !gitpoulpi_testkit::live::enabled("name") { return; }`. So:

- `cargo test --workspace`, the gate, `scripts/check.ps1` and CI never run them (ignored), and never set the switch.
- `--ignored` alone still does not connect: without `GITPOULPI_LIVE_TESTS=1` each prints one `SKIPPED name: ...` line and passes. They never fail for lack of a provider.
- The per-provider variables only say where and as whom. A new live test uses the same two lines; do not invent another gate.

```
GITPOULPI_LIVE_TESTS=1 GITPOULPI_LIVE_REDMINE=http://127.0.0.1:3002 GITPOULPI_LIVE_REDMINE_KEY=$(cat $DIR/redmine.token) \
  cargo test -p gitpoulpi-hosting live_redmine -- --ignored --nocapture
```

(`live_gitea_reads` with `GITPOULPI_LIVE_GITEA[_TOKEN|_PROJECT|_PR]`, `live_gerrit_reads` with `GITPOULPI_LIVE_GERRIT[_USER|_TOKEN|_PROJECT]`, `live_linear_reads` with `GITPOULPI_LIVE_LINEAR_TOKEN`, `live_youtrack_reads` with `GITPOULPI_LIVE_YOUTRACK_URL|_TOKEN`; add `--features linear` / `youtrack` where the feature is optional.) The bench scripts (`test-bench.sh`, `e2e-review.sh`) are manual tools, never called by a test.

**Loud guard.** The real HTTP transport (`ReqwestTransport`) and the avatar fetcher refuse a non-loopback host (`gitpoulpi_hosting::http::external_host_refusal`) when blocking is on and `GITPOULPI_LIVE_TESTS` is not `1`. Blocking is always on in `gitpoulpi-hosting`'s own unit tests (`cfg(test)`), and `xtask`'s PTY driver sets `GITPOULPI_BLOCK_EXTERNAL_HOSTS=1` for the TUI and the daemon it starts. An accidental live call therefore fails with "refusing to connect to ... from a test" instead of connecting. Loopback (`127.x`, `localhost`, `::1`) is always allowed: in-process fake servers (`xtask::mock_host`, `pty_hosting/mock.rs`, the loopback LFS server) stay on. Traps: CLI integration tests (`assert_cmd`) do not set the variable; they stay offline because they use fake or loopback hosts, so a new one that needs a provider must go through a fake.

**YouTrack** (`jetbrains/youtrack`, port 8080) is manual: its first start prints a one-time wizard URL in the container log and needs a
browser to finish (license, base URL, admin), so the script does not manage it. Create a permanent token in the profile afterwards.
**Linear** is SaaS only (a free workspace and a personal API key).

Facts the bench taught: Redmine has no default data until `rake redmine:load_default_data` (the seed does it) and keeps `closed_on`
after a reopen; Gerrit 3.14 replaced HTTP passwords by tokens (`PUT /a/accounts/self/tokens/<id>`; the image's first token is
`secret`) and answers every JSON with a `)]}'` first line; the admin account of the Gerrit image already exists (id 1000000), the cookie
login (`/login/%23%2F?account_id=1000000`) cannot write over REST (it asks for `/a/`), only basic auth does.

## CLI tests

`crates/gitpoulpi-cli/tests/*.rs` drive the built binary with `assert_cmd` against fixture repos and assert on **deserialized JSON**, never on text. Each test passes its own `--session` (the session name selects the daemon; sharing one makes tests fight).

## Feature-off builds

`cargo run -p xtask -- features [--quick]` runs clippy with each cargo feature off plus the off-build tests. Every feature set is checked twice: bins only (no `--all-targets`, what `build-custom` compiles) and `--all-targets`. Both are needed: `xtask` is a dev-dependency of the cli and tui crates and depends on the backends with default features, so `--all-targets` unifies `lfs`/`worktree`/... back on and hides a broken feature-off backend (the `--no-lfs` build once broke while the matrix stayed green); only the bins-only run sees the features as a custom build does. `xtask/tests/feature_contract.rs` checks every locale has a `lang-*` feature and every binary forwards each feature of `full`. CLI integration and PTY tests do not run in reduced builds.
