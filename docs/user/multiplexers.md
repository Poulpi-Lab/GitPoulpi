**English**

# Multiplexers and panes

A multiplexer sits between GitPoulpi and your terminal: it redraws the cells itself and
forwards only the escape sequences it understands. So what works in the terminal alone
([Choosing a terminal](terminals.md)) may not work inside tmux, zellij, screen or herdr.
This page says what passes, and the configuration to add. Results come from the same
November 2026 test pass as the terminal tables: tmux 3.8 (MSYS2: 3.7c), zellij 0.45.1 and
0.46.0, GNU screen 5.0 (macOS ships 4.00.03), herdr 0.9.5, psmux 3.4.0.

GitPoulpi detects a multiplexer from `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` or `tmux*`
and `HERDR_*`. Inside one, pictures default to characters, except where the table below
says GitPoulpi detects a working path. `GITPOULPI_TERMINAL_IMAGE=off` turns all pictures
off, and `ui.image_protocol` forces a protocol.

## At a glance

| | Pictures | Pointer shape (OSC 22) | Clipboard (OSC 52) | Keys |
|---|---|---|---|---|
| **tmux** | sixel when the terminal behind takes it (detected); kitty pictures with `allow-passthrough on` and `ui.image_protocol = "kitty"` | with `allow-passthrough on`, in kitty and foot; not in Ghostty | with `set-clipboard on` | no kitty keyboard; `Ctrl+Enter` and `Ctrl+,` never arrive |
| **zellij** | characters | no | yes | kitty keyboard works |
| **GNU screen** | characters | no | no | legacy keys |
| **herdr** | kitty pictures (detected with `kitty_graphics = true`) | no | yes | not measured |
| **psmux** (Windows) | characters (by design) | no (Windows Terminal has none) | system clipboard works | not measured |
| **MSYS2 tmux** (Windows) | characters (by design) | no | system clipboard works | not measured; the screen is garbled |
| **WezTerm panes**, **Windows Terminal panes**, **kitty and Ghostty splits**, **iTerm2 split panes** | as the terminal itself | as the terminal | as the terminal | as the terminal |
| **iTerm2 `tmux -CC`** | characters | no | no (iTerm2 denies it by default) | not measured |

## tmux

**Pictures.**

- **Sixel** is drawn by tmux itself, re-encoded for a client that supports it. GitPoulpi
  asks tmux once at start-up (`tmux display-message -p '#{client_termfeatures}'`,
  bounded to 400 ms) and uses sixel when the client lists it. That is the case behind foot,
  WezTerm, Rio and Tabby; it is not the case behind kitty and Ghostty (tmux then prints
  "SIXEL IMAGE" text). Needs a tmux built with sixel (Arch's and Homebrew's are;
  check with `tmux -V` and a sixel test). No setting. tmux's own answer to a
  device-attributes query claims sixel whatever sits behind it, which is why GitPoulpi asks
  for the client's features instead.
- **iTerm2 behind tmux** advertises sixel but paints unset pixels opaque, so the mostly
  transparent graph would become a solid rectangle. GitPoulpi recognizes iTerm2
  (`LC_TERMINAL`, `ITERM_SESSION_ID`, the client type) and draws characters.
- **kitty pictures** pass through tmux's DCS passthrough (kitty, Ghostty; also seen in
  iTerm2 and Rio in a raw probe). They need `set -g allow-passthrough on` (tmux 3.3a or
  newer) **and** `ui.image_protocol = "kitty"` in GitPoulpi, which then wraps the uploads
  and deletes. Without the tmux option the wrapper is dropped and the graph stays blank.
- iTerm2 inline images pass through tmux but tmux cannot position them: GitPoulpi never
  uses them there.

**Pointer shape.** GitPoulpi wraps `OSC 22` for the passthrough. It reaches the window in
kitty and foot with `allow-passthrough on`. Ghostty does not show it when wrapped (the
arrow stays). WezTerm, Rio, Alacritty, Tabby, iTerm2 have no `OSC 22` to show.

**Clipboard.** `set -g set-clipboard on`. GitPoulpi also sends the passthrough-wrapped form
(it works with `allow-passthrough on`). Behind kitty, Ghostty, Alacritty and Tabby it
arrives; iTerm2 denies it until its clipboard permission is on.

