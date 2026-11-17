# The shot list. Each `shot_<name>` leaves the TUI in the state the picture
# shows, calls `shot <name>` and returns it to a known state (Esc) or restarts
# the TUI. File names match docs/user/images/<name>.png.
#
# Every picture of docs/user/images is a shot here (themes included).

ALL_SHOTS=(hero about about-crates activity-log activity-log-repository compare-cards log-mermaid sparse-checkout palette palette-settings theme-picker log-graph log-columns-menu log-changes-column context-menu diff diff-split blame blame-card file-history file-history-blame editor editor-encoding-menu editor-eol-menu file-pane-multiselect commit-file-menu commit-column-all commit-column-menu ignore-picker open-with composer ai-explain help help-hosting rebase settings settings-launcher settings-conflict-prevention settings-search settings-profiles settings-ssh settings-integrations settings-connect settings-ai settings-ui settings-editor settings-git settings-repo-ai workflow-settings workflow-sidebar workflow-start undo-toolbar pull-push-tooltip settings-keyboard settings-keyboard-conflicts settings-language settings-ai-presets ai-commit-message theme-serie theme-material settings-appearance-theme settings-log-graph-lanes settings-ai-agent settings-open-in settings-open-in-presets open-in-menu theme-dark theme-light theme-catppuccin-latte theme-catppuccin-mocha theme-cyberpunk theme-dracula theme-everforest-dark theme-gruvbox-dark theme-gruvbox-light theme-kanagawa theme-matrix theme-fire theme-fire-light theme-sunset theme-monokai theme-nord theme-one-dark theme-rose-pine theme-solarized-dark theme-solarized-light theme-terminal theme-tokyo-night theme-catppuccin-frappe theme-catppuccin-macchiato theme-kanagawa-lotus theme-one-light theme-rose-pine-dawn theme-rose-pine-moon theme-tokyo-night-day theme-vesper theme-github-dark theme-github-light theme-vscode-dark theme-vscode-light theme-apprentice theme-ayu-dark theme-ayu-light theme-ayu-mirage theme-cobalt2 theme-deus theme-dracula-soft theme-everforest-light theme-github-dark-colorblind theme-github-dark-dimmed theme-github-dark-high-contrast theme-github-dark-legacy theme-github-light-colorblind theme-github-light-high-contrast theme-github-light-legacy theme-gotham theme-gruvbox-dark-hard-contrast theme-gruvbox-dark-soft-contrast theme-gruvbox-light-hard-contrast theme-gruvbox-light-soft-contrast theme-iceberg-dark theme-iceberg-light theme-jellybeans theme-kanagawa-dragon theme-lucario theme-miasma theme-moonfly theme-night-owl-dark theme-night-owl-light theme-nightfly theme-nightfox theme-nightfox-carbonfox theme-nightfox-dawnfox theme-nightfox-dayfox theme-nightfox-duskfox theme-nightfox-nordfox theme-nightfox-terafox theme-noctis theme-noctis-azureus theme-noctis-bordo theme-noctis-hibernus theme-noctis-lilac theme-noctis-lux theme-noctis-minimus theme-noctis-obscuro theme-noctis-sereno theme-noctis-uva theme-noctis-viola theme-nordic theme-one-half-dark theme-one-half-light theme-panda theme-posterpole theme-posterpole-gray theme-seoul256-dark theme-seoul256-light theme-shades-of-purple theme-shades-of-purple-super-dark theme-sonokai theme-sonokai-andromeda theme-sonokai-atlantis theme-sonokai-espresso theme-sonokai-maia theme-sonokai-shusia theme-srcery theme-tender theme-tokyo-night-moon theme-tokyo-night-storm theme-tomorrow theme-tomorrow-night theme-tomorrow-night-blue theme-tomorrow-night-bright theme-tomorrow-night-eighties theme-zenbones-duckbones theme-zenbones-forestbones-dark theme-zenbones-forestbones-light theme-zenbones-kanagawabones theme-zenbones-neobones-dark theme-zenbones-neobones-light theme-zenbones-nordbones theme-zenbones-rosebones-dark theme-zenbones-rosebones-light theme-zenbones-seoulbones-dark theme-zenbones-seoulbones-light theme-zenbones-tokyobones-dark theme-zenbones-tokyobones-light theme-zenbones-vimbones theme-zenbones-zenburned theme-zenbones-zenwritten-dark theme-zenbones-zenwritten-light undo-history reflog lost-work history-rewrite tag-delete settings-quantity conflict-banner merge-editor-card merge-editor merge-editor-full-file home clone clone-hosted clone-hosted-search init tab-picker new-tab workspace-search workspaces workspace-switch workspace-edit radar review-overview review-files local-issues issue-board review-ai lfs-menu worktrees worktree-menu worktree-create worktree-tab submodules submodule-menu merge-editor-chunk-menu merge-editor-take-both merge-editor-output-edited conflict-file-resolved conflict-commit-merge conflict-banner-rebase merge-editor-deleted-card conflict-banner-continue issues-sidebar issue-conversation issue-reply-composer pr-conversation pr-reply-composer)

# --- helpers shared by the shots -------------------------------------------
seed_config() {
  mkdir -p "$GITPOULPI_CONFIG_DIR"
  cat > "$GITPOULPI_CONFIG_DIR/config.toml" <<T
[general]
auto_fetch_interval_minutes = 0
fetch_on_tab_open = false
remember_tabs = false

[ui]
language = "en"
open_in_default = "Terminal: kitty"
${GPD_UI_EXTRA:-}

[updates]
mode = "off"

[ai]
enabled = true
default_profile = "demo"

[[ai.profiles]]
id = "demo"
name = "Demo agent"
kind = "local_agent"
command = "$GPD_ROOT/fx/fake-ai.sh"
args = []
harness = "claude"

[ai.profiles.env]
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
CLAUDE_CONFIG_DIR = "\${env:HOME}/.claude-work"
T
  # explicit entries: the default list depends on the machine (herdr, Omarchy...)
  local e
  for e in "Terminal: kitty|kitty --directory \${cwd}" "Editor: VS Code|code \${cwd}" "Editor: Zed|zed \${cwd}" "Git tool: lazygit|kitty --directory \${cwd} lazygit" "Explorer: File manager|xdg-open \${cwd}"; do
    printf '\n[[ui.open_in]]\nlabel = "%s"\ncommand = "%s"\n' "${e%%|*}" "${e#*|}" >> "$GITPOULPI_CONFIG_DIR/config.toml"
  done
  # whole tables a shot needs (the fake hosting profiles): see shot_clone-hosted
  printf '%s\n' "${GPD_CONFIG_EXTRA:-}" >> "$GITPOULPI_CONFIG_DIR/config.toml"
}

# fake_desktop: the "Open with" list of a PDF comes from the system's MIME
# database and .desktop files. The TUI sees only these (XDG_DATA_DIRS /
# XDG_CONFIG_DIRS below), never the machine's.
fake_desktop() {
  local d="$GPD_ROOT/xdg/data" c="$GPD_ROOT/xdg/config"
  mkdir -p "$d/mime" "$d/applications" "$c"
  printf '50:application/pdf:*.pdf\n' > "$d/mime/globs2"
  local n
  for n in "Evince|evince %f" "Okular|okular %f" "Firefox|firefox %u" "GIMP|gimp %f"; do
    printf '[Desktop Entry]\nType=Application\nName=%s\nExec=%s\nMimeType=application/pdf;\n' "${n%%|*}" "${n#*|}" \
      > "$d/applications/$(printf %s "${n%%|*}" | tr A-Z a-z).desktop"
  done
  printf '[Default Applications]\napplication/pdf=evince.desktop;\n' > "$c/mimeapps.list"
}

