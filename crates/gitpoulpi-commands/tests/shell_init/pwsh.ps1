# gitpoulpi shell integration (PowerShell). Add to your profile ($PROFILE):
#   gitpoulpi shell-init pwsh | Out-String | Invoke-Expression
# `gp` runs gitpoulpi, then cd's to the active tab's folder when you used
# "Quit and cd here" (or ui.tab_dir_follow = "auto"). Pick another name with
#   gitpoulpi shell-init pwsh --name <name>
# An alias of that name pointing at Get-ItemProperty (PowerShell's built-in
# `gp`) or at gitpoulpi is replaced by the function; anything else is kept.
$gpExisting = Get-Command gp -ErrorAction SilentlyContinue | Select-Object -First 1
$gpReplace = $false
if ($gpExisting -and $gpExisting.CommandType -eq 'Alias' -and ($gpExisting.Definition -match '^(Get-ItemProperty|gitpoulpi(\.exe)?)$')) {
    Remove-Item Alias:gp -Force
    $gpReplace = $true
}
if ($gpExisting -and -not $gpReplace -and $gpExisting.CommandType -ne 'Function') {
    Write-Warning "gitpoulpi shell-init: 'gp' already exists in this session; not overriding it. Use: gitpoulpi shell-init pwsh --name <other-name>"
} else {
    function global:gp {
        $gpExe = Get-Command gitpoulpi -CommandType Application -ErrorAction Stop | Select-Object -First 1
        $tmp = [System.IO.Path]::GetTempFileName()
        try {
            & $gpExe.Source --cwd-file $tmp @args
            $rc = $LASTEXITCODE
            if ((Get-Item -LiteralPath $tmp).Length -gt 0) {
                $dir = (Get-Content -LiteralPath $tmp -Raw).Trim()
                if ($dir -and (Test-Path -LiteralPath $dir -PathType Container) -and ($dir -ne (Get-Location).Path)) {
                    Set-Location -LiteralPath $dir
                }
            }
            $global:LASTEXITCODE = $rc
        } finally {
            Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
        }
    }
}
Remove-Variable gpExisting, gpReplace -ErrorAction SilentlyContinue
