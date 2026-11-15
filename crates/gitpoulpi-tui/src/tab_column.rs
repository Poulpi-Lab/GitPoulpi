//! The repository tabs as a vertical list (`ui.tabs_position = "left"`).
//!
//! One row per entry, top to bottom: the `⌂` Home button, the workspace chip,
//! the Radar and Release notes tabs, the repository tabs (a scrolling window
//! of them), the transient New tab, the `⋯` overflow row and the `+` row.
//! Like the horizontal bar this is pure: it returns the painted rows and the
//! rects of the same walk, in the same [`TabBar`] the row builder returns, so
//! the draw code registers hit targets for either shape without telling them
//! apart.
//!
//! The only state is process-wide, like `profile_bar`: where the column was
//! drawn (the wheel and the drag read it) and how far it is scrolled.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::repo_switcher::{
    close_style, tab_style, NewTabSlot, RepoTab, TabBar, TabHit, FOLDER_BUTTON, NEW_TAB_BUTTON,
    OVERFLOW_BUTTON,
};
use crate::th;
use crate::ui_kit::{display_width, fit};

static AREA: Mutex<Option<Rect>> = Mutex::new(None);
static SCROLL: AtomicUsize = AtomicUsize::new(0);
static LAST_ACTIVE: AtomicUsize = AtomicUsize::new(usize::MAX);

/// Records where the column was drawn this frame (`None`: not drawn).
pub(crate) fn set_area(area: Option<Rect>) {
    *AREA.lock().unwrap_or_else(|e| e.into_inner()) = area;
}

/// The column's rect while it is on screen.
pub(crate) fn area() -> Option<Rect> {
    *AREA.lock().unwrap_or_else(|e| e.into_inner())
}

/// The wheel: moves the visible window by `delta` rows (clamped at the next
/// draw).
pub(crate) fn scroll_by(delta: isize) {
    let now = SCROLL.load(Ordering::Relaxed) as isize;
    SCROLL.store((now + delta).max(0) as usize, Ordering::Relaxed);
}

/// The scroll offset to draw with, and the tab to reveal: the active one,
/// but only on the frame its identity changed, so a wheel scroll away from
/// it is not undone by the next frame.
pub(crate) fn scroll_for_draw(active: Option<usize>) -> (usize, Option<usize>) {
    let follow = active.filter(|a| LAST_ACTIVE.swap(*a, Ordering::Relaxed) != *a);
    (SCROLL.load(Ordering::Relaxed), follow)
}

/// Stores the offset the draw settled on.
pub(crate) fn store_scroll(offset: usize) {
    SCROLL.store(offset, Ordering::Relaxed);
}

/// Everything one draw of the column needs.
pub(crate) struct ColumnArgs<'a> {
    pub(crate) tabs: &'a [RepoTab],
    pub(crate) active: Option<usize>,
    pub(crate) management: bool,
    pub(crate) folder_hovered: bool,
    pub(crate) hovered_tab: Option<usize>,
    pub(crate) hovered_close: Option<usize>,
    pub(crate) new_tab: NewTabSlot,
    /// The first visible repository tab, before clamping.
    pub(crate) scroll: usize,
    /// A tab to bring into the window.
    pub(crate) follow: Option<usize>,
}

/// What a draw of the column settled on.
pub(crate) struct Column {
    pub(crate) bar: TabBar,
    /// The first visible repository tab, after clamping and following.
    pub(crate) offset: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fixed {
    Folder,
    Chip,
    Radar,
    Workspaces,
    Notes,
    NewTab,
    Plus,
}

/// A full-width row of `text` (padded or cut to `width`), as one span.
fn padded(text: &str, width: u16, style: Style) -> Span<'static> {
    let cut = fit(text, width as usize);
    let pad = (width as usize).saturating_sub(display_width(&cut));
    Span::styled(format!("{cut}{}", " ".repeat(pad)), style)
}

