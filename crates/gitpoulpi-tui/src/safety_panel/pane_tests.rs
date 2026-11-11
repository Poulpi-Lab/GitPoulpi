//! The Reflog / Lost work panes (commit list | header, files, hunks) and the
//! movable frame, driven without a terminal or a daemon: the daemon's answers
//! are put where the polls would have put them, then the panel is painted,
//! keyed and clicked.

use std::path::PathBuf;
use std::sync::Arc;

use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use gitpoulpi_core::domain::{
    CommitNode, DiffLine, FileDiff, FileStatus, Hunk, HunkId, LineOrigin, LostCommit, LostSource,
    Oid,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::detail::Loaded;
use super::panes::{Divider, Pane};
use super::render::Hit;
use super::*;
use crate::dialog_kit::buffer_text;

const SCREEN: Rect = Rect::new(0, 0, 200, 50);

fn oid(c: char) -> Oid {
    Oid::from_hex(&c.to_string().repeat(40)).unwrap()
}

fn mods(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn key(code: KeyCode) -> KeyEvent {
    mods(code, KeyModifiers::NONE)
}

fn mouse(kind: MouseEventKind, col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn down(col: u16, row: u16) -> MouseEvent {
    mouse(MouseEventKind::Down(MouseButton::Left), col, row)
}

fn drag(col: u16, row: u16) -> MouseEvent {
    mouse(MouseEventKind::Drag(MouseButton::Left), col, row)
}

fn up(col: u16, row: u16) -> MouseEvent {
    mouse(MouseEventKind::Up(MouseButton::Left), col, row)
}

fn hunk(old: &str, new: &str) -> Hunk {
    Hunk {
        id: HunkId(1),
        old_start: 1,
        old_lines: 1,
        new_start: 1,
        new_lines: 1,
        lines: vec![
            DiffLine {
                origin: LineOrigin::Delete,
                content: old.into(),
                staged: false,
            },
            DiffLine {
                origin: LineOrigin::Add,
                content: new.into(),
                staged: false,
            },
        ],
    }
}

/// A Lost work panel on one commit that changed `src/a.rs` and `README.md`,
/// the way the polls leave it once the daemon has answered everything.
fn lost_panel() -> SafetyPanel {
    let activity = Arc::new(crate::activity::Activity::default());
    let mut panel = SafetyPanel::open(activity.clone(), Tab::Lost, None, 100);
    let c = oid('c');
    panel.lost.commits = vec![LostCommit {
        oid: c,
        summary: "lost work".into(),
        author: "A".into(),
        time: 0,
        parents: vec![oid('d')],
        source: LostSource::Reflog,
    }];
    panel.lost.loaded = true;
    let paths: Vec<PathBuf> = vec!["src/a.rs".into(), "README.md".into()];
    panel.lost.detail.fill(
        c,
        Loaded {
            node: Some(CommitNode {
                id: c,
                parents: vec![oid('d')],
                summary: "lost work".into(),
                message: "lost work\n\nthe body of it".into(),
                author_name: "Ann".into(),
                author_email: "a@x".into(),
                timestamp: 0,
                refs: Vec::new(),
            }),
            statuses: [
                (paths[0].clone(), FileStatus::Modified),
                (paths[1].clone(), FileStatus::Added),
            ]
            .into_iter()
            .collect(),
            paths: paths.clone(),
        },
    );
    for (path, old, new) in [
        ("src/a.rs", "old_line()", "new_line()"),
        ("README.md", "# before", "# after"),
    ] {
        panel.panes.diff.keep(
            (c, PathBuf::from(path)),
            FileDiff {
                path: path.into(),
                status: FileStatus::Modified,
                hunks: vec![hunk(old, new)],
            },
        );
    }
    panel.poll_panes_for_test();
    panel
}

impl SafetyPanel {
    /// What a poll does with the answers that are already in.
    fn poll_panes_for_test(&mut self) {
        let want = self.pane_commit();
        self.poll_panes(want);
    }

    /// Paints and refreshes the state a paint leaves (hit rects, geometry).
    fn paint_at(&mut self, screen: Rect) -> Buffer {
        self.poll_panes_for_test();
        let mut buf = Buffer::empty(screen);
        self.render_in(screen, &mut buf);
        buf
    }

    fn hit_rect(&self, wanted: Hit) -> Rect {
        self.hits
            .iter()
            .find(|(_, h)| *h == wanted)
            .map(|(r, _)| *r)
            .unwrap_or_else(|| panic!("{wanted:?} not among the hits"))
    }
}

#[test]
fn the_dialog_at_200_by_50_lays_out_list_files_and_hunks() {
    let mut panel = lost_panel();
    let buf = panel.paint_at(SCREEN);
    let text = buffer_text(&buf);
    // 160 x 44, centered.
    assert_eq!(panel.frame, Rect::new(20, 3, 160, 44), "{text}");
    for want in [
        "lost work",       // the commit row, and the header
        "Ann",             // the header's author
        "the body of it",  // the message
        "2 files changed", // the files box title
        "a.rs",
        "README.md",
        "Changes in src/a.rs",
        "old_line()",
        "new_line()",
    ] {
        assert!(text.contains(want), "{want:?} missing\n{text}");
    }
    let lay = panel.panes.layout;
    assert!(
        lay.list.right() < lay.files.x,
        "the list is left of the right pane"
    );
    assert_eq!(lay.files.x, lay.diff.x);
    assert!(
        lay.files.bottom() <= lay.diff.y,
        "the files are above the hunks"
    );
    // Every pane has a hit, and so has each divider.
    for hit in [
        Hit::Pane(Pane::List),
        Hit::Pane(Pane::Files),
        Hit::Pane(Pane::Diff),
        Hit::Divider(Divider::Vertical),
        Hit::Divider(Divider::Horizontal),
        Hit::Mode(crate::status_pane::FileListMode::Path),
        Hit::File(0),
    ] {
        panel.hit_rect(hit);
    }
    // The corners carry the resize grips.
    let f = panel.frame;
    assert_eq!(buf[(f.right() - 1, f.bottom() - 1)].symbol(), "\u{25e2}");
    assert_eq!(buf[(f.x, f.bottom() - 1)].symbol(), "\u{25e3}");
}

#[test]
fn tab_walks_list_files_diff_and_ctrl_tab_walks_the_views() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    assert_eq!(panel.panes.focus, Pane::List);
    panel.on_key(key(KeyCode::Tab));
    assert_eq!(panel.panes.focus, Pane::Files);
    panel.on_key(key(KeyCode::Tab));
    assert_eq!(panel.panes.focus, Pane::Diff);
    panel.on_key(key(KeyCode::Tab));
    assert_eq!(panel.panes.focus, Pane::List);
    panel.on_key(key(KeyCode::BackTab));
    assert_eq!(panel.panes.focus, Pane::Diff);
    assert_eq!(panel.tab, Tab::Lost, "Tab never changed the view");
    panel.on_key(mods(KeyCode::Tab, KeyModifiers::CONTROL));
    assert_eq!(panel.tab, Tab::History);
    panel.on_key(mods(KeyCode::BackTab, KeyModifiers::CONTROL));
    assert_eq!(panel.tab, Tab::Lost);
    panel.on_key(key(KeyCode::Char('2')));
    assert_eq!(panel.tab, Tab::Reflog);
    panel.on_key(key(KeyCode::Char('3')));
    assert_eq!(panel.tab, Tab::Lost);
}

#[test]
fn the_file_cursor_picks_which_hunks_show_and_v_flips_path_and_tree() {
    let mut panel = lost_panel();
    let text = buffer_text(&panel.paint_at(SCREEN));
    assert!(text.contains("new_line()"), "{text}");
    panel.on_key(key(KeyCode::Tab));
    panel.on_key(key(KeyCode::Down));
    let text = buffer_text(&panel.paint_at(SCREEN));
    assert!(
        text.contains("Changes in README.md") && text.contains("# after"),
        "{text}"
    );
    assert!(!text.contains("new_line()"));
    panel.on_key(key(KeyCode::Char('v')));
    assert_eq!(panel.panes.mode, crate::status_pane::FileListMode::Tree);
    let text = buffer_text(&panel.paint_at(SCREEN));
    assert!(text.contains("src/"), "a folder row in tree mode\n{text}");
}

#[test]
fn a_click_on_a_file_row_shows_its_hunks() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    let row = panel.hit_rect(Hit::File(1));
    panel.on_mouse(down(row.x + 2, row.y));
    panel.on_mouse(up(row.x + 2, row.y));
    let text = buffer_text(&panel.paint_at(SCREEN));
    assert!(text.contains("Changes in README.md"), "{text}");
    assert_eq!(panel.panes.focus, Pane::Files);
}

#[test]
fn dragging_the_vertical_divider_resizes_the_list_and_double_click_resets() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    let before = panel.panes.layout.list.width;
    let grab = panel.hit_rect(Hit::Divider(Divider::Vertical));
    let (x, y) = (grab.x, grab.y + 3);
    panel.on_mouse(down(x, y));
    panel.on_mouse(drag(x + 20, y));
    panel.on_mouse(up(x + 20, y));
    panel.paint_at(SCREEN);
    assert_eq!(panel.panes.layout.list.width, before + 20);
    // Two quick presses on the divider put it back.
    let grab = panel.hit_rect(Hit::Divider(Divider::Vertical));
    panel.on_mouse(down(grab.x, grab.y + 3));
    panel.on_mouse(up(grab.x, grab.y + 3));
    panel.on_mouse(down(grab.x, grab.y + 3));
    panel.on_mouse(up(grab.x, grab.y + 3));
    panel.paint_at(SCREEN);
    assert_eq!(panel.panes.layout.list.width, before);
}

