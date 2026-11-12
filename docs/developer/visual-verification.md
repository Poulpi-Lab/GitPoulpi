# Visually verifying the TUI

**Default: drive the TUI in a pty.** No window, focus, compositor or screenshot: deterministic, parallel, never touches the maintainer's desktop, and asserts on both the characters and the colours painted.

**Exception: a real window**, only for questions about *pixels* (see [When you really do need a window](#when-you-really-do-need-a-window)), under the [focus lock](#the-focus-lock). Desktop-driven verification once produced three false diagnoses in one day (a sibling agent's window captured, a hover that "did nothing" because the pointer was warped, a screenshot of the frame before the click); none of them exist in a pty.

## The pty driver

`xtask::pty` (`xtask/src/pty/`): `portable-pty` (real pty on Unix, ConPTY on Windows) + `vt100` (byte stream → cell grid with colours). No `cfg(windows)`/`cfg(unix)` anywhere in it: `cfg(unix)` on a test is a platform losing coverage, prefer a portable fixture. Used as a library by `crates/gitpoulpi-tui/tests/pty_*.rs`, and by hand:

```
cargo run -p xtask -- tui -          # script on stdin (or a file); --help lists commands
```
```
size 200 50
cwd /path/to/some/repo
env GITPOULPI_CONFIG_DIR /tmp/my-probe-config
spawn target/debug/gitpoulpi-tui -s my-probe
ready                       # wait for mouse capture (fine on a Unix pty; tests use first output + content)
wait Commit details         # wait for a label; prints its `col row`
click 186 2
find ~ many.txt
stable 300                  # wait until the grid stops changing
screen | styled | cell 41 20
```

- Coordinates are **zero-based `(col, row)`** like `ratatui::Rect`; the SGR wire protocol is one-based and converted once, in `xtask::pty::mouse`.
- **Always a private `GITPOULPI_CONFIG_DIR` and `-s` session name**: the daemon publishes its port in `<config>/sessions/<name>.port`, so without them your probe joins someone else's daemon and repo.
- Key specs: named keys (`Enter`, `PageDown`, `F5`), characters, `C-`/`M-`/`S-` prefixes. An unknown name is an error (a typo'd `PgDown` sent as six letters looks like an unwired shortcut). Control keys: `xtask::pty::keys::control`. Without the kitty keyboard protocol some chords cannot exist: `Ctrl+[` = Esc, `Ctrl+H` = Backspace, `Ctrl+I`/`J`/`M` = Tab/Enter, `Ctrl+]` arrives as `Ctrl+5` (why the pane mode is `Ctrl+N`).
- API (`Terminal`): `builder(..).spawn()`, `send_keys`/`send_text`, `click`/`right_click`/`double_click`/`move_to`/`drag`/`scroll`, `press`/`drag_to`/`release` (assert mid-drag), `screen()` (`cell`, `find`, `find_all`, `contains`), `wait_for`/`wait_until`/`wait_for_text`/`wait_for_stable`, `stop()` (also on `Drop`).

### Rules that each cost a red CI run or a false diagnosis

- **Never sleep; wait on a predicate.** `wait_until` polls every 5 ms and returns the instant it holds; a failure dumps the screen as it stood. Use `wait_for("what", ...)` so a CI timeout names the thing.
- **Readiness = what the program drew**, never terminal state. `wait_for_mouse_capture` is a trap: a pseudo-console may consume `\e[?1006h` instead of forwarding it. Use `wait_for_first_output`, then wait for something only the repo can paint (a fixture commit message, not a border: the chrome is drawn before the daemon answers).
- **"Bytes arrived" is not "the child drew".** ConPTY writes its own 27-byte preamble; the driver once counted it and asserted against a blank grid on Windows. `Terminal::bytes_drawn` discounts it.
- **The predicate must describe the whole state.** A terminal has no frame boundaries: a snapshot can be half old frame, half new, and a partial predicate can be satisfied by it (a divider test passed locally and failed on a slow runner). Reproduce tearing on demand: `GITPOULPI_PTY_TEAR_MS=10 cargo test -p gitpoulpi-tui --test <pty_x>`; run every new PTY test under it once.
- **An escape sequence goes out in ONE write** (`write_once` in `xtask/src/pty/input.rs` is the only writer). Split, crossterm reads the `ESC` as the Escape key and the rest as typed letters: "the click did nothing".
- `wait_for_stable` watches the grid, not bytes: this TUI redraws on a tick and never goes quiet.
- Mouse via SGR 1006 goes through the real dispatch and hit targets. `move_to` is a real motion event (hover works); a held button reports `Drag`, never `Moved`, so hover-driven highlights must also be repainted by the drag handler (a real bug). Double-click: both clicks in one write, always inside `DOUBLE_CLICK_WINDOW` (400 ms, `pane_layout.rs`).
- Fixed size (200x50), one `FixtureRepoBuilder` repo, own config dir and session per test, no hard-coded coordinates (locate by label), falsified when written.
- Failure messages distinguish: child exited (with its output) / alive but wrote nothing / wrote N bytes but the screen differs. `xtask/tests/pty_driver.rs` drives the trivial `xtask emit` child to tell a driver bug from a UI bug.

## When you really do need a window

Only for rendering questions a cell grid cannot answer:
- **Font metrics** (how many pixel rows a glyph lights; why the graph uses box-drawing, `commit-graph.md`).
- **Glyph presentation**: a text-default emoji flipping to emoji presentation under another style, a VS16-dependent glyph rendering as a hole or tofu. Both shipped as bugs; the pty reports the code point faithfully, the bug is in shaping. Avoid ambiguous-presentation glyphs (text-default ones need VS15 U+FE0E; never rely on VS16).
- **Cursor shape** (`SetCursorStyle` is a request), terminal-specific behaviour, pixel image protocols.

Everything else (hit targets, layout, colours, hover, staging) the pty answers better.

### User-guide screenshots

`scripts/docs-screenshots.sh` retakes `docs/user/images/*.png` in a private **headless sway** + kitty (remote control) + grim, on deterministic fixture repos and a temp config: it never touches the maintainer's desktop. The script header has the user-space sway recipe (`SWAY_BIN`/`SWAY_LIB`; `/tmp` copies vanish on reboot). ~15 min for all shots; rerun after visible UI changes.

The `clone-hosted` and `clone-hosted-search` shots (Clone dialog on a connected account) start the **fake hosting service** (`cargo run -p xtask -- mock-host --bare-root DIR`, see `testing.md`) and write two `[[integrations]]` profiles pointing at it into the temp config (`GPD_CONFIG_EXTRA`, tokens `GITPOULPI_TOKEN_HUB/LAB`); nothing leaves 127.0.0.1. The same server serves any ad hoc check of a hosted-account screen: run it, put `kind = "github_enterprise"` / `"gitlab_self_managed"` profiles with `url = "http://127.0.0.1:PORT"` in a temp config dir. The `open-with` shot needs applications registered for a PDF: `fake_desktop` (`shots.sh`) writes a `mime/globs2`, four `.desktop` files and a `mimeapps.list` under the temp root and points `XDG_DATA_DIRS` / `XDG_CONFIG_DIRS` at them, so the list never shows the machine's real applications. Traps the script now handles: the toolbar's LFS button only exists where LFS is initialized, so `lfs-menu` initializes it and runs last; a repository's link/issue source file is read from `.git/gitpoulpi/repo-state.toml` (the old `.gitpoulpi/` copy is only migrated once, so a second shot writing it was silently ignored); `[updates] mode = "off"` keeps an update-check line out of the Activity Log shot; log rows are truncated (`Merge branch 'feat...`), so click on a short prefix. The three Open in shots (`settings-open-in`, `settings-open-in-presets`, `open-in-menu`) run the TUI with `GPD_CLEAN_ENV=1` (`clean_env_args` in `shots.sh`): Open in detection reads the environment (`KITTY_WINDOW_ID`, `HERDR_*`, `TMUX`, `WAYLAND_DISPLAY`, `TERM_PROGRAM`...) and `PATH`, so the TUI process gets those variables unset (kitty sets its own `KITTY_*` inside the window, hence the explicit names) and a `PATH` holding only symlinks to git, git-lfs and basic tools plus harmless stubs for `tmux`, `wezterm` and `alacritty`; kitty still hosts the window. The `history-rewrite` shot picks `feature/moons` as the only scope (a pushed `main` is protected and refuses a preview) and never applies the rules. The `settings-keyboard-conflicts` shot creates its clash through `GPD_CONFIG_EXTRA` (`[keybindings.status] stage_toggle = "q"`). Screenshots are English only (the script has no per-language pass; the localized docs reuse the English images).

### The focus lock

A real window is global state (one pointer, one focus); two agents once collided and one reported on its sibling's window. Hold the lock for the whole launch-drive-capture run:

```
cargo run -p xtask -- desktop-lock -- ./my-verification-script.sh
```

It wraps a command (released when the child exits, whatever the reason). `--wait` (120 s) queues, `--stale` (600 s) breaks an abandoned lock.

### Linux / Wayland (Hyprland), if you must

Omarchy, Hyprland 0.56.x, `foot`, `grim`, `wtype`, `ydotool`.
- Hyprland dispatches in **Lua**: `hyprctl dispatch 'hl.dsp.focus({ window = "title:gp-x" })'`, `hl.dsp.window.move({ workspace = "2" })`, `hl.dsp.cursor.move({ x = 1200, y = 640 })`, `hl.dsp.window.resize({ x = 700, y = 950, exact = true })`. Syntax reference: `~/.config/hypr/bindings.lua`.
- **Never switch the maintainer's workspace.** Launch onto the active one, verify, leave.
- Launch detached or it dies with exit 144: `setsid foot --app-id=gp-x -T gp-x -e <cmd> </dev/null >/dev/null 2>&1 &`.
- Capture **your** window by the **pid you launched** (`hyprctl clients -j` → `at`/`size` → `grim -g`); a region capture photographs whatever is on top. `hl.dsp.focus` does not raise a tiled window over a floating one; `hl.dsp.window.pin()` does.
- Wait ~0.8 s after synthetic input before capturing (else the previous frame); if surprising, capture twice.
- Hover needs a **relative** `ydotool mousemove` (absolute is a warp: no motion event), crossing a cell boundary (approach over ~2 cells, 1 px at a time).
- `ydotool` socket path must be short (`/tmp/.ydotool_socket`, export `YDOTOOL_SOCKET`); its absolute axis is not layout coordinates (measure with `hyprctl cursorpos`). Pixels → layout: `win.at + pixel_offset / monitor.scale`, scale per monitor.
- **A private nested compositor** (headless sway with software rendering, a virtual pointer and keyboard) keeps a terminal x multiplexer sweep off the maintainer's compositor entirely; `grim -c` includes the pointer image, which is how a pointer shape (`OSC 22`) is seen. Drive each terminal by the pid you launched, read the terminal's replies from a raw-sequence probe first, GitPoulpi second. Clipboard write-backs (`wl-paste`) need a focused client: a terminal that "copies nothing" in such a setup is a setup limit, not a result.

### Windows, if you must

PowerShell + Win32 P/Invoke.
- Capture with **`PrintWindow(PW_RENDERFULLCONTENT)` by handle**, never `CopyFromScreen` (a screen region: whatever is on top). `SetForegroundWindow` is refused to background processes.
- Identify the window by **process lineage**: your `gitpoulpi-tui.exe` by binary path under your worktree's target dir → parent → `MainWindowHandle`, verified with `GetWindowThreadProcessId`. "Most recent wezterm-gui" has twice captured a sibling's UI. (`--always-new-process` is a `wezterm` flag, not `wezterm-gui`.)
- `PrintWindow` works on WezTerm's GPU surface on an unlocked session. Capture twice and keep the second (fresh/resized windows can return solid white/black); keep the window modest (110x34 worked where 200x50 came back black).
- `PostMessage(WM_CHAR)` delivers keys to a non-foreground window.
- **Pointer shape** (`OSC 22`) is checked with the real OS pointer: `GetCursorInfo` after moving the mouse over a cell, comparing the handle with the system cursors (ARROW, HAND, IBEAM, SIZEWE...). A grid of positions sweeps a whole screen in one run. A raw `printf` of the sequence in the same terminal is the control that tells the terminal's behavior apart from GitPoulpi's.
- **A locked workstation blocks every capture API** (secure desktop, by design: never work around it). Say so in the report instead of concluding a fix works; a colour defect once shipped because a locked session silently downgraded verification. The pty path is unaffected.

## Buffer-test caveats

- Untouched cells carry `bg: Some(Color::Reset)`, not `None`; the cell after a double-width emoji is blanked to reset but never emitted.
- **Falsify every test** (remove the fix, see red, restore): three tests here once passed while asserting nothing.
