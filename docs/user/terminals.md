**English** · [Čeština](cs/terminals.md) · [Deutsch](de/terminals.md) · [Español](es/terminals.md) · [Français](fr/terminals.md) · [Italiano](it/terminals.md) · [日本語](ja/terminals.md) · [한국어](ko/terminals.md) · [Nederlands](nl/terminals.md) · [Polski](pl/terminals.md) · [Português (Brasil)](pt-BR/terminals.md) · [Português (Portugal)](pt-PT/terminals.md) · [Русский](ru/terminals.md) · [Svenska](sv/terminals.md) · [Türkçe](tr/terminals.md) · [Українська](uk/terminals.md) · [简体中文](zh-CN/terminals.md)

# Choosing a terminal

GitPoulpi runs in any terminal with mouse support, but it looks and feels best in
one that draws pictures, shows true color and sends the full keyboard. This page
says what each terminal gives you on each OS, and which setting to change.

**How to read it.** The tables come from a test pass in **November 2026**: every
terminal was driven with raw escape sequences and, where it could run it, with
GitPoulpi itself, on Linux, Windows 11 and macOS 15. Versions are listed
[below](#versions-tested). Every cell is one of:

- **works**;
- **partial** (and what is missing);
- **does not work** (and why: the *terminal*, the *OS layer*, or *GitPoulpi*);
- **n/a** (the terminal does not exist on that OS);
- **not measured** (and why).

Multiplexers (tmux, zellij, screen, herdr, psmux, panes) have their own page:
[Multiplexers](multiplexers.md).

## Best picks

| OS | Best picks, in order | Why |
|---|---|---|
| **Windows** | **Windows Terminal**, then **WezTerm** and **Rio** | Windows Terminal draws the graph and the avatars as sixel pictures, anchored right, and is the launcher's default. WezTerm and Rio draw them as iTerm2 pictures, anchored right since the November 2026 ConPTY fix (see [Windows](#windows)). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty and Ghostty give pictures, the pointer shapes and the full keyboard. foot draws sixel pictures and has the full keyboard and the pointer shapes. |
| **macOS** | **iTerm2**, then **kitty** and **Ghostty** | iTerm2 3.7.7 draws pictures through the kitty protocol. Terminal.app works but has neither pictures nor true color. kitty and Ghostty match their Linux behavior (not measured on macOS). |

On any OS, **screen** and **zellij** hide the terminal's picture protocol from
GitPoulpi, so it draws characters; tmux passes sixel. See
[Multiplexers](multiplexers.md).

## What GitPoulpi uses

- **Pictures** (graph column, author avatars, image diff and File View): the
  **kitty graphics protocol** with Unicode placeholders, **iTerm2 inline images**
  (`OSC 1337`), or **sixel**. Detection is deny-by-default, from `TERM`,
  `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`,
  `WT_SESSION`, `KONSOLE_VERSION` and the multiplexer variables. When the
  environment names nobody (an xterm started with `-ti vt340`, a VS Code terminal
  with images on, any terminal reached over SSH, where `TERM_PROGRAM` is not
  forwarded), the terminal's own answers decide: GitPoulpi asks once at start-up
  (see "The start-up query" below) and uses the terminal's XTVERSION name (kitty,
  Ghostty, WezTerm, iTerm2, mintty, foot) or sixel in its device attributes.
  Inside a multiplexer the answers are ignored, because tmux claims sixel whatever
  sits behind it.
- **Which protocol where.** kitty, Ghostty and iTerm2 3.7.7 or newer: the kitty
  protocol. WezTerm and Rio: the iTerm2 protocol, blending the PNG's
  transparency. mintty and iTerm2 older than 3.7.7: the iTerm2 protocol with an
  opaque backdrop. Windows Terminal, foot, Tabby, Konsole and mlterm: **sixel**.
  Everything else: characters, and the avatar becomes the initials badge.
  `ui.image_protocol` forces one.
