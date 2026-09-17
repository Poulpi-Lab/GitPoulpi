# 0001 - Pluggable Git backend (git2 + CLI fallback, plus a standalone system-git backend)

## Status
Accepted

## Context
A closed-source git integration would be opaque to us. We need a Rust implementation that's both fast for common operations and complete for the long tail of git functionality (interactive rebase, complex merges, signed commits). `git2` (libgit2 bindings) is fast and safe but doesn't cover everything git can do; shelling out to `git` covers everything but is slower and requires parsing output.

## Decision
Define `GitBackend` as a trait (`gitpoulpi-core`) with two implementations:
- `Git2Backend` - the default, a decorator over raw libgit2 calls that dispatches to an inner `SystemGitBackend` for operations git2 doesn't support well (see `docs/architecture/git-backend-trait.md` for the per-operation matrix).
- `SystemGitBackend` - drives the installed `git` binary directly, parsing only machine-readable output. Selectable standalone via `general.default_backend = "system"`, and reused as `Git2Backend`'s fallback target.

## Consequences
- Both implementations must behave identically for every operation implemented - enforced by a shared contract-test suite in `gitpoulpi-testkit`, run against both.
- The capability matrix in `git-backend-trait.md` is documentation of which operations git2 does natively and which it delegates to the system `git`; keep it current. (The former `CapabilitySet` code was removed; `GitBackend::name()` replaces it.)
- Users who don't trust or want git2 at all can select the pure system-git backend.
