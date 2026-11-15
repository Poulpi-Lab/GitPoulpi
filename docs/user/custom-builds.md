**English** · [Čeština](cs/custom-builds.md) · [Deutsch](de/custom-builds.md) · [Español](es/custom-builds.md) · [Français](fr/custom-builds.md) · [Italiano](it/custom-builds.md) · [日本語](ja/custom-builds.md) · [한국어](ko/custom-builds.md) · [Nederlands](nl/custom-builds.md) · [Polski](pl/custom-builds.md) · [Português (Brasil)](pt-BR/custom-builds.md) · [Português (Portugal)](pt-PT/custom-builds.md) · [Русский](ru/custom-builds.md) · [Svenska](sv/custom-builds.md) · [Türkçe](tr/custom-builds.md) · [Українська](uk/custom-builds.md) · [简体中文](zh-CN/custom-builds.md)

# Custom builds

For anyone who wants a smaller binary: an interactive script asks which
languages, hosting integrations, themes, logo colors and features to include, then builds only
those with the optimized profile. The normal build and the official release
are unchanged (everything on).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd or PowerShell (scripts/build-custom.ps1)
```

Output lands in `target/custom/`, never over the official `target/dist/`. The
script prints each binary's size and the difference from `target/dist/` when
that build exists. It also writes `BUILD-FEATURES.txt` next to the binaries.

## What you choose

| Choice | If you turn it off |
|---|---|
| Languages | Only English (always built) plus the ones you check. The Settings picker lists only built languages. |
| Themes | GitPoulpi Dark, GitPoulpi Light and Terminal are always built (`system` needs Dark and Light). Every other theme is picked by family (`catppuccin`, `gruvbox`, `nightfox`, ...; `--list-themes` prints them). Unselected themes are not compiled in: they are missing from the Settings combo and `gitpoulpi theme list`, and a config naming one falls back like an unknown theme, with the usual warning. |
| Logo colors | The pixel octopus of the About window comes in 20 colorways (`--list-logo-colors`). Pick the ones to compile in (at least one), the default one (About shows it first; it is also what a stored `ui.logo_color` that is not compiled in falls back to; it must be one of the compiled ones; official default: lavender) and the color of the Windows executable icon (any colorway, default: the logo default). With a single colorway the logo still spins on click but stays that color, and the color list (`c`, right click) is not offered. |
| Integrations (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | With none: clone by URL only; no issues, no pull/merge requests. |
| AI | No AI buttons, commands or `ai` CLI/MCP tools. |
| MCP | No `gitpoulpi-mcp` binary and no `gitpoulpi mcp` command. The `ui` agent commands stay. |
| LFS, Worktree, Submodules | The matching management screens and commands are gone; a typed "not built into this binary" answer is returned. |
| Avatar download | Initials badges only, no network fetch. |
| Image preview | Image files show the plain binary notice; **Edit in** still works. |
| Radar | No Radar tab, `Alt+R` or palette command; the Radar never opens. Needs an integration (with none it is inert anyway). |
| Mermaid | No Copy as Mermaid gitGraph (Log menu, `Shift+Y`, palette) and no `log --format mermaid` in the CLI/MCP. |
| Board | No Kanban board view (ISSUES header button, palette) and no `hosting board` command. Needs an integration. |
| Conversation | No in-app conversation view of an issue or request: Enter on an issue row, a board card or a Radar row opens the browser as before. Needs an integration. |
| Review | No in-app code review view (row menu, Enter on a pull request row, palette, Radar `Review`) and no AI review assist. The `hosting review` CLI/MCP commands stay (Gerrit and agents use them). Needs an integration. |
| History workbench (advanced history rewriting: authors, dates, messages, content) | No History Rewrite Workbench: no `Rewrite:` palette commands, no workbench chord, keys or help section, no "Rewrite history" rows in the Log menus, no "Modify History Deeper..." button in the Amend dialog, no "Create rules with AI" or script AI, no script limits on the Sync settings page and no history rewrite AI blocks in the AI settings. The `rewrite` CLI/MCP commands, `ai rewrite-script`, `ai rewrite-rules` and `ui propose rewrite` are gone from the command tree. `.mailmap` identities (`mailmap`, `log.use_mailmap`) are not part of it and stay. |
| Global search | No Global search button in the title bar, no `Ctrl+Alt+F`, no `Search: Workspace...` palette command and no `gitpoulpi search` CLI/MCP command. The Log find bar (`ui show search`) is a different feature and stays. |
| Workspaces | The window works with the single default tab list ("No workspace"): no workspace chip, no Workspaces tab, no `Alt+W` / `Alt+J`, no `Workspace:` palette commands, no "Add to workspace" tab rows, no `gitpoulpi workspace` CLI/MCP commands, no `--workspace` flag on `commit` and `search`. Tabs and sessions work as usual. A `workspaces.toml` written by another build is neither read nor changed; a workspace's default profile and AI instructions do not apply. The Radar and the Global search cover the open tabs. |
| Rewrite scripts (embedded Lua) | Script rules in a history rewrite are refused with a typed "not built into this binary" answer, and no Lua is linked. The History workbench itself stays (it has its own choice). Meaningless, and warned about, when the History workbench is off. |

Your config files are shared between builds: a setting for something that is
not built is kept and ignored.

## Interactive

The Bash script uses `whiptail` or `dialog` checklists (three screens, then a
summary), or numbered prompts when neither exists. The PowerShell script uses
`Out-GridView` when available, else prompts. Your previous choices are
pre-checked.

## Non-interactive

Any argument skips the questions and goes straight to
`cargo run -p xtask -- build-custom`:

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # previous choices, print the cargo command only
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Option | Meaning |
|---|---|
| `--langs fr,de` / `all` / `en` | Languages besides English. Codes are the locale file names (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Providers: GitHub, GitLab (incl. self-managed), Azure DevOps (Services and Server), Bitbucket (Cloud and Data Center) (hosting), Gitea and Forgejo incl. Codeberg (`gitea`), Gerrit Code Review (`gerrit`, review only) (hosting), Jira, Trello, Linear, YouTrack (Cloud and Server), Redmine (issue trackers), Local Markdown issues (`local-issues`, a folder or Obsidian vault). Each of `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello`, `youtrack` is its own cargo feature; sizes in [binary-size](../developer/binary-size.md). |
| `--themes all` / `none` / `nord,catppuccin` | Theme families to compile in (default `all`). `--list-themes` prints the families. |
| `--logo-colors all` / `violet,teal` | Logo colorways to compile in (default `all`, kept in the table's order, which is the spin order). `--list-logo-colors` prints them (`*` = official default). |
| `--logo-default teal` | The default colorway; must be among the compiled ones (default: lavender, else the first compiled). |
| `--icon-color sun` | Color of the Windows executable icon (default: the logo default). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | Each defaults to on. |
| `--[no-]history-rewrite` | The History workbench (advanced history rewriting: authors, dates, messages, content). Default on. |
| `--[no-]global-search` | Global search (title-bar button, `Ctrl+Alt+F`, `gitpoulpi search`). Default on. |
| `--[no-]workspaces` | Workspaces (named groups of repositories). Default on; off leaves only the default tab list. |
| `--[no-]rewrite-scripts` | Embedded Lua for the Script rules of a rewrite. Default on; ignored with a warning when the History workbench is off. |
| `--save <name\|file>` | Store the resolved choices as a saved selection (a name, or a file path). |
| `--preset <name\|file>` | Start from a saved selection (a name, or a file path); the other options override it, wherever they stand on the line. |
| `--list-presets`, `--delete-preset <name>` | Print the saved selections and their folder / delete one by name. |
| `--last` | Reuse `target/custom-build/last-choices.env`; other options override it. |
| `--full` | No selection at all (everything on). |
| `--profile custom\|dist\|release` | Default `custom` (the `dist` settings, separate output directory). |
| `--dry-run` | Print the exact cargo command and stop. |
| `-- <cargo args>` | Passed to cargo, for example `--target x86_64-unknown-linux-musl`. |

PowerShell names: `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

