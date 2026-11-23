use super::chord::{is_ctrl_enter_class, Key};
use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::HashSet;

fn ev(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, mods)
}

#[test]
fn every_action_once_and_ids_unique() {
    let mut ids = HashSet::new();
    let mut actions = HashSet::new();
    for def in ACTIONS {
        assert!(ids.insert(def.id), "duplicate id {}", def.id);
        assert!(
            actions.insert(def.action),
            "duplicate action {:?}",
            def.action
        );
        let (ctx, name) = def.id.split_once('.').expect("id is <ctx>.<name>");
        assert_eq!(ctx, def.ctx.name(), "{} sits in the wrong ctx", def.id);
        assert!(
            name.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
            "{} is not snake_case",
            def.id
        );
    }
}

#[test]
fn every_default_parses() {
    // Palette commands ship unbound.
    for def in ACTIONS.iter().filter(|d| d.ctx != Ctx::Palette) {
        assert!(!def.defaults.is_empty(), "{} has no default", def.id);
        for text in def.defaults {
            KeyChord::parse(text).unwrap_or_else(|e| panic!("{} default {text:?}: {e}", def.id));
        }
    }
}

#[test]
fn defaults_have_no_same_context_duplicates() {
    let mut seen = HashSet::new();
    for def in ACTIONS {
        for text in def.defaults {
            let chord = KeyChord::parse(text).unwrap().class();
            assert!(
                seen.insert((def.ctx, chord)),
                "{} reuses {text} within {}",
                def.id,
                def.ctx.name()
            );
        }
    }
}

#[test]
fn chord_parse_print_round_trip() {
    for def in ACTIONS {
        for text in def.defaults {
            let chord = KeyChord::parse(text).unwrap();
            assert_eq!(KeyChord::parse(&chord.to_string()), Ok(chord), "{text}");
        }
    }
    for (text, shown) in [
        ("ctrl+o", "Ctrl+O"),
        ("Ctrl+O", "Ctrl+O"),
        ("X", "Shift+X"),
        ("Shift+x", "Shift+X"),
        ("shift+X", "Shift+X"),
        ("Ctrl+Shift+n", "Ctrl+Shift+N"),
        ("Ctrl++", "Ctrl++"),
        ("+", "+"),
        ("ctrl+pgdn", "Ctrl+PageDown"),
        ("f2", "F2"),
        ("Alt+Enter", "Alt+Enter"),
    ] {
        assert_eq!(KeyChord::parse(text).unwrap().to_string(), shown, "{text}");
    }
    assert_eq!(KeyChord::parse("Shift+x"), KeyChord::parse("X"));
    assert_eq!(KeyChord::parse("Ctrl+o"), KeyChord::parse("Ctrl+O"));
    assert!(KeyChord::parse("").is_err());
    assert!(KeyChord::parse("Ctrl+Nope").is_err());
    assert!(KeyChord::parse("Ctrl+Alt+@").is_err(), "AltGr");
}

#[test]
fn from_event_normalises() {
    let upper = KeyChord::from_event(&ev(KeyCode::Char('X'), KeyModifiers::NONE));
    let upper_shift = KeyChord::from_event(&ev(KeyCode::Char('X'), KeyModifiers::SHIFT));
    assert_eq!(upper, KeyChord::parse("Shift+x").ok());
    assert_eq!(upper, upper_shift);
    let raw = KeyChord::from_event(&ev(KeyCode::Char('\u{7}'), KeyModifiers::NONE));
    assert_eq!(raw, KeyChord::parse("Ctrl+G").ok());
    let newline = KeyChord::from_event(&ev(KeyCode::Char('\n'), KeyModifiers::NONE));
    assert_eq!(newline, KeyChord::parse("Ctrl+Enter").ok());
    let ctrl_j = KeyChord::from_event(&ev(KeyCode::Char('j'), KeyModifiers::CONTROL));
    assert_eq!(ctrl_j, newline);
    // A symbol's Shift bit is its glyph, not a modifier.
    let question = KeyChord::from_event(&ev(KeyCode::Char('?'), KeyModifiers::SHIFT));
    assert_eq!(question, KeyChord::parse("?").ok());
    let release = KeyEvent::new_with_kind(
        KeyCode::Char('a'),
        KeyModifiers::NONE,
        crossterm::event::KeyEventKind::Release,
    );
    assert_eq!(KeyChord::from_event(&release), None);
}

