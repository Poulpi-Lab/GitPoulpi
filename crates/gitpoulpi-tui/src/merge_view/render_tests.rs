//! Goldens and hit-rect tests for the merge editor's paint walk.
//!
//! The goldens are plain text (one file per layout under `golden/`): the
//! cell symbols of a `TestBackend` frame, trailing blanks trimmed.
//! Regenerate after an intended change with
//! `GITPOULPI_UPDATE_GOLDEN=1 cargo test -p gitpoulpi-tui --lib merge_view`.

use crate::th;
use gitpoulpi_core::domain::{
    BlobContent, ConflictEntry, ConflictSide, Eol, MergeDocument, MergeRegion, Oid, SideBlob,
    SideLabels, StageEntry, WorktreeFileState,
};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::Terminal;

use super::layout::{self, Split};
use super::model::MergeModel;
use super::{MergeCmd, MergeHit, MergePane, MergeScroll, MergeView};
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

fn blob(n: u8, content: Vec<String>) -> Option<SideBlob> {
    Some(SideBlob {
        oid: Oid([n; 20]),
        mode: 0o100644,
        size: 0,
        content: BlobContent::Text {
            lines: content,
            crlf: false,
            final_newline: true,
        },
    })
}

/// The side's file: the merged text with that side's block in every
/// conflict, then `edit` applied (what the other side changed that git
/// merged in silently).
fn projection(regions: &[MergeRegion], side: ConflictSide) -> Vec<String> {
    let mut out = Vec::new();
    for r in regions {
        out.extend(r.lines_of(side).iter().cloned());
    }
    out
}

/// Three conflicts and some auto-merged lines (B changed a line A keeps
/// the old form of; B added a comment A lacks):
///
/// 0 both sides touched, 1 a short one, 2 a one-line pair.
fn regions() -> Vec<MergeRegion> {
    vec![
        merged(&[
            "use std::io;",
            "",
            "fn login(&mut self, user: &str) {",
            "    let cfg = self.cfg.clone();",
        ]),
        conflict(
            &["    let token = get();"],
            &[
                "    let token = self.auth.token()?;",
                "    self.audit(user);",
            ],
            &[
                "    let token = fetch(user)?;",
                "    log::info!(\"login {user}\");",
                "    self.metrics.inc();",
            ],
        ),
        merged(&["    Ok(token)", "    // B only", "}", ""]),
        conflict(&["fn old() {}"], &["fn a() {}"], &["fn b() {}"]),
        merged(&["// end"]),
        conflict(&[], &["const A: u8 = 1;"], &["const B: u8 = 2;"]),
        merged(&["// tail"]),
    ]
}

fn document(final_newline: bool, eol: Eol) -> MergeDocument {
    let regions = regions();
    let mut ours = projection(&regions, Ours);
    let mut theirs = projection(&regions, Theirs);
    let mut base = projection(&regions, Base);
    // B changed the cfg line; A still has the old one.
    ours[3] = "    let cfg = self.cfg();".to_string();
    base[3] = "    let cfg = self.cfg();".to_string();
    // B added a comment; A and the base lack it.
    let at = ours.iter().position(|l| l == "    // B only").unwrap();
    ours.remove(at);
    let at = base.iter().position(|l| l == "    // B only").unwrap();
    base.remove(at);
    theirs[3] = "    let cfg = self.cfg.clone();".to_string();
    MergeDocument {
        entry: ConflictEntry::from_stages("src/auth/login.rs".into(), stage(1), stage(2), stage(3))
            .unwrap(),
        base: blob(1, base),
        ours: blob(2, ours),
        theirs: blob(3, theirs),
        regions,
        labels: SideLabels {
            current: "Current \u{b7} main (HEAD)".into(),
            incoming: "Incoming \u{b7} feature/login".into(),
            base: "Base \u{b7} merge-base 9f8e7d6".into(),
            description: "merging feature/login into main".into(),
            ..Default::default()
        },
        worktree: WorktreeFileState::Markers,
        eol,
        bom: false,
        final_newline,
        encoding: None,
        marker_size: 7,
    }
}

/// Conflict 0: both sides, B first, one of A's lines left out; conflict 1:
/// A then typed over; conflict 2: untouched.
fn fixture_model(final_newline: bool, eol: Eol) -> MergeModel {
    let mut m = MergeModel::new(document(final_newline, eol));
    m.take_b_then_a(0);
    m.toggle_line(0, Ours, 1);
    m.take_a(1);
    let (first, _) = m.region_lines(1).unwrap();
    let at = m.output().line_start(first) + 5;
    m.edit(|d| {
        d.set_cursor(at, false);
        d.insert_text("!");
    });
    m
}

pub(crate) fn fixture() -> MergeView {
    let mut v = MergeView::new(fixture_model(true, Eol::Lf));
    v.file_pos = (2, 3);
    v
}

fn draw(v: &mut MergeView, w: u16, h: u16) -> (Buffer, Vec<(Rect, MergeHit)>) {
    draw_hovered(v, w, h, None)
}

fn draw_hovered(
    v: &mut MergeView,
    w: u16,
    h: u16,
    hovered: Option<MergeHit>,
) -> (Buffer, Vec<(Rect, MergeHit)>) {
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|frame| hits = v.render(frame, Rect::new(0, 0, w, h), hovered))
        .unwrap();
    (terminal.backend().buffer().clone(), hits)
}

fn text(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in 0..buf.area.height {
        let row: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
        out.push_str(row.trim_end());
        out.push('\n');
    }
    out
}

fn golden(name: &str, buf: &Buffer) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/merge_view/golden")
        .join(format!("{name}.txt"));
    let got = text(buf);
    if std::env::var_os("GITPOULPI_UPDATE_GOLDEN").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing golden {path:?}; run with GITPOULPI_UPDATE_GOLDEN=1"));
    assert_eq!(
        got, want,
        "golden {name} differs (GITPOULPI_UPDATE_GOLDEN=1 regenerates)"
    );
}

fn hit_rects(hits: &[(Rect, MergeHit)], which: MergeHit) -> Vec<Rect> {
    hits.iter()
        .filter(|(_, h)| *h == which)
        .map(|(r, _)| *r)
        .collect()
}

fn has(hits: &[(Rect, MergeHit)], which: MergeHit) -> bool {
    !hit_rects(hits, which).is_empty()
}

// -- goldens --------------------------------------------------------------

#[test]
fn golden_wide_two_panes() {
    let (buf, _) = draw(&mut fixture(), 110, 30);
    golden("merge_wide", &buf);
}

#[test]
fn golden_wide_second_conflict_current() {
    let mut v = fixture();
    v.model.go_to(1);
    let (buf, _) = draw(&mut v, 110, 30);
    golden("merge_wide_current_2", &buf);
}

