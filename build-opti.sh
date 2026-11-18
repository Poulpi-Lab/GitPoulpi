#!/usr/bin/env sh
# Shortcut for scripts/build-opti.sh: the official optimized build (dist
# profile, thin LTO) of the three binaries, with their sizes.
exec "$(dirname "$0")/scripts/build-opti.sh" "$@"