- **iTerm2 3.7.7.** Every `OSC 1337` inline image opens a modal "Allow
  Terminal-Initiated Display?" dialog that blocks the window until you tick
  "Remember my choice". The same pictures sent through the kitty protocol, Unicode
  placeholders included, draw without a question, so GitPoulpi uses the kitty
  protocol there. `ui.image_protocol = "iterm2"` still gives the inline images.
- **Sixel** is drawn for the **graph column** (lanes, nodes and, with
  `ui.avatars = "graph"`, the avatars inside the commit circles) and for the
  **avatars** elsewhere (Log, commit detail, File History, Blame, the image view). A
  sixel has no alpha, no ids and no scaling, so the picture is exactly `columns x
  cell width` by `rows x cell height` pixels, drawn only where something is, over
  blank cells that keep their background; it is re-encoded only when the lanes, the
  scroll position or the zoom change. A sixel has no delete: GitPoulpi writes text
  over its cells to remove it. Sixel needs the real cell size in pixels: Windows
  Terminal, Tabby and mintty answer `CSI 16 t` once at start-up and after a zoom
  (never blocking if they stay silent); elsewhere it comes from the terminal's
  window size, or from `GITPOULPI_TERMINAL_CELL_PX=10x20`. A terminal that draws
  sixel without saying so needs `ui.image_protocol = "sixel"`. Windows Terminal
  before 1.22 does not draw sixel; set `ui.image_protocol = "none"` there.
- **True color**: themes are RGB. `ui.color_depth = "auto"` drops to 256 colors on a
  `*-256color` `TERM` without `COLORTERM=truecolor` (and in Terminal.app, which has
  no true color), and to 16 colors on the Linux console.
- **Mouse**: SGR mouse reporting with buttons, drag and motion (hover feedback, drag
  and drop, right-click, double-click).
- **The start-up query.** Four questions go out in one write: the kitty keyboard flags
  (`CSI ? u`), the background color (`OSC 11`, only used for `ui.theme = system`), the
  terminal's name (XTVERSION, `CSI > 0 q`) and its device attributes (`CSI c`, last: every
  terminal answers it, so it closes the read). The read is one `poll` loop with a 100 ms
  deadline, finished before the keyboard reader starts. A terminal that answers nothing
  costs the 100 ms once and everything stays as the environment says. Unix only: on
  Windows the console reports keys itself and the environment decides. The answers are in
  the Activity Log's environment report (`start-up query:` line).
