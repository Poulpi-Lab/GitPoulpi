use super::*;
use gitpoulpi_core::domain::{
    DiffLine, FileStatus, HunkId, Oid, SplitFile, SplitSource, WholeReason,
};

fn line(origin: LineOrigin) -> DiffLine {
    DiffLine {
        origin,
        content: "x\n".into(),
        staged: false,
    }
}

/// Indices: 0 context, 1 add, 2 add, 3 delete, 4 context.
fn hunk(id: u32) -> Hunk {
    use LineOrigin::{Add, Context, Delete};
    Hunk {
        id: HunkId(id),
        old_start: 1,
        old_lines: 3,
        new_start: 1,
        new_lines: 4,
        lines: vec![
            line(Context),
            line(Add),
            line(Add),
            line(Delete),
            line(Context),
        ],
    }
}

/// `src/a.rs` (two hunks), `src/b.rs` (one hunk), `logo.png` (binary).
fn source() -> SplitSource {
    SplitSource {
        commit: Oid([3; 20]),
        parent: Some(Oid([2; 20])),
        message: "feat: parser and logo".into(),
        sign_default: false,
        files: vec![
            SplitFile {
                path: "src/a.rs".into(),
                status: FileStatus::Modified,
                whole_only: None,
                hunks: vec![hunk(0), hunk(1)],
            },
            SplitFile {
                path: "src/b.rs".into(),
                status: FileStatus::Modified,
                whole_only: None,
                hunks: vec![hunk(0)],
            },
            SplitFile {
                path: "logo.png".into(),
                status: FileStatus::Added,
                whole_only: Some(WholeReason::Binary),
                hunks: vec![],
            },
        ],
    }
}

fn form() -> SplitForm {
    SplitForm::new(source())
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ch(c: char) -> KeyEvent {
    key(KeyCode::Char(c))
}

fn piece_hunk(path: &str, hunk: u32) -> SplitPiece {
    SplitPiece::Hunk {
        path: path.into(),
        hunk: HunkId(hunk),
    }
}

fn piece_lines(path: &str, hunk: u32, lines: &[u32]) -> SplitPiece {
    SplitPiece::Lines {
        path: path.into(),
        hunk: HunkId(hunk),
        lines: lines.to_vec(),
    }
}

fn piece_file(path: &str) -> SplitPiece {
    SplitPiece::File { path: path.into() }
}

/// Moves the changes cursor onto `row`.
fn at(f: &mut SplitForm, row: SplitRow) {
    f.select_row(row);
    assert_eq!(f.cursor_row(), Some(row));
}

fn name_second_part(f: &mut SplitForm) {
    f.parts[1].message = "feat: second".into();
}

#[test]
fn it_starts_with_everything_in_the_first_part_and_the_second_empty() {
    let f = form();
    assert_eq!(f.part_count(), 2);
    assert_eq!(f.part_message(0), "feat: parser and logo");
    assert_eq!(f.part_message(1), "");
    let parts = f.build_parts();
    assert_eq!(
        parts[0].pieces,
        [
            piece_hunk("src/a.rs", 0),
            piece_hunk("src/a.rs", 1),
            piece_hunk("src/b.rs", 0),
            piece_file("logo.png"),
        ]
    );
    assert!(parts[1].pieces.is_empty());
    assert_eq!(f.validation(), Err(SplitRefusal::EmptyPart));
}

#[test]
fn a_whole_only_file_moves_as_a_file_piece() {
    let mut f = form();
    name_second_part(&mut f);
    at(&mut f, SplitRow::File(2));
    f.assign_selection(Some(1));
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Part(1));
    assert_eq!(f.build_parts()[1].pieces, [piece_file("logo.png")]);
    assert_eq!(f.validation(), Ok(()));
    assert_eq!(f.unassigned_count(), 0);
}

