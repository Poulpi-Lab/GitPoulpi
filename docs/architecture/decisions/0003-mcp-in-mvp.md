# 0003 - MCP server (and a background daemon) in the MVP, not deferred

## Status
Accepted - **point 1 amended by [0005](0005-phase-1-is-the-tui.md) (2026-09-19)**: `gitpoulpi-mcp` moves to Phase 2, and Phase 1's agent-facing surface is the CLI instead. Points 2 and 3 (the daemon, the IPC seam, daemon lifecycle) are unchanged and still in force.

## Context
Most tools bolt AI-agent integration on after the human-facing product is stable. This project's explicit requirement is the opposite: the CLI and MCP server must be usable by an AI agent from Phase 1, alongside the TUI, and a mutation made by an agent (via MCP) while the user has the TUI open must show up live in that TUI without a restart.

## Decision
1. Ship `gitpoulpi-mcp` in Phase 1/MVP, not a later phase.
2. To make "live in the TUI" structural rather than a convention every frontend has to independently honor, introduce a background daemon (`gitpoulpi-daemon`) per named session that owns all state (open repos, backends, session, operation log). `gitpoulpi-tui`, `gitpoulpi-cli`, and `gitpoulpi-mcp` are all IPC clients (`gitpoulpi-ipc`) over this daemon - none of them touch `GitBackend` directly.
3. Daemon lifecycle is explicit and user-controllable (`gitpoulpi daemon start|stop|restart|status`), but also auto-starts transparently on first client use so there's no manual "start the server" step for normal usage.

## Consequences
- MCP tools reuse `gitpoulpi-core` logic through the daemon, never re-implementing or shelling to the CLI - avoiding drift between what a human can do and what an agent can do.
- The daemon needs a config-gated mutation policy (`[mcp] allow_mutations`, `mutation_allowlist`) since an agent mutating real git history needs stricter guardrails than a human clicking in the TUI.
- This adds real complexity (an IPC protocol, a daemon lifecycle, cross-platform transport abstraction) up front instead of deferring it - accepted because retrofitting live cross-frontend consistency later would be far more invasive than building it in from Phase 0/1.
