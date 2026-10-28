**English** · [Čeština](cs/installation.md) · [Deutsch](de/installation.md) · [Español](es/installation.md) · [Français](fr/installation.md) · [Italiano](it/installation.md) · [日本語](ja/installation.md) · [한국어](ko/installation.md) · [Nederlands](nl/installation.md) · [Polski](pl/installation.md) · [Português (Brasil)](pt-BR/installation.md) · [Português (Portugal)](pt-PT/installation.md) · [Русский](ru/installation.md) · [Svenska](sv/installation.md) · [Türkçe](tr/installation.md) · [Українська](uk/installation.md) · [简体中文](zh-CN/installation.md)

# Installation

GitPoulpi has three executables:

| Executable | What it is |
|---|---|
| `gitpoulpi-tui` | The terminal UI. |
| `gitpoulpi` | The command-line interface (`--json` for scripts). With no subcommand, it opens the UI. |
| `gitpoulpi-mcp` | The MCP server for AI agents. See [MCP and agents](mcp-and-agents.md). |

Next to them the archive holds `gitpoulpi-launch`, a tiny program the shortcuts run to show a GitPoulpi window that is already open (`gitpoulpi launch`). On Windows it has no console window, so a click never flashes one. It is installed, updated and removed with the others.

## What you need

- A **system `git`** on your `PATH`. GitPoulpi uses libgit2 for most
  operations and your `git` for the rest.
  Version **2.34 or newer** is required. On macOS the `git` from Apple's
  Command Line Tools works, and so does Homebrew's.
- A **terminal** with mouse support. kitty, Ghostty, WezTerm, iTerm2 and the
  sixel terminals (Windows Terminal, foot) draw the commit graph as pictures.
  Other terminals (the plain Windows console, tmux and most others) show a
  character graph. See
  [Choosing a terminal](terminals.md).
- A **Nerd Font** or a font with emoji, for the icons in the toolbar and the
  sidebar.
- Optional: **`git-lfs`** 3.0 or newer, for Git LFS repositories. Without it,
  LFS files show as small pointer files and GitPoulpi tells you why. See
  [Git LFS](lfs.md) for how to install it.
- Optional: a **credential helper** (Git Credential Manager, libsecret,
  `pass`, Keychain), to store hosting tokens and AI API keys. See
  [Hosting and issue trackers](hosting-integration.md).

## From a release archive

A release archive is `gitpoulpi-<version>-<target>.zip` (Windows) or
`.tar.gz` (Linux, macOS), with a `.sha256` file next to it. Check the
archive against the `.sha256` file, unpack it, and put the three executables
in a folder on your `PATH`. `INSTALL.txt` in the archive gives the same steps.

## With an installer

Each release also ships an installer per OS. Both make the same choices
(shortcuts, PATH, a command alias such as `gp`) by calling the installed
`gitpoulpi` itself, the code behind Settings > General > Launcher, so what the
installer sets up and what Settings shows always agree.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

It installs for the current user, with no administrator prompt, into
`%LOCALAPPDATA%\Programs\GitPoulpi` (the first dialog offers an all-users
install and the install folder). Checkboxes: Start menu shortcut, Desktop
shortcut, add to PATH. A page asks for the **command alias**: `gp` by default,
any name you like, empty for none. PowerShell already uses `gp` for
`Get-ItemProperty`, so choose another name there or use `gitpoulpi`.

