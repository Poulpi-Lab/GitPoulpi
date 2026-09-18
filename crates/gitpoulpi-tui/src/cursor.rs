//! OS mouse-pointer shape hinting via the Kitty/WezTerm `OSC 22` escape
//! sequence. `crossterm` only exposes the *text caret* style
//! (`SetCursorStyle`/DECSCUSR), never the OS pointer glyph, but the terminal
//! emulator that owns the window can be told to change the pointer via an
//! escape sequence read from the application's own output stream.
//!
//! Kitty defined `OSC 22 ; <css-cursor-name> ST` to set the OS pointer to
//! one of ~30 standard CSS cursor names while it hovers the application's
//! content area (a bare `OSC 22 ; ST` resets to the default arrow). WezTerm
//! has an **open, unmerged** PR (wez/wezterm#6292) implementing the same
//! protocol, supporting `pointer`, `text`, `row-resize`/`ns-resize` and
//! `ew-resize`/`ns-resize` (anything else falls back to the plain arrow),
//! so on a WezTerm build without it this is a no-op. It is not a
//! Windows-specific `SetCursor`/`user32.dll` hack: the terminal emulator
//! owns its window's cursor handling and a child console process cannot
//! reach past it, so OSC 22 is the only path. That is also why this needs no
//! `cfg`-gated platform split: the sequence travels over the same stdout
//! stream everywhere, and a terminal that does not understand it ignores it
//! (as for any unrecognized OSC terminated with ST), so emitting it
//! unconditionally is safe.
//!
//! `crossterm`'s `SetCursorStyle` (text-caret shape) is a separate concern
//! and untouched: the app hides/repositions the real text cursor for its own
//! UI. The affordance that works on every terminal is the in-content hover
//! highlight (see `HOVER_BG` in `lib.rs`).

use std::io::Write;

/// The OS pointer shapes this app has a reason to request. Kept narrow (not
/// all ~30 CSS names) to match what WezTerm's OSC 22 PR supports, so nothing
/// requested is silently dropped by the "unsupported value" fallback to the
/// arrow once it ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CursorShape {
    /// The default OS arrow - nothing interactive under the pointer.
    Default,
    /// A generally-clickable element (toolbar buttons, commit rows, menu
    /// items, pills).
    Pointer,
    /// Hovering a horizontal pane-divider or a Log column's resize
    /// boundary drag zone.
    ColResize,
    /// A vertical (row) resize zone: the border between two Refs sidebar
    /// sections.
    RowResize,
    /// A ref is actually being *carried*: the open hand shows from the moment
    /// a ref drag starts until it is dropped or canceled, and at no other
    /// time.
    ///
    /// Hovering a ref is hovering a clickable thing, so it takes `Pointer`
    /// like every other row and button; the open hand means "you are holding
    /// this". The rule lives in `desired_cursor_shape` and is pinned by
    /// `hovering_a_ref_shows_the_pointing_finger_not_the_open_hand`.
    ///
    /// No `Grabbing` (the closed hand CSS switches to while the button is
    /// held) on purpose: the open hand is wanted *during* the drag, and this
    /// enum is deliberately narrow, so `grabbing` would be a second name
    /// today's terminals drop, at the price of a different glyph than wanted
    /// on the terminals that do implement it.
    ///
    /// `grab` is already outside WezTerm PR 6292's supported set, so it falls
    /// back to the plain arrow there while Kitty shows the real hand. That is
    /// why the drag's in-content feedback (the lit drop target and the footer
    /// banner, see `RefDrag` in `lib.rs`) carries the gesture, with the OS
    /// pointer as the bonus.
    Grab,
    /// An editable text field (search box, filter, Settings text row, dialog
    /// input, composer): the I-beam, OSC 22 `text`.
    Text,
}

impl CursorShape {
    /// The OSC 22 CSS cursor name for this shape, or `None` for `Default`
    /// (which resets via an empty-parameter OSC 22 instead of naming
    /// "default": both are valid per the Kitty spec, but the reset form is
    /// the more widely implemented).
    fn css_name(self) -> Option<&'static str> {
        match self {
            CursorShape::Default => None,
            CursorShape::Pointer => Some("pointer"),
            CursorShape::ColResize => Some("ew-resize"),
            CursorShape::RowResize => Some("ns-resize"),
            CursorShape::Grab => Some("grab"),
            CursorShape::Text => Some("text"),
        }
    }
}

/// Writes the `OSC 22` sequence for `shape` to `w` and flushes. The caller
/// (the `MouseEventKind::Moved` handler in `lib.rs`) calls this only when the
/// desired shape changed since the last frame, to avoid spamming the
/// terminal's input parser on every pixel of mouse movement.
pub(crate) fn emit_cursor_shape<W: Write>(w: &mut W, shape: CursorShape) -> std::io::Result<()> {
    emit_gated(w, shape, pointer_shapes_allowed())
}

