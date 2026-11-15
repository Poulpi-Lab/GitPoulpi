**English** · [Čeština](cs/hosting-integration.md) · [Deutsch](de/hosting-integration.md) · [Español](es/hosting-integration.md) · [Français](fr/hosting-integration.md) · [Italiano](it/hosting-integration.md) · [日本語](ja/hosting-integration.md) · [한국어](ko/hosting-integration.md) · [Nederlands](nl/hosting-integration.md) · [Polski](pl/hosting-integration.md) · [Português (Brasil)](pt-BR/hosting-integration.md) · [Português (Portugal)](pt-PT/hosting-integration.md) · [Русский](ru/hosting-integration.md) · [Svenska](sv/hosting-integration.md) · [Türkçe](tr/hosting-integration.md) · [Українська](uk/hosting-integration.md) · [简体中文](zh-CN/hosting-integration.md)

# Hosting and issue-tracker integrations

GitPoulpi connects to these products:

- **Hosting** (pull requests, issues, cloning from your account, SSH keys):
  GitHub, GitHub Enterprise, GitLab, GitLab Self-Managed, Azure DevOps,
  Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea and Forgejo
  (including Codeberg). **Gerrit Code Review** is hosting for review only: its changes are the
  pull requests, it has no issues.
- **Issue trackers** (issues only): Jira Cloud, Jira Data Center, Trello,
  Linear, YouTrack (Cloud and Server) and Redmine.
- **Local Markdown notes** (issues only, no token, no network): a folder of
  Markdown notes, such as an Obsidian vault folder, see
  [Local issues and Obsidian](local-issues.md).

Every integration uses a **personal token**. There is no GitPoulpi account and
no OAuth sign-in. Integrations are optional and **opt-in per repository**:
nothing is fetched until you link a repository. Your token is stored in your
credential helper, never in `config.toml`. Plain `git` operations (fetch, pull,
push) do not need an integration: they use the credentials that your `git`
already has.

## 1. Create a token on the host

The connect form has a *Generate a token on <host>* link. It opens the token
page of the host with the right scopes already selected. To do it by hand:

| Host | Where | Scopes |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (repositories, pull requests, issues), `read:user` (who you are), `admin:public_key` (only to add or remove SSH keys) |
| GitLab, GitLab Self-Managed | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, on `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`, `Work Items (Read & write)`. Create the token **for that organization**, not for "All accessible organizations": global tokens stop working on 2026-12-01. |
| Azure DevOps Server | The same page on your server (`https://<server>/<collection>/_usersSettings/tokens`) | The same two scopes |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. For SSH keys, also `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`. |
| Bitbucket Data Center | Profile picture > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | Settings > Applications > Generate New Token, on `<host>/user/settings/applications` | Read `user`; read and write `repository`, `issue` and `user` (the last for SSH keys). Gitea shows no scopes after the fact, so GitPoulpi cannot warn about a missing one: a call that is refused says so. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Read, write, create issues. The key acts as you. |
| YouTrack (Cloud and Server) | Your profile > Account Security > New token, on `<host>/users/me?tab=account-security` | The `YouTrack` scope. A permanent token acts as you. |
| Gerrit | Settings > HTTP Credentials > Generate new token (recent servers; older ones: Generate new password), on `<host>/settings/#HTTPCredentials` | None. The token is the **HTTP password** and acts as you; the user name goes next to it. |
| Redmine | My account > API access key > Show, on `<host>/my/account`. The administrator must have turned on **Administration > Settings > API > Enable REST web service**. | None. The key acts as you. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (the classic kind, without scopes) | None. The token acts as you. |
| Jira Data Center | Profile > Personal Access Tokens | None. The token acts as you. |
| Trello | First an **API key** from `trello.com/apps/admin` (create a Power-Up, then generate an API key). Then a **token** from `trello.com/1/authorize?...&key=<your key>`. | `read`, `write` |

Give the token a name such as "GitPoulpi" and an expiry date. Copy the token:
the host shows it only once.

## 2. Connect it

1. Press `Ctrl+,` and choose **Integrations**.
2. Choose the product on the left. Products are in two groups: **Hosting** and
   **Issue trackers**.
3. Press **Connect**. To add a second account of the same product, use
   **+ Add profile**.

   ![The Integrations page: the products on the left and a Connect button for GitHub](images/settings-integrations.png)

