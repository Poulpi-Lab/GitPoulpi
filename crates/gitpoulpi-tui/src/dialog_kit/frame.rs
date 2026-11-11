//! Movable (and resizable) dialogs.
//!
//! A dialog adopts the frame with a [`Win`] (one `const` per dialog, keyed by
//! its id): it asks [`Win::place`] for its rect (its own default rect, or
//! where the user put it), paints inside it, then calls [`Win::decorate`]
//! (corner grips, the arrange banner, pointer shapes, and the registration
//! the gestures are routed from). Nothing else is per dialog: the event loop
//! feeds every mouse event and key to this module first ([`on_mouse`],
//! [`on_key`]), and what it does not consume is the dialog's.
//!
//! * **Mouse**: drag the title bar to move (the close cross keeps priority);
//!   a resizable dialog also resizes by its left / right / bottom borders and
//!   bottom corners. A drag continues outside the dialog and its release is
//!   consumed, so letting go outside never closes anything.
//! * **Keyboard twin**: `Ctrl+N m` (`keymap::Action::NavArrangeDialog`) starts
//!   the *arrange* mode on the dialog on top: arrows move, Shift+arrows
//!   resize (resizable ones), `0` resets, Enter or Esc leave it. The palette
//!   has the same rows ([`arrange_top`], [`reset_top`]).
//! * **Where it is kept**: ONE setting, `ui.dialog_geometry`, for every
//!   dialog; each dialog's geometry is keyed by its own id. `session` keeps
//!   it in a process-wide map, `saved` also writes the whole map to one file
//!   in the data folder and restores every id at the next launch, `never`
//!   forgets a dialog's geometry as soon as it is closed.
//!
//! The geometry is clamped to the screen on every `place`. A movable-only
//! dialog keeps its content-driven size and only its position is kept.

use std::collections::{HashMap, HashSet};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_config::DialogGeometry;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::pointer_map::{self, Kind};
use crate::th;

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

    /// Inside `screen`, at least `min` (or the whole screen when smaller).
    fn clamped(self, screen: Rect, min: (u16, u16)) -> Self {
        let w = self.w.max(min.0.min(screen.width)).min(screen.width);
        let h = self.h.max(min.1.min(screen.height)).min(screen.height);
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
    Ignored,
    Consumed,
}

#[derive(Debug, Clone)]
struct Drag {
    id: &'static str,
    grip: Grip,
    held: (u16, u16),
    start: Geo,
    min: (u16, u16),
    resizable: bool,
    screen: Rect,
}

/// A dialog as the last paint registered it.
#[derive(Debug, Clone)]
struct Live {
    id: &'static str,
    rect: Rect,
    close: Option<Rect>,
    /// `Some(min)`: resizable.
    min: Option<(u16, u16)>,
    /// False while a popup of the dialog is up (a new drag does not start).
    enabled: bool,
    screen: Rect,
}

// -- the state, process-wide -------------------------------------------------

#[derive(Default)]
struct Store {
    geos: HashMap<String, Geo>,
    /// Ids kept only while the dialog is open (`never`).
    ephemeral: HashSet<String>,
    /// The saved file has been read (once per process).
    loaded: bool,
    screens: HashMap<&'static str, Rect>,
    live: Vec<Live>,
    drag: Option<Drag>,
    arrange: Option<&'static str>,
    /// `Ctrl+N` was pressed over a framed dialog: `m` is the next key.
    armed: bool,
}

#[cfg(not(test))]
static STORE: std::sync::Mutex<Option<Store>> = std::sync::Mutex::new(None);

// A unit test gets a store of its own (tests run one per thread), and never
// touches the real data folder.
#[cfg(test)]
thread_local! {
    static STORE: std::cell::RefCell<Option<Store>> = const { std::cell::RefCell::new(None) };
    static TEST_PATH: std::cell::RefCell<Option<std::path::PathBuf>> = const { std::cell::RefCell::new(None) };
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

/// The file `saved` keeps every dialog's geometry in (`None` in tests, unless
/// one set a path, and without a config location).
fn saved_path() -> Option<std::path::PathBuf> {
    #[cfg(test)]
    {
        TEST_PATH.with(|p| p.borrow().clone())
    }
    #[cfg(not(test))]
    {
        gitpoulpi_config::ConfigPaths::discover().map(|p| p.data_dir().join("dialog_geometry.txt"))
    }
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

/// Writes every kept (not `never`) geometry to the one file.
fn persist(store: &Store) {
    let Some(path) = saved_path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = gitpoulpi_config::secure_fs::create_private_dir_all(dir);
    }
    let kept: HashMap<String, Geo> = store
        .geos
        .iter()
        .filter(|(id, _)| !store.ephemeral.contains(*id))
        .map(|(id, g)| (id.clone(), *g))
        .collect();
    let _ = gitpoulpi_config::secure_fs::write_private(&path, render_saved(&kept).as_bytes());
}

// -- the dialog's handle -------------------------------------------------------

/// How a dialog adopts the frame: one `const` per dialog.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Win {
    id: &'static str,
    /// `Some(min)`: resizable (never below `min`); `None`: movable only.
    min: Option<(u16, u16)>,
}

