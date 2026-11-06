//! The version chip: the running version in the main window's title bar,
//! between the app name and the repository path. It turns into `v0.2.0 available` in the
//! accent colour when a newer release is known; a click opens the update panel
//! then (its Release notes button leads to the tab), else the Release notes tab
//! (as does the palette's `Help: Release notes`).
//!
//! The rect comes out of the paint (`paint`), like the Activity button's: one
//! walk, so a click never lands beside the glyphs.

use std::cell::Cell;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::{dialog_kit, display_width, th};
use gitpoulpi_i18n::msg::updates as msg;

thread_local! {
    /// Where the chip was last painted; empty when the frame had no room.
    static RECT: Cell<Rect> = const { Cell::new(Rect::new(0, 0, 0, 0)) };
    static POINTER: Cell<Option<(u16, u16)>> = const { Cell::new(None) };
}

/// What the chip says now.
pub(crate) fn text() -> String {
    let snapshot = super::snapshot();
    if matches!(snapshot.install.phase, super::auto::Phase::Ready(_)) {
        return msg::CHIP_RESTART.get().to_string();
    }
    match snapshot.available {
        Some(release) => msg::chip_available(&release.version),
        None => super::running_label(),
    }
}

/// Whether the chip announces a newer version.
pub(crate) fn is_available() -> bool {
    let snapshot = super::snapshot();
    snapshot.available.is_some() || matches!(snapshot.install.phase, super::auto::Phase::Ready(_))
}

/// Whether a click opens the update panel (a release is announced, or an
/// update is under way or failed); otherwise it opens the Release notes tab.
pub(crate) fn opens_panel() -> bool {
    let snapshot = super::snapshot();
    snapshot.available.is_some() || !matches!(snapshot.install.phase, super::auto::Phase::Idle)
}

/// Cells the chip takes (a space either side).
pub(crate) fn width() -> u16 {
    display_width(&text()) as u16 + 2
}

/// The chip's text without a development build's hash.
fn short_text() -> String {
    let snapshot = super::snapshot();
    if matches!(snapshot.install.phase, super::auto::Phase::Ready(_)) {
        return msg::CHIP_RESTART.get().to_string();
    }
    match snapshot.available {
        Some(release) => msg::chip_available(&release.version),
        None => format!("v{}", super::RUNNING),
    }
}

fn short_width() -> u16 {
    display_width(&short_text()) as u16 + 2
}

/// The width the chip takes in the title bar when `room` cells are free for
/// it: the full label (with a development build's hash), else the bare
/// version, else 0 (no chip drawn, no rect registered).
pub(crate) fn fit(room: u16) -> u16 {
    let (full, short) = (width(), short_width());
    if room >= full {
        full
    } else if room >= short {
        short
    } else {
        0
    }
}

/// Paints the chip at `(x, y)` in the title bar (`reserved` cells, from
/// [`fit`]) and records its rect.
pub(crate) fn paint(buf: &mut Buffer, x: u16, y: u16, reserved: u16) {
    if reserved == 0 {
        RECT.set(Rect::default());
        return;
    }
    let rect = Rect::new(x, y, reserved, 1);
    let hovered = POINTER
        .get()
        .is_some_and(|(x, y)| rect.contains((x, y).into()));
    let base = if is_available() {
        Style::default()
            .fg(th().ui.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(th().ui.muted)
    };
    let style = if hovered {
        base.bg(th().ui.button_hover)
    } else {
        base
    };
    dialog_kit::put(
        buf,
        rect.x,
        rect.y,
        &format!(
            " {} ",
            if reserved == width() {
                text()
            } else {
                short_text()
            }
        ),
        reserved,
        style,
    );
    RECT.set(rect);
}

/// Forgets the chip (nothing painted it this frame).
pub(crate) fn clear() {
    RECT.set(Rect::default());
}

/// Whether `(col, row)` is on the chip as last painted.
pub(crate) fn hit(col: u16, row: u16) -> bool {
    RECT.get().contains((col, row).into())
}

/// The mouse, before the panes: `true` when this press is on the chip (see
/// [`opens_panel`] for where it leads). The pointer is noted for the hover either way.
pub(crate) fn route_mouse(mouse: crossterm::event::MouseEvent) -> bool {
    use crossterm::event::{MouseButton, MouseEventKind};
    POINTER.set(Some((mouse.column, mouse.row)));
    mouse.kind == MouseEventKind::Down(MouseButton::Left) && hit(mouse.column, mouse.row)
}
