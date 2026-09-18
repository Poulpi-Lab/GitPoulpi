//! Buffer tests: hunks, the columns, grouping, the card's dwell, the keys.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use gitpoulpi_core::domain::{Blame, BlameContent, BlameLine, Oid};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::paint::Drawn;
use super::*;
use crate::avatar::LogAvatarMode;
use crate::daemon_link::{FileInsightKind, FileInsightPayload, FileInsightRequest};
use crate::keymap::Keymap;

const NOW: i64 = 1_700_000_000 + 90 * 86_400;

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn line(commit: u8, author: &str, ts: i64, summary: &str, n: usize) -> BlameLine {
    BlameLine {
        commit: oid(commit),
        author_name: author.into(),
        author_email: format!("{}@example.com", author.to_lowercase().replace(' ', ".")),
        timestamp: ts,
        summary: summary.into(),
        final_line: n as u32 + 1,
        orig_line: n as u32 + 1,
        content: format!("let code_{n} = {n};"),
        committer_time: ts,
        orig_path: PathBuf::from("src/lib.rs"),
        previous: None,
    }
}

/// Lines 0-2 by Ada (old), 3-4 by Bob (newer), 5 by Ada again, 6 uncommitted.
fn lines() -> Vec<BlameLine> {
    let mut out = Vec::new();
    for n in 0..3 {
        out.push(line(
            1,
            "Ada Lovelace",
            1_700_000_000,
            "first commit by Ada",
            n,
        ));
    }
    for n in 3..5 {
        out.push(line(
            2,
            "Bob Builder",
            1_700_000_000 + 60 * 86_400,
            "second by Bob",
            n,
        ));
    }
    out.push(line(
        3,
        "Ada Lovelace",
        1_700_000_000 + 80 * 86_400,
        "Ada comes back",
        5,
    ));
    let mut wip = line(0, "Not committed yet", 0, "", 6);
    wip.author_email = String::new();
    out.push(wip);
    out
}

fn insight() -> FileInsight {
    let request = FileInsightRequest {
        path: PathBuf::from("src/lib.rs"),
        at: oid(9),
        kind: FileInsightKind::Blame,
        worktree: false,
        staged: false,
    };
    let mut insight = FileInsight::loading(request.clone());
    insight.accept(
        &request,
        FileInsightPayload::Blame(Blame {
            path: PathBuf::from("src/lib.rs"),
            at: oid(9),
            size: 0,
            content: BlameContent::Lines(lines()),
        }),
    );
    insight
}

fn draw(
    insight: &mut FileInsight,
    width: u16,
    hovered: Option<BlameHit>,
    now: Instant,
) -> (Buffer, Drawn) {
    let area = Rect::new(0, 0, width, 12);
    let mut buf = Buffer::empty(area);
    let keys = Keymap::defaults();
    let mut env = BlameEnv {
        hovered,
        now,
        now_secs: NOW,
        mode: LogAvatarMode::Off,
        fallback: gitpoulpi_config::AvatarFallback::Initials,
        keys: &keys,
        pictures: None,
    };
    let drawn = render(&mut buf, area, insight, &mut env);
    (buf, drawn)
}

fn row(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect()
}

fn screen(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| row(buf, y))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The live settings are process-wide: one test at a time touches them.
static SETTINGS: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn with_side<R>(left: bool, f: impl FnOnce() -> R) -> R {
    let _guard = SETTINGS.lock().unwrap_or_else(|e| e.into_inner());
    let mut ui = gitpoulpi_config::UiConfig::default();
    if left {
        ui.blame_side = gitpoulpi_config::BlameSide::Left;
    }
    crate::live_settings::install_blame(&ui);
    let out = f();
    crate::live_settings::install_blame(&gitpoulpi_config::UiConfig::default());
    out
}

