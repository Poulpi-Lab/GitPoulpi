**English** · [Čeština](cs/updates.md) · [Deutsch](de/updates.md) · [Español](es/updates.md) · [Français](fr/updates.md) · [Italiano](it/updates.md) · [日本語](ja/updates.md) · [한국어](ko/updates.md) · [Nederlands](nl/updates.md) · [Polski](pl/updates.md) · [Português (Brasil)](pt-BR/updates.md) · [Português (Portugal)](pt-PT/updates.md) · [Русский](ru/updates.md) · [Svenska](sv/updates.md) · [Türkçe](tr/updates.md) · [Українська](uk/updates.md) · [简体中文](zh-CN/updates.md)

# Updates

GitPoulpi checks for newer releases and, depending on how it was installed, either replaces its own files or hands the update to the program that installed it. Nothing is ever replaced or restarted without your action.

## The chip and the panel

The title bar shows the running version (`v1.0.0`) between the app name and the repository path. When a newer release is known the chip turns into `v0.2.0 available` in the accent color.

Press `Ctrl+N` then `u`, run **Updates: Show Update** from the command palette (`Ctrl+P`), or click the chip: the update panel opens only when a newer release exists or an update is under way; otherwise a click opens the Release notes tab. It is a window over the app: `Esc`, `q`, the close cross or a click outside closes it. It shows:

- the running version and **Installed with**: how GitPoulpi got onto this computer (it decides everything below);
- where the update stands (checking, downloading with a percentage, ready, failed);
- the buttons of this installation's procedure, and the exact command when there is one;
- a line from your administrator when they set one.

