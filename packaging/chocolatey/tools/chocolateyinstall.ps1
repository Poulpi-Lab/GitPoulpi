# TEMPLATE, NOT PUBLISHED. see the release-and-packaging and install-channels designs ("Chocolatey").
# @SHA256_WINDOWS_SETUP@ is the first field of gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe.sha256.
#
# Chocolatey runs elevated, so this installs for all users into Program Files (/ALLUSERS). /ALIAS= means no
# `gp` alias. /CHANNEL=choco makes the installer write channel=choco into the marker next to the exes, so
# GitPoulpi knows Chocolatey owns the updates (it runs `choco upgrade`).
#
# Package parameters, for an internalised package on a private feed:
#   choco install gitpoulpi --params "/Channel:choco-internal /Feed:https://nexus.example/repository/choco/"
# pass /CHANNEL= and /FEED= through. A policy.toml or config.toml that is already deployed is never touched:
# this script writes neither.
$ErrorActionPreference = 'Stop'

$pp = Get-PackageParameters
$channel = 'choco'
if ($pp.ContainsKey('Channel') -and $pp['Channel'] -match '^[a-z-]+$') { $channel = $pp['Channel'] }
$silentArgs = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP- /ALLUSERS /TASKS="startmenu,addtopath" /ALIAS= /CHANNEL=' + $channel
if ($pp.ContainsKey('Feed') -and $pp['Feed']) { $silentArgs += ' /FEED="' + $pp['Feed'] + '"' }

$packageArgs = @{
    packageName    = $env:ChocolateyPackageName
    fileType       = 'exe'
    url64bit       = 'https://github.com/Poulpi-Lab/GitPoulpi/releases/download/releases/v@VERSION@/gitpoulpi-@VERSION@-x86_64-pc-windows-msvc-setup.exe'
    checksum64     = '@SHA256_WINDOWS_SETUP@'
    checksumType64 = 'sha256'
    silentArgs     = $silentArgs
    validExitCodes = @(0)
    softwareName   = 'GitPoulpi'
}

Install-ChocolateyPackage @packageArgs