#[test]
fn dragging_the_horizontal_divider_gives_the_files_more_rows() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    let before = panel.panes.layout.files.height;
    let grab = panel.hit_rect(Hit::Divider(Divider::Horizontal));
    panel.on_mouse(down(grab.x + 10, grab.y));
    panel.on_mouse(drag(grab.x + 10, grab.y + 6));
    panel.on_mouse(up(grab.x + 10, grab.y + 6));
    panel.paint_at(SCREEN);
    assert!(panel.panes.layout.files.height > before);
}

#[test]
fn the_alt_keys_move_the_dividers_and_alt_0_resets_them() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    let (w, h) = (
        panel.panes.layout.list.width,
        panel.panes.layout.files.height,
    );
    panel.on_key(mods(KeyCode::Right, KeyModifiers::ALT));
    panel.on_key(mods(KeyCode::Down, KeyModifiers::ALT));
    panel.paint_at(SCREEN);
    assert!(panel.panes.layout.list.width > w);
    assert!(panel.panes.layout.files.height > h);
    panel.on_key(mods(KeyCode::Left, KeyModifiers::ALT));
    panel.on_key(mods(KeyCode::Left, KeyModifiers::ALT));
    panel.paint_at(SCREEN);
    assert!(panel.panes.layout.list.width < w);
    panel.on_key(mods(KeyCode::Char('0'), KeyModifiers::ALT));
    panel.paint_at(SCREEN);
    assert_eq!(
        (
            panel.panes.layout.list.width,
            panel.panes.layout.files.height
        ),
        (w, h)
    );
}

