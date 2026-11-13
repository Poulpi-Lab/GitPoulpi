# Where we are, and what to pick up next

A snapshot for picking the project up cold: what the product does, where the
TUI's code lives, what is still open, what only a human can verify, and the
traps that have already caused real bugs. It is not a log. The per-feature record and the reasoning behind each decision is
`docs/architecture/session-and-state.md`. The code is the source of truth;
when this file and the code disagree, fix this file.

## The short version

- **TUI** (`gitpoulpi` / `gp` with no subcommand): commit graph (pixel picture
  through iTerm2 or Kitty graphics, text fallback), Refs sidebar (local, remote,
  worktrees, stashes, tags, submodules, pull requests, issues), Log with
  resizable/reorderable columns, search and multi-select, central Diff / File
  View / Blame / History / built-in editor / image view / three-way merge
  editor, working-changes pane with hunk and line staging and a commit
  composer, command palette, Preferences, themes, 17 UI languages, undo/redo,
  repository tabs and a Repository Management (`⌂`) tab.
- **CLI**: every `GitBackend` operation as a `gitpoulpi` subcommand with
  `--json`, through the session daemon like every other frontend.
- **MCP** (`gitpoulpi-mcp`, ADR 0007): one tool per `gitpoulpi-commands::cli::Cli`
  leaf, generated from the clap tree and run through the same `execute` as the
  CLI, plus `ui_*` tools that let an agent show things in a running TUI and
  propose changes for the user to confirm.
- **Hosting**: GitHub, GitLab (incl. self-managed), Azure DevOps (+ Server),
  Bitbucket Cloud (+ Data Center), Gitea / Forgejo (incl. Codeberg), Jira,
  Trello, Linear, YouTrack as personal-token integrations:
  PR/MR lists, create, checkout, issues, clone from an account, SSH keys.
- **AI** (off by default, `gitpoulpi-ai`): commit messages, compose commits,
  explain, merge suggestions, through an API or a local agent CLI.
- **Git LFS** (ADR 0009) through the system `git-lfs`; a missing program is a
  typed error (`Error::LfsUnavailable`) with the install link.
- **Updates and install channels**: the marker's `channel=` says how GitPoulpi
  was installed (`gitpoulpi_config::release::InstallChannel`); `gitpoulpi-update`
  resolves it and plans the update per channel, the title-bar chip opens the
  update panel (`tui/updates/`), and the new crate `crates/gitpoulpi-updater`
  (`gitpoulpi-updater`) does the swap or runs the package manager after the TUI
  exits. A machine-wide policy (`gitpoulpi_config::policy`) can lock it. User
  pages: `docs/user/updates.md`, `enterprise-distribution.md`, `docker.md`;
  release side: `docs/developer/releasing.md`. Open: the Windows hand checks
  (UAC swap, Restart Manager, `choco upgrade`, revert) under "Things to verify
  by hand".
- **Custom builds**: cargo features per language/integration/feature
  (`docs/user/custom-builds.md`); a feature that is off is a `*_stub.rs` module.

## Where the TUI's code lives (`crates/gitpoulpi-tui/src/`)

Each top-level module starts with `use super::*;` and `lib.rs` glob-imports it
back, so they share one namespace; items crossing a module line are
`pub(crate)`. Unit tests sit in each module's own `tests` (a sibling `tests/`
directory or `*_tests.rs` for the big ones); shared fixtures in
`test_support.rs`. PTY tests (the real binary in a pseudo-terminal, driven by
`xtask/src/pty/`) are in `crates/gitpoulpi-tui/tests/pty_*.rs`.

**Entry and loop**
- `lib.rs`: `pub fn run` (the loop over repository windows, on a UI thread with
  a 64 MiB stack), the `hide_in_graph_icon!` macro, the module list.
- `event_loop.rs` + `event_loop/`: `run_in_repo` builds one window's
  `init::WindowState` and runs `tick -> draw_frame -> drain_input ->
  after_input -> apply_outcomes` (`window/`). `init/` (state setup),
  `input/` (key and mouse handlers, the command layer `input/commands`),
  `draw/` (one file per pane, each taking a `...DrawCtx`), `window/`
  (`drain.rs` the input drain, `idle.rs` redraw on demand, `apply.rs`,
  `agent_*.rs` for MCP view intents), `hover.rs`, `open_menu.rs`.
- `redraw.rs` (`poke`, `send_notify`), `quiet_backend.rs` (writes nothing when
  nothing changed), `exit_log.rs` (every exit leaves a record), `live_settings.rs`
  (process-wide statics read live), `activity.rs`.

**Frame, layout, shared widgets**
- `chrome/`: focus and central-view enums, toolbar, `HitTarget` and
  `find_hit_target` (`hit.rs`), `HELP_TEXT`, count-prefix and `Ctrl+N` badge logic.
- `ui_kit.rs`: `ClippedRender`, `anchored_popup`, `display_width`, truncation,
  `row_to_index`, list scroll/scrollbar helpers, loading and notice messages.
- `pane_layout.rs`, `pane_split.rs`, `sidebar_layout.rs`: pane split, dividers,
  collapse, `resize_grab_at`, persisted sizes.
- `button_bar.rs` (`fit_bar`, the `⋯` overflow), `context_menu.rs`, `combo.rs`,
  `dialog_kit.rs`, `picker_view.rs`, `tooltip.rs` (the only tooltip owner),
  `scrollbar.rs`, `scrollbar_repeat.rs`, `scroll_marks.rs`, `minimap.rs`,
  `cursor.rs` (OSC 22 pointer shape), `clipboard.rs` (OSC 52).
- `text_edit.rs` (one caret/selection core for every field), `text_field.rs`
  (drawing a field), `text_select.rs` (selectable text in read-only views).
- `theme/` (roles, presets, `th()` runtime, insta snapshots), `color_depth.rs`,
  `terminal_bg.rs`, `theme_picker.rs`, `language_picker.rs`, `terminology.rs`.
- `keymap/` (one registry of key -> action per context), `kitty_keyboard.rs`.

**Log and graph**
- `graph/`: layout into `TreePart`s (`part.rs`, `layout.rs`), the raster
  picture (`raster*.rs`, `inline_image.rs`), stash rows and the display order
  `LogRows` (`stash.rs`), lazy paging (`paging.rs`), ref connector, cache.
- `log_table/`: commit rows, columns, the Log row arithmetic (`geometry.rs`),
  the WIP row (`wip.rs`). `log_pills.rs`, `log_menu/`, `pill_menu.rs`,
  `log_search.rs`, `log_multiselect.rs`, `log_inline_ref.rs`.
- `avatar/` (sources, cache, per-row and detail pictures), `terminal_image/`
  (iTerm2 and Kitty protocols, detection), `image_decode/`.

**Sidebar, status, commit**
- `refs_sidebar.rs`, `refs_tree.rs`, `ref_menu/`, `ref_drop.rs`, `submodules_ui.rs`,
  `fold_state.rs`.
- `status_pane.rs` (row model), `status_list/`, `status_optimistic.rs`,
  `composer.rs`, `compose_ui/` (AI compose), `commit_detail.rs`,
  `commit_files_cache.rs`, `wip_view.rs`.

**Central views**
- `diff_view/`, `diff_actions.rs` (hunk/line stage and discard), `diff_content.rs`
  (whole-file mode), `file_view.rs` + `file_panel.rs` (File View / Blame /
  History in the centre), `blame_view/` (the Blame view: hunks, layout, paint, card/menu, ops;
  `history_view/` (the File History split: paged list, selection/range, diff spec, rename merge; its diff is the ordinary
  Diff view kept as a `Compare` state keyed `HISTORY_KEY`; `rename.rs` merges a rename commit's two files into one;
  `render.rs` paints the two-line rows (`ROW_H = 2`) and the Diff/File/Blame control in the right box's border;
  per-tick work in `event_loop/window/history.rs`, input in `event_loop/input/commands/history.rs`),
  state in `FileInsight::blame`, actions in `event_loop/input/commands/blame.rs`), `editor/` + `editor_buffer.rs` + `editor_highlight.rs`,
  `image_view/` + `image_editor.rs`, `merge_view/` + `merge_flow.rs`.
- Banners over the Log's top row: `rebase_banner.rs`, `operation_banner.rs`,
  `autostash_banner.rs`, `stuck_banner.rs`. `rebase_ui/` (interactive rebase and
  cherry-pick plan), `reword_ui.rs`, `undo_ui.rs`, `safety_panel/` (Undo history, Reflog and Lost work panel: an IPC client over the `Requester`/`Reply` of `daemon_link/raw.rs`, a `RepoDialog::Safety` in `repo_dialogs.rs`; menu rows open it through `safety_panel::ask_to_open`).

