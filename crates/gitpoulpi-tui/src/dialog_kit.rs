//! Drawing primitives shared by the repository dialogs (the folder
//! picker, Init, Clone).
//!
//! The dialogs draw straight into a `Buffer` rather than building
//! `Line`s: they are two-column layouts with buttons and fields placed
//! at computed columns, and a `Buffer` lets every piece return the rect
//! it landed in -- which is how the hit rects come out of the same walk
//! that paints (the `log_table` / `tab_bar` rule). Every text piece is
//! measured in terminal columns (`display_width`), never `str::len`.

use crate::th;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::{button_bg, display_width, fit, text_field};

mod discard;
mod dismiss;
pub(crate) mod frame;
pub(crate) use discard::{DiscardGuard, Verdict};
#[allow(unused_imports)]
pub(crate) use dismiss::{
    dismiss, panel_closable, CloseHit, Confirm, ConfirmEvent, Dismiss, DismissAction, DismissKind,
    Guard,
};
pub(crate) use frame::Win;

/// Paints `rect` with `style` (spaces).
pub(crate) fn fill(buf: &mut Buffer, rect: Rect, style: Style) {
    let rect = rect.intersection(buf.area);
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            let cell = &mut buf[(x, y)];
            cell.set_symbol(" ");
            cell.set_style(style);
        }
    }
}

/// A row of a list: painted as a whole band and registered as the arrow, so
/// a row is the same everywhere (the arrow, lit as a whole on hover) and the
/// controls painted inside it afterwards (a checkbox, a button) keep their hand.
/// `bg` is the row's selection or hover fill.
pub(crate) fn row_band(buf: &mut Buffer, rect: Rect, bg: ratatui::style::Color) {
    fill(buf, rect, Style::default().bg(bg));
    crate::pointer_map::plain(rect);
}

/// The fill of a list row: selected, hovered or at rest.
pub(crate) fn row_bg(selected: bool, hovered: bool) -> ratatui::style::Color {
    if selected {
        th().ui.selection
    } else if hovered {
        th().ui.hover
    } else {
        th().ui.panel
    }
}

/// Writes `text` at `(x, y)`, cut to `max` columns; returns the rect it
/// covered.
pub(crate) fn put(buf: &mut Buffer, x: u16, y: u16, text: &str, max: u16, style: Style) -> Rect {
    if max == 0 || !buf.area.contains((x, y).into()) {
        return Rect::new(x, y, 0, 0);
    }
    let room = max.min(buf.area.right().saturating_sub(x));
    let text = fit(text, room as usize);
    let (end, _) = buf.set_stringn(x, y, &text, room as usize, style);
    Rect::new(x, y, end.saturating_sub(x), 1)
}

/// A rounded box with a title, filled with the panel background.
pub(crate) fn panel(buf: &mut Buffer, rect: Rect, title: &str) {
    use crate::wide_edge::Clear;
    use ratatui::widgets::{Block, BorderType, Borders, Widget};
    let rect = rect.intersection(buf.area);
    Clear.render(rect, buf);
    Block::default()
        .title(format!(" {title} "))
        .title_style(
            Style::default()
                .fg(th().ui.text)
                .add_modifier(Modifier::BOLD),
        )
        .border_type(BorderType::Rounded)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(th().ui.accent))
        .style(Style::default().bg(th().ui.panel))
        .render(rect, buf);
}

/// A button: ` label `, filled; returns its rect. A disabled button is
/// drawn flat and dim and does not react to hover.
pub(crate) fn button(buf: &mut Buffer, x: u16, y: u16, label: &str, state: ButtonState) -> Rect {
    let style = button_style(state);
    let text = format!(" {label} ");
    let rect = put(buf, x, y, &text, display_width(&text) as u16, style);
    // A disabled button still answers a click (it says why it is inert).
    crate::pointer_map::hand(rect);
    rect
}