#[test]
fn golden_wide_base_shown() {
    let mut v = fixture();
    v.show_base = true;
    let (buf, _) = draw(&mut v, 140, 30);
    golden("merge_wide_base", &buf);
}

/// A | B side by side from 60 columns up (owner), the panes
/// just narrower; both sides' lines and the pick gutters are on screen.
#[test]
fn golden_side_by_side_at_80_and_62_columns() {
    for (w, h, name) in [(80u16, 24u16, "merge_wide_80"), (62, 20, "merge_wide_62")] {
        let (buf, hits) = super::layout::tests_override::with(60, || draw(&mut fixture(), w, h));
        let t = text(&buf);
        assert!(
            t.contains("A \u{b7}") && t.contains("B \u{b7}"),
            "both panes at {w}x{h}\n{t}"
        );
        assert!(!has(&hits, MergeHit::Tab(Ours)), "no tab strip at {w}x{h}");
        golden(name, &buf);
    }
}

/// Long lines scroll sideways, A and B together, and are never wrapped: the
/// tail of a line cut at 62 columns shows after a step to the right, and
/// scrolling back shows the start again.
#[test]
fn the_panes_scroll_sideways_together_and_never_wrap() {
    super::layout::tests_override::with(60, || {
        let mut v = fixture();
        let (buf, _) = draw(&mut v, 62, 20);
        let before = text(&buf);
        // (The Output below shows the line whole; a row with both panes'
        // copies would hold two.)
        assert!(
            !before
                .lines()
                .any(|l| l.matches("user: &str) {").count() == 2),
            "cut at the panes' edge\n{before}"
        );
        v.scroll_h_by(16);
        let (buf, _) = draw(&mut v, 62, 20);
        let after = text(&buf);
        // The same row of both panes shows the shifted text.
        let rows: Vec<&str> = after
            .lines()
            .filter(|l| l.contains("user: &str) {"))
            .collect();
        assert!(
            rows.iter().any(|l| l.matches("user: &str) {").count() == 2),
            "A and B scrolled together\n{after}"
        );
        v.scroll_h_by(-1000);
        assert_eq!(v.h_scroll, 0, "back to the start");
        v.scroll_h_by(1000);
        assert!(
            v.h_scroll < 60,
            "clamped by the widest line: {}",
            v.h_scroll
        );
    });
}

/// Shift+Right / Shift+Left (and Shift+L / Shift+H) scroll the focused top
/// pane's text sideways, both panes together.
#[test]
fn shift_arrows_scroll_the_panes_sideways() {
    use crate::clipboard::Clipboard;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let mut v = fixture();
    let mut clip = Clipboard::new();
    draw(&mut v, 100, 30);
    let key = |code, mods| KeyEvent::new(code, mods);
    v.on_key(&key(KeyCode::Right, KeyModifiers::SHIFT), &mut clip);
    assert_eq!(v.h_scroll, 8);
    v.on_key(&key(KeyCode::Char('L'), KeyModifiers::SHIFT), &mut clip);
    assert_eq!(v.h_scroll, 16);
    v.on_key(&key(KeyCode::Left, KeyModifiers::SHIFT), &mut clip);
    v.on_key(&key(KeyCode::Char('H'), KeyModifiers::SHIFT), &mut clip);
    assert_eq!(v.h_scroll, 0);
    v.on_key(&key(KeyCode::Left, KeyModifiers::SHIFT), &mut clip);
    assert_eq!(v.h_scroll, 0, "not below the start");
}

/// Shift+wheel, a sideways wheel and the keys all scroll the top panes, and
/// only them.
#[test]
fn shift_wheel_and_the_keys_scroll_the_top_panes_sideways() {
    use crossterm::event::{KeyModifiers, MouseEventKind};
    super::layout::tests_override::with(60, || {
        let mut v = fixture();
        draw(&mut v, 100, 30);
        let top = v.areas.top;
        let inside = (top.x + 3, top.y + 1);
        assert!(!v.hscroll_mouse(
            MouseEventKind::ScrollDown,
            KeyModifiers::NONE,
            inside.0,
            inside.1
        ));
        assert!(v.hscroll_mouse(
            MouseEventKind::ScrollDown,
            KeyModifiers::SHIFT,
            inside.0,
            inside.1
        ));
        assert_eq!(v.h_scroll, 8);
        assert!(v.hscroll_mouse(
            MouseEventKind::ScrollRight,
            KeyModifiers::NONE,
            inside.0,
            inside.1
        ));
        assert_eq!(v.h_scroll, 16);
        assert!(v.hscroll_mouse(
            MouseEventKind::ScrollLeft,
            KeyModifiers::NONE,
            inside.0,
            inside.1
        ));
        assert_eq!(v.h_scroll, 8);
        // Over the Output it is not the top panes' business.
        let out = v.areas.out;
        assert!(!v.hscroll_mouse(
            MouseEventKind::ScrollRight,
            KeyModifiers::NONE,
            out.x + 2,
            out.y
        ));
    });
}

#[test]
fn golden_narrow_tab_a() {
    let (buf, _) = draw(&mut fixture(), 50, 24);
    golden("merge_narrow_a", &buf);
}

#[test]
fn golden_narrow_tab_b() {
    let mut v = fixture();
    v.tab = Theirs;
    let (buf, _) = draw(&mut v, 50, 24);
    golden("merge_narrow_b", &buf);
}

#[test]
fn golden_narrow_tab_base() {
    let mut v = fixture();
    v.tab = Base;
    let (buf, _) = draw(&mut v, 50, 24);
    golden("merge_narrow_base", &buf);
}

#[test]
fn golden_too_small() {
    let (buf, hits) = draw(&mut fixture(), 50, 12);
    golden("merge_too_small", &buf);
    assert!(
        has(&hits, MergeHit::Close),
        "the notice still has a way out"
    );
}

#[test]
fn golden_crlf_and_no_final_newline() {
    let mut v = MergeView::new(fixture_model(false, Eol::Crlf));
    v.model.go_to(2);
    // Scroll to the end so the note is on screen.
    draw(&mut v, 110, 24);
    v.scroll_output_by(1000);
    let (buf, _) = draw(&mut v, 110, 24);
    let t = text(&buf);
    assert!(t.contains("CRLF"), "the Output title names the EOL:\n{t}");
    assert!(
        t.contains("no newline at end of file"),
        "the note under the Output:\n{t}"
    );
    golden("merge_crlf_no_eol", &buf);
}

// -- content ----------------------------------------------------------------