# Open in detection reads the environment and PATH (tmux/herdr/kitty/wezterm
# variables, every preset's program on PATH): with GPD_CLEAN_ENV=1 the TUI runs
# with a generic machine instead of the one taking the shots (kitty still hosts
# the window, only the TUI process's environment changes). Fills `clean`.
clean_env_args() {
  local bin="$GPD_ROOT/cleanbin" t v
  if [ ! -d "$bin" ]; then
    mkdir -p "$bin"
    for t in git git-lfs sh bash env cat sed grep awk sort head tail tr dirname basename mkdir rm cp mv ls date sleep; do
      ln -s "$(command -v "$t")" "$bin/$t" 2>/dev/null || true
    done
    for t in tmux wezterm alacritty; do printf '#!/bin/sh\nexit 0\n' > "$bin/$t"; chmod +x "$bin/$t"; done
  fi
  for v in $(compgen -e | grep -E '^(HERDR_|KITTY_|TMUX|ZELLIJ|WEZTERM_|CMUX_|ALACRITTY_|GHOSTTY_|KONSOLE_|GNOME_TERMINAL|WT_|OMARCHY|HYPRLAND|WAYLAND_DISPLAY|DISPLAY|TERM_PROGRAM|XDG_CURRENT_DESKTOP|XDG_SESSION)'); do clean+=(-u "$v"); done
  for v in KITTY_WINDOW_ID KITTY_LISTEN_ON KITTY_PID KITTY_INSTALLATION_DIR KITTY_PUBLIC_KEY WAYLAND_DISPLAY DISPLAY TERM_PROGRAM; do clean+=(-u "$v"); done
  clean+=(PATH="$bin")
}

# start_tui <repo dir>: a fresh daemon-backed TUI in kitty on that repository.
start_tui() {
  seed_config
  fake_desktop
  local fx="$GPD_ROOT/fx"
  export GPD_CWD="$fx/home/src"
  local clean=()
  [ "${GPD_CLEAN_ENV:-}" = 1 ] && clean_env_args
  kitty_up env "${clean[@]}" HOME="$fx/home" GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null \
    LC_ALL=C.UTF-8 TZ=UTC GITPOULPI_CONFIG_DIR="$GITPOULPI_CONFIG_DIR" \
    GITPOULPI_TOKEN_HUB=fake-token GITPOULPI_TOKEN_LAB=fake-token \
    XDG_DATA_HOME="$GPD_ROOT/xdg/data" XDG_CONFIG_HOME="$GPD_ROOT/xdg/config" \
    XDG_DATA_DIRS="$GPD_ROOT/xdg/data" XDG_CONFIG_DIRS="$GPD_ROOT/xdg/config" XDG_CACHE_HOME="$GPD_ROOT/xdg/cache" XDG_STATE_HOME="$GPD_ROOT/xdg/state" \
    "$TUI" -s gpdocs --repo "$1"
  wait_text "main" 30
  case $1 in
    *orbit-tools) wait_text "feat(chart): two" 30 ;;
    *orbit-rebase) wait_text "Rebasing feature" 30 ;;
    *orbit-conflict) wait_text "Merge in progress" 30 ;;
    *) wait_text "docs: design" 30 || wait_text "Merge" 5 ;;
  esac
  GPD_SETTLE=0.8 settle
}
stop_tui() { kitty_down; "$CLI" -s gpdocs daemon stop >/dev/null 2>&1 || true; }

# --- shots -----------------------------------------------------------------
# esc_all: back to the plain main window.
esc_all() {
  esc; esc; esc
  local i; for i in 1 2 3; do screen | grep -qF "Repository Management" || break; esc; done
  norm_layout
}
# The Log | right pane divider sits at column 129 on a fresh layout; views that
# open a diff move it, so put it back before every shot.
norm_layout() {
  local p; p=$(find_text "╭›") || return 0; set -- $p
  [ "$1" = 129 ] || { drag "$1" 28 129 28; sleep 0.4; }
}
select_wip() { esc_all; click_text "// W" 1 3; sleep 0.6; }
# commit row text, e.g. "table" -> the Log row of "feat(render): table of planets"
open_diff() { click_text "$1" 1 2; wait_text "$2" 10; }

shot_hero() { shot hero; }

# ensure_tui: one TUI for consecutive shots; a shot that needs another
# config calls `restart_tui [extra [ui] toml] [repo]`.
ensure_tui() { [ -S "${GPD_SOCK:-/nonexistent}" ] || start_tui "${1:-$GPD_ROOT/fx/orbit}"; }
restart_tui() { GPD_UI_EXTRA=${1:-}; export GPD_UI_EXTRA; stop_tui; start_tui "${2:-$GPD_ROOT/fx/orbit}"; }

