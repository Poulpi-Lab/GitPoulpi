#!/usr/bin/env bash
# Publishes the GitPoulpi crates to crates.io, in dependency order (the release workflow's `crates` job).
#
# Needs CARGO_REGISTRY_TOKEN in the environment (a token, or the short-lived one that
# rust-lang/crates-io-auth-action produces from Trusted Publishing). Idempotent: a version that is already on
# crates.io is skipped, so a failed run can simply be started again. Each crate is retried a few times because
# the index of a just-published dependency takes a moment to show up.
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

for crate in "${CRATES[@]}"; do
  if [ ${#extra[@]} -eq 0 ] && published "$crate"; then
    echo "== $crate $version is already on crates.io, skipped"
    continue
  fi
  for attempt in $(seq 1 "$ATTEMPTS"); do
    echo "== $crate $version (attempt $attempt of $ATTEMPTS)"
    if cargo publish -p "$crate" "${extra[@]}"; then
      break
    fi
    # An upload that went through but whose answer was lost: nothing left to retry.
    if [ ${#extra[@]} -eq 0 ] && published "$crate"; then
      break
    fi
    if [ "$attempt" = "$ATTEMPTS" ]; then
      echo "::error::$crate could not be published" >&2
      exit 1
    fi
    sleep "$PAUSE"
  done
done
