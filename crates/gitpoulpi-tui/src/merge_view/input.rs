//! The merge editor's keys, mouse and commands (plan §2.4).
//!
//! Everything funnels into [`MergeCmd`]: a key (through the `merge` and
//! `merge_sides` keymap contexts), a click on a [`MergeHit`], a menu row and
//! a palette entry all run the same [`MergeView::run`], so the surfaces
//! cannot drift apart (maintainer rule 4). A command changes the view and the
//! model itself; what needs the window (writing the file, opening another
//! one, closing the editor) comes back as an [`Effect`] in the [`Outcome`].
//!
//! No IPC here: the window turns an effect into a link request
//! (`merge_flow.rs`) and feeds the answer back.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use ratatui::layout::Rect;

use gitpoulpi_core::domain::{ConflictResolution, ConflictSide, ResolveOptions};
use gitpoulpi_i18n::msg::merge as tr;

use crate::clipboard::Clipboard;
use crate::editor::{find_on_key, find_step_in, find_typed_in, text_press, Find, FindKey};
use crate::keymap::{self, Action, Ctx};
use crate::text_edit::KeyEdit;

use super::layout::Split;
use super::model::{Hint, SaveChecks};
use super::render::replaced_edit_text;
use super::view::MergeView;
use super::{MergeHit, MergePane, MergeScroll};

/// Lines per wheel notch, as the editor's.
const WHEEL_LINES: isize = 3;

/// Previous / next conflicted file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    Next,
    Prev,
}

/// What to do once a save (or a discard) has gone through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Then {
    Stay,
    Close,
    File(Step),
}

pub(crate) use super::cmd::MergeCmd;

/// What a command asks of the window.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Effect {
    /// Close the editor (nothing is left to save or the work was discarded).
    Close,
    /// Write the Output. The request is built by
    /// [`MergeView::save_request`].
    Save { stage: bool, then: Then },
    /// Open the previous / next conflicted file.
    Goto(Step),
    /// Put this file's conflict back, then open it again.
    Unresolve,
    /// Read the file again (the stages changed under the editor).
    Reload,
    /// Stage a side's whole file as it is.
    UseFile(ConflictSide),
    /// Remove the file (`git rm`): the deletion wins.
    Delete,
    /// Open the file in the external merge tool.
    ExternalTool,
    /// Show or hide the Refs and Status panels (window layout).
    TogglePanes,
    /// ✦ Run the AI for conflict `conflict` (0-based); the window owns the
    /// worker and puts the answer back with `MergeModel::set_region_lines`.
    AiSuggest { conflict: usize },
}

/// What a key, a click or a command did.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Outcome {
    /// For the status line.
    pub message: Option<String>,
    pub effect: Option<Effect>,
}

impl Outcome {
    fn say(message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            effect: None,
        }
    }

    fn effect(effect: Effect) -> Self {
        Self {
            message: None,
            effect: Some(effect),
        }
    }
}

/// The boxes that sit over the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Prompt {
    /// Save / Discard / Cancel: unsaved work and a way out.
    Unsaved { then: Then },
    /// A save that would leave conflicts empty or markers in the file.
    Confirm {
        checks: SaveChecks,
        stage: bool,
        then: Then,
    },
    /// Reset the whole file while the Output holds typed text.
    ResetAll,
}

/// The strip that replaces the key hints when something outside moved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Strip {
    /// The file on disk was edited by something else (plan §2.5 case 2).
    EditedOutside { disk: Option<String> },
    /// The path stopped being conflicted behind the editor's back.
    Resolved,
    /// The stages changed (another frontend re-conflicted the file, or a
    /// save was refused as stale).
    Changed,
}

/// One row of a popup menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MenuRow {
    pub label: String,
    pub cmd: MergeCmd,
}

/// A popup menu: the context menu of a conflict chunk, or the `…` menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Menu {
    /// The cell it was opened at (its top left, before clamping).
    pub at: (u16, u16),
    pub rows: Vec<MenuRow>,
    pub selected: usize,
    /// The conflict the rows act on (a chunk menu).
    pub conflict: Option<usize>,
}

/// A drag the body started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Drag {
    Splitter,
    /// The separator after top pane `n`.
    Divider(u8),
    /// A scrollbar's thumb held `grab` cells below its top.
    Thumb(MergeScroll, i32),
    /// A text selection dragged out in the Output.
    Text,
}

/// Where the panes were last painted: what the mouse needs to map a point
/// back to a pane (set by the paint walk, never recomputed here).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Areas {
    /// The whole view.
    pub all: Rect,
    /// The A / B / Base panes.
    pub top: Rect,
    pub out_title: Rect,
    pub out: Rect,
    /// The left edge of the top panes and the widths painted (wide: one
    /// per pane, `panes` of them; narrow: `panes` is 0).
    pub pane_x0: u16,
    pub pane_w: [u16; 3],
    pub panes: u8,
    /// The Output's gutter width in columns.
    pub gutter: u16,
}

/// What only the input knows.
#[derive(Debug, Default)]
pub(crate) struct Ui {
    pub prompt: Option<Prompt>,
    pub menu: Option<Menu>,
    pub strip: Option<Strip>,
    pub drag: Option<Drag>,
    /// The Top and Output scrollbars as last painted (the mouse's view).
    pub bars: std::cell::Cell<[Option<crate::scrollbar::Track>; 2]>,
    /// The last press on a splitter or divider, for the double click.
    pub last_press: Option<(std::time::Instant, MergeHit)>,
    /// A save is in flight: the button shows it and a second one waits.
    pub saving: Option<Then>,
    /// The line the keyboard cursor sits on, in the current conflict's
    /// block of the focused side.
    pub cursor: usize,
    /// The A / B / Base pane that had the focus before the Output took it
    /// (Esc in the Output goes back there).
    pub last_side: Option<MergePane>,
    /// The caret sits on the placeholder row of this (empty) conflict:
    /// the next typed character opens a line there.
    pub placeholder: Option<usize>,
    /// The Output's `Ctrl+F` bar.
    pub find: Option<Find>,
    /// The last find step found nothing.
    pub find_miss: bool,
    /// The last press in the Output (a second one on the same cell
    /// selects the word).
    pub last_click: Option<(std::time::Instant, u16, u16)>,
}

impl Ui {
    /// A box is up: it owns the keys and the pointer.
    pub(crate) fn modal(&self) -> bool {
        self.prompt.is_some() || self.menu.is_some()
    }
}

fn is_text_key(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char(_) | KeyCode::Enter | KeyCode::Tab)
        && !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
}

impl MergeView {
    // -- the keyboard ---------------------------------------------------------

    /// Whether the window leaves `key` to itself (the palette, the prefixes
    /// and the other chords the editor has no use for), as the editor's
    /// `passes_through` does. A box keeps every key.
    pub(crate) fn passes_through(&self, key: &KeyEvent) -> bool {
        if self.ui.modal() || self.ui.find.is_some() {
            return false;
        }
        let km = keymap::current();
        if km.is(Action::GlobalGitPrefix, key)
            || km.is(Action::GlobalNavPrefix, key)
            || km.is(Action::GlobalPalette, key)
        {
            return true;
        }
        if km.action_in(Ctx::Merge, key).is_some() {
            return false;
        }
        // A `Ctrl` chord the editor has no use for stays the window's,
        // except the text-editing ones the Output keeps.
        matches!(key.code, KeyCode::Char(c)
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && !"acukxvzyf".contains(c.to_ascii_lowercase()))
    }

