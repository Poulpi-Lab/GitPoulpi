#!/bin/sh
# GitPoulpi installer for Linux and macOS. Attached to every GitHub release.
#
#   curl -fsSL https://github.com/Poulpi-Lab/GitPoulpi/releases/latest/download/install.sh | sh
#   sh install.sh --archive gitpoulpi-0.2.0-x86_64-unknown-linux-gnu.tar.gz --yes
#   sh install.sh --uninstall
#
# It downloads the archive for this OS and CPU from the release (or takes a
# local one with --archive), checks the .sha256 next to it, copies the program
# files into --prefix (gitpoulpi, gitpoulpi-tui, gitpoulpi-mcp, gitpoulpi-launch
# and gitpoulpi-updater, with the .gitpoulpi-dist marker that records
# channel=archive, so GitPoulpi updates itself in place) and links `gitpoulpi`, `gitpoulpi-tui` and `gitpoulpi-mcp`
# into ~/.local/bin. The command alias, the PATH entry and the shortcuts are made by
# the installed `gitpoulpi` itself (`alias create`, `path add`, `desktop
# shortcut install`), the same code as Settings > General > Launcher, so the
# installer and Settings never disagree. --uninstall calls `gitpoulpi uninstall`
# for the same reason. Nothing outside HOME is touched and no root is needed.

set -eu

REPO="Poulpi-Lab/GitPoulpi"
MARKER=".gitpoulpi-dist"
# Files of the archive that `gitpoulpi uninstall` leaves (it only removes the
# programs); the uninstall below removes them.
DOCS="INSTALL.txt README.md LICENSE THIRD-PARTY-LICENSES.txt CHANGELOG.md"

usage() {
    cat <<'EOF'
Usage: install.sh [options]

  --prefix DIR      install the program files here (default ~/.local/opt/gitpoulpi)
  --version X.Y.Z   install this release instead of the latest
  --archive FILE    use a local archive (its FILE.sha256 must sit next to it)
  --alias NAME      make NAME a command that starts GitPoulpi (default: gp)
  --no-alias        do not make a command alias
  --no-path         do not add GitPoulpi to the PATH of new shells
  --desktop         also make the application menu and Desktop shortcuts
  --yes, -y         never ask; use the flags and the defaults
  --no-verify       install an --archive that has no .sha256 file (not advised)
  --repo OWNER/NAME download from this GitHub repository
  --uninstall       remove what this installer and Settings made
  -h, --help        this text

Without --yes the script asks about the alias, the PATH and the shortcuts.
EOF
}

say() { printf '%s\n' "$*"; }
warn() { printf 'install.sh: warning: %s\n' "$*" >&2; }
die() {
    printf 'install.sh: %s\n' "$*" >&2
    exit 1
}

PREFIX="${HOME:-}/.local/opt/gitpoulpi"
BIN_DIR="${HOME:-}/.local/bin"
VERSION=""
ARCHIVE=""
ALIAS_NAME=""
ALIAS_SET=0
DO_PATH=1
PATH_SET=0
DO_DESKTOP=0
DESKTOP_SET=0
ASSUME_YES=0
VERIFY=1
UNINSTALL=0

[ -n "${HOME:-}" ] || die "HOME is not set"

while [ $# -gt 0 ]; do
    case "$1" in
    --prefix)
        [ $# -ge 2 ] || die "--prefix needs a folder"
        PREFIX="$2"
        shift
        ;;
    --version)
        [ $# -ge 2 ] || die "--version needs a number"
        VERSION="${2#v}"
        shift
        ;;
    --archive)
        [ $# -ge 2 ] || die "--archive needs a file"
        ARCHIVE="$2"
        shift
        ;;
    --alias)
        [ $# -ge 2 ] || die "--alias needs a name"
        ALIAS_NAME="$2"
        ALIAS_SET=1
        shift
        ;;
    --no-alias)
        ALIAS_NAME=""
        ALIAS_SET=1
        ;;
    --no-path)
        DO_PATH=0
        PATH_SET=1
        ;;
    --desktop)
        DO_DESKTOP=1
        DESKTOP_SET=1
        ;;
    --yes | -y) ASSUME_YES=1 ;;
    --no-verify) VERIFY=0 ;;
    --repo)
        [ $# -ge 2 ] || die "--repo needs OWNER/NAME"
        REPO="$2"
        shift
        ;;
    --uninstall) UNINSTALL=1 ;;
    -h | --help)
        usage
        exit 0
        ;;
    *)
        usage >&2
        die "unknown option: $1"
        ;;
    esac
    shift
