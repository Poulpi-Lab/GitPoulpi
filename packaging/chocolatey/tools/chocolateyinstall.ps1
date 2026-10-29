# TEMPLATE, NOT PUBLISHED. see the release-and-packaging design ("Chocolatey").
# @SHA256_WINDOWS_SETUP@ is the first field of gitpoulpi-<version>-x86_64-pc-windows-msvc-setup.exe.sha256.
#
# Chocolatey runs elevated, so this installs for all users into Program Files (/ALLUSERS). /ALIAS= and
# /NOSELFUPDATE (no `gp` alias, no self-update marker) need the small gitpoulpi.iss change described in the
# plan; until then Inno ignores them.
$ErrorActionPreference = 'Stop'

$packageArgs = @{
    packageName    = $env:ChocolateyPackageName
    fileType       = 'exe'
    url64bit       = 'https://github.com/Poulpi-Lab/GitPoulpi/releases/download/releases/v@VERSION@/gitpoulpi-@VERSION@-x86_64-pc-windows-msvc-setup.exe'
    checksum64     = '@SHA256_WINDOWS_SETUP@'
    checksumType64 = 'sha256'
    silentArgs     = '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP- /ALLUSERS /TASKS="startmenu,addtopath" /ALIAS= /NOSELFUPDATE'
    validExitCodes = @(0)
    softwareName   = 'GitPoulpi'
}

Install-ChocolateyPackage @packageArgs
