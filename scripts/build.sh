#!/usr/bin/env bash
# Daily incremental build: the `release` profile (incremental, 16 codegen
# units, no LTO). Fast rebuilds, optimized binaries in target/release/.
# Packaging stays on build-opti (the `dist` profile).
# Windows twin: scripts/build.ps1.
#
#   scripts/build.sh                 (or ./build.sh at the root; extra args go to cargo)
set -euo pipefail
cd "$(dirname "$0")/.."

cargo build --release --locked \
    -p gitpoulpi-cli -p gitpoulpi-tui -p gitpoulpi-mcp "$@"

echo
echo "Binaries (target/release):"
for bin in gitpoulpi gitpoulpi-tui gitpoulpi-mcp gitpoulpi-launch; do
    f="target/release/$bin"
    if [ -f "$f" ]; then
        awk -v b="$bin" -v s="$(wc -c < "$f")" 'BEGIN { printf "  %-15s %6.1f MB\n", b, s / 1048576 }'
    fi
done
