#!/usr/bin/env bash
# Theme slice T3: rewrite the legacy role constants (MUTED, ACCENT, ...) to
# their `th().<group>.<role>` accessors across the TUI crate. Idempotent; a
# branch in flight rebases by running this script over its tree.
#   scripts/theme-sweep.sh [dir ...]      (default: crates/gitpoulpi-tui/src)
# Named `use` imports of the constants are dropped (the glob `use crate::*` /
# `use super::*` already brings `th` into scope); comment lines are left alone.
set -euo pipefail
cd "$(dirname "$0")/.."
dirs=("$@"); [ ${#dirs[@]} -gt 0 ] || dirs=(crates/gitpoulpi-tui/src)

find "${dirs[@]}" -name '*.rs' -not -path '*/graph/part.rs' -print0 | xargs -0 perl -0777 -i -pe '
  my %m = (
    MUTED => "th().ui.muted", PRIMARY => "th().ui.text", ACCENT => "th().ui.accent",
    AMBER => "th().status.warning", DANGER => "th().status.danger", AI_COLOR => "th().status.ai",
    SELECTION_BG => "th().ui.selection", HOVER_BG => "th().ui.hover",
    LOCAL_COLOR => "th().refs.local", REMOTE_COLOR => "th().refs.remote", TAG_COLOR => "th().refs.tag",
    BUTTON_BG => "th().ui.button", BUTTON_HOVER_BG => "th().ui.button_hover",
    PANEL_BG => "th().ui.panel", PLAIN_BUTTON_HOVER_BG => "th().ui.button_hover",
    DISABLED => "th().ui.disabled", SELECTED_TEXT_BG => "th().ui.text_selection_bg",
    SELECTED_TEXT_FG => "th().ui.text_selection_fg",
  );
  my $names = join "|", sort { length($b) <=> length($a) } keys %m;
  # 1. drop the names from named use lists
  s#^([ \t]*(?:pub(?:\([a-z]+\))? )?use [^;]*;)#
    my $u = $1;
    if ($u =~ /\{/ && $u !~ /\*/) {
      $u =~ s/(?<![A-Za-z0-9_:])(?:$names)\b\s*,?\s*//g;
      $u =~ s/::\{\s*\}//g;
    } elsif ($u =~ /::(?:$names);/) {
      $u = "__EMPTY__";
    }
    $u
  #gme;
  s/^[ \t]*(?:pub(?:\([a-z]+\))? )?use [^;]*::\{\s*\};\n//gm;
  s/^[ \t]*(?:pub(?:\([a-z]+\))? )?use [^;]*\{\s*\};\n//gm;
  s/^__EMPTY__\n//gm;
  s/^[ \t]*use crate;\n//gm;
  # 2. rewrite uses, line by line, skipping comment lines
  my @l = split /(?<=\n)/, $_;
  for (@l) {
    next if /^\s*\/\//;
    s/(?<![A-Za-z0-9_:])(?:(?:crate|super)::(?:(?:theme|dialog_kit|text_field)::)?|theme::)?\b($names)\b(?![A-Za-z0-9_]*\s*:\s*[A-Za-z(&])/$m{$1}/g unless /^\s*(?:pub(?:\([a-z]+\))? )?(?:const|static) /;
  }
  $_ = join "", @l;
'
