#!/usr/bin/env bash
# Publishes the GitPoulpi crates to crates.io, in dependency order (the release workflow's `crates` job).
#
# Needs CARGO_REGISTRY_TOKEN in the environment (a token, or the short-lived one that
# rust-lang/crates-io-auth-action produces from Trusted Publishing). Idempotent: a version that is already on
# crates.io is skipped, so a failed run can simply be started again. Each crate is retried a few times because
# the index of a just-published dependency takes a moment to show up. A 429 "too many new crates" is waited out
# (until the date crates.io names, plus 15 s, capped at 20 min) and does not count as an attempt.
#
# `xtask`, `gitpoulpi-testkit` and `gitpoulpi-updater` are never published (`publish = false`).
# The order is checked against `cargo metadata` by `xtask/tests/crates_io.rs`: add a new crate here.
#
#   scripts/publish-crates.sh            publish
#   scripts/publish-crates.sh --dry-run  `cargo publish --dry-run` of each crate (needs the previous ones on crates.io,
#                                        so only the first crate really verifies; use `cargo publish --workspace --dry-run`)
set -euo pipefail

CRATES=(
  gitpoulpi-core
  gitpoulpi-i18n
  gitpoulpi-logo-build
  gitpoulpi-pty
  gitpoulpi-secrets
  gitpoulpi-cli-backend
  gitpoulpi-config
  gitpoulpi-git2-backend
  gitpoulpi-hosting
  gitpoulpi-ipc
  gitpoulpi-log
  gitpoulpi-session
  gitpoulpi-update
  gitpoulpi-ai
  gitpoulpi-daemon
  gitpoulpi-tui
  gitpoulpi-commands
  gitpoulpi-mcp
  gitpoulpi-cli
)

ATTEMPTS=6
PAUSE=30
MAX_WAIT=1200 # longest single wait for the new-crate rate limit (20 min)

if [ "${1:-}" = "--list" ]; then
  printf '%s\n' "${CRATES[@]}"
  exit 0
fi
extra=()
[ "${1:-}" = "--dry-run" ] && extra=(--dry-run)
if [ ${#extra[@]} -eq 0 ] && [ -z "${CARGO_REGISTRY_TOKEN:-}" ]; then
  echo "CARGO_REGISTRY_TOKEN is not set" >&2
  exit 1
fi

version=$(cargo metadata --format-version 1 --no-deps |
  python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="gitpoulpi-core"))')

published() {
  # 200 = this version exists; the API asks for a User-Agent.
  [ "$(curl -s -o /dev/null -w '%{http_code}' -A 'gitpoulpi-release' "https://crates.io/api/v1/crates/$1/$version")" = "200" ]
}

# Seconds to sleep after a 429: until the date in the message plus 15 s, at most MAX_WAIT, at least 15.
rate_limit_wait() {
  local when now target secs
  when=$(sed -n 's/.*try again after \(.*\) GMT.*/\1/p' "$1" | head -n 1)
  secs=$MAX_WAIT
  if [ -n "$when" ] && target=$(date -u -d "$when GMT" +%s 2>/dev/null); then
    now=$(date -u +%s)
    secs=$((target - now + 15))
  fi
  [ "$secs" -lt 15 ] && secs=15
  [ "$secs" -gt "$MAX_WAIT" ] && secs=$MAX_WAIT
  echo "$secs"
}

for crate in "${CRATES[@]}"; do
  if [ ${#extra[@]} -eq 0 ] && published "$crate"; then
    echo "== $crate $version is already on crates.io, skipped"
    continue
  fi
  attempt=1
  while [ "$attempt" -le "$ATTEMPTS" ]; do
    echo "== $crate $version (attempt $attempt of $ATTEMPTS)"
    log=$(mktemp)
    if cargo publish -p "$crate" "${extra[@]}" 2>&1 | tee "$log"; then
      rm -f "$log"
      break
    fi
    # crates.io rate-limits NEW crates (429 "... try again after <RFC 2822 date> GMT"): wait until that
    # moment and retry without using up an attempt.
    if [ ${#extra[@]} -eq 0 ] && grep -q 'published too many new crates' "$log"; then
      wait_secs=$(rate_limit_wait "$log")
      rm -f "$log"
      echo "::notice::$crate: crates.io new-crate rate limit, waiting ${wait_secs}s before retrying"
      sleep "$wait_secs"
      continue
    fi
    rm -f "$log"
    # An upload that went through but whose answer was lost: nothing left to retry.
    if [ ${#extra[@]} -eq 0 ] && published "$crate"; then
      break
    fi
    if [ "$attempt" = "$ATTEMPTS" ]; then
      echo "::error::$crate could not be published" >&2
      exit 1
    fi
    sleep "$PAUSE"
    attempt=$((attempt + 1))
  done
done
