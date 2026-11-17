**English** · [Čeština](cs/pull-requests-and-issues.md) · [Deutsch](de/pull-requests-and-issues.md) · [Español](es/pull-requests-and-issues.md) · [Français](fr/pull-requests-and-issues.md) · [Italiano](it/pull-requests-and-issues.md) · [日本語](ja/pull-requests-and-issues.md) · [한국어](ko/pull-requests-and-issues.md) · [Nederlands](nl/pull-requests-and-issues.md) · [Polski](pl/pull-requests-and-issues.md) · [Português (Brasil)](pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](pt-PT/pull-requests-and-issues.md) · [Русский](ru/pull-requests-and-issues.md) · [Svenska](sv/pull-requests-and-issues.md) · [Türkçe](tr/pull-requests-and-issues.md) · [Українська](uk/pull-requests-and-issues.md) · [简体中文](zh-CN/pull-requests-and-issues.md)

# Pull requests and issues

Reading, creating and managing pull / merge requests and issues from the sidebar, the board and the toolbar. Connect an account and link a repository first: see [Hosting and issue-tracker integrations](hosting-integration.md). Everything said about one issue or request (comments, reviews, threads) is in [Conversations](conversations.md); reading and reviewing a request is in [Reviewing a pull request](review.md).

## Pull requests and issues in the sidebar

### PULL REQUESTS

The section has a header with the count and a `+` (create a pull request),
the provider and project dropdowns, a search box and a state dropdown (*Open*,
*Merged / closed*, *All*). Then come four groups, each with its count:

- **My Pull Requests**: you are the author.
- **Assigned to me**.
- **Awaiting my review**: you are a requested reviewer.
- **All Pull Requests**: folded at the start.

The search box filters every group (title, number, author, branch). Every
change of the state dropdown asks the host again.

A row starts with a state glyph: `●` open (green), `○` draft, `◆` merged
(purple), `✕` closed (red). The request of the checked-out branch is bold.
Rows read `#12  title` (GitHub, Bitbucket) or `!53  title` (GitLab,
Azure DevOps).

### ISSUES

The section title names the provider: `GITHUB ISSUES`, `JIRA ISSUES`, and so
on. It reads `... WORK ITEMS` for Azure DevOps and `... CARDS` for Trello.
Below the dropdowns: a search box, the state dropdown (*Open*,
*Assigned to me*, *Closed*, *All*) and the issues. A Jira row reads
`ABC-12  title`, with the type and the status dimmed when the sidebar is wide
enough.

