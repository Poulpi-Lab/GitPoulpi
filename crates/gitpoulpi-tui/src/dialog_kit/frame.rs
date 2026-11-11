//! A dialog's movable, resizable frame.
//!
//! [`FloatingFrame`] owns where a dialog sits and how big it is. The dialog
//! asks it for its rect every frame ([`FloatingFrame::layout`]), paints its
//! own content inside, then lets the frame add its marks
//! ([`FloatingFrame::paint`]: the corner grips, the arrange banner) and
//! register its pointer shapes. Mouse and keys go through the frame first
//! ([`FloatingFrame::on_mouse`], [`FloatingFrame::on_key`]); what it does not
//! consume is the dialog's.
//!
//! * **Mouse**: drag the title bar to move (the close cross keeps priority),
//!   drag the left / right / bottom borders or a bottom corner to resize. A
//!   drag continues outside the dialog (the terminal reports the pointer
//!   wherever it is) and its release is consumed, so letting go outside never
//!   closes anything.
//! * **Keyboard twin**: `Ctrl+N m` (`keymap::Action::NavArrangeDialog`) starts
//!   the *arrange* mode: arrows move, Shift+arrows resize, `0` resets, Enter or
//!   Esc leave it. The palette has the same entries.
//! * **Where it is kept** (`ui.dialog_geometry`): `session` keeps the geometry
//!   in a process-wide map keyed by the dialog's id, `saved` also writes it to
//!   the data folder and restores it at the next launch, `never` forgets it
//!   with the dialog.
//!
//! The geometry is clamped to the screen on every layout (a terminal resized
//! under an open dialog never strands it off-screen) and never below
//! [`MIN_W`] x [`MIN_H`] (or the screen, if that is smaller).

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_config::DialogGeometry;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::pointer_map::{self, Kind};
use crate::th;

/// The smallest a dialog may be resized to.
pub(crate) const MIN_W: u16 = 60;
pub(crate) const MIN_H: u16 = 16;

/// Columns / rows one arrow key moves or resizes by in arrange mode.
const KEY_STEP_X: i32 = 2;
const KEY_STEP_Y: i32 = 1;

/// A dialog's place and size, in screen cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Geo {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) w: u16,
    pub(crate) h: u16,
}

impl Geo {
    fn of(rect: Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            w: rect.width,
            h: rect.height,
        }
    }

    fn rect(self) -> Rect {
        Rect::new(self.x, self.y, self.w, self.h)
    }

    /// Inside `screen`, at least the minimum size (or the whole screen when
    /// that is smaller).
    fn clamped(self, screen: Rect) -> Self {
        let w = self.w.max(MIN_W.min(screen.width)).min(screen.width);
        let h = self.h.max(MIN_H.min(screen.height)).min(screen.height);
        let x = self
            .x
            .clamp(screen.x, screen.right().saturating_sub(w).max(screen.x));
        let y = self
            .y
            .clamp(screen.y, screen.bottom().saturating_sub(h).max(screen.y));
        Self { x, y, w, h }
    }
}

/// The part of the frame a press can take hold of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Grip {
    /// The title bar: move.
    Title,
    Left,
    Right,
    Bottom,
    BottomLeft,
    BottomRight,
}

impl Grip {
    /// The pointer a grip shows.
    fn kind(self) -> Kind {
        match self {
            Grip::Title => Kind::Move,
            Grip::Left | Grip::Right => Kind::ColResize,
            Grip::Bottom => Kind::RowResize,
            Grip::BottomRight => Kind::NwseResize,
            Grip::BottomLeft => Kind::NeswResize,
        }
    }
}

/// What the frame made of a mouse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FrameMouse {
    /// Not the frame's: the dialog handles it.
    Ignored,
    /// The frame took it (a press on a grip, a drag, a release of one).
    Consumed,
}

#[derive(Debug, Clone, Copy)]
struct Drag {
    grip: Grip,
    /// Where the pointer held the frame when the press landed (title drag).
    held: (u16, u16),
    start: Geo,
}

// -- the geometry kept per dialog id ---------------------------------------

