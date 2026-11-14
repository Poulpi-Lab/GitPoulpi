**English** · [Čeština](cs/mcp-and-agents.md) · [Deutsch](de/mcp-and-agents.md) · [Español](es/mcp-and-agents.md) · [Français](fr/mcp-and-agents.md) · [Italiano](it/mcp-and-agents.md) · [日本語](ja/mcp-and-agents.md) · [한국어](ko/mcp-and-agents.md) · [Nederlands](nl/mcp-and-agents.md) · [Polski](pl/mcp-and-agents.md) · [Português (Brasil)](pt-BR/mcp-and-agents.md) · [Português (Portugal)](pt-PT/mcp-and-agents.md) · [Русский](ru/mcp-and-agents.md) · [Svenska](sv/mcp-and-agents.md) · [Türkçe](tr/mcp-and-agents.md) · [Українська](uk/mcp-and-agents.md) · [简体中文](zh-CN/mcp-and-agents.md)

# MCP and agents

An AI agent (Claude Code, Cursor, Codex, ...) can drive GitPoulpi through its
**MCP server** or its **CLI**. Both go through the same background daemon as the
window, so what the agent does shows up live in your open GitPoulpi, and both run
exactly the same code for each command.

This page is about *external* agents. GitPoulpi's own AI buttons (commit
messages, explanations, the commit composer) are configured separately: see
[AI assistance](ai.md).

## Quick setup

1. Put `gitpoulpi` and `gitpoulpi-mcp` on your `PATH` ([Installation](installation.md)).
2. Register the server with your agent. It speaks stdio: nothing to start, no port.
3. Optionally choose what the agent may do in `config.toml`'s `[mcp]` table
   (below). The defaults let it read the repository (changes are refused until you set
   `allow_mutations = true`), show you things only after you confirm, and
   never pre-fill dialogs.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` writes it to the repository's `.mcp.json` instead, so everyone
who opens the project gets it. The same file by hand, with the optional extras:

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "command": "gitpoulpi",
      "args": ["mcp", "serve", "--stdio", "--ui-control", "auto"],
      "env": { "GITPOULPI_CONFIG_DIR": "/home/me/.config/gitpoulpi-agent" }
    }
  }
}
```

- `--ui-control auto|ask|off` overrides `[mcp] ui_control` for this agent only.
- `GITPOULPI_CONFIG_DIR` gives the agent its own `config.toml` (its own `[mcp]`
  limits, no access to your settings). Leave it out to share yours.
- `gitpoulpi --repo <path> mcp serve --stdio` pins the server to one repository;
  otherwise a tool acts on the repository around the server's working directory,
  or the `repo` argument of the call.

**Other clients**: any MCP client that launches a stdio server.

| Client | Where | What |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | the client's MCP JSON file | the `mcpServers` block above (VS Code names the key `servers`) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Anything else | its documentation | the command `gitpoulpi-mcp` (standalone binary, same server, no arguments needed) |

**Over HTTP** (clients that connect to a URL):

```
gitpoulpi-mcp --http                         # [mcp] bind, default http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

Stateless JSON, same tools, same `[mcp]` gates. Every request needs the bearer
token (`[mcp] http_auth`, on by default): print it with `gitpoulpi mcp token
--show` (or `--copy`, `--rotate`; the token lives in a user-only file under the
private data folder) and give it to the client as a header:

```
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp \
  --header "Authorization: Bearer <token>"
```

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "type": "http",
      "url": "http://127.0.0.1:8808/mcp",
      "headers": { "Authorization": "Bearer <token>" }
    }
  }
}
```

A missing or wrong token gets `401` with `WWW-Authenticate: Bearer`. A rotated
token applies at once, with no server restart. Turning `http_auth` off is for a
loopback endpoint where you trust every local account; the server then warns on
stderr.

### Reaching the endpoint from another machine

The safest setup needs no remote listener at all: keep the loopback address and
forward the port over SSH from the machine that runs the agent, `ssh -L
8808:127.0.0.1:8808 host`. Then `--allow-remote` is not involved.

`--allow-remote` (a non-loopback address) is refused unless authentication is on
with a token in place **and** the traffic is encrypted. The refusal names what is
missing. There are two ways to be encrypted:

- **TLS in the server** (`[mcp] tls = "self_signed"` or `"files"`). The server
  terminates TLS itself: it holds the key, runs the handshake, and only a finished
  handshake reaches the HTTP layer; a plain-HTTP client fails the handshake and no
  plain listener exists beside it. So the encryption is a fact the server
  establishes on every connection, not an assumption. With `self_signed`, the
  first start generates a pair under the private data folder and prints its
  SHA-256 fingerprint (also in Settings); a client trusts the server by pinning
  that fingerprint or the certificate file. With `files`, set `tls_cert` and
  `tls_key` to your PEM files and clients verify the certificate by its CA.
  Clients authenticate with the bearer token; client certificates are not used.
- **A tunnel or reverse proxy** (`[mcp] behind_proxy = true`). A tunnel agent or
  proxy connects like any local or LAN client and its headers can be forged, so
  the server cannot tell from inside whether one is there. That is why it is an
  explicit acknowledgement (shown with a red warning in Settings) instead of a
  detection. Run the tunnel on the same machine and keep the loopback address when
  you can. With a non-loopback address behind a proxy, the hop between the proxy
  and the server is plain text on that network.