#[test]
fn the_diff_and_the_files_each_have_a_bar_that_can_be_pressed_and_dragged() {
    let mut panel = lost_panel();
    // Many files and a long diff, so both bars exist.
    let c = oid('c');
    let paths: Vec<PathBuf> = (0..80)
        .map(|n| PathBuf::from(format!("f{n:02}.rs")))
        .collect();
    panel.lost.detail.fill(
        c,
        Loaded {
            node: None,
            statuses: paths
                .iter()
                .map(|p| (p.clone(), FileStatus::Added))
                .collect(),
            paths: paths.clone(),
        },
    );
    let long = FileDiff {
        path: "f00.rs".into(),
        status: FileStatus::Added,
        hunks: vec![Hunk {
            id: HunkId(1),
            old_start: 0,
            old_lines: 0,
            new_start: 1,
            new_lines: 200,
            lines: (0..200)
                .map(|n| DiffLine {
                    origin: LineOrigin::Add,
                    content: format!("line number {n}"),
                    staged: false,
                })
                .collect(),
        }],
    };
    panel.panes.diff.keep((c, PathBuf::from("f00.rs")), long);
    panel.paint_at(SCREEN);
    let lay = panel.panes.layout;
    // Files bar: press the track below the thumb and drag.
    let col = lay.files.right() - 1;
    panel.on_mouse(down(col, lay.files.bottom() - 3));
    assert!(panel.panes.files.offset > 0, "the files bar jumped");
    panel.on_mouse(up(col, lay.files.bottom() - 3));
    let shown = buffer_text(&panel.paint_at(SCREEN));
    assert!(
        shown.contains("f6") && !shown.contains("+ f00.txt"),
        "painted after the jump\n{shown}"
    );
    panel.on_mouse(down(col, lay.files.bottom() - 3));
    let top = panel.panes.files.offset;
    panel.on_mouse(drag(col, lay.files.y + 2));
    assert!(
        panel.panes.files.offset < top,
        "and the thumb followed the drag up"
    );
    panel.on_mouse(up(col, lay.files.y + 2));
    assert!(panel.panes.files.grab.is_none());
    // Diff bar.
    let col = lay.diff.right() - 1;
    panel.on_mouse(down(col, lay.diff.bottom() - 3));
    assert!(panel.panes.scroll.offset > 0, "the diff bar jumped");
    assert_eq!(panel.panes.focus, Pane::Diff);
    let top = panel.panes.scroll.offset;
    panel.on_mouse(drag(col, lay.diff.y + 2));
    assert!(panel.panes.scroll.offset < top);
    panel.on_mouse(up(col, lay.diff.y + 2));
    assert!(panel.panes.scroll.grab.is_none());
}

