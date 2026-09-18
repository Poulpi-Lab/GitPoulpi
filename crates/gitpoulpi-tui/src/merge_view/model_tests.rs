use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gitpoulpi_core::domain::{
    compose_resolution, ConflictEntry, ConflictPicks, ConflictSide, Eol, MergeDocument,
    MergeRegion, Oid, StageEntry, WorktreeFileState,
};

use super::*;

use ConflictSide::{Base, Ours, Theirs};

fn lines(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

fn merged(xs: &[&str]) -> MergeRegion {
    MergeRegion::Merged { lines: lines(xs) }
}

fn conflict(base: &[&str], ours: &[&str], theirs: &[&str]) -> MergeRegion {
    MergeRegion::Conflict {
        base: lines(base),
        ours: lines(ours),
        theirs: lines(theirs),
    }
}

fn stage(n: u8) -> Option<StageEntry> {
    Some(StageEntry {
        mode: 0o100644,
        oid: Oid([n; 20]),
    })
}

fn merge_doc(regions: Vec<MergeRegion>) -> MergeDocument {
    MergeDocument {
        entry: ConflictEntry::from_stages("f.txt".into(), stage(1), stage(2), stage(3)).unwrap(),
        base: None,
        ours: None,
        theirs: None,
        regions,
        labels: Default::default(),
        worktree: WorktreeFileState::Markers,
        eol: Eol::Lf,
        bom: false,
        final_newline: true,
        encoding: None,
        marker_size: 7,
    }
}

/// head, x / conflict 1 / mid / conflict 2 / tail
fn two_conflicts() -> MergeDocument {
    merge_doc(vec![
        merged(&["head", "x"]),
        conflict(&["b"], &["a1", "a2"], &["t1", "t2", "t3"]),
        merged(&["mid"]),
        conflict(&[], &["o"], &["t"]),
        merged(&["tail"]),
    ])
}

fn model() -> MergeModel {
    MergeModel::new(two_conflicts())
}

fn key(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)
}

/// Types `s` at byte `at` of the Output through the model.
fn type_at(m: &mut MergeModel, at: usize, s: &str) {
    m.edit(|d| {
        d.set_cursor(at, false);
        for c in s.chars() {
            let k = if c == '\n' {
                KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
            } else {
                key(c)
            };
            d.on_key(&k, 10);
        }
    });
}

fn start_of(m: &MergeModel, c: usize) -> usize {
    m.states[c].start
}

#[test]
fn a_fresh_model_is_the_merge_with_every_conflict_empty_and_unpicked() {
    let m = model();
    assert_eq!(m.output_text(), "head\nx\nmid\ntail\n");
    assert_eq!(m.conflict_count(), 2);
    assert_eq!(m.status(0), Some(ConflictStatus::Unpicked));
    assert_eq!(m.status(1), Some(ConflictStatus::Unpicked));
    assert_eq!(m.tally(), (2, 0, 0));
    assert!(!m.is_dirty());
    assert!(!m.can_undo());
    let none = [ConflictPicks::none(), ConflictPicks::none()];
    assert_eq!(m.output_text(), compose_resolution(&two_conflicts(), &none));
}

#[test]
fn every_whole_conflict_choice_matches_the_core_composition() {
    let combos: [&[ConflictSide]; 8] = [
        &[Ours],
        &[Theirs],
        &[Base],
        &[Ours, Theirs],
        &[Theirs, Ours],
        &[Ours, Base],
        &[Base, Theirs, Ours],
        &[],
    ];
    let doc = two_conflicts();
    for first in combos {
        for second in combos {
            let mut m = model();
            for (c, sides) in [(0, first), (1, second)] {
                if sides.is_empty() {
                    m.clear(c);
                } else {
                    m.take(c, sides);
                }
            }
            let picks: Vec<ConflictPicks> = doc
                .conflicts()
                .zip([first, second])
                .map(|(r, sides)| ConflictPicks::take(r, sides))
                .collect();
            assert_eq!(
                m.output_text(),
                compose_resolution(&doc, &picks),
                "{first:?} / {second:?}"
            );
            assert_eq!(m.status(0), Some(ConflictStatus::Picked));
            assert_eq!(m.tally(), (0, 2, 0));
        }
    }
}

#[test]
fn the_named_takes_are_a_b_ab_ba_and_base() {
    let mut m = model();
    m.take_a(0);
    assert_eq!(m.region_text(0), "a1\na2\n");
    m.take_b(0);
    assert_eq!(m.region_text(0), "t1\nt2\nt3\n");
    m.take_a_then_b(0);
    assert_eq!(m.region_text(0), "a1\na2\nt1\nt2\nt3\n");
    m.take_b_then_a(0);
    assert_eq!(m.region_text(0), "t1\nt2\nt3\na1\na2\n");
    m.take_base(0);
    assert_eq!(m.region_text(0), "b\n");
    m.clear(0);
    assert_eq!(m.region_text(0), "");
    assert_eq!(
        m.status(0),
        Some(ConflictStatus::Picked),
        "none is a decision"
    );
}