#[test]
fn hunks_group_consecutive_lines_of_one_commit() {
    let (hunks, hunk_of) = hunks::build(&lines());
    let spans: Vec<(usize, usize)> = hunks.iter().map(|h| (h.start, h.end)).collect();
    assert_eq!(spans, [(0, 3), (3, 5), (5, 6), (6, 7)]);
    assert_eq!(hunk_of, [0, 0, 0, 1, 1, 2, 3]);
    assert!(hunks[3].uncommitted && !hunks[0].uncommitted);
    // Ada's two hunks share her color; Bob has his own.
    assert_eq!(hunks[0].color, hunks[2].color);
    assert_ne!(hunks[0].color, hunks[1].color);
    // Oldest is cold (0), newest warm (1).
    assert_eq!(hunks[0].heat, 0.0);
    assert_eq!(hunks[2].heat, 1.0);
    assert!(hunks[1].heat > 0.0 && hunks[1].heat < 1.0);
}

#[test]
fn old_is_cold_and_recent_is_warm() {
    let (ro, _, bo) = rgb(hunks::heat_color(0.0));
    let (rn, _, bn) = rgb(hunks::heat_color(1.0));
    assert!(bo > ro, "old is blue");
    assert!(rn > bn, "new is warm");
}

fn rgb(c: ratatui::style::Color) -> (u8, u8, u8) {
    match c {
        ratatui::style::Color::Rgb(r, g, b) => (r, g, b),
        other => panic!("not rgb: {other:?}"),
    }
}

#[test]
fn the_summary_is_shown_once_per_hunk_not_on_every_line() {
    with_side(false, || {
        let mut insight = insight();
        let (buf, _) = draw(&mut insight, 100, None, Instant::now());
        let text = screen(&buf);
        // The 3-line hunk by Ada: its summary once, its author once.
        assert_eq!(text.matches("first commit by Ada").count(), 1, "{text}");
        assert_eq!(text.matches("second by Bob").count(), 1, "{text}");
        // Both of Ada's hunks name her once each.
        assert_eq!(text.matches("Ada Lovelace").count(), 2, "{text}");
        // Line numbers and code are there for every line.
        for n in 0..6 {
            assert!(text.contains(&format!("let code_{n} = {n};")), "{text}");
        }
        // Uncommitted lines say so.
        assert!(text.contains("Not committed yet"), "{text}");
    });
}

#[test]
fn the_annotation_is_on_the_right_by_default_and_on_the_left_when_asked() {
    let col_of = |text: &str, needle: &str| {
        text.lines()
            .find_map(|l| l.find(needle))
            .unwrap_or_else(|| panic!("no {needle} in {text}"))
    };
    let right = with_side(false, || {
        let mut insight = insight();
        screen(&draw(&mut insight, 110, None, Instant::now()).0)
    });
    assert!(
        col_of(&right, "Ada Lovelace") > col_of(&right, "let code_0"),
        "right side:\n{right}"
    );
    let left = with_side(true, || {
        let mut insight = insight();
        screen(&draw(&mut insight, 110, None, Instant::now()).0)
    });
    assert!(
        col_of(&left, "Ada Lovelace") < col_of(&left, "let code_0"),
        "left side:\n{left}"
    );
}

#[test]
fn a_hunk_that_starts_above_the_view_repeats_its_annotation_on_the_first_visible_row() {
    with_side(false, || {
        let mut insight = insight();
        insight.offset = 1;
        insight.selected = 1;
        let (buf, _) = draw(&mut insight, 100, None, Instant::now());
        // Row 0 of the screen is file line 1, inside Ada's first hunk.
        assert!(row(&buf, 0).contains("Ada Lovelace"), "{}", screen(&buf));
    });
}

