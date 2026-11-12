//! The help popup (`?`): a filter field like the other popups', a row of
//! tabs (the key bindings table, then one concise guide per area of the user
//! docs), the body with the shared scrollbar, and a hint band whose second
//! line is a link to the matching user docs page on GitHub (`Ctrl+D` or a
//! click opens it through the same browser path as every other link).
//! Tabs switch with `Tab` / `Shift+Tab`, `Left` / `Right`, `Alt+1`..`Alt+8`
//! or a click; the filter narrows the current tab.

use crate::help_guide::{docs_url, lines as guide_lines, TABS};
use crate::popup_list::View;
use crate::ui_kit::{centered_rect, display_width, truncate_with_ellipsis, ClippedRender};
use crate::wide_edge::Clear;
use crate::{text_field, th};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;
use std::cell::{Cell, RefCell};

thread_local! {
    /// The active tab (kept while the popup is closed: it reopens where it was).
    static TAB: Cell<usize> = const { Cell::new(0) };
    /// Where the pointer last was over the popup (the hover highlight is
    /// worked out at paint time from the headers' rects: `forget` runs every
    /// frame, so it must not own this).
    static POINTER: Cell<Option<(u16, u16)>> = const { Cell::new(None) };
    /// The tab headers as the last frame painted them.
    static TAB_RECTS: RefCell<Vec<Rect>> = const { RefCell::new(Vec::new()) };
    /// The popup, the docs link row and the body list as the last frame
    /// painted them (hit rects come from the paint walk).
    static RECTS: Cell<Option<Rects>> = const { Cell::new(None) };
}

#[derive(Clone, Copy)]
struct Rects {
    popup: Rect,
    link: Rect,
    /// The close cross on the popup's top border (`None` when too narrow).
    cross: Option<Rect>,
}

/// Whether `(col, row)` is on the close cross, as last painted.
pub(crate) fn on_cross(col: u16, row: u16) -> bool {
    RECTS
        .get()
        .and_then(|r| r.cross)
        .is_some_and(|c| c.contains((col, row).into()))
}

/// Whether `(col, row)` is inside the popup, as last painted.
pub(crate) fn contains(col: u16, row: u16) -> bool {
    RECTS
        .get()
        .is_some_and(|r| r.popup.contains((col, row).into()))
}

/// Whether `(col, row)` is on the docs link row.
pub(crate) fn on_link(col: u16, row: u16) -> bool {
    RECTS
        .get()
        .is_some_and(|r| r.link.contains((col, row).into()))
}

/// The active tab.
pub(crate) fn tab() -> usize {
    TAB.get().min(TABS.len() - 1)
}

/// Selects tab `i` (clamped).
pub(crate) fn set_tab(i: usize) {
    TAB.set(i.min(TABS.len() - 1));
}

/// Moves to the next (`+1`) or previous (`-1`) tab, wrapping round.
pub(crate) fn step_tab(delta: i32) {
    let n = TABS.len() as i32;
    set_tab((tab() as i32 + delta).rem_euclid(n) as usize);
}

/// The tab header at `(col, row)`, as last painted.
pub(crate) fn tab_at(col: u16, row: u16) -> Option<usize> {
    TAB_RECTS.with_borrow(|r| r.iter().position(|t| t.contains((col, row).into())))
}

/// Records the pointer position (the hover highlight).
pub(crate) fn hover(col: u16, row: u16) {
    POINTER.set(Some((col, row)));
}

/// Opens the active tab's docs page; the status line's answer.
pub(crate) fn open_docs() -> String {
    let url = docs_url(tab());
    match crate::repo_menu::open_url(&url) {
        Ok(()) => gitpoulpi_i18n::msg::help_page::opened_docs(&url),
        Err(e) => crate::activity_log::report_problem_clear(
            crate::activity_log::ProblemSource::Tool,
            &gitpoulpi_i18n::msg::help_page::open_docs_failed(&url, e),
        ),
    }
}

/// The tab headers' text and x offsets inside `width` cells: ` label ` each,
/// one space between; when they do not fit the longest labels are cut.
fn tab_cells(width: usize) -> Vec<(usize, String)> {
    let labels: Vec<&str> = TABS.iter().map(|t| t.label.get()).collect();
    let n = labels.len();
    let used_at = |max: usize| -> usize {
        labels
            .iter()
            .map(|l| display_width(l).min(max) + 2)
            .sum::<usize>()
            + (n - 1)
    };
    let mut max_label = labels.iter().map(|l| display_width(l)).max().unwrap_or(1);
    while max_label > 1 && used_at(max_label) > width {
        max_label -= 1;
    }
    let mut x = 0;
    labels
        .iter()
        .map(|l| {
            let label = if display_width(l) > max_label {
                truncate_with_ellipsis(l, max_label)
            } else {
                (*l).to_string()
            };
            let cell = format!(" {label} ");
            let at = x;
            x += display_width(&cell) + 1;
            (at, cell)
        })
        .collect()
}

