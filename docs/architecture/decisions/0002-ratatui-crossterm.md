# 0002 - ratatui + crossterm for the TUI

## Status
Accepted

## Context
The TUI must be genuinely cross-platform (Windows primary, Linux incl. Wayland/Hyprland, macOS best-effort), lightweight (the whole point is escaping Electron's footprint), and support both full keyboard control and mouse interaction (right-click menus, drag-and-drop, double-click).

## Decision
Use `ratatui` (the actively-maintained successor to `tui-rs`) with the `crossterm` backend. `termion` was considered and rejected - it's Unix-only, which fails the Windows-primary requirement outright. `cursive` was considered and rejected - it's a widget/form-oriented toolkit, less suited to a custom commit-graph/drag-and-drop-heavy UI than ratatui's immediate-mode rendering.

## Consequences
- `crossterm` talks to the terminal emulator, not the display server, so X11 vs. Wayland is a non-issue by construction - but terminal-emulator capability differences (mouse-reporting mode, true color) need feature detection and graceful degradation to keyboard-only.
- ratatui doesn't retain a widget tree, so mouse hit-testing requires the app to maintain its own per-frame `Rect` registry (`crates/gitpoulpi-tui/src/chrome/hit.rs`: hit lists built during the paint walk; see "UI rules every change must respect" in `../../developer/where-we-are.md`).
- No built-in mouse-capture story beyond crossterm's raw event stream - the TUI owns its own click/double-click/drag-state tracking.