![The ISSUES section of the sidebar with the right-click menu of issue #7 open, Open conversation first](images/issues-sidebar.png)

### Actions on a row

- `Enter` or a double-click opens the row: a pull request in the [review](review.md) (GitHub, GitLab, Gitea,
  Forgejo, Gerrit), an issue, and a request on Azure DevOps or Bitbucket, as a [conversation](conversations.md)
  in the window, and anything else in your browser. The row menu's **Open in browser** always opens the browser.
- Right-click, or `.`, opens the row menu:
  - **Open conversation** (issues, and requests that have no review view), **Open in browser**, **Copy URL**,
    **Copy number**.
  - For a pull request: **Edit on <provider>**, **Check out source branch**,
    **View source-branch commits**. For a request from a fork, the check-out row reads
    **Check out pull request (fork)**, and **View** is grayed out.
  - For an issue: **Create branch from issue**. It opens the new branch prompt
    with a name such as `12-fix-the-thing` (or `ABC-123-fix-the-thing` for
    Jira).
  - On GitHub, GitLab, Gitea and Forgejo, a pull request row also has **Review...**
    (also **Enter** on the row): the in-app review, see
    [Reviewing a pull request](review.md).
  - On GitHub, GitLab, Gitea and Forgejo, both rows also have **Edit assignees...**,
    **Edit labels...** and **Set milestone...** (and **Edit reviewers...** on
    a pull request): see
    [Assignees, reviewers, labels and milestone](#assignees-reviewers-labels-and-milestone).
- **Edit on <provider>** has three other ways in: the pencil `✎` at the right
  end of a hovered or selected pull request row, `e` on the selected row, and
  the palette command *Integrations: Edit Linked Pull Request on the Host*
  (for the request of the checked-out branch).
- If the source branch is not in the repository yet, GitPoulpi says so. Fetch
  first (`Ctrl+G f`).

**Checkout of a pull request.** GitPoulpi fetches the request into a local
branch `pr/<n>` (`mr/<n>` on GitLab) that tracks it, and switches to it. On
GitHub, GitLab, Gitea and Forgejo this works for forks too (GitPoulpi fetches the special head
ref of the host). Azure DevOps and Bitbucket fetch the source branch of the
request. A request from a fork is refused there: add the fork as a remote and
check out its branch yourself. Undo returns to the previous branch. CLI:
`gitpoulpi hosting checkout-pr <n>`.

**Reviewing a pull request from the command line (and an agent).** On GitHub
(including Enterprise), GitLab (including self-managed), Gitea and Forgejo a request can be
read and reviewed without leaving GitPoulpi: `hosting review show <n>` (files,
merge state, checks), `hosting review threads <n>` (review threads and
conversation), `hosting review comment`, `hosting review reply`, `hosting review resolve`
and `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. The diff comes from your repository, not the
host: `hosting review fetch <n>` fetches the request's head into
`refs/gitpoulpi/pr/<n>` and its target branch into `refs/gitpoulpi/pr/<n>-base`
(neither is a branch, nothing is checked out). Line numbers are those of that
diff: `new` is the line at the head, `old` the line at the merge base. GitLab
has no "request changes" review state: it is refused there, with nothing sent.
Gitea and Forgejo take a comment on one line only (no ranges); a thread is the
comments on one line of a file. Gitea can resolve a thread, Forgejo 11 cannot
(the host has no endpoint for it: the call says so). A reply on a server
without the reply endpoint is posted as a one-comment review on the same line.
Gitea refuses to approve or reject your own request.
Without a clone: add `--project owner/name` (GitLab: `group/project`) to `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` and `suggest` to review a project that is not the current repository,
`hosting review diff <n> --project P` reads the diff (hunks per file) from the host's API, and
`hosting file <path> --project P [--ref R]` reads a file at a revision (GitHub, GitLab, Gitea, Forgejo and Gerrit; on Gerrit the diff of a
patch set is read file by file and the file comes from the branch or, for a commit id, the commit). A file the host does
not show in the diff (too big) is marked `omitted`.
The MCP tools are the same commands (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Assignees, reviewers, labels and milestone

On GitHub (including Enterprise), GitLab (including self-managed), Gitea and
Forgejo, you can change who a pull request or issue is assigned to, who reviews it, its labels
and its milestone without leaving GitPoulpi. Other providers show none of it.

- **Row menu**: **Edit assignees...**, **Edit reviewers...** (pull requests
  only), **Edit labels...**, **Set milestone...**.
- **Palette**: *Hosting: Edit Assignees...*, *Hosting: Edit Reviewers...*,
  *Hosting: Edit Labels...*, *Hosting: Set Milestone...* each first list the
  requests and issues of the sidebar to choose from. In the
  [Radar](radar.md) the same four act on the selected row.
- The picker opens at once and loads the project's people or labels (and what
  the item has now) from the host. A spinner row shows after about 150 ms.
  Type to filter (a fuzzy match), `Up` / `Down` move, `Space` checks or
  unchecks, `Enter` applies, `Esc` cancels, `Tab` reaches the buttons. A click
  on a row checks it, and the wheel scrolls. People show an initials badge,
  labels their color.
- The milestone is a dropdown (with **No milestone** to clear it).
- After **Apply** the host is updated, and the lists and the Radar show the
  change after their next refresh, which starts right away.

The create forms (a pull request, an issue) have the same choices as a row of
buttons: `Assignees: none`, `Reviewers: none`, `Labels: none`,
`Milestone: none`. On GitHub, GitLab, Gitea and Forgejo the Labels button replaces the typed
labels field of the issue form. If a host takes the new pull request but
refuses some of its labels or people, the status line says so.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>`, and `--assignee`, `--reviewer`, `--label`,
`--milestone` on `hosting pr` and `hosting issue create`; see the
[CLI reference](cli-reference.md).

### Keys in the sections

| Key | Action |
|---|---|
| `Enter` / double-click | open the conversation in GitPoulpi where the host has one (a pull request on GitHub, GitLab, Gitea or Gerrit opens the review), else the browser |
| `o` | open the row in the browser |
| `.` / right-click | row menu |
| `e` | edit the selected pull request on the host |
| `n` | new pull request (PULL REQUESTS) or new issue (ISSUES) |
| `v` | open the state dropdown |
| `r` | refresh |
| `Enter` / `Space` on a header | fold or unfold the section |

### Reconnect and offline

When the token is missing, rejected or lacks a scope, the first row of the
section reads `Reconnect (token rejected)`. Click it, or press `Enter`, to open
the profile in Settings > Integrations.

When the host cannot be reached, the last lists stay on screen and the header
shows their age (`· 5m old`). The last answer is also cached on disk
(`.gitpoulpi/hosting-cache.json`), so the lists show before the network
answers.

## The issue board

The ISSUES header's **Board** button (palette: **Issues: Open Board**) shows the issues of the linked source as a Kanban
board over the window. The columns depend on the source:

| Source | Columns | Moving a card |
|---|---|---|
| Local Markdown notes | the statuses (`statuses` of the profile, then any other in use) | rewrites the note's `status` |
| GitHub, GitLab, Gitea, Forgejo | **Open**, then the labels you chose, then **Closed** | swaps the status label; to **Closed** it closes the issue, to a column again it reopens it |
| Trello | the board's open lists, left to right | moves the card to the list |
| Redmine | the issue statuses | changes the issue's status (Redmine's workflow rules apply) |
| Jira (Cloud and Data Center) | the project's statuses, to-do first, done last | runs the workflow transition that leads to the column (`GET` then `POST .../transitions`); with no such transition the card goes back and the status line says `no transition from X to Y` |
| Azure DevOps (Services and Server) | the states of the project's work item types, by state category | sets `System.State`; a work item whose type lacks the column's state goes back, with the reason (`a Bug has no state Done`) |

On a GitHub or GitLab project, **Columns** (or `c`, or **Board: Set Column Labels...**) asks for the labels that are the
columns, in order, for example `status::todo, status::doing, status::review`. It proposes the labels of the project that
look like statuses. The choice is saved for the repository. With no labels the board is Open / Closed. Moving a card
only touches the status labels you listed: its other labels stay. A host board lists the open issues and the 60 most
recently closed ones.

A Jira transition that asks for required fields (a resolution, say) is refused by Jira and the card goes back with
Jira's own message: do that one on the site.

The same board is available to scripts and agents: `gitpoulpi hosting board` prints the columns with their cards and
`gitpoulpi hosting board move <card> <column>` moves one (see [the CLI reference](cli-reference.md)).

A card shows the number, the title, the assignees as initials, the labels and the milestone. Drag it to another column,
or `Shift+Left` / `Shift+Right`, or **Board: Move Card To...** (`m`). It moves at once; after about 150 ms a spinner
says the host is still working, and if the host refuses, the card goes back and the status line says why. `Enter` opens
the issue's [conversation](conversations.md) in GitPoulpi, and the board comes back when you close it; `e` edits assignees, labels and milestone; `n` starts a new issue in
the selected column; `a`, `l`, `M` and `/` filter by assignee, label, milestone and text, `x` clears the filters; `r`
reads the host again. The board reads again when the sidebar's lists refresh. The Radar has no board.

## Create a pull request

Open the form with the `+` in the PULL REQUESTS header, `n` in the section, or
the palette command *Pull Requests: Create Pull Request...*. A branch menu
also has **Start a pull request from <branch>...**.

The form has:

- One tab per connected profile that has pull requests (`Left` / `Right` on
  the tab strip).
- **From repo / Branch** and **To repo / Branch** comboboxes. Long lists are
  searchable. You can type a branch or repository that the lists do not show.
- **Title**, filled with the subject of the tip commit.
- **Description**, on several lines (`Enter` adds a new line, `Tab` leaves the
  field).
- **Assignees**, **Reviewers**, **Labels** and **Milestone** buttons (GitHub,
  GitLab, Gitea and Forgejo), which open the picker.
- **Submit as draft**.
- With [AI](ai.md) on: **Generate title and description**.

The form opens on the profile, repository and branches of the checked-out
branch (`feature -> main`). A different source repository makes a request
from a fork.

**Create Pull Request** (or `Ctrl+S`) creates the request on the host. The
result opens in your browser and the status line names it.

## Create an issue

When the ISSUES section has a provider **and** a project, its header has a
`+` (*+ New issue*). Also: `n` in the section, or the palette command
*Hosting: New Issue...*.

The form has **Title** (required), **Description** (several lines) and
**Labels** (comma-separated, for example `bug, ui`; on GitHub, GitLab, Gitea and Forgejo the
Labels, Assignees and Milestone buttons open the picker instead). Jira and Azure DevOps
add a **Type** dropdown. Trello adds a **List** dropdown. Linear and YouTrack
add a **Status** dropdown (the workflow state, or the `State` value, the issue
starts in). Redmine adds a **Type** dropdown with the project's trackers (Bug, Feature,
Support...); it has no labels, so there is no Labels button for it and a label
given on the command line is refused. The form fills these
dropdowns from the host when it opens.

