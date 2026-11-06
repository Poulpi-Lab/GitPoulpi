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
in a folder on your `PATH` (keep `gitpoulpi-updater` and the `.gitpoulpi-dist`
marker next to them: the marker says `channel=archive`, so the build can update
itself, see [Updates](updates.md)). `INSTALL.txt` in the archive gives the same steps.

### Verify a download

Every release file is signed with the project's [minisign](https://jedisct1.github.io/minisign/) key, and GitPoulpi checks that signature itself before it installs an update. For a file you downloaded by hand, the `.minisig` next to it is the signature. The public key (key id `B461FFCBF6F6A4F2`) is committed as `packaging/minisign.pub` in the repository:

```
untrusted comment: minisign public key B461FFCBF6F6A4F2
RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

```
minisign -Vm gitpoulpi-<version>-<target>.tar.gz -P RWTypPb2y/9htNMSQTZUXq/V/RpK04Y9Ca5ub3SgALrpMbywtSM1UAoL
```

Read the trusted comment the command prints: it must be `file:<the file's name> version:<the version you expect>`. A signature that is valid but names another file or another version is not a signature for yours.

Where the release was built with build provenance attestations (the repository's `ATTESTATIONS` variable), the GitHub CLI checks where a file was built:

```
gh attestation verify gitpoulpi-<version>-<target>.tar.gz --repo <owner>/gitpoulpi
```

The `.sha256` file only catches a damaged download. The signature is what shows the file is the project's.

## With an installer

Each release also ships an installer per OS. Both make the same choices
(shortcuts, PATH, a command alias such as `gp`) by calling the installed
`gitpoulpi` itself, the code behind Settings > General > Launcher, so what the
installer sets up and what Settings shows always agree.

### Windows: `gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe`

The first dialog asks who the install is for:

| Choice | Where | Administrator rights | What it means for updates |
|---|---|---|---|
| **Only for me** (default) | `%LOCALAPPDATA%\Programs\GitPoulpi` | none | GitPoulpi replaces its own files: **Update and restart** in the [update panel](updates.md), no prompt. |
| **For all users** | `Program Files\GitPoulpi` | one prompt at install | Each update asks for administrator approval (a Windows prompt), and the GitPoulpi processes of other users on the computer are closed first. |

The same choice works silently: `gitpoulpi-<version>-...-setup.exe /CURRENTUSER` or `/ALLUSERS`.

Checkboxes: Start menu shortcut, Desktop shortcut, add to PATH. A page asks for the **command alias**: `gp` by default, any name you like, empty for none (`/ALIAS=name` or `/ALIAS=` on the command line). PowerShell already uses `gp` for `Get-ItemProperty`, so choose another name there or use `gitpoulpi`.

The installer copies the executables (including `gitpoulpi-updater`, the small program that swaps files during an update) and the `.gitpoulpi-dist` marker. The marker records how GitPoulpi was installed (`inno-user` or `inno-machine`), which is what the update panel shows as **Installed with**. Running the setup again keeps your choices (scope, shortcuts, alias).

Uninstall from Windows Settings > Apps: it runs `gitpoulpi uninstall` (alias, PATH entries, shortcuts) and deletes the files, including the updater's `.old` leftovers. Your settings and repositories stay.

### Linux and macOS: `install.sh`

```
curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
```

It picks the archive for your OS and CPU, checks its `.sha256`, installs (channel `archive`: GitPoulpi updates itself in place) into
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

## With a package manager

Every channel below writes the same `.gitpoulpi-dist` marker next to the programs, with its own channel name. GitPoulpi reads it to know who owns updates: the package manager does, so GitPoulpi never swaps files behind its back. The update panel (click the version in the title bar) then runs the package manager's own command, or shows it so you can copy it. See [Updates](updates.md).

| Channel | Install | Notes |
|---|---|---|
| **Chocolatey** (Windows) | `choco install gitpoulpi` | Installs for all users, so updates ask for administrator approval. |
| **winget** (Windows) | `winget install --id GitPoulpi.GitPoulpi` | Add `--scope user` or `--scope machine` to choose; machine scope asks for administrator approval at install and at each update. |
| **Scoop** (Windows) | `scoop bucket add gitpoulpi https://github.com/Poulpi-Lab/scoop-gitpoulpi` then `scoop install gitpoulpi` | Per user, no administrator rights. |
| **Homebrew** (macOS, Linux) | `brew install Poulpi-Lab/gitpoulpi/gitpoulpi` | The tap is `Poulpi-Lab/homebrew-gitpoulpi`. |
| **apt** (Debian, Ubuntu) | see [the apt repository](#apt-debian-and-ubuntu) | Signed repository; a `.deb` is also attached to every release. |
| **MSI** (Windows, managed) | `msiexec /i gitpoulpi-<version>-x86_64-pc-windows-msvc.msi /qn` | For administrators: all users, no in-app updates. See [Enterprise distribution](enterprise-distribution.md#msi-with-intune-or-group-policy). |
| **AUR** (Arch Linux) | `yay -S gitpoulpi-bin` (or `paru -S gitpoulpi-bin`) | Without a helper: `git clone https://aur.archlinux.org/gitpoulpi-bin.git`, then `cd gitpoulpi-bin && makepkg -si`. |
| **cargo** | `cargo binstall gitpoulpi-cli` | Downloads the release archive instead of compiling. |
| **cargo, from source** | `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli` | Compiles; needs a Rust toolchain. |
| **Docker** | `docker pull poulpilab/gitpoulpi` | See [Running in Docker](docker.md). The same image is on `ghcr.io/poulpi-lab/gitpoulpi`. |

The names above are the ones the release process publishes to (see [Releasing](../developer/releasing.md)): a channel that is not live yet answers "not found", and the release archive or installer always works.

A package manager's own PATH handling applies: Chocolatey, winget, Scoop and Homebrew put the programs on the PATH themselves, and none of them makes the `gp` alias (run `gitpoulpi alias create gp`, or Settings > General > Launcher).

### apt (Debian and Ubuntu)

```
sudo install -d /etc/apt/keyrings
curl -fsSL https://poulpi-lab.github.io/apt/gitpoulpi-archive-keyring.gpg | sudo tee /etc/apt/keyrings/gitpoulpi.gpg >/dev/null
sudo curl -fsSL -o /etc/apt/sources.list.d/gitpoulpi.sources https://poulpi-lab.github.io/apt/gitpoulpi.sources
sudo apt update && sudo apt install gitpoulpi
```

The repository supports `amd64` and `arm64` and is signed with a key that belongs to the project. The programs go to `/usr/lib/gitpoulpi` with links in `/usr/bin`. The package depends on `git` and recommends `git-lfs`. Without the repository, install the `.deb` of a release with `sudo apt install ./gitpoulpi_<version>_<arch>.deb`: GitPoulpi then tells you to download the next `.deb` yourself, because it only offers `apt` commands when it finds the repository in `/etc/apt/sources.list.d/gitpoulpi.*`.

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

The title bar shows the running version (`v1.0.0`) between the app name and the repository path. Click it, or press `Ctrl+N` then `u`, or run **Updates: Show Update** from the command palette (`Ctrl+P`): the update panel opens with the version, how GitPoulpi was installed and what to do about a newer release. **Help: Release Notes** opens the changelog.

Once a day, in the background, GitPoulpi asks where its releases are published (GitHub by default) whether a newer one exists (`updates.mode`, [Settings > Synchronization](settings.md#synchronization)). When one does, the chip reads `v0.2.0 available` in the accent color.

What the panel does depends on how GitPoulpi was installed: a release archive or the Windows installer replaces its own files, a package manager runs its own command, a managed or container install says how updates reach it. A build made with `cargo build` never updates. The full description, with revert, `auto` against `notify` and what an administrator can lock, is in [Updates](updates.md).

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
