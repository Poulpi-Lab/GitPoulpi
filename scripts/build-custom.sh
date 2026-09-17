#!/usr/bin/env bash
# Interactive custom build: pick the languages, hosting integrations,
# themes, logo colors and features to include, for a smaller binary. Output lands in target/custom/
# (the official target/dist/ is never touched). Windows twin:
# scripts/build-custom.ps1. Details: docs/user/custom-builds.md.
#
#   scripts/build-custom.sh          (or ./build-custom.sh at the root)
#       whiptail/dialog checklists, else numbered prompts
#   scripts/build-custom.sh --themes nord,catppuccin   (all|none|families)
#   scripts/build-custom.sh --logo-colors violet,teal --logo-default teal --icon-color teal
#   scripts/build-custom.sh --langs fr,de --integrations none --no-ai --no-mcp
#       any argument = non-interactive, passed straight to
#       `cargo run -p xtask -- build-custom` (--help lists them; --last
#       reuses the previous choices; --dry-run prints the cargo command)
#   scripts/build-custom.sh --preset mine --no-ai --save mine2
#       saved selections (outside target/): --save, --preset (a name or a
#       file path), --list-presets, --delete-preset
set -euo pipefail
cd "$(dirname "$0")/.."

xt() { cargo run -q -p xtask -- build-custom "$@"; }

if [ "$#" -gt 0 ]; then
    xt "$@"
    exit $?
fi

ENVFILE=target/custom-build/last-choices.env
LOCALES=crates/gitpoulpi-i18n/locales

# Saved choice with a default of "everything on".
saved() { # KEY DEFAULT
    local v=""
    if [ -f "$ENVFILE" ]; then v=$(sed -n "s/^$1=//p" "$ENVFILE" | head -n1); fi
    echo "${v:-$2}"
}
has() { case ",$1," in *",$2,"*) return 0 ;; esac; return 1; }

UI=plain
if command -v whiptail >/dev/null 2>&1; then UI=whiptail
elif command -v dialog >/dev/null 2>&1; then UI=dialog; fi
[ -t 0 ] && [ -t 1 ] || UI=plain