#[test]
fn the_card_opens_after_the_dwell_and_not_before() {
    with_side(false, || {
        let mut insight = insight();
        let t0 = Instant::now();
        // The pointer arrives on Ada's hunk: no card yet.
        let (buf, _) = draw(&mut insight, 110, Some(BlameHit::Annot(1)), t0);
        assert!(!screen(&buf).contains(&oid(1).to_hex()), "{}", screen(&buf));
        assert!(insight.blame_hover_deadline(t0).is_some());
        // Resting on it past the delay: the full sha, author + email, actions.
        let (buf, drawn) = draw(
            &mut insight,
            110,
            Some(BlameHit::Annot(1)),
            t0 + HOVER_DELAY + Duration::from_millis(5),
        );
        let text = screen(&buf);
        assert!(text.contains(&oid(1).to_hex()), "{text}");
        assert!(text.contains("ada.lovelace@example.com"), "{text}");
        assert!(text.contains("Show commit in Log"), "{text}");
        assert!(text.contains("Copy commit SHA"), "{text}");
        // Its rows come out as popup hits, the body last.
        let ops: Vec<BlameOp> = drawn
            .popup_hits
            .iter()
            .filter_map(|(_, h)| match *h {
                BlameHit::Op(op) => Some(op),
                _ => None,
            })
            .collect();
        assert_eq!(ops, BlameOp::POPUP);
        assert_eq!(
            drawn.popup_hits.last().map(|(_, h)| *h),
            Some(BlameHit::Card)
        );
        // Pointing at the card keeps it up; leaving drops it.
        let (buf, _) = draw(
            &mut insight,
            110,
            Some(BlameHit::Card),
            t0 + HOVER_DELAY + Duration::from_millis(50),
        );
        assert!(screen(&buf).contains(&oid(1).to_hex()));
        let (buf, _) = draw(&mut insight, 110, None, t0 + Duration::from_secs(1));
        assert!(!screen(&buf).contains(&oid(1).to_hex()));
    });
}

#[test]
fn the_pinned_card_follows_the_selected_hunk() {
    with_side(false, || {
        let mut insight = insight();
        insight.selected = 3;
        insight.blame.as_mut().unwrap().pinned = true;
        let (buf, _) = draw(&mut insight, 110, None, Instant::now());
        let text = screen(&buf);
        assert!(text.contains(&oid(2).to_hex()), "{text}");
        assert!(text.contains("bob.builder@example.com"), "{text}");
    });
}

#[test]
fn an_uncommitted_hunk_has_no_actions() {
    with_side(false, || {
        let mut insight = insight();
        insight.selected = 6;
        insight.blame.as_mut().unwrap().pinned = true;
        let (_, drawn) = draw(&mut insight, 110, None, Instant::now());
        assert!(!drawn
            .popup_hits
            .iter()
            .any(|(_, h)| matches!(h, BlameHit::Op(_))));
        assert!(insight.blame_selected_commit().is_none());
    });
}

#[test]
fn hunk_navigation_moves_between_hunks_and_between_hunks_of_one_commit() {
    let mut insight = insight();
    insight.blame_nav(BlameOp::NextHunk, 10).unwrap();
    assert_eq!(insight.selected, 3);
    insight.blame_nav(BlameOp::NextHunk, 10).unwrap();
    assert_eq!(insight.selected, 5);
    // Previous inside a hunk goes to its start, then to the one before.
    insight.selected = 4;
    insight.blame_nav(BlameOp::PrevHunk, 10).unwrap();
    assert_eq!(insight.selected, 3);
    insight.blame_nav(BlameOp::PrevHunk, 10).unwrap();
    assert_eq!(insight.selected, 0);
    assert!(insight.blame_nav(BlameOp::PrevHunk, 10).is_err());
    // Ada's other hunk shares no commit id here (1 and 3 differ): none.
    assert!(insight.blame_nav(BlameOp::NextSame, 10).is_err());
}