done

case "$PREFIX" in
/*) ;;
*) die "--prefix must be an absolute path: $PREFIX" ;;
esac

# Asks a yes/no question on the terminal (even when the script itself is piped
# in). Prints nothing and answers the default when there is no terminal.
ask_yes_no() {
    # $1 question, $2 default (y or n)
    if [ "$ASSUME_YES" -eq 1 ] || ! ( : </dev/tty ) 2>/dev/null; then
        [ "$2" = y ]
        return
    fi
    if [ "$2" = y ]; then hint="Y/n"; else hint="y/N"; fi
    printf '%s [%s] ' "$1" "$hint" >/dev/tty
    read -r answer </dev/tty || answer=""
    case "$answer" in
    "") [ "$2" = y ] ;;
    y | Y | yes | YES | Yes) true ;;
    *) false ;;
    esac
}

ask_text() {
    # $1 question, $2 default; prints the answer
    if [ "$ASSUME_YES" -eq 1 ] || ! ( : </dev/tty ) 2>/dev/null; then
        printf '%s' "$2"
        return
    fi
    printf '%s [%s] ' "$1" "$2" >/dev/tty
    read -r answer </dev/tty || answer=""
    if [ -z "$answer" ]; then printf '%s' "$2"; else printf '%s' "$answer"; fi
}

# ---------------------------------------------------------------- uninstall
if [ "$UNINSTALL" -eq 1 ]; then
    cli="$PREFIX/gitpoulpi"
    if [ -x "$cli" ]; then
        # Removes the alias, the PATH block, the shortcuts and the programs.
        # It refuses a folder without the marker, so a development checkout
        # can never be removed by pointing --prefix at it.
        "$cli" uninstall || die "gitpoulpi uninstall failed; nothing else was removed"
    else
        warn "no gitpoulpi in $PREFIX: only the links are removed"
    fi
    # The links this installer made (never a file that is not ours).
    for name in gitpoulpi gitpoulpi-tui gitpoulpi-mcp; do
        link="$BIN_DIR/$name"
        if [ -L "$link" ]; then
            case "$(readlink "$link")" in
            "$PREFIX"/*) rm -f "$link" ;;
            esac
        fi
    done
    if [ ! -e "$PREFIX/$MARKER" ]; then
        for doc in $DOCS; do rm -f "$PREFIX/$doc"; done
        rmdir "$PREFIX" 2>/dev/null || true
    fi
    say "GitPoulpi was uninstalled. Your settings and repositories were not touched."
    exit 0
fi

# ------------------------------------------------------------------ install
fetch() {
    # $1 url, $2 destination file
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --retry 2 -o "$2" "$1"
    elif command -v wget >/dev/null 2>&1; then
        wget -q -O "$2" "$1"
    else
        die "neither curl nor wget is installed"
    fi
}

sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | cut -d ' ' -f 1
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | cut -d ' ' -f 1
    else
        die "neither sha256sum nor shasum is installed; cannot verify the download"
    fi
}

host_target() {
    os=$(uname -s)
    cpu=$(uname -m)
    case "$os/$cpu" in
    Linux/x86_64 | Linux/amd64) echo "x86_64-unknown-linux-gnu" ;;
    Linux/aarch64 | Linux/arm64) echo "aarch64-unknown-linux-gnu" ;;
    Darwin/arm64 | Darwin/aarch64) echo "aarch64-apple-darwin" ;;
    Darwin/x86_64) echo "x86_64-apple-darwin" ;;
    *) die "no GitPoulpi release for $os on $cpu; build it from source" ;;
    esac
}

WORK=$(mktemp -d "${TMPDIR:-/tmp}/gitpoulpi-install.XXXXXX")
trap 'rm -rf "$WORK"' EXIT INT TERM

if [ -z "$ARCHIVE" ]; then
    target=$(host_target)
    if [ -z "$VERSION" ]; then
        say "Looking up the latest release of $REPO ..."
        fetch "https://api.github.com/repos/$REPO/releases/latest" "$WORK/latest.json" ||
            die "cannot read the latest release of $REPO (a private repository needs --archive); try --version X.Y.Z"
        VERSION=$(sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\(releases\/\)\{0,1\}v\{0,1\}\([^"]*\)".*/\2/p' "$WORK/latest.json" | head -n 1)
        [ -n "$VERSION" ] || die "the latest release has no tag_name"
    fi
    name="gitpoulpi-$VERSION-$target.tar.gz"
    base="https://github.com/$REPO/releases/download/releases/v$VERSION"
    say "Downloading $name ..."
    fetch "$base/$name" "$WORK/$name" || die "cannot download $base/$name"
    fetch "$base/$name.sha256" "$WORK/$name.sha256" || die "cannot download $base/$name.sha256"
    ARCHIVE="$WORK/$name"