#[test]
fn spelling_classes() {
    for (a, b) in [
        ("Ctrl+J", "Ctrl+Enter"),
        ("Ctrl+@", "Ctrl+Space"),
        ("Ctrl+_", "Ctrl+/"),
        ("Ctrl+7", "Ctrl+/"),
    ] {
        assert_eq!(
            KeyChord::parse(a).unwrap().class(),
            KeyChord::parse(b).unwrap().class(),
            "{a} vs {b}"
        );
    }
}

#[test]
fn ctrl_enter_class_matches_legacy_commit_chord() {
    use KeyModifiers as M;
    assert!(is_ctrl_enter_class(&ev(KeyCode::Enter, M::CONTROL)));
    assert!(is_ctrl_enter_class(&ev(KeyCode::Char('j'), M::CONTROL)));
    assert!(is_ctrl_enter_class(&ev(KeyCode::Char('\n'), M::NONE)));
    assert!(!is_ctrl_enter_class(&ev(
        KeyCode::Char('j'),
        M::CONTROL | M::ALT
    )));
    assert!(!is_ctrl_enter_class(&ev(KeyCode::Enter, M::NONE)));
}

#[test]
fn window_chords_resolve_like_the_old_guards() {
    use KeyModifiers as M;
    let km = Keymap::defaults();
    assert!(km.is(Action::GlobalHome, &ev(KeyCode::Char('o'), M::CONTROL)));
    assert!(km.is(Action::GlobalHome, &ev(KeyCode::Char('O'), M::CONTROL)));
    assert!(km.is(
        Action::GlobalHome,
        &ev(KeyCode::Char('O'), M::CONTROL | M::SHIFT)
    ));
    assert!(!km.is(Action::GlobalHome, &ev(KeyCode::Char('o'), M::NONE)));
    assert!(km.is(
        Action::GlobalClone,
        &ev(KeyCode::Char('N'), M::CONTROL | M::SHIFT)
    ));
    assert!(!km.is(Action::GlobalClone, &ev(KeyCode::Char('n'), M::CONTROL)));
    assert!(km.is(Action::GlobalNavPrefix, &ev(KeyCode::Char('n'), M::CONTROL)));
    assert!(km.is(Action::GlobalSettings, &ev(KeyCode::Char(','), M::CONTROL)));
    assert!(!km.is(Action::GlobalSettings, &ev(KeyCode::Char(','), M::NONE)));
    assert!(km.is(Action::GlobalNextTab, &ev(KeyCode::PageDown, M::CONTROL)));
    assert!(km.is(Action::GlobalPrevTab, &ev(KeyCode::PageUp, M::CONTROL)));
    assert!(!km.is(Action::GlobalNextTab, &ev(KeyCode::PageDown, M::NONE)));
    // The Shift twin wins by exactness when both would match.
    assert_eq!(
        km.action_in(Ctx::Global, &ev(KeyCode::Char('N'), M::CONTROL | M::SHIFT)),
        Some(Action::GlobalClone)
    );
    assert_eq!(
        km.action_in(Ctx::Global, &ev(KeyCode::Char('n'), M::CONTROL)),
        Some(Action::GlobalNavPrefix)
    );
}

#[test]
fn prefix_tables_look_at_the_code_alone() {
    use KeyModifiers as M;
    let km = Keymap::defaults();
    let git = |c, m| km.action_in(Ctx::GitPrefix, &ev(KeyCode::Char(c), m));
    assert_eq!(git('s', M::NONE), Some(Action::GitStageFile));
    // An unbound Ctrl+<key> falls back to the bare key.
    assert_eq!(git('s', M::CONTROL), Some(Action::GitStageFile));
    assert_eq!(git('P', M::SHIFT), Some(Action::GitPush));
    assert_eq!(git('P', M::NONE), Some(Action::GitPush));
    assert_eq!(git('p', M::NONE), Some(Action::GitPull));
    assert_eq!(git('z', M::NONE), None);
    assert_eq!(
        km.action_in_code(Ctx::NavPrefix, KeyCode::Char('=')),
        Some(Action::NavGrowSection)
    );
    assert_eq!(km.action_in_code(Ctx::NavPrefix, KeyCode::Char('5')), None);
}

