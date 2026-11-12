//! Where the caret of a focused text field is shown (`ui.text_cursor`).
//!
//! Every editable field is drawn by `text_field::edit_lines`. In `drawn` mode
//! it paints its own caret (a reversed character, or a glyph after the text)
//! and the terminal's cursor stays hidden. In `terminal` mode it paints no
//! caret: it tags the cell the caret sits on with [`MARK`], and the finished
//! frame is scanned once ([`take_from_buffer`]) for that tag, which is the
//! only place that knows the field's final position on screen (the field
//! itself only knows its own rows and columns). The tag is cleared and the
//! terminal's real cursor is put there (`Frame::set_cursor_position`), as a
//! bar (DECSCUSR), so the terminal's own cursor look applies and an input
//! method's candidate window opens at the caret.
//!
//! A cell an overlay painted over loses its tag with the rest of its style,
//! so a field hidden under a dialog never gets the cursor.
//!
//! The mode is per thread: the UI thread installs it, and unit tests (which
//! draw fields on their own threads and pin the drawn caret in snapshots)
//! start in `drawn`.

use std::cell::Cell;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

use gitpoulpi_config::TextCursor;
use ratatui::buffer::Buffer;
use ratatui::layout::Position;
use ratatui::style::Modifier;

/// The style bit that tags the caret cell until the frame is finished. A
/// real blinking style is never used by this UI, and the tag never reaches
/// the terminal: it is cleared before the frame is flushed.
pub(crate) const MARK: Modifier = Modifier::RAPID_BLINK;

/// DECSCUSR: a blinking bar.
const BAR: &[u8] = b"\x1b[5 q";
/// DECSCUSR: back to the user's own cursor style.
const RESTORE: &[u8] = b"\x1b[0 q";

thread_local! {
    static MODE: Cell<TextCursor> = const {
        Cell::new(if cfg!(test) { TextCursor::Drawn } else { TextCursor::Terminal })
    };
    /// Whether a field asked for the terminal cursor while drawing this frame.
    static ASKED: Cell<bool> = const { Cell::new(false) };
    /// The position the last finished frame put the cursor at.
    static LAST: Cell<Option<Position>> = const { Cell::new(None) };
}

/// Whether the bar style was sent and not yet restored. Process-wide: the
/// error exit restores from another thread than the one that sent it.
static STYLE_SENT: AtomicBool = AtomicBool::new(false);

/// Installs the setting (start-up, and again when Settings closes).
pub(crate) fn install(mode: TextCursor) {
    MODE.with(|m| m.set(mode));
}

/// The current mode.
pub(crate) fn mode() -> TextCursor {
    MODE.with(Cell::get)
}

/// Whether the terminal's cursor is the caret.
pub(crate) fn is_terminal() -> bool {
    mode() == TextCursor::Terminal
}

/// Called by a field that has the keyboard and drew a caret cell tagged with
/// [`MARK`].
pub(crate) fn note_marked() {
    ASKED.with(|a| a.set(true));
}

/// The style bit that shows a caret on one cell: [`MARK`] in `terminal` mode
/// (the cursor goes there), reversed video in `drawn` mode. For the few
/// painters that draw a caret without `text_field` (the code editor, the
/// merge editor).
pub(crate) fn caret_modifier() -> Modifier {
    if is_terminal() {
        note_marked();
        MARK
    } else {
        Modifier::REVERSED
    }
}

/// Finds the tagged caret cell in the finished frame (the first one in reading
/// order when several are tagged), clears every tag and returns its
/// position. Costs nothing on a frame where no field asked.
pub(crate) fn take_from_buffer(buf: &mut Buffer) -> Option<Position> {
    if !ASKED.with(|a| a.replace(false)) {
        LAST.with(|l| l.set(None));
        return None;
    }
    let area = buf.area;
    let mut found = None;
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            if cell.modifier.contains(MARK) {
                cell.modifier.remove(MARK);
                found.get_or_insert(Position::new(x, y));
            }
        }
    }
    LAST.with(|l| l.set(found));
    found
}

/// Where the last finished frame put the cursor, for re-placing it after a
/// raw write (a picture) moved it.
pub(crate) fn last() -> Option<Position> {
    LAST.with(Cell::get)
}

/// Sends the bar style when the mode turned to `terminal`, and the user's own
/// style back when it turned to `drawn`. Writes nothing when nothing changed.
pub(crate) fn sync_style<W: Write>(w: &mut W) {
    let want = is_terminal();
    if STYLE_SENT.swap(want, Ordering::SeqCst) == want {
        return;
    }
    let _ = w.write_all(if want { BAR } else { RESTORE });
    let _ = w.flush();
}

/// Hands the user's cursor style back on every way out (normal exit, error
/// exit, panic, update). A no-op when the bar was never sent.
pub(crate) fn restore_style<W: Write>(w: &mut W) {
    if STYLE_SENT.swap(false, Ordering::SeqCst) {
        let _ = w.write_all(RESTORE);
        let _ = w.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    /// `STYLE_SENT` is process-wide: the tests that touch it take turns.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn a_tagged_cell_is_found_untagged_and_reported() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 3));
        buf[(4, 1)].modifier.insert(MARK);
        note_marked();
        assert_eq!(take_from_buffer(&mut buf), Some(Position::new(4, 1)));
        assert!(!buf[(4, 1)].modifier.contains(MARK), "the tag is cleared");
        assert_eq!(last(), Some(Position::new(4, 1)));
    }

    #[test]
    fn a_frame_nobody_asked_for_has_no_cursor_even_with_a_stray_tag() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 3));
        buf[(4, 1)].modifier.insert(MARK);
        assert_eq!(take_from_buffer(&mut buf), None);
    }

    #[test]
    fn the_bar_goes_out_once_and_is_restored_once() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        restore_style(&mut Vec::new());
        install(TextCursor::Terminal);
        let mut out = Vec::new();
        sync_style(&mut out);
        sync_style(&mut out);
        assert_eq!(out, BAR);
        restore_style(&mut out);
        restore_style(&mut out);
        assert_eq!(out, [BAR, RESTORE].concat());
        install(TextCursor::Drawn);
    }

    #[test]
    fn switching_to_drawn_gives_the_users_style_back() {
        let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        restore_style(&mut Vec::new());
        install(TextCursor::Terminal);
        let mut out = Vec::new();
        sync_style(&mut out);
        install(TextCursor::Drawn);
        sync_style(&mut out);
        assert_eq!(out, [BAR, RESTORE].concat());
    }
}
