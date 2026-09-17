<#
.SYNOPSIS
  Interactive custom build: pick the languages, hosting integrations, themes,
  logo colors and features to include, for a smaller binary. Output: target\custom\ (the
  official target\dist\ is never touched). Bash twin: scripts/build-custom.sh.
  Details: docs/user/custom-builds.md.

.EXAMPLE
  .\build-custom                      # pick in Out-GridView, else prompts
  .\build-custom -Langs fr,de -Integrations none -NoAi -NoMcp
  .\build-custom -Last -DryRun        # reuse the previous choices, print only
  .\build-custom -Preset mine -NoAi -Save mine2   # saved selections (outside target\)
  .\build-custom -ListPresets ; .\build-custom -DeletePreset mine

  Any parameter = non-interactive. Extra cargo arguments: -CargoArgs '--target','x'.
#>
[CmdletBinding()]
param(
    [string]$Langs,
    [string]$Integrations,
    [string]$Themes,
    [string]$LogoColors, [string]$LogoDefault, [string]$IconColor,
    [switch]$NoAi, [switch]$NoMcp, [switch]$NoLfs, [switch]$NoWorktree,
    [switch]$NoSubmodules, [switch]$NoAvatarFetch, [switch]$NoImageView,
    [switch]$NoRadar, [switch]$NoMermaid, [switch]$NoBoard, [switch]$NoReview, [switch]$NoHistoryRewrite, [switch]$NoGlobalSearch, [switch]$NoWorkspaces, [switch]$NoRewriteScripts,
    [string]$Preset, [string]$Save, [switch]$ListPresets, [string]$DeletePreset,
    [switch]$Last, [switch]$Full, [switch]$DryRun,
    [ValidateSet('custom', 'dist', 'release')][string]$Profile = 'custom',
    [string[]]$CargoArgs
)
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..')

function Invoke-Xtask([string[]]$XtaskArgs) {
    & cargo run -q -p xtask -- build-custom @XtaskArgs
    exit $LASTEXITCODE
}

# Non-interactive: any parameter at all.
if ($PSBoundParameters.Count -gt 0) {
    $a = @()
    if ($Langs) { $a += '--langs', $Langs }
    if ($Integrations) { $a += '--integrations', $Integrations }
    if ($Themes) { $a += '--themes', $Themes }
    if ($LogoColors) { $a += '--logo-colors', $LogoColors }
    if ($LogoDefault) { $a += '--logo-default', $LogoDefault }
    if ($IconColor) { $a += '--icon-color', $IconColor }
    if ($NoAi) { $a += '--no-ai' }
    if ($NoMcp) { $a += '--no-mcp' }
    if ($NoLfs) { $a += '--no-lfs' }
    if ($NoWorktree) { $a += '--no-worktree' }
    if ($NoSubmodules) { $a += '--no-submodules' }
    if ($NoAvatarFetch) { $a += '--no-avatar-fetch' }
    if ($NoImageView) { $a += '--no-image-view' }
    if ($NoRadar) { $a += '--no-radar' }
    if ($NoMermaid) { $a += '--no-mermaid' }
    if ($NoBoard) { $a += '--no-board' }
    if ($NoReview) { $a += '--no-review' }
    if ($NoHistoryRewrite) { $a += '--no-history-rewrite' }
    if ($NoGlobalSearch) { $a += '--no-global-search' }
    if ($NoWorkspaces) { $a += '--no-workspaces' }
    if ($NoRewriteScripts) { $a += '--no-rewrite-scripts' }
    if ($Preset) { $a += '--preset', $Preset }
    if ($Save) { $a += '--save', $Save }
    if ($ListPresets) { $a += '--list-presets' }
    if ($DeletePreset) { $a += '--delete-preset', $DeletePreset }
    if ($Last) { $a += '--last' }
    if ($Full) { $a += '--full' }
    if ($DryRun) { $a += '--dry-run' }
    if ($PSBoundParameters.ContainsKey('Profile')) { $a += '--profile', $Profile }
    if ($CargoArgs) { $a += '--'; $a += $CargoArgs }
    Invoke-Xtask $a
}

