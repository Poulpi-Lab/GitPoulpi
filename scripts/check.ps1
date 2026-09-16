<#
.SYNOPSIS
    Run the gates locally: fmt, clippy, and the test suite in tiers.

.DESCRIPTION
    CI tests only on Ubuntu (see the budget note at the top of
    .github/workflows/ci.yml): the Windows runner cannot finish the suite
    inside the job timeout. Windows -- the daily driver -- is therefore tested
    *here*, on the Windows machine. Its CI job only runs fmt and clippy.

    The tests come in tiers (`cargo run -p xtask -- test`): "fast" is every
    test except the PTY end-to-end ones (`pty_*` targets), "pty" is only
    those. The default run does fmt, clippy and the fast tier; the PTY tier is
    opt-in because it takes far longer. Run `-Full` before a release tag.
    The last lines say which tiers ran and which were SKIPPED, so a skipped
    tier is never mistaken for a green one.

    Exits non-zero on the first failing gate, so it chains:
    `scripts/check.ps1 && git push`.

.PARAMETER Quick
    fmt and clippy only -- what the pre-push hook runs. No tests.

.PARAMETER Pty
    Also run the PTY end-to-end tier (only the `pty_*` test targets).

.PARAMETER Full
    Everything: fmt, clippy, the fast tier and the PTY tier. Before a release tag.

.PARAMETER Failed
    fmt, then only what failed in the last test run (`xtask test --failed`,
    from target/xtask-test-failures.tsv). Converges as the failures get fixed;
    run the full tiers again before a release tag. Skips clippy to keep the
    fix loop short (the full tiers run it); add -Clippy when the fix is about it.

.PARAMETER Clippy
    With -Failed: run clippy too.

.PARAMETER Only
    With -Failed: rerun only the recorded failures whose target or test name
    contains one of these patterns (`xtask test --failed --only ...`), i.e. the
    ones the fix in hand is meant to cure. The others stay recorded.

.PARAMETER Features
    Also run the feature-matrix gate (`cargo run -p xtask -- features --quick`:
    clippy of the three binary crates with every optional feature off, then
    with `lang-fr` + github, then `lang-fr` + jira). Runs after the other gates. Untested on Windows.

.EXAMPLE
    pwsh scripts/check.ps1
    pwsh scripts/check.ps1 -Pty
    pwsh scripts/check.ps1 -Full
    pwsh scripts/check.ps1 -Failed
    pwsh scripts/check.ps1 -Failed -Only pty_clone_hosted,pty_rewrite_paths
    pwsh scripts/check.ps1 -Features
    pwsh scripts/check.ps1 -Quick
#>
[CmdletBinding()]
param([switch]$Quick, [switch]$Features, [switch]$Pty, [switch]$Full, [switch]$Failed, [string[]]$Only = @(), [switch]$Clippy)

$ErrorActionPreference = 'Stop'
Set-Location (Split-Path -Parent $PSScriptRoot)

# No debug info in the gate's builds: the test profile inherits `dev`, and the
# DWARF/PDB it writes is a large part of the link time of ~400 test binaries
# on Windows. Backtraces keep function names, they lose file:line. Set
# CARGO_PROFILE_DEV_DEBUG yourself (e.g. `1`) before running this script to
# keep them. It changes the build fingerprint, so a plain `cargo test` after
# this script rebuilds once; use the same setting in both to share a build.
if ($null -eq $env:CARGO_PROFILE_DEV_DEBUG) { $env:CARGO_PROFILE_DEV_DEBUG = '0' }

$ran = [System.Collections.Generic.List[string]]::new()
$skipped = [System.Collections.Generic.List[string]]::new()

function Invoke-Gate {
    param([string]$Name, [string[]]$Command)
    Write-Host "==> $Name" -ForegroundColor Cyan
    $started = Get-Date
    & $Command[0] $Command[1..($Command.Length - 1)]
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED: $Name" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host ("    ok in {0:n0}s" -f ((Get-Date) - $started).TotalSeconds) -ForegroundColor Green
    $ran.Add($Name)
}

