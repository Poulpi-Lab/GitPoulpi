#!/usr/bin/env sh
# Shortcut for scripts/build-custom.sh: the interactive custom build (pick
# languages, integrations and features for a smaller binary).
exec "$(dirname "$0")/scripts/build-custom.sh" "$@"
