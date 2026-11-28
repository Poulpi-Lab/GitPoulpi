//! Edit in Working Directory: a built-in editor for a
//! working-tree file, in the central column.
//!
//! Opened from a working-tree file's Diff view or File View (the bar's
//! `Edit` button, the `e` key, or the palette), it takes the central
//! column the way File View does (`CentralView::Editor`), over the view
//! it was opened from, which is kept underneath: closing the editor goes
//! back to it. It collapses the Refs pane like the other central views.
//!
//! The pieces:
//!
//! - `editor_buffer::Doc` -- the text, the caret,
//!   undo/redo and the byte-exact load/save;
//! - `editor_highlight::Highlight` -- syntect, visible lines only,
//!   incremental;
//! - this module -- the bar (Save / Undo / Redo / Find and a status
//!   line), the bordered body with a line-number gutter, the caret, the
//!   selection, scrollbars, the mouse, the
//!   Save / Discard / Cancel prompt, `Ctrl+F` find, and the atomic save.
//!
//! Saving writes a temp file beside the target and renames it over, so a
//! crash mid-write never leaves half a file. The event loop then asks the
//! daemon for a refresh, and the Status pane, the WIP row and a diff
//! underneath follow.

mod render;
mod status_menu;
mod vocab;

pub(crate) use status_menu::StatusMenu;
use status_menu::{OpenMenu, Then};

use super::*;
use gitpoulpi_i18n::msg::editor as msg;

use std::time::SystemTime;

use crate::clipboard::Clipboard;
use crate::editor_buffer::{char_cells_in, line_width_in, Doc, Indent};
use crate::editor_highlight::Highlight;
use crate::keymap::{Action, Ctx};
use crate::scrollbar::TrackPress;
use crate::text_edit::{Caret, KeyEdit};
use crate::text_field::EditLook;

/// How long one frame may spend highlighting before drawing the rest
/// plain (they fill in over the next frames).
const HIGHLIGHT_BUDGET: Duration = Duration::from_millis(20);
/// Lines per wheel notch - the File View's own step.
const WHEEL_LINES: usize = 3;
const FIND_FIELD_WIDTH: u16 = 24;

/// Everything clickable the editor draws. Carried by
/// `HitTarget::Editor` so hover and clicks go through the window's one
/// hit-test list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum EditorHit {
    Save,
    Undo,
    Redo,
    Find,
    /// `ui.minimap`, the Diff view's `Map` toggle: on (accent) while it is on.
    ToggleMinimap,
    /// The bar's `[x] Follow selection` checkbox (`editor.follow_selection`).
    FollowSelection,
    /// The status bar's chips; each opens its menu (`StatusMenu`).
    EncodingChip,
    EolChip,
    IndentChip,
    FindPrev,
    FindNext,
    Close,
    PromptSave,
    PromptDiscard,
    PromptCancel,
    /// The scrollbar thumbs - hover only; the body's mouse handler
    /// drags them.
    VThumb,
    HThumb,
    /// The minimap - the body's mouse handler jumps; here for the pointer
    /// shape only.
    Minimap,
}

const BAR_BUTTONS: &[(gitpoulpi_i18n::M, EditorHit)] = &[
    (msg::SAVE, EditorHit::Save),
    (msg::UNDO, EditorHit::Undo),
    (msg::REDO, EditorHit::Redo),
    (msg::FIND, EditorHit::Find),
    (
        gitpoulpi_i18n::msg::diff_view::B_MAP,
        EditorHit::ToggleMinimap,
    ),
];

/// What a key, a click or a paste asks of the window.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct EditorOutcome {
    /// For the status line.
    pub(crate) message: Option<String>,
    /// The file was written: refresh the status and anything showing it.
    pub(crate) saved: bool,
    /// Close the editor.
    pub(crate) close: bool,
    /// Flip `ui.minimap` (the event loop holds the config path).
    pub(crate) toggle_minimap: bool,
    /// Flip `editor.follow_selection` (the event loop holds the config path).
    pub(crate) toggle_follow: bool,
}

