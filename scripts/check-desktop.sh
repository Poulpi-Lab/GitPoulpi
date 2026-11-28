#!/usr/bin/env bash
# Checks the desktop frontend (desktop/, its own cargo workspace): format, lints, tests.
# Honours CARGO_TARGET_DIR like cargo itself.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../desktop"

echo "== cargo fmt --all --check"
cargo fmt --all --check

echo "== cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings

echo "== cargo test --workspace"
cargo test --workspace

echo "desktop checks passed"
