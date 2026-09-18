//! The layout test of the i18n plan (section 6.3): the key screens drawn
//! with `TestBackend` at 100 and 160 columns in every shipped language and
//! in the padded pseudo-locale `qps`, with the git terms off and on.
//!
//! What it asserts, for every screen:
//!
//! * no wide glyph starts in the last column (nothing spills past the
//!   terminal) and no wrapped or laid-out text line is wider than its box;
//! * no garbled box border: every rounded corner has its edge continuing
//!   under or over it (text never overwrites a corner or a side), every
//!   top border closes on the same row;
//! * the toolbar's labels stay within their budget and its hit rects are
//!   disjoint and inside the bar;
//! * the Log's column headers are printed whole or cut with `…`;
//! * the palette, the help overlay, Settings, the merge editor (wide and
//!   narrow), the context menus, the composer and Home keep these rules.
//!
//! `qps` pads every message by about 35 %: it is the stress case. It must
//! keep every structural rule (borders, no spill, disjoint hits); the
//! per-label budgets are asserted for the real languages only.

use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

use gitpoulpi_i18n::{override_git_terms, override_lang, Lang, SHIPPED};

use crate::chrome::{
    help_lines, toolbar_render_with, toolbar_text, ToolbarButton, ToolbarEnabled, TOOLBAR_BUTTONS,
};
use crate::display_width;
use crate::ui_kit::ClippedRender as _;

const WIDTHS: [u16; 2] = [100, 160];
const HEIGHT: u16 = 30;

/// Runs `f` for every language (shipped + `qps`), both git vocabularies and
/// both terminal widths.
fn each_setting(mut f: impl FnMut(Lang, bool, u16)) {
    // Shipped languages (a wired but unshipped one draws English) + qps.
    for lang in SHIPPED.iter().copied().chain([Lang::Qps]) {
        for terms in [false, true] {
            let _l = override_lang(lang);
            let _t = override_git_terms(terms);
            for width in WIDTHS {
                f(lang, terms, width);
            }
        }
    }
}

fn label(screen: &str, lang: Lang, terms: bool, width: u16) -> String {
    format!("{screen} [{} terms={terms} {width} cols]", lang.code())
}

/// One row as text. A double-width glyph's trailing cell holds a blank the
/// terminal never shows, so it is skipped: "撤销" reads as written.
fn row(buf: &Buffer, y: u16) -> String {
    let mut out = String::new();
    let mut x = 0;
    while x < buf.area.width {
        let symbol = buf[(x, y)].symbol();
        out.push_str(symbol);
        x += display_width(symbol).max(1) as u16;
    }
    out
}

fn screen_text(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| row(buf, y))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A scrollbar's arrow or thumb sitting in a border column.
fn is_bar_glyph(s: &str) -> bool {
    matches!(
        s,
        "\u{2191}"
            | "\u{2193}"
            | "\u{25B2}"
            | "\u{25BC}"
            | "\u{2591}"
            | "\u{2592}"
            | "\u{2588}"
            | "\u{2503}"
            | "\u{25A0}"
    )
}