# Everything below runs on one TUI session; each shot returns to the main window.
shot_palette-settings() {
  key ctrl+p; wait_text "Command Palette"; text "Settings:"; wait_text "Settings: "; sleep 0.5
  shot palette-settings; esc_all
}
shot_palette() {
  key ctrl+p; wait_text "Command Palette"; text "stage"; wait_text "Git: Stage All Changes"
  shot palette; esc_all
}
# second_tab on|off: open / close the conflict demo repository as another tab.
second_tab() {
  if [ "$1" = open ]; then
    env HOME="$GPD_ROOT/fx/home" GITPOULPI_CONFIG_DIR="$GITPOULPI_CONFIG_DIR" "$CLI" -s gpdocs repo open "$GPD_ROOT/fx/orbit-conflict" >/dev/null 2>&1 || true
    sleep 1
  else
    esc_all
    screen | head -1 | grep -qF "orbit-conflict" || click_text "orbit-conflict" 1 1
    sleep 0.5; key ctrl+w; sleep 0.8; esc_all
  fi
}
shot_home() {
  restart_tui ""
  second_tab open
  key ctrl+o; wait_text "OPEN REPOSITORIES"; shot home
}
shot_new-tab() {
  restart_tui ""
  second_tab open
  key alt+t; wait_text "New tab" 10; sleep 0.6; shot new-tab; esc_all
}
shot_workspace-search() {
  restart_tui ""
  second_tab open
  key ctrl+alt+f; wait_text "Search" 10; text "main"; sleep 2.5; shot workspace-search; esc_all
}
ws_cli() { env HOME="$GPD_ROOT/fx/home" GITPOULPI_CONFIG_DIR="$GITPOULPI_CONFIG_DIR" "$CLI" -s gpdocs "$@" >/dev/null 2>&1 || true; }
# ws_seed: three workspaces, one with a default profile (the `work` table comes
# from GPD_CONFIG_EXTRA, see ws_up), one whose repository is gone. The CLI
# refuses a path that does not exist, so the repository is made, added, deleted.
ws_config() { printf '\n[[profiles]]\nid = "work"\nname = "Work"\nauthor_name = "Mara Quinn"\nauthor_email = "mara@acme.example"\n'; }
ws_seed() {
  local d old="$GPD_ROOT/fx/old-demo" moons="$GPD_ROOT/fx/moons-demo"
  for d in "$old" "$moons"; do
    [ -d "$d" ] && continue
    mkdir -p "$d"
    git -C "$d" init -q -b main
    git -C "$d" -c user.name=Fixture -c user.email=f@example.com commit -q --allow-empty -m "Initial commit"
    git -C "$d" remote add origin "git@example.com:acme/${d##*/}.git"
  done
  ws_cli workspace create "Orbit" --add "$GPD_ROOT/fx/orbit" --add "$GPD_ROOT/fx/orbit-conflict" --color blue --profile work --description "The planets app"
  ws_cli workspace create "Side projects" --add "$moons" --color green
  ws_cli workspace create "Archive" --add "$old" --add "$moons" --color purple
  rm -rf "$old"
}
ws_up() {
  GPD_CONFIG_EXTRA=$(ws_config); export GPD_CONFIG_EXTRA
  restart_tui ""
  ws_seed
}
shot_workspaces() {
  ws_up
  key alt+j; wait_text "New workspace"; sleep 0.6
  key Right; sleep 0.8
  shot workspaces; esc_all
}
shot_workspace-switch() {
  [ -n "${GPD_CONFIG_EXTRA:-}" ] || ws_up
  esc_all; key alt+w; sleep 0.6; key Down; key Return; sleep 1.5
  key alt+w; sleep 0.8; shot workspace-switch; esc_all
}
shot_workspace-edit() {
  [ -n "${GPD_CONFIG_EXTRA:-}" ] || ws_up
  esc_all; key alt+j; wait_text "New workspace"; sleep 0.5
  key e; sleep 1; shot workspace-edit; esc_all; unset GPD_CONFIG_EXTRA
}
# The Radar over two repositories linked to the fake GitLab (its merge
# requests and issues come from `xtask mock-host`). Linking writes the
# repository's own .git/gitpoulpi/repo-state.toml, removed again afterwards.
lp_link() {
  local d
  for d in orbit:orbit-co/orbit orbit-conflict:orbit-co/orbit-conflict; do
    mkdir -p "$GPD_ROOT/fx/${d%%:*}/.git/gitpoulpi"
    printf '[pulls_source]\nintegration = "lab"\nproject = "%s"\n\n[issues_source]\nintegration = "lab"\nproject = "%s"\n' "${d#*:}" "${d#*:}" > "$GPD_ROOT/fx/${d%%:*}/.git/gitpoulpi/repo-state.toml"
  done
}
lp_unlink() { rm -rf "$GPD_ROOT/fx/orbit/.git/gitpoulpi" "$GPD_ROOT/fx/orbit-conflict/.git/gitpoulpi"; }
shot_radar() {
  mock_up
  GPD_CONFIG_EXTRA=$(mock_profiles); export GPD_CONFIG_EXTRA
  lp_link
  restart_tui ""
  second_tab open
  key alt+r; wait_text "MY PULL REQUESTS" 20; sleep 3
  key Down; key Down; key Down; key p; sleep 1.2
  shot radar; esc_all
  lp_unlink; unset GPD_CONFIG_EXTRA; stop_tui
}
# The in-app review of merge request !58 of `xtask mock-host` (two files, two threads,
# a conversation). The mock serves the host side; the code comes from git, so the demo
# repository's origin gets the request's head as `refs/merge-requests/58/head`, made in
# a throwaway clone (the demo repository itself is not touched).
review_head() {
  local tmp="$GPD_ROOT/review-head" n
  rm -rf "$tmp"
  git clone -q "$GPD_ROOT/fx/orbit.git" "$tmp"
  (
    cd "$tmp"
    export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
    export GIT_AUTHOR_NAME=Ines GIT_AUTHOR_EMAIL=ines@example.com
    export GIT_COMMITTER_NAME=Ines GIT_COMMITTER_EMAIL=ines@example.com
    export GIT_AUTHOR_DATE="2026-10-10 09:00:00 +0000" GIT_COMMITTER_DATE="2026-10-10 09:00:00 +0000"
    mkdir -p src docs
    {
      echo "//! The planet table."
      echo
      for n in 1 2 3 4 5 6 7 8 9; do echo "pub const PLANET_$n: &str = \"planet $n\";"; done
      echo "pub fn moons() -> Vec<&'static str> { Vec::new() }"
      echo "pub fn count() -> usize { moons().len() }"
      echo "// the end"
    } > src/planets.rs
    printf '# Moons\n\nThe moons column and the data behind it.\n' > docs/moons.md
    git add -A
    git -c commit.gpgsign=false commit -q -m "feat: moons in the planet table"
    git push -q origin HEAD:refs/merge-requests/58/head
  )
  rm -rf "$tmp"
}
# Opens the review of !58 from its row's menu (the All group is folded by default).
open_review() {
  wait_text "Add moons" 20
  rclick_text "Add moons" 1 2; wait_text "Review..."
  click_text "Review..." 1 1; wait_text "Review !58" 15; sleep 1.2
}
shot_review-overview() {   # the single-page project of conv_up: both threads on the Overview
  conv_up; open_pr
  shot review-overview
  conv_down
}
shot_review-files() {
  mock_up
  GPD_CONFIG_EXTRA=$(mock_profiles); export GPD_CONFIG_EXTRA
  lp_link; review_head
  restart_tui ""
  open_review
  key alt+2; wait_text "Why a Vec here" 20; sleep 0.8
  shot review-files; esc_all
  lp_unlink; unset GPD_CONFIG_EXTRA; stop_tui
}
# The ✦ AI review (slice R3): "Suggest review" on the same request, the demo agent
# answers with two comments; the Files page shows them as ghost comments under their lines.
shot_review-ai() {
  mock_up
  GPD_CONFIG_EXTRA=$(mock_profiles); export GPD_CONFIG_EXTRA
  lp_link; review_head
  restart_tui ""
  open_review
  key s; wait_text "Review options" 10; key Return; ai_send; wait_text "Suggestions (2)" 30; sleep 0.5
  key alt+2; wait_text "allocates on every call" 20; sleep 0.8
  shot review-ai; esc_all
  lp_unlink; unset GPD_CONFIG_EXTRA; stop_tui
}
# Local issues (slice L1): a folder of Markdown notes as the ISSUES source, no
# token and no network (the repository's own `docs/issues`), with its row menu.
shot_local-issues() {
  local notes="$GPD_ROOT/fx/orbit/docs/issues"
  mkdir -p "$notes" "$GPD_ROOT/fx/orbit/.git/gitpoulpi"
  printf -- '---\nid: 1\ntitle: Orbit chart flickers on resize\nstatus: doing\nassignees: [mara]\nlabels: [bug, ui]\nmilestone: v0.1\ncreated: 2026-10-12\n---\nSeen on a 4K terminal.\n' > "$notes/1-orbit-chart-flickers-on-resize.md"
  printf -- '---\nid: 2\ntitle: Add a moons overlay\nstatus: todo\nlabels: [feature]\ncreated: 2026-10-13\n---\n' > "$notes/2-add-a-moons-overlay.md"
  printf -- '---\nid: 3\ntitle: Document the board\nstatus: todo\ncreated: 2026-10-14\n---\n' > "$notes/3-document-the-board.md"
  printf '[issues_source]\nintegration = "notes"\nproject = "docs/issues"\n' > "$GPD_ROOT/fx/orbit/.git/gitpoulpi/repo-state.toml"
  GPD_CONFIG_EXTRA=$(printf '\n[[integrations]]\nid = "notes"\nkind = "local_markdown"\nurl = "docs/issues"\ntoken_ref = "integration:notes"\nname = "notes"\nuser = { login = "mara" }\n'); export GPD_CONFIG_EXTRA
  restart_tui ""
  wait_text "Orbit chart flickers" 20
  rclick_text "Orbit chart flickers" 1 2; wait_text "Close issue"; sleep 0.5
  shot local-issues; esc_all
  rm -rf "$GPD_ROOT/fx/orbit/docs/issues" "$GPD_ROOT/fx/orbit/.git/gitpoulpi"; unset GPD_CONFIG_EXTRA; stop_tui
}
# The Kanban board (slice L2) of the same local folder: columns by status,
# cards with initials, label chips and a milestone, the selected card ready to move.
shot_issue-board() {
  local notes="$GPD_ROOT/fx/orbit/docs/issues"
  mkdir -p "$notes" "$GPD_ROOT/fx/orbit/.git/gitpoulpi"
  printf -- '---\nid: 1\ntitle: Orbit chart flickers on resize\nstatus: doing\nassignees: [mara]\nlabels: [bug, ui]\nmilestone: v0.1\ncreated: 2026-10-12\n---\nSeen on a 4K terminal.\n' > "$notes/1-orbit-chart-flickers-on-resize.md"
  printf -- '---\nid: 2\ntitle: Add a moons overlay\nstatus: todo\nassignees: [tomas, ines]\nlabels: [feature]\nmilestone: v0.2\ncreated: 2026-10-13\n---\n' > "$notes/2-add-a-moons-overlay.md"
  printf -- '---\nid: 3\ntitle: Document the board\nstatus: todo\ncreated: 2026-10-14\n---\n' > "$notes/3-document-the-board.md"
  printf -- '---\nid: 4\ntitle: Release notes for v0.1\nstatus: review\nassignees: [mara]\nlabels: [docs]\nmilestone: v0.1\ncreated: 2026-10-14\n---\n' > "$notes/4-release-notes-for-v0-1.md"
  printf -- '---\nid: 5\ntitle: Fix the graph lanes\nstatus: done\nassignees: [ines]\nlabels: [bug]\ncreated: 2026-10-10\n---\n' > "$notes/5-fix-the-graph-lanes.md"
  printf '[issues_source]\nintegration = "notes"\nproject = "docs/issues"\n' > "$GPD_ROOT/fx/orbit/.git/gitpoulpi/repo-state.toml"
  GPD_CONFIG_EXTRA=$(printf '\n[[integrations]]\nid = "notes"\nkind = "local_markdown"\nurl = "docs/issues"\ntoken_ref = "integration:notes"\nname = "notes"\nuser = { login = "mara" }\n'); export GPD_CONFIG_EXTRA
  restart_tui ""
  wait_text "Orbit chart flickers" 20
  key ctrl+p; wait_text "Command Palette"; text "Open Board"; wait_text "Issues: Open Board"; key Return
  wait_text "Issue board" 20; wait_text "Add a moons overlay" 20; sleep 0.8
  shot issue-board; esc_all
  rm -rf "$GPD_ROOT/fx/orbit/docs/issues" "$GPD_ROOT/fx/orbit/.git/gitpoulpi"; unset GPD_CONFIG_EXTRA; stop_tui
}
shot_clone() { esc_all; key ctrl+o; wait_text "OPEN REPOSITORIES"; clickt_home Clone; wait_text "Clone"; shot clone; esc; }
# The fake hosting service (`cargo run -p xtask -- mock-host`): a GitHub and a
# GitLab side on 127.0.0.1 with several owners each, nothing leaves the
# machine. Started on first use, stopped by the cleanup trap (mock_down).
mock_up() {
  [ -n "${GPD_MOCK_PORT:-}" ] && return 0
  local xt="${GPD_XTASK:-$repo/target/debug/xtask}"
  [ -x "$xt" ] || (cd "$repo" && cargo build -q -p xtask) || die "cannot build xtask"
  "$xt" mock-host --bare-root "$GPD_ROOT/hosting" > "$GPD_ROOT/mock-host.out" 2>"$GPD_ROOT/mock-host.err" &
  GPD_MOCK_PID=$!
  local i
  for i in $(seq 1 100); do
    GPD_MOCK_PORT=$(head -1 "$GPD_ROOT/mock-host.out" 2>/dev/null || true)
    [ -n "$GPD_MOCK_PORT" ] && return 0
    sleep 0.1
  done
  die "the mock host did not start: $(cat "$GPD_ROOT/mock-host.err")"
}
mock_down() { [ -n "${GPD_MOCK_PID:-}" ] && kill "$GPD_MOCK_PID" 2>/dev/null || true; }
mock_profiles() {
  cat <<T

[[integrations]]
id = "hub"
kind = "github_enterprise"
url = "http://127.0.0.1:$GPD_MOCK_PORT"
token_ref = "integration:hub"
name = "hub"
user = { login = "mara-quinn" }

[[integrations]]
id = "lab"
kind = "gitlab_self_managed"
url = "http://127.0.0.1:$GPD_MOCK_PORT"
token_ref = "integration:lab"
name = "lab"
user = { login = "mara" }
T
}
# The Clone dialog on a connected GitHub account: one section per owner.
shot_clone-hosted() {
  mock_up
  GPD_CONFIG_EXTRA=$(mock_profiles); export GPD_CONFIG_EXTRA
  restart_tui ""
  key ctrl+o; wait_text "OPEN REPOSITORIES"; clickt_home Clone
  wait_text "GitHub · hub"; click_text "GitHub · hub" 1 2
  wait_text "harbor-guild (2)"; shot clone-hosted
}
# ... and the search narrowing it (the dialog is still open from the shot above).
shot_clone-hosted-search() {
  screen | grep -qF "harbor-guild (2)" || shot_clone-hosted
  local p; p=$(find_text "Where to clone to") || die "no clone dialog"; set -- $p
  click $(($1 + 2)) $(($2 + 4)); text "lumen"
  wait_text "lumen-labs (5)"; shot clone-hosted-search
  esc; esc; unset GPD_CONFIG_EXTRA; stop_tui   # the next shot restarts without the fake profiles
}
shot_init()  { screen | grep -qF "OPEN REPOSITORIES" || { esc_all; key ctrl+o; wait_text "OPEN REPOSITORIES"; }; clickt_home Init; wait_text "Initialize in"; shot init; esc; esc; }
clickt_home() { click_text "$1" 1 1; sleep 0.8; }
shot_log-graph() {
  esc_all; click_text "b7ef09f" 1 1; wait_text "Merge branch 'feature/moons'"; shot log-graph
}
shot_log-columns-menu() {
  esc_all; rclick_text "Author" 1 2; wait_text "Changes"; shot log-columns-menu
}
shot_log-changes-column() {
  esc_all; rclick_text "Author" 1 2; wait_text "Columns (click"
  click_text "[ ] Changes" 1 5; sleep 0.8; esc_all
  local p; p=$(find_text "╭›"); set -- $p; local c=$1 r=$(($2 + 26))
  drag $c $r $((c + 26)) $r; sleep 0.6
  shot log-changes-column
  p=$(find_text "╭›"); set -- $p; drag $1 $r $c $r; sleep 0.5
  rclick_text "Message" 1 2; wait_text "Columns (click"; click_text "[x] Changes" 1 5; sleep 0.5; esc_all   # back to the default columns
}
shot_context-menu() {
  esc_all; rclick_text "477b211" 1 1; wait_text "Cherry"; shot context-menu; esc_all
}
shot_diff() {
  select_wip; click_text "table.rs" 1 1; wait_text "Hunk"; shot diff
}
shot_diff-split() { sleep 0.2; click_text "Split" 1 1; sleep 0.6; shot diff-split; click_text "Inline" 1 1; esc_all; }
# The Blame and File History shots run with the minimap on and the avatars
# drawn (the offline fixture avatars, as in compare-cards); restart_tui "" restores.
blame_open() {
  restart_tui 'avatars = "graph"
minimap = true'
  select_wip; click_text "planet.rs" 1 1; wait_text "Blame"; click_text "Blame" 1 1; sleep 1
}
shot_blame()      { blame_open; shot blame; esc_all; restart_tui ""; }
# blame-card: the hunk card of the selected hunk (`i`).
shot_blame-card() { blame_open; key n; key n; key i; sleep 1; shot blame-card; esc_all; restart_tui ""; }
# file-history: a commit in the middle of the list selected, minimap on.
history_open() {
  restart_tui 'avatars = "graph"
minimap = true'
  select_wip; click_text "planet.rs" 1 1; wait_text "Blame"; click_text "File" 2 1; sleep 1.2
  key Down; key Down; sleep 1
}
shot_file-history()       { history_open; shot file-history; esc_all; restart_tui ""; }
# file-history-blame: the same selection with the right side in Blame mode (Alt+3).
shot_file-history-blame() { history_open; key alt+3; sleep 1.5; shot file-history-blame; esc_all; restart_tui ""; }
# editor_open: the editor on table.rs of the working changes (highlighting done).
editor_open() {
  select_wip; click_text "table.rs" 1 1; wait_text "Edit"; click_text "Edit" 1 1; sleep 1
  local i; for i in $(seq 1 40); do   # an idle editor may keep "highlighting..." until the next input: nudge it
    screen | grep -qF "highlighting" || break; raw '\e[<35;60;30M'; sleep 0.3
  done
  wait_text "Rust" 20; sleep 0.5
}
shot_editor()       { editor_open; shot editor; esc_all; esc_all; }
shot_editor-encoding-menu() { editor_open; click_text "UTF-8" 1 1; sleep 0.8; shot editor-encoding-menu; esc; esc_all; esc_all; }
shot_editor-eol-menu() { editor_open; click_text " LF " 1 1; sleep 0.8; shot editor-eol-menu; esc; esc_all; esc_all; }
# ai_send: the "Review what is sent" dialog opens before every AI request; Send is its focused button.
ai_send() {
  local i
  for i in $(seq 1 40); do
    if screen | grep -qF "Review what is sent"; then key Return; return 0; fi
    sleep 0.1
  done
}
shot_composer()     { select_wip; click_text "Compose commits with AI" 1 2; ai_send; wait_text "The AI proposes"; shot composer; esc_all; }
shot_ai-commit-message() {
  select_wip; click_text "✦ 72" 1 0; ai_send; wait_text "status section" 15; shot ai-commit-message
}
shot_ai-explain()   { select_wip; key ctrl+g; key w; ai_send; wait_text "double-check" 15; shot ai-explain; esc_all; }
shot_help()         { esc_all; text "?"; wait_text "Hosting"; click_text "Keys" 1 1; wait_text "Mouse"; shot help; esc_all; }
shot_help-hosting() { esc_all; text "?"; wait_text "Hosting"; click_text "Hosting" 1 1; wait_text "Link a repository"; shot help-hosting; esc_all; }
about_open() {
  esc_all; key ctrl+p; wait_text "Command Palette"; text "About"; wait_text "About GitPoulpi"; key Return
  wait_text "Third-party crates"
}
shot_about() { about_open; shot about; esc_all; }
shot_about-crates() {
  about_open; text "l"; sleep 1
  shot about-crates; esc_all
}
# --- Activity Log, comparison, sparse checkout ---------------------
# A fetch that fails (origin points at a folder that does not exist, restored after
# the Git operations shot) gives the Activity Log an error entry to show.
shot_activity-log() {
  esc_all
  git -C "$GPD_ROOT/fx/orbit" remote set-url origin "$GPD_ROOT/fx/no-such-remote.git"
  key ctrl+g; key f; wait_text "Fetch failed" 20; sleep 0.8
  click_text "Activity" 1 3; wait_text "Application"; click_text "Application" 1 2; sleep 0.8
  wait_text "Show:"; sleep 0.6
  shot activity-log
}
shot_activity-log-repository() {
  click_text "Git operations" 1 2; sleep 0.8; wait_text "Scope"
  shot activity-log-repository; esc_all
  git -C "$GPD_ROOT/fx/orbit" remote set-url origin "$GPD_ROOT/fx/orbit.git"
}
shot_compare-cards() {
  restart_tui 'avatars = "graph"'
  click_text "b7ef09f" 1 1; sleep 0.5
  key space; key shift+Down; key shift+Down; sleep 1
  shot compare-cards; esc_all; restart_tui ""
}
# Copy as Mermaid gitGraph: three rows selected (as in compare-cards), the
# popup open on its live preview.
shot_log-mermaid() {
  click_text "b7ef09f" 1 1; sleep 0.5
  key space; key shift+Down; key shift+Down; sleep 0.5
  key shift+y; wait_text "Copy as Mermaid gitGraph"; sleep 0.8
  shot log-mermaid; esc_all
}
shot_sparse-checkout() {
  esc_all; key ctrl+p; wait_text "Command Palette"; text "Sparse"; wait_text "Repository: Sparse Checkout..."; key Return
  wait_text "Sparse checkout"; text "src docs"; sleep 0.6
  shot sparse-checkout; esc; esc_all
}

