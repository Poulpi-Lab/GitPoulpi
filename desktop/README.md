# GitPoulpi desktop

A native desktop frontend for GitPoulpi. It shows the same repository views as the terminal UI (refs
sidebar, commit log with graph, commit and working-tree details, diff) and talks to the same daemon, so
changes made from the terminal UI, the CLI or MCP show up here too.

What the desktop covers compared with the terminal UI is tracked in [parity/](parity/README.md); the
generated table is [docs/parity.md](docs/parity.md).

## Commands, keys and menus

Commands come from the shared registry in `gitpoulpi-app`, the same one the terminal UI uses: key
bindings (`[keybindings]` of `config.toml` overrides them), the command palette (Ctrl+P) with its
shortcuts, fuzzy matching and availability reasons, and the menu bar (File, Edit, View, Repository,
Branch, Git, Hosting, Tools, Help; F10 or Alt + the underlined letter). Everything runs through one
dispatcher (`src/commands/`): a command the desktop does not implement yet says so in the status line,
and its menu item and palette row are dimmed with the reason. The keybindings view lists every bound key
by context (`?`, or the Keybindings item of the Tools menu).

This frontend is under development on the `desktop` branch. It opens the repository through the same
per-session daemon the terminal UI and the CLI use (the shared `gitpoulpi-link` client), so a commit made
elsewhere shows up in the Log without any action. Every command runs through one dispatcher
(`src/commands/`) that resolves the shared registry's ids: a toolbar button, a menu item, a palette row and a
key (`Ctrl+G f`, `Ctrl+Z`, `o` in the Refs, `n` in a diff, ...) all end in the same place and give the same
"not available yet" answer for what the desktop does not do yet. The toolbar's Fetch, Pull, Push, Undo and
Redo run git operations; the other toolbar buttons answer "not available yet". `--session NAME` joins another session.

## Staging and committing

Select the WIP row of the Log and the right pane lists the working changes: **Unstaged Files** (edited,
new and conflicted files) and **Staged Files**, each with its count and its bulk buttons. Click selects
(Ctrl+click toggles, Shift+click takes a range), a double click or Enter opens the file's diff, the hover
button stages or unstages a row, and a row can be dragged onto the other list. The keyboard has all of
it: Up / Down (Shift extends), Space, Ctrl+A, Esc, Ctrl+Space, `d`, `m` (Path or Tree), `z` and the
`Ctrl+G s / u / a / x` chords. Stage and unstage never ask; discard always asks first (how many files, how
many untracked) and Undo takes it back. Each action is one undo entry, a multi-selection included.

In a working-tree diff the cursor row (Up / Down, `j` / `k`, or a click) picks the hunk: `Space` stages or
unstages it, `x` discards it, `v` starts a line selection that `Enter` stages and `Shift+X` discards.
The hunk header has the same buttons. The diff follows the index after each action.

The commit form under the lists has the summary (with the 72-character counter), the description,
Amend and Skip hooks, the identity the commit is made as, and the commit button, which says why it cannot
commit yet. `Ctrl+Enter` in a field, `Ctrl+G Enter` and the button commit; the form clears only after the
commit succeeded, and a refusal (a hook, nothing staged) keeps the text and shows the daemon's message.
A stage that would add files that look like secrets asks first (Stage anyway or Cancel).

## Branches, tags and stashes

The toolbar's Branch, Tag, Stash and Pop buttons, the `Ctrl+G` chords (`b`, `t`, `Shift+T`, `m`, `o`, `d`, `r`, `Shift+U`)
and the palette create branches and tags (small forms that check the name as the terminal UI does), check out, merge,
rebase onto, rename, fast-forward, set or remove an upstream, delete (always after a question, never with a default
destructive answer), push or annotate a tag, and stash, pop, apply or drop. The Refs rows and the Log's ref pills have the
terminal UI's context menu (right click, or the menu key / `Shift+F10` on the selected row); a double click checks out; a
branch dropped on another offers merge, rebase and fast-forward. Rows of that menu the desktop cannot run yet are dimmed
with the reason. A checkout, merge, rebase or pull blocked by local changes asks to stash them and continue. A rebase or
merge that stops on conflicts is reported in the status line: the resolution view is not available yet.

## Layout

`desktop/` is its own cargo workspace with its own `Cargo.lock`; the root workspace excludes it. The
crate is `crates/gitpoulpi-desktop` (binary `gitpoulpi-desktop`) and depends on the shared crates by path, among them `gitpoulpi-app` (commit-graph lanes, stash rows, row model).

## Build and run

From this folder:

```sh
cargo build
cargo run -- [REPO] [--theme SLUG] [--lane-theme SLUG] [--lang CODE]
```

`REPO` is a repository or a folder inside one; it defaults to the current directory.

Interface settings (theme, language, layout, column widths, window size) are stored in `desktop.toml`
inside the GitPoulpi config directory. `GITPOULPI_CONFIG_DIR` points the program at another directory.

## Checks

```sh
scripts/check-desktop.sh   # from the repository root: fmt, clippy -D warnings, tests
```

The first build compiles the UI toolkit and takes several minutes.

## Fonts

The UI and monospace fonts are compiled into the binary. Their licences are in
[THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
