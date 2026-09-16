<#
.SYNOPSIS
    Install a git pre-push hook that runs `scripts/check.ps1 -Quick`.

.DESCRIPTION
    fmt and clippy are the two gates that fail most often and cost a whole CI
    round trip when they do; they take seconds locally. The tests are not in
    the hook, so run `scripts/check.ps1` (fast tier) yourself before a push
    that changes behavior, and `-Full` before a release tag. Bypass once with `git push
    --no-verify`.
#>
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$hooks = (git -C $root rev-parse --git-common-dir).Trim()
if (-not [IO.Path]::IsPathRooted($hooks)) { $hooks = Join-Path $root $hooks }
$hook = Join-Path $hooks 'hooks/pre-push'
@'
#!/bin/sh
# Installed by scripts/install-hooks.ps1.
exec pwsh -NoProfile -File "$(git rev-parse --show-toplevel)/scripts/check.ps1" -Quick
'@ -replace "`r`n", "`n" | Set-Content -NoNewline -Encoding utf8NoBOM $hook
Write-Host "installed $hook"
