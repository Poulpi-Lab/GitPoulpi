use super::*;

use gitpoulpi_i18n::Lang;

/// An empty filter is the identity: every section is there, and the
/// merge-request vocabulary reaches the titles.
#[test]
fn help_text_follows_the_pr_terminology_setting() {
    crate::terminology::install(gitpoulpi_config::PrTerminology::MergeRequest);
    let text = filter_help_text("");
    let filtered = filter_help_text("merge request");
    crate::terminology::install(gitpoulpi_config::PrTerminology::Auto);
    assert!(text.contains("Merge requests and issues"), "{text}");
    assert!(!text.contains("pull request"), "{text}");
    assert!(filtered.contains("Merge requests and issues"));
}

#[test]
fn empty_filter_keeps_every_section() {
    let km = keymap::Keymap::defaults();
    let text = filter_help_text_with(&km, "");
    let titles = HELP_TEXT
        .iter()
        .filter(|e| matches!(e, HelpEntry::Sec(_)))
        .count();
    let blank_separated = text.split("\n\n").count();
    assert_eq!(titles, blank_separated);
    assert!(text.starts_with("Global\n"));
}

fn km_with(entries: &[(&str, &str, &[&str])]) -> keymap::Keymap {
    keymap::Keymap::from_config(&gitpoulpi_config::KeybindingsConfig::from_entries(
        entries
            .iter()
            .map(|(c, a, v)| (*c, *a, v.iter().map(|s| s.to_string()).collect())),
    ))
}

/// Every message of the table, in `lang`, raw (placeholders unresolved).
fn raw_messages(lang: Lang) -> Vec<&'static str> {
    let mut out = Vec::new();
    for e in HELP_TEXT {
        match *e {
            HelpEntry::Sec(m) | HelpEntry::Sub(m) => out.push(m.get_in(lang)),
            HelpEntry::Item(keys, m) | HelpEntry::Gated(_, keys, m) => {
                match keys {
                    HelpKeys::Lit(s) => out.push(s),
                    HelpKeys::Msg(k) => out.push(k.get_in(lang)),
                }
                out.push(m.get_in(lang));
            }
        }
    }
    out
}

fn placeholders(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = s;
    while let Some(open) = rest.find('{') {
        let close = open + rest[open..].find('}').expect("closed placeholder");
        out.push(rest[open + 1..close].trim_start_matches('*'));
        rest = &rest[close + 1..];
    }
    out
}

/// Every `{ctx.action}` in the help names a registered action, and
/// none survives resolution - in English and in French.
#[test]
fn help_placeholders_all_resolve() {
    let km = keymap::Keymap::defaults();
    for lang in [Lang::En, Lang::Fr] {
        for line in raw_messages(lang) {
            for id in placeholders(line) {
                assert!(
                    keymap::ACTIONS.iter().any(|d| d.id == id),
                    "{lang:?} help names unknown action {id} in {line:?}"
                );
            }
            let out = resolve_keys(&km, line);
            assert!(!out.contains('{') && !out.contains("(unbound)"), "{out}");
        }
    }
}

/// A translation names the same actions as the English text, no more, no
/// fewer: a key the French forgot would hide a binding.
#[test]
fn french_help_names_the_same_actions_as_english() {
    let mut i = 0;
    let en: Vec<&str> = raw_messages(Lang::En);
    let fr: Vec<&str> = raw_messages(Lang::Fr);
    assert_eq!(en.len(), fr.len());
    while i < en.len() {
        let (mut a, mut b) = (placeholders(en[i]), placeholders(fr[i]));
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b, "{:?} vs {:?}", en[i], fr[i]);
        i += 1;
    }
}

/// Every action whose dispatch reads the keymap is documented in the
/// help (its placeholder is in a key column or a description), so a
/// rebinding is visible there.
#[test]
fn every_rebindable_action_is_in_help() {
    let all = raw_messages(Lang::En).join("\n").replace("{*", "{");
    for def in keymap::ACTIONS
        .iter()
        .filter(|d| d.live() && !matches!(d.ctx, keymap::Ctx::Prompt | keymap::Ctx::Palette))
    {
        assert!(
            all.contains(&format!("{{{}}}", def.id)),
            "{} has no help line",
            def.id
        );
    }
}

#[test]
fn help_shows_the_rebound_prefix_and_not_the_old_one() {
    let km = km_with(&[("global", "git_prefix", &["Ctrl+B"])]);
    let text = filter_help_text_with(&km, "");
    assert!(text.contains("Git actions - Ctrl+B, then:"), "{text}");
    assert!(text.contains("  Ctrl+B s        stage the selected unstaged file"));
    assert!(!text.contains("Ctrl+G s") && !text.contains("Ctrl+G, then"));
    // falsified by the defaults
    let text = filter_help_text_with(&keymap::Keymap::defaults(), "");
    assert!(text.contains("  Ctrl+G s        stage the selected unstaged file"));
}

#[test]
fn help_gutter_keeps_two_spaces_after_a_long_label() {
    let km = km_with(&[("global", "palette", &["Ctrl+Alt+Shift+P"])]);
    let text = filter_help_text_with(&km, "command palette");
    assert!(
        text.contains("  Ctrl+Alt+Shift+P  open the command palette"),
        "{text}"
    );
}

