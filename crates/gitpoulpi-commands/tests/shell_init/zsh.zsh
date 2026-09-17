# gitpoulpi shell integration (zsh). Add to ~/.zshrc:
#   eval "$(gitpoulpi shell-init zsh)"
# `gp` runs gitpoulpi, then cd's to the active tab's folder when you used
# "Quit and cd here" (or ui.tab_dir_follow = "auto"). Pick another name with
#   gitpoulpi shell-init zsh --name <name>
if (( ! ${+functions[gp]} )) && whence -w gp >/dev/null 2>&1; then
  print -u2 "gitpoulpi shell-init: 'gp' already exists in this shell; not overriding it. Use: gitpoulpi shell-init zsh --name <other-name>"
else
  function gp {
    local tmp dir rc
    tmp="$(mktemp "${TMPDIR:-/tmp}/gitpoulpi-cwd.XXXXXX")" || return
    command gitpoulpi --cwd-file "$tmp" "$@"
    rc=$?
    if [[ -s "$tmp" ]]; then
      dir="$(<"$tmp")"
      if [[ -d "$dir" && "$dir" != "$PWD" ]]; then
        builtin cd -- "$dir"
      fi
    fi
    rm -f -- "$tmp"
    return $rc
  }
fi