**Create issue** (or `Ctrl+S`) creates it on the host. `Esc` or **Cancel**
closes the form. If the host refuses (rejected token, missing scope, unknown
project, unknown label), the form shows the reason and keeps what you typed.
On success the status line names the new issue and the ISSUES section is
refreshed.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## Create on the host's own web page

The New Issue form and the Create Pull Request form have a
**Create on <provider>** button, top right. The key is `Alt+W`. It opens the
web form of the host in your browser, filled with what you typed so far. The
dialog stays open. It needs no token and makes no API call, so it also works
after an API error.

- GitHub has no URL parameter for *draft*, so the draft box is not carried
  over. GitLab gets a `Draft: ` title prefix.
- GitLab takes labels by id only, so they are added as a `/label ~"x"` quick
  action at the end of the description.
- A very long description is cut so that the URL stays under 6000 characters,
  with a `[...truncated: paste the rest]` note.
- Bitbucket Data Center has no web form address, so it has no
  **Create on <provider>** button.

CLI: `gitpoulpi hosting issue create ... --web` and `gitpoulpi hosting pr --web`
print the same URL.

## The toolbar chip

At the far end of the toolbar, a chip shows the request of the checked-out
branch: `GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) or
`BB #12` (Bitbucket). Its color shows the state: green open, gray draft,
purple merged, red closed.

