#!/bin/sh
# Builds the signed apt repository served from the `Poulpi-Lab/apt` gh-pages branch.
#
#   sh build-repo.sh REPO_DIR
#
# REPO_DIR holds pool/*.deb (the release job copies the new .deb files there). The script:
#   1. keeps only the newest KEEP versions per package and architecture (default 5);
#   2. writes dists/stable/main/binary-<arch>/Packages(.gz) with apt-ftparchive;
#   3. writes dists/stable/Release and signs it into InRelease and Release.gpg;
#   4. exports the public key as gitpoulpi-archive-keyring.gpg (binary) and .asc (armored).
#
# Environment:
#   APT_GPG_KEY_ID      fingerprint or id of the signing key (required; the key is already imported)
#   APT_GPG_PASSPHRASE  passphrase of that key (optional: a key without one needs none)
#   KEEP                versions to keep per package and architecture (default 5)
#
# The key belongs to the Poulpi-Lab project, never to a person. Needs apt-ftparchive (apt-utils), gpg, gzip.
set -eu

repo="${1:?usage: build-repo.sh REPO_DIR}"
keep="${KEEP:-5}"
key="${APT_GPG_KEY_ID:?APT_GPG_KEY_ID is not set}"
suite="stable"
component="main"
archs="amd64 arm64"

cd "$repo"
[ -d pool ] || { echo "build-repo.sh: $repo/pool is missing" >&2; exit 1; }

# 1. Prune. Files are named <package>_<version>_<arch>.deb; version order is `sort -V`.
for arch in $archs; do
    for name in $(ls pool 2>/dev/null | sed -n "s/^\\(.*\\)_[^_]*_${arch}\\.deb\$/\\1/p" | sort -u); do
        ls pool | grep "^${name}_.*_${arch}\\.deb\$" | sort -V | head -n "-${keep}" | while read -r old; do
            echo "pruning pool/$old"
            rm -f "pool/$old"
        done
    done
done

# 2. Packages indexes.
for arch in $archs; do
    dir="dists/$suite/$component/binary-$arch"
    mkdir -p "$dir"
    apt-ftparchive --arch "$arch" packages pool >"$dir/Packages"
    gzip -9 -k -f "$dir/Packages"
done

# 3. Release, then the signatures.
conf=$(mktemp)
trap 'rm -f "$conf"' EXIT
cat >"$conf" <<EOF
APT::FTPArchive::Release::Origin "GitPoulpi";
APT::FTPArchive::Release::Label "GitPoulpi";
APT::FTPArchive::Release::Suite "$suite";
APT::FTPArchive::Release::Codename "$suite";
APT::FTPArchive::Release::Architectures "$archs";
APT::FTPArchive::Release::Components "$component";
APT::FTPArchive::Release::Description "GitPoulpi apt repository";
EOF
apt-ftparchive -c "$conf" release "dists/$suite" >"dists/$suite/Release"

gpg_sign() {
    if [ -n "${APT_GPG_PASSPHRASE:-}" ]; then
        gpg --batch --yes --pinentry-mode loopback --passphrase "$APT_GPG_PASSPHRASE" --local-user "$key" "$@"
    else
        gpg --batch --yes --pinentry-mode loopback --local-user "$key" "$@"
    fi
}
gpg_sign --clearsign -o "dists/$suite/InRelease" "dists/$suite/Release"
gpg_sign --armor --detach-sign -o "dists/$suite/Release.gpg" "dists/$suite/Release"

# 4. Public key, for Signed-By.
gpg --batch --yes --export "$key" >gitpoulpi-archive-keyring.gpg
gpg --batch --yes --armor --export "$key" >gitpoulpi-archive-keyring.asc

echo "repository built in $repo"
