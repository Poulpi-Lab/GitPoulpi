# Roadmap

**Why this file exists.** Until now the phase structure lived only in an
external planning document that was never committed. Code and docs cite it by
section - "§1.1 of the plan", "§6 of the plan", "the plan's §0/§7", "the plan's
Phase 3 (Hardening & performance) roadmap" - and a reader with no access to that
document had no way to resolve any of it. Several of those citations are load-
bearing: they are the only stated justification for decisions still in force.

This file is the reconstruction, assembled from every in-repo reference
(`grep -rn "the plan" docs crates`, `grep -rn "Phase [0-9]" docs crates`) plus
the ADRs in `architecture/decisions/`. Where a phase boundary is inferred rather
than quoted, it says so. **From here on this file is the authority, not the
external plan** - if the two disagree, fix this one or write an ADR.

## The surviving index of the external plan

These are the section numbers cited in the tree, and what each one is cited
*for*. They are kept as a decoder ring for older comments; nothing new should
cite them.

| Section | Cited by | Subject |
|---|---|---|
| §0, §7 | `docs/architecture/git-backend-trait.md` | Cross-platform constraints and the credential strategy (ADR-0004) |
| §1.1 | `docs/architecture/overview.md`, `crates/gitpoulpi-cli/src/cli.rs` | The daemon-owns-all-state architecture; CLI/MCP first-class from day one |
| §5 | `crates/gitpoulpi-cli/src/cli.rs` | Named sessions |
| §6 | `crates/gitpoulpi-mcp/src/main.rs` | The MCP server |

---

## Phase 0 - Workspace & scaffolding · **done**

Thirteen crates plus `xtask`, each with its module doc comment stating what it
is and what lands later; the full `clap` command surface parsed but unwired;
placeholder binaries for every frontend; CI green on `windows-latest` and
`ubuntu-latest` (required) and `macos-latest` (non-blocking).

Evidence: `docs/architecture/overview.md` ("required checks from Phase 0
onward"), and the crates' early doc comments.

## Phase 1 - MVP

As originally scoped, Phase 1 was everything needed for GitPoulpi to be a
complete client for one person on one repo, across **all three** frontends:

1. `GitBackend` defined in `gitpoulpi-core`, implemented twice (`Git2Backend`,
   `SystemGitBackend`), with the shared contract-test suite in
   `gitpoulpi-testkit` proving they agree (ADR-0001).
2. `gitpoulpi-daemon` + `gitpoulpi-ipc`: one daemon per named session owning all
   state, with a live event stream (ADR-0003).
3. The TUI as an IPC client: commit graph, refs sidebar, working-changes pane,
   diff, commit, the mouse/keyboard/palette interaction model (ADR-0002).
4. The CLI as an IPC client: every `GitBackend` operation as a subcommand with
   `--json`.
5. **The MCP server** as an IPC client, mirroring the CLI 1:1 - explicitly *in*
   the MVP rather than deferred (ADR-0003, "Ship `gitpoulpi-mcp` in Phase 1/MVP,
   not a later phase").
6. Config: `GITPOULPI_*` environment-variable overrides and full validation
   (`docs/user/config-reference.md`).
7. Credential resolution through the git credential helper (ADR-0004; the
   encrypted-file fallback was dropped).

> **Re-scoped 2026-09-19.** Items 5 and 6–7, plus the remaining CLI/session
> surface, have moved to Phase 2. Phase 1 is now the TUI and the machinery
> underneath it. See **ADR-0005** for the decision and its reasoning; the
> current Phase 1 definition is the section below.

### Phase 1 as it now stands - the TUI phase

**In scope**

- `GitBackend` × 2 backends + the contract suite. *Substantially done*: both
  backends are feature-complete for every operation, including interactive
  rebase (`rebase_plan`/`rebase_interactive`/`rebase_state`/`rebase_continue`/
  `rebase_skip`/`rebase_abort`), and the contract suite covers most of them
  (`fetch`/`pull`/`push`/`clone` and hunk/line staging are the gaps - see the
  frontend surface matrix in `architecture/git-backend-trait.md`).
- Daemon + IPC. *Done for dispatch*, with one real hole: no detached background
  process. Every daemon today lives inside the process that started it.
- The TUI. *The bulk of the remaining work*, and the reason for the re-scope.
  What works and what does not is tracked in
  `developer/where-we-are.md`; the operation-by-operation view is the frontend
  surface matrix.
- **Git LFS: done (2026-09-30).** Read on every backend with no program needed
  (`lfs_status`, `lfs_files`); install / track / untrack / prune / fetch / pull /
  push / locks through the system `git-lfs` (ADR-0009); clone downloads the objects;
  libgit2 is kept off the working tree in LFS repositories; CLI (`lfs ...`), MCP and
  the TUI (tag, diff card, banner, palette, file menu, locks, Preferences); a missing
  `git-lfs` is a typed, explained state everywhere, never a crash. User view:
  `user/lfs.md`. Non-goals: a bundled LFS implementation, `git lfs migrate`, custom
  transfer agents.
- File View, Blame and File History: done (backend, CLI/MCP, the TUI split
  History and the Blame); decisions in `developer/the done design
  ("history-blame"), open gaps in `developer/where-we-are.md`.
- Hunk- and line-level staging in the TUI. `StageTarget::{Hunk,Line}` already
  exists in the domain and the CLI already reaches it; this is one of the
  features a graphical client needs.
- `insta` golden-snapshot tests for the TUI, which `CLAUDE.md` mandates and
  which do not exist (`developer/testing.md` explains the current state).

**Explicitly deferred to Phase 2** (ADR-0005): the session/config/hosting CLI
command groups, `GITPOULPI_*` env overrides, and credential resolution beyond
what the installed `git` already does for us. (The MCP server, also
originally on this list, shipped 2026-09-28 - see below.)

## Phase 2 - Agent surface, hosting, interactive rebase

- **The MCP server** (`gitpoulpi-mcp`) - **done** (2026-09-28). SDK: `rmcp`
  (`server`+`transport-io` features, no `macros` - see
  `docs/architecture/decisions/0007-mcp-sdk-and-shared-command-layer.md`).
  A tool registry generated from `gitpoulpi-commands::cli::Cli`'s clap tree
  rather than hand-mirrored, running through the same `execute` the CLI uses
  (see `docs/user/mcp-and-agents.md`).
- **The rest of the CLI** (`session`, `config`, `refs`, `hosting`, worktrees),
  **config and credentials** (`config.toml` read live, token store),
  **hosting** (GitHub, GitLab incl. self-managed, then Azure DevOps, Bitbucket,
  Jira, Trello), **interactive rebase** and **named-session / per-repo-state
  persistence** - **all done**. The record is `architecture/session-and-state.md`
  and `developer/the done design.

## Phase 3 - Hardening, performance, packaging, release

Named "Hardening & performance" by the external plan, per
`docs/developer/release-process.md`.

- Version bumping across the workspace, `xtask`-driven packaging (Windows
  installer/winget; Linux deb/rpm/AUR or a static binary; Homebrew best-effort),
  changelog generation.
- `gitpoulpi diagnose` - a diagnostics bundle for bug reports
  (`docs/user/troubleshooting.md`).
- Performance work on large repositories, and whatever the TUI's own profiling
  turns up.

## How to read this alongside the other docs

- `developer/where-we-are.md` - the cold-start entry point. **Start there**,
  not here.
- `architecture/git-backend-trait.md` - which operations are native vs system
  git, and how the CLI/MCP surface maps onto `GitBackend`.
- `architecture/decisions/` - why the phases are shaped the way they are.