#[test]
fn dragging_the_title_moves_the_dialog_and_a_release_outside_never_closes_it() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    let f = panel.frame;
    let outcome = panel.on_mouse(down(f.x + 40, f.y));
    assert!(matches!(outcome, PanelOutcome::Stay));
    panel.on_mouse(drag(f.x + 10, f.y + 2));
    let outcome = panel.on_mouse(up(3, 1));
    assert!(
        matches!(outcome, PanelOutcome::Stay),
        "a release outside is not a click"
    );
    panel.paint_at(SCREEN);
    assert_ne!(panel.frame.x, f.x);
    assert_eq!((panel.frame.width, panel.frame.height), (f.width, f.height));
}

#[test]
fn the_close_cross_keeps_priority_over_the_title_drag() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    let cross = panel.close.rect().expect("a cross");
    let outcome = panel.on_mouse(down(cross.x, cross.y));
    assert!(matches!(outcome, PanelOutcome::Close));
}

#[test]
fn dragging_a_bottom_corner_resizes_and_the_borders_do_too() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    let f = panel.frame;
    let (cx, cy) = (f.right() - 1, f.bottom() - 1);
    panel.on_mouse(down(cx, cy));
    panel.on_mouse(drag(cx - 30, cy - 10));
    panel.on_mouse(up(cx - 30, cy - 10));
    panel.paint_at(SCREEN);
    assert_eq!(
        (panel.frame.width, panel.frame.height),
        (f.width - 30, f.height - 10)
    );
    // The right border widens it again.
    let f = panel.frame;
    panel.on_mouse(down(f.right() - 1, f.y + 5));
    panel.on_mouse(up(f.right() + 9, f.y + 5));
    panel.paint_at(SCREEN);
    assert_eq!(panel.frame.width, f.width + 10);
}

