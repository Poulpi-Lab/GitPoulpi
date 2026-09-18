use super::*;

fn form() -> ComposeForm {
    ComposeForm::new(vec![
        ("feat: a".into(), vec!["a.rs".into(), "b.rs".into()]),
        ("fix: c".into(), vec!["c.rs".into()]),
        ("docs: d".into(), vec!["d.md".into()]),
    ])
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn alt(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::ALT)
}

#[test]
fn commits_reorder_and_the_selection_follows() {
    let mut f = form();
    f.move_commit(0, 1);
    let messages: Vec<&str> = f.commits().iter().map(|c| c.message.as_str()).collect();
    assert_eq!(messages, ["fix: c", "feat: a", "docs: d"]);
    assert_eq!(f.selected, ComposeRow::Commit(1));
    f.move_commit(2, 1); // past the end: nothing
    f.move_commit(0, -1);
    assert_eq!(f.commits().len(), 3);
}

#[test]
fn a_file_moves_between_commits_and_an_emptied_commit_disappears() {
    let mut f = form();
    // c.rs (the only file of commit 1) goes to commit 0: commit 1 is gone.
    f.move_file(1, 0, -1);
    assert_eq!(f.commits().len(), 2);
    assert_eq!(f.commits()[0].files, ["a.rs", "b.rs", "c.rs"]);
    assert_eq!(f.commits()[1].message, "docs: d");
    assert_eq!(f.selected, ComposeRow::File(0, 2));
    // Moving off the ends does nothing.
    f.move_file(0, 0, -1);
    assert_eq!(f.commits()[0].files.len(), 3);
}

#[test]
fn a_file_can_be_split_into_a_commit_of_its_own_but_not_a_lone_file() {
    let mut f = form();
    f.file_to_new_commit(0, 1);
    assert_eq!(f.commits().len(), 4);
    assert_eq!(f.commits()[1].files, ["b.rs"]);
    assert_eq!(f.commits()[1].message, "New commit");
    assert_eq!(f.commits()[0].files, ["a.rs"]);
    f.file_to_new_commit(0, 0);
    assert_eq!(
        f.commits().len(),
        4,
        "a lone file is already its own commit"
    );
    assert!(f.error.is_some());
}

#[test]
fn a_dropped_commit_is_left_out_of_the_plan() {
    let mut f = form();
    f.toggle_drop(1);
    let plan = f.plan();
    assert_eq!(plan.len(), 2);
    assert!(plan.iter().all(|(m, _)| m != "fix: c"));
    f.toggle_drop(1);
    assert_eq!(f.plan().len(), 3);
}

#[test]
fn applying_builds_one_action_in_order_and_refuses_an_empty_plan() {
    let mut f = form();
    match f.try_apply() {
        ComposeOutcome::Submit(Action::ComposeCommits { commits }) => {
            assert_eq!(commits.len(), 3);
            assert_eq!(commits[0].message, "feat: a");
            assert_eq!(commits[0].paths.len(), 2);
        }
        other => panic!("{other:?}"),
    }
    for i in 0..3 {
        f.toggle_drop(i);
    }
    assert_eq!(f.try_apply(), ComposeOutcome::Stay);
    assert!(f.error.as_deref().unwrap().contains("dropped"));
}

/// `compose.apply = "Alt+c"`: the new chord creates the commits.
#[test]
fn a_rebound_apply_chord_creates_the_commits() {
    let km = crate::keymap::test_keymap(&[("compose", "apply", &["Alt+c"])]);
    crate::keymap::with_test_keymap(km, || {
        let mut f = form();
        assert_eq!(f.on_key(ctrl('s')), ComposeOutcome::Stay);
        let out = f.on_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::ALT));
        assert!(matches!(out, ComposeOutcome::Submit(_)), "{out:?}");
    });
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