/// A tab-like row: ` name ` then the close cell then a pad cell. Returns the
/// line and the close cell's rect (empty width when the cross is hidden).
fn tab_line(
    area: Rect,
    y: u16,
    name: &str,
    style: Style,
    cross: Style,
    shown: bool,
) -> (Line<'static>, Rect) {
    let label_w = area.width.saturating_sub(2);
    let label = padded(&format!(" {name}"), label_w, style);
    let close = Span::styled(if shown { "x" } else { " " }, cross);
    let rect = Rect::new(area.x + label_w, y, u16::from(shown), 1);
    (
        Line::from(vec![label, close, Span::styled(" ", style)]),
        rect,
    )
}

/// Lays the column out inside `area` (which excludes the rows the caller
/// keeps for the search, settings and folder buttons).
pub(crate) fn tab_column(area: Rect, a: ColumnArgs<'_>) -> Column {
    let active = if a.management { None } else { a.active };
    let home_selected = a.management
        && !a.new_tab.open
        && !a.new_tab.radar.is_some_and(|l| l.open)
        && !a.new_tab.workspaces.is_some_and(|l| l.open)
        && a.new_tab.notes.is_none();
    let n = a.tabs.len();
    let h = area.height as usize;

    // The fixed rows, in the order they are dropped when the column is too
    // short for them (last entry dropped first).
    let mut head = vec![Fixed::Folder];
    if a.new_tab.radar.is_some() {
        head.push(Fixed::Radar);
    }
    if a.new_tab.workspaces.is_some() {
        head.push(Fixed::Workspaces);
    }
    if a.new_tab.notes.is_some() {
        head.push(Fixed::Notes);
    }
    if a.new_tab.chip.is_some() {
        head.insert(1, Fixed::Chip);
    }
    let mut tail = vec![Fixed::Plus];
    if a.new_tab.open {
        tail.insert(0, Fixed::NewTab);
    }
    let want = n.clamp(1, 3);
    let overflow_if = |room: usize| n > room;
    let mut drops = [
        Fixed::Chip,
        Fixed::Notes,
        Fixed::Workspaces,
        Fixed::Radar,
        Fixed::NewTab,
        Fixed::Plus,
    ]
    .into_iter();
    let (has_more, visible) = loop {
        // Rows for the tabs when the overflow row is (not) there.
        let fixed = head.len() + tail.len();
        let room = h.saturating_sub(fixed);
        let with_more = overflow_if(room);
        let room = if with_more {
            room.saturating_sub(1)
        } else {
            room
        };
        if room >= want {
            break (with_more, room);
        }
        match drops.next() {
            Some(d) => {
                head.retain(|f| *f != d);
                tail.retain(|f| *f != d);
            }
            None => break (with_more, room),
        }
    };
    let visible = visible.min(n);

    let mut offset = a.scroll.min(n.saturating_sub(visible));
    if let Some(f) = a.follow.filter(|f| *f < n && visible > 0) {
        if f < offset {
            offset = f;
        } else if f >= offset + visible {
            offset = f + 1 - visible;
        }
    }

    let mut rows: Vec<(Rect, Line<'static>)> = Vec::new();
    let mut bar = TabBar {
        line: Line::default(),
        rows: Vec::new(),
        hits: Vec::new(),
        folder: None,
        plus: None,
        more: None,
        chip: None,
        radar: None,
        radar_close: None,
        workspaces: None,
        workspaces_close: None,
        notes: None,
        notes_close: None,
        scroll_left: None,
        scroll_right: None,
        offset: 0,
    };
    let mut y = area.y;
    let row = |y: u16| Rect::new(area.x, y, area.width, 1);

    for f in &head {
        if y >= area.bottom() {
            break;
        }
        match f {
            Fixed::Folder => {
                let style = if home_selected {
                    Style::default()
                        .fg(th().ui.on_accent)
                        .bg(th().ui.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(if a.folder_hovered {
                            th().ui.text
                        } else {
                            th().ui.accent
                        })
                        .bg(if a.folder_hovered {
                            th().ui.selection
                        } else {
                            th().ui.hover
                        })
                };
                let text = format!(
                    "{FOLDER_BUTTON}{}",
                    gitpoulpi_i18n::msg::settings_value::STARTUP_HOME.get()
                );
                rows.push((row(y), Line::from(padded(&text, area.width, style))));
                bar.folder = Some(row(y));
            }
            Fixed::Chip => {
                if let Some(c) = a.new_tab.chip.as_ref() {
                    let bg = if c.hovered || c.open {
                        th().ui.selection
                    } else {
                        th().ui.hover
                    };
                    let label_style = Style::default()
                        .fg(if c.none { th().ui.muted } else { th().ui.text })
                        .bg(bg);
                    let name = fit(&c.label, area.width.saturating_sub(4) as usize);
                    let used = 3 + display_width(&name);
                    let pad = (area.width as usize).saturating_sub(used);
                    rows.push((
                        row(y),
                        Line::from(vec![
                            Span::styled(" ", label_style),
                            Span::styled(
                                crate::workspace_ui::DOT.to_string(),
                                Style::default().fg(c.dot).bg(bg),
                            ),
                            Span::styled(format!(" {name}{}", " ".repeat(pad)), label_style),
                        ]),
                    ));
                    bar.chip = Some(row(y));
                }
            }
            Fixed::Radar => {
                if let Some(l) = a.new_tab.radar {
                    let style = tab_style(l.open, l.hovered);
                    let shown = l.open || l.hovered || l.close_hovered;
                    let cross = close_style(style, l.open, l.close_hovered);
                    let (line, close) = tab_line(
                        area,
                        y,
                        gitpoulpi_i18n::msg::radar::TITLE.get(),
                        style,
                        cross,
                        shown,
                    );
                    rows.push((row(y), line));
                    bar.radar = Some(row(y));
                    bar.radar_close = shown.then_some(close);
                }
            }
            Fixed::Workspaces => {
                if let Some(l) = a.new_tab.workspaces {
                    let style = tab_style(l.open, l.hovered);
                    let shown = l.open || l.hovered || l.close_hovered;
                    let cross = close_style(style, l.open, l.close_hovered);
                    let (line, close) = tab_line(
                        area,
                        y,
                        gitpoulpi_i18n::msg::workspaces::PAGE_WORKSPACES.get(),
                        style,
                        cross,
                        shown,
                    );
                    rows.push((row(y), line));
                    bar.workspaces = Some(row(y));
                    bar.workspaces_close = shown.then_some(close);
                }
            }
            Fixed::Notes => {
                if let Some(nt) = a.new_tab.notes {
                    let style = tab_style(true, false);
                    let cross = close_style(style, true, nt.close_hovered);
                    let (line, close) = tab_line(
                        area,
                        y,
                        gitpoulpi_i18n::msg::updates::TAB_TITLE.get(),
                        style,
                        cross,
                        true,
                    );
                    rows.push((row(y), line));
                    bar.notes = Some(row(y));
                    bar.notes_close = Some(close);
                }
            }
            _ => {}
        }
        y += 1;
    }

    if n == 0 && !a.new_tab.open && y < area.bottom() {
        let text = format!(" {}", gitpoulpi_i18n::msg::tab_bar::NO_REPO.get());
        rows.push((
            row(y),
            Line::from(padded(
                &text,
                area.width,
                Style::default().fg(th().ui.muted),
            )),
        ));
        y += 1;
    }

    for index in offset..(offset + visible).min(n) {
        if y >= area.bottom() {
            break;
        }
        let is_active = Some(index) == active;
        let hovered = a.hovered_tab == Some(index);
        let style = tab_style(is_active, hovered);
        let cross = close_style(style, is_active, a.hovered_close == Some(index));
        let shown = is_active || hovered || a.hovered_close == Some(index);
        let (line, close) = tab_line(area, y, &a.tabs[index].name, style, cross, shown);
        rows.push((row(y), line));
        bar.hits.push(TabHit {
            index,
            tab: row(y),
            close,
        });
        y += 1;
    }

    if has_more && y < area.bottom() {
        let hovered = a.new_tab.more_hovered;
        let style = Style::default()
            .fg(if hovered {
                th().ui.text
            } else {
                th().ui.accent
            })
            .bg(if hovered {
                th().ui.selection
            } else {
                th().ui.hover
            });
        let text = format!(
            "{OVERFLOW_BUTTON}\u{25B2}{} \u{25BC}{}",
            offset,
            n.saturating_sub(offset + visible)
        );
        rows.push((row(y), Line::from(padded(&text, area.width, style))));
        bar.more = Some(row(y));
        y += 1;
    }

    for f in &tail {
        if y >= area.bottom() {
            break;
        }
        match f {
            Fixed::NewTab => {
                let style = tab_style(true, false);
                let cross = style.fg(th().ui.accent);
                let label = gitpoulpi_i18n::msg::tab_bar::NEW_TAB.get();
                let (line, close) = tab_line(area, y, label, style, cross, true);
                rows.push((row(y), line));
                bar.hits.push(TabHit {
                    index: n,
                    tab: row(y),
                    close,
                });
            }
            Fixed::Plus => {
                let hovered = a.new_tab.plus_hovered;
                let style = Style::default()
                    .fg(if hovered {
                        th().ui.text
                    } else {
                        th().ui.accent
                    })
                    .bg(if hovered {
                        th().ui.selection
                    } else {
                        th().ui.hover
                    });
                rows.push((
                    row(y),
                    Line::from(padded(NEW_TAB_BUTTON, area.width, style)),
                ));
                bar.plus = Some(row(y));
            }
            _ => {}
        }
        y += 1;
    }

    bar.rows = rows;
    Column { bar, offset }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo_switcher::{tab_at, tab_bar_click, SwitcherAction};

    fn tabs(n: usize) -> Vec<RepoTab> {
        (0..n)
            .map(|i| RepoTab {
                path: format!("/r/{i}").into(),
                name: format!("repository-with-a-long-name-{i}"),
                branch: None,
            })
            .collect()
    }

    fn args(tabs: &[RepoTab], active: usize) -> ColumnArgs<'_> {
        ColumnArgs {
            tabs,
            active: Some(active),
            management: false,
            folder_hovered: false,
            hovered_tab: None,
            hovered_close: None,
            new_tab: NewTabSlot::default(),
            scroll: 0,
            follow: Some(active),
        }
    }

    fn text(line: &Line<'_>) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    const AREA: Rect = Rect {
        x: 1,
        y: 2,
        width: 24,
        height: 12,
    };

    #[test]
    fn every_row_is_exactly_as_wide_as_the_column_and_names_are_cut_with_an_ellipsis() {
        let t = tabs(3);
        let col = tab_column(AREA, args(&t, 0));
        assert!(!col.bar.rows.is_empty());
        for (rect, line) in &col.bar.rows {
            assert_eq!(display_width(&text(line)), AREA.width as usize, "{rect:?}");
            assert_eq!(rect.width, AREA.width);
        }
        let first = col.bar.hits[0].index;
        let drawn = col
            .bar
            .rows
            .iter()
            .find(|(r, _)| *r == col.bar.hits[0].tab)
            .map(|(_, l)| text(l))
            .unwrap();
        assert_eq!(first, 0);
        assert!(drawn.contains('\u{2026}'), "{drawn:?}");
    }

    #[test]
    fn home_comes_first_then_the_tabs_in_order_then_the_plus() {
        let t = tabs(3);
        let col = tab_column(AREA, args(&t, 1));
        let folder = col.bar.folder.unwrap();
        let plus = col.bar.plus.unwrap();
        assert_eq!(folder.y, AREA.y);
        let ys: Vec<u16> = col.bar.hits.iter().map(|h| h.tab.y).collect();
        assert_eq!(ys, vec![AREA.y + 1, AREA.y + 2, AREA.y + 3]);
        assert_eq!(plus.y, AREA.y + 4);
        assert!(col.bar.more.is_none());
    }

    #[test]
    fn the_close_cross_is_only_on_the_active_or_hovered_tab_and_is_hit_first() {
        let t = tabs(3);
        let mut a = args(&t, 1);
        a.hovered_tab = Some(2);
        let col = tab_column(AREA, a);
        let hits = &col.bar.hits;
        assert_eq!(hits[0].close.width, 0);
        assert_eq!(hits[1].close.width, 1);
        assert_eq!(hits[2].close.width, 1);
        let cross = hits[1].close;
        assert_eq!(cross.x, AREA.right() - 2);
        assert_eq!(
            tab_bar_click(hits, cross.x, cross.y),
            Some(SwitcherAction::Close(1))
        );
        assert_eq!(
            tab_bar_click(hits, hits[1].tab.x + 1, hits[1].tab.y),
            Some(SwitcherAction::Focus(1))
        );
        assert_eq!(tab_at(hits, cross.x, cross.y), Some(1));
    }

    #[test]
    fn more_tabs_than_rows_scroll_and_the_overflow_row_counts_the_hidden_ones() {
        let t = tabs(30);
        let col = tab_column(AREA, args(&t, 0));
        let more = col.bar.more.expect("overflow row");
        let drawn = text(&col.bar.rows.iter().find(|(r, _)| *r == more).unwrap().1);
        assert!(drawn.contains('\u{2026}'), "{drawn:?}");
        let visible = col.bar.hits.len();
        assert!((3..30).contains(&visible));
        assert!(
            drawn.contains(&format!("\u{25BC}{}", 30 - visible)),
            "{drawn:?}"
        );
        // Following a tab far down brings it into the window.
        let mut a = args(&t, 25);
        a.follow = Some(25);
        let far = tab_column(AREA, a);
        assert!(far.bar.hits.iter().any(|h| h.index == 25));
        assert!(far.offset > 0);
        // Without `follow` the scroll stays where the wheel put it.
        let mut a = args(&t, 25);
        a.follow = None;
        a.scroll = 4;
        let held = tab_column(AREA, a);
        assert_eq!(held.offset, 4);
        assert_eq!(held.bar.hits[0].index, 4);
        // Scrolling past the end clamps.
        let mut a = args(&t, 0);
        a.follow = None;
        a.scroll = 999;
        let end = tab_column(AREA, a);
        assert_eq!(end.bar.hits.last().unwrap().index, 29);
    }

    /// The rows really paint what the rects say: the cross glyph sits under
    /// the close rect, the Home glyph on the first row, the plus on its own.
    #[test]
    fn the_painted_buffer_matches_the_rects() {
        use ratatui::buffer::Buffer;
        use ratatui::widgets::{Paragraph, Widget};
        let t = tabs(3);
        let col = tab_column(AREA, args(&t, 1));
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 20));
        for (rect, line) in &col.bar.rows {
            Paragraph::new(line.clone()).render(*rect, &mut buf);
        }
        let cross = col.bar.hits[1].close;
        assert_eq!(buf[(cross.x, cross.y)].symbol(), "x");
        let folder = col.bar.folder.unwrap();
        let row: String = (folder.x..folder.right())
            .map(|x| buf[(x, folder.y)].symbol().to_string())
            .collect();
        assert!(row.contains('\u{2302}'), "{row:?}");
        let plus = col.bar.plus.unwrap();
        assert_eq!(buf[(plus.x + 1, plus.y)].symbol(), "+");
        // Nothing is painted outside the column.
        assert_eq!(buf[(AREA.right(), AREA.y)].symbol(), " ");
        // The selected tab has the selected background, its neighbour not.
        let sel = buf[(col.bar.hits[1].tab.x + 2, col.bar.hits[1].tab.y)].bg;
        let rest = buf[(col.bar.hits[0].tab.x + 2, col.bar.hits[0].tab.y)].bg;
        assert_ne!(sel, rest);
    }

    #[test]
    fn a_very_short_column_keeps_home_and_a_few_tabs_inside_its_rect() {
        let t = tabs(8);
        for height in 0..8 {
            let area = Rect::new(0, 0, 24, height);
            let col = tab_column(area, args(&t, 0));
            for (r, _) in &col.bar.rows {
                assert!(r.bottom() <= area.bottom(), "{height}: {r:?}");
            }
            for h in &col.bar.hits {
                assert!(h.tab.bottom() <= area.bottom());
            }
        }
    }
}
