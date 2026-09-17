# 0007 - MCP SDK choice, and a shared command layer between CLI and MCP

## Status
Accepted (2026-09-28).

## Context
Phase 2 §6 calls for `gitpoulpi-mcp` to become real: a tool per `GitBackend`-mirroring CLI subcommand, running through the same daemon a TUI session already uses (ADR-0003). Two open questions: which Rust MCP SDK, and how to expose ~60 tools without hand-writing (and inevitably drifting from) a second copy of the CLI's argument parsing.

## Decision
1. **SDK: [`rmcp`](https://crates.io/crates/rmcp)** (the official Rust MCP SDK), `server` + `transport-io` features only. It builds cleanly against this workspace's toolchain (rustc 1.93; rmcp's edition-2024 floor is met even though the workspace's own `rust-version` is 1.80) with one online fetch (neither `rmcp` nor `schemars` were in `Cargo.lock` or the local registry beforehand) and no extra system dependencies on Windows (pure Rust, on `tokio`). No fallback hand-rolled JSON-RPC layer was needed.
2. **No `macros` feature.** `rmcp`'s `#[tool]` macros want one Rust function per tool; with tools generated from a clap tree, that would mean writing the CLI's argument list a second time by hand, which is exactly the drift this design avoids. `gitpoulpi-mcp::server::GitPoulpiServer` implements `rmcp::ServerHandler` directly (`list_tools`/`call_tool`), reading a tool registry built by walking `Cli::command()` (`gitpoulpi-mcp::registry`).
3. **Extract `crates/gitpoulpi-commands`** out of `gitpoulpi-cli`: the clap `Cli` tree, the IPC `Session` client, and `execute` (one `GitBackend` mapping per subcommand). `gitpoulpi-cli::main` and `gitpoulpi-mcp::server::GitPoulpiServer::handle_call` both call `gitpoulpi_commands::execute` - an MCP tool call and the equivalent `gitpoulpi <subcommand>` invocation are, past argument parsing, the identical function call. Zero-drift by construction rather than by convention.
4. **`Session` gained `publish: bool`** (`Session::connect`) and `set_repo` (`Session::set_repo`). The CLI still connects unpublished per short-lived command (`publish: false`, matching its pre-existing behavior: an embedded daemon it starts does not publish its port, since a discovery file pointing at a process that is about to exit would cost every other client a failed connection first). The MCP server is long-lived like the TUI, so it connects `publish: true` (an already-open TUI on the same session joins it, and vice versa) and keeps **one** connection alive across many tool calls, retargeting it per call with `set_repo` rather than reconnecting (the CLI's embedded, unpublished daemons don't publish a port a second connection could find anyway).

## Consequences
- A CLI subcommand's argument, description or requiredness change is picked up by its MCP tool automatically the next time `registry::tools()` runs - nothing to update by hand, and `registry`'s own tests (`every_cli_leaf_is_a_tool_or_excluded`, `excluded_entries_name_real_cli_leaves`) fail loudly if a new CLI leaf is added without a decision (tool, or `EXCLUDED` with a reason) being made about it.
- JSON Schema for each tool's arguments is derived from clap metadata (`gitpoulpi-mcp::schema`) rather than from `schemars` derives on the domain types - `gitpoulpi-core` gains no new dependency.
- A handful of CLI leaves are deliberately excluded from the tool surface (daemon lifecycle, `mcp serve` itself, `config set`, unimplemented session/refs actions, `hosting ssh-key create`) - see `docs/user/mcp-and-agents.md` for the list and reasons.
- Per-call `session` (a different daemon than the one the server connected to at startup) is accepted in every tool's schema for forward compatibility but refused at call time today - the server holds one live connection, and switching it mid-session isn't implemented.