#[test]
fn the_reword_popup_edits_message_and_description_and_esc_discards() {
    let mut f = form();
    f.on_key(key(KeyCode::Char('e')));
    assert!(f.editing.is_some());
    f.on_key(key(KeyCode::Char('!')));
    // Edited: the first Esc asks inline, the second discards.
    f.on_key(key(KeyCode::Esc));
    assert!(f.editing.is_some());
    f.on_key(key(KeyCode::Esc));
    assert!(f.editing.is_none());
    assert_eq!(f.commits()[0].message, "feat: a");
    // Enter opens it too; Enter in the summary moves to the
    // description; Ctrl+S keeps both.
    f.on_key(key(KeyCode::Enter));
    f.on_key(key(KeyCode::Char('!')));
    f.on_key(key(KeyCode::Enter));
    for c in "body".chars() {
        f.on_key(key(KeyCode::Char(c)));
    }
    f.on_key(ctrl('s'));
    assert!(f.editing.is_none());
    assert_eq!(f.commits()[0].message, "feat: a!\n\nbody");
    // The plan carries the whole message.
    assert_eq!(f.plan()[0].0, "feat: a!\n\nbody");
    // An empty message is refused in the popup, which stays open.
    let mut g = form();
    g.on_key(key(KeyCode::Char('e')));
    g.on_key(ctrl('a'));
    g.on_key(key(KeyCode::Backspace));
    g.on_key(ctrl('s'));
    assert!(g.editing.is_some());
    assert_eq!(g.commits()[0].message, "feat: a");
}

#[test]
fn a_double_click_on_any_row_opens_the_popup_a_single_one_only_selects() {
    let mut f = form();
    f.on_click(ComposePart::Row(ComposeRow::Commit(1)));
    assert_eq!(f.selected, ComposeRow::Commit(1));
    assert!(f.editing.is_none());
    f.on_click(ComposePart::Row(ComposeRow::Commit(1)));
    assert_eq!(f.editing.as_ref().map(|e| e.0), Some(1));
    f.on_key(key(KeyCode::Esc));
    // A file row edits its commit.
    f.on_click(ComposePart::Row(ComposeRow::File(0, 1)));
    f.on_click(ComposePart::Row(ComposeRow::File(0, 1)));
    assert_eq!(f.editing.as_ref().map(|e| e.0), Some(0));
}

#[test]
fn the_selected_proposal_is_the_one_the_selected_file_belongs_to() {
    let mut f = form();
    assert_eq!(f.selected_commit().map(|c| c.0), Some(0));
    f.on_click(ComposePart::Row(ComposeRow::File(1, 0)));
    assert_eq!(f.selected_commit().map(|c| c.0), Some(1));
}

#[test]
fn keys_drive_every_control() {
    let mut f = form();
    f.on_key(key(KeyCode::Down)); // file a.rs
    assert_eq!(f.selected, ComposeRow::File(0, 0));
    f.on_key(alt(KeyCode::Down)); // to the next commit
    assert_eq!(f.commits()[1].files, ["c.rs", "a.rs"]);
    f.on_key(key(KeyCode::Char('n'))); // a.rs into its own commit
    assert_eq!(f.commits().len(), 4);
    f.on_key(key(KeyCode::Char('d'))); // drop it
    assert!(f.commits()[2].dropped);
    f.on_key(key(KeyCode::Char('r'))); // reset
    assert_eq!(f.commits().len(), 3);
    assert_eq!(f.on_key(key(KeyCode::Esc)), ComposeOutcome::Close);
    assert!(matches!(
        f.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)),
        ComposeOutcome::Submit(_)
    ));
}

#[test]
fn clicks_do_what_their_keys_do() {
    let mut f = form();
    f.on_click(ComposePart::CommitDown(0));
    assert_eq!(f.commits()[1].message, "feat: a");
    f.on_click(ComposePart::CommitDrop(1));
    assert!(f.commits()[1].dropped);
    f.on_click(ComposePart::Reset);
    assert_eq!(f.commits()[0].message, "feat: a");
    assert_eq!(f.on_click(ComposePart::Cancel), ComposeOutcome::Close);
    assert!(matches!(
        f.on_click(ComposePart::Apply),
        ComposeOutcome::Submit(_)
    ));
}