- Hover it for the title, the state and `source → target`.
- Click it, or press `Ctrl+G g`, to open the request in the browser.
- Right-click it for **Open in browser**, **Copy URL**, **Refresh** and
  **Switch profile**.

A pushed branch without a request shows a dim `+ PR` chip. Click it to start a
pull request. There is no chip when no profile matches, when the branch is not
pushed, or when the host could not be reached. On a narrow window the chip is
hidden to keep room for the toolbar buttons.

## When the lists refresh

The lists refresh:

- when you open a repository;
- when the checked-out branch changes;
- after a push or a fetch;
- every *Pull Requests & Issues Refresh (minutes)*
  (`general.hosting_refresh_minutes`, default 5, `0` turns the timer off;
  Settings > Integrations);
- when you close Settings;
- when you press `r`, run *Integrations: Refresh Pull Requests and Issues*, or
  change a filter.

Requests never block the window.

Hosts limit how many calls you can make. GitPoulpi sends the `ETag` of the
last answer back, so an unchanged list costs little (nothing on GitHub). When a
host reports that few calls are left, the status line says so. When none are
left, GitPoulpi stops asking that host until the limit resets, and the lists
stay as they are.

## Terminology

GitHub, Azure DevOps and Bitbucket say "pull request". GitLab says "merge
request". *Review request terminology* (`ui.pr_terminology`, on
Settings > Integrations) chooses the word for every label, menu, palette
command and message:

- **Auto (per host)**: follows the linked provider of the repository ("Pull
  Request" while nothing is linked).
- **Pull Request** or **Merge Request**: always that word.

The change applies at once. The CLI and MCP always say "pull/merge request".
This page says "pull request".

## References in commit messages and branch names

When the issues or pull requests of the repository are linked, references in
the commit message of Commit details are underlined. A click opens the linked
page. Keyboard: the palette command
*Integrations: Open Commit Reference (#issue / !request)...* (a menu when the
message has several references). A repository with nothing linked has no
links.

What counts as a reference depends on the provider of the ISSUES section:

| ISSUES linked to | In a message | In a branch name |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` and `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, only when `ABC` is the key of the linked project. Text such as `UTF-8`, `SHA-256` or `ISO-8601` is never a link. | `ABC-123` anywhere in the name (`feature/ABC-123-x`), in upper case |
| Linear | `ENG-123`, only when `ENG` is the key of the linked team (or of a team in the profile's team list). `UTF-8` is never a link. | Linear's own `branchName` when you create the branch from an issue; `ENG-123` anywhere in the name is read back |
| YouTrack | `ABC-123`, only when `ABC` is the short name of the linked project (or of one in the profile's project list) | `ABC-123` anywhere in the name |
| Redmine | `#123`, only when Redmine is the linked ISSUES source (a pull request source keeps its own sigil) | `123-fix-login` |
| Trello | a `https://trello.com/c/<shortLink>` address; `#12` only when a cached card has the number 12 | `12-fix-login` |

| PULL REQUESTS linked to | In a message |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

When both lists match `#N`, the issue wins. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**In conversations.** The text of an issue or request is read with the grammar of its host, which is wider than the
table above: `owner/repo#7`, `!7`, `GH-7`, `@login`, commit ids and the other forms listed in
[Cross references](conversations.md#cross-references). The repository's linked sources are matched first, so a Jira key in a GitHub
comment still opens its issue in the app. `gitpoulpi hosting markdown --body TEXT --id I` shows what the app finds in
a text.

### Insert an issue reference

In the commit composer, the `#▾` button at the right end of the Summary opens
a filterable list of the linked issues. The issue named by the current branch
comes first, marked *(from branch)*. `Enter` or a click inserts its reference
(`ABC-123`, `#12`, `AB#12`) at the caret. Keyboard: `Alt+R` in the composer.
Palette: *Commit: Insert Issue Reference...*. Without a linked issue list, the
button is grayed out and says *link an issue list in the sidebar first*.
GitPoulpi never adds a reference by itself.