#[test]
fn a_hunk_moves_between_parts_and_back() {
    let mut f = form();
    name_second_part(&mut f);
    at(&mut f, SplitRow::Hunk(0, 1));
    f.assign_selection(Some(1));
    assert_eq!(f.row_owner(SplitRow::Hunk(0, 1)), RowOwner::Part(1));
    assert_eq!(f.row_owner(SplitRow::Hunk(0, 0)), RowOwner::Part(0));
    // The file now has hunks in two parts.
    assert_eq!(f.row_owner(SplitRow::File(0)), RowOwner::Mixed);
    assert_eq!(f.build_parts()[1].pieces, [piece_hunk("src/a.rs", 1)]);
    assert_eq!(f.validation(), Ok(()));
    // Back to part 1: part 2 is empty again.
    f.assign_selection(Some(0));
    assert_eq!(f.validation(), Err(SplitRefusal::EmptyPart));
}

#[test]
fn a_whole_text_file_assigns_every_hunk_and_never_double_books_one() {
    let mut f = form();
    name_second_part(&mut f);
    // Hunk 0 of a.rs goes to part 2 first, then the whole file goes to
    // part 1: the hunk is taken over, not assigned twice.
    at(&mut f, SplitRow::Hunk(0, 0));
    f.assign_selection(Some(1));
    at(&mut f, SplitRow::File(0));
    f.assign_selection(Some(0));
    assert_eq!(f.row_owner(SplitRow::File(0)), RowOwner::Part(0));
    assert_eq!(f.row_owner(SplitRow::Hunk(0, 0)), RowOwner::Part(0));
    assert_ne!(f.validation(), Err(SplitRefusal::AssignedTwice));
    // Giving the file to part 2 builds hunk pieces, never File + Hunk.
    f.assign_selection(Some(1));
    assert_eq!(
        f.build_parts()[1].pieces,
        [piece_hunk("src/a.rs", 0), piece_hunk("src/a.rs", 1)]
    );
    assert_eq!(f.validation(), Ok(()));
}

#[test]
fn a_plan_naming_a_file_and_one_of_its_hunks_keeps_the_first_owner() {
    let parts = vec![
        SplitPart {
            message: "one".into(),
            pieces: vec![piece_file("src/a.rs"), piece_file("logo.png")],
        },
        SplitPart {
            message: "two".into(),
            pieces: vec![piece_hunk("src/a.rs", 0), piece_hunk("src/b.rs", 0)],
        },
    ];
    // As typed, the plan is refused by the shared check...
    assert_eq!(
        validate_split(&source(), &parts),
        Err(SplitRefusal::AssignedTwice)
    );
    // ...the editor cannot hold it, so hunk 0 stays with the file's part.
    let f = SplitForm::with_parts(source(), parts);
    assert_eq!(f.row_owner(SplitRow::Hunk(0, 0)), RowOwner::Part(0));
    assert_eq!(f.row_owner(SplitRow::Hunk(1, 0)), RowOwner::Part(1));
    assert_eq!(f.validation(), Ok(()));
}

#[test]
fn lines_of_a_hunk_split_over_two_parts() {
    let mut f = form();
    name_second_part(&mut f);
    at(&mut f, SplitRow::Hunk(0, 0));
    f.toggle_line_mode();
    assert!(f.line_mode());
    // Line mode lands on the hunk's first changed line (index 1).
    assert_eq!(f.cursor_row(), Some(SplitRow::Line(0, 0, 1)));
    // Shift+Down extends over the next changed line (index 2).
    f.on_key(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT));
    assert_eq!(
        f.selection(),
        [SplitRow::Line(0, 0, 1), SplitRow::Line(0, 0, 2)]
    );
    f.on_key(ch('2'));
    let parts = f.build_parts();
    assert_eq!(parts[1].pieces, [piece_lines("src/a.rs", 0, &[1, 2])]);
    // The rest of the hunk (the delete) stays in part 1 as lines.
    assert!(parts[0].pieces.contains(&piece_lines("src/a.rs", 0, &[3])));
    assert_eq!(f.row_owner(SplitRow::Hunk(0, 0)), RowOwner::Mixed);
    assert_eq!(f.validation(), Ok(()));
    // A plain move drops the range.
    f.on_key(key(KeyCode::Down));
    assert_eq!(f.selection(), [SplitRow::Line(0, 0, 3)]);
}