#[test]
fn the_popup_draws_its_rows_and_registers_its_buttons() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let f = form();
    let mut terminal = Terminal::new(TestBackend::new(110, 24)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|frame| {
            hits = f.render(frame, frame.area(), None);
        })
        .unwrap();
    let buffer = terminal.backend().buffer();
    let text: String = (0..24)
        .map(|y| {
            (0..110)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("Compose commits with AI"), "{text}");
    assert!(text.contains("feat: a"), "{text}");
    assert!(text.contains("b.rs"), "{text}");
    assert!(text.contains("Create 3 commits"), "{text}");
    for part in [ComposePart::Apply, ComposePart::Cancel, ComposePart::Reset] {
        assert!(hits.iter().any(|(_, p)| *p == part), "{part:?}");
    }
    // The selected commit row offers its four controls.
    for part in [
        ComposePart::CommitUp(0),
        ComposePart::CommitDown(0),
        ComposePart::CommitEdit(0),
        ComposePart::CommitDrop(0),
    ] {
        assert!(hits.iter().any(|(_, p)| *p == part), "{part:?}");
    }
}

#[test]
fn a_dropped_file_lands_in_exactly_one_commit_and_collapsed_rows_hide_files() {
    let mut f = form();
    f.on_press(ComposePart::Row(ComposeRow::File(0, 1)), 0, 0);
    f.on_file_drag(Some(ComposePart::Row(ComposeRow::Commit(2))));
    f.on_drop(Some(ComposePart::Row(ComposeRow::Commit(2))));
    assert_eq!(f.commits()[0].files, ["a.rs"]);
    assert_eq!(f.commits()[2].files, ["d.md", "b.rs"]);
    let all: Vec<&String> = f.commits().iter().flat_map(|c| &c.files).collect();
    assert_eq!(all.len(), 4);
    // A press without motion is only a click: nothing moves.
    f.on_press(ComposePart::Row(ComposeRow::File(0, 0)), 0, 0);
    f.on_drop(Some(ComposePart::Row(ComposeRow::Commit(1))));
    assert_eq!(f.commits()[0].files, ["a.rs"]);
    // Collapsing hides the file rows and selects the commit.
    f.on_click(ComposePart::Row(ComposeRow::File(2, 0)));
    f.toggle_expanded(2, Some(false));
    assert!(!f.rows().contains(&ComposeRow::File(2, 0)));
    assert_eq!(f.selected, ComposeRow::Commit(2));
    let c = ComposeForm::with_expanded(vec![("x".into(), vec!["a".into()])], false);
    assert_eq!(c.rows(), [ComposeRow::Commit(0)]);
}

#[test]
fn the_move_dropdown_offers_the_other_commits_and_a_new_one() {
    let mut f = form();
    f.on_click(ComposePart::Row(ComposeRow::File(0, 0)));
    f.on_key(key(KeyCode::Char('m')));
    assert!(f.is_moving());
    f.on_key(key(KeyCode::Esc));
    assert!(!f.is_moving());
    f.on_key(key(KeyCode::Char('m')));
    f.on_key(key(KeyCode::Down)); // second option: commit 3 (docs: d)
    f.on_key(key(KeyCode::Enter));
    assert!(!f.is_moving());
    assert_eq!(f.commits()[2].files, ["d.md", "a.rs"]);
}

/// The review in French (I8b): header, hint and the footer buttons stay in
/// the view at 100 columns, git terms on or off.
#[test]
fn the_french_review_fits_with_git_terms_on_and_off() {
    use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    for terms in [false, true] {
        let _l = override_lang(Lang::Fr);
        let _t = override_git_terms(terms);
        let f = form();
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        let mut hits = Vec::new();
        terminal
            .draw(|frame| hits = f.render(frame, frame.area(), None))
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let all: String = (0..24)
            .map(|y| (0..100).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!all.contains("{$"), "{all}");
        assert!(all.contains("L'IA propose 3"), "terms={terms}\n{all}");
        assert!(all.contains("Relisez"), "terms={terms}\n{all}");
        let create = if terms {
            "Créer 3 validations"
        } else {
            "Créer 3 commits"
        };
        assert!(all.contains(create), "terms={terms}\n{all}");
        for (rect, _) in &hits {
            assert!(rect.x + rect.width <= 100, "{rect:?}");
        }
    }
}