impl Win {
    /// Move only (a small prompt or confirmation whose size follows its text).
    pub(crate) const fn movable(id: &'static str) -> Self {
        Self { id, min: None }
    }

    /// Move and resize, never smaller than `min_w` x `min_h`.
    pub(crate) const fn resizable(id: &'static str, min_w: u16, min_h: u16) -> Self {
        Self {
            id,
            min: Some((min_w, min_h)),
        }
    }

    /// The rect to paint in: where the user put the dialog (per
    /// `ui.dialog_geometry`), else `want`, the dialog's own default. Always
    /// inside `screen`. Pure: callable from paint and from hit-testing.
    pub(crate) fn place(&self, screen: Rect, want: Rect) -> Rect {
        let mode = crate::live_settings::dialog_geometry();
        let kept = with_store(|s| {
            s.screens.insert(self.id, screen);
            if mode == DialogGeometry::Saved {
                load_saved(s);
            }
            s.geos.get(self.id).copied()
        });
        let Some(geo) = kept else {
            return want;
        };
        match self.min {
            Some(min) => geo.clamped(screen, min).rect(),
            None => Geo {
                x: geo.x,
                y: geo.y,
                w: want.width,
                h: want.height,
            }
            .clamped(screen, (1, 1))
            .rect(),
        }
    }

    /// After the dialog painted in `rect`: registers it for the gestures, adds
    /// the grips and (in arrange mode) the banner, and sets the pointer
    /// shapes. `close` is the close cross (it keeps its click); `enabled` is
    /// false while a popup of the dialog is up.
    pub(crate) fn decorate(
        &self,
        buf: &mut Buffer,
        rect: Rect,
        close: Option<Rect>,
        enabled: bool,
    ) {
        let (arranging, drag, screen) = with_store(|s| {
            let screen = s.screens.get(self.id).copied().unwrap_or(buf.area);
            s.live.retain(|l| l.id != self.id);
            s.live.push(Live {
                id: self.id,
                rect,
                close,
                min: self.min,
                enabled,
                screen,
            });
            (
                s.arrange == Some(self.id),
                s.drag.as_ref().filter(|d| d.id == self.id).cloned(),
                screen,
            )
        });
        paint_marks(buf, rect, self.min.is_some(), arranging);
        if enabled || drag.is_some() {
            register_pointer(rect, close, self.min.is_some());
        }
        if let Some(drag) = drag {
            pointer_map::register(screen, drag.grip.kind());
        }
    }
}

/// Called once per frame before painting: forgets what the dialogs that were
/// closed kept (`never`), ends gestures on them, and clears the registry.
pub(crate) fn begin_frame() {
    with_store(|s| {
        let open: HashSet<&str> = s.live.iter().map(|l| l.id).collect();
        let gone: Vec<String> = s
            .ephemeral
            .iter()
            .filter(|id| !open.contains(id.as_str()))
            .cloned()
            .collect();
        for id in gone {
            s.ephemeral.remove(&id);
            s.geos.remove(&id);
        }
        if s.arrange.is_some_and(|id| !open.contains(id)) {
            s.arrange = None;
        }
        if s.drag.as_ref().is_some_and(|d| !open.contains(d.id)) {
            s.drag = None;
        }
        if s.live.is_empty() {
            s.armed = false;
        }
        s.live.clear();
    });
}

/// A framed dialog is on screen (as of the last paint).
pub(crate) fn any_live() -> bool {
    with_store(|s| !s.live.is_empty())
}

/// The id of the dialog on top.
#[cfg(test)]
pub(crate) fn top_id() -> Option<&'static str> {
    with_store(|s| s.live.last().map(|l| l.id))
}

/// The arrange mode is on for some dialog.
#[cfg(test)]
pub(crate) fn is_arranging() -> bool {
    with_store(|s| s.arrange.is_some())
}