# pick TITLE PRESELECTED_CSV|ALL tag|label ... -> chosen tags on stdout
pick() {
    local title=$1 pre=$2; shift 2
    local item tag label state out i n=0 line
    if [ "$UI" != plain ]; then
        local args=()
        for item in "$@"; do
            tag=${item%%|*}; label=${item#*|}; state=OFF
            if [ "$pre" = ALL ] || has "$pre" "$tag"; then state=ON; fi
            args+=("$tag" "$label" "$state")
        done
        out=$("$UI" --title "$title" --separate-output --checklist \
            "Space toggles, Enter accepts" 22 70 14 "${args[@]}" 3>&1 1>&2 2>&3) || return 1
        echo "$out" | tr '\n' ' ' | tr -d '"'
        return 0
    fi
    local -a tags=() on=()
    for item in "$@"; do
        tag=${item%%|*}
        tags+=("$tag")
        if [ "$pre" = ALL ] || has "$pre" "$tag"; then on+=(1); else on+=(0); fi
    done
    while true; do
        echo >&2; echo "$title" >&2
        n=0
        for item in "$@"; do
            label=${item#*|}
            printf '  %2d) [%s] %s\n' $((n + 1)) "$([ "${on[$n]}" = 1 ] && echo x || echo ' ')" "$label" >&2
            n=$((n + 1))
        done
        read -r -p "Numbers to toggle (e.g. 1 3), Enter to accept: " line || return 1
        [ -z "$line" ] && break
        for i in $line; do
            case "$i" in
                *[!0-9]*) ;;
                *) if [ "$i" -ge 1 ] && [ "$i" -le "${#tags[@]}" ]; then
                       on[$((i - 1))]=$((1 - on[i - 1]))
                   fi ;;
            esac
        done
    done
    for ((i = 0; i < ${#tags[@]}; i++)); do
        if [ "${on[$i]}" = 1 ]; then printf '%s ' "${tags[$i]}"; fi
    done
}

# pick1 TITLE CURRENT tag|label ... -> the chosen tag on stdout (single choice)
pick1() {
    local title=$1 cur=$2; shift 2
    local item tag label state out i=0 line default_i=1
    if [ "$UI" != plain ]; then
        local args=()
        for item in "$@"; do
            tag=${item%%|*}; label=${item#*|}; state=OFF
            [ "$tag" = "$cur" ] && state=ON
            args+=("$tag" "$label" "$state")
        done
        out=$("$UI" --title "$title" --radiolist \
            "Space selects, Enter accepts" 22 70 14 "${args[@]}" 3>&1 1>&2 2>&3) || return 1
        echo "$out" | tr -d '"'
        return 0
    fi
    local -a tags=()
    echo >&2; echo "$title" >&2
    for item in "$@"; do
        tag=${item%%|*}; label=${item#*|}
        tags+=("$tag"); i=$((i + 1))
        [ "$tag" = "$cur" ] && default_i=$i
        printf '  %2d) %s\n' "$i" "$label" >&2
    done
    while true; do
        read -r -p "Number (Enter keeps $default_i): " line || return 1
        [ -z "$line" ] && line=$default_i
        case "$line" in
            *[!0-9]*) ;;
            *) if [ "$line" -ge 1 ] && [ "$line" -le "${#tags[@]}" ]; then
                   echo "${tags[$((line - 1))]}"; return 0
               fi ;;
        esac
    done
}

csv() { local IFS=,; echo "$*"; }

# Start from: the last build (default), a saved selection, or everything on.
# The chosen one is resolved by xtask (unknown/missing keys are reported on
# stderr) into a scratch file that the saved() lookups then read.
PRESETS=$(xt --list-presets | sed '/^#/d')
if [ -n "$PRESETS" ]; then
    START_ITEMS=("last|Previous build choices")
    while IFS= read -r name; do START_ITEMS+=("preset:$name|Saved selection: $name"); done <<<"$PRESETS"
    START_ITEMS+=("defaults|Everything on")
    START=$(pick1 "Start from" last "${START_ITEMS[@]}") || exit 1
    case "$START" in
        preset:*)
            ENVFILE=$(mktemp)
            trap 'rm -f "$ENVFILE"' EXIT
            xt --print-choices --preset "${START#preset:}" >"$ENVFILE" || exit 1 ;;
        defaults) ENVFILE=/dev/null ;;
    esac
fi

# Screen 1: languages (English is always built).
LANG_ITEMS=()
for f in "$LOCALES"/*.toml; do
    stem=$(basename "$f" .toml)
    [ "$stem" = en ] && continue
    name=$(sed -n 's/^language_name *= *"\(.*\)"/\1/p' "$f" | head -n1)
    code=$(echo "$stem" | tr '[:upper:]' '[:lower:]')
    LANG_ITEMS+=("$code|$name ($code)")
done
SAVED_LANGS=$(saved LANGS ALL)
[ "$SAVED_LANGS" = all ] && SAVED_LANGS=ALL
[ "$SAVED_LANGS" = en ] && SAVED_LANGS=none
echo "English is always included." >&2
# shellcheck disable=SC2046
LANGS_SEL=$(pick "Languages" "$SAVED_LANGS" "${LANG_ITEMS[@]}") || exit 1

# Screen 2: integrations (none = clone by URL only, no issues, no PR/MR).
SAVED_INT=$(saved INTEGRATIONS github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues)
INT_SEL=$(pick "Hosting integrations (none: clone by URL only, no issues or PR/MR)" "$SAVED_INT" \
    "github|GitHub" "gitlab|GitLab (incl. self-managed)" \
    "azure-devops|Azure DevOps (Services and Server)" "bitbucket|Bitbucket Cloud" \
    "jira|Jira (issues, Cloud and Data Center)" "trello|Trello (issues)" \
    "gitea|Gitea and Forgejo incl. Codeberg (hosting, issues)" \
    "linear|Linear (issues)" "youtrack|YouTrack (issues, Cloud and Server)" \
    "redmine|Redmine (issues)" "gerrit|Gerrit Code Review (review only)" \
    "local-issues|Local Markdown issues (a folder or Obsidian vault)") || exit 1

# Screen 3: themes (GitPoulpi Dark, Light and Terminal are always built).
THEME_ITEMS=()
THEME_COUNT=0
while IFS=$'\t' read -r fam label; do
    [ -n "$fam" ] || continue
    THEME_ITEMS+=("$fam|$label")
    THEME_COUNT=$((THEME_COUNT + 1))
done < <(xt --list-themes)
SAVED_THEMES=$(saved THEMES ALL)
[ "$SAVED_THEMES" = all ] && SAVED_THEMES=ALL
echo "GitPoulpi Dark, GitPoulpi Light and Terminal are always included." >&2
THEMES_SEL=$(pick "Themes (one entry per family)" "$SAVED_THEMES" "${THEME_ITEMS[@]}") || exit 1

# Screen 4: logo (the octopus of the About window and the Windows .exe icon).
LOGO_ITEMS=()
LOGO_COUNT=0
LOGO_DEFAULT_SLUG=lavender
while IFS=$'\t' read -r slug label; do
    [ -n "$slug" ] || continue
    LOGO_ITEMS+=("$slug|$label")
    LOGO_COUNT=$((LOGO_COUNT + 1))
    case "$label" in *" *") LOGO_DEFAULT_SLUG=$slug ;; esac
done < <(xt --list-logo-colors)
SAVED_LOGO=$(saved LOGO_COLORS all)
[ "$SAVED_LOGO" = all ] && SAVED_LOGO=ALL
echo "At least one color is needed (* = the official default)." >&2
while true; do
    LOGO_SEL=$(pick "Logo colors compiled in (About lists and cycles them)" "$SAVED_LOGO" "${LOGO_ITEMS[@]}") || exit 1
    [ -n "$(echo $LOGO_SEL)" ] && break
    echo "Pick at least one color." >&2
    SAVED_LOGO=ALL
done
SEL_ITEMS=()
for item in "${LOGO_ITEMS[@]}"; do
    has "$(echo $LOGO_SEL | tr ' ' ',')" "${item%%|*}" && SEL_ITEMS+=("$item")
done
SAVED_DEF=$(saved LOGO_DEFAULT auto)
[ "$SAVED_DEF" = auto ] && SAVED_DEF=$LOGO_DEFAULT_SLUG
has "$(echo $LOGO_SEL | tr ' ' ',')" "$SAVED_DEF" || SAVED_DEF=$(echo $LOGO_SEL | cut -d' ' -f1)
LOGO_DEF=$(pick1 "Default logo color (About, and the fallback of a stored color that is not compiled)" "$SAVED_DEF" "${SEL_ITEMS[@]}") || exit 1
SAVED_ICON=$(saved ICON_COLOR auto)
[ "$SAVED_ICON" = auto ] && SAVED_ICON=$LOGO_DEF
ICON_SEL=$(pick1 "Windows executable icon color (used when building for Windows)" "$SAVED_ICON" "${LOGO_ITEMS[@]}") || exit 1

# Screen 5: features.
FEAT_PRE=""
for pair in ai:AI mcp:MCP lfs:LFS worktree:WORKTREE submodules:SUBMODULES avatar-fetch:AVATAR_FETCH image-view:IMAGE_VIEW radar:RADAR mermaid:MERMAID board:BOARD review:REVIEW history-rewrite:HISTORY_REWRITE global-search:GLOBAL_SEARCH workspaces:WORKSPACES rewrite-scripts:REWRITE_SCRIPTS; do
    [ "$(saved "${pair#*:}" 1)" = 1 ] && FEAT_PRE="$FEAT_PRE,${pair%%:*}"
done
FEAT_SEL=$(pick "Features" "$FEAT_PRE" \
    "ai|AI assistance (commit messages, conflicts...)" \
    "mcp|MCP server (gitpoulpi-mcp binary)" \
    "lfs|Git LFS management" \
    "worktree|Worktree management" \
    "submodules|Submodule management" \
    "avatar-fetch|Download avatars (off: initials only)" \
    "image-view|Image preview (off: no GIF/WebP/BMP/ICO decoders)" \
    "radar|Radar (PRs and issues of every repository, needs an integration)" \
    "mermaid|Copy as Mermaid gitGraph and log --format mermaid" \
    "board|Kanban board view and hosting board (needs an integration)" \
    "review|In-app code review and its AI assist (needs an integration)" \
    "history-rewrite|History workbench (advanced history rewriting: authors, dates, messages, content)" \
    "global-search|Global search (title-bar button, Ctrl+Alt+F, search command)" \
    "workspaces|Workspaces (named groups of repositories; off: only the default tab list)" \
    "rewrite-scripts|Embedded Lua for rewrite scripts (needs the History workbench)") || exit 1

ARGS=()
# shellcheck disable=SC2086
THEMES_CSV=$(csv $THEMES_SEL)
if [ "$(echo "$THEMES_SEL" | wc -w)" -eq "$THEME_COUNT" ]; then THEMES_CSV=all; fi
ARGS+=(--themes "${THEMES_CSV:-none}")
# shellcheck disable=SC2086
LOGO_CSV=$(csv $LOGO_SEL)
if [ "$(echo "$LOGO_SEL" | wc -w)" -eq "$LOGO_COUNT" ]; then LOGO_CSV=all; fi
ARGS+=(--logo-colors "$LOGO_CSV" --logo-default "$LOGO_DEF" --icon-color "$ICON_SEL")
# shellcheck disable=SC2086
LANGS_CSV=$(csv $LANGS_SEL); ARGS+=(--langs "${LANGS_CSV:-en}")
# shellcheck disable=SC2086
INT_CSV=$(csv $INT_SEL); ARGS+=(--integrations "${INT_CSV:-none}")
for feat in ai mcp lfs worktree submodules avatar-fetch image-view radar mermaid board review history-rewrite global-search workspaces rewrite-scripts; do
    # Lua without the rewrite it plugs into is meaningless: not passed at all.
    if [ "$feat" = rewrite-scripts ] && ! has "$(echo "$FEAT_SEL" | tr ' ' ',')" history-rewrite; then continue; fi
    if has "$(echo "$FEAT_SEL" | tr ' ' ',')" "$feat"; then ARGS+=("--$feat"); else ARGS+=("--no-$feat"); fi
done

echo >&2
echo "Summary: ${ARGS[*]}" >&2
# Offer to keep this selection (outside target/, so cargo clean keeps it).
SAVE_NAME=""
if [ "$UI" != plain ]; then
    SAVE_NAME=$("$UI" --title "Save" --inputbox "Save this selection as (empty: do not save)" 10 60 3>&1 1>&2 2>&3) || SAVE_NAME=""
else
    read -r -p "Save this selection as (name, Enter to skip): " SAVE_NAME || SAVE_NAME=""
fi
if [ -n "$SAVE_NAME" ]; then
    xt "${ARGS[@]}" --save "$SAVE_NAME" --print-choices >/dev/null || exit 1
fi
if [ "$UI" != plain ]; then
    "$UI" --title "Build" --yesno "Build with: ${ARGS[*]}" 12 70 || exit 1
else
    read -r -p "Build now? [Y/n] " ans || exit 1
    case "$ans" in n | N) exit 1 ;; esac
fi
xt "${ARGS[@]}"
