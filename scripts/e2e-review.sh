#!/usr/bin/env bash
# Drives GitPoulpi's review view by hand against the real test hosts
# (scripts/test-bench.sh: GitLab, Gitea, Forgejo, Gerrit in Docker; GitHub's private
# sandbox repository), in a *scratch* environment and a private headless sway.
#
#   scripts/e2e-review.sh SCRATCH_DIR up github|gitlab|gitea|forgejo|gerrit
#   scripts/e2e-review.sh SCRATCH_DIR key ctrl+p | text hi | click COL ROW | clickt TEXT [n] [dx] |
#                                     find TEXT | screen | snap NAME | wait TEXT [secs] | raw '\e[A'
#   scripts/e2e-review.sh SCRATCH_DIR down            stop the TUI, the daemon and sway, forget the state
#
# Nothing outside SCRATCH_DIR is written: HOME, GITPOULPI_CONFIG_DIR and every
# XDG_* directory live inside it, the clones and a git credential store too
# (mode 600), and GIT_CONFIG_GLOBAL points at a file in it. Tokens reach the
# TUI only as GITPOULPI_TOKEN_<ID> environment variables. Never run it with
# SCRATCH_DIR inside a repository.
#
# Environment:
#   GITPOULPI_BENCH_DIR  where test-bench.sh keeps <host>.token and password
#                        (default ~/.cache/gitpoulpi-bench)
#   SWAY_BIN / SWAY_LIB  a user-space sway (see scripts/docs-screenshots.sh)
#   GPD_TUI / GPD_CLI    the release binaries (default: $CARGO_TARGET_DIR/release)
#   GPD_OUT              where `snap` writes PNGs (default SCRATCH_DIR/shots)
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
scratch=${1:?usage: e2e-review.sh SCRATCH_DIR up HOST|key ...|down}
shift
mkdir -p "$scratch"
scratch=$(cd "$scratch" && pwd)
case $scratch in
  "$HOME"/.config*|"$HOME"/.ssh*|"$HOME"/.local*) echo "refusing a scratch dir inside your real config" >&2; exit 1 ;;
esac
cmd=${1:-}
shift || true

BENCH=${GITPOULPI_BENCH_DIR:-$HOME/.cache/gitpoulpi-bench}
REAL_HOME=$HOME
TARGET=${CARGO_TARGET_DIR:-}
TUI=${GPD_TUI:-${TARGET:+$TARGET/release/gitpoulpi-tui}}
CLI=${GPD_CLI:-${TARGET:+$TARGET/release/gitpoulpi}}

GPD_ROOT=$scratch
GPD_WORK=$scratch/work
GPD_OUT=${GPD_OUT:-$scratch/shots}
GPD_SESSION=e2erev
mkdir -p "$GPD_WORK" "$GPD_OUT"
source "$here/docs-screenshots/lib.sh"

# The whole scratch environment (also what the TUI and the CLI are started with).
export HOME="$scratch/home"
export GITPOULPI_CONFIG_DIR="$scratch/config"
export XDG_CONFIG_HOME="$scratch/xdg/config" XDG_DATA_HOME="$scratch/xdg/data" XDG_STATE_HOME="$scratch/xdg/state" \
  XDG_CACHE_HOME="$scratch/xdg/cache"
export GIT_CONFIG_GLOBAL="$scratch/gitconfig" GIT_CONFIG_SYSTEM=/dev/null GITPOULPI_DESKTOP_NO_HOOKS=1
export LC_ALL=C.UTF-8 TZ=UTC
mkdir -p "$HOME" "$GITPOULPI_CONFIG_DIR" "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_STATE_HOME" "$XDG_CACHE_HOME"
RT=$scratch/run   # the private sway's wayland socket and the daemon's sockets live here too
mkdir -p "$RT"; chmod 700 "$RT"
export XDG_RUNTIME_DIR=$RT
unset WAYLAND_DISPLAY DISPLAY HYPRLAND_INSTANCE_SIGNATURE SWAYSOCK
state=$scratch/state
[ -f "$state" ] && source "$state"
save() { printf 'GPD_WAYLAND=%q\nGPD_SWAY_PID=%q\nGPD_SOCK=%q\nHOST=%q\n' "${GPD_WAYLAND:-}" "${GPD_SWAY_PID:-}" "${GPD_SOCK:-}" "${HOST:-}" > "$state"; }

host_info() { # host -> id kind url project user token-file clone-url
  case $1 in
    github) echo "hub github https://github.com Poulpi-Lab/gitpoulpi-sandbox poulpi-lab gh https://github.com/Poulpi-Lab/gitpoulpi-sandbox.git" ;;
    gitlab) echo "lab gitlab_self_managed http://localhost:8929 root/gitpoulpi-sandbox root gitlab.token http://localhost:8929/root/gitpoulpi-sandbox.git" ;;
    gitea) echo "tea gitea http://localhost:3000 poulpi/gitpoulpi-sandbox poulpi gitea.token http://localhost:3000/poulpi/gitpoulpi-sandbox.git" ;;
    forgejo) echo "fjo forgejo http://localhost:3001 poulpi/gitpoulpi-sandbox poulpi forgejo.token http://localhost:3001/poulpi/gitpoulpi-sandbox.git" ;;
    gerrit) echo "ger gerrit http://127.0.0.1:8081 gitpoulpi-sandbox admin gerrit.token http://127.0.0.1:8081/a/gitpoulpi-sandbox" ;;
    *) die "unknown host $1" ;;
  esac
}
host_token() { # token file name or "gh"
  if [ "$1" = gh ]; then (HOME=$REAL_HOME gh auth token); else tr -d '\n' < "$BENCH/$1"; fi
}

