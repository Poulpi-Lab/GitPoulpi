#!/usr/bin/env sh
# Shortcut for scripts/build.sh: the daily incremental build (release
# profile). Packaging: ./build-opti.sh.
exec "$(dirname "$0")/scripts/build.sh" "$@"