/// A drag of a frame is in progress.
#[cfg(test)]
pub(crate) fn is_dragging() -> bool {
    with_store(|s| s.drag.is_some())
}

/// Starts the arrange mode on the dialog on top. `false`: none is open.
pub(crate) fn arrange_top() -> bool {
    with_store(|s| {
        s.armed = false;
        s.drag = None;
        match s.live.last() {
            Some(l) => {
                s.arrange = Some(l.id);
                true
            }
            None => false,
        }
    })
}

/// Puts the dialog on top back at its default place and size. `false`: none.
pub(crate) fn reset_top() -> bool {
    let mode = crate::live_settings::dialog_geometry();
    with_store(|s| {
        let Some(id) = s.live.last().map(|l| l.id) else {
            return false;
        };
        s.drag = None;
        s.geos.remove(id);
        s.ephemeral.remove(id);
        if mode == DialogGeometry::Saved {
            persist(s);
        }
        true
    })
}

/// `Ctrl+N` over a framed dialog without badges of its own: `m` is next.
pub(crate) fn arm_prefix() {
    with_store(|s| s.armed = true);
}

/// Whether the prefix was armed (and disarms it).
pub(crate) fn take_prefix() -> bool {
    with_store(|s| std::mem::take(&mut s.armed))
}

fn grip_at(l: &Live, col: u16, row: u16) -> Option<Grip> {
    let r = l.rect;
    if r.width < 4 || r.height < 4 || !r.contains((col, row).into()) {
        return None;
    }
    let on_close = l.close.is_some_and(|c| c.contains((col, row).into()));
    let (left, right, bottom) = (r.x, r.right() - 1, r.bottom() - 1);
    if row == r.y {
        return (!on_close && col > left && col < right).then_some(Grip::Title);
    }
    l.min?;
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

/// Feeds one mouse event. `Consumed`: a gesture of a frame took it (a press on
/// a grip, a drag, its release): nothing else sees it.
pub(crate) fn on_mouse(mouse: MouseEvent) -> FrameMouse {
    let (col, row) = (mouse.column, mouse.row);
    let mode = crate::live_settings::dialog_geometry();
    let mut keep = false;
    let out = with_store(|s| match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // The topmost dialog under the pointer decides.
            let Some(l) = s
                .live
                .iter()
                .rev()
                .find(|l| l.rect.contains((col, row).into()))
                .cloned()
            else {
                return FrameMouse::Ignored;
            };
            if !l.enabled {
                return FrameMouse::Ignored;
            }
            let Some(grip) = grip_at(&l, col, row) else {
                return FrameMouse::Ignored;
            };
            s.arrange = None;
            s.drag = Some(Drag {
                id: l.id,
                grip,
                held: (col.saturating_sub(l.rect.x), row.saturating_sub(l.rect.y)),
                start: Geo::of(l.rect),
                min: l.min.unwrap_or((1, 1)),
                resizable: l.min.is_some(),
                screen: l.screen,
            });
            FrameMouse::Consumed
        }
        MouseEventKind::Drag(MouseButton::Left) if s.drag.is_some() => {
            drag_to(s, col, row, mode);
            FrameMouse::Consumed
        }
        MouseEventKind::Up(MouseButton::Left) if s.drag.is_some() => {
            drag_to(s, col, row, mode);
            s.drag = None;
            keep = true;
            FrameMouse::Consumed
        }
        _ => FrameMouse::Ignored,
    });
    if keep && mode == DialogGeometry::Saved {
        with_store(|s| persist(s));
    }
    out
}

fn drag_to(s: &mut Store, col: u16, row: u16, mode: DialogGeometry) {
    let Some(drag) = s.drag.clone() else {
        return;
    };
    let st = drag.start;
    let (col, row) = (i32::from(col), i32::from(row));
    let right = i32::from(st.x) + i32::from(st.w);
    let (mut x, mut y, mut w, mut h) = (
        i32::from(st.x),
        i32::from(st.y),
        i32::from(st.w),
        i32::from(st.h),
    );
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
    let min_w = i32::from(drag.min.0.min(drag.screen.width));
    let min_h = i32::from(drag.min.1.min(drag.screen.height));
    if matches!(drag.grip, Grip::Left | Grip::BottomLeft) {
        x = x.min(right - min_w);
        w = right - x;
    }
    if drag.resizable {
        w = w.max(min_w);
        h = h.max(min_h);
    }
    set(s, drag.id, drag.screen, drag.min, (x, y, w, h), mode);
}