$envFile = 'target\custom-build\last-choices.env'
function Get-Saved([string]$Key, [string]$Default) {
    if (Test-Path $envFile) {
        $line = Get-Content $envFile | Where-Object { $_ -like "$Key=*" } | Select-Object -First 1
        if ($line) { return $line.Substring($Key.Length + 1) }
    }
    return $Default
}
function Test-Has([string]$Csv, [string]$Tag) { return ((",$Csv,") -like "*,$Tag,*") }

# Items: objects with Tag, Name, On. Returns the chosen tags.
$hasGrid = [bool](Get-Command Out-GridView -ErrorAction SilentlyContinue)
function Select-Items([string]$Title, $Items) {
    if ($hasGrid) {
        $picked = $Items | Select-Object Tag, Name |
            Out-GridView -Title "$Title (Ctrl+click to select; the saved choices are not pre-selected here)" -PassThru
        return @($picked | ForEach-Object { $_.Tag })
    }
    while ($true) {
        Write-Host ""; Write-Host $Title
        for ($i = 0; $i -lt $Items.Count; $i++) {
            $mark = if ($Items[$i].On) { 'x' } else { ' ' }
            Write-Host ("  {0,2}) [{1}] {2}" -f ($i + 1), $mark, $Items[$i].Name)
        }
        $line = Read-Host 'Numbers to toggle (e.g. 1 3), Enter to accept'
        if ([string]::IsNullOrWhiteSpace($line)) { break }
        foreach ($n in ($line -split '[ ,]+')) {
            if ($n -match '^\d+$' -and [int]$n -ge 1 -and [int]$n -le $Items.Count) {
                $Items[[int]$n - 1].On = -not $Items[[int]$n - 1].On
            }
        }
    }
    return @($Items | Where-Object { $_.On } | ForEach-Object { $_.Tag })
}

# One choice among Items (Tag, Name); Current is the tag kept on Enter.
function Select-One([string]$Title, $Items, [string]$Current) {
    if ($hasGrid) {
        $picked = $Items | Select-Object Tag, Name | Out-GridView -Title "$Title (select one row)" -OutputMode Single
        if ($picked) { return $picked.Tag }
        return $Current
    }
    Write-Host ""; Write-Host $Title
    $default = 1
    for ($i = 0; $i -lt $Items.Count; $i++) {
        if ($Items[$i].Tag -eq $Current) { $default = $i + 1 }
        Write-Host ("  {0,2}) {1}" -f ($i + 1), $Items[$i].Name)
    }
    while ($true) {
        $line = Read-Host "Number (Enter keeps $default)"
        if ([string]::IsNullOrWhiteSpace($line)) { return $Items[$default - 1].Tag }
        if ($line -match '^\d+$' -and [int]$line -ge 1 -and [int]$line -le $Items.Count) { return $Items[[int]$line - 1].Tag }
    }
}

# Start from: the last build (default), a saved selection, or everything on.
# xtask resolves a saved one (unknown/missing keys are reported) into a
# scratch file that Get-Saved then reads.
$presetNames = @(& cargo run -q -p xtask -- build-custom --list-presets | Where-Object { $_ -and -not $_.StartsWith('#') })
if ($presetNames.Count) {
    $startItems = @([pscustomobject]@{ Tag = 'last'; Name = 'Previous build choices' })
    foreach ($n in $presetNames) { $startItems += [pscustomobject]@{ Tag = "preset:$n"; Name = "Saved selection: $n" } }
    $startItems += [pscustomobject]@{ Tag = 'defaults'; Name = 'Everything on' }
    $start = Select-One 'Start from' $startItems 'last'
    if ($start -like 'preset:*') {
        $envFile = [System.IO.Path]::GetTempFileName()
        & cargo run -q -p xtask -- build-custom --print-choices --preset $start.Substring(7) | Set-Content $envFile
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    } elseif ($start -eq 'defaults') {
        $envFile = Join-Path ([System.IO.Path]::GetTempPath()) 'gitpoulpi-no-such-choices.env'
    }
}

