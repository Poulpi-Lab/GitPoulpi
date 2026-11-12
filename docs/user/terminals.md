**English** · [Čeština](cs/terminals.md) · [Deutsch](de/terminals.md) · [Español](es/terminals.md) · [Français](fr/terminals.md) · [Italiano](it/terminals.md) · [日本語](ja/terminals.md) · [한국어](ko/terminals.md) · [Nederlands](nl/terminals.md) · [Polski](pl/terminals.md) · [Português (Brasil)](pt-BR/terminals.md) · [Português (Portugal)](pt-PT/terminals.md) · [Русский](ru/terminals.md) · [Svenska](sv/terminals.md) · [Türkçe](tr/terminals.md) · [Українська](uk/terminals.md) · [简体中文](zh-CN/terminals.md)

# Choosing a terminal

GitPoulpi runs in any terminal with mouse support, but it looks and feels best in
one that draws pictures, shows true color and sends the full keyboard. This page
ranks the terminals per OS, says what each one gives you, and which setting to
change. Facts were checked against each project's documentation, changelog or
issue tracker in October 2026 (see [Sources](#sources)); nothing here was tested on
every terminal. Anything marked **(unverified)** could not be confirmed
from a primary source.

## Best picks

| OS | Best picks, in order | Why |
|---|---|---|
| **Windows** | **WezTerm**, then **Windows Terminal** | WezTerm draws the graph and the avatars as pictures, with transparency. Windows Terminal has the best OS integration (the launcher's default); it draws the graph and the avatars as sixel pictures (no transparency). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, then **foot** | kitty and Ghostty give pictures and the full keyboard. WezTerm gives pictures too. foot draws the graph and the avatars as sixel pictures, is fast and has the full keyboard. On Omarchy, pick Ghostty or kitty for pictures. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, then **iTerm2** | Same reasoning. Terminal.app works but has neither pictures nor true color. |

On any OS, **zellij and screen** (and tmux, except for sixel) hide the terminal's picture protocol from
GitPoulpi, so it draws characters (see [Multiplexers](#multiplexers)).

## What GitPoulpi uses

- **Pictures** (graph column, author avatars, image diff and File View): the
  **kitty graphics protocol** with Unicode placeholders, **iTerm2 inline images**
  (`OSC 1337`), or **sixel**, in that order of preference. Detection is
  deny-by-default, from `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`,
  `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` and the
  multiplexer variables. When the environment names nobody (an xterm started with
  `-ti vt340`, a VS Code terminal with images on, any terminal reached over SSH, where
  `TERM_PROGRAM` is not forwarded), the terminal's own answers decide: GitPoulpi asks
  once at start-up, in a single write and a single read bounded to 100 ms (see "The start-up query" below), and uses the terminal's XTVERSION name
  (kitty, Ghostty, WezTerm, iTerm2, mintty, foot) or sixel in its device attributes.
  Inside a multiplexer the answers are ignored: tmux claims sixel whatever sits behind it.
  kitty and Ghostty use the kitty protocol. WezTerm uses
  the iTerm2 protocol (on purpose) and blends the PNG's transparency. iTerm2, Rio
  and mintty use the iTerm2 protocol with an opaque backdrop. Windows Terminal, foot,
  Konsole and mlterm use **sixel**. With none of them, avatars are the initials badge.
- **Sixel** is drawn for the **graph column** (lanes, nodes and, with `ui.avatars = "graph"`, the avatars inside the commit circles) and for the **avatars** elsewhere (Log, commit detail, File History, Blame, the image view). A sixel has no alpha, no ids and no scaling, so the picture is exactly `columns x cell width` by `rows x cell height` pixels, drawn only where something is (anti-aliased edges below half opacity are dropped), over blank cells that keep their background; the encoded picture is cached and re-encoded only when the lanes, the scroll position or the zoom change, and it is sent again only when a cell under it was repainted. A sixel has no delete: GitPoulpi writes text over its cells to remove it. Sixel needs the real cell size in pixels: Windows Terminal is asked for it once at start-up and after a zoom (`CSI 16 t`, answered in about 10 ms, never blocking if it stays silent); elsewhere it comes from the terminal's window size, or from `GITPOULPI_TERMINAL_CELL_PX=10x20`. A terminal that draws sixel is recognized by its device attributes (a VS Code terminal with `terminal.integrated.enableImages`, an xterm started with `-ti vt340`); one that draws it without saying so needs `ui.image_protocol = "sixel"`. Windows Terminal before 1.22 does not draw sixel; set `ui.image_protocol = "none"` there (the graph then stays characters, as in every terminal without pictures).
- **True color**: themes are RGB. `ui.color_depth = "auto"` drops to 256 colors on a
  `*-256color` `TERM` without `COLORTERM=truecolor` (and in Terminal.app), and to 16
  colors on the Linux console. kitty, WezTerm and Windows Terminal count as truecolor.
- **Mouse**: SGR mouse reporting with buttons, drag and motion (hover feedback, drag
  and drop, right-click, double-click).
- **The start-up query.** Four questions go out in one write: the kitty keyboard flags
  (`CSI ? u`), the background color (`OSC 11`, only used for `ui.theme = system`), the
  terminal's name (XTVERSION, `CSI > 0 q`) and its device attributes (`CSI c`, last: every
  terminal answers it, so it closes the read). The read is one `poll` loop on the
  controlling terminal with a 100 ms deadline, finished before the keyboard reader
  starts, so nothing is left running to eat a keystroke. A terminal that answers nothing
  costs the 100 ms once and everything stays as the environment says. Unix only: on
  Windows the console reports keys itself and the environment decides. The answers are in
  the environment report's `start-up query:` line.
- **Keyboard**: the **kitty keyboard protocol** (disambiguate flag only) is requested at
  start-up with a 100 ms timeout. It makes `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` and
  `Ctrl+Shift+Z` arrive. Without it the legacy keys stay (`Ctrl+Y` redoes, `Alt` variants
  replace some chords). See
  [Keybindings](keybindings.md#keys-the-terminal-cannot-send-and-the-kitty-keyboard-protocol).
- **Clipboard**: every copy action goes by one policy. Locally, the **system clipboard**
  of the machine GitPoulpi runs on, plus `OSC 52` to the terminal when the terminal is
  known to support it (kitty, WezTerm, Ghostty, foot, Alacritty, Windows Terminal, Rio,
  Tabby, iTerm2); `OSC 52` is also the fallback when the system clipboard fails. **Over
  SSH, or with no display** (a Linux console, a container), the system clipboard would be
  the remote one or does not exist, so only `OSC 52` is sent, and inside tmux it is sent
  wrapped for the passthrough as well as bare. The status line says "copied" only when the
  system clipboard took the text; what went to the terminal says "sent ... to the terminal
  clipboard (OSC 52, not confirmed)", because nothing confirms that the terminal obeyed
  (the Linux console, GNU screen and VTE ignore it). The Activity Log's environment report
  names the route in its `clipboard:` line.
- **Mouse pointer shape** (resize pointer over dividers, hand over clickable items):
  `OSC 22`, a bonus where it is implemented.
- **Text caret**: with `ui.text_cursor = "terminal"` (the default) the terminal's own cursor is
  put at the caret of the focused text field, as a blinking bar (`DECSCUSR`), and hidden
  elsewhere; your own cursor style comes back on exit. `drawn` paints GitPoulpi's own caret instead.
- **Focus events** are enabled (tooltips close when the window loses focus).
  **Bracketed paste** is requested at start-up and released on every way out (Unix; the Windows
  console input path has no paste brackets), so a multi-line paste arrives as one paste: its
  line breaks never submit a field and its letters never fire shortcuts.
  GitPoulpi does not request OSC 8 hyperlinks or undercurl.
  On the picture path only, each frame is wrapped in a synchronized update
  (mode 2026, ignored by terminals that do not know it) so a picture never flashes
  over a popup.
- **Emoji and symbols**: GitPoulpi only uses glyphs with default emoji presentation
  or single-width BMP shapes, because terminals and fonts disagree on variation
  selectors.

## Capability table

"Pictures" is what GitPoulpi draws in that terminal today (**kitty**, **iTerm2** or
**sixel** protocol, or **text**), not what the terminal could do.

| Terminal | OS | Pictures (graph, avatars, image diff) | Truecolor | Kitty keyboard (full chords) | OSC 52 copy | OSC 22 pointer |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | yes | yes | yes | yes |
| **Ghostty** (1.3) | Linux, macOS (no Windows build) | kitty | yes | yes | yes | yes, since 1.0 |
| **WezTerm** | Windows, Linux, macOS | iTerm2, with transparency | yes | only with `enable_kitty_keyboard = true` (default `false`) | yes (write only) | no (the pull request is still open) |
| **foot** (1.28) | Linux (Wayland) | sixel (graph and avatars; no kitty graphics) | yes | yes, since 1.10.3 | yes (`security.osc52`) | yes, since 1.12 |
| **Alacritty** (0.17) | all | text (no image support upstream) | yes | yes, since 0.13 | yes (write; paste off by default) | no (not implemented) |
| **Konsole** | Linux | sixel (graph and avatars) | yes | being added in 2026 (merge request !1188); release (unverified) | write-only, merged mid-2024 | (unverified) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | text | yes | no (not in a released VTE) | no (VTE ignores it) | no |
| **iTerm2** (3.7) | macOS | iTerm2, opaque backdrop | yes | yes, since 3.6.9 (check Profiles > Keys) | off by default: enable "Applications in terminal may access clipboard" | (unverified) |
| **Terminal.app** | macOS | text | no (256 colors) | no | (unverified) | (unverified) |
| **Windows Terminal** (1.25) | Windows | sixel (graph and avatars) | yes | yes, stable since 1.25 (Oct 2026); Preview from 1.25 (Mar 2026) | yes (write; only while focused) | no (not implemented) |
| **Rio** | Windows, Linux, macOS | iTerm2, opaque backdrop (it also speaks kitty graphics) | yes | yes (on by default) | yes | no (not implemented) |
| **Warp** | macOS, Linux, Windows | text (not detected) | yes | yes | yes (it honors the sequence) | (unverified) |
| **VS Code terminal** (xterm.js) | all | text (sixel and iTerm2 images exist behind `terminal.integrated.enableImages`; `ui.image_protocol = "sixel"` uses the sixel) | yes | (unverified) | (unverified) | no |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, opaque backdrop | yes | (unverified; not documented) | (unverified; not documented) | yes |
| **Windows console** (conhost, cmd, PowerShell window) | Windows | text | recent Windows 10/11 only | not needed: it reports modifiers itself | no | no |
| **ConEmu** | Windows | text | (unverified) | (unverified) | (unverified) | (unverified) |
| **tmux, zellij, screen** | all | text; sixel through tmux when the terminal behind it takes sixel (see [Multiplexers](#multiplexers)) | needs settings (zellij: yes) | not in tmux or screen | tmux: with `set-clipboard on`; zellij: yes | tmux: with `allow-passthrough on`; zellij, screen: no |

The "no (not implemented)" cells for OSC 22 come from absence in a project's
changelog and documentation plus user reports, not from a test; treat them as
"not documented". The page does not tabulate emoji and variation-selector width: it
differs per terminal and font, which is why GitPoulpi avoids VS16 glyphs (see
[What GitPoulpi uses](#what-gitpoulpi-uses)).

## Per terminal

### Windows

- **WezTerm.** The best picture support on Windows. Keep `ui.image_protocol` on `auto`.
  In `wezterm.lua`, set `config.enable_kitty_keyboard = true` for the full chords and a
  font with emoji fallback, for example
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  WezTerm's last tagged stable release is 20240203-110809-5046fc22 (Feb 2024); newer
  fixes ship in nightlies. WezTerm has no OSC 22 pointer shape yet.
- **Windows Terminal.** Truecolor, mouse, OSC 52 and good emoji. GitPoulpi draws the
  the graph and the avatars as sixel pictures (Windows Terminal 1.22 or later). Pick a Nerd Font in *Settings > Profiles > Defaults > Appearance > Font face*.
  Hold `Shift` to select text while the mouse is captured. The pointer shape does not
  change. On a build older than 1.25 (no kitty keyboard protocol), use the `Alt` variants.
- **Rio** works with an opaque picture backdrop; less tested.
- **mintty** (Git Bash, MSYS2, Cygwin) draws pictures through the iTerm2 protocol with an
  opaque backdrop; less tested.
- **Windows console, ConEmu, cmd**: avoid. Pictures are drawn badly or not at all.
  Double-clicking the exe reopens GitPoulpi in your chosen terminal
  (*Settings > Launcher*).

### Linux

- **kitty.** Everything works with no setting. `Shift+click` is kitty's own, so use
  `Alt+click` to extend a selection ([Mouse](mouse-and-gestures.md)).
- **Ghostty.** Everything works with no setting. If icons show as holes, add an emoji
  or Nerd Font to `font-family` (repeat the key to add fallbacks).
- **WezTerm.** Pictures through the iTerm2 protocol, transparency blended. Same
  `enable_kitty_keyboard` note as on Windows.
- **foot.** Sixel graph and avatars, full keyboard, fast. Set `font=JetBrainsMono Nerd Font:size=11` in `foot.ini`.
- **Alacritty.** Character graph, kitty keyboard since 0.13. Alacritty has no image
  support.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Work with characters and legacy keys: use
  the `Alt` variants and `Ctrl+Y` for redo (a recent Konsole may add the kitty keyboard
  protocol). VTE copy actions (`OSC 52`) put nothing on the clipboard. An input method (IBus, Fcitx) may grab the
  `Ctrl+Space` stage toggle - **(unverified)**; rebind it in *Settings > Keyboard & mouse*.
- **Wayland, Hyprland, Omarchy.** The mouse-pointer hint (`OSC 22`) works in kitty, Ghostty and
  foot. The Omarchy launcher entry goes through `xdg-terminal-exec`.

### macOS

- **kitty, Ghostty, WezTerm**: as on Linux.
- **iTerm2.** Pictures with an opaque backdrop (a visible rectangle if your background
  differs from the theme's; `ui.graph_renderer = "text"` removes it). Enable
  *Settings > General > Selection > Applications in terminal may access clipboard* for
  copy, and *Profiles > Keys > Left Option key: Esc+* for the `Alt` chords.
- **Terminal.app.** 256 colors only: GitPoulpi picks `ui.color_depth = 256` itself and
  themes look flatter. No pictures. Tick *Use Option as Meta key* for `Alt` chords.

### VS Code and Warp

The VS Code terminal (`TERM_PROGRAM=vscode`) and Warp are not recognized as picture
terminals, so GitPoulpi draws characters. Both are usable. In VS Code, turn on `terminal.integrated.enableImages` and set
`ui.image_protocol = "sixel"` for avatar pictures. Warp documents kitty keyboard protocol
support; the VS Code terminal's support is **(unverified)**.

## Multiplexers

GitPoulpi detects tmux, zellij, screen and herdr (`TMUX`, `ZELLIJ`, `STY`, `TERM=screen*`
or `tmux*`, `HERDR_*`). What reaches the terminal behind each (checked with raw escape
sequences, tmux 3.8, zellij 0.45.1, screen 5.0, herdr 0.9.5, November 2026):

| | Pictures | Pointer shape (OSC 22) | OSC 52 |
|---|---|---|---|
| **tmux** | sixel is drawn by tmux itself when the terminal behind it takes sixel (foot, WezTerm, Rio): detected, no setting. kitty pictures need `allow-passthrough on` and `ui.image_protocol = "kitty"` (kitty, Ghostty) | with `allow-passthrough on` GitPoulpi wraps it: seen in kitty and foot, not in Ghostty | yes, with `set-clipboard on` (off by default: nothing arrives) |
| **zellij** | text (the kitty protocol is accepted but not its Unicode placeholders; no sixel, no iTerm2) | no | yes |
| **screen** | text (the payload is printed) | no | no |
| **herdr** | kitty pictures (detected when `kitty_graphics = true`) | no | yes |

In `~/.tmux.conf`:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later: pointer shapes, forced kitty pictures
```

To force pictures through a multiplexer that forwards them, set `ui.image_protocol` to
`kitty`. herdr with `kitty_graphics = true`
inside kitty, Ghostty or WezTerm is detected automatically; see
[Troubleshooting](troubleshooting.md#the-graph-is-drawn-with-characters-not-as-a-picture-herdr-tmux-other-terminals).
`GITPOULPI_TERMINAL_IMAGE=off` turns all pictures off.

Keys: tmux does not answer the kitty keyboard query, and `extended-keys` does not bring
`Ctrl+Enter` or `Ctrl+,` through (measured with tmux 3.8 inside kitty: both arrive as
plain `Enter` and `,` with `extended-keys` off, on or always). GitPoulpi never asks tmux for
extended keys, so that setting changes nothing here: use the `Alt` variants inside tmux.
zellij 0.45 answers the kitty keyboard query, so the full keyboard works there.

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
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` is always correct. Restart. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | `auto` or `off`, if a terminal or multiplexer misbehaves. Restart. |
| `ui.mouse_enabled` | Off leaves the mouse to the terminal. Restart. |
| `ui.launch_terminal` | The terminal the OS shortcuts open ([Settings](settings.md)). |

See also [Pictures](image-view.md), [Keybindings](keybindings.md) and
[Troubleshooting](troubleshooting.md).

## Sources

Checked in October 2026. Primary sources unless noted.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); kitty keyboard from the [terminfo.dev matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol) (secondary)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (secondary), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52 from a [Warp OSC 52 advisory write-up](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) (secondary)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (no truecolor): [Apple discussion](https://discussions.apple.com/thread/255315820) (secondary)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Comparison sites (secondary, used only to cross-check): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Still unverified: sixel in Konsole and mlterm (taken from the projects' own feature lists, not re-checked against a release, and not tried on a real window in any terminal); OSC 22 in Konsole, iTerm2, Terminal.app and Warp; OSC 52 in Terminal.app,
VS Code and mintty; the kitty keyboard protocol in VS Code, mintty and ConEmu; every
ConEmu cell; and emoji width per terminal. The release in which Konsole ships the kitty
keyboard protocol was not confirmed.