fn set(
    s: &mut Store,
    id: &'static str,
    screen: Rect,
    min: (u16, u16),
    (x, y, w, h): (i32, i32, i32, i32),
    mode: DialogGeometry,
) {
    let geo = Geo {
        x: x.clamp(0, i32::from(u16::MAX)) as u16,
        y: y.clamp(0, i32::from(u16::MAX)) as u16,
        w: w.clamp(1, i32::from(u16::MAX)) as u16,
        h: h.clamp(1, i32::from(u16::MAX)) as u16,
    }
    .clamped(screen, min);
    s.geos.insert(id.to_string(), geo);
    if mode == DialogGeometry::Never {
        s.ephemeral.insert(id.to_string());
    } else {
        s.ephemeral.remove(id);
    }
    if let Some(l) = s.live.iter_mut().find(|l| l.id == id) {
        l.rect = geo.rect();
    }
}

/// A key while the arrange mode is on. `true`: the frame took it.
pub(crate) fn on_key(key: KeyEvent) -> bool {
    let mode = crate::live_settings::dialog_geometry();
    let mut persist_now = false;
    let took = with_store(|s| {
        let Some(id) = s.arrange else {
            return false;
        };
        let Some(l) = s.live.iter().find(|l| l.id == id).cloned() else {
            s.arrange = None;
            return false;
        };
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let (dx, dy) = match key.code {
            KeyCode::Left => (-KEY_STEP_X, 0),
            KeyCode::Right => (KEY_STEP_X, 0),
            KeyCode::Up => (0, -KEY_STEP_Y),
            KeyCode::Down => (0, KEY_STEP_Y),
            KeyCode::Char('0') => {
                s.geos.remove(id);
                s.ephemeral.remove(id);
                persist_now = true;
                return true;
            }
            KeyCode::Enter | KeyCode::Esc => {
                s.arrange = None;
                persist_now = true;
                return true;
            }
            _ => return true,
        };
        let r = l.rect;
        let min = l.min.unwrap_or((1, 1));
        let (x, y) = (i32::from(r.x), i32::from(r.y));
        let (w, h) = (i32::from(r.width), i32::from(r.height));
        if shift && l.min.is_some() {
            let size = (
                x,
                y,
                (w + dx).max(i32::from(min.0)),
                (h + dy).max(i32::from(min.1)),
            );
            set(s, id, l.screen, min, size, mode);
        } else if !shift {
            set(s, id, l.screen, min, (x + dx, y + dy, w, h), mode);
        }
        true
    });
    if persist_now && mode == DialogGeometry::Saved {
        with_store(|s| persist(s));
    }
    took
}

fn register_pointer(r: Rect, close: Option<Rect>, resizable: bool) {
    if r.width < 4 || r.height < 4 {
        return;
    }
    let (left, right, bottom) = (r.x, r.right() - 1, r.bottom() - 1);
    let mut title = Rect::new(left + 1, r.y, r.width - 2, 1);
    if let Some(c) = close {
        if c.x > title.x && c.x < title.right() {
            title.width = c.x - title.x;
        }
    }
    pointer_map::register(title, Kind::Move);
    if !resizable {
        return;
    }
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
}

