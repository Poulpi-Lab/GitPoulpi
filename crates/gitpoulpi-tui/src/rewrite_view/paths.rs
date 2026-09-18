//! The path rules in the workbench (slice S20): extracting a folder into a
//! new repository (a clone, or a submodule of this one) needs a folder for
//! that repository. It is picked with the app's folder picker over the
//! workbench, after the extract-mode question, and the repository is
//! created in a new folder of it named after the extracted directory. After
//! the apply the new repository opens in a new tab.
//!
//! Everything the screen needs about it is here so the shared workbench
//! files only call into this one.

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyEvent, MouseEvent};
use gitpoulpi_core::domain::rewrite::{ExtractMode, RewriteOutcome};
use gitpoulpi_i18n::msg::rewrite_paths as p;
use gitpoulpi_i18n::msg::rewrite_view as t;
use gitpoulpi_i18n::M;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::{RewriteKeyOutcome, RewriteView};
use crate::folder_listing::{Lister, ListerEvent};
use crate::folder_picker::{picker_rect, FolderPicker, PickerOutcome};

/// The folder picker over the workbench and what it is picking for.
pub(super) struct TargetPicker {
    picker: FolderPicker,
    lister: Lister,
    mode: ExtractMode,
    /// The folder the new repository gets inside the chosen one.
    name: String,
}

/// What the path rules keep besides the recipe.
#[derive(Default)]
pub(crate) struct PathsState {
    picker: RefCell<Option<TargetPicker>>,
    /// Where the new repository goes (a clone or a submodule extraction).
    pub(super) target: Option<PathBuf>,
    /// The apply on its way: how, and where.
    applying: Option<(ExtractMode, PathBuf)>,
    /// What the last apply made, for the result panel.
    done: Option<(ExtractMode, PathBuf)>,
    /// A new repository the window should open in a new tab.
    open_after: Option<PathBuf>,
}

/// The note for each way of extracting, in the order of `EXTRACT_CHOICES`.
const NOTES: [M; 3] = [p::NOTE_CLONE, p::NOTE_IN_PLACE, p::NOTE_SUBMODULE];

/// The question's prompt with what the highlighted way does.
pub(super) fn extract_prompt(cursor: usize) -> String {
    let base = t::Q_EXTRACT_PROMPT.get();
    match NOTES.get(cursor) {
        Some(note) => format!("{base}  {}", note.get()),
        None => base.to_string(),
    }
}

/// `parent/name`, or `parent/name-2`... when that exists and is not empty.
pub(super) fn free_target(parent: &Path, name: &str) -> PathBuf {
    let plain = parent.join(name);
    let free = |p: &Path| match std::fs::read_dir(p) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => true,
    };
    if free(&plain) {
        return plain;
    }
    (2..)
        .map(|n| parent.join(format!("{name}-{n}")))
        .find(|p| free(p))
        .expect("an unbounded counter finds a free name")
}

impl RewriteView {
    /// The last segment of the extracted directory: the new repository's
    /// folder name.
    fn extract_name(&self) -> String {
        self.recipe
            .extraction()
            .and_then(|(dir, _)| dir.trim_matches('/').rsplit('/').next())
            .filter(|n| !n.is_empty())
            .unwrap_or("extracted")
            .to_string()
    }

    /// The answer to the extract-mode question: in place goes straight to the
    /// preview, the others first ask where the new repository goes.
    pub(super) fn choose_extract_mode(&mut self, mode: ExtractMode) {
        if mode == ExtractMode::InPlace {
            self.choices.extract_mode = Some(mode);
            self.paths.target = None;
            self.preview();
            return;
        }
        let home = gitpoulpi_config::home_dir();
        let start = self
            .paths
            .target
            .as_deref()
            .and_then(Path::parent)
            .filter(|p| p.is_dir())
            .map(Path::to_path_buf)
            .or_else(|| home.clone())
            .unwrap_or_else(|| PathBuf::from("."));
        let picker = FolderPicker::new(p::PICK_TITLE.get(), p::PICK_CONFIRM.get(), start);
        let lister = Lister::default();
        lister.request_places(home, Vec::new());
        *self.paths.picker.borrow_mut() = Some(TargetPicker {
            picker,
            lister,
            mode,
            name: self.extract_name(),
        });
    }

    pub(crate) fn has_target_picker(&self) -> bool {
        self.paths.picker.borrow().is_some()
    }

    /// Hands the picker the listings it asked for. Never waits.
    pub(super) fn pump_paths(&mut self) {
        let mut slot = self.paths.picker.borrow_mut();
        let Some(tp) = slot.as_mut() else { return };
        for event in tp.lister.drain() {
            match event {
                ListerEvent::Listed(dir, listing) => tp.picker.deliver(&dir, listing),
                ListerEvent::Places(places) => tp.picker.set_places(places),
            }
        }
        for dir in tp.picker.wants() {
            tp.lister.request(dir);
        }
    }

