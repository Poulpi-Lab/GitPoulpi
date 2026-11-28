use super::*;
use gitpoulpi_secrets::InMemoryTokens;

fn state(dir: &Path) -> PreferencesState {
    let mut state = PreferencesState::with_parts(
        Config::default(),
        Some(dir.join("config.toml")),
        dir,
        RepoState::default(),
        RepoState::path_for(dir, ".gitpoulpi"),
        std::sync::Arc::new(InMemoryTokens::default()),
    );
    state.show_keybindings(false);
    state
}

fn press(state: &mut PreferencesState, code: KeyCode) {
    state.keys_key(&KeyEvent::new(code, KeyModifiers::NONE));
}

fn typed(state: &mut PreferencesState, text: &str) {
    for c in text.chars() {
        press(state, KeyCode::Char(c));
    }
}

fn select(state: &mut PreferencesState, id: &str) {
    state.keys.selected = ACTIONS.iter().position(|d| d.config_id() == id).expect(id);
}

fn file(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("tui.toml")).unwrap_or_default()
}

fn draw(state: &mut PreferencesState) -> String {
    use ratatui::backend::TestBackend;
    let mut terminal = ratatui::Terminal::new(TestBackend::new(140, 40)).unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..40)
        .map(|y| {
            (0..140)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn search_matches_label_id_and_chord_text() {
    let km = Keymap::defaults();
    let find = |q: &str| -> Vec<&'static str> {
        ACTIONS
            .iter()
            .filter(|d| matches_query(q, d, &km))
            .map(|d| d.config_id())
            .collect()
    };
    assert!(find("stage selected").contains(&"git_prefix.stage_file"));
    assert!(find("git_prefix.stage_file").contains(&"git_prefix.stage_file"));
    let by_chord = find("ctrl+g");
    assert!(by_chord.contains(&"git_prefix.stage_file"), "{by_chord:?}");
    assert!(by_chord.contains(&"global.git_prefix"));
    assert!(!by_chord.contains(&"nav_prefix.help"));
    assert!(find("zzzz-nothing").is_empty());
}

#[test]
fn the_context_and_show_filters_narrow_the_list() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    assert!(s.keys.visible().len() >= ACTIONS.len().min(10));
    s.keys.context.select(Some("nav_prefix".into()));
    s.keys.reselect();
    assert!(s
        .keys
        .visible()
        .iter()
        .all(|i| ACTIONS[*i].ctx == Ctx::NavPrefix));
    s.keys.context.select(Some("all".into()));
    s.keys.show.select(Some("modified".into()));
    s.keys.reselect();
    assert!(s.keys.visible().is_empty(), "nothing is modified yet");
}

#[test]
fn capture_conflict_offers_replace_and_replace_unbinds_the_other() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    select(&mut s, "git_prefix.stage_file");
    press(&mut s, KeyCode::Enter);
    press(&mut s, KeyCode::Char('e'));
    let Some(Popup::Capture(cap)) = s.keys.popup.clone() else {
        panic!("capture popup");
    };
    assert_eq!(cap.analysis.replace, Some(Bind::GitEditMessage));
    assert_eq!(cap.buttons()[0], Btn::Replace);
    press(&mut s, KeyCode::Enter);
    assert!(s.keys.popup.is_none());
    let text = file(dir.path());
    assert!(text.contains("stage_file = \"e\""), "{text}");
    assert!(text.contains("edit_message = []"), "{text}");
    assert_eq!(s.keys.km.label(Bind::GitStageFile), "Ctrl+G e");
    assert!(s.keys.km.chords_of(Bind::GitEditMessage).is_empty());
    assert!(s.message.as_deref().unwrap_or("").contains("saved to"));
}

#[test]
fn a_free_key_saves_and_a_row_reset_removes_the_entry() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    select(&mut s, "git_prefix.stage_file");
    press(&mut s, KeyCode::Enter);
    press(&mut s, KeyCode::Char('1'));
    // A digit is the vim count in a pane; after a prefix it is free.
    let Some(Popup::Capture(cap)) = s.keys.popup.clone() else {
        panic!()
    };
    assert!(cap.analysis.replace.is_none());
    press(&mut s, KeyCode::Enter);
    assert!(file(dir.path()).contains("stage_file = \"1\""));
    assert!(s.keys.km.is_modified(Bind::GitStageFile));
    press(&mut s, KeyCode::Char('r'));
    assert!(
        !file(dir.path()).contains("stage_file"),
        "{}",
        file(dir.path())
    );
    assert!(!s.keys.km.is_modified(Bind::GitStageFile));
}

