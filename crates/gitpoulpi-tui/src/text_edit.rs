//! The one editing core every text field in the TUI types through
//!
//! One copy of the caret, selection and key handling, so fields cannot
//! disagree on details such as which one takes `Ctrl+U` or filters AltGr
//! characters out:
//!
//! - [`Caret`] is the caret and the selection of one field, as byte
//!   offsets into its text. It sits *beside* the field's `String` rather
//!   than wrapping it, so every reader of the text (filters, submits,
//!   renders) is untouched, and a field whose text is replaced from outside
//!   (a folder chosen in the picker, a path completed) just has its caret
//!   clamped back into range on the next use.
//! - [`Caret::on_key`] is the key map: characters, `Ctrl+A` select all,
//!   `Ctrl+U` / `Ctrl+K` delete to the start / end of the line, Backspace
//!   and Delete, the arrows (with Shift extending the selection and Ctrl
//!   jumping words), Home/End, and `Ctrl+C` / `Ctrl+X` handing the
//!   selection back for the clipboard. Enter, Esc and Tab are never
//!   claimed: what they do is the field's business, not the editor's.
//! - [`FieldEditor`] is the owner-side glue: one caret for whichever of
//!   a surface's fields has the keyboard, reset whenever that changes,
//!   plus the per-frame record of where each field was drawn so a mouse
//!   press, drag or double-click can be turned into a caret position or a
//!   selection.
//!
//! The drawing half (turning text + caret into a scrolled, highlighted
//! row of spans) is `text_field::edit_spans`.

use std::cell::RefCell;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;

use crate::text_field;

// The caret itself (offsets, selection, edit operations, history) is
// `gitpoulpi_app::text`, shared with the desktop frontend. What stays here
// is what reads terminal key events and mouse positions.
#[allow(unused_imports)]
pub(crate) use gitpoulpi_app::text::{
    char_class, column_of, floor_boundary, line_end, line_start, next_boundary, offset_at_column,
    prev_boundary, word_left, word_right, Caret, KeyEdit, Motion, Op,
};

/// Decodes `key` into an editing operation, or `None` when the key is
/// not the editor's.
///
/// Ctrl+letter arrives two ways depending on the terminal: as
/// `Char('a')` with `CONTROL` set, or - some Windows consoles, some
/// Unix ttys - as the bare C0 control character (`\x01` for `Ctrl+A`).
/// Both spellings are recognized. `CONTROL | ALT` together is AltGr on
/// Windows (`@`, `#`, `€` on Swiss and French layouts) and types its
/// character, unless that character is an ASCII letter or digit, which
/// only a real `Ctrl+Alt` chord produces.
pub(crate) fn decode(key: &KeyEvent, multiline: bool) -> Option<Op> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let control_char = |c: char| match c {
        '\u{1}' => Some(Op::SelectAll),
        '\u{15}' => Some(Op::KillToLineStart),
        '\u{b}' => Some(Op::KillToLineEnd),
        '\u{3}' => Some(Op::Copy),
        '\u{18}' => Some(Op::Cut),
        '\u{1a}' => Some(Op::Undo),
        '\u{19}' => Some(Op::Redo),
        _ => None,
    };
    match key.code {
        KeyCode::Char(c) if c.is_control() => control_char(c),
        KeyCode::Char(c) if ctrl && alt => (!c.is_ascii_alphanumeric()).then_some(Op::Insert(c)),
        KeyCode::Char(c) if ctrl => match c.to_ascii_lowercase() {
            'a' => Some(Op::SelectAll),
            'u' => Some(Op::KillToLineStart),
            'k' => Some(Op::KillToLineEnd),
            'c' => Some(Op::Copy),
            'x' => Some(Op::Cut),
            'z' if shift => Some(Op::Redo),
            'z' => Some(Op::Undo),
            'y' => Some(Op::Redo),
            _ => None,
        },
        KeyCode::Char(_) if alt => None,
        KeyCode::Char(c) => Some(Op::Insert(c)),
        KeyCode::Backspace if !alt => Some(Op::Backspace),
        KeyCode::Delete => Some(Op::Delete),
        KeyCode::Left if ctrl => Some(Op::Move(Motion::WordLeft, shift)),
        KeyCode::Right if ctrl => Some(Op::Move(Motion::WordRight, shift)),
        KeyCode::Left => Some(Op::Move(Motion::Left, shift)),
        KeyCode::Right => Some(Op::Move(Motion::Right, shift)),
        KeyCode::Home if ctrl && multiline => Some(Op::Move(Motion::TextStart, shift)),
        KeyCode::End if ctrl && multiline => Some(Op::Move(Motion::TextEnd, shift)),
        KeyCode::Home => Some(Op::Move(Motion::LineStart, shift)),
        KeyCode::End => Some(Op::Move(Motion::LineEnd, shift)),
        KeyCode::Up if multiline => Some(Op::Move(Motion::Up, shift)),
        KeyCode::Down if multiline => Some(Op::Move(Motion::Down, shift)),
        _ => None,
    }
}