#[derive(Default)]
struct Store {
    geos: HashMap<String, Geo>,
    /// The saved file has been read (once per process).
    loaded: bool,
}

#[cfg(not(test))]
static STORE: std::sync::Mutex<Option<Store>> = std::sync::Mutex::new(None);

// A unit test gets a store of its own (tests run one per thread), and never
// touches the real data folder.
#[cfg(test)]
thread_local! {
    static STORE: std::cell::RefCell<Option<Store>> = const { std::cell::RefCell::new(None) };
}

fn with_store<R>(f: impl FnOnce(&mut Store) -> R) -> R {
    #[cfg(not(test))]
    {
        let mut guard = STORE.lock().unwrap_or_else(|e| e.into_inner());
        f(guard.get_or_insert_with(Store::default))
    }
    #[cfg(test)]
    {
        STORE.with(|s| f(s.borrow_mut().get_or_insert_with(Store::default)))
    }
}

/// The file `saved` keeps geometry in (`None` in tests and without a config
/// location).
fn saved_path() -> Option<std::path::PathBuf> {
    if cfg!(test) {
        return None;
    }
    gitpoulpi_config::ConfigPaths::discover().map(|p| p.data_dir().join("dialog_geometry.txt"))
}

/// `id x y w h` per line.
fn parse_saved(text: &str) -> HashMap<String, Geo> {
    let mut out = HashMap::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(id), Some(x), Some(y), Some(w), Some(h)) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) else {
            continue;
        };
        if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (x.parse(), y.parse(), w.parse(), h.parse()) {
            out.insert(id.to_string(), Geo { x, y, w, h });
        }
    }
    out
}

fn render_saved(geos: &HashMap<String, Geo>) -> String {
    let mut ids: Vec<&String> = geos.keys().collect();
    ids.sort();
    ids.into_iter()
        .map(|id| {
            let g = geos[id];
            format!("{id} {} {} {} {}\n", g.x, g.y, g.w, g.h)
        })
        .collect()
}

fn load_saved(store: &mut Store) {
    if store.loaded {
        return;
    }
    store.loaded = true;
    let Some(path) = saved_path() else {
        return;
    };
    if let Ok(text) = std::fs::read_to_string(path) {
        for (id, geo) in parse_saved(&text) {
            store.geos.entry(id).or_insert(geo);
        }
    }
}

fn persist(store: &Store) {
    let Some(path) = saved_path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = gitpoulpi_config::secure_fs::create_private_dir_all(dir);
    }
    let _ = gitpoulpi_config::secure_fs::write_private(&path, render_saved(&store.geos).as_bytes());
}

// -- the frame -------------------------------------------------------------

/// One dialog's place and size. See the module doc.
pub(crate) struct FloatingFrame {
    id: &'static str,
    /// The largest default size (the dialog's own choice), shrunk to the
    /// screen.
    default_max: (u16, u16),
    mode: DialogGeometry,
    /// The geometry the user chose (`None`: the default, centered).
    chosen: Option<Geo>,
    /// The rect of the last layout.
    rect: Rect,
    screen: Rect,
    drag: Option<Drag>,
    arrange: bool,
}

impl FloatingFrame {
    /// A frame for the dialog `id`, opening on what was kept for it (per
    /// `ui.dialog_geometry`) or on a centered rect up to `default_max`.
    pub(crate) fn new(id: &'static str, default_max: (u16, u16)) -> Self {
        let mode = crate::live_settings::dialog_geometry();
        let chosen = if mode == DialogGeometry::Never {
            None
        } else {
            with_store(|s| {
                if mode == DialogGeometry::Saved {
                    load_saved(s);
                }
                s.geos.get(id).copied()
            })
        };
        Self {
            id,
            default_max,
            mode,
            chosen,
            rect: Rect::default(),
            screen: Rect::default(),
            drag: None,
            arrange: false,
        }
    }