#[test]
fn the_order_is_the_order_the_sides_were_picked_in() {
    let mut m = model();
    assert!(m.toggle_chunk(0, Theirs));
    assert!(m.toggle_chunk(0, Ours));
    assert_eq!(m.region_text(0), "t1\nt2\nt3\na1\na2\n");
    assert_eq!(m.order_badge(0, Theirs), Some(1));
    assert_eq!(m.order_badge(0, Ours), Some(2));
    assert_eq!(m.chunk_pick(0, Ours), ChunkPick::All);

    let mut m = model();
    m.toggle_chunk(0, Ours);
    m.toggle_chunk(0, Theirs);
    assert_eq!(m.region_text(0), "a1\na2\nt1\nt2\nt3\n");
    assert_eq!(m.order_badge(0, Ours), Some(1));
}

#[test]
fn swapping_the_order_reverses_it_and_needs_two_sides() {
    let mut m = model();
    assert!(!m.swap_order(0), "nothing picked");
    m.take_a(0);
    assert!(!m.swap_order(0), "one side only");
    assert_eq!(m.order_badge(0, Ours), None, "no badge for a single side");
    m.take_a_then_b(0);
    assert!(m.swap_order(0));
    assert_eq!(m.region_text(0), "t1\nt2\nt3\na1\na2\n");
    assert!(m.swap_order(0));
    assert_eq!(m.region_text(0), "a1\na2\nt1\nt2\nt3\n");
    // Swapping is one undo step.
    m.swap_order(0);
    assert!(m.undo());
    assert_eq!(m.region_text(0), "a1\na2\nt1\nt2\nt3\n");
}

#[test]
fn single_lines_toggle_and_keep_their_file_order_within_a_side() {
    let mut m = model();
    assert!(m.toggle_line(0, Theirs, 2));
    assert!(m.toggle_line(0, Theirs, 0));
    assert_eq!(m.region_text(0), "t1\nt3\n", "file order inside a side");
    assert_eq!(m.chunk_pick(0, Theirs), ChunkPick::Some);
    assert!(m.line_picked(0, Theirs, 2));
    assert!(!m.line_picked(0, Theirs, 1));
    assert!(m.toggle_line(0, Ours, 1));
    assert_eq!(
        m.region_text(0),
        "t1\nt3\na2\n",
        "sides in the order first picked"
    );
    // Dropping a side's last line drops it from the order.
    assert!(m.toggle_line(0, Theirs, 0));
    assert!(m.toggle_line(0, Theirs, 2));
    assert_eq!(m.region_text(0), "a2\n");
    assert_eq!(m.order_badge(0, Ours), None);
    assert_eq!(m.chunk_pick(0, Theirs), ChunkPick::None);
    // A side picked again goes last.
    assert!(m.toggle_line(0, Theirs, 1));
    assert_eq!(m.region_text(0), "a2\nt2\n");
    // Out of range lines are refused.
    assert!(!m.toggle_line(0, Theirs, 3));
    assert!(!m.toggle_line(5, Theirs, 0));
}

#[test]
fn toggling_the_last_picked_line_off_makes_the_conflict_unpicked_again() {
    let mut m = model();
    m.toggle_line(1, Ours, 0);
    assert_eq!(m.status(1), Some(ConflictStatus::Picked));
    m.toggle_line(1, Ours, 0);
    assert_eq!(m.status(1), Some(ConflictStatus::Unpicked));
    assert!(!m.is_dirty() || m.output().text() == "head\nx\nmid\ntail\n");
}

#[test]
fn a_chunk_toggle_fills_a_partial_side_then_clears_it() {
    let mut m = model();
    m.toggle_line(0, Theirs, 1);
    assert_eq!(m.chunk_pick(0, Theirs), ChunkPick::Some);
    m.toggle_chunk(0, Theirs);
    assert_eq!(m.chunk_pick(0, Theirs), ChunkPick::All);
    assert_eq!(m.region_text(0), "t1\nt2\nt3\n");
    m.toggle_chunk(0, Theirs);
    assert_eq!(m.chunk_pick(0, Theirs), ChunkPick::None);
    assert_eq!(m.region_text(0), "");
    // An empty side has nothing to toggle.
    let mut m = MergeModel::new(merge_doc(vec![conflict(&[], &[], &["t"])]));
    assert!(!m.toggle_chunk(0, Ours));
    assert!(m.toggle_chunk(0, Theirs));
}

#[test]
fn a_pick_moves_the_anchors_of_the_conflicts_after_it() {
    let mut m = model();
    m.take_b(1);
    assert_eq!(m.output_text(), "head\nx\nmid\nt\ntail\n");
    m.take_b(0);
    assert_eq!(m.output_text(), "head\nx\nt1\nt2\nt3\nmid\nt\ntail\n");
    assert_eq!(m.region_text(0), "t1\nt2\nt3\n");
    assert_eq!(m.region_text(1), "t\n");
    m.take_a(0);
    assert_eq!(m.output_text(), "head\nx\na1\na2\nmid\nt\ntail\n");
    assert_eq!(m.region_text(1), "t\n");
    assert_eq!(m.region_lines(0), Some((2, 4)));
    assert_eq!(m.region_lines(1), Some((5, 6)));
}