#[test]
fn line_mode_toggles_back_to_the_hunk_and_ignores_files() {
    let mut f = form();
    at(&mut f, SplitRow::File(0));
    f.toggle_line_mode();
    assert!(!f.line_mode(), "a file row has no lines to list");
    at(&mut f, SplitRow::File(2));
    f.toggle_line_mode();
    assert!(!f.line_mode(), "a whole-only file has no lines");
    at(&mut f, SplitRow::Hunk(1, 0));
    f.toggle_line_mode();
    assert!(f.line_mode());
    f.toggle_line_mode();
    assert!(!f.line_mode());
    assert_eq!(f.cursor_row(), Some(SplitRow::Hunk(1, 0)));
    assert!(f.rows().iter().all(|r| !matches!(r, SplitRow::Line(..))));
}

#[test]
fn a_new_part_takes_the_selection_and_unassign_gives_it_back() {
    let mut f = form();
    at(&mut f, SplitRow::Hunk(1, 0));
    f.new_part_from_selection();
    assert_eq!(f.part_count(), 3);
    assert_eq!(f.row_owner(SplitRow::Hunk(1, 0)), RowOwner::Part(2));
    assert_eq!(f.selected_part(), 2);
    at(&mut f, SplitRow::Hunk(1, 0));
    f.assign_selection(None);
    assert_eq!(f.row_owner(SplitRow::Hunk(1, 0)), RowOwner::Unassigned);
    assert_eq!(f.unassigned_count(), 1);
}

#[test]
fn removing_a_part_unassigns_its_changes_and_renumbers_the_rest() {
    let mut f = form();
    f.add_part(); // three parts
    at(&mut f, SplitRow::Hunk(1, 0));
    f.assign_selection(Some(2));
    at(&mut f, SplitRow::File(2));
    f.assign_selection(Some(1));
    f.remove_part(1);
    assert_eq!(f.part_count(), 2);
    // The logo was part 2's: unassigned now. The b.rs hunk was part 3's:
    // part 2 now.
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Unassigned);
    assert_eq!(f.row_owner(SplitRow::Hunk(1, 0)), RowOwner::Part(1));
    assert_eq!(f.unassigned_count(), 1);
    assert_eq!(f.validation(), Err(SplitRefusal::EmptyMessage));
}

#[test]
fn a_split_always_keeps_two_parts() {
    let mut f = form();
    f.remove_part(0);
    assert_eq!(f.part_count(), 2);
    assert_eq!(
        f.status_text().as_deref(),
        Some(refusal_text(SplitRefusal::TooFewParts).as_str())
    );
}

#[test]
fn reordering_parts_carries_their_changes() {
    let mut f = form();
    name_second_part(&mut f);
    at(&mut f, SplitRow::File(2));
    f.assign_selection(Some(1));
    f.move_part(1, -1);
    assert_eq!(f.part_message(0), "feat: second");
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Part(0));
    assert_eq!(f.row_owner(SplitRow::Hunk(0, 0)), RowOwner::Part(1));
    assert_eq!(f.selected_part(), 0);
    // Off the ends: nothing.
    f.move_part(0, -1);
    assert_eq!(f.part_message(0), "feat: second");
}

#[test]
fn validation_names_the_first_refusal_in_words() {
    let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::En);
    let mut f = form();
    assert_eq!(
        f.validation_text().as_deref(),
        Some("a new commit has no changes assigned to it")
    );
    at(&mut f, SplitRow::File(2));
    f.assign_selection(Some(1));
    assert_eq!(
        f.validation_text().as_deref(),
        Some("a new commit has an empty message")
    );
    name_second_part(&mut f);
    at(&mut f, SplitRow::Hunk(0, 0));
    f.assign_selection(None);
    assert_eq!(
        f.validation_text().as_deref(),
        Some("some changes are not assigned to any commit")
    );
    assert_eq!(f.unassigned_count(), 1);
    f.assign_selection(Some(0));
    assert_eq!(f.validation_text(), None);
}