/// The look of a [`button`], for callers that build spans instead of
/// painting into a buffer (Settings' inline buttons).
pub(crate) fn button_style(state: ButtonState) -> Style {
    match state {
        ButtonState::Disabled => Style::default()
            .fg(th().ui.disabled)
            .bg(th().chrome.disabled_button_bg),
        ButtonState::Primary { hovered, focused } => Style::default()
            .fg(th().ui.on_accent)
            .bg(if hovered || focused {
                crate::hover_fill(th().ui.accent, th().ui.on_accent, 30)
            } else {
                th().ui.accent
            })
            .add_modifier(Modifier::BOLD),
        ButtonState::Normal { hovered, focused } => Style::default()
            .fg(th().ui.on_button)
            .bg(button_bg(hovered || focused))
            .add_modifier(if focused {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonState {
    Normal { hovered: bool, focused: bool },
    Primary { hovered: bool, focused: bool },
    Disabled,
}

/// The width a button with `label` takes.
pub(crate) fn button_width(label: &str) -> u16 {
    display_width(label) as u16 + 2
}

/// A single-line input: a tinted band with the text scrolled so the
/// caret stays visible, and a block caret when focused.
/// An empty unfocused field shows `placeholder` dimmed.
///
/// `edit` is the dialog's shared editor and this field's id in it
/// the field's text rect is recorded there for the
/// mouse, and its caret and selection are drawn.
pub(crate) fn text_input<K: Copy + PartialEq>(
    buf: &mut Buffer,
    rect: Rect,
    value: &str,
    placeholder: &str,
    focused: bool,
    hovered: bool,
    edit: (&crate::text_edit::FieldEditor<K>, K),
) {
    let (editor, id) = edit;
    let text_rect = Rect::new(rect.x + 1, rect.y, rect.width.saturating_sub(1), 1);
    crate::pointer_map::field(rect);
    editor.drawn(id, text_rect, false);
    let bg = crate::input_bg(focused, hovered);
    fill(buf, rect, Style::default().bg(bg));
    let width = rect.width as usize;
    if value.is_empty() && !focused {
        put(
            buf,
            rect.x + 1,
            rect.y,
            placeholder,
            rect.width.saturating_sub(1),
            Style::default().fg(th().ui.muted).bg(bg),
        );
        return;
    }
    let style = Style::default().fg(th().ui.text).bg(bg);
    let line = text_field::edit_line(
        value,
        editor.caret(id),
        width.saturating_sub(1),
        focused,
        text_field::EditLook::new(style, "\u{2588}"),
    );
    if buf.area.contains((text_rect.x, text_rect.y).into()) {
        let room = text_rect
            .width
            .min(buf.area.right().saturating_sub(text_rect.x));
        buf.set_line(text_rect.x, text_rect.y, &line, room);
    }
}

/// A multi-line input: like [`text_input`], `rect.height` lines of the
/// text, scrolled to keep the caret in view. While focused and longer
/// than its box it draws the standard scrollbar over its rightmost
/// column and returns that bar's track, for the maintainer's mouse handling.
pub(crate) fn text_area<K: Copy + PartialEq>(
    buf: &mut Buffer,
    rect: Rect,
    value: &str,
    placeholder: &str,
    focused: bool,
    hovered: bool,
    edit: (&crate::text_edit::FieldEditor<K>, K),
) -> Option<crate::scrollbar::Track> {
    let (editor, id) = edit;
    let top = editor.caret(id).map_or(0, |c| c.top.get());
    let has_bar = focused && text_field::field_track(rect, value, top).is_some();
    let text_rect = Rect::new(
        rect.x + 1,
        rect.y,
        rect.width.saturating_sub(1 + u16::from(has_bar)),
        rect.height,
    );
    crate::pointer_map::field(rect);
    editor.drawn(id, text_rect, true);
    let bg = crate::input_bg(focused, hovered);
    fill(buf, rect, Style::default().bg(bg));
    if value.is_empty() && !focused {
        put(
            buf,
            rect.x + 1,
            rect.y,
            placeholder,
            rect.width.saturating_sub(1),
            Style::default().fg(th().ui.muted).bg(bg),
        );
        return None;
    }
    let style = Style::default().fg(th().ui.text).bg(bg);
    let lines = text_field::edit_lines(
        value,
        editor.caret(id),
        text_rect.width as usize,
        text_rect.height as usize,
        focused,
        text_field::EditLook::new(style, "\u{2588}"),
    );
    for (i, line) in lines.iter().enumerate() {
        let y = text_rect.y + i as u16;
        if buf.area.contains((text_rect.x, y).into()) {
            let room = text_rect
                .width
                .min(buf.area.right().saturating_sub(text_rect.x));
            buf.set_line(text_rect.x, y, line, room);
        }
    }
    // The scroll `edit_lines` settled on is what the thumb shows.
    let top = editor.caret(id).map_or(0, |c| c.top.get());
    let track = text_field::field_track(rect, value, top).filter(|_| focused)?;
    crate::scrollbar::render_buf(buf, track, true, false);
    Some(track)
}

/// A row of `key what` hints, left to right, stopping at `max`.
/// The effective key of `action` for a footer hint (`-` when unbound).
pub(crate) fn key_label(action: crate::keymap::Action) -> String {
    match crate::keymap::current().shown(action) {
        k if k.is_empty() => "-".to_string(),
        k => k,
    }
}

/// A fixed key's name, spelled by the keymap's own chord printer (key names
/// are not translated and never live in a message).
pub(crate) fn key_name(key: crate::keymap::chord::Key) -> String {
    use crate::keymap::chord::{KeyChord, Mods};
    KeyChord::new(Mods::NONE, key).to_string()
}

/// `Ctrl+<c>`, spelled by the keymap's chord printer.
pub(crate) fn ctrl_key(c: char) -> String {
    use crate::keymap::chord::{Key, KeyChord, Mods};
    KeyChord::new(Mods::CTRL, Key::Char(c)).to_string()
}

/// `Alt+<key>`, spelled by the keymap's chord printer.
pub(crate) fn alt_key(key: crate::keymap::chord::Key) -> String {
    use crate::keymap::chord::{KeyChord, Mods};
    let alt = Mods {
        alt: true,
        ..Mods::NONE
    };
    KeyChord::new(alt, key).to_string()
}

/// `Alt+` followed by the two arrows: the history keys of the folder picker.
pub(crate) fn alt_arrows() -> String {
    use crate::keymap::chord::{Key, KeyChord, Mods};
    let alt = Mods {
        alt: true,
        ..Mods::NONE
    };
    let left = KeyChord::new(alt, Key::Left).to_string();
    let prefix = left.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    format!("{prefix}\u{2190}/\u{2192}")
}

pub(crate) fn hints(buf: &mut Buffer, x: u16, y: u16, max: u16, pairs: &[(&str, &str)]) {
    let mut cx = x;
    let end = x + max;
    for (index, (key, what)) in pairs.iter().enumerate() {
        let sep = if index == 0 { "" } else { " \u{00b7} " };
        let piece = format!("{sep}{key} ");
        let need = (display_width(&piece) + display_width(what)) as u16;
        if cx + need > end {
            break;
        }
        cx += put(
            buf,
            cx,
            y,
            &piece,
            need,
            Style::default().fg(th().ui.accent).bg(th().ui.panel),
        )
        .width;
        cx += put(
            buf,
            cx,
            y,
            what,
            need,
            Style::default().fg(th().ui.muted).bg(th().ui.panel),
        )
        .width;
    }
}

/// The app's checkbox mark, `[x]` / `[ ]`: what every checklist draws (the
/// Settings and dialog checkboxes, the column choosers). One place, so no
/// list invents a glyph a font may draw as an unrelated icon.
pub(crate) fn checkbox_mark(checked: bool) -> &'static str {
    if checked {
        "[x]"
    } else {
        "[ ]"
    }
}

/// A checkbox `[x] label` / `[ ] label`; returns its rect.
pub(crate) fn checkbox(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    max: u16,
    label: &str,
    checked: bool,
    state: CheckState,
) -> Rect {
    let text = format!("{} {label}", checkbox_mark(checked));
    let style = match state {
        CheckState::Disabled => Style::default().fg(th().ui.disabled).bg(th().ui.panel),
        CheckState::Enabled { hovered, focused } => {
            Style::default().fg(th().ui.text).bg(if focused {
                th().ui.selection
            } else if hovered {
                th().ui.hover
            } else {
                th().ui.panel
            })
        }
    };
    let rect = put(buf, x, y, &text, max, style);
    crate::pointer_map::hand(rect);
    rect
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CheckState {
    Enabled { hovered: bool, focused: bool },
    Disabled,
}

/// The first registered rect containing `(col, row)`. Hit lists are
/// registered **inner-first** (a band covers the button inside it).
pub(crate) fn hit_at<H: Copy>(hits: &[(Rect, H)], col: u16, row: u16) -> Option<H> {
    hits.iter()
        .find(|(r, _)| r.contains((col, row).into()))
        .map(|(_, h)| *h)
}

/// `Ctrl+N` badges for a dialog: from the rects of its controls (the ones its
/// own paint walk registered for the mouse), in screen order, each with the
/// cell a press clicks. A badge starts on the cell before its control, so it
/// hides fewer letters of a label; a press is a left click there, so the
/// badge and the mouse can never disagree.
pub(crate) fn nav_cells(rects: impl IntoIterator<Item = Rect>) -> Vec<(Rect, (u16, u16))> {
    let mut cells: Vec<Rect> = rects
        .into_iter()
        .filter(|r| r.width > 0 && r.height > 0)
        .collect();
    cells.sort_by_key(|r| (r.y, r.x));
    cells.dedup_by_key(|r| (r.x, r.y));
    cells
        .into_iter()
        .map(|r| {
            (
                Rect::new(r.x.saturating_sub(1), r.y, r.width + 1, 1),
                (r.x, r.y),
            )
        })
        .collect()
}

/// Like [`nav_cells`] for a screen whose controls overlap (a row with a check
/// box inside it): `hits` is the click order, `keep` picks the controls that
/// get a badge, and each one's cell is the first on its top row where the
/// click really reaches it, so a badge never presses its neighbour.
pub(crate) fn nav_cells_in<H: Copy + PartialEq>(
    hits: &[(Rect, H)],
    keep: impl Fn(H) -> bool,
) -> Vec<(Rect, (u16, u16))> {
    let reaches = |x: u16, y: u16, hit: H| {
        hits.iter()
            .find(|(r, _)| r.contains((x, y).into()))
            .is_some_and(|(_, h)| *h == hit)
    };
    let mut cells: Vec<(Rect, (u16, u16))> = hits
        .iter()
        .filter(|(r, h)| r.width > 0 && r.height > 0 && keep(*h))
        .filter_map(|(r, h)| {
            (r.x..r.right()).find(|x| reaches(*x, r.y, *h)).map(|x| {
                (
                    Rect::new(x.saturating_sub(1), r.y, 1 + r.width - (x - r.x), 1),
                    (x, r.y),
                )
            })
        })
        .collect();
    cells.sort_by_key(|(_, (x, y))| (*y, *x));
    cells.dedup_by_key(|(_, at)| *at);
    cells
}

/// The dialog's rect: centered, `want` wide and tall at most, never
/// bigger than the frame.
pub(crate) fn centered(frame: Rect, want_w: u16, want_h: u16) -> Rect {
    let w = want_w.min(frame.width);
    let h = want_h.min(frame.height);
    Rect::new(
        frame.x + (frame.width - w) / 2,
        frame.y + (frame.height - h) / 2,
        w,
        h,
    )
}

/// Everything painted in a buffer, one string per row (tests).
#[cfg(test)]
pub(crate) fn buffer_text(buf: &Buffer) -> String {
    let area = buf.area;
    let mut out = String::new();
    for y in area.y..area.y + area.height {
        let mut skip = 0;
        for x in area.x..area.x + area.width {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let symbol = buf[(x, y)].symbol();
            skip = display_width(symbol).saturating_sub(1);
            out.push_str(symbol);
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Theme;
    use ratatui::style::Color;

    #[test]
    fn put_returns_the_rect_it_painted_and_stays_in_the_buffer() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 2));
        let rect = put(&mut buf, 6, 0, "abcdefgh", 20, Style::default());
        assert_eq!(rect, Rect::new(6, 0, 4, 1));
        assert!(buffer_text(&buf).starts_with("      abc\u{2026}"));
        // Outside the buffer: nothing painted, no panic.
        assert_eq!(put(&mut buf, 3, 9, "x", 5, Style::default()).width, 0);
    }

    fn lum((r, g, b): (u8, u8, u8)) -> f64 {
        let ch = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * ch(r) + 0.7152 * ch(g) + 0.0722 * ch(b)
    }

    /// Label-on-fill contrast of the painted button cell.
    fn painted_ratio(state: ButtonState) -> Option<f64> {
        let mut buf = Buffer::empty(Rect::new(0, 0, 12, 1));
        button(&mut buf, 0, 0, "Discard", state);
        let cell = &buf[(2, 0)];
        let (Color::Rgb(fr, fg, fb), Color::Rgb(br, bg, bb)) = (cell.fg, cell.bg) else {
            return None;
        };
        let (a, b) = (lum((fr, fg, fb)), lum((br, bg, bb)));
        Some((a.max(b) + 0.05) / (a.min(b) + 0.05))
    }

    /// Maintainer report 2026-10-08: on light themes the inactive buttons of the
    /// merge editor bar and the Unsaved-changes dialog painted dark text on
    /// the dark `ui.button` fill. Every preset x every button state must
    /// read: enabled 4.5, disabled (muted) 3.
    #[test]
    fn every_button_state_reads_in_every_preset() {
        let mut failures = Vec::new();
        for spec in gitpoulpi_config::theme::presets() {
            let theme = Theme::from(&spec);
            let _guard = crate::theme::override_theme(&theme);
            let mut states = Vec::new();
            for hovered in [false, true] {
                for focused in [false, true] {
                    states.push((
                        format!("normal hovered={hovered} focused={focused}"),
                        ButtonState::Normal { hovered, focused },
                        4.5,
                    ));
                    states.push((
                        format!("primary hovered={hovered} focused={focused}"),
                        ButtonState::Primary { hovered, focused },
                        4.5,
                    ));
                }
            }
            states.push(("disabled".into(), ButtonState::Disabled, 3.0));
            for (name, state, min) in states {
                if let Some(r) = painted_ratio(state) {
                    if r < min {
                        failures.push(format!("{}: {name} {r:.2} < {min}", spec.slug));
                    }
                }
            }
        }
        assert!(failures.is_empty(), "{failures:#?}");
    }
}