/// Paints the popup. `scroll` is clamped to the content and `inner_height`
/// receives the visible body rows (the `PgUp`/`PgDn` step).
pub(crate) fn render(frame: &mut Frame, filter: &str, scroll: &mut u16, inner_height: &mut u16) {
    let popup = WIN.place(frame.area(), centered_rect(70, 80, frame.area()));
    frame.render_clipped(Clear, popup);
    let [input_area, tabs_area, body_area, hint_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(popup);
    let tab = tab();
    let inner_w = body_area.width.saturating_sub(2) as usize;
    let help_body: Vec<Line<'static>> = if tab == 0 {
        crate::chrome::help_body(filter, inner_w)
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect()
    } else {
        guide_lines(tab, filter, inner_w)
    };
    let total_lines = help_body.len();
    let rows = body_area.height as usize;
    *inner_height = body_area.height.max(1);
    *scroll = (*scroll).min(total_lines.saturating_sub(rows) as u16);

    let title = match (tab, filter.is_empty()) {
        (0, true) => gitpoulpi_i18n::msg::help_page::TITLE.get().to_string(),
        (0, false) => gitpoulpi_i18n::msg::help_page::title_filtered(total_lines as u64, filter),
        (_, true) => gitpoulpi_i18n::msg::help_tabs::TITLE.get().to_string(),
        (_, false) => gitpoulpi_i18n::msg::help_tabs::title_filtered(filter),
    };
    let draft_w = input_area.width.saturating_sub(2) as usize;
    let draft_line = text_field::edit_line(
        filter,
        None,
        draft_w,
        true,
        text_field::EditLook::new(Style::default().bg(crate::input_focus_bg()), "_"),
    );
    frame.render_clipped(
        Paragraph::new(draft_line).block(
            Block::default()
                .title(format!(" {title} "))
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(th().ui.accent)),
        ),
        input_area,
    );

    // The tab headers: hit rects come from this paint.
    let tab_inner = Rect::new(
        tabs_area.x + 1,
        tabs_area.y,
        tabs_area.width.saturating_sub(2),
        1,
    );
    let pointer = POINTER.get();
    let mut spans: Vec<Span> = Vec::new();
    let mut rects: Vec<Rect> = Vec::new();
    let mut cursor = 0usize;
    for (i, (at, cell)) in tab_cells(tab_inner.width as usize).into_iter().enumerate() {
        if at > cursor {
            spans.push(Span::raw(" ".repeat(at - cursor)));
        }
        let w = display_width(&cell);
        let rect =
            Rect::new(tab_inner.x + at as u16, tab_inner.y, w as u16, 1).intersection(tab_inner);
        let over = pointer.is_some_and(|p| rect.contains(p.into()));
        let style = if i == tab {
            Style::default()
                .fg(th().ui.on_accent)
                .bg(th().ui.accent)
                .add_modifier(Modifier::BOLD)
        } else if over {
            Style::default().fg(th().ui.text).bg(th().ui.hover)
        } else {
            Style::default().fg(th().ui.muted)
        };
        spans.push(Span::styled(cell, style));
        rects.push(rect);
        cursor = at + w;
    }
    TAB_RECTS.with_borrow_mut(|r| *r = rects);
    frame.render_clipped(
        Paragraph::new(Line::from(spans)).block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(th().ui.accent)),
        ),
        tabs_area,
    );

    frame.render_clipped(
        Paragraph::new(help_body).scroll((*scroll, 0)).block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(Style::default().fg(th().ui.accent)),
        ),
        body_area,
    );
    View::set_window(*scroll as usize, rows, total_lines);
    View::paint_bar(frame.buffer_mut(), body_area, total_lines);

    // The address shown starts at `docs/`: the whole one is wider than the popup.
    let shown_url = docs_url(tab);
    let shown_url = shown_url
        .split_once("/main/")
        .map_or(shown_url.as_str(), |(_, p)| p);
    let link_text = gitpoulpi_i18n::msg::help_page::docs_link(shown_url);
    frame.render_clipped(
        Paragraph::new(vec![
            ratatui::text::Line::styled(
                gitpoulpi_i18n::msg::help_tabs::HINT.get(),
                Style::default().fg(th().ui.muted),
            ),
            ratatui::text::Line::styled(
                link_text,
                Style::default()
                    .fg(th().ui.accent)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ])
        .block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(Style::default().fg(th().ui.accent)),
        ),
        hint_area,
    );
    let link = Rect::new(
        hint_area.x + 1,
        hint_area.y + 1,
        hint_area.width.saturating_sub(2),
        1,
    );
    let over_cross = pointer.is_some_and(|(c, r)| {
        let spot = Rect::new(input_area.right().saturating_sub(5), input_area.y, 3, 1);
        spot.contains((c, r).into())
    });
    let cross = crate::chrome::close_cross::paint(frame.buffer_mut(), input_area, over_cross);
    RECTS.set(Some(Rects { popup, link, cross }));
    // Movable and resizable like the other popups; `Ctrl+N` badges the filter
    // box, the tabs, the first body row and the docs link.
    WIN.decorate(frame.buffer_mut(), popup, cross, true);
    crate::dialog_kit::frame::offer_list_popup(input_area, body_area);
    crate::dialog_kit::frame::offer(&TAB_RECTS.with_borrow(Clone::clone));
    crate::dialog_kit::frame::offer(&[link]);
}

/// The help overlay moves and resizes: the body takes the rows.
const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::resizable("help", 40, 10);

/// Forgets the rects once the popup is closed (a stale one is a click on a
/// popup that is no longer there).
pub(crate) fn forget() {
    RECTS.set(None);
    TAB_RECTS.with_borrow_mut(Vec::clear);
}

#[cfg(test)]
#[path = "help_view_tests.rs"]
mod tests;