#[test]
fn every_git_prefix_default_resolves_to_its_own_action() {
    let km = Keymap::defaults();
    for def in ACTIONS.iter().filter(|d| d.ctx.ignores_modifiers()) {
        for text in def.defaults {
            let chord = KeyChord::parse(text).unwrap();
            let code = match chord.key {
                Key::Char(c) if chord.mods.shift => KeyCode::Char(c.to_ascii_uppercase()),
                Key::Char(c) => KeyCode::Char(c),
                Key::Enter => KeyCode::Enter,
                Key::Backspace => KeyCode::Backspace,
                Key::Left => KeyCode::Left,
                Key::Right => KeyCode::Right,
                Key::Tab => KeyCode::Tab,
                other => panic!("unexpected {other:?}"),
            };
            let mods = if chord.mods.shift {
                crossterm::event::KeyModifiers::SHIFT
            } else if chord.mods.ctrl {
                // A Ctrl chord after a prefix (`Ctrl+G Ctrl+P`) matches exactly.
                crossterm::event::KeyModifiers::CONTROL
            } else {
                crossterm::event::KeyModifiers::NONE
            };
            assert_eq!(
                km.action_in_code_mods(def.ctx, code, mods),
                Some(def.action),
                "{}",
                def.id
            );
        }
    }
}

#[test]
fn labels_carry_the_effective_prefix() {
    let km = Keymap::defaults();
    assert_eq!(km.label(Action::GitStageFile), "Ctrl+G s");
    assert_eq!(km.label(Action::GitPush), "Ctrl+G Shift+P");
    assert_eq!(km.label(Action::NavCollapseLeft), "Ctrl+N Shift+H");
    assert_eq!(km.labels(Action::NavGrowSection), ["Ctrl+N +", "Ctrl+N ="]);
    assert_eq!(km.label(Action::GlobalSettings), "Ctrl+,");
    assert_eq!(km.label(Action::PromptAiStashMessage), "Ctrl+G");
}

#[test]
fn installed_keymap_is_the_defaults() {
    let km = current();
    assert!(km.is(
        Action::GlobalPalette,
        &ev(KeyCode::Char('p'), KeyModifiers::CONTROL)
    ));
}

// ---- config, conflicts, reach ----

use super::chord::Reach;
#[cfg(feature = "ai")]
use super::conflict::detect_with;
use super::conflict::{detect, Level, Platform};
use gitpoulpi_config::KeybindingsConfig;

const LINUX: Platform = Platform {
    windows: false,
    enhanced: false,
};

fn cfg(entries: &[(&str, &str, &[&str])]) -> KeybindingsConfig {
    KeybindingsConfig::from_entries(
        entries
            .iter()
            .map(|(c, a, v)| (*c, *a, v.iter().map(|s| s.to_string()).collect())),
    )
}

fn press(c: char, m: KeyModifiers) -> KeyEvent {
    ev(KeyCode::Char(c), m)
}

#[test]
fn an_override_replaces_the_defaults() {
    let km = Keymap::from_config(&cfg(&[
        ("global", "git_prefix", &["Ctrl+B"]),
        ("git_prefix", "stage_file", &["z", "Shift+Z"]),
    ]));
    assert!(km.issues().is_empty(), "{:?}", km.issues());
    assert!(km.is(Action::GlobalGitPrefix, &press('b', KeyModifiers::CONTROL)));
    assert!(!km.is(Action::GlobalGitPrefix, &press('g', KeyModifiers::CONTROL)));
    assert_eq!(km.label(Action::GitStageFile), "Ctrl+B z");
    assert_eq!(
        km.labels(Action::GitStageFile),
        ["Ctrl+B z", "Ctrl+B Shift+Z"]
    );
    assert_eq!(km.action_in_code(Ctx::GitPrefix, KeyCode::Char('s')), None);
    assert_eq!(
        km.action_in_code(Ctx::GitPrefix, KeyCode::Char('Z')),
        Some(Action::GitStageFile)
    );
}

#[test]
fn a_clash_in_one_context_reverts_the_later_id() {
    let km = Keymap::from_config(&cfg(&[("git_prefix", "stage_file", &["a"])]));
    assert_eq!(km.issues().len(), 1, "{:?}", km.issues());
    assert_eq!(km.label(Action::GitStageFile), "Ctrl+G s");
    assert_eq!(km.label(Action::GitStageAll), "Ctrl+G a");
}

