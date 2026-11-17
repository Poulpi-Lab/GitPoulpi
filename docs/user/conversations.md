**English** · [Čeština](cs/conversations.md) · [Deutsch](de/conversations.md) · [Español](es/conversations.md) · [Français](fr/conversations.md) · [Italiano](it/conversations.md) · [日本語](ja/conversations.md) · [한국어](ko/conversations.md) · [Nederlands](nl/conversations.md) · [Polski](pl/conversations.md) · [Português (Brasil)](pt-BR/conversations.md) · [Português (Portugal)](pt-PT/conversations.md) · [Русский](ru/conversations.md) · [Svenska](sv/conversations.md) · [Türkçe](tr/conversations.md) · [Українська](uk/conversations.md) · [简体中文](zh-CN/conversations.md)

# Conversations

The conversation view of an issue, work item, card or pull / merge request. It needs an account and a linked repository: see [Hosting and issue-tracker integrations](hosting-integration.md); the lists that open it are in [Pull requests and issues](pull-requests-and-issues.md); reviewing code is in [Reviewing a pull request](review.md).

A **conversation** is everything said about one issue, work item, card, change or pull / merge request: the
description, the comments, the reviews, the threads on code and the events (labels, assignments, state changes,
pushes). GitPoulpi reads it from the host, shows it formatted, and lets you answer without leaving the window.

### Opening one