#[test]
fn the_french_words_follow_the_git_terms_setting() {
    let _l = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let mut f = form();
    {
        let _t = gitpoulpi_i18n::override_git_terms(false);
        assert_eq!(
            f.validation_text().as_deref(),
            Some("au moins un nouveau commit est vide")
        );
    }
    let _t = gitpoulpi_i18n::override_git_terms(true);
    assert_eq!(
        f.validation_text().as_deref(),
        Some("au moins une nouvelle validation est vide")
    );
    f.add_part();
}

#[test]
fn reset_restores_the_opening_plan() {
    let mut f = form();
    name_second_part(&mut f);
    f.add_part();
    at(&mut f, SplitRow::File(2));
    f.assign_selection(Some(2));
    f.reset();
    assert_eq!(f.part_count(), 2);
    assert_eq!(f.part_message(1), "");
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Part(0));
}

#[test]
fn keys_assign_unassign_and_create_parts() {
    let mut f = form();
    // Changes pane: down to the logo (rows: a.rs, h0, h1, b.rs, h0, logo).
    for _ in 0..5 {
        f.on_key(key(KeyCode::Down));
    }
    assert_eq!(f.cursor_row(), Some(SplitRow::File(2)));
    f.on_key(ch('2'));
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Part(1));
    f.on_key(ch('0'));
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Unassigned);
    // A part that does not exist changes nothing.
    f.on_key(ch('7'));
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Unassigned);
    f.on_key(ch('n'));
    assert_eq!(f.part_count(), 3);
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Part(2));
}

#[test]
fn tab_switches_panes_and_the_parts_pane_has_its_own_keys() {
    let mut f = form();
    assert_eq!(f.pane(), SplitPane::Changes);
    f.on_key(key(KeyCode::Tab));
    assert_eq!(f.pane(), SplitPane::Parts);
    f.on_key(ch('n'));
    assert_eq!(f.part_count(), 3);
    assert_eq!(f.selected_part(), 2);
    f.on_key(ch('k'));
    assert_eq!(f.selected_part(), 1);
    // Reorder with the default Shift+K, delete with d.
    f.on_key(ch('K'));
    assert_eq!(f.selected_part(), 0);
    f.on_key(ch('d'));
    assert_eq!(f.part_count(), 2);
    f.on_key(ch('s'));
    assert!(f.sign());
}

#[test]
fn the_message_editor_sets_a_part_s_message() {
    let mut f = form();
    f.on_key(key(KeyCode::Tab));
    f.on_key(ch('j'));
    f.on_key(ch('e'));
    assert!(f.is_editing());
    for c in "fix: two".chars() {
        f.on_key(ch(c));
    }
    f.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL));
    assert!(!f.is_editing());
    assert_eq!(f.part_message(1).trim(), "fix: two");
}