The installer copies the three executables and the `.gitpoulpi-dist` marker, so
[Update now](#updating-in-place) works exactly as for the zip. A per-user
install is writable, so it can update itself; an all-users install in Program
Files needs elevation to be replaced, so reinstall it with the new setup instead.
Uninstall from Windows Settings > Apps: it runs `gitpoulpi uninstall` (alias,
PATH entries, shortcuts) and deletes the files, including the updater's `.old`
leftovers. Your settings and repositories stay.

### Linux and macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

It picks the archive for your OS and CPU, checks its `.sha256`, installs into
`~/.local/opt/gitpoulpi` and links `gitpoulpi` and `gitpoulpi-mcp` into
`~/.local/bin`. No root is needed and nothing outside your home is touched.
Without `--yes` it asks about the alias (default `gp`), the PATH and shortcuts.

| Flag | Effect |
|---|---|
| `--prefix DIR` | Install the program files in `DIR` (absolute). |
| `--version X.Y.Z` | Install this release instead of the latest. |
| `--archive FILE` | Use a local archive; `FILE.sha256` must sit next to it (`--no-verify` skips that). No network is used. |
| `--alias NAME` / `--no-alias` | Make `NAME` start GitPoulpi (`gitpoulpi alias create`), or none. |
| `--no-path` | Do not add GitPoulpi to the PATH of new shells (`gitpoulpi path add` otherwise). |
| `--desktop` | Also make the application menu and Desktop shortcuts. |
| `--yes` | Never ask: the flags and the defaults apply (alias `gp`, PATH yes, shortcuts no). |
| `--uninstall` | Run `gitpoulpi uninstall`, then remove the links and the folder. |

A latest-release lookup needs the repository to be public; for a private one
download the archive in a browser and use `--archive`.

## From source

You need a recent stable Rust toolchain (<https://rustup.rs>).

```
git clone https://github.com/Poulpi-Lab/GitPoulpi.git
cd GitPoulpi
cargo build --release
```

The executables are in `target/release/`. The `release` profile builds
quickly. For the smallest and fastest executables, use the `dist` profile. It
takes longer to build:

```
cargo build --profile dist
```

The `build-opti` scripts at the root of the repository run this build for the
three executables and print their sizes:

```
./build-opti.sh                  # Linux, macOS
build-opti                       # Windows (cmd or PowerShell)
```

To build only the languages and features you use, see
[Custom builds](custom-builds.md).

## Build a release archive yourself

```
cargo run -p xtask -- dist
```

This builds the `dist` profile and writes
`target/dist/gitpoulpi-<version>-<target>.zip` (Windows) or `.tar.gz` (other
systems). The archive holds the three executables, `INSTALL.txt`,
`README.md`, `LICENSE` and `CHANGELOG.md`. A `.sha256` file is written next
to it.

## Updates

The title bar shows the running version (`v1.0.0`) between the app name and
the repository path. Click it, or run **Help: Release Notes** from the command palette (`Ctrl+P`): a Release notes tab opens with the changelog, scrolled
to the running version.

Once a day, in the background, GitPoulpi asks GitHub whether a newer release
exists (`updates.mode`, [Settings > Synchronization](settings.md#synchronization)).
When one does, the chip reads `v0.2.0 available` in the accent color and its
click opens the Release notes tab with that release's notes on top. The tab has
three buttons, each with a key and a palette command:

- **Download** (`d`, *Updates: Download*) opens the release page in your browser.
- **Skip this version** (`s`, *Updates: Skip This Version*) hides the notice
  until a newer release appears.
- **Check now** (`r`, *Updates: Check for Updates Now*) asks again at once.

`updates.mode = "off"` makes no request at all. Without a network connection,
or while the repository is private and no github.com token is stored, the check
fails quietly (one line in the application log, nothing in the interface).
`GITPOULPI_UPDATE_API` points the check at another API root (used by the tests).

### Updating in place

A build installed from a release archive can replace itself. The archive carries
a small `.gitpoulpi-dist` file next to the executables; a running build is
"updatable" only when that file sits next to its executable (and names its
platform). A build made with `cargo`, a custom build copied elsewhere, or a
package that dropped the file never updates itself: its Release notes tab says
so and offers **Download** only.

For an updatable build the tab adds:

- **Update now** (`u`, *Updates: Update Now*) downloads the archive and its
  `.sha256` in the background (a job in the status line's indicator, with its
  percentage, cancellable like any job), checks the hash, unpacks the archive
  and puts each new file beside the old one as `<name>.new`. Nothing running is
  touched. In `auto` mode the same happens by itself as soon as a check finds a
  newer release.
- **Restart to update** (`R`, *Updates: Restart to Update*; the chip says the
  same once the download is staged) is always your action, never automatic. It
  stops the session's standalone daemon (an embedded one goes with the window),
  swaps the files and starts the new version with the same arguments and
  session.
- **Revert to the previous version** (`v`, *Updates: Revert to the Previous Version*) swaps the
  previous files back and restarts. The previous version's files stay as
  `<name>.old` until the new version has been running for a minute, then they
  are deleted, so the button is there for that first run. It is not part of the
  git undo journal: it changes GitPoulpi's own files, not a repository.

How the files are replaced: on Linux and macOS each new file is renamed over the
old one (atomic; a running program keeps its old copy until it exits). On
Windows a running `.exe` cannot be overwritten, so the old one is renamed to
`<name>.old` and the new one moved in; a `.old` that is still running is deleted
by a later start. The swap is all or nothing: if another program holds one of the
executables (an MCP client running `gitpoulpi-mcp`, say), GitPoulpi names the
file, changes nothing and leaves the download staged for another try.

What is refused, each with a message: a release without a `.sha256` file, a hash
that does not match, an archive file whose name is not the one for this platform
(`gitpoulpi-<version>-<target>.<zip|tar.gz>`), an archive whose marker names
another version or platform, and any download link outside these hosts: GitHub's
`github.com`, `api.github.com`, `objects.githubusercontent.com`,
`release-assets.githubusercontent.com` and `github-releases.githubusercontent.com`
(the first request and every redirect hop; a redirect anywhere else stops the
download). A private repository's archives need a browser session, so the
download fails there until the repository is public.

## Check that it works

```
gitpoulpi --version
cd path/to/a/git/repository
gitpoulpi-tui
```

`--version` prints the version and the commit the build was made from. Then
follow [Getting started](getting-started.md).

## Uninstall

`gitpoulpi uninstall` (what both installers run) removes the command aliases,
the PATH entries and the shortcuts GitPoulpi made, then the program files of a
release install; it refuses on a build without the `.gitpoulpi-dist` marker
(a cargo build), and `--keep-files` leaves the program files. It is not
available to MCP agents. Without an installer, delete the three executables.
GitPoulpi also keeps these files:

| What | Where |
|---|---|
| Settings (`config.toml`) and sessions | Your OS configuration folder. `GITPOULPI_CONFIG_DIR` changes it. |
| Per-repository state | `gitpoulpi/` inside the repository's `.git` folder by default (`session.per_repo_state_dir` can move it to `.gitpoulpi/` in the work tree or to the user data folder). |
| Caches (avatars) | Your OS cache folder. |
| Tokens and API keys | Your credential helper. **Settings > SSH & credentials > Forget all usernames and passwords** removes them. |

See [Configuration reference](config-reference.md) for the exact paths.