#[test]
fn wide_shows_checkboxes_pick_gutters_badges_markers_and_origins() {
    let (buf, _) = draw(&mut fixture(), 110, 50);
    let t = text(&buf);
    // Chunk checkboxes: A has one of two lines picked, B all.
    assert!(t.contains("[-] Conflict 1"), "{t}");
    assert!(t.contains("[x] Conflict 1"), "{t}");
    // Order badges: B first, A second.
    assert!(t.contains("1st"), "{t}");
    assert!(t.contains("2nd"), "{t}");
    // Pick gutters.
    assert!(t.contains('\u{2713}'), "a picked line shows a check:\n{t}");
    assert!(t.contains('+'), "an unpicked line shows a plus:\n{t}");
    // The auto-merged lines carry their marker in the other pane.
    assert!(t.contains("\u{2591} B"), "{t}");
    // The edited conflict says so, and the Output gutter has the pencil.
    assert!(t.contains("\u{270e} edited"), "{t}");
    // The placeholder is a row of the Output, not text.
    assert!(t.contains("Conflict 3 \u{b7} nothing picked yet"), "{t}");
    // Output origin letters: B first then A for conflict 1.
    assert!(t.contains(" B     let token = fetch(user)?;"), "{t}");
    assert!(t.contains(" A     let token = self.auth.token()?;"), "{t}");
    // Header and counter.
    assert!(t.contains("file 2 of 3"), "{t}");
    assert!(t.contains("Conflict 1 / 3 \u{b7} 1 left"), "{t}");
    assert!(t.contains("merging feature/login into main"), "{t}");
}

#[test]
fn the_current_conflicts_frame_is_bright_and_the_others_dim() {
    let mut v = fixture();
    v.model.go_to(1);
    let (buf, _) = draw(&mut v, 110, 30);
    let corner = |header: &str| {
        // The `┌` of the row holding `header`, first pane.
        let y = (0..buf.area.height)
            .find(|&y| {
                let row: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect();
                row.contains(&format!("] {header}"))
            })
            .unwrap();
        let x = (0..buf.area.width)
            .find(|&x| buf[(x, y)].symbol() == "\u{250c}")
            .unwrap();
        buf[(x, y)].fg
    };
    assert_ne!(corner("Conflict 1"), corner("Conflict 2"));
}

#[test]
fn degraded_alignment_is_flagged_in_the_pane_title() {
    let mut v = fixture();
    v.aligned.degraded[1] = true;
    let (buf, _) = draw(&mut v, 110, 30);
    assert!(text(&buf).contains("A \u{b7} Current \u{b7} main (HEAD) (approximate)"));
}

#[test]
fn a_missing_base_says_so() {
    let mut doc = document(true, Eol::Lf);
    doc.base = None;
    let mut v = MergeView::new(MergeModel::new(doc));
    v.show_base = true;
    let (buf, _) = draw(&mut v, 140, 30);
    assert!(text(&buf).contains("no common ancestor (both added)"));
}

#[test]
fn the_replaced_edit_hint_names_the_keymaps_undo_key() {
    assert_eq!(
        super::render::replaced_edit_text("Ctrl+L"),
        "replaced your edit \u{b7} Ctrl+L restores"
    );
    let mut v = fixture();
    v.note = Some(super::render::replaced_edit_text("Ctrl+Z"));
    let (buf, _) = draw(&mut v, 110, 30);
    assert!(text(&buf).contains("Ctrl+Z restores"));
}

#[test]
fn nothing_is_painted_outside_the_given_area() {
    let mut v = fixture();
    let mut terminal = Terminal::new(TestBackend::new(140, 40)).unwrap();
    terminal
        .draw(|frame| {
            v.render(frame, Rect::new(10, 5, 110, 30), None);
        })
        .unwrap();
    let buf = terminal.backend().buffer();
    for y in 0..40 {
        for x in 0..140 {
            let inside = (10..120).contains(&x) && (5..35).contains(&y);
            if !inside {
                assert_eq!(buf[(x, y)].symbol(), " ", "({x},{y}) was painted");
            }
        }
    }
}

// -- hit rects ----------------------------------------------------------------

#[test]
fn wide_registers_every_toolbar_button() {
    let (_, hits) = draw(&mut fixture(), 140, 30);
    for h in [
        MergeHit::PrevConflict,
        MergeHit::NextConflict,
        MergeHit::TakeAllA,
        MergeHit::TakeAllB,
        MergeHit::ToggleBase,
        MergeHit::Save,
        MergeHit::SaveResolve,
        MergeHit::Close,
        MergeHit::PrevFile,
        MergeHit::NextFile,
        MergeHit::Splitter,
    ] {
        assert!(has(&hits, h), "no hit rect for {h:?}");
    }
    assert!(!has(&hits, MergeHit::More), "nothing is hidden at 140 cols");
}

#[test]
fn narrow_registers_the_tabs_and_the_more_menu() {
    let (_, hits) = draw(&mut fixture(), 50, 24);
    for h in [
        MergeHit::Tab(Ours),
        MergeHit::Tab(Theirs),
        MergeHit::Tab(Base),
        MergeHit::More,
        MergeHit::PrevConflict,
        MergeHit::NextConflict,
        MergeHit::SaveResolve,
        MergeHit::Close,
        MergeHit::Splitter,
    ] {
        assert!(has(&hits, h), "no hit rect for {h:?}");
    }
    assert!(!has(&hits, MergeHit::TakeAllA), "Take all moved into `...`");
    assert!(!has(&hits, MergeHit::ToggleBase));
}

#[test]
fn every_checkbox_and_every_conflict_line_has_a_target() {
    let mut v = fixture();
    v.show_base = true;
    // Tall enough to show every row.
    let (_, hits) = draw(&mut v, 150, 60);
    let blocks = [(0usize, 1usize, 2usize, 3usize), (1, 1, 1, 1), (2, 0, 1, 1)];
    for (c, base, ours, theirs) in blocks {
        for side in [Ours, Theirs, Base] {
            assert!(
                has(&hits, MergeHit::Chunk { conflict: c, side }),
                "chunk {c} {side:?}"
            );
        }
        for (side, n) in [(Base, base), (Ours, ours), (Theirs, theirs)] {
            for line in 0..n {
                assert!(
                    has(
                        &hits,
                        MergeHit::Line {
                            conflict: c,
                            side,
                            line
                        }
                    ),
                    "line {line} of {side:?} in conflict {c}"
                );
            }
        }
    }
    // The badge of the two-sided conflict, in both panes.
    assert!(hit_rects(&hits, MergeHit::Badge { conflict: 0 }).len() >= 2);
    // The Output: every visible document line and the placeholder.
    let lines = v.model.output().line_count() - 1;
    for i in 0..lines {
        assert!(has(&hits, MergeHit::OutputLine(i)), "output line {i}");
    }
    assert!(has(&hits, MergeHit::Placeholder { conflict: 2 }));
}

