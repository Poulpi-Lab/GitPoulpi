//! The Output pane's editing (slice S7): typing on a placeholder row, the
//! selection and clipboard chords, indent, find, Esc, the mouse, and the
//! one undo stack across picks and typing.

use crate::th;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use gitpoulpi_core::domain::{
    ConflictEntry, Eol, MergeDocument, MergeRegion, Oid, StageEntry, WorktreeFileState,
};
use ratatui::layout::Rect;

use crate::clipboard::Clipboard;

use super::model::MergeModel;
use super::{MergeCmd, MergePane, MergeView};

fn lines(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

fn stage(n: u8) -> Option<StageEntry> {
    Some(StageEntry {
        mode: 0o100644,
        oid: Oid([n; 20]),
    })
}

/// head, x / conflict 1 / mid / conflict 2 / tail
fn view() -> MergeView {
    let regions = vec![
        MergeRegion::Merged {
            lines: lines(&["head", "    x"]),
        },
        MergeRegion::Conflict {
            base: lines(&["b"]),
            ours: lines(&["a1", "a2"]),
            theirs: lines(&["t1", "t2", "t3"]),
        },
        MergeRegion::Merged {
            lines: lines(&["mid"]),
        },
        MergeRegion::Conflict {
            base: vec![],
            ours: lines(&["o"]),
            theirs: lines(&["t"]),
        },
        MergeRegion::Merged {
            lines: lines(&["tail"]),
        },
    ];
    let doc = MergeDocument {
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
    };
    let mut v = MergeView::new(MergeModel::new(doc));
    v.heights = (10, 8);
    v.areas.out = Rect::new(0, 20, 80, 8);
    v.areas.gutter = 5;
    v
}

fn press(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, mods)
}