**Keys.** tmux does not answer the kitty keyboard query, and `extended-keys` does not
bring `Ctrl+Enter` or `Ctrl+,` through: measured with tmux 3.8 inside kitty, both arrive
as plain `Enter` and `,` with `extended-keys` off, on or always. Letter chords such as
`Ctrl+Shift+Z` only arrive with `extended-keys` on or always **and**
`extended-keys-format csi-u` (tmux 3.5 or newer) **and** an application that asks for
extended keys; GitPoulpi never does, so the setting changes nothing here. Use the `Alt`
variants inside tmux
([Keybindings](keybindings.md#keys-the-terminal-cannot-send-and-the-kitty-keyboard-protocol)).

**Config** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # truecolor
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a or later: pointer shapes, forced kitty pictures
```

**tmux inside WSL.** Not measured. It is the Linux
build, so expect the same as above with Windows Terminal as the outer terminal.

## zellij

Pictures: zellij accepts the kitty protocol but not its Unicode placeholders, and carries
no sixel or iTerm2 images, so GitPoulpi draws characters (a deliberate veto). Pointer shape:
swallowed, so GitPoulpi does not send it. Clipboard: `OSC 52` is forwarded (works behind
kitty, Ghostty, Alacritty, Tabby; iTerm2 and Terminal.app deny or ignore it). Keys: zellij
answers the kitty keyboard query, so the full keyboard works.

zellij 0.46 opens a "Welcome to Zellij" popup on a fresh start: press `Esc`. A session that
exited keeps its name and the next start of the same name asks what to do.
No configuration is needed.

## GNU screen

Nothing passes: pictures and `OSC 22` are printed as characters or dropped, `OSC 52` is
ignored, and the keyboard is the legacy one. GitPoulpi draws characters. screen 4.00.03 (the
version macOS ships) also garbles colors and wide glyphs in the interface; screen 5 was
tested on Linux only. Use tmux or zellij for a usable session.

## herdr

herdr embeds a terminal that re-emits **kitty pictures**. GitPoulpi uses them when herdr's
`config.toml` says `kitty_graphics = true` (under `[terminal]`, or `[experimental]` on older
versions) and the terminal around herdr draws them (kitty, Ghostty, WezTerm): detected,
no setting. If the config lives elsewhere (for example under `HERDR_HOME`, which GitPoulpi
does not read) set `ui.image_protocol = "kitty"`. Over SSH (`herdr --remote`) GitPoulpi
cannot see the local client, so it draws characters; see
[Troubleshooting](troubleshooting.md#the-graph-is-drawn-with-characters-not-as-a-picture-herdr-tmux-other-terminals).
`OSC 22` is not forwarded by herdr 0.9.5, so GitPoulpi does not send it. `OSC 52` arrives
behind kitty, Ghostty, Alacritty and Tabby. In the raw probe herdr drew kitty pictures in
kitty, Ghostty and iTerm2, with the default config as well. Keys: not measured.

## psmux and MSYS2 tmux (Windows)

- **psmux** (Rust, ConPTY, tmux command language; `winget install psmux`) sets `TMUX`, so
  GitPoulpi draws characters there by design. Inside Windows Terminal it runs normally
  and copy works through the system clipboard; the pointer does not change (Windows
  Terminal has no `OSC 22`). Other host terminals were not measured.
- **tmux from MSYS2 or Cygwin** also sets `TMUX` (characters by design), and in the test the
  screen was garbled: stale frames drawn over each other in mintty and in Windows Terminal
  (with and without `-u`; without it box drawing printed as `qqqq`). A native Windows
  program inside an MSYS2 pseudo-terminal is not redrawn correctly by tmux. That is not
  fixable in GitPoulpi: use psmux, Windows Terminal panes or WezTerm panes.
- zellij and screen have no Windows release. herdr 0.9.5 was installed but not run.

## Terminal panes

Panes that belong to the terminal itself are not multiplexers for GitPoulpi:

- **WezTerm panes** (`wezterm cli split-pane`, `wezterm connect`, SSH domains): WezTerm draws
  everything, so pictures work (graph and avatars in the pane) and the pointer behaves as in
  WezTerm itself (shapes work in a build with upstream WezTerm PR #8092; the pane divider
  gives the resize pointer). Not measured on Linux or macOS.
- **Windows Terminal panes** (`wt ; split-pane`): sixel graph and avatars work in the pane,
  copy works, no pointer shapes.
- **kitty, Ghostty and iTerm2 splits**: native, as the terminal. Not measured on macOS for
  kitty and Ghostty (they need a GPU).

## iTerm2 `tmux -CC` (control mode)

iTerm2 opens a native window for each tmux window and keeps a "[tmux bash]" gateway window.
`TMUX` is set, so GitPoulpi draws characters in the native window. Images do reach iTerm2
through the control connection (the "Allow Terminal-Initiated Display?" dialog appeared
for an `OSC 1337` test), but GitPoulpi does not use them there. The kitty payload of a
forced `ui.image_protocol = "kitty"` is printed in the gateway window, so leave it on
`auto`. `OSC 22` and `OSC 52` do not arrive.

## Settings to know

| Setting | Use |
|---|---|
| `ui.image_protocol` | `kitty` forces kitty pictures through a multiplexer that forwards them (tmux with `allow-passthrough on`, herdr with a config GitPoulpi does not read); `none` for characters. |
| `ui.graph_renderer` | `text` is always correct. |
| `ui.kitty_keyboard` | `off` if a multiplexer misbehaves with the keyboard query. |

See [Choosing a terminal](terminals.md) for the terminal behind the multiplexer, and
[Troubleshooting](troubleshooting.md) when the graph is drawn with characters.