setup_host() {
  local h=$1 id kind url project user tf clone token
  read -r id kind url project user tf clone <<<"$(host_info "$h")"
  token=$(host_token "$tf")
  # git credentials: a store file in the scratch dir only
  local sf=$scratch/git-credentials proto=${clone%%://*} rest=${clone#*://}
  local enc; enc=$(printf '%s' "$token" | sed -e 's/%/%25/g' -e 's/+/%2B/g' -e 's|/|%2F|g' -e 's/=/%3D/g' -e 's/@/%40/g' -e 's/:/%3A/g')
  (umask 077; printf '%s://%s:%s@%s\n' "$proto" "$user" "$enc" "${rest%%/*}" >> "$sf")
  printf '[credential]\n\thelper = store --file %s\n[user]\n\tname = E2E\n\temail = e2e@example.com\n[init]\n\tdefaultBranch = main\n' "$sf" > "$GIT_CONFIG_GLOBAL"
  mkdir -p "$scratch/clones"
  [ -d "$scratch/clones/$h/.git" ] || git clone -q "$clone" "$scratch/clones/$h"
  {
    printf '[[integrations]]\nid = "%s"\nkind = "%s"\n' "$id" "$kind"
    [ "$url" = - ] || printf 'url = "%s"\n' "$url"
    printf 'token_ref = "integration:%s"\nname = "%s"\nuser = { login = "%s" }\n' "$id" "$h" "$user"
    [ "$kind" != gerrit ] || printf 'account = "%s"\n' "$user"
  } > "$GITPOULPI_CONFIG_DIR/config.toml"
  cat >> "$GITPOULPI_CONFIG_DIR/config.toml" <<T

[ui]
open_in_default = "Terminal: kitty"
T
  mkdir -p "$scratch/clones/$h/.gitpoulpi"
  # the link; an existing state file also holds pending comments, so it is kept
  [ -f "$scratch/clones/$h/.gitpoulpi/repo-state.toml" ] ||
    printf '[pulls_source]\nintegration = "%s"\nproject = "%s"\n\n[issues_source]\nintegration = "%s"\nproject = "%s"\n' "$id" "$project" "$id" "$project" \
      > "$scratch/clones/$h/.gitpoulpi/repo-state.toml"
  # keep the repository's own state files out of `git status`
  printf '.gitpoulpi/\n' >> "$scratch/clones/$h/.git/info/exclude"
  TOKEN_VAR="GITPOULPI_TOKEN_$(printf %s "$id" | tr a-z- A-Z_)"
  export "$TOKEN_VAR=$token"
  GPD_CWD=$scratch/clones/$h
  kitty_up env HOME="$HOME" GIT_CONFIG_GLOBAL="$GIT_CONFIG_GLOBAL" GIT_CONFIG_SYSTEM=/dev/null LC_ALL=C.UTF-8 TZ=UTC \
    GITPOULPI_CONFIG_DIR="$GITPOULPI_CONFIG_DIR" GITPOULPI_DESKTOP_NO_HOOKS=1 "$TOKEN_VAR=$token" \
    XDG_CONFIG_HOME="$XDG_CONFIG_HOME" XDG_DATA_HOME="$XDG_DATA_HOME" XDG_STATE_HOME="$XDG_STATE_HOME" XDG_CACHE_HOME="$XDG_CACHE_HOME" \
    XDG_RUNTIME_DIR="$RT" "$TUI" -s "$GPD_SESSION" --repo "$scratch/clones/$h"
  unset "$TOKEN_VAR"
}

case $cmd in
  up)
    HOST=${1:?host}
    [ -x "$TUI" ] || die "no TUI binary: set GPD_TUI or CARGO_TARGET_DIR (cargo build --release -p gitpoulpi-tui -p gitpoulpi-cli)"
    if [ -z "${GPD_SWAY_PID:-}" ] || ! kill -0 "$GPD_SWAY_PID" 2>/dev/null; then sway_up; fi
    GPD_SOCK=""
    save
    setup_host "$HOST"
    save
    wait_text "main" 30 || true
    ;;
  key) key "$@" ;; text) text "$1" ;; raw) raw "$1" ;;
  click) click "$@" ;; rclick) right_click "$@" ;; wheel) wheel "$@" ;; hover) hover "$@" ;;
  clickt) click_text "$@" ;; rclickt) rclick_text "$@" ;;
  screen) screen ;; find) find_text "$1" "${2:-1}" ;; wait) wait_text "$@" ;;
  snap) shot "$1" ;;
  token) # print the env var name only; useful to run curl by hand: eval "$(... token HOST)"
    read -r _ _ _ _ _ tf _ <<<"$(host_info "$1")"; host_token "$tf" ;;
  down)
    kitty_down || true
    [ -x "$CLI" ] && "$CLI" -s "$GPD_SESSION" daemon stop >/dev/null 2>&1 || true
    sway_down
    rm -f "$state"
    ;;
  *) die "unknown command '$cmd'" ;;
esac