#[test]
fn typing_before_a_region_shifts_it_and_edits_nothing() {
    let mut m = model();
    m.take_a(0);
    m.take_b(1);
    type_at(&mut m, 0, "new first line\n");
    assert_eq!(m.region_text(0), "a1\na2\n");
    assert_eq!(m.region_text(1), "t\n");
    assert_eq!(m.status(0), Some(ConflictStatus::Picked));
    assert_eq!(m.status(1), Some(ConflictStatus::Picked));
    assert!(m.output().text().starts_with("new first line\nhead\n"));
    // Edit the line between the conflicts.
    let at = m.output().text().find("mid").unwrap();
    type_at(&mut m, at, "!");
    assert_eq!(m.region_text(0), "a1\na2\n");
    assert_eq!(m.region_text(1), "t\n");
    assert!(m.tally().2 == 0);
}

#[test]
fn typing_inside_a_region_marks_only_that_conflict_edited() {
    let mut m = model();
    m.take_a(0);
    m.take_b(1);
    let at = start_of(&m, 0) + 2; // after "a1"... inside the first line
    type_at(&mut m, at, "X");
    assert_eq!(m.status(0), Some(ConflictStatus::Edited));
    assert_eq!(m.status(1), Some(ConflictStatus::Picked));
    assert_eq!(m.region_text(0), "a1X\na2\n");
    assert_eq!(m.tally(), (0, 1, 1));
    assert_eq!(m.region_text(1), "t\n");
}

#[test]
fn typing_at_the_start_and_end_of_a_region() {
    let mut m = model();
    m.take_a(0);
    // Start of the first line of the region: inside it.
    let s = start_of(&m, 0);
    type_at(&mut m, s, "A");
    assert_eq!(m.region_text(0), "Aa1\na2\n");
    // End of its last line (before the "\n"): inside it.
    let e = m.states[0].end - 1;
    type_at(&mut m, e, "Z");
    assert_eq!(m.region_text(0), "Aa1\na2Z\n");
    // Start of the line after the region: not inside.
    let e = m.states[0].end;
    type_at(&mut m, e, "M");
    assert_eq!(m.region_text(0), "Aa1\na2Z\n");
    assert!(m.output().text().contains("Mmid"));
}

#[test]
fn adding_a_line_inside_a_region_grows_it() {
    let mut m = model();
    m.take_a(0);
    let e = m.states[0].end - 1;
    m.edit(|d| {
        d.set_cursor(e, false);
        d.on_key(&KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), 10);
        d.on_key(&key('n'), 10);
    });
    assert_eq!(m.region_text(0), "a1\na2\nn\n");
    assert!(m.is_edited(0));
    assert_eq!(m.region_lines(0), Some((2, 5)));
    assert_eq!(m.output_text(), "head\nx\na1\na2\nn\nmid\ntail\n");
}

#[test]
fn deleting_across_a_boundary_extends_the_region_over_whole_lines() {
    let mut m = model();
    m.take_a(0);
    // Backspace the line break after the region: the next line joins the
    // region's last line, which the region then owns whole.
    let e = m.states[0].end;
    m.edit(|d| {
        d.set_cursor(e, false);
        d.on_key(&KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE), 10);
    });
    assert_eq!(m.region_text(0), "a1\na2mid\n");
    assert!(m.is_edited(0));
    assert_eq!(m.output().text(), "head\nx\na1\na2mid\ntail\n");
}

#[test]
fn typing_the_original_text_back_un_edits_the_region() {
    let mut m = model();
    m.take_a(0);
    let s = start_of(&m, 0);
    type_at(&mut m, s, "Q");
    assert!(m.is_edited(0));
    m.edit(|d| {
        d.set_cursor(s + 1, false);
        d.on_key(&KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE), 10);
    });
    assert_eq!(m.region_text(0), "a1\na2\n");
    assert_eq!(m.status(0), Some(ConflictStatus::Picked));
}

#[test]
fn picking_again_replaces_an_edit_and_undo_brings_it_back() {
    let mut m = model();
    m.take_a(0);
    let s = start_of(&m, 0);
    type_at(&mut m, s, "my ");
    assert_eq!(m.region_text(0), "my a1\na2\n");
    assert!(m.hint.is_none());

    assert!(m.take_b(0));
    assert_eq!(m.region_text(0), "t1\nt2\nt3\n");
    assert_eq!(m.status(0), Some(ConflictStatus::Picked));
    assert_eq!(m.take_hint(), Some(Hint::ReplacedEdit));
    assert_eq!(m.take_hint(), None, "shown once");

    assert!(m.undo());
    assert_eq!(m.region_text(0), "my a1\na2\n");
    assert_eq!(m.status(0), Some(ConflictStatus::Edited));
    assert!(m.redo());
    assert_eq!(m.region_text(0), "t1\nt2\nt3\n");
    assert_eq!(m.status(0), Some(ConflictStatus::Picked));
}

