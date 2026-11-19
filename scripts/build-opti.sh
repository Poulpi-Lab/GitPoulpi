#!/usr/bin/env bash
# Optimized build for official releases: the `dist` profile (thin LTO, 16
# codegen units, stripped). Slow to build, smallest and fastest binaries.
# Outputs land in target/dist/. Windows twin: scripts/build-opti.ps1.
#
#   scripts/build-opti.sh            (or ./build-opti.sh at the root; extra args go to cargo)
set -euo pipefail
cd "$(dirname "$0")/.."

# sccache cannot cache LTO'd final links and gains little here: build without it.
RUSTC_WRAPPER= cargo build --profile dist --locked \
    -p gitpoulpi-cli -p gitpoulpi-updater -p gitpoulpi-shim "$@"

# gitpoulpi-tui and gitpoulpi-mcp are copies of the shim (as in the release archives).
cp -f target/dist/gitpoulpi-shim target/dist/gitpoulpi-tui
cp -f target/dist/gitpoulpi-shim target/dist/gitpoulpi-mcp

echo
echo "Optimized binaries (target/dist):"
for bin in gitpoulpi gitpoulpi-tui gitpoulpi-mcp gitpoulpi-launch gitpoulpi-updater; do
    f="target/dist/$bin"
    if [ -f "$f" ]; then
        awk -v b="$bin" -v s="$(wc -c < "$f")" 'BEGIN { printf "  %-15s %6.1f MB\n", b, s / 1048576 }'
    fi
done