4. Fill in the form and paste the token. The token is shown in clear text while
   you type, so make sure that nobody is watching. The form lists the scopes
   that it needs.

   ![The connect form: a Personal Access Token field, the scopes repo, read:user and admin:public_key, a link to generate a token on github.com, and the Connect and Cancel buttons](images/settings-connect.png)

5. Press **Connect**. GitPoulpi checks the token with the host in the
   background. The page then shows a card with your name and login,
   *Connected* in green, and the SSH key section. A token without one of the
   wanted scopes still connects, and the missing scopes are listed in red.

On the card: **Edit** opens the form again. **Disconnect** deletes the token
and keeps the profile. **Remove** deletes both, after a confirmation.

Keyboard on the Integrations page: `Tab` / `Shift+Tab` move between the
product list, the profile strip and the form. `a` adds a profile, `e` edits
it, `d` or `Delete` removes it, `o` opens the token page, `Esc` goes back one
step.

### What each form asks for

The *Generate a token* link works only after you fill in the host or the
organization.

- **GitHub, GitLab:** only the token. **GitHub Enterprise** and
  **GitLab Self-Managed** also ask for the **Host domain**
  (`gitlab.example.com`; https unless you type `http://`). A name without a dot
  (a Docker service or a LAN machine) needs a port or `http://`: `gitea:3000`,
  `http://nas`. A self-hosted server can be
  plain http, for example a local container at `http://localhost:8080`. A token
  sent over plain http travels unencrypted, so use http only on a local or
  trusted network.
- **Azure DevOps:** the **Organization**, the name in
  `dev.azure.com/<organization>`. A `https://<org>.visualstudio.com` address is
  also accepted. Two organizations need two profiles. A repository uses the
  profile of the organization that its remote names. **Azure DevOps Server**
  asks for the **Collection URL**, for example
  `tfs.corp/tfs/DefaultCollection`. GitPoulpi does not use the system
  credential helper for Azure DevOps: Git Credential Manager stores a sign-in
  token there, not a personal access token.
- **Bitbucket:** your **Atlassian account email** and an API token. If the
  email is empty, the token is sent as a bearer token: use this for a workspace
  or repository access token. GitPoulpi finds your workspaces from the token.
  **Bitbucket Data Center:** the **Server URL** and an HTTP access token.
- **Gitea, Forgejo:** the **Host domain** and the token. **Forgejo** starts on
  `https://codeberg.org`: replace it for a self-hosted server. Gitea has no
  public default. An `http://localhost:3000` address works for a local server.
  Both kinds share one client (Forgejo is a Gitea fork with the same
  `/api/v1`); the token is sent as `Authorization: token <token>`. The
  system credential helper is not used for them.
- **Jira Cloud:** the **Site URL** (`acme.atlassian.net`), your **Atlassian
  account email** and a classic API token. Scoped Jira tokens are not
  supported. **Jira Data Center:** the **Server URL** and a personal access
  token.
- **Trello:** the **API key** and the **token**. The API key is public, so it
  is kept in `config.toml` (as `account`). The token is secret and goes to the
  credential helper. The form has two links: *Get an API key on Trello* and,
  after you type the key, *Generate a token*. The key and the token are sent in
  a header, never in a URL.

- **Linear:** only the **API key**: Linear has one public host, so there is
  no address to type. The key is sent as is in the `Authorization` header
  (Linear takes no `Bearer` for personal keys). To make the `ENG-12` references
  of commit messages open the right page, store the profile URL with your
  workspace (`https://linear.app/acme`, from the CLI: `--url`); the issue rows
  themselves always carry the exact address.
- **YouTrack:** the **Site URL** (`acme.youtrack.cloud` for Cloud, your own
  server otherwise, with its context path if it has one, for example
  `host/youtrack`) and a permanent token, sent as `Authorization: Bearer`. The
  same client serves Cloud and Server (`<url>/api`).

