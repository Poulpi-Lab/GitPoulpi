---
name: gitpoulpi
description: Use GitPoulpi's MCP tools or `gitpoulpi --json` CLI instead of raw git in a repository the user works on with GitPoulpi, to get structured JSON, keep the user's open GitPoulpi window in sync, and show them commits, diffs and files in that window.
---

# GitPoulpi

GitPoulpi is a terminal git client. Its MCP server and its CLI expose the same
commands, run by the same daemon as the user's window, so what
you do appears live on their screen.

## When to use it

- The user runs GitPoulpi, asks for it, or the repository has a `.gitpoulpi/`
  folder: do git work through GitPoulpi rather than raw `git`.
- You want structured output: every command returns JSON (`--json` on the CLI,
  `structuredContent` over MCP), so never parse git's English.
- You want to show the user something in their window (a commit, a diff at a
  line, a file): the `ui_*` tools.
- Raw `git` is still fine for what GitPoulpi does not cover: `git config`,
  `bisect`, `gc`, hooks, plumbing.

## How to call it

- **MCP** (preferred when the `gitpoulpi` server is registered): one tool per
  CLI command, path joined with `_`: `status`, `log`, `branch_create`,
  `tag_delete_remote`, `ui_show_diff`. Every tool also takes `repo` (a path).
- **CLI** otherwise: `gitpoulpi --json <command>`; `--repo <path>` instead of
  `cd`. Errors are `{"error": "..."}` with exit code 1.
- Revisions accept anything git does (`HEAD~2`, a branch, a short sha).
- Nothing is interactive: a risky step needs an explicit flag (`--yes`,
  `--force-with-lease`) or is refused with a reason. Never wait for a prompt.
- `gitpoulpi <command> --help` lists every flag. Full lists:
  `docs/user/cli-reference.md` and `docs/user/mcp-and-agents.md`.

## Recipes

```
gitpoulpi --json status
gitpoulpi --json log --limit 50 [--path src/lib.rs]
gitpoulpi --json diff [--staged]
gitpoulpi --json stage --file src/main.rs [--hunk 0 [--lines 2,3]]
gitpoulpi --json commit -m "fix: handle empty input"
gitpoulpi --json undo --dry-run            # then: undo [--yes]
gitpoulpi --json lost-commits               # then, to get one's work back without committing it:
gitpoulpi --json apply-commit <rev>        # its changes onto the working tree, all or nothing (fails: nothing changed); undoable
gitpoulpi --json stash from-commit <rev> [-m "rescued"]   # or keep them as a stash; working tree untouched; undoable
gitpoulpi --json branch create feature HEAD~1
gitpoulpi --json branch upstream list      # a branch's primary upstream + extra push targets / pull sources, each ahead/behind (no network)
gitpoulpi --json branch upstream add gitlab[:dst] [--pull]   # extra push target (also remove, pull <r> --on|--off, primary <r/b>); undoable; exit 2 = refused
gitpoulpi --json checkout feature --autostash
gitpoulpi --json pull --mode ff-only
gitpoulpi --json push -u
gitpoulpi --json rebase-plan main          # build a plan, then:
gitpoulpi --json rebase main --interactive --plan plan.json
gitpoulpi --json split-source HEAD~1       # number a commit's hunks, then:
gitpoulpi --json split-commit HEAD~1 --plan split.json [--rest-message "chore: rest"] [--dry-run]
gitpoulpi --json rewrite preview --all-local --match-email old@example.com --set-name "Casey Example" --set-email casey@example.com --signatures strip
                                            # plan only (survives between commands, no daemon needed; `--all` prints every row); review it, then:
gitpoulpi --json rewrite apply --run RUN --digest DIGEST [--yes]   # one undo entry; exit 3 = pushed refs, ask the user
gitpoulpi --json rewrite run --recipe recipe.toml     # preview + apply in one call
gitpoulpi --json rewrite push --dry-run               # leased force push of the rewritten refs; protected branches only when named
gitpoulpi --json rewrite push --yes                   # destructive (network), not undoable; exit 4 = a ref was rejected
gitpoulpi --json rewrite note                         # text for the collaborators who have the old history
gitpoulpi --json rewrite purge                        # exit 3 = lists what is deleted for good and what blocks; --yes deletes the old history and its undo (MCP: only with [mcp] allow_history_purge)
gitpoulpi --json mailmap list --unmapped           # the names and emails of the history, with counts; `mapped_to` says what .mailmap already maps
gitpoulpi --json mailmap add --canonical "Alex Doe <alex@example.org>" --from "alex <alex@old.example.com>"   # fixes the Log's names without rewriting: edits .mailmap (not committed; undoable)
gitpoulpi --json conflicts list --names-only   # then: see "Resolving a conflict" below
gitpoulpi --json conflicts check            # would merging into the base branch conflict? (files, no side effect)
gitpoulpi --json hosting pr-for-branch     # the current branch's PR/MR, or null
gitpoulpi --json hosting pr --title "..." --body "..." [--draft]
gitpoulpi --json lfs status                # works even without git-lfs installed
```