/// The frame's marks on the dialog's border: the corner grips (resizable
/// only), and in the arrange mode the banner on the bottom border with the
/// border in the warning color.
fn paint_marks(buf: &mut Buffer, rect: Rect, resizable: bool, arranging: bool) {
    let r = rect.intersection(buf.area);
    if r.width < 4 || r.height < 4 {
        return;
    }
    let (left, right, bottom) = (r.x, r.right() - 1, r.bottom() - 1);
    let border = Style::default().fg(th().ui.accent).bg(th().ui.panel);
    let grip = border.add_modifier(Modifier::BOLD);
    if arranging {
        let warn = Style::default().fg(th().status.warning).bg(th().ui.panel);
        for x in left..=right {
            buf[(x, r.y)].set_style(warn);
            buf[(x, bottom)].set_style(warn);
        }
        for y in r.y..=bottom {
            buf[(left, y)].set_style(warn);
            buf[(right, y)].set_style(warn);
        }
        let hint = gitpoulpi_i18n::msg::dialog_frame::arrange_hint(super::key_name(
            crate::keymap::chord::Key::Enter,
        ));
        super::put(
            buf,
            left + 2,
            bottom,
            &format!(" {hint} "),
            r.width.saturating_sub(6),
            warn.add_modifier(Modifier::BOLD),
        );
    }
    if resizable {
        buf[(right, bottom)].set_symbol("\u{25e2}").set_style(grip);
        buf[(left, bottom)].set_symbol("\u{25e3}").set_style(grip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    const SCREEN: Rect = Rect::new(0, 0, 200, 50);
    const BIG: Win = Win::resizable("t_big", 60, 16);
    const SMALL: Win = Win::movable("t_small");
    const OTHER: Win = Win::resizable("t_other", 40, 10);

    fn mouse(kind: MouseEventKind, col: u16, row: u16) -> MouseEvent {
        MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        }
    }

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn want() -> Rect {
        Rect::new(20, 3, 160, 44)
    }

    fn other_want() -> Rect {
        Rect::new(10, 5, 80, 20)
    }

    /// One frame of a dialog: place, decorate.
    fn show(win: Win, want: Rect) -> Rect {
        let r = win.place(SCREEN, want);
        let mut buf = Buffer::empty(SCREEN);
        let close = Rect::new(r.right().saturating_sub(4), r.y, 3, 1);
        win.decorate(&mut buf, r, Some(close), true);
        r
    }

    fn fresh() {
        STORE.with(|s| *s.borrow_mut() = None);
        TEST_PATH.with(|p| *p.borrow_mut() = None);
        crate::live_settings::install_dialog_geometry(DialogGeometry::Session);
    }

    fn drag(from: (u16, u16), to: (u16, u16)) {
        on_mouse(mouse(
            MouseEventKind::Down(MouseButton::Left),
            from.0,
            from.1,
        ));
        on_mouse(mouse(MouseEventKind::Drag(MouseButton::Left), to.0, to.1));
        on_mouse(mouse(MouseEventKind::Up(MouseButton::Left), to.0, to.1));
    }

    #[test]
    fn the_default_is_the_dialogs_own_rect() {
        fresh();
        assert_eq!(show(BIG, want()), want());
    }

    #[test]
    fn a_title_drag_moves_and_survives_the_pointer_leaving_the_dialog() {
        fresh();
        let r = show(BIG, want());
        let down = mouse(MouseEventKind::Down(MouseButton::Left), r.x + 30, r.y);
        assert_eq!(on_mouse(down), FrameMouse::Consumed);
        let moved = mouse(MouseEventKind::Drag(MouseButton::Left), 5, 0);
        assert_eq!(on_mouse(moved), FrameMouse::Consumed);
        let up = mouse(MouseEventKind::Up(MouseButton::Left), 5, 0);
        assert_eq!(on_mouse(up), FrameMouse::Consumed);
        assert!(!is_dragging());
        let moved = show(BIG, want());
        assert_eq!((moved.x, moved.y), (0, 0), "clamped at the top left");
        assert_eq!((moved.width, moved.height), (160, 44));
    }

    #[test]
    fn the_close_cross_keeps_the_title_press() {
        fresh();
        let r = show(BIG, want());
        let down = mouse(MouseEventKind::Down(MouseButton::Left), r.right() - 3, r.y);
        assert_eq!(on_mouse(down), FrameMouse::Ignored);
        assert!(!is_dragging());
    }

    #[test]
    fn corner_and_border_drags_resize_within_the_limits() {
        fresh();
        let r = show(BIG, want());
        drag((r.right() - 1, r.bottom() - 1), (100, 30));
        let small = show(BIG, want());
        assert_eq!((small.x, small.y), (r.x, r.y));
        assert_eq!((small.width, small.height), (100 - r.x + 1, 30 - r.y + 1));
        drag((small.right() - 1, small.bottom() - 1), (0, 0));
        let min = show(BIG, want());
        assert_eq!((min.width, min.height), (60, 16));
        let right = min.right();
        drag((min.x, min.y + 3), (min.x - 5, min.y + 3));
        let wider = show(BIG, want());
        assert_eq!(wider.right(), right);
        assert_eq!(wider.width, 65);
    }

    #[test]
    fn a_movable_dialog_moves_but_has_no_resize_grips() {
        fresh();
        let w = Rect::new(30, 10, 60, 8);
        let r = show(SMALL, w);
        let corner = mouse(
            MouseEventKind::Down(MouseButton::Left),
            r.right() - 1,
            r.bottom() - 1,
        );
        assert_eq!(on_mouse(corner), FrameMouse::Ignored);
        drag((r.x + 5, r.y), (r.x + 15, r.y + 6));
        let moved = show(SMALL, w);
        assert_eq!((moved.x, moved.y), (40, 16));
        assert_eq!((moved.width, moved.height), (60, 8), "size follows content");
        let taller = show(SMALL, Rect::new(30, 10, 60, 12));
        assert_eq!((taller.x, taller.y, taller.height), (40, 16, 12));
    }

    #[test]
    fn a_smaller_screen_clamps_the_geometry_every_frame() {
        fresh();
        let r = show(BIG, want());
        drag((r.x + 3, r.y), (r.x + 20, r.y + 2));
        let small = Rect::new(0, 0, 80, 24);
        let r = BIG.place(small, want());
        assert!(r.right() <= 80 && r.bottom() <= 24);
    }

    #[test]
    fn arrange_mode_moves_resizes_resets_and_leaves_on_two_dialogs() {
        fresh();
        for (win, w, resizes) in [
            (BIG, want(), true),
            (SMALL, Rect::new(30, 10, 60, 8), false),
        ] {
            let before = show(win, w);
            assert!(!on_key(key(KeyCode::Left, KeyModifiers::NONE)));
            assert!(arrange_top());
            assert!(on_key(key(KeyCode::Left, KeyModifiers::NONE)));
            assert_eq!(show(win, w).x, before.x - 2);
            assert!(on_key(key(KeyCode::Up, KeyModifiers::SHIFT)));
            let after = show(win, w);
            let want_h = if resizes { before.height - 1 } else { w.height };
            assert_eq!(after.height, want_h);
            assert!(on_key(key(KeyCode::Char('0'), KeyModifiers::NONE)));
            assert_eq!(show(win, w), before, "0 resets");
            assert!(on_key(key(KeyCode::Esc, KeyModifiers::NONE)));
            assert!(!is_arranging());
            begin_frame();
        }
    }

    #[test]
    fn arrange_acts_on_the_dialog_on_top() {
        fresh();
        show(BIG, want());
        let top = show(OTHER, other_want());
        assert_eq!(top_id(), Some("t_other"));
        assert!(arrange_top());
        on_key(key(KeyCode::Right, KeyModifiers::NONE));
        assert_eq!(show(OTHER, other_want()).x, top.x + 2);
        assert_eq!(show(BIG, want()), want(), "the one below is untouched");
    }

    #[test]
    fn session_keeps_per_dialog_geometry_and_never_forgets_on_close() {
        fresh();
        let a = show(BIG, want());
        drag((a.x + 3, a.y), (a.x + 13, a.y + 2));
        let moved = show(BIG, want());
        assert_ne!(moved, want());
        assert_eq!(BIG.place(SCREEN, want()), moved, "next instance of the id");
        assert_eq!(OTHER.place(SCREEN, other_want()), other_want());
        crate::live_settings::install_dialog_geometry(DialogGeometry::Never);
        begin_frame();
        let o = show(OTHER, other_want());
        drag((o.x + 3, o.y), (o.x + 9, o.y + 4));
        let m = show(OTHER, other_want());
        assert_eq!((m.x, m.y), (16, 9), "lives while the dialog is open");
        begin_frame();
        begin_frame();
        assert_eq!(
            OTHER.place(SCREEN, other_want()),
            other_want(),
            "starts over once closed"
        );
        fresh();
    }

    #[test]
    fn saved_keeps_every_dialog_in_one_file_and_restores_each() {
        fresh();
        let dir = std::env::temp_dir().join(format!("gp-geo-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("dialog_geometry.txt");
        let _ = std::fs::remove_file(&file);
        TEST_PATH.with(|p| *p.borrow_mut() = Some(file.clone()));
        crate::live_settings::install_dialog_geometry(DialogGeometry::Saved);
        let a = show(BIG, want());
        drag((a.x + 3, a.y), (a.x + 13, a.y + 2));
        let o = show(OTHER, other_want());
        drag((o.right() - 1, o.bottom() - 1), (70, 20));
        let (a2, o2) = (show(BIG, want()), show(OTHER, other_want()));
        assert_ne!(a2, want());
        assert_eq!(o2.width, 61);
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("t_big ") && text.contains("t_other "), "{text}");
        // A new launch: the map starts empty, the file restores both.
        STORE.with(|s| *s.borrow_mut() = None);
        assert_eq!(BIG.place(SCREEN, want()), a2);
        assert_eq!(OTHER.place(SCREEN, other_want()), o2);
        // Reset of one dialog rewrites the file without it.
        show(OTHER, other_want());
        assert!(reset_top());
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("t_big ") && !text.contains("t_other "), "{text}");
        let _ = std::fs::remove_dir_all(&dir);
        fresh();
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
