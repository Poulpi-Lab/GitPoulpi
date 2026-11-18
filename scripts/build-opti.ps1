<#
.SYNOPSIS
    Optimized build for official releases.

.DESCRIPTION
    Builds gitpoulpi (CLI + TUI + daemon), gitpoulpi-tui, gitpoulpi-mcp and gitpoulpi-launch
    with the `dist` profile (thin LTO, 16 codegen units, stripped): slow to
    build, smallest and fastest binaries. Outputs land in target\dist\.
    Linux/macOS twin: scripts/build-opti.sh.

.EXAMPLE
    pwsh scripts/build-opti.ps1
    build-opti          (root shortcut, cmd or PowerShell; extra args go to cargo)
#>
$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)

$bins = 'gitpoulpi', 'gitpoulpi-tui', 'gitpoulpi-mcp', 'gitpoulpi-launch', 'gitpoulpi-updater'

# Windows refuses to delete a running .exe (an open TUI, a session daemon) but
# lets it be renamed: move the previous binaries aside before building so the
# final copy never fails with "Accès refusé" after a long LTO link. The running
# processes keep their renamed image; the asides are swept on a later run.
Get-ChildItem 'target\dist\*.exe.*.old' -ErrorAction SilentlyContinue |
    Remove-Item -Force -ErrorAction SilentlyContinue
foreach ($bin in $bins) {
    $f = Join-Path 'target\dist' "$bin.exe"
    if (Test-Path $f) {
        Move-Item $f "$f.$([DateTime]::Now.Ticks).old"
    }
}

# sccache cannot cache LTO'd final links and gains little here: build without it.
$env:RUSTC_WRAPPER = ''
cargo build --profile dist --locked -p gitpoulpi-cli -p gitpoulpi-tui -p gitpoulpi-mcp -p gitpoulpi-updater @args
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ''
Write-Host 'Optimized binaries (target\dist):'
foreach ($bin in $bins) {
    $f = Join-Path 'target\dist' "$bin.exe"
    if (Test-Path $f) {
        '  {0,-15} {1,6:N1} MB' -f $bin, ((Get-Item $f).Length / 1MB) | Write-Host
    }
}