#[test]
fn a_pick_on_an_unedited_conflict_leaves_no_hint() {
    let mut m = model();
    m.take_a(0);
    m.take_b(0);
    assert_eq!(m.take_hint(), None);
}

#[test]
fn undo_and_redo_walk_picks_and_typing_in_one_stack() {
    let mut m = model();
    let fresh = m.output_text();
    m.take_a(0); // 1
    m.take_b_then_a(1); // 2
    let s = start_of(&m, 1);
    type_at(&mut m, s, "zz"); // 3
    m.swap_order(1); // 4: replaces the edit
    let after_all = m.output_text();
    assert_eq!(m.region_text(1), "o\nt\n");

    assert!(m.undo()); // 4
    assert_eq!(m.region_text(1), "zzt\no\n");
    assert!(m.is_edited(1));
    assert!(m.undo()); // 3
    assert_eq!(m.region_text(1), "t\no\n");
    assert_eq!(m.status(1), Some(ConflictStatus::Picked));
    assert!(m.undo()); // 2
    assert_eq!(m.status(1), Some(ConflictStatus::Unpicked));
    assert_eq!(m.region_text(0), "a1\na2\n");
    assert!(m.undo()); // 1
    assert_eq!(m.output_text(), fresh);
    assert_eq!(m.status(0), Some(ConflictStatus::Unpicked));
    assert!(!m.undo());

    for _ in 0..4 {
        assert!(m.redo());
    }
    assert_eq!(m.output_text(), after_all);
    assert!(!m.redo());
    assert_eq!(m.picks(1).unwrap().order, vec![Ours, Theirs]);
}

#[test]
fn undo_restores_the_checkboxes_with_the_text() {
    let mut m = model();
    m.toggle_chunk(0, Theirs);
    m.toggle_line(0, Ours, 1);
    assert_eq!(m.chunk_pick(0, Ours), ChunkPick::Some);
    m.undo();
    assert_eq!(m.chunk_pick(0, Ours), ChunkPick::None);
    assert_eq!(m.chunk_pick(0, Theirs), ChunkPick::All);
    m.undo();
    assert_eq!(m.chunk_pick(0, Theirs), ChunkPick::None);
    assert_eq!(m.status(0), Some(ConflictStatus::Unpicked));
    m.redo();
    m.redo();
    assert_eq!(m.region_text(0), "t1\nt2\nt3\na2\n");
}

#[test]
fn undoing_typing_keeps_the_anchors_exact() {
    let mut m = model();
    m.take_a(0);
    m.take_b(1);
    // Type into an empty region's neighbor and a region, then undo all.
    let at = m.output().text().find("mid").unwrap();
    m.edit(|d| {
        d.set_cursor(at, false);
        d.insert_text("more\nlines\n");
    });
    let s = start_of(&m, 1);
    type_at(&mut m, s, "k");
    assert_eq!(m.region_text(1), "kt\n");
    m.undo();
    m.undo();
    assert_eq!(m.region_text(0), "a1\na2\n");
    assert_eq!(m.region_text(1), "t\n");
    assert_eq!(m.output_text(), "head\nx\na1\na2\nmid\nt\ntail\n");
    assert_eq!(m.tally(), (0, 2, 0));
}

#[test]
fn typing_into_an_empty_region_belongs_to_it() {
    let mut m = model();
    let s = start_of(&m, 1);
    type_at(&mut m, s, "new");
    assert_eq!(m.status(1), Some(ConflictStatus::Edited));
    assert_eq!(m.status(0), Some(ConflictStatus::Unpicked));
    // The joined line is owned whole by the region (documented edge).
    assert_eq!(m.region_text(1), "newtail\n");
    assert!(m.undo());
    assert_eq!(m.region_text(1), "");
    assert_eq!(m.status(1), Some(ConflictStatus::Unpicked));
}

#[test]
fn consecutive_typing_is_one_undo_step_with_one_snapshot() {
    let mut m = model();
    m.take_a(0);
    let s = start_of(&m, 0);
    m.edit(|d| d.set_cursor(s, false));
    for c in "xyz".chars() {
        m.edit(|d| {
            d.on_key(&key(c), 10);
        });
    }
    assert_eq!(m.region_text(0), "xyza1\na2\n");
    assert!(m.undo());
    assert_eq!(m.region_text(0), "a1\na2\n");
    assert_eq!(m.status(0), Some(ConflictStatus::Picked));
    assert!(m.undo());
    assert_eq!(m.status(0), Some(ConflictStatus::Unpicked));
}