#[test]
fn bad_entries_give_issues_and_keep_defaults() {
    let km = Keymap::from_config(&cfg(&[
        ("global", "git_prefix", &["Ctrl+Nope"]),
        ("global", "no_such", &["F5"]),
        ("nowhere", "x", &["F5"]),
        ("global", "quit", &["Enter"]),
        ("git_prefix", "pull", &["Alt+p"]),
    ]));
    assert_eq!(km.issues().len(), 5, "{:?}", km.issues());
    assert_eq!(km.label(Action::GlobalGitPrefix), "Ctrl+G");
    assert_eq!(km.label(Action::GitPull), "Ctrl+G p");
    assert!(km
        .issues()
        .iter()
        .any(|i| i.message.contains("unknown context")));
    assert!(km
        .issues()
        .iter()
        .any(|i| i.message.contains("navigation keys are fixed")));
    let msg = issues_message(km.issues()).unwrap();
    assert!(msg.starts_with("keybindings: 5 entries ignored"), "{msg}");
    let one = Keymap::from_config(&cfg(&[("global", "git_prefix", &["Ctrl+Nope"])]));
    assert!(issues_message(one.issues())
        .unwrap()
        .starts_with("keybindings: 1 entry ignored"));
}

#[test]
fn unbound_means_no_chord() {
    let km = Keymap::from_config(&cfg(&[("global", "tab_menu", &[])]));
    assert!(km.issues().is_empty());
    assert_eq!(km.label(Action::GlobalTabMenu), "");
    assert!(!km.is(Action::GlobalTabMenu, &press('t', KeyModifiers::CONTROL)));
}

#[test]
fn defaults_have_no_conflicts() {
    let km = Keymap::defaults();
    for platform in [
        LINUX,
        Platform {
            windows: true,
            enhanced: false,
        },
    ] {
        let errors: Vec<_> = detect(&km, platform)
            .into_iter()
            .filter(|c| c.level == Level::Error)
            .collect();
        assert!(errors.is_empty(), "{errors:?}");
    }
}

#[test]
fn a_prefix_clash_with_a_global_chord_is_an_error_and_reverts() {
    let km = Keymap::from_config(&cfg(&[("global", "git_prefix", &["Ctrl+P"])]));
    assert_eq!(km.issues().len(), 1);
    assert_eq!(km.label(Action::GlobalGitPrefix), "Ctrl+G");
    assert_eq!(km.label(Action::GlobalPalette), "Ctrl+P");
    // the two prefixes must differ
    let km = Keymap::from_config(&cfg(&[("global", "nav_prefix", &["Ctrl+G"])]));
    assert_eq!(km.issues().len(), 1);
    assert_eq!(km.label(Action::GlobalNavPrefix), "Ctrl+N");
}

#[test]
fn a_prefix_on_a_typing_key_is_an_error() {
    let km = Keymap::from_config(&cfg(&[("global", "git_prefix", &["x"])]));
    assert_eq!(km.issues().len(), 1);
    assert!(
        km.issues()[0].message.contains("typing key"),
        "{:?}",
        km.issues()
    );
    assert_eq!(km.label(Action::GlobalGitPrefix), "Ctrl+G");
}

#[test]
fn ctrl_j_and_ctrl_enter_conflict_through_the_spelling_class() {
    let km = Keymap::from_config(&cfg(&[
        ("global", "home", &["Ctrl+J"]),
        ("global", "tab_menu", &["Ctrl+Enter"]),
    ]));
    assert_eq!(km.issues().len(), 1, "{:?}", km.issues());
}

#[test]
fn prefix_second_key_cannot_carry_alt() {
    let km = Keymap::from_config(&cfg(&[("git_prefix", "pull", &["Alt+X"])]));
    assert_eq!(km.issues().len(), 1);
}

// The one action that shadows is the AI stash message (`Ctrl+G` in the prompt).
#[cfg(feature = "ai")]
#[test]
fn shadowing_a_parent_only_warns() {
    // Prompt is nobody's child today; pretend it is a child of Global.
    fn prompt_under_global(ctx: Ctx) -> Option<Ctx> {
        (ctx == Ctx::Prompt).then_some(Ctx::Global)
    }
    let found = detect_with(&Keymap::defaults(), LINUX, prompt_under_global);
    let shadow = found
        .iter()
        .find(|c| c.action == Action::PromptAiStashMessage && c.other.is_some())
        .expect("Ctrl+G in the prompt hides the git prefix");
    assert_eq!(shadow.level, Level::Warning);
    assert!(found.iter().all(|c| c.level != Level::Error));
}

#[test]
fn text_field_chords_warn() {
    let km = Keymap::from_config(&cfg(&[("global", "tab_menu", &["Ctrl+K"])]));
    let c = km.conflicts();
    assert!(c
        .iter()
        .any(|c| c.action == Action::GlobalTabMenu && c.message.contains("text-editing")));
}

