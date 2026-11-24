**English** · [Čeština](cs/radar.md) · [Deutsch](de/radar.md) · [Español](es/radar.md) · [Français](fr/radar.md) · [Italiano](it/radar.md) · [日本語](ja/radar.md) · [한국어](ko/radar.md) · [Nederlands](nl/radar.md) · [Polski](pl/radar.md) · [Português (Brasil)](pt-BR/radar.md) · [Português (Portugal)](pt-PT/radar.md) · [Русский](ru/radar.md) · [Svenska](sv/radar.md) · [Türkçe](tr/radar.md) · [Українська](uk/radar.md) · [简体中文](zh-CN/radar.md)

# The Radar

The Radar is a tab of its own, right after Home. It is one list of what is waiting for you across **several repositories**: the pull (or merge)
requests you wrote, the ones that ask for your review, the issues assigned to you or created by you, and
the repositories where you left work in progress. It is the Home window with the
Radar view on top, so that nothing underneath can steal a click or a key.

It needs a code-host profile (see [Hosting and issue trackers](hosting-integration.md); the lists of one repository are in [Pull requests and issues](pull-requests-and-issues.md), and a row's discussion opens as a [conversation](conversations.md)) and
repositories linked to it, or a profile with its account-wide lists turned on (see "Beyond the scope" below).
A repository that is not linked is never asked: it shows a **Link...** line instead (see "Repositories with something to say" below).

![The Radar over two repositories: a pinned request, then Needs my review, My pull requests, Assigned to me, issues and work in progress](images/radar.png)

**Review...** on a pull or merge request row (or `e`) opens the repository with the request's in-app review
([Reviewing a pull request](review.md)). For a request of a repository you cloned but never linked to the host, the clone is
linked to the request's project first (unless it already has its own pull request link), so the review starts in its tab.

### The Actions column

At the right of a request or issue row, small buttons do what a key does on the selected row (a click selects the row and
runs the action; rest the pointer on a button for its name and key):

```
Wide:    #58  Round invoice totals  feature/login → main   alice   2d   Review  [globe]  [talk]  ⊕  ×  ⇣  ⋮
Narrow:  #58  Round invoice totals  feature/login → main                 [rev]  [globe]  [talk]  ⋮
Issue:   #12  Crash on empty list                         bob     5h   Start work  [globe]  [talk]  ×  ⋮
```

| Button | Key | Palette |
|---|---|---|
| **Review** (a request that waits for your review; asks how when it is not cloned) | `e` | `Radar: Review Request...` |
| **Start work** (a cloned issue) | `n` | `Radar: Start Work on Issue...` |
| the globe: open the page on the host | `o` | `Radar: Open in Browser` |
| the lines glyph: open the conversation | `i` | `Radar: Open Issue Conversation` |
| `»` **Mark as ready** (a draft request) | `y` | `Radar: Mark as Ready` |
| `⊕` **Merge** (a request) | `m` | `Radar: Merge Request...` |
| `×` **Close** (a request or an issue) | `x` | `Radar: Close...` |
| `⇣` **Clone** (a row whose repository is not cloned) | `g` | `Radar: Clone Repository...` |

A row without an action leaves its cell empty, so each button stays in the same column on every row. When the window is
narrow the Review and Start work words become glyphs before any other column disappears; the Ready, Merge, Close and Clone
buttons leave the column before the title does, but stay in the row menu (`⋮`, right-click, or `Shift+F10`), where an action
that cannot run is grayed with its reason. The column is automatic (it is not
one of the sizable columns). A double-click or `Enter` on a request that is not cloned asks how to review it, like **Review**.

### The branches of a request

A request row shows `source → target` in the **Branches** column, right after the repository (`alice:feature/login → main`
when the source branch is in a fork); an issue row leaves the cell empty, and a Radar with no request that names its branches
has no such column. When the cell is too narrow the source is cut first (an ellipsis) and the target stays whole as long as
it can. The column shrinks before it disappears: in a narrow window it is the last of the optional columns to leave
(after the state, author and age, before the repository and the title); once it is gone the whole branches stay in the
title's tooltip and on the status line of the selected row. A wide author column you sized yourself gives way to the title
and the branches. The Actions column is never pushed out. Rest the pointer on a cut text for the
whole of it. GitHub and Gitea list requests without their branches, so the Radar reads them after the list (once per request
until it changes). A host that did not name the branches shows nothing there. An issue row names its repository in the same chip as a request row (the
folder name of the clone, or `owner/name` when it is not cloned).

### Acting on the host

**Merge**, **Close** and **Mark as ready** work on GitHub, GitLab, Gitea and Forgejo (other hosts gray them with "Not supported
by ..."; issues can be closed on the same hosts).

- **Merge** asks first: the box names the request, its project and branches, and offers the method (a merge commit, squash,
  and rebase where the host has it; GitLab merges or squashes). It is refused with its reason for a draft, a request that
  conflicts with its target, or failing checks; a refusal by the host (no permission, a rule of the branch) is shown on the
  status line.
- **Close** asks first and closes without merging.
- **Mark as ready** takes a draft out of draft (GitLab and Gitea: the `Draft:` / `WIP:` marker leaves the title). It asks
  nothing: it is reversible on the host.
- **Clone** opens the ordinary Clone dialog on the row's project (the URL is filled, the folder picker is the usual one). Once
  the clone exists the Radar places the row on it the next time it reads the remotes (reopen it or Refresh), and Clone then
  says where the clone is.

Merge and Close change the host and **cannot be undone from GitPoulpi** (the box says so); reopen a closed request on the host.
After a successful call the lists are asked again, so a merged or closed row leaves and a ready one loses its `draft` chip.

## Opening it

- **`Alt+R`** (rebindable: `global.radar`), from anywhere.
- The palette: `Radar: Open`.
- The **Radar** tab, right after Home in the tab bar.
- The **`+`** menu's **Radar** row, while the tab is closed.

From a repository tab this takes you to the Home window with the Radar open; `Esc` or **Back** returns
to the list of repositories, and the `⌂` does the same.

## Closing the tab, or turning the Radar off

The tab closes like a repository tab: its **x**, a middle click, `Ctrl+W` while it shows, or the palette's
`Radar: Close tab`. Home is the only tab that cannot close. A closed tab stays closed for that session
across restarts, until you open the Radar again (`Alt+R`, `Radar: Open`, or the `+` menu); it then
comes back right after Home. Closing it changes no setting.

If you never use it, turn it off in **Settings > General > Enable the Radar**
(`general.radar_enabled`, see the [config reference](config-reference.md)). Off, the tab, its palette
commands and `Alt+R` are gone and GitPoulpi fetches nothing for it. It applies at once, without a restart.

## What it covers: the scope

The **Scope** combobox picks which repositories are asked:

- a **workspace**, by name (its repositories, even those with no tab open);
- **Open tabs**: the repositories open as tabs of this session.
- **Default clone folder** and **Default browse folder**: every repository found in the folder set in
  [Settings](config-reference.md) (`repos.clone_root`, `repos.browse_root`), whether or not it has a tab.

It opens on the last scope you picked here (kept in `radar.toml`), as long as it still exists; before
your first pick, or when that workspace was deleted, on the session's active workspace, else on the open
tabs. The choice is for this view only: it does not change the window's active workspace. When two repositories have the same folder name, the parent folder is added to the chip
so you can tell them apart.

### The two folder scopes

A folder scope looks for repositories **two levels deep**: the folders directly in it, and the folders inside
those that are not repositories themselves (`~/src/work/app`). Hidden folders are skipped, a repository is
never entered, and at most 200 repositories are listed. The search runs in the background, never on the
thread that draws, and its result is kept until you refresh: the Refresh button, `r`, `F5` (the window's refresh key, rebindable) or `Radar: Refresh`
searches the folder again (a repository cloned since then shows up).

Hosting stays opt-in per repository: a repository of the folder that is **not linked** to a host is never
asked. It only contributes its work-in-progress row, has a `not linked` line with **Link...** under
Repositories (it opens that repository's pull request section, where you pick the provider and the
project), and the status line counts them (`3 not linked (work in progress only)`).

A folder that is not set is greyed in the list (`Default browse folder (set it in Settings)`); picking it
opens its Settings row (Settings > General) and leaves the scope as it was. The clone folder counts as set
through its per-OS default (the folder the Clone dialog starts on) as long as that folder exists; the
browse folder has no default. The scope list is read from the settings each time it opens, so a change made
there applies at once.

## The list

| Group | What it holds |
|---|---|
| Pinned | Everything you pinned, whatever its kind, in the order you pinned it |
| Needs my review | Requests that ask for your review (and that you did not write) |
| My requests | The requests you wrote; the chip says `draft` for a draft |
| Assigned to me | Requests assigned to you |
| Mentioning me | Open requests and issues that mention you and are not already in a group above (an issue neither assigned to nor created by you shows here only when it mentions you) |
| Other requests | The remaining open requests |
| Issues assigned to me | Open issues assigned to you |
| Issues created by me | Open issues you created |
| Work in progress | Repositories of the scope with uncommitted changes or unpushed commits |
| Snoozed | Items you hid for a while; folded until you open it |
| Repositories | Repositories with something to say (offline, token to renew, not linked) |

The words "pull request" and "merge request" follow the usual terminology setting and the host.

Each row shows the item kind (`⇄` request, `●` issue, `✎` work in progress), the repository chip
(in the workspace's color), the number, the title, the author, the age and, where there is room,
the labels. A work-in-progress row shows the branch and what is waiting in it: `✎` files changed,
`+` new files, `↑` commits to push, `↓` commits to pull.

Counts are in each heading. Within a group, the most recently updated item comes first.

The status line under the filter shows a spinner when the hosts take longer than a moment to answer.
The cached lists (the same cache the sidebar uses) show at once and are replaced as the answers arrive.

### Columns

The repository chip, the branches, the `#` (number and check marks), the author and the age are **sizable columns**. Left on automatic, the repository column is as wide
as the longest name on screen (up to 40 cells, and at most a third of the window), so `owner/name` and an
account list's `group/sub/project` are not cut on a normal terminal. The title takes what is left. A name that
does not fit ends in an ellipsis (`group/sub...`), and:

- **Rest the pointer** on a cut repository, title, author or label list and a tooltip shows the whole text. Select
  the row and the status line under the filter names the full project and title too, so the keyboard sees
  the same.
- **Drag the separator** (the cell right after the repository name, after the `#`, or just before the author or the age) to size the
  column; **double-click** it to fit the column again. The list has a **header row** (repository, branches, `#`,
  title, author, age) and the separators follow `ui.column_resize` like the Log's: with `line` (the
  default) a muted `│` rests in the separator on the header row, and while the pointer is on the separator
  (on the header or on any row of the list), while the column is picked with the keys, or while you drag, a thin
  accent line is drawn down the whole list; a drag from any point of it resizes. Once you let go, the line goes away unless the
  pointer is still on the separator (then it goes with the pointer): a mouse drag never leaves a column picked. With `header` there is no tick or line: the first
  cell after the separator, on the header row, lights up and is the only place to drag from. The pointer turns
  into a resize pointer over a separator in terminals that support it. The state chip has a
  fixed width, and so has the age (the widest age shown, until you size it; an age cut by a narrow column shows its
  full date in a tooltip), so the author starts at the same cell on every row.
- Or use the keys: **`C`** picks a column (again, or `Tab`, for the next one), **`]`** widens it by two
  cells, **`[`** narrows it, **`0`** puts every column back to automatic, `Esc` leaves. The palette has the
  same: `Radar: Pick Column`, `Radar: Widen Column`, `Radar: Narrow Column`, `Radar: Reset Column Widths`.

The widths are kept in `radar.toml` (`[columns]`, `repo`, `branches`, `number`, `author` and `age`, in cells; a file without `branches` loads as before, the column then fits its content up to 28 cells) and survive a restart. They
are the same sizing gestures as the Log's columns.

### Checks and conflicts

A request row carries two small marks after its number when the host said something:

| Mark | Meaning |
|---|---|
| `✓` | the head commit's checks passed |
| `✗` | a check failed |
| `●` | checks are queued or running |
| `!` (next to it) | the request conflicts with its target branch |

Rest the pointer on the marks and a tooltip says it in words ("Checks failed", "Conflicts with main");
select the row and the status line under the filter says the same, so the keyboard sees it too. A request
with no mark is one the host said nothing about (no CI configured, or not asked).

- **GitLab** sends the pipeline status and the conflict flag with the merge request list: every row has
  its marks.
- **GitHub** does not: the list has neither. GitPoulpi asks for them per request (the request itself for
  its merge state, the head commit's combined status and its check runs: three calls), for at most
  **15 open requests per repository**, yours first. If GitHub refuses one of those calls (a rate limit),
  the rest stay unmarked until the next refresh. The answers are saved with the hosting cache.

- **Gitea and Forgejo** are read like GitHub: the list carries the conflict flag (`mergeable`), the commit
  status is asked per request (at most 15 per repository, yours first), and the issues are the same groups.
  Mentions use the server's search (`mentioned_by`).

### Mentions

**Mentioning me** and the **Show: Mentioning me** filter use what the host knows about mentions. GitHub is
asked through its search (`mentions:`), GitLab through your to-do list: only mentions still **pending**
there count, so a to-do you marked as done leaves the group.

Closed and merged requests and closed issues are not listed.

## Views and filters

Four comboboxes and a text field narrow the list. Every choice is a dropdown: click it, or use its key.

- **View** (`v`): All, requests, Issues, WIP (work in progress), Snoozed.
- **Show** (`f`): Everyone, Mine (requests you wrote, issues you created), Assigned to me,
  Review requested, Mentioning me.
- **Scope** (`w`): see above.
- **Profile** (`P`): All profiles, or one integration profile of Settings > Integrations (named as
  Settings names it, with its provider added when two profiles share a name). It narrows every
  section, the repositories' rows and the account-wide rows alike, to the rows of that profile (a
  work-in-progress row has no profile, so a chosen profile hides it). One profile at a time. The
  choice is remembered in `radar.toml`; a profile that no longer exists falls back to All profiles.
  The combobox is always there, like the other three, and a narrow window shortens it (and then the
  labels) rather than dropping it. Palette: `Radar: Filter by Profile...`. When the profile hides
  everything, the empty list says so.
- **Filter text** (`/` or `Ctrl+F`): fuzzy match on the repository, number, title, author, branch and
  labels. `Enter` gives the keys back to the list and keeps the text, `Esc` clears it.

`Ctrl+N` shows a badge on the filter, each of the four comboboxes, the Refresh and Back buttons and every
section heading on screen. Press a badge's character: the filter gets the keyboard, a combobox
opens, a button runs, and a heading gets the selection (`Enter` or `Space` then folds it). `Esc`
cancels. See [Badges](keybindings.md#badges).

## Actions

Select a row and use its menu (right-click, the `⋮` at the end of the row, `.` or `Menu`), a key, or the
palette (`Radar: ...`). A mouse action always has a key and a palette twin.

| Action | Key | Palette |
|---|---|---|
| Open repository (its tab opens, or is switched to; a request that is not cloned asks how to review it) | `Enter` / double-click | `Radar: Open Repository` |
| Review... (a request; a request that is not cloned asks how) | `e` | `Radar: Review Request...` |
| Review a request through the host's API (no clone) | `a` in the question | `Radar: Review Through the API` |
| Clone the repository, then review the request | `c` in the question | `Radar: Clone and Review...` |
| Open in browser | `o` | `Radar: Open in Browser` |
| Check out branch (a request) | `c` | `Radar: Check Out Branch` |
| Create worktree... (a request) | `t` | `Radar: Create Worktree...` |
| Show in repository (its list) | `s` | `Radar: Show in Repository` |
| Start work on issue... (an issue) | `n` | `Radar: Start Work on Issue...` |
| Pin to top, or Unpin | `p` / click the `★` | `Radar: Pin or Unpin` |
| Edit assignees... / reviewers... / labels..., Set milestone... (GitHub, GitLab) | row menu | `Radar: Edit Assignees...`, `Radar: Edit Reviewers...`, `Radar: Edit Labels...`, `Radar: Set Milestone...` |
| Snooze... | `z` | `Radar: Snooze...` |
| Unsnooze | `u` | |
| Show or hide the requests of repositories that are not cloned | `a` | `Radar: Show Not-Cloned Requests` |
| Show or hide drafts of the account-wide rows | `d` | `Radar: Show Drafts` |
| Turn on a host's account-wide review requests | `T` / `Enter` on its hint row / **Turn on** | `Radar: Turn On Account-Wide Review Requests...` |
| Refresh | `r`, `F5` | `Radar: Refresh` |
| The row menu | `.` | `Radar: Row Menu...` |
| Back to Home | `Esc` | `Radar: Back to Home` |

Actions that need the repository's own window (check out, worktree, show, start work, link, the metadata pickers) switch to its
tab first and run there once it has loaded:

- **Check out** uses the sidebar's own check-out: the branch when it is known locally, the request's head
  fetched into a local branch otherwise (a fork's, for instance).
- **Create worktree...** opens the Create worktree dialog on the request's branch.
- **Show in the repository** selects the item in the PULL REQUESTS or ISSUES list.
- **Start work on an issue** opens the New branch prompt with a name made from the issue (`12-fix-the-thing`).

### Pinning and snoozing

A **pin** keeps an item on top. A **snooze** hides it for **1 hour**, **4 hours**, **Until tomorrow**
(08:00 the next morning, your local time, even when it is not yet 08:00 now) or **Until it changes** (the host reports an update on the item, or, for a work-in-progress row,
its counters move). Both are kept in `radar.toml` in your configuration folder, keyed by host,
repository, kind and number, and are shared by every session on the machine.

### Repositories with something to say

A repository with a problem gets its own line at the end of the list, and its cached items stay on screen:

- **offline, 5m old**: the host could not be reached;
- **Reconnect**: the token is missing or refused; the button opens Settings > Integrations;
- **paused this round**: the host said to slow down (or refused the token), so GitPoulpi did not ask it
  again for the other repositories of this round;
- **not linked, GitHub suggested**: the repository's remote matches a profile, but hosting is opt-in per
  repository, so nothing is asked until you link it. **Link...** takes you to the repository's PULL
  REQUESTS section, where you pick the provider and the project.

At most four repositories are asked at the same time, and never on the thread that draws the screen.

## Beyond the scope: account-wide rows

The scope covers the repositories you cloned. A request that waits for your review in a repository you
never cloned is invisible there. **Account-wide rows** ask the host itself, once per profile, and
join the list whatever the scope is: they are about you, not about a folder.

They are **off by default** (a host's search costs API quota). While a connected code host has none of
its lists on, the top of the list shows a line for it:

`GitHub: list the requests waiting for your review, in repositories you have not cloned too  [Turn on]`

Click **Turn on**, press `Enter` on that line, press `T`, or run `Radar: Turn On Account-Wide Review
Requests...`: it writes `integrations.radar.review_requests = true` for that profile in `config.toml` and
asks the host at once. The other lists (every open request of your namespaces, issues assigned to you,
drafts, namespaces) are set with `gitpoulpi hosting integrations radar` or in the profile's settings; see
the [config reference](config-reference.md) and the [CLI reference](cli-reference.md). GitHub, GitLab,
Gitea, Forgejo, Azure DevOps, Bitbucket Data Center and Gerrit answer; Bitbucket Cloud has no account-wide
endpoint, so its requests show only for the linked repositories of the scope.

The profile's **Namespaces** limit all of its lists to some repositories (Settings > Integrations > Radar: a text area with one entry per line, and a **Regular expressions** checkbox). A line matches a repository's full path when it equals it or is a path prefix ending at a `/`: `orbit` keeps `orbit/products/x` and drops `orbitx/y` and `training/y`. With the checkbox on, each line is a regular expression run against the full path as you write it (`^orbit/(products|infra)/`); a line that is not valid is shown under the field and not saved. Empty means no limit.

How the rows are placed:

- A row that the scope already lists (same host, project, kind and number) appears once: **the scope's
  row wins**.
- A row whose project you have cloned somewhere (recent repositories, workspaces, the folders the folder
  scopes found; matched on the remote URL) carries that clone: **Enter**, check out, worktree and the
  others open it like any row.
- Any other row is **not cloned**: the repository chip is dimmed and shows the project (`acme/app`), and
  the status line says `Not cloned` when it is selected. **Open in browser** works, **Review** asks how
  to review it (below), and check out, worktree, show in repository and start work say
  `clone the repository first`.

### Reviewing a request

**Review** (`e`) on a request of the scope's own repositories opens that repository's tab with the review
in it. On a request of an account-wide row the way depends on whether a clone is known and on the
profile's **Request of a repository that is not cloned** setting (Settings > Integrations, `review_mode`
in [the config reference](config-reference.md)):

| Setting | A clone is known | No clone is known |
|---|---|---|
| Ask (default) | asks: **Open the local clone** (`l`) or **Review through the API** (`a`) | asks: **Clone it, then review** (`c`) or **Review through the API** (`a`) |
| Clone it | opens the clone | opens the Clone dialog |
| Review through the API | reviews through the API | reviews through the API |

The question is a small dialog: the letters pick, **Enter** picks the highlighted row, `Esc`, **Cancel**,
the `✕` and a click outside close it, and **Remember for GitHub** (`r`) writes the profile's setting so the
question is not asked again (the Settings row changes it back). `Ctrl+N` badges its controls.

- **Review through the API** opens the review in the Home window, over the Radar, with the code taken from
  the host (see [Reviewing without a clone](review.md#reviewing-without-a-clone)). `Esc` or the `✕` gives
  the Radar back; the palette's *Review: ...* rows work on it.
- **Clone it, then review** opens the Clone dialog on the connected account the row came from (the same list of repositories,
  grouped by owner), with the request's repository selected and the default clone folder filled in; change
  either, then clone, with the same sign-in as any clone from that account. If the repository is not in the
  account's list (you are not a member of it), the list stays open with the search on its name and a note says
  so; for a host that cannot list repositories, the dialog opens on **Clone with URL** with the address filled in. When the clone has landed, the new repository is linked to the host
  (its PULL REQUESTS source, and its ISSUES source when the host has issues and none is set), its tab opens and
  the review of that request is open in it. Canceling the dialog, or a clone that fails, links nothing.
- Azure DevOps and Bitbucket requests have no in-app review: **Review** opens the request's page in the
  browser and says so on the status line.

The palette rows *Radar: Review Through the API* and *Radar: Clone and Review...* skip the question.

Two checkboxes next to the filter (they appear once an account-wide list is being asked), kept in
`radar.toml`:

| Checkbox | Key | Default | Effect |
|---|---|---|---|
| `[x] Not cloned` | `a` | on | Lists the rows of repositories with no known clone. Off, only rows of known clones stay. |
| `[ ] Drafts` | `d` | off | Lists draft requests of the account-wide rows. Drafts of the scope's own repositories are always listed. |

Both are on `Ctrl+N` badges and in the palette (`Radar: Show Not-Cloned Requests`, `Radar: Show Drafts`).

A profile's answer is kept in `radar-account.json` for **5 minutes**: opening the Radar again within that
time shows the rows without asking the host. **Refresh** (`r` or `F5`) always asks: every repository and every account-wide list, whatever the age of the last answer. A host that said to slow down
or refused the token is not asked again in the same round. At most 300 rows are kept per profile and kind,
and a line under **Repositories** says how many more existed, or why a profile could not be read
(**Reconnect** opens Settings > Integrations).

## Limits

- Account-wide rows come from GitHub, GitLab (including self-managed), Gitea, Forgejo, Azure DevOps,
  Bitbucket Data Center and Gerrit profiles (not Bitbucket Cloud). Issues assigned to you come from the first
  five; Bitbucket and Gerrit have no issues.
- The per-repository lists use GitHub, GitLab (including self-managed), Gitea, Forgejo, Azure DevOps,
  Bitbucket and Gerrit profiles; checks, conflicts and mentions are read for the first four only.
- GitLab mentions only come from pending to-dos; GitHub, Gitea and Forgejo checks are asked for 15 requests per repository.