# Screen 1: languages (English is always built).
$savedLangs = Get-Saved 'LANGS' 'all'
$langItems = foreach ($row in (& cargo run -q -p xtask -- build-custom --list-langs)) {
    $code, $name = $row -split "`t", 2
    [pscustomobject]@{ Tag = $code; Name = "$name ($code)"; On = ($savedLangs -eq 'all' -or (Test-Has $savedLangs $code)) }
}
Write-Host 'English is always included.'
$langSel = Select-Items 'Languages' @($langItems)

# Screen 2: integrations.
$savedInt = Get-Saved 'INTEGRATIONS' 'github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues'
$intItems = @(
    [pscustomobject]@{ Tag = 'github'; Name = 'GitHub'; On = (Test-Has $savedInt 'github') },
    [pscustomobject]@{ Tag = 'gitlab'; Name = 'GitLab (incl. self-managed)'; On = (Test-Has $savedInt 'gitlab') },
    [pscustomobject]@{ Tag = 'azure-devops'; Name = 'Azure DevOps (Services and Server)'; On = (Test-Has $savedInt 'azure-devops') },
    [pscustomobject]@{ Tag = 'bitbucket'; Name = 'Bitbucket Cloud'; On = (Test-Has $savedInt 'bitbucket') },
    [pscustomobject]@{ Tag = 'jira'; Name = 'Jira (issues, Cloud and Data Center)'; On = (Test-Has $savedInt 'jira') },
    [pscustomobject]@{ Tag = 'trello'; Name = 'Trello (issues)'; On = (Test-Has $savedInt 'trello') },
    [pscustomobject]@{ Tag = 'gitea'; Name = 'Gitea and Forgejo incl. Codeberg (hosting, issues)'; On = (Test-Has $savedInt 'gitea') },
    [pscustomobject]@{ Tag = 'linear'; Name = 'Linear (issues)'; On = (Test-Has $savedInt 'linear') },
    [pscustomobject]@{ Tag = 'youtrack'; Name = 'YouTrack (issues, Cloud and Server)'; On = (Test-Has $savedInt 'youtrack') },
    [pscustomobject]@{ Tag = 'redmine'; Name = 'Redmine (issues)'; On = (Test-Has $savedInt 'redmine') },
    [pscustomobject]@{ Tag = 'gerrit'; Name = 'Gerrit Code Review (review only)'; On = (Test-Has $savedInt 'gerrit') },
    [pscustomobject]@{ Tag = 'local-issues'; Name = 'Local Markdown issues (a folder or Obsidian vault)'; On = (Test-Has $savedInt 'local-issues') }
)
$intSel = Select-Items 'Hosting integrations (none: clone by URL only, no issues or PR/MR)' $intItems

# Screen 3: themes (GitPoulpi Dark, Light and Terminal are always built).
$savedThemes = Get-Saved 'THEMES' 'all'
$themeItems = foreach ($row in (& cargo run -q -p xtask -- build-custom --list-themes)) {
    $fam, $name = $row -split "`t", 2
    [pscustomobject]@{ Tag = $fam; Name = "$name ($fam)"; On = ($savedThemes -eq 'all' -or (Test-Has $savedThemes $fam)) }
}
Write-Host 'GitPoulpi Dark, GitPoulpi Light and Terminal are always included.'
$themeSel = Select-Items 'Themes (one entry per family)' @($themeItems)