#[test]
fn arrange_mode_moves_resizes_and_resets_from_the_keyboard() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    let f = panel.frame;
    panel.arrange();
    assert!(crate::dialog_kit::frame::is_arranging());
    // The arrange mode takes the keys: `q` is not "close" there.
    assert!(matches!(
        panel.on_key(key(KeyCode::Char('q'))),
        PanelOutcome::Stay
    ));
    panel.on_key(key(KeyCode::Right));
    panel.on_key(mods(KeyCode::Down, KeyModifiers::SHIFT));
    let buf = panel.paint_at(SCREEN);
    assert_eq!(panel.frame.x, f.x + 2);
    assert_eq!(panel.frame.height, f.height + 1, "Shift+Down grew it a row");
    assert!(buffer_text(&buf).contains("Arrange:"), "the frame says so");
    panel.on_key(key(KeyCode::Char('0')));
    panel.paint_at(SCREEN);
    assert_eq!(panel.frame, f);
    panel.on_key(key(KeyCode::Esc));
    assert!(!crate::dialog_kit::frame::is_arranging());
    assert!(matches!(
        panel.on_key(key(KeyCode::Esc)),
        PanelOutcome::Close
    ));
}

#[test]
fn the_palette_rows_arrange_and_reset_the_dialog_and_reset_the_panes() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    let state = panel.journal_state();
    let f = panel.frame;
    panel.run_command(SafetyCmd::ArrangeDialog, &state);
    assert!(crate::dialog_kit::frame::is_arranging());
    panel.on_key(key(KeyCode::Left));
    panel.on_key(key(KeyCode::Enter));
    panel.paint_at(SCREEN);
    assert_ne!(panel.frame, f);
    panel.run_command(SafetyCmd::ResetDialog, &state);
    panel.paint_at(SCREEN);
    assert_eq!(panel.frame, f);
    panel.on_key(mods(KeyCode::Right, KeyModifiers::ALT));
    assert!(panel.panes.cols.is_some());
    panel.run_command(SafetyCmd::ResetPanes, &state);
    assert!(panel.panes.cols.is_none());
    panel.run_command(SafetyCmd::NextPane, &state);
    assert_eq!(panel.panes.focus, Pane::Files);
    panel.run_command(SafetyCmd::PathTree, &state);
    assert_eq!(panel.panes.mode, crate::status_pane::FileListMode::Tree);
    // History has no panes: those rows say why.
    panel.set_tab(Tab::History);
    let (_, why) = panel.run_command(SafetyCmd::NextPane, &state);
    assert!(why.is_some());
    let (_, why) = panel.run_command(SafetyCmd::ArrangeDialog, &state);
    assert!(why.is_none(), "the frame works in every view");
}

#[test]
fn ctrl_n_badges_name_the_view_tabs_and_the_panes_and_a_press_jumps_there() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    let targets = panel.nav_targets();
    // Three view tabs and three panes.
    assert_eq!(targets.len(), 6, "{targets:?}");
    let diff = panel.hit_rect(Hit::Pane(Pane::Diff));
    let cell = targets
        .iter()
        .find(|(_, at)| at.1 == diff.y && at.0 > diff.x)
        .expect("a badge on the diff pane")
        .1;
    panel.on_mouse(down(cell.0, cell.1));
    panel.on_mouse(up(cell.0, cell.1));
    assert_eq!(panel.panes.focus, Pane::Diff);
    let tab = panel.hit_rect(Hit::Tab(Tab::History));
    let cell = targets
        .iter()
        .find(|(_, at)| *at == (tab.x, tab.y))
        .unwrap()
        .1;
    panel.on_mouse(down(cell.0, cell.1));
    assert_eq!(panel.tab, Tab::History);
}