impl EditorOutcome {
    fn say(message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            ..Self::default()
        }
    }
    fn close() -> Self {
        Self {
            close: true,
            ..Self::default()
        }
    }
    fn toggle_follow() -> Self {
        Self {
            toggle_follow: true,
            ..Self::default()
        }
    }
    fn toggle_minimap() -> Self {
        Self {
            toggle_minimap: true,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drag {
    Text,
    VThumb(i32),
    HThumb(i32),
    Minimap,
}

/// The `Ctrl+F` bar.
#[derive(Debug)]
pub(crate) struct Find {
    pub(crate) query: String,
    pub(crate) caret: Caret,
    /// `(query, doc version)` -> match starts, so the count is not
    /// recomputed every frame.
    cache: Option<(String, u64, Vec<usize>)>,
}

/// What a key typed into the find bar asks for ([`find_on_key`]).
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FindKey {
    /// Esc: close the bar.
    Close,
    /// Select the next (`true`) or previous match.
    Step(bool),
    /// The save chord (the editor saves; the merge view has its own).
    Save,
    /// The query changed: select its first match from the selection.
    Typed(String),
    /// Copy (`false`) or cut (`true`) this part of the query.
    Clip(String, bool),
    Nothing,
}

impl Find {
    /// Opens the bar over `slot` (or re-aims it), seeded with a one-line
    /// selection of `doc`, the query selected.
    pub(crate) fn open(slot: &mut Option<Find>, doc: &Doc) {
        let seed = doc
            .selection()
            .map(|(a, b)| &doc.text()[a..b])
            .filter(|s| !s.contains('\n'))
            .map(str::to_string);
        let find = slot.get_or_insert_with(|| Find {
            query: String::new(),
            caret: Caret::default(),
            cache: None,
        });
        if let Some(seed) = seed {
            find.query = seed;
        }
        find.caret.select_all(&find.query);
    }
}

/// A key typed while the find bar is up: which bar action it is, applied
/// to `find` when it only edits the query. Shared by the editor and the
/// merge editor's Output.
pub(crate) fn find_on_key(find: &mut Find, key: &KeyEvent) -> FindKey {
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let km = crate::keymap::current();
    match key.code {
        KeyCode::Esc => return FindKey::Close,
        KeyCode::Enter => return FindKey::Step(!shift),
        KeyCode::Up => return FindKey::Step(false),
        KeyCode::Down => return FindKey::Step(true),
        _ if km.is(Action::EditorFindNext, key) => return FindKey::Step(!shift),
        _ if km.is(Action::EditorFind, key) => return FindKey::Step(true),
        _ if km.is(Action::EditorSave, key) => return FindKey::Save,
        _ => {}
    }
    match find.caret.on_key(&mut find.query, key, false) {
        KeyEdit::Edited => FindKey::Typed(find.query.clone()),
        KeyEdit::Copy(s) => FindKey::Clip(s, false),
        KeyEdit::Cut(s) => FindKey::Clip(s, true),
        _ => FindKey::Nothing,
    }
}

/// Selects the match of `query` after (or before) `from`; whether there
/// was one.
pub(crate) fn select_match_in(doc: &mut Doc, query: &str, from: usize, forward: bool) -> bool {
    match doc.find(query, from, forward) {
        Some((a, b)) => {
            doc.select_range(a, b);
            true
        }
        None => false,
    }
}

/// F3 / Enter / the arrows of the find bar: the next match from the
/// selection (or the caret). `None` when the query is empty.
pub(crate) fn find_step_in(doc: &mut Doc, query: &str, forward: bool) -> Option<bool> {
    if query.is_empty() {
        return None;
    }
    let from = match (doc.selection(), forward) {
        (Some((_, b)), true) => b,
        (Some((a, _)), false) => a,
        (None, _) => doc.cursor(),
    };
    Some(select_match_in(doc, query, from, forward))
}

/// The query was typed in: the first match from the selection's start.
pub(crate) fn find_typed_in(doc: &mut Doc, query: &str) -> Option<bool> {
    if query.is_empty() {
        return None;
    }
    let from = doc.selection().map_or(doc.cursor(), |(a, _)| a);
    Some(select_match_in(doc, query, from, true))
}

/// A left press in a text area at byte offset `at` (cell `col`, `row`):
/// a double click selects the word, a shift click extends the selection.
/// Returns whether a text drag starts.
pub(crate) fn text_press(
    last_click: &mut Option<(Instant, u16, u16)>,
    doc: &mut Doc,
    at: usize,
    (col, row): (u16, u16),
    shift: bool,
) -> bool {
    let now = Instant::now();
    let double = last_click.is_some_and(|(t, c, r)| {
        now.duration_since(t) < DOUBLE_CLICK_WINDOW && c == col && r == row
    });
    *last_click = Some((now, col, row));
    if double {
        doc.select_word_at(at);
        false
    } else {
        doc.set_cursor(at, shift);
        true
    }
}

/// Modification time and size, to notice the file changing on disk
/// under the editor.
type Stamp = (Option<SystemTime>, u64);

fn stamp(path: &Path) -> Option<Stamp> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok(), meta.len()))
}