#[test]
fn apply_submits_only_a_valid_plan() {
    let mut f = form();
    // Not valid yet: the reason is shown and nothing is submitted.
    let out = f.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert_eq!(out, SplitFormOutcome::Stay);
    assert_eq!(
        f.status_text().as_deref(),
        Some(refusal_text(SplitRefusal::EmptyPart).as_str())
    );
    name_second_part(&mut f);
    at(&mut f, SplitRow::File(2));
    f.assign_selection(Some(1));
    f.set_sign(true);
    match f.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)) {
        SplitFormOutcome::Submit(Action::SplitCommit { req }) => {
            assert_eq!(req.commit, Oid([3; 20]));
            assert!(req.sign);
            assert_eq!(req.parts.len(), 2);
            assert_eq!(req.parts[1].pieces, [piece_file("logo.png")]);
            assert_eq!(validate_split(&source(), &req.parts), Ok(()));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(f.on_key(key(KeyCode::Esc)), SplitFormOutcome::Close);
}

/// The bindable keys follow the keymap, and a rebound chord replaces the default.
#[test]
fn a_rebound_line_mode_key_toggles_line_mode() {
    let km = crate::keymap::test_keymap(&[("split", "line_mode", &["w"])]);
    crate::keymap::with_test_keymap(km, || {
        let mut f = form();
        at(&mut f, SplitRow::Hunk(0, 0));
        f.on_key(ch('v'));
        assert!(!f.line_mode(), "the default chord is gone");
        f.on_key(ch('w'));
        assert!(f.line_mode());
    });
}

// ---- the paint walk and the mouse (plan slice S7) -------------------------------

use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn paint(
    f: &SplitForm,
    w: u16,
    h: u16,
    hovered: Option<SplitHit>,
) -> (String, Vec<(Rect, SplitHit)>) {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|frame| hits = f.render(frame, frame.area(), hovered))
        .unwrap();
    let buf = terminal.backend().buffer();
    let text = (0..h)
        .map(|y| {
            (0..w)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    (text, hits)
}

/// The changes list outgrows its room: the shared bar rides the editor's
/// right border, a drag on the thumb scrolls the window and the selection
/// stays; a short list has none.
#[test]
fn a_long_changes_list_has_a_scrollbar_the_thumb_drags() {
    use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    let mut source = source();
    source.files = (0..40)
        .map(|i| SplitFile {
            path: format!("src/f{i:02}.rs").into(),
            status: FileStatus::Modified,
            whole_only: None,
            hunks: vec![hunk(0)],
        })
        .collect();
    let f = SplitForm::new(source);
    let (text, _) = paint(&f, 140, 30, None);
    let track = f.view.bar.get().expect("a bar");
    let ev = |kind, row| MouseEvent {
        kind,
        column: track.rect.x,
        row,
        modifiers: KeyModifiers::NONE,
    };
    assert!(f.changes_bar_mouse(ev(
        MouseEventKind::Down(MouseButton::Left),
        track.rect.y + 1
    )));
    assert!(f.changes_bar_mouse(ev(
        MouseEventKind::Drag(MouseButton::Left),
        track.rect.bottom() + 9
    )));
    f.changes_bar_mouse(ev(MouseEventKind::Up(MouseButton::Left), 0));
    let (after, _) = paint(&f, 140, 30, None);
    assert_ne!(text, after, "the list scrolled");
    assert!(after.contains("f39"), "{after}");
    let short = form();
    paint(&short, 140, 30, None);
    assert!(short.view.bar.get().is_none());
}

fn hit_rect(hits: &[(Rect, SplitHit)], hit: SplitHit) -> Rect {
    hits.iter()
        .find(|(_, h)| *h == hit)
        .unwrap_or_else(|| panic!("no hit {hit:?} in {hits:?}"))
        .0
}

fn press(f: &mut SplitForm, hits: &[(Rect, SplitHit)], hit: SplitHit) -> SplitFormOutcome {
    let r = hit_rect(hits, hit);
    f.on_press(hit, r.x, r.y)
}

#[test]
fn the_editor_draws_its_two_panes_and_buttons() {
    let mut f = form();
    name_second_part(&mut f);
    at(&mut f, SplitRow::File(2));
    f.assign_selection(Some(1));
    let (text, _) = paint(&f, 110, 24, None);
    insta::assert_snapshot!("editor", text);
}

#[test]
fn line_mode_lists_the_lines_of_the_open_hunk() {
    let mut f = form();
    at(&mut f, SplitRow::Hunk(0, 0));
    f.toggle_line_mode();
    f.assign_selection(Some(1));
    let (text, _) = paint(&f, 110, 24, None);
    assert!(text.contains("line mode"), "{text}");
    insta::assert_snapshot!("editor_line_mode", text);
}

#[test]
fn the_frame_carries_a_close_cross_and_no_esc_hint_on_its_border() {
    let (text, hits) = paint(&form(), 110, 24, None);
    let top = text.lines().next().unwrap();
    assert!(top.contains('\u{2715}'), "{top}");
    assert!(!top.contains("Esc"), "{top}");
    let cross = hit_rect(&hits, SplitHit::Close);
    assert_eq!((cross.y, cross.width), (0, 3));
}

#[test]
fn every_row_has_an_assignment_field_and_every_button_a_rect() {
    let f = form();
    let (_, hits) = paint(&f, 110, 24, None);
    for row in f.rows() {
        hit_rect(&hits, SplitHit::Assign(row));
        hit_rect(&hits, SplitHit::Row(row));
    }
    for hit in [
        SplitHit::Apply,
        SplitHit::Cancel,
        SplitHit::Reset,
        SplitHit::Sign,
        SplitHit::NewPart,
        SplitHit::Part(0),
        SplitHit::Part(1),
    ] {
        hit_rect(&hits, hit);
    }
    // Inner first: a row's field is registered before the band behind it.
    let at_of = |hit| hits.iter().position(|(_, h)| *h == hit).unwrap();
    assert!(at_of(SplitHit::Assign(SplitRow::File(0))) < at_of(SplitHit::Row(SplitRow::File(0))));
}

#[test]
fn clicking_a_row_selects_it_and_a_part_selects_the_part() {
    let mut f = form();
    let (_, hits) = paint(&f, 110, 24, None);
    press(&mut f, &hits, SplitHit::Row(SplitRow::Hunk(1, 0)));
    assert_eq!(f.cursor_row(), Some(SplitRow::Hunk(1, 0)));
    assert_eq!(f.pane(), SplitPane::Changes);
    press(&mut f, &hits, SplitHit::Part(1));
    assert_eq!((f.pane(), f.selected_part()), (SplitPane::Parts, 1));
}

#[test]
fn a_double_click_on_a_part_edits_its_message() {
    let mut f = form();
    let (_, hits) = paint(&f, 110, 24, None);
    press(&mut f, &hits, SplitHit::Part(1));
    assert!(!f.is_editing());
    press(&mut f, &hits, SplitHit::Part(1));
    assert!(f.is_editing(), "the second click of a pair edits");
}

#[test]
fn the_assignment_field_opens_a_list_and_a_click_on_an_item_assigns() {
    let mut f = form();
    name_second_part(&mut f);
    let (_, hits) = paint(&f, 110, 24, None);
    press(&mut f, &hits, SplitHit::Assign(SplitRow::File(2)));
    assert!(f.assign_combo_open());
    // The popup is painted over everything and is the only target.
    let (text, hits) = paint(&f, 110, 24, None);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].1, SplitHit::Modal);
    assert!(text.contains("2 feat: second"), "{text}");
    let item = {
        let combo = f.view.combo.borrow();
        // Unassigned, 1, 2, + New commit: the third row is commit 2.
        combo.as_ref().unwrap().item_rects()[2].0
    };
    f.on_combo_mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: item.x + 1,
        row: item.y,
        modifiers: KeyModifiers::NONE,
    });
    assert!(!f.assign_combo_open());
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Part(1));
}

