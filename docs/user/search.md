**English** · [Čeština](cs/search.md) · [Deutsch](de/search.md) · [Español](es/search.md) · [Français](fr/search.md) · [Italiano](it/search.md) · [日本語](ja/search.md) · [한국어](ko/search.md) · [Nederlands](nl/search.md) · [Polski](pl/search.md) · [Português (Brasil)](pt-BR/search.md) · [Português (Portugal)](pt-PT/search.md) · [Русский](ru/search.md) · [Svenska](sv/search.md) · [Türkçe](tr/search.md) · [Українська](uk/search.md) · [简体中文](zh-CN/search.md)

# Search

`Ctrl+Alt+F` (or `Ctrl+P`, then `Search: Workspace...`) opens a fuzzy finder over
**every repository you are working in**: the repositories of the active
[workspace](workspaces.md) when there is one, else the open
tabs. The **Global search** button (the magnifier and its label in the title bar, just left of
the gear) opens it too. It finds:

- **repositories** (by name, then by path),
- **branches** (local and remote-tracking) and **tags**,
- **worktrees** (by folder, then by path and branch),
- **stashes** (by message),
- **commits** (by subject; a commit's body is searched only when its subject does not
  match and you typed three characters or more),
- **pull or merge requests** and **issues** (by title, `#number`, author and branch).

![The search popup over two open repositories: branches, a stash and a commit matching "main", each with its repository on the right](images/workspace-search.png)

Results are grouped by kind, best match first inside each group. Every row has the
kind's icon, the matched characters highlighted, a dim detail (the upstream of a
branch, the author and branch of a request, a stash's position), and the repository's
name at the right, so two repositories with a `main` branch are never confused. A
spinner appears in the title when a repository is still indexing its commits (the
first search in a big repository can take a moment); the other kinds are there at once
and the commits join as soon as they are ready.

## Going there

| Key | Does |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Move through the results (the group headings are skipped). |
| `Enter` | Goes to the result: the repository's tab opens (or is switched to), then the result is shown. |
| `Esc` | Closes the search. |
| `Alt+1` .. `Alt+8` | Switch a kind on or off (the chips above the list, in their order). `Alt+0` turns them all on. |

What "goes there" means:

| Result | Enter |
|---|---|
| Branch or tag | Selects it in the Refs pane and its commit in the Log. |
| Commit | Selects it in the Log, loading more of the Log first when it is older than what is loaded. |
| Stash | Selects its row in the STASHES section. |
| Worktree | Opens it as a tab. |
| Pull/merge request, issue | Selects its row in the sidebar (in that repository's tab). |
| Repository | Switches to its tab (opening it when it is not open). |

`Ctrl+N Backspace` (or `Back` in the palette) returns to where you were, like any
view change an agent makes for you. Nothing is checked out, merged or changed by going
somewhere.

With the mouse: a click on a row selects it, a double click goes there, a click on a
chip switches that kind on or off, the wheel scrolls, and a click outside closes the
popup.

## Narrowing with a prefix

Start the query with a prefix to search one kind only, whatever the chips say:

| Prefix | Kind |
|---|---|
| `r:` | repositories |
| `b:` | branches |
| `t:` | tags |
| `w:` | worktrees |
| `s:` | stashes |
| `c:` | commits |
| `pr:` (or `mr:`, `p:`) | pull or merge requests |
| `i:` | issues |
| `#` | pull/merge requests and issues (`#12` finds number 12) |

`fix: login` is a plain query, not a prefix: only a single letter (or `pr:` / `mr:`)
followed by a colon, or a leading `#`, counts.

## How it matches

The same three tiers as the command palette: your text verbatim, then verbatim
anywhere, then its characters in order but apart. An earlier match and a shorter text
win. Upper and lower case are the same.

## Pull requests and issues

They come from what GitPoulpi already loaded for each repository (the lists behind the
PULL REQUESTS and ISSUES sidebar sections), kept in a small cache file in the
repository's `.gitpoulpi/` folder. The search **never calls GitHub or GitLab**: a
repository whose requests were never loaded in a window has none to find, and what you
see is as fresh as the sidebar's last refresh. Without a linked account there is
nothing to find, and nothing to configure.

## From the command line

`gitpoulpi search <query> [--workspace ID] [--kind K]... [--limit N]` does the same
and prints JSON (see the [CLI reference](cli-reference.md#workspaces-and-search));
the MCP server has a `search` tool, so an agent can find what you can.
`search.commit_limit` in [Settings](config-reference.md#search-the-workspace-search)
bounds how many commits of each repository are indexed (`0` turns commit search off).

## Changing the key

`Search: Workspace...` is a palette command and the key is the `global.workspace_search`
action: rebind it under Settings > Keyboard & mouse, or in the `[keybindings.global]` table (`workspace_search = "Ctrl+Alt+F"`).