#[test]
fn take_all_applies_one_side_to_every_conflict_in_one_undo_step() {
    let mut m = model();
    assert!(m.take_all_b());
    assert_eq!(m.output_text(), "head\nx\nt1\nt2\nt3\nmid\nt\ntail\n");
    assert_eq!(m.tally(), (0, 2, 0));
    assert!(m.take_all_a());
    assert_eq!(m.output_text(), "head\nx\na1\na2\nmid\no\ntail\n");
    assert!(!m.take_all_a(), "already so: nothing to do");
    assert!(m.undo());
    assert_eq!(m.output_text(), "head\nx\nt1\nt2\nt3\nmid\nt\ntail\n");
    assert!(m.undo());
    assert_eq!(m.output_text(), "head\nx\nmid\ntail\n");
    assert_eq!(m.tally(), (2, 0, 0));
}

#[test]
fn take_all_keeps_text_typed_between_the_conflicts_and_replaces_edits() {
    let mut m = model();
    m.take_a(0);
    let s = start_of(&m, 0);
    type_at(&mut m, s, "edit ");
    let at = m.output().text().find("mid").unwrap();
    type_at(&mut m, at, "typed ");
    assert!(m.take_all_b());
    assert_eq!(m.output_text(), "head\nx\nt1\nt2\nt3\ntyped mid\nt\ntail\n");
    assert_eq!(m.tally(), (0, 2, 0));
    assert!(m.undo());
    assert_eq!(m.region_text(0), "edit a1\na2\n");
    assert!(m.output().text().contains("typed mid"));
}

#[test]
fn take_all_with_both_sides_uses_the_given_order() {
    let mut m = model();
    m.take_all(&[Theirs, Ours]);
    assert_eq!(m.region_text(1), "t\no\n");
    assert_eq!(m.order_badge(1, Theirs), Some(1));
}

#[test]
fn reset_goes_back_to_the_fresh_merge_and_is_undoable() {
    let mut m = model();
    assert!(!m.reset(), "nothing to reset");
    m.take_a(0);
    let s = start_of(&m, 0);
    type_at(&mut m, s, "zz");
    let at = m.output().text().find("tail").unwrap();
    type_at(&mut m, at, "q");
    assert!(m.reset());
    assert_eq!(m.output_text(), "head\nx\nmid\ntail\n");
    assert_eq!(m.tally(), (2, 0, 0));
    assert!(m.undo());
    assert!(m.output().text().contains("zza1"));
    assert!(m.output().text().contains("qtail"));
}

#[test]
fn crlf_and_bom_come_back_on_the_file_and_the_editor_holds_lf() {
    let mut d = two_conflicts();
    d.eol = Eol::Crlf;
    d.bom = true;
    let mut m = MergeModel::new(d);
    m.take_a_then_b(0);
    m.take_a(1);
    assert!(!m.output().text().contains('\r'));
    let file = m.output_file();
    assert_eq!(file.eol, Eol::Crlf);
    assert_eq!(
        file.text,
        "head\r\nx\r\na1\r\na2\r\nt1\r\nt2\r\nt3\r\nmid\r\no\r\ntail\r\n"
    );
    assert!(file.bom);
    let bytes = file.utf8_bytes();
    assert_eq!(&bytes[..3], b"\xEF\xBB\xBF");
    assert!(bytes.ends_with(b"tail\r\n"));
    assert_eq!(file.content_string().chars().next(), Some('\u{feff}'));
    assert_eq!(file.encoding, None);
}

#[test]
fn the_editor_eol_setting_overrides_the_files() {
    let m = MergeModel::new(two_conflicts()).with_eol(Eol::Crlf);
    assert_eq!(m.output_file().text, "head\r\nx\r\nmid\r\ntail\r\n");
    let mut d = two_conflicts();
    d.eol = Eol::Crlf;
    let m = MergeModel::new(d).with_eol(Eol::Lf);
    assert_eq!(m.output_file().text, "head\nx\nmid\ntail\n");
}

#[test]
fn a_missing_final_newline_is_applied_on_output_only() {
    let mut d = two_conflicts();
    d.final_newline = false;
    let mut m = MergeModel::new(d.clone());
    m.take_a(0);
    m.take_b(1);
    assert!(
        m.output().text().ends_with("tail\n"),
        "the editor keeps whole lines"
    );
    assert_eq!(m.output_text(), "head\nx\na1\na2\nmid\nt\ntail");
    let picks = [
        ConflictPicks::take(d.conflicts().next().unwrap(), &[Ours]),
        ConflictPicks::take(d.conflicts().nth(1).unwrap(), &[Theirs]),
    ];
    assert_eq!(m.output_text(), compose_resolution(&d, &picks));
    let mut crlf = d;
    crlf.eol = Eol::Crlf;
    let m = MergeModel::new(crlf);
    assert_eq!(m.output_file().text, "head\r\nx\r\nmid\r\ntail");
}

