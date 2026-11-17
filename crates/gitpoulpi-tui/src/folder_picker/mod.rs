//! The folder picker: GitPoulpi's own "choose a folder" dialog, modelled
//! on the Windows one.
//!
//! ```text
//! ╭ Open a repository ─────────────────────────────────────────────╮
//! │ [↑] C:\Projets\GitPoulpi█                                      │
//! │                                                                │
//! │ QUICK ACCESS         │ Name                                    │
//! │  Home                │  ..                                     │
//! │  Desktop             │  crates\                                │
//! │  C:\                 │  docs\                                  │
//! │ FOLDERS              │  Cargo.toml                   (dimmed)  │
//! │  C:\                 │                                         │
//! │   Projets            │                                         │
//! │    GitPoulpi         │                                         │
//! │     crates           │                                         │
//! │ Selected: C:\Projets\GitPoulpi   git repository                │
//! │ Tab zones · Enter open · Ctrl+S select   [ Cancel ] [ Open ]   │
//! ╰────────────────────────────────────────────────────────────────╯
//! ```
//!
//! Three zones -- the path field, the left column (quick places and the
//! folder tree down to the current folder and its subfolders), the
//! current folder's content -- with `Tab` / `Shift+Tab` between them.
//!
//! **Pure**, like every dialog in this TUI: no IO here at all. The picker
//! says which folders it wants listed ([`FolderPicker::wants`]) and is
//! handed the answers later ([`FolderPicker::deliver`]); the event loop
//! routes the requests through `folder_listing::Lister`, which reads on a
//! thread -- so a slow network drive never blocks a keystroke, and the
//! tests drive the same state with synchronous reads of a temp tree.

mod bars;
mod input;
mod render;

use std::path::{Path, PathBuf};

use crate::keymap::Action as Bind;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_i18n::msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};

use crate::dialog_kit::{self, ButtonState, CheckState};
use crate::dir_picker::{self, DirEntryInfo};
use crate::display_width;
use crate::folder_listing::{FolderListing, Place, PlaceKind};

/// Which part of the picker has the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Zone {
    Path,
    /// The filter field on the toolbar row.
    Filter,
    Left,
    Content,
}

/// The two text fields the picker can have - never both drawn with the
/// keyboard at once, but drawn on screen at once (the path field always,
/// the "New folder" field while it is open), so they need distinct ids
/// in the shared editor rather than sharing `()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PickerField {
    Path,
    NewFolder,
    Filter,
}

/// Whether the picker chooses a folder (the default - "Open a
/// repository", "Create worktree here") or a file. A file picker never chooses the folder shown --
/// only a file row - and files render selectable instead of dimmed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PickerMode {
    #[default]
    Folder,
    File,
}

/// A row of the left column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LeftRow {
    Heading(gitpoulpi_i18n::M),
    /// A quick place, by index into [`FolderPicker::places`].
    Place(usize),
    /// A node of the folder tree: an ancestor of the current folder, the
    /// current folder itself, or one of its subfolders.
    Tree {
        path: PathBuf,
        name: String,
        depth: usize,
        current: bool,
        is_repo: bool,
        /// A dot-folder or Windows hidden-attribute one, drawn dimmed. Always `false` for an
        /// ancestor: the chain
        /// to the folder shown is never hidden from itself.
        is_hidden: bool,
        /// On the open path (an ancestor or the current folder): the expanded fold mark.
        /// Every other row is a collapsed folder.
        open: bool,
    },
}

impl LeftRow {
    fn selectable(&self) -> bool {
        !matches!(self, LeftRow::Heading(_))
    }
    fn target(&self, places: &[Place]) -> Option<PathBuf> {
        match self {
            LeftRow::Heading(_) => None,
            LeftRow::Place(i) => places.get(*i).map(|p| p.path.clone()),
            LeftRow::Tree { path, .. } => Some(path.clone()),
        }
    }
}

/// A row of the content pane. Files come last and are never selectable:
/// the picker chooses folders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ContentRow {
    Parent(PathBuf),
    Dir(usize),
    File(usize),
}

