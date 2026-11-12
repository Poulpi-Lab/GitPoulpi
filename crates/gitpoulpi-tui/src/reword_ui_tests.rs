use super::*;
use crate::test_support::*;
use gitpoulpi_core::domain::RefVisibility;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

fn type_str(form: &mut RewordForm, s: &str) {
    for c in s.chars() {
        form.on_key(key(KeyCode::Char(c)));
    }
}

fn branch(name: &str, target: u8, is_head: bool, upstream: Option<&str>) -> Ref {
    Ref {
        kind: RefKind::LocalBranch { name: name.into() },
        target: oid(target),
        visibility: RefVisibility::Shown,
        is_head,
        upstream: upstream.map(str::to_string),
        tag_sync: None,
    }
}

fn remote(remote: &str, name: &str, target: u8) -> Ref {
    Ref {
        kind: RefKind::RemoteBranch {
            remote: remote.into(),
            name: name.into(),
        },
        target: oid(target),
        visibility: RefVisibility::Shown,
        is_head: false,
        upstream: None,
        tag_sync: None,
    }
}

/// 1 <- 2 <- 3 <- 4 (main, HEAD); origin/main is at 2; 9 is on
/// another branch, off 1.
fn graph() -> CommitGraph {
    CommitGraph {
        nodes: vec![
            commit(4, &[3]),
            commit(3, &[2]),
            commit(9, &[1]),
            commit(2, &[1]),
            commit(1, &[]),
        ],
    }
}

#[test]
fn a_commit_on_the_upstream_is_pushed_and_one_above_it_is_not() {
    let refs = vec![
        branch("main", 4, true, Some("origin/main")),
        remote("origin", "main", 2),
    ];
    let g = graph();
    assert_eq!(
        pushed_upstream(&g, &refs, oid(2)).as_deref(),
        Some("origin/main")
    );
    assert_eq!(
        pushed_upstream(&g, &refs, oid(1)).as_deref(),
        Some("origin/main")
    );
    assert_eq!(pushed_upstream(&g, &refs, oid(3)), None);
    assert_eq!(pushed_upstream(&g, &refs, oid(4)), None);
    // No upstream: nothing to force push.
    let unset = vec![branch("main", 4, true, None), remote("origin", "main", 2)];
    assert_eq!(pushed_upstream(&g, &unset, oid(2)), None);
}

#[test]
fn only_head_and_its_ancestors_are_on_the_current_branch() {
    let refs = vec![branch("main", 4, true, None)];
    let g = graph();
    assert_eq!(on_current_branch(&g, &refs, oid(4)), Some(true));
    assert_eq!(on_current_branch(&g, &refs, oid(1)), Some(true));
    assert_eq!(on_current_branch(&g, &refs, oid(9)), Some(false));
    // Detached HEAD: the Log cannot tell, the backend decides.
    let detached = vec![branch("main", 4, false, None)];
    assert_eq!(on_current_branch(&g, &detached, oid(9)), None);
}

#[test]
fn the_fields_edit_summary_and_description() {
    let mut form = RewordForm::new(oid(1), "summary\n\nbody line", None);
    assert_eq!(form.summary, "summary");
    assert_eq!(form.description, "body line");
    type_str(&mut form, "!");
    // Enter in the summary moves to the description; there Enter
    // breaks the line.
    form.on_key(key(KeyCode::Enter));
    assert_eq!(form.focus, RewordPart::Description);
    type_str(&mut form, " more");
    form.on_key(key(KeyCode::Enter));
    type_str(&mut form, "second");
    assert_eq!(form.text(), "summary!\n\nbody line more\nsecond");
    // Up on the first line of the description goes back to the summary.
    form.on_key(key(KeyCode::Up));
    assert_eq!(form.focus, RewordPart::Description);
    form.on_key(key(KeyCode::Up));
    assert_eq!(form.focus, RewordPart::Summary);
    // A multi-line paste into the summary splits at its first line.
    form.on_paste("x\r\ny\r\nz");
    assert_eq!(form.summary, "summary!x");
    assert_eq!(form.focus, RewordPart::Description);
    assert!(form.description.starts_with("y\nz"));
}