/// The editor on one working-tree file.
pub(crate) struct Editor {
    /// Repository-relative, as the diff and the File View name it.
    pub(crate) path: PathBuf,
    abs: PathBuf,
    pub(crate) doc: Doc,
    hl: Highlight,
    /// The last frame left highlighting behind (budget spent before the
    /// visible lines were done): the loop must draw again, input or not.
    hl_pending: bool,
    /// First line and first display column drawn.
    top: usize,
    left: usize,
    /// Scroll so the caret is visible on the next render (after a key;
    /// not after the wheel, which moves the view and not the caret).
    follow: bool,
    /// The Save / Discard / Cancel prompt is up.
    pub(crate) prompt: bool,
    find: Option<Find>,
    stamp: Option<Stamp>,
    /// The file changed on disk and the user was told; the next save
    /// overwrites it.
    overwrite_armed: bool,
    longest: (u64, usize),
    /// The bordered body and the text area (inside the gutter) as last
    /// drawn - what the mouse is mapped through.
    body: Rect,
    text_area: Rect,
    drag: Option<Drag>,
    last_click: Option<(Instant, u16, u16)>,
    /// `(doc version, selected text)` -> the lines holding it, so the
    /// scrollbar ruler does not rescan the file every frame.
    ruler_cache: Option<(u64, String, Vec<usize>)>,
    /// The selected text as of the last frame, tinted where else it occurs.
    occ_needle: String,
    /// The minimap (`ui.minimap`) and where it was last drawn (`None` while
    /// it is off or the window is too narrow); `minimap_pad` is the columns
    /// it took from the text.
    minimap: minimap::Minimap,
    minimap_rect: Option<Rect>,
    minimap_pad: u16,
    /// The status menu that is open, if any.
    menu: Option<OpenMenu>,
    /// Where the three chips (`status_menu::CHIPS`) were last drawn, and the
    /// bar they are on: what a menu hangs from.
    chips: [Option<Rect>; 3],
    bar: Rect,
    /// What the prompt does once answered.
    then: Then,
    /// How the Follow-selection prompt was answered, until the window
    /// takes it (`take_follow_answer`).
    follow_answer: Option<FollowAnswer>,
}

/// The answer to the prompt raised when another file was selected while
/// the editor has unsaved changes and `editor.follow_selection` is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FollowAnswer {
    /// Saved or discarded: open the selected file.
    Switch,
    /// Canceled: stay on this file.
    Stay,
}