#[test]
fn the_assignment_list_has_a_keyboard_twin_and_can_make_a_new_commit() {
    let mut f = form();
    at(&mut f, SplitRow::File(2));
    f.on_key(key(KeyCode::Enter));
    assert!(f.assign_combo_open());
    // Escape closes the list and nothing changes.
    f.on_key(key(KeyCode::Esc));
    assert!(!f.assign_combo_open());
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Part(0));
    f.on_key(ch('m'));
    // The list opens on commit 1 (its owner): 2, + New commit.
    f.on_key(key(KeyCode::Down));
    f.on_key(key(KeyCode::Down));
    f.on_key(key(KeyCode::Enter));
    assert_eq!(f.part_count(), 3);
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Part(2));
}

#[test]
fn an_outside_click_closes_a_pristine_plan_and_is_swallowed_once_edited() {
    use crate::dialog_kit::DismissKind;
    let mut f = form();
    let _ = paint(&f, 110, 24, None);
    assert_eq!(
        f.request_dismiss(DismissKind::Outside),
        SplitFormOutcome::Close
    );
    at(&mut f, SplitRow::File(2));
    f.assign_selection(Some(1));
    assert!(f.is_dirty());
    assert_eq!(
        f.request_dismiss(DismissKind::Outside),
        SplitFormOutcome::Stay
    );
    // The cross and Cancel still close an edited plan.
    assert_eq!(
        f.request_dismiss(DismissKind::Cross),
        SplitFormOutcome::Close
    );
    assert_eq!(
        f.request_dismiss(DismissKind::Cancel),
        SplitFormOutcome::Close
    );
    // Undoing the edits makes it pristine again.
    f.reset();
    assert!(!f.is_dirty());
}

