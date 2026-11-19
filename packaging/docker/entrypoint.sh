#!/bin/sh
# Entry point of the GitPoulpi image: the terminal UI under tini (signal forwarding and zombie reaping).
# `gitpoulpi` and `gitpoulpi-mcp` are reachable with `docker run --entrypoint gitpoulpi ...`.
#
# GITPOULPI_SAFE_DIRECTORY=* trusts every bind-mounted repository, whatever its owner (the system git
# refuses repositories owned by another user otherwise).
set -eu

if [ "${GITPOULPI_SAFE_DIRECTORY:-}" = "*" ]; then
    git config --global --add safe.directory '*' || echo "entrypoint: could not set safe.directory" >&2
fi

exec tini -- gitpoulpi "$@"