#[test]
fn hit_rects_stay_in_the_area_and_never_overlap() {
    for (w, h, base) in [(110u16, 30u16, false), (140, 40, true), (80, 24, false)] {
        let mut v = fixture();
        v.show_base = base;
        let (_, hits) = draw(&mut v, w, h);
        let area = Rect::new(0, 0, w, h);
        for (i, (a, ha)) in hits.iter().enumerate() {
            assert!(area.contains((a.x, a.y).into()), "{ha:?} at {a:?}");
            assert!(a.right() <= w && a.bottom() <= h, "{ha:?} at {a:?} outside");
            assert!(a.width > 0 && a.height > 0, "{ha:?} is empty");
            for (b, hb) in &hits[i + 1..] {
                assert!(
                    a.intersection(*b).area() == 0,
                    "{ha:?} {a:?} overlaps {hb:?} {b:?} at {w}x{h}"
                );
            }
        }
    }
}

#[test]
fn a_file_with_a_long_enough_body_gets_scrollbar_thumbs() {
    let mut v = fixture();
    let (_, hits) = draw(&mut v, 110, 16);
    assert!(
        has(&hits, MergeHit::Thumb(MergeScroll::Top))
            || has(&hits, MergeHit::Thumb(MergeScroll::Output))
    );
}

// -- Full file (S7b) ------------------------------------------------------------

fn full_file_view() -> MergeView {
    let mut v = fixture();
    v.full_file = true;
    v
}

/// The cell where `needle` starts on the first row of `buf` showing it
/// (searching the left half, pane A).
fn find_text(buf: &Buffer, needle: &str) -> (u16, u16) {
    for y in 0..buf.area.height {
        let row: String = (0..buf.area.width / 2)
            .map(|x| buf[(x, y)].symbol())
            .collect();
        if let Some(at) = row.find(needle) {
            return (row[..at].chars().count() as u16, y);
        }
    }
    panic!("{needle:?} not on screen:\n{}", text(buf));
}

#[test]
fn golden_wide_full_file() {
    let (buf, _) = draw(&mut full_file_view(), 110, 30);
    golden("merge_wide_full_file", &buf);
}

#[test]
fn golden_narrow_full_file() {
    let (buf, _) = draw(&mut full_file_view(), 80, 24);
    golden("merge_narrow_full_file", &buf);
}

#[test]
fn golden_wide_full_file_in_french() {
    let _lang = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let (buf, _) = draw(&mut full_file_view(), 140, 30);
    assert!(text(&buf).contains("Fichier entier"));
    golden("merge_wide_full_file_fr", &buf);
}

#[test]
fn golden_narrow_full_file_in_french() {
    let _lang = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let (buf, _) = draw(&mut full_file_view(), 80, 24);
    golden("merge_narrow_full_file_fr", &buf);
}

#[test]
fn full_file_colors_match_the_split_diffs_styles() {
    use crate::diff_view::line_marker;
    use gitpoulpi_core::domain::LineOrigin;
    let (_, add) = line_marker(&LineOrigin::Add);
    let (_, del) = line_marker(&LineOrigin::Delete);
    let mut v = MergeView::new(MergeModel::new(document(true, Eol::Lf)));
    v.full_file = true;
    v.focus = MergePane::Output;
    let (buf, _) = draw(&mut v, 110, 40);
    // Changed (replaces the base's `get()` line) and added lines of A.
    for needle in ["self.auth.token()", "self.audit(user)"] {
        let (x, y) = find_text(&buf, needle);
        assert_eq!(buf[(x, y)].bg, add.bg.unwrap(), "{needle} background");
        assert_eq!(buf[(x, y)].fg, add.fg.unwrap(), "{needle} foreground");
        assert_eq!(buf[(2, y)].symbol(), "\u{2502}", "the frame stays");
        assert_eq!(buf[(1, y)].symbol(), "+", "{needle} marker");
    }
    // An unchanged line carries no diff color.
    let (x, y) = find_text(&buf, "use std::io;");
    assert_ne!(buf[(x, y)].bg, add.bg.unwrap());
    assert_eq!(buf[(1, y)].symbol(), " ");
    // Off, nothing is colored.
    let (buf, _) = draw(
        &mut MergeView::new(MergeModel::new(document(true, Eol::Lf))),
        110,
        40,
    );
    let (x, y) = find_text(&buf, "self.audit(user)");
    assert_ne!(buf[(x, y)].bg, add.bg.unwrap());
    // A base line the side removed is flagged on the next line.
    let mut doc = document(true, Eol::Lf);
    if let Some(SideBlob {
        content: BlobContent::Text { lines, .. },
        ..
    }) = doc.ours.as_mut()
    {
        lines.remove(1);
    }
    let mut v = MergeView::new(MergeModel::new(doc));
    v.full_file = true;
    let (buf, _) = draw(&mut v, 110, 40);
    let (_, y) = find_text(&buf, "fn login");
    assert_eq!(buf[(1, y)].symbol(), "-");
    assert_eq!(buf[(1, y)].bg, del.bg.unwrap());
}

#[test]
fn the_toolbar_has_a_full_file_toggle_that_lights_when_on() {
    let (_, hits) = draw(&mut fixture(), 110, 30);
    assert!(has(&hits, MergeHit::ToggleFullFile));
    let (buf, hits) = draw(&mut fixture(), 110, 30);
    let r = hit_rects(&hits, MergeHit::ToggleFullFile)[0];
    let off = buf[(r.x, r.y)].bg;
    let (buf, hits) = draw(&mut full_file_view(), 110, 30);
    let r = hit_rects(&hits, MergeHit::ToggleFullFile)[0];
    assert_ne!(buf[(r.x, r.y)].bg, off);
}

#[test]
fn the_suggest_button_shows_only_with_ai_on_in_ai_purple_with_the_sparkle() {
    let (buf, hits) = draw(&mut fixture(), 140, 30);
    assert!(!has(&hits, MergeHit::AiSuggest), "AI off: no button");
    let mut v = fixture();
    v.ai_enabled = true;
    let (buf_on, hits) = draw(&mut v, 140, 30);
    assert!(has(&hits, MergeHit::AiSuggest));
    let r = hit_rects(&hits, MergeHit::AiSuggest)[0];
    assert_eq!(buf_on[(r.x + 1, r.y)].symbol(), "\u{2726}");
    assert_eq!(buf_on[(r.x + 1, r.y)].fg, th().status.ai);
    let _ = buf;
}

// -- hover ----------------------------------------------------------------------

fn bg_at(buf: &Buffer, rect: Rect) -> Color {
    buf[(rect.x, rect.y)].bg
}

#[test]
fn hovering_a_toolbar_button_lights_it() {
    let (buf0, hits) = draw(&mut fixture(), 140, 30);
    let r = hit_rects(&hits, MergeHit::SaveResolve)[0];
    let (buf1, _) = draw_hovered(&mut fixture(), 140, 30, Some(MergeHit::SaveResolve));
    assert_ne!(bg_at(&buf0, r), bg_at(&buf1, r));
    let r = hit_rects(&hits, MergeHit::Save)[0];
    let (buf1, _) = draw_hovered(&mut fixture(), 140, 30, Some(MergeHit::Save));
    assert_ne!(bg_at(&buf0, r), bg_at(&buf1, r));
    let r = hit_rects(&hits, MergeHit::Close)[0];
    let (buf1, _) = draw_hovered(&mut fixture(), 140, 30, Some(MergeHit::Close));
    assert_ne!(
        bg_at(&buf0, r),
        bg_at(&buf1, r),
        "the close cross is plain at rest"
    );
}

