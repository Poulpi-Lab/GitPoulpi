# ADR-0006 - One session holds many repositories; the daemon stays one per session

**Status**: accepted
**Date**: 2026-09-22
**Supersedes**: nothing. **Amends**: the "Scope note" on `gitpoulpi_daemon::Daemon`, and
the default-session derivation described in `session-and-state.md`'s
"Session collision, closed".

## The question

A Git client keeps several repositories open at once, as tabs. GitPoulpi's
architecture principle 3 says *one daemon per named session owns all
state*. Those two sentences do not by themselves say whether

- **(A)** a session holds N repositories and its one daemon serves all of
  them, or
- **(B)** each repository gets its own session and therefore its own
  daemon, with the TUI holding N client connections.

Everything downstream - the tab bar, the recent-repositories list, how a
clone's progress reaches the window that asked for it, how many processes
are running on a developer's laptop at 5pm on a Friday - depends on the
answer, so it is written down before any of it was built.

## Decision

**(A). A named session holds N repositories. One daemon per session,
serving every repository in it.** A repository is not a session; it is a
*tab within* one.

## Why

**The daemon is already repo-agnostic, and deliberately so.** Every
`gitpoulpi_ipc::Request` carries `repo: Option<RepoId>`, `dispatch`
turns that into a `RepoRef` per request, and `server.rs` holds a
`RepoLocks` map keyed by `RepoId` - with a comment saying, in as many
words, that unrelated repositories dispatch concurrently. The daemon
holds *no* per-repository state that would have to be duplicated or
partitioned. (B) would not simplify anything inside the daemon; it would
only multiply the daemons.

**One daemon per repository is one process per repository.** The
motivating complaint about Electron-based clients is their footprint. Shipping
a Rust TUI that spawns a background process per open tab reintroduces the
same shape in a different language. Eight open repositories is eight
listening sockets, eight `.port` files, eight shutdown races.

**Principle 3's promise is about agreement, not about counting.** What it
buys is that a CLI mutation and a TUI view of the same repository cannot
disagree. That holds under (A) exactly as well as under (B), and for the
same reason: the `RepoId` is the key, and both frontends derive it from
the repository *root* (`474f469`). What (A) adds is that two repositories
opened side by side in one window also cannot disagree with each other - 
a submodule and its superproject, say, or two worktrees of one
repository, which is a case (B) actively makes worse by putting them in
two processes that each hold their own index lock.

**A clone has no repository yet.** `OperationKind::Clone` is the one
operation with `repo: None`; under (B) there is no session it naturally
belongs to, and the TUI's existing clone palette entry is *permanently
unavailable* for exactly this reason ("a TUI window is bound to one
`RepoRef` for its whole life"). Under (A) the answer is obvious: the
clone runs on the current session's daemon, and the resulting repository
is opened as a new tab in the same session.

## What this changes

1. **`gitpoulpi_config::default_session_name` stays as it is, and its
   meaning is narrowed.** It derives `<basename>-<hash>` from a
   repository root, which reads like "one session per repository" and is
   the strongest argument for (B). It is *not* that: it is the default
   *entry point* when a user types `gitpoulpi` inside a repository with
   no `-s`. The session it names then goes on to hold whatever other
   repositories the user opens into it. `-s work` remains the way to put
   several repositories on one session deliberately from the start, and
   is now also what a second `gitpoulpi` in another repository should
   pass if the user wants both in one window.

   The consequence worth stating plainly, because it will surprise
   someone: opening repository B as a tab inside a session that was
   derived from repository A means B is served by A's daemon. That is
   correct under (A), and it is why the tab bar shows the session name.

2. **The daemon's "Scope note" is now stale in one respect.** It says a
   real multi-repo registry "is session/tab work for later". That work is
   this ADR, and the conclusion is that no registry is needed: opening
   the repository per request, under a per-`RepoId` lock, *is* the
   registry. If profiling ever shows the per-request open costing real
   time, a cache keyed by `RepoId` goes in behind the same seam without
   any protocol change.

3. **Which repositories are open is UI state, and does not live in the
   daemon.** This is the one place where "the daemon owns all state"
   needs a boundary drawn, so: the daemon owns all *repository* state - 
   anything a `GitBackend` could tell you. The list of open tabs, their
   order, which one is focused, and the recent-repositories list are
   properties of a *window*, not of a repository, and they are persisted
   by `gitpoulpi-session` into the config directory
   (`sessions/<name>.toml` for tabs, `recent.toml` for recents).

   The test for which side of the line something falls on: *would two
   frontends disagreeing about it be a bug?* Two clients disagreeing
   about whether a file is staged is a bug - daemon. A TUI and a CLI
   disagreeing about which tab is focused is not a disagreement at all,
   because the CLI has no tabs - session file.

4. **Clone progress is a broadcast event.** `Event::CloneProgress`
   carries the destination path, so a window that did not start the
   clone can ignore it and the one that did can draw a bar. It is
   broadcast rather than sent only to the requesting connection because
   the daemon's event channel already broadcasts, and because a second
   window watching a long clone finish is useful rather than harmful.

## What was rejected, and why it is worth recording

- **A daemon per repository (B)**, above: more processes, no state
  simplification, no home for `clone`.
- **A daemon per repository with a session-level supervisor**: all of
  (B)'s costs plus a supervisor to write, to keep alive, and to debug.
- **Putting the tab list in the daemon.** It would make `gitpoulpi
  status` in a terminal able to change which tab an open TUI is showing,
  which is not a feature anybody asked for, and it would mean a headless
  CLI invocation resurrecting a window layout.

## Consequences for the protocol

Additive only. `OperationKind::Discover` is a new read;
`Event::CloneProgress` is a new event variant. Both ends ship together,
so an older client meeting a newer daemon is not a supported
configuration - and the `.port` discovery file is rewritten by each
daemon start, so a stale client is already the case that reconnects.