**Without MCP**, an agent with a shell uses the CLI: `gitpoulpi --json <command>`
(see the [CLI reference](cli-reference.md)). Give it the ready-made skill
[`skills/gitpoulpi/SKILL.md`](../../skills/gitpoulpi/SKILL.md), which says when
and how to use GitPoulpi instead of raw `git`.

## Permissions

| Switch | Default | Effect |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp` refuse to start (Settings > Agents & MCP > *MCP server*). Read when the server starts: a running server keeps its settings. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Address `--http` uses when none is given (loopback only unless `--allow-remote`, which needs authentication and encryption, see above). Settings > Agents & MCP > *MCP server port* edits its port and keeps the host. |
| `[mcp] http_auth` | `true` | `--http` needs `Authorization: Bearer <token>` (`gitpoulpi mcp token`). Settings > Agents & MCP > *HTTP authentication*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` or `files` (your PEM paths): the server terminates TLS itself. Settings > Agents & MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | You vouch that a tunnel or reverse proxy encrypts the traffic, which lets `--allow-remote` run without TLS. |
| `[mcp] allow_mutations` | `false` | Off: every tool that changes something is refused; reads, `ui_*` and proposals still work. `true` lets the agent commit, push, reset... |
| `[mcp] mutation_allowlist` | `[]` | Non-empty: only these tools may change things, e.g. `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: the agent cannot move your window (`ui_state` / `ui_viewers` still answer). |
| `[mcp] ui_control` | `"ask"` | `ask`: a chip in your window that you confirm; `auto`: applied unless you are busy; `off`: refused. Per agent: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Lets the agent open pre-filled dialogs you confirm (Settings > Agents & MCP > *Let AI agents propose changes for me to confirm*). |
| `[mcp] allow_sensitive_staging` | `false` | Only in `config.toml`. Off, the `stage` tool does not offer `allow_sensitive`, and a stage that would add a sensitive file (a private key, a `.env`, a recognized token) is refused with `{error: "sensitive_files", files}`: the agent is to ask you. `true` lets it pass `allow_sensitive`. See [Staging and committing](staging-and-committing.md#sensitive-files). |
| `[mcp] allow_history_purge` | `false` | Only in `config.toml`. `true` lists the tool `rewrite_purge`, which deletes the old history of a history rewrite (and the undo of it) for good. Off: the agent does not see it. It still needs `allow_mutations`. |
| `[ui] allow_agent_view_control` | `true` | Your window's master switch (Settings > Agents & MCP > *Allow AI agents to drive the view*). Off: no agent can reach your window. |
| `[ui] launch` | `"off"` | Whether an agent may open a GitPoulpi window when none is open (see [below](#when-no-window-is-open-or-it-is-in-another-session)). |

What this means in practice:

- **Out of the box the agent is read-only**: it can read, show you things in
  your window (`ui_*`) and, if you allow it, propose changes you confirm. To let
  it change the repository (commit, push, reset...) set `allow_mutations = true`;
  for a narrow grant keep it `true` with a `mutation_allowlist`. Every tool
  carries MCP annotations (`readOnlyHint`, `destructiveHint`, `openWorldHint`),
  so a client such as Claude Code still asks you before the risky ones.
- **Conflicts**: `conflicts_check` (a prediction, before any merge), `conflicts_list`, `conflicts_show` and `conflicts_state` are
  reads; `conflicts_resolve` (it discards the side not kept, `--delete` removes
  the file) and `conflicts_abort` (drops the whole merge/rebase/cherry-pick) are
  destructive; `conflicts_unresolve` is a plain write. A refused call (`stale`,
  `not_conflicted`, `pick_incomplete`...) answers `{"refused": {...}}` with exit
  code 2 instead of failing.
- **Stuck states**: `conflicts_am_continue` is a write, `conflicts_am_skip`,
  `conflicts_am_abort` and `index_lock_remove` are destructive, `index_lock_show`
  is a read.
- **History rewrite**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (the size analysis), `rewrite_scan_secrets` (findings with a redacted preview, never a value),
  `rewrite_script_check`, `rewrite_script_test` and
  `rewrite_discard` are reads (a preview changes nothing; it only keeps a plan in
  the daemon and on disk in the git dir, so `rewrite_apply` also works after a restart). `rewrite_apply` and
  `rewrite_run` are destructive: they move many refs to rewritten commits as one
  undo entry (`undo` puts them back) and never push. Pass the recipe as
  `recipe_json` (or `recipe`, a path) and secrets as `secrets_file`, never inline.
  Shell, Python and external Lua scripts (`script_language`) are not sandboxed
  and **an agent cannot trust them**: there is no `rewrite_script_trust` tool and
  no `trust` argument, so such a script runs only after a person ran
  `gitpoulpi rewrite script trust FILE` on that exact text (a changed script is
  refused again, `script_not_trusted` with the hash in `refused.script`), and
  only when `rewrite.external_scripts` switches its language on
  (`scripts_disabled`).
  A refusal answers `{"refused": {"reason", "message", "detail"}}` and a plan that
  moves pushed refs answers `{"needs_confirmation": ...}` (exit code 3) until the
  call sets `yes` - show it to the user first.
- **Never reachable by an agent**: storing a token (`hosting login`), creating
  and uploading an SSH key, `config set` (it could rewrite its own limits), and
  the daemon lifecycle. A token is never shown to the agent.
- **Account-wide lists**: `hosting_radar` is a read (requests waiting for your
  review, open requests, assigned issues across every repository of a profile,
  from a five-minute cache unless `refresh`); `hosting_integrations_radar`
  changes a profile's `[integrations.radar]` table, so `allow_mutations` gates it
  (even when called with no option, which only prints the settings).
- **Review context**: `ai_review_context` shows what a code review draft is given
  besides the diff (CLAUDE.md, AGENTS.md, an extra file, where a local agent runs). With
  a setting argument it saves a default (global, or the `repo`'s or the `project`'s own),
  so `allow_mutations` gates it, like the other tools that change a setting; with none it
  only prints. Like every `ai_*` tool it is listed only while `ai.enabled` is on.
  `hosting_review_suggest` takes the same arguments for one run only.
- **The `[mcp]` gates apply to the MCP server.** The CLI is your own shell and is
  not gated by `allow_mutations`; only `ui_control` and `allow_ui_propose` also
  apply to `gitpoulpi ui ...`.
- A refused call is an ordinary tool error (`isError: true`, the reason in
  `structuredContent.error`), never a silent no-op.

```toml
[mcp]
allow_mutations = false        # true: the agent may change the repository
mutation_allowlist = []        # e.g. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" lets the agent move your view without asking
allow_ui_propose = false       # true: the agent may pre-fill Reword / Split / commit / branch / tag dialogs
allow_sensitive_staging = false  # true: the agent may stage files the staging guard flags
allow_history_purge = false    # true: the agent sees rewrite_purge (deletes old history for good)
```

## The tools

One tool per CLI command, named after its path with `_` (`branch create` is
`branch_create`). Each takes the command's arguments plus `repo` (the repository;
default: the server's) and `session`. A successful call's `structuredContent` is
exactly what `gitpoulpi --json <command>` prints. **Kind**: *read* never changes
anything; *write* and *destructive* are mutations (gated); *network* talks to a
remote or a host; *view* moves your window only; *propose* opens a dialog you
confirm.

<!-- mcp-tools:begin -->
| Tool | CLI | Kind |
|---|---|---|
| `ai_commit_message` | `gitpoulpi ai commit-message` | read, network |
| `ai_explain` | `gitpoulpi ai explain` | read, network |
| `ai_prompt` | `gitpoulpi ai prompt` | read |
| `ai_resolve_conflict` | `gitpoulpi ai resolve-conflict` | read, network |
| `ai_review_context` | `gitpoulpi ai review-context` | write |
| `ai_rewrite_rules` | `gitpoulpi ai rewrite-rules` | read, network |
| `ai_rewrite_script` | `gitpoulpi ai rewrite-script` | read, network |
| `ai_split_plan` | `gitpoulpi ai split-plan` | read, network |
| `alias_status` | `gitpoulpi alias status` | read |
| `amend` | `gitpoulpi amend` | destructive |
| `apply_commit` | `gitpoulpi apply-commit` | write |
| `autostash_start_over` | `gitpoulpi autostash start-over` | destructive |
| `blame` | `gitpoulpi blame` | read |
| `branch_create` | `gitpoulpi branch create` | write |
| `branch_delete` | `gitpoulpi branch delete` | destructive |
| `branch_delete_remote` | `gitpoulpi branch delete-remote` | destructive, network |
| `branch_fast_forward` | `gitpoulpi branch fast-forward` | write |
| `branch_list` | `gitpoulpi branch list` | read |
| `branch_rename` | `gitpoulpi branch rename` | write |
| `branch_set_upstream` | `gitpoulpi branch set-upstream` | write |
| `branch_track` | `gitpoulpi branch track` | write |
| `branch_unset_upstream` | `gitpoulpi branch unset-upstream` | write |
| `branch_upstream_add` | `gitpoulpi branch upstream add` | write |
| `branch_upstream_list` | `gitpoulpi branch upstream list` | read |
| `branch_upstream_primary` | `gitpoulpi branch upstream primary` | write |
| `branch_upstream_pull` | `gitpoulpi branch upstream pull` | write |
| `branch_upstream_remove` | `gitpoulpi branch upstream remove` | write |
| `checkout` | `gitpoulpi checkout` | write |
| `cherry_pick` | `gitpoulpi cherry-pick` | write |
| `clone` | `gitpoulpi clone` | write, network |
| `commit` | `gitpoulpi commit` | write |
| `config_get` | `gitpoulpi config get` | read |
| `config_path` | `gitpoulpi config path` | read |
| `conflicts_abort` | `gitpoulpi conflicts abort` | destructive |
| `conflicts_am_abort` | `gitpoulpi conflicts am-abort` | destructive |
| `conflicts_am_continue` | `gitpoulpi conflicts am-continue` | write |
| `conflicts_am_skip` | `gitpoulpi conflicts am-skip` | destructive |
| `conflicts_check` | `gitpoulpi conflicts check` | read |
| `conflicts_list` | `gitpoulpi conflicts list` | read |
| `conflicts_resolve` | `gitpoulpi conflicts resolve` | destructive |
| `conflicts_show` | `gitpoulpi conflicts show` | read |
| `conflicts_skip` | `gitpoulpi conflicts skip` | destructive |
| `conflicts_state` | `gitpoulpi conflicts state` | read |
| `conflicts_unresolve` | `gitpoulpi conflicts unresolve` | write |
| `convert` | `gitpoulpi convert` | write |
| `diff` | `gitpoulpi diff` | read |
| `discard` | `gitpoulpi discard` | destructive |
| `fetch` | `gitpoulpi fetch` | write, network |
| `file_history` | `gitpoulpi file-history` | read |
| `flow_config_init` | `gitpoulpi flow config init` | write |
| `flow_config_show` | `gitpoulpi flow config show` | read |
| `flow_detect` | `gitpoulpi flow detect` | read |
| `flow_finish` | `gitpoulpi flow finish` | destructive, network |
| `flow_import` | `gitpoulpi flow import` | write |
| `flow_issue_new` | `gitpoulpi flow issue new` | write, network |
| `flow_issue_wait` | `gitpoulpi flow issue wait` | write, network |
| `flow_pick_up` | `gitpoulpi flow pick-up` | write, network |
| `flow_promote` | `gitpoulpi flow promote` | destructive |
| `flow_set_parent` | `gitpoulpi flow set-parent` | write |
| `flow_start` | `gitpoulpi flow start` | write, network |
| `flow_status` | `gitpoulpi flow status` | read |
| `flow_sync` | `gitpoulpi flow sync` | destructive |
| `flow_templates` | `gitpoulpi flow templates` | read |
| `hosting_board_move` | `gitpoulpi hosting board move` | write, network |
| `hosting_board_show` | `gitpoulpi hosting board show` | read, network |
| `hosting_checkout_pr` | `gitpoulpi hosting checkout-pr` | write, network |
| `hosting_clone` | `gitpoulpi hosting clone` | write, network |
| `hosting_edit_url` | `gitpoulpi hosting edit-url` | read, network |
| `hosting_file` | `gitpoulpi hosting file` | read, network |
| `hosting_integrations_add` | `gitpoulpi hosting integrations add` | write |
| `hosting_integrations_list` | `gitpoulpi hosting integrations list` | read |
| `hosting_integrations_radar` | `gitpoulpi hosting integrations radar` | write |
| `hosting_integrations_remove` | `gitpoulpi hosting integrations remove` | destructive |
| `hosting_integrations_test` | `gitpoulpi hosting integrations test` | read, network |
| `hosting_issue_base` | `gitpoulpi hosting issue base` | write |
| `hosting_issue_close` | `gitpoulpi hosting issue close` | write |
| `hosting_issue_create` | `gitpoulpi hosting issue create` | write, network |
| `hosting_issue_edit` | `gitpoulpi hosting issue edit` | write, network |
| `hosting_issue_move` | `gitpoulpi hosting issue move` | write |
| `hosting_issue_options` | `gitpoulpi hosting issue options` | read, network |
| `hosting_issue_reopen` | `gitpoulpi hosting issue reopen` | write |
| `hosting_issues` | `gitpoulpi hosting issues` | read, network |
| `hosting_labels` | `gitpoulpi hosting labels` | read, network |
| `hosting_link` | `gitpoulpi hosting link` | write |
| `hosting_list` | `gitpoulpi hosting list` | read |
| `hosting_logout` | `gitpoulpi hosting logout` | destructive |
| `hosting_members` | `gitpoulpi hosting members` | read, network |
| `hosting_milestones` | `gitpoulpi hosting milestones` | read, network |
| `hosting_open` | `gitpoulpi hosting open` | read |
| `hosting_pr` | `gitpoulpi hosting pr` | write, network |
| `hosting_pr_edit` | `gitpoulpi hosting pr-edit` | write, network |
| `hosting_pr_for_branch` | `gitpoulpi hosting pr-for-branch` | read, network |
| `hosting_projects` | `gitpoulpi hosting projects` | read, network |
| `hosting_prs` | `gitpoulpi hosting prs` | read, network |
| `hosting_radar` | `gitpoulpi hosting radar` | read, network |
| `hosting_refs` | `gitpoulpi hosting refs` | read |
| `hosting_review_abandon` | `gitpoulpi hosting review abandon` | write, network |
| `hosting_review_comment` | `gitpoulpi hosting review comment` | write, network |
| `hosting_review_diff` | `gitpoulpi hosting review diff` | read, network |
| `hosting_review_fetch` | `gitpoulpi hosting review fetch` | write, network |
| `hosting_review_push` | `gitpoulpi hosting review push` | write, network |
| `hosting_review_reply` | `gitpoulpi hosting review reply` | write, network |
| `hosting_review_resolve` | `gitpoulpi hosting review resolve` | write, network |
| `hosting_review_show` | `gitpoulpi hosting review show` | read, network |
| `hosting_review_submit` | `gitpoulpi hosting review submit` | write, network |
| `hosting_review_submit_change` | `gitpoulpi hosting review submit-change` | write, network |
| `hosting_review_suggest` | `gitpoulpi hosting review suggest` | read, network |
| `hosting_review_threads` | `gitpoulpi hosting review threads` | read, network |
| `hosting_ssh_key_delete` | `gitpoulpi hosting ssh-key delete` | destructive, network |
| `hosting_ssh_key_list` | `gitpoulpi hosting ssh-key list` | read, network |
| `hosting_use` | `gitpoulpi hosting use` | write |
| `ignore` | `gitpoulpi ignore` | write |
| `index_lock_remove` | `gitpoulpi index-lock remove` | destructive |
| `index_lock_show` | `gitpoulpi index-lock show` | read |
| `job_cancel` | `gitpoulpi job cancel` | write |
| `job_list` | `gitpoulpi job list` | read |
| `job_wait` | `gitpoulpi job wait` | read |
| `lfs_checkout` | `gitpoulpi lfs checkout` | write |
| `lfs_fetch` | `gitpoulpi lfs fetch` | write, network |
| `lfs_files` | `gitpoulpi lfs files` | read |
| `lfs_install` | `gitpoulpi lfs install` | write |
| `lfs_lock` | `gitpoulpi lfs lock` | write, network |
| `lfs_locks` | `gitpoulpi lfs locks` | read, network |
| `lfs_prune` | `gitpoulpi lfs prune` | destructive |
| `lfs_pull` | `gitpoulpi lfs pull` | write, network |
| `lfs_push` | `gitpoulpi lfs push` | destructive, network |
| `lfs_status` | `gitpoulpi lfs status` | read |
| `lfs_track` | `gitpoulpi lfs track` | write |
| `lfs_unlock` | `gitpoulpi lfs unlock` | destructive, network |
| `lfs_untrack` | `gitpoulpi lfs untrack` | write |
| `licenses` | `gitpoulpi licenses` | read |
| `log` | `gitpoulpi log` | read |
| `lost_commits` | `gitpoulpi lost-commits` | read |
| `mailmap_add` | `gitpoulpi mailmap add` | write |
| `mailmap_check` | `gitpoulpi mailmap check` | read |
| `mailmap_list` | `gitpoulpi mailmap list` | read |
| `merge` | `gitpoulpi merge` | write |
| `patch_apply` | `gitpoulpi patch apply` | write |
| `patch_create` | `gitpoulpi patch create` | write |
| `path_status` | `gitpoulpi path status` | read |
| `pull` | `gitpoulpi pull` | write, network |
| `push` | `gitpoulpi push` | destructive, network |
| `rebase` | `gitpoulpi rebase` | destructive |
| `rebase_plan` | `gitpoulpi rebase-plan` | read |
| `redo` | `gitpoulpi redo` | destructive |
| `reflog` | `gitpoulpi reflog` | read |
| `reflog_restore` | `gitpoulpi reflog-restore` | destructive |
| `refs_hide` | `gitpoulpi refs hide` | write |
| `refs_show` | `gitpoulpi refs show` | write |
| `refs_solo` | `gitpoulpi refs solo` | write |
| `remote_add` | `gitpoulpi remote add` | write |
| `remote_clear_push_url` | `gitpoulpi remote clear-push-url` | write |
| `remote_default` | `gitpoulpi remote default` | write |
| `remote_list` | `gitpoulpi remote list` | read |
| `remote_remove` | `gitpoulpi remote remove` | destructive |
| `remote_rename` | `gitpoulpi remote rename` | write |
| `remote_set_url` | `gitpoulpi remote set-url` | write |
| `remote_group_abort` | `gitpoulpi remote-group abort` | destructive |
| `remote_group_continue` | `gitpoulpi remote-group continue` | destructive, network |
| `remote_group_plan` | `gitpoulpi remote-group plan` | read, network |
| `remote_group_retry` | `gitpoulpi remote-group retry` | write, network |
| `remote_group_status` | `gitpoulpi remote-group status` | read |
| `repo_close` | `gitpoulpi repo close` | write |
| `repo_discover` | `gitpoulpi repo discover` | read |
| `repo_init` | `gitpoulpi repo init` | write |
| `repo_list` | `gitpoulpi repo list` | read |
| `repo_open` | `gitpoulpi repo open` | write |
| `repo_recent` | `gitpoulpi repo recent` | read |
| `repo_auth_show` | `gitpoulpi repo-auth show` | read |
| `repo_auth_test` | `gitpoulpi repo-auth test` | write, network |
| `repo_config_set` | `gitpoulpi repo-config set` | write |
| `repo_config_show` | `gitpoulpi repo-config show` | read |
| `repo_config_unset` | `gitpoulpi repo-config unset` | write |
| `reset` | `gitpoulpi reset` | destructive |
| `revert` | `gitpoulpi revert` | write |
| `reword` | `gitpoulpi reword` | destructive |
| `rewrite_analyze` | `gitpoulpi rewrite analyze` | read |
| `rewrite_apply` | `gitpoulpi rewrite apply` | destructive |
| `rewrite_check_signing` | `gitpoulpi rewrite check-signing` | read |
| `rewrite_diff` | `gitpoulpi rewrite diff` | read |
| `rewrite_discard` | `gitpoulpi rewrite discard` | read |
| `rewrite_maps` | `gitpoulpi rewrite maps` | write |
| `rewrite_note` | `gitpoulpi rewrite note` | read |
| `rewrite_preview` | `gitpoulpi rewrite preview` | read |
| `rewrite_purge` | `gitpoulpi rewrite purge` | destructive |
| `rewrite_push` | `gitpoulpi rewrite push` | destructive, network |
| `rewrite_rows` | `gitpoulpi rewrite rows` | read |
| `rewrite_run` | `gitpoulpi rewrite run` | destructive |
| `rewrite_scan_secrets` | `gitpoulpi rewrite scan-secrets` | read |
| `rewrite_script_check` | `gitpoulpi rewrite script check` | read |
| `rewrite_script_test` | `gitpoulpi rewrite script test` | read |
| `rewrite_tools` | `gitpoulpi rewrite tools` | read |
| `rm_file` | `gitpoulpi rm-file` | destructive |
| `search` | `gitpoulpi search` | read |
| `session_delete` | `gitpoulpi session delete` | destructive |
| `session_list` | `gitpoulpi session list` | read |
| `session_new` | `gitpoulpi session new` | write |
| `session_rename` | `gitpoulpi session rename` | write |
| `show` | `gitpoulpi show` | read |
| `sparse_checkout_add` | `gitpoulpi sparse-checkout add` | write |
| `sparse_checkout_disable` | `gitpoulpi sparse-checkout disable` | write |
| `sparse_checkout_set` | `gitpoulpi sparse-checkout set` | write |
| `sparse_checkout_status` | `gitpoulpi sparse-checkout status` | read |
| `split_commit` | `gitpoulpi split-commit` | destructive |
| `split_source` | `gitpoulpi split-source` | read |
| `ssh_command_clear` | `gitpoulpi ssh-command clear` | write |
| `ssh_command_set` | `gitpoulpi ssh-command set` | write |
| `ssh_command_show` | `gitpoulpi ssh-command show` | read |
| `stage` | `gitpoulpi stage` | write |
| `stash_apply` | `gitpoulpi stash apply` | write |
| `stash_drop` | `gitpoulpi stash drop` | destructive |
| `stash_from_commit` | `gitpoulpi stash from-commit` | write |
| `stash_list` | `gitpoulpi stash list` | read |
| `stash_pop` | `gitpoulpi stash pop` | write |
| `stash_push` | `gitpoulpi stash push` | write |
| `status` | `gitpoulpi status` | read |
| `submodule_init` | `gitpoulpi submodule init` | write |
| `submodule_list` | `gitpoulpi submodule list` | read |
| `submodule_update` | `gitpoulpi submodule update` | write, network |
| `tag_annotate` | `gitpoulpi tag annotate` | write |
| `tag_create` | `gitpoulpi tag create` | write |
| `tag_delete` | `gitpoulpi tag delete` | destructive, network |
| `tag_delete_remote` | `gitpoulpi tag delete-remote` | destructive, network |
| `tag_fetch` | `gitpoulpi tag fetch` | write, network |
| `tag_list` | `gitpoulpi tag list` | read |
| `tag_push` | `gitpoulpi tag push` | write, network |
| `theme_export` | `gitpoulpi theme export` | read |
| `theme_list` | `gitpoulpi theme list` | read |
| `treat` | `gitpoulpi treat` | write |
| `ui_back` | `gitpoulpi ui back` | view |
| `ui_highlight` | `gitpoulpi ui highlight` | view |
| `ui_notify` | `gitpoulpi ui notify` | view |
| `ui_propose_branch` | `gitpoulpi ui propose branch` | propose |
| `ui_propose_commit_message` | `gitpoulpi ui propose commit-message` | propose |
| `ui_propose_resolution` | `gitpoulpi ui propose resolution` | propose |
| `ui_propose_reword` | `gitpoulpi ui propose reword` | propose |
| `ui_propose_rewrite` | `gitpoulpi ui propose rewrite` | propose |
| `ui_propose_split` | `gitpoulpi ui propose split` | propose |
| `ui_propose_tag` | `gitpoulpi ui propose tag` | propose |
| `ui_show_commit` | `gitpoulpi ui show commit` | view |
| `ui_show_conflict` | `gitpoulpi ui show conflict` | view |
| `ui_show_diff` | `gitpoulpi ui show diff` | view |
| `ui_show_file` | `gitpoulpi ui show file` | view |
| `ui_show_home` | `gitpoulpi ui show home` | view |
| `ui_show_issue` | `gitpoulpi ui show issue` | view |
| `ui_show_keys` | `gitpoulpi ui show keys` | view |
| `ui_show_palette` | `gitpoulpi ui show palette` | view |
| `ui_show_pr` | `gitpoulpi ui show pr` | view |
| `ui_show_ref` | `gitpoulpi ui show ref` | view |
| `ui_show_search` | `gitpoulpi ui show search` | view |
| `ui_show_settings` | `gitpoulpi ui show settings` | view |
| `ui_show_tab` | `gitpoulpi ui show tab` | view |
| `ui_show_wip` | `gitpoulpi ui show wip` | view |
| `ui_state` | `gitpoulpi ui state` | view |
| `ui_viewers` | `gitpoulpi ui viewers` | view |
| `undo` | `gitpoulpi undo` | destructive |
| `undo_forget` | `gitpoulpi undo-forget` | write |
| `undo_restore` | `gitpoulpi undo-restore` | write |
| `unstage` | `gitpoulpi unstage` | write |
| `workspace_add` | `gitpoulpi workspace add` | write |
| `workspace_create` | `gitpoulpi workspace create` | write |
| `workspace_delete` | `gitpoulpi workspace delete` | destructive |
| `workspace_list` | `gitpoulpi workspace list` | read |
| `workspace_remove` | `gitpoulpi workspace remove` | write |
| `workspace_rename` | `gitpoulpi workspace rename` | write |
| `workspace_set_ai_instructions` | `gitpoulpi workspace set-ai-instructions` | write |
| `workspace_set_profile` | `gitpoulpi workspace set-profile` | write |
| `workspace_show` | `gitpoulpi workspace show` | read |
| `worktree_add` | `gitpoulpi worktree add` | write |
| `worktree_changes` | `gitpoulpi worktree changes` | read |
| `worktree_list` | `gitpoulpi worktree list` | read |
| `worktree_remove` | `gitpoulpi worktree remove` | destructive |

Not exposed as tools:

| CLI | Why |
|---|---|
| `gitpoulpi hosting integrations trust` | accepting a changed address, account or certificate is the user's decision; an agent is told the security check needs the user |
| `gitpoulpi ai trust` | binding an API key to another address is the user's decision |
| `gitpoulpi mcp serve` | the MCP server cannot spawn/serve a copy of itself |
| `gitpoulpi mcp token` | prints, copies and rotates the key that guards this very server: an agent must not read or replace it |
| `gitpoulpi completions` | shell completion generation, not a GitBackend operation |
| `gitpoulpi shell-init` | prints a wrapper function for the user's own shell, not a GitBackend operation |
| `gitpoulpi desktop shortcut status` | launcher shortcuts are the user's desktop, not a repository; they need the TUI's icon renderer |
| `gitpoulpi desktop shortcut install` | writes files into the user's launcher, Start menu and Desktop |
| `gitpoulpi desktop shortcut remove` | deletes files from the user's launcher, Start menu and Desktop |
| `gitpoulpi desktop folder-menu status` | the file manager menu is the user's desktop, not a repository |
| `gitpoulpi desktop folder-menu install` | writes entries into the user's file manager menus and registry |
| `gitpoulpi desktop folder-menu remove` | deletes entries from the user's file manager menus and registry |
| `gitpoulpi alias create` | writes a command into the user's PATH; an agent never changes the user's shell |
| `gitpoulpi alias remove` | deletes a command from the user's PATH; an agent never changes the user's shell |
| `gitpoulpi path add` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi path remove` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi uninstall` | removes GitPoulpi's own program files, aliases, PATH entries and shortcuts from the machine |
| `gitpoulpi launch` | starts a terminal window on the user's desktop; the launcher shortcuts' own entry point |
| `gitpoulpi daemon run` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon start` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon stop` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon restart` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon status` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi rewrite script trust` | trusting an unsandboxed script is a person's decision; an agent can only run scripts a person trusted |
| `gitpoulpi config set` | an agent must not be able to rewrite its own gating config |
| `gitpoulpi hosting ssh-key create` | writes a private key and uploads it; too dangerous to let an agent automate |
| `gitpoulpi hosting login` | takes a secret token on stdin; a token must not pass through an agent's tool-call transcript |
| `gitpoulpi repo-auth set` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi repo-auth clear` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi flow trust-hooks` | trusting the shell commands a repository commits is a person's decision; an agent can only run hooks a person trusted |
<!-- mcp-tools:end -->