#[test]
fn tmux_and_screen_prefixes_are_flagged_as_commonly_taken() {
    let km = Keymap::from_config(&cfg(&[
        ("global", "git_prefix", &["Ctrl+B"]),
        ("global", "nav_prefix", &["Ctrl+A"]),
    ]));
    assert!(km.issues().is_empty());
    let c = km.conflicts();
    let info = |a| {
        c.iter()
            .find(|c| c.action == a && c.level == Level::Info)
            .map(|c| c.message.clone())
            .unwrap_or_default()
    };
    assert!(info(Action::GlobalGitPrefix).contains("commonly taken by tmux"));
    assert!(info(Action::GlobalNavPrefix).contains("commonly taken by GNU screen"));
}

#[test]
fn reach_table() {
    let p = |t: &str| KeyChord::parse(t).unwrap();
    assert_eq!(p("Ctrl+,").reach(true, false), Reach::Ok);
    assert_eq!(p("Ctrl+,").reach(false, false), Reach::Unsendable);
    assert_eq!(p("Ctrl+,").reach(false, true), Reach::Ok);
    assert_eq!(p("Ctrl+5").reach(false, false), Reach::Unsendable);
    assert_eq!(
        p("Ctrl+Shift+N").reach(false, false),
        Reach::NotDistinct(p("Ctrl+N"))
    );
    assert_eq!(p("Ctrl+Shift+N").reach(true, false), Reach::Ok);
    assert_eq!(
        p("Ctrl+I").reach(false, false),
        Reach::NotDistinct(p("Tab"))
    );
    assert_eq!(
        p("Ctrl+M").reach(false, false),
        Reach::NotDistinct(p("Enter"))
    );
    assert_eq!(p("Ctrl+G").reach(false, false), Reach::Ok);
    assert_eq!(p("Ctrl+/").reach(false, false), Reach::Ok);
    assert_eq!(p("Alt+n").reach(false, false), Reach::Ok);
    assert_eq!(p("Ctrl+PageDown").reach(false, false), Reach::Ok);
    assert_eq!(p("F2").reach(false, false), Reach::Ok);
    let km = Keymap::defaults();
    assert!(detect(&km, LINUX)
        .iter()
        .any(|c| c.action == Action::GlobalSettings && c.message.contains("does not send")));
    assert!(!detect(
        &km,
        Platform {
            windows: true,
            enhanced: false
        }
    )
    .iter()
    .any(|c| c.message.contains("this terminal")));
}

#[test]
fn badges_never_equal_a_nav_prefix_key() {
    use crate::chrome::navigation_badge_char;
    let km = Keymap::from_config(&cfg(&[("nav_prefix", "collapse_refs", &["b"])]));
    assert!(km.issues().is_empty());
    let taken = km.nav_prefix_chars();
    assert!(taken.contains(&'b') && !taken.contains(&'H'));
    // The default keymap stays the installed one in unit tests, so check
    // the invariant on it: no badge is a default nav key.
    let defaults = Keymap::defaults().nav_prefix_chars();
    for i in 0..60 {
        let c = navigation_badge_char(i);
        assert!(!defaults.contains(&c) && c != 'q', "badge {c} at {i}");
    }
}

#[test]
fn defaults_shadow_nothing_but_the_whitelist() {
    let km = Keymap::defaults();
    let shadows: Vec<_> = detect(&km, LINUX)
        .into_iter()
        .filter(|c| c.level == Level::Warning && c.message.contains("hides"))
        .collect();
    assert!(shadows.is_empty(), "{shadows:?}");
}

#[test]
fn a_pane_chord_equal_to_a_global_one_warns_and_still_applies() {
    let km = Keymap::from_config(&cfg(&[("status", "compose", &["r"])]));
    assert!(km.issues().is_empty(), "{:?}", km.issues());
    assert_eq!(
        km.action_in(Ctx::Status, &press('r', KeyModifiers::NONE)),
        Some(Action::StatusCompose)
    );
    let warn = km.conflicts().into_iter().find(|c| {
        c.level == Level::Warning
            && c.action == Action::StatusCompose
            && c.message.contains("hides")
    });
    assert!(warn.is_some(), "{:?}", km.conflicts());
}