fn plain(c: char) -> KeyEvent {
    press(KeyCode::Char(c), KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    press(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn out(v: &MergeView) -> String {
    v.model.output().text().to_string()
}

/// Focus the Output with the caret on conflict `c`'s first row.
fn edit_in_output(v: &mut MergeView, c: usize) {
    v.model.go_to(c);
    v.run(MergeCmd::EditInOutput);
    assert_eq!(v.focus, MergePane::Output);
}

#[test]
fn typing_on_the_placeholder_row_opens_a_line_and_does_not_join_the_next_one() {
    let mut v = view();
    let mut clip = Clipboard::new();
    edit_in_output(&mut v, 0);
    v.on_key(&plain('Z'), &mut clip);
    v.on_key(&plain('Q'), &mut clip);
    assert_eq!(out(&v), "head\n    x\nZQ\nmid\ntail\n");
    // The typed line is the conflict's region, the next line is not.
    assert_eq!(v.model.region_text(0), "ZQ\n");
    assert!(v.model.is_edited(0));
    // The other conflict is untouched and still unpicked.
    assert_eq!(v.model.region_text(1), "");
}

#[test]
fn enter_and_a_paste_on_the_placeholder_row_open_a_line_too() {
    let mut v = view();
    let mut clip = Clipboard::new();
    edit_in_output(&mut v, 1);
    v.on_key(&press(KeyCode::Enter, KeyModifiers::NONE), &mut clip);
    // Conflict 2 (after "mid") got exactly one empty line; "tail" is intact.
    assert_eq!(out(&v), "head\n    x\nmid\n\ntail\n");
    assert_eq!(v.model.output().cursor_line_col(), (4, 0));
    assert_eq!(v.model.region_text(1), "\n");

    let mut v = view();
    edit_in_output(&mut v, 0);
    v.paste("one\ntwo\n");
    assert_eq!(v.model.region_text(0), "one\ntwo\n");
    assert!(out(&v).contains("two\nmid\n"));
}

#[test]
fn typing_into_a_region_with_lines_stays_in_place() {
    let mut v = view();
    let mut clip = Clipboard::new();
    v.run(MergeCmd::TakeA);
    edit_in_output(&mut v, 0);
    v.on_key(&plain('!'), &mut clip);
    assert_eq!(v.model.region_text(0), "!a1\na2\n");
    assert!(v.model.is_edited(0));
}

#[test]
fn shift_arrows_select_and_cut_copy_paste_move_text() {
    let mut v = view();
    let mut clip = Clipboard::new();
    v.run(MergeCmd::TakeA);
    edit_in_output(&mut v, 0);
    // Select "a1" with Shift+Right twice, cut it, paste it back at the end.
    v.on_key(&press(KeyCode::Right, KeyModifiers::SHIFT), &mut clip);
    v.on_key(&press(KeyCode::Right, KeyModifiers::SHIFT), &mut clip);
    assert_eq!(v.model.output().selection().map(|(a, b)| b - a), Some(2));
    v.on_key(&ctrl('x'), &mut clip);
    assert_eq!(v.model.region_text(0), "\na2\n");
    v.on_key(&press(KeyCode::Down, KeyModifiers::NONE), &mut clip);
    v.on_key(&press(KeyCode::End, KeyModifiers::NONE), &mut clip);
    v.on_key(&ctrl('v'), &mut clip);
    assert_eq!(v.model.region_text(0), "\na2a1\n");
    // Copy leaves the text alone.
    v.on_key(&press(KeyCode::Home, KeyModifiers::SHIFT), &mut clip);
    v.on_key(&ctrl('c'), &mut clip);
    assert_eq!(v.model.region_text(0), "\na2a1\n");
}

#[test]
fn tab_indents_enter_keeps_the_indentation_and_shift_tab_outdents() {
    let mut v = view();
    let mut clip = Clipboard::new();
    // Line 1 of the Output is "    x" (4-space indent).
    v.model.output_for_caret().set_cursor(5, false);
    v.focus_pane(MergePane::Output);
    v.on_key(&press(KeyCode::End, KeyModifiers::NONE), &mut clip);
    v.on_key(&press(KeyCode::Enter, KeyModifiers::NONE), &mut clip);
    v.on_key(&plain('y'), &mut clip);
    assert!(out(&v).starts_with("head\n    x\n    y\n"), "{}", out(&v));
    v.on_key(&press(KeyCode::Tab, KeyModifiers::NONE), &mut clip);
    assert!(out(&v).starts_with("head\n    x\n    y"), "{}", out(&v));
    v.on_key(&press(KeyCode::BackTab, KeyModifiers::SHIFT), &mut clip);
    v.on_key(&press(KeyCode::BackTab, KeyModifiers::SHIFT), &mut clip);
    assert!(out(&v).starts_with("head\n    x\ny"), "{}", out(&v));
    // Tab never leaves the Output.
    assert_eq!(v.focus, MergePane::Output);
}

#[test]
fn find_selects_matches_and_esc_closes_the_bar_before_leaving_the_output() {
    let mut v = view();
    let mut clip = Clipboard::new();
    v.run(MergeCmd::TakeA);
    v.focus_pane(MergePane::A);
    v.focus_pane(MergePane::Output);
    v.on_key(&ctrl('f'), &mut clip);
    assert!(v.find_line().is_some());
    v.on_key(&plain('a'), &mut clip);
    v.on_key(&plain('2'), &mut clip);
    let (a, b) = v.model.output().selection().expect("a match is selected");
    assert_eq!(&v.model.output().text()[a..b], "a2");
    // Enter looks for the next one: only one, so it wraps onto itself;
    // an absent query reports no match.
    v.on_key(&press(KeyCode::Char('z'), KeyModifiers::NONE), &mut clip);
    assert!(v.find_line().unwrap().contains("no match"));
    // Esc closes the bar and stays in the Output; the second Esc leaves.
    v.on_key(&press(KeyCode::Esc, KeyModifiers::NONE), &mut clip);
    assert!(v.find_line().is_none());
    assert_eq!(v.focus, MergePane::Output);
    v.on_key(&press(KeyCode::Esc, KeyModifiers::NONE), &mut clip);
    assert_ne!(v.focus, MergePane::Output);
}

#[test]
fn f3_steps_through_the_matches() {
    let mut v = view();
    let mut clip = Clipboard::new();
    v.run(MergeCmd::TakeA);
    v.focus_pane(MergePane::Output);
    v.model.output_for_caret().set_cursor(0, false);
    v.on_key(&ctrl('f'), &mut clip);
    for c in "a".chars() {
        v.on_key(&plain(c), &mut clip);
    }
    let first = v.model.output().selection().unwrap();
    v.on_key(&press(KeyCode::Enter, KeyModifiers::NONE), &mut clip);
    let second = v.model.output().selection().unwrap();
    assert_ne!(first, second);
}

#[test]
fn undo_goes_back_through_typing_and_picks_in_one_stack() {
    let mut v = view();
    let mut clip = Clipboard::new();
    v.run(MergeCmd::TakeA);
    edit_in_output(&mut v, 0);
    v.on_key(&plain('!'), &mut clip);
    v.focus_pane(MergePane::A);
    v.run(MergeCmd::TakeB);
    assert_eq!(v.model.region_text(0), "t1\nt2\nt3\n");
    v.run(MergeCmd::Undo);
    assert_eq!(v.model.region_text(0), "!a1\na2\n");
    v.run(MergeCmd::Undo);
    assert_eq!(v.model.region_text(0), "a1\na2\n");
    v.run(MergeCmd::Redo);
    assert_eq!(v.model.region_text(0), "!a1\na2\n");
}

#[test]
fn a_press_puts_the_caret_a_drag_selects_and_a_double_click_takes_the_word() {
    let mut v = view();
    v.run(MergeCmd::TakeA);
    // Output rows: 0 head, 1 "    x", 2 a1, 3 a2 ... drawn from row 20.
    let col = |c: u16| 5 + c;
    assert!(v.output_press(2, KeyModifiers::NONE, col(0), 22));
    assert_eq!(v.focus, MergePane::Output);
    assert!(v.on_mouse(
        MouseEventKind::Drag(crossterm::event::MouseButton::Left),
        col(2),
        23
    ));
    let (a, b) = v.model.output().selection().expect("dragged selection");
    assert_eq!(&v.model.output().text()[a..b], "a1\na2");
    assert!(v.on_mouse(
        MouseEventKind::Up(crossterm::event::MouseButton::Left),
        col(2),
        23
    ));
    assert!(v.ui.drag.is_none());
    // Shift+click extends from the selection's anchor (where the drag began).
    v.output_press(0, KeyModifiers::SHIFT, col(2), 20);
    let (a, b) = v.model.output().selection().unwrap();
    assert_eq!(&v.model.output().text()[a..b], "ad\n    x\n");
}

#[test]
fn the_find_bar_keeps_every_character_typed() {
    let mut v = view();
    let mut clip = Clipboard::new();
    v.focus_pane(MergePane::Output);
    v.on_key(&ctrl('f'), &mut clip);
    for c in "l4".chars() {
        v.on_key(&plain(c), &mut clip);
    }
    assert!(v.find_line().unwrap().contains("l4"), "{:?}", v.find_line());
}

#[test]
fn the_selection_is_tinted_in_the_output_including_the_line_break() {
    use ratatui::buffer::Buffer;
    let mut v = view();
    v.run(MergeCmd::TakeA);
    // "a1\na2": line 2 from col 1 to the end of line 3.
    let doc = v.model.output_for_caret();
    let a = doc.offset_at(2, 1);
    let b = doc.offset_at(3, 1);
    doc.select_range(a, b);
    let area = Rect::new(0, 0, 20, 1);
    let mut buf = Buffer::empty(area);
    super::render::paint_selection(&mut buf, v.model.output(), 2, 0, 0, 20);
    let tinted: Vec<u16> = (0..20)
        .filter(|&x| buf[(x, 0)].bg == th().ui.text_selection_bg)
        .collect();
    // "1" and the line break.
    assert_eq!(tinted, vec![1, 2]);
}

#[test]
fn ai_suggest_is_refused_when_off_or_busy_and_asks_the_window_when_on() {
    use super::Effect;
    let mut v = view();
    let out = v.run(MergeCmd::AiSuggest);
    assert!(out.effect.is_none() && out.message.is_some(), "AI is off");
    v.ai_enabled = true;
    v.ai_busy = true;
    let out = v.run(MergeCmd::AiSuggest);
    assert!(out.effect.is_none() && out.message.is_some(), "busy");
    v.ai_busy = false;
    v.model.go_to(1);
    let out = v.run(MergeCmd::AiSuggest);
    assert_eq!(out.effect, Some(Effect::AiSuggest { conflict: 1 }));
}

#[test]
fn the_ai_rows_of_the_menus_exist_only_with_ai_on() {
    let mut v = view();
    v.open_chunk_menu(0, (2, 2));
    let rows = |v: &MergeView| v.ui.menu.as_ref().unwrap().rows.clone();
    assert!(!rows(&v).iter().any(|r| r.cmd == MergeCmd::AiSuggest));
    v.ai_enabled = true;
    v.open_chunk_menu(0, (2, 2));
    let ai = rows(&v)
        .into_iter()
        .find(|r| r.cmd == MergeCmd::AiSuggest)
        .expect("the ✦ row");
    assert!(ai.label.starts_with('\u{2726}'), "{}", ai.label);
    v.open_more_menu((2, 2));
    assert!(rows(&v).iter().any(|r| r.cmd == MergeCmd::AiSuggest));
}

// -- un-pick, reset, fluid navigation ---------------------------------------------

use super::model::ConflictStatus;

#[test]
fn unpick_puts_the_current_conflict_back_to_unresolved_by_key_and_command() {
    let mut v = view();
    let mut clip = Clipboard::new();
    v.model.take_a(0);
    v.model.take_b(1);
    assert_eq!(v.model.status(0), Some(ConflictStatus::Picked));
    v.model.go_to(0);
    v.focus_pane(MergePane::A);
    // `u` in a side pane.
    v.on_key(&plain('u'), &mut clip);
    assert_eq!(v.model.status(0), Some(ConflictStatus::Unpicked));
    assert_eq!(v.model.region_text(0), "");
    // The other conflict keeps its pick.
    assert_eq!(v.model.status(1), Some(ConflictStatus::Picked));
    // The command (palette / menu / Alt+U) does the same, and is one undo step.
    v.model.go_to(1);
    let out = v.run(MergeCmd::Unpick);
    assert!(out.message.is_some());
    assert_eq!(v.model.status(1), Some(ConflictStatus::Unpicked));
    assert!(v.model.undo());
    assert_eq!(v.model.status(1), Some(ConflictStatus::Picked));
}

#[test]
fn unpick_also_drops_a_typed_edit_of_the_conflict() {
    let mut v = view();
    let mut clip = Clipboard::new();
    edit_in_output(&mut v, 0);
    v.on_key(&plain('Z'), &mut clip);
    assert!(v.model.is_edited(0));
    v.run(MergeCmd::Unpick);
    assert_eq!(v.model.status(0), Some(ConflictStatus::Unpicked));
    assert_eq!(out(&v), "head\n    x\nmid\ntail\n");
}

#[test]
fn reset_all_is_immediate_without_edits_and_asks_first_with_typed_text() {
    use super::input::Prompt;
    let mut v = view();
    let mut clip = Clipboard::new();
    v.model.take_all_a();
    let o = v.run(MergeCmd::Reset);
    assert!(o.message.is_some() && v.ui.prompt.is_none());
    assert_eq!(v.model.tally().0, 2, "both conflicts unpicked again");

    // With typed text: a box, and Esc keeps everything.
    v.model.take_all_b();
    edit_in_output(&mut v, 0);
    v.on_key(&plain('Z'), &mut clip);
    v.run(MergeCmd::Reset);
    assert_eq!(v.ui.prompt, Some(Prompt::ResetAll));
    let typed = out(&v);
    v.on_key(&press(KeyCode::Esc, KeyModifiers::NONE), &mut clip);
    assert!(v.ui.prompt.is_none());
    assert_eq!(out(&v), typed);
    // Enter resets (one undo step brings the text back).
    v.run(MergeCmd::Reset);
    v.on_key(&press(KeyCode::Enter, KeyModifiers::NONE), &mut clip);
    assert_eq!(out(&v), "head\n    x\nmid\ntail\n");
    assert_eq!(v.model.tally().0, 2);
    assert!(v.model.undo());
    assert_eq!(out(&v), typed);
}

#[test]
fn alt_digits_jump_to_a_pane_on_the_current_conflict() {
    let mut v = view();
    let mut clip = Clipboard::new();
    v.model.go_to(1);
    let alt = |c| press(KeyCode::Char(c), KeyModifiers::ALT);
    v.on_key(&alt('2'), &mut clip);
    assert_eq!(v.focus, MergePane::B);
    assert_eq!(v.model.current(), 1);
    v.on_key(&alt('3'), &mut clip);
    assert_eq!(v.focus, MergePane::Output);
    // The caret is on conflict 2's region.
    assert_eq!(v.model.conflict_at(v.model.output().cursor()), Some(1));
    v.on_key(&alt('1'), &mut clip);
    assert_eq!(v.focus, MergePane::A);
    assert_eq!(v.model.current(), 1);
}

#[test]
fn h_l_and_the_arrows_walk_the_panes_keeping_the_conflict() {
    let mut v = view();
    let mut clip = Clipboard::new();
    v.model.go_to(1);
    v.focus_pane(MergePane::A);
    v.on_key(&plain('l'), &mut clip);
    assert_eq!(v.focus, MergePane::B);
    assert_eq!(v.model.current(), 1, "same conflict, not the top");
    v.on_key(&press(KeyCode::Right, KeyModifiers::NONE), &mut clip);
    assert_eq!(v.focus, MergePane::Output);
    assert_eq!(v.model.conflict_at(v.model.output().cursor()), Some(1));
    // In the Output the arrows are text; Alt+2 goes back.
    v.on_key(&press(KeyCode::Char('2'), KeyModifiers::ALT), &mut clip);
    assert_eq!(v.focus, MergePane::B);
    v.on_key(&plain('h'), &mut clip);
    assert_eq!(v.focus, MergePane::A);
    // No wrap at the left edge.
    v.on_key(&press(KeyCode::Left, KeyModifiers::NONE), &mut clip);
    assert_eq!(v.focus, MergePane::A);
    assert_eq!(v.model.current(), 1);
    // Shift+Left stays the sideways scroll.
    v.h_scroll = 20;
    v.on_key(&press(KeyCode::Left, KeyModifiers::SHIFT), &mut clip);
    assert!(v.h_scroll < 20);
    assert_eq!(v.focus, MergePane::A);
}

#[test]
fn a_vim_count_repeats_the_motions() {
    let mut v = view();
    let mut clip = Clipboard::new();
    v.focus_pane(MergePane::A);
    // Conflict 1's A block has two lines, conflict 2's one: `2j` crosses
    // into the next conflict.
    v.on_key_counted(&plain('j'), &mut clip, 2);
    assert_eq!((v.model.current(), v.ui.cursor), (1, 0));
    v.on_key_counted(&plain('k'), &mut clip, 1);
    assert_eq!(v.model.current(), 0);
    // `n` with a count: three conflicts on, wrapping over two.
    v.on_key_counted(&plain('n'), &mut clip, 3);
    assert_eq!(v.model.current(), 1);
    v.on_key_counted(&plain('N'), &mut clip, 3);
    assert_eq!(v.model.current(), 0);
    // A count is spent by the key that used it.
    v.on_key(&plain('n'), &mut clip);
    assert_eq!(v.model.current(), 1);
}
