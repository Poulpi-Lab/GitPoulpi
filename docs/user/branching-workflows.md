**English** · [Čeština](cs/branching-workflows.md) · [Deutsch](de/branching-workflows.md) · [Español](es/branching-workflows.md) · [Français](fr/branching-workflows.md) · [Italiano](it/branching-workflows.md) · [日本語](ja/branching-workflows.md) · [한국어](ko/branching-workflows.md) · [Nederlands](nl/branching-workflows.md) · [Polski](pl/branching-workflows.md) · [Português (Brasil)](pt-BR/branching-workflows.md) · [Português (Portugal)](pt-PT/branching-workflows.md) · [Русский](ru/branching-workflows.md) · [Svenska](sv/branching-workflows.md) · [Türkçe](tr/branching-workflows.md) · [Українська](uk/branching-workflows.md) · [简体中文](zh-CN/branching-workflows.md)

# Branching workflows

A branching workflow tells GitPoulpi how your team uses branches: which ones live
forever (`main`, `develop`, `production`), which are short-lived topic branches
(`feature/...`, `hotfix/...`), how they are named, and where each one starts and
ends. It is Git Flow made customizable: fixed names, prefixes and suffixes, or
patterns with placeholders, so the branches a bot creates from an issue are
recognized like your own.

This page covers the model, the file, the presets, detection and import, the
read-only commands, the actions that start, finish, sync and promote branches
(CLI, MCP and app), and the screens of the app: the settings editor, the grouped
sidebar, the `Flow` menu and its dialogs.

## Concepts

- A **base branch** is long-lived. A base may have a **parent** (`develop` into
  `main`); a trunk has none. A base can list several candidate names
  (`main` or `master`, `develop` or `dev`): whichever exists is used.
- A **topic type** is a kind of short-lived branch (feature, bugfix, release,
  hotfix, support...). It has a **name pattern**, a **parent** (where it starts
  and where it is finished into) and a way to finish.
- Every branch is **classified**: a base (by name), a topic of some type (by
  pattern), or unclassified.
- The **parent of a branch** is, in order: the one recorded when GitPoulpi created
  the branch (`git config branch.<name>.gitpoulpi-parent`), else Git Town's
  `git-town-branch.<name>.parent` when present, else the parent of its type.

## Where it is stored

| What | Where | Committed |
|---|---|---|
| The shared workflow | `.gitpoulpi/workflow.toml` in the repository | Yes, by you: GitPoulpi writes it only when you ask |
| Your own tweaks | `[workflow]` in `.gitpoulpi/repo-state.toml` | No (that file is ignored) |

The state directory ignores itself (`.gitpoulpi/.gitignore`), except
`workflow.toml`, which is meant to be committed. The effective workflow is the
shared file with your override on top: a base with the same name, or a topic with
the same id, is replaced; new ones are added; `disable_bases` and `disable_topics`
remove shared ones for you only.

Saving over an existing file keeps the keys this version does not know (a newer
GitPoulpi's, a teammate's), but not comments or key order.

## The file

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom (informative)

[[base]]
name = "main"
names = ["master"]             # other spellings of the same branch
protected = true               # never deleted, direct commits are suspect

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"                # merge | squash | rebase | fast-forward
tag_on_merge = false
# environment = true           # an environment branch (GitLab Flow): promoted downstream

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
patterns = ["features/{slug}"] # more names that also match; never used to create
# regex = "^feat-(?P<slug>.+)$"  # also matches; named groups become fields
parent = "develop"             # where it starts, and the default finish target
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # opt-in shell commands, always shown before they run
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # host-driven flows, see below
mode = "local"                 # local | host-creates-branch | bot
```

`merge`, `finish`, `finish_mode` and `hooks` drive the [actions](#actions) below;
`issue_flow` drives the [host-driven flows](#host-driven-flows).

### Name patterns

Text with `{placeholders}`:

| Placeholder | Matches | Rendered from |
|---|---|---|
| `{slug}` | free text; may contain `/` when last | the text, lowercased, accents folded, `-` between words, at most 50 characters |
| `{issue}` | digits (`123`) | `123` or `#123` |
| `{issue_key}` | a tracker key (`PROJ-123`) | as given |
| `{user}` | a name without `/` | slugified |
| `{date}` | `2026-10-16` or `20261016` | as given |
| `{version}` | `1.2.0`, `v1.2`, `1.2.0-rc1` | as given |
| `{n}` | digits | as given |

