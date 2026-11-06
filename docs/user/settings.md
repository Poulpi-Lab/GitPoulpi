**English** · [Čeština](cs/settings.md) · [Deutsch](de/settings.md) · [Español](es/settings.md) · [Français](fr/settings.md) · [Italiano](it/settings.md) · [日本語](ja/settings.md) · [한국어](ko/settings.md) · [Nederlands](nl/settings.md) · [Polski](pl/settings.md) · [Português (Brasil)](pt-BR/settings.md) · [Português (Portugal)](pt-PT/settings.md) · [Русский](ru/settings.md) · [Svenska](sv/settings.md) · [Türkçe](tr/settings.md) · [Українська](uk/settings.md) · [简体中文](zh-CN/settings.md)

# Settings

Open the Settings with `Ctrl+,`, `Ctrl+N` then `,`, the gear in the title bar
(left of the `×`), or `Ctrl+P` > *Settings*. The Settings screen replaces the
window while it is open. Close it with the `✕` at the top right, `Esc` or
`q`. It asks first only when closing would lose something (see
[Closing Settings](#closing-settings)).

Changes are saved to `config.toml` at once. The
[configuration reference](config-reference.md) describes every key, its default
value and its environment variable. This page describes the Settings screen.

![The Settings screen: a search box and the page list on the left, the General page on the right](images/settings.png)

## Using the Settings screen

| To | Do |
|---|---|
| Move between the page list and the options | `Tab` / `Shift+Tab`, or click. |
| Move | `j` / `k` or the arrow keys. |
| Change an option | `Enter` or `Space`, or click its value. See below. |
| Close | Click `✕`, or press `Esc` or `q`. While a field, a dropdown or a confirmation is open, `Esc` only closes that. |

What `Enter` or `Space` does depends on the row:

- A checkbox toggles.
- A dropdown opens. The arrow keys choose, `Enter` accepts.
- A text or number field starts editing. The arrow keys, `Home`, `End`,
  `Backspace` and `Delete` work, and a click places the caret. `Enter`, `Tab`,
  `Up`, `Down` or a click elsewhere saves the value. `Esc` cancels. A refused
  value stays in the field, in red, and the status line gives the reason.
- A row of buttons (**Create**, **Remove**, **Test**...) shows only the buttons that apply
  now. `Left` / `Right` choose one, `Enter` presses it, or click it. A button that
  deletes something asks first.

On every page the values start in one column, so the labels line up on the left
and the fields line up on the right. A text or number is a tinted band, a
choice is a band ending in `▾`, a checkbox is `[x]`. A long label wraps instead
of being cut, and on a narrow window the value moves under its label.

A grayed row says why it is inactive. For example, **Row tint strength** is
grayed while **Graph lane background** is **Off**.

### Lists

Pages that edit several items of the same kind (Profiles, AI, Workflow, Open in,
Images) show them as a **list on the left and the selected item's fields on the
right**. On a narrow window the list sits above the fields. The list has the same
shape everywhere:

- A row's mark tells its state: `●` the active profile, `★` the default entry, `⚠`
  an item that is not finished (a profile with no email), with the reason where the
  details would go.
- Under the list: **+ Add**, **Duplicate**, **Remove**, `↑` and `↓`. A button that
  cannot apply now (`↑` on the first row) is dimmed. **+ Add ▾** opens a dropdown
  where the page offers presets or kinds.
- **Remove** asks first, except for an item you added a moment ago and did not edit.
- Drag a row to move it; a bar shows where it will land, and `Esc` cancels the drag.
  A double click moves into the item's fields.

| Key | Does |
|---|---|
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | Select an item. |
| `Insert` | Add an item (opens the dropdown when there is one). |
| `Ctrl+D` | Duplicate the item. |
| `Delete` | Remove the item, after a confirmation. |
| `Alt+Up` / `Alt+Down` | Move the item up or down. |
| `Enter` or `Right` | Go to the item's fields. |
| `Left` | Back to the page list. |
| `Alt+Left` / `Alt+Right` | Switch between the General tab and the list tab (anywhere on the page). |

Each of these pages has two **sub-tabs** under its title, so a setting of the page
is never mistaken for a property of the selected item: **General**, the page's own
settings (the active profile, the default entry, the instructions every AI action
receives, the workflow template...), and a tab named after the items (**Profiles**,
**Profiles (connections)**, **Entries**, **Editors**, **Rules**) that holds the list
and the selected item's fields. A page opens on **General**; Settings remembers the
tab you left each page on until you quit, along with the selected row and the scroll
position of each page (and tab). Click a tab, or press `Alt+Left` /
`Alt+Right`. The search box, the command palette's `Settings:` rows and the links
that say "set it in Settings" open the tab the setting is on.
[Keyboard & mouse](#keyboard--mouse) has the same two tabs (**General**, **Keybindings**).

The keys can be rebound in **Keyboard & mouse** (context *preferences*, see
[Keybindings](keybindings.md)).

### Multi-line editors

The settings that hold several lines (such as the sparse checkout paths, the commit template's description, the base branches of Conflict prevention and the workflow hooks) open in a small editor with `Enter` or a click; `Ctrl+S` keeps the text and `Esc` drops it. The mouse works there as in any text box:

- A click places the caret where you clicked.
- Dragging selects text; the wheel scrolls.
- A double click selects the word under the pointer.
- `Shift` + click extends the selection from the caret to the pointer.

The keyboard offers the same: `Shift` with the arrow keys selects, `Ctrl` with the arrow keys jumps by word, `Ctrl+A` selects everything, `Ctrl+C` and `Ctrl+X` copy and cut.

### Closing Settings

The `✕`, `Esc` and `q` do the same. Settings applies each change at once, so most
of the time it just closes. It asks **Close Settings?** only when something would
be lost:

- the **Workflow** page has changes that are not saved;
- the commit message template has changes that are not saved;
- a profile has no author email (the commits it makes cannot be attributed to you).

The box offers **Go to it** (opens that page), **Discard and close** or **Close
anyway**, and **Cancel**. The small editors inside Settings (an AI instructions
text, a workflow hook, the sparse checkout paths) have their own `✕` and ask
**Discard changes?** only when their text was edited.

### Popups over Settings

The title line shows the application's octopus and name, like the main window's.
Click the octopus to open About.

A few window-wide popups open over Settings and give it back as it was (the page,
the scroll position and any unsaved Workflow or template changes stay):

- the command palette (`Ctrl+P`),
- the theme picker (`Ctrl+N y`, **App: Change Theme...**), which previews each
  theme on Settings itself; `Esc` puts the previous theme back,
- the language and graph color pickers (`Ctrl+N` then the same letters as in the main window),
- the workspace search (`Ctrl+Alt+F`),
- About (`Ctrl+N a`, the octopus, or **Help: About GitPoulpi**) and the help overlay (`?`).

Close each with `Esc`, its own `✕`, or a click outside it. Commands that act on
what Settings covers (a tab, a branch, a diff...) are listed in the palette but
grayed out with "close Settings first", and the `Ctrl+N` chords for them show that
line on Settings' bottom border: close Settings, then run them.

`Ctrl+N` shows a badge on each control of the page: the search box, the page list (one badge, on the current page), the sub-tabs, the list and its buttons, every field on screen and the close cross. Press a badge's character and it acts like a click on that control: a text field opens for editing, a checkbox is ticked, a dropdown opens, a button runs. `Esc` cancels. A field scrolled out of view has no badge. While you type in a field, or a dropdown, an editor or the search is open, `Ctrl+N` belongs to it. See [Badges](keybindings.md#badges).

### The search box

The box at the top of the page list searches the name and the description of
every option on every page. Click it, or press `/` or `Ctrl+F`, and type. A list
of matches drops down. Each match shows the option and its page. `Up` / `Down`
choose, `Enter` goes to the option, `Esc` closes the list.

![Searching the Settings for "diff": the matching options from the Log & graph, Diff & editor and Git pages](images/settings-search.png)

## Settings from the command palette

Most settings can be changed without opening this screen: type `Settings:` in the command palette (`Ctrl+P`). The rows are generated from the pages' own fields, so a new setting shows up in both.

- A checkbox is **Settings: Toggle ...**. It flips at once, and the row shows `[x]` or `[ ]`.
- A dropdown is **Settings: ...**. `Enter` narrows the palette to its values, the current one marked `current`; `Enter` on a value picks it.
- A number or a text opens a prompt filled with the current value. A value the page would refuse (not a number, outside the range) is refused the same way, and the prompt stays open.
- A setting that needs its page (a list, a combo whose options are found at run time) opens Settings on it.

![The command palette filtered on Settings:, with one row per setting and its current value at the right](images/palette-settings.png)

A value is saved to `config.toml` and applied exactly as the page does. The English words of the setting's key (`remember tabs`) find the row in any language.

## The pages

The page list has two groups:

- **Settings**: global settings.
- **Repo-specific settings**: settings for the repository of the active tab.
  They show only while a repository is open.

### General

![The Language dropdown of the General page: Auto and the seventeen languages, with a search field](images/settings-language.png)

| Option | What it does |
|---|---|
| **Language** | The interface language: **Auto** (your system language, else English), English, Français, Español, Português (Brasil), Português (Portugal), Italiano, Deutsch, 简体中文, 日本語, 한국어, Русский, Українська, Polski, Čeština, Nederlands, Svenska, Türkçe. It applies at once, with no restart. The Log's month names and dates follow it. The `GITPOULPI_LANG` environment variable (for example `GITPOULPI_LANG=fr`) overrides this choice. The CLI and MCP output stay in English. |
| **Translate git terms** | Off by default: commit, push, pull, stash, tag and the other git terms stay in English in every language. On: the language's own git vocabulary is used (French: *Tirer*, *Pousser*, *Remiser*, *Étiquette*). Applies at once. |
| **Use English git words** | Only for Japanese, Korean and Chinese: shows *stage*, *unstage*, *staged* as English words instead of their transcription. No effect while **Translate git terms** is on. |
| **Remember tabs** | Reopens the tabs of the last session at launch. Off, a launch inside a repository opens only that repository, and a launch outside one starts on Home with no repository tab. Needs a restart. |
| **Enable the Radar** | On by default (`general.radar_enabled`). Off hides the Radar tab, its palette commands and `Alt+R`, and nothing is fetched for it in the background. Applies at once. See [the Radar](radar.md#closing-the-tab-or-turning-the-radar-off). Not shown in a build without the Radar. |
| **Start tab inside a repository** | The tab a start opens on when the folder it starts in is inside a repository (`general.startup_tab_in_repo`): **Current repository** (default, that repository's tab, opened if needed), **Last tab** (the tab that was active at the last quit), **Home**, **Radar**. A repository given with `--repo` always opens its own tab. **Last tab** is Home when **Remember tabs** is off or nothing is remembered; **Radar** is Home when the Radar is off or not built. Read once at startup; needs a restart. |
| **Start tab outside a repository** | The tab a start opens on when the folder it starts in is not a repository (a user folder, for example) (`general.startup_tab_outside_repo`): **Home** (default), **Last tab**, **Radar**. **Last tab** is Home when **Remember tabs** is off or nothing is remembered; **Radar** is Home when the Radar is off or not built. Read once at startup; needs a restart. |
| **Start tab from the launcher** | The tab a start opens on when GitPoulpi is started from the desktop or Start menu shortcut it wrote (they pass `--launcher`), whatever the folder (`general.startup_tab_launcher`): **Last tab** (default), **Home**, **Radar**. Wins over the two settings above. **Last tab** is Home when **Remember tabs** is off or nothing is remembered; **Radar** is Home when the Radar is off or not built. Shortcuts written by an older version get the flag at the next start. Read once at startup; needs a restart. On Windows a copy you pinned to the taskbar is rewritten at the next start too, and a start with no shell around it (double-click, the exe pinned itself) counts as a launcher start. |
| **Group worktree tabs together** | On by default (`general.group_worktree_tabs`). The tabs of a repository's linked worktrees sit right after that repository's tab, and a worktree tab you drag away snaps back next to it. Off leaves tabs where you put them. Turning it on regroups the open tabs. |
| **Default clone folder** | Where the Clone dialog starts. Empty: `%USERPROFILE%\source\repos` on Windows, `~/src` elsewhere. |
| **Clone dialog starts on** | The source the Clone dialog opens on (`repos.clone_start_source`): **Last used** (default, the source it showed when it was last closed, by cloning or cancelling), **Clone with URL**, or one of your connected integrations. A source that is no longer available falls back to Clone with URL. A clone started from the Radar keeps its own source. |
| **Default browse folder** | Where Browse opens. Empty: the last folder you used, else your home folder. |
| **Repository data location** | Where GitPoulpi keeps its per-repository data (`session.per_repo_state_dir`): **Inside .git** (default: nothing appears in your working tree, all worktrees share it), **In the working tree** (a `.gitpoulpi` folder; needed to commit a team `workflow.toml`) or **In my user data folder** (outside the repository). A `.gitpoulpi` folder left by an older version is moved to the new place the first time it is needed, and removed once empty; a file git tracks (a committed `workflow.toml`) is never moved or deleted and stays readable. |
| **Default branch name** | The first branch of a repository you create with Init. Empty: git decides. It renames no existing branch. |
| **Enable proactive conflict detection** | On by default (`general.conflict_detection`). Warns on the Log when merging the current branch into the branch it started from would conflict, before you try. A repository can turn it off for itself on its **Conflict prevention** page. See [Resolving merge conflicts](merge-conflicts.md#conflict-prevention). Applies when Settings closes. |
| **Log level**, **Keep log files for**, **Maximum size of a log file** | The application log behind the Activity Log: the most verbose level recorded (`log.level`), how long files are kept (a whole number and a unit, days, weeks, months or years, on one line: click the number to type it, Up and Down step it, Tab goes to the unit and Enter opens its list; 1 to 10 years at most, default 1 week, `log.retention_days` and `log.retention_unit`) and the size at which a day's file is rotated (a number and MiB or GiB, 1 MiB to 1 GiB, `log.max_file_mb` and `log.max_file_unit`). |
| **Show how long operations take**, **Extended logging** | The Activity Log's Git operations tab: the duration of each operation (`log.timings`, on by default) and the background work too, refresh and auto-fetch (`log.extended`, off by default). See [When something fails](troubleshooting.md#the-activity-log). |
| **Undo enabled**, **Undo history depth**, **Record changes made outside GitPoulpi** | The **Undo history** group. Turns Undo on or off, and sets how many operations it keeps (default 100, `0` means unlimited). Turning Undo off erases the undo history, after a confirmation. The last one also records what a shell or an agent does to branches, tags, stashes and HEAD. See [Undo and redo](undo-redo.md). |

#### Launcher (desktop integration)

The **Launcher** group, the last on the General page, creates a shortcut that starts
GitPoulpi from the OS launcher, in the terminal you choose.

| Row | What it does |
|---|---|
| **Terminal for shortcuts** | `ui.launch_terminal`: **Automatic**, **OS default**, or one of the terminals found on this machine (the same detection as **Open in**). Automatic: Windows Terminal on Windows (else WezTerm, Alacritty, the plain console), `xdg-terminal-exec` on Linux (Omarchy; else the first terminal found), Terminal on macOS unless iTerm2, Ghostty, WezTerm or kitty is installed. |
| **Reopen in that terminal on double-click** | Windows only (`ui.relaunch_in_terminal`). Double-clicking the exe opens the default console, which draws pictures badly: GitPoulpi reopens itself in the chosen terminal and the first window closes. Not done when it was started from a terminal. |
| **Show the open window on click** | `ui.reuse_window` (on by default). A click on a GitPoulpi shortcut (`gitpoulpi launch`) shows the window of a GitPoulpi that is already open instead of starting another one. A new terminal starts only when none is open, the setting is off, or the window cannot come forward. It comes forward on Windows, in WezTerm and on Hyprland; on other Linux desktops the click still starts a new window. To open another window on purpose, hold **Shift** while clicking (Windows, also on the pinned taskbar icon) or use the **New Window** action of the Linux launcher entry (right-click the icon); `gitpoulpi launch --new-window` does it from a script. Not used on macOS (the Dock already does it). |
| **Command alias** | `ui.command_alias`: a dropdown with **None** and `gp`, or type any name (letters, digits, `-`, `_`, `.`; no spaces or path separators). The inline **Create** button makes that name start GitPoulpi (it becomes **Remove** once the alias exists, and removes every alias GitPoulpi made). A warning shows under the row when the name already runs something else on your PATH, and on Windows when PowerShell has a built-in alias of that name (`gp` is `Get-ItemProperty` there, and it wins over any file). A shell alias or function cannot be seen from here: oh-my-zsh defines `gp` itself and it wins over ours. |
| **Add-tab alias** | `ui.command_alias_add`: a second command, for example `gpa`, that opens the current folder as a new tab of the GitPoulpi that is already running (`gitpoulpi --add`) instead of starting another one; with none running it starts GitPoulpi as usual, and with several it asks which one to join. A dropdown with **None**, `gpa` and the alias name followed by `a`, or type any name (same rules as above). The one **Create** button makes both commands, **Remove** removes every command GitPoulpi made; each has its own status row. On Linux and macOS it is a two-line script, on Windows a `.cmd`. See the [CLI reference](cli-reference.md#add-a-folder-to-the-open-window). |
| **Add GitPoulpi to PATH** | Puts the folder GitPoulpi runs from and the alias folder on your PATH, for new terminals. The row's button reads **Add** or **Remove**, and **Remove** takes out exactly what was added. Folders already on PATH are left alone (the row then says **already on PATH**). |
| **Linux: app launcher** | Writes `~/.local/share/applications/gitpoulpi.desktop` (honors `XDG_DATA_HOME`) and the icon in `~/.local/share/icons/hicolor/256x256/apps`. On Omarchy the entry is the one `omarchy-tui-install` makes (`xdg-terminal-exec --app-id=TUI.tile -e <exe>`); for other terminals the matching command (`kitty --class gitpoulpi`, `wezterm start --class gitpoulpi --`, `alacritty --class gitpoulpi -e`, `foot --app-id=gitpoulpi`, ...). Every entry runs its command through `gitpoulpi launch --`, which shows a GitPoulpi that is already open (see **Show the open window on click**). |
| **Linux: desktop icon** | The same entry in your Desktop folder, when you have one. |
| **Windows: Start menu** | `GitPoulpi.lnk` in `%APPDATA%\Microsoft\Windows\Start Menu\Programs`. A copy pinned to the taskbar follows icon and color changes by itself; if its icon ever stays old, unpin it and pin it again (right-click the Start menu entry, then **Pin to taskbar**). |
| **Windows: desktop shortcut** | `GitPoulpi.lnk` on your Desktop. |
| **macOS: Applications** | `~/Applications/GitPoulpi.app`, a small bundle that opens the terminal running GitPoulpi. |

Each row shows its state: **not created**, **created**, **outdated** (the exe moved or
another terminal is chosen; GitPoulpi rewrites it by itself within a second, so you rarely see it) or **not ours** (a file of that name
exists and was not made by GitPoulpi, so it is never touched). **Remove** deletes only files
GitPoulpi made (they carry an `X-GitPoulpi=1` marker). The icon is the octopus in your logo color
(About > color); on Linux each color has its own icon name, so a launcher that caches icons shows the new color. Once a shortcut is created it stays up to date by itself: when you change the color,
the terminal or the exe moves, GitPoulpi rewrites the shortcuts it made (never one it did not make, and
never creating one), at start-up and while it runs. There is no Update button; only **Create** (for a
shortcut that does not exist yet) and **Remove**.

**Taskbar and dock grouping.** GitPoulpi runs inside a terminal window, so a taskbar normally shows it under the terminal's icon, with no "running" dot under the pinned GitPoulpi icon. Windows: the shortcuts carry the AppUserModelID `GitPoulpi.GitPoulpi`, and a GitPoulpi started from one gives its terminal window the same ID and icon (it finds the window through the terminal program that runs it, else by giving it a unique title for a moment, and keeps `GitPoulpi` as the window title; the steps are in the Activity log as `window grouping: ...`), so the window sits under the pinned GitPoulpi icon. Hold **Shift** while clicking the pinned icon to open a new window instead of showing the open one. A terminal window with several tabs moves as a whole: all its tabs group under GitPoulpi's icon while GitPoulpi runs in it. Shortcuts and taskbar pins made earlier are rewritten by the same sync; if a pin still shows the terminal's icon, unpin it and pin it again. The icon file name carries a hash of the picture (`gitpoulpi-<hash>.ico` in the GitPoulpi folder of the local app data), so a new logo color reaches the pinned button too; older files are deleted once no shortcut uses them. Linux: kitty, foot, Alacritty, WezTerm and Ghostty are started with the window class `gitpoulpi` and the `.desktop` entry names it (`StartupWMClass`), so a dock groups the window under the launcher's icon; other terminals keep their own icon. macOS: nothing to do, the Dock groups by app.

![Settings > General scrolled to the Launcher group: terminal for shortcuts and the app launcher row](images/settings-launcher.png)

##### Command alias and PATH, per system

| System | Alias | PATH |
|---|---|---|
| Linux, macOS | A symbolic link `~/.local/bin/<name>` to `gitpoulpi-tui`. A link cannot hold a marker, so it counts as ours when its target is a file called `gitpoulpi-tui`; a regular file, or a link to anything else, is **not ours** and is never replaced or removed. | A block between `# >>> GitPoulpi PATH (X-GitPoulpi=1) >>>` and `# <<< GitPoulpi PATH <<<` in `~/.bashrc` and `~/.zshrc` (whichever exist; `~/.zshrc` on macOS, `~/.bashrc` otherwise when neither does), and `~/.config/fish/conf.d/gitpoulpi-path.fish` when fish is configured. Remove deletes the block (or the fish file) and nothing else. |
| Windows | `%LOCALAPPDATA%\Programs\GitPoulpi\bin\<name>.cmd`, a two-line shim holding `REM X-GitPoulpi=1`. | The user PATH in `HKCU\Environment`, written through PowerShell, then a `WM_SETTINGCHANGE` broadcast so new programs see it. The value `GitPoulpiPathAdded` records what was added, and Remove takes out only that. |

New terminals see the PATH change, not the ones already open. Neither action is journaled: there is no Undo for them, **Remove** is the way back.

The palette has the same actions (**Launcher: Create Shortcut**, **Launcher: Create Desktop Shortcut**, **Launcher: Remove Shortcuts**, **Launcher: Create Command Alias**, **Launcher: Remove Command Alias**, **Launcher: Add to PATH**, **Launcher: Remove from PATH**), and `gitpoulpi desktop shortcut
status|install|remove [--where launcher|desktop|startmenu]` does it from a script.

The palette command **App: Change Language...** (`Ctrl+N g`) opens the same
language list. There, the highlighted language previews live; `Enter` keeps it
and `Esc` restores the previous one.

### Synchronization

How GitPoulpi keeps a repository in step with its remotes, and what it does with
your local changes meanwhile.

![The Synchronization page: the Auto-fetch interval as a number and a unit (minutes) on one line, then the other options](./images/settings-quantity.png)

| Option | What it does |
|---|---|
| **Auto-fetch interval** | Fetches every remote in the background at this interval, a number and a unit (minutes or hours) on one line. Default 5 minutes, maximum 60 minutes (1 hour), `0` turns it off. `general.auto_fetch_interval_minutes` stays in minutes; `general.auto_fetch_interval_unit` only records how it is shown. Needs a restart. |
| **Fetch when a tab opens** | Runs Fetch All in the background when a repository tab opens. On by default. A repository fetched in the last 60 seconds is not fetched again. |
| **Auto-prune** | Fetches also remove remote-tracking branches that no longer exist on the remote. Off by default. |
| **Auto-prune tags** | Fetches also delete local tags that the remote has deleted. Only tags that had been on the remote are touched: a tag you never pushed is kept. Off by default, like git. Needs git 2.41 or newer. CLI: `gitpoulpi fetch --prune-tags`; config key `general.auto_prune_tags`. See [Branches, tags and the Refs sidebar](log-and-history.md#branches-tags-and-the-refs-sidebar). |
| **Keep submodules up to date** | After a checkout or a pull, updates the submodules to the commits the repository records. See [Submodules](submodules.md). |
| **Delete ".orig" files after merging** | After a merge completes, deletes the `*.orig` backup files. Off by default. |
| **Auto-stash before checkout, pull, merge or rebase** | What to do when local changes block one of these operations: **ask** (a dialog offers to stash the changes, run the operation and restore the stash), **always** (do it without asking) or **never** (refuse). |
| **Include untracked files in auto-stash** | The automatic stash also takes untracked files. |

Also on this page, under **Updates**:

| Option | What it does |
|---|---|
| **Check for updates** | A dropdown (`updates.mode`): **Notify** (default) checks once a day in the background and shows `vX.Y.Z available` in the title bar; **Auto** also downloads the update in the background when GitPoulpi was installed from a release archive (you still choose when to restart); **Off** never makes a request. Applies at once. See [Updates](updates.md). |
| **Update channel** | A dropdown (`updates.channel`): **Stable** (default) or **Prerelease** (also release candidates). Applies at once. |
| **Installed with** | Read-only. How GitPoulpi got onto this computer (a release archive, the installer for this user or all users, Chocolatey, winget, Scoop, Homebrew, apt, an MSI package, the AUR, a Docker image, cargo, or a development build). It decides how updates are applied, see [Updates](updates.md). |

A row your administrator has set in the machine-wide update policy shows **Set by your administrator** and cannot be changed here.

Also on this page, under **History rewrite**, the time limits of external scripts (see [History rewrite](history-rewrite.md)):

| Option | What it does |
|---|---|
| **Script answer time limit** | How long an external script gets to answer one commit, a number and a unit (milliseconds or seconds) on one line. Default 5 seconds, 1 millisecond to 10 minutes (`rewrite.script_timeout_ms`, `rewrite.script_timeout_unit`). On a timeout the whole process is stopped and the rewrite fails with nothing moved. |
| **Script time limit per pass** | How long one whole pass of an external script may take, a number and a unit (seconds, minutes or hours). Default 1 hour, up to 24 hours (`rewrite.script_total_timeout_s`, `rewrite.script_total_timeout_unit`). |

### Profiles

![The Profiles page](images/settings-profiles.png)

Commit identity profiles: the name and email used for commits made from
GitPoulpi. The **General** tab holds the **Active profile** dropdown; the
**Profiles** tab is a [list](#lists) (add, duplicate, remove, reorder) with the
fields of the selected one. Each
profile has a **Profile ID**, a **Profile name**, an **Author name** and an
**Author email**. A profile with no email shows `⚠` in the list. With no active
profile, or a profile with a blank name or email, the repository's own
`user.name` and `user.email` apply.

With two or more profiles, the toolbar shows a profile combobox at its right
end. It sets the profile for the active repository only. After you choose, a
dialog offers to also write the identity into that repository's config.
The palette command is **Repository: Change Profile...**.

These profiles are not the hosting profiles of the Integrations page, nor the AI
profiles of the AI page.

### SSH & credentials

![The SSH & credentials page](images/settings-ssh.png)

SSH uses your system `ssh`, so your ssh-agent and your ssh config keep working.
GitPoulpi never reads key material.

| Option | What it does |
|---|---|
| **Private key path** | When set, fetch, pull, push and clone run git with `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes`, so ssh offers only that key. Empty: the agent and your ssh config apply unchanged. |
| **Public key path** | Used only when the private key path is empty: ssh picks the matching key held by ssh-agent (or a hardware key). |
| **Key type** | The type **Generate key pair** creates: `ed25519`, `rsa` (4096 bits) or `ecdsa` (NIST P-256). |
| **HTTPS credentials** | How fetch, pull, push and clone authenticate over HTTPS: **credential_helper** (git's own helper, the default), **prompt** (ignore helpers; git asks on a terminal) or **none** (no credentials, public remotes only). |
| **Forget all usernames and passwords** | Removes every token GitPoulpi stored in your credential helper. It asks for confirmation first. |
| **Generate key pair** | Creates a key pair of the **Key type** in `~/.ssh/gitpoulpi_<type>`, with no passphrase, and records its two paths. It never overwrites a file. |

The key paths apply to the next transfer. A `~/` prefix means your home folder.

A setting made elsewhere always wins over these paths. The order, highest first:

1. `GIT_SSH_COMMAND` or `GIT_SSH` in the environment GitPoulpi runs in.
2. `core.sshCommand` in the repository's git config or your global git config.
   The tab menu's *SSH key...* item writes this for one repository.
3. The two key paths of this page.
4. ssh's own defaults and the agent.

A key can also belong to a hosting account; see
[Hosting and issue trackers](hosting-integration.md#ssh-key-per-profile).

### Integrations

![The Integrations page: the providers on the left and the Connect button for GitHub](images/settings-integrations.png)

Connects your accounts. The **Hosting** group has GitHub, GitHub Enterprise,
GitLab, GitLab Self-Managed, Azure DevOps, Azure DevOps Server, Bitbucket,
Bitbucket Data Center, Gitea, Forgejo and Gerrit. The **Issue trackers** group
has Jira Cloud, Jira Data Center, Trello, Linear, YouTrack, Redmine and Local
Markdown. Each provider has **Connect**, a token field, the scopes the
token needs and a link to the host's token page. This page has its own guide:
[Hosting and issue trackers](hosting-integration.md).

Two options sit above the providers. Press `Up` on the first provider to reach
them.

| Option | What it does |
|---|---|
| **Pull Requests and issues refresh** | How often the sidebar lists and the toolbar chip refresh, a number and a unit (minutes or hours). Default 5 minutes, up to 2 hours, `0` turns the timer off (`general.hosting_refresh_minutes`, `general.hosting_refresh_unit`). |
| **Review request terminology** | The word used for hosted reviews: **Auto (per host)**, **Pull request** or **Merge request**. |
| **Hide the pull request chip when it is closed or merged** | On by default (`ui.hide_finished_pr_chip`; it reads "merge request" on GitLab). The chip on the toolbar shows the latest request of the checked-out branch; once that request is closed or merged it is hidden and the dim **+ PR** chip (start a new request) shows instead, as if the branch had none. Off, the chip always shows the latest request, whatever its state. Applies at once. |

### AI

![The AI page: Enable AI, global instructions and the per-action blocks](images/settings-ai.png)

On the **General** tab: the switch that turns AI on, the default profile, the
instructions every action receives, and a profile and instructions for each action.
The **Profiles (connections)** tab is a [list](#lists) of the AI profiles with an
inline table for a profile's environment variables. The *Code review* block of the General tab also
holds the defaults of the [review context](ai.md#review-context): **Run the agent in the repository**
(*Profile setting*, *Yes*, *No*), **Include CLAUDE.md**, **Include AGENTS.md** and **Extra file** (a path or a
link; empty adds none, with a **Browse...** button under it that opens a file picker; `Enter` on the button is
its keyboard form). This page has its own guide: [AI assistance](ai.md).

### Agents & MCP

What AI agents may do in this window, and whether the MCP server may run. The
two MCP rows show only in builds that include the server; see
[MCP and agents](mcp-and-agents.md).

| Option | What it does |
|---|---|
| **MCP server** | On (default): an agent can start the server (`gitpoulpi mcp serve`). Off: it refuses to start. Saved as `mcp.enabled`. A change applies the next time an agent starts the server; a server that is already running keeps its settings. |
| **MCP server port** | The port `--http` listens on when no address is given, 1 to 65535 (default 8808). Saved in the port of `mcp.bind`; its host part is kept (`127.0.0.1` when empty). Applies the next time an agent starts the server. |
| **Allow AI agents to drive the view** | Lets agents show you a commit, a file or a notice in this window (`ui show`). Off: no agent reaches the window. |
| **Let AI agents propose changes for me to confirm** | Lets agents open pre-filled Reword, commit, branch or tag dialogs that you confirm. Off by default. See [MCP and agents](mcp-and-agents.md#permissions). |
| **Open a window for agents**, **Launch command (custom)** | Whether an agent may open a GitPoulpi window when none is open, and with which terminal or multiplexer. Off by default. |

### Appearance

![The Appearance page: theme, avatars, input color, color depth and graph colors (the date formats follow below)](images/settings-ui.png)

![The Theme combo open on Appearance with Light highlighted: the whole Settings screen already repaints in that theme](images/settings-appearance-theme.png)

| Option | What it does |
|---|---|
| **Theme** | **System (follows terminal)**, a built-in theme, or one of your own theme files. The highlighted theme previews live: `Enter` keeps it, `Esc` restores the previous one. The **Colors** strip under the row shows the colors in force. See [Themes](themes.md). |
| **Author avatars** | **off** (default, no network), **graph** (in the commit dot) or **author** (in the Author column). When on, a hash of each author email is sent to Gravatar and to your hosting provider. |
| **Avatar sources**, **Avatar fallback** | Where pictures are looked up, and in what order. For authors with no picture: **identicon** (a generated pattern) or **initials**. |
| **Input color**, **Custom input color** | The background of text fields: **Follow theme** (default), a fixed dark color, or your own `#rrggbb`. |
| **Color depth** | **auto** (default), **truecolor**, **256** or **16**. See [Themes](themes.md). |
| **Graph color theme** | **Follow theme** (default: the theme's own lane colors) or any lane palette. Each palette shows a few of its colors. It applies when the Settings close. The palette command **App: Change Graph Color Theme...** (`Ctrl+N Shift+Y`) previews each palette live. |
| **Graph lane background** | **Off** (transparent) or **Row tint**: a faint band of the lane color behind the graph column, with a thin bar at its edge. |
| **Row tint strength** | How faded the Row tint band is: **Strong**, **Medium** (default) or **Subtle**. The edge bar keeps its full color. |
| **Toolbar position** | `ui.toolbar_position`: **Top**, **Bottom** (default), **Left** or **Right**. Left and Right make a column beside the panes; under 80 columns it is drawn at the bottom. Applies at once. Palette: `Settings: Toolbar position...`. |
| **Toolbar button alignment** | `ui.toolbar_align`: **Start** (default), **Center** or **End**: where the buttons sit along the toolbar. Applies at once. |
| **Pin the pull request button to the end** | On by default (`ui.toolbar_pr_pinned`; it reads "merge request" on GitLab). The request button stays at the far end (right, or bottom in a column) whatever the alignment; off, it flows with the other buttons. Applies at once. |
| **Status line position** | `ui.statusbar_position`: **Top** (above the repository tabs) or **Bottom** (default). The Home tab keeps its own status line at the bottom. Applies at once. |
| **Tab bar position** | `ui.tabs_position`: Where the repository tabs sit: Top (default), Bottom, or Left, a list with one tab per row at the window's left edge. A window under 80 columns draws them at the top instead. Beside a column toolbar the tabs stay outermost. Applies at once. |
| **Date/time locale** | **system** (default): month and weekday names in the interface language. **english**: always English names. |
| **Date format**, **Date/time format**, **Verbose date format** | The date patterns of the File History and Blame date column, of the Log's Date column, and of the commit details. Each shows a live preview. See [Configuration reference](config-reference.md#ui) for the tokens. |

![The Graph color theme combo: Follow theme, then every lane palette with a few of its colors](images/settings-log-graph-lanes.png)

### Log & graph

How the commit graph is drawn and how many commits the Log loads. The graph
colors are on the **Appearance** page.

| Option | What it does |
|---|---|
| **Graph renderer** | **auto** draws a picture where the terminal supports it, and characters elsewhere. **image** forces the picture, **text** forces characters. Needs a restart. |
| **Image protocol** | How pictures reach the terminal: **auto**, **kitty**, **iterm2**, **sixel** or **none**. Needs a restart. |
| **Show stashes in the graph** | A row for each stash in the Log. Off: stashes show in the sidebar only. |
| **Show a ghost branch on the hovered commit** | On by default (`ui.graph_ghost_ref`). In the Ref column, the hovered commit with no ref of its own shows a faded pill with the name of the branch it belongs to (else its remote branch), deduced from the history, where the column has room. A branch that was deleted after a merge still shows, struck through, under the name in the merge commit's subject (`Merge branch 'x'`, `Merge pull request #N from owner/x`, ...); squash merges, rebases and fast-forwards leave no such trace. Tags are never shown as a ghost. Hovering the pill names its kind. Clicking it does nothing special; the palette command **Create branch at selected commit** makes a branch. |
| **Ref-to-commit connector** | A thin line from a ref label to its commit dot. |
| **Column layout** | `ui.log_columns_layout`, applied at once to the Log. **Fit the pane** (the default): every column fits the pane exactly and only Message flexes. **Scroll sideways**: columns keep their widths, Message too, and the table scrolls sideways when it is wider than the pane (`Shift+Left` / `Shift+Right`, the horizontal wheel, a bar on the bottom border). The same switch is in the Log's column chooser and the palette. |
| **Column resizing** | `ui.column_resize`, applied at once to the Log, the Radar and the commit column of the diff, file and blame views. **line** (the default): each column boundary is a one-cell gap with a small tick in the header, a thin accent line is drawn down the whole table while the pointer is on the boundary, and dragging anywhere along it resizes. **header**: no tick, the first character of the header cell after the boundary lights up and is the only place to drag from. The sizing keys (`Shift+C` in the Log, `C` in the Radar, `Tab` in the commit column) do the same in both. |
| **Show WIP change counts in the message when the Changes column is hidden** | Shows the change counts (`✎ 3  + 2  − 1`) in the WIP row's message. When the Changes column is shown, the counts go there instead. |
| **Branch folder tree** | Groups branches and tags on `/` into folders in the sidebar. `y` in the Refs pane toggles it. |
| **Left pane sections** | Whose choice the left pane's shown/hidden sections are: **per repository** (the default; each repository keeps its own set, the global one is the starting point) or **global** (one set for every repository). Switching back and forth keeps both sets. See [mouse and gestures](mouse-and-gestures.md). (`ui.sidebar_sections_scope`) |
| **Click a ref to reveal it in the graph** | A single click on a ref in the Refs pane scrolls the Log to where it points and flashes the row (the commit selection and the right panel stay put). Off: a click only selects the sidebar row. **Reveal in graph** works either way. |
| **Composer proposals** | Whether the AI commit composer opens with the files of each proposal **expanded** or **collapsed**. |
| **Initial commits in graph** | How many commits the Log loads when a repository opens. Default 5000, minimum 500. Higher values slow down large repositories. |
| **Lazy load commits in graph** | On: more commits load in the background when you scroll to the oldest loaded commit. Off: the Log stops at the initial commits. |
| **Commits per lazy page** | How many commits one background load fetches. |
| **Maximum commits in graph** | Lazy loading stops at this number. `0` means no limit. Each commit uses about 300 bytes of memory. |
| **Count the total commits with** | How the status line learns the total in `loaded / total` (`log.commit_count`, a dropdown, applies at once): **System git** runs one `git rev-list --count` process (default, fastest normally); **libgit2** counts inside GitPoulpi and starts no process, for machines where antivirus software scans every command and makes each git call slow; **Off** never counts and shows only the loaded commits. |
| **Show authors through .mailmap** | The Log, blame and file history show each author as the repository's `.mailmap` names them (`log.use_mailmap`, a switch, on by default, applies at once). The stored commits are never changed. Off shows the names as committed. See `history-rewrite.md`, *The Identities tab*. |

### Diff & editor

![The Diff and editor page](images/settings-editor.png)

| Option | What it does |
|---|---|
| **Tab size** | Columns per tab stop, 1 to 8 (default 4). |
| **End of line** | What the editor writes on save: **Auto (keep the file's)**, **LF** or **CRLF**. |
| **Syntax highlighting** | Language colors in the editor. |
| **Line numbers** | The line number column of the editor and the diff. |
| **Follow the selection in the editor** | While the editor is open, selecting another file in the right pane opens it in the editor too, after asking to save any unsaved changes. A file that cannot be edited opens in its usual view instead. Only for the working changes: a file picked from a commit leaves the editor for that commit's diff. On by default (`editor.follow_selection`). |
| **Word wrap** | Wraps long diff lines. |
| **Default layout**, **Hunks only**, **Show whitespace-only changes**, **Minimap** | The initial state of the diff view. The diff view bar changes them and saves them. |
| **Blame annotations**, **Blame annotation width** | Which side of the Blame view holds the annotation column (avatar, author, age, summary): **right** (the default) or **left**, and its width in cells (24 to 90). Dragging the column's border in the Blame view, or the `>` / `<` keys, saves the width here (`ui.blame_side`, `ui.blame_width`). See [Blame](log-and-history.md#blame). |
| **Merge editor: side by side from (columns)** | From this width, the merge editor shows A and B side by side. Default 100, minimum 40, `0` means always. Below it, A and B are tabs. See [Resolving merge conflicts](merge-conflicts.md). |
| **External editor** | The command for a file's *Open in editor* item. `{file}` is the path. Empty: VS Code if installed, else the system default. |
| **External diff tool** | The command for *Open in external diff tool*. `{local}` and `{remote}` are the two versions. Empty: `git difftool`. |
| **External merge tool** | The command for *Open in merge tool* on a conflict card. `{base}`, `{local}`, `{remote}` and `{merged}` are the files. Empty: `git mergetool`. |
| **File explorer** | The command for *Show in folder*. `{path}` is the file, `{dir}` is its folder. |

### Open in

![The Open in page, Entries tab: the list of entries, the entry being edited with its label and command, and the + Add button](images/settings-open-in.png)

Edits the list of the **Open in** button in the title bar. The **General** tab
holds the **Quit & cd** folder buttons, the **Default entry** and **Restore detected
defaults**. The **Entries** tab is a [list](#lists) (a label and a command each):
**+ Add ▾** offers the presets and **Custom**, and the default entry is marked `★`.
The fields of the selected entry include a **Test** button. See
[Open in](open-in.md).

The first row, **Follow the active tab's folder**, has two values:

- **buttons** (default): the tab row shows **Quit & cd here** when the active
  tab is not the folder GitPoulpi was started in.
- **auto**: no button. Every quit changes your shell's folder to the active
  tab, and opening a new tab runs the default Open in entry.

See [Following the active tab's folder](following-the-tab-folder.md).

### Images

How image files are shown, and which program **Edit in** uses. See
[Pictures](image-view.md). The **General** tab holds the viewer options below, the
**Default entry** and **Restore detected defaults**; the **Editors** tab is the
list of image editors. A build without image preview has no viewer options.

| Option | What it does |
|---|---|
| **Preview images** | Shows an image file as a picture in the Diff view and the File View. Off: the plain binary notice. On by default. |
| **Without an image protocol** | On a terminal with no inline-image protocol: **Half-block colors** (the default) or **Info card only**. |
| **Maximum preview size** | Bigger images are not decoded; the info card shows instead. A number and MiB or GiB, default 32 MiB, up to 1 GiB (`ui.image_view_max_mib`, `ui.image_view_max_unit`). |
| **Zoom in pixelated from** | `1x`, `2x` (default) or `never`: from this zoom, the picture shows square pixels instead of a smoothed picture. |
| **Sync before/after zoom and scroll** | The two panes of an image diff share one zoom and one scroll position. On by default. |

The **Editors** tab is a [list](#lists) like the [Open in](open-in.md) one:
per editor a **Label**, a **Command**, the variable
chips `${file}` `${dir}` `${name}` `${repo}` and **Test with a sample image**;
**+ Add ▾** offers presets and **Custom**, and **Restore detected defaults**
resets the list. Until you edit it, the list holds the editors installed on
this machine and the system's default app. Every change applies at once.

### Keyboard & mouse

![The Keyboard & mouse page: every action by context with its keys, a search box and filters](images/settings-keyboard.png)

**Settings > Keyboard & mouse** has two tabs (click, or `Alt+Left` / `Alt+Right`).
**General** holds the keyboard protocol and the mouse settings; **Keybindings**
lists every shortcut by context, with a search box (`/`) and filters. On a row:

| Key | Action |
|---|---|
| `Enter` or double-click | Captures a new key. Conflicts and terminal warnings show while you press it. |
| `a` | Adds an alternative key. |
| `u` | Unbinds the action. |
| `r` | Resets the row to its default. |
| `R` | Resets every shortcut, after a confirmation. |
| `c` or a click on the row's `!` | Lists the action with every action its key clashes with; `Esc` shows them all again. A key that clashes with another action's is red, a warning about the key alone (a terminal that cannot send it) orange. |

Use it to resolve clashes with tmux, AutoHotkey, herdr or your terminal. The
same settings live in the `[keybindings]` table of `config.toml`; see the
[configuration reference](config-reference.md#keybindings) and
[Keybindings](keybindings.md).

![The Keyboard & mouse page after pressing c on Quit: the key q is red on both Quit and Stage or Unstage the Selected Row, the two actions that clash](images/settings-keyboard-conflicts.png)

Three options are on the **General** tab.

| Option | What it does |
|---|---|
| **Kitty keyboard protocol** | **auto** asks the terminal for the kitty keyboard protocol, so `Ctrl+,`, `Ctrl+Enter` and `Ctrl+Shift+N` arrive. **off** never asks; use it if a terminal misbehaves. Needs a restart. |
| **Mouse enabled** | Off leaves the mouse and text selection to your terminal. Needs a restart. |
| **Mouse wheel lines** | How many rows one wheel step moves in the Log. Needs a restart. |

### Repo-specific: Git & encoding

![The Git and Encoding page for this repository](images/settings-git.png)

| Option | What it does |
|---|---|
| **Git backend** | `git2` (libgit2, the default) or `system` (your installed `git`). Global. It applies to daemons started after the change. |
| **Pull button** | What the Pull button does: merge (fast-forward if possible), fast-forward only, or rebase. Global. The `▾` items do not change it. |
| **Push button** | **direct** pushes at once; **ask** shows the remote menu first. Global. |
| **Default encoding** | The encoding used to show text that is **not valid UTF-8**: file contents, blame, diffs and commit messages. The page has a global row and a row for this repository; the repository row wins. Valid UTF-8 is never re-decoded, and staging never re-encodes anything. |
| **core.autocrlf**, **core.eol** | This repository's own git settings. The row writes `core.autocrlf` or `core.eol` in the repository's git config. **(inherited)** removes the repository value, so your global git config applies. `Ctrl+Z` restores the previous value. The CLI equivalent is `gitpoulpi repo-config`. |
| **Git LFS** | Shows Git LFS in the interface. See [Git LFS](lfs.md). |
| **Install LFS filters before pull and push** | Before a pull or a push, runs `git lfs install --local` when the repository uses LFS and its filters are missing. |
| **git-lfs program** | The detected `git-lfs` program (read-only). |
| **Initialize LFS for this repository** | Runs `git lfs install --local` now. |

### Repo-specific: Sparse checkout

The same controls as the Sparse checkout dialog (`Repository: Sparse Checkout...`), as a page of this repository's
settings (`gitpoulpi ui show settings sparse-checkout`).

| Option | What it does |
|---|---|
| **State** | Whether this working tree is a sparse checkout. |
| **Directories to keep** | One directory per line, relative to the repository root. `Enter` (or a click) opens a multi-line editor: `Ctrl+S` keeps the text, `Esc` drops it. While sparse checkout is on, the field shows the current directories. Git's cone mode only knows directories (no patterns, no single files), the files at the root are always checked out, and an empty list is refused. |
| **Start sparse checkout** | Checks out only the listed directories. Once sparse checkout is on, the same button reads **Apply directory list** and applies the edited list: a directory that left the list is removed from your working directory. Greyed out while the list is empty. |
| **Stop sparse checkout** | Back to a full checkout: every file comes back (after a confirmation). Greyed out while sparse checkout is off. |
| **Refresh sparse checkout** | Applies the current list again (for example after a script or another tool checked out files outside it). It sends the current directories, not the unsent text of the field. Greyed out while sparse checkout is off. |

The buttons run as ordinary background actions (the page shows "applying the sparse checkout..." while one runs). Like
the dialog, they are not on the Undo stack: nothing is lost, and **Disable** brings every file back.

### Repo-specific: LFS

The LFS state of this repository (`gitpoulpi ui show settings lfs`; not in a build without LFS).

- **Not initialized:** "Git LFS is not set up in `<repository>`" and the **Set up LFS here** button,
  which runs `git lfs install --local` (this repository's filters and hooks, never your global configuration; needs
  `git-lfs`). When the repository's `.gitattributes` already routes files through LFS but its hooks are missing, the
  page says so.
- **Initialized:** "Git LFS is set up in `<repository>`", and the patterns tracked in `.gitattributes`
  (`filter=lfs`).

LFS counts as **initialized** when this repository has its own LFS hooks (the `pre-push` or `post-checkout` hook that
`git lfs install --local` writes). Filters alone do not count: `git lfs install` at user level sets them for every
repository, so every repository would look initialized.

The toolbar's **`LFS ▾`** button follows the same state: it shows only in a repository where LFS is initialized or
where `.gitattributes` already uses it, and stays hidden in a plain repository (and until the first LFS read of a
repository has landed, so it can appear a moment after the repository opens). Initializing from this page makes it
appear. The `LFS:` palette commands stay available everywhere, **LFS: Initialize for This Repository** included.

### Repo-specific: Commit

How a commit made in this repository behaves (`gitpoulpi ui show settings commit`). Both switches are **off** by
default and are saved for this repository only (`[preferences]` in
[`repo-state.toml`](config-reference.md#per-repository-settings-gitpoulpirepo-statetoml)).

| Setting | What it does |
|---|---|
| **Push on every commit** | After a successful commit, pushes the checked-out branch to its upstream, as the usual push (a tracked transfer, with the in-flight text "committing, then pushing..."). A branch with no upstream is committed but not pushed, and the notice says so: the first push of a new branch stays the **Push** button's, which sets the upstream. A rejected push opens the usual push-rejected card; the commit stays. |
| **Bypass git hooks by default for every commit** | The commit form's **Skip hooks** checkbox (next to **Amend previous commit**; `Shift+H` in the Status pane, or **Commit: Toggle Skip Hooks** in the palette) starts checked, so the `pre-commit` and `commit-msg` hooks do not run (`git commit --no-verify`). It stays a choice made for each commit: after a commit it returns to this default. |

Without the default, a repository's commit hooks run on every commit, whichever backend GitPoulpi uses (a repository
with commit hooks is committed through the system `git`, because libgit2 runs none). A hook that fails shows its
output in its Activity Log entry (the status line says the commit failed) and nothing is committed.

### Repo-specific: Merge options

One switch, **Squash** (off by default). When on, merging a local branch (the branch's menu, a drag and drop, `Merge`
in the palette) stages the branch's changes without making a merge commit (`git merge --squash`): the commit is left to you.
The commit form opens with git's squash message (`SQUASH_MSG`, comment lines removed), once per
squash. Nothing is in progress afterwards (no `MERGE_HEAD`), so **Abort** does not apply: to back out, discard the
staged changes. A squash that stops on conflicts leaves them in the Status pane like any conflicted merge.
The staged changes are not an undo entry (no ref moved, so there is nothing to restore); the CLI's
`gitpoulpi merge <branch> --squash` does the same, whatever the setting.

### Repo-specific: Message template

A message the commit form starts with (`gitpoulpi ui show settings commit-template`).

- **Prefill an empty commit panel with the template**: an *empty* commit form is pre-filled with the template, once per commit
  (clearing it by hand is respected until the next commit).
- **Drop comment lines when making a commit**: when the commit is made, lines starting with `#` (or the repository's
  `core.commentChar`) are removed from the message, as `git commit --cleanup=strip` would. GitPoulpi commits with
  `-m`, which git does not clean up, so it does this itself. With the switch off, a `#` line stays in the commit.
- **Summary** (the label counts the 72 columns left, and how many are over) and a multi-line **Description**
  (`Enter` or a click opens the editor, `Ctrl+S` keeps it).
- **Save template** writes the template to `commit-template.txt` in the repository's data folder (see
  `session.per_repo_state_dir`) and sets this repository's `commit.template` to that file, so `git commit` on the
  command line and other tools use it too. If `commit.template` already points at another file, the page shows that
  file's path, edits *that file* and leaves the setting alone.

### Repo-specific: Conflict prevention

`gitpoulpi ui show settings conflict-prevention`. The page of the proactive conflict detection of this repository
(see [Resolving merge conflicts](merge-conflicts.md#conflict-prevention)):

- **Check for conflicts with the base branch**: on by default. Unchecking it disables the detection
  only for this repository; the global switch is **Enable proactive conflict detection** on the General page.
  Both must be on.
- **Base branches** (`**` by default): the branches the current branch may have started from, one pattern per line
  (`Enter` or a click opens the editor, `Ctrl+S` keeps it). The pattern table under it is the reference:

  | Pattern | Matches |
  |---|---|
  | `origin/main` | only that branch |
  | `origin/**` | every branch of the remote `origin` |
  | `origin/feature/**` | the branches of `origin` that start with `feature/` |
  | `!origin/abc` | excludes that branch |

  A local branch is its bare name (`main`), a remote-tracking one has its remote in front (`origin/main`). `*` and `?`
  stay inside one path segment, `**` crosses them. Blank lines and `#` comments are ignored; a list of exclusions
  only means "every branch except these". It is stored in `repo-state.toml` and read again when Settings closes.

### Repo-specific: AI

![The repository's AI page: repository instructions, with Add to global or Replace global, and a profile and instructions for each action](images/settings-repo-ai.png)

This repository's AI profile and instructions for each action, and the
*Repository Instructions* that apply to every action in this repository. Empty
fields use the global AI settings. See [AI assistance](ai.md#instructions). Its *Code review* block repeats the
four review context rows: each choice starts on *Use global (value)*, showing the global value, and an empty
**Extra file** shows the global one grayed (and has the same **Browse...** button; the picker starts at the
repository root and a file inside the repository is stored relative to it); the **No extra file** switch turns the
global extra file off for this repository, while an empty field keeps following it; **Reset to global** (shown once the repository has values of its own)
puts all of them back. They are stored in this repository's `repo-state.toml`.

### Repo-specific: Workflow

`gitpoulpi ui show settings workflow`. The branching workflow of this repository (template, branch detection, base
branch): it has its own guide, [Branching workflows](branching-workflows.md#settings--workflow). The **General** tab
holds what is not a rule (template, detection, hooks, issue flow, where it is saved); the **Rules** tab is a
[list](#lists) in groups (**Base branches**, **Topic types**, **Finish rules**); **+ Add ▾** picks the kind and
items move within their group. Unlike the other pages it keeps a draft (the bar is on both tabs): a bar pinned at the bottom of the page shows
**● Unsaved** (or **No changes**) with **Save to the shared file**, **Only for me** and **Discard changes**, and closing
Settings with an unsaved draft asks first.

## Files, environment variables and the CLI

- The settings file is `config.toml` in your OS config folder.
  `gitpoulpi config path` prints its location.
- Per-repository state lives in `.gitpoulpi/repo-state.toml` inside the
  repository.
- `gitpoulpi config get <key>` and `gitpoulpi config set <key> <value>` read and
  write every setting.
- Environment variables and command-line flags override the file.

The [configuration reference](config-reference.md) gives the details.