### Branching workflow actions are plan-then-execute

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` and `flow_set_parent`
([Branching workflows](branching-workflows.md#actions)) never run on a bare
call. `dry_run: true` returns the plan (every git step and command, warnings,
what Undo cannot take back, blockers) as a normal result; `yes: true` runs it;
with neither, the call is refused and carries the plan. The workflow file's
`[hooks]` are shell commands from a file the team shares, so they run only with
an explicit `run_hooks: true`: the plan lists them (`hooks`, `hooks_run`) so you
can show the user the exact commands first. A merge that stops on conflicts
answers `status: "conflicted"` (an error result, exit code 2); after the
conflict tools resolved the files, call the same tool with `continue_: true`
or `abort: true`. Opening a merge request (`finish` in `merge-request` mode)
is not undoable and says so in the plan.

### What an agent does with plain `git` is in the undo history too

An agent that creates branches and worktrees, commits, resets or drops stashes
with its own `git` (not through these tools) is still visible: the daemon
diffs the repository's refs against what it last saw and records each change
as an `undo --list` entry with `"source": "outside"`, git's reason
(`outside.message`) and the committer git recorded. The `undo`, `redo` and
`undo_restore` tools take these entries like any other, so an agent (or the
user, with `Ctrl+Z`) can bring back a branch that was deleted. Edits to files
are never captured. See [Undo and redo](undo-redo.md#changes-made-outside-gitpoulpi).

## Showing things to the user

An agent can point at something instead of describing it: `ui_show_commit`,
`ui_show_diff` (a file at a line, inline or split), `ui_show_file` (content,
blame or history at a line), `ui_show_ref`, `ui_show_pr`, `ui_show_issue`,
`ui_show_conflict` (the merge editor on a conflicted file, at a conflict),
`ui_show_wip` (`composer` puts the cursor in the commit summary, typing nothing),
`ui_show_tab`, `ui_show_home`, `ui_show_search`, `ui_show_palette` (the query
typed, never run), `ui_show_keys`, `ui_show_settings` (a page and a setting,
never edited), `ui_highlight` (tint lines of the open diff or file),
`ui_notify` (a toast) and `ui_back` (undo the agent's last view change).
`ui_state` reads what a window shows (`text: true` adds the visible text);
`ui_viewers` lists the windows.

These tools **never change the repository** and cannot type, click or edit a
setting. Revisions and paths are checked before anything reaches the window.

**What the user sees and decides**:

- `ask` (default), or whenever you are typing, a dialog or Settings is open, you
  are dragging, or you pressed a key in the last 1.5 s: the agent gets
  `deferred` at once and your status line shows a chip *"claude-code wants to show
  ..."*. `Ctrl+N Enter` or a click shows it; `Esc` or `Ctrl+N Backspace`
  dismisses it. It expires after 60 s.
- Applied: the view moves, a 4 s toast says who moved it, and the tab gets a
  purple `✦` until your next key. `Ctrl+N Backspace` (palette `Agent: Back`)
  goes back; the window remembers ten views.
- Nothing is ever unhidden or checked out: a commit your solo filter hides
  answers `partial` (`hidden_by_filter`); a PR or issue list that is not linked to
  a provider answers `hosting_not_linked`.

**Answers**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, with the window's `state` when
applied. CLI exit codes: 0 applied/partial/proposed, 3 deferred, 1 otherwise.
Calls are rate-limited (burst of 5, then one per second).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "The bug is the early return above."
$ gitpoulpi --json ui back
```

