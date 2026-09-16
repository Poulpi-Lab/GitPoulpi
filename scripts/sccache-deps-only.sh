#!/bin/sh
# RUSTC_WRAPPER for local builds: sccache only for third-party code.
#
# Cargo sets CARGO_MANIFEST_DIR for every rustc call and every build script
# (so also for the C compiles `cc` routes through the wrapper). Registry and
# git dependencies live under CARGO_HOME; everything else is this workspace
# (the main checkout or a worktree under .claude/worktrees/), whose sources
# change constantly and must never come out of a shared cache.
# Usage: rustc-wrapper = "scripts/sccache-deps-only.sh" in the gitignored
# .cargo/config.toml (relative paths resolve against the dir holding .cargo/).
cargo_home="${CARGO_HOME:-$HOME/.cargo}"
case "${CARGO_MANIFEST_DIR:-}" in
  "$cargo_home"/registry/* | "$cargo_home"/git/*) exec sccache "$@" ;;
  *) exec "$@" ;;
esac
