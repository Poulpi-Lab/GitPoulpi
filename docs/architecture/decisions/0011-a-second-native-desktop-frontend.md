# 0011 - A second, native desktop frontend

## Status
Accepted (2026-11-28).

## Context
The terminal UI is the base interface of GitPoulpi and stays so. Some people prefer to work in a desktop environment, with windows, a pointer and system fonts, and would use GitPoulpi more if it offered that. Everything the terminal UI shows is already decided by the daemon (ADR 0006) and the pure logic around it, so a second frontend is mostly a second way of drawing the same facts.

## Decision
1. **The terminal UI stays the base interface.** Nothing here replaces, slows down or changes it. A release that ships a desktop frontend still ships the same terminal UI, CLI and MCP server.
2. **A native desktop frontend is added** for people who prefer a desktop environment. It is an IPC client of the same per-session daemon as the TUI, the CLI and the MCP server (ADR 0006): it owns no repository state, so a change made from any frontend shows up live in the others.
3. **No embedded web view.** The desktop frontend draws natively. The lightweight footprint the terminal UI is built for (no browser engine shipped with the program) applies to it as much.
4. **Shared, frontend-neutral logic lives in `gitpoulpi-app`.** What decides *what* is shown (the commit-graph lane model, stash rows and the Log's row arithmetic, lazy log paging) is one implementation used by both frontends. The crate has no dependency on `ratatui`, `crossterm` or any UI toolkit (a test fails if one appears), and it never names a real colour: the layout hands out neutral references (`ColorRef`) that each frontend resolves against its own theme. Rendering (glyphs, bitmaps, strokes), terminal caches and anything tied to a toolkit stay in the frontend.
5. **The desktop code lives in a `desktop/` directory with its own Cargo workspace and lock file**, so the toolkit's dependency tree never enters the main workspace, its lock file, its dependency audit or the terminal builds. It depends on the main workspace's crates by path.
6. **It is developed on a `desktop` branch until it reaches feature parity**, meaning everything except the features that only make sense in a terminal. `main` only receives the shared extraction (`gitpoulpi-app`), which has no behaviour change.
7. **UI settings are split per interface.** Appearance and layout settings that only one interface understands (fonts, pane sizes, terminal image protocols, desktop window state) live in that interface's own settings, not in a shared table that the other interface would have to ignore. Settings about repositories and behaviour stay shared.

## Consequences
- Moving logic out of `gitpoulpi-tui` into `gitpoulpi-app` is a refactor with no behaviour change: the TUI's tests, snapshots and PTY tests must pass unchanged. The first slice moved the commit-graph lane fold, the stash row placement with `LogRows`, and the lazy paging with `LogLoadSettings`; the TUI keeps a thin adapter that resolves `ColorRef` to its own colours.
- Every further extraction follows the same rule: move what is pure and belongs to the model, leave what draws.
- A feature added to the TUI that has a neutral part now has a home for it; the desktop frontend picks it up without a second implementation.
- Two frontends mean two sets of visual checks. The terminal checks of `docs/developer/visual-verification.md` stay as they are.
