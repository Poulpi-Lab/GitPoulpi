# gitpoulpi shell integration (bash). Add to ~/.bashrc:
#   eval "$(gitpoulpi shell-init bash)"
# `gp` runs gitpoulpi, then cd's to the active tab's folder when you used
# "Quit and cd here" (or ui.tab_dir_follow = "auto"). Pick another name with
#   gitpoulpi shell-init bash --name <name>
if [ "$(type -t gp 2>/dev/null)" != "function" ] && command -v gp >/dev/null 2>&1; then
  echo "gitpoulpi shell-init: 'gp' already exists in this shell; not overriding it. Use: gitpoulpi shell-init bash --name <other-name>" >&2
else
  function gp {
    local tmp dir rc
    tmp="$(mktemp "${TMPDIR:-/tmp}/gitpoulpi-cwd.XXXXXX")" || return
    command gitpoulpi --cwd-file "$tmp" "$@"
    rc=$?
    if [ -s "$tmp" ]; then
      dir="$(cat -- "$tmp")"
      if [ -d "$dir" ] && [ "$dir" != "$PWD" ]; then
        builtin cd -- "$dir"
      fi
    fi
    rm -f -- "$tmp"
    return $rc
  }
fi
