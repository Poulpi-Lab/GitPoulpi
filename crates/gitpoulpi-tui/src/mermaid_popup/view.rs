//! "Open diagram": the read-only viewer of one ```mermaid block of a
//! conversation (the button under the block, its links-menu row and the
//! palette command all end here). The terminal draws no graphics, so the
//! viewer shows the diagram's script, scrollable, with a Copy button that
//! hands it to any renderer. Modal like the Copy as gitGraph popup beside it
//! (`mermaid_popup` dispatches to this when it is the one open).
//!
//! Keys: Up/Down/j/k, PageUp/PageDown, Home/End scroll; Enter copies; Esc
//! closes. Mouse: wheel and scrollbar = those keys, the Copy and Close
//! buttons, the cross, a click away = Esc.

use std::sync::Mutex;

use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_i18n::msg::mermaid as msg;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, BorderType, Borders, Widget};
use ratatui::Frame;

use super::Outcome;
use crate::dialog_kit::{self, ButtonState};
use crate::scrollbar::{self, BarDrag, Track};
use crate::th;

#[derive(Default)]
struct Rects {
    outer: Rect,
    copy: Rect,
    close: Rect,
}

struct View {
    text: String,
    lines: Vec<String>,
    top: usize,
    rows: usize,
    pointer: Option<(u16, u16)>,
    bar: BarDrag,
    track: Option<Track>,
    rects: Rects,
    cross: dialog_kit::CloseHit,
}

static VIEW: Mutex<Option<View>> = Mutex::new(None);

impl View {
    fn max_top(&self) -> usize {
        self.lines.len().saturating_sub(self.rows.max(1))
    }

    fn scroll_to(&mut self, top: usize) {
        self.top = top.min(self.max_top());
    }

    fn scroll_by(&mut self, delta: isize) {
        self.scroll_to((self.top as isize + delta).max(0) as usize);
    }
}

/// Opens the viewer on a diagram's script.
pub(crate) fn open_script(text: &str) {
    let text: String = text
        .chars()
        .filter(|c| !crate::md_render::is_banned(*c))
        .collect();
    let view = View {
        lines: text.lines().map(|l| l.replace('\t', "    ")).collect(),
        text,
        top: 0,
        rows: 10,
        pointer: None,
        bar: BarDrag::default(),
        track: None,
        rects: Rects::default(),
        cross: Default::default(),
    };
    if let Ok(mut slot) = VIEW.lock() {
        *slot = Some(view);
    }
}

pub(super) fn is_open() -> bool {
    VIEW.lock().is_ok_and(|v| v.is_some())
}

fn copy_out(slot: &mut Option<View>) -> Outcome {
    match slot.take() {
        Some(view) => Outcome::CopyDiagram(view.text),
        None => Outcome::None,
    }
}

pub(super) fn on_key(key: KeyEvent) -> Outcome {
    let Ok(mut slot) = VIEW.lock() else {
        return Outcome::None;
    };
    let Some(view) = slot.as_mut() else {
        return Outcome::None;
    };
    let page = view.rows.max(2) - 1;
    match key.code {
        KeyCode::Esc => *slot = None,
        KeyCode::Up | KeyCode::Char('k') => view.scroll_by(-1),
        KeyCode::Down | KeyCode::Char('j') => view.scroll_by(1),
        KeyCode::PageUp => view.scroll_by(-(page as isize)),
        KeyCode::PageDown => view.scroll_by(page as isize),
        KeyCode::Home => view.scroll_to(0),
        KeyCode::End => view.scroll_to(usize::MAX),
        KeyCode::Enter => return copy_out(&mut slot),
        _ => {}
    }
    Outcome::None
}