- **Resolving a conflict** (merge, rebase, cherry-pick, revert stopped on it).
  Never edit the `<<<<<<<` markers by hand. Instead:
  1. `conflicts state` (which operation) and `conflicts list --names-only`
     (which files and what kind: `BothModified`, `DeletedByUs`, ...).
  2. `conflicts show <path>`: the labels and the numbered conflicts, each with
     its A (`ours`, the current side; during a rebase the branch you rebase
     onto) and B (`theirs`, the incoming side) lines.
  3. Resolve: `conflicts resolve <path> --pick 1=a,2=ba,3=b` (a, b, ab, ba,
     none per conflict; `--default a|b` for the rest), or `--both`,
     `--ours`/`--theirs` for the whole file, or write the final text to a file
     and `--content-file F`, or `--delete` to accept a deletion. A refusal
     (`stale`: the file changed since you looked, `not_conflicted`,
     `pick_incomplete`) exits 2 with `{"refused": {"reason", "message"}}`; re-run
     `show` and decide again.
  4. Then `rebase --continue` / `commit` as for the operation. `conflicts
     unresolve <path>` takes a file back; `conflicts abort` drops the whole
     operation (destructive: confirm with the user first).
  When the user is at the window, prefer showing it to them rather than
  resolving behind their back: `ui_show_conflict` opens the merge editor on the
  file (at conflict N), and `ui_propose_resolution` (`pick` as in step 3, or a
  content file) opens it with your picks already in, for them to review and
  confirm with Save & Mark Resolved (needs `allow_ui_propose`). Binary, LFS,
  deleted-on-one-side, symlink and submodule conflicts have no lines to
  propose: `--side` / `--delete` on `conflicts resolve`, or show the card.
- **Undo exists**: `undo` takes back the last commit, amend, checkout, branch
  change, discard, ...; check with `undo --dry-run`, and tell the user they can
  press `Ctrl+Z` in the window too.
- **Auto-stash**: if local changes block a checkout, merge, rebase or pull, the
  call fails naming `--autostash` (unless the user chose `always`). Pass
  `--autostash` only when moving their uncommitted work aside is what they want.
  If putting it back conflicts, the stash stays in the list and the error says
  so: report it, never retry or drop the stash yourself.
- **AI helpers** (`ai_commit_message`, `ai_explain`) only work when the user
  turned GitPoulpi's own AI on; you can usually write the message yourself.
- **Hosting**: `hosting prs` / `issues` need the repository linked
  (`hosting link pulls|issues --id <profile>`). You can never log in: if a token
  is missing, ask the user to run `gitpoulpi hosting login --id <id>
  --token-stdin` or set `GITPOULPI_TOKEN_<ID>`.
- **LFS**: anything but `lfs status` / `lfs files` needs `git-lfs`; if it is
  missing, tell the user to install it (https://git-lfs.com). `lfs track` leaves
  `.gitattributes` unstaged: commit it.

## What you may do (the user decides, in `config.toml` `[mcp]`)

| Setting | Default | If it refuses you |
|---|---|---|
| `allow_mutations` | `false` | You are read-only (the default): describe the change, or propose it (below); the user can set it to `true`. |
| `mutation_allowlist` | empty | Only the listed tools may change things. |
| `allow_ui` / `ui_control` | `true` / `ask` | You cannot move the window; just describe. |
| `allow_ui_propose` | `false` | Ask the user; do not work around it with the mutating tools. |

A refusal is an error naming the setting. Never try to change these settings
(`config set` is not available to you), and never ask for or handle a token.
These gates apply to the MCP server; with the CLI, act as carefully as if they
applied.

## Showing things to the user

`ui_show_commit`, `ui_show_diff` (`path`, `rev` or `unstaged`/`staged`,
`line`), `ui_show_file` (`blame`/`history`, `line`), `ui_show_ref`,
`ui_show_pr`, `ui_show_issue`, `ui_show_wip`, `ui_show_conflict` (the merge
editor on a conflicted file, at a conflict), `ui_show_settings`,
`ui_show_palette`, `ui_highlight` (lines of the open diff/file), `ui_notify`,
`ui_back` (CLI: `gitpoulpi --json ui show ...`). They only move the view.

- `applied` / `partial`: done. `deferred`: the user sees a chip and decides
  (default `ask` mode, or they are typing); mention it in your reply and carry on.
  Never repeat or loop.
- `no_viewer`: no GitPoulpi window is open; tell the user. Pass `launch: true`
  only if they asked to see it and set up `[ui] launch`.
- `hosting_not_linked`, `hidden_by_filter`: say so; do not link or unhide.
- `ui_state` (with `text: true`) tells you what is on screen. Use sparingly.

## Proposing a change for the user to confirm

When the user should review text you wrote, prefer a proposal to the direct
tool: `ui_propose_commit_message` (summary, body) over `commit`,
`ui_propose_reword` over `reword`, `ui_propose_branch` / `ui_propose_tag` over
`branch_create` / `tag_create`, and `ui_propose_resolution` (`path`, `pick` like
`conflicts resolve --pick`, or a content file) over `conflicts_resolve` when the
user should look at a merge resolution before it is written: the merge editor
opens with it filled in and only their Save & Mark Resolved applies it. The dialog opens pre-filled; only the user's own
confirm applies it. Needs `allow_ui_propose = true`. The answer's `proposal` id
shows up later in `ui_state`'s `proposals` (`accepted`, `accepted_edited`,
`rejected`, ...). One at a time; never re-send a rejected one.