    /// The rect for this frame inside `screen`: the geometry the user chose,
    /// or the default; always on screen and never under the minimum.
    pub(crate) fn layout(&mut self, screen: Rect) -> Rect {
        self.screen = screen;
        let geo = match self.chosen {
            Some(geo) => {
                let clamped = geo.clamped(screen);
                if clamped != geo {
                    // Kept as clamped: a later, larger terminal does not pull
                    // it back out. Not written to disk, only a drag does that.
                    self.chosen = Some(clamped);
                }
                clamped
            }
            None => Geo::of(super::centered(
                screen,
                screen.width.saturating_sub(4).min(self.default_max.0),
                screen.height.saturating_sub(2).min(self.default_max.1),
            ))
            .clamped(screen),
        };
        self.rect = geo.rect();
        self.rect
    }

    /// The rect of the last layout.
    #[cfg(test)]
    pub(crate) fn rect(&self) -> Rect {
        self.rect
    }

    /// Whether the user has moved or resized it (it is not at its default).
    #[cfg(test)]
    pub(crate) fn is_arranged(&self) -> bool {
        self.chosen.is_some()
    }

    /// A drag of the frame is in progress.
    pub(crate) fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// The arrange mode (keyboard twin) is on.
    #[cfg(test)]
    pub(crate) fn arranging(&self) -> bool {
        self.arrange
    }

    /// Starts the arrange mode.
    pub(crate) fn arrange_start(&mut self) {
        self.arrange = true;
    }

    /// Puts the frame back at its default place and size, and forgets what
    /// was kept for it.
    pub(crate) fn reset(&mut self) {
        self.chosen = None;
        self.drag = None;
        if self.mode != DialogGeometry::Never {
            with_store(|s| {
                s.geos.remove(self.id);
                if self.mode == DialogGeometry::Saved {
                    persist(s);
                }
            });
        }
    }

    /// The part of the frame at `(col, row)`; `close` is true over the close
    /// cross, which keeps its click.
    pub(crate) fn grip_at(&self, col: u16, row: u16, close: bool) -> Option<Grip> {
        let r = self.rect;
        if r.width < 4 || r.height < 4 || !r.contains((col, row).into()) {
            return None;
        }
        let (left, right, bottom) = (r.x, r.right() - 1, r.bottom() - 1);
        if row == r.y {
            // The corners of the top row are not grips (nothing to resize
            // there), the cross is the dialog's.
            return (!close && col > left && col < right).then_some(Grip::Title);
        }
        if row == bottom {
            return Some(match col {
                c if c == left => Grip::BottomLeft,
                c if c == right => Grip::BottomRight,
                _ => Grip::Bottom,
            });
        }
        match col {
            c if c == left => Some(Grip::Left),
            c if c == right => Some(Grip::Right),
            _ => None,
        }
    }