#[test]
fn hovering_a_checkbox_a_line_and_the_badge_lights_them() {
    let (buf0, hits) = draw(&mut fixture(), 140, 40);
    for hit in [
        MergeHit::Chunk {
            conflict: 1,
            side: Theirs,
        },
        MergeHit::Line {
            conflict: 0,
            side: Theirs,
            line: 1,
        },
        MergeHit::Badge { conflict: 0 },
        MergeHit::PrevFile,
    ] {
        let r = hit_rects(&hits, hit)[0];
        let (buf1, _) = draw_hovered(&mut fixture(), 140, 40, Some(hit));
        assert_ne!(bg_at(&buf0, r), bg_at(&buf1, r), "{hit:?} did not light");
    }
}

#[test]
fn hovering_the_splitter_and_an_output_row_lights_them() {
    let (buf0, hits) = draw(&mut fixture(), 110, 50);
    let r = hit_rects(&hits, MergeHit::Splitter)[0];
    let (buf1, _) = draw_hovered(&mut fixture(), 110, 50, Some(MergeHit::Splitter));
    assert_ne!(
        buf0[(r.x + r.width - 2, r.y)].fg,
        buf1[(r.x + r.width - 2, r.y)].fg
    );
    let hit = MergeHit::OutputLine(2);
    let r = hit_rects(&hits, hit)[0];
    let (buf1, _) = draw_hovered(&mut fixture(), 110, 50, Some(hit));
    assert_ne!(bg_at(&buf0, r), bg_at(&buf1, r));
    let hit = MergeHit::Placeholder { conflict: 2 };
    let r = hit_rects(&hits, hit)[0];
    let (buf1, _) = draw_hovered(&mut fixture(), 110, 50, Some(hit));
    assert_ne!(bg_at(&buf0, r), bg_at(&buf1, r));
}

#[test]
fn the_ai_slot_is_hidden_for_now() {
    // `✦ Suggest` arrives with slice S9; until then no glyph of it is drawn.
    let (buf, _) = draw(&mut fixture(), 140, 30);
    assert!(!text(&buf).contains('\u{2726}'));
}

// -- scroll sync and state ---------------------------------------------------------

#[test]
fn scrolling_a_pane_moves_the_other_through_the_region_map() {
    let mut v = fixture();
    draw(&mut v, 110, 16);
    assert_eq!((v.top_scroll, v.out_scroll), (0, 0));
    v.scroll_top_by(8);
    assert_eq!(v.top_scroll, 8);
    assert!(v.out_scroll > 0, "the Output followed");
    let after_top = v.out_scroll;
    v.scroll_output_by(-100);
    assert_eq!(v.out_scroll, 0);
    assert_eq!(v.top_scroll, 0, "and back");
    v.scroll_output_by(2);
    assert!(v.top_scroll > 0);
    assert!(after_top > 0);
}

#[test]
fn revealing_a_conflict_puts_its_header_and_region_near_the_top() {
    let mut v = fixture();
    draw(&mut v, 110, 16);
    v.model.go_to(2);
    v.reveal_current();
    let h = v.top.header_at[2];
    assert_eq!(v.top_scroll, (h - 1).min(v.top.rows.len() - v.heights.0));
    let (buf, _) = draw(&mut v, 110, 16);
    assert!(text(&buf).contains("[ ] Conflict 3"), "{}", text(&buf));
}

#[test]
fn the_split_cycles_and_changes_the_pane_heights() {
    let mut v = fixture();
    draw(&mut v, 110, 30);
    let a = v.heights;
    v.cycle_split();
    draw(&mut v, 110, 30);
    assert!(
        v.heights.1 > a.1 && v.heights.0 < a.0,
        "{a:?} -> {:?}",
        v.heights
    );
    // On a short terminal `z` toggles halves.
    let mut v = fixture();
    draw(&mut v, 80, 20);
    assert!(v.compact);
    v.cycle_split();
    draw(&mut v, 80, 20);
    assert_eq!(v.heights.1, 0);
    v.cycle_split();
    draw(&mut v, 80, 20);
    assert_eq!(v.heights.0, 0);
}

#[test]
fn focus_marks_the_pane_title_and_the_caret() {
    let mut v = fixture();
    v.focus = MergePane::Output;
    v.model.output_for_caret().set_cursor(0, false);
    let (buf, _) = draw(&mut v, 110, 30);
    // The caret cell of the Output is reversed.
    let reversed = (0..buf.area.height).any(|y| {
        (0..buf.area.width).any(|x| {
            buf[(x, y)]
                .modifier
                .contains(ratatui::style::Modifier::REVERSED)
        })
    });
    assert!(reversed);
}

#[test]
fn highlighting_catches_up_over_a_few_frames_and_reports_pending_meanwhile() {
    let mut v = fixture();
    // The first frames may not finish the grammars' lazy compilation
    // inside the 20 ms budget; the loop keeps drawing until they do.
    let mut frames = 0;
    loop {
        draw(&mut v, 110, 30);
        frames += 1;
        if !v.highlight_pending() {
            break;
        }
        assert!(frames < 500, "highlighting never caught up");
    }
    assert!(v.hl[1].highlighted() > 0, "something was highlighted");
}

/// The A and B panes are syntax-highlighted like the Output: once the
/// highlighters have caught up, a keyword of a pane's line has a color
/// other than the plain text one.
#[test]
fn the_side_panes_are_highlighted_like_the_output() {
    let mut v = fixture();
    for _ in 0..500 {
        draw(&mut v, 110, 30);
        if !v.highlight_pending() {
            break;
        }
    }
    let (buf, _) = draw(&mut v, 110, 30);
    let plain = ratatui::style::Color::Rgb(192, 197, 206);
    // The panes' columns: A on the left half, B on the right half.
    for (name, from, to) in [("A", 0u16, 54u16), ("B", 56, 110)] {
        let colorful = (0..buf.area.height).any(|y| {
            (from..to).any(|x| {
                let c = &buf[(x, y)];
                c.symbol().trim() != ""
                    && matches!(c.fg, ratatui::style::Color::Rgb(..))
                    && c.fg != plain
                    && y > 4
                    && y < 15
            })
        });
        assert!(colorful, "pane {name} has no highlighted text");
    }
}

// -- French ---------------------------------------------------------------------

#[test]
fn golden_wide_in_french() {
    let _lang = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let (buf, _) = draw(&mut fixture(), 110, 30);
    let t = text(&buf);
    assert!(t.contains("fichier 2 sur 3"), "{t}");
    assert!(t.contains("Conflit 1"), "{t}");
    golden("merge_wide_fr", &buf);
}