#[test]
fn a_swallowed_outside_click_makes_the_cross_flash() {
    use crate::dialog_kit::DismissKind;
    let mut f = form();
    at(&mut f, SplitRow::File(2));
    f.assign_selection(Some(1));
    let (_, hits) = paint(&f, 110, 24, None);
    let cross = hit_rect(&hits, SplitHit::Close);
    let style_at = |f: &SplitForm| {
        let mut t = Terminal::new(TestBackend::new(110, 24)).unwrap();
        t.draw(|frame| {
            f.render(frame, frame.area(), None);
        })
        .unwrap();
        t.backend().buffer()[(cross.x + 1, cross.y)].style()
    };
    let calm = style_at(&f);
    f.request_dismiss(DismissKind::Outside);
    assert_ne!(calm, style_at(&f), "the cross flashes");
    assert_eq!(calm, style_at(&f), "and only once");
}

#[test]
fn a_drag_over_lines_extends_the_selection_and_one_key_assigns_the_range() {
    let mut f = form();
    at(&mut f, SplitRow::Hunk(0, 0));
    f.toggle_line_mode();
    let first = f.cursor_row().unwrap();
    let SplitRow::Line(fi, hi, _) = first else {
        panic!("{first:?}")
    };
    f.extend_selection_to(SplitRow::Line(fi, hi, 2));
    assert_eq!(f.selection().len(), 2, "lines 1 and 2");
    f.assign_selection(Some(1));
    assert_eq!(
        f.build_parts()[1].pieces,
        [piece_lines("src/a.rs", 0, &[1, 2])]
    );
}

#[test]
fn a_refusal_keeps_the_editor_and_the_plan() {
    let mut f = form();
    name_second_part(&mut f);
    at(&mut f, SplitRow::File(2));
    f.assign_selection(Some(1));
    let SplitFormOutcome::Submit(_) = f.try_apply() else {
        panic!("a valid plan submits")
    };
    f.begin_applying();
    assert!(f.is_applying());
    // Keys wait for the answer (only Esc leaves).
    assert_eq!(f.on_key(ch('n')), SplitFormOutcome::Stay);
    assert_eq!(f.part_count(), 2);
    f.refused("that file can only be assigned as a whole".into());
    assert!(!f.is_applying());
    assert_eq!(f.row_owner(SplitRow::File(2)), RowOwner::Part(1));
    let (text, _) = paint(&f, 130, 24, None);
    assert!(text.contains("can only be assigned as a whole"), "{text}");
    // The same plan can be sent again.
    assert!(matches!(f.try_apply(), SplitFormOutcome::Submit(_)));
}

#[test]
fn the_wheel_scrolls_the_changes_without_moving_the_cursor() {
    let mut f = form();
    let _ = paint(&f, 110, 14, None);
    let cursor = f.cursor_row();
    f.scroll_changes(3);
    let _ = paint(&f, 110, 14, None);
    assert_eq!(f.cursor_row(), cursor);
}

#[test]
fn too_small_to_draw_registers_only_the_cross() {
    let (_, hits) = paint(&form(), 30, 8, None);
    assert!(hits.iter().all(|(_, h)| *h == SplitHit::Close), "{hits:?}");
}