#[test]
fn tab_walks_fields_and_buttons() {
    let mut form = RewordForm::new(oid(1), "s", None);
    let mut seen = vec![form.focus];
    for _ in 0..4 {
        form.on_key(key(KeyCode::Tab));
        seen.push(form.focus);
    }
    assert_eq!(
        seen,
        vec![
            RewordPart::Summary,
            RewordPart::Description,
            RewordPart::Cancel,
            RewordPart::Apply,
            RewordPart::Summary
        ]
    );
    form.on_key(key(KeyCode::BackTab));
    assert_eq!(form.focus, RewordPart::Apply);
    assert_eq!(form.on_key(key(KeyCode::Esc)), RewordFormOutcome::Close);
}

#[test]
fn apply_sends_the_whole_message_and_refuses_empty_or_unchanged() {
    let mut form = RewordForm::new(oid(1), "old\n", None);
    assert_eq!(form.on_key(ctrl(KeyCode::Enter)), RewordFormOutcome::Stay);
    assert!(form.error.as_deref().unwrap().contains("unchanged"));
    for _ in 0..3 {
        form.on_key(key(KeyCode::Backspace));
    }
    assert_eq!(
        form.on_key(ctrl(KeyCode::Char('s'))),
        RewordFormOutcome::Stay
    );
    assert!(form.error.as_deref().unwrap().contains("empty"));
    type_str(&mut form, "new");
    form.on_key(key(KeyCode::Enter));
    type_str(&mut form, "body");
    assert_eq!(
        form.on_key(ctrl(KeyCode::Enter)),
        RewordFormOutcome::Submit(Action::Reword {
            commit: oid(1),
            message: "new\n\nbody".into(),
        })
    );
    // Edited: Esc asks inline, a second Esc discards.
    assert_eq!(form.on_key(key(KeyCode::Esc)), RewordFormOutcome::Stay);
    assert_eq!(form.on_key(key(KeyCode::Esc)), RewordFormOutcome::Close);
}

/// `reword.apply = "Alt+a"`: Alt+A applies; Ctrl+Enter and Ctrl+S do not.
#[test]
fn a_rebound_apply_chord_applies() {
    let km = crate::keymap::test_keymap(&[("reword", "apply", &["Alt+a"])]);
    crate::keymap::with_test_keymap(km, || {
        let mut form = RewordForm::new(oid(1), "a\n\nb", None).embedded();
        assert_eq!(form.on_key(ctrl(KeyCode::Enter)), RewordFormOutcome::Stay);
        assert_eq!(
            form.on_key(ctrl(KeyCode::Char('s'))),
            RewordFormOutcome::Stay
        );
        assert!(matches!(
            form.on_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT)),
            RewordFormOutcome::Submit(_)
        ));
    });
}

#[test]
fn an_embedded_popup_accepts_the_text_it_opened_with() {
    let mut form = RewordForm::new(oid(1), "a\n\nb", None).embedded();
    assert!(matches!(
        form.on_key(ctrl(KeyCode::Enter)),
        RewordFormOutcome::Submit(_)
    ));
}

/// A pushed commit takes two presses of Apply, the second on a
/// button that says what it does; Esc between them goes back to
/// editing rather than closing.
#[test]
fn a_pushed_commit_asks_once_more_before_rewriting() {
    let mut form = RewordForm::new(oid(1), "old", Some("origin/main".into()));
    type_str(&mut form, " edited");
    assert_eq!(form.on_key(ctrl(KeyCode::Enter)), RewordFormOutcome::Stay);
    assert!(form.confirming);
    assert_eq!(form.apply_label(), "Rewrite anyway");
    assert_eq!(form.on_key(key(KeyCode::Esc)), RewordFormOutcome::Stay);
    assert!(!form.confirming);
    form.on_key(ctrl(KeyCode::Enter));
    assert!(matches!(
        form.on_click(RewordPart::Apply),
        RewordFormOutcome::Submit(Action::Reword { .. })
    ));
    assert!(form.submitted_message().contains("force push"));
}

fn paint(form: &RewordForm) -> (Vec<String>, Vec<(Rect, RewordPart)>) {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(70, 24)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|frame| hits = form.render(frame, frame.area(), None))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let rows = (0..24)
        .map(|y| (0..70).map(|x| buf[(x, y)].symbol().to_string()).collect())
        .collect();
    (rows, hits)
}