### Proposing a change (`ui_propose_*`)

With `allow_ui_propose = true`, an agent can open one of your dialogs
**pre-filled**, marked *✦ Proposed by <agent>*. Nothing happens until you press
that dialog's own confirm; you can edit first or cancel.

| Tool | Opens | You confirm with |
|---|---|---|
| `ui_propose_reword` (rev, message) | the Reword popup | *Update message* (a pushed commit still asks about force-pushing) |
| `ui_propose_commit_message` (summary, body) | the commit form, filled; never commits | the Commit button |
| `ui_propose_branch` (name, start point) | the inline New Branch box | `Enter` |
| `ui_propose_tag` (name, target, message) | the inline New Tag box (annotated when there is a message) | `Enter` |
| `ui_propose_split` (rev, plan file) | the split editor over the commit, filled from the plan | *Split* (`Ctrl+S`); `Esc` rejects |
| `ui_propose_rewrite` (recipe file) | the history rewrite workbench with the recipe's rules in it; nothing is previewed or applied | your own *Preview* then *Apply* (`Ctrl+Enter`); `Esc` rejects. A recipe with a script is refused, secret values are dropped |
| `ui_propose_resolution` (path, pick or content file) | the merge editor on a conflicted file, with the picks (or the whole text) already in the Output | *Save & mark resolved* (`Ctrl+S`); closing it rejects |