Anything else is literal, so prefixes and suffixes are just text:
`feature/{slug}`, `{user}/{issue}-{slug}-wip`, `bot/{issue_key}`. Each
placeholder may appear once, and two placeholders need text between them. A
pattern must produce valid branch names (the rules of `git check-ref-format`):
no spaces, no `~ ^ : ? * [ \`, no `..`, no component starting with `.` or ending
with `.lock`.

A `regex` is for matching only (it is anchored on both ends); use it for names no
pattern can express. Named groups become the fields of the branch.

### Which topic wins

1. A branch named like a base (any of its names) is that base.
2. Otherwise every pattern of every topic that matches is a candidate: `pattern`,
   then `patterns`, then `regex`, topic by topic in file order. The one that fixes
   the most literal characters wins (`release/{version}` beats `{user}/{slug}`
   beats a bare `{slug}`); a regex counts its literal prefix. A tie goes to the
   earlier one in the file.
3. Otherwise the branch is unclassified.

A broken pattern or regex is reported as a warning and skipped; the rest keeps
working.

## Templates

| Template | Bases | Topics |
|---|---|---|
| `git-flow` | `main` or `master`; `develop` or `dev` (parent main) | `feature/` and `bugfix/` (from develop), `release/` (from develop, into main and develop, tagged), `hotfix/` (from main, into main and develop, tagged), `support/` (from main); the plural spellings (`features/`, `releases/`...) match too |
| `github-flow` | `main` | `feature/` or any name, from main, merge request into main, squash, deleted afterwards |
| `gitlab-flow` | `main`; `pre-production` and `production` (environment branches, chained main, pre-production, production) | `feature/` or any name from main; `release/{version}` as a cherry-pick target |
| `trunk` | `main` | short-lived `{user}/{slug}` from main, rebased |
| `custom` | none | none |

Environment branches also match as `preprod`, `staging` and `prod`. A template is
a starting point: print it with `flow config init`, edit the file.

## Detection and import

`gitpoulpi flow detect` ranks the templates for the branches the repository has,
with a score out of 100 and the concrete names found (`master` for `main`, `dev`
for `develop`, `features/` for `feature/`). Bases count for 60 points, the share
of other branches the template's specific patterns recognize for 40, and a
long-lived branch of another template (a `develop` in a GitHub Flow candidate)
takes 25 off.

`gitpoulpi flow import --from git-flow` reads the classic git-flow settings
(`gitflow.branch.master`, `gitflow.branch.develop`,
`gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`) from the
repository's own `.git/config`; `--from git-town` reads `git-town.main-branch` and
`git-town.perennial-branches` (the parent of each branch,
`git-town-branch.<name>.parent`, is picked up as a recorded parent). Nothing is
applied silently: the command prints the TOML, and `--write` creates
`.gitpoulpi/workflow.toml` only if it does not exist.

## Commands

All of them take `--json`; the MCP server exposes them as tools (`flow_status`,
`flow_templates`, `flow_detect`, `flow_config_show` read; the actions are
described below).

| Command | What it does |
|---|---|
| `gitpoulpi flow status` | Every local and remote branch: kind (base, topic, unclassified), topic type and fields, parent and where it came from (`recorded`, `git_town`, `type`, `base`), and for local branches how many commits it is `ahead` of and `behind` its parent. Also the warnings found in the definition, the ranked detection and any other tool's settings that could be imported. |
| `gitpoulpi flow templates` | The presets above, with their full definition. Needs no repository. |
| `gitpoulpi flow detect` | The ranked templates with the bases and topics found. |
| `gitpoulpi flow config [--show-effective]` | The shared file (its path, whether it exists, the parsed workflow, unknown keys), or with `--show-effective` the workflow your branches are classified with. `flow config show` is the same. |
| `gitpoulpi flow config init --template <t> [--write]` | Print the TOML of a template; `--write` creates `.gitpoulpi/workflow.toml` and refuses to overwrite one. |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | As described above. |

Ahead/behind counts are computed for at most 300 local branches per call.

A workflow file that cannot be read (bad TOML, a wrongly typed value) is an error
that names the file; it is never ignored silently.

## Actions

`start`, `finish`, `sync`, `promote` and `set-parent` act on the repository. Each
one first **plans**: it lists every git step it will do, with the git command
each amounts to, the warnings, what cannot be undone and anything that blocks
it. Nothing changes until you confirm. Each run is **one entry in the undo
history** (`gitpoulpi undo` takes back the branches, the tag, the deleted
branch, the recorded parent and the checkout together).

| Command | What it does |
|---|---|
| `gitpoulpi flow start <type> [name=value ...] [--from <ref>] [--publish]` | Names the branch from the type's pattern (`flow start feature "Login form"` makes `feature/login-form`; `flow start release version=1.2.0` makes `release/1.2.0`; a bare word is the `slug`; `user` and `date` default to you and today), creates it from the type's parent or `--from` (a branch, a remote branch or a tag), checks it out and records its parent. `--publish` pushes it with `-u` (not undone by `undo`). |
| `gitpoulpi flow finish [<branch>]` | For each `finish` target of the type, in order: check the target out and merge the branch with the target's strategy (`merge`: always a merge commit; `squash`: one commit titled `--title` or from the branch name; `rebase`: rebase the branch on the target, then fast-forward; `fast-forward`), create the tag when the target has one (`tag = "v{version}"`, annotated), then check the parent out and delete the local branch when the type says so (`--delete` / `--keep` override it; `--delete-remote` also deletes it on the remote, not undoable). With `finish_mode = "merge-request"` (or `--mode merge-request`) it only pushes the branch and opens the pull or merge request into the parent, with `--title`, `--draft`, `--assignee`, `--reviewer`, `--label`, `--milestone` and `--integration` (not undoable; nothing is merged or deleted locally). `--mode local-merge` overrides a `merge-request` type, and a type that says `ask` needs one of the two. |
| `gitpoulpi flow sync [<branch>] [--stack] [--rebase\|--merge]` | Brings the parent's commits into the branch: a merge, or a rebase when the branch's strategy is `rebase` (or `--rebase`). With `--stack` also the branches whose parent is this one (recorded with `set-parent`), recursively, in order. Goes back to the branch you were on. |
| `gitpoulpi flow promote <env-branch>` | GitLab Flow environment branches: merges the branch it is downstream of (`main` into `pre-production`, `pre-production` into `production`) into it, one step, and goes back. Fast-forwards when it can. |
| `gitpoulpi flow set-parent <branch> <parent>` / `--clear` | Writes (or removes) `branch.<name>.gitpoulpi-parent`. Refuses a cycle. Undoable. |

A branch started from somewhere else finishes there: when the recorded parent
(from `--from` or `set-parent`) differs from the type's, the first finish target
is the recorded parent.

### Confirming, planning, scripting

| Flag | Meaning |
|---|---|
| `--dry-run` | Print the plan as JSON (`{"plan": ...}`) and stop. Exit code 1 when something blocks it. |
| `--yes` | Run without asking. |
| neither | On a terminal: the plan is shown and you are asked. Otherwise (a script, `--json`, an agent) the plan comes back as `{"needs_confirmation": {"plan": ...}}` with exit code 3 and nothing changed. `start` without `--publish` or hooks, `set-parent` and `--abort` need no confirmation. |
| `--run-hooks` / `--no-hooks` | See [hooks](#hooks). |
| `--autostash` | Stash local changes that block a checkout or merge and put them back afterwards, whatever `general.auto_stash` says. |

A blocked plan names the reason: a base branch where a topic is needed, a name
that already exists, a missing field (`release/{version}` needs `version=...`),
a tag that exists, a target branch that is behind its remote (`--allow-behind`
goes on anyway), a merge or rebase already in progress, a workflow that is not
configured.

With the **auto-stash** setting (`general.auto_stash`), working changes that
block a checkout or merge are stashed and restored around the action exactly as
for a plain checkout; with `never` the action stops before changing anything.

### Conflicts

A merge or rebase that stops on conflicts leaves the normal conflict state (the
conflict banner, `gitpoulpi conflicts ...`) and the flow is remembered. Exit
code 2, `status: "conflicted"`, the paths in `conflicts`. Resolve the files and
mark them resolved, then:

- `gitpoulpi flow finish --continue` commits the merge (or continues the
  rebase, or commits the squash with its message) and runs the remaining steps;
- `gitpoulpi flow finish --abort` aborts the merge or rebase, goes back to the
  branch the flow started on and forgets the rest. Steps that already
  completed (the first target of a double finish, with its tag) stay: `undo`
  takes them back.

A flow stopped by a failing hook is resumed the same way (`--continue` runs that
step again). The conflicted run and its continuation are two undo entries.

### Merge requests

The daemon pushes the branch; opening the request needs your hosting token, so
the CLI does it with the same code as `gitpoulpi hosting pr` (an integration
that matches the remote, else the host's pre-filled web page, printed as a URL).
The result carries `merge_request` (what was asked) and `opened` (what the host
answered), or `open_error` when the branch is pushed but the request could not be
opened.

### Hooks

`[hooks]` in the workflow file lists shell commands (`after_start`,
`before_finish`). They come from a file your team shares, so they **never run
unless you ask**: `--run-hooks`, or yes to the question a terminal asks after
listing them. The plan always lists them, with `hooks_run: false` when they will
not run. A hook runs in a shell with the repository as working directory and
`GITPOULPI_FLOW_ACTION` / `GITPOULPI_FLOW_BRANCH` set; its exact command, exit
code and whole output are in the result. A hook that exits non-zero stops the
flow there (a `before_finish` hook runs before anything changed, so it is a plain
error; after an `after_start` hook the branch exists and the flow can be
continued or aborted). `--no-hooks` skips them without asking. A hook can do
anything, so `undo` only takes back the git changes.

### For agents (MCP)

The tools are `flow_start`, `flow_finish`, `flow_sync`, `flow_promote` and
`flow_set_parent` (finish, sync and promote are annotated destructive, so
`allow_mutations` gates them). They are plan-then-execute: call with
`dry_run: true` to get the plan (a normal result), then again with `yes: true`
to run it. Without either the call is refused and carries the plan, so nothing
runs by accident. Hooks run only with an explicit `run_hooks: true`; the plan
lists them (`hooks`, `hooks_run`) so the agent can show the user the commands
first. After a conflict, `continue_: true` (the flag is `--continue`) or
`abort: true`.

### Undo

| Action | What `undo` does |
|---|---|
| start | Deletes the branch, forgets its parent, checks the previous branch out again. |
| finish | Restores every branch it moved, the tag, the deleted branch and the checkout, in one step. A pushed branch, a remote deletion and an opened merge request stay (the plan says so). |
| sync | Puts every updated branch back, files included for the checked-out one. |
| promote | Puts the environment branch back. |
| set-parent | Restores the previous recorded parent. |

## Host-driven flows

Teams where an issue starts the work: the issue is written from a template, and the
host (GitLab, GitHub, Gitea, Forgejo) or a bot makes the branch and the merge
request. GitPoulpi creates the issue, waits for what the host makes, and picks it
up: a local branch tracking the remote one, its parent recorded (the merge request's
target), checked out. It needs the ISSUES section linked to a tracker
(`gitpoulpi hosting link issues`); the PULL REQUESTS link says where the branches
and requests live (the ISSUES project when it is not linked).

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # a bot often triggers on a label
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # a file of the repository
branch_from_issue = "{type}/{issue}-{slug}"   # what the host or bot names the branch
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # how long `wait` polls
merge_request = true                 # host-creates-branch: also a draft request
assignees = []
```