#[test]
fn next_hunk_of_the_same_commit_skips_other_commits() {
    let mut lines = lines();
    lines[5].commit = oid(1);
    let request = FileInsightRequest {
        path: PathBuf::from("a.rs"),
        at: oid(9),
        kind: FileInsightKind::Blame,
        worktree: false,
        staged: false,
    };
    let mut insight = FileInsight::loading(request.clone());
    insight.accept(
        &request,
        FileInsightPayload::Blame(Blame {
            path: PathBuf::from("a.rs"),
            at: oid(9),
            size: 0,
            content: BlameContent::Lines(lines),
        }),
    );
    insight.blame_nav(BlameOp::NextSame, 10).unwrap();
    assert_eq!(insight.selected, 5);
    insight.blame_nav(BlameOp::PrevSame, 10).unwrap();
    assert_eq!(insight.selected, 0);
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn go_to_line_takes_digits_and_enter() {
    let km = Keymap::defaults();
    let mut insight = insight();
    assert_eq!(
        insight.blame_key(&km, &key(KeyCode::Char('g')), 10),
        KeyOutcome::Handled
    );
    for c in ['5', 'x', '7'] {
        // A non-digit is swallowed by the prompt too.
        assert_eq!(
            insight.blame_key(&km, &key(KeyCode::Char(c)), 10),
            KeyOutcome::Handled
        );
    }
    assert_eq!(insight.blame.as_ref().unwrap().goto.as_deref(), Some("57"));
    insight.blame_key(&km, &key(KeyCode::Backspace), 10);
    assert_eq!(
        insight.blame_key(&km, &key(KeyCode::Enter), 10),
        KeyOutcome::Handled
    );
    assert_eq!(insight.selected, 4, "line 5 is index 4");
    assert!(insight.blame.as_ref().unwrap().goto.is_none());
}

#[test]
fn the_card_key_toggles_and_esc_closes_it_before_anything_else() {
    let km = Keymap::defaults();
    let mut insight = insight();
    insight.blame_key(&km, &key(KeyCode::Char('i')), 10);
    assert!(insight.blame.as_ref().unwrap().pinned);
    assert_eq!(
        insight.blame_key(&km, &key(KeyCode::Esc), 10),
        KeyOutcome::Handled
    );
    assert!(!insight.blame.as_ref().unwrap().pinned);
    // With nothing open, Esc is the generic handler's (it closes the view).
    assert_eq!(
        insight.blame_key(&km, &key(KeyCode::Esc), 10),
        KeyOutcome::Pass
    );
}

#[test]
fn action_keys_become_ops_and_enter_is_left_to_the_log_jump() {
    let km = Keymap::defaults();
    let mut insight = insight();
    for (c, op) in [
        ('b', BlameOp::BlameParent),
        ('d', BlameOp::ShowDiff),
        ('n', BlameOp::NextHunk),
    ] {
        assert_eq!(
            insight.blame_key(&km, &key(KeyCode::Char(c)), 10),
            KeyOutcome::Run(op)
        );
    }
    assert_eq!(
        insight.blame_key(&km, &key(KeyCode::Enter), 10),
        KeyOutcome::Pass
    );
    assert_eq!(
        insight.blame_key(&km, &key(KeyCode::Char('j')), 10),
        KeyOutcome::Pass
    );
}

#[test]
fn the_menu_opens_on_a_committed_hunk_only_and_enter_runs_the_row() {
    let km = Keymap::defaults();
    let mut insight = insight();
    assert!(!insight.blame_open_menu(6, 5, 5, 10), "uncommitted");
    assert!(insight.blame_open_menu(1, 5, 5, 10));
    assert_eq!(insight.selected, 1);
    insight.blame_key(&km, &key(KeyCode::Down), 10);
    assert_eq!(
        insight.blame_key(&km, &key(KeyCode::Enter), 10),
        KeyOutcome::Run(BlameOp::CopySha)
    );
    assert!(insight.blame.as_ref().unwrap().menu.is_none());
}

#[test]
fn the_border_drag_resizes_and_reports_the_final_width() {
    let _guard = SETTINGS.lock().unwrap_or_else(|e| e.into_inner());
    crate::live_settings::install_blame(&gitpoulpi_config::UiConfig::default());
    let mut insight = insight();
    let (_, drawn) = draw(&mut insight, 110, None, Instant::now());
    let handle = drawn
        .hits
        .iter()
        .find(|(_, h)| *h == BlameHit::Handle)
        .map(|(r, _)| *r)
        .expect("the border is a hit target");
    use crossterm::event::{MouseButton::Left, MouseEventKind as K};
    assert_eq!(
        insight.blame_mouse(K::Down(Left), handle.x, Some(BlameHit::Handle)),
        MouseOutcome::Handled
    );
    // Dragging 10 cells to the left widens a right-hand column by 10.
    // (the border sits where the clamped, displayed column ends).
    let before = super::layout::layout(Rect::new(0, 0, 110, 12), 7, false, 46).ann_w;
    insight.blame_mouse(K::Drag(Left), handle.x - 10, None);
    assert_eq!(crate::live_settings::blame_width(), before + 10);
    assert_eq!(
        insight.blame_mouse(K::Up(Left), handle.x - 10, None),
        MouseOutcome::Width(before + 10)
    );
    crate::live_settings::install_blame(&gitpoulpi_config::UiConfig::default());
}

#[test]
fn a_narrow_pane_drops_the_annotations_but_keeps_the_code() {
    with_side(false, || {
        let mut insight = insight();
        let (buf, drawn) = draw(&mut insight, 34, None, Instant::now());
        let text = screen(&buf);
        assert!(!text.contains("Ada Lovelace"), "{text}");
        assert!(text.contains("let code_0"), "{text}");
        assert!(!drawn
            .hits
            .iter()
            .any(|(_, h)| matches!(h, BlameHit::Annot(_))));
    });
}

#[test]
fn the_parent_target_is_the_previous_revision_and_path() {
    let mut insight = insight();
    // Bob's commit renamed the file: its previous revision has another path.
    if let FileInsightData::Blame(Blame {
        content: BlameContent::Lines(lines),
        ..
    }) = &mut insight.data
    {
        for l in lines.iter_mut().skip(3).take(2) {
            l.previous = Some((oid(7), PathBuf::from("old/lib.rs")));
        }
    }
    insight.selected = 3;
    let request = insight.blame_parent_target().unwrap();
    assert_eq!(request.at, oid(7));
    assert_eq!(request.kind, FileInsightKind::Blame);
    assert_eq!(request.path, PathBuf::from("old/lib.rs"));
    // A commit that added the file has nothing earlier.
    insight.selected = 0;
    assert!(insight.blame_parent_target().is_err());
}

#[test]
fn a_70_column_pane_keeps_the_code_and_goes_compact() {
    for width in [70u16, 80] {
        with_side(false, || {
            let mut insight = insight();
            let (buf, drawn) = draw(&mut insight, width, None, Instant::now());
            let text = screen(&buf);
            assert!(text.contains("let code_0 = 0;"), "{width}: {text}");
            assert!(text.contains("Ada"), "{width}: {text}");
            let lay = super::layout::layout(Rect::new(0, 0, width, 12), 7, false, 46);
            assert!(lay.text_w >= super::layout::MIN_TEXT, "{width}: {lay:?}");
            if width == 70 {
                // Compact: no second line with the summary under the author.
                assert!(!text.contains("first commit by Ada"), "{text}");
            }
            assert!(drawn.hits.iter().any(|(_, h)| *h == BlameHit::Handle));
        });
    }
}

#[test]
fn the_user_width_is_only_clamped_for_display() {
    let _guard = SETTINGS.lock().unwrap_or_else(|e| e.into_inner());
    crate::live_settings::install_blame(&gitpoulpi_config::UiConfig::default());
    let want = crate::live_settings::blame_width();
    let mut insight = insight();
    let _ = draw(&mut insight, 70, None, Instant::now());
    assert_eq!(crate::live_settings::blame_width(), want);
    assert!(super::layout::layout(Rect::new(0, 0, 70, 12), 7, false, want).ann_w < want);
}