# The Git LFS checks need the `git-lfs` program. Where it is installed they are
# mandatory (GITPOULPI_REQUIRE_LFS=1 turns a "not installed" skip into a
# failure); where it is not, they skip and this says so out loud.
& git lfs version *> $null
if ($LASTEXITCODE -eq 0) {
    $env:GITPOULPI_REQUIRE_LFS = '1'
} else {
    Write-Host 'WARNING: git-lfs is not installed; the LFS checks will be SKIPPED.' -ForegroundColor Yellow
}

Invoke-Gate 'fmt'    @('cargo', 'fmt', '--all', '--check')
if ($Failed -and -not $Clippy) {
    $skipped.Add('clippy (fix loop: -Failed without -Clippy)')
} else {
    Invoke-Gate 'clippy' @('cargo', 'clippy', '--workspace', '--all-targets', '--', '-D', 'warnings')
}
# `--no-fail-fast` for the same reason as in CI: report every failing test
# binary, not just the first. The tiers are built by `xtask test`.
$runFast = ((-not $Quick) -and (-not $Failed))
$runPty = ((-not $Quick) -and (-not $Failed) -and ($Pty -or $Full))
# xtask runs from a copy: the test tiers rebuild xtask's own binary, and
# Windows refuses to overwrite an executable that is running.
$xtask = $null
if ($runFast -or $runPty -or $Failed -or $Features) {
    Invoke-Gate 'build xtask' @('cargo', 'build', '-q', '-p', 'xtask')
    $exe = if ($IsWindows) { 'xtask.exe' } else { 'xtask' }
    $runner = Join-Path 'target' 'xtask-runner'
    New-Item -ItemType Directory -Force -Path $runner | Out-Null
    $xtask = Join-Path $runner $exe
    Copy-Item -Force (Join-Path 'target' 'debug' $exe) $xtask
}
if ($Failed) {
    $rerun = @($xtask, 'test', '--failed')
    # `pwsh -File` hands "a,b" over as one string: split it here.
    foreach ($p in ($Only -split ',' | Where-Object { $_ })) { $rerun += @('--only', $p) }
    Invoke-Gate 'test (failed last run)' $rerun
    $skipped.Add('test (full tiers: run them again before a release tag)')
} elseif ($runFast -and $runPty) {
    # One pool for both tiers: the long fast binaries (the contract suite)
    # overlap the PTY ones instead of running before them.
    Invoke-Gate 'test (fast + pty)' @($xtask, 'test', '--tier', 'full', '--no-fail-fast')
} elseif ($runFast) {
    Invoke-Gate 'test (fast)' @($xtask, 'test', '--tier', 'fast', '--no-fail-fast')
    $skipped.Add('test (pty)')
} else { $skipped.Add('test (fast)'); $skipped.Add('test (pty)') }
# Licenses: only where cargo-deny is installed (deny.toml holds the allow-list);
# the notices' freshness is a normal test (xtask/tests/licenses.rs).
& cargo deny --version *> $null
if ($LASTEXITCODE -eq 0) {
    Invoke-Gate 'licenses' @('cargo', 'deny', 'check', 'licenses')
} else {
    Write-Host 'cargo-deny is not installed; the license allow-list check is SKIPPED.' -ForegroundColor Yellow
    $skipped.Add('licenses')
}
if ($Features) {
    Invoke-Gate 'features' @($xtask, 'features', '--quick')
} else { $skipped.Add('features') }
Write-Host ('Ran:     ' + ($ran -join ', ')) -ForegroundColor Green
if ($skipped.Count -gt 0) {
    Write-Host ('SKIPPED: ' + ($skipped -join ', ') + '  (not tested, not green)') -ForegroundColor Yellow
}
Write-Host 'Gates that ran passed.' -ForegroundColor Green