The theme selection reaches the build as the environment variable
`GITPOULPI_THEMES` (`all`, `none` or a comma list of families), read by
`gitpoulpi-config`'s build script, which drops the other palettes at compile
time; set it yourself for a plain `cargo build` if you prefer. `gitpoulpi
--version` prints a `themes:` line with the set that was built.

The logo choices are the environment variables `GITPOULPI_LOGO_COLORS`
(`all` or a comma list of slugs), `GITPOULPI_LOGO_DEFAULT` and
`GITPOULPI_ICON_COLOR`, read by the build scripts of `gitpoulpi-tui` (it
filters `assets/logo/colourways.json` into the table the About window
embeds) and of the three binary crates (the icon). The source of the colors
stays `assets/logo/colourways.json`.

### Windows executable icon

When the target is Windows, the `gitpoulpi`, `gitpoulpi-tui` and `gitpoulpi-mcp`
executables get the octopus as their icon, generated at build time (no
Python or ImageMagick needed) in the chosen icon color, sizes 16 to 128 px.
It is embedded with the `winresource` crate and needs the platform resource
compiler (`rc.exe` from the Windows SDK on MSVC, `windres` on GNU). If it is
missing the build only prints a warning and the executable keeps the default
icon. On the GNU toolchain only `gitpoulpi` carries the icon (the resource
would otherwise be linked twice). Builds for Linux and macOS ignore all this
except the color table.

