# gitpoulpi shell integration (fish). Add to ~/.config/fish/config.fish:
#   gitpoulpi shell-init fish | source
# `gp` runs gitpoulpi, then cd's to the active tab's folder when you used
# "Quit and cd here" (or ui.tab_dir_follow = "auto"). Pick another name with
#   gitpoulpi shell-init fish --name <name>
if type -q gp; and not functions -q gp
    echo "gitpoulpi shell-init: 'gp' already exists in this shell; not overriding it. Use: gitpoulpi shell-init fish --name <other-name>" >&2
else
    function gp
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
