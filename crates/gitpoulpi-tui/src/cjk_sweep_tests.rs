//! The CJK render sweep: the major surfaces drawn in Simplified Chinese,
//! Japanese and Korean at 80x24, 120x30 and 200x50 over a background of
//! CJK text, with `render_check`'s invariants asserted on every buffer:
//!
//! * (a) no wide glyph whose blank continuation was overwritten, none in the
//!   last column;
//! * (b) every box intact on every row of every open popup;
//! * (c) nothing written outside the widget's area (the background keeps
//!   every cell the widget does not own);
//! * (d) a menu's shortcut column ends flush at the popup's inner edge.
//!
//! The existing `i18n_layout_tests` run the same checker over the other
//! screens (toolbar, Settings pages, palette, help, merge editor, composer,
//! Home, jobs, rewrite...) in every language at two widths; this file adds the
//! popups and dropdowns the owner reported, at the three sizes, over the
//! CJK-text background that exposes a glyph cut by an edge.

use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::Terminal;

use gitpoulpi_i18n::{override_git_terms, override_lang, Lang};

use crate::context_menu::{Entry, Menu};
use crate::render_check as rc;

pub(crate) const LANGS: [Lang; 3] = [Lang::ZhCn, Lang::Ja, Lang::Ko];
pub(crate) const SIZES: [(u16, u16); 3] = [(80, 24), (120, 30), (200, 50)];

/// Runs `f` for each CJK language, both git vocabularies and every size.
pub(crate) fn each_cjk(mut f: impl FnMut(Lang, bool, u16, u16)) {
    for lang in LANGS {
        for terms in [false, true] {
            let _l = override_lang(lang);
            let _t = override_git_terms(terms);
            for (w, h) in SIZES {
                f(lang, terms, w, h);
            }
        }
    }
}

/// A screen full of CJK text: what a Log of CJK commit subjects looks like
/// under a popup. Rows alternate their start by one cell so that an overlay's
/// left edge cuts a glyph on every other row.
pub(crate) fn cjk_background(w: u16, h: u16) -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, w, h));
    let text = "提交分支拣选还原합병日本語の変更点".repeat(40);
    for y in 0..h {
        let start = usize::from(y % 2);
        let line: String = text.chars().skip(start).collect();
        let x0 = if y % 2 == 0 { 0 } else { 1 };
        buf.set_stringn(x0, y, &line, usize::from(w - x0), Style::default());
    }
    buf
}