#[test]
fn french_pane_titles_counter_and_tallies_are_translated() {
    let _lang = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let (buf, _) = draw(&mut fixture(), 140, 30);
    let t = text(&buf);
    assert!(t.contains("Courant \u{b7} main (HEAD)"), "{t}");
    assert!(t.contains("Entrant \u{b7} feature/login"), "{t}");
    assert!(t.contains("merge de feature/login dans main"), "{t}");
    assert!(t.contains("Conflit 1 / 3 \u{b7}"), "{t}");
    assert!(t.contains("restant"), "{t}");
    assert!(!t.contains("Incoming") && !t.contains("Current"), "{t}");
    {
        let _terms = gitpoulpi_i18n::override_git_terms(true);
        let (buf, _) = draw(&mut fixture(), 140, 30);
        assert!(text(&buf).contains("fusion de feature/login dans main"));
    }
}

#[test]
fn golden_narrow_in_french() {
    let _lang = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    let (buf, _) = draw(&mut fixture(), 80, 24);
    golden("merge_narrow_fr", &buf);
}

#[test]
fn french_labels_fit_at_every_width_and_every_button_stays_reachable() {
    let _lang = gitpoulpi_i18n::override_lang(gitpoulpi_i18n::Lang::Fr);
    for (w, h) in [
        (40u16, 14u16),
        (50, 24),
        (60, 14),
        (62, 20),
        (70, 24),
        (80, 24),
        (99, 24),
        (100, 24),
        (140, 30),
    ] {
        let (_, hits) = draw(&mut fixture(), w, h);
        let area = Rect::new(0, 0, w, h);
        for (r, hit) in &hits {
            assert!(
                r.right() <= area.right() && r.bottom() <= area.bottom(),
                "{hit:?} {r:?} at {w}x{h}"
            );
        }
        for need in [
            MergeHit::SaveResolve,
            MergeHit::Close,
            MergeHit::PrevConflict,
        ] {
            assert!(has(&hits, need), "{need:?} missing at {w}x{h}");
        }
    }
    let (buf, _) = draw(&mut fixture(), 50, 12);
    assert!(text(&buf).contains("L'\u{e9}diteur de merge demande"));
}

// -- resizable panes (SR) -------------------------------------------------------

fn pane_widths(v: &mut MergeView, w: u16, h: u16) -> Vec<u16> {
    let (_, hits) = draw(v, w, h);
    let mut ds: Vec<Rect> = hits
        .iter()
        .filter(|(_, h)| matches!(h, MergeHit::Divider(_)))
        .map(|(r, _)| *r)
        .collect();
    ds.sort_by_key(|r| r.x);
    let mut x = v.areas.pane_x0;
    let mut out = Vec::new();
    for d in &ds {
        out.push(d.x - x);
        x = d.x + 1;
    }
    out.push(v.areas.pane_x0 + v.areas.top.width - x);
    out
}

#[test]
fn wide_registers_the_dividers_and_narrow_does_not() {
    let (_, hits) = draw(&mut fixture(), 140, 30);
    assert!(has(&hits, MergeHit::Divider(0)));
    assert!(!has(&hits, MergeHit::Divider(1)));
    let mut v = fixture();
    v.show_base = true;
    let (_, hits) = draw(&mut v, 140, 30);
    assert!(has(&hits, MergeHit::Divider(0)) && has(&hits, MergeHit::Divider(1)));
    let (_, hits) = draw(&mut fixture(), 60, 24);
    assert!(!has(&hits, MergeHit::Divider(0)));
}

#[test]
fn hovering_or_dragging_a_divider_lights_it() {
    let (buf0, hits) = draw(&mut fixture(), 140, 30);
    let r = hit_rects(&hits, MergeHit::Divider(0))[0];
    let cell = (r.x, r.y + 2);
    let (buf1, _) = draw_hovered(&mut fixture(), 140, 30, Some(MergeHit::Divider(0)));
    assert_ne!(buf0[cell].fg, buf1[cell].fg, "hover");
    let mut v = fixture();
    draw(&mut v, 140, 30);
    assert!(v.begin_drag(MergeHit::Divider(0), 0));
    let (buf2, _) = draw(&mut v, 140, 30);
    assert_ne!(buf0[cell].fg, buf2[cell].fg, "drag");
}

#[test]
fn dragging_the_divider_resizes_a_and_b_and_double_click_resets() {
    use crossterm::event::{MouseButton, MouseEventKind};
    let mut v = fixture();
    let before = pane_widths(&mut v, 140, 30);
    let (_, hits) = draw(&mut v, 140, 30);
    let r = hit_rects(&hits, MergeHit::Divider(0))[0];
    assert!(v.begin_drag(MergeHit::Divider(0), 0));
    v.on_mouse(MouseEventKind::Drag(MouseButton::Left), r.x - 20, r.y + 3);
    v.on_mouse(MouseEventKind::Up(MouseButton::Left), r.x - 20, r.y + 3);
    let after = pane_widths(&mut v, 140, 30);
    assert_eq!(after[0], before[0] - 20, "{before:?} -> {after:?}");
    assert_eq!(after[0] + after[1], before[0] + before[1]);
    // Far past the edge: A keeps its minimum.
    v.ui.last_press = None;
    assert!(v.begin_drag(MergeHit::Divider(0), 0));
    v.on_mouse(MouseEventKind::Drag(MouseButton::Left), 0, r.y + 3);
    v.on_mouse(MouseEventKind::Up(MouseButton::Left), 0, r.y + 3);
    assert_eq!(pane_widths(&mut v, 140, 30)[0], layout::MIN_PANE_W);
    // Two quick presses: reset, and no drag is left running.
    v.ui.last_press = None;
    assert!(v.begin_drag(MergeHit::Divider(0), 0));
    v.on_mouse(MouseEventKind::Up(MouseButton::Left), 0, 0);
    assert!(v.begin_drag(MergeHit::Divider(0), 0));
    assert!(v.ui.drag.is_none() && v.ui.last_press.is_none());
    assert_eq!(pane_widths(&mut v, 140, 30), before);
}

#[test]
fn the_ratio_survives_a_terminal_resize_and_the_tabs_threshold_still_applies() {
    let mut v = fixture();
    let w = pane_widths(&mut v, 140, 30);
    v.run(MergeCmd::WidenIncoming);
    let moved = pane_widths(&mut v, 140, 30);
    assert!(moved[1] > w[1], "{w:?} {moved:?}");
    let wider = pane_widths(&mut v, 200, 30);
    assert!(wider[1] > wider[0]);
    assert!(wider[1] * 100 / (wider[0] + wider[1]) >= 50);
    // Under the threshold the tabs are back whatever the ratio.
    let (_, hits) = draw(&mut v, 90, 30);
    assert!(!has(&hits, MergeHit::Divider(0)));
}

