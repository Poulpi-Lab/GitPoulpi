//! The workspace search popup's paint: a query field, the kind chips, a
//! grouped list (kind icon, repository chip, label with the matched
//! characters highlighted, a dim detail) and a hint band. The rects the
//! mouse needs (the list's, each chip's) are captured from this walk, never
//! recomputed.

use std::cell::RefCell;

use crate::wide_edge::Clear;
use gitpoulpi_core::domain::SearchKind;
use gitpoulpi_i18n::msg;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};

use super::model::{Item, Row, ORDER};
use super::WorkspaceSearch;
use crate::ui_kit::{centered_rect, display_width, truncate_with_ellipsis, ClippedRender};
use crate::{text_field, th};

thread_local! {
    /// Where each kind chip was painted last frame: its rect and its slot in
    /// [`ORDER`]. A chip that did not fit is neither drawn nor listed.
    static CHIPS: RefCell<Vec<(Rect, usize)>> = const { RefCell::new(Vec::new()) };
}

thread_local! {
    /// The close cross's rect as painted last frame (`None` when not drawn).
    static CLOSE: std::cell::Cell<Option<Rect>> = const { std::cell::Cell::new(None) };
}

/// Whether `(col, row)` is on the popup's close cross.
pub(crate) fn close_at(col: u16, row: u16) -> bool {
    CLOSE
        .with(|c| c.get())
        .is_some_and(|r| r.contains((col, row).into()))
}

/// The chip under `(col, row)`, as its slot in [`ORDER`].
pub(crate) fn chip_at(col: u16, row: u16) -> Option<usize> {
    CHIPS.with(|c| {
        c.borrow()
            .iter()
            .find(|(r, _)| r.contains((col, row).into()))
            .map(|(_, slot)| *slot)
    })
}

/// The kind's glyph. Every one has default emoji presentation (no VS16), the
/// commit dot is a text-default BMP shape: see "Emoji presentation" in
/// `docs/developer/where-we-are.md`. Always two cells wide when drawn
/// ([`icon_cell`] pads the one-cell dot).
fn icon(kind: SearchKind, item: Option<&Item>) -> &'static str {
    match kind {
        SearchKind::Repo => crate::theme::FOLDER_CLOSED_ICON,
        SearchKind::Branch => {
            let remote = matches!(
                item.map(|i| &i.target),
                Some(gitpoulpi_core::domain::SearchTarget::Ref { name }) if name.starts_with("refs/remotes/")
            );
            if remote {
                crate::theme::REMOTE_ICON
            } else {
                crate::theme::LOCAL_ICON
            }
        }
        SearchKind::Tag => crate::theme::TAG_ICON,
        SearchKind::Worktree => crate::theme::WORKTREE_ICON,
        SearchKind::Stash => crate::theme::STASH_ICON,
        SearchKind::Commit => "\u{25cf}",
        SearchKind::PullRequest => crate::theme::PULL_REQUEST_ICON,
        SearchKind::Issue => crate::theme::ISSUE_ICON,
    }
}

fn icon_cell(kind: SearchKind, item: Option<&Item>) -> String {
    let glyph = icon(kind, item);
    if display_width(glyph) >= 2 {
        glyph.to_string()
    } else {
        format!("{glyph} ")
    }
}

/// The kind's name, as a chip and as a group heading.
pub(crate) fn kind_name(kind: SearchKind) -> &'static str {
    match kind {
        SearchKind::Repo => msg::search::KIND_REPO.get(),
        SearchKind::Branch => msg::search::KIND_BRANCH.get(),
        SearchKind::Tag => msg::search::KIND_TAG.get(),
        SearchKind::Worktree => msg::search::KIND_WORKTREE.get(),
        SearchKind::Stash => msg::search::KIND_STASH.get(),
        SearchKind::Commit => msg::search::KIND_COMMIT.get(),
        SearchKind::PullRequest => msg::search::KIND_PULL_REQUEST.get(),
        SearchKind::Issue => msg::search::KIND_ISSUE.get(),
    }
}