`{type}` in `branch_from_issue` is the topic type's id; the rest are the usual
[name pattern](#name-patterns) tokens (`{issue}`, `{slug}`, `{issue_key}`...). A
branch belongs to issue 12 when it matches that pattern with `{issue}` equal to 12
(`feature/12-login` does, `feature/120-x` does not).

| Mode | After the issue is created |
|---|---|
| `local` | Nothing on the host. With `--pick-up`, a `flow start` of the type with the issue number and title (undoable like any start). |
| `host-creates-branch` | GitPoulpi creates the branch **on the host** from the type's parent (GitLab: the branches API; GitHub: `createLinkedBranch`, so the issue shows it under Development; Gitea/Forgejo: the branches API), and a **draft** request that says `Closes #12`. A host that refuses an empty request (GitHub: no commits yet) leaves the branch and says so as a warning. |
| `bot` | Nothing: a bot (a webhook handler, a CI job, a GitHub App) makes the branch and request. `--wait` polls until `wait_for` is satisfied. |

What `wait` looks at, for every provider: the host's own links (GitLab
`issues/:iid/related_merge_requests` and `closed_by`; GitHub linked branches and the
timeline's `connected` / `cross-referenced` pull requests), the branches whose name
matches `branch_from_issue`, and the open requests whose source branch does. It polls
with a growing pause (1, 2, 3, 5, 8, then every 10 s) until `wait_for` is satisfied or
`timeout_secs` ends, and a host that is slow or briefly unreachable after an answer is
retried. A source the host does not offer (a Gitea has no issue links) is skipped; when
every source fails, the first error is the answer.

| Command | What it does |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | Creates the issue from the template (`issue_title`, `issue_labels`, the description file, `--body` instead of it) and goes on as the mode says. `--dry-run` prints what would be created (`{"plan": ...}`) and creates nothing. `--pick-up` waits first in `bot` mode. Exit code 4: the wait ran out (the issue exists; `flow issue wait` keeps waiting). |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | Polls for what the host made for an issue, with progress on stderr on a terminal; Ctrl+C cancels. `--once` looks a single time. Exit code 4: nothing yet. |
| `gitpoulpi flow pick-up <#12 \| !5 \| branch> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | Fetches, creates the local branch tracking `origin/<branch>`, records its parent and checks it out. `#12` looks the issue's branch or request up on the host, `!5` a request, anything else is a branch name. The parent is the request's target (`--parent` overrides it), else the type's parent. Needs no `--yes`: it is local and undoable, like `start`. A `--dry-run` still fetches (it only moves remote-tracking refs, and the plan needs to see the branch). |

`pick-up` is one entry in the undo history: `gitpoulpi undo` deletes the branch,
forgets the parent and checks the previous branch out again (the fetch stays, like
every fetch). It is planned like the other actions: `--dry-run` prints its plan (`create_branch`,
`set_upstream`, `checkout`, `record_parent`) and the usual blockers apply (working
changes follow the auto-stash setting). The request is
`{"action": "pick_up", "branch": ..., "parent": ...}`.

Issues made by a flow are **watched** (`watched_issues` in `.gitpoulpi/repo-state.toml`,
never committed): after every fetch the app matches the remote branches against
`branch_from_issue` and tells you when one appeared for a watched issue. A pick-up
settles it.

### For agents (host-driven)

The tools are `flow_issue_new`, `flow_issue_wait` and `flow_pick_up` (write, network;
`allow_mutations` gates them). `flow_issue_new` with `dry_run: true` returns the
plan; `flow_issue_wait` blocks up to `timeout` seconds, so give an agent a short one
and call it again (`once: true` answers at once). Both return JSON with `found`
(`branches`, `requests`) and `timed_out`.

### Host-driven flows in the app

With a workflow and a linked tracker the Flow menu gets *New <Type> issue...* (one per
type; palette `Flow: New <Type> Issue...`, and `Flow: New Issue...`) and *Pick up a
host-made branch...* (palette `Flow: Pick Up Host Branch...`).

- **New issue** asks for the type (a dropdown) and the title; it shows where the
  description comes from and who makes the branch. *Pick the branch up when it is there*
  is on by default. *Create issue* creates it, then in `bot` mode shows the **waiting
  card**: a spinner, what it waits for, the elapsed time and the time it gives up, and
  **Stop waiting** (`Esc`). Stopping does not lose the issue: it stays watched. When
  the branch is there it is picked up and the status line says so.
- **A branch appeared** is a card that opens after a fetch brought the branch of a
  watched issue (when no other dialog is open): *Pick up* takes it over, *Later* closes
  it (it is not offered again until the app restarts).
- **Pick up** takes `#12`, `!5` or a branch name and shows what it found before picking
  it up.

Everything runs in the background; a call that lasts more than 150 ms shows a
spinner.

### Trying it on a bench

`scripts/fake-gitlab-bot.sh` is a stand-in bot for the Docker GitLab
(`scripts/test-bench.sh`): it watches a project for issues with a label and makes
`<type>/<iid>-<slug>` (with one commit) and a draft request that closes the issue.
`cargo run -p xtask -- mock-host --flow-bare <bare repo>` serves the same
thing offline (a GitLab-shaped API over a real bare repository, with the bot).

## In the app

![The sidebar with a Git Flow workflow: main and develop marked as base branches, feature branches in a Feature folder, the rest under Other](images/workflow-sidebar.png)

![Start Feature: the type, the slug, the start point and the branch name it will create](images/workflow-start.png)

![Settings > Repository > Workflow, General tab: the template, detection from the branches, hooks and issue flow, with the Rules tab beside it](images/workflow-settings.png)

Everything below appears only for a repository that has a workflow (a
`.gitpoulpi/workflow.toml`, or an override of yours). Without one nothing changes:
the sidebar is the ordinary tree and there is no `Flow` button.

### Settings > Workflow

*Settings > Repo-specific settings > Workflow* edits the definition. It works on a
draft and saves nothing until you press a Save button.

- **Template** is a dropdown of the presets; picking one replaces the bases and
  topic types below with the preset (your hooks and issue flow stay).
- **Detect from the branches** reads this repository's branch names and shows the
  closest template with its score, the base names it found (`master` for `main`),
  how many branches each topic type recognizes, with examples, and the score of
  every template. **Use the detected workflow** loads it into the draft.
- When the repository holds another tool's configuration (Git Flow's `gitflow.*`,
  Git Town's `git-town.*`), **Import it** is offered and loads the imported
  definition into the draft. Nothing is applied until you press it.
- **Base branches**, **Topic types** and their **Finish rules** are one
  [list](settings.md#lists) in groups on the **Rules** tab; the **General** tab
  holds the rest (template, detection, hooks, issue flow). Select an item to edit it on the right (name and
  other names, parent, protected, merge strategy, tag on merge, environment
  branch; label, name pattern, other patterns, regular expression, parent, finish
  mode, delete after finish). **+ Add ▾** picks what to add (a finish rule belongs
  to the selected topic type); add, duplicate, remove and drag work as in every
  Settings list, and items move within their group.
  The **name pattern** row lists the tokens (`{slug} {issue} {issue_key} {user}
  {date} {version} {n}`); under it a live preview shows the existing branches the
  pattern matches and the name it would create, and a pattern that does not parse
  says why in red, right under the field (nothing typed is lost).
- **Hooks** are shell commands, one per line (`After start`, `Before finish`),
  edited in a multi-line editor (`Ctrl+S` saves, `Esc` cancels). They run
  commands on your machine: the page says so, and the dialogs show them before
  they run.
- **Issue flow** carries the fields the [host-driven flows](#host-driven-flows) use
  (mode, issue title and labels, branch pattern, what to wait for), stored in the
  workflow file.
- **Problems** lists what the definition gets wrong (a parent that does not
  exist, a broken pattern).
- A bar pinned at the bottom of the page shows **● Unsaved** (or **No changes**).
  **Save to the shared file** writes the draft to `.gitpoulpi/workflow.toml` (commit
  it), keeping keys this version does not know, and folds your own override into it.
  **Only for me** keeps just the difference from the shared file in your repository
  state (`.gitpoulpi/repo-state.toml`, not committed). **Discard changes** reads what
  is saved again. Closing Settings with an unsaved draft asks first.

### The sidebar

LOCAL and REMOTE are grouped by the workflow: the base branches first, each with a
`[base]` badge (`[base, protected]` when protected), then one folder per topic type
named after its label with the number of branches (`Feature (3)`), then the
branches no type claims under `Other`. Folders collapse like any folder; the plain
trees come back with *Flow > Group branches by type* (the Flow menu row) or the
palette command `Flow: Toggle Grouped Branches`. Hovering a branch shows its type,
its parent and how far behind (and ahead of) the parent it is.

### The Flow menu and its dialogs

The toolbar's **Flow** button (`Ctrl+G Shift+W`, palette `Flow: Open Menu`) lists
*Start <Type>...* for every topic type, *Finish current branch...*,
*Sync with parent...*, *Promote...* (GitLab Flow environment branches), *Set
parent...*, the grouping toggle and *Workflow settings...*. The palette has the
same commands (`Flow: Start Feature...` is generated for each type).

- **Start** is a form built from the type's name pattern: a field per token. `{issue}`
  and `{issue_key}` come with a dropdown of the linked tracker's issues, which also
  fills `{slug}` from the issue's title; `{version}` suggests the version after your
  highest release tag; `{user}` is your active profile; `{date}` is today. A line
  shows the branch name the pattern makes (or why it cannot: an invalid ref, a name
  already taken). Start from is a dropdown (the type's parent first), and *Publish*
  pushes the new branch.
- **Finish**, **Sync**, **Promote** and **Set parent** ask for what they need, then
  show the **preview** the daemon works out (the same plan `gitpoulpi flow ... --dry-run`
  prints): every step in order, warnings (a target behind its remote, branches stacked
  on this one), and **blockers** (a target that does not exist, a target behind its
  remote) which turn the Run button off and say why. Steps that cannot be taken back
  (a push, a remote branch delete, a merge request) are marked `!` and colored, with
  a line "Undo will not take back: ...". Hooks that would run are listed under a
  warning, with a *Run the hooks* checkbox (off by default). When the type's finish
  mode is `ask`, Finish first asks *Finish by*: merge locally, or open a pull or
  merge request.
- The action then runs with a mark per step (a spinner after 150 ms) and the dialog
  shows how each step went. The whole action is **one Undo entry**: `Ctrl+Z` takes
  back the start, finish, sync or promote at once (the Undo history reads
  `Flow finish feature/login`); pushes, remote deletes and merge requests stay, as the
  preview said.
- A **conflict** stops the action with **Continue** and **Abort**. The dialog can be
  closed to resolve the conflict with the usual conflict tools, then reopened with the
  palette command `Flow: Continue or Abort Stopped Action...` (also a row of the Flow
  menu while an action is stopped); Continue refuses while files are still in conflict
  and keeps both buttons. Abort goes back to the branch the action started on; steps
  already done stay (undo takes them back).
- A finish by **merge request** pushes the branch, then opens the usual *Create Pull
  Request* form (*Create Merge Request* on GitLab) prefilled: the target is the branch's parent, the title comes from the
  branch (and issue). The metadata buttons (assignees, reviewers, labels, milestone)
  work as in any pull request. When no integration answers for the remote, the compare
  URL is opened in the browser instead (and shown on the status line).
- Step and blocker texts are English, like the CLI's; the dialog's own text is
  translated. A working tree with changes follows the `auto-stash` setting.

Every one of these is reachable from the keyboard and the palette, and every
dropdown is a combobox (Enter opens it, typing filters, Enter picks).

The host-driven rows (*New <Type> issue...*, *Pick up a host-made branch...*) are
described [above](#host-driven-flows-in-the-app).