**Repositories, tabs, dialogs**
- `tabs.rs` (tab effects), `repo_switcher.rs` (tab bar, `RepoTab`),
  `repo_manager/` (the `⌂` tab, pure), `repo_menu.rs`, `repo_dialogs.rs` (owns
  the Browse / Init / Clone / Worktree / patch dialogs), `folder_picker/`,
  `folder_listing.rs`, `init_dialog.rs`, `clone_dialog/`, `worktree_dialog.rs`,
  `clone_task.rs`, `init_task.rs`, `repo_probe.rs`, `session_store.rs`,
  `open_in_bar.rs` + `tab_dir.rs` (Open in, follow the tab's folder).

**Host-driven flows (slice W4)**: hosting half in `gitpoulpi-hosting/src/flow_engine.rs` (+ `gitlab_flow`, `github_flow`, `gitea/flow`), CLI in `hosting_cmd/flow_issue.rs`, pick-up is `FlowAction::PickUp` of the W2 planner, TUI in `flow/issue_flow.rs` and `flow/issue_watch.rs`; the daemon never talks to a host.

**Branching workflows (slice W3, the branching-workflows design)**
- `flow/`: `view.rs` (the daemon's `Workflow` report kept process-wide; grouped sidebar rows, branch tooltip), `source.rs`
  (keeps it fresh), `cmd.rs` (`FlowCmd`: the `Flow ▾` menu, palette and `Ctrl+G W` run the same command), `form.rs`
  (Start form logic), `dialog.rs` (Start / Finish / Sync / Promote / Set parent as a `RepoDialog::Flow`), `actions.rs`
  (`FlowActions`, the seam to W2's daemon requests, with a recording fake: `GITPOULPI_FLOW_FAKE`).
- The grouped LOCAL / REMOTE rows come out of `refs_tree::ref_display_rows` like every other row list; a new row source
  must go through it. `preferences/workflow_page.rs` is the Settings page; `palette/flow.rs` the generated
  `Flow: Start <Type>...` rows.

**Editor text format, file-list menus, Open with, palette (2026-10-09/10)**
- `editor_buffer.rs` (`Doc`) holds a *format model* next to the text: encoding
  (`Charset`, with BOM), line-break kind and indentation, so a save writes the
  bytes back exactly (the buffer's own text is always Unicode with `\n`). Decode
  never substitutes U+FFFD and encode never writes `?`: both report instead.
  `editor_charset.rs` is only the TUI's vocabulary (labels, menu rows) over
  `gitpoulpi_core::textconv`, which is the *same* pure code the daemon uses for
  the Convert menu on files on disk and `gitpoulpi convert`. Not git's textconv
  diff driver (the cli backend passes `--no-textconv`).
- `editor/status_menu.rs`: the Encoding / Line endings / Indentation chips of the
  editor bar as `Combo` popups (never click-to-cycle). While one is open it owns
  keys and pointer (`Editor::menu_mouse` plus the event loop's mouse prelude).
- `event_loop/window/follow.rs`: `editor.follow_selection`. One pre-frame check
  (editor's file != the view's file) instead of a hook per selection path, so a
  click, Enter and the palette all follow; the view underneath is already the
  new file's, which is why closing the editor lands on it.
- `file_selection.rs` (multi-select of the right pane's file lists, the Log's
  gestures; a path set kept *beside* each list's cursor, never instead of it) and
  `pane_menu.rs` (pure rows of the commit-file / View-all-files / folder /
  empty-area / "N files" menus and the shared Convert submenu).
  `event_loop/input/commands/pane.rs` runs them, so click, `.` key and palette
  cannot drift.
- `binary_hint.rs`: process-wide cache so a menu can say "Treat as Binary/Text"
  without waiting on the daemon (attributes decide, not content: fed by the
  daemon's `BinaryPaths` answer, local NUL sniff until then).
- `open_with/` (+ `gitpoulpi-config` `os_apps/`: `linux.rs` from `mime/globs2`,
  `mimeapps.list` and `.desktop` files; `windows.rs` from the registry; macOS the
  default only): the "Open in" list of a binary file. The OS query runs on a
  background thread once per extension (a registry query is hundreds of ms on
  Windows); the parsers take the file system / registry as a parameter and are
  tested on fake machines. A commit's version opens as a read-only temp copy.
- `gitpoulpi-core` `domain/rules.rs`: where an ignore / attribute rule may be
  written and the exact line (`src/a/b.txt` becomes `/a/b.txt` in
  `src/.gitignore`). Shared by both backends, the TUI's "which file" picker and
  the CLI, so they cannot disagree.
- `palette/settings.rs` + `preferences/palette_settings.rs`: the `Settings: ...`
  palette rows, generated from the Preferences field model (`row_pool`), applied by
  the page's own commit functions on a *detached* state. Never list one by hand.
  `palette/bindings.rs`: a palette command with no key action gets
  `Action::Palette(i)`, config `[keybindings.palette] <command id>`; the id is
  derived from the variant and payload, so renaming a command id breaks users'
  bindings.
- `crates/gitpoulpi-tui/build_crates.rs` (called from `build.rs`): the About
  window's crate list, `name version url licence` per line from `Cargo.lock` and
  the local cargo registry (no network). Url = manifest `repository`, else the
  crates.io page; offline or missing manifest degrades to `?` / crates.io.

**Daemon link, settings, hosting, AI**
- `daemon_link/`: the IPC client thread (`link_loop.rs`, `reader.rs`,
  `refresh.rs`, `workers.rs`, `handle.rs`), `Action`s, outcomes.
- `preferences/` (pages, fields, persistence, search), `integrations_page/`,
  `integrations_ssh.rs`, `live_settings.rs`.
- `hosting_task/` (worker), `hosting_ui/` (sidebar sections, chip, menus),
  `pull_request/`, `issue_form.rs`, `issue_ref_picker.rs`, `issue_tokens.rs`,
  `remote_links.rs`, `remotes_ui/`, `git_actions.rs`.
- `ai_ui.rs`, `lfs_ui.rs`, `ui_intent/` + `ui_viewer.rs` (MCP view control),
  `external_tools.rs`, `new_file.rs`.
- Benches (`#[ignore]`d): `commit_files_bench.rs`, `open_time_bench.rs`.
  Lints: `source_scan.rs` (no hard-coded colours, UI text extracted).

## Working end to end

All phases of `docs/roadmap.md` up to and including Phase 2 are built: TUI,
CLI, MCP, hosting, interactive rebase. The operation-by-operation coverage
across contract suite / daemon / TUI / CLI / MCP is the frontend surface
matrix at the end of `docs/architecture/git-backend-trait.md`. User-facing
behaviour is documented under `docs/user/` (index `docs/user/README.md`).
Phase 3 (hardening, Windows validation, re-enabling CI, packaging) is what is
left; no release is being published for now.

## What to pick up next

Nothing is in flight; next are the hand checks (below) and the Windows gate
`pwsh scripts/check.ps1 -Features`. Open items carried by earlier entries and
not closed since:

- Stuck states: reconcile happens only on dispatched requests, not on
  cache-served reads; the rollback-after-failed-undo path and `conflicts skip`
  on an `am` patch are untested.
- A `--no-image-view` (feature-off) build has 3 pre-existing `preferences`
  lib test failures (submodule, Integrations, ja git terms).
- CLI `rebase --plan` cannot request a cherry-pick plan (`cherry_pick` is only
  reachable over IPC/TUI); no "rebase HEAD onto here"; no dragging the Log
  multi-selection onto a branch.
- Scrollbar arrow repeat keeps ticking at the end of a list; holding the track
  does not repeat.
- Interactive rebase popup and banners have no insta snapshot (buffer tests only).
- Windows: several tests failed on a clean HEAD on 2026-10-06 (`pty_paste`,
  `pty_kitty_keyboard auto_pushes`, `pty_undo` tooltip, `pty_home_shortcuts`
  `Ctrl+,`, `xtask build_custom_script`); rerun the Windows gate before
  trusting them.
- Load-only flakes (pass alone): `pty_log_cache`, the pty highlight band test.

### Decisions the maintainer took, so nobody re-opens them

- **The commit graph is drawn, not composed from glyphs.** The maintainer's words:
  *"Il ne faut pas utiliser des glyphes tout faits qui ont des marges -- il
  faut dessiner le tout."* The answer is the rasterised Graph column (iTerm2 /
  Kitty), with `○ ● ◎` circles as the text fallback. Rejected, do not
  revisit: `│` + U+20DD (sheared the column on Windows), a rounder code point
  (`⬤ ⭘ ⌽ Φ ◉ ◯` measured; best 7 px -> 4 px gap), sub-cell mosaics (braille,
  sextants, octants missing from JetBrainsMono NFM, Consolas, Lucida Console),
  and the lane painted as a cell background (gap-free but read as *"ce
  background derrière les commits"*). Details and font tables:
  `docs/developer/commit-graph.md`.
- **`general.default_backend = "git2"`** is the default and stays configurable
  (config file, `GITPOULPI_*`, `--backend`).
- **Force push is only ever `--force-with-lease`**, in the backend, the CLI and
  the TUI. A diverged push offers Pull (merge) then push / Force push (with
  lease) / Cancel. Fetch is every remote, pruned unless `general.auto_prune` is turned off (it is on by default).
- **The default remote is per repository** (`repo-state.toml`), unset = HEAD's
  upstream remote, then `origin`, then the only remote
  (`gitpoulpi_session::resolve_default_remote`).
- **Reword re-creates commits with the same trees, never a rebase**: no
  conflicts, merges kept, index and working tree untouched. A pushed commit
  asks twice.
- **Discard by folder / Discard all act on Unstaged paths only**; silently
  unstaging first would be a surprising double action.
- **Tokens never go in a file**: OS credential helper, else memory for the
  session. No encrypted secrets file.
- **No test touches the network.** Hosting, avatars and AI go through recorded
  or fake transports; a live test must be opt-in behind an env var.
- **Every closing of a tab asks**, as requested (the confirmation-fatigue
  alternative -- ask only with uncommitted changes, undo otherwise -- was noted,
  not built).
- **No external directory-picker registry** until a second implementation
  exists; notes on what a yazi / OS-dialog picker must solve are in
  `dir_picker.rs`'s header (yazi needs `--cwd-file`, not `--chooser-file`).
- **Every visible control works or is visibly disabled with a reason.** No
  inert checkbox, no "not wired yet" setting: a Preferences row is `live` or
  `coming(reason)` (`every_editable_row_is_live_or_explicitly_coming`).

## Known issues - real, reproduced, deliberately not fixed

- **A discard confirm holds the hunk id / line indices chosen when it opened.**
  If another client changes the file meanwhile, `y` acts on what those ids name
  now; in practice the patch fails to apply and nothing is written.
- **`PullOutcome` has no `Rebased`**: a rebase pull reports `Merged`.
- **Reword leaves other local branches and tags on the old commits** (like
  `git rebase` without `--update-refs`); only HEAD's branch moves.
- **File History has no compact one-line row setting** (two lines per commit, `ROW_H = 2`, is fixed).
- **Pixel avatars in the History and Blame are unchecked in a graphics terminal**
  (initials badges are covered by tests; the picture path only by the Log's).
- **Blame through a merge's second parent may differ between the backends**
  (git2 vs the system `git blame`); not pinned by a contract test yet.
- **Stash rows are not drawn while soloing** (`stash_rows_shown`): the solo
  paths index the unfiltered history.
- **A detached HEAD's WIP marker has no connector**: the TUI cannot tell which
  commit a detached HEAD is.
- **The Menu/Apps key never arrives under classic conhost**: crossterm's
  Windows backend has no `VK_APPS` arm, and the kitty keyboard protocol is
  Unix-only here. `Shift+F10` works everywhere and is the documented key.
- **`Ctrl+Enter` arrives as plain `Enter` on many terminals.** Commit chords
  also accept `Alt+Enter`, `Ctrl+J` (and `Ctrl+S` in the rebase form).
- **On Windows the cell size is a guess (9x21)**: `window_size()` has no
  Windows implementation. Pictures are supersampled 2x;
  `GITPOULPI_TERMINAL_CELL_PX=<w>x<h>` overrides.
- **Fork pull requests cannot be checked out on Azure DevOps and Bitbucket
  Cloud** (refused with the reason).
- **Through ConPTY the pty harness cannot send `Alt+key`**: the ESC prefix
  becomes a bare `Esc`, so `Alt+Up/Down` in the rebase form is unit-tested only.

## Things to verify by hand

- **Terminal compatibility pass (November 2026)** (`docs/user/terminals.md`, `docs/user/multiplexers.md`; raw probes and GitPoulpi runs, results kept in the private notes): on a real desktop with a GPU: (1) Windows: Rio, WezTerm official and mintty with `ui.graph_renderer = "image"` (graph rows anchored right or wrong), and a two-line paste into the commit Summary and the new-branch prompt in Windows Terminal, WezTerm, Rio, Alacritty, Tabby (no prompt submitted). (2) Windows: a WezTerm build with upstream PR #8092: hover a divider and a button (hand, resize arrows). (3) macOS with a GPU: kitty, Ghostty, WezTerm, Rio and Warp are **not measured** in the docs; run the probe and the GitPoulpi start there and fill the cells. (4) macOS iTerm2 3.7.7: graph without the "Allow Terminal-Initiated Display?" dialog, copy after switching on the clipboard permission, `Alt` chords with Left Option key `Esc+`. (5) Linux: Warp (needs XWayland), foot/Rio/Alacritty `OSC 52` copy with a focused window, Tabby GitPoulpi run, herdr GitPoulpi run, WezTerm panes. (6) tmux with the documented config inside kitty and foot: pointer shapes and graph; inside Ghostty the wrapped `OSC 22` is not shown (terminal). (7) Copy a short SHA over `ssh` into the terminal (`OSC 52`) in kitty and Windows Terminal. (8) Windows: classic console double-click still reopens in Windows Terminal; started directly by WezTerm, Rio, Tabby or `wt.exe` it stays.
- **CLI `repo-auth test --push` with a `sh.exe` token command, no TUI open (Windows)** (the helper reaching an unpublished embedded daemon is CLI- and daemon-tested on loopback; the real `sh.exe` path with spaces is not): in a private HTTPS repository, `gitpoulpi repo-auth set --https token-command --yes --hosts github.com --command "C:/Program Files/Git/bin/sh.exe" <path>/token.sh`, close every GitPoulpi window and MCP server of that session, then `gitpoulpi repo-auth test --push` and `gitpoulpi fetch` pass (no `AuthFailed`, no "terminal prompts disabled"); afterwards `%APPDATA%` holds no `sessions/*.cred` file. If the helper ever cannot reach the daemon, the failure reads `CredentialHelperUnreachable`, never `AuthFailed`.
- **Updater and install channels** (`crates/gitpoulpi-updater`, the update panel; pending on Windows; the logic runs against fakes in the Linux tiers): on Windows (1) an all-users install: **Update and restart** shows the UAC prompt, other users' GitPoulpi processes close, the new version opens in a new window and the status line says `Updated to v...`; decline the prompt and see `Update canceled ... Nothing changed.` (2) The Restart Manager really lists and closes a process of another user that holds an install file. (3) `choco upgrade gitpoulpi` and `winget upgrade` re-run the setup and the marker keeps its channel (and `feed=` for `choco-internal`, which needs Chocolatey to remember package parameters). (4) **Revert** after an Inno user update and after an all-users one. (5) The Inno uninstall key's `DisplayVersion` follows an update. On Linux and macOS: Homebrew and the `.deb` marker land next to the binaries (`channel=homebrew`, `channel=apt`).
- **Terminal run mode of local agents** (`docs/user/ai.md#run-mode-headless-or-virtual-terminal`; the client, the profile rows and the
  Generate button are tested against a fake interactive agent, no real agent was ever started): for each of Claude Code, Codex and
  Gemini, in a scratch repository with a staged change: (1) Settings > AI > profile > Run mode = Interactive; press **Open the
  agent...**: your terminal opens the agent in the working folder; answer its trust / login question once, close it. (2) Press the
  Generate sparkle: the composer fills with a commit message and nothing else happens on screen. Check that no stray `<<GP-...>>`
  marker is in the message and that a long answer is complete. (3) With a folder the agent has never seen, Generate fails with
  "Use Open the agent..." instead of hanging. (4) Esc during a request closes the agent (no leftover `claude` / `codex` /
  `gemini` process). (5) Windows: repeat once (ConPTY).

- `gp -a` / `gpa` (`docs/user/cli-reference.md#add-a-folder-to-the-open-window`; the tab, the question and the exit codes are
  PTY- and CLI-tested, the window coming forward is not): (1) Settings > Launcher: type `gpa` in **Add-tab alias**,
  press **Create**, open a new terminal in another repository's folder with GitPoulpi running elsewhere, type `gpa`: the
  running window shows a new tab for that folder and comes forward (Windows: foreground; WezTerm; Hyprland: focus), the terminal
  prints one line and returns. (2) With two GitPoulpi sessions open, `gpa` lists both with their tabs; `2` + Enter joins the
  second, Enter alone cancels. (3) With none open, `gpa` starts GitPoulpi on that folder. (4) On Windows check that `gpa.cmd`
  runs from PowerShell and `cmd` (a name PowerShell already uses, like `gp`, still loses to its built-in alias).
- Number-and-unit rows (maintainer, 2026-10-22): the same `[ n ] [unit v]` line as the retention for six more settings. Settings >
  Synchronization: "Auto-fetch interval" (minutes or hours, 0 = off; a fresh config reads `5 minutes`; `auto_fetch_interval_minutes = 60`
  by hand reads `1 hour`), "Script answer time limit" (milliseconds or seconds, `5 seconds` fresh) and "Script time limit per
  pass" (seconds, minutes, hours, `1 hour` fresh). Integrations: "Pull Requests and issues refresh" (minutes or hours, up to 2 hours).
  General: "Maximum size of a log file" (MiB or GiB, `5 MiB`). Images: "Maximum preview size" (MiB or GiB, `32 MiB`). For each: type
  a number, the key keeps its own base unit (`script_timeout_ms = 2000`) and `<name>_unit` records the shown unit; pick the larger
  unit with a big number typed and it is cut to the most that fits (GiB: 1); a number above the range stays red with the reason
  under the row. An older config without the unit keys loads and shows the largest even unit, and looking writes nothing. The
  palette's "Settings:" entry for these rows opens the page. Not checked by hand: a wide-glyph language, a narrow window.
- Expand/collapse all folders (maintainer, 2026-10-22; `docs/user/mouse-and-gestures.md`, `tree_fold.rs`): (1) In TAGS (or
  LOCAL/REMOTE) with `a/b/c` names, right-click the section header or the empty space under its rows: **Expand all**,
  **Collapse all**, **Show/hide sections...** (leads to the checklist). (2) Right-click a folder row: **Expand all
  subfolders** / **Collapse all subfolders** act on it and everything below, not on its siblings. (3) Keys in the Refs pane:
  `=` expand all, `-` collapse all, `*` / `Shift+Right` and `_` / `Shift+Left` on the folder under the cursor; the same four
  on Unstaged, Staged, View all files and a commit's changed files (Tree mode), and the palette's *Refs:* / *Files:* rows.
  (4) Fold a folder, filter on a name inside it: the match is shown, the folder open; clear the filter: folded again; quit
  during the filter and restart: your fold is still saved. (5) A remote's own row (`origin`) has no fold entry in its menu
  (its menu is the remote menu); the keys and palette rows work on it.
- Log retention as number + unit (maintainer, 2026-10-22): Settings > General, "Keep log files for" shows `[ 1 ] [week v]` on one
  line (a fresh config). Click the number, type 2, Enter: it reads `2` `weeks`; click the unit: a list opens (no cycling), pick
  months: `2 months`. `config.toml` holds `retention_days = 60` and `retention_unit = "months"`. Keyboard: Enter types, Up/Down
  step while typing, Tab commits and moves to the unit, Enter opens its list, BackTab goes back. Type 0 or 99999: the field stays
  red with the reason under it and nothing is written. An old config with only `retention_days = 90` shows `3 months`; 45 shows
  `45 days`. Pick years with 60 typed: it becomes 10 years. Not checked by hand: a wide-glyph language, a narrow window.
- Settings multi-line editors (Conflict prevention > Base branches, Sparse paths, template description, the Workflow hooks):
  click puts the caret on the clicked character (also on a wide glyph or after a tab), drag selects (past the edge selects to
  it), double-click selects a word, Shift+click extends from the caret, the wheel scrolls a long list; then `Ctrl+C`/`Ctrl+X`/
  typing/`Delete` act on the selection. Buttons (Save, Cancel) still click normally.
- Where a tag lives, and the tag Delete scope (maintainer, 2026-10-22; `docs/user/log-and-history.md` "Where a tag lives"):
  (1) On a repository with a remote, create tags `a` (push it), `b` (never push) and fetch: TAGS shows `a` with the
  left-right marker and `b` with the up arrow; hover each for the sentence. (2) Right-click `a` > **Delete a...**: the
  three numbered rows; `1` deletes locally and the footer says the remote still has it, **Fetch** brings it back. (3) Same
  with `3` then `y`: both gone, **Fetch** does not bring it back, **Ctrl+Z** restores `a` locally (up arrow) and
  **Push** puts it back on the remote. (4) `git tag -d a` behind its back while the app is open: `a` turns into the dimmed
  down arrow with **Fetch a from origin**. (5) Move a tag locally (`git tag -f a HEAD~1`) after pushing: the up-down marker.
  (6) In Settings > Sync turn on **Auto-prune tags**, delete a pushed tag on the remote (`git push origin --delete a`),
  fetch: the local `a` goes too, an unpushed `b` stays. Not checked by hand: Windows, a remote with several remotes' rows.
- Clone a repository, `j`/`k`/`g`/`G`: with the focus on **Clone from** (Alt+S) `j`/`k` move the highlight like Down/Up and
  `g`/`G` go to the first/last source; on the repository list (Tab into it) the same keys move its cursor. In the search box and
  the URL field `j`/`k` are typed letters.

- `.mailmap` and the Identities tab (S16, `docs/user/history-rewrite.md`): (1) on a repository where one person committed under
  two emails, **Rewrite: Merge Identities (.mailmap)...** opens the workbench on the Identities tab: every pair with its counts,
  already-mapped ones say `mapped to ...`; with a long list the cursor row's counts show on the last line. (2) Tick two, **Merge
  into...** (or `m`): the dialog lists them numbered, the one with most commits first; `Esc` writes nothing; `1` writes the
  `.mailmap` (`git status` shows it modified, nothing committed) and the status line says so with **Undo**. (3) Close the
  workbench: the Log's Author column and blame show the canonical name; Settings > Log & graph > *Show authors through .mailmap*
  off shows the stored ones within 2 s. (4) Ctrl+Z (or **Undo** / `u` on the tab) puts the old `.mailmap` (or no file) back. (5)
  Preview an author rule on a pushed branch: the Summary offers **Use .mailmap** (`m`); it opens the tab with the matches ticked.
  (6) Edit `.mailmap` in an editor: the Log follows without a restart. Not done: the Global search index shows stored names.

- Tab picker (`Ctrl+N t`): with many tabs open and a late one active, the highlight starts on the active tab (scrolled into view),
  Up/Down/Alt+Up/Down move from there; typing a filter jumps to the best match; a click outside closes it.
- History rewrite workbench (S8, `docs/user/history-rewrite.md#the-workbench`): (1) `Ctrl+G Shift+H` opens it over the Log; the
  cross, Esc and a click outside (empty recipe) close it, a click outside with rules only flashes the cross. (2) Add an identity
  rule with the mouse (Add rule row, then a click on a field with several values opens a list, never cycles), fill the emails,
  F5 shows the progress bar then the Summary; the Commits / Branches and tags / Impact tabs; Apply, then Undo in the result panel
  puts the branch back and closes the workbench. (3) On a repository with a pushed protected branch the Branches and tags tab
  lists it unticked with Check all / Uncheck all. (4) A signed tip: F5 asks about the signatures (three choices, no default). (5)
  Right-click a commit > Rewrite history > From this commit to HEAD..., and multi-select 3 commits > Rewrite history > Selected
  commits.... (6) Narrow the window under 56 columns: the workbench says to enlarge it. Not done in S8: the blob diff drill-in of
  a Commits row, `Enter` / double click on a row to show it in the Log, the Push / Copy note / Purge buttons of the result panel
  (slices S13 / S15).

- History rewrite push and Secrets (S13 / S14 TUI, `docs/user/history-rewrite.md#pushing-the-result`): (1) on a repository with a
  remote, rewrite a pushed non-protected branch (Apply asks "Rewrite pushed history?"): the result panel lists it with a box, a
  pushed tag too, `main` unticked with *protected* and Check all / Uncheck all. Untick the tag, **Push rewritten branches**: the row
  says *pushed*, the remote has the branch rewritten and the tag untouched (`git ls-remote`). (2) Rewrite, then let a colleague (or
  a second clone) push to the branch before you click Push: *rejected: the remote moved*, nothing overwritten; **Fetch and review**
  closes the workbench and fetches. (3) **Copy the note** then paste in an editor: your language, the git commands in English.
  **Open the commit map** shows two paths; a click copies one. (4) Undo after a push: the journal's confirmation says the remote
  keeps the new history. (5) Secrets tab: **Scan for secrets** on a repo with an old token: the progress bar says *Scanning*; the
  row shows `ghp_****abcd`, never more; counts of skipped binary files; **Remove from history (n)** adds a rule, Preview shows the
  Summary, Apply removes it (`git log -p | grep` finds nothing), Undo brings it back. Rewrite something else and come back to the
  tab: it says the findings are old. Not done: pushing a tag to a remote whose tag mirror is missing (the backend skips it), a
  cancel of the push, the host-side checklist after a secret (S15).

- History rewrite purge screen (S15 TUI, `docs/user/history-rewrite.md#the-purge-screen`): (1) rewrite a folder out of history on
  a repository that has a stash and an `origin`; in the result panel **Purge old history...** (or `e`) opens the checklist: the
  entries, the counts, the stash and the remote-tracking branch as rows with **Drop** / **Fetch** and **Copy name**; **Purge** is
  greyed and `p` does nothing. (2) **Drop** asks "Drop stash@{0}?" with numbered answers and no default; Enter alone does nothing;
  `2` keeps it. After `1` the row is gone without a click on Check again. (3) **Fetch** clears the remote-tracking row once the
  remote has the rewritten branch. (4) When the list is clear **Purge** opens "Purge for good?" naming the operations that lose
  their undo; `1` runs: the step names change, "cannot be stopped" is shown, no button reacts, Esc and the cross do nothing; the
  outcome says the old history is gone; `git cat-file -e <old blob>` fails. (5) Close the screen, Ctrl+Z: the status line says
  the operation is "(purged)" and explains why. (6) The palette row **Rewrite: Purge Old History...** opens the screen alone;
  its cross closes the workbench too. (7) Wheel and PageUp/PageDown scroll the host steps; **Copy link** copies the GitHub
  support address. Not done: the Undo history panel has no purge button (use the palette), and "Go to branch / worktree" is a
  Copy name button.

- History rewrite Script rule (S10 TUI, `docs/user/history-rewrite.md#the-script-rule`): (1) palette **Rewrite: Run a Script...**
  opens the workbench with a Script rule and its editor on the starter template; "OK: rewrite_commit found" appears about a third
  of a second after you stop typing, and the Parameters rows (one per `--! param`, label next to the field, a bool is a box, a
  number refuses a word) follow the header as you edit it. (2) Type a syntax error: the problem line shows under the script with
  its number in red in the gutter and **Test on selection** says to fix it first; **Preview** (after closing the editor) is
  refused with the line in the Preview pane. (3) Multi-select 3 commits in the Log (Ctrl+click) before opening: the button says
  "3 selected commits"; with nothing selected it tests the 20 newest. The table lists `abc1234  Author name: old -> new` and the
  script's `log(...)` lines. Cancel during a long test stops waiting at once (the daemon finishes the run in the background, it
  is bounded by the script limits). (4) **Save to file...** / **Load from file...** with the default
  `.gitpoulpi/rewrite/script.lua`; `../x` is refused. (5) The cross, a click outside the editor and `Esc` close it; Tab walks
  editor, parameters, file, buttons, results; F5 tests. (6) Build with `--no-default-features --features history-rewrite`: the
  Script kind is greyed in the Add menu with "not built into this version" and the palette row opens the workbench with that
  notice. (7) Shrink the window under 40 columns for the editor: it asks for more room. Not done: syntax highlighting, a click on
  the failing commit to reveal it in the Log, a daemon-side cancel of a running test (the test is a plain request).

- History rewrite, end to end (the S25 collapse; the user page is `docs/user/history-rewrite.md`, read it top to bottom first).
  (1) **Rewrite a pushed branch, push with lease, purge**: in a clone with a remote, a topic branch pushed to it, add an identity
  rule on it, Preview, Apply (it asks "Rewrite pushed history?"), then in the result panel tick the branch and **Push rewritten
  branches**: `git ls-remote` shows the new tip; make a colleague (a second clone) push to the branch first and the same push is
  rejected for that ref and overwrites nothing. Then **Purge old history...** (palette *Rewrite: Purge Old History...* also works
  after closing the workbench): a stash on the old history blocks it with the reason, drop it, purge, and `git cat-file -e <old
  blob>` fails; the operation is marked "(purged)" and undo no longer offers it. (2) **Undo with a dirty worktree**: after an
  Apply that changes files (a Remove files rule), edit an unrelated tracked file and Undo: it asks first, the old files come back
  and your edit stays; edit a file the old history also changes and the undo is refused with its reason instead of overwriting.
  (3) **Re-sign with ssh**: `gpg.format = ssh` and `user.signingkey` set, **Check signing** in the signature question (`4`)
  reports the time of one signature (unlock the agent first), choose *Re-sign only the commits that were signed*, and
  `git log --show-signature` verifies them; with a broken key the preview is refused with the reason before anything is read.
  (4) **Fix message encoding**: write a commit with `git hash-object -t commit --literally` carrying `encoding ISO-8859-1` and a
  Latin-1 message, add `RÃ©Ã©criture` as another message, run *Rewrite: Fix Message Encoding...*: the Commits tab shows the old
  and new text and an *encoding* row, a message that holds both `Ã©` and a real `é` is counted under "left as it is", Apply and
  `git log` shows UTF-8 without conversion. (5) **An agent's proposal**: `gitpoulpi ui propose rewrite --recipe r.toml` (with
  `allow_ui_propose`) opens the workbench over your window with the rules in it and the purple banner; waiting does nothing,
  `Esc` rejects (`ui state` says `rejected`), your own *Preview* then `Ctrl+Enter` applies (`accepted`); a recipe with a script is
  refused by the command.

- Split commit (menu *Split commit...*, `docs/user/log-and-history.md#splitting-a-commit`): (1) with a signing key configured and
  `commit.gpgSign = true`, split a signed commit: the *Sign commits* box starts checked, the new commits and the ones on top show
  as signed (`git log --show-signature`); unchecked, they are unsigned and the editor says so. (2) With a dirty worktree (staged and
  unstaged edits), split HEAD~1, then `Ctrl+Z`: the original commit is back and the edits are untouched. (3) In the interactive
  rebase popup press `t` / **split...** on a row, split, and land back on the popup on the new history; press split twice after
  editing the plan to see the discard confirmation. (4) `gitpoulpi ui propose split <rev> --plan plan.json` (with
  `allow_ui_propose`): the editor opens pre-filled with the banner, `Esc` rejects, `Ctrl+S` splits.

- Windows Terminal (1.22+), `ui.avatars = "graph"`: the Graph column is a picture (lanes, nodes with
  the author faces), aligned with the rows at 100 % and after Ctrl+scroll zoom (cell size is
  re-asked on resize), scrolling and moving the selection leave no ghost fragments, a menu or
  dialog over the column hides and restores it, and it feels as fast as in WezTerm. Also:
  Start menu shortcut with `launch_terminal = wezterm` opens one WezTerm window only.

- Windows launcher (never compiled when written): build `dist`, run the installer, pin the Start menu
  entry to the taskbar, click it: GitPoulpi opens in its terminal under the pinned icon with the
  "running" dot; click the pinned icon again: the same window comes forward (no second terminal, no
  console flash); with Settings > Launcher > "Show the open window on click" off, a click
  opens a second window. Minimized window: it is restored. Check the window title is back to normal
  a second after start. WezTerm and Windows Terminal both. If grouping does not happen, look for
  "window grouping" lines in the Activity Log.
- Linux (Hyprland + WezTerm / kitty / foot): the launcher entry opens one window; a second click
  focuses it; the dock/bar groups it under the entry's icon.

These are covered by unit, buffer and PTY tests only; no human has seen them
in a real terminal. Use
the release build, a throwaway `GITPOULPI_CONFIG_DIR`, your own `-s <session>`,
and a scratch repository.

**Background jobs, Cancel and the global log (background-ops B1-B13; only PTY, daemon and buffer tests so far)**
- Windows: clone a large repository from Home, close the dialog, switch tab and back: the `Clone: name` indicator keeps its progress, `[x]` stops it within a second or two, no `git.exe` / `git-remote-https.exe` / credential helper survives (Task Manager), the half folder is gone. Same for a fetch / push on a slow link, and a pull canceled during its fetch (HEAD, index, stash list untouched), then one canceled once it says "merging - cannot stop safely".
- Windows and Linux: start an AI request (real provider and `claude -p`), switch tab, come back: the answer is applied; cancel it by clicking `[x]` from another tab. Fetch all on a workspace of a few real repositories, switch tab, check the `Workspace: name` job in the panel and that Cancel stops it after the repository it is on.
- A clone that fails from Home (wrong URL, no permission): the card appears, its Details opens the global Activity Log on that entry (Home's status-bar Activity button, `Ctrl+N Shift+G` and the palette reach it too); the Scope box (click it, or `s`) narrows both tabs to a repository, a workspace or all of them; after a restart the operations list starts empty (only the Application tab keeps the old lines).
- Activity Log scope, real session: open it from a repository tab (it says `This repository (name)` and lists that repository's operations), then from Home (`All repositories`); pick another repository or a workspace in the Scope box by click and by `s` + arrows: both tabs follow; on the Application tab the lines of the other repositories vanish while start-up and daemon lines stay, marked `[app-wide]`; close and reopen from the same entry point: the last pick is back.- Sixel graph and the avatars on Windows Terminal with the indicator on screen: the indicator sits on the status line row, check it does not flicker the picture.
**In-app code review (slice R2; only the mock host and fixtures were used)**
- On a real GitHub and a real GitLab request: open it from the row (Enter), Files page, comment on one line and on a range (GitHub), Finish review with Comment / Approve / Request changes (GitLab: no Request changes), then look at the host: the comments sit on the right lines, a renamed file's comment too, a context-line comment on GitLab (both line numbers).
- A request with a few hundred files and a force-pushed head: the Files page stays responsive (diffs are asked per file), Refresh (`F5`) picks up the new head, pending comments survive and the "request changed" warning shows.
- Reply / Resolve on GitHub (GraphQL) and GitLab (discussions); the folded thread after Resolve; Esc, `Alt+1/2/3`, the palette rows (`Review: ...`) on a small terminal (the view needs about 56x14 cells).
- Radar `Review...` / `e` for a repository that is not open yet, and `Alt+R` in the workspace search for a result of another repository.
**Radar folder scopes (Default clone folder, Default browse folder)**
- Real `~/src` (or your clone folder) with a few dozen repositories, some linked to GitHub/GitLab and some not: pick the scope (`w`); the rows of the linked ones show with their pull requests and issues, the others only as work in progress, the status line says `N not linked` and each has a `Link...` line under REPOSITORIES. Typing stays smooth while the folder is scanned (the scan runs on a thread; a slow or network drive must not stall keys). Clone a repository from the Clone dialog, then `r` in the Radar: it appears (the scan result is cached until Refresh). With the browse folder empty in Settings the entry is greyed `(set it in Settings)` and picking it opens that Settings row.
**Local Markdown issues and Obsidian (slice L1)**
- Preferences > Integrations > Local Markdown > Add profile with Browse on a real Obsidian vault sub-folder; link a repository's ISSUES to it. Row menu "Open in Obsidian" really opens the note in that vault (Linux `xdg-open obsidian://`, Windows/macOS handler); outside a vault the entry is greyed with its reason. Create a Base from the palette and open `issues.base` in Obsidian: the table and the kanban view (grouped by status, in the configured order) must show the notes; if Obsidian rejects the file, fix `obsidian.rs::base_text` and the user page. Edit a note in Obsidian while GitPoulpi shows the list: the row changes within a second or two; close an issue from GitPoulpi: the file's `status` changes and Obsidian's kanban moves the card. A vault on a slow/network drive must not stall typing.
**Editor status chips (encoding, line breaks, indentation)**
- Open a real Windows-1252 file (a French `.txt` saved by an old Windows tool) and a Shift_JIS or UTF-16 one: the right encoding on the bar, the right text; Reopen with Encoding to fix a wrong guess; Convert to UTF-8, save, `file -i` agrees. A CRLF project file: Convert to LF, then the diff shows only line endings. Tabs to spaces on a real source file. Look at the dropdown anchoring on a narrow window and under the light themes.
**File lists of the right pane (menus, multi-select, Convert)**
- On a real repository: Ctrl+click / Shift+click / Alt+click in Unstaged, Staged, a commit's files and View all files (your terminal may keep Shift+click for its own selection: Alt+click is the twin); `Insert` / `Ctrl+Space` / `Shift+Down` from the keyboard; the menu key on a selection. Right-click a commit file: every row, the greyed ones with their reason, Restore then `Ctrl+Z`. Convert a folder of real CRLF files to LF and a Windows-1252 file to UTF-8, look at the diff, then `Ctrl+Z`. Wide terminals and the light themes: the 15-row Encoding and 12-row Indentation drill-ins on a short terminal.
**Open with for binary files (Windows registry and macOS never run by the author)**
- On Windows: a `.pdf`, `.docx` and `.xlsx` in the Diff view and the File View: the Open in list holds the user's choice first (bullet) then the registered programs (`UserChoice`, `OpenWithProgids`, `OpenWithList`); launching one opens the file; a commit's version opens as a read-only temp copy. On macOS only the default application is expected.
**Launcher shortcuts (Windows and macOS never run by the author)**
- Windows: Settings > Launcher: Create the Start menu and Desktop `.lnk` (PowerShell `WScript.Shell`, `-EncodedCommand`); check the target (`wt.exe "<exe>"`), the icon (`%LOCALAPPDATA%\GitPoulpi\gitpoulpi-<hash>.ico`, octopus in the logo colour; only the file of the current colour is left), Remove; pin to the taskbar by hand, then change the logo colour and check the pinned icon follows. Double-click the exe with `ui.launch_terminal = windows-terminal`: it must reopen in Windows Terminal and the first console close; from a shell it must stay.
- Windows, WezTerm: open the Command Palette (`Ctrl+P`) with the list scrolling; every row that has a key button ends in `[k]` (a plain button), no bar mark, and nothing from behind the popup shows next to the scrollbar. Scroll and filter to repaint.
- macOS: `~/Applications/GitPoulpi.app` opens in Terminal.app, iTerm2, WezTerm, kitty, Ghostty (written from Apple's layout; untested).
- Linux: the `.desktop` entry shows in the launcher (Omarchy: `omarchy-launch-walker`), icon in the user's logo colour.
**File manager menu, Open in GitPoulpi (`docs/user/settings.md#file-manager-menu-open-in-gitpoulpi`; writers unit-tested against temp dirs and a memory registry, the Linux Install/Remove PTY-tested with fake file managers, no real file manager or registry ever used)**
- Windows 10 and 11: Settings > Launcher > File manager menu > Install; right-click a folder, the background of an open folder and a drive (Windows 11: Show more options / Shift+F10): **Open in GitPoulpi** with the octopus icon opens that folder (a new tab of the open window when one is open and Show the open window on click is on, else the chosen terminal on it). A drive root (`C:\`) opens the drive, not `C:"`. Remove: the entries are gone and `HKCU\Software\Classes\Directory\shell` is left only if it held other verbs.
- Linux: Dolphin (KF6), Nemo, Thunar, GNOME Files and Caja: the entry on a selected folder and, where the file manager has one, on the background (GNOME Files and Caja: under Scripts); it opens the folder. Thunar with your own custom actions: they survive Install and Remove unchanged.
- macOS: the Quick Action appears under right-click > Quick Actions on a folder in Finder (after enabling it in System Settings > Keyboard > Keyboard Shortcuts > Services if needed) and opens the folder. If it never registers, ship the macOS row as coming soon (the `.wflow` format is undocumented).
**Sparse checkout (Windows never run by the author)**
- Clone with Sparse checkout ticked on Windows: `git clone --sparse` then `sparse-checkout set --cone` with `/`-separated directories; the footer `sparse` badge; the palette dialog's Apply / Disable on a real big repository (time to check out, in-flight text on the status line); a directory holding uncommitted changes must be refused (or kept) by git and the answer shown, never lost.
**From the 2026-10-08 resume entry (priority)**
- Stuck-state banners on a real repo: an auto-stashed pull whose pop conflicts,
  Start over, Skip, `git am`.
- Commit-detail avatar on Kitty and herdr (blank box on WezTerm was fixed);
  light-theme editor text; the new theme presets.
- **More providers** (`docs/user/hosting-integration.md` is the script), never
  run against a real account:
  - Azure DevOps Services (organization-scoped PAT; the global PAT shutdown is
    2026-12-01): `connectionData` name, invalid PAT reads as rejected (203 +
    sign-in page), PR list/paging/create/draft, every browser URL, `pr/<n>`
    checkout + Undo, fork refused, WIQL issues (custom state names show open),
    new work item, 429 / `Retry-After`. Server 2022+: collection URL,
    `api-version=7.0`, `ssh://` and `http://` remotes.
  - Bitbucket Cloud (email + API token, and workspace access token): workspace
    discovery, PR list with `q=`, `next` paging, states, create incl. fork, the
    undocumented new-PR URL, SSH keys, `X-RateLimit-NearLimit`, avatars.
    Data Center: Bearer token, `X-AUSERNAME`, `start/limit/isLastPage`,
    `refs/pull-requests/<id>/from`.
  - Jira Cloud (classic token + email): `search/jql` paging, 500 cap, JQL with
    quotes, new issue via `POST /rest/api/2/issue` (if Cloud refuses v2, wrap in
    ADF on v3), 429. Data Center 8.14+: PAT as Bearer, `startAt`, `myself.name`.
  - Trello: authorize link (`name=`, `expiration=never`), `Authorization: OAuth`
    header, board > 1000 cards cut not failing, new card, 429.
  - In the app: the Integrations page for all eleven kinds, the ISSUES picker
    (Jira JQL, Trello lists), the composer `#▾` picker (`Alt+R`), `Fixes ABC-12,
    see UTF-8` underlines only `ABC-12`. Native readers for ru, uk, tr, ko,
    zh-CN, pl, cs and the narrow Clone note.
- **Gitea / Forgejo** (slice G of the integrations-wave-2 design), checked live
  against local Docker containers (a Gitea and a Forgejo 11): token check,
  clone, SSH key add / delete, PR / issue lists and create (draft = `WIP:`),
  labels / members / milestones and edits, review show / threads / comment /
  reply / resolve (Gitea only) / submit / fetch. Not checked: Codeberg itself,
  Gitea older than the container, a server with a path prefix (`/gitea`).
- **Linear / YouTrack** (slice T1 of the integrations-wave-2 design) were never run against a real
  instance (fakes and a loopback mock only). With a key / token: `GITPOULPI_LIVE_LINEAR_TOKEN` or
  `GITPOULPI_LIVE_YOUTRACK_URL` + `_TOKEN`, then `cargo test -p gitpoulpi-hosting live_ -- --ignored --nocapture`;
  then by hand: link a team / project, list, create with a state, edit assignee / labels / cycle (Linear) or
  fix version (YouTrack), `hosting issue move`. Likeliest fixes are listed in the plan's "Not done".
- **Image view**: WezTerm and Windows iTerm2-protocol terminals (PNG diff,
  File View, merge card; sharp, inside its pane, follows a divider drag and a
  resize, hidden under popups), kitty / Ghostty (memory over 50 images:
  `terminal_image::write_retired` frees them), herdr with `kitty_graphics`,
  Windows Terminal half-blocks, Edit in presets per OS, merge card at 100 and 80
  columns, the Images page in a `--no-image-view` build; native readers for fr,
  ja, ko, zh-CN, tr.

**Earlier, still unchecked**
- Theme T7: Apple Terminal `auto` = 256; Windows Terminal `system` (no OSC 11,
  reads `COLORFGBG`, no start-up stall); Linux console = 16 colours; light
  kitty/WezTerm/Alacritty picks Light without losing early keys; tmux/ssh.
- Theme live preview, Light under kitty / WezTerm / Windows Terminal.
- Open in presets flagged `verified = false` in `gitpoulpi-config/src/open_in.rs`
  (Settings > Open in > Test on current tab): cmux, psmux, Warp, zellij, wezterm
  cli, alacritty msg, ghostty, xdg-terminal-exec, kitty remote, Konsole, GNOME
  Terminal, editors, JetBrains, Windows `wt` / `explorer` / `cmd` / `wsl` / Git
  Bash, macOS `open -a`; `${web_url}` on GitHub and self-managed GitLab; herdr
  "new workspace" (not run: it would switch the maintainer's view); Omarchy default
  `kitty --directory`.
- `gp` shell-init on bash / zsh / fish / PowerShell (Quit & cd here);
  `ui.tab_dir_follow = "auto"`; Terminal here with the real terminal.
- Merge conflict editor: full width and `F9`, sideways scroll in A/B/Base,
  banners at several widths, every card (binary, LFS, delete/modify, rename,
  symlink, mode, submodule, > 8 MiB), external merge tool, `Alt+PageUp/Down`
  under WezTerm / kitty / Windows Terminal, colours light and dark, agent
  proposals (`gitpoulpi ui propose resolution`).
- MCP show-UI: `ui_show_diff` lands in the last focused of two windows (needs
  terminal focus reporting; tmux `focus-events on`), the band clears on cursor
  move, `ui_highlight` outside the hunks, the `✦` tab marker, chip expiry,
  `ui_back`; proposals with `Ctrl+Enter` in WezTerm/kitty; MCP over HTTP with a
  real client.
- Git LFS against a real host: GitHub and self-managed GitLab clone progress,
  `lfs pull`, push progress, locks (someone else's, Force unlock, host without
  locking), the missing-program banner on Windows and Linux, a 10k-file repo
  (the 10 s poll must not be felt); the LFS toolbar with a real `git-lfs`.
- Hosting sidebar event-loop arms (row click / double-click / right-click, `.`,
  `v`, Enter, chip), a real account's repo list in Clone, a fork PR, 80 columns.
- `build-custom` interactive (whiptail) and `build-custom.ps1` on Windows.
- herdr `--remote` with the new build (graph defaults to text over SSH).
- Undo/Redo toolbar and `Ctrl+Z/Y`; Keyboard Shortcuts page and `Ctrl+,` under
  kitty protocol, tmux, herdr; graph themes and row tint at two zooms;
  SUBMODULES section; auto-stash dialog; `gitpoulpi ui notify` with a TUI open.
- Graph picture: no tooltip trace over the Graph column, markers after `Ctrl +`
  zoom, fast wheel; ref-to-commit connector in both renderers (thinner than
  lanes, passes under them, ends at the ring); Log pixel avatars (corners match
  the selection/hover band, no picture left on the wrong row while scrolling).
- Glyph widths in a real font stack: `✎ ★ ☆ ⋮ ⎇ □ ┄ ↠ ⊞ ✖ ▾ ⋯`, `📁 📂` not
  flipping monochrome on a bold row, `Soloing 1/N` title clean.
- Mouse: stash Pop/Apply/Drop from the sidebar menu, the third pane's file menu,
  list scrollbar thumb drags (incl. the Log with its header), ref drag-and-drop
  rows (a dirty tree must stop the merge), `Ctrl+click` multi-select (some
  terminals swallow it), rebase row drag-reorder, folder picker splitter drag.
- Text fields: `Ctrl+Enter` in the Description (WezTerm and Windows Terminal),
  `Ctrl+A/U/K` on Windows, AltGr characters on a Swiss-French keyboard, click
  and drag in a scrolled field with CJK / emoji, `Ctrl+C` not interrupting.
- Built-in editor: CRLF and no-final-newline round trip shows only your change
  in `git diff`, tabs vs spaces, undo by word, a 5 000-line file, binary refused.
- Windows specifics: `\\?\` never shown after a renamed repo folder or a pasted
  verbatim path; a hidden-attribute folder dimmed in the picker; an unplugged
  drive in quick places keeps the window responsive; Open in terminal (`wt`).
- Preferences with two or three real integrations opens instantly (tokens read
  "checking..." then settle); no GCM window ever appears.
- Recipe for terminal-mode settings (`ui.mouse_enabled`): check the escape
  sequences, not the screen: `script -q -c "gitpoulpi -s <probe>" out.raw`,
  then grep for `ESC[?1000h` / `ESC[?1006h`.
- Settings with popups over it: with a Workflow draft unsaved, `Ctrl+P`, `Ctrl+N y` (the whole Settings screen
  re-colours while you move through themes; `Esc` restores), `Ctrl+Alt+F` and `Ctrl+N a` each open over Settings and
  `Esc` returns to it with the draft and page intact; the octopus in Settings' title line opens About; in the palette
  a command like "Toggle Word Wrap" is grayed with "close Settings first". Check at 80 columns too.
- Windows, real mouse: double-click a language or theme in Settings (searchable comboboxes), and click one entry in the tab bar's "Open in" list when it has more than seven: a single click runs it.

## Conventions that bit us, worth knowing before you edit

Each of these caused a real bug. Most have a test that was confirmed to fail
without the fix; keep it that way.

### Install channels

- **The marker's channel survives a swap.** The release archive's marker says
  `channel=archive`; `install::apply` keeps the installed marker's `channel=` and
  `feed=` instead, so updating a Chocolatey or Scoop install in place can never
  turn it into an archive install. See "The updater owns the swap" in
  `docs/architecture/session-and-state.md`.

### Address bar: PATH beats App Paths in ShellExecute

- Typing `gp` or `gitpoulpi` in File Explorer's address bar goes through
  ShellExecute, which looks at the current folder and `PATH` first and only
  then at `HKCU\...\App Paths`. So with the bin folder on `PATH` the alias
  `.cmd` runs (a console flashes), and App Paths (the console-less
  `gitpoulpi-launch.exe`) answers only where `PATH` has no such program.
  Explorer's own `PATH` is the one from sign-in: right after `path add` the bin
  folder is not on it, so only App Paths answers, and with no App Paths key the
  typed name goes to the browser as `http://gp/` (seen on a Windows 11 hand run;
  with the key, no console window appeared). Both routes must keep the folder: the shim exports `GITPOULPI_CMDCMDLINE`
  and `relaunch::explorer_start` recognises `cmd.exe /c` with exactly two
  processes on the console (an interactive prompt has no `/c`, PowerShell makes
  three); the launcher path reads the working folder itself. Do not test the
  window class for the shim case: Windows Terminal as the default terminal
  hosts that cmd in a pseudo-console and the folder must still be kept. The
  variable is read and removed first thing in `main` (`explorer_folder`), and
  the terminal relaunch gets the folder as `--repo`, which beats the
  "Last tab" setting.

### Slow first launch (macOS)

- On macOS the first launch of a new executable (and of the developer-tools
  shims such as `python3`) can take seconds. A tool probe that times out is
  slow, never "not installed": `tools.rs` reports it installed with no version
  (`VERSION_TIMEOUT` is 15 s, injectable in tests). Tests that wait for a
  launched program poll against a generous deadline, not a short fixed loop.

### Slow start on Windows: a refused loopback connect takes ~2 s

- Never `TcpStream::connect` to a port taken from a discovery file without a bound: use `gitpoulpi_ipc::discovery` (`connect`, `live_port`, `remove_if_stale`), and probe many files concurrently. Linux refuses instantly, so only the Windows VM shows it; `xtask perf --tui` has startup rows for it (`session-and-state.md`, "Discovery files have an owner").
- `tui-stall input` entries of about one frame while something animates were the drain's first `poll` waiting for input (spinner pacing), not a stalled UI thread; the trace now logs `input` only when an event was handled.

### Every IPC connect goes through the token handshake

- Never `TcpStream::connect` or build a port by hand to talk to a daemon: read the endpoint with `gitpoulpi_ipc::discovery::endpoint` (or `live_endpoint`, which also probes) and connect with `Client::connect(&Endpoint)` or `discovery::connect(&Endpoint)`. Both send the `Hello` with the token from `sessions/<name>.token`; a raw socket is refused. A test that spawns a daemon uses `DaemonHandle::endpoint()`; a test double of a daemon only has to answer the first `Hello` (any non-`auth_required` answer counts as accepted).
- Discovery files are written user-only through `discovery::publish` (token first, then the port) and removed through `discovery::remove_files`; a hand-written `std::fs::write` of a port file leaves a daemon nobody can authenticate to.
- Local state goes through `gitpoulpi_config::secure_fs` (`create_private_dir_all`, `write_private`: atomic, created `0600`, never a `chmod` after the write). Never set a process-wide `umask` for this: it would also change the modes of checked-out files.

### The text caret (`ui.text_cursor`)

- Every editable field is drawn by `text_field::edit_lines`; never paint a caret by hand. With `terminal` (the default) the field draws no caret glyph and tags the caret's cell with `text_cursor::MARK`; the finished frame is scanned once (`take_from_buffer`, in the draw closure) and the terminal's real cursor goes there. A painter outside `text_field` (the code editor, the merge editor) takes its caret style from `text_cursor::caret_modifier()`.
- The cursor is placed from the finished buffer on purpose: a field only knows its own rows and columns, and a cell an overlay painted over loses its tag, so a field under a dialog never gets the cursor. Raw writes after the draw (pictures) move the cursor; the frame puts it back from `text_cursor::last()`.
- The mode is thread-local and starts as `drawn` under `cfg(test)`, so unit snapshots keep the glyph; the PTY harness pins `GITPOULPI_UI_TEXT_CURSOR=drawn` for the same reason (`pty_text_cursor` sets `terminal`). The DECSCUSR bar is restored on every way out (`restore_style`: normal exit, error exit, panic hook, update).
- Bracketed paste (`term_modes`) is requested on Unix only and released on the same ways out; a PTY test that injects `ESC[200~` itself proves the handlers, not the request (`pty_bracketed_paste` checks the request). On Windows there are no brackets: a terminal that pastes without asking (WezTerm, Rio, Alacritty, Tabby, Warp, mintty) types the text, so every line break is an `Enter` that submits the commit summary. `paste_burst` turns a run of already-queued key events (an interior `Enter`, or 8+ keys of more than one character) into one `Event::Paste` in the input drain (`pty_paste_burst`, Windows only).

### Buttons are real buttons, never bracketed text

- A clickable control is drawn through the button widget (`dialog_kit::button` / `button_style`: ` label ` padded on the button fill, hover, focus and disabled looks, registered in the pointer map as a hand), never as `[ Edit ]`, `[+ Add]`, `[✖]` or `[ 1 ▾]`. The only brackets left are the checkbox marks `[x]` / `[ ]` (`dialog_kit::checkbox_mark`), which are hand controls like the buttons, and key hints inside a label (`Tree [m]`). `button_audit_tests.rs` scans the source and every catalog for bracketed labels and fails on one; a new surface that draws its own buttons registers each rect with `pointer_map::hand` (a button over a text area otherwise shows the I-beam: the prompt editor's snippet buttons did).
- A page that keeps its own hit list (the Integrations page) registers it once per frame (`register_pointer`, reversed: the list is first-wins, the map last-wins) and hands the same rects to the `Ctrl+N` badges (`nav_hits`), so a control cannot have a click without a badge. Hit rects that are painted off-screen and windowed (its scroll) are converted before they are registered.
- A multi-line field with its own scrollbar (the Radar's Namespaces) paints the bar from a `scrollbar::Track` and keeps the track for the mouse (`BarDrag`): arrows step, the thumb drags, the track jumps. A bar drawn by hand cannot be pressed.

### Nothing is typed in the status line

- A prompt is drawn inline (a Log row, a Refs row, the Status list), in the small name box (`ref_name_box.rs`, when no inline spot could draw this frame), or in a `prompt_dialog` (stash message, tag message, upstream, LFS patterns); a number or text setting is edited on the Settings page, and the Refs filter types into the sidebar's own box. The footer shows only the key hint (`InputMode::InlineName` shows `footer.confirm_hint`, the filter shows `filter_refs_hint`), and `FooterDrawCtx` has no field editor, so it cannot host a caret.
- Guarded twice: `footer_guard_tests` fails when `footer.rs` mentions a field or caret helper, and `tests/pty_status_line_never_types.rs` opens every prompt, types a sentinel and checks it is on screen but not in the status line. Use `common::status_line_text` for such checks: it finds the status line by shape (a plain framed row with no box drawing inside), so it holds in every toolbar, status bar and tab bar position. A fixed row (the last, the last three) is wrong in the layouts where the footer is not at the bottom and passes vacuously; it returns an empty string for a full-window page such as Settings, which has no status line.

### Ctrl+Z / Ctrl+Y belong to a focused text field

- Every field that types through `text_edit::Caret` has an edit history (`text_history.rs`: grouped typing, one entry per paste / cut / kill / snippet, bounded, dropped when the text was replaced from outside). Filter boxes without a caret use `text_history::QueryHistory`. A focused field claims the chords (`Caret::claims`), so the global git undo never fires while a field has the keyboard (`keys_normal.rs`); elsewhere `Ctrl+Z` / `Ctrl+Y` keep their global meaning. The code editor and merge editor keep their own undo.
- Text changed behind the caret's back (`text.replace_range`, a field assigned from outside) clears the history on the next use; an edit that should stay undoable goes through `Caret::insert` / `replace_range`.

### Log row arithmetic (the repeat offender)

- Rows go through `log_table/geometry.rs`: `log_top_y`, `log_index_to_row`,
  `log_row_to_index`, `log_visible_rows`. A hand-rolled `log_area.y + N` or
  `area.height - K` is how clicks land on the wrong commit.
- A display index maps to a commit **only** through `graph::LogRows`
  (`log_order`): the WIP row is index 0 on a dirty tree (`has_wip_row`) and
  every stash row shifts the commits below it. A new `+ has_wip` is a bug.
- The image graph and the Log avatars do not re-derive rows:
  `visible_raster_rows` / `visible_commit_rows` walk the same facts as the hit
  test. Banners over the Log are overlays on its top row, never a resize of
  `log_area`.
- Another list of the Log's commits (the commit column, `commit_column/`) reads `LogLayout` and selects through the
  Log's own `log_state`: its rows are hit by commit id, its keys step over WIP and stash rows with
  `LogRows::entry`, and its lazy loading is `lazy_log.next_page` measured on its own window. Never keep a second
  selection or offset-to-commit table. Its Graph picture is a second `GraphImage` through the Log's own raster helpers,
  and a view's left border is the pane divider (the resize grab owns it): a button for a hidden column cannot live
  there (`session-and-state.md`, "The commit column").
- The Log table is built in one place, `log_table`, with `column_spacing(0)`:
  ratatui's default spacing of 1 puts rendered column edges up to two cells
  off `column_x_offsets`, and every resize strip lands mid-column.
- Column x positions come from one `LogColumnsGeo` (`log_table/geo.rs`, `CommandCtx::log_geo()`), never from widths and order rebuilt by hand: in the `scroll` layout the on-screen offsets are clipped by `h_offset`, a grip exists only while its cell is in the viewport, and a hit-test built with offset 0 maps the pointer to the wrong column while scrolled (`pty_log_scroll_layout`).
- The three panes are split in one place (`pane_areas`, `pane_divider_xs`),
  never `width * pct / 100`: the layout solver rounds where that floors.
- `Message` must never disappear: `LogColumn::message_reservation` is carved
  first, then whole columns go in `SACRIFICE_ORDER`. A new column goes into that
  order deliberately; widths sum to the inner width exactly.

### `row_to_index`'s `top_offset`

- Its third parameter is an absolute offset from `area.y`, not a header-row
  count. It changed meaning once, and code written against the old meaning
  merged, compiled, passed every test and shifted every click by one row. A
  bordered sub-pane's first row is `area.y + 1` (`diff_row_at` takes the outer
  rect for that reason).

### Hidden refs change the Log's walk

- A Log answer or page read under an older hidden-refs filter generation is dropped (`RefreshOutcome::log_filter_gen`, `LogPageOutcome::log_filter_gen`), and the window restarts its lazy loader and forgets its Log hash and total when the filter moves, in one place (`WindowState::tick`). A new site that changes `hidden_from_graph` needs nothing: the detector compares the filter each frame. `refs/remotes/<r>/HEAD` is a symbolic mirror and never a tip.

### Pinned tabs (Radar, Workspaces) are views of the management window

- Both are `RepoManager` states, not windows: `radar.is_open()` and `page == HomePage::Workspaces`. Every `ManagementEntry` that shows something else must reset the other (`tabs::go_to_management`), or two tabs look selected. A new pinned tab needs: a `NewTabSlot` field, one `Fixed::` row in `tab_column`, `HitTarget` pair (tab + `x`, `x` registered first), hover predicates, middle click, `Ctrl+W`, the palette close command, the `+` menu row, a global key, and a session flag read at startup (`init`) and published for the paint walk.
- The review through the API (a request of a repository that is not cloned) is hosted **by the Radar itself** (`RadarUi::api_review`, `radar/review_flow.rs`), not by a second slot on `RepoManager`: the Radar is already the first stop of the Home window's keys, mouse (body only: counting the hosted view in `popup_open` would swallow tab-bar clicks), paste (`drain.rs`), poll (`window/apply.rs`) and redraw pokes, so the view is routed to first there. What lives outside: `repo_manager::passes_through` (only the palette passes, and only while `takes_palette`), the palette (`tick.rs` takes `review_avail` from the repository dialogs, else the Radar; `commands/app.rs` runs `Review` rows on the hosted view first), `nav_targets` (empty while hosted), and the view is built by the event loop (`commands/shared.rs::open_api_review`: it knows whether AI is on). The review choice dialog is a Radar popup too (`popup_open`, `Spot::Choice`). "Clone it, then review" carries its request in `after_clone` keyed by the clone's destination (no expiry) and sets the 20 s `radar::pending` only once the clone has landed.
- `pty_pinned_tabs` pins the three tab-bar positions; the `x` is found as the first `x` after the label with only blanks between (`close_cell`), because the Radar view's own title says "Radar" too.

### Settings covers the status bar, and kitty draws U+2699 blank

- Settings is a full-screen surface over the footer, but the footer's hit rects from the last paint stayed live (first the version chip, now in the title bar, then the Activity button at the status line's right end): a click on Settings' bottom scroll arrow opened them. `chip::route_mouse` is skipped while Settings is open and the overlay pass calls `activity_log::clear_status_button` after painting Settings. Also, the event loop clears the scrollbars' hover pointer while Settings owns the mouse, so `PreferencesState::render` hands its own `mouse_pos` to `scrollbar::with_pointer` (else no arrow lights). `pty_settings_scroll`.
- U+2699 (text-default emoji) is an empty cell in kitty, with or without VS15/VS16 (screenshotted with `scripts/docs-screenshots.sh`'s headless kitty). Under kitty/Ghostty the gear is U+26ED (`chrome::gear_label`); the pty driver strips `KITTY_WINDOW_ID`/`TERM_PROGRAM` so tests keep finding U+2699.

### Hit targets: inner-first, and rects from the paint walk

- **Register hit targets inner-first.** `find_hit_target` returns the *first*
  registered rect containing the point; an outer band (a row, a tab, a header)
  covers the button inside it. Registered outer-first, the button is
  unreachable while looking fine. Bitten in four panes (diff hunk button, tab
  `x`, overlay rows, header buttons).
- Popups and menus drawn over the panes go to the **front** of `frame_hits`
  (`splice(0..0, ..)`) or look clicks up among their own targets first
  (`remote_hit_at`): they are painted after the panes.
- **Hit rects come out of the walk that paints** (the `log_table` rule; the
  `button_bar` "two walks" warning): `tab_bar`, `fit_bar`, `repo_manager::render`,
  `dialog_kit` all return spans and rects together. A close label's rect is read
  back from the rendered border row (`close_label_rect`), not recomputed from
  the title's width. Widths are `display_width`, never `chars().count()`: the
  `🔖 Tag` label is two cells and shifted every toolbar rect after it.
- **Too narrow to draw means not registered.** A hit target with no glyph under
  it is a click that does nothing. A hover state that widens an element must
  gate the element on its *narrowest* form (the REMOTE `+` vanished on hover).
- **Column boundaries are one helper** (`column_resize.rs`): the Log, the Radar and the commit column
  all build their grab rect with `grab_rect(mode, Edge, header_y, bottom)`, light it with `paint_grab` and
  rest a `paint_tick` in the header. `ui.column_resize` (`line` default, `header`) only changes that rect:
  the gap is the left column's own last cell (the Radar's and the commit column's one-cell separator), so
  the layout and the row tints are the same in both modes. In `line` mode the pointer holds the gap, one
  cell before the edge it moves (`pointer_to_edge`); the Radar has a header row (`CHROME_TOP` 6), its
  columns come from one `Geo` shared by header, rows and separators. The pointer shape goes through
  `cursor::set_pointer_shape` (change only, never in a multiplexer, reset on exit). The commit column's parts have a spacer cell between them, and it is the Graph's row-tint end cell (the Log's boundary cell, where the picture's end bar sits): the picture and the text tint run through it (`render`, `row_line`'s `spacer_tint`) and its `line` gap is that spacer (`Edge::before`). Moving the gap onto the last lane cell instead leaves a plain cell between band and message. A release brings no
  `Moved` (and some terminals send none after it): every table re-reads the hover where the button came up
  (the Log and the commit column via `resize_grab_at` in `mouse_drag::up`, the Radar in `on_mouse`), and a
  mouse press never "picks" a column: only the keyboard column mode keeps one lit until Esc (the Radar's
  `divider_press` once set `col_pick`, which left the line drawn for good).
- **One predicate per question.** `resize_grab_at` returns what a point grabs
  and its exact rect; hover highlight, drag arming and the pointer shape all
  read it. *If it highlights, dragging it works, and the reverse.* No grab
  margin. Anything painted over the management view or a full-pane page must
  drop the rects it covers (`frame_hits.retain`), and whatever must stay
  clickable on top (the gear) is drawn and registered again after it.

### A click pair needs state that survives a frame

- A double click is two presses a human apart with a frame between them;
  `Terminal::double_click` writes both in one go, so a test never sees it.
  `popup_list::View` is wiped by `begin_frame` unless the popup painted through
  it (`window_start`), so a combobox keeps its first click in its own
  `ClickPair`. Before that, every searchable combobox (Settings > Language, the
  Open in list past seven entries) ignored double clicks outside the tests.
  Test hand-paced: `pty_double_click_gap` (press, hold, release, wait, again).
  Action menus (Open in, Open with, Edit in) pick on the first click.
- **Two PTY clicks on one cell within 400 ms are a double click** (the second is ignored by `file_row_click`): a test
  that opens a file from the right panel's list twice in a row (the list rows sit at the same cells for one-file
  commits) must wait out `DOUBLE_CLICK_WINDOW` between them (`pty_commit_column.rs::open_from_the_list`).
- PTY tests on Windows need `GITPOULPI_RELAUNCHED=1` (set by `common::launch`):
  alone in its pty console the exe relaunches itself in Windows Terminal and
  exits.

### Every scrollbar drags (no bespoke bar handlers)

- A bar drawn with `scrollbar::render` must be pressable the same way: thumb
  grab + drag (even off the track), track click, end arrows (the arrow repeat
  is generic, it replays the `Down`). Lists go through `ui_kit::ListBar` /
  `list_bar_press` / `list_bar_drag` (the commit message block and the About
  crate list were "wheel + arrows only" until they did); text-field bars in
  dialogs use `scrollbar::BarDrag`; the merge view keeps its last painted
  tracks in `Ui::bars` and registers `MergeHit::Bar` runs around the thumb.
- **A track press never pages.** The maintainer's rule: pressing the bare track
  brings the thumb under the pointer (centred, clamped) and the press keeps
  dragging until release, horizontal bars included. One rule for every site:
  `scrollbar::Track::press(pos)` returns `TrackPress::Arrow(step)` or
  `Grab { grab, jump }` (jump = apply `drag_offset` at once). Editor, combo,
  popup lists, diff, File Insight and both image views use it; `page_delta`
  is gone. A new bar must call `press`, not re-derive thumb/track logic.
- A bar painted without arrows (`vertical_plain`) next to a handler that tests
  `arrow_step` is a dead click: there is no plain variant any more.
- A popup lights its own bars from its own pointer (`scrollbar::with_pointer`),
  never the global one, so nothing behind it hovers.
- **No scrollable area without a bar, and a guard says so.**
  `scrollbar_guard_tests.rs` fails for a source file (its code, not its test
  modules) that windows a list with `.skip(..).take(..)`, or renders a ratatui
  `List`/`Table` with a state or a `Paragraph` with `.scroll(..)`, without the
  shared bar code; its two allow-lists (`PENDING`, `STATEFUL_PENDING`) only
  shrink. A file that merely imports a `Combo` is not exempt.
- A window that follows a selection must follow it **when the selection moves**
  (remember the cursor it last followed), never on every paint: a paint that
  re-clamps to the cursor undoes the wheel and the bar (the workspace dialog's
  repository list did).
- A tall context menu keeps its scroll state in `context_menu/scroll.rs`
  (one menu is open at a time; `Menu::render` takes `&self`): the registry is
  cleared as a frame starts, refilled by the paint, and `event_loop::input`
  offers every mouse event to it before anything else, because the bar sits on
  the popup's border where no hit target is registered.
- Header text above a list (`rewrite_view/head.rs`) is clipped by priority so
  the list keeps `MIN_LIST` rows in a short pane; informing lines go first.

### Highlighted without acting

Mouse affordances here have repeatedly lit up without doing anything (the
resize highlight a cell off the grab, the redraw bug below, menus whose rows
lit but whose click went through). Hence: context menus are one flat list of
rows sharing `context_menu::Menu` (a drill-in, never a hover flyout); a modal
owns every click (a click outside closes it, never reaches the pane below); a
palette or picker click is translated into that popup's own `Enter`; an
overflow-menu choice is a synthetic click on the `⋯` with `forced_hit` naming
the hidden button. No second copy of any action.

### Pointer and hover rules (one widget, one shape)

Three rules, owned by the widgets, not by the screens:

1. **A row is the arrow and lights as a whole** (the Refs pane's wash,
   `th().ui.hover`): list rows of every pane and of every dialog (clone, folder
   picker, Workspaces, board cards, Jobs, meta pickers, review files and
   commits, rebase steps, compose proposals, rewrite rules, Radar items).
2. **A control is the hand**: combobox field, checkbox, toggle, button, close
   cross, expand arrow (group and folder headers), tab, link, scrollbar. A
   control painted inside a row (a checkbox mark, a Cancel button) keeps the hand.
3. **An input box is the I-beam**, the whole box, never its label.

How it is kept consistent: `pointer_map` is cleared each frame and every
shared widget registers its own kind **when it paints**: `dialog_kit::button`,
`checkbox`, `text_input`, `text_area`, `Combo::render_field`,
`close_cross::paint`, `scrollbar::render_buf`, `dialog_kit::row_band` (a list
row: registers `Plain`, fills the band, so use `row_bg` for the selected / hover
fill). The sidebar's own rows (hosting panes) register per row in
`HostingUi::pointer_kind`. The ordinary mouse path applies the registered
shape over the hit-target answer only when that answer is the arrow
(`pointer_map::over`); resize, carry and the hand of a hit target stay above it.
A screen that paints through those widgets gets the shape and the hover for free;
do not add a per-screen `pointer_hand_at`, and do not register a whole hit list
as `Hand` (`from_hits`) when it holds rows: return `Kind::Plain` for rows, `None`
for rows that register themselves (they hold a control inside). A later `Plain`
over an earlier `Hand` hides it: paint the row first, its controls after.

Left on the per-hit-target path (not through `pointer_map`), on purpose: the
rebase, compose, split and rewrite workbenches and the Reword form map their
`HitTarget` variants in `desired_cursor_shape` (rows arrow, text boxes
I-beam, the rest hand), because they paint many small hand-drawn controls
(`Pick v`, arrows) no shared widget draws. The Radar and Settings answer from
their own hit lists (`pointer_hand_at`) for the same reason, and agree with the
rules above. The commit details' info lines: `commit: <sha>` copies the short id
(the palette's Copy short SHA), the author and date lines amend on a double
click; the stats line has no action and keeps the arrow, as does the parent
label (no "go to parent" command exists). The message block is the I-beam and
lights as a whole (`commit_detail::light`), never the whole pane.
Pinned by `pointer_map::tests::each_widget_kind_registers_its_pointer_when_it_paints`,
`hosting_ui::tests::rows::the_issues_pane_and_the_settings_page_register_the_hand_on_their_comboboxes`
and `pty_pointer_widgets`.

### The mouse re-poll `continue` bug

The input drain polls at the **top** of the loop (`event_loop/window/drain.rs`).
With the poll at the bottom, each of ten `continue`s in the mouse arm skipped
it, the loop went back into a blocking `event::read()`, and the frame was not
redrawn until the next input: "clicks only work after a key press". Do not
move the poll. **A test for a redraw bug must send no input after the thing
under test**: every existing mouse test passed against the broken loop because
a click's release unblocked its press.

### Redraw on demand

The loop draws only on input, a deadline or a `poke` (`event_loop/window/idle.rs`).
A new time-driven visual must add its deadline to `next_deadline` (or
`animating`); a worker handing the UI something sends with `send_notify` or
calls `poke` after changing shared state. Otherwise it appears one keystroke late.

### Never block the render/input thread

**Never block the input loop on a git subprocess call**, a credential helper,
a directory read or a network request. A process spawn costs 200-500 ms on the
a Windows machine (`git --version` ~350 ms, up to 1.5 s under load).
What happened when one got onto the loop:
- Preferences called `TokenStore::has` (`git credential fill`) from `fields()`
  on every frame: a Git Credential Manager window per integration per frame.
  Then from `open`: 5-8 s to open. Now a background probe
  (`spawn_token_presence_probe`), rows read "checking...".
- "Open in" detection listed `PATH x PATHEXT` per program on the input thread:
  9-12 s before the first frame on Windows (`repo_menu::path_lookup`).
- A fetch held the daemon's repo lock and froze every pane (`TransferLocks`).

So: `repo_probe` and `folder_listing` run on threads and the row draws at once
with "still looking"; a directory listing is capped (`LISTING_LIMIT`); graph
pages are fire-and-forget requests (`DaemonLink::request_log_page`, at most one
per frame, one in flight, `graph/paging.rs`); network `Action`s are spawned
(`Action::is_network`); never add an LFS read to the refresh unconditionally
(`LfsGate`). `GITPOULPI_TIMING=1` writes `gitpoulpi-timing.log` (temp dir) and a
`tui-stall` trace; `docs/developer/open-time.md` and `perf-budgets.md` hold the
budgets.

**The selected commit's files** (`commit_files_bench`, run with
`cargo test -p gitpoulpi-tui --lib --release commit_files_bench -- --ignored --nocapture`,
`GITPOULPI_BENCH_REPO=<path>` for another repo). Measured on this repository,
a Windows machine, git2 backend, before -> after the fix:

| scenario | before | after |
|---|---|---|
| `j` walk, first visit, list shown | median 264 ms, p90 863 ms | median 72 ms, p90 110 ms |
| revisit a commit | median 265 ms | 0.7 ms (cache) |
| random clicks, list shown | median 69 ms, p90 644 ms | median 29 ms, p90 45 ms |
| held `j` over 15 rows, after release | median 602 ms | 62 ms list / 201 ms hunks |

The causes, each a rule: daemon reads share the per-repo lock (`RwLock`,
`dispatch::is_read`) while mutations stay exclusive; `diff_files` lists deltas
without hunks so the list shows first; one commit-files worker with an LRU cache
that drops superseded selections; system git runs one `git diff <c>^ <c>`, not
`rev-parse` + `diff`.

### Loading state, not an error

Until the first `RefreshOutcome` arrives, the five results are seeded with a
placeholder `Err("connecting to the session daemon...")`. That placeholder was
once drawn as a real error, pushed as one cell into the Log table and clipped
to `log error: backend`. Decide "loading" from `still_loading`, never from the
placeholder's text; draw loading and error messages with
`ui_kit::loading_lines` / `render_centered_message` over the whole pane, never
as a table row. Past `LOADING_TIMEOUT` (15 s) it says "Still loading... (large
repository?)", never nothing. A link that stops shows `pane_notice` ("Lost the
connection ... Press r to retry"): every exit path of the link loop carries a
reason (a new `return` there must too), and `PendingRequests` fails every
waiting call on a lost connection. Pinned by `tests/pty_loading.rs`.

### Emoji presentation, and glyphs that render as holes

- Use plain ASCII in labels, or glyphs with **default emoji presentation**
  (`Emoji_Presentation=Yes`, fully qualified, no VS16), or text-default BMP
  shapes known to be single width in every monospace font (`○ ● ◎`, U+00B7).
  **A VS16-dependent glyph renders as a hole**, or flips presentation under
  bold or a different style. Both happened (`HIDE_IN_GRAPH_ICON` needs its own
  guard). Folder icons are `📁` / `📂` (Emoji_Presentation=Yes).
  No exceptions: the palette's key button was `⌨️` (U+2328 + VS16) and in
  WezTerm on Windows it drew as a bar on every row, while the cell ratatui
  skips after a wide glyph kept what was behind the popup (stray characters
  by the scrollbar). It is now ASCII `[k] ` (`palette::KEY_BUTTON`). Popup
  rows end in ASCII only; `palette_overlay` tests paint over a frame of `X`
  and assert the rect is fully overwritten and the row tail is ASCII.
- **An overlay edge may cut a CJK glyph in two.** ratatui's `diff` never emits
  the cell after a wide glyph, so a border cell right of a still-wide glyph is
  never written (a missing `╰`/`│` in zh-CN/ja/ko). The overlays' `Clear` is
  `wide_edge::Clear` (a drop-in for `ratatui::widgets::Clear`: import it, never
  the ratatui one) and blanks the half-glyph its left edge cuts. **No
  whole-frame post-pass**: a buffer cannot tell a glyph cut by an overlay from
  a wide glyph painted legitimately over a non-blank cell (ratatui's `Span`
  leaves the continuation as it was, e.g. the title bar's 🐙 over the border's
  `─`), and a "continuation is not blank" pass erased the octopus. Pinned by
  `wide_edge::tests` (CJK panels/menus and the octopus survival test).
- **A CJK popup is checked by `render_check_tests`, not by eye.** `render_check_tests.rs`
  (test-only) holds the buffer invariants: a wide glyph keeps its blank
  continuation (a `─` under a title glyph is the one tolerated leftover), a
  popup's box is whole on every row (a scrollbar may stand in for the right
  border), nothing is written outside the popup, a menu's shortcut column ends
  flush. `cjk_sweep_tests` runs them for zh-CN, ja and ko at 80x24, 120x30 and
  200x50 over a background of CJK text (so a popup edge cuts a glyph on every
  other row); `i18n_layout_tests::check_buffer` runs the wide-glyph rule over
  every screen it draws. A new popup: add it to the sweep. In a pty,
  `Screen::line` shows a wide glyph's second half as a space ("简 体"); read
  rows with `Screen::compact_line` (`pty_cjk_popups`).
- **Never a combining mark.** `unicode-width` = 0 describes Unicode, not the
  user's font: U+20DD is in no Windows monospace font, the fallback broke the
  grapheme cluster and sheared the whole Graph column.
- Glyphs WezTerm draws wider than one cell shift the row and corrupt the next
  repaint (`✓ ◉` in the solo gutter; now `CHECKED_OUT_MARK` / `SOLO_FULL_MARK`).
- **A glyph never guarantees a junction; only a painted background or a
  picture can.** Glyphs sit inside side bearings. *Paint joins, glyphs decorate.*
- Check font coverage before proposing a character family (WPF `GlyphTypeface`
  from PowerShell; tables in `commit-graph.md`). `TreeCell::span` is the only
  place a `TreePart` becomes characters; a layout that wants a string literal
  wants a new `TreePart` (`no_node_is_left_unconnected`).

### Falsifying buffer tests (break -> red -> restore)

Delete or break the fix, confirm the test goes red, restore it. Three tests in
this project passed while asserting nothing. Assert on what the terminal is
handed (a `TestBackend` buffer, or the PTY screen), never on a string the test
just generated. Stale cells: ratatui's `TestBackend` only receives diffed
cells, like a real terminal, so it reproduces leftovers faithfully. insta is
used only for the theme snapshots (`theme/snapshot_tests.rs`); everything else
is hand-written buffer assertions (`docs/developer/testing.md`).

### Never repoint `GITPOULPI_CONFIG_DIR` from a test

It is process-wide and the daemon's `.port` file lives there: a test that set
it raced `ensure_daemon` in an unrelated test of the same binary and
deadlocked. Use the `*_in(config_dir, ..)` twins (`session_store`) and pass the
discovery file explicitly (`start_at`, `ensure_daemon_at`); environment-driven
code takes the environment as a lookup (`terminal_image::protocol_for`). Same
for any process-wide static (`DATE_PATTERNS_TEST_LOCK` serialises the tests
touching date patterns).

### Repaint and pictures

- `ratatui` `List`, `Paragraph` and `Block` set the style of their whole area
  but write symbols only where they have content: a cell no item reaches keeps
  whatever a popup left. Render `Clear` before such a widget, every frame.
- Pictures bypass the cell buffer. `GraphImage::finish_frame` (and the avatar
  equivalents) must stay the last call of the frame: anything painted after it
  is invisible to the covered-cells check. A changed Kitty picture gets a new
  id, never a re-upload under an id cells point at; `release_all` deletes every
  id before leaving the alternate screen. Pictures must be hidden whenever the
  Log is not the visible view (remanence over Settings/diff on Windows).
- A Log avatar slot any of whose cells a popup covers shows **no picture**
  (iTerm2, with or without blending): the slot's cells are written as the
  popup's where it drew and the table's initials elsewhere, so a terminal never
  keeps half an avatar beside the About window or a menu, and the picture is
  sent again when the popup goes (`log_images::hidden_cells`). The tests model a
  terminal (`terminal_image/vt_sim.rs`: writing a cell drops its tile) rather
  than matching bytes. The Graph column's picture, one image, still sits under
  a popup that covers less than half of it (popup cells repainted over it).
- **A pane multiplexer inherits its parent's environment**: inside herdr the
  code believed it was WezTerm and painted base64. Detection denies on any
  multiplexer signal (`HERDR_*`, `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*|tmux*`)
  unless herdr's config enables Kitty graphics; `ui.graph_renderer` /
  `ui.image_protocol` override; `GITPOULPI_TERMINAL_IMAGE=off` wins.
- Never query the terminal and wait on stdin for the reply from a **helper thread** (cell
  size, Kitty `a=q`): a blocked read cannot be canceled and ate the pty suite's keystrokes,
  and on Windows the reply is not bytes. The one allowed read is `startup_probe`: a `poll`
  with a deadline on the calling thread, once, before the event reader exists (Unix only),
  one write and one read for keyboard flags, OSC 11, XTVERSION and DA1 (the sentinel).
  Its answers only add a protocol where the environment names nobody (DA1 `4` = sixel,
  XTVERSION name), never inside a multiplexer (tmux claims sixel in its own DA1).
- **Sixel** (`terminal_image/sixel.rs`, `Protocol::Sixel`) is detected from the
  environment (`WT_SESSION`, `TERM=foot*|mlterm*`, `KONSOLE_VERSION`), after
  kitty and iTerm2, then from the start-up probe's DA1 parameter 4 (VS Code with images,
  xterm `-ti vt340`); a terminal that draws sixel without saying so opts in with
  `ui.image_protocol = "sixel"`. It is an `Encoded` that
  `blends()` (transparent pixels, `P2 = 1`): same buffer discipline as `Iterm2Alpha`
  (blank reserved cells, `under` cells rewritten before the picture, `over` after), so
  there is nothing to delete and no id. It must be **exactly** `cols x cell.width` by
  `rows x cell.height` pixels (`encode_image` resamples; it reads `cell_size()` itself).
  The Graph column is a sixel too (2026-10-19; `draws_graph` = `draws_images`): one picture
  per frame like WezTerm's, avatars inside the nodes. Release cost of a 50-row x 14-cell pane
  with faces: raster ~20 ms (shared with kitty) + sixel encode ~6 ms, 67 KB; the encode is
  cached by `key_of` and idles for free. **Sixel needs the true cell size**: Windows has no
  `TIOCGWINSZ`, so `terminal_image::measure_cell_px` asks `CSI 16 t` once at start-up and on a
  resize (`cell_query.rs`: `WaitForSingleObject` on the console input handle, 200 ms deadline,
  no helper thread, silent terminals are never asked again). A sixel on the last screen row
  may scroll some terminals (the cursor moves below the image); the Log never ends there.
- **PTY tests on Windows must carry `GITPOULPI_RELAUNCHED=1`** (`tests/common` sets it): a
  lone process in a ConPTY looks like a double-click and `relaunch.rs` opens a real Windows
  Terminal window. `terminals::inside` also stops a relaunch when the chosen terminal's own
  marker (`WT_SESSION`, `WEZTERM_PANE`...) is set; the WezTerm shortcut targets
  `wezterm-gui.exe` (the console-subsystem `wezterm.exe` opened a stray console window).
  PTY tests that need detection must blank `HERDR_*`/`TMUX`/`TERM_PROGRAM`: the pty
  inherits the runner's multiplexer.
- Tooltips: `tooltip.rs` (`TooltipLifecycle`) is the only owner; do not add
  tooltips anywhere else. Popups are bounded to their own pane, not `frame.area()`.

### Closing a surface, and Settings list pages

- **Every closable framed surface paints `chrome/close_cross::paint`** and registers the rect it returns, so a click
  target exists exactly when a cross was drawn. No hand-rolled ` ✕ ` at `right() - 5`, no "Esc" text beside it (the one
  exception is the key-capture popup, where Esc is the key that cannot be captured). A cross is never drawn on menus,
  dropdowns, the palette or pickers. A hint or status line that mentions `Esc` among other keys is fine (maintainer,
  2026-10-22): only an "Esc" label next to the cross is the problem.
- **Dialogs implement `dialog_kit::Dismiss`** (`is_dirty`, `guard`) and route cross, `Esc`, an outside click and Cancel
  through `dismiss(DismissKind)`, which answers Close, Swallow or Confirm. A dirty form swallows an outside click; `Esc`
  and the cross ask "Discard changes?" only when real work (typed multi-line text, an unsaved draft) would be lost.
  Confirmations are `dialog_kit::Confirm` with its hit rects first. An inside click never closes a panel.
- **Settings has no daemon link and applies each change at once**, so only the Workflow draft, the commit template
  draft, an open sub-editor and an unfinished list item (a profile with no email) can block closing
  (`PreferencesState::close_guard`).
- **A list page plugs into `preferences/list_page.rs`**: answer `PrefSection::has_list`, build a `ListSpec`, apply each
  `ListOp` in `apply_list_op`, and make `fields()` return the detail rows of the selected item. Layout, `PrefFocus::List`,
  keys (`list_add`, `list_duplicate`, `list_remove`, `list_move_up/down` in `Ctx::Preferences`), drag and the remove
  confirmation are generic (`list_editor.rs`). Never add a "select the item" combo plus Move/Remove buttons again.
- **Actions are inline buttons** (`Control::Buttons`) showing only the ones that apply; destructive ones confirm.
- **A number with a unit is `Control::Quantity`** (`preferences/quantity.rs`), never a preset dropdown of "1 week, 2 weeks...":
  a number field and a unit dropdown on one line, the row's key storing `number x unit weight` and `unit_key` the unit read
  in. Its two parts are the row's `btn_cells` (part 0 number, 1 unit, `ai.chip` is the focused part), so hover and click
  rects come from the same walk as inline buttons. Reuse it for the next duration or size row: `quantity_rows.rs` builds the
  six that exist besides the retention (one `Row` spec each: base unit, offered units, range, `allow_zero`), the config side is
  `DisplayUnit` + `read_quantity` (`gitpoulpi-config/src/schema/quantity.rs`), and the key is `<name>_unit` next to the number.
- `popup_list::View` is wiped each frame, so a list's double click owns a `ClickPair`; the add dropdown and the remove
  confirmation take the pointer before the list's own hits.

### Input routing

- **Popups over Settings.** Settings' key arm (`modal.rs`) runs only in `InputMode::Normal` with About closed, and
  `PreferencesState::passes_through` lets the palette, workspace search, `Ctrl+N` and help chords reach the ordinary
  dispatch. The palette and the theme / language / search pickers are painted *after* Settings
  (`overlay_menus.rs`, `draw_picker_popups`), and `draw_overlays` skips them while it is open. Which commands work
  over Settings is `PaletteCommand::stays_in_settings` (everything else is grayed, and `run_command` and the `Ctrl+N`
  table refuse it with a line on Settings' bottom border: the status line is covered). A new app-wide popup command
  belongs in that list.
- **A bare-letter binding also matches under Ctrl.** Blame's `n` / `p` / `g` swallowed `Ctrl+P` and the `Ctrl+N`
  prefix until `file_panel_passes_through` (`file_panel.rs`) was narrowed to "every Ctrl chord except the view's own
  copy chord". Any new central view with bare-letter actions needs the same test, with a Ctrl chord.
- **History and Blame row arithmetic has one source.** `history_view::ROW_H` (2 terminal rows per commit) and
  `blame_view::layout::Layout` feed paint, hit rects, wheel, scrollbar and paging; a second computation drifted
  from the first. Never write a literal `2` for a History row's height.

- **`q` never quits from inside a view**: over the Diff / File View / editor it
  closes the view; only over the graph does it quit. `q` falling through to quit
  ended `gp` silently with exit code 0. A new central view must claim `q`.
- **Every way out of the process leaves a record**: a new quit path sets
  `quit_how`; a fallible call in the loop gets
  `.map_err(exit_log::io_context("..."))`. An error restores the terminal and
  prints the chain; up to 20 consecutive failed console reads are tolerated.
- **A popup with text fields must be in `count_prefix_may_apply`**: the vim
  count runs before dispatch and swallows digits (Preferences numbers, paths,
  URLs, the commit-files filter all lost digits). New text popups that are not
  an `InputMode` set `text_popup_open`. The same for a popup whose rows are
  picked by digit (the tag Delete scope dialog's `1`-`9`): its unit test passed
  while the PTY test showed the digit never arrived.
- **A tag row is not always a local tag.** `refs()` lists a tag only a remote has
  (`Ref::is_remote_only_tag`, `tag_sync.local == false`); code that acts on a
  *local* tag (push, delete, annotate, checkout) must skip it, and every surface
  reads where a tag lives from `tag_where_ui`, never from `pushed_to` alone.
- A surface that owns the keyboard must still let `?`, `Ctrl+P`, `Ctrl+,`,
  `Ctrl+N` through when no field is focused (`passes_through` on Preferences,
  the management tab, the editor, the merge view), and help is drawn last.
- Never re-indent a large `match` in a change: a reindented copy merged against
  new arms produced silently duplicated arms. Find a preamble seam instead (the
  count prefix lives before `match event`).
- `Ctrl+N` badges are built in the same pass that draws them, from the frame's
  own rects; a pane with no badge is unreachable from the keyboard.
- Settings badges (`preferences/nav.rs`) are clicks: `nav_targets` lists a cell on each control from the rects the
  paint walk captured, `nav_press` sends a left click there, so a badge and the mouse cannot disagree. They are drawn
  in `draw_overlay_menus` after Settings paints (`draw_overlays` skips its pane badges while Settings is open) and
  nothing is badged while a field, combobox, search, editor or confirmation of Settings owns the keyboard. `Ctrl+N`
  is no longer `list_add` (`Insert` is). The Keyboard and Integrations pages' own bodies are not badged yet.
- Dialog and Activity Log badges are clicks too: each dialog's `nav_targets` lists a cell on every control from the
  `hits` its paint walk registered (`dialog_kit::nav_cells`), `RepoDialogs::nav_press` / `activity_log::nav_press` send a
  left click there (`FocusTarget::Dialog` / `Activity`). The prefix starts in `modal.rs` (dialogs) and `input/mod.rs`
  (Activity) only when `nav_targets` is not empty, so a dialog with an open list or none keeps `Ctrl+N`. The folder
  picker's New folder key is `Alt+N` for that reason. Not badged: the tab picker, jobs panel, help, About, review/board pages, Meta picker, Safety panel, the Settings Keyboard and Integrations bodies.
  The Amend dialog and the history workbench keep their form inside `InputMode`, so their `Ctrl+N` is a `badges` flag on
  the form (not `InputMode::Navigation`): `badge_ready` says when it may start (not while a popup, question, editor or
  overlay owns the key), the badge key is read in `keys_modes.rs` (`badge_cell`) and clicked through the form's own mouse
  handler, and the badges are drawn in `draw_overlay_menus` from `frame_hits` (`dialog_kit::nav_cells_in` picks a cell
  the click really reaches).

### State, persistence, daemon

- **`RepoId` comes from the repo root, never the cwd**, or a TUI in a
  subdirectory and a CLI beside it are two repos to the daemon. The default
  session name is derived from the root (`default_session_name`).
- **A repository switch restarts the inner loop** (`run_in_repo` returns a
  `NextWindow`), not a live re-subscription. Hence: **never spawn a long-lived
  service on a per-window runtime** (the embedded daemon died at every tab
  switch: infinite "Still loading..."); use `embedded_daemon_runtime()`.
- A recent entry gone from disk is shown greyed, not silently dropped; only a
  path that is a repository (`session_store::is_repository`) is ever recorded.
  Paths are compared after `gitpoulpi_core::paths::strip_verbatim_prefix`
  (`\\?\C:\` has two backslashes; a one-backslash constant never matched).
- **`None` in `RepoPreferences` means inherit** and must round-trip as `None`.
  A secret follows its id: renaming moves the token, removing deletes it.
- A new `persist_pane_layout` call passes `left_auto_collapse.persisted(..)`,
  never the raw flag, or a diff open at save time persists a collapse the user
  never chose. Edge-trigger auto-expansions (`wip_selection_should_expand_status`)
  so they never fight a user who re-collapses.
- A commit-detail file row's index is into `commit_paths_cache`, not
  `outcome.files`; under View all files they differ. Resolve by path
  (`file_panel::commit_file_open`).
- The Diff view's Blame/File View/History on a working-tree half act on HEAD:
  `DiffViewState::commit` is an all-zero placeholder there
  (`diff_insight_revision` decides).
- `LineSelector::line_indices` are the *backend's* numbering: hiding whitespace
  rewrites a hunk, so rows carry their original index.

- **Outside changes (`journal/outside.rs`, `core/journal/outside.rs`): sync
  before, adopt after.** A request that can move refs first *syncs* (diffs the
  repository against `refs-snapshot.json` and records what is outside), runs,
  then *adopts* (rewrites the snapshot) so its own moves are never recorded
  twice. `dispatch` is re-entered by auto-stash steps and start-over: the
  outermost request owns detection (`Busy`), nested ones skip it, or the pull
  inside an auto-stash is recorded as an outside change. A merge, rebase or
  cherry-pick that is stopped skips both: the outside rebase is then recorded
  once, when it ends; our own stopped merge taints the snapshot so its end is
  adopted, not recorded (`TAINTED`). Transfers (`TRANSFERS`) block read-side
  syncs. Tests that exercise a *blocked* entry turn the recording off
  (`GITPOULPI_GENERAL_JOURNAL_OUTSIDE=false`), or an outside commit becomes the
  newest entry instead of a refusal; spawn-budget tests do the same.
- **The fingerprint is racy by design.** File times are coarse; a state read
  within 3 s of the newest mtime is never trusted, or a `git branch -D` right
  after a snapshot is missed. Do not "optimise" the 3 s away.
- **Reflog messages name the move, not the ref.** `branch: Created from HEAD`
  says nothing about which branch: `OutsideChange::label` appends the ref
  names. `update-ref` without `-m` leaves an empty message: fall back to
  "moved branch x". A symbolic ref (`origin/HEAD`) is listed among the refs by
  the system backend only: the snapshot drops the symrefs.
- **Effect of an outside entry.** `Effect::Files` (`reset --keep`) only when
  the checked-out branch itself moved and HEAD stayed on it, or a detached
  HEAD moved; if HEAD moved too (`checkout -b`) the entry is `Effect::Refs`
  with a `Checkout` step (`OperationKind::Outside` is in `checks_out`), or the
  plan would `MoveHeadKeep` a branch that is not checked out any more.

### Backends and git

- **`SystemGitBackend` pins git's output shape** against user config at one
  seam, `process::git()` (`diff.mnemonicPrefix` broke nine contract tests on a
  real machine while CI, with no user config, stayed green). Signing, hooks and
  credentials stay the user's.
- **One splitter for bytes to lines**, `BlobContent::from_bytes`, used by both
  backends (`core.autocrlf` otherwise gives the same file two line counts and
  shifts every blame line). Binary and oversized files are successful results
  (`Binary`, `TooLarge`), sized from the object header first.
- **git2 must never write a working tree, or compare one with the index, in an
  LFS repository** (no LFS filter: pointer text on checkout, every stat-dirty
  file "Modified"). `Git2Backend` routes those to system git when
  `lfs::is_active`; a new git2-native working-tree operation needs the same
  guard and an LFS contract check (ADR 0009, `git-backend-trait.md`).
- Hunk patches are rebuilt from `git diff` bytes (`cli-backend/src/patch.rs`),
  keeping CRLF and the no-newline marker; reverse direction turns an unselected
  addition into context and drops an unselected removal.
- **git separates progress with `\r`**, not `\n` (`run_streaming_stderr`,
  `parse_clone_progress`); strip a matched prefix by length (`remote: Counting
  objects:` has two colons).
- Tags: `LogOptions::all_refs` enumerates branches, tags and remotes, never
  `--all` (the `refs/gitpoulpi/remote-tags/` mirror would enter the graph). The
  mirror has one writer; a tag at a different commit on the remote is not
  "pushed"; `tag_sync: None` means not computed. The mirror fetch is a second
  `git fetch`, never an extra refspec (that replaces the user's).
- An operation with no repo to target (`Discover`, `Init`, `Clone`) is answered
  in `dispatch` before `repo_ref`. `init` refuses an existing repository root
  and returns what `discover` would.
- `git lfs track --lockable` makes files read-only: fixtures that overwrite a
  file track it non-lockable. Credential lookups for app tokens are
  non-interactive (`credential.interactive=false`, `GIT_TERMINAL_PROMPT=0`,
  `GCM_INTERACTIVE=Never`); the user's real remote credentials may prompt.
- A git spawn costs ~1.5 s on the a Windows machine: contract checks that
  build several fixtures take tens of seconds there. Slow is not a hang.

### Folder arguments from Explorer: the `"C:\"` trap

- Explorer substitutes `"%V"` with the folder in quotes, and on a drive root that is `"C:\"`: argv parsing reads `\"` as an escaped quote, so the program receives `C:"`. Every `--folder` value goes through `terminals::normalize_folder_arg` (strips a trailing `"`, gives a bare `C:` its `\`) in `launch::split_launch_args`, which parses the arguments of `gitpoulpi-launch.exe`, the program the Explorer verbs run. A new Windows entry point that takes a folder from Explorer must do the same.

### PTY, ConPTY and Windows

- **ConPTY asks questions and waits for answers**: a session opens with
  `ESC[6n` (cursor position) and `ESC[c` (device attributes), and on the CI
  console host the child does not start until they are answered.
  `xtask/src/pty` `answer_queries` replies like a terminal (cfg-free). A
  Windows desktop does not enforce it: green locally, red on the runner.
- The ConPTY preamble (27 bytes) is not the child's output: waits use
  `bytes_drawn()`, not `bytes_received()`.
- `stop()` drops the writer and the master before joining the reader, with a
  bounded join (`READER_GRACE`): a ConPTY keeps the pipe open until this
  process lets go. **A test that can hang is worse than one that fails.**
- `Terminal::exited()` asks the child's exit status: ConPTY never reports EOF,
  so every "did not exit" assertion was vacuous on Windows.
- A platform-specific skip is a hypothesis, not a fix; the local machine and
  the runner are different terminals.
- `pty_lfs` ignores the `stop()` error: the kill can land in a `git lfs` chain
  whose children still hold the ConPTY.
- `GITPOULPI_NO_EXIT_PAUSE=1` and `GITPOULPI_NO_BROWSER=1` keep PTY tests from
  waiting on Enter or opening a browser; `GITPOULPI_REQUIRE_LFS=1` makes a
  missing `git-lfs` a failure instead of a skip.
- Linux-only red after the Windows background-jobs batch (all were stale
  tests): identical network requests now join one job, so a test of "transfers
  queue" must send *different* ones (`Fetch` + `FetchAll`); an LFS pull reaches
  git-lfs as `fetch <remote>` then `checkout`, never `pull`; and an `Escape`
  followed at once by `Ctrl+O` is `Alt+Ctrl+O` on a Unix pty (ConPTY hides it),
  so wait for the card to vanish first.

### Screenshots

`docs/developer/visual-verification.md` (Windows and Linux/Hyprland halves):
the naive approach captures whatever window is on top. `PrintWindow` returns
blank frames on a locked session; that is not worked around.

### UI rules every change must respect

- **Popups own the pointer.** A popup's hit rects go *first* in the hit list,
  because `find_hit_target` takes the first match; over a popup row nothing
  behind it may hover or receive the click. Two bugs came from this: the click
  falling through a menu (7b2af6c) and the hover showing through (83dd615).
- **Hit rects come from the paint walk,** never from separately recomputed
  geometry (see also "Log row arithmetic" and `row_to_index`'s `top_offset`).
- **Comboboxes, never click-to-cycle.** A choice with more than two values, or
  whose values are not plain on/off, is a dropdown (`crates/gitpoulpi-tui/src/combo.rs`);
  booleans may stay checkboxes.
- **Every mouse action has a keyboard twin,** and the command palette reaches
  100% of features.
- **Every AI control carries the sparkle** (in the AI purple), and AI runs only
  on an explicit user action: never on open, focus, refresh or as a side effect.
- **Hosting is opt-in per repository.** Providers are only suggested from the
  remotes; nothing is linked or fetched until the user picks, and an unlink
  persists. No accounts, no OAuth: personal tokens only.
- **Speed is the point.** Every action feels instant and shows in-flight
  feedback past about 150 ms (`docs/developer/perf-budgets.md`).
- **Glyphs: avoid emoji presentation.** A text-default emoji such as 👁 needs
  VS15 (U+FE0E), or WezTerm draws it in colour.
- **New user-visible TUI strings go in the i18n catalogs** in all shipped
  languages, source-control words as `{$term}` tokens (`docs/developer/i18n.md`). The
  `no_new_hard_coded_ui_strings` lint fails on any flagged literal; mark
  non-UI text `// i18n-english`.
- **Every new or changed mutation decides its Undo/Redo story:** journaled with
  undo/redo restoring the *whole* state (index conflict stages,
  `MERGE_HEAD`/rebase state, WIP, stashes), or explicitly not undoable with the
  Undo button saying so. Add a PTY or daemon test for each.
- **Tests use temp config dirs.** Running a `gitpoulpi` binary by hand to check
  something: export `GITPOULPI_CONFIG_DIR` (and a temp `HOME`/`XDG_CONFIG_HOME`)
  to a scratch dir first.

## On the APEX workflow

Work runs as one worktree agent per batch, merged in sequence.
Agents branch from the `main` they
started on, so later merges carry semantic conflicts that compile fine; both
real merge bugs of the first parallel batch were of that shape. Rebuild and
retest after each merge, not once at the end.

### A process note worth more than any of the above

Five pushes once went to a red CI because local runs were treated as
sufficient. Run the gate the platform needs (`scripts/check.ps1` on Windows,
the cargo gate on Linux) and, when CI is enabled, watch the run
(`scripts/watch-ci.ps1` accepts any revision spelling).

### CJK popups "spaced out" in kitty, WezTerm and herdr (images on)

- Only where pictures are drawn (kitty graphics / iTerm2 images, also through herdr with `kitty_graphics = true` and `ui.graph_renderer = "image"`), never in a pty test or foot: `terminal_image::write_images_layered` rewrites the cells a popup drew over a picture with ratatui's `backend.draw`, and the buffer holds a wide glyph's second half as a plain `" "`. Written, that blank is a real space one column right (every CJK char shifted the row, borders pushed out), and a rewrite starting on a second half erased its glyph. `without_continuations` drops the blanks (the backend then moves the cursor to the next cell itself) and `painted_cell` swaps a left-edge second half for its glyph. Any new code that writes buffer cells outside ratatui's own diff must do the same.
