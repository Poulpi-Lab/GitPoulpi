//! The workspace form's "AI instructions" popup: what the AI is told in
//! every repository of the workspace, for all actions or for one, and how
//! it combines with the global instructions (add after them, or replace
//! them). A workspace's words sit between the global and the repository
//! ones (`gitpoulpi_ai::prompts::system_prompt_layered`).
//!
//! One text area, switched between the "every action" text and each action's
//! by a combobox; the form keeps the whole set and writes it with the
//! workspace. The same reading as the repository's AI settings: empty text
//! means "nothing here".

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_config::{AiAction, RepoAiInstructions, RepoInstructionsMode, WorkspaceAi};
use gitpoulpi_i18n::msg::workspace_ai as m;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::combo::{Combo, ComboOption, ComboOutcome};
use crate::dialog_kit::{self, ButtonState};
use crate::keymap::Action as Bind;
use crate::text_edit::{FieldEditor, KeyEdit};
use crate::th;

/// An action's name in the UI language.
pub(crate) fn action_label(action: AiAction) -> &'static str {
    use gitpoulpi_i18n::msg::ai_page as a;
    match action {
        AiAction::CommitMessage => a::ACTION_COMMIT_MESSAGE,
        AiAction::ExplainChanges => a::ACTION_EXPLAIN,
        AiAction::StashMessage => a::ACTION_STASH,
        AiAction::PullRequestDescription => a::ACTION_PULL_REQUEST,
        AiAction::CommitComposer => a::ACTION_COMPOSER,
        AiAction::ConflictResolve => a::ACTION_CONFLICT_RESOLVE,
        AiAction::CodeReview => gitpoulpi_i18n::msg::review_ai::ACTION_CODE_REVIEW,
        AiAction::SplitCommit => a::ACTION_SPLIT_COMMIT,
        AiAction::RewriteScript => gitpoulpi_i18n::msg::rewrite_ai::ACTION,
        AiAction::RewriteRules => gitpoulpi_i18n::msg::rewrite_rules_ai::ACTION,
        AiAction::WorktreeName => a::ACTION_WORKTREE_NAME,
    }
    .get()
}