    /// Feeds one mouse event. `close` is whether the press is on the close
    /// cross, `enabled` is false while a popup of the dialog is up (a new
    /// drag does not start under it; one already started finishes).
    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent, close: bool, enabled: bool) -> FrameMouse {
        let (col, row) = (mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) if enabled => {
                let Some(grip) = self.grip_at(col, row, close) else {
                    return FrameMouse::Ignored;
                };
                self.arrange = false;
                self.drag = Some(Drag {
                    grip,
                    held: (
                        col.saturating_sub(self.rect.x),
                        row.saturating_sub(self.rect.y),
                    ),
                    start: Geo::of(self.rect),
                });
                FrameMouse::Consumed
            }
            MouseEventKind::Drag(MouseButton::Left) if self.drag.is_some() => {
                self.drag_to(col, row);
                FrameMouse::Consumed
            }
            MouseEventKind::Up(MouseButton::Left) if self.drag.is_some() => {
                self.drag_to(col, row);
                self.drag = None;
                self.keep();
                FrameMouse::Consumed
            }
            _ => FrameMouse::Ignored,
        }
    }

    fn drag_to(&mut self, col: u16, row: u16) {
        let Some(drag) = self.drag else {
            return;
        };
        let s = drag.start;
        let (col, row) = (i32::from(col), i32::from(row));
        let (right, bottom) = (
            i32::from(s.x) + i32::from(s.w),
            i32::from(s.y) + i32::from(s.h),
        );
        let mut x = i32::from(s.x);
        let mut y = i32::from(s.y);
        let mut w = i32::from(s.w);
        let mut h = i32::from(s.h);
        match drag.grip {
            Grip::Title => {
                x = col - i32::from(drag.held.0);
                y = row - i32::from(drag.held.1);
            }
            Grip::Right => w = col - x + 1,
            Grip::Left => {
                x = col;
                w = right - col;
            }
            Grip::Bottom => h = row - y + 1,
            Grip::BottomRight => {
                w = col - x + 1;
                h = row - y + 1;
            }
            Grip::BottomLeft => {
                x = col;
                w = right - col;
                h = row - y + 1;
            }
        }
        let min_w = i32::from(MIN_W.min(self.screen.width));
        let min_h = i32::from(MIN_H.min(self.screen.height));
        if matches!(drag.grip, Grip::Left | Grip::BottomLeft) {
            // The right edge stays put: the left one stops at the minimum.
            x = x.min(right - min_w);
            w = right - x;
        }
        w = w.max(min_w);
        h = h.max(min_h);
        let _ = bottom;
        self.set(x, y, w, h);
    }

    /// Sets the geometry (clamped to the screen) and keeps it per the mode.
    fn set(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let geo = Geo {
            x: x.clamp(0, i32::from(u16::MAX)) as u16,
            y: y.clamp(0, i32::from(u16::MAX)) as u16,
            w: w.clamp(1, i32::from(u16::MAX)) as u16,
            h: h.clamp(1, i32::from(u16::MAX)) as u16,
        }
        .clamped(self.screen);
        self.chosen = Some(geo);
        self.rect = geo.rect();
        if self.mode != DialogGeometry::Never {
            with_store(|s| {
                s.geos.insert(self.id.to_string(), geo);
            });
        }
    }

    /// Writes the geometry to disk (`saved`), when a gesture is over.
    fn keep(&self) {
        if self.mode == DialogGeometry::Saved && self.chosen.is_some() {
            with_store(|s| persist(s));
        }
    }

    /// Moves the frame by `(dx, dy)` cells.
    pub(crate) fn nudge(&mut self, dx: i32, dy: i32) {
        let g = Geo::of(self.rect);
        self.set(
            i32::from(g.x) + dx,
            i32::from(g.y) + dy,
            i32::from(g.w),
            i32::from(g.h),
        );
    }

    /// Resizes the frame by `(dw, dh)` cells (the top left stays).
    pub(crate) fn grow(&mut self, dw: i32, dh: i32) {
        let g = Geo::of(self.rect);
        self.set(
            i32::from(g.x),
            i32::from(g.y),
            (i32::from(g.w) + dw).max(i32::from(MIN_W)),
            (i32::from(g.h) + dh).max(i32::from(MIN_H)),
        );
    }

    /// A key while the arrange mode is on. `true`: the frame took it.
    pub(crate) fn on_key(&mut self, key: KeyEvent) -> bool {
        if !self.arrange {
            return false;
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let (dx, dy) = match key.code {
            KeyCode::Left => (-KEY_STEP_X, 0),
            KeyCode::Right => (KEY_STEP_X, 0),
            KeyCode::Up => (0, -KEY_STEP_Y),
            KeyCode::Down => (0, KEY_STEP_Y),
            KeyCode::Char('0') => {
                self.reset();
                return true;
            }
            KeyCode::Enter | KeyCode::Esc => {
                self.arrange = false;
                self.keep();
                return true;
            }
            _ => return true,
        };
        if shift {
            self.grow(dx, dy);
        } else {
            self.nudge(dx, dy);
        }
        true
    }

    /// Registers the frame's grips with the pointer map (after the dialog's
    /// own registrations, so a grip wins over a body painted under it). While
    /// a drag is on, the whole screen answers with the drag's shape.
    pub(crate) fn register_pointer(&self, close: Option<Rect>) {
        let r = self.rect;
        if r.width < 4 || r.height < 4 {
            return;
        }
        let (left, right, bottom) = (r.x, r.right() - 1, r.bottom() - 1);
        let mut title = Rect::new(left + 1, r.y, r.width - 2, 1);
        if let Some(c) = close {
            // Cut the cross out of the title strip (it sits at its right end).
            if c.x > title.x && c.x < title.right() {
                title.width = c.x - title.x;
            }
        }
        pointer_map::register(title, Kind::Move);
        pointer_map::register(
            Rect::new(left, r.y + 1, 1, r.height.saturating_sub(2)),
            Kind::ColResize,
        );
        pointer_map::register(
            Rect::new(right, r.y + 1, 1, r.height.saturating_sub(2)),
            Kind::ColResize,
        );
        pointer_map::register(Rect::new(left + 1, bottom, r.width - 2, 1), Kind::RowResize);
        pointer_map::register(Rect::new(left, bottom, 1, 1), Kind::NeswResize);
        pointer_map::register(Rect::new(right, bottom, 1, 1), Kind::NwseResize);
        if let Some(drag) = self.drag {
            pointer_map::register(self.screen, drag.grip.kind());
        }
    }

    /// The frame's marks on the dialog's border: the corner grips, and in
    /// the arrange mode the banner (`hint`) on the bottom border with the
    /// border in the warning color.
    pub(crate) fn paint(&self, buf: &mut Buffer, hint: &str) {
        let r = self.rect.intersection(buf.area);
        if r.width < 4 || r.height < 4 {
            return;
        }
        let (left, right, bottom) = (r.x, r.right() - 1, r.bottom() - 1);
        let border = Style::default().fg(th().ui.accent).bg(th().ui.panel);
        let grip = border.add_modifier(Modifier::BOLD);
        if self.arrange {
            let warn = Style::default().fg(th().status.warning).bg(th().ui.panel);
            for x in left..=right {
                buf[(x, r.y)].set_style(warn);
                buf[(x, bottom)].set_style(warn);
            }
            for y in r.y..=bottom {
                buf[(left, y)].set_style(warn);
                buf[(right, y)].set_style(warn);
            }
            super::put(
                buf,
                left + 2,
                bottom,
                &format!(" {hint} "),
                r.width.saturating_sub(6),
                warn.add_modifier(Modifier::BOLD),
            );
        }
        buf[(right, bottom)].set_symbol("\u{25e2}").set_style(grip);
        buf[(left, bottom)].set_symbol("\u{25e3}").set_style(grip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;

    const SCREEN: Rect = Rect::new(0, 0, 200, 50);

    fn mouse(kind: MouseEventKind, col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn frame() -> FloatingFrame {
        let mut f = FloatingFrame::new("test", (160, 44));
        f.layout(SCREEN);
        f
    }

    #[test]
    fn the_default_is_centered_and_capped() {
        let f = frame();
        assert_eq!(f.rect(), Rect::new(20, 3, 160, 44));
        assert!(!f.is_arranged());
    }

    #[test]
    fn a_title_drag_moves_and_survives_the_pointer_leaving_the_dialog() {
        let mut f = frame();
        let r = f.rect();
        let down = mouse(MouseEventKind::Down(MouseButton::Left), r.x + 30, r.y);
        assert_eq!(f.on_mouse(down, false, true), FrameMouse::Consumed);
        // The pointer goes outside the (old) dialog, then lets go there.
        let drag = mouse(MouseEventKind::Drag(MouseButton::Left), 5, 0);
        assert_eq!(f.on_mouse(drag, false, true), FrameMouse::Consumed);
        let up = mouse(MouseEventKind::Up(MouseButton::Left), 5, 0);
        assert_eq!(f.on_mouse(up, false, true), FrameMouse::Consumed);
        assert!(!f.dragging());
        let moved = f.layout(SCREEN);
        assert_eq!((moved.x, moved.y), (0, 0), "clamped at the top left");
        assert_eq!((moved.width, moved.height), (160, 44));
    }

    #[test]
    fn the_close_cross_keeps_the_title_press() {
        let mut f = frame();
        let r = f.rect();
        let down = mouse(MouseEventKind::Down(MouseButton::Left), r.right() - 3, r.y);
        assert_eq!(f.on_mouse(down, true, true), FrameMouse::Ignored);
        assert!(!f.dragging());
    }

    #[test]
    fn corner_and_border_drags_resize_within_the_limits() {
        let mut f = frame();
        let r = f.rect();
        let (cx, cy) = (r.right() - 1, r.bottom() - 1);
        f.on_mouse(
            mouse(MouseEventKind::Down(MouseButton::Left), cx, cy),
            false,
            true,
        );
        f.on_mouse(
            mouse(MouseEventKind::Drag(MouseButton::Left), 100, 30),
            false,
            true,
        );
        f.on_mouse(
            mouse(MouseEventKind::Up(MouseButton::Left), 100, 30),
            false,
            true,
        );
        let small = f.layout(SCREEN);
        assert_eq!((small.x, small.y), (r.x, r.y));
        assert_eq!((small.width, small.height), (100 - r.x + 1, 30 - r.y + 1));
        // Far too small: stops at the minimum.
        let (cx, cy) = (small.right() - 1, small.bottom() - 1);
        f.on_mouse(
            mouse(MouseEventKind::Down(MouseButton::Left), cx, cy),
            false,
            true,
        );
        f.on_mouse(
            mouse(MouseEventKind::Up(MouseButton::Left), 0, 0),
            false,
            true,
        );
        let min = f.layout(SCREEN);
        assert_eq!((min.width, min.height), (MIN_W, MIN_H));
        // The left border moves the left edge, the right edge stays.
        let right = min.right();
        f.on_mouse(
            mouse(MouseEventKind::Down(MouseButton::Left), min.x, min.y + 3),
            false,
            true,
        );
        f.on_mouse(
            mouse(MouseEventKind::Up(MouseButton::Left), min.x - 5, min.y + 3),
            false,
            true,
        );
        let wider = f.layout(SCREEN);
        assert_eq!(wider.right(), right);
        assert_eq!(wider.width, MIN_W + 5);
    }

    #[test]
    fn a_smaller_screen_clamps_the_geometry_every_frame() {
        let mut f = frame();
        let small = Rect::new(0, 0, 80, 24);
        let r = f.layout(small);
        assert!(small.contains(r.as_position()) && r.right() <= 80 && r.bottom() <= 24);
    }

    #[test]
    fn arrange_mode_moves_resizes_resets_and_leaves() {
        let mut f = frame();
        let key = |code, modifiers| KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        assert!(
            !f.on_key(key(KeyCode::Left, KeyModifiers::NONE)),
            "not arranging"
        );
        f.arrange_start();
        let before = f.rect();
        assert!(f.on_key(key(KeyCode::Left, KeyModifiers::NONE)));
        assert_eq!(f.rect().x, before.x - 2);
        assert!(f.on_key(key(KeyCode::Up, KeyModifiers::SHIFT)));
        assert_eq!(f.rect().height, before.height - 1);
        assert!(f.on_key(key(KeyCode::Char('0'), KeyModifiers::NONE)));
        assert_eq!(f.layout(SCREEN), before, "0 resets");
        assert!(f.on_key(key(KeyCode::Esc, KeyModifiers::NONE)));
        assert!(!f.arranging());
    }

    #[test]
    fn session_keeps_the_geometry_for_the_next_instance_and_never_does_not() {
        let mut f = frame();
        f.arrange_start();
        f.on_key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE));
        let moved = f.rect();
        let mut again = FloatingFrame::new("test", (160, 44));
        assert_eq!(again.layout(SCREEN), moved);
        crate::live_settings::install_dialog_geometry(DialogGeometry::Never);
        let mut fresh = FloatingFrame::new("test", (160, 44));
        assert_eq!(fresh.layout(SCREEN), Rect::new(20, 3, 160, 44));
        crate::live_settings::install_dialog_geometry(DialogGeometry::Session);
    }

    #[test]
    fn the_saved_file_round_trips() {
        let mut geos = HashMap::new();
        geos.insert(
            "a".to_string(),
            Geo {
                x: 1,
                y: 2,
                w: 70,
                h: 20,
            },
        );
        geos.insert(
            "b".to_string(),
            Geo {
                x: 0,
                y: 0,
                w: 99,
                h: 30,
            },
        );
        assert_eq!(parse_saved(&render_saved(&geos)), geos);
        assert!(parse_saved("garbage\n1 2 3\n").is_empty());
    }
}