shot_theme-picker() {   # App: Change Theme... with the Colored prefilter
  esc_all; key ctrl+p; wait_text "Command Palette"; text "Change Theme"; wait_text "App: Change Theme"; key Return
  wait_text "Colored"; click_text "Colored" 1 1; sleep 1.2; shot theme-picker; esc; esc_all
}
shot_rebase() {
  esc_all; rclick_text "b0072d4" 1 1; wait_text "Interactively"; click_text "Interactively" 1 3
  wait_text "Start rebase" 10
  # the second commit: Squash, the third: Drop (the action dropdown opens on Enter, the first letter picks)
  key Down; text s; key Down; text d; sleep 0.5
  shot rebase; esc; esc
}

# --- Settings ----------------------------------------------------------------
settings_open() { esc_all; key ctrl+comma; wait_text "Settings - Tab"; sleep 0.5; }
settings_page() { screen | grep -qF "Settings - Tab" || settings_open; GPD_MAXCOL=28 click_text "$1" "${2:-1}" 3; sleep 0.7; }
# settings_tab <label>: a sub-tab of a master-detail page (General, or the list
# tab's name). The strip reads " General   <label> ", unlike the sidebar rows.
settings_tab() {   # General is the strip's first match: it is above the sidebar's own row
  if [ "$1" = General ]; then click_text " General " 1 2; else click_text "General   $1" 1 11; fi; sleep 0.7
}
shot_settings()          { settings_open; settings_page "General"; shot settings; }
shot_settings-launcher() {   # the Launcher page (own sidebar entry)
  settings_open; settings_page "Launcher"; sleep 0.6
  shot settings-launcher
}
shot_settings-conflict-prevention() {   # the global Conflict prevention page
  settings_open; settings_page "Conflict prevention"; sleep 0.6
  shot settings-conflict-prevention
}
shot_settings-search()   { click_text "Search settings" 1 2; text "diff"; sleep 0.8; shot settings-search; esc; esc; }
shot_settings-profiles() { settings_open; settings_page "Profiles"; settings_tab "Profiles"; shot settings-profiles; }
shot_settings-ssh()      { settings_page "SSH"; shot settings-ssh; }
shot_settings-integrations() { settings_page "Integrations"; shot settings-integrations; }
shot_settings-connect()  { settings_page "Integrations"; click_text "Connect" 2 2; sleep 0.8; shot settings-connect; click_text "Cancel" 1 2; }
shot_settings-ai()       { settings_page "AI"; settings_tab "General"; sleep 0.5; shot settings-ai; }
shot_settings-ai-agent()    { settings_page "AI"; settings_tab "Profiles (connections)"; for i in 1 2 3 4 5 6 7 8; do wheel 100 30 1; done; sleep 0.5; shot settings-ai-agent; }
shot_settings-appearance-theme() {
  settings_page "Appearance"; click_text "System (follows" 1 3; sleep 0.8
  key Down; key Down; sleep 1.2   # Dark, then Light: the whole screen repaints under the open list
  shot settings-appearance-theme; esc; esc   # Esc restores the previous theme
}
shot_settings-log-graph-lanes() {
  settings_page "Appearance"; local i; for i in 1 2 3 4 5 6; do find_text "Graph color theme" >/dev/null && break; wheel 100 30 1; sleep 0.3; done; sleep 0.4; click_text "Graph color theme" 1 60; sleep 0.8
  key Down; key Down; key Down; sleep 1; shot settings-log-graph-lanes; esc; esc
}
shot_settings-open-in() { GPD_CLEAN_ENV=1 restart_tui ""; settings_page "Open in"; settings_tab "Entries"; sleep 0.5; shot settings-open-in; }
shot_settings-open-in-presets() { settings_page "Open in"; settings_tab "Entries"; click_text "+ Add" 1 3; sleep 0.8; shot settings-open-in-presets; esc; esc_all; GPD_CLEAN_ENV= restart_tui ""; }
shot_settings-ui()       { settings_page "Appearance"; shot settings-ui; }
shot_settings-editor()   { settings_page "Diff & editor"; shot settings-editor; }
shot_settings-git()      { settings_page "Git & encoding"; shot settings-git; }
shot_settings-repo-ai()  { settings_page "AI" 2; shot settings-repo-ai; esc_all; }
shot_settings-keyboard() { settings_page "Keyboard & mouse"; settings_tab "Keybindings"; sleep 0.5; shot settings-keyboard; }
shot_settings-keyboard-conflicts() {   # stage_toggle rebound to q, which another action owns
  GPD_CONFIG_EXTRA='[keybindings.status]
stage_toggle = "q"'; export GPD_CONFIG_EXTRA
  restart_tui ""
  settings_open; settings_page "Keyboard & mouse"; settings_tab "Keybindings"; sleep 0.5
  GPD_MAXCOL=60 click_text "Quit" 1 2; sleep 0.5   # the row whose q clashes (red)
  text "c"; sleep 0.8
  shot settings-keyboard-conflicts; esc; esc_all
  unset GPD_CONFIG_EXTRA; stop_tui
}
shot_settings-language() { settings_page "General"; click_text "English" 1 1; sleep 0.8; shot settings-language; esc; }
shot_settings-ai-presets() {   # the Agent type dropdown of a local agent profile
  settings_page "AI"; settings_tab "Profiles (connections)"; click_text "(claude)" 1 1; sleep 0.8; shot settings-ai-presets; esc; esc_all
}