#[test]
fn the_popup_shows_the_message_the_warning_and_its_buttons() {
    let form = RewordForm::new(
        oid(1),
        "the summary\n\nthe body",
        Some("origin/main".into()),
    );
    let (rows, hits) = paint(&form);
    let all = rows.join("\n");
    assert!(all.contains("Reword commit message"), "{all}");
    assert!(all.contains("the summary"), "{all}");
    assert!(all.contains("the body"), "{all}");
    assert!(all.contains("force pushed (with lease)"), "{all}");
    for part in [
        RewordPart::Summary,
        RewordPart::Description,
        RewordPart::Cancel,
        RewordPart::Apply,
    ] {
        assert!(hits.iter().any(|(_, p)| *p == part), "no hit for {part:?}");
    }
    // The Apply button's cells are where its label is painted.
    let (apply, _) = hits.iter().find(|(_, p)| *p == RewordPart::Apply).unwrap();
    let painted: String = rows[apply.y as usize]
        .chars()
        .skip(apply.x as usize)
        .take(apply.width as usize)
        .collect();
    assert_eq!(painted, " Update message ");

    let unpushed = RewordForm::new(oid(1), "the summary", None);
    let (rows, _) = paint(&unpushed);
    assert!(!rows.join("\n").contains("force push"));
}

/// The counter at the summary's right counts down from
/// `SUMMARY_CHAR_LIMIT`.
#[test]
fn the_summary_counter_counts_down_and_goes_negative() {
    let short = RewordForm::new(oid(1), "short", None);
    let (rows, _) = paint(&short);
    assert!(
        rows.join("\n")
            .contains(&format!("{}", SUMMARY_CHAR_LIMIT - 5)),
        "{}",
        rows.join("\n")
    );
    let over = RewordForm::new(oid(1), &"x".repeat(SUMMARY_CHAR_LIMIT + 3), None);
    let (rows, _) = paint(&over);
    assert!(rows.join("\n").contains("-3"), "{}", rows.join("\n"));
}

/// The popup in French (I8b): the pushed warning, the field placeholders and
/// the buttons all fit in the 80-column popup, git terms on or off.
#[test]
fn the_french_popup_fits_with_git_terms_on_and_off() {
    use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    for terms in [false, true] {
        let _l = override_lang(Lang::Fr);
        let _t = override_git_terms(terms);
        let form = RewordForm::new(oid(1), "", Some("origin/main".into()));
        let mut terminal = Terminal::new(TestBackend::new(90, 24)).unwrap();
        let mut hits = Vec::new();
        terminal
            .draw(|frame| hits = form.render(frame, frame.area(), None))
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let all: String = (0..24)
            .map(|y| (0..90).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!all.contains("{$"), "{all}");
        for needle in ["Reformuler le message", "Description", "Annuler"] {
            assert!(all.contains(needle), "terms={terms}: {needle}\n{all}");
        }
        assert!(all.contains("Mettre à jour le message"), "{all}");
        assert!(all.contains("Déjà poussé vers origin/main"), "{all}");
        assert!(all.contains("Tab suivant"), "{all}");
        let buttons: Vec<Rect> = hits
            .iter()
            .filter(|(_, p)| matches!(p, RewordPart::Cancel | RewordPart::Apply))
            .map(|(r, _)| *r)
            .collect();
        assert_eq!(buttons.len(), 2);
        assert!(!buttons[0].intersects(buttons[1]));
        for r in buttons {
            assert!(r.x + r.width <= 90);
        }
    }
}

