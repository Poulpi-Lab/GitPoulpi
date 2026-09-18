# Helpers for docs-screenshots.sh. Sourced, not run.
#
# The window lives in a private *headless sway* compositor, so it never shows
# on (or switches) the maintainer's Hyprland workspaces, nothing else can be in the
# picture (no bar, no notification, no pointer), and no keyboard focus or mouse
# is shared with the maintainer. Input goes through kitty's remote control
# (`kitten @ send-text/send-key`), capture is `grim` on the whole headless
# output. (An earlier version used a headless *Hyprland* output; creating and
# removing it parks the maintainer's other monitor on a different workspace and
# leaks their windows into the capture, so do not go back to that.)

: "${GPD_KITTY_FONT:=JetBrainsMono Nerd Font}"
: "${GPD_KITTY_FONT_SIZE:=11}"
: "${GPD_W:=1800}"
: "${GPD_H:=1060}"

kk() { kitten @ --to "unix:$GPD_SOCK" "$@"; }

# --- headless sway --------------------------------------------------------
sway_up() {
  [ -x "${SWAY_BIN:-$(command -v sway || true)}" ] || die "sway is needed (pacman -S sway); it only ever runs headless"
  mkdir -p "$GPD_WORK"
  cat > "$GPD_WORK/sway.conf" <<S
output * resolution ${GPD_W}x${GPD_H} position 0 0 scale 1
output * bg #0B0C16 solid_color
default_border none
default_floating_border none
gaps inner 0
for_window [app_id=".*"] fullscreen enable
S
  local before after
  before=$(ls "$XDG_RUNTIME_DIR"/wayland-* 2>/dev/null | grep -v '\.lock$' || true)
  env -u HYPRLAND_INSTANCE_SIGNATURE -u WAYLAND_DISPLAY -u DISPLAY \
    WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1 WLR_RENDERER_ALLOW_SOFTWARE=1 \
    LD_LIBRARY_PATH="${SWAY_LIB:-}${SWAY_LIB:+:}${LD_LIBRARY_PATH:-}" \
    setsid "${SWAY_BIN:-sway}" -c "$GPD_WORK/sway.conf" </dev/null >"$GPD_WORK/sway.log" 2>&1 &
  GPD_SWAY_PID=$!
  local i
  for i in $(seq 1 100); do
    after=$(ls "$XDG_RUNTIME_DIR"/wayland-* 2>/dev/null | grep -v '\.lock$' || true)
    GPD_WAYLAND=$(comm -13 <(sort <<<"$before") <(sort <<<"$after") | head -1)
    [ -n "$GPD_WAYLAND" ] && break
    sleep 0.1
  done
  [ -n "$GPD_WAYLAND" ] || { cat "$GPD_WORK/sway.log" >&2; die "headless sway did not start"; }
  GPD_WAYLAND=$(basename "$GPD_WAYLAND")
  export GPD_WAYLAND
  sleep 0.5
}
sway_down() {
  [ -n "${GPD_SWAY_PID:-}" ] && kill "$GPD_SWAY_PID" 2>/dev/null || true
}

# --- kitty ----------------------------------------------------------------
# kitty_up <program> [args...]: runs it full screen in the headless sway.
kitty_up() {
  mkdir -p "$GPD_WORK"
  cat > "$GPD_WORK/kitty.conf" <<K
font_family $GPD_KITTY_FONT
font_size $GPD_KITTY_FONT_SIZE
foreground #ddf7ff
background #0B0C16
selection_foreground #ddf7ff
selection_background #1f253a
cursor #ddf7ff
cursor_text_color #0B0C16
color0 #0B0C16
color1 #50f872
color2 #4fe88f
color3 #50f7d4
color4 #829dd4
color5 #86a7df
color6 #7cf8f7
color7 #ddf7ff
color8 #2d3450
color9 #85ff9d
color10 #9cf7c2
color11 #a4ffec
color12 #c4d2ed
color13 #cddbf4
color14 #d1fffe
color15 #ddf7ff
window_padding_width 4 8
hide_window_decorations yes
confirm_os_window_close 0
cursor_shape block
cursor_blink_interval 0
enable_audio_bell no
remember_window_size no
allow_remote_control socket-only
listen_on unix:$GPD_ROOT/kitty.sock
shell_integration disabled
K
  # nothing of the machine's terminal setup leaks in (herdr, an outer kitty, tmux...):
  # the Open in lists would otherwise show "Detected: Herdr: ..." entries
  local unset_env=() v
  for v in $(compgen -e | grep -E '^(HERDR_|KITTY_|TMUX|ZELLIJ|WEZTERM_)'); do unset_env+=(-u "$v"); done
  env -u HYPRLAND_INSTANCE_SIGNATURE -u DISPLAY "${unset_env[@]}" WAYLAND_DISPLAY="$GPD_WAYLAND" \
    setsid kitty --config "$GPD_WORK/kitty.conf" --class gp-docs -T gp-docs --directory "${GPD_CWD:-$HOME}" "$@" \
    </dev/null >"$GPD_WORK/kitty.log" 2>&1 &
  local i
  for i in $(seq 1 100); do
    # kitty appends -<pid> to the socket path it was configured with
    GPD_SOCK=$(ls "$GPD_ROOT"/kitty.sock-* 2>/dev/null | head -1 || true)
    [ -S "${GPD_SOCK:-/nonexistent}" ] && kk ls >/dev/null 2>&1 && return 0
    sleep 0.1
  done
  cat "$GPD_WORK/kitty.log" >&2
  die "kitty never came up in the headless compositor"
}
kitty_down() {
  [ -S "${GPD_SOCK:-/nonexistent}" ] && kk close-window --match all >/dev/null 2>&1
  sleep 0.5
  rm -f "$GPD_ROOT"/kitty.sock-*
  GPD_SOCK=""
}

