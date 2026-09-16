<#
.SYNOPSIS
    Watch the CI run for a commit until it finishes, polling on a tight interval.

.DESCRIPTION
    A CI run that is silently red (or hung) costs hours if nobody looks. This
    polls `gh run list` every -IntervalSeconds (default 60 -- the ceiling is two
    minutes, by standing request) and prints a one-line status per poll: the
    run's overall state plus the per-job state, so a job that is stuck on one
    step is visible while it is still stuck and not after the 30-minute cap.

    On failure it prints the tail of every failing job's log and exits 1, so it
    also works as a gate in a chain: `watch-ci.ps1 && something-else`.

.PARAMETER Sha
    Commit to watch, in any spelling git understands -- a short SHA, a full
    SHA, a branch name, HEAD~2. Defaults to HEAD. It is resolved to the full
    40-character SHA before use, because `gh run list --commit` matches on the
    full SHA only: handed an abbreviated one it matches nothing and reports no
    run, which is indistinguishable from a run that has not started yet. That
    cost this project five consecutive red pushes that nobody noticed.

.PARAMETER IntervalSeconds
    Poll period. Default 60, clamped to 120 -- polling less often than every two
    minutes defeats the point of the script.

.PARAMETER TimeoutMinutes
    Give up waiting after this long. Default 35: CI jobs are capped at 30
    minutes in .github/workflows/ci.yml, so this outlasts a capped run.

.EXAMPLE
    pwsh scripts/watch-ci.ps1
    pwsh scripts/watch-ci.ps1 -Sha 2321a81 -IntervalSeconds 30
    pwsh scripts/watch-ci.ps1 -Sha main
#>
[CmdletBinding()]
param(
    [string]$Sha,
    [int]$IntervalSeconds = 60,
    [int]$TimeoutMinutes = 35,
    [int]$LogTailLines = 60
)

$ErrorActionPreference = 'Stop'

if ($IntervalSeconds -gt 120) {
    Write-Warning "Interval $IntervalSeconds s is above the two-minute ceiling; using 120."
    $IntervalSeconds = 120
}
if ($IntervalSeconds -lt 10) { $IntervalSeconds = 10 }

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    throw "gh (GitHub CLI) is not on PATH -- this script drives it."
}

if (-not $Sha) { $Sha = 'HEAD' }

# `gh run list --commit` compares against the full 40-character SHA and does
# no prefix matching, so anything shorter silently matches nothing. Resolve
# whatever the caller typed -- short SHA, branch, HEAD~2 -- to the full one
# here, and fail loudly rather than watching a commit that does not exist.
$resolved = (git rev-parse --verify --quiet "$Sha^{commit}" 2>$null)
if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($resolved)) {
    throw "'$Sha' does not resolve to a commit in this repository."
}
$Sha = $resolved.Trim()
$short = $Sha.Substring(0, 8)

# A commit that was never pushed has no run and never will, but the poll loop
# below cannot tell that from a run that is merely slow to appear: both print
# 'no run yet'. Say so once, up front, instead of letting the caller watch an
# empty sky for 35 minutes.
$onRemote = git branch -r --contains $Sha 2>$null
if ([string]::IsNullOrWhiteSpace($onRemote)) {
    Write-Warning "$short is not on any remote-tracking branch -- it looks unpushed, so no run will ever appear for it."
}

function Get-Run {
    param([string]$Commit)
    $json = gh run list --commit $Commit --limit 1 --json databaseId,status,conclusion,workflowName,url 2>$null
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($json)) { return $null }
    $runs = $json | ConvertFrom-Json
    if ($runs.Count -eq 0) { return $null }
    return $runs[0]
}

function Format-Jobs {
    param([long]$RunId)
    $json = gh run view $RunId --json jobs 2>$null
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($json)) { return '' }
    $jobs = ($json | ConvertFrom-Json).jobs
    ($jobs | ForEach-Object {
        $state = if ($_.status -ne 'completed') {
            $step = $_.steps | Where-Object { $_.status -eq 'in_progress' } | Select-Object -First 1
            if ($step) { "running:$($step.name)" } else { $_.status }
        } else { $_.conclusion }
        "$($_.name)=$state"
    }) -join '  '
}

$deadline = (Get-Date).AddMinutes($TimeoutMinutes)
$started = Get-Date
Write-Host "Watching CI for $short (poll ${IntervalSeconds}s, giving up after ${TimeoutMinutes}m)" -ForegroundColor Cyan

while ($true) {
    $run = Get-Run -Commit $Sha
    $elapsed = '{0:mm\:ss}' -f ((Get-Date) - $started)
    $stamp = (Get-Date).ToString('HH:mm:ss')

    if ($null -eq $run) {
        Write-Host "[$stamp +$elapsed] no run yet for $short"
    }
    else {
        $jobs = Format-Jobs -RunId $run.databaseId
        if ($run.status -eq 'completed') {
            $color = if ($run.conclusion -eq 'success') { 'Green' } else { 'Red' }
            Write-Host "[$stamp +$elapsed] $($run.status)/$($run.conclusion)  $jobs" -ForegroundColor $color
            Write-Host $run.url

            if ($run.conclusion -eq 'success') { exit 0 }

            $failed = (gh run view $run.databaseId --json jobs | ConvertFrom-Json).jobs |
                Where-Object { $_.conclusion -notin @('success', 'skipped') }
            foreach ($job in $failed) {
                Write-Host ""
                Write-Host "--- $($job.name) ($($job.conclusion)) : last $LogTailLines log lines ---" -ForegroundColor Yellow
                gh run view --job=$($job.databaseId) --log 2>$null | Select-Object -Last $LogTailLines
            }
            exit 1
        }

        Write-Host "[$stamp +$elapsed] $($run.status)  $jobs"
    }

    if ((Get-Date) -gt $deadline) {
        Write-Host "Gave up after $TimeoutMinutes minutes; the run is still $($run.status)." -ForegroundColor Yellow
        exit 2
    }
    Start-Sleep -Seconds $IntervalSeconds
}