#[test]
fn the_cross_is_a_hit_and_closes_an_unchanged_popup_but_asks_when_edited() {
    let mut form = RewordForm::new(oid(1), "old", None);
    let (rows, hits) = paint(&form);
    let all = rows.join("\n");
    assert!(all.contains('\u{2715}'), "{all}");
    assert!(!all.contains("Esc"), "no Esc hint\n{all}");
    assert!(hits.iter().any(|(_, p)| *p == RewordPart::Close));
    assert_eq!(form.on_click(RewordPart::Close), RewordFormOutcome::Close);
    assert!(form.is_outside(0, 0));
    type_str(&mut form, "!");
    assert_eq!(form.on_click(RewordPart::Close), RewordFormOutcome::Stay);
    let (rows, _) = paint(&form);
    assert!(rows.join("\n").contains("Discard your changes?"));
    // Another key keeps editing and clears the question.
    form.on_key(key(KeyCode::Left));
    assert_eq!(form.on_key(key(KeyCode::Esc)), RewordFormOutcome::Stay);
    assert_eq!(form.on_key(key(KeyCode::Esc)), RewordFormOutcome::Close);
}

// -- the Amend scopes ---------------------------------------------------

fn amend_form(scope: AmendScope) -> RewordForm {
    // 2001-02-03 04:05:06 UTC.
    RewordForm::new(oid(1), "the summary\n\nthe body", None).amend(
        scope,
        "Ada Lovelace",
        "ada@example.org",
        981_173_106,
    )
}

#[test]
fn every_amend_scope_shows_all_fields_in_tab_order_and_focuses_its_own() {
    let all = amend_form(AmendScope::All);
    assert_eq!(all.focus, RewordPart::Summary);
    assert_eq!(all.author_date, "2001-02-03 04:05:06 +00:00");
    let mut seen = Vec::new();
    let mut form = all;
    // The deeper button sits before Cancel in builds with the workbench.
    let deeper = form.deeper_available();
    for _ in 0..8 + usize::from(deeper) {
        seen.push(form.focus);
        form.on_key(key(KeyCode::Tab));
    }
    let mut expected = vec![
        RewordPart::Summary,
        RewordPart::Description,
        RewordPart::AuthorName,
        RewordPart::AuthorEmail,
        RewordPart::AuthorDate,
    ];
    if deeper {
        expected.push(RewordPart::Deeper);
    }
    expected.extend([RewordPart::Cancel, RewordPart::Apply, RewordPart::Summary]);
    assert_eq!(seen, expected);
    assert_eq!(amend_form(AmendScope::Author).focus, RewordPart::AuthorName);
    assert_eq!(
        amend_form(AmendScope::Description).focus,
        RewordPart::Description
    );
    assert_eq!(amend_form(AmendScope::Summary).focus, RewordPart::Summary);
    assert_eq!(amend_form(AmendScope::Date).focus, RewordPart::AuthorDate);
    // The same complete dialog whatever the entry point.
    for scope in [
        AmendScope::Summary,
        AmendScope::Description,
        AmendScope::Author,
        AmendScope::Date,
    ] {
        let (rows, hits) = paint(&amend_form(scope));
        let text = rows.join("\n");
        assert!(
            text.contains("Amend commit") && text.contains("the summary"),
            "{scope:?}"
        );
        for part in [
            RewordPart::Description,
            RewordPart::AuthorEmail,
            RewordPart::AuthorDate,
        ] {
            assert!(hits.iter().any(|(_, p)| *p == part), "{scope:?} {part:?}");
        }
    }
}

#[test]
fn applying_sends_only_the_fields_that_changed() {
    let mut form = amend_form(AmendScope::All);
    // Nothing changed yet.
    assert_eq!(form.try_apply(), RewordFormOutcome::Stay);
    assert!(form.error.as_deref().unwrap().contains("Nothing changed"));
    // The email only.
    form.set_focus(RewordPart::AuthorEmail);
    type_str(&mut form, ".uk");
    assert_eq!(
        form.try_apply(),
        RewordFormOutcome::Submit(Action::AmendCommit {
            commit: oid(1),
            amend: Box::new(CommitAmend {
                author_email: Some("ada@example.org.uk".into()),
                ..CommitAmend::default()
            }),
        })
    );
    // The summary and the author name together.
    form.set_focus(RewordPart::Summary);
    type_str(&mut form, "!");
    form.set_focus(RewordPart::AuthorName);
    type_str(&mut form, " Jr");
    let RewordFormOutcome::Submit(Action::AmendCommit { amend, .. }) = form.try_apply() else {
        panic!("expected a submit");
    };
    assert_eq!(amend.message.as_deref(), Some("the summary!\n\nthe body"));
    assert_eq!(amend.author_name.as_deref(), Some("Ada Lovelace Jr"));
    assert_eq!(amend.author_date, None);
}