/// The terminal key half of [`Caret`]: a `KeyEvent` is decoded into an
/// [`Op`] and handed to [`Caret::run`].
pub(crate) trait CaretKeys {
    /// Whether [`CaretKeys::on_key`] would do something with `key` - asked
    /// by callers whose key dispatch is a `match` with guards, so a key
    /// the editor claims never also reaches a pane shortcut. `Ctrl+C` and
    /// `Ctrl+X` are only claimed when there is a selection to copy.
    fn claims(&self, text: &str, key: &KeyEvent, multiline: bool) -> bool;

    /// Applies one key to `text`. See the module docs for the key map.
    fn on_key(&mut self, text: &mut String, key: &KeyEvent, multiline: bool) -> KeyEdit;
}

impl CaretKeys for Caret {
    fn claims(&self, text: &str, key: &KeyEvent, multiline: bool) -> bool {
        self.claims_op(text, decode(key, multiline))
    }

    fn on_key(&mut self, text: &mut String, key: &KeyEvent, multiline: bool) -> KeyEdit {
        match decode(key, multiline) {
            Some(op) => self.run(text, op),
            None => KeyEdit::Ignored,
        }
    }
}

/// Where one field was drawn this frame: the rect its *text* occupies
/// (not its label or border), and whether it is multi-line.
#[derive(Debug, Clone, Copy)]
struct Drawn<K> {
    id: K,
    rect: Rect,
    multiline: bool,
}

/// The caret of whichever field of one surface has the keyboard, plus
/// the mouse half of editing.
///
/// A surface (the main window, a dialog) has several fields but only one is
/// ever typed into, so one [`Caret`] is enough: it belongs to `owner` and is
/// reset whenever another field takes over (entering a field starts with the
/// caret at its end and nothing selected).
#[derive(Debug, Clone)]
pub(crate) struct FieldEditor<K> {
    owner: Option<K>,
    caret: Caret,
    /// Filled by the renders of this frame, read by the next mouse event.
    drawn: RefCell<Vec<Drawn<K>>>,
    /// A left press landed in this field and the button is still down:
    /// drags extend the selection from where it went down.
    dragging: bool,
    /// Where the field being dragged in was drawn when the press landed.
    drag_rect: Option<(Rect, bool)>,
    last_click: Option<(Instant, u16, u16)>,
}

/// Where a caret sits is not part of what a form *is*: two forms holding
/// the same values are equal whatever their fields' carets, so a form
/// that derives `PartialEq` can hold an editor without its comparisons
/// (tests asserting on a form's contents) turning on caret state.
impl<K> PartialEq for FieldEditor<K> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<K> Eq for FieldEditor<K> {}

impl<K> Default for FieldEditor<K> {
    fn default() -> Self {
        Self {
            owner: None,
            caret: Caret::default(),
            drawn: RefCell::new(Vec::new()),
            dragging: false,
            drag_rect: None,
            last_click: None,
        }
    }
}

impl<K: Copy + PartialEq> FieldEditor<K> {
    /// Tells the editor which field has the keyboard now (`None`: none
    /// does). A change of owner resets the caret.
    pub(crate) fn sync(&mut self, active: Option<K>) {
        if self.owner != active {
            self.owner = active;
            self.caret = Caret::default();
            self.dragging = false;
        }
    }

    /// Puts the maintainer's caret back at the end with nothing selected --
    /// after its text was replaced wholesale (a completion accepted).
    pub(crate) fn reset(&mut self) {
        self.caret = Caret::default();
        self.dragging = false;
    }

    /// The caret of field `id`, when it is the one with the keyboard.
    pub(crate) fn caret(&self, id: K) -> Option<&Caret> {
        (self.owner == Some(id)).then_some(&self.caret)
    }

    /// The caret of field `id`, taking ownership for it first.
    pub(crate) fn caret_mut(&mut self, id: K) -> &mut Caret {
        self.sync(Some(id));
        &mut self.caret
    }