else
    [ -f "$ARCHIVE" ] || die "no such archive: $ARCHIVE"
fi

if [ "$VERIFY" -eq 1 ]; then
    sumfile="$ARCHIVE.sha256"
    [ -f "$sumfile" ] || die "$sumfile is missing; cannot verify the archive (--no-verify skips the check)"
    want=$(cut -d ' ' -f 1 "$sumfile" | head -n 1)
    got=$(sha256_of "$ARCHIVE")
    [ "$want" = "$got" ] || die "checksum mismatch for $ARCHIVE: expected $want, got $got"
    say "Checksum OK."
fi

mkdir "$WORK/unpacked"
tar -xzf "$ARCHIVE" -C "$WORK/unpacked" || die "cannot unpack $ARCHIVE"
src=""
for d in "$WORK"/unpacked/*/; do
    [ -d "$d" ] && src="${d%/}" && break
done
[ -n "$src" ] || die "the archive holds no folder"
for f in gitpoulpi gitpoulpi-tui gitpoulpi-mcp gitpoulpi-launch "$MARKER"; do
    [ -f "$src/$f" ] || die "the archive is missing $f"
done
# The archive's marker names the channel already; an older archive without it
# gets the line here, so the install is never a "legacy" one.
grep -q '^channel=' "$src/$MARKER" || printf 'channel=archive\n' >>"$src/$MARKER"

# Questions (only when the flags did not answer them).
if [ "$ALIAS_SET" -eq 0 ]; then
    ALIAS_NAME=$(ask_text "Command alias that starts GitPoulpi (empty for none)" "gp")
fi
if [ "$PATH_SET" -eq 0 ]; then
    if ask_yes_no "Add GitPoulpi to the PATH of new shells?" y; then DO_PATH=1; else DO_PATH=0; fi
fi
if [ "$DESKTOP_SET" -eq 0 ]; then
    if ask_yes_no "Add application menu and Desktop shortcuts?" n; then DO_DESKTOP=1; else DO_DESKTOP=0; fi
fi

mkdir -p "$PREFIX" "$BIN_DIR"
# Each file is copied beside its destination, then renamed over it: a running
# older version keeps working, and a failed copy never leaves half a program.
for f in "$src"/* "$src/$MARKER"; do
    [ -f "$f" ] || continue
    base=$(basename "$f")
    cp -p "$f" "$PREFIX/$base.installing"
    mv -f "$PREFIX/$base.installing" "$PREFIX/$base"
done
for name in gitpoulpi gitpoulpi-tui gitpoulpi-mcp; do
    link="$BIN_DIR/$name"
    if [ -e "$link" ] && [ ! -L "$link" ]; then
        warn "$link exists and is not a link: left alone"
        continue
    fi
    ln -sf "$PREFIX/$name" "$link"
done
say "Installed into $PREFIX (links in $BIN_DIR)."

# From here on the installed CLI does the work, the same code as Settings. A
# step that fails is reported but does not undo the install.
cli="$PREFIX/gitpoulpi"
if [ -n "$ALIAS_NAME" ]; then
    "$cli" alias create "$ALIAS_NAME" || warn "could not make the alias $ALIAS_NAME (gitpoulpi alias create $ALIAS_NAME)"
fi
if [ "$DO_PATH" -eq 1 ]; then
    "$cli" path add || warn "could not add GitPoulpi to the PATH (gitpoulpi path add)"
fi
if [ "$DO_DESKTOP" -eq 1 ]; then
    "$cli" desktop shortcut install || warn "could not make the launcher shortcut"
    "$cli" desktop shortcut install --where desktop || warn "could not make the Desktop shortcut"
fi

if [ "$DO_PATH" -eq 1 ]; then
    say "Open a new terminal, then run: ${ALIAS_NAME:-gitpoulpi}"
else
    say "Run it with: $BIN_DIR/${ALIAS_NAME:-gitpoulpi} (add $BIN_DIR to PATH to drop the folder)"
fi