    /// [`MergeView::on_key`] with the vim count the window parsed (`3j`):
    /// the motions repeat that many times.
    pub(crate) fn on_key_counted(
        &mut self,
        key: &KeyEvent,
        clipboard: &mut Clipboard,
        count: usize,
    ) -> Outcome {
        self.count = count.max(1);
        let out = self.on_key(key, clipboard);
        self.count = 1;
        out
    }

    pub(crate) fn on_key(&mut self, key: &KeyEvent, clipboard: &mut Clipboard) -> Outcome {
        self.note = None;
        self.note_input();
        if self.ui.prompt.is_some() {
            return self.prompt_key(key);
        }
        if self.ui.menu.is_some() {
            return self.menu_key(key);
        }
        if self.card.is_some() {
            return self.card_key(key);
        }
        let km = keymap::current();
        if let Some(cmd) = km.action_in(Ctx::Merge, key).and_then(cmd_of) {
            return self.run(cmd);
        }
        match key.code {
            KeyCode::Esc if self.ui.find.is_some() => {
                self.ui.find = None;
                return Outcome::default();
            }
            KeyCode::Esc => {
                return if self.focus == MergePane::Output {
                    self.focus_pane(self.ui.last_side.unwrap_or(MergePane::A));
                    Outcome::default()
                } else {
                    self.run(MergeCmd::Close)
                };
            }
            KeyCode::Tab if self.focus != MergePane::Output => {
                return self.run(MergeCmd::FocusNext);
            }
            KeyCode::BackTab if self.focus != MergePane::Output => {
                return self.run(MergeCmd::FocusPrev);
            }
            _ => {}
        }
        if self.focus == MergePane::Output {
            return self.output_key(key, clipboard);
        }
        if self.ui.find.is_some() {
            self.ui.find = None;
        }
        self.side_key(key)
    }