#[test]
fn fixed_keys_and_reserved_digits_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    select(&mut s, "global.refresh");
    press(&mut s, KeyCode::Enter);
    for code in [KeyCode::Char('j'), KeyCode::Char('5'), KeyCode::Down] {
        press(&mut s, KeyCode::Char('x'));
        press(&mut s, code);
        let Some(Popup::Capture(cap)) = s.keys.popup.clone() else {
            panic!()
        };
        assert!(cap.analysis.refusal.is_some(), "{code:?}");
        assert!(!cap.buttons().contains(&Btn::Save));
    }
    // Esc is never capturable: it cancels.
    press(&mut s, KeyCode::Esc);
    assert!(s.keys.popup.is_none());
    assert!(file(dir.path()).is_empty());
}

#[test]
fn type_it_instead_parses_or_reports() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    select(&mut s, "global.palette");
    press(&mut s, KeyCode::Enter);
    press(&mut s, KeyCode::Char('x'));
    // x is free? it is a global chord of nothing: pick TypeIt by Tab.
    let Some(Popup::Capture(cap)) = s.keys.popup.clone() else {
        panic!()
    };
    let at = cap
        .buttons()
        .iter()
        .position(|b| *b == Btn::TypeIt)
        .unwrap();
    for _ in 0..at {
        press(&mut s, KeyCode::Tab);
    }
    press(&mut s, KeyCode::Enter);
    typed(&mut s, "Ctrl+Nope");
    press(&mut s, KeyCode::Enter);
    let Some(Popup::Capture(cap)) = s.keys.popup.clone() else {
        panic!()
    };
    assert!(cap.typing.is_some() && cap.error.is_some());
    for _ in 0.."Ctrl+Nope".len() {
        press(&mut s, KeyCode::Backspace);
    }
    typed(&mut s, "Alt+F2");
    press(&mut s, KeyCode::Enter);
    let Some(Popup::Capture(cap)) = s.keys.popup.clone() else {
        panic!()
    };
    assert_eq!(cap.chord.map(|c| c.to_string()).as_deref(), Some("Alt+F2"));
    assert!(cap.typing.is_none());
}

#[test]
fn unbind_and_reset_all_use_the_writers() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    select(&mut s, "git_prefix.pull");
    press(&mut s, KeyCode::Char('u'));
    assert!(
        file(dir.path()).contains("pull = []"),
        "{}",
        file(dir.path())
    );
    assert!(s.keys.km.chords_of(Bind::GitPull).is_empty());
    // R asks first; Esc cancels and leaves the file alone.
    press(&mut s, KeyCode::Char('R'));
    assert!(matches!(s.keys.popup, Some(Popup::ConfirmReset { .. })));
    press(&mut s, KeyCode::Esc);
    assert!(file(dir.path()).contains("pull = []"));
    press(&mut s, KeyCode::Char('R'));
    press(&mut s, KeyCode::Char('y'));
    assert!(
        !file(dir.path()).contains("keybindings"),
        "{}",
        file(dir.path())
    );
    assert!(!s.keys.km.chords_of(Bind::GitPull).is_empty());
}

#[test]
fn a_modified_row_shows_the_dot_and_reset_and_a_warning_shows_a_bang() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.toml"),
        "[keybindings.git_prefix]\npull = \"Shift+X\"\n[keybindings.global]\nsettings = \"Ctrl+9\"\n",
    )
    .unwrap();
    let config = Config::load(&dir.path().join("config.toml")).unwrap();
    let mut s = PreferencesState::with_parts(
        config,
        Some(dir.path().join("config.toml")),
        dir.path(),
        RepoState::default(),
        RepoState::path_for(dir.path(), ".gitpoulpi"),
        std::sync::Arc::new(InMemoryTokens::default()),
    );
    s.show_keybindings(false);
    s.keys.query = "pull".into();
    s.keys.reselect();
    let screen = draw(&mut s);
    let row = screen
        .lines()
        .find(|l| l.contains("Pull") && !l.contains("LFS"))
        .expect("row");
    assert!(row.contains('\u{2022}') && row.contains("Reset"), "{row}");
    if cfg!(not(windows)) {
        s.keys.query = "Settings".into();
        s.keys.reselect();
        let screen = draw(&mut s);
        let row = screen
            .lines()
            .find(|l| l.contains("Ctrl+9"))
            .expect("settings row");
        assert!(row.contains('!'), "{row}");
    }
}