#[test]
fn a_conflict_at_the_end_of_a_file_without_a_final_newline() {
    let mut d = merge_doc(vec![merged(&["a"]), conflict(&[], &["x"], &["y"])]);
    d.final_newline = false;
    let mut m = MergeModel::new(d.clone());
    assert_eq!(m.output_text(), "a");
    m.take_b(0);
    assert_eq!(m.output_text(), "a\ny");
    let picks = [ConflictPicks::take(
        d.conflicts().next().unwrap(),
        &[Theirs],
    )];
    assert_eq!(m.output_text(), compose_resolution(&d, &picks));
    m.clear(0);
    assert_eq!(m.output_text(), "a");
}

#[test]
fn the_encoding_decision_is_carried_to_the_writer() {
    let mut d = two_conflicts();
    d.encoding = Some("windows-1252".into());
    let m = MergeModel::new(d);
    assert_eq!(m.output_file().encoding.as_deref(), Some("windows-1252"));
}

#[test]
fn a_file_with_no_conflict_regions_has_nothing_to_pick() {
    let mut m = MergeModel::new(merge_doc(vec![merged(&["a", "b"])]));
    assert_eq!(m.conflict_count(), 0);
    assert_eq!(m.counter_label(), "No conflicts");
    assert!(!m.take_all_a());
    assert_eq!(m.next_conflict(), None);
    assert_eq!(m.output_text(), "a\nb\n");
    assert!(m.save_checks().is_clean());
}

#[test]
fn save_checks_report_unpicked_conflicts_but_not_picked_or_edited_ones() {
    let mut m = model();
    assert_eq!(m.save_checks().unpicked, vec![0, 1]);
    m.take_a(0);
    assert_eq!(m.save_checks().unpicked, vec![1]);
    // An explicit "none" is a decision.
    m.clear(1);
    assert!(m.save_checks().is_clean());
    // Typing into an unpicked conflict also settles it.
    let mut m = model();
    m.take_a(0);
    let s = start_of(&m, 1);
    type_at(&mut m, s, "mine\n");
    assert_eq!(m.status(1), Some(ConflictStatus::Edited));
    assert!(m.save_checks().is_clean());
}

#[test]
fn save_checks_find_marker_like_lines_with_their_line_numbers() {
    let mut d = two_conflicts();
    if let MergeRegion::Conflict { ours, .. } = &mut d.regions[1] {
        *ours = lines(&["<<<<<<< HEAD", "real", "=======", ">>>>>>>"]);
    }
    let mut m = MergeModel::new(d);
    m.take_a(0);
    m.take_a(1);
    // head, x, <<<<<<< HEAD (3), real, ======= (5), >>>>>>> (6)
    assert_eq!(m.save_checks().marker_lines, vec![3, 5, 6]);
}

#[test]
fn only_exactly_seven_markers_count_and_not_longer_runs_or_glued_text() {
    let mut d = two_conflicts();
    if let MergeRegion::Conflict { ours, .. } = &mut d.regions[1] {
        *ours = lines(&[
            "========",
            "=======x",
            "<<<<<< short",
            "|||||||| long",
            "||||||| base",
        ]);
    }
    let mut m = MergeModel::new(d);
    m.take_all_a();
    assert_eq!(m.save_checks().marker_lines, vec![7]);
}

#[test]
fn a_marker_typed_in_the_output_is_caught_too() {
    let mut m = model();
    m.take_all_a();
    assert!(m.save_checks().is_clean());
    let at = m.output().text().find("mid").unwrap();
    type_at(&mut m, at, "=======\n");
    assert_eq!(m.save_checks().marker_lines, vec![5]);
}

#[test]
fn navigation_wraps_and_the_counter_reads_k_of_n_and_what_is_left() {
    let mut m = model();
    assert_eq!(m.counter_label(), "Conflict 1 / 2 · 2 left");
    assert_eq!(m.next_conflict(), Some(1));
    assert_eq!(m.counter_label(), "Conflict 2 / 2 · 2 left");
    assert_eq!(m.next_conflict(), Some(0), "wraps past the last");
    assert_eq!(m.prev_conflict(), Some(1), "wraps past the first");
    m.take_a(1);
    assert_eq!(m.counter_label(), "Conflict 2 / 2 · 1 left");
    assert_eq!(
        m.counter(),
        Counter {
            current: 2,
            total: 2,
            left: 1
        }
    );
    m.take_a(0);
    assert_eq!(m.counter_label(), "Conflict 2 / 2 · 0 left");
    assert!(m.go_to(0));
    assert!(!m.go_to(9));
    assert_eq!(m.current(), 0);
}

#[test]
fn next_unpicked_skips_the_handled_ones() {
    let mut m = MergeModel::new(merge_doc(vec![
        conflict(&[], &["a"], &["b"]),
        merged(&["m"]),
        conflict(&[], &["a"], &["b"]),
        merged(&["m"]),
        conflict(&[], &["a"], &["b"]),
    ]));
    m.take_a(1);
    assert_eq!(m.next_unpicked(), Some(2));
    assert_eq!(m.next_unpicked(), Some(0), "wraps over the picked one");
    m.take_a(0);
    m.take_a(2);
    assert_eq!(m.next_unpicked(), None);
    assert_eq!(m.current(), 0, "left alone");
}