impl Editor {
    /// Opens `rel` under the working tree `root`, or says why not.
    pub(crate) fn open(root: &Path, rel: &Path) -> Result<Editor, String> {
        let shown = rel.display().to_string();
        let abs = root.join(rel);
        let meta = std::fs::metadata(&abs).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => msg::not_in_tree(&shown),
            _ => msg::cannot_open(&shown, e),
        })?;
        if meta.is_dir() {
            return Err(msg::is_folder(&shown));
        }
        if meta.len() > editor_buffer::MAX_EDIT_BYTES {
            return Err(editor_buffer::LoadRefusal::TooLarge(meta.len()).message(&shown));
        }
        let bytes = std::fs::read(&abs).map_err(|e| msg::cannot_read(&shown, e))?;
        let doc = Doc::decode(&bytes).map_err(|why| why.message(&shown))?;
        let hl = Highlight::for_file(rel, doc.line(0));
        Ok(Editor {
            path: rel.to_path_buf(),
            stamp: stamp(&abs),
            abs,
            doc,
            hl,
            hl_pending: false,
            top: 0,
            left: 0,
            follow: true,
            prompt: false,
            find: None,
            overwrite_armed: false,
            longest: (u64::MAX, 0),
            body: Rect::default(),
            text_area: Rect::default(),
            drag: None,
            last_click: None,
            ruler_cache: None,
            occ_needle: String::new(),
            minimap: minimap::Minimap::default(),
            minimap_rect: None,
            minimap_pad: 0,
            menu: None,
            chips: [None; 3],
            bar: Rect::default(),
            then: Then::Close,
            follow_answer: None,
        })
    }

    /// The loop should keep drawing: highlighting is still catching up
    /// (it advances only inside `render`, a time budget at a time).
    pub(crate) fn highlight_pending(&self) -> bool {
        self.hl_pending
    }

    fn shown(&self) -> String {
        self.path.display().to_string()
    }

    /// Keys the editor leaves to the window: the `Ctrl` chords it has no
    /// use for (the palette's `Ctrl+P`, the `Ctrl+N` prefix...). While
    /// the prompt or the find bar is up it keeps every key.
    pub(crate) fn passes_through(&self, key: &KeyEvent) -> bool {
        if self.prompt || self.find.is_some() || self.menu.is_some() {
            return false;
        }
        // The prefixes and the palette are the window's whatever they are
        // bound to (a rebound prefix may sit on one of the editor's own
        // letters).
        let km = crate::keymap::current();
        if km.is(crate::keymap::Action::GlobalGitPrefix, key)
            || km.is(crate::keymap::Action::GlobalNavPrefix, key)
            || km.is(crate::keymap::Action::GlobalPalette, key)
        {
            return true;
        }
        // Its own chords: the bound editor actions, and the text-editing
        // ones that stay fixed.
        if km.action_in(Ctx::Editor, key).is_some() {
            return false;
        }
        ctrl_letter(key).is_some_and(|c| !"acukx".contains(c))
    }

    /// Whether a mouse event at (`col`, `row`) is the body's: inside its
    /// border, below the top border (which carries the `✕ Esc`).
    pub(crate) fn owns_point(&self, col: u16, row: u16) -> bool {
        self.body.contains((col, row).into()) && row > self.body.y
    }

    pub(crate) fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    /// Writes the file: a temp file beside it, then renamed over it.
    pub(crate) fn save(&mut self) -> EditorOutcome {
        let shown = self.shown();
        if !self.doc.is_dirty() && !self.overwrite_armed {
            return EditorOutcome::say(msg::no_changes(&shown));
        }
        if !self.overwrite_armed && self.stamp.is_some() && stamp(&self.abs) != self.stamp {
            self.overwrite_armed = true;
            return EditorOutcome::say(msg::changed_on_disk(&shown));
        }
        let bytes = match self.doc.try_encode() {
            Ok(bytes) => bytes,
            Err(why) => {
                return EditorOutcome::say(msg::save_unencodable(
                    self.doc.charset().name(),
                    why.count,
                    format!("{:?}", why.ch),
                    why.line,
                ));
            }
        };
        match write_atomic(&self.abs, &bytes) {
            Ok(()) => {
                self.doc.mark_saved();
                self.stamp = stamp(&self.abs);
                self.overwrite_armed = false;
                EditorOutcome {
                    message: Some(msg::saved(&shown)),
                    saved: true,
                    ..EditorOutcome::default()
                }
            }
            Err(e) => EditorOutcome::say(crate::activity_log::report_problem_clear(
                crate::activity_log::ProblemSource::Save,
                &msg::save_failed(&shown, e),
            )),
        }
    }

    /// Esc / `✕`: closes, or asks first when there are unsaved changes.
    pub(crate) fn request_close(&mut self) -> EditorOutcome {
        if self.doc.is_dirty() {
            self.prompt = true;
            self.then = Then::Close;
            EditorOutcome::default()
        } else {
            EditorOutcome::close()
        }
    }

    /// Another file was selected with follow-selection on: asks Save /
    /// Discard / Cancel first when there are unsaved changes (`true`, the
    /// answer comes through `take_follow_answer`); `false` when clean.
    pub(crate) fn ask_follow(&mut self) -> bool {
        if self.doc.is_dirty() {
            self.prompt = true;
            self.then = Then::Follow;
            true
        } else {
            false
        }
    }

    /// Whether the follow prompt is up or its answer is waiting to be taken.
    pub(crate) fn follow_pending(&self) -> bool {
        (self.then == Then::Follow && self.prompt) || self.follow_answer.is_some()
    }

    pub(crate) fn take_follow_answer(&mut self) -> Option<FollowAnswer> {
        self.follow_answer.take()
    }

    fn undo(&mut self) -> EditorOutcome {
        self.follow = true;
        if self.doc.undo() {
            EditorOutcome::default()
        } else {
            EditorOutcome::say(msg::NOTHING_UNDO.get())
        }
    }

    fn redo(&mut self) -> EditorOutcome {
        self.follow = true;
        if self.doc.redo() {
            EditorOutcome::default()
        } else {
            EditorOutcome::say(msg::NOTHING_REDO.get())
        }
    }

    fn open_find(&mut self) {
        Find::open(&mut self.find, &self.doc);
    }

    /// Selects the next (or previous) match of the find query.
    fn find_step(&mut self, forward: bool) -> EditorOutcome {
        let Some(query) = self.find.as_ref().map(|f| f.query.clone()) else {
            return EditorOutcome::default();
        };
        match find_step_in(&mut self.doc, &query, forward) {
            Some(found) => self.found(&query, found),
            None => EditorOutcome::default(),
        }
    }

    fn found(&mut self, query: &str, found: bool) -> EditorOutcome {
        if found {
            self.follow = true;
            EditorOutcome::default()
        } else {
            EditorOutcome::say(msg::no_match_for(format!("{query:?}")))
        }
    }

    /// A bar or prompt button, clicked - the same action its key runs.
    pub(crate) fn click(&mut self, hit: EditorHit) -> EditorOutcome {
        match hit {
            EditorHit::Save => self.save(),
            EditorHit::Undo => self.undo(),
            EditorHit::Redo => self.redo(),
            EditorHit::Find => {
                self.open_find();
                EditorOutcome::default()
            }
            EditorHit::ToggleMinimap => EditorOutcome::toggle_minimap(),
            EditorHit::FollowSelection => EditorOutcome::toggle_follow(),
            EditorHit::EncodingChip => self.open_chip(StatusMenu::Encoding),
            EditorHit::EolChip => self.open_chip(StatusMenu::LineEndings),
            EditorHit::IndentChip => self.open_chip(StatusMenu::Indentation),
            EditorHit::FindPrev => self.find_step(false),
            EditorHit::FindNext => self.find_step(true),
            EditorHit::Close => self.request_close(),
            EditorHit::PromptSave => self.prompt_save(),
            EditorHit::PromptDiscard => self.prompt_discard(),
            EditorHit::PromptCancel => self.prompt_cancel(),
            EditorHit::VThumb | EditorHit::HThumb | EditorHit::Minimap => EditorOutcome::default(),
        }
    }

    fn open_chip(&mut self, kind: StatusMenu) -> EditorOutcome {
        self.open_menu(kind);
        EditorOutcome::default()
    }

    fn prompt_save(&mut self) -> EditorOutcome {
        let mut out = self.save();
        if out.saved {
            match std::mem::take(&mut self.then) {
                Then::Close => out.close = true,
                Then::Reopen(cs) => {
                    self.prompt = false;
                    out.message = self.reopen_now(cs).message;
                }
                Then::Follow => {
                    self.prompt = false;
                    self.follow_answer = Some(FollowAnswer::Switch);
                }
            }
        }
        out
    }

    /// A pasted string (bracketed paste, or `Ctrl+V`).
    pub(crate) fn paste(&mut self, text: &str) {
        if self.prompt {
            return;
        }
        if let Some(find) = self.find.as_mut() {
            let one_line: String = text.chars().filter(|c| !c.is_control()).collect();
            find.caret.insert(&mut find.query, &one_line);
            return;
        }
        self.doc.insert_text(text);
        self.follow = true;
    }

    pub(crate) fn on_key(&mut self, key: &KeyEvent, clipboard: &mut Clipboard) -> EditorOutcome {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        let ctrl = ctrl_letter(key);
        if self.prompt {
            let km = crate::keymap::current();
            let plain = ctrl.is_none();
            return if plain && (key.code == KeyCode::Enter || km.is(Action::EditorPromptSave, key))
            {
                self.prompt_save()
            } else if plain && km.is(Action::EditorPromptDiscard, key) {
                self.prompt_discard()
            } else if plain && (key.code == KeyCode::Esc || km.is(Action::EditorPromptCancel, key))
            {
                self.prompt_cancel()
            } else {
                EditorOutcome::default()
            };
        }
        if self.menu.is_some() {
            return self.menu_key(key);
        }
        let km = crate::keymap::current();
        if let Some(find) = self.find.as_mut() {
            return match find_on_key(find, key) {
                FindKey::Close => {
                    self.find = None;
                    EditorOutcome::default()
                }
                FindKey::Step(forward) => self.find_step(forward),
                FindKey::Save => self.save(),
                FindKey::Typed(query) => match find_typed_in(&mut self.doc, &query) {
                    Some(found) => self.found(&query, found),
                    None => EditorOutcome::default(),
                },
                FindKey::Clip(s, _) => EditorOutcome::say(
                    clipboard.copy(gitpoulpi_i18n::msg::input::WHAT_SELECTION.get(), &s),
                ),
                FindKey::Nothing => EditorOutcome::default(),
            };
        }
        match km.action_in(Ctx::Editor, key) {
            Some(Action::EditorSave) => return self.save(),
            Some(Action::EditorUndo) => return self.undo(),
            Some(Action::EditorRedo) => return self.redo(),
            Some(Action::EditorFind) => {
                self.open_find();
                return EditorOutcome::default();
            }
            Some(Action::EditorToggleMinimap) => return EditorOutcome::toggle_minimap(),
            Some(Action::EditorPaste) => {
                return match clipboard.paste() {
                    Ok(text) => {
                        self.paste(&text);
                        EditorOutcome::default()
                    }
                    Err(why) => EditorOutcome::say(why),
                };
            }
            _ => {}
        }
        match key.code {
            KeyCode::Esc => self.request_close(),
            _ if km.is(Action::EditorFindNext, key) => {
                self.open_find();
                self.find_step(!shift)
            }
            _ => {
                let page = (self.text_area.height as usize).saturating_sub(1).max(1);
                let edit = self.doc.on_key(key, page);
                self.follow = true;
                match edit {
                    KeyEdit::Copy(s) => EditorOutcome::say(
                        clipboard.copy(gitpoulpi_i18n::msg::input::WHAT_SELECTION.get(), &s),
                    ),
                    KeyEdit::Cut(s) => EditorOutcome::say(clipboard.copy(msg::CLIP_CUT.get(), &s)),
                    _ => EditorOutcome::default(),
                }
            }
        }
    }

    /// The byte offset under screen cell (`col`, `row`), clamped into the
    /// text area so a drag past an edge selects to that edge.
    fn offset_at_cell(&self, col: u16, row: u16) -> usize {
        let ta = self.text_area;
        let r = row
            .min(ta.bottom().saturating_sub(1))
            .max(ta.y)
            .saturating_sub(ta.y);
        let line = (self.top + r as usize).min(self.doc.line_count() - 1);
        let c = col.saturating_sub(ta.x) as usize;
        self.doc.offset_at(line, self.left + c)
    }

    fn viewport(&self) -> usize {
        (self.text_area.height as usize).max(1)
    }

    fn scroll_by(&mut self, delta: isize) {
        let max = self.doc.line_count().saturating_sub(self.viewport());
        self.top = (self.top as isize + delta).clamp(0, max as isize) as usize;
    }

    fn scroll_h(&mut self, delta: isize) {
        let max = self.longest.1 as isize;
        self.left = (self.left as isize + delta).clamp(0, max.max(0)) as usize;
    }

    /// The ticks of the vertical scrollbar's overview ruler: the caret's
    /// line, and every line holding the selected text (see `scroll_marks`).
    fn selected_needle(&self) -> String {
        self.doc
            .selection()
            .map(|(a, b)| self.doc.text()[a.min(b)..a.max(b)].to_string())
            .unwrap_or_default()
    }

    fn ruler_ticks(&mut self) -> Vec<(usize, scroll_marks::Mark)> {
        use scroll_marks::Mark;
        let selected = self.selected_needle();
        let version = self.doc.version();
        let fresh = matches!(&self.ruler_cache, Some((v, s, _)) if *v == version && *s == selected);
        if !fresh {
            let doc = &self.doc;
            let lines =
                scroll_marks::occurrences(&selected, (0..doc.line_count()).map(|i| doc.line(i)));
            self.ruler_cache = Some((version, selected, lines));
        }
        let mut ticks: Vec<(usize, Mark)> = self
            .ruler_cache
            .iter()
            .flat_map(|(_, _, lines)| lines.iter().map(|&l| (l, Mark::Occurrence)))
            .collect();
        ticks.push((self.doc.cursor_line_col().0, Mark::Cursor));
        ticks
    }

    fn v_track(&self) -> Option<scrollbar::Track> {
        scrollbar::Track::vertical(self.body, self.doc.line_count(), self.top)
    }

    fn h_track(&self) -> Option<scrollbar::Track> {
        scrollbar::Track::horizontal(self.body, self.h_content(), self.left)
    }

    /// The horizontal bar's content length: the longest line plus the
    /// caret cell after it, plus the gutter the track also spans.
    fn h_content(&self) -> usize {
        self.longest.1
            + 1
            + self.text_area.x.saturating_sub(self.body.x + 1) as usize
            + self.minimap_pad as usize
    }

    /// A mouse event in the body (`owns_point`), or any drag / release
    /// while a drag the body started is in progress.
    pub(crate) fn on_mouse(
        &mut self,
        kind: MouseEventKind,
        modifiers: KeyModifiers,
        col: u16,
        row: u16,
    ) {
        if self.prompt {
            return;
        }
        let shift = modifiers.contains(KeyModifiers::SHIFT);
        let v = self.v_track();
        let h = self.h_track();
        match kind {
            // The minimap: a click (or a drag along it) centers the view on
            // the line drawn there.
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left)
                if matches!(self.drag, None | Some(Drag::Minimap))
                    && self
                        .minimap_rect
                        .is_some_and(|r| r.contains((col, row).into())) =>
            {
                let rect = self.minimap_rect.expect("checked by the guard");
                let total = self.doc.line_count();
                let line = minimap::line_at(rect, total, row);
                self.top = minimap::top_for(line, total, self.viewport());
                self.drag = Some(Drag::Minimap);
            }
            MouseEventKind::Down(MouseButton::Left)
                if v.is_some_and(|t| t.rect.contains((col, row).into())) =>
            {
                let t = v.expect("checked by the guard");
                let pos = row.saturating_sub(t.rect.y);
                match t.press(pos) {
                    TrackPress::Arrow(step) => self.scroll_by(step),
                    TrackPress::Grab { grab, jump } => {
                        self.drag = Some(Drag::VThumb(grab));
                        if jump {
                            let pos = i32::from(row) - i32::from(t.rect.y);
                            self.top =
                                t.drag_offset(self.doc.line_count(), self.viewport(), grab, pos);
                        }
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Left)
                if h.is_some_and(|t| t.rect.contains((col, row).into())) =>
            {
                let t = h.expect("checked by the guard");
                let pos = col.saturating_sub(t.rect.x);
                match t.press(pos) {
                    TrackPress::Arrow(step) => self.scroll_h(step),
                    TrackPress::Grab { grab, jump } => {
                        self.drag = Some(Drag::HThumb(grab));
                        if jump {
                            let pos = i32::from(col) - i32::from(t.rect.x);
                            self.left =
                                t.drag_offset(self.h_content(), t.rect.width as usize, grab, pos);
                        }
                    }
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                let at = self.offset_at_cell(col, row);
                if text_press(&mut self.last_click, &mut self.doc, at, (col, row), shift) {
                    self.drag = Some(Drag::Text);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => match self.drag {
                Some(Drag::Text) => {
                    // Past the top or bottom edge, the view follows.
                    let ta = self.text_area;
                    if row < ta.y {
                        self.scroll_by(-1);
                    } else if row >= ta.bottom() {
                        self.scroll_by(1);
                    }
                    let at = self.offset_at_cell(col, row);
                    self.doc.set_cursor(at, true);
                }
                Some(Drag::Minimap) => {}
                Some(Drag::VThumb(grab)) => {
                    if let Some(t) = v {
                        let pos = row as i32 - t.rect.y as i32;
                        self.top = t.drag_offset(self.doc.line_count(), self.viewport(), grab, pos);
                    }
                }
                Some(Drag::HThumb(grab)) => {
                    if let Some(t) = h {
                        let pos = col as i32 - t.rect.x as i32;
                        self.left =
                            t.drag_offset(self.h_content(), t.rect.width as usize, grab, pos);
                    }
                }
                None => {}
            },
            MouseEventKind::Up(MouseButton::Left) => self.drag = None,
            MouseEventKind::ScrollLeft => self.scroll_h(-4),
            MouseEventKind::ScrollRight => self.scroll_h(4),
            MouseEventKind::ScrollDown if shift => self.scroll_h(4),
            MouseEventKind::ScrollUp if shift => self.scroll_h(-4),
            MouseEventKind::ScrollDown => self.scroll_by(WHEEL_LINES as isize),
            MouseEventKind::ScrollUp => self.scroll_by(-(WHEEL_LINES as isize)),
            _ => {}
        }
    }
}

/// The letter of a `Ctrl+letter` chord, whichever way the terminal sent
/// it (`Char('s')` with `CONTROL`, or the bare C0 control character --
/// see `text_edit::decode`). AltGr (`CONTROL | ALT`) is a chord only for
/// letters and digits.
fn ctrl_letter(key: &KeyEvent) -> Option<char> {
    let alt = key.modifiers.contains(KeyModifiers::ALT);
    match key.code {
        KeyCode::Char(c) if ('\u{1}'..='\u{1a}').contains(&c) => Some((b'a' + c as u8 - 1) as char),
        KeyCode::Char(c)
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && (!alt || c.is_ascii_alphanumeric()) =>
        {
            Some(c.to_ascii_lowercase())
        }
        _ => None,
    }
}

/// Writes `bytes` to `path` without ever leaving it half-written: into a
/// temp file in the same directory (same filesystem, so the rename is
/// atomic), flushed, given the original's permissions, then renamed over
/// the original. On failure the temp file is removed and the original is
/// untouched.
pub(crate) fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let dir = path.parent().unwrap_or(Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = dir.join(format!(".{name}.gitpoulpi-save-{}.tmp", std::process::id()));
    let result = (|| {
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        if let Ok(meta) = std::fs::metadata(path) {
            std::fs::set_permissions(&tmp, meta.permissions())?;
        }
        std::fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

#[cfg(test)]
mod tests;