/// The structural rules every drawn screen must keep.
fn check_buffer(what: &str, buf: &Buffer) {
    let w = buf.area.width;
    let h = buf.area.height;
    let at = |x: u16, y: u16| buf[(x, y)].symbol();
    for y in 0..h {
        // Nothing wide may start in the last column: it would spill.
        let last = at(w - 1, y);
        assert!(
            display_width(last) <= 1,
            "{what}: a wide glyph {last:?} spills past the edge on row {y}\n{}",
            screen_text(buf)
        );
        for x in 0..w {
            let s = at(x, y);
            let down = (y + 1 < h).then(|| at(x, y + 1));
            let up = (y > 0).then(|| at(x, y - 1));
            let bad = |msg: &str| {
                panic!(
                    "{what}: garbled border at ({x},{y}) {s:?}: {msg}\n{}",
                    screen_text(buf)
                )
            };
            // A rounded top corner continues downwards (or is a one-row box).
            if s == "\u{256D}" || s == "\u{256E}" {
                if let Some(d) = down {
                    if !matches!(
                        d,
                        "\u{2502}" | "\u{2570}" | "\u{256F}" | "\u{251C}" | "\u{2524}"
                    ) && !is_bar_glyph(d)
                    {
                        // A box of one content row can have a text row
                        // below only if the corner's side is overwritten.
                        bad(&format!("the side below is {d:?}"));
                    }
                }
            }
            if s == "\u{2570}" || s == "\u{256F}" {
                if let Some(u) = up {
                    if !matches!(
                        u,
                        "\u{2502}" | "\u{256D}" | "\u{256E}" | "\u{251C}" | "\u{2524}"
                    ) && !is_bar_glyph(u)
                    {
                        bad(&format!("the side above is {u:?}"));
                    }
                }
            }
        }
        // Every top-left corner has its top-right on the row, in order, and
        // the same for the bottom corners (a box is never half-drawn).
        let r = row(buf, y);
        let tl = r.matches('\u{256D}').count();
        let tr = r.matches('\u{256E}').count();
        let bl = r.matches('\u{2570}').count();
        let br = r.matches('\u{256F}').count();
        assert!(
            tl == tr && bl == br,
            "{what}: unbalanced box corners on row {y}: {r:?}\n{}",
            screen_text(buf)
        );
    }
}