#[test]
fn the_caret_selects_the_current_conflict() {
    let mut m = model();
    m.take_a(0);
    m.take_b(1);
    let inside = start_of(&m, 1) + 1;
    m.output_for_caret().set_cursor(inside, false);
    m.follow_caret();
    assert_eq!(m.current(), 1);
    assert_eq!(m.conflict_at(start_of(&m, 0)), Some(0));
    assert_eq!(m.conflict_at(0), None);
    m.output_for_caret().set_cursor(0, false);
    m.follow_caret();
    assert_eq!(m.current(), 1, "outside a region: unchanged");
}

#[test]
fn origins_name_the_side_each_output_line_came_from() {
    let mut m = model();
    m.toggle_chunk(0, Theirs);
    m.toggle_line(0, Ours, 1);
    m.take_a(1);
    // head, x, t1, t2, t3, a2, mid, o, tail
    let o: Vec<Origin> = (0..9).map(|l| m.origin_of_line(l)).collect();
    assert_eq!(
        o,
        vec![
            Origin::Common,
            Origin::Common,
            Origin::Side(Theirs),
            Origin::Side(Theirs),
            Origin::Side(Theirs),
            Origin::Side(Ours),
            Origin::Common,
            Origin::Side(Ours),
            Origin::Common,
        ]
    );
}

#[test]
fn typed_lines_show_the_pencil_in_a_region_and_in_a_gap() {
    let mut m = model();
    m.take_a(0);
    m.take_a(1);
    let s = start_of(&m, 0);
    type_at(&mut m, s, "e");
    assert_eq!(m.origin_of_line(2), Origin::Edited);
    assert_eq!(m.origin_of_line(3), Origin::Edited, "the whole region");
    let at = m.output().text().find("tail").unwrap();
    type_at(&mut m, at, "g");
    let last = m.output().line_count() - 2;
    assert_eq!(m.output().line(last), "gtail");
    assert_eq!(m.origin_of_line(last), Origin::Edited);
    assert_eq!(m.origin_of_line(0), Origin::Common, "other gaps stay clean");
}

#[test]
fn the_placeholder_row_sits_where_an_unpicked_conflict_will_land() {
    let mut m = model();
    assert_eq!(m.placeholder_line(0), Some(2));
    assert_eq!(m.placeholder_line(1), Some(3));
    m.take_a(0);
    assert_eq!(m.placeholder_line(0), None);
    assert_eq!(m.placeholder_line(1), Some(5));
    m.clear(1);
    assert_eq!(m.placeholder_line(1), None, "decided empty: no placeholder");
    assert_eq!(m.region_lines(1), Some((5, 5)));
}

#[test]
fn dirty_follows_the_decisions_and_the_text_and_save_resets_it() {
    let mut m = model();
    assert!(!m.is_dirty());
    m.take_a(0);
    assert!(m.is_dirty());
    m.mark_saved();
    assert!(!m.is_dirty());
    m.take_b(0);
    assert!(m.is_dirty());
    m.undo();
    assert!(!m.is_dirty(), "undone back to the saved state");
    let s = start_of(&m, 0);
    type_at(&mut m, s, "t");
    assert!(m.is_dirty());
    m.undo();
    assert!(!m.is_dirty());
    // A "none" on a conflict with nothing in it changes no text but is a
    // decision.
    let mut m = MergeModel::new(merge_doc(vec![conflict(&[], &[], &[])]));
    assert!(m.clear(0));
    assert!(m.is_dirty());
}

#[test]
fn a_cleared_conflict_is_not_dirty_twice_and_a_no_op_pick_is_refused() {
    let mut m = model();
    assert!(m.take_a(0));
    assert!(!m.take_a(0), "same picks again: no new undo step");
    assert!(m.undo());
    assert!(!m.can_undo());
}

#[test]
fn the_cache_restores_picks_and_output_for_the_same_stage_oids() {
    let mut cache = PicksCache::default();
    let mut m = model();
    m.toggle_chunk(0, Theirs);
    m.toggle_line(0, Ours, 0);
    m.take_b(1);
    let s = start_of(&m, 1);
    type_at(&mut m, s, "hand ");
    m.go_to(1);
    let text = m.output().text().to_string();
    cache.store(&m);
    assert_eq!(cache.len(), 1);

    let back = cache.open(two_conflicts());
    assert_eq!(back.output().text(), text);
    assert_eq!(back.current(), 1);
    assert_eq!(back.status(0), Some(ConflictStatus::Picked));
    assert_eq!(back.status(1), Some(ConflictStatus::Edited));
    assert_eq!(back.picks(0).unwrap().order, vec![Theirs, Ours]);
    assert_eq!(back.chunk_pick(0, Theirs), ChunkPick::All);
    assert!(back.is_dirty(), "unsaved work still prompts on close");
    assert!(!back.can_undo(), "history is lost across a close");
    // The restored model keeps working.
    let mut back = back;
    assert!(back.take_a(1));
    assert_eq!(back.region_text(1), "o\n");
    assert!(back.undo());
    assert_eq!(back.region_text(1), "hand t\n");
}

