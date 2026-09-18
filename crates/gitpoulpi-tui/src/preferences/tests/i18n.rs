//! Settings in another language: layout, search in both languages, and the
//! stored value staying the schema word whatever the display says.

use super::*;
use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};

/// The body width of a 100-column screen: 100 - 2 (frame) - 28 (sidebar) - 2.
const BODY_WIDTH: usize = 68;

fn field_index(state: &PreferencesState, key: &str) -> usize {
    state
        .fields()
        .iter()
        .position(|f| matches!(f.target, Target::Global(k) if k == key))
        .unwrap_or_else(|| panic!("no row for {key}"))
}

#[test]
fn general_page_in_french_fits_at_100_columns() {
    let _fr = override_lang(Lang::Fr);
    let dir = tempfile::tempdir().unwrap();
    let mut state = super::state(dir.path());
    go(&mut state, PrefSection::Sync);
    let screen = draw_screen(&mut state);
    assert!(screen.contains("Intervalle d'auto-fetch"), "{screen}");
    assert!(screen.contains("Paramètres"), "{screen}");
    assert!(screen.contains("Profil courant"), "{screen}");
    assert!(!screen.contains("Auto-Fetch Interval"), "{screen}");
    // Every row of the page, translated: label, leader and value on one
    // line no wider than the body, whichever row is selected.
    for lang in [Lang::Fr, Lang::En] {
        let _l = override_lang(lang);
        for terms in [false, true] {
            let _t = override_git_terms(terms);
            // The AI page is the next slice (I4b); its rows are English.
            for section in PrefSection::ALL
                .iter()
                .filter(|s| !matches!(s, PrefSection::Ai | PrefSection::RepoAi))
            {
                let dir = tempfile::tempdir().unwrap();
                let mut state = super::state(dir.path());
                state.section = PrefSection::ALL.iter().position(|s| s == section).unwrap();
                for (index, field) in state.fields().iter().enumerate() {
                    if field.coming.is_some() {
                        // The reason replaces the value and was always one long row.
                        continue;
                    }
                    for selected in [false, true] {
                        for line in state.field_lines(field, index, selected, BODY_WIDTH) {
                            let text: String =
                                line.spans.iter().map(|s| s.content.as_ref()).collect();
                            assert!(
                                display_width(&text) <= BODY_WIDTH,
                                "{lang:?} {section:?} {:?}: {} cells: {text}",
                                field.label,
                                display_width(&text)
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn a_long_label_wraps_in_its_column_instead_of_being_cut() {
    let _de = override_lang(Lang::De);
    let dir = tempfile::tempdir().unwrap();
    let mut state = super::state(dir.path());
    state.section = PrefSection::ALL
        .iter()
        .position(|s| *s == PrefSection::LogGraph)
        .unwrap();
    let fields = vec![Field::plain(
        "Automatisch abrufen, wenn die Anwendung gestartet wird",
        Control::Toggle(false),
        Target::None,
    )];
    let (index, field) = (0, &fields[0]);
    state.set_label_col(&fields, 80);
    let lines = state.field_lines(field, index, false, 80);
    let text = |l: &Line| {
        l.spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect::<String>()
    };
    let rows: Vec<String> = lines.iter().map(text).collect();
    let label = crate::terminology::tr(&field.label).into_owned();
    assert!(
        display_width(&label) > 20,
        "the German label is long: {label}"
    );
    assert!(rows.iter().all(|r| !r.contains('\u{2026}')), "{rows:?}");
    // The value sits on the first line, and every word of the label is drawn
    // on the two first lines.
    assert!(
        rows[0].contains("[ ]") || rows[0].contains("[x]"),
        "{rows:?}"
    );
    let words: Vec<&str> = label.split_whitespace().collect();
    assert!(
        words
            .iter()
            .all(|w| rows[..2].iter().any(|r| r.contains(w))),
        "every word of the label is drawn: {rows:?}"
    );
}

#[test]
fn search_finds_a_row_by_its_french_or_its_english_label() {
    let _fr = override_lang(Lang::Fr);
    let find = |query: &str| -> String {
        let dir = tempfile::tempdir().unwrap();
        let mut state = super::state(dir.path());
        state.on_key(KeyEvent::from(KeyCode::Char('/')));
        for c in query.chars() {
            state.on_key(KeyEvent::from(KeyCode::Char(c)));
        }
        state.on_key(KeyEvent::from(KeyCode::Enter));
        assert!(state.search.is_none(), "{query}: Enter jumped");
        assert_eq!(state.current_section(), PrefSection::DiffEditor, "{query}");
        state.fields()[state.field].label.clone()
    };
    assert_eq!(find("tabulation"), "Taille de tabulation");
    assert_eq!(find("tab size"), "Taille de tabulation");
    assert_eq!(find("TAB SIZE"), "Taille de tabulation");
}

#[test]
fn search_in_english_does_not_double_up() {
    let dir = tempfile::tempdir().unwrap();
    let state = super::state(dir.path());
    let entries = state.search_entries();
    assert!(entries.iter().all(|e| e.label_en.is_empty()));
}

#[test]
fn choices_show_the_translation_and_store_the_schema_word() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = super::state(dir.path());
    go(&mut state, PrefSection::Sync);
    let _fr = override_lang(Lang::Fr);
    let index = field_index(&state, "general.auto_stash");
    state.field = index;
    let fields = state.fields();
    assert_eq!(fields[index].value_text(), "demander");
    let Control::Choice { options, .. } = &fields[index].control else {
        panic!("a choice");
    };
    assert_eq!(options, &["ask", "always", "never"], "raw schema words");
    state.commit_choice(&fields[index].target, 1);
    assert_eq!(
        state.config.general.auto_stash,
        gitpoulpi_config::AutoStashMode::Always
    );
    assert_eq!(state.fields()[index].value_text(), "toujours");

    // The four rows that used to store their label: still the schema word.
    for (key, pick, raw) in [
        ("ui.graph_lane_background", 1, "row_tint"),
        ("editor.eol", 2, "crlf"),
        ("ui.pr_terminology", 1, "pull_request"),
        ("ui.graph_row_tint_strength", 2, "subtle"),
    ] {
        let section = PrefSection::ALL
            .iter()
            .position(|s| {
                state
                    .fields_of(*s)
                    .iter()
                    .any(|f| matches!(f.target, Target::Global(k) if k == key))
            })
            .unwrap();
        state.section = section;
        let index = field_index(&state, key);
        state.field = index;
        let target = state.fields()[index].target.clone();
        state.commit_choice(&target, pick);
        let on_disk = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
        assert!(on_disk.contains(raw), "{key}: {on_disk}");
    }
}

#[test]
fn the_git_terms_setting_reaches_settings_labels() {
    let _fr = override_lang(Lang::Fr);
    let dir = tempfile::tempdir().unwrap();
    let mut state = super::state(dir.path());
    go(&mut state, PrefSection::Sync);
    let label = |terms: bool| {
        let _t = override_git_terms(terms);
        state.fields()[0].label.clone()
    };
    assert_eq!(label(false), "Intervalle d'auto-fetch");
    assert_eq!(label(true), "Intervalle de récupération auto");
}
