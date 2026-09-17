# 0005 - Phase 1 is re-scoped around the TUI; MCP and the rest of the CLI/session surface move to Phase 2

## Status
Accepted, 2026-09-19. Amends ADR-0003 point 1 ("Ship `gitpoulpi-mcp` in Phase 1/MVP, not a later phase"). The rest of ADR-0003 - the daemon, the IPC seam, and the rule that no frontend touches `GitBackend` directly - stands unchanged. **Historical:** Phase 2 has since shipped `gitpoulpi-mcp` as a mirror of the shared command layer (ADR-0007); this ADR only explains the ordering. `crates/gitpoulpi-cli/src/commands.rs` mentioned below is now `crates/gitpoulpi-commands`.

## Context
ADR-0003 put the agent-facing surface in the MVP deliberately, to avoid the usual pattern of bolting AI integration onto a finished human product. That reasoning was about *architecture*, and it worked: the daemon exists, every frontend is an IPC client, and the CLI now reaches every `GitBackend` operation through it. None of that is being revisited.

What changed is the observation of which surface is actually being used. The TUI is the thing in daily use, and it is also the surface where the interaction model is still moving - panes, the palette, hit targets, the graph, column behavior and the diff view have all been reshaped repeatedly, several times in the same week. An MCP tool registry written against that surface today would be written against a moving target, and the cost of keeping it in step would be paid on every TUI change.

The mirroring argument cuts the other way too. `crates/gitpoulpi-cli/src/commands.rs` already maps every operation, with `--json` emitting the serialized domain types specifically so an MCP registry can mirror it arm for arm. That mirror is cheap to build *once*, and it gets cheaper the more settled the domain types are. Building it now buys an agent surface a few weeks early and costs a second place to update on every domain change during exactly the period when the domain is changing most.

Meanwhile an agent working in this repo is not blocked: the CLI is a real, scriptable, JSON-emitting frontend today (`skills/gitpoulpi/SKILL.md`), so the capability ADR-0003 wanted exists - just over a subprocess rather than over MCP.

## Decision
1. **Phase 1 is the TUI phase.** It covers `GitBackend` and its two backends, the contract suite, the daemon and IPC, and the TUI built on top of them - including the read-only backend gap (read-blob, blame, per-path log) and hunk/line staging, which are TUI features blocked on backend work.
2. **`gitpoulpi-mcp` moves to Phase 2.** It stays a mirror of the CLI's command mapping rather than a second mapping of its own, and it is built once the TUI's interaction model has settled.
3. **The remaining CLI and session surface moves with it**: the `session`, `config`, `refs` and `hosting` command groups, `GITPOULPI_*` environment overrides, config validation, and credential resolution beyond what the installed `git` already provides.
4. Nothing about the *architecture* is deferred. The daemon, the IPC protocol and the one-owner-of-state rule remain Phase 1 and remain the reason a later MCP server is a thin mapping instead of a rewrite.

## Consequences
- ADR-0003's "usable by an AI agent from Phase 1" requirement is met in Phase 1 by the **CLI**, not by MCP. That is a real reduction in ambition and is recorded as such rather than quietly reinterpreted.
- `docs/roadmap.md` is the phase definition of record; the MCP entry moves out of Phase 1 there.
- Every crate doc comment and doc page that says MCP "lands in Phase 1" is now wrong. They are corrected where they are touched rather than in one sweep, and `roadmap.md` is the tiebreaker in the meantime.
- The risk accepted: an agent-facing surface that arrives later than originally promised, and a mirror that must be written against whatever `commands.rs` looks like then. Mitigated by the CLI's JSON contract already being the domain types, which is the part that would otherwise drift.
- The risk avoided: a tool registry maintained in lockstep with a UI that is still being redesigned week to week.
