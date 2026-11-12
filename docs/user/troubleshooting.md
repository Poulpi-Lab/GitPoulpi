**English** · [Čeština](cs/troubleshooting.md) · [Deutsch](de/troubleshooting.md) · [Español](es/troubleshooting.md) · [Français](fr/troubleshooting.md) · [Italiano](it/troubleshooting.md) · [日本語](ja/troubleshooting.md) · [한국어](ko/troubleshooting.md) · [Nederlands](nl/troubleshooting.md) · [Polski](pl/troubleshooting.md) · [Português (Brasil)](pt-BR/troubleshooting.md) · [Português (Portugal)](pt-PT/troubleshooting.md) · [Русский](ru/troubleshooting.md) · [Svenska](sv/troubleshooting.md) · [Türkçe](tr/troubleshooting.md) · [Українська](uk/troubleshooting.md) · [简体中文](zh-CN/troubleshooting.md)

# Troubleshooting and FAQ

Start with the quick answers below; the longer sections cover Git LFS and the
graph rendering. If none of it helps, file an issue on the GitHub repository with
your OS, your terminal, `gitpoulpi --version`, and what you did.

## When something fails

A failed operation shows no pop-up and no framed card: the **status line** says it, in red, with the title and the reason (*error: Fetch failed: The remote could not be reached.*), and it stays there until another message replaces it. The Activity button at its right end counts the errors you have not looked at yet. Click the red message, or run the palette's `Activity Log: Open the Last Error`, to open the Activity Log on that entry. Every failure you triggered works this way, not only git operations:

- **git operations**: a fetch, a push, a pull, a checkout, a merge, a rebase, a commit, a stash...
- **A clone** (and creating a new repository): "Clone failed".
- **A call to GitHub or GitLab**: the repository lists of the pickers and the Clone dialog, creating a pull request or an issue, checking a token or an SSH key on the Integrations page: "Hosting request failed".
- **An external program**: the editor, the diff or merge tool, the file explorer, the browser, the terminal or the launcher, an image editor, the *Test* buttons of the Settings: "External tool failed".
- **An AI request** (a commit message, an explanation, a conflict suggestion...): "AI request failed".
- **A file that could not be written**: the built-in editor's Save, the merge editor, a setting that could not be saved: "Save failed".