    pub(super) fn on_target_outcome(&mut self, outcome: PickerOutcome) {
        let chosen = {
            let mut slot = self.paths.picker.borrow_mut();
            let Some(tp) = slot.as_mut() else { return };
            match outcome {
                PickerOutcome::Stay
                | PickerOutcome::ShowHiddenChanged(_)
                | PickerOutcome::ShowSiblingsChanged(_) => return,
                PickerOutcome::Cancel => {
                    *slot = None;
                    return;
                }
                PickerOutcome::CreateFolder(path) => {
                    match std::fs::create_dir(&path) {
                        Ok(()) => {
                            let name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_default();
                            tp.picker.folder_created(&name);
                        }
                        Err(e) => tp.picker.folder_create_failed(&e.to_string()),
                    }
                    return;
                }
                PickerOutcome::Choose(dir) => {
                    let target = free_target(&dir, &tp.name);
                    let mode = tp.mode;
                    *slot = None;
                    (mode, target)
                }
            }
        };
        self.choices.extract_mode = Some(chosen.0);
        self.paths.target = Some(chosen.1);
        self.preview();
    }

    pub(super) fn on_target_key(&mut self, key: KeyEvent) -> RewriteKeyOutcome {
        let outcome = {
            let mut slot = self.paths.picker.borrow_mut();
            match slot.as_mut() {
                Some(tp) => tp.picker.on_key(key),
                None => return RewriteKeyOutcome::Stay,
            }
        };
        self.on_target_outcome(outcome);
        RewriteKeyOutcome::Stay
    }

    pub(crate) fn target_mouse(&mut self, mouse: MouseEvent) {
        let outcome = {
            let mut slot = self.paths.picker.borrow_mut();
            match slot.as_mut() {
                Some(tp) => tp.picker.on_mouse(mouse, false),
                None => return,
            }
        };
        self.on_target_outcome(outcome);
    }

    pub(super) fn target_paste(&mut self, text: &str) {
        if let Some(tp) = self.paths.picker.borrow_mut().as_mut() {
            tp.picker.paste(text);
        }
    }

    /// Draws the picker over the workbench.
    pub(super) fn paint_target_picker(&self, buf: &mut Buffer, bounds: Rect) {
        if let Some(tp) = self.paths.picker.borrow_mut().as_mut() {
            tp.picker.render(picker_rect(bounds), buf);
        }
    }

    /// Where the apply sends the new repository, when the answer to the
    /// question was a clone or a submodule.
    pub(super) fn extract_target(&mut self) -> Option<PathBuf> {
        let mode = self.choices.extract_mode?;
        if mode == ExtractMode::InPlace {
            return None;
        }
        let target = self.paths.target.clone()?;
        self.paths.applying = Some((mode, target.clone()));
        Some(target)
    }

    /// An apply finished: remembers what it made and queues the new tab.
    pub(super) fn paths_applied(&mut self, outcome: &RewriteOutcome) {
        let _ = outcome;
        if let Some((mode, target)) = self.paths.applying.take() {
            self.paths.open_after = Some(target.clone());
            self.paths.done = Some((mode, target));
        } else {
            self.paths.done = None;
        }
        self.paths.target = None;
    }

    /// The apply failed or was refused: nothing was made.
    pub(super) fn paths_apply_failed(&mut self) {
        self.paths.applying = None;
    }

    /// The new repository to open in a new tab, once.
    pub(crate) fn take_open_repo(&mut self) -> Option<PathBuf> {
        self.paths.open_after.take()
    }

    /// The first line of the result panel: what was rewritten, or what the
    /// extraction made.
    pub(super) fn result_headline(&self, outcome: &RewriteOutcome) -> String {
        match &self.paths.done {
            Some((ExtractMode::Clone, path)) => p::done_extracted_clone(path.display()),
            Some((ExtractMode::Submodule, path)) => p::done_extracted_submodule(path.display()),
            _ => t::done_summary(outcome.commits_changed),
        }
    }

    /// The Summary lines of a preview that extracts into a new repository.
    pub(super) fn extract_summary(&self) -> Vec<String> {
        let target = self
            .paths
            .target
            .as_ref()
            .map(|path| p::sum_extract_target(path.display()));
        match self.choices.extract_mode {
            Some(ExtractMode::Clone) => [Some(p::SUM_EXTRACT_CLONE.get().to_string()), target]
                .into_iter()
                .flatten()
                .collect(),
            Some(ExtractMode::Submodule) => {
                [Some(p::SUM_EXTRACT_SUBMODULE.get().to_string()), target]
                    .into_iter()
                    .flatten()
                    .collect()
            }
            _ => Vec::new(),
        }
    }
}
