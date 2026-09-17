//! `gitpoulpi shell-init <shell>`: the wrapper function that lets quitting
//! GitPoulpi leave the calling shell in the active tab's folder.
//!
//! A program cannot change its parent shell's directory, so the wrapper runs
//! `gitpoulpi --cwd-file <tmp> ...`, the TUI writes the folder into that file
//! on quit (when asked to: "Quit and cd here", or `ui.tab_dir_follow =
//! "auto"`), and the wrapper `cd`s there afterwards -- the yazi / lf pattern.
//!
//! The scripts are printed, never installed: nothing here touches an rc file.

/// A shell the wrapper is written for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
    /// PowerShell 5.1 and 7+.
    #[value(alias = "powershell")]
    Pwsh,
}

/// The function name used when `--name` is not given.
pub const DEFAULT_NAME: &str = "gp";

/// Whether `name` is safe to paste into a script as a function name.
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && !name.starts_with('-')
}

/// The wrapper script for `shell`, defining a function called `name`.
pub fn script(shell: ShellKind, name: &str) -> String {
    let template = match shell {
        ShellKind::Bash => BASH,
        ShellKind::Zsh => ZSH,
        ShellKind::Fish => FISH,
        ShellKind::Pwsh => PWSH,
    };
    template.replace("@NAME@", name)
}

const BASH: &str = r#"# gitpoulpi shell integration (bash). Add to ~/.bashrc:
#   eval "$(gitpoulpi shell-init bash)"
# `@NAME@` runs gitpoulpi, then cd's to the active tab's folder when you used
# "Quit and cd here" (or ui.tab_dir_follow = "auto"). Pick another name with
#   gitpoulpi shell-init bash --name <name>
if [ "$(type -t @NAME@ 2>/dev/null)" != "function" ] && command -v @NAME@ >/dev/null 2>&1; then
  echo "gitpoulpi shell-init: '@NAME@' already exists in this shell; not overriding it. Use: gitpoulpi shell-init bash --name <other-name>" >&2
else
  function @NAME@ {
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
"#;

const ZSH: &str = r#"# gitpoulpi shell integration (zsh). Add to ~/.zshrc:
#   eval "$(gitpoulpi shell-init zsh)"
# `@NAME@` runs gitpoulpi, then cd's to the active tab's folder when you used
# "Quit and cd here" (or ui.tab_dir_follow = "auto"). Pick another name with
#   gitpoulpi shell-init zsh --name <name>
if (( ! ${+functions[@NAME@]} )) && whence -w @NAME@ >/dev/null 2>&1; then
  print -u2 "gitpoulpi shell-init: '@NAME@' already exists in this shell; not overriding it. Use: gitpoulpi shell-init zsh --name <other-name>"
else
  function @NAME@ {
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
"#;

const FISH: &str = r#"# gitpoulpi shell integration (fish). Add to ~/.config/fish/config.fish:
#   gitpoulpi shell-init fish | source
# `@NAME@` runs gitpoulpi, then cd's to the active tab's folder when you used
# "Quit and cd here" (or ui.tab_dir_follow = "auto"). Pick another name with
#   gitpoulpi shell-init fish --name <name>
if type -q @NAME@; and not functions -q @NAME@
    echo "gitpoulpi shell-init: '@NAME@' already exists in this shell; not overriding it. Use: gitpoulpi shell-init fish --name <other-name>" >&2
else
    function @NAME@
        set -l tmpdir /tmp
        set -q TMPDIR; and set tmpdir $TMPDIR
        set -l tmp (mktemp $tmpdir/gitpoulpi-cwd.XXXXXX); or return
        command gitpoulpi --cwd-file $tmp $argv
        set -l rc $status
        if test -s $tmp
            set -l dir (cat $tmp)
            if test -d "$dir"; and test "$dir" != "$PWD"
                builtin cd -- "$dir"
            end
        end
        rm -f $tmp
        return $rc
    end
end
"#;

const PWSH: &str = r#"# gitpoulpi shell integration (PowerShell). Add to your profile ($PROFILE):
#   gitpoulpi shell-init pwsh | Out-String | Invoke-Expression
# `@NAME@` runs gitpoulpi, then cd's to the active tab's folder when you used
# "Quit and cd here" (or ui.tab_dir_follow = "auto"). Pick another name with
#   gitpoulpi shell-init pwsh --name <name>
# An alias of that name pointing at Get-ItemProperty (PowerShell's built-in
# `gp`) or at gitpoulpi is replaced by the function; anything else is kept.
$gpExisting = Get-Command @NAME@ -ErrorAction SilentlyContinue | Select-Object -First 1
$gpReplace = $false
if ($gpExisting -and $gpExisting.CommandType -eq 'Alias' -and ($gpExisting.Definition -match '^(Get-ItemProperty|gitpoulpi(\.exe)?)$')) {
    Remove-Item Alias:@NAME@ -Force
    $gpReplace = $true
}
if ($gpExisting -and -not $gpReplace -and $gpExisting.CommandType -ne 'Function') {
    Write-Warning "gitpoulpi shell-init: '@NAME@' already exists in this session; not overriding it. Use: gitpoulpi shell-init pwsh --name <other-name>"
} else {
    function global:@NAME@ {
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
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn golden(shell: ShellKind, file: &str) {
        let got = script(shell, DEFAULT_NAME);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/shell_init")
            .join(file);
        if std::env::var_os("GITPOULPI_BLESS").is_some() {
            std::fs::write(&path, &got).unwrap();
        }
        let want = std::fs::read_to_string(&path).unwrap_or_default();
        assert_eq!(
            got.replace("\r\n", "\n"),
            want.replace("\r\n", "\n"),
            "{file} changed: rerun with GITPOULPI_BLESS=1 to accept"
        );
    }

    #[test]
    fn bash_output_is_pinned() {
        golden(ShellKind::Bash, "bash.sh");
    }

    #[test]
    fn zsh_output_is_pinned() {
        golden(ShellKind::Zsh, "zsh.zsh");
    }

    #[test]
    fn fish_output_is_pinned() {
        golden(ShellKind::Fish, "fish.fish");
    }

    #[test]
    fn pwsh_output_is_pinned() {
        golden(ShellKind::Pwsh, "pwsh.ps1");
    }

    #[test]
    fn the_name_is_substituted_everywhere() {
        for shell in [
            ShellKind::Bash,
            ShellKind::Zsh,
            ShellKind::Fish,
            ShellKind::Pwsh,
        ] {
            let text = script(shell, "gpl");
            assert!(!text.contains("@NAME@"));
            assert!(text.contains("gpl"));
        }
    }

    #[test]
    fn names_are_checked() {
        assert!(valid_name("gp") && valid_name("my_gp-2"));
        assert!(!valid_name("") && !valid_name("a b") && !valid_name("a;rm") && !valid_name("-x"));
    }
}