fn draw(width: u16, height: u16, f: impl FnOnce(&mut ratatui::Frame)) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| {
            f(frame);
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

// ------------------------------------------------------------------ toolbar

#[test]
fn the_toolbar_keeps_its_budget_and_its_hits_apart() {
    each_setting(|lang, terms, width| {
        let what = label("toolbar", lang, terms, width);
        let area = Rect::new(0, 0, width, 1);
        let mut toolbar = None;
        let buf = draw(width, 1, |frame| {
            let t = toolbar_render_with(
                area,
                None,
                None,
                ToolbarEnabled::ALL,
                crate::chrome::ToolbarPlacement::default(),
            );
            frame.render_clipped(Paragraph::new(t.line.clone()), area);
            toolbar = Some(t);
        });
        check_buffer(&what, &buf);
        let toolbar = toolbar.unwrap();
        // Hits are inside the bar and pairwise disjoint.
        for (i, (a, _)) in toolbar.hits.iter().enumerate() {
            assert!(a.right() <= width, "{what}: hit {a:?} leaves the bar");
            for (b, _) in &toolbar.hits[i + 1..] {
                assert!(!a.intersects(*b), "{what}: hits overlap {a:?} {b:?}");
            }
        }
        // Budget: a label of a real language is at most 12 cells (the
        // English ones are at most 6); `qps` is padded on purpose.
        if lang != Lang::Qps {
            for (_, m, button) in TOOLBAR_BUTTONS {
                assert!(
                    display_width(m.get()) <= 12,
                    "{what}: {button:?} label {:?} is {} cells",
                    m.get(),
                    display_width(m.get())
                );
            }
        }
        // A button the bar shows is drawn whole (its glyph, a space, the
        // label); one that does not fit goes to the overflow menu, never
        // half drawn.
        let line = row(&buf, 0);
        let shown: Vec<ToolbarButton> = toolbar
            .hits
            .iter()
            .filter_map(|(_, t)| match t {
                crate::chrome::HitTarget::Toolbar(b) => Some(*b),
                _ => None,
            })
            .collect();
        for b in shown {
            let (_, text) = crate::chrome::toolbar_parts(b);
            assert!(
                line.contains(text),
                "{what}: {b:?} is registered but its label {text:?} is not whole in {line:?}"
            );
        }
        let _ = toolbar_text(ToolbarButton::Undo);
    });
}

// ----------------------------------------------------------------- settings

#[test]
fn settings_pages_hold_their_layout() {
    use crate::preferences::tests::state_at;
    each_setting(|lang, terms, width| {
        // Every page that does not need the AI feature.
        for section in [
            "General",
            "Sync",
            "Ssh",
            "AgentsMcp",
            "Appearance",
            "LogGraph",
            "DiffEditor",
            "Keybindings",
            "OpenIn",
        ] {
            let what = label(&format!("settings {section}"), lang, terms, width);
            let dir = tempfile::tempdir().unwrap();
            let mut st = state_at(dir.path(), section);
            let buf = draw(width, HEIGHT, |frame| st.render(frame, frame.area()));
            check_buffer(&what, &buf);
        }
    });
}

// ------------------------------------------------------------------ palette

#[test]
fn the_palette_keeps_borders_and_fits_its_rows() {
    use crate::event_loop::draw::palette_overlay::render_command_palette;
    each_setting(|lang, terms, width| {
        for draft in ["", "pull"] {
            let what = label(&format!("palette {draft:?}"), lang, terms, width);
            let mut popup = None;
            let editor = crate::text_edit::FieldEditor::default();
            let journal = gitpoulpi_core::domain::JournalState::default();
            let buf = draw(width, HEIGHT, |frame| {
                render_command_palette(
                    frame,
                    draft,
                    0,
                    crate::palette::PaletteContext::default(),
                    &editor,
                    &journal,
                    false,
                    &mut popup,
                );
            });
            check_buffer(&what, &buf);
            let popup = popup.expect("the palette recorded its popup");
            assert!(popup.popup.right() <= width, "{what}");
        }
    });
}

// --------------------------------------------------------------------- help

#[test]
fn the_help_overlay_wraps_inside_its_width() {
    each_setting(|lang, terms, width| {
        let what = label("help", lang, terms, width);
        // The overlay's inner width (the popup is a centered box with a border).
        let inner = usize::from(width) * 80 / 100 - 2;
        let km = crate::keymap::current();
        for filter in ["", "commit"] {
            for line in help_lines(&km, filter, inner) {
                assert!(
                    display_width(&line) <= inner,
                    "{what}: {} cells > {inner}: {line:?}",
                    display_width(&line)
                );
            }
        }
    });
}

/// Every tab of the help popup, in every language: whole box, every header
/// hit-testable and apart, guide lines inside the width.
#[test]
fn the_help_tabs_hold_their_layout_in_every_language() {
    each_setting(|lang, terms, width| {
        for tab in 0..crate::help_guide::TABS.len() {
            let what = label(&format!("help tab {tab}"), lang, terms, width);
            crate::help_view::set_tab(tab);
            let (mut scroll, mut inner) = (0u16, 0u16);
            let buf = draw(width, HEIGHT, |frame| {
                crate::help_view::render(frame, "", &mut scroll, &mut inner);
            });
            check_buffer(&what, &buf);
            let text = screen_text(&buf);
            let mut prev_end = 0u16;
            for i in 0..crate::help_guide::TABS.len() {
                let found =
                    (0..width).find(|&x| crate::help_view::tab_at(x, 6).is_some_and(|t| t == i));
                let x = found.unwrap_or_else(|| panic!("{what}: tab {i} has no hit\n{text}"));
                assert!(
                    x >= prev_end,
                    "{what}: tab {i} overlaps the previous\n{text}"
                );
                prev_end = (x..width)
                    .take_while(|&c| crate::help_view::tab_at(c, 6) == Some(i))
                    .last()
                    .unwrap_or(x)
                    + 1;
            }
            if tab > 0 {
                let inner_w = usize::from(width) * 70 / 100 - 4;
                for filter in ["", "ssh"] {
                    for line in crate::help_guide::text(tab, filter, inner_w).lines() {
                        assert!(
                            display_width(line) <= inner_w,
                            "{what}: {} cells > {inner_w}: {line:?}",
                            display_width(line)
                        );
                    }
                }
            }
        }
        crate::help_view::forget();
        crate::help_view::set_tab(0);
    });
}

// -------------------------------------------------------------- merge editor

#[test]
fn the_merge_editor_holds_its_layout_wide_and_narrow() {
    use crate::merge_view::render_tests::fixture;
    each_setting(|lang, terms, width| {
        // Wide at the terminal width, narrow (tabbed) at 62 columns.
        for w in [width, 62] {
            let what = label(&format!("merge editor {w}"), lang, terms, width);
            let mut view = fixture();
            let buf = draw(w, HEIGHT, |frame| {
                let _ = view.render(frame, Rect::new(0, 0, w, HEIGHT), None);
            });
            check_buffer(&what, &buf);
        }
    });
}

// ------------------------------------------------------------- context menus

#[test]
fn context_menus_stay_inside_the_screen() {
    use crate::context_menu::Menu as ContextMenu;
    use crate::refs_sidebar::SidebarSection;
    each_setting(|lang, terms, width| {
        // Anchored far to the right: the menu has to shift left.
        for col in [2u16, width - 6] {
            let what = label(&format!("ref menu at {col}"), lang, terms, width);
            let ctx = crate::log_menu::MenuContext {
                current_branch: Some("main".into()),
                remote_twin: Some("origin/feature".into()),
                ..crate::log_menu::MenuContext::default()
            };
            let entries = crate::ref_menu::ref_menu_entries(
                SidebarSection::Local,
                "feature",
                &ctx,
                false,
                false,
                false,
            );
            let menu = ContextMenu::new("feature", entries, col, 2);
            let mut rects = Vec::new();
            let buf = draw(width, HEIGHT, |frame| rects = menu.render(frame, None));
            check_buffer(&what, &buf);
            assert_rows_whole(&what, &menu, &buf);
            for r in rects {
                assert!(r.right() <= width && r.bottom() <= HEIGHT, "{what}: {r:?}");
            }
            let what = label(&format!("commit menu at {col}"), lang, terms, width);
            let ctx = crate::log_menu::MenuContext {
                current_branch: Some("main".into()),
                ..crate::log_menu::MenuContext::default()
            }
            .with_ai(true);
            let menu = ContextMenu::new(
                "a1b2c3d",
                crate::log_menu::commit_menu_entries(&ctx, false),
                col,
                2,
            );
            let buf = draw(width, HEIGHT, |frame| drop(menu.render(frame, None)));
            check_buffer(&what, &buf);
            assert_rows_whole(&what, &menu, &buf);
        }
    });
}

/// Every row label of `menu` is on screen in full, on a line that closes
/// with the border: a longer label widens the menu instead of being cut.
fn assert_rows_whole<A: Clone>(what: &str, menu: &crate::context_menu::Menu<A>, buf: &Buffer) {
    let lines: Vec<String> = (0..buf.area.height).map(|y| row(buf, y)).collect();
    for r in 0..menu.row_count() {
        let label = menu.row_label(r);
        if label.is_empty() {
            continue;
        }
        let line = lines
            .iter()
            .find(|l| l.contains(label.as_str()))
            .unwrap_or_else(|| {
                panic!(
                    "{what}: row {label:?} is cut or missing\n{}",
                    lines.join("\n")
                )
            });
        assert!(
            line.trim_end().ends_with('\u{2502}'),
            "{what}: row {label:?} is not closed by the border: {line:?}"
        );
    }
}

// ----------------------------------------------------------------- composer

#[test]
fn the_composer_keeps_its_buttons_on_one_row() {
    use crate::composer::{render_commit_composer, ComposerHover, COMPOSER_MIN_HEIGHT};
    each_setting(|lang, terms, width| {
        for amend in [false, true] {
            for staged in [0usize, 3] {
                let what = label(
                    &format!("composer amend={amend} staged={staged}"),
                    lang,
                    terms,
                    width,
                );
                let buf = draw(width, COMPOSER_MIN_HEIGHT + 2, |frame| {
                    let mut hits = Vec::new();
                    render_commit_composer(
                        frame,
                        Rect::new(0, 0, width, COMPOSER_MIN_HEIGHT + 2),
                        &mut hits,
                        "fix: x",
                        "",
                        amend,
                        None,
                        staged,
                        false,
                        ComposerHover::default(),
                        &crate::text_edit::FieldEditor::default(),
                    );
                    // Hits layer on purpose (the amend row under its checkbox,
                    // the AI button inside the summary box): not asserted here.
                    let _ = &hits;
                });
                check_buffer(&what, &buf);
            }
        }
    });
}

// ------------------------------------------------------------- split commit

#[test]
fn the_split_editor_holds_its_layout() {
    use gitpoulpi_core::domain::{
        DiffLine, FileStatus, Hunk, HunkId, LineOrigin, Oid, SplitFile, SplitSource, WholeReason,
    };
    let hunk = Hunk {
        id: HunkId(0),
        old_start: 1,
        old_lines: 2,
        new_start: 1,
        new_lines: 3,
        lines: vec![
            DiffLine {
                origin: LineOrigin::Context,
                content: "fn a() {}\n".into(),
                staged: false,
            },
            DiffLine {
                origin: LineOrigin::Add,
                content: "fn b() {}\n".into(),
                staged: false,
            },
        ],
    };
    let source = SplitSource {
        commit: Oid([7; 20]),
        parent: Some(Oid([6; 20])),
        message: "feat: add the parser and the logo".into(),
        sign_default: false,
        files: vec![
            SplitFile {
                path: "src/parse.rs".into(),
                status: FileStatus::Modified,
                whole_only: None,
                hunks: vec![hunk],
            },
            SplitFile {
                path: "assets/logo.png".into(),
                status: FileStatus::Added,
                whole_only: Some(WholeReason::Binary),
                hunks: vec![],
            },
        ],
    };
    each_setting(|lang, terms, width| {
        for signed in [false, true] {
            let what = label(&format!("split editor sign={signed}"), lang, terms, width);
            let mut form = crate::split_ui::SplitForm::new(source.clone());
            form.set_sign(signed);
            form.toggle_line_mode();
            let buf = draw(width, HEIGHT, |frame| {
                let _ = form.render(frame, Rect::new(0, 0, width, HEIGHT), None);
            });
            check_buffer(&what, &buf);
            let text = screen_text(&buf);
            assert!(
                !text.contains("{$") && !text.contains("{key}"),
                "{what}\n{text}"
            );
        }
    });
}

// --------------------------------------------------------------------- Home

#[test]
fn home_holds_its_layout() {
    use crate::repo_manager::{render, KnownRepo, RepoManager};
    fn repo(name: &str, tab: Option<usize>, recent: Option<usize>) -> KnownRepo {
        KnownRepo {
            path: std::path::PathBuf::from(format!("/work/{name}")),
            name: name.to_string(),
            branch_hint: Some("main".into()),
            exists: true,
            is_repo: true,
            tab,
            favorite: None,
            recent,
            facts: None,
        }
    }
    each_setting(|lang, terms, width| {
        for repos in [
            Vec::new(),
            vec![repo("alpha", Some(0), Some(0)), repo("beta", None, Some(1))],
        ] {
            let what = label(&format!("home ({} repos)", repos.len()), lang, terms, width);
            let mut manager = RepoManager::new(repos);
            let area = Rect::new(0, 0, width, HEIGHT);
            let buf = draw(width, HEIGHT, |frame| {
                let out = render(&mut manager, area);
                frame.render_clipped(Paragraph::new(out.lines.clone()), area);
            });
            check_buffer(&what, &buf);
        }
    });
}

#[test]
fn the_jobs_panel_and_indicator_hold_their_layout() {
    use gitpoulpi_core::domain::{OperationKind, RepoId};
    use gitpoulpi_core::NotCancellableReason;
    use gitpoulpi_ipc::{JobInfo, JobState};
    let job = |id: u64, kind: OperationKind, progress: Option<u8>| JobInfo {
        id,
        kind,
        repo: Some(RepoId(format!("/work/repo-{id}"))),
        dest: None,
        label: String::new(),
        state: JobState::Running,
        phase: None,
        progress,
        started_at: Some(0),
        joiners: 0,
        queued_after: None,
        not_cancellable: None,
        error: None,
    };
    let mut stuck = job(3, OperationKind::Pull, None);
    stuck.not_cancellable = Some(NotCancellableReason::Integrating);
    let mut queued = job(4, OperationKind::Push, None);
    queued.state = JobState::Queued;
    for j in [
        job(1, OperationKind::Push, Some(42)),
        job(2, OperationKind::Fetch, None),
        stuck,
        queued,
    ] {
        crate::jobs_ui::changed(j);
    }
    crate::jobs_ui::run(crate::jobs_ui::JobsCmd::Show);
    each_setting(|lang, terms, width| {
        let what = label("jobs panel", lang, terms, width);
        let buf = draw(width, HEIGHT, crate::jobs_ui::render_panel);
        check_buffer(&what, &buf);
        let text = screen_text(&buf);
        assert!(
            text.contains("repo-3"),
            "{what}: the rows are drawn
{text}"
        );

        // The indicator, at the right of a footer row.
        let what = label("jobs indicator", lang, terms, width);
        let mut footer = Buffer::empty(Rect::new(0, 0, width, 1));
        let w = crate::jobs_ui::width_for();
        assert!(w > 0 && w <= width / 2, "{what}: indicator is {w} wide");
        crate::jobs_ui::paint_indicator(&mut footer, Rect::new(0, 0, width, 1), width - 12, 0);
        check_buffer(&what, &footer);
        assert!(
            row(&footer, 0).contains("[x]"),
            "{what}: {}",
            row(&footer, 0)
        );
    });
}

// -------------------------------------------------------- main window pieces

#[test]
fn log_column_headers_are_whole_or_cut_with_an_ellipsis() {
    use crate::{LogColumn, LogColumnWidths};
    use ratatui::style::Style;
    use ratatui::widgets::{Cell, TableState};
    each_setting(|lang, terms, width| {
        // The Log pane's share of the terminal (about 46 % at 100 columns,
        // 55 % at 160).
        let log_w = if width == 100 { 46 } else { 88 };
        let what = label("log headers", lang, terms, width);
        let hidden = std::collections::HashSet::new();
        let order: Vec<LogColumn> = LogColumn::ALL.into_iter().collect();
        let widths = LogColumnWidths::for_area_width(log_w, &hidden, &Default::default());
        let area = Rect::new(0, 0, log_w, 4);
        let buf = draw(log_w, 4, |frame| {
            let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
            frame.render_stateful_clipped(
                crate::log_table::log_table(Vec::new(), header, &order, &widths, Style::default()),
                area,
                &mut TableState::default(),
            );
        });
        check_buffer(&what, &buf);
        if lang == Lang::Qps {
            return;
        }
        let line = row(&buf, 1);
        for c in &order {
            let cell_w = widths.width_of(*c) as usize;
            if cell_w == 0 {
                continue;
            }
            let title = c.title();
            let whole = line.contains(title);
            // Cut with an ellipsis, or the column is too narrow for even
            // one letter plus the ellipsis (English has the same limit).
            let cut = title
                .chars()
                .take(1)
                .next()
                .is_some_and(|first| line.contains(&format!("{first}\u{2026}")));
            let en = gitpoulpi_i18n::with_lang(Lang::En, || display_width(c.title()));
            assert!(
                whole || cut || cell_w < en.min(display_width(title)),
                "{what}: header {title:?} of {c:?} ({cell_w} cells) is neither whole nor cut with an ellipsis in {line:?}"
            );
        }
    });
}

#[test]
fn sidebar_titles_fit_the_sidebar() {
    use crate::refs_sidebar::SidebarSection;
    let mut too_wide = Vec::new();
    each_setting(|lang, terms, width| {
        if lang == Lang::Qps || width != 100 {
            return;
        }
        for merge in [false, true] {
            let _m = gitpoulpi_i18n::override_merge_terms(merge);
            let what = label(&format!("sidebar titles merge={merge}"), lang, terms, width);
            for s in SidebarSection::ORDER {
                let t = s.display_title();
                if display_width(t) > 18 {
                    too_wide.push(format!("{what}: {t:?}"));
                }
            }
        }
    });
    assert!(too_wide.is_empty(), "{}", too_wide.join("\n"));
}

/// The detector itself: a corner whose side was overwritten by text, and an
/// unbalanced box, must fail, so a green run means something.
#[test]
fn the_border_check_catches_a_garbled_box() {
    let mut buf = Buffer::empty(Rect::new(0, 0, 8, 3));
    for (x, y, s) in [
        (0, 0, "\u{256D}"),
        (7, 0, "\u{256E}"),
        (0, 1, "\u{2502}"),
        (7, 1, "\u{2502}"),
        (0, 2, "\u{2570}"),
        (7, 2, "\u{256F}"),
    ] {
        buf[(x, y)].set_symbol(s);
    }
    check_buffer("a clean box", &buf);
    buf[(7, 1)].set_symbol("x");
    let broken = std::panic::catch_unwind(|| check_buffer("a garbled box", &buf));
    assert!(broken.is_err(), "text over a side must be caught");
}

// ------------------------------------------------------ rewrite workbench

#[test]
fn the_rewrite_workbench_holds_its_layout() {
    use crate::daemon_link::RawQueue;
    use crate::rewrite_view::{
        PlanState, PreviewTab, Question, RecipeStore, RewriteView, RuleTemplate, ScopeContext,
    };
    use gitpoulpi_core::domain::rewrite::{
        Counts, PlanDigest, PushState, RefChange, RewritePlan, RewriteWarning, RuleCount, RunId,
    };
    use gitpoulpi_core::domain::Oid;

    let queue = RawQueue::default();
    let open = || {
        RewriteView::open(
            queue.requester(),
            RecipeStore::default(),
            ScopeContext {
                head_branch: Some("main".into()),
                branches: vec!["main".into()],
                ..ScopeContext::default()
            },
        )
    };
    let plan = RewritePlan {
        run: RunId("run".into()),
        digest: PlanDigest("digest".into()),
        counts: Counts {
            commits_walked: 9_800,
            commits_changed: 412,
            commits_dropped: 3,
            refs_moved: 2,
            tags_moved: 4,
            signed_changed: 31,
            ..Counts::default()
        },
        refs: vec![RefChange {
            name: "refs/heads/main".into(),
            old: Oid([1; 20]),
            new: Some(Oid([2; 20])),
            pushed: PushState::OnRemote {
                remote: "origin".into(),
                remote_oid: Oid([1; 20]),
                rewritten_on_remote: 412,
            },
            protected: true,
            included: false,
            checked_out_in: None,
        }],
        warnings: vec![
            RewriteWarning::SignaturesAffected { count: 31 },
            RewriteWarning::PushedRefs { count: 1 },
            RewriteWarning::StashesKeepOldHistory { count: 2 },
        ],
        per_rule: vec![RuleCount {
            rule: 0,
            commits: 412,
        }],
        rows: Vec::new(),
    };
    each_setting(|lang, terms, width| {
        let mut states: Vec<(&str, RewriteView)> = Vec::new();
        states.push(("empty", open()));
        let mut rules = open();
        for template in [
            RuleTemplate::Identity,
            RuleTemplate::DateShift,
            RuleTemplate::MessageReplace,
            RuleTemplate::LfsImport,
        ] {
            rules.add_rule(template);
        }
        states.push(("rules", rules));
        for (name, tab) in [
            ("summary", PreviewTab::Summary),
            ("refs", PreviewTab::Refs),
            ("impact", PreviewTab::Impact),
            ("commits", PreviewTab::Commits),
        ] {
            let mut v = open();
            v.add_rule(RuleTemplate::Identity);
            v.add_rule(RuleTemplate::DateShift);
            v.plan = Some(plan.clone());
            v.plan_state = PlanState::Current;
            v.tab = tab;
            states.push((name, v));
        }
        let mut asking = open();
        asking.add_rule(RuleTemplate::Identity);
        asking.question = Some(Question::Signatures {
            count: Some(31),
            cursor: 0,
        });
        states.push(("question", asking));
        // The Script rule: its pane, then its editor with a test result.
        let mut script = open();
        script.add_rule(RuleTemplate::Script);
        states.push(("script rule", open().with_rule(RuleTemplate::Script)));
        script.scope_ctx.test_commits = vec![Oid([1; 20]), Oid([2; 20])];
        script.scope_ctx.test_from_selection = true;
        script.open_script_editor();
        script.script_test_result_for_layout();
        states.push(("script editor", script));
        for (name, view) in states {
            let what = label(&format!("rewrite workbench {name}"), lang, terms, width);
            let buf = draw(width, HEIGHT, |frame| {
                let _ = view.render(frame, Rect::new(0, 0, width, HEIGHT), None);
            });
            check_buffer(&what, &buf);
            let text = screen_text(&buf);
            assert!(
                !text.contains("{$") && !text.contains("{key}") && !text.contains("{n}"),
                "{what}\n{text}"
            );
        }
    });
}

// ---------------------------------------------------------- purge screen

#[test]
fn the_purge_screen_holds_its_layout() {
    use crate::daemon_link::RawQueue;
    use crate::rewrite_view::{PurgeConfirm, RecipeStore, RewriteView, ScopeContext};
    use gitpoulpi_core::domain::rewrite::{
        HostChecklist, PurgeBlocker, PurgeBlockerKind, PurgeCheck, PurgeEntryInfo, PurgeOutcome,
        PurgeStatus,
    };

    let queue = RawQueue::default();
    let kinds = [
        PurgeBlockerKind::Branch,
        PurgeBlockerKind::Tag,
        PurgeBlockerKind::RemoteTracking,
        PurgeBlockerKind::Stash,
        PurgeBlockerKind::Worktree,
        PurgeBlockerKind::UndoPin,
        PurgeBlockerKind::Notes,
        PurgeBlockerKind::Replace,
        PurgeBlockerKind::Other,
    ];
    let check = |blocked: bool, url: &str| PurgeCheck {
        entries: vec![
            PurgeEntryInfo {
                id: 7,
                label: "Rewrite history".into(),
                rewrite: true,
            },
            PurgeEntryInfo {
                id: 5,
                label: "Stage file".into(),
                rewrite: false,
            },
        ],
        already_purged: Vec::new(),
        outcome: PurgeOutcome {
            status: if blocked {
                PurgeStatus::Blocked
            } else {
                PurgeStatus::Ready
            },
            blockers: if blocked {
                kinds
                    .iter()
                    .map(|kind| PurgeBlocker {
                        kind: *kind,
                        name: "stash@{0}".into(),
                        commits: 2,
                    })
                    .collect()
            } else {
                Vec::new()
            },
            old_commits: 12,
            old_blobs: 30,
            old_objects: 55,
            reflogs: vec!["HEAD".into()],
            reflog_entries: 4,
            pins_dropped: 0,
            runs_removed: 0,
            remaining: 0,
        },
        hosts: vec![HostChecklist::new("origin", url)],
        prompt: String::new(),
    };
    each_setting(|lang, terms, width| {
        let open = || {
            RewriteView::open(
                queue.requester(),
                RecipeStore::default(),
                ScopeContext::default(),
            )
        };
        let mut states: Vec<(&str, RewriteView)> = Vec::new();
        for (name, url) in [
            ("github", "git@github.com:a/b.git"),
            ("gitlab", "https://gitlab.example.org/a/b"),
            ("azure", "https://dev.azure.com/o/p/_git/r"),
            ("bitbucket", "https://bitbucket.org/a/b.git"),
            ("other", "/srv/git/x.git"),
        ] {
            let mut v = open();
            v.purge_for_layout(check(name == "github", url), None);
            states.push((name, v));
        }
        let mut ready = open();
        ready.purge_for_layout(check(false, "git@github.com:a/b.git"), None);
        states.push(("ready", ready));
        let mut purge = open();
        purge.purge_for_layout(
            check(false, "git@github.com:a/b.git"),
            Some(PurgeConfirm::Purge { cursor: None }),
        );
        states.push(("confirm", purge));
        let mut drop = open();
        drop.purge_for_layout(
            check(true, "git@github.com:a/b.git"),
            Some(PurgeConfirm::Drop {
                name: "stash@{0}".into(),
                id: 0,
                cursor: Some(0),
            }),
        );
        states.push(("drop", drop));
        for (name, view) in states {
            let what = label(&format!("purge screen {name}"), lang, terms, width);
            let buf = draw(width, HEIGHT, |frame| {
                let _ = view.render(frame, Rect::new(0, 0, width, HEIGHT), None);
            });
            check_buffer(&what, &buf);
            let text = screen_text(&buf);
            assert!(
                !text.contains("{$") && !text.contains("{key}") && !text.contains("{n}"),
                "{what}\n{text}"
            );
        }
    });
}