    fn side_key(&mut self, key: &KeyEvent) -> Outcome {
        let km = keymap::current();
        let side = self.focus.side().unwrap_or(ConflictSide::Ours);
        if let Some(action) = km.action_in(Ctx::MergeSides, key) {
            let c = self.model.current();
            let cursor = self.ui.cursor;
            let outcome = match action {
                Action::MergeToggleLine => self.pick(|m| m.toggle_line(c, side, cursor)),
                Action::MergeToggleChunk => self.pick(|m| m.toggle_chunk(c, side)),
                Action::MergeSideNextConflict => return self.run(MergeCmd::NextConflict),
                Action::MergeSidePrevConflict => return self.run(MergeCmd::PrevConflict),
                Action::MergeSideUnpick => return self.run(MergeCmd::Unpick),
                Action::MergeSideResetAll => return self.run(MergeCmd::Reset),
                Action::MergeSideToggleBase => return self.run(MergeCmd::ToggleBase),
                Action::MergeSideToggleFullFile => return self.run(MergeCmd::ToggleFullFile),
                other => match cmd_of(other) {
                    Some(cmd) => return self.run(cmd),
                    None => Outcome::default(),
                },
            };
            return outcome;
        }
        // The context-menu key opens the chunk menu, as a right-click does.
        if km.is(Action::GlobalContextMenu, key) {
            self.open_chunk_menu(self.model.current(), (2, 2));
            return Outcome::default();
        }
        let page = self.heights.0.saturating_sub(1).max(1) as isize * self.count as isize;
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT);
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(1),
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(-1),
            KeyCode::Left | KeyCode::Char('h') if plain => return self.run(MergeCmd::PaneLeft),
            KeyCode::Right | KeyCode::Char('l') if plain => return self.run(MergeCmd::PaneRight),
            KeyCode::PageDown => self.scroll_top_by(page),
            KeyCode::PageUp => self.scroll_top_by(-page),
            KeyCode::Home => self.scroll_top_by(isize::MIN / 2),
            KeyCode::End => self.scroll_top_by(isize::MAX / 2),
            _ => {}
        }
        Outcome::default()
    }

    /// The arrow keys: the line cursor moves down the focused side's
    /// conflict blocks, crossing into the next conflict at the end of one.
    fn move_cursor(&mut self, delta: isize) {
        for _ in 0..self.count.max(1) {
            self.move_cursor_once(delta);
        }
    }

    fn move_cursor_once(&mut self, delta: isize) {
        let side = self.focus.side().unwrap_or(ConflictSide::Ours);
        let c = self.model.current();
        let n = self.model.block(c, side).len();
        let want = self.ui.cursor as isize + delta;
        if want < 0 {
            if c > 0 && self.model.go_to(c - 1) {
                self.ui.cursor = self.model.block(c - 1, side).len().saturating_sub(1);
                self.reveal_current();
            } else {
                self.ui.cursor = 0;
            }
        } else if want as usize >= n {
            if c + 1 < self.model.conflict_count() && self.model.go_to(c + 1) {
                self.ui.cursor = 0;
                self.reveal_current();
            } else {
                self.ui.cursor = n.saturating_sub(1);
            }
        } else {
            self.ui.cursor = want as usize;
        }
    }

    /// Typing in the Output: the document's own key handling, with the
    /// anchors following and the clipboard chords the editor has.
    fn output_key(&mut self, key: &KeyEvent, clipboard: &mut Clipboard) -> Outcome {
        let km = keymap::current();
        if self.ui.find.is_some() {
            return self.find_key(key, clipboard);
        }
        if km.is(Action::EditorFind, key) {
            Find::open(&mut self.ui.find, self.model.output());
            self.ui.find_miss = false;
            return Outcome::default();
        }
        if km.is(Action::EditorFindNext, key) {
            let shift = key.modifiers.contains(KeyModifiers::SHIFT);
            Find::open(&mut self.ui.find, self.model.output());
            return self.find_step(!shift);
        }
        if km.is(Action::EditorPaste, key) {
            return match clipboard.paste() {
                Ok(text) => {
                    self.paste(&text);
                    Outcome::default()
                }
                Err(why) => Outcome::say(why),
            };
        }
        if is_text_key(key) {
            if let Some(at) = self.open_placeholder_line() {
                // Enter on a placeholder row is the line it opened: the
                // caret goes past it instead of breaking it again.
                if key.code == KeyCode::Enter {
                    self.model.output_for_caret().set_cursor(at + 1, false);
                    self.model.follow_caret();
                    self.ensure_caret_visible();
                    return Outcome::default();
                }
            }
        }
        self.ui.placeholder = None;
        let page = self.heights.1.saturating_sub(1).max(1);
        let edit = self.model.edit(|d| d.on_key(key, page));
        self.model.follow_caret();
        self.ensure_caret_visible();
        match edit {
            KeyEdit::Copy(s) => Outcome::say(clipboard.copy(tr::CLIP_SELECTION.get(), &s)),
            KeyEdit::Cut(s) => Outcome::say(clipboard.copy(tr::CLIP_CUT.get(), &s)),
            _ => Outcome::default(),
        }
    }

    /// A key while the find bar is up: the editor's own bar logic.
    fn find_key(&mut self, key: &KeyEvent, clipboard: &mut Clipboard) -> Outcome {
        let Some(find) = self.ui.find.as_mut() else {
            return Outcome::default();
        };
        match find_on_key(find, key) {
            FindKey::Close => {
                self.ui.find = None;
                Outcome::default()
            }
            FindKey::Step(forward) => self.find_step(forward),
            FindKey::Save => self.run(MergeCmd::Save),
            FindKey::Typed(query) => {
                let found = find_typed_in(self.model.output_for_caret(), &query);
                self.found(found)
            }
            FindKey::Clip(s, cut) => {
                let what = if cut {
                    tr::CLIP_CUT
                } else {
                    tr::CLIP_SELECTION
                };
                Outcome::say(clipboard.copy(what.get(), &s))
            }
            FindKey::Nothing => Outcome::default(),
        }
    }

    /// Selects the next (or previous) match of the find bar's query.
    fn find_step(&mut self, forward: bool) -> Outcome {
        let Some(query) = self.ui.find.as_ref().map(|f| f.query.clone()) else {
            return Outcome::default();
        };
        let found = find_step_in(self.model.output_for_caret(), &query, forward);
        self.found(found)
    }

    fn found(&mut self, found: Option<bool>) -> Outcome {
        self.ui.find_miss = found == Some(false);
        if found == Some(true) {
            self.model.follow_caret();
            self.ensure_caret_visible();
        }
        Outcome::default()
    }

    /// The find bar's text for the hint row: the query with its caret
    /// and what the last step found.
    pub(super) fn find_line(&self) -> Option<String> {
        let f = self.ui.find.as_ref()?;
        let at = f.caret.cursor(&f.query).min(f.query.len());
        let query = format!("{}\u{258f}{}", &f.query[..at], &f.query[at..]);
        Some(if self.ui.find_miss {
            tr::find_bar_miss(&query)
        } else {
            tr::find_bar(&query)
        })
    }

    /// A pasted string (bracketed paste or `Ctrl+V`) into the Output.
    pub(crate) fn paste(&mut self, text: &str) {
        if self.focus != MergePane::Output || self.ui.modal() {
            return;
        }
        // Whole lines (ending in a break) need no line opened for them.
        if text.ends_with('\n') {
            self.ui.placeholder = None;
        } else {
            self.open_placeholder_line();
        }
        self.model.edit(|d| d.insert_text(text));
        self.model.follow_caret();
        self.ensure_caret_visible();
    }

    /// Typing on the placeholder row of an unpicked conflict: the region
    /// is empty, so a character would join the line after it. A line break
    /// goes in first, the caret stays in front of it (the model's known
    /// edge, see its module doc).
    fn open_placeholder_line(&mut self) -> Option<usize> {
        let c = self.ui.placeholder.take()?;
        let m = &mut self.model;
        let empty = m.region_lines(c).is_some_and(|(a, b)| a == b);
        if empty && m.output().selection().is_none() {
            let at = m.output().cursor();
            m.edit(|d| {
                d.insert_text("\n");
                d.set_cursor(at, false);
            });
            return Some(at);
        }
        None
    }

    /// Scrolls the Output so the caret's line is on screen.
    fn ensure_caret_visible(&mut self) {
        let out = self.out_layout();
        let line = self.model.output().cursor_line_col().0;
        let row = out.row_of_line(line);
        let h = self.heights.1.max(1);
        if row < self.out_scroll {
            self.out_scroll = row;
        } else if row >= self.out_scroll + h {
            self.out_scroll = row + 1 - h;
        }
    }

    // -- the boxes ------------------------------------------------------------

    fn prompt_key(&mut self, key: &KeyEvent) -> Outcome {
        let km = keymap::current();
        let plain = !key.modifiers.contains(KeyModifiers::CONTROL);
        if !plain {
            return Outcome::default();
        }
        if key.code == KeyCode::Enter || km.is(Action::EditorPromptSave, key) {
            return self.prompt_answer(PromptAnswer::Primary);
        }
        if km.is(Action::EditorPromptDiscard, key) {
            return self.prompt_answer(PromptAnswer::Secondary);
        }
        if key.code == KeyCode::Esc || km.is(Action::EditorPromptCancel, key) {
            return self.prompt_answer(PromptAnswer::Cancel);
        }
        Outcome::default()
    }

    fn prompt_answer(&mut self, answer: PromptAnswer) -> Outcome {
        let Some(prompt) = self.ui.prompt.take() else {
            return Outcome::default();
        };
        match (prompt, answer) {
            (_, PromptAnswer::Cancel) => Outcome::default(),
            (Prompt::ResetAll, PromptAnswer::Primary) => self.reset_all(),
            (Prompt::ResetAll, PromptAnswer::Secondary) => Outcome::default(),
            // Save: the file is written as it is and stays conflicted.
            (Prompt::Unsaved { then }, PromptAnswer::Primary) => {
                Outcome::effect(Effect::Save { stage: false, then })
            }
            (Prompt::Unsaved { then }, PromptAnswer::Secondary) => match then {
                Then::File(step) => Outcome::effect(Effect::Goto(step)),
                _ => Outcome::effect(Effect::Close),
            },
            (
                Prompt::Confirm {
                    stage,
                    then,
                    checks,
                },
                PromptAnswer::Primary,
            ) => {
                let _ = checks;
                self.ui.saving = Some(then);
                Outcome::effect(Effect::Save { stage, then })
            }
            // Go to the first conflict the save was worried about.
            (Prompt::Confirm { checks, .. }, PromptAnswer::Secondary) => {
                if let Some(&c) = checks.unpicked.first() {
                    self.model.go_to(c);
                    self.ui.cursor = 0;
                    self.reveal_current();
                } else if let Some(&line) = checks.marker_lines.first() {
                    self.focus_pane(MergePane::Output);
                    let doc = self.model.output_for_caret();
                    let at = doc.offset_at(line.saturating_sub(1), 0);
                    doc.set_cursor(at, false);
                    self.ensure_caret_visible();
                }
                Outcome::default()
            }
        }
    }

    fn menu_key(&mut self, key: &KeyEvent) -> Outcome {
        let Some(menu) = self.ui.menu.as_mut() else {
            return Outcome::default();
        };
        let n = menu.rows.len().max(1);
        match key.code {
            KeyCode::Esc => {
                self.ui.menu = None;
            }
            KeyCode::Down | KeyCode::Char('j') => menu.selected = (menu.selected + 1) % n,
            KeyCode::Up | KeyCode::Char('k') => menu.selected = (menu.selected + n - 1) % n,
            KeyCode::Enter => {
                let i = menu.selected;
                return self.run_menu_row(i);
            }
            _ => {}
        }
        Outcome::default()
    }

    fn run_menu_row(&mut self, i: usize) -> Outcome {
        let Some(menu) = self.ui.menu.take() else {
            return Outcome::default();
        };
        let Some(row) = menu.rows.get(i) else {
            return Outcome::default();
        };
        if let Some(c) = menu.conflict {
            self.model.go_to(c);
            self.ui.cursor = 0;
        }
        self.run(row.cmd)
    }

    pub(super) fn open_chunk_menu(&mut self, conflict: usize, at: (u16, u16)) {
        if self.model.is_outside() || conflict >= self.model.conflict_count() {
            return;
        }
        let mut rows = vec![
            (tr::MENU_TAKE_A.get().to_string(), MergeCmd::TakeA),
            (tr::MENU_TAKE_B.get().to_string(), MergeCmd::TakeB),
            (tr::MENU_A_THEN_B.get().to_string(), MergeCmd::TakeBoth),
            (
                tr::MENU_B_THEN_A.get().to_string(),
                MergeCmd::TakeBothReversed,
            ),
            (tr::MENU_NONE.get().to_string(), MergeCmd::Clear),
            (tr::MENU_UNPICK.get().to_string(), MergeCmd::Unpick),
            (tr::MENU_EDIT.get().to_string(), MergeCmd::EditInOutput),
        ];
        if self.ai_enabled {
            rows.push((ai_label(tr::MENU_AI_SUGGEST.get()), MergeCmd::AiSuggest));
        }
        self.ui.menu = Some(Menu {
            at,
            rows: rows
                .into_iter()
                .map(|(label, cmd)| MenuRow {
                    label: label.to_string(),
                    cmd,
                })
                .collect(),
            selected: 0,
            conflict: Some(conflict),
        });
    }

    pub(super) fn open_more_menu(&mut self, at: (u16, u16)) {
        let mut rows = vec![
            (tr::MENU_TAKE_ALL_A.get().to_string(), MergeCmd::TakeAllA),
            (tr::MENU_TAKE_ALL_B.get().to_string(), MergeCmd::TakeAllB),
            (tr::MENU_BASE.get().to_string(), MergeCmd::ToggleBase),
            (
                tr::MENU_FULL_FILE.get().to_string(),
                MergeCmd::ToggleFullFile,
            ),
            (tr::MENU_RESET.get().to_string(), MergeCmd::Reset),
            (tr::MENU_USE_A_FILE.get().to_string(), MergeCmd::UseFileA),
            (tr::MENU_USE_B_FILE.get().to_string(), MergeCmd::UseFileB),
        ];
        if self.ai_enabled {
            rows.push((ai_label(tr::MENU_AI_SUGGEST.get()), MergeCmd::AiSuggest));
        }
        self.ui.menu = Some(Menu {
            at,
            rows: rows
                .into_iter()
                .map(|(label, cmd)| MenuRow {
                    label: label.to_string(),
                    cmd,
                })
                .collect(),
            selected: 0,
            conflict: None,
        });
    }

    /// ✦ The AI suggestion for the current conflict: checks that it can run
    /// here, then asks the window to start it.
    fn ai_suggest(&mut self) -> Outcome {
        if !self.ai_enabled {
            return Outcome::say(tr::MSG_AI_OFF.get());
        }
        if self.ai_busy {
            return Outcome::say(tr::MSG_AI_BUSY.get());
        }
        let c = self.model.current();
        if self.card().is_some() || self.model.is_outside() || c >= self.model.conflict_count() {
            return Outcome::say(tr::MSG_AI_NO_TEXT.get());
        }
        Outcome::effect(Effect::AiSuggest { conflict: c })
    }

    /// The window is about to end with unsaved work in the editor: ask
    /// first (Save / Discard / Cancel).
    pub(crate) fn ask_close(&mut self) {
        self.ui.prompt = Some(Prompt::Unsaved { then: Then::Close });
    }

    /// Closes whatever box is up (a click elsewhere).
    pub(crate) fn dismiss_popup(&mut self) {
        self.ui.menu = None;
    }

    // -- commands --------------------------------------------------------------

    /// The one place every surface's action lands.
    pub(crate) fn run(&mut self, cmd: MergeCmd) -> Outcome {
        let c = self.model.current();
        match cmd {
            MergeCmd::NextConflict => self.go_conflict(true),
            MergeCmd::PrevConflict => self.go_conflict(false),
            MergeCmd::NextFile => self.leave(Then::File(Step::Next)),
            MergeCmd::PrevFile => self.leave(Then::File(Step::Prev)),
            MergeCmd::ChunkA => self.pick(|m| m.toggle_chunk(c, ConflictSide::Ours)),
            MergeCmd::ChunkB => self.pick(|m| m.toggle_chunk(c, ConflictSide::Theirs)),
            MergeCmd::TakeA => self.pick(|m| m.take_a(c)),
            MergeCmd::TakeB => self.pick(|m| m.take_b(c)),
            MergeCmd::TakeBoth => self.pick(|m| m.take_a_then_b(c)),
            MergeCmd::TakeBothReversed => self.pick(|m| m.take_b_then_a(c)),
            MergeCmd::TakeBase => self.pick(|m| m.take_base(c)),
            MergeCmd::Clear => self.pick(|m| m.clear(c)),
            MergeCmd::Unpick => {
                let before = self.model.status(c);
                let out = self.pick(|m| m.unpick(c));
                if out.message.is_none() && before != self.model.status(c) {
                    self.ui.placeholder = None;
                    return Outcome::say(tr::MSG_UNPICKED.get());
                }
                out
            }
            MergeCmd::SwapOrder => self.pick(|m| m.swap_order(c)),
            MergeCmd::TakeAllA => self.pick(|m| m.take_all_a()),
            MergeCmd::TakeAllB => self.pick(|m| m.take_all_b()),
            MergeCmd::ToggleBase => {
                self.show_base = !self.show_base;
                if !self.show_base && self.focus == MergePane::Base {
                    self.focus_pane(MergePane::A);
                }
                if !self.show_base && self.tab == ConflictSide::Base {
                    self.tab = ConflictSide::Ours;
                }
                Outcome::default()
            }
            MergeCmd::ToggleFullFile => {
                self.full_file = !self.full_file;
                Outcome::default()
            }
            MergeCmd::Undo => {
                if self.model.undo() {
                    self.after_history();
                    Outcome::default()
                } else {
                    Outcome::say(tr::MSG_NOTHING_UNDO.get())
                }
            }
            MergeCmd::Redo => {
                if self.model.redo() {
                    self.after_history();
                    Outcome::default()
                } else {
                    Outcome::say(tr::MSG_NOTHING_REDO.get())
                }
            }
            MergeCmd::Save => self.save(false),
            MergeCmd::SaveResolve => self.save(true),
            MergeCmd::Reset => {
                if self.model.is_outside() {
                    return Outcome::say(tr::MSG_PICKS_OFF.get());
                }
                // Typed text would be thrown away: ask first.
                if self.model.has_manual_edits() {
                    self.ui.prompt = Some(Prompt::ResetAll);
                    return Outcome::default();
                }
                self.reset_all()
            }
            MergeCmd::FocusPane(pane) => self.goto_pane(pane),
            MergeCmd::PaneLeft => self.step_pane(false),
            MergeCmd::PaneRight => self.step_pane(true),
            MergeCmd::CycleHeight => {
                self.cycle_split();
                Outcome::default()
            }
            MergeCmd::WidenCurrent => self.widen_side(ConflictSide::Ours),
            MergeCmd::WidenIncoming => self.widen_side(ConflictSide::Theirs),
            MergeCmd::ResetPaneSizes => {
                self.cols = Default::default();
                self.split = Split::DEFAULT;
                Outcome::say(tr::MSG_PANES_RESET.get())
            }
            MergeCmd::Close => self.leave(Then::Close),
            MergeCmd::FocusNext => self.cycle_focus(true),
            MergeCmd::FocusPrev => self.cycle_focus(false),
            MergeCmd::EditInOutput => {
                self.focus_pane(MergePane::Output);
                if let Some((first, end)) = self.model.region_lines(c) {
                    let doc = self.model.output_for_caret();
                    let at = doc.offset_at(first, 0);
                    doc.set_cursor(at, false);
                    self.ui.placeholder = (first == end).then_some(c);
                    self.ensure_caret_visible();
                }
                Outcome::default()
            }
            MergeCmd::ContinueFromDisk => match self.ui.strip.take() {
                Some(Strip::EditedOutside { disk: Some(text) }) => {
                    self.load_disk(&text);
                    Outcome::say(tr::MSG_LOADED_DISK.get())
                }
                other => {
                    self.ui.strip = other;
                    Outcome::default()
                }
            },
            MergeCmd::StartOver => {
                if matches!(self.ui.strip, Some(Strip::EditedOutside { .. })) {
                    self.ui.strip = None;
                }
                Outcome::default()
            }
            MergeCmd::Unresolve => Outcome::effect(Effect::Unresolve),
            MergeCmd::Reload => Outcome::effect(Effect::Reload),
            MergeCmd::UseFileA => Outcome::effect(Effect::UseFile(ConflictSide::Ours)),
            MergeCmd::UseFileB => Outcome::effect(Effect::UseFile(ConflictSide::Theirs)),
            MergeCmd::UseFileBase => Outcome::effect(Effect::UseFile(ConflictSide::Base)),
            MergeCmd::Keep => match self.card.as_ref().and_then(|c| c.keep) {
                Some(side) => Outcome::effect(Effect::UseFile(side)),
                None => Outcome::say(tr::REASON_NOT_CARD.get()),
            },
            MergeCmd::Delete => Outcome::effect(Effect::Delete),
            MergeCmd::ExternalTool => Outcome::effect(Effect::ExternalTool),
            MergeCmd::AiSuggest => self.ai_suggest(),
            MergeCmd::TogglePanes => Outcome::effect(Effect::TogglePanes),
            MergeCmd::ScrollLeft => {
                self.scroll_h_by(-(super::hscroll::H_STEP as isize));
                Outcome::default()
            }
            MergeCmd::ScrollRight => {
                self.scroll_h_by(super::hscroll::H_STEP as isize);
                Outcome::default()
            }
        }
    }

    fn after_history(&mut self) {
        self.model.follow_caret();
        self.ensure_caret_visible();
    }

    /// A pick, with the one-time hint it may leave and the guard against
    /// an Output that came from the disk.
    fn pick(&mut self, f: impl FnOnce(&mut super::model::MergeModel) -> bool) -> Outcome {
        if self.model.is_outside() {
            return Outcome::say(tr::MSG_PICKS_OFF.get());
        }
        if self.model.conflict_count() == 0 {
            return Outcome::default();
        }
        if f(&mut self.model) {
            if let Some(Hint::ReplacedEdit) = self.model.take_hint() {
                let key = keymap::current().shown(Action::MergeUndo);
                self.note = Some(replaced_edit_text(&key));
            }
            self.ensure_caret_visible();
        }
        Outcome::default()
    }

    fn go_conflict(&mut self, forward: bool) -> Outcome {
        let mut moved = None;
        for _ in 0..self.count.max(1) {
            moved = if forward {
                self.model.next_conflict()
            } else {
                self.model.prev_conflict()
            };
        }
        if moved.is_some() {
            self.ui.cursor = 0;
            self.reveal_current();
        }
        Outcome::default()
    }

    /// Leaving this file (closing it, or going to another): a dirty Output
    /// asks first.
    fn leave(&mut self, then: Then) -> Outcome {
        if self.model.is_dirty() {
            self.ui.prompt = Some(Prompt::Unsaved { then });
            return Outcome::default();
        }
        match then {
            Then::File(step) => Outcome::effect(Effect::Goto(step)),
            _ => Outcome::effect(Effect::Close),
        }
    }

    fn save(&mut self, stage: bool) -> Outcome {
        // A card has no text to write: its buttons resolve the file.
        if self.ui.saving.is_some() || self.card.is_some() {
            return Outcome::default();
        }
        if stage {
            let checks = self.model.save_checks();
            if !checks.is_clean() {
                self.ui.prompt = Some(Prompt::Confirm {
                    checks,
                    stage,
                    then: Then::Close,
                });
                return Outcome::default();
            }
        }
        let then = if stage { Then::Close } else { Then::Stay };
        self.ui.saving = Some(then);
        Outcome::effect(Effect::Save { stage, then })
    }

    /// Focus moves to `pane`; in the narrow layout a side's tab follows.
    pub(crate) fn focus_pane(&mut self, pane: MergePane) {
        if self.focus != MergePane::Output && pane == MergePane::Output {
            self.ui.last_side = Some(self.focus);
        }
        if pane != MergePane::Output {
            self.ui.find = None;
        }
        self.focus = pane;
        if let Some(side) = pane.side() {
            self.tab = side;
            self.ui.cursor = 0;
        }
    }

    /// The panes in reading order: A, Base, B, Output in the wide layout;
    /// the narrow tabs read A, B, Base, then the Output.
    fn pane_order(&self) -> Vec<MergePane> {
        if self.narrow {
            return vec![
                MergePane::A,
                MergePane::B,
                MergePane::Base,
                MergePane::Output,
            ];
        }
        let mut order = vec![MergePane::A];
        if self.show_base {
            order.push(MergePane::Base);
        }
        order.push(MergePane::B);
        order.push(MergePane::Output);
        order
    }

    /// Focus moves to `pane` and it lands on the current conflict: the
    /// panes scroll to it, the Output's caret goes to its region.
    fn goto_pane(&mut self, pane: MergePane) -> Outcome {
        if pane == MergePane::Base && !self.narrow && !self.show_base {
            self.show_base = true;
        }
        self.focus_pane(pane);
        self.reveal_current();
        if pane == MergePane::Output {
            return self.run(MergeCmd::EditInOutput);
        }
        Outcome::default()
    }

    /// `h` / `l` and the arrows: the neighboring pane, no wrap.
    fn step_pane(&mut self, forward: bool) -> Outcome {
        let order = self.pane_order();
        let i = order.iter().position(|p| *p == self.focus).unwrap_or(0);
        let next = if forward { i + 1 } else { i.saturating_sub(1) };
        match order.get(next) {
            Some(&pane) if pane != self.focus => self.goto_pane(pane),
            _ => Outcome::default(),
        }
    }

    /// Every conflict unpicked and the Output the fresh merge, one undo
    /// step (after the confirm, when typed text is at stake).
    fn reset_all(&mut self) -> Outcome {
        if self.model.reset() {
            self.ui.cursor = 0;
            self.ui.placeholder = None;
            self.reveal_current();
            Outcome::say(tr::MSG_RESET.get())
        } else {
            Outcome::default()
        }
    }

    fn cycle_focus(&mut self, forward: bool) -> Outcome {
        let order = self.pane_order();
        let i = order.iter().position(|p| *p == self.focus).unwrap_or(0);
        let n = order.len();
        let next = if forward {
            (i + 1) % n
        } else {
            (i + n - 1) % n
        };
        self.focus_pane(order[next]);
        Outcome::default()
    }

    /// Loads the file on disk into the Output (the "edited outside" case).
    fn load_disk(&mut self, text: &str) {
        let merge = self.model.merge().clone();
        let model = super::model::MergeModel::from_disk(merge, text);
        self.replace_model(model);
    }

    // -- saving ----------------------------------------------------------------

    /// The link request that writes the Output. `None` when there is no
    /// text to write (a card, in a later slice).
    pub(crate) fn save_request(&self, stage: bool) -> super::SaveRequest {
        let file = self.model.output_file();
        let resolution = match &file.encoding {
            Some(label) => ConflictResolution::UseEncoded {
                text: file.content_string(),
                encoding: label.clone(),
            },
            None => ConflictResolution::UseContent(file.content_string()),
        };
        super::SaveRequest {
            path: self.path().to_path_buf(),
            resolution,
            stage,
            expected: ResolveOptions::oids_of(&self.model.merge().entry),
        }
    }

    // -- the mouse ---------------------------------------------------------------

    /// A left click on a hit rect the paint walk registered.
    pub(crate) fn click(&mut self, hit: MergeHit, col: u16, row: u16) -> Outcome {
        self.note = None;
        // A box owns the pointer: its own rows work, anything else only
        // closes a menu.
        match hit {
            MergeHit::PromptSave if self.ui.prompt.is_some() => {
                return self.prompt_answer(PromptAnswer::Primary)
            }
            MergeHit::PromptDiscard if self.ui.prompt.is_some() => {
                return self.prompt_answer(PromptAnswer::Secondary)
            }
            MergeHit::PromptCancel | MergeHit::PromptClose if self.ui.prompt.is_some() => {
                return self.prompt_answer(PromptAnswer::Cancel)
            }
            MergeHit::MenuRow(i) if self.ui.menu.is_some() => return self.run_menu_row(i),
            _ => {}
        }
        if self.ui.prompt.is_some() {
            return Outcome::default();
        }
        if self.ui.menu.take().is_some() {
            return Outcome::default();
        }
        match hit {
            MergeHit::Close => self.run(MergeCmd::Close),
            MergeHit::PrevFile => self.run(MergeCmd::PrevFile),
            MergeHit::NextFile => self.run(MergeCmd::NextFile),
            MergeHit::PrevConflict => self.run(MergeCmd::PrevConflict),
            MergeHit::NextConflict => self.run(MergeCmd::NextConflict),
            MergeHit::TakeAllA => self.run(MergeCmd::TakeAllA),
            MergeHit::TakeAllB => self.run(MergeCmd::TakeAllB),
            MergeHit::ResetAll => self.run(MergeCmd::Reset),
            MergeHit::ConflictMark(c) => {
                if self.model.go_to(c) {
                    self.ui.cursor = 0;
                    self.reveal_current();
                }
                Outcome::default()
            }
            MergeHit::ToggleBase => self.run(MergeCmd::ToggleBase),
            MergeHit::ToggleFullFile => self.run(MergeCmd::ToggleFullFile),
            MergeHit::AiSuggest => self.run(MergeCmd::AiSuggest),
            MergeHit::More => {
                self.open_more_menu((col, row + 1));
                Outcome::default()
            }
            MergeHit::Save => self.run(MergeCmd::Save),
            MergeHit::SaveResolve => self.run(MergeCmd::SaveResolve),
            MergeHit::Tab(side) => {
                self.focus_pane(MergePane::of_side(side));
                Outcome::default()
            }
            MergeHit::Chunk { conflict, side } => {
                self.focus_pane(MergePane::of_side(side));
                self.model.go_to(conflict);
                self.ui.cursor = 0;
                self.pick(|m| m.toggle_chunk(conflict, side))
            }
            MergeHit::Line {
                conflict,
                side,
                line,
            } => {
                self.focus_pane(MergePane::of_side(side));
                self.model.go_to(conflict);
                self.ui.cursor = line;
                self.pick(|m| m.toggle_line(conflict, side, line))
            }
            MergeHit::Badge { conflict } => {
                self.model.go_to(conflict);
                self.run(MergeCmd::SwapOrder)
            }
            MergeHit::OutputLine(line) => {
                self.focus_pane(MergePane::Output);
                let gutter = self.areas.gutter;
                let x = col.saturating_sub(self.areas.out.x + gutter) as usize;
                let doc = self.model.output_for_caret();
                let at = doc.offset_at(line, x);
                doc.set_cursor(at, false);
                self.ui.placeholder = None;
                self.model.follow_caret();
                Outcome::default()
            }
            MergeHit::Placeholder { conflict } => {
                self.model.go_to(conflict);
                self.run(MergeCmd::EditInOutput)
            }
            MergeHit::StripPrimary => match self.ui.strip {
                Some(Strip::EditedOutside { .. }) => self.run(MergeCmd::ContinueFromDisk),
                Some(_) => self.run(MergeCmd::Close),
                None => Outcome::default(),
            },
            MergeHit::StripSecondary => match self.ui.strip {
                Some(Strip::EditedOutside { .. }) => self.run(MergeCmd::StartOver),
                Some(Strip::Resolved) => self.run(MergeCmd::Unresolve),
                Some(Strip::Changed) => self.run(MergeCmd::Reload),
                None => Outcome::default(),
            },
            // A drag starts on the press ([`MergeView::begin_drag`]).
            MergeHit::Splitter | MergeHit::Divider(_) | MergeHit::Thumb(_) | MergeHit::Bar(_) => {
                Outcome::default()
            }
            MergeHit::Choice(i) => self.card_choice(i),
            MergeHit::MenuRow(_)
            | MergeHit::PromptSave
            | MergeHit::PromptDiscard
            | MergeHit::PromptCancel
            | MergeHit::PromptClose => Outcome::default(),
        }
    }

    /// A right click: the context menu of a conflict.
    pub(crate) fn right_click(&mut self, hit: MergeHit, col: u16, row: u16) -> Outcome {
        self.note = None;
        if self.ui.prompt.is_some() {
            return Outcome::default();
        }
        self.ui.menu = None;
        let conflict = match hit {
            MergeHit::Chunk { conflict, .. }
            | MergeHit::Line { conflict, .. }
            | MergeHit::Badge { conflict }
            | MergeHit::Placeholder { conflict } => Some(conflict),
            _ => None,
        };
        if let Some(c) = conflict {
            self.open_chunk_menu(c, (col, row));
        }
        Outcome::default()
    }

    /// A left press on an Output line: the caret goes there, Shift
    /// extends the selection, a double click selects the word, and a drag
    /// from here selects (see [`MergeView::on_mouse`]).
    pub(crate) fn output_press(
        &mut self,
        line: usize,
        modifiers: KeyModifiers,
        col: u16,
        row: u16,
    ) -> bool {
        if self.ui.modal() {
            return false;
        }
        self.note = None;
        self.focus_pane(MergePane::Output);
        let x = col.saturating_sub(self.areas.out.x + self.areas.gutter) as usize;
        let shift = modifiers.contains(KeyModifiers::SHIFT);
        let doc = self.model.output_for_caret();
        let at = doc.offset_at(line, x);
        if text_press(&mut self.ui.last_click, doc, at, (col, row), shift) {
            self.ui.drag = Some(Drag::Text);
        }
        self.ui.placeholder = None;
        self.model.follow_caret();
        true
    }

    /// The drag of a selection: to the cell under the pointer, the Output
    /// scrolling when the pointer is above or below it.
    fn drag_text(&mut self, col: u16, row: u16) {
        let out = self.areas.out;
        if out.height == 0 {
            return;
        }
        if row < out.y {
            self.scroll_output_by(-1);
        } else if row >= out.bottom() {
            self.scroll_output_by(1);
        }
        let r = row.clamp(out.y, out.bottom() - 1) - out.y;
        let layout = self.out_layout();
        let last = self.model.output().line_count().saturating_sub(1);
        let line = match layout.row_at(self.out_scroll + r as usize) {
            Some(super::layout::ORow::Line(i)) => i,
            Some(super::layout::ORow::Placeholder(c)) => {
                self.model.placeholder_line(c).unwrap_or(last)
            }
            _ => last,
        };
        let x = col.saturating_sub(out.x + self.areas.gutter) as usize;
        let doc = self.model.output_for_caret();
        let at = doc.offset_at(line.min(last), x);
        doc.set_cursor(at, true);
        self.model.follow_caret();
    }

    /// A press on a splitter or a scrollbar thumb starts a drag.
    pub(crate) fn begin_drag(&mut self, hit: MergeHit, row: u16) -> bool {
        match hit {
            MergeHit::Splitter | MergeHit::Divider(_) => {
                // A second press on the same handle is a double click:
                // back to the default size, no drag.
                let now = std::time::Instant::now();
                let double = self.ui.last_press.is_some_and(|(t, h)| {
                    h == hit && now.duration_since(t) < crate::pane_layout::DOUBLE_CLICK_WINDOW
                });
                if double {
                    self.ui.last_press = None;
                    if hit == MergeHit::Splitter {
                        self.split = Split::DEFAULT;
                    } else {
                        self.cols = Default::default();
                    }
                    return true;
                }
                self.ui.last_press = Some((now, hit));
                self.ui.drag = Some(match hit {
                    MergeHit::Divider(i) => Drag::Divider(i),
                    _ => Drag::Splitter,
                });
                true
            }
            MergeHit::Thumb(which) | MergeHit::Bar(which) => {
                self.press_bar(which, row);
                true
            }
            _ => false,
        }
    }

    /// The wheel over a pane, and a drag in progress. Returns whether the
    /// event was the body's.
    pub(crate) fn on_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if !matches!(kind, MouseEventKind::Moved) {
            self.note_input();
        }
        if self.ui.modal() {
            return matches!(kind, MouseEventKind::ScrollUp | MouseEventKind::ScrollDown);
        }
        match kind {
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                let d = if kind == MouseEventKind::ScrollUp {
                    -WHEEL_LINES
                } else {
                    WHEEL_LINES
                };
                let at = (col, row).into();
                if self.areas.top.contains(at) {
                    self.scroll_top_by(d);
                    true
                } else if self.areas.out.contains(at) || self.areas.out_title.contains(at) {
                    self.scroll_output_by(d);
                    true
                } else {
                    false
                }
            }
            MouseEventKind::Drag(_) => {
                match self.ui.drag {
                    Some(Drag::Splitter) => self.drag_splitter(row),
                    Some(Drag::Divider(i)) => self.drag_divider(i, col),
                    Some(Drag::Thumb(which, grab)) => self.drag_thumb(which, grab, row),
                    Some(Drag::Text) => self.drag_text(col, row),
                    None => return false,
                }
                true
            }
            MouseEventKind::Up(_) => self.ui.drag.take().is_some(),
            _ => false,
        }
    }

    /// What `Ctrl+N` badges: the panes in badge order (A, B, Output, then
    /// Base), each with the cell its badge goes on. From the last paint.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, MergePane)> {
        let a = self.areas;
        if a.all.width == 0 || self.card.is_some() {
            return Vec::new();
        }
        let mut panes: Vec<(Rect, MergePane)> = Vec::new();
        if let Some((sides, widths)) = self.painted_panes() {
            let mut x = a.pane_x0;
            for (side, w) in sides.iter().zip(widths) {
                panes.push((Rect::new(x, a.top.y, w, 1), MergePane::of_side(*side)));
                x += w + 1;
            }
        } else {
            for (i, pane) in [MergePane::A, MergePane::B, MergePane::Base]
                .into_iter()
                .enumerate()
            {
                panes.push((Rect::new(a.top.x + 3 * i as u16, a.top.y, 3, 1), pane));
            }
        }
        let rank = |p: MergePane| match p {
            MergePane::A => 0,
            MergePane::B => 1,
            MergePane::Output => 2,
            MergePane::Base => 3,
        };
        panes.push((
            Rect::new(a.out.x, a.out.y, a.out.width, 1),
            MergePane::Output,
        ));
        panes.sort_by_key(|(_, p)| rank(*p));
        panes
    }

    /// The painted pane widths, and the sides they belong to.
    fn painted_panes(&self) -> Option<(Vec<ConflictSide>, Vec<u16>)> {
        let n = usize::from(self.areas.panes);
        let sides = self.wide_sides();
        (n >= 2 && n == sides.len()).then(|| (sides, self.areas.pane_w[..n].to_vec()))
    }

    /// Moves divider `i` so that the pane left of it ends at column `col`.
    fn drag_divider(&mut self, i: u8, col: u16) {
        let Some((sides, widths)) = self.painted_panes() else {
            return;
        };
        let i = usize::from(i);
        let start = i32::from(self.areas.pane_x0)
            + widths[..i].iter().map(|w| i32::from(*w) + 1).sum::<i32>();
        self.cols = self
            .cols
            .with_divider(&sides, &widths, i, i32::from(col) - start);
        self.refresh_pane_widths(&sides, &widths);
    }

    /// Keeps the painted widths current until the next paint, so keys
    /// that arrive in one burst each step from the last one's result.
    fn refresh_pane_widths(&mut self, sides: &[ConflictSide], widths: &[u16]) {
        let usable: u16 = widths.iter().sum();
        for (slot, w) in self
            .areas
            .pane_w
            .iter_mut()
            .zip(self.cols.widths(usable, sides))
        {
            *slot = w;
        }
    }

    /// The keys: pane `side` (A or B) takes a step more of the width from
    /// its neighbor.
    fn widen_side(&mut self, side: ConflictSide) -> Outcome {
        let Some((sides, widths)) = self.painted_panes() else {
            return Outcome::say(tr::MSG_PANES_TABBED.get());
        };
        let usable: u32 = widths.iter().map(|w| u32::from(*w)).sum();
        let step = ((usable * super::layout::KEY_STEP_PERMILLE / 1000).max(2)) as i32;
        let Some(pos) = sides.iter().position(|s| *s == side) else {
            return Outcome::default();
        };
        // A takes from its right neighbor, B from its left one.
        let (div, left) = if pos == 0 {
            (0, i32::from(widths[0]) + step)
        } else {
            (pos - 1, i32::from(widths[pos - 1]) - step)
        };
        self.cols = self.cols.with_divider(&sides, &widths, div, left);
        self.refresh_pane_widths(&sides, &widths);
        Outcome::default()
    }

    fn drag_splitter(&mut self, row: u16) {
        let a = self.areas;
        let avail = u32::from(a.top.height) + u32::from(a.out.height);
        if avail == 0 {
            return;
        }
        // The title row is where the pointer is; the Output's content is
        // what lies below it.
        let out_h = u32::from(a.out.bottom().saturating_sub(row + 1));
        let pct = (out_h * 100 / avail).clamp(10, 85) as u16;
        self.split = Split::Percent(pct);
    }

    /// The bar `which` as last painted, with its content length.
    fn bar_of(&self, which: MergeScroll) -> Option<(crate::scrollbar::Track, usize)> {
        let (slot, len) = match which {
            MergeScroll::Top => (0, self.top.rows.len()),
            MergeScroll::Output => (1, self.out_layout().total()),
        };
        self.ui.bars.get()[slot].map(|t| (t, len))
    }

    fn scroll_bar_by(&mut self, which: MergeScroll, delta: isize) {
        match which {
            MergeScroll::Top => self.scroll_top_by(delta),
            MergeScroll::Output => self.scroll_output_by(delta),
        }
    }

    /// A press on a scrollbar: an end arrow steps one line, the thumb is
    /// grabbed where it is held, the track jumps the thumb under the
    /// pointer and keeps dragging (the lists' behavior, `ui_kit`).
    fn press_bar(&mut self, which: MergeScroll, row: u16) {
        let Some((track, _)) = self.bar_of(which) else {
            return;
        };
        let pos = row.saturating_sub(track.rect.y);
        if let Some(step) = track.arrow_step(pos) {
            self.scroll_bar_by(which, step);
        } else if track.on_thumb(pos) {
            let grab = i32::from(pos) - i32::from(track.thumb_start);
            self.ui.drag = Some(Drag::Thumb(which, grab));
        } else {
            let grab = i32::from(track.thumb_len / 2);
            self.ui.drag = Some(Drag::Thumb(which, grab));
            self.drag_thumb(which, grab, row);
        }
    }

    fn drag_thumb(&mut self, which: MergeScroll, grab: i32, row: u16) {
        let Some((track, len)) = self.bar_of(which) else {
            return;
        };
        let view = usize::from(track.rect.height);
        let pos = i32::from(row) - i32::from(track.rect.y);
        let want = track.drag_offset(len, view, grab, pos);
        let cur = match which {
            MergeScroll::Top => self.top_scroll,
            MergeScroll::Output => self.out_scroll,
        };
        self.scroll_bar_by(which, want as isize - cur as isize);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PromptAnswer {
    Primary,
    Secondary,
    Cancel,
}

/// A control's label with the AI sparkle in front (maintainer rule 2).
pub(super) fn ai_label(label: &str) -> String {
    format!("{} {label}", crate::theme::AI_SPARK)
}

/// The command a keymap action stands for.
fn cmd_of(action: Action) -> Option<MergeCmd> {
    Some(match action {
        Action::MergeNextConflict | Action::MergeSideNextConflict => MergeCmd::NextConflict,
        Action::MergePrevConflict | Action::MergeSidePrevConflict => MergeCmd::PrevConflict,
        Action::MergeNextFile => MergeCmd::NextFile,
        Action::MergePrevFile => MergeCmd::PrevFile,
        Action::MergeNextPane => MergeCmd::FocusNext,
        Action::MergePrevPane => MergeCmd::FocusPrev,
        Action::MergeToggleBase | Action::MergeSideToggleBase => MergeCmd::ToggleBase,
        Action::MergeTogglePanes => MergeCmd::TogglePanes,
        Action::MergeScrollLeft => MergeCmd::ScrollLeft,
        Action::MergeScrollRight => MergeCmd::ScrollRight,
        Action::MergeToggleFullFile | Action::MergeSideToggleFullFile => MergeCmd::ToggleFullFile,
        Action::MergeUndo => MergeCmd::Undo,
        Action::MergeRedo => MergeCmd::Redo,
        Action::MergeSave => MergeCmd::Save,
        Action::MergeSaveResolve => MergeCmd::SaveResolve,
        Action::MergeAiSuggest => MergeCmd::AiSuggest,
        Action::MergeTakeA => MergeCmd::TakeA,
        Action::MergeTakeB => MergeCmd::TakeB,
        Action::MergeTakeBoth => MergeCmd::TakeBoth,
        Action::MergeTakeBothReversed => MergeCmd::TakeBothReversed,
        Action::MergeClear => MergeCmd::Clear,
        Action::MergeSwapOrder => MergeCmd::SwapOrder,
        Action::MergeTakeAllA => MergeCmd::TakeAllA,
        Action::MergeTakeAllB => MergeCmd::TakeAllB,
        Action::MergeCycleHeight => MergeCmd::CycleHeight,
        Action::MergeWidenCurrent => MergeCmd::WidenCurrent,
        Action::MergeWidenIncoming => MergeCmd::WidenIncoming,
        Action::MergeResetPanes => MergeCmd::ResetPaneSizes,
        Action::MergeUnpick | Action::MergeSideUnpick => MergeCmd::Unpick,
        Action::MergeResetAll | Action::MergeSideResetAll => MergeCmd::Reset,
        Action::MergeFocusA => MergeCmd::FocusPane(MergePane::A),
        Action::MergeFocusB => MergeCmd::FocusPane(MergePane::B),
        Action::MergeFocusOutput => MergeCmd::FocusPane(MergePane::Output),
        Action::MergeFocusBase => MergeCmd::FocusPane(MergePane::Base),
        _ => return None,
    })
}
