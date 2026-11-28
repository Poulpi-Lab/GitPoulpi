//! The Sparse checkout and LFS pages (Settings > Repository): the repository's
//! own checkout shape and its Git LFS state (plan `repo-settings.md`, slice 2).
//!
//! The pages own no git logic. Sparse checkout reuses the dialog's validation
//! (`sparse_dialog::parse_dirs`) and the daemon actions behind it
//! (`Action::SparseSet` / `SparseDisable`); the LFS page's button is
//! `PrefAction::InitializeLfs`. The screen has no daemon link of its own, so a
//! press becomes a [`SparseRequest`] the event loop takes (like the LFS
//! install request) and sends as an ordinary tracked action; the state the
//! pages show is pushed in by the event loop from its last refresh.

use gitpoulpi_core::domain::{LfsRepoStatus, SparseState};
use gitpoulpi_i18n::msg::settings_repo as t;
use gitpoulpi_i18n::msg::settings_workflow as wf;

use super::editor_close::Act;
use super::*;

/// The rows of these pages that are not plain config keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RpTarget {
    /// The multi-line "Directories to keep" field.
    SparsePaths,
    /// The Message template page's one-line Summary.
    TemplateSummary,
    /// The Message template page's multi-line Description.
    TemplateDescription,
    /// The Conflict prevention page's multi-line Base branches.
    BaseBranches,
    /// The Conflict prevention page's "Base branches from" choice.
    BaseSource,
    /// The Conflict prevention page's "Save as list..." name.
    SaveAsList,
    /// General's "Default base branches" choice.
    ConflictDefaultList,
    /// General's "New list" name.
    ConflictListNew,
    /// General: the name of the list at this index (sorted by name).
    ConflictListName(usize),
    /// General: the multi-line patterns of the list at this index.
    ConflictListPatterns(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RpButton {
    /// Start sparse checkout, or apply the edited list when it is on.
    Apply,
    Disable,
    /// Apply the current list again.
    Reapply,
    /// Write the commit template file (Message template page).
    SaveTemplate,
    /// General: stop using a default list (the built-in `**` again).
    ConflictResetGlobal,
    /// General: make the list at this index the default.
    ConflictListDefault(usize),
    /// General: delete the list at this index.
    ConflictListDelete(usize),
    /// Conflict prevention page: drop this repository's choice and own list.
    ConflictResetRepo,
    /// Conflict prevention page: own list from the branching workflow.
    ConflictUseWorkflow,
}

/// What a button asked of the daemon, taken by the event loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SparseRequest {
    Set(Vec<String>),
    Disable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditorButton {
    Save,
    Cancel,
}

struct PathsEdit {
    target: RpTarget,
    text: String,
    field: crate::text_edit::FieldEditor<()>,
    close: super::editor_close::EditorClose,
}

#[derive(Default)]
pub(super) struct RepoPages {
    sparse: SparseState,
    /// The edited list, `None` while the field follows the repository's state.
    draft: Option<String>,
    editor: Option<PathsEdit>,
    editor_hits: Vec<(Rect, EditorButton)>,
    request: Option<SparseRequest>,
    lfs: Option<LfsRepoStatus>,
    /// The Message template page's draft (`commit_pages`).
    pub(super) template: super::commit_pages::TemplateDraft,
    /// The status line's in-flight text ("applying the sparse checkout...").
    in_flight: Option<String>,
}

/// The directories typed one per line.
fn lines_to_dirs(text: &str) -> Result<Vec<String>, String> {
    let joined = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    crate::sparse_dialog::parse_dirs(&joined)
}

impl PreferencesState {
    // -- what the event loop pushes in and takes out ----------------------------

    /// The repository's sparse checkout as the last refresh read it. A change
    /// (an apply landed, another client edited it) drops the unsent draft.
    pub(crate) fn set_sparse(&mut self, state: &SparseState) {
        if self.repo_pages.sparse != *state {
            self.repo_pages.sparse = state.clone();
            self.repo_pages.draft = None;
        }
    }

    /// The LFS state of the last refresh (`None`: not read yet, or LFS is off).
    pub(crate) fn set_lfs_status(&mut self, status: Option<&LfsRepoStatus>) {
        if self.repo_pages.lfs.as_ref() != status {
            self.repo_pages.lfs = status.cloned();
        }
    }

    /// What the link is doing for a while, for the in-flight row.
    pub(crate) fn set_in_flight(&mut self, text: Option<String>) {
        if self.repo_pages.in_flight != text {
            self.repo_pages.in_flight = text;
        }
    }

    pub(crate) fn take_sparse_request(&mut self) -> Option<SparseRequest> {
        self.repo_pages.request.take()
    }

    // -- the pages --------------------------------------------------------------

    fn rp_paths_text(&self) -> String {
        match &self.repo_pages.draft {
            Some(draft) => draft.clone(),
            None if self.repo_pages.sparse.enabled => self.repo_pages.sparse.dirs.join("\n"),
            None => String::new(),
        }
    }

    fn rp_in_flight_row(&self) -> Option<Field> {
        self.repo_pages
            .in_flight
            .clone()
            .map(|text| Field::plain(&text, Control::Info(String::new()), Target::None))
    }

    pub(super) fn sparse_fields(&self) -> Vec<Field> {
        let sparse = &self.repo_pages.sparse;
        let text = self.rp_paths_text();
        let mut fields: Vec<Field> = Vec::new();
        fields.extend(self.rp_in_flight_row());
        fields.push(
            Field::plain(
                t::SPARSE_STATE.get(),
                Control::Info(
                    if sparse.enabled {
                        t::SPARSE_ON.get()
                    } else {
                        t::SPARSE_OFF.get()
                    }
                    .to_string(),
                ),
                Target::None,
            )
            .with_help(if sparse.enabled {
                gitpoulpi_i18n::msg::sparse::STATUS_ON.get()
            } else {
                gitpoulpi_i18n::msg::sparse::STATUS_OFF.get()
            }),
        );
        fields.push(
            Field::plain(
                t::SPARSE_PATHS.get(),
                Control::Area(text.clone()),
                Target::RepoPage(RpTarget::SparsePaths),
            )
            .with_placeholder(t::SPARSE_PLACEHOLDER.get())
            .with_help(t::SPARSE_PATHS_HELP.get()),
        );
        // One row of the buttons that apply: Enable while it is off, else
        // Update, Reapply and Disable.
        let no_paths = text.trim().is_empty();
        let mut buttons = vec![Btn::new(
            if sparse.enabled {
                t::SPARSE_UPDATE.get()
            } else {
                t::SPARSE_ENABLE.get()
            },
            PrefAction::RepoPage(RpButton::Apply),
        )];
        let mut help = vec![if sparse.enabled {
            t::SPARSE_UPDATE_HELP.get()
        } else {
            t::SPARSE_ENABLE_HELP.get()
        }];
        if sparse.enabled {
            buttons.push(Btn::new(
                t::SPARSE_REAPPLY.get(),
                PrefAction::RepoPage(RpButton::Reapply),
            ));
            buttons.push(Btn::new(
                t::SPARSE_DISABLE.get(),
                PrefAction::RepoPage(RpButton::Disable),
            ));
            help.push(t::SPARSE_REAPPLY_HELP.get());
            help.push(t::SPARSE_DISABLE_HELP.get());
            if no_paths {
                buttons[0].enabled = false;
                help.insert(0, t::SPARSE_NEEDS_PATHS.get());
            }
        }
        let row = Field::buttons("", buttons).with_help(&help.join(" "));
        fields.push(if no_paths && !sparse.enabled {
            row.coming(t::SPARSE_NEEDS_PATHS.get())
        } else {
            row
        });
        fields
    }

    pub(super) fn lfs_fields(&self) -> Vec<Field> {
        let mut fields: Vec<Field> = Vec::new();
        fields.extend(self.rp_in_flight_row());
        let enabled = self
            .repo_state
            .preferences
            .lfs_enabled
            .unwrap_or(self.config.git.lfs_enabled);
        if !enabled {
            fields.push(Field::heading(t::LFS_HEADING_STATE.get()));
            fields.push(Field::plain(
                t::LFS_OFF.get(),
                Control::Info(String::new()),
                Target::None,
            ));
            return fields;
        }
        let repo = self.repo_label.clone();
        let Some(status) = self.repo_pages.lfs.as_ref() else {
            fields.push(Field::heading(t::LFS_HEADING_STATE.get()));
            fields.push(Field::plain(
                t::LFS_READING.get(),
                Control::Info(String::new()),
                Target::None,
            ));
            return fields;
        };
        let initialized = crate::lfs_ui::initialized(status);
        fields.push(Field::heading(t::LFS_HEADING_INIT.get()));
        fields.push(Field::plain(
            &if initialized {
                t::lfs_initialized(&repo)
            } else {
                t::lfs_not_initialized(&repo)
            },
            Control::Info(String::new()),
            Target::None,
        ));
        if !initialized {
            if status.uses_lfs {
                fields.push(Field::plain(
                    t::LFS_USES.get(),
                    Control::Info(String::new()),
                    Target::None,
                ));
            }
            fields.push(
                Field::button(t::LFS_INITIALIZE.get(), PrefAction::InitializeLfs)
                    .with_help(t::LFS_INITIALIZE_HELP.get()),
            );
        }
        fields.push(Field::heading(t::LFS_HEADING_PATTERNS.get()));
        if status.patterns.is_empty() {
            fields.push(Field::plain(
                t::LFS_NO_PATTERNS.get(),
                Control::Info(String::new()),
                Target::None,
            ));
        } else {
            for (i, p) in status.patterns.iter().enumerate() {
                let field = Field::plain(
                    &p.pattern,
                    Control::Info(p.source.display().to_string()),
                    Target::None,
                );
                fields.push(if i == 0 {
                    field.with_help(t::LFS_PATTERNS_HELP.get())
                } else {
                    field
                });
            }
        }
        fields
    }

    // -- buttons ----------------------------------------------------------------

    pub(super) fn rp_button(&mut self, button: RpButton) {
        match button {
            RpButton::Apply => match lines_to_dirs(&self.rp_paths_text()) {
                Ok(dirs) => self.rp_request(SparseRequest::Set(dirs)),
                Err(why) => self.message = Some(why),
            },
            RpButton::Disable => {
                if self.repo_pages.sparse.enabled {
                    self.rp_request(SparseRequest::Disable);
                }
            }
            // `git sparse-checkout set --cone` with the current list rewrites
            // the patterns and re-applies them to the work tree: the same
            // result as `git sparse-checkout reapply`, with the existing
            // (journal-free, tested) daemon action and no new backend call.
            RpButton::SaveTemplate => self.template_save(),
            RpButton::ConflictResetGlobal
            | RpButton::ConflictListDefault(_)
            | RpButton::ConflictListDelete(_)
            | RpButton::ConflictResetRepo
            | RpButton::ConflictUseWorkflow => self.conflict_button(button),
            RpButton::Reapply => {
                let dirs = self.repo_pages.sparse.dirs.clone();
                if self.repo_pages.sparse.enabled && !dirs.is_empty() {
                    self.rp_request(SparseRequest::Set(dirs));
                }
            }
        }
    }

    fn rp_request(&mut self, request: SparseRequest) {
        self.message = Some(
            match request {
                SparseRequest::Set(_) => gitpoulpi_i18n::msg::sparse::APPLYING,
                SparseRequest::Disable => gitpoulpi_i18n::msg::sparse::DISABLING,
            }
            .get()
            .to_string(),
        );
        self.repo_pages.request = Some(request);
    }

    // -- the multi-line paths editor -------------------------------------------

    pub(super) fn rp_open_editor(&mut self, target: RpTarget) {
        let text = match target {
            RpTarget::SparsePaths => self.rp_paths_text(),
            RpTarget::TemplateDescription => self.repo_pages.template.description.clone(),
            RpTarget::BaseBranches => self.base_branches_text(),
            RpTarget::ConflictListPatterns(i) => self.conflict_list_text(i),
            _ => return,
        };
        let mut field = crate::text_edit::FieldEditor::default();
        field.caret_mut(()).set_cursor(&text, text.len(), false);
        self.repo_pages.editor = Some(PathsEdit {
            target,
            close: super::editor_close::EditorClose::new(&text),
            text,
            field,
        });
    }

    #[cfg(test)]
    pub(super) fn rp_editor_close(&self) -> Option<&super::editor_close::EditorClose> {
        self.repo_pages.editor.as_ref().map(|e| &e.close)
    }

    pub(super) fn rp_editor_open(&self) -> bool {
        self.repo_pages.editor.is_some()
    }

    fn rp_editor_save(&mut self) {
        self.repo_pages.editor_hits.clear();
        if let Some(edit) = self.repo_pages.editor.take() {
            match edit.target {
                RpTarget::TemplateDescription => {
                    if self.repo_pages.template.description != edit.text {
                        self.repo_pages.template.description = edit.text;
                        self.repo_pages.template.dirty = true;
                    }
                }
                RpTarget::BaseBranches => self.set_base_branches(&edit.text),
                RpTarget::ConflictListPatterns(i) => self.set_conflict_list_patterns(i, &edit.text),
                _ => self.repo_pages.draft = Some(edit.text),
            }
        }
    }

    fn rp_editor_cancel(&mut self) {
        self.repo_pages.editor = None;
        self.repo_pages.editor_hits.clear();
    }

    pub(super) fn rp_editor_key(&mut self, key: KeyEvent) -> bool {
        let Some(edit) = self.repo_pages.editor.as_mut() else {
            return true;
        };
        match edit.close.key(key, &edit.text) {
            Act::Close => {
                self.rp_editor_cancel();
                return true;
            }
            Act::Spent => return true,
            Act::Pass => {}
        }
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char('s' | 'S' | '\u{13}') if ctrl || key.code == KeyCode::Char('\u{13}') => {
                self.rp_editor_save()
            }
            KeyCode::Enter => edit.field.caret_mut(()).insert(&mut edit.text, "\n"),
            _ => {
                edit.field.caret_mut(()).on_key(&mut edit.text, &key, true);
            }
        }
        true
    }

    pub(super) fn rp_editor_paste(&mut self, text: &str) {
        if let Some(edit) = self.repo_pages.editor.as_mut() {
            edit.field.caret_mut(()).insert(&mut edit.text, text);
        }
    }

    pub(super) fn rp_editor_mouse(
        &mut self,
        kind: MouseEventKind,
        col: u16,
        row: u16,
        shift: bool,
    ) {
        let Some(edit) = self.repo_pages.editor.as_mut() else {
            return;
        };
        let event = crossterm::event::MouseEvent {
            kind,
            column: col,
            row,
            modifiers: KeyModifiers::NONE,
        };
        match edit.close.mouse(event, &edit.text) {
            Act::Close => return self.rp_editor_cancel(),
            Act::Spent => return,
            Act::Pass => {}
        }
        if edit.close.asking() {
            return;
        }
        if edit
            .field
            .area_mouse((), &edit.text, kind, (col, row), shift)
        {
            return;
        }
        if kind != MouseEventKind::Down(MouseButton::Left) {
            return;
        }
        match crate::dialog_kit::hit_at(&self.repo_pages.editor_hits, col, row) {
            Some(EditorButton::Save) => self.rp_editor_save(),
            Some(EditorButton::Cancel) if edit.close.cancel_button(&edit.text) == Act::Close => {
                self.rp_editor_cancel()
            }
            _ => {}
        }
    }

    /// Draws the paths editor over the page, when it is open. Its hit rects
    /// come out of this walk.
    pub(super) fn render_repo_pages_editor(&mut self, frame: &mut Frame, area: Rect) {
        self.repo_pages.editor_hits.clear();
        let mouse = self.mouse_pos;
        let Some(edit) = self.repo_pages.editor.as_mut() else {
            return;
        };
        let width = (area.width * 4 / 5).max(30).min(area.width);
        let height = (area.height / 2).max(8).min(area.height);
        let rect = WIN.place(
            area,
            Rect::new(
                area.x + (area.width - width) / 2,
                area.y + (area.height - height) / 2,
                width,
                height,
            ),
        );
        let title = match edit.target {
            RpTarget::TemplateDescription => t::TEMPLATE_DESCRIPTION.get(),
            RpTarget::BaseBranches => gitpoulpi_i18n::msg::conflict_prevention::BASES.get(),
            RpTarget::ConflictListPatterns(_) => {
                gitpoulpi_i18n::msg::conflict_prevention::LIST_PATTERNS.get()
            }
            _ => t::SPARSE_PATHS.get(),
        };
        edit.close
            .framed(frame.buffer_mut(), rect, title, &WIN, true);
        super::editor_close::footer(
            frame.buffer_mut(),
            rect,
            &wf::editor_footer(crate::dialog_kit::ctrl_key('s')),
        );
        let inner = rect.inner(ratatui::layout::Margin::new(1, 1));
        let text_area = Rect::new(
            inner.x,
            inner.y,
            inner.width,
            inner.height.saturating_sub(2),
        );
        let look = crate::text_field::EditLook::new(
            Style::default()
                .fg(crate::th().ui.text)
                .bg(crate::input_focus_bg()),
            "\u{2588}",
        );
        let lines = crate::text_field::edit_lines(
            &edit.text,
            edit.field.caret(()),
            text_area.width.saturating_sub(1) as usize,
            text_area.height as usize,
            true,
            look,
        );
        frame.render_widget(Paragraph::new(lines), text_area);
        // The field's click rect: the same width `edit_lines` wrapped to.
        edit.field.begin_frame();
        edit.field.drawn(
            (),
            Rect::new(
                text_area.x,
                text_area.y,
                text_area.width.saturating_sub(1),
                text_area.height,
            ),
            true,
        );
        use crate::dialog_kit::{self, ButtonState};
        let y = inner.bottom().saturating_sub(1);
        let cancel_label = gitpoulpi_i18n::msg::dialog::CANCEL.get();
        let save_label = wf::EDITOR_SAVE.get();
        let cancel_w = dialog_kit::button_width(cancel_label);
        let save_w = dialog_kit::button_width(save_label);
        let cancel_x = inner.right().saturating_sub(cancel_w);
        let save_x = cancel_x.saturating_sub(save_w + 1);
        let hovered = |r: Rect| mouse.is_some_and(|(c, row)| r.contains((c, row).into()));
        let save_rect = Rect::new(save_x, y, save_w, 1);
        let cancel_rect = Rect::new(cancel_x, y, cancel_w, 1);
        let buf = frame.buffer_mut();
        dialog_kit::button(
            buf,
            save_x,
            y,
            save_label,
            ButtonState::Primary {
                hovered: hovered(save_rect),
                focused: false,
            },
        );
        dialog_kit::button(
            buf,
            cancel_x,
            y,
            cancel_label,
            ButtonState::Normal {
                hovered: hovered(cancel_rect),
                focused: false,
            },
        );
        self.repo_pages.editor_hits = vec![
            (save_rect, EditorButton::Save),
            (cancel_rect, EditorButton::Cancel),
        ];
        if let Some(edit) = self.repo_pages.editor.as_mut() {
            edit.close.render(frame.buffer_mut());
        }
    }
}

/// The editor resizes: its text area takes the whole rect, so it grows and
/// shrinks with it.
const WIN: crate::dialog_kit::Win =
    crate::dialog_kit::Win::resizable("settings_repo_pages_editor", 40, 10);