#[test]
fn the_cache_ignores_a_snapshot_when_the_stages_changed() {
    let mut cache = PicksCache::default();
    let mut m = model();
    m.take_all_a();
    cache.store(&m);
    let mut moved = two_conflicts();
    moved.entry.theirs = stage(9);
    let fresh = cache.open(moved);
    assert_eq!(fresh.tally(), (2, 0, 0));
    assert_eq!(fresh.output_text(), "head\nx\nmid\ntail\n");
    // Another path with the same oids is another file.
    let mut other = two_conflicts();
    other.entry.path = "g.txt".into();
    assert_eq!(cache.open(other).tally(), (2, 0, 0));
}

#[test]
fn storing_an_untouched_model_forgets_the_file_and_clear_empties_the_cache() {
    let mut cache = PicksCache::default();
    let mut m = model();
    m.take_a(0);
    cache.store(&m);
    assert_eq!(cache.len(), 1);
    m.undo();
    cache.store(&m);
    assert_eq!(cache.len(), 0);
    m.take_a(0);
    cache.store(&m);
    cache.clear();
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.open(two_conflicts()).tally(), (2, 0, 0));
}

#[test]
fn a_file_edited_outside_loads_the_disk_text_and_disables_picks() {
    let mut d = two_conflicts();
    d.worktree = WorktreeFileState::EditedNoMarkers { crlf: true };
    d.bom = true;
    let mut m = MergeModel::from_disk(d, "one\r\ntwo\r\n");
    assert!(m.is_outside());
    assert_eq!(m.counter_label(), "edited outside");
    assert_eq!(m.output().text(), "one\ntwo\n");
    assert!(!m.take_all_a());
    assert!(!m.toggle_chunk(0, Ours));
    assert_eq!(m.conflict_count(), 0);
    assert!(m.save_checks().is_clean());
    // Still an editor: typing works and the file keeps its ending.
    m.edit(|d| d.insert_text("!"));
    assert!(m.is_dirty());
    let f = m.output_file();
    assert_eq!(f.text, "!one\r\ntwo\r\n");
    assert!(f.bom);
}

#[test]
fn a_disk_file_without_a_final_newline_round_trips() {
    let mut d = two_conflicts();
    d.worktree = WorktreeFileState::EditedNoMarkers { crlf: false };
    let m = MergeModel::from_disk(d, "one\ntwo");
    assert_eq!(m.output_text(), "one\ntwo");
    assert_eq!(m.output_file().text, "one\ntwo");
    let m = MergeModel::from_disk(two_conflicts(), "");
    assert_eq!(m.output_file().text, "");
}

#[test]
fn block_gives_each_panes_lines() {
    let m = model();
    assert_eq!(m.block(0, Ours), ["a1", "a2"]);
    assert_eq!(m.block(0, Theirs).len(), 3);
    assert_eq!(m.block(0, Base), ["b"]);
    assert!(m.block(9, Ours).is_empty());
}

#[test]
fn many_adjacent_empty_conflicts_stay_ordered_through_edits() {
    let mut m = MergeModel::new(merge_doc(vec![
        conflict(&[], &["a"], &["b"]),
        merged(&["m"]),
        conflict(&[], &["c"], &["d"]),
        conflict(&[], &["e"], &["f"]),
    ]));
    // Conflicts 1 and 2 are both empty at the end of the text.
    m.take_a(1);
    m.take_b(2);
    assert_eq!(m.output_text(), "m\nc\nf\n");
    m.take_all_b();
    assert_eq!(m.output_text(), "b\nm\nd\nf\n");
    let starts: Vec<_> = (0..3).map(|c| m.states[c].start).collect();
    assert!(starts.windows(2).all(|w| w[0] <= w[1]));
    m.undo();
    assert_eq!(m.output_text(), "m\nc\nf\n");
}

#[test]
fn a_suggested_region_is_one_edited_undo_step_and_shifts_the_next_region() {
    let mut m = model();
    let fresh = m.output_text();
    assert!(m.set_region_lines(0, &lines(&["s1", "s2"])));
    assert_eq!(m.region_text(0), "s1\ns2\n");
    assert_eq!(m.status(0), Some(ConflictStatus::Edited));
    assert_eq!(m.status(1), Some(ConflictStatus::Unpicked));
    assert_eq!(m.output_text(), "head\nx\ns1\ns2\nmid\ntail\n");
    assert!(m.take_b(1));
    assert_eq!(m.region_text(1), "t\n");
    assert!(m.undo());
    assert!(m.undo(), "the suggestion is one step");
    assert_eq!(m.output_text(), fresh);
    assert_eq!(m.status(0), Some(ConflictStatus::Unpicked));
    assert!(!m.undo());
    assert!(!m.set_region_lines(0, &[]), "an empty region stays as is");
    assert!(!m.set_region_lines(9, &lines(&["x"])));
}