#[test]
fn enter_never_runs_an_action_it_only_navigates_in_the_file_list() {
    let mut panel = lost_panel();
    panel.paint_at(SCREEN);
    assert!(matches!(
        panel.on_key(key(KeyCode::Enter)),
        PanelOutcome::Stay
    ));
    assert!(
        panel.overlay.is_none() && panel.job.is_none(),
        "Enter on the commit does nothing"
    );
    panel.on_key(key(KeyCode::Tab));
    panel.on_key(key(KeyCode::Enter));
    assert!(panel.overlay.is_none() && panel.job.is_none());
    assert_eq!(
        panel.panes.focus,
        Pane::Diff,
        "Enter on a file goes to its hunks"
    );
}

#[test]
fn every_action_button_shows_the_letter_that_runs_it() {
    let mut panel = lost_panel();
    let text = buffer_text(&panel.paint_at(SCREEN));
    for want in [
        "Create branch here [b]",
        "Create tag here [t]",
        "Cherry-pick [c]",
        "Apply to working tree [w]",
        "Save as stash [s]",
    ] {
        assert!(text.contains(want), "{want:?} missing\n{text}");
    }
    // The letter on a button is the one the key map runs.
    for (act, tab) in [
        (Act::CreateBranch, Tab::Lost),
        (Act::SaveAsStash, Tab::Reflog),
        (Act::ShowInLog, Tab::Reflog),
        (Act::RestoreRef, Tab::Reflog),
    ] {
        let c = super::input::key_of(act, tab).expect("a letter");
        assert_eq!(super::input::act_for_key(c, tab), Some(act));
    }
    assert_eq!(super::input::key_of(Act::ShowInLog, Tab::Reflog), Some('l'));
}

/// Every shipped language's letters: one character each, never Enter, unique
/// per view (a Cyrillic letter also counts as its Latin twin, the key it
/// shares on a Latin layout) and clear of the view and navigation keys.
#[test]
fn action_letters_are_unique_single_chars_in_every_language_and_view() {
    use super::input::{acts_in, letter_in};
    use crate::guide_dialog::latin_twin;
    use gitpoulpi_i18n::{msg::undo_safety as m, SHIPPED};
    // `1 2 3` switch views, `q` closes, `v` toggles the diff mode, `j k` move,
    // `G` goes to the end, `.` opens the menu.
    const RESERVED: [char; 9] = ['1', '2', '3', 'q', 'v', 'j', 'k', 'g', '.'];
    assert!(!SHIPPED.is_empty());
    for &lang in SHIPPED {
        for msg in [
            m::KEY_UNDO_TO,
            m::KEY_REDO_TO,
            m::KEY_RESTORE_ONLY,
            m::KEY_OFFER,
            m::KEY_RESTORE_REF,
            m::KEY_SHOW_IN_LOG,
            m::KEY_CREATE_BRANCH,
            m::KEY_CREATE_TAG,
            m::KEY_APPLY,
            m::KEY_STASH,
            m::KEY_LOAD_MORE,
            m::KEY_CHERRY_PICK,
        ] {
            let text = msg.get_in(lang);
            assert_eq!(text.chars().count(), 1, "{}: {text:?}", lang.code());
            assert!(
                !text.chars().any(|c| c.is_control() || c.is_whitespace()),
                "{}: {text:?} is not a typeable letter",
                lang.code()
            );
        }
        for tab in [Tab::History, Tab::Reflog, Tab::Lost] {
            let mut seen: Vec<(char, Act)> = Vec::new();
            for &act in acts_in(tab) {
                let k = letter_in(act, lang);
                for c in [Some(k), latin_twin(k)].into_iter().flatten() {
                    assert!(
                        !RESERVED.contains(&c),
                        "{} {tab:?}: {act:?} uses the reserved key {c:?}",
                        lang.code()
                    );
                    if let Some((_, other)) = seen.iter().find(|(s, _)| *s == c) {
                        panic!(
                            "{} {tab:?}: {act:?} and {other:?} share the key {c:?}",
                            lang.code()
                        );
                    }
                    seen.push((c, act));
                }
            }
        }
    }
}