/// What a click or a hover landed on. Registered inner-first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PickerHit {
    UpButton,
    PathField,
    Completion(usize),
    Left(usize),
    Content(usize),
    Cancel,
    Choose,
    /// The divider between the left column and the content pane
    /// dragged to resize, hover lights it.
    Splitter,
    /// The "Show hidden folders" checkbox.
    ShowHidden,
    /// The "Show sibling folders" checkbox.
    ShowSiblings,
    /// The "New folder" button.
    NewFolder,
    /// The inline name field it opens, while it is up.
    NewFolderField,
    /// The filter field on the toolbar row.
    FilterField,
    /// The blank cell column left of the places column: a click focuses
    /// that list without choosing a row. What its `Ctrl+N` badge presses.
    LeftList,
    /// The same for the folders list, in the cell column left of it.
    ContentList,
}

/// What the dialog's owner should do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PickerOutcome {
    Stay,
    Cancel,
    Choose(PathBuf),
    /// "New folder" was submitted with a valid name:
    /// the owner creates it (the one piece of IO this otherwise-pure
    /// dialog asks for) and reports back with
    /// [`FolderPicker::folder_created`] or [`FolderPicker::folder_create_failed`].
    CreateFolder(PathBuf),
    /// "Show hidden folders" was toggled: the owner persists it to
    /// `repos.show_hidden`.
    ShowHiddenChanged(bool),
    /// "Show sibling folders" was toggled: the owner persists it to
    /// `repos.show_siblings`.
    ShowSiblingsChanged(bool),
}

/// How many completions the dropdown shows at once.
const DROPDOWN_ROWS: usize = 8;
/// How many listings are kept for going back and for completion.
const CACHE: usize = 64;
/// The most sub-folders one level of the left tree lists when siblings are shown.
const SIBLING_CAP: usize = 500;
/// The splitter's minimum left-column width - enough
/// for "QUICK ACCESS" and the deepest ordinary tree row to stay legible.
const SPLIT_MIN_LEFT: u16 = 16;
/// The content pane's minimum width - a name column plus its
/// "git repository" tag still has room.
const SPLIT_MIN_CONTENT: u16 = 20;
/// How far one `Alt+Left`/`Alt+Right` moves the splitter.
const SPLIT_STEP: i32 = 2;

thread_local! {
    /// The splitter share the next picker opens with (per mille of the
    /// pane row), and whether a change is waiting to be written. A
    /// thread-local, not a static: the UI thread owns every picker, and
    /// tests on other threads never see each other's drags.
    static SPLIT: std::cell::Cell<(Option<u16>, bool)> =
        const { std::cell::Cell::new((None, false)) };
}

/// Publishes the remembered splitter share at start (`0` = the default).
pub(crate) fn install_split(permille: u16) {
    SPLIT.with(|c| c.set(((permille > 0).then_some(permille), false)));
}

/// The new splitter share, once, when a picker moved it since the last
/// call: what the window writes to the session file.
pub(crate) fn take_split_change() -> Option<u16> {
    SPLIT.with(|c| {
        let (share, dirty) = c.get();
        if dirty {
            c.set((share, false));
            share
        } else {
            None
        }
    })
}