/// How many instruction texts `ai` holds (the form's one-line summary).
pub(crate) fn count_sets(ai: &WorkspaceAi) -> usize {
    usize::from(!ai.general.text.trim().is_empty())
        + ai.actions
            .values()
            .filter(|a| !a.text.trim().is_empty())
            .count()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum AiBoxOutcome {
    Stay,
    Cancel,
    Apply(WorkspaceAi),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Part {
    Scope,
    Mode,
    Text,
    Cancel,
    Apply,
}

const ORDER: [Part; 5] = [
    Part::Scope,
    Part::Mode,
    Part::Text,
    Part::Cancel,
    Part::Apply,
];

pub(super) struct AiBox {
    scope: Combo,
    mode: Combo,
    /// The whole set; `current`'s entry is stale while it is being edited
    /// ([`Self::commit`] writes it back).
    draft: WorkspaceAi,
    /// `""` = every action, else an action key.
    current: String,
    text: String,
    edit: FieldEditor<()>,
    bar: Option<crate::scrollbar::Track>,
    drag: crate::scrollbar::BarDrag,
    pub(super) focus: Part,
    hover: Option<Part>,
    hits: Vec<(Rect, Part)>,
    /// The close cross on the frame.
    close: dialog_kit::CloseHit,
}

fn mode_value(mode: RepoInstructionsMode) -> &'static str {
    match mode {
        RepoInstructionsMode::Add => "add",
        RepoInstructionsMode::Replace => "replace",
    }
}

impl AiBox {
    pub(super) fn new(ai: WorkspaceAi) -> Self {
        let mut scope = Combo::new("");
        let mut options = vec![ComboOption::new(m::SCOPE_ALL.get(), "")];
        options.extend(
            AiAction::ALL
                .iter()
                .map(|a| ComboOption::new(action_label(*a), a.key())),
        );
        scope.set_options(options);
        scope.select(Some(String::new()));
        let mut mode = Combo::new("");
        mode.set_options(vec![
            ComboOption::new(m::MODE_ADD.get(), "add"),
            ComboOption::new(m::MODE_REPLACE.get(), "replace"),
        ]);
        let mut me = Self {
            scope,
            mode,
            draft: ai,
            current: String::new(),
            text: String::new(),
            edit: FieldEditor::default(),
            bar: None,
            drag: Default::default(),
            focus: Part::Text,
            hover: None,
            close: dialog_kit::CloseHit::default(),
            hits: Vec::new(),
        };
        me.load("");
        me
    }

    fn entry(&self, key: &str) -> RepoAiInstructions {
        if key.is_empty() {
            self.draft.general.clone()
        } else {
            self.draft.actions.get(key).cloned().unwrap_or_default()
        }
    }

    /// Shows the text and mode held for `key`.
    fn load(&mut self, key: &str) {
        let entry = self.entry(key);
        self.current = key.to_string();
        self.text = entry.text;
        self.mode.select(Some(mode_value(entry.mode).to_string()));
        self.edit.sync(Some(()));
    }

    /// Writes the text and mode on screen into the draft.
    fn commit(&mut self) {
        let entry = RepoAiInstructions {
            text: self.text.trim().to_string(),
            mode: match self.mode.selected() {
                Some("replace") => RepoInstructionsMode::Replace,
                _ => RepoInstructionsMode::Add,
            },
        };
        if self.current.is_empty() {
            self.draft.general = entry;
        } else if entry.text.is_empty() {
            self.draft.actions.remove(&self.current);
        } else {
            self.draft.actions.insert(self.current.clone(), entry);
        }
    }

    /// The scope combo moved: keep what was typed, show the other text.
    fn follow_scope(&mut self) {
        let wanted = self.scope.selected().unwrap_or_default().to_string();
        if wanted != self.current {
            self.commit();
            self.load(&wanted);
        }
    }

    fn open_combo(&mut self) -> Option<&mut Combo> {
        [&mut self.scope, &mut self.mode]
            .into_iter()
            .find(|c| c.is_open())
    }

    fn apply(&mut self) -> AiBoxOutcome {
        self.commit();
        AiBoxOutcome::Apply(self.draft.clone())
    }

    pub(super) fn paste(&mut self, text: &str) {
        if let Some(c) = self.open_combo() {
            c.paste(text);
            return;
        }
        if self.focus != Part::Text {
            return;
        }
        let clean = text.replace("\r\n", "\n").replace('\r', "\n");
        self.edit.sync(Some(()));
        let mut value = std::mem::take(&mut self.text);
        self.edit.caret_mut(()).insert(&mut value, &clean);
        self.text = value;
    }

    pub(super) fn on_key(&mut self, key: KeyEvent) -> AiBoxOutcome {
        if let Some(c) = self.open_combo() {
            c.on_key(key);
            self.follow_scope();
            return AiBoxOutcome::Stay;
        }
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if self.focus == Part::Text {
            self.edit.sync(Some(()));
            if key.code == KeyCode::Enter {
                let mut value = std::mem::take(&mut self.text);
                self.edit.caret_mut(()).insert(&mut value, "\n");
                self.text = value;
                return AiBoxOutcome::Stay;
            }
            if self.edit.claims((), &self.text, &key, true) {
                let mut value = std::mem::take(&mut self.text);
                let edit = self.edit.on_key((), &mut value, &key, true);
                self.text = value;
                if let KeyEdit::Copy(s) | KeyEdit::Cut(s) = edit {
                    crate::clipboard::copy_shared(
                        gitpoulpi_i18n::msg::review_view::NOUN_SELECTION.get(),
                        &s,
                    );
                }
                return AiBoxOutcome::Stay;
            }
        }
        let combo_focus = matches!(self.focus, Part::Scope | Part::Mode);
        if combo_focus && !matches!(key.code, KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc) {
            let combo = if self.focus == Part::Scope {
                &mut self.scope
            } else {
                &mut self.mode
            };
            if combo.on_key(key) == ComboOutcome::Handled {
                self.follow_scope();
                return AiBoxOutcome::Stay;
            }
        }
        match key.code {
            KeyCode::Esc => return AiBoxOutcome::Cancel,
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                let at = ORDER.iter().position(|p| *p == self.focus).unwrap_or(0);
                let n = ORDER.len();
                self.focus = ORDER[if forward {
                    (at + 1) % n
                } else {
                    (at + n - 1) % n
                }];
            }
            _ if crate::keymap::current().is(Bind::DialogSubmit, &key) => return self.apply(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                return match self.focus {
                    Part::Apply => self.apply(),
                    Part::Cancel => AiBoxOutcome::Cancel,
                    _ => AiBoxOutcome::Stay,
                }
            }
            _ => {}
        }
        AiBoxOutcome::Stay
    }

    pub(super) fn on_mouse(&mut self, mouse: MouseEvent) -> AiBoxOutcome {
        if let Some(c) = self.open_combo() {
            c.on_mouse(mouse);
            self.follow_scope();
            return AiBoxOutcome::Stay;
        }
        let top = self.edit.caret(()).map_or(0, |c| c.top.get());
        let lines = self.text.split('\n').count();
        if let Some(top) = self.drag.on_mouse(self.bar, mouse, lines, top) {
            let rows = self.bar.map_or(1, |t| t.rect.height as usize);
            self.edit
                .caret_mut(())
                .scroll_lines_to(&self.text, top, rows);
            return AiBoxOutcome::Stay;
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        if matches!(mouse.kind, MouseEventKind::Moved) {
            self.close.set_hover(mouse.column, mouse.row);
        }
        if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left))
            && self.close.hit(mouse.column, mouse.row)
        {
            return AiBoxOutcome::Cancel;
        }
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.edit.is_dragging() {
            self.edit
                .dialog_mouse(Some(()), &self.text, mouse.kind, mouse.column, mouse.row);
            return AiBoxOutcome::Stay;
        }
        match mouse.kind {
            MouseEventKind::Moved => self.hover = hit,
            MouseEventKind::Down(MouseButton::Left) => match hit {
                Some(Part::Text) => {
                    self.focus = Part::Text;
                    self.edit.sync(Some(()));
                    self.edit.dialog_mouse(
                        Some(()),
                        &self.text,
                        mouse.kind,
                        mouse.column,
                        mouse.row,
                    );
                }
                Some(Part::Scope) => {
                    self.focus = Part::Scope;
                    self.scope.on_mouse(mouse);
                }
                Some(Part::Mode) => {
                    self.focus = Part::Mode;
                    self.mode.on_mouse(mouse);
                }
                Some(Part::Apply) => return self.apply(),
                Some(Part::Cancel) => return AiBoxOutcome::Cancel,
                None => {}
            },
            MouseEventKind::Up(MouseButton::Left) => {
                self.edit
                    .dialog_mouse(Some(()), &self.text, mouse.kind, mouse.column, mouse.row);
            }
            _ => {}
        }
        AiBoxOutcome::Stay
    }

    pub(super) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let rect = WIN.place(area, dialog_kit::centered(area, 80, 20));
        self.edit.begin_frame();
        self.close.framed(buf, rect, m::BOX_TITLE.get(), &WIN, true);
        let inner = Rect::new(
            rect.x + 2,
            rect.y + 1,
            rect.width.saturating_sub(4),
            rect.height.saturating_sub(2),
        );
        let mut hits: Vec<(Rect, Part)> = Vec::new();
        if inner.width < 30 || inner.height < 10 {
            self.hits = hits;
            return;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let ai = Style::default().fg(th().status.ai).bg(th().ui.panel);
        let mut y = inner.y;
        dialog_kit::put(buf, inner.x, y, crate::theme::AI_SPARK, 1, ai);
        dialog_kit::put(
            buf,
            inner.x + 2,
            y,
            m::BOX_HELP.get(),
            inner.width.saturating_sub(2),
            muted,
        );
        y += 2;
        let label_w: u16 = 12;
        let combo_w = inner.width.saturating_sub(label_w).min(36);
        dialog_kit::put(buf, inner.x, y, m::SCOPE.get(), label_w, muted);
        let scope = Rect::new(inner.x + label_w, y, combo_w, 1);
        self.scope.render_field(
            buf,
            scope,
            self.focus == Part::Scope,
            self.hover == Some(Part::Scope),
        );
        hits.push((scope, Part::Scope));
        y += 1;
        dialog_kit::put(buf, inner.x, y, m::MODE.get(), label_w, muted);
        let mode = Rect::new(inner.x + label_w, y, combo_w, 1);
        self.mode.render_field(
            buf,
            mode,
            self.focus == Part::Mode,
            self.hover == Some(Part::Mode),
        );
        hits.push((mode, Part::Mode));
        y += 2;
        let button_y = inner.bottom().saturating_sub(1);
        let text = Rect::new(
            inner.x,
            y,
            inner.width,
            button_y.saturating_sub(y + 1).max(2),
        );
        self.bar = dialog_kit::text_area(
            buf,
            text,
            &self.text,
            m::PLACEHOLDER.get(),
            self.focus == Part::Text,
            self.hover == Some(Part::Text),
            (&self.edit, ()),
        );
        let cancel_label = m::CANCEL.get();
        let apply_label = m::APPLY.get();
        let apply_w = dialog_kit::button_width(apply_label);
        let cancel_w = dialog_kit::button_width(cancel_label);
        let apply_x = inner.right().saturating_sub(apply_w);
        let cancel_x = apply_x.saturating_sub(cancel_w + 1);
        let cancel = dialog_kit::button(
            buf,
            cancel_x,
            button_y,
            cancel_label,
            ButtonState::Normal {
                hovered: self.hover == Some(Part::Cancel),
                focused: self.focus == Part::Cancel,
            },
        );
        let apply = dialog_kit::button(
            buf,
            apply_x,
            button_y,
            apply_label,
            ButtonState::Primary {
                hovered: self.hover == Some(Part::Apply),
                focused: self.focus == Part::Apply,
            },
        );
        // Buttons and combos before the text area they do not overlap.
        hits.insert(0, (cancel, Part::Cancel));
        hits.insert(0, (apply, Part::Apply));
        hits.push((text, Part::Text));
        let key = dialog_kit::key_label(Bind::DialogSubmit);
        dialog_kit::hints(
            buf,
            inner.x,
            button_y,
            cancel_x.saturating_sub(inner.x + 1),
            &[(key.as_str(), m::HINT_APPLY.get())],
        );
        self.hits = hits;
        self.scope.render_popup(buf, buf.area);
        self.mode.render_popup(buf, buf.area);
    }
}

/// The dialog resizes: the text area takes the rows it gains.
const WIN: crate::dialog_kit::Win = crate::dialog_kit::Win::resizable("workspace_ai", 50, 14);