#[test]
fn a_message_only_change_keeps_the_reword_request() {
    let mut form = amend_form(AmendScope::Summary);
    type_str(&mut form, "!");
    assert_eq!(
        form.try_apply(),
        RewordFormOutcome::Submit(Action::AmendCommit {
            commit: oid(1),
            amend: Box::new(CommitAmend {
                message: Some("the summary!\n\nthe body".into()),
                ..CommitAmend::default()
            }),
        })
    );
    let mut message = RewordForm::new(oid(1), "old", None);
    type_str(&mut message, "!");
    assert!(matches!(
        message.try_apply(),
        RewordFormOutcome::Submit(Action::Reword { .. })
    ));
}

#[test]
fn a_bad_name_email_or_date_is_refused_in_place() {
    let mut form = amend_form(AmendScope::Author);
    form.author_name = "  ".into();
    assert_eq!(form.try_apply(), RewordFormOutcome::Stay);
    assert!(form.error.as_deref().unwrap().contains("name and email"));
    form.author_name = "Ada".into();
    form.author_email = "not an email".into();
    assert_eq!(form.try_apply(), RewordFormOutcome::Stay);
    assert!(form.error.as_deref().unwrap().contains("name and email"));
    form.author_email = "ada@example.org".into();
    form.author_date = "tomorrow".into();
    assert_eq!(form.try_apply(), RewordFormOutcome::Stay);
    assert!(form.error.as_deref().unwrap().contains("2025-01-31"));
    form.author_date = "2001-02-03 04:05:06 +05:30".into();
    assert!(matches!(
        form.try_apply(),
        RewordFormOutcome::Submit(Action::AmendCommit { .. })
    ));
}

#[test]
fn down_and_up_move_between_the_author_fields_and_enter_ends_on_apply() {
    let mut form = amend_form(AmendScope::Author);
    form.on_key(key(KeyCode::Down));
    assert_eq!(form.focus, RewordPart::AuthorEmail);
    form.on_key(key(KeyCode::Down));
    assert_eq!(form.focus, RewordPart::AuthorDate);
    form.on_key(key(KeyCode::Up));
    assert_eq!(form.focus, RewordPart::AuthorEmail);
    form.on_key(key(KeyCode::Enter));
    assert_eq!(form.focus, RewordPart::AuthorDate);
    form.on_key(key(KeyCode::Enter));
    assert_eq!(form.focus, RewordPart::Apply);
    // Pasting into a single-line field keeps its first line.
    form.set_focus(RewordPart::AuthorName);
    form.on_paste(" Jr\nsecond line");
    assert_eq!(form.author_name, "Ada Lovelace Jr");
}

#[test]
fn the_amend_dialog_paints_labels_prefilled_fields_and_a_hit_for_each() {
    let form = amend_form(AmendScope::All);
    let (rows, hits) = paint(&form);
    let all = rows.join("\n");
    for needle in [
        "Amend commit",
        "the summary",
        "the body",
        "Author name",
        "Ada Lovelace",
        "Author email",
        "ada@example.org",
        "Author date",
        "2001-02-03 04:05:06 +00:00",
        "Apply",
        "Cancel",
    ] {
        assert!(all.contains(needle), "{needle}\n{all}");
    }
    for part in [
        RewordPart::Summary,
        RewordPart::Description,
        RewordPart::AuthorName,
        RewordPart::AuthorEmail,
        RewordPart::AuthorDate,
        RewordPart::Cancel,
        RewordPart::Apply,
        RewordPart::Close,
    ] {
        assert!(hits.iter().any(|(_, p)| *p == part), "no hit for {part:?}");
    }
}

#[test]
fn the_form_moves_resizes_and_arranges() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let form = RewordForm::new(oid(1), "summary\n\nbody", None);
    crate::dialog_kit::frame::testing::check_gestures(true, || {
        let mut terminal = Terminal::new(TestBackend::new(160, 50)).unwrap();
        terminal
            .draw(|frame| drop(form.render(frame, frame.area(), None)))
            .unwrap();
    });
}