#[test]
fn pane_bindings_follow_the_config() {
    let km = Keymap::from_config(&cfg(&[
        ("status", "stage_toggle", &["g"]),
        ("log", "toggle_file_mode", &["Shift+M"]),
        ("home", "favorite", &["s"]),
        ("global", "find_in_log", &["Shift+F"]),
    ]));
    assert!(km.issues().is_empty(), "{:?}", km.issues());
    let plain = |c| press(c, KeyModifiers::NONE);
    assert_eq!(
        km.action_in(Ctx::Status, &plain('g')),
        Some(Action::StatusStageToggle)
    );
    assert_eq!(km.action_in(Ctx::Status, &plain(' ')), None);
    assert!(km.is(Action::LogToggleFileMode, &press('M', KeyModifiers::SHIFT)));
    assert!(!km.is(Action::LogToggleFileMode, &plain('m')));
    assert_eq!(
        km.action_in(Ctx::Home, &plain('s')),
        Some(Action::HomeFavorite)
    );
    assert!(km.is(Action::GlobalFindInLog, &press('F', KeyModifiers::SHIFT)));
    assert!(!km.is(Action::GlobalFindInLog, &plain('f')));
}

// ---- views, editor and popup contexts ----

#[test]
fn view_and_popup_bindings_follow_the_config() {
    let km = test_keymap(&[
        ("diff", "stage_hunk", &["a"]),
        ("diff", "close", &[]),
        ("file_insight", "copy", &["Ctrl+Y"]),
        ("editor", "save", &["Alt+s"]),
        ("rebase", "reset", &["Alt+r"]),
        ("dialog", "browse", &["Alt+b"]),
        ("refs", "hosting_filter", &["Shift+V"]),
    ]);
    let plain = |c| press(c, KeyModifiers::NONE);
    assert!(km.is(Action::DiffStageHunk, &plain('a')));
    assert!(!km.is(Action::DiffStageHunk, &plain(' ')));
    assert!(!km.is(Action::DiffClose, &plain('q')), "unbound");
    assert!(km.is(Action::FileInsightCopy, &press('y', KeyModifiers::CONTROL)));
    assert!(!km.is(Action::FileInsightCopy, &plain('y')));
    assert_eq!(
        km.action_in(Ctx::Editor, &press('s', KeyModifiers::ALT)),
        Some(Action::EditorSave)
    );
    assert_eq!(
        km.action_in(Ctx::Editor, &press('s', KeyModifiers::CONTROL)),
        None
    );
    assert!(km.is(Action::RebaseReset, &press('r', KeyModifiers::ALT)));
    assert!(km.is(Action::DialogBrowse, &press('b', KeyModifiers::ALT)));
    assert!(km.is(Action::RefsHostingFilter, &press('V', KeyModifiers::SHIFT)));
}

#[test]
fn diff_and_editor_defaults_resolve_like_the_old_guards() {
    let km = Keymap::defaults();
    let plain = |c| press(c, KeyModifiers::NONE);
    for (c, a) in [
        ('r', Action::DiffLfsRaw),
        ('e', Action::DiffEdit),
        ('q', Action::DiffClose),
        ('y', Action::DiffCopy),
        (' ', Action::DiffStageHunk),
        ('x', Action::DiffDiscardHunk),
        ('v', Action::DiffSelectLines),
        ('n', Action::DiffNextHunk),
        ('p', Action::DiffPrevHunk),
        ('m', Action::DiffToggleLayout),
        ('w', Action::DiffToggleWrap),
        ('s', Action::DiffToggleWhitespace),
    ] {
        assert_eq!(km.action_in(Ctx::Diff, &plain(c)), Some(a), "{c}");
    }
    assert!(km.is(Action::DiffDiscardLines, &press('X', KeyModifiers::SHIFT)));
    assert!(km.is(Action::DiffToggleMinimap, &press('M', KeyModifiers::SHIFT)));
    assert!(km.is(
        Action::DiffStageLines,
        &ev(KeyCode::Enter, KeyModifiers::NONE)
    ));
    assert!(km.is(Action::DiffCopy, &press('c', KeyModifiers::CONTROL)));
    // Redo's two chords, and undo not stealing Ctrl+Shift+Z.
    let z = |m| press('z', m);
    assert_eq!(
        km.action_in(Ctx::Editor, &z(KeyModifiers::CONTROL | KeyModifiers::SHIFT)),
        Some(Action::EditorRedo)
    );
    assert_eq!(
        km.action_in(Ctx::Editor, &z(KeyModifiers::CONTROL)),
        Some(Action::EditorUndo)
    );
    // The C0 spelling of Ctrl+S.
    assert_eq!(
        km.action_in(Ctx::Editor, &plain('\u{13}')),
        Some(Action::EditorSave)
    );
}