/// Draws `f` over `bg` (a frame whose buffer starts as `bg`) and returns the
/// finished buffer.
pub(crate) fn draw_over(bg: &Buffer, f: impl FnOnce(&mut ratatui::Frame)) -> Buffer {
    let area = bg.area;
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    terminal
        .draw(|frame| {
            *frame.buffer_mut() = bg.clone();
            f(frame);
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

/// The box a menu drew: the union of its row rects, grown by its border.
fn menu_box(rects: &[Rect], area: Rect) -> Rect {
    let mut r = rects
        .iter()
        .copied()
        .find(|r| r.width > 0)
        .expect("a menu draws rows");
    for q in rects.iter().filter(|q| q.width > 0) {
        r = r.union(*q);
    }
    Rect::new(
        r.x.saturating_sub(1),
        r.y.saturating_sub(1),
        r.width + 2,
        r.height + 2,
    )
    .intersection(area)
}

/// Draws `menu` over the CJK background and asserts every invariant.
fn check_menu<A: Clone + PartialEq + std::fmt::Debug>(what: &str, menu: &Menu<A>, w: u16, h: u16) {
    let bg = cjk_background(w, h);
    let mut rects = Vec::new();
    let buf = draw_over(&bg, |frame| rects = menu.render(frame, None));
    let popup = menu_box(&rects, bg.area);
    // (a) and (b): the whole buffer, background glyphs cut by the popup's
    // edges included.
    let mut all = rc::wide_violations(&buf);
    all.extend(rc::box_violations_of(&buf, popup));
    assert!(
        all.is_empty(),
        "{what}: {}\n{}",
        all.join("\n"),
        rc::dump(&buf)
    );
    // (c): nothing outside the popup changed, except the half-glyph its left
    // edge had to blank (one cell, on the popup's left).
    for (x, y) in rc::changed_outside(&bg, &buf, popup) {
        let cut = x + 1 == popup.x && !bg[(x, y)].symbol().is_ascii();
        assert!(
            cut,
            "{what}: ({x},{y}) {:?} -> {:?} was written outside the popup {popup:?}\n{}",
            bg[(x, y)].symbol(),
            buf[(x, y)].symbol(),
            rc::dump(&buf)
        );
    }
    // (d): every shortcut ends flush at the inner edge of the popup.
    let mut hints: Vec<(u16, String)> = Vec::new();
    for (r, rect) in rects.iter().enumerate() {
        let hint = menu.row_hint(r);
        if !hint.is_empty() && rect.width > 0 {
            hints.push((rect.y, hint));
        }
    }
    if !hints.is_empty() {
        let col = rc::right_aligned_at(&buf, &hints)
            .unwrap_or_else(|e| panic!("{what}: shortcut column: {e}\n{}", rc::dump(&buf)));
        assert_eq!(col, popup.right() - 1, "{what}: shortcuts are not flush");
    }
    // Every label is on screen whole, contiguous (no spacing between CJK
    // characters), on a row closed by the border.
    for (r, rect) in rects.iter().enumerate() {
        let label = menu.row_label(r);
        if label.is_empty() || rect.width == 0 {
            continue;
        }
        let text = rc::row_text(&buf, rect.y);
        assert!(
            text.contains(&label),
            "{what}: label {label:?} is not contiguous on its row: {text:?}"
        );
    }
}

/// `check_menu` on `menu` and on every state a row activation leads to
/// (submenu rows swap the content of the popup).
fn check_with_flyouts<A: Clone + PartialEq + std::fmt::Debug>(
    what: &str,
    menu: &Menu<A>,
    w: u16,
    h: u16,
    flyouts: bool,
) {
    check_menu(what, menu, w, h);
    for r in (0..menu.row_count()).filter(|_| flyouts) {
        if !menu.row_is_selectable(r) {
            continue;
        }
        let mut m = menu.clone();
        drop(m.activate(r));
        check_menu(&format!("{what} row {r} activated"), &m, w, h);
    }
}

fn anchors(w: u16, h: u16) -> Vec<(u16, u16)> {
    vec![
        (0, 0),
        (1, 1),
        (w / 2, h / 2),
        (w / 2 + 1, 3),
        (w - 3, 2),
        (w - 1, h - 1),
        (5, h - 4),
    ]
}

#[test]
fn commit_and_ref_menus_hold_over_cjk_text() {
    each_cjk(|lang, terms, w, h| {
        for (n, (col, row)) in anchors(w, h).into_iter().enumerate() {
            let ctx = crate::log_menu::MenuContext {
                current_branch: Some("main".into()),
                remote_twin: Some("origin/功能/登录".into()),
                ..crate::log_menu::MenuContext::default()
            }
            .with_ai(true);
            let commit = Menu::new(
                "a1b2c3d",
                crate::log_menu::commit_menu_entries(&ctx, false),
                col,
                row,
            );
            let refm = Menu::new(
                "功能/登录",
                crate::ref_menu::ref_menu_entries(
                    crate::refs_sidebar::SidebarSection::Local,
                    "功能/登录",
                    &ctx,
                    false,
                    false,
                    false,
                ),
                col,
                row,
            );
            let tag = |name: &str| {
                format!(
                    "{name} menu [{} terms={terms} {w}x{h} @{col},{row}]",
                    lang.code()
                )
            };
            check_with_flyouts(&tag("commit"), &commit, w, h, n % 3 == 0);
            check_with_flyouts(&tag("ref"), &refm, w, h, n % 3 == 0);
        }
    });
}

#[test]
fn a_hand_built_cjk_menu_with_hints_is_aligned() {
    each_cjk(|lang, terms, w, h| {
        let menu = Menu::new(
            "标题",
            vec![
                Entry::action("将拣选到 main", 1u8).with_key(crate::keymap::Action::GitPull),
                Entry::action("还原此提交…", 2u8),
                Entry::submenu("将 main 重置到此提交", vec![Entry::action("软重置", 3u8)]),
                Entry::action("日本語のラベル", 4u8).with_key(crate::keymap::Action::GitPush),
            ],
            3,
            2,
        );
        let what = format!("hand-built menu [{} terms={terms} {w}x{h}]", lang.code());
        check_menu(&what, &menu, w, h);
    });
}

/// Asserts (a), (b) on `popup`'s edges and (c) for an overlay drawn by `f`
/// over the CJK background.
fn check_overlay(
    what: &str,
    w: u16,
    h: u16,
    popup_of: impl Fn(&Buffer) -> Rect,
    f: impl FnOnce(&mut ratatui::Frame),
) {
    let bg = cjk_background(w, h);
    let buf = draw_over(&bg, f);
    let popup = popup_of(&buf);
    let mut all = rc::wide_violations(&buf);
    all.extend(rc::edges_violations(&buf, popup));
    assert!(
        all.is_empty(),
        "{what}: {}\n{}",
        all.join("\n"),
        rc::dump(&buf)
    );
    for (x, y) in rc::changed_outside(&bg, &buf, popup) {
        assert!(
            x + 1 == popup.x,
            "{what}: ({x},{y}) {:?} -> {:?} was written outside {popup:?}\n{}",
            bg[(x, y)].symbol(),
            buf[(x, y)].symbol(),
            rc::dump(&buf)
        );
    }
}

#[test]
fn the_palette_and_help_overlays_hold_over_cjk_text() {
    use crate::event_loop::draw::palette_overlay::render_command_palette;
    each_cjk(|lang, terms, w, h| {
        for draft in ["", "pull"] {
            let what = format!("palette {draft:?} [{} terms={terms} {w}x{h}]", lang.code());
            let popup = std::cell::RefCell::new(None);
            let editor = crate::text_edit::FieldEditor::default();
            let journal = gitpoulpi_core::domain::JournalState::default();
            check_overlay(
                &what,
                w,
                h,
                |_| {
                    popup
                        .borrow()
                        .as_ref()
                        .map(|p: &crate::chrome::PalettePopup| p.popup)
                        .unwrap()
                },
                |frame| {
                    render_command_palette(
                        frame,
                        draft,
                        0,
                        crate::palette::PaletteContext::default(),
                        &editor,
                        &journal,
                        false,
                        &mut popup.borrow_mut(),
                    )
                },
            );
        }
        let what = format!("help [{} terms={terms} {w}x{h}]", lang.code());
        check_overlay(
            &what,
            w,
            h,
            |buf| crate::ui_kit::centered_rect(70, 80, buf.area),
            |frame| crate::help_view::render(frame, "", &mut 0, &mut 0),
        );
    });
}