One proposal at a time; a commit-message proposal is refused while you have your
own draft in the composer, and a resolution proposal while the merge editor holds
unsaved work of yours or when the file's stages changed since the agent read them.
A resolution you change before confirming is reported as `accepted_edited`. The answer has a `proposal` id; what you decided is in
`ui_state`'s `proposals` (`offered`, `open`, `accepted`, `accepted_edited`,
`rejected`, `expired`, `failed`).

### When no window is open, or it is in another session

- A window belongs to the session it was started in. If the agent's session has
  none, `ui show` looks for **one** window in another session that has the
  repository open and uses it (`routed_session` in the answer). Several match:
  `no_viewer` naming them; pass `--session`.
- **Opening a window for the agent** is off by default. Set
  `[ui] launch` (Settings > Agents & MCP > *Open a window for agents*) to a terminal or
  multiplexer preset or a `custom` command; then a call with `launch: true`
  (CLI `gitpoulpi ui --launch show ...`) opens one and waits up to 10 s. With
  `ui_control = "auto"` it is also opened without the flag; with `ask` the agent
  gets `no_viewer` and should tell you.

| Preset | Command |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (your default terminal) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (new window), `zellij` (new pane), `cmux` (new workspace) | inside a running multiplexer, in the repository folder |
| `custom` | `[ui] launch_command`, e.g. `my-term --cd {repo} {gitpoulpi} --session {session}`; split into arguments, no shell |

If a preset does not work with your version of the program, use `custom`.

## Errors

- A malformed call (unknown tool or argument, a missing required one) is a
  JSON-RPC `-32602 invalid_params` error.
- A refusal, or a git or daemon error (bad revision, conflict, ...), is a tool
  error: `isError: true` with the message in `content` and
  `structuredContent.error`.