/// Set once any non-default shape has been sent, so the exit paths reset the
/// pointer only when this process changed it.
static SHAPE_SENT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether `OSC 22` may be written at all: not behind a multiplexer (tmux,
/// screen, zellij and herdr swallow or print it, and nothing here wraps it
/// for passthrough) and not on a terminal that has no pointer to change.
fn pointer_shapes_allowed() -> bool {
    static ALLOWED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ALLOWED.get_or_init(|| {
        let bare = std::env::var("TERM").is_ok_and(|t| matches!(t.as_str(), "linux" | "dumb"));
        !bare && !crate::terminal_image::inside_multiplexer()
    })
}

fn emit_gated<W: Write>(w: &mut W, shape: CursorShape, allowed: bool) -> std::io::Result<()> {
    if !allowed {
        return Ok(());
    }
    if shape != CursorShape::Default {
        SHAPE_SENT.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    write_osc(w, shape)
}

/// Sends `shape` only when it differs from `last` (the state the terminal is
/// in), and records it. `true` when the shape changed. The Radar's mouse path
/// and the unit tests use it.
pub(crate) fn emit_on_change<W: Write>(
    w: &mut W,
    last: &mut CursorShape,
    shape: CursorShape,
    allowed: bool,
) -> std::io::Result<bool> {
    if shape == *last {
        return Ok(false);
    }
    emit_gated(w, shape, allowed)?;
    *last = shape;
    Ok(true)
}

/// [`emit_on_change`] with the terminal gate (`pointer_shapes_allowed`).
pub(crate) fn set_pointer_shape<W: Write>(
    w: &mut W,
    last: &mut CursorShape,
    shape: CursorShape,
) -> std::io::Result<bool> {
    emit_on_change(w, last, shape, pointer_shapes_allowed())
}

/// Gives the pointer back to the terminal (exit, panic): the default shape,
/// and only if this process ever changed it.
pub(crate) fn reset_pointer<W: Write>(w: &mut W) {
    if SHAPE_SENT.swap(false, std::sync::atomic::Ordering::Relaxed) && pointer_shapes_allowed() {
        let _ = write_osc(w, CursorShape::Default);
        let _ = w.flush();
    }
}

fn write_osc<W: Write>(w: &mut W, shape: CursorShape) -> std::io::Result<()> {
    match shape.css_name() {
        Some(name) => write!(w, "\u{1b}]22;{name}\u{1b}\\"),
        None => write!(w, "\u{1b}]22;\u{1b}\\"),
    }?;
    w.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_names_match_wezterm_pr_6292s_supported_set() {
        assert_eq!(CursorShape::Pointer.css_name(), Some("pointer"));
        assert_eq!(CursorShape::ColResize.css_name(), Some("ew-resize"));
        assert_eq!(CursorShape::RowResize.css_name(), Some("ns-resize"));
        assert_eq!(CursorShape::Default.css_name(), None);
    }

    /// `grab` is the one name this enum requests that PR 6292 does not
    /// implement (it falls back to the arrow there, and renders as a real
    /// open hand on Kitty). Pinned so the day someone widens that set, the
    /// trade-off recorded on `CursorShape::Grab` is re-read.
    #[test]
    fn grab_is_the_css_open_hand_and_nothing_else() {
        assert_eq!(CursorShape::Grab.css_name(), Some("grab"));
    }

    #[test]
    fn a_text_field_is_the_css_text_cursor() {
        assert_eq!(CursorShape::Text.css_name(), Some("text"));
    }

    #[test]
    fn emit_writes_a_terminated_osc_22_sequence() {
        let mut buf = Vec::new();
        emit_gated(&mut buf, CursorShape::Pointer, true).unwrap();
        assert_eq!(buf, b"\x1b]22;pointer\x1b\\");
    }

    /// Entering a boundary sends one `ew-resize`, moving along it sends
    /// nothing, leaving sends one reset; a gated terminal gets no bytes.
    #[test]
    fn the_pointer_shape_is_sent_on_a_change_only() {
        let mut out = Vec::new();
        let mut last = CursorShape::Default;
        let mut send = |shape| emit_on_change(&mut out, &mut last, shape, true).unwrap();
        assert!(send(CursorShape::ColResize));
        assert!(!send(CursorShape::ColResize));
        assert!(!send(CursorShape::ColResize));
        assert!(send(CursorShape::Default));
        assert!(!send(CursorShape::Default));
        assert_eq!(out, b"\x1b]22;ew-resize\x1b\\\x1b]22;\x1b\\");

        let mut quiet = Vec::new();
        let mut last = CursorShape::Default;
        assert!(emit_on_change(&mut quiet, &mut last, CursorShape::ColResize, false).unwrap());
        assert!(quiet.is_empty());
    }

    #[test]
    fn emit_default_writes_an_empty_reset_sequence() {
        let mut buf = Vec::new();
        emit_gated(&mut buf, CursorShape::Default, true).unwrap();
        assert_eq!(buf, b"\x1b]22;\x1b\\");
    }
}