# --- branching workflows (slice W3) -----------------------------------------------
# A Git Flow workflow file for the demo repository (and a develop branch): the
# sidebar groups the branches by type, the toolbar gets `Flow`, the Start form
# builds a branch name. The file is removed after the shots.
workflow_fixture() {
  local r="$GPD_ROOT/fx/orbit"
  git -C "$r" branch -q develop main 2>/dev/null || true
  mkdir -p "$r/.gitpoulpi"
  cat > "$r/.gitpoulpi/workflow.toml" <<'T'
template = "git-flow"

[[base]]
name = "main"
names = ["master"]
protected = true

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
parent = "develop"
finish = [{ into = "develop", merge = "merge" }]
delete_after_finish = true

[[topic]]
id = "hotfix"
label = "Hotfix"
pattern = "hotfix/{version}"
parent = "main"
finish = [{ into = "main", merge = "merge", tag = "v{version}" }, { into = "develop", merge = "merge" }]
T
}
workflow_fixture_off() { rm -f "$GPD_ROOT/fx/orbit/.gitpoulpi/workflow.toml"; git -C "$GPD_ROOT/fx/orbit" branch -q -D develop 2>/dev/null || true; }
# workflow_up: the workflow file is there and the TUI has read it (any shot of
# this group can run alone).
workflow_up() {
  esc_all
  screen | grep -qF "Feature (" && return 0
  workflow_fixture; restart_tui ""; wait_text "Feature (" 20
}
shot_workflow-settings() {   # Settings > Repository > Workflow, the template and the bases
  workflow_up
  settings_open; settings_page "Workflow"; settings_tab "General"; wait_text "Workflow template" 10; sleep 0.6
  shot workflow-settings; esc_all
}
shot_workflow-sidebar() {    # LOCAL grouped by type, base badges, the Flow button
  workflow_up; sleep 0.8
  shot workflow-sidebar
}
shot_workflow-start() {      # Flow > Start Feature..., the name builds as the slug is typed
  workflow_up; click_text "Flow" 1 2; wait_text "Start Feature..." 10; click_text "Start Feature..." 1 3
  wait_text "Start from" 10; text "Add ASCII planets"; wait_text "feature/add-ascii-planets" 10; sleep 0.6
  shot workflow-start; esc_all
  workflow_fixture_off; restart_tui ""
}