# --- input ----------------------------------------------------------------
key()  { kk send-key "$@"; settle; }              # key ctrl+comma
text() { kk send-text -- "$1"; settle; }          # text "hello"
raw()  { kk send-text -- "$(printf '%b' "$1")"; settle; }
esc()  { kk send-key escape; settle; }
settle() { sleep "${GPD_SETTLE:-0.4}"; }

# SGR mouse, 1-based cells as the terminal reports them.
click()       { raw "\e[<0;$1;$2M\e[<0;$1;$2m"; }
right_click() { raw "\e[<2;$1;$2M\e[<2;$1;$2m"; }
# drag <col> <row> <col2> <row2>: press, move with the button held, release.
drag() { raw "\e[<0;$1;$2M"; raw "\e[<32;$(( ($1 + $3) / 2 ));$(( ($2 + $4) / 2 ))M"; raw "\e[<32;$3;$4M"; raw "\e[<0;$3;$4m"; }
hover()       { raw "\e[<35;$1;$2M"; sleep "${GPD_HOVER:-1.2}"; }   # tooltips need a dwell
wheel()       { raw "\e[<$((64 + ${3:-0}));$1;$2M"; }               # 0 up, 1 down

screen() { kk get-text --extent screen; }
# wait_text <fixed string> [seconds]: poll the screen, never sleep blindly.
wait_text() {
  local want=$1 limit=${2:-15} i
  for i in $(seq 1 $((limit * 10))); do
    screen | grep -qF -- "$want" && return 0
    sleep 0.1
  done
  echo "timed out waiting for: $want" >&2; screen >&2; return 1
}
# find_text <fixed string> [n]: prints "col row" (1-based terminal cells) of the
# n-th (default first) match, counting double-width characters as two cells.
find_text() {
  screen | python3 -c '
import sys, unicodedata
want, nth = sys.argv[1], int(sys.argv[2])
maxcol = int(sys.argv[3])
def w(c):
    if unicodedata.category(c) in ("Mn", "Me", "Cf") or c == "\ufe0e" or c == "\ufe0f": return 0
    return 2 if unicodedata.east_asian_width(c) in ("W", "F") else 1
n = 0
for row, line in enumerate(sys.stdin.read().split("\n"), 1):
    start = 0
    while (i := line.find(want, start)) >= 0:
        col = sum(w(c) for c in line[:i]) + 1
        start = i + 1
        if col > maxcol: continue
        n += 1
        if n == nth:
            print(col, row); sys.exit(0)
        continue
        start = i + 1
sys.exit(1)' "$1" "${2:-1}" "${GPD_MAXCOL:-9999}"
}
# click_text <text> [n] [dx]: click on a label found on screen.
click_text() { local p; p=$(find_text "$1" "${2:-1}") || die "no '$1' on screen"; set -- $p "${3:-1}"; click $(($1 + $3)) "$2"; }
rclick_text() { local p; p=$(find_text "$1" "${2:-1}") || die "no '$1' on screen"; set -- $p "${3:-1}"; right_click $(($1 + $3)) "$2"; }
hover_text() { local p; p=$(find_text "$1" "${2:-1}") || die "no '$1' on screen"; set -- $p "${3:-1}"; hover $(($1 + $3)) "$2"; }

# --- capture --------------------------------------------------------------
# shot <name>: the whole headless output (kitty is full screen on it).
shot() {
  local name=$1
  sleep 0.8   # let the compositor present the last frame
  WAYLAND_DISPLAY="$GPD_WAYLAND" grim "$GPD_OUT/$name.png"
  echo "  captured $name.png"
}

die() { echo "docs-screenshots: $*" >&2; [ -n "${GPD_SOCK:-}" ] && screen 2>/dev/null | grep -v "^[│ ]*$" | cut -c1-170 | head -${GPD_DUMP:-14} >&2; exit 1; }
