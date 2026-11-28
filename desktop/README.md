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

This frontend is under development on the `desktop` branch. It currently reads a repository and does not
yet run git operations.

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