    /// Applies `key` to field `id`'s `text`.
    pub(crate) fn on_key(
        &mut self,
        id: K,
        text: &mut String,
        key: &KeyEvent,
        multiline: bool,
    ) -> KeyEdit {
        self.caret_mut(id).on_key(text, key, multiline)
    }

    /// A dialog's key for its focused field `id`: `None` when it isn't
    /// an editing key (the dialog's own handling applies), else what it
    /// did, plus the status sentence of a `Ctrl+C`/`Ctrl+X` that put the
    /// selection on the clipboard.
    pub(crate) fn dialog_key(
        &mut self,
        id: K,
        text: &mut String,
        key: &KeyEvent,
    ) -> Option<(KeyEdit, Option<String>)> {
        if !self.claims(id, text, key, false) {
            return None;
        }
        let edit = self.on_key(id, text, key, false);
        let message = match &edit {
            KeyEdit::Copy(s) | KeyEdit::Cut(s) => Some(crate::clipboard::copy_shared(
                gitpoulpi_i18n::msg::input::WHAT_SELECTION.get(),
                s,
            )),
            _ => None,
        };
        Some((edit, message))
    }

    /// A dialog's mouse event for field `id` (the focused one, when a
    /// press landed on it): a press places the caret or on a
    /// double-click selects a word, a drag selects, a release ends the
    /// drag. Returns whether the event was the editor's - a drag in
    /// progress owns every drag and the release that ends it.
    pub(crate) fn dialog_mouse(
        &mut self,
        id: Option<K>,
        text: &str,
        kind: crossterm::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        use crossterm::event::{MouseButton, MouseEventKind};
        match kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(id) = id.filter(|id| self.field_at(col, row) == Some(*id)) {
                    self.mouse_down(id, text, col, row, Instant::now());
                    return true;
                }
                false
            }
            MouseEventKind::Drag(MouseButton::Left) => self.mouse_drag(text, col, row),
            MouseEventKind::Up(MouseButton::Left) => {
                let was = self.is_dragging();
                self.mouse_up();
                was
            }
            _ => false,
        }
    }

    /// The mouse for a modal multi-line editor holding one field `id`
    /// (the settings paths/hooks editors): press, Shift+press, drag,
    /// release and the wheel. Returns whether the event was the field's.
    pub(crate) fn area_mouse(
        &mut self,
        id: K,
        text: &str,
        kind: crossterm::event::MouseEventKind,
        (col, row): (u16, u16),
        shift: bool,
    ) -> bool {
        use crossterm::event::{MouseButton, MouseEventKind};
        let delta: isize = match kind {
            MouseEventKind::ScrollDown => 3,
            MouseEventKind::ScrollUp => -3,
            _ => 0,
        };
        if delta != 0 {
            let Some(rect) = self.drawn_rect(id, col, row).map(|d| d.0) else {
                return false;
            };
            if !rect.contains((col, row).into()) {
                return false;
            }
            let top = self.caret(id).map_or(0, |c| c.top.get());
            let top = (top as isize + delta).max(0) as usize;
            self.caret_mut(id)
                .scroll_lines_to(text, top, rect.height as usize);
            return true;
        }
        if kind == MouseEventKind::Down(MouseButton::Left) {
            if self.field_at(col, row) != Some(id) {
                return false;
            }
            self.press(id, text, col, row, Instant::now(), shift);
            return true;
        }
        self.dialog_mouse(Some(id), text, kind, col, row)
    }

    /// Whether `key` is an editing key for field `id` - see
    /// [`Caret::claims`].
    pub(crate) fn claims(&self, id: K, text: &str, key: &KeyEvent, multiline: bool) -> bool {
        match self.caret(id) {
            Some(caret) => caret.claims(text, key, multiline),
            None => Caret::default().claims(text, key, multiline),
        }
    }

    /// Forgets last frame's field rects; the draw that follows records
    /// this frame's.
    pub(crate) fn begin_frame(&self) {
        self.drawn.borrow_mut().clear();
    }

    /// Records that field `id`'s text was drawn in `rect` this frame.
    pub(crate) fn drawn(&self, id: K, rect: Rect, multiline: bool) {
        if rect.width > 0 && rect.height > 0 {
            crate::pointer_map::field(rect);
            self.drawn.borrow_mut().push(Drawn {
                id,
                rect,
                multiline,
            });
        }
    }

    /// The field whose text rect holds (`col`, `row`), if any.
    pub(crate) fn field_at(&self, col: u16, row: u16) -> Option<K> {
        self.drawn
            .borrow()
            .iter()
            .find(|d| d.rect.contains((col, row).into()))
            .map(|d| d.id)
    }

    /// Where field `id` was drawn: the drawing under (`col`, `row`) when
    /// there is one (a field drawn twice - the commit summary, in the
    /// composer and in the Log's WIP row - is clicked in one of them),
    /// else its first.
    fn drawn_rect(&self, id: K, col: u16, row: u16) -> Option<(Rect, bool)> {
        let drawn = self.drawn.borrow();
        drawn
            .iter()
            .find(|d| d.id == id && d.rect.contains((col, row).into()))
            .or_else(|| drawn.iter().find(|d| d.id == id))
            .map(|d| (d.rect, d.multiline))
    }

    /// The byte offset a pointer at (`col`, `row`) points at in field
    /// `id` drawn in `rect`, clamped into it so a drag past the edge
    /// selects to the edge.
    fn offset_in(
        &self,
        id: K,
        (rect, multiline): (Rect, bool),
        text: &str,
        col: u16,
        row: u16,
    ) -> usize {
        let col = col.min(rect.right().saturating_sub(1)).max(rect.x) - rect.x;
        let row = row.min(rect.bottom().saturating_sub(1)).max(rect.y) - rect.y;
        let caret = self.caret(id).cloned().unwrap_or_default();
        text_field::offset_at(
            text,
            &caret,
            rect.width as usize,
            multiline,
            col as usize,
            row as usize,
        )
    }

    /// A left press at (`col`, `row`) in field `id` (which the caller
    /// has already given the keyboard): places the caret there, or on a
    /// double-click selects the word under it, and starts a drag.
    pub(crate) fn mouse_down(&mut self, id: K, text: &str, col: u16, row: u16, now: Instant) {
        self.press(id, text, col, row, now, false);
    }

    /// [`Self::mouse_down`], where `extend` (Shift held) grows the
    /// selection from the caret to the pointer instead of moving it.
    pub(crate) fn press(
        &mut self,
        id: K,
        text: &str,
        col: u16,
        row: u16,
        now: Instant,
        extend: bool,
    ) {
        let Some(drawn) = self.drawn_rect(id, col, row) else {
            return;
        };
        let at = self.offset_in(id, drawn, text, col, row);
        self.drag_rect = Some(drawn);
        let double = self.last_click.is_some_and(|(t, c, r)| {
            now.duration_since(t) < crate::DOUBLE_CLICK_WINDOW && c == col && r == row
        });
        self.last_click = Some((now, col, row));
        let caret = self.caret_mut(id);
        if extend {
            caret.set_cursor(text, at, true);
            self.dragging = true;
        } else if double {
            caret.select_word_at(text, at);
            self.dragging = false;
        } else {
            caret.set_cursor(text, at, false);
            self.dragging = true;
        }
    }

    /// A drag with the left button: extends the selection when a press
    /// started in the owning field. Returns whether it did.
    pub(crate) fn mouse_drag(&mut self, text: &str, col: u16, row: u16) -> bool {
        let Some(id) = self.owner.filter(|_| self.dragging) else {
            return false;
        };
        let Some(drawn) = self.drag_rect else {
            return false;
        };
        let at = self.offset_in(id, drawn, text, col, row);
        self.caret.set_cursor(text, at, true);
        true
    }

    /// Whether a press in a field is still being dragged.
    pub(crate) fn is_dragging(&self) -> bool {
        self.dragging && self.owner.is_some()
    }

    /// The owning field, when a drag is in progress.
    pub(crate) fn dragging_field(&self) -> Option<K> {
        self.owner.filter(|_| self.dragging)
    }

    pub(crate) fn mouse_up(&mut self) {
        self.dragging = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    fn ctrl(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }
    fn shift(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::SHIFT)
    }

    /// Types `keys` into a fresh field holding `start` (caret at its
    /// end) and returns the text and the caret.
    fn run(start: &str, multiline: bool, keys: &[KeyEvent]) -> (String, Caret) {
        let mut text = start.to_string();
        let mut caret = Caret::default();
        for k in keys {
            caret.on_key(&mut text, k, multiline);
        }
        (text, caret)
    }

    #[test]
    fn typing_appends_at_the_caret_which_starts_at_the_end() {
        let (text, caret) = run("ab", false, &[key(KeyCode::Char('c'))]);
        assert_eq!(text, "abc");
        assert_eq!(caret.cursor(&text), 3);
        let (text, _) = run("ac", false, &[key(KeyCode::Left), key(KeyCode::Char('b'))]);
        assert_eq!(text, "abc");
    }

    #[test]
    fn ctrl_a_selects_all_then_typing_replaces_it() {
        let (text, caret) = run("hello", false, &[ctrl('a')]);
        assert_eq!(caret.selection(&text), Some((0, 5)));
        let (text, caret) = run("hello", false, &[ctrl('a'), key(KeyCode::Char('x'))]);
        assert_eq!(text, "x");
        assert_eq!(caret.selection(&text), None);
    }

    #[test]
    fn backspace_and_delete_remove_the_selection_whole() {
        let (text, _) = run("hello", false, &[ctrl('a'), key(KeyCode::Backspace)]);
        assert_eq!(text, "");
        let (text, _) = run(
            "hello",
            false,
            &[
                key(KeyCode::Home),
                shift(KeyCode::Right),
                shift(KeyCode::Right),
                key(KeyCode::Delete),
            ],
        );
        assert_eq!(text, "llo");
    }

    #[test]
    fn shift_arrows_extend_and_shrink_the_selection() {
        let (text, caret) = run(
            "abcdef",
            false,
            &[
                shift(KeyCode::Left),
                shift(KeyCode::Left),
                shift(KeyCode::Left),
                shift(KeyCode::Right),
            ],
        );
        assert_eq!(caret.selected_text(&text), Some("ef"));
        // A plain arrow collapses the selection to its side.
        let mut caret = caret;
        let mut t = text.clone();
        caret.on_key(&mut t, &key(KeyCode::Left), false);
        assert_eq!(caret.selection(&t), None);
        assert_eq!(caret.cursor(&t), 4);
    }

    #[test]
    fn the_control_character_spellings_of_ctrl_letters_work_too() {
        let (text, caret) = run("hello", false, &[key(KeyCode::Char('\u{1}'))]);
        assert_eq!(caret.selection(&text), Some((0, 5)));
        let (text, _) = run("hello", false, &[key(KeyCode::Char('\u{15}'))]);
        assert_eq!(text, "");
    }

    #[test]
    fn ctrl_u_and_ctrl_k_on_a_single_line() {
        let mid = [key(KeyCode::Left), key(KeyCode::Left)];
        let (text, caret) = run("abcdef", false, &[&mid[..], &[ctrl('u')]].concat());
        assert_eq!(text, "ef");
        assert_eq!(caret.cursor(&text), 0);
        let (text, caret) = run("abcdef", false, &[&mid[..], &[ctrl('k')]].concat());
        assert_eq!(text, "abcd");
        assert_eq!(caret.cursor(&text), 4);
    }

    #[test]
    fn ctrl_u_and_ctrl_k_stop_at_the_current_lines_ends() {
        // Caret in the middle line, after "mid".
        let keys = [key(KeyCode::Up), key(KeyCode::End)];
        let (text, _) = run(
            "one\nmiddle\nlast",
            true,
            &[
                &keys[..],
                &[key(KeyCode::Left)],
                &[key(KeyCode::Left)],
                &[key(KeyCode::Left)],
                &[ctrl('u')],
            ]
            .concat(),
        );
        assert_eq!(text, "one\ndle\nlast");
        let (text, _) = run(
            "one\nmiddle\nlast",
            true,
            &[
                &keys[..],
                &[key(KeyCode::Left)],
                &[key(KeyCode::Left)],
                &[key(KeyCode::Left)],
                &[ctrl('k')],
            ]
            .concat(),
        );
        assert_eq!(text, "one\nmid\nlast");
        // At a line's end, Ctrl+K has nothing to delete -- it does not
        // join the next line.
        let (text, _) = run(
            "one\nmiddle\nlast",
            true,
            &[&keys[..], &[ctrl('k')]].concat(),
        );
        assert_eq!(text, "one\nmiddle\nlast");
    }

    #[test]
    fn up_and_down_keep_the_column_in_multiline_and_are_ignored_in_single_line() {
        let (text, caret) = run("abcdef\nxy", true, &[key(KeyCode::Up)]);
        assert_eq!(caret.cursor(&text), 2);
        let mut t = "abc".to_string();
        assert_eq!(
            Caret::default().on_key(&mut t, &key(KeyCode::Up), false),
            KeyEdit::Ignored
        );
    }

    #[test]
    fn enter_esc_and_tab_are_never_claimed() {
        let caret = Caret::default();
        for code in [KeyCode::Enter, KeyCode::Esc, KeyCode::Tab] {
            assert!(!caret.claims("x", &key(code), true));
        }
        assert!(caret.claims("x", &key(KeyCode::Char('q')), false));
        assert!(!caret.claims("x", &ctrl('p'), false));
    }

    #[test]
    fn copy_and_cut_need_a_selection() {
        let caret = Caret::default();
        assert!(!caret.claims("abc", &ctrl('c'), false));
        let mut text = "abc".to_string();
        let mut caret = Caret::default();
        caret.select_all(&text);
        assert!(caret.claims(&text, &ctrl('c'), false));
        assert_eq!(
            caret.on_key(&mut text, &ctrl('c'), false),
            KeyEdit::Copy("abc".into())
        );
        assert_eq!(text, "abc");
        assert_eq!(
            caret.on_key(&mut text, &ctrl('x'), false),
            KeyEdit::Cut("abc".into())
        );
        assert_eq!(text, "");
    }

    #[test]
    fn altgr_characters_type_but_real_ctrl_alt_chords_do_not() {
        let altgr = KeyModifiers::CONTROL | KeyModifiers::ALT;
        let (text, _) = run("a", false, &[KeyEvent::new(KeyCode::Char('@'), altgr)]);
        assert_eq!(text, "a@");
        let normalized = |c| {
            let crossterm::event::Event::Key(k) = crate::keymap::normalize_event(
                crossterm::event::Event::Key(KeyEvent::new(KeyCode::Char(c), altgr)),
            ) else {
                unreachable!()
            };
            k
        };
        let (text, _) = run(
            "a",
            false,
            &[normalized('@'), normalized('{'), normalized('\\')],
        );
        assert_eq!(text, "a@{\\");
        let (text, _) = run("a", false, &[KeyEvent::new(KeyCode::Char('u'), altgr)]);
        assert_eq!(text, "a");
    }

    #[test]
    fn double_click_word_selection() {
        let text = "fix: the parser_bug  now";
        let mut caret = Caret::default();
        caret.select_word_at(text, 11); // inside "parser_bug"
        assert_eq!(caret.selected_text(text), Some("parser_bug"));
        caret.select_word_at(text, 3); // the colon
        assert_eq!(caret.selected_text(text), Some(":"));
        caret.select_word_at(text, 20); // the double space
        assert_eq!(caret.selected_text(text), Some("  "));
        caret.select_word_at(text, text.len()); // past the end: last word
        assert_eq!(caret.selected_text(text), Some("now"));
        // Multi-line: a word never runs across a newline.
        let text = "ab\ncd";
        caret.select_word_at(text, 3);
        assert_eq!(caret.selected_text(text), Some("cd"));
        caret.select_word_at(text, 2);
        assert_eq!(caret.selected_text(text), Some("ab"));
    }

    #[test]
    fn word_jumps() {
        let text = "foo bar.baz";
        assert_eq!(word_left(text, text.len()), 8);
        assert_eq!(word_left(text, 4), 0);
        assert_eq!(word_right(text, 0), 4);
        assert_eq!(word_right(text, 4), 7);
    }

    #[test]
    fn wide_and_multibyte_chars_are_edited_as_whole_chars() {
        let (text, _) = run(
            "日本語",
            false,
            &[key(KeyCode::Left), key(KeyCode::Backspace)],
        );
        assert_eq!(text, "日語");
        let (text, _) = run("🚀x", false, &[key(KeyCode::Home), key(KeyCode::Delete)]);
        assert_eq!(text, "x");
    }

    #[test]
    fn a_caret_left_behind_by_an_outside_change_is_clamped() {
        let mut caret = Caret::default();
        let mut text = "abcdef".to_string();
        caret.on_key(&mut text, &key(KeyCode::Left), false);
        let mut shorter = "ab".to_string();
        assert_eq!(caret.cursor(&shorter), 2);
        caret.on_key(&mut shorter, &key(KeyCode::Char('c')), false);
        assert_eq!(shorter, "abc");
    }

    #[test]
    fn the_editor_resets_the_caret_when_another_field_takes_over() {
        let mut editor = FieldEditor::<u8>::default();
        let mut text = "abc".to_string();
        editor.on_key(1, &mut text, &ctrl('a'), false);
        assert!(editor.caret(1).unwrap().selection(&text).is_some());
        editor.sync(Some(2));
        assert!(editor.caret(1).is_none());
        editor.sync(Some(1));
        assert!(editor.caret(1).unwrap().selection(&text).is_none());
    }

    #[test]
    fn click_drag_and_double_click_through_the_drawn_rect() {
        let mut editor = FieldEditor::<u8>::default();
        let text = "hello world";
        editor.drawn(7, Rect::new(10, 3, 30, 1), false);
        editor.sync(Some(7));
        let t0 = Instant::now();
        editor.mouse_down(7, text, 12, 3, t0);
        assert_eq!(editor.caret(7).unwrap().cursor(text), 2);
        assert!(editor.mouse_drag(text, 16, 3));
        assert_eq!(editor.caret(7).unwrap().selected_text(text), Some("llo "));
        // Dragging past the right edge selects to the end.
        editor.mouse_drag(text, 200, 3);
        assert_eq!(
            editor.caret(7).unwrap().selected_text(text),
            Some("llo world")
        );
        editor.mouse_up();
        assert!(!editor.mouse_drag(text, 12, 3));
        // Double-click on "world".
        let t1 = t0 + std::time::Duration::from_secs(5);
        editor.mouse_down(7, text, 18, 3, t1);
        editor.mouse_up();
        editor.mouse_down(7, text, 18, 3, t1 + std::time::Duration::from_millis(100));
        assert_eq!(editor.caret(7).unwrap().selected_text(text), Some("world"));
    }

    /// The modal multi-line editors: wide glyphs map to whole chars, Shift
    /// extends, a drag selects across lines, the wheel scrolls.
    #[test]
    fn area_mouse_maps_wide_glyphs_extends_with_shift_and_scrolls() {
        use crossterm::event::{MouseButton as B, MouseEventKind as K};
        let mut editor = FieldEditor::<()>::default();
        let text = "日本語abc\nsecond\nthird\nfourth\nfifth";
        editor.caret_mut(());
        editor.drawn((), Rect::new(10, 2, 20, 2), true);
        // Column 12 is the second wide glyph (cells 12-13): before 本.
        assert!(editor.area_mouse((), text, K::Down(B::Left), (12, 2), false));
        assert_eq!(editor.caret(()).unwrap().cursor(text), "日".len());
        editor.area_mouse((), text, K::Up(B::Left), (12, 2), false);
        // The second half of 本 still lands before 本.
        editor.area_mouse((), text, K::Down(B::Left), (13, 2), false);
        assert_eq!(editor.caret(()).unwrap().cursor(text), "日".len());
        editor.area_mouse((), text, K::Up(B::Left), (13, 2), false);
        // Shift+click on `a` (cell 16) extends from the caret.
        editor.area_mouse((), text, K::Down(B::Left), (16, 2), true);
        assert_eq!(editor.caret(()).unwrap().selected_text(text), Some("本語"));
        // A drag onto the next line selects across the newline.
        editor.area_mouse((), text, K::Up(B::Left), (16, 2), true);
        editor.area_mouse((), text, K::Down(B::Left), (10, 2), false);
        assert!(editor.area_mouse((), text, K::Drag(B::Left), (12, 3), false));
        assert_eq!(
            editor.caret(()).unwrap().selected_text(text),
            Some("日本語abc\nse")
        );
        editor.area_mouse((), text, K::Up(B::Left), (12, 3), false);
        // A press outside the field is not the editor's.
        assert!(!editor.area_mouse((), text, K::Down(B::Left), (2, 0), false));
        // The wheel scrolls the view, only over the field.
        assert!(editor.area_mouse((), text, K::ScrollDown, (12, 2), false));
        assert_eq!(editor.caret(()).unwrap().top.get(), 3);
        assert!(!editor.area_mouse((), text, K::ScrollDown, (2, 0), false));
        assert!(editor.area_mouse((), text, K::ScrollUp, (12, 2), false));
        assert_eq!(editor.caret(()).unwrap().top.get(), 0);
    }

    // ---- the edit history (Ctrl+Z / Ctrl+Y) ----

    fn typed(caret: &mut Caret, text: &mut String, s: &str) {
        for c in s.chars() {
            caret.on_key(text, &key(KeyCode::Char(c)), true);
        }
    }

    /// Typing groups per word: "hello world" is three entries (hello, the
    /// space, world), so three Ctrl+Z walk back to nothing.
    #[test]
    fn typing_is_undone_a_word_at_a_time_and_redone() {
        let mut caret = Caret::default();
        let mut text = String::new();
        typed(&mut caret, &mut text, "hello world");
        assert_eq!(caret.on_key(&mut text, &ctrl('z'), true), KeyEdit::Edited);
        assert_eq!(text, "hello ");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "hello");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "");
        assert_eq!(caret.on_key(&mut text, &ctrl('z'), true), KeyEdit::Moved);
        // Ctrl+Y and Ctrl+Shift+Z both redo.
        caret.on_key(&mut text, &ctrl('y'), true);
        assert_eq!(text, "hello");
        let shifted = KeyEvent::new(
            KeyCode::Char('Z'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );
        caret.on_key(&mut text, &shifted, true);
        assert_eq!(text, "hello ");
        assert_eq!(caret.cursor(&text), 6);
        // The bare control characters some terminals send.
        caret.on_key(&mut text, &key(KeyCode::Char('\u{1a}')), true);
        assert_eq!(text, "hello");
        caret.on_key(&mut text, &key(KeyCode::Char('\u{19}')), true);
        assert_eq!(text, "hello ");
    }

    /// A new edit after an undo forgets the redo branch.
    #[test]
    fn a_new_edit_clears_the_redo_branch() {
        let mut caret = Caret::default();
        let mut text = String::new();
        typed(&mut caret, &mut text, "ab");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert!(caret.can_redo(&text));
        typed(&mut caret, &mut text, "x");
        assert!(!caret.can_redo(&text));
        assert_eq!(text, "x");
    }

    #[test]
    fn a_pause_starts_a_new_entry_and_backspaces_group() {
        let mut caret = Caret::default();
        caret.history.set_pause(std::time::Duration::ZERO);
        let mut text = String::new();
        typed(&mut caret, &mut text, "abc");
        assert_eq!(caret.history.len(), 3, "no grouping across pauses");
        let mut caret = Caret::default();
        let mut text = "abcdef".to_string();
        for _ in 0..3 {
            caret.on_key(&mut text, &key(KeyCode::Backspace), true);
        }
        assert_eq!(text, "abc");
        assert_eq!(caret.history.len(), 1, "backspaces are one entry");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "abcdef");
    }

    /// Paste, cut and kill are one entry each and restore caret and selection.
    #[test]
    fn paste_cut_and_kill_are_single_entries() {
        let mut caret = Caret::default();
        let mut text = "one two".to_string();
        caret.insert(&mut text, " three four");
        assert_eq!(text, "one two three four");
        caret.on_key(&mut text, &ctrl('a'), true);
        let cut = caret.on_key(&mut text, &ctrl('x'), true);
        assert!(matches!(cut, KeyEdit::Cut(_)));
        assert_eq!(text, "");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "one two three four");
        assert_eq!(
            caret.selection(&text),
            Some((0, text.len())),
            "the selection comes back with the text"
        );
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "one two");
        caret.on_key(&mut text, &ctrl('u'), true);
        assert_eq!(text, "");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "one two");
    }

    #[test]
    fn the_history_is_bounded() {
        let mut caret = Caret::default();
        caret.history.set_pause(std::time::Duration::ZERO);
        let mut text = String::new();
        let n = crate::text_history::MAX_ENTRIES;
        typed(&mut caret, &mut text, &"x".repeat(n + 50));
        assert_eq!(caret.history.len(), n);
        // The oldest are gone: the walk back stops short of empty.
        let mut steps = 0;
        while caret.undo(&mut text) {
            steps += 1;
        }
        assert_eq!(steps, n);
        assert_eq!(text.len(), 50);
    }

    /// A field whose text was replaced from outside has no history: an
    /// entry would bring back text it no longer holds.
    #[test]
    fn the_history_is_cleared_when_the_text_is_replaced_from_outside() {
        let mut caret = Caret::default();
        let mut text = String::new();
        typed(&mut caret, &mut text, "abc");
        assert!(caret.can_undo(&text));
        text = "from outside".to_string();
        assert!(!caret.can_undo(&text));
        assert_eq!(caret.on_key(&mut text, &ctrl('z'), true), KeyEdit::Moved);
        assert_eq!(text, "from outside");
        // New typing starts a fresh history.
        typed(&mut caret, &mut text, "!");
        caret.on_key(&mut text, &ctrl('z'), true);
        assert_eq!(text, "from outside");
        // The owner's reset drops it too.
        let mut editor = FieldEditor::<u8>::default();
        let mut text = String::new();
        editor.on_key(1, &mut text, &key(KeyCode::Char('q')), false);
        editor.reset();
        assert_eq!(
            editor.on_key(1, &mut text, &ctrl('z'), false),
            KeyEdit::Moved
        );
        assert_eq!(text, "q");
    }

    /// The editor claims the chords, so a focused field never lets them reach
    /// the global git undo.
    #[test]
    fn a_field_claims_ctrl_z_and_ctrl_y() {
        let caret = Caret::default();
        assert!(caret.claims("", &ctrl('z'), false));
        assert!(caret.claims("", &ctrl('y'), false));
    }
}