#[derive(Debug, Clone)]
pub(crate) struct FolderPicker {
    pub title: String,
    /// The confirm button's label: `Open`, `Select folder`.
    pub confirm: String,
    /// Folder or file picker (see [`PickerMode`]).
    pub(crate) mode: PickerMode,
    /// The folder being shown.
    pub dir: PathBuf,
    /// Its content, `None` while it is being read.
    listing: Option<FolderListing>,
    /// Recent listings, newest last - going back is instant, and the
    /// path field completes against them.
    cache: Vec<(PathBuf, FolderListing)>,
    /// Folders to read, drained by [`Self::wants`].
    outbox: Vec<PathBuf>,
    pub places: Vec<Place>,
    pub zone: Zone,
    pub path_draft: String,
    /// The draft is the folder's own path, not something typed: the
    /// first key typed replaces it (the address bar's select-all).
    draft_pristine: bool,
    /// Typed since the last navigation - the completion dropdown shows.
    typing: bool,
    pub completion: Option<usize>,
    /// Index into [`Self::left_rows`], always a selectable row.
    pub left_sel: Option<usize>,
    /// Index into [`Self::content_rows`], always a selectable one.
    /// `None` = nothing selected, and choosing picks the folder shown.
    pub content_sel: Option<usize>,
    left_scroll: usize,
    content_scroll: usize,
    /// The completion dropdown's first row, and what it was last scrolled
    /// for (selection, count): it follows the selection only when that
    /// changes, so a bar drag stays put.
    drop_top: usize,
    drop_key: (Option<usize>, usize),
    left_bar: bars::PickerBar,
    content_bar: bars::PickerBar,
    drop_bar: bars::PickerBar,
    /// A key moved a selection: the next draw scrolls it into view.
    follow: bool,
    /// The open folder changed (or the siblings appeared): draws scroll the
    /// tree so its row is in view, again as ancestor listings arrive and
    /// grow the tree above it, until the user scrolls or selects a row.
    reveal_current: bool,
    pub message: Option<String>,
    pub hover: Option<PickerHit>,
    /// The last draw's hit rects, inner-first.
    hits: Vec<(Rect, PickerHit)>,
    /// The last draw's pane rects, for the wheel.
    left_area: Rect,
    content_area: Rect,
    drop_area: Rect,
    /// The path field's caret and selection. A pristine
    /// draft is shown - and behaves - as all selected. Also drives the
    /// "New folder" inline field: the two are never
    /// focused at once, so one editor suffices.
    edit: crate::text_edit::FieldEditor<PickerField>,
    /// Where the left press being handled landed, for the path field.
    last_press: Option<(u16, u16)>,
    close: dialog_kit::CloseHit,
    /// The "Show hidden folders" checkbox. Dot-folders
    /// (`.git` included) and Windows hidden-attribute entries are drawn
    /// dimmed rather than left out of the listing, so toggling this
    /// never needs a re-read.
    pub show_hidden: bool,
    /// The "Show sibling folders" checkbox: the left tree lists the sub-folders of
    /// every open level, not only the chain down to the current folder.
    pub show_siblings: bool,
    /// An explicit splitter width, set by a drag or
    /// `Alt+Left`/`Alt+Right`. `None` keeps the proportional default.
    splitter_w: Option<u16>,
    /// The remembered left-column share (per mille of the pane row) this
    /// picker opened with; `splitter_w` wins once the user moves it.
    restored_permille: Option<u16>,
    /// The pane row's total width at the last draw, so a key can resize
    /// the splitter without waiting for another mouse event.
    panes_w: u16,
    /// A press on the splitter is being dragged.
    dragging_splitter: bool,
    /// The "New folder" inline name field's draft, while it is open
    /// `None`: the button is shown instead.
    new_folder_draft: Option<String>,
    /// A folder just created (or about to be, from a submitted draft):
    /// selected once its listing lands. See [`Self::folder_created`].
    pending_select: Option<String>,
    /// What is typed in the filter field: the content pane lists only the
    /// entries whose name contains it (case-insensitive). Cleared whenever
    /// the folder changes.
    pub(crate) filter: String,
}