pub(super) fn on_mouse(mouse: MouseEvent) -> Outcome {
    let Ok(mut slot) = VIEW.lock() else {
        return Outcome::None;
    };
    let Some(view) = slot.as_mut() else {
        return Outcome::None;
    };
    let at: ratatui::layout::Position = (mouse.column, mouse.row).into();
    view.pointer = Some((mouse.column, mouse.row));
    if matches!(mouse.kind, MouseEventKind::Moved | MouseEventKind::Drag(_)) {
        view.cross.set_hover(mouse.column, mouse.row);
    }
    if let Some(top) = view
        .bar
        .on_mouse(view.track, mouse, view.lines.len(), view.top)
    {
        view.scroll_to(top);
        return Outcome::None;
    }
    match mouse.kind {
        MouseEventKind::ScrollUp => view.scroll_by(-3),
        MouseEventKind::ScrollDown => view.scroll_by(3),
        MouseEventKind::Down(MouseButton::Left) => {
            if view.rects.copy.contains(at) {
                return copy_out(&mut slot);
            }
            if view.rects.close.contains(at)
                || view.cross.hit(mouse.column, mouse.row)
                || !view.rects.outer.contains(at)
            {
                *slot = None;
            }
        }
        _ => {}
    }
    Outcome::None
}

const WIN: dialog_kit::Win = dialog_kit::Win::resizable("mermaid_view", 50, 12);

pub(super) fn render(frame: &mut Frame) {
    let Ok(mut slot) = VIEW.lock() else {
        return;
    };
    let Some(view) = slot.as_mut() else {
        return;
    };
    let area = frame.area();
    if area.width < 30 || area.height < 10 {
        let outer = dialog_kit::too_small::paint_closable(
            frame.buffer_mut(),
            area,
            msg::VIEW_TITLE.get(),
            &mut view.cross,
        );
        view.rects = Rects {
            outer,
            ..Rects::default()
        };
        return;
    }
    let outer = WIN.place(
        area,
        dialog_kit::centered(
            area,
            100.min(area.width.saturating_sub(2)),
            30.min(area.height.saturating_sub(2)),
        ),
    );
    let buf = frame.buffer_mut();
    view.cross
        .framed(buf, outer, msg::VIEW_TITLE.get(), &WIN, true);
    view.rects = Rects {
        outer,
        ..Rects::default()
    };
    let inner_x = outer.x + 2;
    let inner_w = outer.width.saturating_sub(4);
    let hovered = |r: Rect, p: Option<(u16, u16)>| p.is_some_and(|p| r.contains(p.into()));
    let preview = Rect::new(
        outer.x + 1,
        outer.y + 1,
        outer.width.saturating_sub(2),
        outer.height.saturating_sub(1 + 4),
    );
    view.rows = (preview.height as usize).saturating_sub(2).max(1);
    view.top = view.top.min(view.max_top());
    Block::default()
        .title(format!(" {} ", msg::PREVIEW.get()))
        .title_style(Style::default().fg(th().ui.muted))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(th().ui.accent))
        .style(Style::default().bg(th().ui.panel))
        .render(preview, buf);
    let text_w = preview.width.saturating_sub(3);
    for (i, line) in view.lines.iter().skip(view.top).take(view.rows).enumerate() {
        super::put_script_line(buf, preview.x + 1, preview.y + 1 + i as u16, text_w, line);
    }
    view.track = Track::vertical(preview, view.lines.len(), view.top);
    if let Some(track) = view.track {
        scrollbar::render_buf(buf, track, true, false);
    }
    let by = outer.bottom().saturating_sub(3);
    let copy_label = msg::COPY_BUTTON.get();
    let close_label = msg::CLOSE_BUTTON.get();
    let copy_rect = Rect::new(inner_x, by, dialog_kit::button_width(copy_label), 1);
    let close_rect = Rect::new(
        copy_rect.right() + 2,
        by,
        dialog_kit::button_width(close_label),
        1,
    );
    let pointer = view.pointer;
    view.rects.copy = dialog_kit::button(
        buf,
        copy_rect.x,
        by,
        copy_label,
        ButtonState::Primary {
            hovered: hovered(copy_rect, pointer),
            focused: false,
        },
    );
    view.rects.close = dialog_kit::button(
        buf,
        close_rect.x,
        by,
        close_label,
        ButtonState::Normal {
            hovered: hovered(close_rect, pointer),
            focused: false,
        },
    );
    dialog_kit::hints(
        buf,
        inner_x,
        outer.bottom().saturating_sub(2),
        inner_w,
        &[
            ("Enter", msg::HINT_COPY.get()),
            ("\u{2191}/\u{2193}", msg::HINT_SCROLL.get()),
            ("Esc", msg::HINT_CLOSE.get()),
        ],
    );
}