# --- file lists: menus and multi-select, the ignore list, Open with -------------
shot_file-pane-multiselect() {
  select_wip; click_text "README.md" 1 3; sleep 0.4
  local p; p=$(find_text "planet.rs"); set -- $p; raw "\e[<4;$(($1 + 3));$2M\e[<4;$(($1 + 3));$2m"; sleep 0.6   # shift+click
  right_click $(($1 + 3)) $2; wait_text "files"; sleep 0.5
  shot file-pane-multiselect; esc; esc_all
}
shot_commit-file-menu() {
  esc_all; click_text "0361f44" 1 1; sleep 1
  local p; p=$(find_text "table.rs" 1) || die "no file"; set -- $p
  right_click $(($1 + 2)) $2; wait_text "Open Diff"; sleep 0.5
  shot commit-file-menu; esc; esc_all
}
# commit-column-all: the All list beside a commit's file diff, Ref part on.
# The parts are the column's own config keys (not the Log's).
commit_column_open() {
  restart_tui 'commit_column_columns_visible = ["ref", "graph", "message", "author", "date"]'
  esc_all; click_text "0361f44" 1 1; sleep 1
  click_text "table.rs" 1 2; wait_text "Hunk"; sleep 0.6
  screen | grep -qF "Graph" || { click_text "All" 1 1; sleep 0.8; }
  # widen the column (drag the divider, the left border of the view, four cells
  # right of the column's close mark) so the Author part fits as well
  local p; p=$(GPD_MAXCOL=80 find_text "✕" 1) || die "no commit column"; set -- $p
  drag $(($1 + 4)) 30 $(($1 + 24)) 30; sleep 0.6
  raw '\e[<35;150;40M'; sleep 0.6   # pointer off the divider (it stays lit)
}
shot_commit-column-all() { commit_column_open; shot commit-column-all; }
# commit-column-menu: the header's right-click menu with its check boxes. It
# continues from commit-column-all (the widened width is remembered, so
# opening it again would widen it twice) and restores the default config.
shot_commit-column-menu() {
  { screen | grep -qF "src/table.rs" && screen | grep -qF "Author"; } || commit_column_open
  rclick_text "Graph" 1 1; wait_text "Date"; sleep 0.6
  shot commit-column-menu; esc; esc_all; restart_tui ""
}
shot_ignore-picker() {
  select_wip; rclick_text "planet.rs" 1 3; wait_text "Ignore"; click_text "Ignore" 1 2; sleep 0.8
  click_text "Ignore 'planet.rs'" 1 3; wait_text ".gitignore" 5; sleep 0.6
  shot ignore-picker; esc; esc; esc_all
}
shot_open-with() {
  printf '%%PDF-1.4\0\1\2\3 manual\n' > "$GPD_ROOT/fx/orbit/docs/manual.pdf"
  sleep 1.5; select_wip; click_text "manual.pdf" 1 3; sleep 1.2
  click_text "File View" 1 2; sleep 1.2
  text "O"; wait_text "Evince" 10; sleep 0.5
  shot open-with; esc; esc_all
  rm -f "$GPD_ROOT/fx/orbit/docs/manual.pdf"; sleep 1
}

# --- toolbar and menus -------------------------------------------------------
shot_undo-toolbar() {
  select_wip; click_text "Stage all changes" 1 3; sleep 1
  hover_text "Undo" 1 2; shot undo-toolbar
  click_text "Undo" 1 2; sleep 1; raw '\e[<35;5;30M'
}
shot_open-in-menu() { GPD_CLEAN_ENV=1 restart_tui ""; click_text "Open in ▾" 1 3; sleep 0.8; shot open-in-menu; esc; GPD_CLEAN_ENV= restart_tui ""; }
shot_pull-push-tooltip() { esc_all; click_text "Pull" 1 6; sleep 0.8; shot pull-push-tooltip; esc; }
# The LFS button only exists on a repository where LFS is initialized or in use:
# initialize it for this shot (local hooks and filter only), restore afterwards.
shot_lfs-menu() {
  git -C "$GPD_ROOT/fx/orbit" lfs install --local >/dev/null; restart_tui ""; wait_text "LFS ▾" 20
  click_text "LFS ▾" 1 1; sleep 0.8; shot lfs-menu; esc_all
  git -C "$GPD_ROOT/fx/orbit" lfs uninstall --local >/dev/null 2>&1 || true
  rm -f "$GPD_ROOT"/fx/orbit/.git/hooks/{pre-push,post-checkout,post-commit,post-merge}
  restart_tui ""
}
shot_tab-picker() {
  esc_all; second_tab open; esc_all; text "t"; sleep 0.8; shot tab-picker; esc_all
}

# --- themes -----------------------------------------------------------------
shot_theme-serie() {
  restart_tui 'graph_color_theme = "serie"
graph_lane_background = "row_tint"'
  click_text "b7ef09f" 1 1; sleep 0.6; shot theme-serie
}
shot_theme-material() {
  restart_tui 'graph_color_theme = "material"
graph_lane_background = "row_tint"'
  click_text "b7ef09f" 1 1; sleep 0.6; shot theme-material
  restart_tui ""
}

# the main window in a whole theme (ui.theme), a merge commit selected
theme_shot() {
  restart_tui "theme = \"$1\""
  click_text "b7ef09f" 1 1; sleep 0.8; shot "theme-$1"
}
# every built-in preset (the gallery in docs/user/themes.md), registry order
THEME_SLUGS=(dark light catppuccin-latte catppuccin-mocha cyberpunk dracula
  everforest-dark gruvbox-dark gruvbox-light kanagawa
  matrix fire fire-light sunset monokai nord one-dark rose-pine solarized-dark
  solarized-light terminal tokyo-night catppuccin-frappe catppuccin-macchiato kanagawa-lotus
  one-light rose-pine-dawn rose-pine-moon tokyo-night-day vesper github-dark
  github-light vscode-dark vscode-light apprentice ayu-dark ayu-light
  ayu-mirage cobalt2 deus dracula-soft everforest-light github-dark-colorblind
  github-dark-dimmed github-dark-high-contrast github-dark-legacy github-light-colorblind github-light-high-contrast github-light-legacy
  gotham gruvbox-dark-hard-contrast gruvbox-dark-soft-contrast gruvbox-light-hard-contrast gruvbox-light-soft-contrast iceberg-dark
  iceberg-light jellybeans kanagawa-dragon lucario miasma moonfly
  night-owl-dark night-owl-light nightfly nightfox nightfox-carbonfox nightfox-dawnfox
  nightfox-dayfox nightfox-duskfox nightfox-nordfox nightfox-terafox noctis noctis-azureus
  noctis-bordo noctis-hibernus noctis-lilac noctis-lux noctis-minimus noctis-obscuro
  noctis-sereno noctis-uva noctis-viola nordic one-half-dark one-half-light
  panda posterpole posterpole-gray seoul256-dark seoul256-light shades-of-purple
  shades-of-purple-super-dark sonokai sonokai-andromeda sonokai-atlantis sonokai-espresso sonokai-maia
  sonokai-shusia srcery tender tokyo-night-moon tokyo-night-storm tomorrow
  tomorrow-night tomorrow-night-blue tomorrow-night-bright tomorrow-night-eighties zenbones-duckbones zenbones-forestbones-dark
  zenbones-forestbones-light zenbones-kanagawabones zenbones-neobones-dark zenbones-neobones-light zenbones-nordbones zenbones-rosebones-dark
  zenbones-rosebones-light zenbones-seoulbones-dark zenbones-seoulbones-light zenbones-tokyobones-dark zenbones-tokyobones-light zenbones-vimbones
  zenbones-zenburned zenbones-zenwritten-dark zenbones-zenwritten-light)