/// The remembered sizes round-trip through `layout_pref` /
/// `apply_layout_pref`; the transient halves keep what is stored.
#[test]
fn the_layout_pref_round_trips_and_ignores_the_transient_halves() {
    let mut v = fixture();
    assert_eq!(v.layout_pref(0), (None, 0));
    pane_widths(&mut v, 140, 30);
    v.run(MergeCmd::WidenIncoming);
    v.split = Split::Percent(65);
    let (cols, pct) = v.layout_pref(0);
    assert!(cols.is_some());
    assert_eq!(pct, 65);
    v.split = Split::TopOnly;
    assert_eq!(
        v.layout_pref(65).1,
        65,
        "the short-terminal half is not a size"
    );

    let widths = pane_widths(&mut v, 140, 30);
    let mut fresh = fixture();
    fresh.apply_layout_pref(&cols.unwrap(), 65);
    assert_eq!(pane_widths(&mut fresh, 140, 30), widths);
    assert_eq!(fresh.split, Split::Percent(65));
    // Wild stored values are clamped, a wrong-length list is the default.
    fresh.apply_layout_pref(&[1, 2], 4000);
    assert_eq!(fresh.cols, layout::ColSplit(None));
    assert_eq!(fresh.split, Split::Percent(85));
    fresh.apply_layout_pref(&[], 0);
    assert_eq!(fresh.split, Split::DEFAULT);
}

#[test]
fn keys_widen_the_panes_and_reset_restores_the_defaults() {
    let mut v = fixture();
    let before = pane_widths(&mut v, 140, 30);
    v.run(MergeCmd::WidenCurrent);
    let a = pane_widths(&mut v, 140, 30);
    assert!(a[0] > before[0] && a[1] < before[1], "{before:?} {a:?}");
    v.run(MergeCmd::WidenIncoming);
    v.run(MergeCmd::WidenIncoming);
    let b = pane_widths(&mut v, 140, 30);
    assert!(b[1] > b[0] || b[0] < a[0], "{a:?} {b:?}");
    // The minimum holds however often the key repeats.
    for _ in 0..40 {
        v.run(MergeCmd::WidenIncoming);
        pane_widths(&mut v, 140, 30);
    }
    assert_eq!(pane_widths(&mut v, 140, 30)[0], layout::MIN_PANE_W);
    v.split = Split::Percent(65);
    v.run(MergeCmd::ResetPaneSizes);
    assert_eq!(pane_widths(&mut v, 140, 30), before);
    assert_eq!(v.split, Split::DEFAULT);
    // With Base shown, A | Base | B all keep their minimum.
    v.show_base = true;
    for _ in 0..40 {
        v.run(MergeCmd::WidenCurrent);
        pane_widths(&mut v, 140, 30);
    }
    let w = pane_widths(&mut v, 140, 30);
    assert_eq!(w.len(), 3);
    assert!(w.iter().all(|x| *x >= layout::MIN_PANE_W), "{w:?}");
}

#[test]
fn the_resize_keys_say_so_when_a_and_b_are_tabs() {
    let mut v = fixture();
    draw(&mut v, 60, 24);
    let out = v.run(MergeCmd::WidenCurrent);
    assert!(out.message.is_some());
}

#[test]
fn double_clicking_the_splitter_resets_the_output_height() {
    let mut v = fixture();
    draw(&mut v, 140, 40);
    v.split = Split::Percent(70);
    v.ui.last_press = None;
    assert!(v.begin_drag(MergeHit::Splitter, 0));
    assert!(v.begin_drag(MergeHit::Splitter, 0));
    assert_eq!(v.split, Split::DEFAULT);
    assert!(v.ui.drag.is_none() || v.ui.last_press.is_none());
}

#[test]
fn the_handles_ask_for_a_resize_pointer() {
    use crate::chrome::desired_cursor_shape;
    use crate::chrome::HitTarget;
    use crate::cursor::CursorShape;
    let shape = |h| desired_cursor_shape(Some(&HitTarget::Merge(h)), None, None, None, false, None);
    assert_eq!(shape(MergeHit::Divider(0)), CursorShape::ColResize);
    assert_eq!(shape(MergeHit::Splitter), CursorShape::RowResize);
    assert_eq!(shape(MergeHit::Save), CursorShape::Pointer);
}

// -- conflict marks on the scrollbars, the Reset button -------------------------------

/// A file with many conflicts, so the marks spread over the track.
fn many_conflicts(n: usize) -> MergeView {
    let mut regions = Vec::new();
    for i in 0..n {
        regions.push(MergeRegion::Merged {
            lines: (0..12).map(|k| format!("line {i}.{k}")).collect(),
        });
        regions.push(MergeRegion::Conflict {
            base: vec![],
            ours: vec![format!("ours {i}")],
            theirs: vec![format!("theirs {i}")],
        });
    }
    let mut doc = document(true, Eol::Lf);
    doc.regions = regions;
    MergeView::new(MergeModel::new(doc))
}

fn mark_cells(buf: &Buffer, x: u16) -> Vec<(u16, &str)> {
    (0..buf.area.height)
        .map(|y| (y, buf[(x, y)].symbol()))
        .filter(|(_, s)| matches!(*s, "\u{2590}" | "\u{25c6}"))
        .collect()
}

#[test]
fn the_scrollbars_mark_every_conflict_open_handled_and_current() {
    let mut v = many_conflicts(8);
    v.model.take_a(1);
    v.model.go_to(3);
    let (buf, hits) = draw(&mut v, 110, 30);
    let marks: Vec<(Rect, usize)> = hits
        .iter()
        .filter_map(|(r, h)| match h {
            MergeHit::ConflictMark(c) => Some((*r, *c)),
            _ => None,
        })
        .collect();
    assert!(!marks.is_empty(), "marks have hit rects");
    let x = marks[0].0.x;
    // Both scrollbars carry marks: the top panes' and the Output's.
    let cells = mark_cells(&buf, x);
    assert!(cells.len() >= 6, "{cells:?}");
    assert!(
        cells.iter().any(|(_, s)| *s == "\u{25c6}"),
        "the current one"
    );
    // Each mark's color tells open from handled.
    let color = |y: u16| buf[(x, y)].fg;
    let open = crate::scroll_marks::look(crate::scroll_marks::Mark::ConflictOpen).1;
    let done = crate::scroll_marks::look(crate::scroll_marks::Mark::ConflictDone).1;
    assert!(cells.iter().any(|(y, _)| color(*y) == open));
    assert!(cells.iter().any(|(y, _)| color(*y) == done));
}