- **Keyboard**: the **kitty keyboard protocol** (disambiguate flag only) is requested at
  start-up. It makes `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` and `Ctrl+Shift+Z` arrive.
  Without it the legacy keys stay (`Ctrl+Y` redoes, the `Alt` variants replace some
  chords). See
  [Keybindings](keybindings.md#keys-the-terminal-cannot-send-and-the-kitty-keyboard-protocol).
- **Clipboard**: every copy action goes by one policy. Locally, the **system clipboard**
  of the machine GitPoulpi runs on, plus `OSC 52` to the terminal when the terminal is
  known to honor it (kitty, WezTerm, Ghostty, foot, Alacritty, Windows Terminal, Rio,
  Tabby, iTerm2); `OSC 52` is also the fallback when the system clipboard fails. **Over
  SSH, or with no display** (a Linux console, a container), only `OSC 52` is sent, and
  inside tmux it is sent wrapped for the passthrough as well as bare. The status line says
  "copied" only when the system clipboard took the text; what went to the terminal says
  "sent ... to the terminal clipboard (OSC 52, not confirmed)", because nothing confirms
  that the terminal obeyed. The Activity Log's environment report names the route in its
  `clipboard:` line.
- **Mouse pointer shape** (resize pointer over dividers, hand over clickable items, I-beam
  over text fields): `OSC 22`, a bonus where the terminal implements it. GitPoulpi always
  sends it (except on the Linux console and behind screen, zellij and herdr, which swallow
  or print it; behind tmux it is wrapped for the passthrough). Under `TERM_PROGRAM=mintty`
  it sends mintty's own cursor names (`hand`, `ibeam`, `sizewe`, ...), the only ones
  mintty reads. **Rio does not implement `OSC 22` on any OS**: its source parses the
  sequence and discards it, so no setting on either side changes that.
- **Text caret**: with `ui.text_cursor = "terminal"` (the default) the terminal's own cursor is
  put at the caret of the focused text field, as a blinking bar (`DECSCUSR`), and hidden
  elsewhere; your own cursor style comes back on exit. `drawn` paints GitPoulpi's own caret instead.
- **Focus events** are enabled (tooltips close when the window loses focus).
  **Bracketed paste** is requested at start-up and released on every way out (Unix), so a
  multi-line paste arrives as one paste: its line breaks never submit a field and its
  letters never fire shortcuts. On Windows the console has no paste brackets; see
  [Windows](#windows) for what GitPoulpi does instead.
  GitPoulpi does not request OSC 8 hyperlinks or undercurl.
  On the picture path only, each frame is wrapped in a synchronized update
  (mode 2026, ignored by terminals that do not know it) so a picture never flashes
  over a popup.
- **Emoji and symbols**: GitPoulpi only uses glyphs with default emoji presentation
  or single-width BMP shapes, because terminals and fonts disagree on variation
  selectors.

## The matrix

Rows are terminals, columns are the OS. "GitPoulpi" in a cell means GitPoulpi itself was
run there; "probe" means only raw escape sequences were. **n/a** cells: kitty has no
native Windows build (releases ship Linux and macOS); Ghostty has no Windows build
(only its libghostty library is cross-platform); foot is a Wayland terminal (Linux only);
xterm is an X11 client (Linux, and macOS through XQuartz); iTerm2 and Terminal.app are macOS
only; Windows Terminal, conhost and mintty are Windows only (mintty also under Cygwin and
MSYS2).

**Not measured on macOS: kitty, Ghostty, WezTerm, Rio and Warp.** They need a GPU
(Metal or OpenGL 3.3) and could not start in the macOS pass: each printed its own start-up
error. Their Linux results are the reference for what the same program does; rerun them
on a Mac with a GPU before relying on a macOS cell.

### Pictures: which protocol GitPoulpi picks, and does it show

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **works**: kitty protocol (GitPoulpi) | n/a | not measured (needs a GPU) |
| **Ghostty** | **works**: kitty protocol (GitPoulpi) | n/a | not measured (needs a GPU) |
| **WezTerm** | **works**: iTerm2 protocol with transparency (GitPoulpi). Placeholders draw squashed, so the kitty protocol is not used | **works**: iTerm2 protocol with transparency, graph and avatars on their rows (official 20240203 and the upstream PR #8092 build). Official 20240203 put the pictures on the wrong rows before GitPoulpi showed the cursor around them, see "The ConPTY limit". In the VM run of the official build the graph picture also sat one cell to the right (the first letter of each Log message hidden); the maintainer's own WezTerm does not show it, cause unknown (version, font or scaling) | not measured (needs a GPU) |
| **foot** | **works**: sixel with transparency (GitPoulpi) | n/a | n/a |
| **Rio** | **works**: iTerm2 protocol with transparency (GitPoulpi) | **works**: iTerm2 protocol with transparency, cell-exact at every font size tried (0.5.28; GitPoulpi). Sixel and kitty never arrive (ConPTY drops them), so there is nothing to choose | not measured (needs a GPU) |
| **Alacritty** | text: **works** (no image support in the terminal) | text: **partial** (no pictures; image diff as blocks) | text: **works** |
| **Tabby** | **works**: sixel with transparency (probe; GitPoulpi not run in Tabby on Linux) | **works**: sixel, aligned (GitPoulpi) | **works**: sixel, transparency kept (GitPoulpi) |
| **Warp** | not measured (it needs XWayland and mis-decodes synthetic keystrokes, so nothing could be typed) | **does not work** (terminal: no sixel, iTerm2 or kitty images); GitPoulpi not measured (could not be started from Warp's input box) | not measured (needs a GPU) |
| **xterm** | probe: sixel **works** with `-ti vt340`; GitPoulpi picks it from the device attributes (the start-up query came after the run), or set `ui.image_protocol = "sixel"` | n/a | text: **works**; sixel draws (probe) and needs `ui.image_protocol = "sixel"` |
| **iTerm2** | n/a | n/a | **works**: kitty protocol on 3.7.7 and newer, no dialog (GitPoulpi). Older versions get the inline images with an opaque backdrop. Sixel draws but paints unset pixels opaque, so `ui.image_protocol = "sixel"` gives a solid rectangle |
| **Terminal.app** | n/a | n/a | text: **works** (no pictures) |
| **Windows Terminal** | n/a | **works**: sixel (GitPoulpi, 1.25) | n/a |
| **conhost** | n/a | text: **partial** (character graph, half-block image diff) | n/a |
| **mintty** | n/a | **does not work** (terminal behind ConPTY: no picture is drawn at all, not even a plain `OSC 1337` one placed with the cursor shown; the graph column stays blank; text and palette are fine) | n/a |

### Mouse pointer shape (OSC 22)

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **works** | n/a | not measured (needs a GPU) |
| **Ghostty** | **works** | n/a | not measured (needs a GPU) |
| **WezTerm** | **does not work** (terminal: not implemented in the build tested) | official: **does not work** (terminal). Build with upstream WezTerm PR #8092, which adds pointer shapes: **works** (hand, I-beam, resize arrows) | not measured (needs a GPU) |
| **foot** | **works** | n/a | n/a |
| **Rio** | **does not work** (terminal: parsed and discarded) | **does not work** (terminal, same reason; not ConPTY) | **does not work** (terminal, same source; run only on Linux and Windows) |
| **Alacritty** | **does not work** (terminal) | **does not work** (terminal) | **does not work** (terminal) |
| **Tabby** | **does not work** (terminal: xterm.js) | **does not work** (terminal) | **does not work** (terminal) |
| **Warp** | not measured | **does not work** (terminal) | not measured |
| **xterm** | **does not work** (terminal: CSS names unknown) | n/a | not measured (the pointer was outside the window) |
| **iTerm2** | n/a | n/a | **does not work** (terminal) |
| **Terminal.app** | n/a | n/a | **does not work** (terminal) |
| **Windows Terminal** | n/a | **does not work** (terminal) | n/a |
| **conhost** | n/a | **does not work** (terminal) | n/a |
| **mintty** | n/a | **works** with mintty's own names; GitPoulpi sends them | n/a |

Where the cell says "does not work", the divider or button under the mouse still lights up.

### Keyboard protocol

GitPoulpi asks for the kitty keyboard protocol on Linux and macOS. On Windows it does not
ask: the console reports modifiers itself, and the chords below are limited by what the
terminal keeps for itself.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **works** | n/a | not measured (needs a GPU) |
| **Ghostty** | **works** | n/a | not measured (needs a GPU) |
| **WezTerm** | **partial**: no answer until `enable_kitty_keyboard = true` in `wezterm.lua` (default `false`) | **partial**: WezTerm keeps `Ctrl+Shift+N` (opens its window); `Ctrl+,` is not delivered (official build). The PR #8092 build keeps neither chord, but `Ctrl+,` still does not open Settings (no kitty keyboard answer through ConPTY) | not measured (needs a GPU) |
| **foot** | **works** | n/a | n/a |
| **Rio** | **works** (on by default) | **partial**: Rio keeps `Ctrl+Shift+N` (opens its window) | not measured (needs a GPU) |
| **Alacritty** | **works** (0.13 and later) | **partial**: `Ctrl+Shift+N` arrives as `Ctrl+N`, `Ctrl+,` is not delivered | **works** |
| **Tabby** | **does not work** (terminal: no answer; it keeps `Ctrl+Shift+N` and `Ctrl+,` for itself) | **partial**: Tabby keeps `Ctrl+Shift+N` and `Ctrl+,` | **does not work** (terminal: no answer); some chords did not arrive |
| **Warp** | not measured | not measured | not measured (needs a GPU) |
| **xterm** | **does not work** (terminal) | n/a | **does not work** (terminal) |
| **iTerm2** | n/a | n/a | **works** (3.7.7; check *Profiles > Keys* if it does not) |
| **Terminal.app** | n/a | n/a | **does not work** (terminal: no answer; legacy keys) |
| **Windows Terminal** | n/a | **partial**: WT keeps `Ctrl+Shift+N` (new window) and `Ctrl+,` (its settings) | n/a |
| **conhost** | n/a | not measured | n/a |
| **mintty** | n/a | **partial**: neither chord had a visible effect | n/a |

Where a chord does not arrive, use the `Alt` variants
([Keybindings](keybindings.md#keys-the-terminal-cannot-send-and-the-kitty-keyboard-protocol)),
or rebind the chord in *Settings > Keyboard & mouse*.

### Clipboard (copy actions)

A copy always uses the system clipboard when there is a local display, so these cells
describe the terminal's `OSC 52` path, which matters over SSH, in containers and as the
fallback. Every GitPoulpi copy test that used the system clipboard worked.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **works** | n/a | not measured (needs a GPU) |
| **Ghostty** | **works** (seen behind tmux, zellij, herdr); bare: not measured (a clipboard write needs keyboard focus, which the test setup did not give it) | n/a | not measured (needs a GPU) |
| **WezTerm** | **works** | **works** | not measured (needs a GPU) |
| **foot** | not measured (a write needs keyboard focus, which the test setup did not give it) | n/a | n/a |
| **Rio** | not measured (same reason) | **works** | not measured (needs a GPU) |
| **Alacritty** | not measured (same reason) | **does not work** (clipboard unchanged with the default config; not investigated) | **works** |
| **Tabby** | **works** | **works** | **works** |
| **Warp** | not measured | **does not work** (clipboard unchanged) | not measured (needs a GPU) |
| **xterm** | not measured | n/a | **does not work** (clipboard unchanged) |
| **iTerm2** | n/a | n/a | **does not work** until *Settings > General > Selection > Applications in terminal may access clipboard* is on |
| **Terminal.app** | n/a | n/a | **does not work** (terminal) |
| **Windows Terminal** | n/a | **works** (while focused) | n/a |
| **conhost** | n/a | not measured (no probe run) | n/a |
| **mintty** | n/a | **does not work** (clipboard unchanged with its default settings) | n/a |

### Paste

On Linux and macOS GitPoulpi requests bracketed paste, so a multi-line paste is one paste.
That was not run terminal by terminal (**not measured** per terminal). On Windows the console
has no paste brackets, so a terminal that does not ask first delivers the text as key
presses and every line break would be an `Enter`. GitPoulpi therefore treats key presses
that follow each other within 30 ms (an inner `Enter`, or eight or more characters) as one
paste. Held keys and "abc, `Enter`" typed by hand are not pastes.

| Terminal | Windows (two-line paste into the new-branch prompt) |
|---|---|
| **Windows Terminal** | **works**: WT asks "paste several lines?" first (its own warning), then no branch is created |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **works** (no branch created) |
| **Warp** | not measured (GitPoulpi could not be started inside it) |
| **conhost** | not measured (`Ctrl+V` and `Shift+Insert` sent by the test did not paste into the classic console) |

### Emoji and icon width

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **works** (wide cells aligned; kitty draws the gear U+2699 blank, a known font matter) | WezTerm, Rio, Tabby, Windows Terminal: **works**. Alacritty: **partial** (emoji as monochrome outlines) | Alacritty, Tabby: **works**. Others: not measured (needs a GPU) |
| **Tabby** | **works** | **works** | **works** |
| **Warp** | not measured | **works** | not measured |
| **xterm** | **partial** (boxes: the font) | n/a | **partial** (boxes: the font) |
| **iTerm2**, **Terminal.app** | n/a | n/a | **works** |
| **conhost** | n/a | **partial**: color emoji drawn, wide cells aligned | n/a |
| **mintty** | n/a | **works** | n/a |

### Versions tested

kitty 0.49.2 (macOS 0.50.0 did not start), Ghostty 1.4.0, WezTerm 20260716 on Linux and
20240203 (the last tagged release) on Windows, foot 1.28.0, Rio 0.5.28 (Linux, Windows) and
0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04, xterm 411 (Linux) and 378
(XQuartz), iTerm2 3.7.7, Terminal.app 455.1 (macOS 15.8), Windows Terminal 1.25.2733, mintty
3.8.3. Terminals change fast: a version newer than these may differ.

## Per terminal

### Windows

- **Windows Terminal.** The launcher's default and the best picture support on Windows:
  sixel graph and avatars, OSC 52, good emoji, and it answers the cell-size query. Pick a
  Nerd Font in *Settings > Profiles > Defaults > Appearance > Font face*. Hold `Shift` to
  select text while the mouse is captured. It keeps `Ctrl+Shift+N` and `Ctrl+,`: use the
  `Alt` variants or rebind. The pointer shape does not change.
- **WezTerm.** In `wezterm.lua`, set `config.enable_kitty_keyboard = true` (Linux and
  macOS builds) and a font with emoji fallback, for example
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Pictures use the iTerm2 protocol and nothing needs to be set: the last tagged release,
  20240203-110809 (Feb 2024), anchors them on the right rows (rechecked November 2026 with
  the current GitPoulpi; before the ConPTY fix described below it did not; on a VM the
  graph picture also sat one cell right of its column, which is not seen on the maintainer's
  machine). Upstream WezTerm PR #8092 adds pointer shapes.
- **Rio** (MSI, needs UAC). Pictures (graph, avatars, image diff) use the iTerm2 protocol
  with transparency and land exactly on their cells: measured on 0.5.28 at its default font
  and at 18 and 22.5 px (cells 8x16, 11x21 and 13x26 pixels, always whole pixels), a 20x10-cell and
  a 4x27-cell picture both came out cell-exact. Nothing needs to be set. The pseudo-console
  swallows the cell-size query, so GitPoulpi assumes Rio's cells are twice as tall as wide
  (true at all three sizes); with a font of another proportion set
  `GITPOULPI_TERMINAL_CELL_PX=WxH` (for example `10x24`) to get round commit markers. Rio
  has no pointer shapes on any OS.
- **Tabby.** Sixel graph and avatars, copy and paste work. Tabby keeps `Ctrl+Shift+N` and
  `Ctrl+,` for itself.
- **Alacritty.** Needs OpenGL 3.3. Character graph; copy through `OSC 52` was not on with the default config.
- **mintty** (Git Bash, MSYS2, Cygwin). Pointer shapes work (mintty's own names). Pictures
  do not show (the graph column stays blank; mintty 3.8.3 draws no `OSC 1337` picture behind
  ConPTY even when placed by a plain probe) and it answers no cell-size query there: set
  `ui.graph_renderer = "text"`.
- **Warp.** Not measured with GitPoulpi: it could not be started from Warp's input box. The
  terminal has no sixel, iTerm2 or kitty images.
- **conhost, cmd, a PowerShell window.** Character graph, half-block image diff. By default
  GitPoulpi started from such a window reopens itself in your chosen terminal; see
  "The relaunch rule".

**The ConPTY limit.** Windows runs every terminal's program behind a pseudo-console
(ConPTY). Windows Terminal and WezTerm builds that bundle a recent one pass the picture
sequences through; the one inside Windows (used by Rio and mintty), and the one bundled
with WezTerm 20240203, do not. What is lost: the DCS (sixel) and APC (kitty graphics)
sequences are dropped, so only the iTerm2 pictures (`OSC 1337`) arrive, which is why Rio and
WezTerm use them; `CSI 16 t` (cell size) is only answered in Windows Terminal and Tabby.
What was fixed: an `OSC 1337` crosses ConPTY at once and lands at the cursor ConPTY last
painted, and ConPTY paints the cursor only while it is visible. GitPoulpi keeps the cursor
hidden, so its pictures landed on stale rows (the graph over the footer, a torn screen after
scrolling). It now shows the cursor around each picture it writes on Windows, inside the same
write, and hides it again; Rio and WezTerm 20240203 anchor every picture on its cell
(`pty_conpty_picture_anchor` reads the cursor move behind ConPTY back). Nothing changes
off Windows or for sixel.

**Windows paste.** See the "Paste" table above: GitPoulpi folds a burst of key presses
into one paste, so a multi-line paste never submits a prompt in the terminals tested.

**The relaunch rule.** A double-click on the exe starts it in a classic console window
(conhost) of its own. When `ui.launch_terminal` names another terminal (default: Windows
Terminal), GitPoulpi reopens itself there and the first window closes
(`ui.relaunch_in_terminal`). It does that **only** when it is alone in a classic console
window. A terminal that runs the exe directly (`wt.exe gitpoulpi.exe`, WezTerm, Rio, Alacritty
`-e`, Tabby `run`) is where you want to be, and nothing is relaunched; from a shell it
is not alone, so nothing happens either. Turn it off with `ui.relaunch_in_terminal = false`.

### Linux

- **kitty.** Everything works with no setting. `Shift+click` is kitty's own, so use
  `Alt+click` to extend a selection ([Mouse](mouse-and-gestures.md)).
- **Ghostty.** Everything works with no setting. If icons show as holes, add an emoji
  or Nerd Font to `font-family` (repeat the key to add fallbacks).
- **WezTerm.** Pictures through the iTerm2 protocol, transparency blended. Set
  `enable_kitty_keyboard = true` for the full chords. No pointer shapes.
- **foot.** Sixel graph and avatars, pointer shapes, full keyboard, fast. Set
  `font=JetBrainsMono Nerd Font:size=11` in `foot.ini`. Copy through `OSC 52` is governed by
  `security.osc52`.
- **Rio.** Pictures through the iTerm2 protocol with transparency, full keyboard. No
  pointer shapes (Rio discards `OSC 22`).
- **Alacritty.** Character graph, kitty keyboard since 0.13. No image support.
- **Tabby.** Sixel pictures; no kitty keyboard, no pointer shapes (Electron, xterm.js). It
  keeps `Ctrl+Shift+N` and `Ctrl+,`.
- **xterm.** Start it as `xterm -u8 -ti vt340` for sixel (UTF-8 is needed for the box drawing).
  Boxes instead of emoji: the font.
- **Warp.** Not measured: it needs XWayland and, in the test, mis-decoded synthetic keystrokes.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Not part of the November 2026 pass. Work
  with characters and legacy keys: use the `Alt` variants and `Ctrl+Y` for redo. VTE
  ignores `OSC 52`, so copy over SSH does not reach the clipboard. An input method
  (IBus, Fcitx) may grab the `Ctrl+Space` stage toggle; rebind it in *Settings >
  Keyboard & mouse*. Konsole is recognized for sixel from its own documentation.
- **Wayland, Hyprland, Omarchy.** The pointer hint (`OSC 22`) works in kitty, Ghostty and
  foot. The Omarchy launcher entry goes through `xdg-terminal-exec`.

### macOS

- **iTerm2.** 3.7.7 and newer draw pictures through the kitty protocol, with no question.
  Settings that help:
  - *Settings > General > Selection > Applications in terminal may access clipboard*:
    needed for copy through `OSC 52` (otherwise iTerm2 shows "the terminal attempted to
    access the clipboard but it was denied").
  - *Profiles > Keys > Left Option key: Esc+*: needed for the `Alt` chords.
  - If you set `ui.image_protocol = "iterm2"` (or use a version older than 3.7.7), tick
    "Remember my choice" in the "Allow Terminal-Initiated Display?" dialog. The pictures
    then have an opaque backdrop (a visible rectangle if your background differs from the
    theme's; `ui.graph_renderer = "text"` removes it).
  - The kitty keyboard protocol is on by default (3.6.9 and newer).
- **Terminal.app.** 256 colors only: GitPoulpi picks `ui.color_depth = 256` itself and
  themes look flatter. No pictures, no kitty keyboard, no clipboard through `OSC 52`. Tick
  *Use Option as Meta key* for the `Alt` chords.
- **Alacritty, Tabby.** Alacritty: character graph, kitty keyboard, `OSC 52` copy; set
  `option_as_alt = "Both"` for the `Alt` chords. Tabby: sixel graph and avatars.
- **Option as Meta.** Option composes characters (Option+B types an integral sign) unless
  the terminal turns it into Meta: iTerm2 *Left Option key: Esc+*, Terminal.app *Use Option as
  Meta key*, Alacritty `option_as_alt`. GitPoulpi's `Alt` chords need it, and nowhere is it on
  by default.
- **kitty, Ghostty, WezTerm, Rio, Warp**: not measured on macOS (they need a GPU). Expect the
  Linux behavior.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`; sixel needs `ui.image_protocol = "sixel"`.

### VS Code and Warp

The VS Code terminal (`TERM_PROGRAM=vscode`) is not part of the pass. It is recognized
by its device attributes when `terminal.integrated.enableImages` is on, otherwise GitPoulpi draws
characters; `ui.image_protocol = "sixel"` forces the pictures. Warp: see the per-OS notes above.

## Multiplexers

tmux, zellij, screen, herdr, psmux and the terminals' own panes sit between GitPoulpi and the
terminal and change what passes. The short version: tmux passes sixel and, with
`allow-passthrough on`, pointer shapes and kitty pictures; zellij and screen pass
neither pictures nor pointer shapes; herdr passes kitty pictures. All the details and the
configuration to add are in [Multiplexers](multiplexers.md).

## What degrades, and how

| Missing | What you get instead |
|---|---|
| Picture protocol | Character graph. The avatar becomes the colored initials badge (one letter) whatever `ui.avatar_fallback` says: an identicon squeezed into text cells is unrecognizable. The image diff and File View use colored half-blocks, or an info card. |
| Truecolor | Colors are mapped to the 256 or 16 color palette (`ui.color_depth`). |
| Kitty keyboard protocol | Legacy keys; some chords arrive as another key. Settings warns, and you rebind or use the `Alt` variants. |
| OSC 52 | Locally nothing changes (the system clipboard is used). Over SSH or without a display, copy actions put nothing on the clipboard: select with the mouse (hold `Shift`) and use the terminal's own copy. |
| OSC 22 | The pointer does not change; the divider or button under the mouse still lights up. |
| Emoji font | Boxes or holes instead of icons; the graph and the text still work. Install a Nerd Font or an emoji font and list it as a fallback. |

## Settings to know

| Setting | Use |
|---|---|
| `ui.image_protocol` | `auto` (default), `kitty`, `iterm2`, `sixel`, `none`. Set `none` if you see garbage where pictures should be. Restart. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` is always correct (use it where the ConPTY limit applies). Restart. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` or `off`, if a terminal or multiplexer misbehaves. Restart. |
| `ui.text_cursor` | `terminal` (default) or `drawn`. |
| `ui.mouse_enabled` | Off leaves the mouse to the terminal. Restart. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | The terminal the OS shortcuts open, and whether a double-click reopens GitPoulpi there ([Settings](settings.md)). |

See also [Multiplexers](multiplexers.md), [Pictures](image-view.md),
[Keybindings](keybindings.md) and [Troubleshooting](troubleshooting.md).

## Sources

Documentation checked in October 2026, behavior tested in November 2026. Primary sources unless noted.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); kitty keyboard from the [terminfo.dev matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secondary)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secondary), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [escape sequence support](https://rioterm.com/docs/escape-sequence-support) (no `OSC 22` row), [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 from a [Warp OSC 52 advisory write-up](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secondary)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (no truecolor): [Apple discussion](https://discussions.apple.com/thread/255315820) (secondary)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Comparison sites (secondary, used only to cross-check): [Terminal Trove](https://terminaltrove.com/compare/terminals/)