The red message is on the status line whichever view is in front: a repository tab, Home, the Radar, the Workspaces, Settings (on the screen's bottom row), under the Activity Log or a dialog. Where the status line is covered (Home, the Radar, the Workspaces, Settings) the message is written over its bottom row until you have looked at the error. It follows you from one tab to another. An operation that is still running when you switch tabs is not followed: open the Activity Log to see how it ended.

The lists of the PULL REQUESTS and ISSUES sections that cannot be refreshed keep their own line in the section instead (they refresh by themselves, a message every few seconds would only be noise). A dialog that is still open (Clone, Create pull request...) also keeps its own one-line message, so you can fix the field and try again.

The entry in the Activity Log holds everything the pop-up used to show:

- **Title and reason.** "Fetch failed", then one sentence saying why in plain words, then what to do next. Recognized: the remote has commits you do not have (*pull first*), credentials refused (*check Settings > Integrations or your SSH key*), a stop on conflicts, local changes that would be overwritten (*commit, stash or discard*), the remote not reachable, and another git process holding a lock on the repository. Anything else shows git's own first line. A clone or a hosting call that fails the same way (credentials refused, host not reachable) is worded the same; other failures get one sentence about what failed ("The repository could not be cloned.", "The program could not be run.").
- **The details**, shown when the entry is open (`Enter`, or the click that opened it): the operation, the git command that ran, its exit code and the last lines of what it printed (for the built-in git library, its error class and code). Credentials, tokens and `Authorization` headers are masked before anything is shown, logged or copied.
- **Copy** (the button, or `c`) puts a block of text on the clipboard made to be pasted into a chat with someone, or an AI, who can explain it: GitPoulpi's version, the system, the terminal, the backend, the failing operation with its details and the git version.
- **A button for the next step**, when there is one, next to Copy (or `r`): *Pull first* (a push the remote would reject), *Stash and retry* (local changes in the way of a checkout, merge, rebase or pull), *Retry* (the remote could not be reached), *Open Integrations* (credentials). It runs the step and closes the window. For a failure of another repository than the tab in front only *Open Integrations* is offered: a retry would run in the wrong repository.
- Without the mouse, the command palette has `Activity Log: Open the Last Error`, `Activity Log: Copy the Last Error` and `Activity Log: Run the Suggested Action` (the entry's remedy button, for the error the status line shows).

A push the remote refused because it moved on keeps its own dialog (pull first, force push with lease, cancel); the facts are still in the Activity Log.

### The Activity Log

The **Activity** button at the right end of the status line (a number on it counts the errors you have not looked at yet), `Ctrl+N` then `Shift+A`, and the palette's `View: Activity Log` open the window. `Ctrl+N` shows a badge on each tab, box, field, checkbox and button of the window (and on the first entry of the list); pressing the character does what a click on it does. Two tabs, **Git operations** and **Application**, switched with `Tab`, a click, `g` (Git operations) or `p` (Application), and a **Scope** box under them that applies to both (see below).

- **Application**: the program's own log lines, the window's and the background daemon's, newest first. The **Level** box (a dropdown: Error, Warning, Info, Debug, Trace; `l` opens it) says how much to show; asking for more detail than is being recorded raises the recording level too (and saves it as `log.level`). Under it, **Show** has one checkbox per level (Error, Warning, Info, Debug, Trace, each with how many lines it holds) to hide or show the lines of that level, for instance to see only the errors, and **All** / **None** check or clear every box. Keys `1` to `5` toggle the levels in that order, `a` is All, `n` is None; the palette has `Activity Log: Show or Hide Level ...`, `Activity Log: Show All Levels` and `Activity Log: Show No Level`. Levels above the Level box's choice are grayed (nothing is recorded for them; hover one to see why). The checks are remembered until you quit, and only the Application tab has them. **Filter** (`/`) keeps the lines that contain your text. **Copy all (filtered)** copies what is shown (the checked levels and the filter both apply); **Open log folder** opens the folder in your file manager. **Clear log** (`x`, or the palette's `Activity Log: Clear the Application Log`) asks first, then empties the log files of every kept day and the list on screen; it cannot be undone and it is not in the undo history. A long line is never cut: see "Long lines" below.
![The Activity Log, Application tab: the Level box, the per-level Show checkboxes with All and None, the filter and the log lines](images/activity-log.png)

- **Git operations** (the tab the window opens on): the operations, newest first, each with the time it ran and how long it took (`Push - 123 ms`): opening the repository, loading the commit graph, fetch, pull, push, commit, checkout, merge, rebase, stash, clone. Failures, the status messages, a clone that failed and a workspace's Fetch all are listed here too; `Enter` or a click opens one (its reason, next step, command, exit code and output), `c` or its Copy button copies it, `r` or its button runs the suggested step. `Tab` (or `p`) switches to the Application tab; `Left`/`Right` scroll long lines sideways.
![The Activity Log, Git operations tab: each operation with its time and how long it took, a failed fetch in red](images/activity-log-repository.png)

- **Which action produced an error.** Every failure says what produced it, in the entry, in the Application log and in the copied report. The operation is named the way a person would (`Remove remote`, `Fetch all remotes`, `Hosting request: create pull request`, `Token command`) with its main arguments (`remote=cm6`, a branch, a path; never a message body, and URLs and tokens are masked like everywhere else), then the repository, who asked (`TUI`, `CLI` or `MCP`) and the operation id, the number the daemon gave the request (and its job number for a fetch, push or pull). In the Git operations tab the entry's header reads `Remove remote failed (remote=cm6) - ... [CLI #12]` and its details start with `Operation:`, `Repository:`, `Requested by:` and `Operation id:`; an operation with no title of its own is titled by what it was (`Remove remote failed`). On the Application tab the failure line starts with the same words and a tag: `Remove remote failed (remote=cm6) [op #12, CLI, repo name]: backend error: no such remote: cm6`. A line like that opens its operation: click it, or select it and press `Enter` (the palette's `Activity Log: Show the Operation of This Line`). `]` and `[` select the next and the previous failure line (`Activity Log: Next Failed Operation` / `Activity Log: Previous Failed Operation`); with a line selected the arrow keys move the selection instead of scrolling. The entry that opens is the window's own when the window ran the operation; for a failure of the CLI or an agent against the same session, the window makes an entry from the line. The copied error report has `operation:`, `arguments:`, `repository:`, `requested by:` and `operation id:` lines, so it can be pasted on its own. From a script, a failed command's `--json` error is unchanged; the daemon's answer to a failed request carries the same facts in an added `context` field.

- **Long lines** (both tabs): a header or a detail line wider than the window is never cut. A horizontal scrollbar appears under the list; scroll sideways with `Left` / `Right` (rebindable as `activity_log.scroll_left` / `activity_log.scroll_right`), `Shift` + the wheel or a horizontal wheel, the palette's `Activity Log: Scroll Left` / `Activity Log: Scroll Right`, or the bar (drag the thumb, or press the track to bring the thumb under the pointer). `Home` goes back to the start, and switching tab starts the other tab at the left edge. Wrapped text (a summary, a suggested action) stays wrapped.

- **Scope** (a dropdown under the tabs, `s` opens it, or the palette's `Activity Log: Choose the Scope`): what both tabs are about. The rows are **This repository (name)** (the repository of the tab you opened the log from; not offered from Home, which has none), **All repositories**, each **workspace** and each repository the window knows about. From a repository tab the log opens on *This repository*, from Home (the Activity button, `Ctrl+N` then `Shift+A`) on *All repositories*; `Ctrl+N` then `Shift+G` and the palette's `View: Global Activity Log` always open on *All repositories*, and so does a click on the red error of a failure whose repository is not the tab in front (a failed clone, for instance). Your last choice is remembered for the session, separately for the two ways of opening it. On the Application tab a line belongs to a repository when the program wrote it about one (a failed push, an operation's timing); under *All repositories* each such line says whose it is (`[name]`). A line about no repository at all (start-up, settings, the daemon itself) and an entry that belongs to none (a failed save) are **app-wide**: they show under every scope, marked `[app-wide]` when the scope is narrower than *All repositories*, so a filter never hides a failure. The palette also has `Activity Log: Git Operations` and `Activity Log: Application Log` for the tabs.

- **Copy diagnostics** (a button on both tabs, `d`, always the whole log, whatever the scope; and the palette's `View: Copy Diagnostics`) copies the version, system, terminal and backend, the latest failure with its details and the last 200 log lines at the chosen level: the block to give an AI when something is wrong.

The durations are measured by the window (the request to the daemon's answer, round trip included), so what the CLI or MCP does is not timed. `log.timings` (default on) turns the durations off, `log.extended` (default off) also lists the background work, the periodic refresh and auto-fetch; both are in Settings > General next to the other log settings, and every operation is written to the application log anyway (`info` for what you asked for, `debug` for the background).

Log files are `gitpoulpi-YYYY-MM-DD.log`, kept for a week by default and at most ten years (`log.retention_days`, 1 to 3650), pruned when GitPoulpi starts, and rotated at `log.max_file_mb` (5 MiB). Where they are: [configuration reference](config-reference.md#log-the-application-log). From a script, a failed command's `--json` error carries the same facts under `detail`; see the [CLI reference](cli-reference.md).

A click on the red message opens the log on *All repositories* with that entry open. The Git operations list is the session's; the Application tab keeps the program's log across restarts.

**Background jobs.** A fetch, push, pull, clone or LFS transfer, a workspace's Fetch all / Pull all / Clone missing, an AI request and a pull request or issue being sent are jobs: the right end of the status line shows the running one with an `[x]` that cancels it, and a click on its label (or `Ctrl+N` then `Shift+J`, the palette's `Jobs: Show Background Jobs`) opens the panel with a Cancel button per job (`c` or `Del` as well). A clone runs in the background; an AI request and a workspace walk survive a tab switch and are applied when you come back to their repository. A job past the point where stopping is safe is grayed with the reason; a canceled job leaves the repository as it was.

## Quick answers

**The toolbar and sidebar show boxes or question marks instead of icons.**
Use a Nerd Font or any font with emoji coverage in your terminal. The graph and
text still work without them.

**The commit graph is made of characters, not pictures.** That is the automatic
fallback: pictures need a terminal speaking the kitty graphics protocol (kitty,
Ghostty, herdr with kitty graphics on), the iTerm2 protocol (WezTerm, iTerm2,
mintty) or sixel (Windows Terminal, foot, Konsole). The plain Windows console, tmux and zellij get characters. See the
graph section below to force either, and [Choosing a terminal](terminals.md).

**The mouse does nothing, or selecting text with the mouse does not work.**
GitPoulpi captures the mouse so that clicks reach it. To select text with the
terminal itself, hold `Shift` while you drag (in most terminals). You can also
select inside the diff (drag, then `y`). To turn mouse capture off, use
*Settings > Keyboard & mouse > Mouse enabled* (restart needed). The keyboard
reaches every command.

**A shortcut does not work.** Your terminal, tmux or an AutoHotkey script may
take the key before GitPoulpi sees it (`Ctrl+,` often arrives as a plain `,` on
Linux and macOS terminals; `Ctrl+Shift+N` looks like `Ctrl+N`). Use the
alternatives (`Ctrl+N` then `,` opens the Settings; `Ctrl+P` reaches every
command), or rebind the chord: see [Settings](settings.md#keyboard--mouse).

**I do not remember a shortcut.** Press `Ctrl+P` and type what you want; each
result shows its shortcut. `?` lists everything.

**An option is grayed in the Settings.** It depends on another option. The row
says which one (for example, *Row tint strength* needs *Graph lane background*
set to *Row tint*).

**"Auto-stash pending" over the Log.** GitPoulpi stashed your changes to run an
operation and could not put them back. Use the banner's **Pop now**, **Show
stash**, **Start over** (undo the operation and put your changes back) or **Drop**; see [Auto-stash](log-and-history.md#auto-stash).

**An agent says `no_viewer` or `deferred`.** `no_viewer`: no GitPoulpi window
is open for that repository (or *Allow AI agents to drive the view* is off).
`deferred`: the window shows a chip; `Ctrl+N Enter` accepts it. See [MCP and
agents](mcp-and-agents.md#showing-things-to-the-user).

**I opened GitPoulpi and see an empty window or `log error: backend error`.**
Check you started it inside a Git repository (otherwise you land on the Home
tab: use **Browse**, **Clone** or **Init**). Then run `gitpoulpi daemon status`;
`gitpoulpi daemon restart` restarts the background process for this
repository. Press `r` to refresh.

**Two windows on the same repository behave strangely.** They share one daemon
per repository on purpose, so a change in one shows in the other. If a daemon
seems stuck, `gitpoulpi daemon stop --all` stops every session's daemon; the
next window starts a fresh one.

**A commit, pull or push asks for a password, or fails to authenticate.**
GitPoulpi uses your `git`'s own credential helper and ssh-agent for the
network. Make sure `git fetch` works in a plain terminal first. GitHub/GitLab
tokens for pull requests are separate and optional: see [Hosting and issue
trackers](hosting-integration.md).

**A fetch, pull or push fails with "The configured credential helper failed", "The SSH host key of the remote is not known yet", or a refused sign-in, and its Activity Log entry has an "SSH & credentials..." button.**
The failure comes from how this repository signs in. The button opens
Settings > SSH & credentials, where you pick another source for this
repository: git's own helpers with the repository's helpers ignored, an
integration account, a key file (with a known hosts file), or a helper or token
command of your own; **Test connection** then checks every remote. For a
helper that could not run, `git config --show-origin --get-all
credential.helper` shows where it is set.

**"GitPoulpi did not send the account token to ...".** The repository's remote
points at a server that is not the address the chosen account is bound to
(often a `url.<x>.insteadOf` rewrite in the repository's own config), so the
token was held back. Fix the remote or choose another source on the page.

**"This repository's token command failed", "... changed since you confirmed it", "This repository's sign-in cannot be used".**
The page's choices cannot be carried out: the token command printed no token
(its last error lines are quoted, with secrets removed), a script it runs
changed since you confirmed it (confirm the command again on the page), or the
choice names a key file or account that is gone. Each reading has the
**SSH & credentials...** button.

**OpenSSH says "bad permissions" for a key file.** Other accounts can read the
key. On Settings > SSH & credentials the **Private key file** row shows a
warning with **Restrict to my account**.

**"staging..." stays on the status line.** A background refresh is still running; it
disappears when the file list updates. `r` forces a refresh.

**A very large repository is slow.** Lower *Initial commits in graph* on the
Settings > Log & graph page (the Log loads more as you scroll when *Lazy load commits in graph* is on); hide
the Log's *Changes* column and turn avatars off; turn *Undo enabled* off if you
do not need undo (no bookkeeping at all on staging and commits).
`GITPOULPI_TIMING=1` writes per-request timings
to `gitpoulpi-timing.log` in the temp folder, worth attaching to a performance
report.

**The AI buttons are missing or gray.** AI is off by default; see [AI
assistance](ai.md#troubleshooting).

**GitPoulpi closed by itself.** Every exit is recorded: look for
`gitpoulpi-tui-exit.log` (and `gitpoulpi-tui-panic.txt` after a crash) in the
config directory (`gitpoulpi config path` prints the settings file, which sits there), and attach it to an issue.

**Where are my settings?** `gitpoulpi config path`. Per-repository state is in
`.gitpoulpi/repo-state.toml` inside the repository.

**Does GitPoulpi send anything anywhere?** Only what you ask for: git
operations to your remotes, hosting requests after you connect an account, AI
requests after you press an AI button, and, if you turn avatars on, a hash of
author emails to Gravatar. There is no telemetry. See [AI
assistance](ai.md#privacy).

## Every write fails: "Unable to create index.lock"

A git command that died (a crash, a killed terminal) can leave
`.git/index.lock` behind, and then every change to the repository fails. Once
the file is older than 10 seconds GitPoulpi shows a strip above the Log:
**Remove index.lock** (palette: *Repository: Remove Stale index.lock*; CLI:
`gitpoulpi index-lock remove`). It asks first and deletes only that file.
GitPoulpi cannot reliably tell whether another git program is still running,
so make sure none is (an IDE, another terminal) before you confirm. In a linked
worktree the lock is the worktree's own, in `.git/worktrees/<name>`.

## "git am" stopped and Rebase Abort does nothing

An `am` (applying patches from a mailbox) is not a rebase. A stopped `git am`
has its own strip, **Applying patches (git am)**, with Resolve conflicts,
Continue, Skip and Abort (palette: *git am: Continue / Skip This Patch /
Abort*; CLI: `conflicts am-continue|am-skip|am-abort`). Abort asks first,
drops the patches applied so far and puts the branch back where it was. These
are not undoable, like the rebase buttons.

## Undo is grayed and says "Can't undo ..." (blocked entry)

The newest journal entry is stuck: the repository moved on outside GitPoulpi,
or the entry was interrupted. Hover Undo for the reason; if it is stuck for
good, the tooltip points at **Edit: Forget Blocked Undo Entry** (palette) or
`gitpoulpi undo-forget`. It asks first and only drops the entry from the
journal; the repository is not touched. See [Undo and redo](undo-redo.md).

## Large files show as three lines of text ("LFS files show as pointers")

A file that should be a picture, a model or a big binary contains `version https://git-lfs.github.com/spec/v1` and an `oid sha256:...` line: it is a Git LFS **pointer**, and the real content has not been downloaded (or the LFS filters are not installed).

- Look at the Status pane: a repository that uses LFS while `git-lfs` is missing shows an amber banner. Install the program (https://git-lfs.com; Windows: bundled with Git for Windows or `winget install GitHub.GitLFS`; Debian/Ubuntu: `apt install git-lfs`; macOS: `brew install git-lfs`) and reopen or press `r`.
- `gitpoulpi lfs status` says whether the program is installed, whether the filters and hook are set up and which patterns are tracked; `gitpoulpi lfs files` lists the pointer files and which are `local`.
- With `git-lfs` installed, run **`LFS: Pull Objects`** in the palette (or `gitpoulpi lfs pull`): it downloads the objects and replaces the pointers. It installs the LFS filters for this repository first if there are none.
- If the download fails, the message names the reason (credentials for the remote, a server that no longer has the object). GitPoulpi needs no account of its own: LFS uses your remote's usual credentials (credential helper or ssh-agent).
- A clone of an LFS repository never fails because of LFS; if the download step could not run, the clone is there with pointers, and `LFS: Pull Objects` finishes it later. `gitpoulpi clone --no-lfs` skips the download on purpose.
- You can turn off everything LFS in Settings > Git & encoding > Git LFS (or `[git] lfs_enabled = false`).

## "git-lfs not found" / "Git LFS is not installed"

The message `Git LFS is not installed. This repository stores large files with Git LFS; ...` (or a dialog with an *Open install page* button, or git's own `git-lfs: command not found` / `'lfs' is not a git command`) means the repository is configured for LFS - its `.gitattributes`, `.git/config` filters or `pre-push` hook call `git-lfs` - and the program is not on the `PATH` seen by GitPoulpi. Install it (link above). On Windows the program ships with Git for Windows; if `git lfs version` works in a terminal but GitPoulpi still says it is missing, start GitPoulpi from a terminal that sees the same `PATH`. To hide LFS from GitPoulpi while leaving git's own behavior alone, turn `git.lfs_enabled` off.

## The graph is drawn with characters, not as a picture (herdr, tmux, other terminals)

GitPoulpi draws the Graph column and author avatars as pixels only where it is sure the terminal will show them: iTerm2 inline images in bare WezTerm, iTerm2 and mintty; sixel pictures in Windows Terminal, foot, Konsole and mlterm; the Kitty graphics protocol in kitty, Ghostty, and **herdr when herdr's own config sets `kitty_graphics = true`** (`[terminal]`, or `[experimental]` on older herdr) and herdr runs inside WezTerm, kitty or Ghostty. Everywhere else it draws characters, which is always correct.

- In herdr with Kitty graphics on but a config GitPoulpi does not find (or relying on herdr's default), set `ui.image_protocol = "kitty"` (Settings > Log & graph > Image protocol) and restart.
- In a terminal that shows boxes or garbage where the graph should be, set `ui.image_protocol = "none"` or `ui.graph_renderer = "text"`.
- `GITPOULPI_TERMINAL_IMAGE=off` turns pictures off regardless of the config.

**herdr remote (`herdr --remote <host>`, or `ssh` then `herdr`): the text graph is the default.** Over SSH, GitPoulpi runs on the remote host and can only read the remote's environment and herdr `config.toml`. Whether a Kitty picture appears is decided by the *local* herdr client (its own `kitty_graphics` setting and its own terminal), which GitPoulpi cannot see: when the client does not draw the pictures, the cells stay blank. GitPoulpi therefore draws the text graph whenever it detects herdr over SSH (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`). If your local herdr has `kitty_graphics = true` and a terminal that draws Kitty images (kitty, Ghostty), opt in with `ui.graph_renderer = "image"` (or `ui.image_protocol = "kitty"`) on the remote. Pictures are sent inline (`t=d`), which works over any byte stream. If the column is empty, set `ui.graph_renderer = "text"`.

**A lighter rectangle behind the lanes.** The picture is transparent (straight-alpha RGBA PNG) wherever nothing is drawn, so the terminal's background and the selection band show through. Kitty-protocol terminals and **WezTerm** composite that alpha; GitPoulpi then reserves ordinary blank cells under the picture and sends it after them. Every other iTerm2-protocol terminal (iTerm2 itself, Rio, mintty) is not known to blend PNG alpha, so there the picture carries an opaque backdrop in the theme's color, which can show as a rectangle on a terminal whose background differs from it; `ui.graph_renderer = "text"` removes it. With `ui.graph_lane_background = "off"` nothing is painted behind the lanes on the transparent paths.

See `ui.graph_renderer` and `ui.image_protocol` in the [configuration reference](config-reference.md).
