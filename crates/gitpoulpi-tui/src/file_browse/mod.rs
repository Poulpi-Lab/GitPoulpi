//! "Browse..." for a field that holds one file: the folder picker in file
//! mode, with its own lister so it can sit over a dialog or a Settings page
//! that has no `RepoDialogs` around it.
//!
//! Used by the "Extra file" fields of the code review context (the ✦ Review
//! options dialog and the two Settings > AI > Code review pages). Listing
//! happens on the lister's threads, never on the render thread; this type
//! only moves answers between the lister and the (pure) picker.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crossterm::event::{KeyEvent, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::folder_listing::{Lister, ListerEvent};
use crate::folder_picker::{picker_rect, FolderPicker, PickerOutcome};

/// What the owner of the browser does next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BrowseOutcome {
    Stay,
    Cancel,
    /// The file to store: relative to the repository root when it is inside
    /// it, absolute otherwise.
    Chosen(String),
}

pub(crate) struct FileBrowse {
    picker: FolderPicker,
    lister: Lister,
    root: Option<PathBuf>,
    last_click: Option<(Instant, u16, u16)>,
}

/// Two clicks this close in time, on the same cell, are a double-click.
const DOUBLE: Duration = Duration::from_millis(400);

impl FileBrowse {
    /// Opens the picker. `root` is the repository root (a clone), where it
    /// starts unless `current` already names something to start beside;
    /// `current` is what the field holds now.
    pub(crate) fn open(title: &str, confirm: &str, root: Option<&Path>, current: &str) -> Self {
        let start = start_dir(root, current);
        let mut picker = FolderPicker::new(title, confirm, start);
        picker.set_file_mode();
        let settings = gitpoulpi_config::Settings::resolve_ambient().ok();
        picker.set_show_hidden(
            settings
                .as_ref()
                .is_some_and(|s| s.config().repos.show_hidden),
        );
        picker.set_show_siblings(
            settings
                .as_ref()
                .is_some_and(|s| s.config().repos.show_siblings),
        );
        let lister = Lister::default();
        let extra: Vec<(String, PathBuf)> = root
            .map(|r| {
                let label = r
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| r.display().to_string());
                vec![(label, r.to_path_buf())]
            })
            .unwrap_or_default();
        lister.request_places(gitpoulpi_config::home_dir(), extra);
        Self {
            picker,
            lister,
            root: root.map(Path::to_path_buf),
            last_click: None,
        }
    }

    /// Hands the picker the listings that arrived and starts the ones it
    /// asked for. Never waits.
    pub(crate) fn pump(&mut self) {
        for event in self.lister.drain() {
            match event {
                ListerEvent::Listed(dir, listing) => self.picker.deliver(&dir, listing),
                ListerEvent::Places(places) => self.picker.set_places(places),
            }
        }
        for dir in self.picker.wants() {
            self.lister.request(dir);
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> BrowseOutcome {
        let outcome = self.picker.on_key(key);
        self.outcome(outcome)
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> BrowseOutcome {
        let double = matches!(mouse.kind, MouseEventKind::Down(_)) && {
            let now = Instant::now();
            let again = self.last_click.is_some_and(|(at, x, y)| {
                now.duration_since(at) < DOUBLE && (x, y) == (mouse.column, mouse.row)
            });
            self.last_click = (!again).then_some((now, mouse.column, mouse.row));
            again
        };
        let outcome = self.picker.on_mouse(mouse, double);
        self.outcome(outcome)
    }

    pub(crate) fn paste(&mut self, text: &str) {
        self.picker.paste(text);
    }

    pub(crate) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.pump();
        self.picker.render(picker_rect(area), buf);
    }

    fn outcome(&mut self, outcome: PickerOutcome) -> BrowseOutcome {
        match outcome {
            PickerOutcome::Cancel => BrowseOutcome::Cancel,
            PickerOutcome::Choose(path) => {
                BrowseOutcome::Chosen(stored_path(&path, self.root.as_deref()))
            }
            PickerOutcome::CreateFolder(_) => {
                self.picker
                    .folder_create_failed(gitpoulpi_i18n::msg::review_options::NO_NEW_FOLDER.get());
                BrowseOutcome::Stay
            }
            PickerOutcome::Stay
            | PickerOutcome::ShowHiddenChanged(_)
            | PickerOutcome::ShowSiblingsChanged(_) => BrowseOutcome::Stay,
        }
    }

    #[cfg(test)]
    pub(crate) fn picker(&mut self) -> &mut FolderPicker {
        &mut self.picker
    }
}

/// Where the picker starts: beside the file the field names (relative paths
/// are read from the repository root), else the repository root, else home.
pub(crate) fn start_dir(root: Option<&Path>, current: &str) -> PathBuf {
    let current = current.trim();
    if !current.is_empty() && !current.contains("://") {
        let path = Path::new(current);
        let full = match (path.is_absolute(), root) {
            (true, _) => Some(path.to_path_buf()),
            (false, Some(root)) => Some(root.join(path)),
            (false, None) => None,
        };
        if let Some(parent) = full.as_deref().and_then(Path::parent) {
            if parent.is_dir() {
                return parent.to_path_buf();
            }
        }
    }
    root.map(Path::to_path_buf)
        .or_else(gitpoulpi_config::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// How a chosen file is written into the field: relative to `root` (with
/// `/`, whatever the OS) when it is inside it, absolute otherwise.
pub(crate) fn stored_path(chosen: &Path, root: Option<&Path>) -> String {
    if let Some(root) = root {
        let inside = chosen
            .strip_prefix(root)
            .ok()
            .map(Path::to_path_buf)
            .or_else(|| {
                let (c, r) = (chosen.canonicalize().ok()?, root.canonicalize().ok()?);
                c.strip_prefix(r).ok().map(Path::to_path_buf)
            });
        if let Some(rel) = inside.filter(|r| !r.as_os_str().is_empty()) {
            return rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
        }
    }
    chosen.display().to_string()
}

#[cfg(test)]
mod tests;
