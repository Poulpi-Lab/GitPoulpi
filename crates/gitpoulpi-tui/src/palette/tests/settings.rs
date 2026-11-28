//! The generated `Settings: ...` rows (never written by hand).

use super::*;
use crate::palette::settings::SettingOp;

fn find(query: &str, label: &str) -> Option<&'static PaletteEntry> {
    palette_matches(query, PaletteContext::default())
        .into_iter()
        .map(|m| m.entry)
        .find(|e| e.name() == label)
}

#[test]
fn toggles_open_and_numbers_get_rows_with_the_pages_labels() {
    let toggle = find("settings: toggle", "Settings: Toggle Remember tabs");
    assert!(
        matches!(
            toggle.map(|e| e.command),
            Some(PaletteCommand::Setting(SettingOp::Toggle(_)))
        ),
        "{:?}",
        palette_matches("settings: toggle", PaletteContext::default())
            .iter()
            .map(|m| m.entry.name())
            .collect::<Vec<_>>()
    );
    let number = palette_matches("settings: initial commits", PaletteContext::default());
    assert!(number
        .iter()
        .any(|m| matches!(m.entry.command, PaletteCommand::Setting(SettingOp::Edit(_)))));
    // Ordinary commands are still there, and a query for them is unchanged.
    assert!(find("quit", "App: Quit").is_some());
}

#[test]
fn a_choices_values_show_only_once_the_setting_is_opened() {
    let all = palette_matches("", PaletteContext::default());
    assert!(
        all.iter().all(|m| !matches!(
            m.entry.command,
            PaletteCommand::Setting(SettingOp::Choose(..))
        )),
        "the choices of every setting would bury the palette"
    );
    let sub = palette_matches("settings: auto-stash before", PaletteContext::default())
        .into_iter()
        .find(|m| matches!(m.entry.command, PaletteCommand::Setting(SettingOp::Sub(_))))
        .expect("the choice row");
    let PaletteCommand::Setting(op) = sub.entry.command else {
        unreachable!()
    };
    let query = crate::palette::settings::sub_query(op).expect("a choice narrows the palette");
    let narrowed = palette_matches(&query, PaletteContext::default());
    let names: Vec<&str> = narrowed.iter().map(|m| m.entry.name()).collect();
    assert_eq!(names.len(), 3, "{names:?}");
    assert!(names
        .iter()
        .all(|n| n.starts_with("Settings: Auto-stash before")));
    assert!(narrowed.iter().all(|m| matches!(
        m.entry.command,
        PaletteCommand::Setting(SettingOp::Choose(..))
    )));
}

#[test]
fn a_page_only_setting_opens_the_page_and_no_setting_needs_a_repository() {
    let on_home = PaletteContext {
        in_repository_management: true,
        ..PaletteContext::default()
    };
    for m in palette_matches("settings:", on_home) {
        if matches!(m.entry.command, PaletteCommand::Setting(_)) {
            assert_eq!(m.unavailable, None, "{}", m.entry.name());
        }
    }
}

#[test]
fn settings_rows_have_no_shortcut_to_configure() {
    let any = palette_matches("settings: toggle", PaletteContext::default())
        .into_iter()
        .find(|m| matches!(m.entry.command, PaletteCommand::Setting(_)))
        .unwrap();
    assert_eq!(bind_action(any.entry), None);
}

/// Only the setting row itself narrows the palette; running a picked choice
/// writes it (it must not narrow again).
#[test]
fn only_the_setting_row_has_a_sub_query() {
    let all = palette_matches("settings: default layout", PaletteContext::default());
    assert!(!all.is_empty());
    for m in all {
        if let PaletteCommand::Setting(op) = m.entry.command {
            let is_sub = matches!(op, SettingOp::Sub(_));
            assert_eq!(
                crate::palette::settings::sub_query(op).is_some(),
                is_sub,
                "{}",
                m.entry.name()
            );
        }
    }
}

/// The labels exist only in the UI language; the setting's key in English
/// words still finds the row, so an English query works in French.
#[test]
fn an_english_query_finds_a_setting_in_french() {
    let _fr = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let found = palette_matches("remember tabs", PaletteContext::default());
    assert!(
        found.iter().any(|m| matches!(
            m.entry.command,
            PaletteCommand::Setting(SettingOp::Toggle(_))
        )),
        "{:?}",
        found.iter().map(|m| m.entry.name()).collect::<Vec<_>>()
    );
}

/// The Settings rows that exist only because of what a terminal is carry
/// `Surface::TerminalOnly` with a reason; every other row is for any frontend.
/// The keys are named in `gitpoulpi_app::surface`, so a renamed setting fails
/// here instead of silently becoming "for any frontend".
#[test]
fn the_terminal_only_settings_are_marked_where_the_rows_are_built() {
    use gitpoulpi_app::surface::{Surface, TERMINAL_ONLY_SETTINGS};
    let rows = crate::preferences::palette_settings::rows();
    for (key, reason) in TERMINAL_ONLY_SETTINGS {
        assert!(
            rows.iter().any(|r| r.key == *key),
            "{key} is no longer a Settings row"
        );
        assert!(!reason.trim().is_empty());
    }
    let entries = crate::palette::settings::entries();
    assert!(!entries.is_empty());
    let marked = entries
        .iter()
        .filter(|e| matches!(e.surface, Surface::TerminalOnly(_)))
        .count();
    assert!(
        marked >= TERMINAL_ONLY_SETTINGS.len(),
        "{marked} rows marked"
    );
    for entry in entries {
        if let Surface::TerminalOnly(reason) = entry.surface {
            assert!(!reason.trim().is_empty(), "{}", entry.name());
        }
    }
}