/// Layout at the overlay's real widths: no line wider than the overlay, in
/// English and in French, and a continuation line sits under the text.
#[test]
fn help_fits_80_and_120_columns_in_every_shipped_language() {
    let km = keymap::Keymap::defaults();
    for lang in [Lang::En, Lang::Fr] {
        let _g = gitpoulpi_i18n::override_lang(lang);
        for terms in [false, true] {
            let _t = gitpoulpi_i18n::override_git_terms(terms);
            for width in [80usize, 120] {
                let lines = help_lines(&km, "", width);
                assert!(lines.len() > 300, "{lang:?}: {} lines", lines.len());
                for l in &lines {
                    let w = crate::ui_kit::display_width(l);
                    assert!(w <= width, "{lang:?} {width} cols: {w} wide: {l:?}");
                }
            }
        }
    }
}

#[test]
fn french_help_is_french() {
    let _g = gitpoulpi_i18n::override_lang(Lang::Fr);
    let text = filter_help_text("");
    assert!(text.contains("Général") && text.contains("ouvre la palette de commandes"));
    assert!(!text.contains("open the command palette"));
}

/// The filter matches the translated text and the English one, and both
/// find the same paragraph.
#[test]
fn filter_matches_french_and_english_alike() {
    let km = keymap::Keymap::defaults();
    let _g = gitpoulpi_i18n::override_lang(Lang::Fr);
    let fr = help_lines(&km, "masque la minimap", 100);
    let en = help_lines(&km, "toggle the minimap", 100);
    assert!(!fr.is_empty());
    assert_eq!(fr, en);
    assert!(fr[0].starts_with("Vue Diff"), "{:?}", fr[0]);
    // A key name matches too, whatever the language.
    assert_eq!(
        help_lines(&km, "Ctrl+G s", 100),
        help_lines(&km, "ctrl+g S", 100)
    );
}

#[test]
fn wrap_breaks_at_words_and_cuts_overlong_ones() {
    assert_eq!(wrap("aaa bbb ccc", 7, 7), vec!["aaa bbb", "ccc"]);
    assert_eq!(wrap("aaa bbb", 3, 10), vec!["aaa", "bbb"]);
    assert_eq!(wrap("abcdefghij", 4, 4), vec!["abcd", "efgh", "ij"]);
    assert_eq!(wrap("", 5, 5), vec![""]);
    // a no-break space does not split
    assert_eq!(wrap("a\u{a0}b c", 3, 3), vec!["a\u{a0}b", "c"]);
}

#[test]
fn prefix_hint_lists_ai_keys_only_when_on() {
    let km = keymap::Keymap::defaults();
    let off = prefix_hint(&km, keymap::Ctx::GitPrefix, false);
    let on = prefix_hint(&km, keymap::Ctx::GitPrefix, true);
    assert!(off.starts_with("Ctrl+G: s=stage u=unstage"), "{off}");
    assert!(!off.contains("AI-message") && on.contains("i=AI-message w=AI-explain"));
    let nav = prefix_hint(&km, keymap::Ctx::NavPrefix, false);
    assert!(nav.starts_with("Ctrl+N: h=collapse Refs  l=collapse Status  <badge>"));
    assert!(nav.ends_with("t=select tab  (Esc to cancel)"), "{nav}");
}

#[test]
fn prefix_hint_follows_the_keymap_and_drops_unbound_keys() {
    let km = km_with(&[
        ("global", "git_prefix", &["Ctrl+B"]),
        ("git_prefix", "stage_file", &["z"]),
        ("git_prefix", "pull", &[]),
    ]);
    let hint = prefix_hint(&km, keymap::Ctx::GitPrefix, false);
    assert!(hint.starts_with("Ctrl+B: z=stage u=unstage"), "{hint}");
    assert!(!hint.contains("pull") && !hint.contains("Ctrl+G"), "{hint}");
    let km = km_with(&[("global", "nav_prefix", &["Alt+N"])]);
    assert!(prefix_hint(&km, keymap::Ctx::NavPrefix, false).starts_with("Alt+N: h="));
}

#[test]
fn an_unbound_action_prints_no_empty_parentheses() {
    let km = km_with(&[("git_prefix", "fetch_all", &[])]);
    assert_eq!(keymap::paren(&km, keymap::Action::GitFetchAll), "");
    assert_eq!(
        keymap::paren(&keymap::Keymap::defaults(), keymap::Action::GitFetchAll),
        " (Ctrl+G f)"
    );
}

/// A filter keeps a whole paragraph - heading and body - when
/// *any* of its lines match, case-insensitively, and drops every
/// paragraph that doesn't. "hunk" only appears inside the Diff view
/// paragraph's body lines, never its own "Diff view" heading, so
/// this also proves the heading rides along rather than needing the
/// match itself.
#[test]
fn filter_keeps_whole_matching_paragraphs_and_drops_the_rest() {
    let out = filter_help_text("hunk");
    assert!(
        out.contains("Diff view"),
        "the heading must ride along with its matching body line"
    );
    assert!(out.to_lowercase().contains("hunk"));
    assert!(
        !out.contains("Repositories and tabs"),
        "a paragraph with no match anywhere in it must be dropped entirely"
    );
}

/// Case-insensitive, both directions: upper-case input finds a
/// lower-case line and vice versa.
#[test]
fn filter_is_case_insensitive() {
    assert_eq!(filter_help_text("SETTINGS"), filter_help_text("settings"));
    assert!(!filter_help_text("SETTINGS").is_empty());
}

/// A filter matching nothing returns an empty string, not the whole
/// text nor a panic on the join of zero paragraphs.
#[test]
fn filter_matching_nothing_returns_empty() {
    assert_eq!(filter_help_text("xyzzy-not-a-real-keybinding"), "");
}
