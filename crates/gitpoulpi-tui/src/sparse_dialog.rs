//! The Sparse checkout dialog ("Repository: Sparse Checkout..."): see and
//! edit which directories of the repository are checked out.
//!
//! ```text
//! ╭ Sparse checkout ─────────────────────────────────────────────╮
//! │ Sparse checkout is on: only the directories below, and the   │
//! │ files at the root, are checked out.                          │
//! │                                                              │
//! │  Directories   src docs/guide█                               │
//! │                Separated by spaces or commas, relative to... │
//! │                                                              │
//! │ Tab next · Enter apply   [ Disable (full checkout) ] [ Apply ] │
//! ╰──────────────────────────────────────────────────────────────╯
//! ```
//!
//! Pure: validation, keys, clicks and drawing. The dialog closes on
//! submit and the daemon answers on the status line, like Create
//! worktree: [`SparseOutcome::Apply`] / [`SparseOutcome::Disable`]
//! become an `Action` in the event loop. Editing the field is how
//! directories are added and removed (the whole set is replaced); Disable is
//! the way back to a full checkout.

use crate::th;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::domain::{normalize_sparse_dirs, SparseState};
use gitpoulpi_i18n::msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::dialog_kit::{self, ButtonState};
use crate::keymap::chord::Key;

