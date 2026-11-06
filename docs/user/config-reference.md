**English** · [Čeština](cs/config-reference.md) · [Deutsch](de/config-reference.md) · [Español](es/config-reference.md) · [Français](fr/config-reference.md) · [Italiano](it/config-reference.md) · [日本語](ja/config-reference.md) · [한국어](ko/config-reference.md) · [Nederlands](nl/config-reference.md) · [Polski](pl/config-reference.md) · [Português (Brasil)](pt-BR/config-reference.md) · [Português (Portugal)](pt-PT/config-reference.md) · [Русский](ru/config-reference.md) · [Svenska](sv/config-reference.md) · [Türkçe](tr/config-reference.md) · [Українська](uk/config-reference.md) · [简体中文](zh-CN/config-reference.md)

# Configuration reference

Every key of `config.toml`, its default and what it does. You rarely need to
edit the file by hand: the [Settings screen](settings.md) (`Ctrl+,`) writes it as
you change things, and `gitpoulpi config set` does too.

## Where it is, and how a value is chosen

| | |
|---|---|
| File | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` prints it. |
| Another folder | `GITPOULPI_CONFIG_DIR=<dir>` moves the whole config folder (the file, `sessions/`, the daemon port files): a portable install, a test, or an agent kept away from your own settings. |
| Precedence | built-in default, then `config.toml`, then the `GITPOULPI_*` variable, then a command-line flag. Highest wins. A wrong value (a misspelled choice in the file, a bad variable) is an error, never silently ignored. |
| Variables | Uppercase the key and turn dots into underscores: `general.auto_prune` is `GITPOULPI_GENERAL_AUTO_PRUNE`. Only for keys GitPoulpi actually reads (not the ones marked *not used* below, nor lists such as `ui.log_columns_hidden`). |
| Flags | `--backend git2\|system` overrides `general.default_backend` for a daemon that command starts. |
| Tokens and keys | Never in this file. Hosting tokens and AI API keys go to your system credential helper (Git Credential Manager, libsecret / `pass`, Keychain); the file only holds a reference (`token_ref`). Without a helper, saving a token fails with a message and the token lasts for the session only. |

```
gitpoulpi config path                          # the file, and whether it exists yet
gitpoulpi config get general.auto_stash        # the value, the layer it came from (default/file/env/flag), and whether anything reads it
gitpoulpi --json config get ui                 # a whole section
gitpoulpi config set general.auto_stash always # checked against the schema, then written
```

`config set` keeps the file sparse (only keys somebody set) and keeps keys it was
not asked about, but drops comments. It refuses lists and whole tables, except
`keybindings.*` (several chords separated by spaces; empty unbinds). Edit
`[[profiles]]`, `[[integrations]]` and `[[ai.profiles]]` in Settings or by hand.

In the tables, **Restart** means the value is read at start-up; everything else
applies at once (or at the next refresh, fetch or Settings close).

## `[general]`

| Key | Default | Values and effect |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2, falling back to the `git` program for staging, merge, rebase, stash and network) or `system` (the installed `git` only). A running daemon keeps its backend: **restart** it (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | When local changes block a checkout, pull, merge or rebase: `ask` (a dialog in the TUI; the CLI and MCP fail and name `--autostash`), `always` (stash, run, pop) or `never` (refuse). See [Auto-stash](log-and-history.md#auto-stash). |
| `general.include_untracked_in_autostash` | `false` | The auto-stash also takes untracked files. |
| `general.initial_commits_in_graph` | `5000` | Commits the Log loads first (at least 500). Higher costs memory and drawing time on huge repositories. |
| `general.lazy_load_commits_in_graph` | `true` | Load older commits as you scroll near the end. Off: the Log stops at the initial count. |
| `general.commits_per_lazy_page` | `1000` | Commits per background page (at least 50). |
| `general.max_commits_in_graph` | `100000` | Where lazy loading stops (about 300 bytes per commit); `0` = no limit. |
| `general.auto_fetch_interval_minutes` | `5` | Background Fetch All every N minutes, 0 to 60; `0` = off. **Restart.** |
| `general.auto_fetch_interval_unit` | `"auto"` | The unit Settings shows the interval in: `minutes`, `hours` or `auto`. Display only: the interval is always `auto_fetch_interval_minutes` minutes. `auto` (and a config without the key) picks the largest unit that divides the minutes evenly (60 reads "1 hour", 45 "45 minutes"); a unit that does not fit is ignored. Settings writes both keys; an older GitPoulpi ignores the unit and still reads the minutes. |
| `general.fetch_on_tab_open` | `true` | Fetch All when a tab opens (skipped if GitPoulpi fetched that repository less than 60 s ago). |
| `general.auto_prune` | `false` | Fetches also prune remote-tracking branches that are gone. |
| `general.auto_prune_tags` | `false` | A fetch also deletes the local tags the remote has deleted (only tags that were on it; a tag you never pushed is kept). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Refresh of the pull request / issue lists and the toolbar chip, 0 to 120; `0` = only on demand and on events. |
| `general.hosting_refresh_unit` | `"auto"` | The unit Settings shows the refresh in: `minutes`, `hours` or `auto`, read like `general.auto_fetch_interval_unit`. Display only. |
| `general.keep_submodules_up_to_date` | `false` | After a checkout or pull, `git submodule update --init --recursive` when a submodule is out of step. See [Submodules](submodules.md). |
| `general.default_branch_name` | `"main"` | First branch of a repository created with Init; empty lets git decide. Renames nothing. |
| `general.delete_orig_files_after_merge` | `false` | After a merge that lands, delete the `*.orig` backup files. |
| `general.remember_tabs` | `true` | Reopen last session's tabs. Off, a launch inside a repository opens only that repository, and a launch outside one opens only Home. **Restart.** |
| `general.radar_enabled` | `true` | The [Radar](radar.md): its tab, palette commands, `Alt+R` and its background fetching. Off hides all of it and nothing is fetched for it. Applies at once; not shown in a build without the `radar` feature. |
| `general.startup_tab_in_repo` | `"current-repo"` | The tab a start opens on when the folder it starts in is inside a repository: `"current-repo"` (that repository's tab, opened if needed), `"last"` (the tab that was active at the last quit), `"home"` or `"radar"`. A repository given with `--repo` always opens its own tab. `"last"` is Home with `remember_tabs` off or when nothing is remembered; `"radar"` is Home when the Radar is off or not built. Read once at startup; needs a restart. |
| `general.startup_tab_outside_repo` | `"home"` | The tab a start opens on when the folder it starts in is not a repository (a user folder, for example): `"home"`, `"last"` or `"radar"`. `"last"` is Home with `remember_tabs` off or when nothing is remembered; `"radar"` is Home when the Radar is off or not built. Read once at startup; needs a restart. |
| `general.startup_tab_launcher` | `"last"` | The tab a start opens on when GitPoulpi is started from the desktop or Start menu shortcut it wrote (they pass `--launcher`), whatever the folder: `"home"`, `"last"` or `"radar"`. Wins over the two settings above. `"last"` is Home with `remember_tabs` off or when nothing is remembered; `"radar"` is Home when the Radar is off or not built. Read once at startup; needs a restart. |
| `general.group_worktree_tabs` | `true` | Keep the tabs of a repository's linked worktrees right after that repository's tab: when a worktree opens, when a worktree tab is dragged away (it snaps back next to its group; the repository tab is the group's head and its worktrees follow it) and when the setting is turned on. Off leaves tabs where you put them. |
| `general.undo_depth` | `100` | How many operations Undo can step back; `0` = unlimited. |
| `general.conflict_detection` | `true` | Proactive conflict detection: a badge on the Log when merging the current branch into the branch it started from would conflict, computed in the background (an in-memory merge: nothing in your files, index or refs changes). A repository can turn it off for itself (`[preferences] conflict_detection`). See [Resolving merge conflicts](merge-conflicts.md#conflict-prevention). |
| `general.dev_mode` | absent (automatic) | For people who build GitPoulpi from a checkout; never shown in Settings. Absent: on when the running exe sits in `<checkout>/target/<profile>/` (`Cargo.toml` and `crates/gitpoulpi-tui` next to `target`). `true` / `false` forces it. Dev mode only unlocks `ui.launcher_build`. |
| `general.journal_outside` | `true` | Record what a shell or an AI agent does to branches, tags, stashes and HEAD as entries of the undo history, marked outside, so `Ctrl+Z` can take it back. Edits to files are never captured. Needs `undo_enabled`. See [Undo and redo](undo-redo.md#changes-made-outside-gitpoulpi). |
| `general.undo_enabled` | `true` | `false`: no undo bookkeeping at all (fastest), the undo history is erased, the Undo/Redo buttons disappear. See [Undo and redo](undo-redo.md). |
| `general.active_profile` | `""` | Id of the `[[profiles]]` identity commits are made with. Empty, or a profile with a blank name or email: git's `user.name` / `user.email`. |
| `general.external_editor` | `""` | Command for a file's *Open in editor*, e.g. `code -r {file}`. Empty: VS Code when installed, else the system default. |
| `general.external_diff_tool` | `""` | Command for *Open in external diff tool*: `{local}` (committed / HEAD version), `{remote}` (working-tree / index version). Empty: `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Command for *Open in external merge tool* on a conflict card (a binary, LFS or too large file): `{base}`, `{local}` (A, current), `{remote}` (B, incoming) and `{merged}` (the file the tool writes the result to; GitPoulpi stages it when the tool exits successfully). Empty: `git mergetool --no-prompt`. |
| `general.file_explorer` | per OS | Command for *Show in folder*: `{path}` (the file), `{dir}` (its folder). Defaults: `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

The three command templates are split into arguments first (`"..."` groups a
word), then the placeholders are filled in, so paths with spaces stay one
argument. No shell is involved.

```toml
[[profiles]]                # commit identities (Settings > Profiles)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log and graph**