Every button has a key (printed in the panel's hint line) and a palette command, so the panel works without a mouse:

| Button | Key | Palette command |
|---|---|---|
| **Update and restart** (or **Run update**) | `u` | Updates: Update Now |
| **Copy command** | `c` | Updates: Copy Update Command |
| **Open in terminal** | `t` | Updates: Run Update Command in a Terminal |
| **Revert** | `v` | Updates: Revert to the Previous Version |
| **Release notes** | `n` | Help: Release Notes |
| **Skip this version** | `s` | Updates: Skip This Version |
| **Open notice** (only with a security notice) | `a` | Updates: Open Security Notice |
| **Check now** | `r` | Updates: Check for Updates Now |
| **Close** | `Esc` | |

A command that does not apply to this installation is grayed out in the palette with the reason (for example "this installation is updated another way: see Updates: Show Update").

## What happens, per installation

| Installed with | Check against | What the panel does | Going back |
|---|---|---|---|
| A release archive, or `install.sh` | GitHub releases | **Update and restart**: GitPoulpi downloads and verifies the archive, closes, replaces its files and opens again in the same terminal. | **Revert** swaps the previous files back. |
| The Windows installer, only for me | GitHub releases | The same, with no prompt. On Windows it reopens in a new window. | **Revert** swaps back. |
| The Windows installer, all users | GitHub releases | The same, but Windows asks for administrator approval. Other users' GitPoulpi sessions on the computer are closed, which loses their unsaved input. If you decline, nothing changes. | **Revert** swaps back, with the same prompt. |
| Chocolatey | The Chocolatey community feed | **Run update** runs `choco upgrade gitpoulpi -y` (Windows asks for administrator approval). | The panel shows `choco install gitpoulpi --version <previous> --allow-downgrade -y`. |
| Chocolatey, internal feed | Your organization's feed | **Run update** runs `choco upgrade gitpoulpi -y --source=<feed>`. Package parameters are not sent again, so a deployed configuration stays untouched. | The same `choco install` line with the feed. |
| Scoop | GitHub releases | **Run update** runs `scoop update gitpoulpi` after GitPoulpi has closed completely. | **Revert** runs `scoop reset gitpoulpi@<previous>`. |
| Homebrew | GitHub releases | **Run update** runs `brew upgrade gitpoulpi` in this terminal. | The panel shows `brew install gitpoulpi@<previous>`. |
| apt | GitHub releases | The panel shows `sudo apt update && sudo apt install --only-upgrade gitpoulpi` with **Copy command** and **Open in terminal**. GitPoulpi never runs `sudo` for you. Without the GitPoulpi apt repository on this computer it says to download the new `.deb` instead. | The panel shows `sudo apt install gitpoulpi=<previous>`. |
| An MSI package | GitHub releases, or your administrator's source | Nothing to run: the installation is managed by your administrator, whose message the panel shows. | Not offered. |
| cargo | GitHub releases | The panel shows `cargo binstall gitpoulpi-cli` when `cargo-binstall` is installed, else `cargo install --locked --git https://github.com/Poulpi-Lab/GitPoulpi --tag releases/v<version> gitpoulpi-cli`. | The same command with the previous tag. |
| A Docker image | GitHub releases | Nothing to run: an image is replaced. The panel shows `docker pull poulpilab/gitpoulpi:<version>` and `:latest`. See [Running in Docker](docker.md). | The panel shows `docker pull poulpilab/gitpoulpi:<previous>`. |
| A release archive in a folder you cannot write | GitHub releases | The panel says GitPoulpi cannot replace itself there and points to **Download**. | Not offered. |
| A development build (`cargo build`) | none | Never updates. The chip shows the version and nothing else; the panel says there is no install record. | Not offered. |

"Show" rows never run anything by themselves. **Open in terminal** opens your terminal with the command typed in, and you press Enter.

Releases always come from the project's GitHub repository unless an administrator points the check elsewhere (an internal feed, a mirror or a `latest.json`), see [Enterprise distribution](enterprise-distribution.md). Release notes come from the release or from the changelog built into GitPoulpi, whichever is reachable.

## Notify and auto

`updates.mode` ([Settings > Synchronization](settings.md#synchronization)):

- **Notify** (default): once a day GitPoulpi checks in the background and shows the chip. Nothing is downloaded until you press the button.
- **Auto**: where GitPoulpi replaces its own files (release archive, the Windows installer) it also downloads and verifies the update in the background as soon as it finds one. The chip then reads **Restart to update**, and the restart is still yours to choose. Where a package manager owns the update, **Auto** behaves like **Notify**.
- **Off**: no request is ever made and the chip shows the version only.

The **Update channel** setting (`updates.channel`) chooses between **Stable** and **Prerelease** (release candidates too). A check you start with **Check now** ignores the once-a-day limit. Without a network connection, or while the repository is private and no github.com token is stored, a background check fails quietly with one line in the application log.

## Restarting and what you see afterwards

Pressing **Update and restart** or **Run update** saves your session and closes the window. A separate small program, `gitpoulpi-updater`, does the work, so GitPoulpi's own files are not in use while they are replaced and a package manager is not blocked by a waiting parent. It stops GitPoulpi's background daemons first (the other windows and MCP servers of your user close too), does the update, checks that the new `gitpoulpi --version` prints the expected version, and starts GitPoulpi again: in the same terminal on Linux and macOS, in a new window on Windows. If the check fails it puts the old files back.

The next start shows one line in the status bar:

- `Updated to v1.4.2`
- `Update failed: <reason>. The previous version is still installed.`
- `Update canceled: administrator approval was declined. Nothing changed.`

Only one update runs at a time; a lock file in GitPoulpi's data folder (and one in the install folder when it is writable) keeps a second one out, and a lock older than ten minutes is ignored, so a crashed update never locks GitPoulpi for good. The CLI and the daemon do not start while a fresh lock exists.

## Revert

Where GitPoulpi replaces its own files, **Revert** swaps the previous files back and restarts. They stay as `<name>.old` for a minute after the new version has started, then are deleted, so revert is for the first run after an update. Where a package manager owns the install, revert is the command in the table: the panel shows or runs it. Reverting changes GitPoulpi's own files, not a repository, so it is not part of the git undo journal.

## What is checked and refused

Before anything is replaced GitPoulpi downloads the archive, its `.minisig` signature and its `.sha256`. It checks the signature first, against the project's release key built into GitPoulpi, and the trusted comment of the signature must name this archive and this version. Then it checks the hash. It refuses: a release without a `.minisig` (an older release published before releases were signed cannot be installed through the updater, which includes pinning or going back to one), a signature by another key, a signature for another file or another version, a release without a `.sha256`, a hash that does not match, an archive whose name is not the one for this platform (`gitpoulpi-<version>-<target>.<zip|tar.gz>`), an archive whose marker names another version or platform, and a download link outside GitHub's release hosts (every redirect hop is checked) or the mirror an administrator configured. The swap is all or nothing: if another program holds one of the executables, GitPoulpi names the file, changes nothing and leaves the download for another try. On Windows, an all-users update is re-verified by the elevated step, which downloads the `.minisig` and the `.sha256` itself and checks both again, so a file swapped in the user's download folder in between is refused. Package managers (Chocolatey, Scoop, Homebrew, apt) verify their own packages: GitPoulpi downloads nothing there.

## Security notices

Next to each release the project publishes `security.json`: the list of security notices (an identifier, a severity, the affected versions as a range such as `>=1.2.0, <1.4.3`, the fixed version, a link and a one-sentence summary). The same daily check that looks for a new version also fetches it, from the same place as the release: the release page of the repository for GitHub, the folder of `latest.json` for a mirror, GitHub for the public Chocolatey feed. A notice is used only when `security.json.minisig` verifies against the project key; an unsigned or badly signed file is never shown (one line in the Activity Log says it was ignored).

When a notice names the running version, the chip turns red and reads **Security notice: this version is affected**, on every installation, package-managed ones included. The update panel then lists each notice with its severity, summary, fixed version and link; **Open notice** (`a`, or **Updates: Open Security Notice** in the palette) opens the most severe one's link in the browser. Where a package manager owns the install the panel adds "Update with your package manager."

With update checks off (`updates.mode = "off"`), with an administrator policy whose `check` is `none`, and for a build made with `cargo build`, nothing is fetched and no notice is shown. An organization's own NuGet feed does not make GitPoulpi contact GitHub: its notices come only from the policy's `download_base` mirror.

## Settings an administrator locked

An administrator can set the update mode, the channel, a pinned version, where to check and whether GitPoulpi may update itself at all, in a machine-wide policy. In Settings those rows show **Set by your administrator**, the panel shows their message, and when self-update is turned off the panel offers the commands or a note instead of buttons that replace files. Unreadable policy files fail closed: notify only, no self-update. The administrator's side is in [Enterprise distribution](enterprise-distribution.md#the-policy-file-and-the-registry).

## Environment

`GITPOULPI_UPDATE_API` points the check at another API root (used by the tests).