/// `label` as spans: matched characters (`positions`, indices into the
/// lower-cased text) in the accent color, bold, the rest in `base`; cut at
/// `max_w` columns with an ellipsis.
pub(crate) fn highlighted(
    label: &str,
    positions: &[u32],
    max_w: usize,
    base: Style,
) -> Vec<Span<'static>> {
    // Lower-casing can change the character count (a few scripts): then the
    // positions do not index the label and nothing is highlighted.
    let usable = label.to_lowercase().chars().count() == label.chars().count();
    let hi = base.fg(th().ui.accent).add_modifier(Modifier::BOLD);
    let total = display_width(label);
    let (budget, ellipsis) = if total > max_w {
        (max_w.saturating_sub(1), true)
    } else {
        (max_w, false)
    };
    let mut spans: Vec<Span<'static>> = Vec::new();
    let (mut run, mut run_hi) = (String::new(), false);
    let mut used = 0;
    for (i, c) in label.chars().enumerate() {
        let w = display_width(c.encode_utf8(&mut [0u8; 4]));
        if used + w > budget {
            break;
        }
        used += w;
        let on = usable && positions.binary_search(&(i as u32)).is_ok();
        if on != run_hi && !run.is_empty() {
            spans.push(Span::styled(
                std::mem::take(&mut run),
                if run_hi { hi } else { base },
            ));
        }
        run_hi = on;
        run.push(c);
    }
    if !run.is_empty() {
        spans.push(Span::styled(run, if run_hi { hi } else { base }));
    }
    if ellipsis {
        spans.push(Span::styled("\u{2026}", base));
    }
    spans
}

fn title(search: &WorkspaceSearch) -> String {
    let n = search.scope().repos.len() as u64;
    let mut title = match &search.scope().workspace {
        Some(name) => msg::search::title_workspace(name, n),
        None => msg::search::title_tabs(n),
    };
    if let Some(spinner) = search.spinner() {
        title.push(' ');
        title.push(spinner);
    }
    title
}

/// The chips line, between the popup's side borders. Returns the line and
/// records the rects.
fn chips_line(search: &WorkspaceSearch, area: Rect) -> Line<'static> {
    let border = Style::default().fg(th().ui.accent);
    let inner = area.width.saturating_sub(4) as usize;
    // The richest form that fits: icon and name, then the name, then the icon.
    let text_of = |level: u8, kind: SearchKind| match level {
        0 => format!(" {} {} ", icon_cell(kind, None).trim_end(), kind_name(kind)),
        1 => format!(" {} ", kind_name(kind)),
        _ => format!(" {} ", icon_cell(kind, None).trim_end()),
    };
    let level = (0..2u8)
        .find(|l| {
            ORDER
                .iter()
                .map(|k| display_width(&text_of(*l, *k)) + 1)
                .sum::<usize>()
                <= inner
        })
        .unwrap_or(2);
    let mut spans = vec![Span::styled("\u{2502} ", border)];
    let mut x = area.x + 2;
    let right = area.right().saturating_sub(2);
    let mut found: Vec<(Rect, usize)> = Vec::new();
    for (slot, kind) in ORDER.iter().enumerate() {
        let text = text_of(level, *kind);
        let w = display_width(&text) as u16;
        if x + w > right {
            break;
        }
        let style = if search.chips[slot] {
            Style::default()
                .fg(th().ui.on_accent)
                .bg(th().ui.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(th().ui.muted)
        };
        found.push((Rect::new(x, area.y, w, 1), slot));
        crate::pointer_map::hand(Rect::new(x, area.y, w, 1));
        spans.push(Span::styled(text, style));
        spans.push(Span::raw(" "));
        x += w + 1;
    }
    let used = (x - area.x) as usize;
    spans.push(Span::raw(
        " ".repeat((area.width as usize).saturating_sub(used + 1)),
    ));
    spans.push(Span::styled("\u{2502}", border));
    CHIPS.with(|c| *c.borrow_mut() = found);
    Line::from(spans)
}

fn empty_text(search: &WorkspaceSearch) -> &'static str {
    if search.scope().repos.is_empty() {
        msg::search::EMPTY_NO_REPOSITORIES.get()
    } else if search.busy() {
        msg::search::EMPTY_SEARCHING.get()
    } else if search.draft.trim().is_empty() {
        msg::search::EMPTY_NOTHING_YET.get()
    } else {
        msg::search::EMPTY_NO_MATCH.get()
    }
}

fn item_line(
    search: &WorkspaceSearch,
    item: &Item,
    selected: bool,
    inner_w: usize,
) -> Line<'static> {
    let mut base = Style::default().fg(th().ui.text);
    let mut dim = Style::default().fg(th().ui.muted);
    if selected {
        base = base.bg(th().ui.selection).add_modifier(Modifier::BOLD);
        dim = dim.bg(th().ui.selection);
    }
    let icon = icon_cell(item.kind, Some(item));
    let chip = search
        .scope()
        .repos
        .get(item.repo)
        .map(|r| format!(" {} ", truncate_with_ellipsis(&r.name, 16)))
        .unwrap_or_default();
    let chip_w = display_width(&chip);
    let icon_w = display_width(&icon);
    let left_budget = inner_w.saturating_sub(icon_w + 1 + chip_w + 1);
    let label_w = display_width(&item.label);
    let label_budget = left_budget.min(label_w);
    let mut spans = vec![Span::styled(format!("{icon} "), base)];
    spans.extend(highlighted(
        &item.label,
        &item.positions,
        label_budget,
        base,
    ));
    let mut used = icon_w + 1 + label_budget.min(label_w);
    let detail_room = left_budget.saturating_sub(label_budget + 2);
    if !item.detail.is_empty() && detail_room >= 4 {
        let detail = truncate_with_ellipsis(&item.detail, detail_room);
        used += 2 + display_width(&detail);
        spans.push(Span::styled(format!("  {detail}"), dim));
    }
    let pad = inner_w.saturating_sub(used + chip_w);
    spans.push(Span::styled(" ".repeat(pad), base));
    if chip_w > 0 {
        let chip_style = if selected {
            Style::default().fg(th().ui.accent).bg(th().ui.selection)
        } else {
            Style::default().fg(th().ui.accent)
        };
        spans.push(Span::styled(chip, chip_style));
    }
    Line::from(spans)
}

