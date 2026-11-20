#!/bin/sh
# Fills the package templates of packaging/ for one release. Used by the `channels` jobs of
# .github/workflows/release.yml, and runnable by hand to inspect what a release would publish.
#
#   sh packaging/fill-templates.sh VERSION DIST_DIR OUT_DIR
#
# VERSION   the release version without the tag prefix, for example 0.2.0
# DIST_DIR  the folder holding the release's *.sha256 files (the downloaded `dist-*` artifacts)
# OUT_DIR   receives chocolatey/, winget/, scoop/, homebrew/, aur/gitpoulpi-bin/ and aur/gitpoulpi/
#
# Environment (all optional):
#   SOURCE_SHA256     sha256 of the tag's source tarball, for aur/gitpoulpi (the folder is skipped without it)
#   MAINTAINER        the PKGBUILD Maintainer line (default: Poulpi-Lab)
#   CHOCOLATEY_USER   the nuspec owner (default: Poulpi-Lab)
#   COMMIT            full sha of the release commit, for the nuspec icon URL (default: git rev-parse HEAD of the checkout)
set -eu

version="${1:?usage: fill-templates.sh VERSION DIST_DIR OUT_DIR}"
dist="${2:?usage: fill-templates.sh VERSION DIST_DIR OUT_DIR}"
out="${3:?usage: fill-templates.sh VERSION DIST_DIR OUT_DIR}"
here=$(cd "$(dirname "$0")" && pwd)
maintainer="${MAINTAINER:-Poulpi-Lab}"
choco_user="${CHOCOLATEY_USER:-Poulpi-Lab}"

# First field of <asset>.sha256.
sha() {
    [ -f "$dist/$1.sha256" ] || { echo "fill-templates.sh: $dist/$1.sha256 is missing" >&2; exit 1; }
    cut -d ' ' -f 1 "$dist/$1.sha256" | head -n 1
}

# fill SRC DEST [NAME=VALUE ...]: copies SRC to DEST with @VERSION@ and every given @NAME@ replaced.
fill() {
    src="$1"
    dest="$2"
    shift 2
    mkdir -p "$(dirname "$dest")"
    cp "$src" "$dest"
    sed -i "s|@VERSION@|$version|g" "$dest"
    for pair in "$@"; do
        sed -i "s|@${pair%%=*}@|${pair#*=}|g" "$dest"
    done
}

commit="${COMMIT:-}"
[ -n "$commit" ] || commit=$(git -C "$here" rev-parse HEAD 2>/dev/null) || commit=""
case "$commit" in
    *[!0-9a-f]* | "") echo "fill-templates.sh: no commit sha: set COMMIT or run inside a git checkout" >&2; exit 1 ;;
esac
[ "${#commit}" -eq 40 ] || { echo "fill-templates.sh: COMMIT must be a full 40 character sha, got $commit" >&2; exit 1; }

win="gitpoulpi-$version-x86_64-pc-windows-msvc"
setup_sha=$(sha "$win-setup.exe")
zip_sha=$(sha "$win.zip")

fill "$here/chocolatey/gitpoulpi.nuspec" "$out/chocolatey/gitpoulpi.nuspec" "CHOCOLATEY_USER=$choco_user" "COMMIT=$commit"
fill "$here/chocolatey/tools/chocolateyinstall.ps1" "$out/chocolatey/tools/chocolateyinstall.ps1" "SHA256_WINDOWS_SETUP=$setup_sha"

for f in GitPoulpi.GitPoulpi.yaml GitPoulpi.GitPoulpi.locale.en-US.yaml GitPoulpi.GitPoulpi.installer.yaml; do
    fill "$here/winget/$f" "$out/winget/$f" "SHA256_WINDOWS_SETUP=$setup_sha"
done

fill "$here/scoop/gitpoulpi.json" "$out/scoop/gitpoulpi.json" "SHA256_WINDOWS_ZIP=$zip_sha"

fill "$here/homebrew/gitpoulpi.rb" "$out/homebrew/gitpoulpi.rb" \
    "SHA256_MACOS_ARM64=$(sha "gitpoulpi-$version-aarch64-apple-darwin.tar.gz")" \
    "SHA256_MACOS_X86_64=$(sha "gitpoulpi-$version-x86_64-apple-darwin.tar.gz")" \
    "SHA256_LINUX_X86_64=$(sha "gitpoulpi-$version-x86_64-unknown-linux-gnu.tar.gz")" \
    "SHA256_LINUX_AARCH64=$(sha "gitpoulpi-$version-aarch64-unknown-linux-gnu.tar.gz")"

fill "$here/aur/gitpoulpi-bin/PKGBUILD" "$out/aur/gitpoulpi-bin/PKGBUILD" \
    "MAINTAINER=$maintainer" \
    "SHA256_LINUX_X86_64=$(sha "gitpoulpi-$version-x86_64-unknown-linux-gnu.tar.gz")" \
    "SHA256_LINUX_AARCH64=$(sha "gitpoulpi-$version-aarch64-unknown-linux-gnu.tar.gz")"

if [ -n "${SOURCE_SHA256:-}" ]; then
    fill "$here/aur/gitpoulpi/PKGBUILD" "$out/aur/gitpoulpi/PKGBUILD" \
        "MAINTAINER=$maintainer" "SHA256_SOURCE_TARBALL=$SOURCE_SHA256"
else
    echo "fill-templates.sh: SOURCE_SHA256 is not set, aur/gitpoulpi is skipped" >&2
fi

# Nothing may stay a placeholder.
if grep -rn '@[A-Z0-9_]*@' "$out" >&2; then
    echo "fill-templates.sh: unfilled placeholders above" >&2
    exit 1
fi
echo "templates filled in $out"
