<#
.SYNOPSIS
    Daily incremental build.

.DESCRIPTION
    Builds gitpoulpi (CLI + TUI + daemon), gitpoulpi-tui, gitpoulpi-mcp and gitpoulpi-launch
    with the `release` profile (incremental, 16 codegen units, no LTO): fast
    rebuilds, optimized binaries. Outputs land in target\release\.
    Packaging stays on build-opti (the `dist` profile).
    Linux/macOS twin: scripts/build.sh.

.EXAMPLE
    pwsh scripts/build.ps1
    build               (root shortcut, cmd or PowerShell; extra args go to cargo)
#>
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)

$bins = 'gitpoulpi', 'gitpoulpi-tui', 'gitpoulpi-mcp', 'gitpoulpi-launch'

# Same as build-opti: Windows refuses to delete a running .exe but lets it be
# renamed, so move the previous binaries aside and an open TUI or a session
# daemon never fails the final copy. The asides are swept on a later run.
Get-ChildItem 'target\release\*.exe.*.old' -ErrorAction SilentlyContinue |
    Remove-Item -Force -ErrorAction SilentlyContinue
foreach ($bin in $bins) {
    $f = Join-Path 'target\release' "$bin.exe"
    if (Test-Path $f) {
        Move-Item $f "$f.$([DateTime]::Now.Ticks).old"
    }
}

cargo build --release --locked -p gitpoulpi-cli -p gitpoulpi-tui -p gitpoulpi-mcp @args
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ''
Write-Host 'Binaries (target\release):'
foreach ($bin in $bins) {
    $f = Join-Path 'target\release' "$bin.exe"
    if (Test-Path $f) {
        '  {0,-15} {1,6:N1} MB' -f $bin, ((Get-Item $f).Length / 1MB) | Write-Host
    }
}