for _slug in "${THEME_SLUGS[@]}"; do
  eval "shot_theme-$_slug() { theme_shot $_slug; }"
done
unset _slug
# the last one puts the default theme back for the shots after it
shot_theme-zenbones-zenwritten-light() { theme_shot zenbones-zenwritten-light; restart_tui ""; }

# --- undo history, reflog, lost work ----------------------------------------------
# A copy of orbit (so the other shots keep their repository) where a branch with
# two commits of its own was deleted and two commits were made after it, the
# deletion and the commits through the CLI so the undo journal knows them.
undo_repo() {
  local r="$GPD_ROOT/fx/orbit-undo"
  [ -d "$r" ] && return 0
  cp -a "$GPD_ROOT/fx/orbit" "$r"
  local g=(env HOME="$GPD_ROOT/fx/home" GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null TZ=UTC
    GIT_AUTHOR_NAME=Alice GIT_AUTHOR_EMAIL=alice@example.com GIT_COMMITTER_NAME=Alice GIT_COMMITTER_EMAIL=alice@example.com git -C "$r")
  local c=(env HOME="$GPD_ROOT/fx/home" GITPOULPI_CONFIG_DIR="$GPD_ROOT/undo-config" GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null TZ=UTC "$CLI" -s gpundo --repo "$r")
  "${g[@]}" stash drop -q || true
  "${g[@]}" checkout -q -- . ; "${g[@]}" clean -qfd
  "${g[@]}" checkout -q -b experiment/rings
  echo "rings: 3 of 8" > "$r/rings.txt"; "${g[@]}" add rings.txt; "${g[@]}" commit -q -m "feat(render): draw the rings of Saturn"
  echo "rings: tilt 26.7" >> "$r/rings.txt"; "${g[@]}" commit -q -a -m "feat(render): tilt the rings"
  "${g[@]}" checkout -q main
  "${c[@]}" branch delete experiment/rings >/dev/null 2>&1 || "${c[@]}" branch delete experiment/rings --force >/dev/null 2>&1 || true
  echo "Eight planets." >> "$r/README.md"
  "${c[@]}" stage --all >/dev/null; "${c[@]}" commit -m "docs: count the planets" >/dev/null
  echo "Moons are drawn after their planet." >> "$r/docs/design.md"
  "${c[@]}" stage --all >/dev/null; "${c[@]}" commit -m "docs: draw order of moons" >/dev/null
  "${c[@]}" daemon stop >/dev/null 2>&1 || true
  "${g[@]}" branch --list 'experiment/*' | grep -q . && echo "undo_repo: branch not deleted" >&2
  return 0
}
undo_open() { undo_repo; restart_tui "" "$GPD_ROOT/fx/orbit-undo"; }
shot_undo-history() {
  undo_open
  key alt+z; wait_text "Undo history" 10; sleep 0.8
  # the deleted branch is an older restorable row: select it
  local i; for i in 1 2 3 4; do key Down; done; sleep 0.6
  shot undo-history; esc_all
}
shot_reflog() {
  undo_open
  rclick_text "main" 1 4; wait_text "Reflog" 10; click_text "Reflog" 1 2; wait_text "Past positions" 10; sleep 0.8
  key Down; sleep 0.6
  shot reflog; esc_all
}
shot_lost-work() {
  undo_open
  key ctrl+p; wait_text "Command Palette"; text "Lost"; sleep 0.6; key Return
  wait_text "Lost work" 10; sleep 2
  shot lost-work; esc_all; restart_tui ""
}

# --- recent features: history rewrite, tag delete, number + unit rows -----------
# The history rewrite workbench (Ctrl+G Shift+H): two "Replace an author" rules
# filled in by keys, then Preview (F5). Nothing is applied: the TUI restarts.
rewrite_rule() {   # rewrite_rule <old email> <new email>
  text a; wait_text "Replace an author"; key Return; wait_text "New email"
  key Down; key Return; text "$1"; key Return
  key Down; key Down; key Down; key Return; text "$2"; key Return
  wait_text "$2"
}
shot_history-rewrite() {
  esc_all; key ctrl+g; text H; wait_text "Add a rule to start" 15
  rewrite_rule dana@example.com dana@example.org
  rewrite_rule chen@example.com chen@example.org
  click_text "Current branch" 1 2; sleep 0.6; click_text "Chosen branches" 1 2; sleep 1.2; click_text "Choose..." 1 2; sleep 1.2; click_text "feature/moons" 1 2; click_text "[x] main" 1 2; sleep 0.5; esc; sleep 0.5
  key F5; wait_text "keep their id" 30; sleep 1
  shot history-rewrite; esc_all; restart_tui ""
}
# The tag delete dialog: v0.1.0 and v0.2.0 are on origin and here (both
# markers), v0.3.0 is taken off origin first so it reads "only here"; origin
# gets it back afterwards.
shot_tag-delete() {
  esc_all
  git -C "$GPD_ROOT/fx/orbit" push -q origin :refs/tags/v0.3.0
  key ctrl+g; key f; wait_text "fetched every remote" 20; sleep 0.8
  rclick_text "v0.1.0" 1 1; wait_text "Delete v0.1.0..." 10; click_text "Delete v0.1.0..." 1 2
  wait_text "Where should v0.1.0 be deleted?" 10; sleep 0.8
  shot tag-delete; esc; esc_all
  git -C "$GPD_ROOT/fx/orbit" push -q origin v0.3.0
  restart_tui ""
}
# Settings > Synchronization: the auto-fetch interval and the script time limit
# are a number and a unit on one line.
shot_settings-quantity() { restart_tui ""; settings_open; settings_page "Synchronization"; sleep 0.5; shot settings-quantity; esc_all; }

# --- merge conflicts (own repository, run last) ---------------------------------
shot_conflict-banner() {
  restart_tui "" "$GPD_ROOT/fx/orbit-conflict"; wait_text "Merge in progress"; shot conflict-banner
}
shot_merge-editor() {
  click_text "orbit.rs" 1 1; key Return; wait_text "Conflict 1 /" 10
  text a; key F7; text b; sleep 0.6
  shot merge-editor
}
shot_merge-editor-full-file() { click_text "Full file" 1 2; sleep 0.8; shot merge-editor-full-file; click_text "Full file" 1 2; }
shot_merge-editor-card() {
  click_text "logo.bin" 1 1; key Return; wait_text "Keep A" 10; sleep 0.5
  # the file on disk is not git's merge text for a binary: take the editor's own start
  screen | grep -qF "edited outside the editor" && { click_text "Start over from the merge" 1 3; sleep 0.8; }
  shot merge-editor-card; esc
}