#[test]
fn a_click_on_a_mark_jumps_to_its_conflict_and_marks_do_not_overlap_anything() {
    let mut v = many_conflicts(8);
    let (_, hits) = draw(&mut v, 110, 30);
    for (i, (a, ha)) in hits.iter().enumerate() {
        for (b, hb) in &hits[i + 1..] {
            assert!(
                a.intersection(*b).area() == 0,
                "{ha:?} {a:?} / {hb:?} {b:?}"
            );
        }
    }
    let far = hits
        .iter()
        .filter_map(|(_, h)| match h {
            MergeHit::ConflictMark(c) if *c >= 4 => Some(*c),
            _ => None,
        })
        .next()
        .expect("a mark on a later conflict outside the thumb");
    assert_eq!(v.model.current(), 0);
    v.click(MergeHit::ConflictMark(far), 0, 0);
    assert_eq!(v.model.current(), far);
    // The panes followed: the conflict's header is on screen.
    let header = v.top.header_at[far];
    assert!(
        (v.top_scroll..v.top_scroll + v.heights.0.max(1)).contains(&header),
        "header {header} vs scroll {}",
        v.top_scroll
    );
}

#[test]
fn a_hovered_mark_lights_up() {
    let mut v = many_conflicts(8);
    let (_, hits) = draw(&mut v, 110, 30);
    let (rect, hit) = hits
        .iter()
        .find(|(_, h)| matches!(h, MergeHit::ConflictMark(c) if *c >= 4))
        .copied()
        .unwrap();
    let (buf, _) = draw_hovered(&mut v, 110, 30, Some(hit));
    assert_eq!(buf[(rect.x, rect.y)].bg, th().ui.hover);
}

#[test]
fn the_toolbar_has_a_reset_button_next_to_take_all() {
    let mut v = fixture();
    let (buf, hits) = draw(&mut v, 140, 30);
    let r = hit_rects(&hits, MergeHit::ResetAll);
    assert_eq!(r.len(), 1);
    let label: String = (r[0].x..r[0].right())
        .map(|x| buf[(x, r[0].y)].symbol())
        .collect();
    assert!(label.contains("Reset"), "{label:?}");
    // Hover lights it.
    let (lit, _) = draw_hovered(&mut v, 140, 30, Some(MergeHit::ResetAll));
    assert_ne!(lit[(r[0].x + 1, r[0].y)].bg, buf[(r[0].x + 1, r[0].y)].bg);
    // Its tooltip names the key.
    let tip = crate::tooltip::hint_text(&crate::chrome::HitTarget::Merge(MergeHit::ResetAll))
        .expect("a tooltip");
    assert!(tip.contains("Alt+R") || tip.contains("Alt+r"), "{tip}");
    // The fixture holds a typed edit, so the click asks first.
    v.click(MergeHit::ResetAll, 0, 0);
    assert!(v.ui.prompt.is_some());
    v.click(MergeHit::PromptSave, 0, 0);
    assert_eq!(v.model.tally().0, 3);
    // Nothing typed any more: the next click resets at once.
    v.model.take_all_a();
    let out = v.click(MergeHit::ResetAll, 0, 0);
    assert!(out.message.is_some() && v.ui.prompt.is_none());
    assert_eq!(v.model.tally().0, 3);
}

#[test]
fn the_reset_confirm_box_paints_two_buttons_and_registers_their_hits() {
    let mut v = fixture();
    // The fixture has a typed edit: the Reset asks first.
    v.run(super::MergeCmd::Reset);
    let (buf, hits) = draw(&mut v, 110, 30);
    assert!(text(&buf).contains("Reset the Output?"), "{}", text(&buf));
    assert!(has(&hits, MergeHit::PromptSave));
    assert!(has(&hits, MergeHit::PromptCancel));
    assert!(!has(&hits, MergeHit::PromptDiscard));
    // A click on the primary button does it.
    v.click(MergeHit::PromptSave, 0, 0);
    assert_eq!(v.model.tally().0, 3);
}

/// Every bar: the thumb is grabbed where it is held and follows the pointer
/// (even past the track), the arrows step a line, and a click on the bare
/// track jumps the thumb under it.
#[test]
fn the_merge_scrollbars_drag_step_and_jump() {
    use crossterm::event::{MouseButton, MouseEventKind};
    for which in [MergeScroll::Top, MergeScroll::Output] {
        let mut v = fixture();
        let (_, hits) = draw(&mut v, 110, 16);
        let scroll = |v: &MergeView| match which {
            MergeScroll::Top => v.top_scroll,
            MergeScroll::Output => v.out_scroll,
        };
        if hit_rects(&hits, MergeHit::Thumb(which)).is_empty() {
            continue;
        }
        let bar = v.ui.bars.get()[usize::from(which == MergeScroll::Output)].expect("the bar");
        // Down arrow: one line.
        let arrow = hit_rects(&hits, MergeHit::Bar(which))
            .into_iter()
            .find(|r| r.y + 1 == bar.rect.bottom())
            .expect("the down arrow is a bar hit");
        assert!(v.begin_drag(MergeHit::Bar(which), arrow.y));
        assert_eq!(scroll(&v), 1, "{which:?}: the arrow steps one line");
        assert!(v.ui.drag.is_none(), "an arrow starts no drag");
        // The thumb: press, pull far below the track, release.
        let (_, hits) = draw(&mut v, 110, 16);
        let thumb2 = hit_rects(&hits, MergeHit::Thumb(which))[0];
        assert!(v.begin_drag(MergeHit::Thumb(which), thumb2.y));
        v.on_mouse(MouseEventKind::Drag(MouseButton::Left), 0, 15);
        let end = scroll(&v);
        v.on_mouse(MouseEventKind::Up(MouseButton::Left), 0, 15);
        assert!(end > 1, "{which:?}: the thumb followed the pointer ({end})");
        // The track above the thumb jumps the view back up.
        let (_, hits) = draw(&mut v, 110, 16);
        let now = hit_rects(&hits, MergeHit::Thumb(which))[0];
        // (A short bar can be all arrows and marks: nothing bare to click.)
        let bare = hit_rects(&hits, MergeHit::Bar(which))
            .into_iter()
            .find(|r| r.y > bar.rect.y && r.bottom() <= now.y);
        if let Some(bare) = bare {
            assert!(v.begin_drag(MergeHit::Bar(which), bare.y));
            v.on_mouse(MouseEventKind::Up(MouseButton::Left), 0, bare.y);
            assert!(scroll(&v) < end, "{which:?}: the track click moved up");
        }
    }
}

mod cjk;

#[test]
fn the_confirm_box_close_cross_is_a_hit_and_cancels() {
    let mut v = fixture();
    v.run(super::MergeCmd::Reset);
    let before = v.model.tally();
    let (_, hits) = draw(&mut v, 110, 30);
    assert!(has(&hits, MergeHit::PromptClose));
    v.click(MergeHit::PromptClose, 0, 0);
    assert!(v.ui.prompt.is_none(), "the cross answers like Esc");
    assert_eq!(v.model.tally(), before, "nothing was reset");
}