- **Gerrit:** the **Server URL** (with its context path if it has one), your **User name** and the HTTP
  password as the token. They travel as HTTP basic authentication; GitPoulpi talks to the `/a/` REST API of the
  server and ignores the `)]}'` line each answer starts with. The user name is not secret and is kept in
  `config.toml` (as `account`), the password goes to the credential helper. A repository is linked to a Gerrit
  *project* under PULL REQUESTS (its changes are listed there); Gerrit has no issues, and a change is created by
  pushing (see *Push for review* in [Reviewing](review.md#gerrit)).
- **Redmine:** the **Site URL** of your Redmine (with its subpath if it has one,
  for example `host/redmine`) and your API key, sent as `X-Redmine-API-Key`. If
  Redmine answers 403 or an HTML page, the REST web service is off (ask the
  administrator, or turn it on in **Administration > Settings > API**).

Jira, Trello, Linear, YouTrack and Redmine are issue trackers only: no pull requests, no repositories to
clone and no SSH keys. Bitbucket has no issues (Atlassian retired Bitbucket
Issues). Link the ISSUES section of a Bitbucket repository to a Jira project
instead.

## 3. Where the token is stored

The token is stored in your system **credential helper** (Git Credential
Manager on Windows, libsecret or `pass` on Linux, Keychain on macOS). It uses a
private entry that cannot be confused with the credentials of a real host. It
is never written to `config.toml` and never printed.

If no credential helper is configured, or if it refuses the token, GitPoulpi
keeps the token **in memory for this session only** and tells you. Connect
again after a restart.

GitPoulpi looks for the token of a profile in this order:

1. The environment variable `GITPOULPI_TOKEN_<PROFILE ID>`. Use it to give a
   token without storing it, for example in CI.
2. The token stored for the profile (`token_ref`).
3. For GitHub and GitLab profiles only: the system credential helper for the
   host, when no other enabled profile uses the same host.

Each profile is an `[[integrations]]` entry in `config.toml`. See the
[configuration reference](config-reference.md#integrations-hosting-and-issue-trackers).

## The security check

A saved token is only as safe as the address it is sent to. GitPoulpi therefore remembers, for each integration, what it was connected to, and stops when that changes:

- **The address.** The first time an integration is used (or connected), its address is recorded in `trust.toml`, in the user-only private data folder (`<data>/private/`), not in `config.toml`: editing the configuration can never widen what is trusted. If the address in the configuration is no longer the recorded one, nothing is sent to it and the token is not even read. Spellings of one address (`HTTPS://Git.Example.com:443/`) are the same address; a lookalike host is not.
- **The account.** After you connect a token, and then about once a day (and once per run for a token that comes from `GITPOULPI_TOKEN_*` or the credential helper), GitPoulpi asks the host who the token belongs to and compares the account **id**, not the login. A token that now belongs to someone else is stopped.
- **The certificate.** Servers whose certificate does not chain to a known authority (a private CA, a self-signed server) are accepted only for the exact key you accepted, and the TLS handshake still proves the server holds that key. With strict pinning on, the key must match even when the certificate chains to a public root; the price is that every key rotation asks again, which is why it is opt-in. The public hosts (github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net`, ...) keep no certificate record.
- **Plain http.** A non-loopback `http://` address is refused unless you allowed it for that integration.

Every request is also held to the integration's own address (and, for a public host, its API host such as `api.github.com`): a request anywhere else is refused before it is sent, and a redirect to another address is not followed.

**Integrations connected before this check existed** get their record silently, from the configuration as it was, the first time they are used after the upgrade: they keep today's level of trust and nothing asks.

### From the command line

A stopped command fails with exit code 2 and prints what changed (see the [CLI reference](cli-reference.md)). `gitpoulpi hosting integrations trust <id>` shows what is trusted; `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off` and `--allow-insecure-http on|off` accept one thing each, naming the exact value. After accepting a new address, enter the token again. Disconnecting (`hosting logout`) or removing an integration deletes its token, its trust record and what was cached from it (the saved repository lists of the Clone dialog, the Radar account lists and the remote review drafts of its host).

An agent (MCP) cannot accept anything: it is told that the security check needs you.

### In the terminal UI

A stopped request opens the **Integration security check** dialog over whatever you were doing. It says what changed (the old and the new address, the old and the new account with their ids, or the old and the new certificate as a small table with the SHA-256 fingerprint in groups of four, subject, issuer and validity) and that the token was not sent. The answers are numbered:

1. **Disconnect** deletes the saved token, the trust record and what was cached from the integration.
2. **Trust** the new address, account or certificate (an unencrypted address: **Allow plain http**).
3. **Cancel** changes nothing.

No answer is highlighted until you press an arrow key or a digit, so a stray Enter accepts nothing. Esc and the close cross cancel; a click outside the dialog does nothing. A long body scrolls with the wheel, Page Up/Page Down or the scrollbar. Several checks queue (one per integration and kind of change) and are asked one after the other. A canceled check does not pop up again while it is unchanged; **Integrations: Review security check** in the command palette brings it back.

After you trust a new **address**, the integration's form opens with the token field focused and the hint "Re-entering the token is recommended": the saved token was never sent there. In the form, an address changed on purpose is accepted together with a token typed for it; without a token the form asks for one.

The form also has two switches. **Strict certificate pinning** (for an https address) pins the certificate key the server presents when you connect. **Allow plain http (not encrypted)** is offered only for an `http://` address that is not this machine, with a red warning that the token travels in clear text. The connected card shows the pinned fingerprint.

### What the token may do, and until when

Connecting also asks the host what the token can do. GitHub (classic tokens), GitLab and Bitbucket (when it sends the scopes) report their scopes: the card shows a yellow "This token has more rights than GitPoulpi needs" line listing the scopes beyond what GitPoulpi uses, the sensitive ones (such as `delete_repo`, `admin:org`, `sudo`) in red, with the advice to prefer a fine-grained token with an expiry date. A classic GitHub token (`ghp_...`) gets that advice even when its scopes are exact. A fine-grained GitHub token cannot be asked for its permissions, so only its expiry is shown. The card shows the expiry date when the host says it. A token with no expiry shows no reminder, and the other products report no scopes and raise no warning. The same fields are in the JSON of `hosting integrations login` and `test` (`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). They are read when you connect, so the card shows them for the session that connected.

## 4. Link a repository

Linking is explicit and per repository. When a profile is connected, the Refs
sidebar shows a **PULL REQUESTS** section and an **ISSUES** section after TAGS.
Each section has two dropdowns: the provider and the project.

- The provider dropdown **proposes** the accounts that match the remotes of the
  repository (`origin` first), as rows like `owner/name · origin · GitHub`. One
  pick sets both the provider and the project.
- Nothing is linked automatically. Jira, Trello, Linear, YouTrack and Redmine are never proposed, because
  no remote names them: choose them yourself.
- **None (unlink this repository)**, at the top of the dropdown, unlinks the
  section. Nothing links it again by itself.
- The link is saved in the `.gitpoulpi/repo-state.toml` file of the
  repository.

The two sections are linked **separately**. For example, pull requests can
come from Azure DevOps and issues from Jira.

| Provider | PULL REQUESTS project | ISSUES project |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | the same |
| Azure DevOps | `Project/Repo` (the organization comes from the profile) | `Project` (work items) |
| Gitea, Forgejo | `owner/name` | the same |
| Bitbucket | `workspace/slug` (Data Center: `PROJ/slug`) | none: link Jira |
| Jira | none | the project key, for example `ABC` |
| Trello | none | a board |
| Linear | none | a team key, for example `ENG` (the teams are listed) |
| YouTrack | none | a project short name, for example `ABC` |
| Gerrit | the Gerrit project name, for example `team/tool` | none |
| Redmine | none | a project identifier, for example `bench` (the last part of the project's address) |

The ISSUES project list is searchable. You can also type a project that the
list does not show.

**Extra query.** An issue source has an optional filter row under the project
dropdown. It is combined (AND) with the project:

- Jira: a **JQL** fragment (`component = UI AND labels = backend`).
- Azure DevOps: a **WIQL** condition (`[System.AssignedTo] = @Me`).
- YouTrack: a **YouTrack query** (`#Unresolved Type: Bug`, `tag: ui or tag: ux`).
  It is wrapped in parentheses, so an `or` cannot widen the project. A
  `sort by:` of yours replaces the default `sort by: updated desc`.
- Redmine: none (the state and "assigned to me" filters of the section are enough).
- Linear: none (the state and "assigned to me" filters of the section are
  enough).
- Trello: a dropdown where you check the **lists** to show. With none checked it shows every
  open list.
- GitHub and GitLab ignore it.

`Enter` keeps a typed query, `Esc` drops it. Palette:
*Issues: Edit Query / Lists...*.

From the CLI: `gitpoulpi hosting link pulls --id <profile>` (or `issues`),
`--clear` to unlink, `--filter` for the extra query. See the
[CLI reference](cli-reference.md#commands).

### Clone from a connected account

A connected profile (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) is also a source in the **Clone** dialog
(Home tab, **Clone**, or `c` on the management tab). Each profile is listed
under **Clone with URL** as its own source (`GitHub · name`, `GitLab · name`);
the providers are never merged. Pick one and its repositories load, grouped by
owner: your own account first, then one section per organization or group
(`quarry/tools` for a GitLab sub-group), each under a header with its count.
A lock marks a private repository.

![The Clone dialog on a connected GitHub account: one section per owner, a lock on a private repository](images/clone-hosted.png)

The search box filters across the sections (name, full path and description);
a section with no match disappears. `Enter` on a row fills the clone with that
repository's HTTPS URL and a folder name.

![The Clone dialog with the search narrowing the list to one organization](images/clone-hosted-search.png)

The list of a profile you have opened before shows at once from a saved copy
while the host is asked again (*refreshing...* next to the label); when the
answer lands, new repositories appear and gone ones disappear, and the
highlighted and chosen repository stay where they are. If the refresh fails, the
saved list stays with a note, and the reason is in the Activity Log.

If the host refuses the token, the dialog shows *could not list repositories*
with the reason in place of the list (and the saved copy is deleted); the other
sources still work.

## 5. Pull requests and issues in the sidebar

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

### Actions on a row

- `Enter` or a double-click opens the row in your browser.
- Right-click, or `.`, opens the row menu:
  - **Open in browser**, **Copy URL**, **Copy number**.
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
| `Enter` / double-click | open the row in the browser |
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

## 5b. The issue board

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
the issue on the host (the note, for local issues); `e` edits assignees, labels and milestone; `n` starts a new issue in
the selected column; `a`, `l`, `M` and `/` filter by assignee, label, milestone and text, `x` clears the filters; `r`
reads the host again. The board reads again when the sidebar's lists refresh. The Radar has no board.

## 6. Create a pull request

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

## 7. Create an issue

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

## 8. Create on the host's own web page

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

## 9. The toolbar chip

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

## 10. When the lists refresh

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

## 11. Terminology

GitHub, Azure DevOps and Bitbucket say "pull request". GitLab says "merge
request". *Review request terminology* (`ui.pr_terminology`, on
Settings > Integrations) chooses the word for every label, menu, palette
command and message:

- **Auto (per host)**: follows the linked provider of the repository ("Pull
  Request" while nothing is linked).
- **Pull Request** or **Merge Request**: always that word.

The change applies at once. The CLI and MCP always say "pull/merge request".
This page says "pull request".

## 12. References in commit messages and branch names

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

### Insert an issue reference

In the commit composer, the `#▾` button at the right end of the Summary opens
a filterable list of the linked issues. The issue named by the current branch
comes first, marked *(from branch)*. `Enter` or a click inserts its reference
(`ABC-123`, `#12`, `AB#12`) at the caret. Keyboard: `Alt+R` in the composer.
Palette: *Commit: Insert Issue Reference...*. Without a linked issue list, the
button is grayed out and says *link an issue list in the sidebar first*.
GitPoulpi never adds a reference by itself.

## 13. SSH keys

### SSH key per profile

A connected profile has an **SSH key** section under its card. Without a key,
choose one of:

- **Default key**: your ssh-agent or `~/.ssh`. Nothing changes.
- **Generate new key**: an optional title (default
  `GitPoulpi <host> <date>`). With **Add the public key to <host>** checked,
  GitPoulpi uploads it, if the host does not have it already. The key is
  written without a passphrase to `~/.ssh/gitpoulpi_<profile id>_ed25519` and
  `.pub`. No `ssh-keygen` is needed. An existing file is never overwritten.
- **Custom key**: type a path or **Browse** for a private key file, then press
  **Use this key**.

When a key is attached, the section shows `title (SHA256:...)` with:

- **Remove**: detach the key, and delete it on the host if GitPoulpi added it.
- **Delete**: the same, and also delete the key files, after a confirmation
  that lists both paths.

### SSH key for one repository

To make one repository use a particular key, open the tab menu (`Ctrl+T`, or
right-click the tab) and choose **SSH & credentials...**, or run the palette
command *Repository: SSH & Credentials...*, and pick **Key file** (or **Account
key** for the key of an integration profile) under SSH. The choice is kept in
GitPoulpi's private folder: nothing is written to the repository or to git's
config. See [Settings](settings.md#repo-specific-ssh--credentials).

A repository that still has a `core.sshCommand` of its own, from the former
"SSH key for this repository" pin, shows it on that page, read-only, with
**Move into GitPoulpi**.

CLI and MCP: `gitpoulpi repo-auth` (the per-repository choice) and, for the
`core.sshCommand` pin itself, `gitpoulpi ssh-command show | set --key <file> |
clear`.

The global key on Settings > SSH & credentials (*Private key path*, or
*Generate key pair*) applies to every repository without a choice of its own.
A repository's key file or pin wins over it.

## 13b. Branch protection, for a history rewrite

When you rewrite history ([the workbench](history-rewrite.md)), GitPoulpi
asks the host which branches it protects, so they are left out of the rewrite
by default. It asks only for a repository you **linked** (the pull requests
or issues section of the sidebar, or a pinned profile), with the token of that
profile, and a token that cannot read it just gives "unknown": nothing fails.
What each provider answers, and what the token needs:

| Host | What is read | The token needs |
|---|---|---|
| GitHub (and Enterprise Server) | the `protected` flag of the branches, and the active branch rulesets (`~DEFAULT_BRANCH`, `~ALL` and `refs/heads/...` patterns) | a classic token with `repo` (`public_repo` for a public one), or a fine-grained token with Metadata (read). A ruleset the token cannot read is said, the classic rules still count |
| GitLab (gitlab.com and self-managed) | `protected_branches` (names and `*` patterns) | `read_api`, and a role that may see the repository settings |
| Azure DevOps (Services and Server) | the enabled, blocking branch policies of the repository (`Exact`, `Prefix` and `DefaultBranch` scopes) | Code (read). The "force push" permission is a security setting, it is not read |
| Bitbucket Cloud | the branch restrictions that stop a push (`push`, `force`, `delete`) | `repository:admin`. A restriction on the branching model is not resolved |
| Bitbucket Data Center | the branch permissions that stop a force push (`fast-forward-only`, `pull-request-only`, `read-only`) | repository admin |

The settings page of the protection (and of the default branch, and of the
deletion of the repository) is built from the repository's remote URL for
GitHub, GitLab, Azure DevOps and Bitbucket, even when the repository is not
linked, and shown with the three ways out when the host refuses a push.

## 13c. Account-wide lists in the Radar

The Radar normally watches the repositories you have cloned. A profile can also
list what concerns you on the whole host, cloned or not, with three switches in
its `[integrations.radar]` table (all off by default; see the
[config reference](config-reference.md#integrationsradar-account-wide-lists)):

- **review requests**: open requests that ask for your review;
- **open requests**: every open request of your own namespace and of each org or
  group you are a member of, or of the namespaces you name. Never the whole host;
  the **Namespaces** of the profile (one per line in Settings > Integrations > Radar, optionally
  as regular expressions) limit all three lists: a line matches a repository's full path when it
  equals it or is a path prefix at a `/` (`ptm` keeps `ptm/produits/x`, not `ptmx/y`), and is
  applied to the rows the host returns, plus server side where the host's search allows it
  (GitHub's `user:` / `repo:` qualifiers);
- **assigned issues**: open issues assigned to you.

GitHub, GitLab (including self-managed), Gitea, Forgejo, Azure DevOps (Services
and Server), Bitbucket Data Center and Gerrit answer. How each one is asked:

| Host | Requests | Issues |
|---|---|---|
| Azure DevOps | active pull requests of the whole organization (`searchCriteria.reviewerId` for the ones waiting for you; a request you already voted on is left out); when the server refuses the organization-level route, the first 20 projects are asked one by one. `namespaces` are project names | work items assigned to you that are not Done, Closed or Removed |
| Bitbucket Data Center | your dashboard: `REVIEWER` role not yet approved for review requests, `AUTHOR` and `PARTICIPANT` for open requests; `namespaces` are project keys | none (Data Center has no issue tracker; a note says so) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`; open requests are `is:open projects:<prefix>` per namespace (a project path prefix), or your own and reviewed changes when none is named; a work-in-progress change is a draft | none |

Bitbucket Cloud has no account-wide rows: it offers no endpoint that lists
requests across repositories, and asking each repository would be too many
calls. Its cloned repositories keep their own lists, and `gitpoulpi hosting
radar --id <bitbucket>` says so. When Gerrit has more changes than the limit it
returns, the list is marked cut with a note.

Drafts are left out unless you keep them. Each list is bounded: 300 rows per kind, with a
note saying how many more there were, and on GitHub a few search calls per
round, because its search API allows only 30 a minute. A namespace the token
cannot read becomes a note and the others still load; a rejected token or a
rate limit fails that profile only. Answers are reused for five minutes
(`radar-account.json` in the config folder).

In Settings > Integrations, a connected code-host profile has a **Radar** block
under its SSH key: one checkbox per list (*Requests waiting for my review*,
*Every open request*, *Include drafts*, *Issues assigned to me*, the last one
absent for hosts without issues), a *Namespaces* field (one entry per line, a name matching
that group or owner and everything below it, or a regular expression per line
with *Regular expressions* ticked; empty means your own and every one you belong
to) and the choice *Request of a
repository that is not cloned*: *Ask*, *Clone it* or *Review through the API*.
Everything is saved as you change it. Hosts with no account-wide query (Bitbucket
Cloud) show one line saying so instead. The command palette reaches the block
with *Settings: Radar Options of a Profile...*, and the Settings search finds each
control by its label.

From the command line: `gitpoulpi hosting integrations radar <id> --review-requests on`
turns a list on, `gitpoulpi hosting radar --json` prints the lists (and
`--refresh` skips the cache). AI agents have the same two commands over MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 13d. Acting as a GitHub App bot

A project that must not show a person's name can do everything on the host
through a **GitHub App**: every comment, review, close or reopen and label
change then appears as `<app-slug>[bot]`. This works for one repository at a
time and leaves your other repositories on your own token.

1. Create a GitHub App for the organization with the **Issues** and **Pull
   requests** permissions set to *Read & write* (and *Contents: read*, so
   pull requests can be listed and reviewed), and install it on the
   repository.
2. Write a small script that prints an **installation token** on its standard
   output (the App's private key signs a short-lived JWT, which is exchanged
   for a token that is valid for one hour). GitPoulpi never sees the key.
3. In **Settings > Repository > SSH & credentials**, choose **Token
   command** under HTTPS, enter the script as the **Command** and the host
   (`github.com`) under **Hosts**, and tick **Use for the hosting API
   (issues, pull requests)**. Save and confirm the command (a person always
   confirms it, with its SHA-256). From the command line:
   `gitpoulpi repo-auth set --https token-command --hosts github.com
   --use-for-hosting --yes --command ./app-token.sh`.

What changes for that repository:

- Its issue and pull request calls (listing, creating, commenting,
  reviewing, merging, closing, labels) use the command's token, with the same
  30 second limit, in-memory cache and redaction as git's own sign-in. When
  the host answers 401 because the token expired, the command is asked again
  once. If the command cannot run, the call is refused: your personal token is
  never the fallback.
- There is no "me": an installation token cannot ask who it is. The section
  header names the profile with **(App bot)** and the filters built on "me"
  (*Assigned to me*, *Mine*, *Awaiting my review*) have nothing to match.
  Nothing is shown as an error.
- An App cannot be assigned. Combine the setup with the blocked assignment
  below.
- Turning **Use for the hosting API** on or off needs a person, like a new
  command.

Only GitHub profiles use the repository's token this way; other hosts keep the
profile's own token.

### Blocking assignment in a repository

**Settings > Repository > SSH & credentials** has an **Assignment** row
(**Allowed** by default, or **Blocked**; saved at once). Blocked, assignment
is impossible in that repository's issues and pull requests, whoever asks:

- the assignee picker is left out of the new issue and new pull request forms
  (with a note saying why), the palette's *Hosting: Edit Assignees...* is
  grayed with the reason, and the row menu says so instead of opening it;
- every client refuses an assignment before anything is sent, including
  assignees passed while creating an issue or a pull request: the CLI answers
  `{"refused": {"reason": "assignment_blocked", ...}}` with exit code 2, and
  an agent over MCP gets the same refusal;
- assignees that already exist stay visible (read-only).

`gitpoulpi hosting assignment` shows the setting, `--block` turns it on and
`--allow` turns it off again; only a person can allow again (the MCP server
cannot).

## 14. From the command line

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
gitpoulpi hosting assignment --block
```

See the [CLI reference](cli-reference.md#commands) for every command. A token
is never accepted as an argument, and the MCP server does not offer
`hosting login`, so a token never passes through an agent. `hosting login`
prints `stored: "os"` when the OS credential store keeps the token, or
`stored: "session"` plus `store_error` when the store refused it and the token
lives in memory for this session only.

## Troubleshooting

| Symptom | Fix |
|---|---|
| *Connect* fails with an authentication error | The token is wrong, expired or revoked. Create a new one. |
| A scope is listed in red | The token does not have it. The connection works, but that feature does not. Add the scope on the host, then **Edit** and paste the new token. |
| The sidebar has no PULL REQUESTS section | No enabled profile is connected. Connect one, then link the repository in the section's dropdown. |
| A *Reconnect* row in a list | The token was lost (a restart without a credential helper) or it expired. Click the row to open the profile. |
| A self-managed instance fails | Check that the **Host domain** has no path, and that your computer trusts the certificate of the instance. |

## Known limits

- **Pull requests from forks:** Azure DevOps and Bitbucket cannot check out a
  request from a fork (see [Actions on a row](#actions-on-a-row)). Azure
  DevOps cannot create a pull request from a fork.
- **Trello** shows at most the first 1000 cards of a board. Use the list
  filter to narrow it. **Jira**, **Linear** and **YouTrack** list at most 500
  issues per refresh. Use a JQL fragment or a YouTrack query to narrow it.
- **Linear** takes one assignee per issue: if you pick several in the picker,
  the first is sent. Its "milestone" is the team's **cycle** (project
  milestones are not used). **YouTrack**'s is the `Fix versions` field; a
  version has no number, so the picker numbers the unreleased ones by their
  position in the project. Neither tracker has pull requests.
- **Redmine** takes one assignee per issue (the first of the picker), has no labels,
  and lists at most 500 issues per refresh. Assignees are the members of the project
  (people and groups) and are told apart by their display name. The Kanban board's
  columns are the issue statuses, and moving a card changes the status; the default
  workflow rules of Redmine apply (a role that may not make that transition gets the
  refusal back and the card snaps back). "Resolved" is not a closed status in Redmine's
  default data: the Closed column is the "Closed" and "Rejected" statuses.
- **Bitbucket Data Center** has no browser links for new pull requests,
  branches or commits. The pull request pages open normally.
- **Azure DevOps** shows no author avatars. The "mine" and "review requested"
  filters need an endpoint that can refuse personal access tokens. In that
  case the profile connects without a user name and these two filters are not
  available. An invalid token gets a sign-in page instead of an error;
  GitPoulpi reports it as a rejected token.
- **Gitea and Forgejo:** a draft is a request whose title starts with `WIP:`
  (the server has no draft flag on create). A request lists as reviewers
  everyone who already reviewed it, and withdrawing a review request only
  removes pending ones. Pull requests from the local branch of a fork check
  out through `refs/pull/<n>/head`, like GitHub. Mentions are searched with the
  server's `mentioned_by` filter. The server shows no token scopes, so a
  missing one is only found when a call is refused.
- **The issue board** moves cards on every issue source. On Jira a move runs the workflow transition that leads to
  the column; a transition that asks for fields (a resolution, say) is refused by Jira and the card snaps back with
  its message. On Azure DevOps a work item type without the target state snaps back with the reason.
- **Bitbucket:** app passwords no longer exist; use an API token.
  Repositories are listed workspace by workspace.
- **Jira Cloud:** only classic API tokens work (no scoped tokens, no OAuth).
  A new Jira issue gets a plain-text description.
- Creating a pull request, work item, issue or card happens on the host and
  **cannot be undone** from GitPoulpi. Checking out a request can be undone
  normally.