/// Paints the popup and returns its rects, captured from this walk.
pub(crate) fn render(
    frame: &mut ratatui::Frame,
    search: &WorkspaceSearch,
    field_editor: &crate::text_edit::FieldEditor<crate::modes::MainField>,
) -> crate::chrome::PalettePopup {
    let field = crate::modes::MainField::WorkspaceSearch;
    let popup = WIN.place(frame.area(), centered_rect(86, 72, frame.area()));
    frame.render_clipped(Clear, popup);
    let [input_area, chips_area, list_area, hint_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(popup);
    let accent = Style::default().fg(th().ui.accent);

    let draft_w = input_area.width.saturating_sub(2);
    field_editor.drawn(
        field,
        Rect::new(input_area.x + 1, input_area.y + 1, draft_w, 1),
        false,
    );
    let draft_line = text_field::edit_line(
        &search.draft,
        field_editor.caret(field),
        draft_w as usize,
        true,
        text_field::EditLook::new(Style::default().bg(crate::input_focus_bg()), "_"),
    );
    frame.render_clipped(
        Paragraph::new(draft_line).block(
            Block::default()
                .title(format!(" {} ", title(search)))
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .border_style(accent),
        ),
        input_area,
    );
    let close_hovered = crate::chrome::popup_pointer().is_some_and(|(x, y)| {
        Rect::new(input_area.right().saturating_sub(5), input_area.y, 3, 1).contains((x, y).into())
    });
    CLOSE.with(|c| {
        c.set(crate::chrome::close_cross::paint(
            frame.buffer_mut(),
            input_area,
            close_hovered,
        ))
    });
    frame.render_clipped(Paragraph::new(chips_line(search, chips_area)), chips_area);

    let rows = search.rows();
    let height = list_area.height as usize;
    let window_start = crate::popup_list::View::window_start(search.selected, height, rows.len());
    let inner_w = list_area.width.saturating_sub(2) as usize;
    let items: Vec<ListItem> = if rows.is_empty() {
        vec![ListItem::new(format!("  {}", empty_text(search)))
            .style(Style::default().fg(th().ui.muted))]
    } else {
        rows.iter()
            .enumerate()
            .skip(window_start)
            .take(height)
            .map(|(i, row)| match row {
                Row::Heading(kind, count) => ListItem::new(Line::from(vec![
                    Span::styled(
                        format!(" {} ", kind_name(*kind)),
                        Style::default()
                            .fg(th().ui.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!("({count})"), Style::default().fg(th().ui.muted)),
                ])),
                Row::Item(n) => ListItem::new(item_line(
                    search,
                    &search.items()[*n],
                    i == search.selected,
                    inner_w,
                )),
            })
            .collect()
    };
    frame.render_clipped(
        List::new(items).block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT | Borders::RIGHT)
                .border_style(accent),
        ),
        list_area,
    );
    crate::popup_list::View::paint_bar(frame.buffer_mut(), list_area, rows.len());
    let rects = crate::chrome::PalettePopup {
        popup,
        list: list_area,
        window_start,
        count: rows.len(),
        segments: [Rect::ZERO; 4],
    };
    crate::chrome::paint_popup_row_hover(frame.buffer_mut(), &rects, search.selected);
    frame.render_clipped(
        Paragraph::new(if search.review_request().is_some() {
            msg::search::HINT_REVIEW.get()
        } else {
            msg::search::HINT.get()
        })
        .style(Style::default().fg(th().ui.muted))
        .block(
            Block::default()
                .border_type(BorderType::Rounded)
                .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                .border_style(accent),
        ),
        hint_area,
    );
    let cross = CLOSE.with(|c| c.get());
    WIN.decorate(frame.buffer_mut(), popup, cross, true);
    crate::dialog_kit::frame::offer_list_popup(input_area, list_area);
    rects
}

/// The search moves and resizes: the list grows with the height.
const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::resizable("workspace_search", 50, 12);