| Key | Default | Values and effect |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (a picture where the terminal can draw one, characters elsewhere), `image`, `text`. **Restart.** |
| `ui.image_protocol` | `"auto"` | How pictures (graph, avatars) reach the terminal: `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = Kitty protocol in kitty, Ghostty and a herdr with `kitty_graphics = true`; iTerm2 in WezTerm, iTerm2, Rio, mintty; sixel (avatars only) in Windows Terminal, foot, Konsole, mlterm; none in tmux, zellij, screen and unknown terminals. **Restart.** |
| `ui.theme` | `"system"` | The color theme: a built-in slug (`gitpoulpi theme list` lists them all), the slug of a file in `<config>/themes/`, or `system` (follows the terminal's light or dark background, detected at start-up; the old value `default` means the same). `config set` rejects unknown names. See [Themes](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Lane palette: `theme` (follow the theme) or any lane palette name: `gitpoulpi`, `light`, `material`, `serie`, and the palettes of the other themes. |
| `ui.graph_lane_background` | `"off"` | `off` or `row_tint` (a faint band of the lane color behind the graph, plus a bar at its edge). **App: Toggle Graph Row Tints** (`Ctrl+N Shift+T`) flips it; **App: Change Graph Color Theme...** (`Ctrl+N Shift+Y`) picks `ui.graph_color_theme`. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (only with `row_tint`). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` and `right` draw a column; a window under 80 columns falls back to `bottom`. |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end`: where the buttons sit along the toolbar (left, middle, right of a row; top, middle, bottom of a column). |
| `ui.toolbar_pr_pinned` | `true` | The pull request button is pinned to the far end of the toolbar; `false` lets it flow with the other buttons. |
| `ui.statusbar_position` | `"bottom"` | `top` (above the repository tabs) or `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (the last row of the window, below the status line) or `left` (a column at the left edge; under 80 columns it stays on top). |
| `ui.graph_show_stashes` | `true` | A row per stash in the Log. |
| `ui.ref_connector` | `true` | A thin line from a ref pill to its commit. |
| `ui.column_resize` | `"line"` | How a column boundary is grabbed in the Log, the Radar and the commit column of the diff, file and blame views. `line`: each boundary is a one-cell gap with a small tick in the header; while the pointer is on the boundary, in the header or anywhere down the table, a thin accent line is drawn over the full height and a drag from anywhere along it resizes. `header`: no gap and no tick; the first character of the header cell after the boundary lights up and a drag from there resizes (only the header). The keyboard sizing keys and the palette rows are the same in both modes. |
| `ui.ref_tree` | `true` | Group branches and tags on `/` into folders in the Refs sidebar (`y` toggles it). |
| `ui.graph_ghost_ref` | `true` | The Log's Ref column shows a faded pill with the name of the branch (else remote branch) a hovered commit belongs to, when the commit has no ref of its own and the column has room; never a tag. A deleted branch is inferred from the merge commit subject that merged it and drawn struck through. Hovering the pill names its kind. |
| `ui.reveal_on_ref_click` | `true` | A single click on a ref (branch, tag, remote branch, stash) in the Refs pane scrolls the Log to its commit and flashes the row; the commit selection does not move. |
| `ui.log_wip_counts_in_message` | `true` | WIP change counts in the WIP message while the Changes column is hidden. |
| `ui.log_columns_layout` | `"fit"` | How the Log lays its columns out. `fit`: every visible column fits the pane exactly and only `message` flexes; when the pane is narrow whole columns give way. `scroll`: columns keep their widths (`message` too), nothing gives way, and the table scrolls sideways when it is wider than the pane. |
| `ui.log_column_widths_scroll` | empty | The `scroll` layout's column widths, name -> cells (`message` allowed, unlike `ui.log_column_widths`, which keeps the `fit` layout's widths and never holds `message`). Written when you drag, size or fit a column while the layout is `scroll`. |
| `ui.log_columns_hidden`, `ui.log_columns_order`, `ui.log_column_widths` | empty | The Log columns (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`), written when you hide, drag or resize a column. `message` cannot be hidden. |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | empty | The parts of the [commit column](interface-tour.md#the-commit-column) beside a diff, on their own, apart from the Log's: the shown parts by name (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha`; empty is lanes, message, author and date), their order left to right by the same names (empty is the order just listed; a name left out follows in that order) and the dragged widths in cells per part (a part left out is automatic), written when you choose, move or size the parts. `message` cannot be hidden. A session file written before these keys existed is read once into them. |
| `ui.sidebar_sections_hidden` | empty | The left-pane sections you hid (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`), written from the pane's right-click menu or the palette. At least one existing section always stays shown. This is the default: each repository can hide its own sections (kept in its state file), and this key applies to a repository with no choice of its own; *Use for all repositories* writes it. |
| `ui.sidebar_sections_scope` | `"repository"` | Whose choice the left-pane sections are: `repository` (each repository keeps its own, `ui.sidebar_sections_hidden` being the default) or `global` (every repository shows `ui.sidebar_sections_hidden`, and a change is written there). Switching never loses either set: the per-repository choices stay in their state files while `global` is on. Settings: *Left pane sections*. |
| `ui.language` | `"auto"` | The interface language: `auto` (the system locale, else English), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Live.** `GITPOULPI_LANG` overrides it. A language that is not in this build shows English. **App: Change Language...** (`Ctrl+N g`) sets it too, with a live preview. CLI and MCP output stays in English. |
| `ui.translate_git_terms` | `false` | `false`: commit, push, pull, stash, tag... stay English in every language. `true`: the chosen language's git vocabulary (French "remiser", "étiquette", "valider"...). **Live.** |
| `ui.git_terms_english` | `false` | Only ja, ko and zh-CN: show the English git words (stage, unstage, staged, unstaged...) instead of their transcription. No effect while `ui.translate_git_terms` is `true`. **Live.** |
| `ui.mouse_enabled` | `true` | `false` leaves the mouse (and text selection) to the terminal. **Restart.** |
| `ui.mouse_scroll_lines` | `3` | Log rows per wheel notch, 1 to 20. **Restart.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` asks the terminal for the kitty keyboard protocol (so `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` arrive); `off` never asks. **Restart.** |
| `ui.logo_color` | `""` | Colorway of the logo in the About window, a slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`); empty or unknown means the build's default (`lavender`). Written when you click the logo or pick a color in About. |

**Avatars** (off by default; turning them on sends a SHA-256 hash of author
emails to Gravatar, and asks your connected GitHub/GitLab account)

| Key | Default | Values and effect |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (in the commit circle of the picture graph), `author` (in the Author column), `off`. The old `ui.show_avatars` (`true`) still reads as `author` while `ui.avatars` is absent. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Lookup order: `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. A GitLab host is only asked through a connected profile. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (generated locally) or `initials`. On a terminal with no picture protocol the initials badge is always used. |

**Diff and editor display**

| Key | Default | Values and effect |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` or `split` (`m` in a diff). |
| `ui.diff_hunks_only` | `true` | `false` shows the whole file with the changes in place (`H`). |
| `ui.diff_show_whitespace` | `true` | Show whitespace-only changes (`s`). |
| `ui.diff_word_wrap` | `false` | Wrap long diff lines (`w`; Settings > Diff & editor). |
| `ui.minimap` | `false` | A minimap beside the scrollbar of the diff and the editor (`M`). |
| `ui.mermaid_detail` | `refs` | Detail of the Log's Copy as Mermaid gitGraph: `structure`, `refs`, `subjects`, `full` (remembered from the popup; the default of `log --format mermaid`). |
| `ui.mermaid_orientation` | `lr` | Direction of that graph: `lr` (left to right) or `tb` (top to bottom). |
| `ui.mermaid_fence` | `false` | Wrap the copied Mermaid script in a `mermaid` code fence. |
| `ui.mermaid_subject_width` | `40` | Commit subjects are cut to this many characters in the Mermaid ids (`subjects`, `full`). |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` or `16`: how many colors to draw with. `auto` is truecolor except on Apple Terminal (256), the Linux console (16) and `*-256color` terminals without `COLORTERM` (256). Live. See [Themes](themes.md). |
| `ui.input_color` | `"theme"` | Text-field background: `theme` (the theme's own), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | The `custom` color, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | The AI commit composer opens with each proposal's files `expanded` or `collapsed`. |
| `ui.blame_side` | `"right"` | Which side of the Blame view holds the annotation column (avatar, author, age, summary): `right` (default) or `left`. |
| `ui.blame_width` | `46` | The Blame annotation column's width in cells (24 to 90). Drag its border, or press `<` / `>` in the view; the new width is saved here. |
| `ui.pr_terminology` | `"auto"` | `auto` (pull request on GitHub, merge request on GitLab), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Hides the toolbar chip of the checked-out branch's latest pull or merge request once it is closed or merged; the dim "+ PR" chip (start a new one) shows instead, as if the branch had none. `false`: the chip always shows the latest request, whatever its state. |

The diff toggles you change inside a diff are saved here at once and used by
every diff opened afterwards.

**Dates**: `ui.date_format` (`"YYYY-MM-DD"`, the Blame / File History date column),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, the Log's Date column) and
`ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, the commit details, the Date
tooltip and *Copy > Date*). Tokens: `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D`
`dddd` `ddd` (weekday, full / abbreviated) `HH` `mm` `ss`; anything else is
copied as is. Month and weekday names follow `ui.language`, and a month written
after a day number is the genitive form where the language has one (Russian,
Ukrainian, Polish, Czech). A `D MMMM YYYY` run is written year-first in Japanese,
Chinese and Korean (`2024年3月5日`). `ui.date_locale` (`"system"`, the default, or
`"english"`) keeps the names English whatever the language.

**Agents, terminals and the tab folder**

| Key | Default | Values and effect |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | The window accepts `ui show` / `ui propose` from agents at all (Settings > Agents & MCP > *Allow AI agents to drive the view*). Off, no agent reaches it. See [MCP and agents](mcp-and-agents.md#showing-things-to-the-user). |
| `ui.launch` | `"off"` | Whether an agent may open a GitPoulpi window when none is open: `off`, a preset (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) or `custom`. |
| `ui.launch_command` | `""` | The `custom` template: `{repo}`, `{session}`, `{gitpoulpi}`. Split into arguments, no shell. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (the tab row offers *Quit & cd here*) or `auto` (every quit changes the shell's folder, and a new tab runs the default *Open in* entry). See [Following the tab folder](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Legacy, see below and [Open in](open-in.md). How the old *Terminal here* opened a terminal: `auto` (the `ui.launch` preset, else `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), a preset, or `custom`. |
| `ui.terminal_launch_command` | `""` | The `custom` template; `{repo}` is the folder. |
| `ui.open_in` | *(absent)* | The [Open in](open-in.md) list, an array of tables `[[ui.open_in]]` with `label` and `command`. Absent means "not configured yet": the list is computed from the legacy `ui.terminal_launch*` keys (a non-`auto` value becomes the first entry), the OS file explorer, the multiplexers and terminals detected from the environment (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) and the editors found on `PATH`. The first save from Settings writes the list. `open_in = []` is an empty list. Not settable with `config set` (edit the file). |
| `ui.open_in_default` | `""` | Label of the entry "Open in default" runs; empty or unknown means the first entry. |
| `ui.launch_terminal` | `"auto"` | The terminal the desktop shortcuts (Settings > General > Launcher) and the Windows relaunch start GitPoulpi in. `auto` picks from what is installed (Windows: Windows Terminal, else WezTerm, Alacritty, the plain console; Linux: `xdg-terminal-exec`, else the first terminal found; macOS: iTerm2, Ghostty, WezTerm or kitty if installed, else Terminal), `default` is the OS's own, or a terminal id such as `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal`. |
| `ui.launcher_build` | `"current"` | Dev mode only (see `general.dev_mode`; shown in Settings > Launcher only then, ignored otherwise). `"current"`: shortcuts and the command alias start the build that made them. `"newest"`: they start the most recently built of `target/dist` and `target/release` of the checkout, decided at each click, and starting either build never rewrites them. |
| `ui.command_alias` | `""` | The name of the command that starts GitPoulpi from a terminal, for example `gp` (Settings > General > Launcher, `gitpoulpi alias create`). Empty: none. Letters, digits, `-`, `_` and `.` only. It is only the name the **Create: command alias** button uses; nothing exists until you create it. |
| `ui.command_alias_add` | `""` | The name of the second command, for example `gpa`, that runs `gitpoulpi --add`: it opens the current folder as a new tab of the GitPoulpi that is already running (Settings > General > Launcher, `gitpoulpi alias create gpa --add`). Empty: none. Same name rules; created together with `ui.command_alias` by the **Create** button. |
| `ui.relaunch_in_terminal` | `true` | Windows only: when the exe is started by a double-click into a console of its own, GitPoulpi reopens itself in `ui.launch_terminal` and exits. Off: it stays in that console. |
| `ui.reuse_window` | `true` | A click on a GitPoulpi shortcut (`gitpoulpi launch`) shows the window of a GitPoulpi that is already open instead of starting another one. Off: every click starts a new window. |

**`ui.open_in` commands** are split into arguments first (whitespace, `'...'`, `"..."`), then the variables are substituted inside each argument, so a folder with spaces stays one argument; no shell is involved unless the command starts one (`sh -c '... "$GITPOULPI_CWD"'`). Variables: `${cwd}` (the tab's folder), `${repo}` (repository root), `${name}` (folder base name), `${branch}` (empty when detached), `${web_url}` (the `origin` remote's web page when it is a GitHub or GitLab project, else empty; an entry using it refuses to run when empty), `${env:NAME}` (empty when unset). The old `{repo}` still means `${cwd}`. Any other `${...}` is an error naming it. Example:

```toml
[ui]
open_in_default = "Herdr: new tab"

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Legacy: `ui.terminal_launch` and `ui.terminal_launch_command` are still read (they seed the list while `ui.open_in` is absent: `custom` becomes "Terminal (custom)" with `{repo}` rewritten to `${cwd}`, a preset id becomes that preset) but are superseded by `ui.open_in`.

### Images

| Key | Default | Meaning |
|---|---|---|
| `ui.image_view` | `true` | Show a binary file that is an image (PNG, JPEG, GIF, WebP, BMP, ...) as a picture in the Diff view and the File View. Off: the plain binary notice. |
| `ui.image_view_max_mib` | `32` | Images bigger than this (MiB) are not decoded; an info card shows instead. |
| `ui.image_view_max_unit` | `"auto"` | The unit Settings shows the limit in: `mib`, `gib` (1024 MiB) or `auto`. Display only: the limit is always `image_view_max_mib` MiB, 1 to 1024. |
| `ui.image_view_text_fallback` | `"halfblock"` | On a terminal without an image protocol: `halfblock` (colored half-block characters) or `card` (info card only). |
| `ui.image_view_pixelated_from` | `"2x"` | Zoom from which enlarging shows square pixels instead of smoothing: `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | The before and after panes share one zoom and scroll position. |
| `ui.image_editors` | *(absent)* | The "Edit in" image editors, an array of tables `[[ui.image_editors]]` with `label` and `command`. Absent means "not configured yet": the list is the editors detected on this machine (programs on `PATH`, Flatpak installs, known install folders on Windows, `/Applications` on macOS), then the OS default app. The first save from Settings writes the list. `image_editors = []` is an empty list. Not settable with `config set` (edit the file). |
| `ui.image_editor_default` | `""` | Label of the editor "Edit in default" runs; empty or unknown means the first entry. |

**`ui.image_editors` commands** are split into arguments first (whitespace, `'...'`, `"..."`), then the variables are substituted inside each argument, so a path with spaces stays one argument; no shell is involved. Variables: `${file}` (the image's absolute path), `${dir}` (its folder), `${name}` (its file name), `${repo}` (the repository root, empty for a temp copy), `${env:NAME}` (empty when unset). Any other `${...}` is an error naming it. An image that is not the working-tree file (a commit's version, the index, a deleted file) is opened as a **read-only temp copy**, so an editor cannot touch the repository. Editors with no command line (Affinity Photo from the Microsoft Store, some Photoshop installs) are not in the presets: write a custom entry. Example:

```toml
[ui]
image_editor_default = "GIMP"

[[ui.image_editors]]
label = "GIMP"
command = "gimp ${file}"

[[ui.image_editors]]
label = "Paint.NET"
command = "\"C:\\Program Files\\paint.net\\paintdotnet.exe\" ${file}"
```

**Not used yet** (accepted, saved, but nothing reads them):
`ui.graph_colorization` (removed from Settings;
still accepted on load), `ui.date_word_format` (removed from Settings; still
accepted on load). The old `ui.diff_view_mode`
is read once as `diff_layout` / `diff_hunks_only`.

## `[editor]`

| Key | Default | Values and effect |
|---|---|---|
| `editor.tab_size` | `4` | Columns per tab stop, 1 to 8 (editor and diffs). |
| `editor.eol` | `"auto"` | Line ending the built-in editor writes: `auto` (keep the file's), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Language colors in the editor. |
| `editor.line_numbers` | `true` | Line numbers in the editor and the diffs. |
| `editor.follow_selection` | `true` | With the editor open, selecting another file in the right pane opens it in the editor (asking to save first); a file of a commit leaves the editor for its diff. |

## `[merge]`

The merge conflict editor ([Resolving merge conflicts](merge-conflicts.md)). Settings > Diff & editor shows it.

| Key | Default | Meaning |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | The width, in columns, of the editor's area from which A and B are shown side by side (long lines scroll sideways: Shift+wheel, Shift+arrows). Narrower, it shows one side at a time behind tabs. `40` is the lowest value, `0` means always side by side (the notice under 40 columns stays). The editor hides the Refs and Status panels while it is open, so its area is usually the whole window. |

## `[git]`, `[credentials]`, `[ssh]`

| Key | Default | Values and effect |
|---|---|---|
| `git.pull_mode` | `"merge"` | What Pull (button, `Ctrl+G p`, `Git: Pull`) does: `merge` (fast-forward when possible), `ff_only`, `rebase`. Always passed to git explicitly. The old `fetch_all` reads as `merge`. |
| `git.push_behavior` | `"direct"` | `direct` pushes to the default remote; `ask` shows the remote menu first. |
| `git.default_encoding` | `"utf-8"` | Encoding for text that is **not** valid UTF-8 (file contents, blame, diffs, commit messages without an `encoding` header): any WHATWG label (`windows-1252`, `shift_jis`, ...). Display only; staging never re-encodes. A repository can override it. |
| `git.lfs_enabled` | `true` | Show and drive Git LFS. A repository can override it. See [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | Before a pull or push of a repository that uses LFS, install its LFS filters / pre-push hook (`git lfs install --local`, this repository only) when missing. See [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *Not used*: a repository's `core.autocrlf` / `core.eol` are git's own settings, edited in Settings > Git & encoding (or `gitpoulpi repo-config`). |
| `credentials.https_mode` | `"credential_helper"` | HTTPS authentication for fetch, pull, push, clone: `credential_helper` (git's), `prompt` (ignore helpers; git asks on a terminal, so it fails in the background daemon), `none` (public remotes only). Old `encrypted_file` values read as `credential_helper`. |
| `ssh.key_type` | `"ed25519"` | Type made by *Generate key pair*: `ed25519`, `rsa` (4096 bits), `ecdsa` (NIST P-256); the file is `gitpoulpi_<type>`. SSH itself always uses your ssh-agent and ssh config. |
| `ssh.private_key_path`, `ssh.public_key_path` | unset | Recorded by *Generate key pair*. When `ssh.private_key_path` is set, fetch, pull, push and clone run git with `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`; `ssh.public_key_path` does the same (ssh picks the matching agent key) when the private path is empty. A `GIT_SSH_COMMAND`/`GIT_SSH` in the environment or a `core.sshCommand` in git config (the per-repository pin, `gitpoulpi ssh-command set --key`) wins. Read before each transfer, so changes apply to the next one. |

## `[[integrations]]` (hosting and issue trackers)

Written by Settings > Integrations or `gitpoulpi hosting integrations add`. Every
kind has a client (personal tokens only, no OAuth).

```toml
[[integrations]]
id = "corp"                         # what --id takes; the token is stored under integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # the API root is derived from it
token_ref = "integration:corp"      # a reference, never the token
name = "Corp GitLab"                # optional display name
account = "me@example.com"          # optional, see below
enabled = true                      # false: ignored for host matching
```

A self-hosted server can be plain http: `url = "http://localhost:3000"` is kept
as typed (the scheme and port reach every request, and a remote on that host
matches the profile). A token sent over plain http travels unencrypted, so use
http only on a local or trusted network.

`kind` also accepts `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` and `gerrit`
(Gitea and Forgejo share one client; a Forgejo profile defaults to `https://codeberg.org`). Jira,
Trello, Linear, YouTrack and Redmine are issue trackers only; Gerrit is review only (its changes are the
pull requests, it has no issues). Bitbucket Cloud has no issue tracker.

`account` is the non-secret identity next to the token, empty and omitted for
most kinds: the Atlassian account email for `jira_cloud` and `bitbucket` (an
empty Bitbucket account means a workspace or repository access token, sent as a
Bearer), the API key for `trello`, the user name for `gerrit` (the token is its HTTP
password, sent as HTTP basic). The token itself stays the only secret. For
`azure_devops` the `url` is `https://dev.azure.com/<organization>`; an
`https://<organization>.visualstudio.com` URL is read as the same organization.
For `azure_devops_server` it is the collection root, such as
`https://tfs.example.com/tfs/DefaultCollection`.

`user` (the connected account) and `ssh` (the profile's key choice) are written
for you. The token comes from `GITPOULPI_TOKEN_<ID>` (id uppercased,
non-alphanumerics as `_`), else the stored token, else your credential helper
for that host when only one profile uses it (GitHub and GitLab profiles only: the
other kinds never read the credential helper). Details: [Hosting and issue-tracker
integrations](hosting-integration.md). The old `[[hosting]]` table is *not used*.

For `local_markdown` (a folder of Markdown notes as the issue source) `url` is the
folder, relative to each repository or absolute, and there is no token. Its
optional `[integrations.local]` table sets `statuses` (the board's column order,
default `todo`, `doing`, `done`), `done_status` (default `done`),
`reference_prefix` (`LOC` shows `LOC-12`) and `me` (who "assigned to me" means).
See [Local issues and Obsidian](local-issues.md).

### `[integrations.radar]` (account-wide lists)

Optional table of a code-host profile (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit; not Bitbucket Cloud). Absent
means every key off. Written by `gitpoulpi hosting integrations radar`; read by
`gitpoulpi hosting radar` and the Radar, and edited in Settings > Integrations (the Radar block of a connected profile).

```toml
[integrations.radar]
review_requests = true    # requests waiting for my review, every repository
open_requests = false     # every open request of my namespaces
include_drafts = false    # keep draft requests in these lists
assigned_issues = true    # open issues assigned to me, every repository
namespaces = ["acme"]     # orgs / groups to search for open requests
review_mode = "ask"       # ask | clone | api
```

| Key | Default | Meaning |
|---|---|---|
| `review_requests` | `false` | Open requests that ask for your review, whether or not the repository is cloned |
| `open_requests` | `false` | Every open request of your own namespace and of each org or group you belong to (or of `namespaces`), never "everything the token sees" |
| `include_drafts` | `false` | Draft requests are left out of these lists unless this is on |
| `assigned_issues` | `false` | Open issues assigned to you, every repository |
| `namespaces` | `[]` | Orgs / groups to search for `open_requests`; empty means your own namespace plus every membership (GitHub: at most 12 are searched, GitLab: 10 groups and 10 owned projects, Gitea, Forgejo and Gerrit: 10; Azure DevOps takes project names, Bitbucket Data Center project keys, Gerrit project path prefixes) |
| `review_mode` | `"ask"` | What reviewing a request whose repository is not cloned does: `ask`, `clone` (clone it, then review) or `api` (review through the host's API) |

The answers are cached per profile in `radar-account.json` next to
`config.toml` for five minutes; the file is a cache, safe to delete.

### The trust store (not in `config.toml`)

What each integration and API-key AI profile was trusted for (its address, the
account id, a pinned certificate key, whether plain http was allowed) lives in
`trust.toml` in the user-only private data folder (`<data>/private/trust.toml`:
`%APPDATA%\GitPoulpi\private\` on Windows, `~/.local/share/gitpoulpi/private/`
on Linux). It is deliberately not part of `config.toml`: whoever can edit the
configuration must not be able to widen trust by editing it. Change it with
`gitpoulpi hosting integrations trust` and `gitpoulpi ai trust` (see
[the security check](hosting-integration.md#the-security-check)); the file is
rewritten by GitPoulpi, not by hand.

## `[mcp]` (what AI agents may do)

| Key | Default | Values and effect |
|---|---|---|
| `mcp.allow_mutations` | `false` | Off: MCP tools that change anything are refused; reads, `ui_*` and proposals keep working. `true` lets agents change the repository. |
| `mcp.mutation_allowlist` | `[]` | Non-empty: only these tool names may change things (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false`: refuse the `ui_*` tools that move your window (not `ui_state` / `ui_viewers`). |
| `mcp.ui_control` | `"ask"` | `ask` (the window shows a chip you confirm), `auto` (applied unless you are busy), `off`. `--ui-control` on the server command overrides it per agent. Also applies to `gitpoulpi ui show`. |
| `mcp.allow_ui_propose` | `false` | Allow `ui_propose_*` / `gitpoulpi ui propose`: pre-filled dialogs you confirm yourself. |
| `mcp.allow_history_purge` | `false` | Only in `config.toml` (no Settings row). `true` lists the tool `rewrite_purge` for agents (it still needs `allow_mutations`). Off, an agent does not see the tool at all: a purge deletes the old history of a history rewrite, the undo of it and its reflogs for good. The CLI and the TUI are not gated by it. |
| `mcp.enabled` | `true` | `false`: `gitpoulpi mcp serve` and `gitpoulpi-mcp` refuse to start, with a message naming this key. Settings > Agents & MCP > *MCP server*. Read when an agent starts the server. |
| `mcp.bind` | `"127.0.0.1:8808"` | The address `--http` listens on when you give none (`--http ADDR` / `--bind` win). Loopback only unless `--allow-remote`. Empty or the old `"stdio"` mean the default. Settings > Agents & MCP > *MCP server port* edits the port and keeps the host. |
| `mcp.http_auth` | `true` | `--http` answers only a request with `Authorization: Bearer <token>` (the token of `gitpoulpi mcp token`). `false` removes the check; the server then warns on stderr, and `--allow-remote` is refused. Settings > Agents & MCP > *HTTP authentication*. |
| `mcp.tls` | `"off"` | `off`, `self_signed` (a certificate generated once under the private data folder, its SHA-256 fingerprint printed at start) or `files` (`tls_cert` and `tls_key`). With TLS on, the server terminates TLS itself and no plain listener exists. Settings > Agents & MCP > *TLS*. |
| `mcp.tls_cert` | `""` | PEM certificate chain used with `tls = "files"`. |
| `mcp.tls_key` | `""` | PEM private key used with `tls = "files"`. |
| `mcp.behind_proxy` | `false` | You state that a tunnel or reverse proxy in front of the endpoint carries the encryption. It cannot be checked from inside the server, so `--allow-remote` without TLS needs it set. The hop between the proxy and the server is plain text on that network. Settings > Agents & MCP > *Behind a tunnel or reverse proxy*. |
| `mcp.allow_sensitive_staging` | `false` | Only in `config.toml`. Off, the MCP stage tools do not offer `allow_sensitive`, and a stage that would add a flagged file (see `[privacy]`) is refused with a message telling the agent to ask you. On, an agent may pass `allow_sensitive`. The CLI and the TUI are not gated by it: they ask you. |

`allow_mutations`, `mutation_allowlist` and `allow_ui` gate the MCP server only;
the CLI is your own shell and is not gated by them. See [MCP and
agents](mcp-and-agents.md#permissions).

## `[ai]` (built-in AI, off by default)

| Key | Default | Values and effect |
|---|---|---|
| `ai.enabled` | `false` | The switch. Off, nothing AI shows anywhere and nothing is sent. |
| `ai.default_profile` | `"default"` | Id of the profile actions use when they name none. |
| `ai.global_instructions` | `""` | Text added to every action's prompt. May use `${repo}`, `${uilang}`, ... (see [AI > Prompt variables](ai.md#prompt-variables)); same for the per-action and repository instructions. |
| `ai.actions.<action>.profile` | `""` | Profile for one action; empty = the default. Actions: `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`. A workspace (`workspaces.toml`, `[workspace.ai_instructions]` and `[workspace.ai_overrides.<action>]`) and a repository can add to or replace these instructions: see [AI](ai.md#instructions). |
| `ai.actions.<action>.instructions` | `""` | Text added for that action only. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` or `no`: whether the local agent of a code review draft works inside the repository for that run, overriding the AI profile's `workdir`. A repository can set its own (`gitpoulpi ai review-context --repo`). See [AI > Review context](ai.md#review-context). |
| `ai.review.include_claude_md` | `true` | Give a code review draft the repository's `CLAUDE.md` (root, read at the branch the request targets; cut at 8 KB). |
| `ai.review.include_agents_md` | `true` | Same for `AGENTS.md`. |
| `ai.review.extra_file` | `""` | One more file for a code review draft: a path relative to the repository root, an absolute path, or a link to a file of a code host (read through the matching integration profile, else anonymously). All the files together are cut at 16 KB. |
| `ai.privacy.redact_secrets` | `true` | Replace every secret the scanner recognises in a request (a token, a private key block, a password in a URL, a `password = ...` value that looks random) with `[REDACTED:<rule>]` before it leaves. Settings > AI shows a red warning while it is off. |
| `ai.privacy.exclude_sensitive_files` | `true` | Leave out the diff of every file matching `privacy.sensitive_patterns` from what is sent. A local agent that works in the repository folder can still read files itself: see [AI > Privacy](ai.md#privacy). |
| `ai.sent_preview.<action>` | `true` | One key per action (`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`): show "What is sent" before the request goes out. The dialog's "Don't show again for ..." writes `false`; Settings > AI > *Show what is sent before...* turns them back on. |

A repository's own `[preferences.ai_review]` table (in its `repo-state.toml`, or the entry of a repository that is
not cloned in `remote-reviews.toml`) takes the same keys, all optional, plus `no_extra_file = true`: no extra file for
that repository, whatever `ai.review.extra_file` says. An unset key, an unset `extra_file` and a missing
`no_extra_file` inherit the global value; setting `extra_file` to a file clears the switch and the other way round.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # empty = the kind's default
base_url = ""           # empty = the kind's default (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # where the API key is stored; never the key
command = ""            # local_agent only: the exact program ("claude", an alias, a wrapper, a path)
harness = ""            # local_agent only: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; empty = inferred from `command`
args = []               # local_agent only, e.g. ["-p", "--model", "haiku"]
run_mode = "headless"   # local_agent only: headless (default) | terminal (the agent's interactive screen in a virtual terminal)
terminal_args = []      # local_agent only: the arguments of the terminal mode (args keeps the headless set)
workdir = "default"     # local_agent only: default | dedicated | repository (default = dedicated folder in terminal mode, GitPoulpi's folder in headless mode)
timeout_secs = 0        # local_agent only: seconds before the agent is stopped; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # local_agent only, optional: variables set for that process only
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = one of GitPoulpi's own variables
```

`harness` picks the suggestions Settings > AI offers (headless arguments,
options, model flag and list, environment variables) independently of
`command`; both keys are omitted from the file when empty. Environment values
are never logged and are hidden in Settings when they look like secrets.

Settings > AI offers presets for `harness`/`command`/`args`/`env` (the common agent CLIs, with
their headless flags, model flag and useful environment variables) and for `kind`/`base_url`/`model` (the
hosted and local HTTP providers); see [AI > Configure](ai.md#configure-the-ai).

A profile named *Default* is created when there is none. Old `[[ai.presets]]` /
`[ai.defaults]` files are migrated on load and kept in `[[ai.legacy_presets]]`.
Everything else: [AI assistance](ai.md).

## `[repos]`, `[session]`

| Key | Default | Values and effect |
|---|---|---|
| `repos.clone_root` | `""` | Where the Clone dialog starts; empty = `%USERPROFILE%\source\repos` or `~/src`. |
| `repos.clone_start_source` | `"last"` | The source the Clone dialog opens on: `last` (default, or empty) = the one it showed when it was last closed, `url` = always Clone with URL, or an integration profile id (`id` in its `[[integrations]]` entry). A profile that is gone, disabled or not connected falls back to Clone with URL. The last source itself is state, kept in `clone-source.toml` in the config folder, not here. |
| `repos.browse_root` | `""` | Where Browse opens; empty = the last folder chosen, then home. |
| `repos.show_hidden` | `false` | The folder picker shows hidden folders (`Alt+H` toggles and saves it). |
| `repos.show_siblings` | `false` | The folder picker's left tree also lists the other folders of every open level, as a real hierarchy (`Alt+S` toggles and saves it). |
| `session.per_repo_state_dir` | `"in_git_dir"` | Where GitPoulpi keeps what it remembers about one repository (ref filters, fold state, the personal workflow override, the hosting cache): `in_git_dir` = `<git dir>/gitpoulpi` (shared by all worktrees, never shows in `git status`), `in_worktree` = `<repo>/.gitpoulpi` (the only place a team can commit `workflow.toml`), `user_data` = `<OS data folder>/repos/<name>-<id>` (`%APPDATA%\GitPoulpi\data` on Windows, `$XDG_DATA_HOME/gitpoulpi` on Linux, `~/Library/Application Support/GitPoulpi` on macOS). The old value `".gitpoulpi"` still means `in_worktree`. Changing it moves nothing back: data found in a legacy `<repo>/.gitpoulpi` is moved to the chosen place the first time it is needed. |

## `[log]` (the application log)

What the Activity Log's "Application" tab shows and "Copy diagnostics" quotes. Details: [When something fails](troubleshooting.md#when-something-fails).

| Key | Default | Values and effect |
|---|---|---|
| `log.level` | `"info"` | The most verbose line that is recorded: `error`, `warn`, `info`, `debug` or `trace`. The level box in the Activity Log changes it (and saves it here). |
| `log.retention_days` | `7` | Days of log files kept, today included: 1 to 3650, ten years (a larger or smaller value is clamped). Always a number of days, whatever unit Settings shows it in. Older files are deleted when GitPoulpi starts. A day's file is also capped by `log.max_file_mb`, so the worst case is about `2 x max_file_mb x retention_days`. |
| `log.retention_unit` | `"auto"` | The unit Settings shows `log.retention_days` in: `days`, `weeks`, `months`, `years` or `auto`. Display only, it never changes how long files are kept. A week is 7 days, a month is 30 days (not a calendar month, so the same number of days whenever it is applied) and a year is 365 days. `auto`, and a config written before this key existed, picks the largest unit that divides the days evenly (14 days read "2 weeks", 45 days "45 days"). A unit that does not divide the days (`retention_days = 10` with `weeks`) is ignored and the days are shown. Settings writes both keys; an older GitPoulpi ignores the unit and still reads the days. |
| `log.max_file_mb` | `5` | A day's file is rotated to one `.old` backup at this size, so a day never takes more than twice this. |
| `log.max_file_unit` | `"auto"` | The unit Settings shows the size in: `mib`, `gib` (1024 MiB) or `auto`. Display only: the size is always `max_file_mb` MiB, 1 to 1024. |
| `log.timings` | `true` | The Activity Log's Git operations tab shows how long each operation took (`Push - 123 ms`: open a repository, refresh, fetch, pull, push, commit, checkout, merge, rebase, stash, clone). The durations are measured by the window, from the request to the daemon's answer, so they include the IPC round trip. Off hides the durations and the entries for successful operations. Failures are always listed. |
| `log.extended` | `false` | "Use extended logging": the Git operations tab also lists the background operations (the periodic refresh that polls the status, auto-fetch) with their durations. They are always written to the application log at `debug` level. |
| `log.commit_count` | `"system-git"` | How the status line's `loaded / total` learns the total. `"system-git"` runs `git rev-list --count` (one process, fastest on a normal machine: git reads the commit-graph file). `"libgit2"` counts in-process with a libgit2 revwalk and spawns nothing: pick it when every command-line process is slow to start, typically because antivirus software with real-time scanning inspects each one; on 50,000 commits it takes about 200 ms in the background instead of 12 ms, but no process is started. `"off"` never counts: the bar shows only the loaded commits. Applies live. A path-filtered count always uses system git. |
| `log.use_mailmap` | `true` | Show each author as the repository's `.mailmap` (and `mailmap.file`) names them in the Log, in blame and in file history: the daemon asks git for `%aN`/`%aE` (the system backend) or applies the same files itself (the libgit2 backend), so the answer is identical. The stored commits are not changed and nothing is rewritten. Searching the Log by author matches the names as shown. Applies live; the Log re-reads within its 2 s poll. |

Files are `gitpoulpi-YYYY-MM-DD.log` in the OS state folder (`~/.local/state/gitpoulpi/logs` on Linux; `<cache>\logs` on Windows and macOS; `<config>/cache/logs` under `GITPOULPI_CONFIG_DIR`). Tokens, passwords and `Authorization` headers are masked before anything is written.

## `[search]` (the workspace search)

What `gitpoulpi search` (and its MCP tool) looks through. Details: [Workspaces and search in the CLI](cli-reference.md#workspaces-and-search).

| Key | Default | Values and effect |
|---|---|---|
| `search.commit_limit` | `200000` | Commits (newest first, over every branch, remote branch and tag) whose messages the search index keeps in memory per repository, about 100 bytes plus the message each. A larger history is cut there and the answer says it is `partial`. `0` turns commit search off (branches, tags, stashes and worktrees are still searched). Read live: the next search obeys a change. |

## `[rewrite]` (history rewrite)

What `gitpoulpi rewrite` and the history rewrite workbench protect. Details: [History rewrite](history-rewrite.md).

| Key | Default | Values and effect |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Branches (names, or patterns where `*` matches any text) that a rewrite leaves alone when they are already pushed, unless the run lists them (`--include-protected BRANCH`; `--include-all-protected` for the push; the checkbox per branch in the workbench). Added to the bases that a repository's `workflow.toml` marks `protected`. The push of a rewrite skips them the same way. This is GitPoulpi's own guard: the host's branch protection still decides whether the force push is accepted. Read live: the next preview obeys a change. |
| `rewrite.external_scripts` | `["shell"]` | Which external script languages the Script rule may use: any of `"shell"` (bash or sh; pwsh or powershell for PowerShell scripts), `"python"`, `"lua"` (an installed Lua; the embedded Lua is always on and needs no entry). **An external script is not sandboxed: it runs with your rights.** A language that is not listed is greyed out in the workbench and refused (`scripts_disabled`) by the CLI, MCP and daemon. Even when listed, an external script that comes from the repository or a shared recipe runs only after you trusted it (`gitpoulpi rewrite script trust FILE`, or the question in the workbench); the trust is keyed by the script's content hash, so a changed script asks again. Switch Python on by editing `config.toml` (a list cannot be set with `config set`): `[rewrite]` then `external_scripts = ["shell", "python"]`. Read live. |
| `rewrite.script_timeout_ms` | `5000` | How long an external script gets to answer one commit. On a timeout the whole process tree is killed and the rewrite fails (`script_failed`, naming the commit); nothing has moved. `0` reads as the default. Read live. |
| `rewrite.script_timeout_unit` | `"auto"` | The unit Settings shows the time limit in: `milliseconds`, `seconds` or `auto`. Display only: the limit is always `script_timeout_ms` milliseconds. Settings accepts 1 to 600000 (ten minutes); `0` still reads as the default. Read live. |
| `rewrite.script_total_timeout_s` | `3600` | How long one pass of an external script may take in all. `0` reads as the default. Read live. |
| `rewrite.script_total_timeout_unit` | `"auto"` | The unit Settings shows the time limit in: `seconds`, `minutes`, `hours` or `auto`. Display only: the limit is always `script_total_timeout_s` seconds. Settings accepts 1 to 86400 (a day); `0` still reads as the default. Read live. |

## `[updates]` (looking for a newer release)

What the title bar's version chip does about newer releases. Details: [Updates](updates.md).

| Key | Default | Values and effect |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` asks GitHub once a day, in the background, whether a newer release exists, and turns the version chip into `vX.Y.Z available`. `"auto"` also downloads and stages the update by itself when this build came from a release archive (it still asks before restarting); for any other build it behaves like `"notify"`. `"off"` never makes a request. Applies live. |
| `updates.channel` | `"stable"` | `"stable"` considers only full releases; `"prerelease"` also offers release candidates (`-rc.1`). Applies live. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | The `owner/name` on github.com whose releases are checked. While that repository is private GitHub answers 404 unless a token is stored for github.com (Settings > Integrations); without one the check fails quietly and writes one line to the application log. |

An administrator can set `mode`, `channel` (as `release_channel`) and `repo`, plus a few more update options, in a machine-wide policy file or the Windows registry. A key the policy sets wins over `config.toml` and shows in Settings as **Set by your administrator**. `mode` in the policy is a ceiling: you can pick a lower value, never a higher one. See [Enterprise distribution](enterprise-distribution.md#the-policy-file-and-the-registry).

## `[privacy]` (sensitive files)

| Key | Default | Values and effect |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | Gitignore-style path patterns. A pattern without a `/` matches the file name at any depth, one with a `/` the whole path (a leading `/` anchors it at the root); `*` and `?` stay inside one folder, `**` crosses folders; a `!` re-allows what an earlier pattern matched and the last match wins; case is ignored. Used by the staging guard and by `ai.privacy.exclude_sensitive_files`. |
| `privacy.staging_guard` | `true` | Staging a file that matches a pattern, or whose content holds a recognised secret, asks first (TUI), is refused unless `--allow-sensitive` is given (CLI) or unless `mcp.allow_sensitive_staging` lets the agent pass `allow_sensitive` (MCP). Text files up to 1 MiB each and 32 MiB in all are read; binary files are only matched by name. See [Staging and committing](staging-and-committing.md#sensitive-files). |

## `[keybindings]`

Rebinds the TUI's shortcuts; Settings > Keyboard & mouse edits it.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # default Ctrl+G
nav_prefix = "Alt+n"             # default Ctrl+N
settings   = ["Ctrl+,", "F2"]    # a list = alternatives
tab_menu   = []                  # unbound

[keybindings.git_prefix]         # the key pressed after the git prefix
stage_file = "z"
```

`[keybindings.<context>] <action> = "<chord>"`; a value replaces that action's
defaults, removing the line restores them. Chords: `Ctrl+Shift+N`, `Alt+n`,
`F2`, `PageDown`, `Space`, `,`, `P` (= `Shift+p`). A bad entry never blocks
anything: the TUI ignores it, keeps the default and says so on the status line.
Contexts, action ids, defaults and the conflict rules: [Keybindings](keybindings.md).

## Per-repository settings: `.gitpoulpi/repo-state.toml`

Kept inside the repository; the folder ignores itself (a `.gitignore` of `*`), so
it never shows as a change. Written by the TUI and the CLI; you rarely edit it.

| Key | What it is | Set with |
|---|---|---|
| `[preferences] encoding` | Overrides `git.default_encoding` here | Settings > Git & encoding |
| `[preferences] lfs_enabled` | Overrides `git.lfs_enabled` here | Settings > Git & encoding |
| `[preferences] profile` | The `[[profiles]]` id that signs commits here; absent means `general.active_profile` | the toolbar's profile combobox, **Repository: Change Profile...** |
| `[preferences] autocrlf`, `eol` | *Not used yet* | Settings > Git & encoding |
| `[preferences] push_after_commit` | Push the branch to its upstream after each commit (off when absent) | Settings > Repo-specific settings > Commit |
| `[preferences] skip_hooks_by_default` | The commit form's Skip hooks starts checked (off when absent) | Settings > Repo-specific settings > Commit |
| `[preferences] squash_merge` | A local branch merge squashes: stages, no merge commit (off when absent) | Settings > Repo-specific settings > Merge options |
| `[preferences] template_apply`, `template_strip_comments` | Pre-fill an empty commit form with the template; remove `#` comment lines at commit time (both off when absent) | Settings > Repo-specific settings > Message template |
| `[preferences] conflict_detection` | Proactive conflict detection for this repository; absent follows `general.conflict_detection` | Settings > Repo-specific settings > Conflict prevention |
| `[preferences] conflict_base_branches` | The Base branches patterns, one per line (`**` when absent) | Settings > Repo-specific settings > Conflict prevention |
| `commit-template.txt` (a file beside `repo-state.toml`) | The commit template; git's `commit.template` points at it | Settings > Repo-specific settings > Message template |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"`: this repository's AI instructions | Settings > Repo-specific settings > AI |
| `[preferences.ai_overrides.<action>]` | `profile`, `instructions` for one action here | Settings > Repo-specific settings > AI |
| `default_remote` | The remote one-click Push/Pull/Fetch use | remote menu, `gitpoulpi remote default <name>` |
| `hosting_integration` | The `[[integrations]]` profile pinned here | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | The provider project the PR / issue lists read, as `{ integration, project, filter }`. `filter` is optional: a provider-native extra clause (a Jira JQL fragment, an Azure WIQL condition, Trello list names) ANDed onto the list query; GitHub and GitLab ignore it. | sidebar sections, `gitpoulpi hosting link` |
| `ref_visibility` | Hidden / soloed refs | Refs sidebar, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | Remotes hidden from the graph as a whole (every branch, the ones fetched later too) | the remote's eye and menu, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | Collapsed sidebar sections and branch and tag folders. `[folds]` also holds `status_folders` (collapsed folders of the Unstaged / Staged file trees) and `commit_folders` (collapsed folders of a commit's changed-files tree, the same for every commit), each capped at 500 paths, oldest dropped first | the sidebar, the file trees |
| `pending_auto_stash` | A stash GitPoulpi made and has not put back | written by the daemon; see [Auto-stash](log-and-history.md#auto-stash) |

## Other environment variables

| Variable | Effect |
|---|---|
| `GITPOULPI_CONFIG_DIR` | The config folder (see above). |
| `GITPOULPI_TOKEN_<ID>` | A hosting token for profile `<ID>`, over the stored one (CI, one-off use). |
| `GITPOULPI_GIT` | The `git` executable to run (default: `git` on `PATH`; on Windows the real `mingw64\bin\git.exe` behind Git for Windows' `cmd\git.exe` shim, which saves a process per call). |
| `GITPOULPI_LANG=fr` | The interface language, over `ui.language` (any `ui.language` value; `qps` is a pseudo-locale for translators). An unknown or empty value is ignored. |
| `GITPOULPI_TERMINAL_IMAGE=off` | No pictures at all, whatever the config says. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Picture resolution hints (cell size in pixels; oversampling, default 2, `1` saves CPU). |
| `GITPOULPI_NO_BROWSER=1` | Never launch a web browser (the URL is still shown). |
| `GITPOULPI_TIMING=1` | Write request timings to `gitpoulpi-timing.log` in the temp folder (performance reports). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | Do not wait for Enter after the TUI exits on an error. |