impl FolderPicker {
    /// What `Ctrl+N` badges: the toolbar, the path field, the buttons, and
    /// one badge for the places list and one for the folders list, each in
    /// the blank cell beside its first row (a press focuses the list, it
    /// does not pick a row), never a badge per row.
    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        let mut out = dialog_kit::nav_cells(self.hits.iter().filter_map(|(r, hit)| match hit {
            PickerHit::UpButton
            | PickerHit::PathField
            | PickerHit::Cancel
            | PickerHit::Choose
            | PickerHit::ShowHidden
            | PickerHit::ShowSiblings
            | PickerHit::NewFolder
            | PickerHit::NewFolderField
            | PickerHit::FilterField => Some(*r),
            _ => None,
        }));
        // The badge is drawn from the cell after its rect's first, three
        // cells wide: it starts on the gutter cell it presses.
        out.extend(
            self.hits
                .iter()
                .filter(|(_, hit)| matches!(hit, PickerHit::LeftList | PickerHit::ContentList))
                .map(|(r, _)| (Rect::new(r.x.saturating_sub(1), r.y, 3, 1), (r.x, r.y))),
        );
        out.sort_by_key(|(_, (x, y))| (*y, *x));
        out
    }

    pub(crate) fn new(title: &str, confirm: &str, start: PathBuf) -> Self {
        // `start` sometimes arrives canonicalized (a `RepoRef::root`, or a
        // path pasted from an app that canonicalizes) and Windows'
        // `\\?\` verbatim prefix reads as noise wherever a human sees it
        // -- the path field, the "Selected:" line, the FOLDERS tree's
        // root. Stripped once here, at the one place `dir` is first set,
        // rather than wherever it later gets displayed.
        let start = gitpoulpi_core::paths::strip_verbatim_prefix(&start);
        Self {
            title: title.to_string(),
            confirm: confirm.to_string(),
            mode: PickerMode::Folder,
            path_draft: start.display().to_string(),
            outbox: vec![start.clone()],
            dir: start,
            listing: None,
            cache: Vec::new(),
            places: Vec::new(),
            zone: Zone::Content,
            draft_pristine: true,
            edit: Default::default(),
            last_press: None,
            close: Default::default(),
            typing: false,
            completion: None,
            left_sel: None,
            content_sel: None,
            left_scroll: 0,
            content_scroll: 0,
            drop_top: 0,
            drop_key: (None, 0),
            left_bar: Default::default(),
            content_bar: Default::default(),
            drop_bar: Default::default(),
            follow: false,
            reveal_current: true,
            message: None,
            hover: None,
            hits: Vec::new(),
            left_area: Rect::default(),
            content_area: Rect::default(),
            drop_area: Rect::default(),
            show_hidden: false,
            show_siblings: false,
            splitter_w: None,
            restored_permille: SPLIT.with(|c| c.get().0),
            panes_w: 0,
            dragging_splitter: false,
            new_folder_draft: None,
            pending_select: None,
            filter: String::new(),
        }
    }

    /// Sets the initial "Show hidden folders" state,
    /// remembered in `repos.show_hidden`. A setter rather than a `new`
    /// parameter: most callers (tests, and pickers before the config is
    /// resolved) want the default.
    pub(crate) fn set_show_hidden(&mut self, show: bool) {
        self.show_hidden = show;
    }

    /// Sets the initial "Show sibling folders" state, remembered in
    /// `repos.show_siblings`.
    pub(crate) fn set_show_siblings(&mut self, show: bool) {
        self.show_siblings = show;
        self.want_ancestors();
    }

    /// Flips "Show sibling folders". The ancestors' listings are read lazily,
    /// on the lister's threads, the first time they are needed.
    fn toggle_show_siblings(&mut self) {
        self.show_siblings = !self.show_siblings;
        self.want_ancestors();
        self.left_scroll = 0;
        self.left_sel = None;
        self.reveal_current = true;
    }

    /// Queues the reads of the open path's ancestors (their sub-folders are the
    /// siblings the tree lists), those not already known. Nothing when siblings are off.
    fn want_ancestors(&mut self) {
        if !self.show_siblings {
            return;
        }
        let missing: Vec<PathBuf> = self
            .dir
            .ancestors()
            .skip(1)
            .filter(|p| !p.as_os_str().is_empty())
            .filter(|p| self.cached(p).is_none() && !self.outbox.iter().any(|o| o == p))
            .map(Path::to_path_buf)
            .collect();
        self.outbox.extend(missing);
    }

    /// Turns an existing picker (e.g. one from `repo_dialogs`' `new_picker`,
    /// which carries the quick-access roots and "Show hidden folders") into
    /// a file picker, e.g. "Apply patch...". Files render selectable;
    /// folders still navigate, but neither a bare `Enter` on the folder
    /// shown nor `Ctrl+S` chooses it, only a file row does.
    pub(crate) fn set_file_mode(&mut self) {
        self.mode = PickerMode::File;
    }

    /// A file picker from scratch (see [`Self::set_file_mode`]).
    #[cfg(test)]
    pub(crate) fn for_file(title: &str, confirm: &str, start: PathBuf) -> Self {
        let mut picker = Self::new(title, confirm, start);
        picker.mode = PickerMode::File;
        picker
    }

    // -- the IO boundary --------------------------------------------------

    /// The folders this picker needs read. Drained: each is asked once.
    pub(crate) fn wants(&mut self) -> Vec<PathBuf> {
        self.want_ancestors();
        std::mem::take(&mut self.outbox)
    }

    /// A listing arrived. Kept in the cache whatever it is for; shown
    /// only if it is the folder on screen - an answer for a folder the
    /// user has since left is not drawn over the one they are in.
    pub(crate) fn deliver(&mut self, dir: &Path, listing: FolderListing) {
        self.cache.retain(|(d, _)| d != dir);
        self.cache.push((dir.to_path_buf(), listing.clone()));
        if self.cache.len() > CACHE {
            self.cache.remove(0);
        }
        if dir == self.dir {
            self.listing = Some(listing);
            self.clamp();
            if let Some(name) = self.pending_select.take() {
                if let Some(index) = self
                    .content_rows()
                    .iter()
                    .position(|row| matches!(row, ContentRow::Dir(i) if self.listing.as_ref().is_some_and(|l| l.dirs[*i].name == name)))
                {
                    self.content_sel = Some(index);
                    self.follow = true;
                }
            }
        }
    }

    /// A "New folder" submission was created on disk:
    /// forces a fresh read of the folder shown (the cache would still
    /// hold the pre-creation listing) and selects it once that listing
    /// lands.
    pub(crate) fn folder_created(&mut self, name: &str) {
        let dir = self.dir.clone();
        self.cache.retain(|(d, _)| d != &dir);
        self.listing = None;
        if !self.outbox.contains(&dir) {
            self.outbox.push(dir);
        }
        self.pending_select = Some(name.to_string());
        self.message = None;
    }

    /// The maintainer's `std::fs::create_dir` failed.
    pub(crate) fn folder_create_failed(&mut self, reason: &str) {
        self.message = Some(msg::folder_picker::create_failed(reason));
    }

    pub(crate) fn set_places(&mut self, places: Vec<Place>) {
        self.places = places;
        self.left_sel = None;
    }

    /// The listing on screen, if it has arrived.
    #[cfg(test)]
    pub(crate) fn listing(&self) -> Option<&FolderListing> {
        self.listing.as_ref()
    }

    fn cached(&self, dir: &Path) -> Option<&FolderListing> {
        if dir == self.dir {
            if let Some(listing) = &self.listing {
                return Some(listing);
            }
        }
        self.cache
            .iter()
            .rev()
            .find(|(d, _)| d == dir)
            .map(|(_, l)| l)
    }

    // -- navigation ---------------------------------------------------------

    /// Shows `dir`: from the cache at once if it was seen before (and
    /// re-read anyway), otherwise "Loading..." until its listing lands.
    ///
    /// Every navigation - a click, `Enter` on a typed or pasted path, the
    /// up button, a quick place - funnels through here, which is why
    /// this is also the one place the verbatim prefix is stripped: a
    /// pasted `\\?\C:\...` path (copied from an app that canonicalizes)
    /// must not survive into `dir`, the path field, or the FOLDERS tree.
    pub(crate) fn navigate(&mut self, dir: PathBuf) {
        let dir = gitpoulpi_core::paths::strip_verbatim_prefix(&dir);
        self.listing = self.cached(&dir).cloned();
        self.dir = dir.clone();
        self.outbox.push(dir);
        self.want_ancestors();
        self.path_draft = self.dir.display().to_string();
        self.draft_pristine = true;
        self.typing = false;
        self.completion = None;
        self.content_sel = None;
        self.content_scroll = 0;
        self.left_sel = None;
        self.reveal_current = true;
        self.message = None;
        self.filter.clear();
    }

    /// Up one level; at a root, says so rather than doing nothing.
    pub(crate) fn go_up(&mut self) {
        match dir_picker::parent_of(&self.dir) {
            Some(parent) => {
                let from = self.dir.clone();
                self.navigate(parent);
                // Land on the folder just left, the way Explorer does.
                self.content_sel = self
                    .content_rows()
                    .iter()
                    .position(|row| self.row_path(row).as_deref() == Some(from.as_path()));
                self.follow = true;
            }
            None => self.message = Some(msg::folder_picker::TOP_OF_DRIVE.get().into()),
        }
    }

    /// Flips "Show hidden folders". A pure render-time
    /// filter over the entries already read - see [`FolderListing::dirs`].
    fn toggle_show_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.clamp();
    }

    /// Opens the inline "New folder" name field.
    fn start_new_folder(&mut self) {
        self.new_folder_draft = Some(String::new());
        self.edit.sync(Some(PickerField::NewFolder));
        self.message = None;
    }

    /// A key while the "New folder" field has the keyboard: the shared
    /// editing core first, then Enter (validated with the same
    /// Windows-name rules `init_dialog::folder_name_error` gives the
    /// Init dialog's Name field) and Esc.
    fn new_folder_key(&mut self, key: KeyEvent) -> PickerOutcome {
        let Some(mut draft) = self.new_folder_draft.take() else {
            return PickerOutcome::Stay;
        };
        self.edit.sync(Some(PickerField::NewFolder));
        if let Some((_, copied)) = self
            .edit
            .dialog_key(PickerField::NewFolder, &mut draft, &key)
        {
            if copied.is_some() {
                self.message = copied;
            }
            self.new_folder_draft = Some(draft);
            return PickerOutcome::Stay;
        }
        match key.code {
            KeyCode::Esc => self.message = None,
            KeyCode::Enter => {
                let name = draft.trim().to_string();
                let reason = if name.is_empty() {
                    Some(msg::folder_picker::TYPE_NAME.get().to_string())
                } else {
                    crate::init_dialog::folder_name_error(&name)
                };
                match reason {
                    Some(reason) => {
                        self.message = Some(reason);
                        self.new_folder_draft = Some(draft);
                    }
                    None => return PickerOutcome::CreateFolder(self.dir.join(&name)),
                }
            }
            _ => self.new_folder_draft = Some(draft),
        }
        PickerOutcome::Stay
    }

    /// What choosing now would pick: the selected subfolder when there is
    /// one, otherwise the folder shown - Windows' own rule.
    pub(crate) fn chosen(&self) -> PathBuf {
        match self
            .content_sel
            .and_then(|i| self.content_rows().get(i).cloned())
        {
            Some(ContentRow::Dir(i)) => self
                .listing
                .as_ref()
                .and_then(|l| l.dirs.get(i))
                .map(|d| d.path.clone())
                .unwrap_or_else(|| self.dir.clone()),
            Some(ContentRow::File(i)) if self.mode == PickerMode::File => self
                .listing
                .as_ref()
                .and_then(|l| l.files.get(i))
                .map(|f| self.dir.join(&f.name))
                .unwrap_or_else(|| self.dir.clone()),
            _ => self.dir.clone(),
        }
    }

    /// The confirm button and its key. A folder picker chooses the selected
    /// folder or else the one shown; a file picker only a selected file
    /// (a folder row or nothing is not a choice there).
    pub(crate) fn choose_selected(&self) -> PickerOutcome {
        let on_file = matches!(
            self.content_sel
                .and_then(|i| self.content_rows().get(i).cloned()),
            Some(ContentRow::File(_))
        );
        if self.mode == PickerMode::File && !on_file {
            PickerOutcome::Stay
        } else {
            PickerOutcome::Choose(self.chosen())
        }
    }

    /// Whether [`Self::chosen`] is known to be a git repository.
    pub(crate) fn chosen_is_repo(&self) -> bool {
        match self
            .content_sel
            .and_then(|i| self.content_rows().get(i).cloned())
        {
            Some(ContentRow::Dir(i)) => self
                .listing
                .as_ref()
                .and_then(|l| l.dirs.get(i))
                .is_some_and(|d| d.is_repo),
            _ => self.listing.as_ref().is_some_and(|l| l.is_repo),
        }
    }

    // -- rows -----------------------------------------------------------------

    /// Whether `name` passes the filter field (case-insensitive substring).
    fn passes_filter(&self, name: &str) -> bool {
        self.filter.is_empty() || name.to_lowercase().contains(&self.filter.to_lowercase())
    }

    pub(crate) fn content_rows(&self) -> Vec<ContentRow> {
        let mut rows = Vec::new();
        if let Some(parent) = dir_picker::parent_of(&self.dir) {
            rows.push(ContentRow::Parent(parent));
        }
        if let Some(listing) = &self.listing {
            rows.extend(
                (0..listing.dirs.len())
                    .filter(|&i| {
                        (self.show_hidden || !listing.dirs[i].is_hidden)
                            && self.passes_filter(&listing.dirs[i].name)
                    })
                    .map(ContentRow::Dir),
            );
            rows.extend(
                (0..listing.files.len())
                    .filter(|&i| {
                        (self.show_hidden || !listing.files[i].is_hidden)
                            && self.passes_filter(&listing.files[i].name)
                    })
                    .map(ContentRow::File),
            );
        }
        rows
    }

    /// `(matching, all)` entries of the folder shown, the `..` row not
    /// counted, hidden ones only when they are shown.
    pub(crate) fn entry_counts(&self) -> (usize, usize) {
        let Some(listing) = &self.listing else {
            return (0, 0);
        };
        let visible = |hidden: bool| self.show_hidden || !hidden;
        let all = listing.dirs.iter().filter(|e| visible(e.is_hidden)).count()
            + listing
                .files
                .iter()
                .filter(|e| visible(e.is_hidden))
                .count();
        let shown = self
            .content_rows()
            .iter()
            .filter(|r| !matches!(r, ContentRow::Parent(_)))
            .count();
        (shown, all)
    }

    /// The filter text changed: the rows moved under the selection.
    fn after_filter_edit(&mut self) {
        self.content_sel = None;
        self.content_scroll = 0;
        self.clamp();
    }

    /// How many content rows are selectable: the parent and the folders
    /// always; files too, in [`PickerMode::File`] (they come last).
    fn content_selectable(&self) -> usize {
        self.content_rows()
            .iter()
            .filter(|r| self.mode == PickerMode::File || !matches!(r, ContentRow::File(_)))
            .count()
    }

    fn row_path(&self, row: &ContentRow) -> Option<PathBuf> {
        match row {
            ContentRow::Parent(p) => Some(p.clone()),
            ContentRow::Dir(i) => self
                .listing
                .as_ref()
                .and_then(|l| l.dirs.get(*i))
                .map(|d| d.path.clone()),
            ContentRow::File(i) if self.mode == PickerMode::File => self
                .listing
                .as_ref()
                .and_then(|l| l.files.get(*i))
                .map(|f| self.dir.join(&f.name)),
            ContentRow::File(_) => None,
        }
    }

    pub(crate) fn left_rows(&self) -> Vec<LeftRow> {
        let mut rows = Vec::new();
        if !self.places.is_empty() {
            rows.push(LeftRow::Heading(msg::folder_picker::HEADING_QUICK));
            rows.extend((0..self.places.len()).map(LeftRow::Place));
        }
        rows.push(LeftRow::Heading(msg::folder_picker::HEADING_FOLDERS));
        let chain: Vec<PathBuf> = {
            let mut chain: Vec<PathBuf> = self
                .dir
                .ancestors()
                .filter(|p| !p.as_os_str().is_empty())
                .map(Path::to_path_buf)
                .collect();
            chain.reverse();
            chain
        };
        if !chain.is_empty() {
            self.push_node(&mut rows, &chain, 0);
        }
        rows
    }

    /// The tree row of the open path's entry `d`, then what hangs below it:
    /// the current folder's sub-folders, or - with siblings shown - every
    /// sub-folder of this level, the next path entry (and its own subtree)
    /// standing in its place among them.
    fn push_node(&self, rows: &mut Vec<LeftRow>, chain: &[PathBuf], d: usize) {
        let path = &chain[d];
        let depth = chain.len();
        let current = d + 1 == depth;
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        rows.push(LeftRow::Tree {
            path: path.clone(),
            name,
            depth: d,
            current,
            is_repo: current && self.listing.as_ref().is_some_and(|l| l.is_repo),
            is_hidden: false,
            open: true,
        });
        if current {
            if let Some(listing) = &self.listing {
                for entry in listing
                    .dirs
                    .iter()
                    .filter(|e| self.show_hidden || !e.is_hidden)
                {
                    rows.push(Self::folder_row(entry, depth));
                }
            }
            return;
        }
        let next = &chain[d + 1];
        let level = if self.show_siblings {
            self.cached(path)
        } else {
            None
        };
        let Some(listing) = level else {
            self.push_node(rows, chain, d + 1);
            return;
        };
        let mut placed = false;
        let mut shown = 0;
        for entry in &listing.dirs {
            if entry.path == *next
                || next
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy() == entry.name)
            {
                self.push_node(rows, chain, d + 1);
                placed = true;
            } else if (self.show_hidden || !entry.is_hidden) && shown < SIBLING_CAP {
                shown += 1;
                rows.push(Self::folder_row(entry, d + 1));
            }
        }
        if !placed {
            self.push_node(rows, chain, d + 1);
        }
    }

    fn folder_row(entry: &DirEntryInfo, depth: usize) -> LeftRow {
        LeftRow::Tree {
            path: entry.path.clone(),
            name: entry.name.clone(),
            depth,
            current: false,
            is_repo: entry.is_repo,
            is_hidden: entry.is_hidden,
            open: false,
        }
    }

    fn clamp(&mut self) {
        let n = self.content_selectable();
        if let Some(sel) = self.content_sel {
            self.content_sel = if n == 0 { None } else { Some(sel.min(n - 1)) };
        }
    }

    // -- the path field and its completion ------------------------------------

    /// The folders that could continue what was typed, best first. Empty
    /// until the folder they are in has been listed (a request is queued).
    pub(crate) fn completions(&self) -> Vec<DirEntryInfo> {
        if !self.typing {
            return Vec::new();
        }
        let fragment = dir_picker::split_fragment(&self.dir, &self.path_draft);
        match self.cached(&fragment.dir) {
            Some(listing) => {
                dir_picker::filter_entries(listing.dirs.clone(), &fragment.prefix).collect()
            }
            None => Vec::new(),
        }
    }

    fn after_edit(&mut self) {
        self.typing = true;
        self.completion = None;
        let fragment = dir_picker::split_fragment(&self.dir, &self.path_draft);
        if self.cached(&fragment.dir).is_none() && !self.outbox.contains(&fragment.dir) {
            self.outbox.push(fragment.dir);
        }
    }

    /// Keeps the path field's caret with its focus, and a pristine draft
    /// all selected - the address bar's select-all, which is why the
    /// first key typed replaces it and Backspace clears it.
    fn sync_edit(&mut self) {
        self.edit.sync(match self.zone {
            Zone::Path => Some(PickerField::Path),
            Zone::Filter => Some(PickerField::Filter),
            _ => None,
        });
        if self.zone == Zone::Path && self.draft_pristine {
            self.edit
                .caret_mut(PickerField::Path)
                .select_all(&self.path_draft);
        }
    }

    /// Pasted text goes into the field with the keyboard: the "New
    /// folder" field while it is open, else the path field.
    pub(crate) fn paste(&mut self, text: &str) {
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if let Some(draft) = self.new_folder_draft.as_mut() {
            self.edit.sync(Some(PickerField::NewFolder));
            self.edit
                .caret_mut(PickerField::NewFolder)
                .insert(draft, &clean);
            return;
        }
        if self.zone == Zone::Filter {
            self.sync_edit();
            self.edit
                .caret_mut(PickerField::Filter)
                .insert(&mut self.filter, &clean);
            self.after_filter_edit();
            return;
        }
        self.zone = Zone::Path;
        self.sync_edit();
        self.edit
            .caret_mut(PickerField::Path)
            .insert(&mut self.path_draft, &clean);
        self.draft_pristine = false;
        self.after_edit();
    }

    fn accept_completion(&mut self, index: usize) {
        if let Some(entry) = self.completions().get(index) {
            self.path_draft = format!("{}{}", entry.path.display(), std::path::MAIN_SEPARATOR);
            self.draft_pristine = false;
            self.edit.reset();
            self.after_edit();
        }
    }

    /// `Enter` in the path field: into the typed folder, or - when it is
    /// already the folder shown - choose it.
    fn submit_path(&mut self) -> PickerOutcome {
        let typed = self.path_draft.trim();
        if typed.is_empty() {
            self.message = Some(msg::folder_picker::TYPE_PATH.get().into());
            return PickerOutcome::Stay;
        }
        let target = dir_picker::absolutize(&self.dir, typed);
        if self.mode == PickerMode::File {
            // A file picker never chooses the folder shown; a typed path
            // that names a file of the listing is the choice (no disk read:
            // the listing is already here).
            let named = target
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .filter(|_| target.parent() == Some(self.dir.as_path()));
            let listed = named.is_some_and(|name| {
                self.listing
                    .as_ref()
                    .is_some_and(|l| l.files.iter().any(|f| f.name == name))
            });
            if listed {
                return PickerOutcome::Choose(target);
            }
            if target == self.dir {
                return PickerOutcome::Stay;
            }
        } else if target == self.dir {
            return PickerOutcome::Choose(self.dir.clone());
        }
        self.navigate(target);
        self.zone = Zone::Path;
        PickerOutcome::Stay
    }
}

/// The picker's size in a frame: generous, capped, centered.
pub(crate) fn picker_rect(frame: Rect) -> Rect {
    WIN.place(
        frame,
        dialog_kit::centered(
            frame,
            frame.width.saturating_sub(4).min(120),
            frame.height.saturating_sub(2).min(32),
        ),
    )
}

/// The picker moves and resizes.
pub(crate) const WIN: dialog_kit::Win = dialog_kit::Win::resizable("folder_picker", 44, 12);

#[cfg(test)]
mod tests;

impl dialog_kit::Dismiss for FolderPicker {
    fn is_dirty(&self) -> bool {
        false
    }

    /// Browsing loses nothing: any close just closes.
    fn guard(&self) -> dialog_kit::Guard {
        dialog_kit::Guard::None
    }

    fn close_hit(&self) -> &dialog_kit::CloseHit {
        &self.close
    }

    fn close_hit_mut(&mut self) -> &mut dialog_kit::CloseHit {
        &mut self.close
    }
}