/// The directories typed in a field (separated by spaces or commas), or
/// why they cannot be a sparse checkout. Shared by this dialog and the
/// Clone dialog's field.
pub(crate) fn parse_dirs(text: &str) -> Result<Vec<String>, String> {
    let tokens: Vec<String> = text
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect();
    if tokens.is_empty() {
        return Err(msg::sparse::NEED_DIR.get().into());
    }
    for token in &tokens {
        if normalize_sparse_dirs(std::slice::from_ref(token)).is_err() {
            return Err(msg::sparse::bad_dir(format!("{token:?}")));
        }
    }
    normalize_sparse_dirs(&tokens).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SparseFocus {
    Dirs,
    Apply,
    Disable,
    Cancel,
}

const ORDER: [SparseFocus; 4] = [
    SparseFocus::Dirs,
    SparseFocus::Apply,
    SparseFocus::Disable,
    SparseFocus::Cancel,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SparseOutcome {
    Stay,
    Cancel,
    /// Make the checkout exactly these directories.
    Apply(Vec<String>),
    /// Back to a full checkout.
    Disable,
}

#[derive(Debug, Clone)]
pub(crate) struct SparseDialog {
    /// The state when the dialog opened.
    pub state: SparseState,
    /// The directories field.
    pub dirs: String,
    pub focus: SparseFocus,
    pub message: Option<String>,
    pub hover: Option<SparseFocus>,
    hits: Vec<(Rect, SparseFocus)>,
    close: dialog_kit::CloseHit,
    edit: crate::text_edit::FieldEditor<SparseFocus>,
}

impl SparseDialog {
    /// What `Ctrl+N` badges: the directories box and the buttons.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        dialog_kit::nav_cells(self.hits.iter().map(|(r, _)| *r))
    }

    pub(crate) fn new(state: SparseState) -> Self {
        let dirs = if state.enabled {
            state.dirs.join(" ")
        } else {
            String::new()
        };
        Self {
            state,
            dirs,
            focus: SparseFocus::Dirs,
            message: None,
            hover: None,
            hits: Vec::new(),
            close: Default::default(),
            edit: Default::default(),
        }
    }

    /// What Apply would send, or why it cannot.
    pub(crate) fn plan(&self) -> Result<Vec<String>, String> {
        parse_dirs(&self.dirs)
    }

    fn field_parts(
        &mut self,
    ) -> (
        &mut crate::text_edit::FieldEditor<SparseFocus>,
        Option<(SparseFocus, &mut String)>,
    ) {
        let focus = self.focus;
        let field = (focus == SparseFocus::Dirs).then_some(&mut self.dirs);
        self.edit.sync(field.is_some().then_some(focus));
        (&mut self.edit, field.map(|f| (focus, f)))
    }

    pub(crate) fn paste(&mut self, text: &str) {
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        self.focus = SparseFocus::Dirs;
        if let (edit, Some((id, field))) = self.field_parts() {
            edit.caret_mut(id).insert(field, &clean);
        }
        self.message = None;
    }

    fn apply(&mut self) -> SparseOutcome {
        match self.plan() {
            Ok(dirs) => SparseOutcome::Apply(dirs),
            Err(reason) => {
                self.message = Some(reason);
                SparseOutcome::Stay
            }
        }
    }

    fn activate(&mut self, part: SparseFocus) -> SparseOutcome {
        match part {
            SparseFocus::Cancel => SparseOutcome::Cancel,
            SparseFocus::Disable => {
                if self.state.enabled {
                    SparseOutcome::Disable
                } else {
                    SparseOutcome::Stay
                }
            }
            SparseFocus::Dirs | SparseFocus::Apply => self.apply(),
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> SparseOutcome {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if let (edit, Some((id, field))) = self.field_parts() {
            if let Some((result, copied)) = edit.dialog_key(id, field, &key) {
                if result == crate::text_edit::KeyEdit::Edited {
                    self.message = None;
                }
                if copied.is_some() {
                    self.message = copied;
                }
                return SparseOutcome::Stay;
            }
        }
        match key.code {
            KeyCode::Esc => return SparseOutcome::Cancel,
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                let at = ORDER.iter().position(|f| *f == self.focus).unwrap_or(0);
                let next = if forward {
                    (at + 1) % ORDER.len()
                } else {
                    (at + ORDER.len() - 1) % ORDER.len()
                };
                self.focus = ORDER[next];
            }
            KeyCode::Enter => return self.activate(self.focus),
            KeyCode::Char(' ') if self.focus != SparseFocus::Dirs => {
                return self.activate(self.focus)
            }
            _ => {}
        }
        SparseOutcome::Stay
    }

    fn field_mouse(&mut self, mouse: MouseEvent) -> bool {
        match self.field_parts() {
            (edit, Some((id, field))) => {
                edit.dialog_mouse(Some(id), field, mouse.kind, mouse.column, mouse.row)
            }
            (edit, None) => edit.dialog_mouse(None, "", mouse.kind, mouse.column, mouse.row),
        }
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> SparseOutcome {
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.field_mouse(mouse) {
            return SparseOutcome::Stay;
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(part) = hit {
                    self.focus = part;
                    if part != SparseFocus::Dirs {
                        return self.activate(part);
                    }
                    self.field_mouse(mouse);
                }
            }
            _ => {}
        }
        SparseOutcome::Stay
    }

    pub(crate) fn render(&mut self, rect: Rect, buf: &mut Buffer) {
        self.field_parts();
        self.edit.begin_frame();
        self.close
            .framed(buf, rect, msg::sparse::TITLE.get(), &WIN, true);
        let mut hits = Vec::new();
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        if inner.width < 30 || inner.height < 8 {
            dialog_kit::too_small::note(buf, rect);
            self.hits = hits;
            return;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let status = if self.state.enabled {
            msg::sparse::STATUS_ON.get()
        } else {
            msg::sparse::STATUS_OFF.get()
        };
        dialog_kit::put(buf, inner.x, inner.y, status, inner.width, muted);

        let label_w = (crate::display_width(msg::sparse::DIRECTORIES.get()) as u16 + 2).max(14);
        let field_x = inner.x + label_w;
        let field_w = inner.width.saturating_sub(label_w);
        let y = inner.y + 2;
        dialog_kit::put(
            buf,
            inner.x,
            y,
            msg::sparse::DIRECTORIES.get(),
            label_w,
            muted,
        );
        let field = Rect::new(field_x, y, field_w, 1);
        dialog_kit::text_input(
            buf,
            field,
            &self.dirs,
            msg::sparse::DIRECTORIES_HINT.get(),
            self.focus == SparseFocus::Dirs,
            self.hover == Some(SparseFocus::Dirs),
            (&self.edit, SparseFocus::Dirs),
        );
        hits.push((field, SparseFocus::Dirs));
        dialog_kit::put(
            buf,
            field_x + 1,
            y + 1,
            msg::sparse::NOTE.get(),
            field_w.saturating_sub(1),
            muted,
        );

        let bottom = inner.y + inner.height;
        let plan_y = bottom.saturating_sub(3);
        let plan = self.plan();
        if let Err(reason) = &plan {
            dialog_kit::put(
                buf,
                inner.x,
                plan_y,
                reason,
                inner.width,
                Style::default().fg(th().status.warning).bg(th().ui.panel),
            );
        }
        if let Some(message) = &self.message {
            dialog_kit::put(
                buf,
                inner.x,
                plan_y + 1,
                message,
                inner.width,
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            );
        }

        let footer = bottom.saturating_sub(1);
        // Right-aligned; a narrow dialog clips the labels, never drops one.
        use dialog_kit::button_row::Btn;
        let [cancel, disable, apply] = dialog_kit::button_row::paint(
            buf,
            inner.right(),
            footer,
            inner.width,
            0,
            &[
                Btn {
                    label: msg::dialog::CANCEL.get(),
                    state: ButtonState::Normal {
                        hovered: self.hover == Some(SparseFocus::Cancel),
                        focused: self.focus == SparseFocus::Cancel,
                    },
                },
                Btn {
                    label: msg::sparse::DISABLE.get(),
                    state: if self.state.enabled {
                        ButtonState::Normal {
                            hovered: self.hover == Some(SparseFocus::Disable),
                            focused: self.focus == SparseFocus::Disable,
                        }
                    } else {
                        ButtonState::Disabled
                    },
                },
                Btn {
                    label: msg::sparse::APPLY.get(),
                    state: if plan.is_err() {
                        ButtonState::Disabled
                    } else {
                        ButtonState::Primary {
                            hovered: self.hover == Some(SparseFocus::Apply),
                            focused: self.focus == SparseFocus::Apply,
                        }
                    },
                },
            ],
        )[..] else {
            unreachable!("three buttons in, three rects out")
        };
        let cancel_x = cancel.x;
        hits.insert(0, (cancel, SparseFocus::Cancel));
        if self.state.enabled {
            hits.insert(0, (disable, SparseFocus::Disable));
        }
        hits.insert(0, (apply, SparseFocus::Apply));
        let tab = dialog_kit::key_name(Key::Tab);
        let enter = dialog_kit::key_name(Key::Enter);
        let clear = dialog_kit::ctrl_key('u');
        dialog_kit::hints(
            buf,
            inner.x,
            footer,
            cancel_x.saturating_sub(inner.x + 1),
            &[
                (tab.as_str(), msg::dialog::HINT_NEXT.get()),
                (enter.as_str(), msg::sparse::HINT_APPLY.get()),
                (clear.as_str(), msg::dialog::HINT_CLEAR_FIELD.get()),
            ],
        );
        self.hits = hits;
    }
}

/// The dialog's size in a frame.
impl dialog_kit::Dismiss for SparseDialog {
    fn is_dirty(&self) -> bool {
        let opened = if self.state.enabled {
            self.state.dirs.join(" ")
        } else {
            String::new()
        };
        self.dirs != opened
    }

    fn close_hit(&self) -> &dialog_kit::CloseHit {
        &self.close
    }

    fn close_hit_mut(&mut self) -> &mut dialog_kit::CloseHit {
        &mut self.close
    }
}

pub(crate) fn sparse_rect(frame: Rect) -> Rect {
    WIN.place(
        frame,
        dialog_kit::centered(frame, frame.width.saturating_sub(4).min(100), 11),
    )
}

const WIN: dialog_kit::Win = dialog_kit::Win::movable("sparse");

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    fn sparse(dirs: &[&str]) -> SparseState {
        SparseState {
            enabled: true,
            cone: true,
            dirs: dirs.iter().map(|d| d.to_string()).collect(),
        }
    }

    #[test]
    fn parse_dirs_splits_on_spaces_and_commas_and_refuses_junk() {
        assert_eq!(
            parse_dirs("src, docs/guide  lib/").unwrap(),
            ["src", "docs/guide", "lib"]
        );
        assert!(parse_dirs("  ,  ").is_err());
        assert!(parse_dirs("src ../etc").unwrap_err().contains("../etc"));
    }

    #[test]
    fn the_field_starts_with_the_current_directories() {
        let d = SparseDialog::new(sparse(&["src", "docs"]));
        assert_eq!(d.dirs, "src docs");
        assert_eq!(SparseDialog::new(SparseState::default()).dirs, "");
    }

    #[test]
    fn enter_applies_the_edited_set_and_an_empty_one_is_refused() {
        let mut d = SparseDialog::new(sparse(&["src"]));
        d.on_key(key(KeyCode::Char(' ')));
        d.on_key(key(KeyCode::Char('l')));
        d.on_key(key(KeyCode::Char('i')));
        d.on_key(key(KeyCode::Char('b')));
        assert_eq!(
            d.on_key(key(KeyCode::Enter)),
            SparseOutcome::Apply(vec!["src".into(), "lib".into()])
        );
        let mut empty = SparseDialog::new(SparseState::default());
        assert_eq!(empty.on_key(key(KeyCode::Enter)), SparseOutcome::Stay);
        assert!(empty.message.is_some());
    }

    #[test]
    fn disable_is_reachable_by_keyboard_only_when_sparse() {
        let mut d = SparseDialog::new(sparse(&["src"]));
        d.on_key(key(KeyCode::Tab));
        d.on_key(key(KeyCode::Tab));
        assert_eq!(d.focus, SparseFocus::Disable);
        assert_eq!(d.on_key(key(KeyCode::Enter)), SparseOutcome::Disable);
        let mut off = SparseDialog::new(SparseState::default());
        off.focus = SparseFocus::Disable;
        assert_eq!(off.on_key(key(KeyCode::Enter)), SparseOutcome::Stay);
    }

    #[test]
    fn it_draws_state_field_and_buttons_and_registers_their_rects() {
        let mut d = SparseDialog::new(sparse(&["src"]));
        let area = Rect::new(0, 0, 100, 14);
        let mut buf = Buffer::empty(area);
        d.render(sparse_rect(area), &mut buf);
        let text = dialog_kit::buffer_text(&buf);
        assert!(text.contains("Sparse checkout is on"), "{text}");
        assert!(text.contains("src"));
        assert!(text.contains("Disable (full checkout)"));
        // Each drawn button's rect lands on its own glyphs.
        let apply = d
            .hits
            .iter()
            .find(|(_, h)| *h == SparseFocus::Apply)
            .map(|(r, _)| *r)
            .unwrap();
        let row: String = (apply.x..apply.right())
            .map(|x| buf[(x, apply.y)].symbol().to_string())
            .collect();
        assert!(row.contains("Apply"), "{row:?}");
    }

    #[test]
    fn a_window_too_small_for_the_form_says_so() {
        let mut d = SparseDialog::new(sparse(&["src"]));
        let area = Rect::new(0, 0, 40, 8);
        let mut buf = Buffer::empty(area);
        d.render(sparse_rect(area), &mut buf);
        let text = dialog_kit::buffer_text(&buf);
        assert!(text.contains("Window too small"), "{text}");
        assert!(text.contains('\u{2715}'), "{text}");
    }

    #[test]
    fn a_narrow_dialog_keeps_its_three_buttons() {
        let mut d = SparseDialog::new(sparse(&["src"]));
        let area = Rect::new(0, 0, 36, 14);
        let mut buf = Buffer::empty(area);
        d.render(sparse_rect(area), &mut buf);
        let hits: Vec<_> = d.hits.iter().map(|(_, h)| *h).collect();
        for want in [
            SparseFocus::Apply,
            SparseFocus::Disable,
            SparseFocus::Cancel,
        ] {
            assert!(hits.contains(&want), "{want:?} missing: {hits:?}");
        }
    }
}