# --- worktrees and submodules (own repository: orbit-tools) -----------------------
# orbit-tools has two linked worktrees, a submodule at a newer commit than the one the
# repository records (up arrow), one up to date and one declared but not cloned.
tools_tui() { restart_tui "" "$GPD_ROOT/fx/orbit-tools"; }
shot_worktrees() {
  tools_tui
  GPD_MAXCOL=38 rclick_text "hotfix/axis-labels" 1 2; wait_text "Remove worktree"; esc
  shot worktrees
}
shot_worktree-menu() {
  tools_tui
  GPD_MAXCOL=38 rclick_text "hotfix/axis-labels" 1 2; wait_text "Remove worktree"; sleep 0.4
  shot worktree-menu; esc
}
shot_worktree-create() {
  tools_tui
  GPD_MAXCOL=38 click_text "WORKTREES (3)" 1 28; wait_text "From the current branch"
  click_text "From the current branch" 1 3; wait_text "Name after branch"
  # the proposed names carry a random id: write readable ones
  key ctrl+a; key BackSpace; text "legend-colors"
  local p; p=$(find_text "Worktree branch") || die "no branch field"; set -- $p
  click $(($1 + 20)) "$2"; wait_text "type your own"; text "feature/legend-colors"; wait_text "Use \"feature"; key Return
  wait_text "new branch from main"; sleep 0.4
  shot worktree-create; esc; esc
}
shot_worktree-tab() {
  tools_tui
  GPD_MAXCOL=38 click_text "hotfix/axis-labels" 1 2; key Return
  wait_text "number the labels"; sleep 0.8
  shot worktree-tab
}
shot_submodules() {
  tools_tui
  GPD_MAXCOL=38 click_text "vendor/starfield" 1 3; sleep 0.5
  shot submodules
}
shot_submodule-menu() {
  tools_tui
  GPD_MAXCOL=38 rclick_text "vendor/starfield" 1 3; wait_text "Update (clone if needed)"; sleep 0.4
  shot submodule-menu; esc
}

# --- more merge conflicts ---------------------------------------------------------
# conflict_fresh <repo>: the demo repository back in its stopped state (a shot that
# resolves files must not leave them resolved for the next one), TUI on it.
conflict_fresh() {
  local r=$1 pr="$GPD_ROOT/pristine"
  stop_tui
  mkdir -p "$pr"
  if [ -d "$pr/$r" ]; then rm -rf "${GPD_ROOT:?}/fx/$r"; cp -a "$pr/$r" "$GPD_ROOT/fx/$r"; else cp -a "$GPD_ROOT/fx/$r" "$pr/$r"; fi
  start_tui "$GPD_ROOT/fx/$r"
}
open_conflict() { # open_conflict <file>
  click_text "$1" 1 1; key Return; wait_text "Conflict 1 /" 10; key alt+r; sleep 0.5
}
shot_merge-editor-chunk-menu() {
  conflict_fresh orbit-conflict; open_conflict orbit.rs
  rclick_text "] Conflict 2" 1 3; wait_text "A then B"; sleep 0.4
  shot merge-editor-chunk-menu; esc
}
shot_merge-editor-take-both() {
  conflict_fresh orbit-conflict; open_conflict orbit.rs
  rclick_text "] Conflict 2" 1 3; wait_text "A then B"; click_text "A then B" 1 2; sleep 0.6
  shot merge-editor-take-both
}
# orbit.rs resolved by hand: conflict 1 from A, conflict 2 both, conflict 3 typed in the Output.
resolve_orbit() {
  open_conflict orbit.rs
  rclick_text "] Conflict 1" 1 3; wait_text "Take A only"; click_text "Take A only" 1 2; sleep 0.4
  rclick_text "] Conflict 2" 1 3; wait_text "A then B"; click_text "A then B" 1 2; sleep 0.4
  rclick_text "] Conflict 3" 1 3; wait_text "Edit in Output"; click_text "Edit in Output" 1 2; sleep 0.6
  text "    (1.0 / a).sqrt().max(0.01)"; wait_text "1 edited"; sleep 0.4
}
shot_merge-editor-output-edited() {
  conflict_fresh orbit-conflict; resolve_orbit
  shot merge-editor-output-edited
}
shot_conflict-file-resolved() {
  conflict_fresh orbit-conflict; resolve_orbit
  click_text "Save & mark resolved" 1 3; wait_text "1 CONFLICTED" 10; sleep 0.6
  shot conflict-file-resolved
}
shot_conflict-commit-merge() {
  conflict_fresh orbit-conflict; resolve_orbit
  click_text "Save & mark resolved" 1 3; wait_text "1 CONFLICTED" 10; sleep 0.6
  click_text "logo.bin" 1 1; key Return; wait_text "Keep A" 10
  screen | grep -qF "edited outside the editor" && { click_text "Start over from the merge" 1 3; sleep 0.8; }
  click_text "Keep A" 1 3; wait_text "Commit merge" 10; sleep 0.6
  shot conflict-commit-merge
}
# A rebase stopped on a text conflict and a deleted-here, changed-there file.
shot_conflict-banner-rebase() {
  conflict_fresh orbit-rebase; wait_text "Rebasing feature/cleanup"
  shot conflict-banner-rebase
}
shot_merge-editor-deleted-card() {
  conflict_fresh orbit-rebase
  click_text "legacy.md" 1 1; key Return; wait_text "Accept the deletion" 10; sleep 0.5
  shot merge-editor-deleted-card
}
shot_conflict-banner-continue() {
  conflict_fresh orbit-rebase
  click_text "legacy.md" 1 1; key Return; wait_text "Accept the deletion" 10
  click_text "Accept the deletion" 1 3; wait_text "1 CONFLICTED" 10
  click_text "moons.rs" 1 1; key Return; wait_text "Conflict 1 /" 10
  key alt+r; sleep 0.4; text a; sleep 0.4   # reset what Return took, then pick A explicitly
  click_text "Save & mark resolved" 1 3; wait_text "Continue" 10; sleep 0.8
  shot conflict-banner-continue
}

# --- conversations (the fake hosting service) --------------------------------------
# The pull request is merge request !58 of `orbit-co/orbit-docs` (the one project the mock
# answers in a single page, see `SINGLE_PAGE_PROJECT` in xtask/src/mock_host/review.rs),
# the issue is #7 of the GitHub side.
conv_up() {
  mock_up
  GPD_CONFIG_EXTRA=$(mock_profiles); export GPD_CONFIG_EXTRA
  mkdir -p "$GPD_ROOT/fx/orbit/.git/gitpoulpi"
  printf '[pulls_source]\nintegration = "lab"\nproject = "orbit-co/orbit-docs"\n\n[issues_source]\nintegration = "hub"\nproject = "orbit-co/orbit"\n' \
    > "$GPD_ROOT/fx/orbit/.git/gitpoulpi/repo-state.toml"
  review_head
  restart_tui ""
}
conv_down() { lp_unlink; unset GPD_CONFIG_EXTRA; stop_tui; }
open_issue() { # the conversation of issue #7, from its sidebar row
  wait_text "Crash when the planet" 20
  GPD_MAXCOL=38 click_text "Crash when the planet" 1 2; key Return
  wait_text "Thanks, fixing it" 20; sleep 0.8
}
shot_issues-sidebar() {
  conv_up
  wait_text "Crash when the planet" 20
  GPD_MAXCOL=38 rclick_text "Crash when the planet" 1 2; wait_text "Open conversation"; sleep 0.4
  shot issues-sidebar; esc
  conv_down
}
shot_issue-conversation() {
  conv_up; open_issue
  shot issue-conversation
  conv_down
}
shot_issue-reply-composer() {
  conv_up; open_issue
  key c; sleep 0.5
  text "Thanks Ines! I can reproduce it with an empty save file too, **trying a fix** now:"
  key Return; key Return; text '```rust'; key Return
  text "assert!(planets().is_empty());"; key Return; text '```'
  click_text "Split" 1 2; wait_text "trying a fix"; sleep 0.8
  shot issue-reply-composer
  conv_down
}
open_pr() { # the review of !58, from its sidebar row; the Overview is its conversation
  wait_text "Add moons" 20
  GPD_MAXCOL=38 rclick_text "Add moons" 1 2; wait_text "Review..."
  click_text "Review..." 1 1; wait_text "Review !58" 15; wait_text "Thanks, answering below" 20; sleep 1
}
shot_pr-conversation() {
  conv_up; open_pr
  key Down; key Down; sleep 0.5
  shot pr-conversation
  conv_down
}
shot_pr-reply-composer() {
  conv_up; open_pr
  key c; sleep 0.5
  text "Thanks both, it is built once at start-up. I will add a comment saying so."
  key Return; key Return; text "- [x] answer the thread"; key Return; text "- [ ] add the comment"
  click_text "Split" 1 2; wait_text "built once at start-up" 10; sleep 0.8
  shot pr-reply-composer
  conv_down
}