#[test]
fn fixed_keys_stay_fixed_in_the_new_contexts() {
    // Enter and Space are shortcuts in the diff, never in a popup.
    let enter = KeyChord::parse("Enter").unwrap();
    assert!(conflict::fixed_reason(Ctx::Diff, enter).is_none());
    assert!(conflict::fixed_reason(Ctx::FileInsight, enter).is_none());
    assert!(conflict::fixed_reason(Ctx::Refs, enter).is_some());
    for (ctx, name, chord, ok) in [
        ("diff", "close", "j", false),
        ("diff", "close", "5", false),
        ("file_insight", "copy", "Esc", false),
        ("editor", "save", "a", false),
        ("editor", "save", "Ctrl+A", false),
        ("editor", "save", "Ctrl+K", false),
        ("editor", "save", "Alt+s", true),
        ("rebase", "reset", "Enter", false),
        ("rebase", "reset", "Space", false),
        ("rebase", "reset", "z", true),
        ("dialog", "browse", "b", false),
        ("compose", "apply", "x", false),
    ] {
        let km = Keymap::from_config(&cfg(&[(ctx, name, &[chord])]));
        assert_eq!(
            km.issues().is_empty(),
            ok,
            "{ctx}.{name} = {chord}: {:?}",
            km.issues()
        );
    }
}

#[test]
fn a_view_chord_equal_to_a_global_one_warns_and_applies() {
    let km = Keymap::from_config(&cfg(&[("diff", "toggle_wrap", &["Ctrl+O"])]));
    assert!(km.issues().is_empty(), "{:?}", km.issues());
    assert!(km.conflicts().into_iter().any(|c| c.level == Level::Warning
        && c.action == Action::DiffToggleWrap
        && c.message.contains("hides")));
    assert!(km.is(Action::DiffToggleWrap, &press('o', KeyModifiers::CONTROL)));
}

#[test]
fn popup_contexts_are_independent_tables() {
    // The same chord in two popups is not a conflict.
    let km = Keymap::from_config(&cfg(&[
        ("folder_picker", "new_folder", &["Alt+n"]),
        ("rebase", "reset", &["Alt+n"]),
    ]));
    assert!(km.issues().is_empty(), "{:?}", km.issues());
    // Two actions of one popup are.
    let km = Keymap::from_config(&cfg(&[("rebase", "reset", &["Ctrl+S"])]));
    assert_eq!(km.issues().len(), 1);
}

#[test]
fn the_shift_f10_default_does_not_match_a_plain_f10() {
    let km = Keymap::defaults();
    let f10 = |m| KeyEvent::new(KeyCode::F(10), m);
    assert!(km.is(Action::GlobalContextMenu, &f10(KeyModifiers::SHIFT)));
    assert!(!km.is(Action::GlobalContextMenu, &f10(KeyModifiers::NONE)));
    assert!(km.is(Action::GlobalContextMenu, &press('.', KeyModifiers::NONE)));
}

#[test]
fn fixed_keys_and_digits_cannot_be_bound_in_a_pane() {
    for chord in ["Enter", "Down", "j", "5", "Esc"] {
        let km = Keymap::from_config(&cfg(&[("status", "compose", &[chord])]));
        assert_eq!(km.issues().len(), 1, "{chord}: {:?}", km.issues());
    }
}

#[test]
fn an_action_without_a_feature_tag_is_built_and_a_tag_follows_the_build() {
    use crate::build_features::{offered_with, Feature};
    assert!(ACTIONS.iter().all(|d| d.built() || d.requires.is_some()));
    // The registry's filter, against a build with nothing optional.
    let none = |_: Feature| false;
    assert!(ACTIONS
        .iter()
        .filter(|d| d.requires.is_none())
        .all(|d| offered_with(d.requires, none)));
    assert!(!offered_with(Some(Feature::Ai), none));
}

