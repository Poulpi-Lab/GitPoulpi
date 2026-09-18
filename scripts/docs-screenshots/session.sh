#!/usr/bin/env bash
# Dev helper: a persistent headless sway + kitty so shots can be authored one
# step at a time (each call is a separate process).
#   session.sh up [repo]     start sway, the fixture (once) and the TUI
#   session.sh <cmd> args    key ctrl+p | text hi | click COL ROW | rclick COL ROW |
#                            hover COL ROW | wheel COL ROW 1 | screen | find TEXT |
#                            clickt/rclickt/hovert TEXT [n] [dx]  (act on a label)
#                            raw '\e[A' | snap NAME | restart [repo] | down
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd); repo=$(cd "$here/../.." && pwd)
source "$here/lib.sh"
GPD_ROOT=${GPD_ROOT:-/tmp/gp-doc}; GPD_WORK=$GPD_ROOT/work; GPD_OUT=${GPD_OUT:-/tmp/gpd-live-out}
TUI="${GPD_TUI:-$repo/target/release/gitpoulpi-tui}"; CLI="${GPD_CLI:-$repo/target/release/gitpoulpi}"
export GITPOULPI_CONFIG_DIR=$GPD_ROOT/config
mkdir -p "$GPD_OUT"
[ -f "$GPD_ROOT/state" ] && source "$GPD_ROOT/state"
save() { printf 'GPD_WAYLAND=%q\nGPD_SWAY_PID=%q\nGPD_SOCK=%q\n' "${GPD_WAYLAND:-}" "${GPD_SWAY_PID:-}" "${GPD_SOCK:-}" > "$GPD_ROOT/state"; }
source "$here/shots.sh"
cmd=${1:-}; shift || true
case $cmd in
  up) mkdir -p "$GPD_ROOT"
      [ -d "$GPD_ROOT/fx" ] || "$here/fixture.sh" "$GPD_ROOT/fx" >/dev/null
      sway_up; save; start_tui "${1:-$GPD_ROOT/fx/orbit}"; save ;;
  restart) stop_tui || true; start_tui "${1:-$GPD_ROOT/fx/orbit}"; save ;;
  down) stop_tui || true; sway_down; rm -rf "$GPD_ROOT" ;;
  key) key "$@" ;; text) text "$1" ;; raw) raw "$1" ;;
  click) click "$@" ;; rclick) right_click "$@" ;; hover) hover "$@" ;; clickt) click_text "$@" ;; rclickt) rclick_text "$@" ;; hovert) hover_text "$@" ;; wheel) wheel "$@" ;;
  run) ensure_tui; "shot_$1"; save ;;
  screen) screen ;; find) find_text "$1" ;; snap) shot "$1" ;;
  *) echo "unknown $cmd" >&2; exit 1 ;;
esac
