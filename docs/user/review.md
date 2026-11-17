**English** · [Čeština](cs/review.md) · [Deutsch](de/review.md) · [Español](es/review.md) · [Français](fr/review.md) · [Italiano](it/review.md) · [日本語](ja/review.md) · [한국어](ko/review.md) · [Nederlands](nl/review.md) · [Polski](pl/review.md) · [Português (Brasil)](pt-BR/review.md) · [Português (Portugal)](pt-PT/review.md) · [Русский](ru/review.md) · [Svenska](sv/review.md) · [Türkçe](tr/review.md) · [Українська](uk/review.md) · [简体中文](zh-CN/review.md)

# Reviewing a pull or merge request

Review a pull request (GitHub, Gitea, Forgejo), a merge request (GitLab) or a Gerrit change (see [Gerrit](#gerrit))
**without leaving the repository's window**. The review view shows the description, the conversation, the changed
files with the comment threads under their lines and the commits, and it is where you write the review: comments,
approval and requests for changes. It needs a profile for one of those hosts (see
[Hosting and issue trackers](hosting-integration.md)) and a repository linked to it; other providers do not offer
reviews. A host's limits are respected: GitLab has no *Request changes* verdict and takes one line per comment.
This page calls all of them a *request*.

![The review view, Overview page: the request's header with its metadata buttons, the description, an unresolved and a resolved code thread, the comments and the comment box](images/review-overview.png)

![The Files page: the changed files with their viewed boxes, the diff with a comment thread under its line, and a resolved thread outside the lines shown](images/review-files.png)

## Opening a review

Every way in lands on the same view:

- the **PULL REQUESTS** list of the sidebar: select a row and press **Enter**, or right-click it and choose
  **Review...**;
- the **Radar**: the row's menu, **Review...**, or the `e` key (the repository opens and the review with it;
  a request of a repository you have not cloned asks whether to clone it or to review it through the API,
  see [Reviewing without a clone](#reviewing-without-a-clone));
- the **workspace search** (`Ctrl+Alt+F`): highlight a pull or merge request and press **`Alt+R`**;
- the palette: **Hosting: Review Request...** lists the requests to choose from.

The view fills the window over the repository (it is a mode of the repository tab, not a separate window).
**`Esc`** or the `✕` in its corner leaves it. While it opens, the request is read from the host and its code is
fetched with git in the background; a spinner appears if that takes more than a moment.

## The header

Always visible: the number and title, the state (Open, Draft, Merged, Closed), the branches
(`main ← feature/moons`), the author, the checks (`✓ checks passing`, `✗ checks failing`, `● checks running`),
whether it merges cleanly or conflicts, the size, and **`3 pending comments`** when you have written some.

The four metadata buttons - **Assignees**, **Reviewers**, **Labels**, **Milestone** - open the same pickers as
the sidebar (see [Assignees, reviewers, labels and milestone](pull-requests-and-issues.md#assignees-reviewers-labels-and-milestone));
the header shows the result as soon as the host has taken it. The action buttons: **Check out** (the usual
checkout of the request's branch, which also closes the view), **Open in browser**, **Refresh**
(**`F5`**), and **Finish review**.

## Three pages

A segmented control in the top border switches page: **Overview**, **Files**, **Commits**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`**, or a click).

### Overview

The Overview is **the conversation of the request**, the same timeline and the same comment box an issue has
([Conversations](conversations.md)): the description first (*opened this*), then the comments,
the threads on code with their diff excerpt, the submitted reviews with their verdict (`approved`, `requested
changes`, `commented`, `dismissed`; on GitLab the verdicts are its approvals) and the events, oldest first, on the
newest window with **Load older** at its top. The comment box is docked under the timeline (**New comment**):
**Write**, **Preview** and **Split**, **Cancel** and **Send**. **A comment posted here goes to the host at once**
(nothing is pending: pending comments are the ones on code, sent with the review), appears in the list straight away
and takes its place when the host answers; if the host refuses it, the box opens again with your text and the reason.

The keys are the conversation view's: **`Up`** / **`Down`** (and `Ctrl+J` / `Ctrl+K`) move between entries,
**`r`** replies into the thread of the selected entry, **`Q`** quotes it in a new comment, **`c`** writes a new
comment, **`e`** / **`d`** edit and delete your own comments (a question comes before a delete), **`l`** or `Enter`
opens an entry's links, **`g`** its diagram, **`y`** copies its address, **`o`** opens it in the browser, **`n`**
goes to the next unread entry, **`v`** shows or hides events, **`m`** loads older entries, `Tab` goes between the
timeline and the box, and **Internal note** (`Alt+I`) is there on the hosts that have such notes. The buttons of the
selected entry (**Reply**, **Quote**, **Edit**, **Delete**, **Links**) do the same, and the palette has the
*Conversation: ...* rows next to the *Review: ...* ones. A link is asked about before it opens, like everywhere in the
review.

A thread on code shows its `path:line` as a link (underlined, a real button). A click on it, **`G`** on the selected
thread, or the palette's *Conversation: Open Code of Thread* switches to the **Files** page, opens that file and puts
the cursor on the commented line (on the old or the new side, as it was written) with the thread open under it (a
folded one is unfolded). **Esc**, the **Back to Overview** button at the left of the page tabs, or *Review: Back to
Comment in Overview* return to the same entry of the Overview. A thread whose line is no longer in the diff (outdated)
opens the file at the nearest line and the status line says so; a thread on a file the request no longer changes says
that the file is not in the diff. In an issue or Radar view of the same request there is no code to open: `G` answers
that the review has it.

Where a conversation key and a review key meet, the conversation has the letter on this page: `l` (labels), `m`
(milestone), `Q` (Gerrit's abandon), `e` / `d`, `r`. The labels, milestone and abandon stay on the header's buttons and
in the palette; `a` (assignees), `p` (reviewers), `f` (**Finish review**), `b` (**Check out**), `P`, `D`, `U` and
`Alt+1` / `Alt+2` / `Alt+3` keep their meaning here. While the box has the keyboard every key is the box's, except
`Alt+1` / `Alt+2` / `Alt+3`; `Esc` gives the keyboard back to the timeline and a second `Esc` closes the review.

A comment written on another window or by the CLI shows here by itself: every write of GitPoulpi (this window
included) tells the session, and the windows showing the same request read it again.

The comment box is the Markdown composer shared with [the conversation view](conversations.md#writing)
(also the box of a reply to a thread, a comment on a line and the summary of **Finish review**): `Ctrl+Enter` (also
`Ctrl+S`, `Alt+Enter`) sends, and where AI is on `Alt+E` opens **Write with AI**
([the AI chapter](ai.md#writing-help-on-a-host)). The preview shows the text as the host will, references included
(`#7`, `!12`, `ABC-12`, commit ids); an unsent text is kept per request.

### Files

The changed files on the left (status letter, `+added -removed`, a `●n` count of open threads, a `✎n` count of
your pending comments) and the diff of the selected file on the right. The diff comes from git: the request's
head is fetched into `refs/gitpoulpi/pr/<n>` (never a branch, nothing is checked out) and compared with the
merge base, so the line numbers in the margin are exactly the ones comments use.

- **`[ ]` before a file**: mark it as *viewed* (click, or press **`v`** / `Space` on the list). The title counts them.
  Viewed marks are kept per repository and survive a restart.
- **Threads** sit right under their line: first comment, replies, and the buttons **Reply** and
  **Resolve** / **Reopen**. A resolved thread folds to one row; **`z`** (or `Enter` on it, or a click on the
  `▾`) folds and unfolds any thread.
- **Outdated threads**, and threads on lines the diff does not show, are listed after the hunks under
  *Comments outside the lines shown*, with an `outdated` badge when the code changed under them.
- **`c`** comments on the line the cursor is on (the small `+` at the start of the line does the same by
  mouse). **`Shift+Up` / `Shift+Down`** (or a drag over the lines) select several lines first and the
  comment covers the range. GitLab takes one line per comment, and the view says so instead of selecting.
- **`r`** replies to the thread under the cursor, **`x`** resolves it or reopens it. Both are sent to the host **at
  once**.
- Keys: `Up` / `Down` / `PgUp` / `PgDn` / `Home` / `End` move; `Tab`, `Right` and `Enter` go from the file list
  to the diff, `Left` and `Tab` back; `]` and `[` jump to the next and previous file.

### The diff's display options

The diff pane of **Files** and **Commits** is the [Diff view](staging-and-committing.md)'s own, with the same bar on its first
row and the same remembered choices (they are one setting: what you pick here is what the Diff view opens with,
and the other way round):

- **Inline / Split**: one column, or before and after side by side. In Split a thread, a pending comment or a
  suggestion sits under the half it belongs to: a comment on a removed line under the left half, a comment on an
  added or unchanged line under the right half. Click the half you mean before pressing **`c`** on a row that
  shows a removal and an addition; the keyboard comments on the new side.
- **Hunks**: only the changes with their context, or the whole file with the changes in place. In the whole file a
  thread on a line outside the changes sits under its line instead of in *Comments outside the lines shown*.
  The file's text is read in the background; the hunks stay on screen with a line saying so until it arrives (on
  the Commits page, up to 40 files of a commit are read this way).
- **WS**: show or ignore changes that only differ in whitespace.
- **Wrap**: wrap long lines.
- **Up / down arrows**: the previous and next hunk.
- **Map** (**`M`**): the minimap beside the scrollbar, one setting with the Diff view's. Click or drag on it to
  scroll.
- **Auto** (**`E`**, or *Review: Choose Encoding...*): the encoding the file is read in, as in the Diff view:
  Auto-detect or one of the encodings of the list, chosen for the file the pane shows (on the Commits page, the
  file under the cursor). Both sides of the file are read again in that encoding, in the background, and
  the choice is shared with the Diff view for the same path. A file that cannot be read that way says why above
  its diff.
- **`⋯`**: when the pane is too narrow for every button, the ones that do not fit are behind a `⋯` at the end of
  the bar; its list runs each as if it were clicked. Their keys and palette rows work as always.

The scrollbar's ruler shows where things are: the changes (whole file only, like in the Diff view), a dot `●`
for every thread, pending comment and suggestion, and the cursor.

Long lines are cut at the edge of the pane (turn **Wrap** on to see them whole) and scroll sideways: **`Right`**
and **`Left`** on the diff (`Left` at the left edge goes back to the file list), the sideways wheel or `Shift`
with the wheel, the scrollbar along the bottom border, or *Review: Scroll Diff Left / Right*.

### Selecting text and lines

Where a drag starts decides what it selects. A drag that starts **in the code** selects text, as in the Diff view
(double-click selects a word); **`Ctrl+C`** (or *Review: Copy Selected Text or Line*) copies exactly those
characters, without line numbers or markers, and without a text selection copies the line under the cursor. A
drag that starts **in the gutter** (the line numbers, the marker, the `+`) selects lines, to comment on them as
before, and a double-click in the gutter still starts a comment on that line.

The Diff view's keys `n`, `p`, `m`, `w`, `s` are the review's own, so on the review the toggles are capital
letters: **`V`** layout, **`H`** hunks or whole file, **`S`** whitespace, **`W`** wrap, **`J`** / **`K`** next and
previous hunk, **`E`** the encoding, **`M`** the minimap. Every one has a palette row (*Review: Toggle Inline / Split
Layout*, ...).

### Commits

The commits of the request only (not the history behind it), newest first, and the diff of the selected one
(its changes against its parent), with all of its files one under the other and the same display options.

## Writing a review

A comment on code is **pending**: it is kept on your machine (in the repository's state file, so it survives a
restart) and counted in the header. Nothing is sent until you finish the review. A pending comment shows
under its line with the badge `Pending - line 12` and two buttons: **`e`** (Edit) and **`d`** (Delete).

**Finish review** (**`f`**, the button, or the palette) opens a small form: a summary (optional) and a
**Verdict** list: *Comment*, *Approve*, *Request changes*. Only what the host allows is listed: GitLab has no
*Request changes*. **Submit review** (`Ctrl+S`) sends the summary, the verdict and every pending comment in
one go. **Discard review...** throws the pending comments away after a confirmation.

What is sent at once, without waiting for the review: replies, resolving and reopening, and comments on the
conversation. The comment box says which kind it is (*This comment is posted at once* or *This comment is added to your
pending review*). If the host refuses a reply, the box opens again with your words.

If the request got new commits after you wrote pending comments, the view warns that their lines may have
moved (GitHub and GitLab place a comment on the commit you read; a review of an older head is refused).

## Gerrit

A Gerrit server is connected like the other hosts (see [Hosting and issue trackers](hosting-integration.md): server
URL, user name, HTTP password) and linked under **PULL REQUESTS** to a *project*. Its changes are listed as pull
requests (`#7`, chip `GR`) and reviewed in the same view, with the differences Gerrit brings:

- **Patch sets.** A change has numbered patch sets (each upload of the commit is one). The header has a
  **`Patch set N`** combobox on the top border, next to the page tabs (**`P`**, or the palette row *Review: Choose
  Patch Set...*). It starts on the latest; picking another reads that patch set's files and code, and the
  threads written on other patch sets are listed as *outdated*. The review you finish is sent on the patch set
  that is shown.
- **Compare with another patch set.** A second combobox left of it, **`Compare: base`** (**`D`**, or *Review: Compare
  Patch Sets...*), lists the base and every patch set older than the one shown. Pick patch set 1 while patch set 3
  is shown and the Files page lists, and diffs, only what patch set 3 changed since patch set 1 (the server's own
  difference, so a rebase onto a newer target does not show up as changes); the Commits page shows that one
  difference. Both patch sets are fetched into `refs/gitpoulpi/pr/<n>` and `refs/gitpoulpi/pr/<n>-ps<k>`. Threads
  written on other patch sets than the one shown are *outdated*. Picking an older patch set to read than the
  compared one resets the comparison to the base. The CLI twin is `review show|fetch --compare-with K`.
- **Submit and abandon.** When the server says the change can be submitted (the votes and checks it needs are
  there), a **Submit** button joins the header's buttons; **Abandon** is there while you may abandon the change.
  Both (keys **`U`** and **`Q`**, palette *Review: Submit Change...* / *Review: Abandon Change...*) ask first, in
  a popup that says the action happens on the server and **cannot be undone from GitPoulpi** (a merged change
  stays merged; an abandoned one can be restored in Gerrit's web interface). Declining sends nothing; on success
  the view reads the change again. The CLI twins are `gitpoulpi hosting review submit-change <n>` and
  `hosting review abandon <n> [--message M]`.
- **Votes instead of a verdict.** **Finish review** shows one combobox per label you may vote on - `Code-Review`
  (-2 to +2) and `Verified` where the project has it - listing only the values the server permits **for this
  change and you** (a developer without the right cannot pick +2). It starts on your current vote; only a vote you
  changed is sent. The summary and the pending comments go in the same call. Drafts you saved in the Gerrit
  web interface are published with the review.
- **Threads** are Gerrit's comment chains. A thread is resolved when its newest comment says `unresolved: false`;
  **resolve** posts a reply `Done` that does so, **reopen** a reply `Reopened`. Replies and resolving go to the server
  at once, like on the other hosts.
- **Push for review.** Gerrit creates a change when you push to `refs/for/<branch>`. With the PULL REQUESTS
  section linked to a Gerrit project, the section's **`+`** button and the palette row **Hosting: Push for
  Review...** ask for the target branch and push `HEAD` to `refs/for/<branch>` (a topic can be given on the command
  line). The push cannot be undone from GitPoulpi (abandon the change in Gerrit); the message says so. The CLI
  twin is `gitpoulpi hosting review push`.
- The metadata buttons are read-only for Gerrit (the reviewers are shown), there is no *Request changes* verdict
  (a vote of -1 or -2 is the same thing) and the checks chip is the `Verified` label.

## ✦ AI review draft

![Two ✦ suggestions under their lines in the Files page, each with Accept, Edit and Dismiss, next to an existing thread](images/review-ai.png)

With AI turned on (*Settings > AI*, see [AI](ai.md)) the header has a **`✦ Draft review...`** button (also **`s`**,
and the palette row *Review: ✦ Draft Review...*). It opens the [review options](#review-options) and
**`Enter`** there asks the model. It is the only thing that asks the model: opening a review never does. The model reads the request's title and description, the comments that already exist (so that it
does not repeat them) and the diff, and answers with a **review draft**: the text to submit, the verdict it recommends (Approve, Request changes or Comment), an overall summary and a list of suggested comments. A
long diff is cut for the model - per file, and in total - and the Overview says how many files were shortened
or left out. While it works the status line shows a spinner after a moment and **`Esc`** (or the same button)
cancels it. The *Code review* instructions of the [four levels](ai.md#instructions) apply, so a team or a
workspace can say what a good review looks like.

### Review options

`s` opens the **✦ Review options** dialog, filled with the defaults of the repository (the global ones where the
repository has none). `Enter` (or the **Run** button) starts the run with what the dialog shows; `Esc`, **Cancel**
or the cross closes it and asks nothing.

| Control | Does |
|---|---|
| **Run the agent in the repository** | *Profile setting* (the AI profile's working folder decides), *Yes* or *No*. Greyed out, with the reason under it, for an HTTP profile (it has no working folder) and when the request is reviewed through the API without a clone. |
| **CLAUDE.md**, **AGENTS.md** | Give the model that file of the repository, read at the branch the request targets. When the agent runs in the repository it reads them on its own, and the dialog says so. |
| **Extra file** | One more file: a path in the repository, an absolute path, or a link to a file of a code host. **Browse...** (`Ctrl+B`, or `Enter` on the button) opens a file picker that starts at the repository root (or beside the file already named); a file inside the repository is written as a path relative to the root, one outside it as an absolute path. Typing in the field works as before. |
| **No extra file** | No extra file for this run, even when the global setting or the repository names one; typing a file or picking one with **Browse...** unticks it. With **Save as this repository's defaults** it becomes the repository's *No extra file* switch. An empty field counts as none here, because the dialog starts from what the run would inherit. |
| **Save as this repository's defaults** | Also keeps the values for the next reviews: in the repository's state file, or in `remote-reviews.toml` for a repository that is not cloned. Only a value that differs from the global one is kept, so the rest keeps following *Settings > AI*. |

Everything is reachable with `Tab`, `Space` toggles a checkbox and the path field is an ordinary text field. The
same four settings live in *Settings > AI* (*Code review*) and in *Repo-specific settings > AI*; see
[Review context](ai.md#review-context). What could not be given to the model (an extra file that is missing or
unreadable) and any file that was shortened to fit the limits are listed on the draft card.

Suggestions are **not comments**. They show in two places:

- **Under their line in Files**, as ghost comments marked `✦` with a severity (`info`, `warning`, `issue`) and
  three buttons: **Accept**, **Edit** and **Dismiss**. A suggestion about a line the diff does not show is
  kept, never dropped: it is listed after the hunks under *AI suggestions about the whole file* and, once
  accepted, lands on the file's first changed line with the words *(about line N)* in front.
- **In the Overview**, in a block above the conversation (only after a run): the summary, then the list with the same
  buttons and **Accept all** / **Dismiss all**.

**Accept** (`y`) turns a suggestion into an ordinary **pending comment** (you can still edit or delete it);
**Edit** (`w`) opens the comment box with the model's words first and accepts what you leave; **Dismiss** (`n`)
drops it. **Accept all** (`A`) still only fills the pending list. On the Files page the keys act on the
suggestion under the cursor, on the Overview on the first of the list (`y` and `n` are the conversation's *Copy link* and
*Next unread* again once the list is empty). **Use AI summary** (`i`, or the button
in the Finish review form) puts the model's summary in the review text; it is never inserted on its own.
**Nothing is sent to the host until you submit the review.**

The Overview shows the draft as a card at the top of that block, above the suggestions: the verdict chip, the review text, and three
buttons.

- **✦ Use draft...** (`u`) accepts every open suggestion as a pending comment and opens *Finish review* already
  filled with the draft's text and verdict. You still read it, change it and submit it yourself; cancelling the
  form keeps the draft. The host's limits are kept: GitLab has no *Request changes*, so the form says so and
  falls back to *Comment*; on Gerrit the verdict becomes a Code-Review vote of +1 or -1, only when the server
  permits that value for you (otherwise no vote is set and the form says so).
- **Edit text...** (`t`) changes the review text of the draft before you use it.
- **Discard draft** (`N`) throws away the text, the verdict and the remaining suggestions.

The draft is kept per request, so it is still there when you close and reopen the review. When the request's
head moved since the model wrote it, the card says *The request has changed since this draft was written.*
Submitting the review spends the draft.

## Everything has a key and a palette row

| Action | Key | Palette |
|---|---|---|
| Overview / Files / Commits | `Alt+1` / `Alt+2` / `Alt+3` | *Review: Show Overview*, *Show Files*, *Show Commits* |
| Comment on the line or the range | `c` | *Review: Comment on Line...* |
| Comment on the conversation | `c` on Overview, `C` | *Review: Comment on Conversation...* (opens the Overview's box), *Conversation: New Comment* |
| Open the code of a thread (Overview) / back to the comment | `G` / `Esc` | *Conversation: Open Code of Thread*, *Review: Back to Comment in Overview* |
| Reply to the thread | `r` | *Review: Reply to Thread...* |
| Resolve or reopen | `x` | *Review: Resolve or Reopen Thread* |
| Fold or unfold a thread | `z` | *Review: Fold or Unfold Thread* |
| Edit / delete a pending comment | `e` / `d` | *Review: Edit Pending Comment...*, *Delete Pending Comment* |
| Mark the file as viewed | `v` | *Review: Mark File as Viewed* |
| Inline / Split | `V` | *Review: Toggle Inline / Split Layout* |
| Whole file / only the hunks | `H` | *Review: Toggle Whole File / Only the Hunks* |
| Whitespace-only changes | `S` | *Review: Toggle Whitespace-Only Changes* |
| Word wrap | `W` | *Review: Toggle Word Wrap* |
| Next / previous hunk | `J` / `K` | *Review: Next Hunk*, *Previous Hunk* |
| Next / previous file | `]` / `[` | *Review: Next File*, *Previous File* |
| Finish / discard the review | `f` | *Review: Finish Review...*, *Discard Pending Review...* |
| Refresh | `F5`, `Ctrl+R` | *Review: Refresh* |
| Open in browser / check out | `o` / `b` | *Review: Open in Browser*, *Check Out Branch* |
| Assignees / reviewers / labels / milestone | `a` / `p` / `l` / `m` | *Review: Edit Assignees...* and its three siblings |
| Choose the patch set (Gerrit) | `P` | *Review: Choose Patch Set...* |
| Compare with an older patch set (Gerrit) | `D` | *Review: Compare Patch Sets...* |
| Submit / abandon the change (Gerrit) | `U` / `Q` | *Review: Submit Change...*, *Review: Abandon Change...* |
| ✦ Draft review... (AI on; opens the review options) | `s` | *Review: ✦ Draft Review...* |
| Accept / edit and accept / dismiss a suggestion | `y` / `w` / `n` | *Review: Accept Suggestion*, *Edit and Accept Suggestion...*, *Dismiss Suggestion* |
| Accept / dismiss every suggestion | `A` / `N` | *Review: Accept All Suggestions*, *Dismiss All Suggestions* |
| Use the AI summary in the review text | `i` | *Review: Use AI Summary in Review Text* |
| Use the ✦ draft (fills Finish review) | `u` | *Review: ✦ Use the Draft...* |
| Edit the draft's text | `t` | *Review: Edit the ✦ Draft Text...* |
| Discard the ✦ draft | `N` | *Review: Discard the ✦ Draft* |
| Close | `Esc` | *Review: Close* |

Open the palette (`Ctrl+P`) while the view is open: its rows work on it, and say why one does nothing right
now ("put the cursor on a code line first").

## Reviewing without a clone

A request of a repository that is not on your machine can still be reviewed. From the Radar, **Review**
(`e`) on its row asks how (see [The Radar](radar.md#reviewing-a-request)); **Review through the API** opens
this same view in the Home window, over the Radar. Everything on this page works the same, with these
differences:

- The code (the changed files, each file's diff, the request's commits, a commit's diff, the whole file) is
  read from the host's API, so the first look at a big request takes a moment, and a file the host does not
  send a diff for (a binary or a very big one) says so instead of showing one.
- Your pending comments are kept in `remote-reviews.toml` in the configuration folder, filed under the host
  and the project (`host/project`), instead of in a repository's own state. They are still there when you
  close and reopen the review, and sending the review spends them.
- There is no encoding choice (*Not available without a clone*) and no **Check out** (it says to clone the
  repository first).
- `Esc` or the `✕` in the corner leaves the view and gives the Radar back. The palette (`Ctrl+P`) reaches the
  view's rows as it does in a repository's tab.

To keep the clone instead, choose **Clone it, then review** (or the palette's *Radar: Clone and Review...*):
the Clone dialog opens on the connected account of the request with its repository selected, and the review
opens in the new repository's tab once the clone has landed.

## From the command line and for agents

The same review is available as `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest`
(and the MCP tools made from them): see [CLI reference](cli-reference.md). `threads` also lists the
verdicts already submitted (`reviews`). `suggest <n>` asks the model (AI on) and prints its `verdict`, `body`, summary and
suggestions as JSON without posting anything, so a script or an agent can pick the comments to pass to
`submit --comment`.

None of this needs a clone: `--project owner/name` (with `--id` when several integrations are configured)
runs `show`, `threads`, `comment`, `reply`, `resolve`, `submit` and `suggest` against a project the current
directory knows nothing about. `hosting review diff <n>` prints the diff with its hunks (`--api` reads it from
the host even inside a clone), and `hosting file <path> --project P [--ref R]` reads a file at a revision, for
example the project's `CLAUDE.md`. Both read the host's API: GitHub, GitLab, Gitea, Forgejo and Gerrit.