# Screen 4: logo (the octopus of the About window and the Windows .exe icon).
$savedLogo = Get-Saved 'LOGO_COLORS' 'all'
$officialDefault = 'lavender'
$logoItems = foreach ($row in (& cargo run -q -p xtask -- build-custom --list-logo-colors)) {
    $slug, $name = $row -split "`t", 2
    if ($name -like '* *') { $officialDefault = $slug }
    [pscustomobject]@{ Tag = $slug; Name = $name; On = ($savedLogo -eq 'all' -or (Test-Has $savedLogo $slug)) }
}
Write-Host 'At least one color is needed (* = the official default).'
do {
    $logoSel = Select-Items 'Logo colors compiled in (About lists and cycles them)' @($logoItems)
    if (-not $logoSel.Count) { Write-Host 'Pick at least one color.'; foreach ($i in $logoItems) { $i.On = $true } }
} while (-not $logoSel.Count)
$selItems = @($logoItems | Where-Object { $logoSel -contains $_.Tag })
$savedDef = Get-Saved 'LOGO_DEFAULT' 'auto'
if ($savedDef -eq 'auto') { $savedDef = $officialDefault }
if ($logoSel -notcontains $savedDef) { $savedDef = $logoSel[0] }
$logoDef = Select-One 'Default logo color (About, and the fallback of a stored color that is not compiled)' $selItems $savedDef
$savedIcon = Get-Saved 'ICON_COLOR' 'auto'
if ($savedIcon -eq 'auto') { $savedIcon = $logoDef }
$iconSel = Select-One 'Windows executable icon color' @($logoItems) $savedIcon

# Screen 5: features.
$feat = @(
    @('ai', 'AI assistance (commit messages, conflicts...)', 'AI'),
    @('mcp', 'MCP server (gitpoulpi-mcp binary)', 'MCP'),
    @('lfs', 'Git LFS management', 'LFS'),
    @('worktree', 'Worktree management', 'WORKTREE'),
    @('submodules', 'Submodule management', 'SUBMODULES'),
    @('avatar-fetch', 'Download avatars (off: initials only)', 'AVATAR_FETCH'),
    @('image-view', 'Image preview (off: no GIF/WebP/BMP/ICO decoders)', 'IMAGE_VIEW'),
    @('radar', 'Radar (PRs and issues of every repository, needs an integration)', 'RADAR'),
    @('mermaid', 'Copy as Mermaid gitGraph and log --format mermaid', 'MERMAID'),
    @('board', 'Kanban board view and hosting board (needs an integration)', 'BOARD'),
    @('review', 'In-app code review and its AI assist (needs an integration)', 'REVIEW'),
    @('history-rewrite', 'History workbench (advanced history rewriting: authors, dates, messages, content)', 'HISTORY_REWRITE'),
    @('global-search', 'Global search (title-bar button, Ctrl+Alt+F, search command)', 'GLOBAL_SEARCH'),
    @('workspaces', 'Workspaces (named groups of repositories; off: only the default tab list)', 'WORKSPACES'),
    @('rewrite-scripts', 'Embedded Lua for rewrite scripts (needs the History workbench)', 'REWRITE_SCRIPTS')
)
$featItems = foreach ($f in $feat) {
    [pscustomobject]@{ Tag = $f[0]; Name = $f[1]; On = ((Get-Saved $f[2] '1') -eq '1') }
}
$featSel = Select-Items 'Features' @($featItems)

$a = @()
$a += '--logo-colors', $(if ($logoSel.Count -eq @($logoItems).Count) { 'all' } else { $logoSel -join ',' })
$a += '--logo-default', $logoDef, '--icon-color', $iconSel
$a += '--themes', $(if ($themeSel.Count -eq @($themeItems).Count) { 'all' } elseif ($themeSel.Count) { $themeSel -join ',' } else { 'none' })
$a += '--langs', $(if ($langSel.Count) { $langSel -join ',' } else { 'en' })
$a += '--integrations', $(if ($intSel.Count) { $intSel -join ',' } else { 'none' })
foreach ($f in $feat) {
    # Lua without the rewrite it plugs into is meaningless: not passed at all.
    if ($f[0] -eq 'rewrite-scripts' -and $featSel -notcontains 'history-rewrite') { continue }
    $a += $(if ($featSel -contains $f[0]) { "--$($f[0])" } else { "--no-$($f[0])" })
}
Write-Host ""; Write-Host "Summary: $($a -join ' ')"
$saveName = Read-Host 'Save this selection as (name, Enter to skip)'
if ($saveName) {
    & cargo run -q -p xtask -- build-custom @a --save $saveName --print-choices | Out-Null
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}
$answer = Read-Host 'Build now? [Y/n]'
if ($answer -match '^[nN]') { exit 1 }
Invoke-Xtask $a
