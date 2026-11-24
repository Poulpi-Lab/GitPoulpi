//! The generated `Sequence: <name>` rows (never written by hand).

use super::*;
use crate::keymap::{Action, Keymap};
use crate::palette::matching::PaletteMatch;
use crate::palette::SeqOp;
use crate::sequence_run::registry::{self, refresh_defs, test_def};

fn rows(query: &str) -> Vec<PaletteMatch> {
    palette_matches(query, PaletteContext::default())
        .into_iter()
        .filter(|m| matches!(m.entry.command, PaletteCommand::Sequence(_)))
        .collect()
}

#[test]
fn a_row_follows_the_config_without_a_restart() {
    refresh_defs(&[test_def("pal-a")]);
    let slot_a = registry::slot_of("pal-a").unwrap();
    let found = rows("sequence: name of pal-a");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].entry.name(), "Sequence: Name of pal-a");
    assert_eq!(
        found[0].entry.command,
        PaletteCommand::Sequence(SeqOp::Run(slot_a))
    );
    assert_eq!(found[0].unavailable, None);

    // A sequence added later shows up, one removed goes.
    refresh_defs(&[test_def("pal-a"), test_def("pal-b")]);
    assert_eq!(rows("sequence: name of pal-b").len(), 1);
    refresh_defs(&[test_def("pal-b")]);
    assert!(rows("sequence: name of pal-a")
        .iter()
        .all(|m| m.entry.name() != "Sequence: Name of pal-a"));
    // Found by its own name alone, in the middle of the label.
    assert!(!rows("pal-b").is_empty());
    // Not part of the fixed catalogue.
    assert!(PALETTE_COMMANDS
        .iter()
        .all(|e| !matches!(e.command, PaletteCommand::Sequence(_))));
}

#[test]
fn the_shortcut_column_shows_the_bound_key() {
    refresh_defs(&[test_def("pal-key")]);
    let entry = rows("sequence: name of pal-key")[0].entry;
    assert_eq!(entry_shortcut(&Keymap::defaults(), entry), "");
    let km = Keymap::from_config(&gitpoulpi_config::KeybindingsConfig::from_entries([(
        "sequence",
        "pal-key",
        vec!["Ctrl+Shift+F7".to_string()],
    )]));
    assert!(km.issues().is_empty(), "{:?}", km.issues());
    assert_eq!(entry_shortcut(&km, entry), "Ctrl+Shift+F7");
    // The "configure the shortcut" button opens Settings on its action.
    let PaletteCommand::Sequence(op) = entry.command else {
        unreachable!()
    };
    assert_eq!(bind_action(entry), Some(Action::Sequence(op.slot())));
}

#[test]
fn an_invalid_sequence_is_grayed_with_the_reason() {
    let mut broken = test_def("pal-broken");
    broken.steps.clear();
    refresh_defs(&[broken, test_def("pal-fine")]);
    let broken = &rows("sequence: name of pal-broken")[0];
    assert_eq!(broken.unavailable, Some("the sequence has no steps"));
    assert_eq!(rows("sequence: name of pal-fine")[0].unavailable, None);
}

#[test]
fn a_sequence_does_not_run_over_the_settings_screen() {
    refresh_defs(&[test_def("pal-settings")]);
    let ctx = PaletteContext {
        settings_open: true,
        ..PaletteContext::default()
    };
    let slot = registry::slot_of("pal-settings").unwrap();
    assert_eq!(
        unavailable_reason(PaletteCommand::Sequence(SeqOp::Run(slot)), ctx),
        Some(gitpoulpi_i18n::msg::palette::WHY_CLOSE_SETTINGS_FIRST.get())
    );
}

#[test]
fn the_row_reads_in_the_ui_language() {
    refresh_defs(&[test_def("pal-fr")]);
    let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let found = rows("name of pal-fr");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].entry.name(), "Séquence\u{a0}: Name of pal-fr");
    // An English query still finds it.
    assert_eq!(rows("sequence: name of pal-fr").len(), 1);
}