| From | How |
|---|---|
| An ISSUES row | `Enter` or a double-click; the row menu's **Open conversation**; palette *Hosting: Open Issue Conversation...* (a list of the issues on screen). The row menu's **Open in browser** keeps the browser. |
| A PULL REQUESTS row on Azure DevOps or Bitbucket | `Enter` or **Open conversation**. On GitHub, GitLab, Gitea, Forgejo and Gerrit the row opens the [review](review.md), whose Overview page is this same conversation (the timeline, the keys and the comment box below). |
| A card of the [issue board](pull-requests-and-issues.md#the-issue-board) | `Enter` or a double-click. |
| The [Radar](radar.md) | `i` on an issue or request row, the row menu's **Open conversation**, or *Radar: Open Issue Conversation*. |
| A reference in another conversation | Click it (or `l`, then pick it): `#7`, `!12`, `ABC-12` of a linked source open in the app and push onto a Back stack. |
| An agent | `gitpoulpi ui show issue <n> --view conversation` (also `ui show pr`), see [the CLI reference](cli-reference.md). |

**Repositories that are not cloned.** A Radar row of a repository you have no clone of opens its conversation too:
the view needs only the integration profile and the project, so the Radar hosts it over its own window, and `Esc`
gives the Radar back. What needs a clone is left out: a commit reference opens on the host instead of in the Log, and
*Let the AI read the repository* is grayed with its reason (see [the AI chapter](ai.md#writing-help-on-a-host)). A
request of a cloned repository has its review instead (`e`), and the Radar's **Open conversation** says there is no
conversation to read here for it; an issue of a cloned repository opens it in that repository's tab.

An issue of a host that has none to read (Bitbucket and Gerrit have no issues) opens the browser as before, and a build
without the `conversation` feature does the same everywhere ([custom builds](custom-builds.md)).

### The view

A full-window view: the header (state, who opened it and when, the labels, the number of comments, **Show events**,
**Back**, **Refresh**, **Open in browser**), the **Posting as** line, the timeline, and under it the comment box.

![The conversation of issue #7: the description with a link to #9, two comments, a label event shown by Show events, and the empty New comment box with Write, Preview and Split](images/issue-conversation.png)

![The Overview page of merge request !58: the description, an unresolved and a resolved code thread with their file and line links, the comments, Reply and Quote on the selected thread, and the comment box](images/pr-conversation.png)

- The timeline reads oldest to newest and starts on the newest window (at most 300 entries). **Load older** at its
  top (`m`, palette *Conversation: Load Older Entries*) reads the window before, with the number still to come when
  the host says it.
- A comment shows its author, date, an `edited` chip, a `bot` chip for bots, `internal` for private notes, and the
  buttons **Reply**, **Quote**, **Edit**, **Delete** and **Links** that the selected entry allows. A thread on code
  shows `path:line`, `resolved` or `unresolved`, `outdated`, and its comments. A review shows its verdict.
- Events are one muted line each (*carol added the label lfs*). **Show events** (`v`) hides them; the choice is kept
  per repository. Hosts with no event history (YouTrack, Azure DevOps work items, local notes) have no such checkbox.
- The text is rendered Markdown: headings, nested and task lists, quotes, GitHub-style alerts, highlighted code,
  tables, links, images as `[image: alt]`. Text that arrived in the host's own markup is converted first (see
  [Markup conversion](#markup-conversion)). A `mermaid` code block gets an **Open diagram** button that shows the
  diagram's source in a scrollable viewer (`Enter` copies it); the palette has *Conversation: Open Diagram of Entry*.
  It needs a build with both `conversation` and `mermaid`.
- In the review's Overview a code thread's `path:line` is a link to the code (`G`, *Conversation: Open Code of Thread*;
  see [Review](review.md)); in a conversation opened on its own (an issue, a Radar row) there is no code page to go to.
- The view reads again every 60 seconds and on `F5` (**Refresh**).

**Keys.** Each has a button, a menu row or a palette row (*Conversation: ...*), and `Ctrl+N` badges the buttons.

| Key | Action |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Previous / next entry |
| `PageUp`, `PageDown`, `Home`, `End`, the wheel, the scrollbar | Scroll |
| `Tab`, `Shift+Tab` | Timeline, comment box, then the header buttons |
| `r` | Reply: into the thread of the selected entry, or with its text quoted where the host has no threads |
| `Q` | Quote the selected entry in a new comment |
| `c` | New comment |
| `e`, `d` | Edit, delete one of your own comments (a question comes before a delete) |
| `l`, `Enter` | The links of the entry: `Enter` follows the only one, or opens the menu of them |
| `y`, `o` | Copy the entry's address, open it in the browser |
| `n` | Next unread entry |
| `v`, `m`, `F5` | Show or hide events, Load older, Refresh |
| `Alt+Left`, `Backspace` | Back along the Back stack |
| `q`, `Esc` | Close (asks first when the box holds text) |

### Writing

**New comment** (`c`, or `Tab` to the box), **Reply** (`r`), **Quote** (`Q`) and **Edit** (`e`) use the one Markdown
composer: **Write**, **Preview** and **Split** (`Alt+P` swaps Write and Preview, `Alt+L` turns Split on and needs a
window 100 columns wide). `Ctrl+Enter` (also `Ctrl+S` and `Alt+Enter`) or **Send** posts, **Cancel** drops the box, and
`Alt+E` opens **Write with AI** where AI is on ([the AI chapter](ai.md#writing-help-on-a-host)). The preview is
laid out by the same renderer as the timeline, with the same references. When the host does not take Markdown the
footer names the markup the text is sent in (*Sent as Jira wiki markup*, *Sent as Textile*).

![An issue reply being written in the Split view: the Markdown draft on the left, its rendered preview on the right, and Send](images/issue-reply-composer.png)

- The comment is added to the timeline at once and takes its place when the host answers. If the host refuses, the
  entry goes away and the box opens again with your text and the reason.
- A text you have not sent is kept per item (`remote-conversations.toml` in the config folder) and is there the next
  time you open that box; sending or discarding it removes it. Closing a box that holds text asks first.
- **Reply** on a thread (a review thread, a GitLab discussion, a Linear comment, a Bitbucket or Azure DevOps thread)
  posts into that thread. Where the host has only a flat list, **Reply** starts a new comment with the selected entry
  quoted (`> **author**`), the same as **Quote**.
- **Internal note** (checkbox, `Alt+I`): on GitLab and Redmine the comment is visible to project members only. It is
  shown only on hosts that have such notes, and only on a new comment.
- Posting is an outward action: nothing is journaled and **nothing can be undone from GitPoulpi**. Without a
  connection the box stays, **Send** is grayed with the reason, and the header says the saved copy is shown.

**Who may edit or delete.** Only your own comments carry **Edit** and **Delete**. The host's own word comes first when
it gives one (GitHub's `viewerCanUpdate` and `viewerCanDelete`, `can_edit` elsewhere), then your login against the
comment's author, compared without regard to case; an App bot's comments must also come from a bot account. While the
login of a GitHub App bot is not known, only comments this install posted count (the repository keeps their ids):
nothing is guessed from a name. In a thread **Edit** and **Delete** act on the last comment of it that is yours. A host
that cannot edit or delete (see [the table](#what-each-host-cannot-do)) shows the buttons grayed with the reason.

### Posting as

The line under the header says who your messages are posted as, and the same line is in every box:

| Line | Meaning |
|---|---|
| *Posting as ann (personal token)* | The profile's token: your own login (`GET /user`). |
| *Posting as ci-bot (repository token command)* | The repository's [token command](hosting-integration.md#8-acting-as-a-github-app-bot) is used for the hosting API. |
| *Posting as my-app[bot] (GitHub App bot)* | An installation token. The name comes from **Post as**, else from the host. |
| *Posting as the GitHub App bot (its name is not known yet: set Post as in Settings)* | No name configured, none learned yet. |
| *Posting as ... (project bot)* | A GitLab project or group access token. |
| *Writing as kai (local note)* | A folder of Markdown notes: the profile's `local.me`, else the connected user. |
| *Who you would post as is not known yet. Set Post as in Settings.* | Nothing could be learned. |

**Post as** is a name: for a GitHub App it is `my-app[bot]`. Set it for every repository in **Settings > Integrations >
Post as** (`general.hosting_post_as`), or for one repository in **Settings > Repo-specific settings > SSH & credentials > Hosting**,
which overrides it. Left empty, GitPoulpi asks GitHub (GraphQL `viewer { login }`, which an installation token may or
may not answer) and, failing that, learns the name from the author of the first comment it posts and keeps it with
the repository. A personal token never needs it. See
[Acting as a GitHub App bot](hosting-integration.md#8-acting-as-a-github-app-bot).

### Cross references

References in a conversation are underlined and clickable. The repository's linked ISSUES and PULL REQUESTS sources
are matched first, so `ABC-12` of a linked Jira project opens in the app even inside a GitHub comment; what the
host's own grammar adds opens its page after the link confirmation. A commit id opens in the Log when the repository
is cloned and the id has at least 7 characters, on the host otherwise. Nothing inside a code span, a fenced block, a
Markdown link, an e-mail address or a URL is a reference, and `C#1`, `UTF-8`, `SHA-256` and `ISO-8601` are not.

| Host | Recognised |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, commit ids |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (a sibling project), `@user`, `~label` and `%milestone` as chips, commit ids |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, commit ids |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, `{uuid}` mentions, commit ids |
| Gerrit | `Change-Id` values (`I` and 40 hex digits), commit ids |
| Redmine | `#7`, `r123` revisions, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, only for a project or team of a linked source |
| Trello | card addresses (`trello.com/c/...`) |
| Local notes | `[[note]]`, `[[note\|alias]]` and relative `.md` links to another note of the folder |

`gitpoulpi hosting markdown --body T --id I` prints what the app makes of a text, references included.

### Markup conversion

Hosts that do not speak Markdown are converted both ways, so you read and write Markdown everywhere. What a
converter does not understand stays readable text.

- **Jira** (Cloud and Data Center) stores comments as wiki markup over API v2. What you read is converted to
  Markdown; what you write is converted to wiki markup, and the footer says *Sent as Jira wiki markup*.
- **Redmine** formats text as Textile or Markdown, which the API does not say: the profile's **Text format of the
  server** row (Settings > Integrations, `markup`, Textile by default) tells GitPoulpi. Textile is converted to
  Markdown for reading and back to Textile for writing.
- **Azure DevOps Server** returns work item comments as HTML and takes them as HTML: they are shown as the Markdown
  converted from it and written as HTML. Azure DevOps Services and pull request threads take Markdown.
- **Gerrit** is plain text: it is not rendered as Markdown and what you write is sent as typed.
- Every importer first removes control and escape sequences, so a comment cannot move the terminal.

### The saved copy, Load older, unread

- Each conversation you open is saved (`<state dir>/conversations/<integration>/<project>/<kind>-<n>.json`, user-only,
  no token in it, at most 2 MB a file, the 50 most recent files kept). It paints at once when the item opens and is
  refreshed behind it. When the host cannot be reached the saved copy stays on screen under *Offline: showing the
  copy saved 3 min ago (reason)* and Send is grayed.
- **Unread:** what you saw last is remembered per conversation (the latest 200 are kept). Entries by someone else
  that are newer get an accent bar and a `new` chip, and `n` jumps to the next one. A conversation you open for the
  first time has nothing new.

### What each host cannot do

Every `no` below is a real limit: the control is hidden or grayed with the reason, and the CLI answers an error naming
the host before anything is sent.

| Host | Items | Reply in a thread | Edit and delete own | Resolve a thread | Events | Internal notes | Written as |
|---|---|---|---|---|---|---|---|
| GitHub | issues, pull requests | pull requests | both | pull requests | yes | no | GitHub Markdown |
| GitLab | issues, merge requests | yes (discussions) | both | merge requests | yes | yes | GitLab Markdown |
| Gitea | issues, pull requests | pull requests | both | pull requests | yes | no | Markdown |
| Forgejo | issues, pull requests | pull requests | both | no (the host has no endpoint) | yes | no | Markdown |
| Azure DevOps | work items, pull requests | pull requests | both | no (status shown, not changed) | pull requests only | no | Markdown (HTML on Server work items) |
| Bitbucket (Cloud, Data Center) | pull requests only | yes | both | no (shown, not changed) | yes | no | Markdown |
| Gerrit | changes only | yes | no (neither can be edited or deleted) | yes | yes | no | plain text |
| Jira | issues | no (flat list) | both | no | yes (changelog) | no (needs a project role) | wiki markup |
| Linear | issues | yes (replies to a comment) | both | no | yes | no | Markdown |
| YouTrack | issues | no | both | no | no | no | Markdown |
| Redmine | issues | no | edit only (Redmine 5 and later), never delete | no | yes (journals) | yes (private notes) | Markdown or Textile |
| Trello | cards | no | both | no | yes (moves) | no | Markdown |
| Local notes | notes | no | the local author's own | no | no | no | Markdown |

Also: no host lets you start a new thread on a line from the conversation (do that in the review), and reactions are
shown on GitHub only. A text longer than the host takes is refused (GitHub 65,536 characters, GitLab 1,000,000, Azure
DevOps 150,000, Jira 32,767, Trello 16,384): the CLI checks first, in the app the host's own message is shown and your
text is kept. A local note keeps its comments in a final `## Comments` section of the note file itself, one
`### author - time` entry each, so Obsidian and a shell see them too.

On the command line the same conversations are `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit`
and `comment-delete`, and `hosting review conversation` (see [the CLI reference](cli-reference.md)); for agents they
are MCP tools of the same names (see [MCP and agents](mcp-and-agents.md)).