#[test]
fn the_popup_owns_the_pointer_and_its_hit_rects_come_first() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    select(&mut s, "git_prefix.stage_file");
    press(&mut s, KeyCode::Enter);
    draw(&mut s);
    assert_eq!(s.keys.popup_hits.last().map(|(_, h)| *h), Some(Hit::Popup));
    let (cancel, _) = *s
        .keys
        .popup_hits
        .iter()
        .find(|(_, h)| *h == Hit::Button(Btn::Cancel))
        .expect("cancel button");
    // A click on the button under which a list row is painted
    // cancels the popup and does not select that row.
    let before = s.keys.selected;
    s.on_mouse(MouseEventKind::Down(MouseButton::Left), cancel.x, cancel.y);
    assert!(s.keys.popup.is_none());
    assert_eq!(s.keys.selected, before);
}

/// The page in French, drawn at 100 columns: every control is there and the
/// English words are gone; the stable ids and key names stay as they are.
fn draw_at(state: &mut PreferencesState, w: u16, h: u16) -> String {
    use ratatui::backend::TestBackend;
    let mut terminal = ratatui::Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal
        .draw(|frame| state.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..h)
        .map(|y| (0..w).map(|x| buffer[(x, y)].symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_page_is_in_french_and_fits_100_columns() {
    let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    let screen = draw_at(&mut s, 100, 40).replace('\u{a0}', " ");
    for want in [
        "Clavier et souris",
        "Chercher",
        "nom, id ou touche",
        "Contexte",
        "Tous les contextes",
        "Voir",
        "Tous",
        "Réinit. tout...",
        "modifier",
        "Palette de commandes",
        "Préfixe des actions Git",
        "par défaut : Ctrl+G",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    for gone in ["Command Palette", "Search", "label, id or key"] {
        assert!(
            !screen.contains(gone),
            "{gone:?} is still English:\n{screen}"
        );
    }
    // The ids and the key names are not translated.
    assert!(screen.contains("Ctrl+P"), "{screen}");
    assert!(screen.contains("Enter"), "{screen}");
}

#[test]
fn the_french_header_controls_do_not_overlap() {
    let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    draw_at(&mut s, 100, 40);
    let page = &s.keys;
    assert!(
        page.search_rect.right() <= page.reset_all_rect.x
            || page.search_rect.y != page.reset_all_rect.y
    );
    assert!(
        page.reset_all_rect.right() <= 100,
        "the Reset button runs off the screen: {:?}",
        page.reset_all_rect
    );
}

#[test]
fn the_french_capture_popup_and_conflict_are_translated() {
    let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let _t = gitpoulpi_i18n::override_git_terms(false);
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    select(&mut s, "git_prefix.stage_file");
    press(&mut s, KeyCode::Enter);
    press(&mut s, KeyCode::Char('e'));
    let screen = draw_at(&mut s, 100, 40).replace('\u{a0}', " ");
    for want in [
        "Nouveau raccourci",
        "Appuyez sur la nouvelle touche pour Stage du fichier sélectionné - Esc annule",
        "après Ctrl+G",
        "Touche : e",
        "est déjà Modifier le message de",
        "Remplacer",
        "Saisir à la place...",
        "Annuler",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
}

#[test]
fn the_french_reset_confirmation_and_refusals_are_translated() {
    let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let dir = tempfile::tempdir().unwrap();
    let mut s = state(dir.path());
    s.keys.popup = Some(Popup::ConfirmReset { focus: 1 });
    let screen = draw_at(&mut s, 100, 40).replace('\u{a0}', " ");
    for want in [
        "Réinitialiser tous les raccourcis",
        "Chaque raccourci revient à sa valeur par défaut.",
        "La table [keybindings] est retirée de tui.toml.",
        "Tout réinitialiser",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    s.keys.popup = None;
    select(&mut s, "global.refresh");
    press(&mut s, KeyCode::Enter);
    press(&mut s, KeyCode::Esc);
    let _ = s;
    let refusal = crate::keymap::conflict::analyse(
        &Keymap::defaults(),
        crate::keymap::conflict::KeyTransport::Terminal {
            windows: false,
            enhanced: true,
        },
        Bind::GlobalRefresh,
        &[KeyChord::parse("Esc").unwrap()],
    )
    .refusal
    .unwrap();
    assert_eq!(refusal, "Esc\u{a0}: Esc est fixe\u{a0}: il annule toujours");
}

#[test]
fn french_search_matches_the_english_label_too() {
    let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let km = Keymap::defaults();
    let find = |q: &str| -> Vec<&'static str> {
        ACTIONS
            .iter()
            .filter(|d| matches_query(q, d, &km))
            .map(|d| d.config_id())
            .collect()
    };
    {
        let _t = gitpoulpi_i18n::override_git_terms(true);
        assert!(find("indexer fichier").contains(&"git_prefix.stage_file"));
        assert!(find("stage selected").contains(&"git_prefix.stage_file"));
    }
    let _t = gitpoulpi_i18n::override_git_terms(false);
    assert!(find("stage du fichier").contains(&"git_prefix.stage_file"));
    assert!(find("stage selected").contains(&"git_prefix.stage_file"));
}

#[test]
fn every_french_action_label_and_context_fits_the_list_column() {
    let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    for vocabulary in [false, true] {
        let _t = gitpoulpi_i18n::override_git_terms(vocabulary);
        // Palette commands' labels are the palette's own; the list ellipsizes.
        for def in ACTIONS
            .iter()
            .filter(|d| d.ctx != Ctx::Palette && d.present())
        {
            let (fr, en) = (def.display_label(), def.english_label());
            let (fr, en) = (&*fr, &*en);
            assert!(!fr.is_empty(), "{}", def.config_id());
            // The list column is at most 42 cells: stay near the English width.
            assert!(
                crate::display_width(fr) <= crate::display_width(en).max(40) + 12
                    && crate::display_width(fr) <= 46,
                "{}: {fr:?} is much longer than {en:?}",
                def.config_id()
            );
        }
        for ctx in Ctx::ALL {
            assert!(!ctx.title().get().is_empty());
        }
    }
}

#[test]
fn the_docs_table_stays_english_in_any_language() {
    let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    for def in ACTIONS.iter().filter(|d| d.present()) {
        assert_ne!(def.english_label(), "", "{}", def.config_id());
    }
    assert!(def_label_english("global.palette").contains("Command Palette"));
}

fn def_label_english(id: &str) -> String {
    ACTIONS
        .iter()
        .find(|d| d.config_id() == id)
        .unwrap()
        .english_label()
        .into_owned()
}

/// The selected action row is shown by its highlight alone: no arrow glyph.
#[test]
fn the_selected_action_row_has_no_arrow_marker() {
    let tmp = tempfile::tempdir().unwrap();
    let mut st = state(tmp.path());
    let screen = draw(&mut st);
    assert!(!screen.contains('\u{25B8}'), "{screen}");
}

#[test]
fn the_sequence_pool_shows_only_the_slots_a_sequence_holds() {
    use crate::sequence_run::registry::{refresh_defs, test_def};
    let dir = tempfile::tempdir().unwrap();
    refresh_defs(&[]);
    let s = state(dir.path());
    assert!(
        s.keys
            .visible()
            .iter()
            .all(|i| !matches!(ACTIONS[*i].action, crate::keymap::Action::Sequence(_))),
        "an unused slot is not listed"
    );
    refresh_defs(&[test_def("page-seq")]);
    let s = state(dir.path());
    let listed: Vec<_> = s
        .keys
        .visible()
        .into_iter()
        .filter(|i| matches!(ACTIONS[*i].action, crate::keymap::Action::Sequence(_)))
        .map(|i| {
            (
                ACTIONS[i].config_id(),
                ACTIONS[i].display_label().into_owned(),
            )
        })
        .collect();
    assert_eq!(
        listed,
        vec![(
            "sequence.page-seq",
            "Sequence: Name of page-seq".to_string()
        )]
    );
    // Searchable by its slug and by its name.
    let km = Keymap::defaults();
    let def = ACTIONS
        .iter()
        .find(|d| d.config_id() == "sequence.page-seq")
        .unwrap();
    assert!(matches_query("page-seq", def, &km));
    assert!(matches_query("name of page", def, &km));
}

/// The "entries ignored" line names the file the bad entry is read from:
/// `config.toml` while there is no `tui.toml`, `tui.toml` once it exists.
#[test]
fn the_ignored_entries_line_names_the_file_they_come_from() {
    let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::En);
    let bad = "[keybindings.global]\nsettings = \"Ctrl+Nope\"\n";
    for (file, seed_tui) in [("config.toml", false), ("tui.toml", true)] {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("config.toml"), bad).unwrap();
        if seed_tui {
            std::fs::write(dir.path().join("tui.toml"), bad).unwrap();
        }
        let config = Config::load(&dir.path().join("config.toml")).unwrap();
        let mut s = PreferencesState::with_parts(
            config,
            Some(dir.path().join("config.toml")),
            dir.path(),
            RepoState::default(),
            RepoState::path_for(dir.path(), ".gitpoulpi"),
            std::sync::Arc::new(InMemoryTokens::default()),
        );
        s.show_keybindings(false);
        let screen = draw_at(&mut s, 140, 40);
        let want = format!("{file}: 1 entry ignored");
        assert!(screen.contains(&want), "want {want:?}:\n{screen}");
        let other = if file == "tui.toml" {
            "config.toml: 1 entry"
        } else {
            "tui.toml: 1 entry"
        };
        assert!(!screen.contains(other), "{screen}");
    }
}
