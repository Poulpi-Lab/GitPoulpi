#!/usr/bin/env bash
# Retakes the user-guide screenshots (docs/user/images/*.png) from a
# reproducible state.
#
#   scripts/docs-screenshots.sh [--list] [--keep] [--out DIR] [shot ...]
#
# With no shot names it takes every shot. See docs/developer/visual-verification.md
# for why (and how) a real window is used at all.
#
# What it does, in order:
#   1. builds the demo repositories (fixture.sh) under a temp root, with fixed
#      authors and dates, and a temp GITPOULPI_CONFIG_DIR (never your own);
#   2. starts a private *headless sway* (no window on your desktop, no
#      workspace switch, no focus or pointer shared with you, nothing else in
#      the picture);
#   3. launches kitty full screen in it (1800x1060, so every image has the same
#      size), drives the TUI through kitty's remote control and captures the
#      headless output with grim;
#   4. stops kitty, the daemon and sway.
#
# Nothing here touches your screen, so the desktop lock is not needed; wrap it
# in `cargo run -p xtask -- desktop-lock -- scripts/docs-screenshots.sh` anyway
# if you also use the desktop at the same time as other agents.
# No root? Get a user-space sway: `pacman -Sp sway wlroots0.20 libliftoff` prints
# the package URLs; curl them, `bsdtar -x -f pkg -C DIR`, then
#   SWAY_BIN=DIR/usr/bin/sway SWAY_LIB=DIR/usr/lib scripts/docs-screenshots.sh
# The clone-hosted* shots start the fake hosting service (`xtask mock-host`,
# loopback only; see docs/developer/testing.md) and need `cargo build -p xtask`.
# Needs: sway (headless only), kitty + kitten, grim, jq, git, and a release
# build: cargo build --release -p gitpoulpi-tui -p gitpoulpi-cli
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/.." && pwd)
# shellcheck source=docs-screenshots/lib.sh
source "$here/docs-screenshots/lib.sh"

keep=0; list=0; GPD_OUT="$repo/docs/user/images"; wanted=()
while [ $# -gt 0 ]; do
  case $1 in
    --keep) keep=1 ;;
    --list) list=1 ;;
    --out) GPD_OUT=$2; shift ;;
    -h|--help) sed -n '2,30p' "$0"; exit 0 ;;
    *) wanted+=("$1") ;;
  esac
  shift
done

TUI="${GPD_TUI:-$repo/target/release/gitpoulpi-tui}"
CLI="${GPD_CLI:-$repo/target/release/gitpoulpi}"
source "$here/docs-screenshots/shots.sh"

if [ $list = 1 ]; then printf '%s\n' "${ALL_SHOTS[@]}"; exit 0; fi
[ ${#wanted[@]} -eq 0 ] && wanted=("${ALL_SHOTS[@]}")
for t in "$TUI" "$CLI"; do [ -x "$t" ] || die "missing $t (cargo build --release -p gitpoulpi-tui -p gitpoulpi-cli)"; done
command -v kitten >/dev/null && command -v grim >/dev/null && command -v jq >/dev/null || die "need kitty, grim and jq"
mkdir -p "$GPD_OUT"

GPD_ROOT=${GPD_ROOT:-/tmp/gp-doc}   # fixed: it shows in the window title
rm -rf "$GPD_ROOT"; mkdir -p "$GPD_ROOT"
GPD_WORK="$GPD_ROOT/work"; GPD_SOCK=""
export GITPOULPI_CONFIG_DIR="$GPD_ROOT/config"
cleanup() {
  kitty_down || true
  "$CLI" -s gpdocs daemon stop >/dev/null 2>&1 || true
  mock_down
  sway_down
  [ $keep = 1 ] && echo "kept $GPD_ROOT" || rm -rf "$GPD_ROOT"
}
trap cleanup EXIT

"$here/docs-screenshots/fixture.sh" "$GPD_ROOT/fx" >/dev/null
sway_up
for s in "${wanted[@]}"; do
  echo "== $s"
  ensure_tui
  "shot_$s"
done