## Saved selections

To rebuild a new version without redoing the whole selection, save it under a name and start from it later. Options given together with `--preset` still override the saved values.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

Saved selections are plain `KEY=value` files (the format of `last-choices.env`, readable by a shell). They live in the `custom-builds` folder of GitPoulpi's config directory, outside `target/`, so `cargo clean` does not remove them:

- Linux: `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (`~/.config/gitpoulpi/custom-builds/` by default)
- Windows: `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS: `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` moves that folder together with the config. `--preset` and `--save` also accept a file path (a value containing `/` or `\`, or ending in `.env`), to keep a selection in a dotfiles repository.

A selection written by another version of GitPoulpi still loads. An unknown key, or a language, integration, theme or color that no longer exists, is ignored with a warning. A key the file lacks (a feature added since) takes its default, and an integration added since is included; both are reported on the standard error.

When a saved selection exists, the interactive mode first offers "Start from" (previous choices, a saved selection, or everything on), and at the end it asks whether to save the current selection under a name.

## What it saves

Measured on Linux on 2026-10-29 at commit `65999ef5c`, from the full build, difference from it in parentheses
(details in `docs/developer/binary-size.md`):

| Build (from `full` unless noted) | `gitpoulpi` (MiB) | `gitpoulpi-tui` (MiB) | `gitpoulpi-mcp` (MiB) |
|---|---|---|---|
| Full (everything on) | 64.6 | 50.5 | 31.7 |
| English only | 52.0 (-12.6) | 40.2 (-10.3) | 31.7 (0.0) |
| MCP off | 61.6 (-3.0) | 49.6 (-0.8) | not built |
| AI off | 62.6 (-2.0) | 48.6 (-1.8) | 30.5 (-1.3) |
| All integrations off | 56.3 (-8.2) | 43.7 (-6.7) | 26.7 (-5.0) |
| AI + integrations + avatar fetch off | 52.2 (-12.3) | 39.9 (-10.6) | 23.0 (-8.8) |
| History workbench off (`--no-history-rewrite`) | 61.6 (-3.0) | 48.9 (-1.5) | 28.5 (-3.2) |
| Rewrite scripts off | 63.1 (-1.5) | 49.0 (-1.5) | 31.1 (-0.7) |
| Global search off | 63.5 (-1.1) | 49.5 (-1.0) | 31.6 (-0.1) |
| Workspaces off | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.2) |
| Radar off | 63.4 (-1.2) | 49.3 (-1.2) | 31.7 (0.0) |
| Board off | 63.6 (-1.0) | 49.5 (-1.0) | 31.7 (0.0) |
| Review off | 63.2 (-1.4) | 49.0 (-1.4) | 31.7 (0.0) |
| Mermaid off | 63.6 (-1.0) | 49.5 (-0.9) | 31.6 (-0.1) |
| LFS off | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.3) |
| Worktree off | 63.7 (-0.9) | 49.6 (-0.8) | 31.7 (0.0) |
| Submodules off | 63.7 (-0.9) | 49.6 (-0.9) | 31.6 (-0.1) |
| Avatar fetch off | 63.7 (-0.9) | 49.6 (-0.9) | 31.7 (0.0) |
| Image view off | 63.3 (-1.2) | 49.2 (-1.3) | 31.7 (0.0) |
| Everything off, English only (LFS kept) | 35.2 (-29.4) | 28.9 (-21.5) | not built |
| **Everything off, English only** | **34.9 (-29.7)** | **28.8 (-21.6)** | not built |

## Notes

- A custom build is slow like any `dist` build (fat LTO, a few minutes).
- It runs without sccache, like `build-opti`.
- Language codes are checked against the shipped locales; an unknown code is
  refused with the list of valid ones.