/// The Radar's `Alt+R` is also `merge.reset_all` and
/// `composer.insert_issue_ref`: those are non-pane contexts that win in
/// their own view, so the defaults report no clash (like `Alt+L` and
/// `rewrite.load` before it).
#[test]
fn the_radar_key_shared_with_view_contexts_is_not_a_conflict() {
    let km = Keymap::defaults();
    let alt_r = press('r', KeyModifiers::ALT);
    assert!(km.is(Action::GlobalRadar, &alt_r));
    assert!(km.is(Action::MergeResetAll, &alt_r));
    assert!(km.is(Action::ComposerInsertIssueRef, &alt_r));
    for platform in [
        LINUX,
        Platform {
            windows: true,
            enhanced: false,
        },
    ] {
        let found: Vec<_> = detect(&km, platform)
            .into_iter()
            .filter(|c| c.action == Action::GlobalRadar || c.other == Some(Action::GlobalRadar))
            .collect();
        assert!(found.is_empty(), "{found:?}");
    }
}

/// Ctrl+Tab / Ctrl+Shift+Tab resolve to the tab actions for every spelling
/// a terminal's decoder can produce: kitty `CSI 9;5u` (Tab+Ctrl), `CSI 9;6u`
/// (BackTab+Ctrl+Shift in crossterm), a Windows console (BackTab+Ctrl) and
/// Tab+Ctrl+Shift.
#[test]
fn ctrl_tab_spellings_resolve_to_the_tab_actions() {
    use KeyModifiers as M;
    let km = Keymap::defaults();
    let ctrl = M::CONTROL;
    let both = M::CONTROL | M::SHIFT;
    for (code, mods) in [
        (KeyCode::BackTab, both),
        (KeyCode::BackTab, ctrl),
        (KeyCode::Tab, both),
    ] {
        assert_eq!(
            km.action_in(Ctx::Global, &ev(code, mods)),
            Some(Action::GlobalPrevTab),
            "{code:?} {mods:?}"
        );
    }
    assert_eq!(
        km.action_in(Ctx::Global, &ev(KeyCode::Tab, ctrl)),
        Some(Action::GlobalNextTab)
    );
    assert_eq!(km.action_in(Ctx::Global, &ev(KeyCode::Tab, M::NONE)), None);
    assert_eq!(
        km.action_in(Ctx::Global, &ev(KeyCode::BackTab, M::SHIFT)),
        None
    );
}

#[test]
fn ctrl_tab_pair_is_attributed_to_terminal_tab_switching() {
    use super::chord::{commonly_taken_by, KeyChord};
    for text in ["Ctrl+Tab", "Ctrl+Shift+Tab"] {
        let chord = KeyChord::parse(text).unwrap();
        let tool = commonly_taken_by(chord).unwrap().get();
        for name in ["kitty", "WezTerm", "Ghostty", "Windows Terminal"] {
            assert!(tool.contains(name), "{text}: {tool}");
        }
    }
}

#[test]
fn a_ctrl_chord_after_a_prefix_matches_exactly_before_the_bare_key() {
    use KeyModifiers as M;
    let km = test_keymap(&[("git_prefix", "stage_all", &["Ctrl+X", "a"])]);
    let git = |c, m| km.action_in(Ctx::GitPrefix, &ev(KeyCode::Char(c), m));
    // Ctrl+X is its own key: it is not the discard `x`, and `x` is unchanged.
    assert_eq!(git('x', M::CONTROL), Some(Action::GitStageAll));
    assert_eq!(git('x', M::NONE), Some(Action::GitDiscardAll));
    // An unbound Ctrl+S still means `s`.
    assert_eq!(git('s', M::CONTROL), Some(Action::GitStageFile));
    assert_eq!(git('P', M::SHIFT), Some(Action::GitPush));
    assert_eq!(git('a', M::NONE), Some(Action::GitStageAll));
    assert_eq!(git('z', M::CONTROL), None);
    assert!(detect(&km, LINUX).iter().all(|c| c.level != Level::Error));
}

#[test]
fn only_alt_is_refused_after_a_prefix() {
    let km = Keymap::from_config(&cfg(&[("git_prefix", "stage_all", &["Alt+X"])]));
    assert_eq!(km.issues().len(), 1, "{:?}", km.issues());
    let ok = Keymap::from_config(&cfg(&[("git_prefix", "stage_all", &["Ctrl+X"])]));
    assert!(ok.issues().is_empty(), "{:?}", ok.issues());
    let d = Keymap::defaults();
    let a = |chord: &str| {
        super::conflict::analyse(
            &d,
            LINUX,
            Action::GitStageAll,
            &[KeyChord::parse(chord).unwrap()],
        )
    };
    assert!(a("Alt+X").refusal.is_some());
    assert!(a("Ctrl+X").refusal.is_none());
}
