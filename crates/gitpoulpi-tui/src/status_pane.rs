//! The third pane's row model, shared by both of its modes.
//!
//! The same file list is shown two ways, a flat list of full
//! paths ("Path") or a collapsible directory tree ("Tree"), for both the
//! working-changes pane and a selected commit's changed files. The tree walk
//! lives here as a pure `&[PathBuf] -> Vec<TreeRow>` function that both
//! render sites call, so row-building can be unit-tested directly.
//! Nothing here touches ratatui or the daemon: it maps paths to rows, and
//! the caller decides how a row is painted and what clicking it does.

use gitpoulpi_i18n::msg::status_pane as msg;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{ConflictSide, FileStatus, StatusEntry, WorkingTree};

use crate::context_menu::Entry;

/// Whether a file list is shown as flat full paths or as a directory
/// tree: the `Path`/`Tree` segmented control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileListMode {
    Path,
    Tree,
}

impl FileListMode {
    pub(crate) fn toggled(self) -> Self {
        match self {
            FileListMode::Path => FileListMode::Tree,
            FileListMode::Tree => FileListMode::Path,
        }
    }
}

/// Which half of the working-changes pane a file row belongs to. A
/// partially-staged file legitimately appears in both, which is why this
/// is carried per-row rather than derived from the entry alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StatusSection {
    Unstaged,
    Staged,
}

impl StatusSection {
    /// Display order, top to bottom. Unstaged first: it's the half the
    /// user is usually acting on.
    pub(crate) const ORDER: [StatusSection; 2] = [StatusSection::Unstaged, StatusSection::Staged];

    pub(crate) fn title(self) -> &'static str {
        match self {
            StatusSection::Unstaged => msg::UNSTAGED_TITLE.get(),
            StatusSection::Staged => msg::STAGED_TITLE.get(),
        }
    }

    /// Label of the button in this section's header: the bulk action that
    /// empties it.
    pub(crate) fn bulk_label(self) -> &'static str {
        match self {
            StatusSection::Unstaged => msg::STAGE_ALL.get(),
            StatusSection::Staged => msg::UNSTAGE_ALL.get(),
        }
    }

    /// Label of the per-row button revealed on hover.
    pub(crate) fn row_label(self) -> &'static str {
        match self {
            StatusSection::Unstaged => msg::STAGE_FILE.get(),
            StatusSection::Staged => msg::UNSTAGE_FILE.get(),
        }
    }
}

/// One row produced by the tree walk. `File` carries an index back into
/// the slice it was built from rather than a borrowed entry, so the same
/// walk serves the working tree (`StatusEntry`) and a commit's changed
/// files (`FileDiff`) without being generic over either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TreeRow {
    Folder {
        depth: usize,
        /// Full path of the folder, used as the collapse-set key.
        path: PathBuf,
        label: String,
        collapsed: bool,
    },
    File {
        depth: usize,
        label: String,
        index: usize,
    },
}

impl TreeRow {
    /// A row the user can act on (stage, open a diff, ...). Folder rows
    /// are navigation only, so callers skip them when mapping a selection
    /// to a file.
    pub(crate) fn file_index(&self) -> Option<usize> {
        match self {
            TreeRow::File { index, .. } => Some(*index),
            TreeRow::Folder { .. } => None,
        }
    }
}

/// Directory components of `path`, i.e. everything but the file name.
fn parent_components(path: &Path) -> Vec<String> {
    let mut parts: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    parts.pop(); // the file name itself
    parts
}

fn file_label(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned())
}

/// Builds the visible rows for a list of paths.
///
/// In `Path` mode this is one flat row per path, full path as the label. In
/// `Tree` mode, directories become their own rows and a directory present
/// in `collapsed` hides its whole subtree, nested folder rows included (the
/// part a naive "skip files under a collapsed dir" filter gets wrong).
///
/// `filter`, when non-empty, keeps only paths containing it
/// (case-insensitive) and is applied *before* the walk, so folders that end
/// up empty never get a row.
pub(crate) fn build_tree_rows(
    paths: &[PathBuf],
    mode: FileListMode,
    collapsed: &HashSet<PathBuf>,
    filter: &str,
) -> Vec<TreeRow> {
    let needle = filter.to_lowercase();
    let mut kept: Vec<(usize, &PathBuf)> = paths
        .iter()
        .enumerate()
        .filter(|(_, p)| needle.is_empty() || p.to_string_lossy().to_lowercase().contains(&needle))
        .collect();

    if mode == FileListMode::Path {
        return kept
            .into_iter()
            .map(|(index, p)| TreeRow::File {
                depth: 0,
                label: p.to_string_lossy().into_owned(),
                index,
            })
            .collect();
    }

    // Directories before their siblings' files, stable within a level.
    kept.sort_by_key(|(_, p)| *p);

    let mut rows = Vec::new();
    // The directory stack currently "open" in the output, as components.
    let mut open: Vec<String> = Vec::new();

    for (index, path) in kept {
        let dirs = parent_components(path);

        // Unwind to the deepest shared ancestor.
        let shared = open
            .iter()
            .zip(dirs.iter())
            .take_while(|(a, b)| a == b)
            .count();
        open.truncate(shared);

        // Descend into the new directories, emitting a row for each, but
        // stop the moment any ancestor is collapsed. The collapsed ancestor
        // may have been emitted on an EARLIER iteration and so already be
        // inside `open`, in which case `shared` skips right past it and the
        // descent would emit its children; so the check runs over the whole
        // path, not just the newly-entered part.
        let mut hidden = false;
        for (depth, part) in dirs.iter().enumerate() {
            let folder_path: PathBuf = dirs[..=depth].iter().collect();
            let is_collapsed = collapsed.contains(&folder_path);
            if depth >= shared {
                rows.push(TreeRow::Folder {
                    depth,
                    path: folder_path,
                    label: part.clone(),
                    collapsed: is_collapsed,
                });
                open.push(part.clone());
            }
            if is_collapsed {
                hidden = true;
                break;
            }
        }

        if !hidden {
            rows.push(TreeRow::File {
                depth: dirs.len(),
                label: file_label(path),
                index,
            });
        }
    }

    rows
}

/// Every folder path appearing in `paths`, for "Collapse All".
pub(crate) fn all_folders(paths: &[PathBuf]) -> HashSet<PathBuf> {
    let mut out = HashSet::new();
    for path in paths {
        let dirs = parent_components(path);
        for d in 0..dirs.len() {
            out.insert(dirs[..=d].iter().collect());
        }
    }
    out
}

/// Whether this row is an unmerged path: a file git holds at stages 1/2/3
/// rather than stage 0.
///
/// Porcelain v2 reports an unmerged path with BOTH columns set (`u` records
/// become `index: Conflicted, worktree: Conflicted`, see the cli backend's
/// `status.rs`), so the naive per-column split below would list the same
/// conflicted file in Unstaged *and* Staged at once.
pub(crate) fn entry_is_conflicted(entry: &StatusEntry) -> bool {
    entry.index == Some(FileStatus::Conflicted) || entry.worktree == Some(FileStatus::Conflicted)
}

/// Splits the working tree into the two sections. A file that is
/// both staged and further modified appears in both, which is why this
/// returns paths per section instead of partitioning.
///
/// A *conflicted* file is the exception: it is pinned to the Unstaged half
/// alone. It has work left to do, "unstage this conflict" is not a
/// `GitBackend` operation, and listing it twice would show Take Ours / Take
/// Theirs rows in a section where they read as already-resolved.
pub(crate) fn section_paths(tree: &WorkingTree, section: StatusSection) -> Vec<PathBuf> {
    tree.files
        .iter()
        .filter(|f| match section {
            StatusSection::Staged => f.index.is_some() && !entry_is_conflicted(f),
            StatusSection::Unstaged => f.worktree.is_some(),
        })
        .map(|f| f.path.clone())
        .collect()
}

/// The status a row should show: the index status in the staged section,
/// the worktree status in the unstaged one.
pub(crate) fn entry_status(entry: &StatusEntry, section: StatusSection) -> Option<FileStatus> {
    match section {
        StatusSection::Staged => entry.index,
        StatusSection::Unstaged => entry.worktree,
    }
}

/// Single-character marker per status, for the gutter: a small
/// pencil (U+270E) for a modification, drawn in `file_status_color`'s amber.
///
/// All BMP, no emoji-presentation glyph and no variation selector: a
/// VS16-dependent glyph in a list gutter produces the ref-pill rendering
/// "hole".
pub(crate) fn status_marker(status: FileStatus) -> &'static str {
    match status {
        FileStatus::Added | FileStatus::Untracked => "+",
        FileStatus::Modified | FileStatus::Staged => "\u{270E}",
        FileStatus::Deleted => "-",
        FileStatus::Renamed => ">",
        FileStatus::Conflicted => "!",
    }
}

/// One entry of a file's right-click menu (and its palette twin).
///
/// Every variant runs: `apply_file_context_action` (and
/// `apply_lfs_file_action` for the LFS rows) is the single place each
/// is carried out, for the mouse, the keyboard menu and the palette alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileContextAction {
    Stage,
    Unstage,
    /// Open the merge conflict editor on this conflicted file. Offered on
    /// a conflicted row only, ahead of the three whole-file resolutions.
    OpenMergeEditor,
    /// Resolve an unmerged path by taking one side of it whole
    /// (`ConflictResolution::TakeSide`). Only offered on a conflicted
    /// row - see `file_context_entries` - because on any other file the
    /// backend has no stages 1/2/3 to take from.
    ResolveUsing(ConflictSide),
    Discard,
    /// Append `/<path>` to `.gitignore`.
    IgnoreFile,
    /// Append `*.<ext>` to `.gitignore`.
    IgnoreExtension,
    /// Write a `.gitattributes` rule making git diff this file as binary /
    /// as text, or every file with its extension (`treat_binary` etc.:
    /// each opens the "which file?" picker first, like the ignore rows).
    TreatFileBinary,
    TreatFileText,
    TreatExtensionBinary,
    TreatExtensionText,
    /// Stash only this path (`GitBackend::stash_push_paths`).
    StashFile,
    FileHistory,
    FileBlame,
    ExternalDiff,
    /// The configured external editor (`general.external_editor`).
    OpenInEditor,
    OpenInDefault,
    /// The image editors list on a picture (`image_view::edit_in`).
    EditImageIn,
    /// The apps the OS registers for the file's type (`open_with`).
    OpenWith,
    ShowInFolder,
    CopyPath,
    CreatePatch,
    /// The built-in editor tab.
    EditFile,
    /// Remove the file from disk, after a confirmation.
    DeleteFile,
    /// Git LFS (only offered when LFS is on and `git-lfs` is usable, see
    /// `lfs_ui::LfsUi::file_menu_actions`): route this one file, or every
    /// file of its extension, through LFS; take / release / break a lock.
    LfsTrackFile,
    LfsTrackExtension,
    LfsLock,
    LfsUnlock,
    LfsForceUnlock,
    /// Rewrite the file's encoding, line endings or indentation on disk
    /// (`Action::ConvertFiles`).
    Convert(gitpoulpi_core::textconv::ConvertOp),
}

impl FileContextAction {
    /// The row's label for actions whose text does not depend on the
    /// file. `file_context_entries` words the name-bearing ones itself.
    pub(crate) fn label(self) -> &'static str {
        match self {
            FileContextAction::Stage => msg::FA_STAGE.get(),
            FileContextAction::Unstage => msg::FA_UNSTAGE.get(),
            FileContextAction::OpenMergeEditor => msg::FA_OPEN_MERGE_EDITOR.get(),
            FileContextAction::ResolveUsing(ConflictSide::Ours) => msg::FA_RESOLVE_OURS.get(),
            FileContextAction::ResolveUsing(ConflictSide::Theirs) => msg::FA_RESOLVE_THEIRS.get(),
            FileContextAction::ResolveUsing(ConflictSide::Base) => msg::FA_RESOLVE_BASE.get(),
            FileContextAction::Discard => msg::FA_DISCARD.get(),
            FileContextAction::IgnoreFile => msg::FA_IGNORE_FILE.get(),
            FileContextAction::IgnoreExtension => msg::FA_IGNORE_EXTENSION.get(),
            FileContextAction::TreatFileBinary => msg::FA_TREAT_FILE_BINARY.get(),
            FileContextAction::TreatFileText => msg::FA_TREAT_FILE_TEXT.get(),
            FileContextAction::TreatExtensionBinary => msg::FA_TREAT_EXTENSION_BINARY.get(),
            FileContextAction::TreatExtensionText => msg::FA_TREAT_EXTENSION_TEXT.get(),
            FileContextAction::StashFile => msg::FA_STASH_FILE.get(),
            FileContextAction::FileHistory => msg::FA_FILE_HISTORY.get(),
            FileContextAction::FileBlame => msg::FA_FILE_BLAME.get(),
            FileContextAction::ExternalDiff => msg::FA_EXTERNAL_DIFF.get(),
            FileContextAction::OpenInEditor => msg::FA_OPEN_IN_EDITOR.get(),
            FileContextAction::OpenInDefault => msg::FA_OPEN_IN_DEFAULT.get(),
            FileContextAction::EditImageIn => gitpoulpi_i18n::msg::image_edit_in::MENU.get(),
            FileContextAction::OpenWith => gitpoulpi_i18n::msg::open_with::MENU.get(),
            FileContextAction::ShowInFolder => msg::FA_SHOW_IN_FOLDER.get(),
            FileContextAction::CopyPath => msg::FA_COPY_PATH.get(),
            FileContextAction::CreatePatch => msg::FA_CREATE_PATCH.get(),
            FileContextAction::EditFile => msg::FA_EDIT_FILE.get(),
            FileContextAction::DeleteFile => msg::FA_DELETE_FILE.get(),
            FileContextAction::LfsTrackFile => msg::FA_LFS_TRACK_FILE.get(),
            FileContextAction::LfsTrackExtension => msg::FA_LFS_TRACK_EXTENSION.get(),
            FileContextAction::LfsLock => msg::FA_LFS_LOCK.get(),
            FileContextAction::LfsUnlock => msg::FA_LFS_UNLOCK.get(),
            FileContextAction::LfsForceUnlock => msg::FA_LFS_FORCE_UNLOCK.get(),
            FileContextAction::Convert(_) => gitpoulpi_i18n::msg::pane_menu::CONVERT.get(),
        }
    }

    /// Whether running this needs the confirm modal first: discarding
    /// throws away uncommitted work and deleting removes the file, both
    /// with no undo, so both go through the same `PendingDestructive`
    /// path as branch delete and stash drop.
    pub(crate) fn is_destructive(self) -> bool {
        matches!(
            self,
            FileContextAction::Discard | FileContextAction::DeleteFile
        )
    }
}

/// What a file row's menu is built from.
pub(crate) struct FileMenuInput<'a> {
    pub(crate) section: StatusSection,
    /// The row's own status (index status in Staged, worktree in Unstaged).
    pub(crate) status: Option<FileStatus>,
    pub(crate) conflicted: bool,
    pub(crate) path: &'a Path,
    /// `LfsUi::file_menu_actions` for this path (empty when LFS is off).
    pub(crate) lfs: Vec<FileContextAction>,
    /// "VS Code", "notepad++", ... for the `Open in <editor>` row.
    pub(crate) editor_name: String,
    /// Whether git diffs the file as binary (`None`: git has not answered
    /// yet, the row is a grayed-out placeholder until it does).
    pub(crate) binary: Option<bool>,
}

/// The menu for a file, as a tree for
/// `context_menu::Menu` (separators between the groups, `Ignore` and `LFS`
/// as submenus).
///
/// `conflicted` puts the three Take-a-side rows on top instead of Stage:
/// staging an unmerged path without choosing a side is what produces a
/// commit full of `<<<<<<<` markers, so the resolution leads and plain
/// `Stage` is left off until the file is resolved.
///
/// An untracked file drops `Discard changes` (Delete
/// file is the way to lose it) and the history/blame/diff rows, which have
/// nothing to show. The Staged list swaps `Stage` for `Unstage` and drops
/// `Discard changes` and `Ignore`, which would do nothing for an indexed
/// file.
pub(crate) fn file_context_entries(input: &FileMenuInput<'_>) -> Vec<Entry<FileContextAction>> {
    use FileContextAction as A;
    let name = input
        .path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| input.path.to_string_lossy().into_owned());
    let ext = input
        .path
        .extension()
        .map(|e| e.to_string_lossy().into_owned());
    // Red rows are the ones that stop at a confirm modal.
    let tool = |label: &str, a: A| Entry::Action {
        label: label.to_string(),
        action: a,
        destructive: a.is_destructive(),
        key: match a {
            A::Stage => Some(crate::keymap::Action::GitStageFile),
            A::Unstage => Some(crate::keymap::Action::GitUnstageFile),
            _ => None,
        },
    };

    if input.conflicted {
        return vec![
            tool(A::OpenMergeEditor.label(), A::OpenMergeEditor),
            Entry::Separator,
            tool(
                A::ResolveUsing(ConflictSide::Ours).label(),
                A::ResolveUsing(ConflictSide::Ours),
            ),
            tool(
                A::ResolveUsing(ConflictSide::Theirs).label(),
                A::ResolveUsing(ConflictSide::Theirs),
            ),
            tool(
                A::ResolveUsing(ConflictSide::Base).label(),
                A::ResolveUsing(ConflictSide::Base),
            ),
            Entry::Separator,
            tool(A::FileHistory.label(), A::FileHistory),
            tool(A::FileBlame.label(), A::FileBlame),
            Entry::Separator,
            tool(A::ExternalDiff.label(), A::ExternalDiff),
            tool(A::CopyPath.label(), A::CopyPath),
        ];
    }

    let staged = input.section == StatusSection::Staged;
    let untracked = matches!(input.status, Some(FileStatus::Untracked));
    let on_disk = !matches!(input.status, Some(FileStatus::Deleted));

    let mut out: Vec<Entry<A>> = Vec::new();
    out.push(if staged {
        tool(msg::FA_UNSTAGE.get(), A::Unstage)
    } else {
        tool(msg::FA_STAGE.get(), A::Stage)
    });
    if !staged && !untracked {
        out.push(tool(A::Discard.label(), A::Discard));
    }
    if !staged {
        let mut items = vec![tool(&msg::fm_ignore_name(&name), A::IgnoreFile)];
        if let Some(ext) = &ext {
            items.push(tool(&msg::fm_ignore_ext(ext), A::IgnoreExtension));
        }
        out.push(Entry::submenu(msg::FM_IGNORE.get(), items));
    }
    out.push(match input.binary {
        Some(binary) => treat_submenu(binary, &name, ext.as_deref()),
        None => Entry::Disabled {
            label: msg::FM_TREAT_PENDING.get().into(),
        },
    });
    // The row keeps its place whatever git answers, so a click aimed at the
    // rows below cannot land on another one when the answer arrives.
    if on_disk {
        use gitpoulpi_i18n::msg::pane_menu as pm;
        out.push(match input.binary {
            Some(false) => crate::pane_menu::convert_submenu(A::Convert),
            Some(true) => Entry::disabled(pm::CONVERT.get(), pm::BINARY_FILE.get()),
            None => Entry::Disabled {
                label: pm::CONVERT.get().into(),
            },
        });
    }
    if !input.lfs.is_empty() {
        let items = input
            .lfs
            .iter()
            .map(|&a| match a {
                A::LfsTrackFile => tool(&msg::fm_track_name(&name), a),
                A::LfsTrackExtension => tool(&msg::fm_track_ext(ext.as_deref().unwrap_or("")), a),
                other => tool(other.label(), other),
            })
            .collect();
        out.push(Entry::submenu("LFS", items));
    }
    out.push(tool(A::StashFile.label(), A::StashFile));
    out.push(Entry::Separator);

    if !untracked {
        out.push(tool(A::FileHistory.label(), A::FileHistory));
        out.push(tool(A::FileBlame.label(), A::FileBlame));
        out.push(Entry::Separator);
    }

    if untracked {
        out.push(Entry::disabled(
            A::ExternalDiff.label(),
            msg::FM_UNTRACKED.get(),
        ));
    } else {
        out.push(tool(A::ExternalDiff.label(), A::ExternalDiff));
    }
    if on_disk {
        out.push(tool(&msg::fm_open_in(&input.editor_name), A::OpenInEditor));
        out.push(tool(A::OpenInDefault.label(), A::OpenInDefault));
        if crate::image_view::edit_in::is_image_path(input.path) {
            out.push(tool(A::EditImageIn.label(), A::EditImageIn));
        } else if crate::open_with::menu_offered(&input.path.to_string_lossy()) {
            out.push(tool(A::OpenWith.label(), A::OpenWith));
        }
        out.push(tool(A::ShowInFolder.label(), A::ShowInFolder));
    }
    out.push(Entry::Separator);

    out.push(tool(A::CopyPath.label(), A::CopyPath));
    out.push(tool(A::CreatePatch.label(), A::CreatePatch));
    if on_disk {
        out.push(Entry::Separator);
        out.push(tool(A::EditFile.label(), A::EditFile));
        out.push(tool(A::DeleteFile.label(), A::DeleteFile));
    }
    out
}

/// The "Treat as Binary" / "Treat as Text" submenu of a file row: a text
/// file offers Binary, a binary one Text, each for this file or for its
/// extension. A separate group so it merges apart from the "Open with" row.
fn treat_submenu(binary: bool, name: &str, ext: Option<&str>) -> Entry<FileContextAction> {
    use FileContextAction as A;
    let (file, by_ext, title) = if binary {
        (
            A::TreatFileText,
            A::TreatExtensionText,
            msg::FM_TREAT_TEXT.get(),
        )
    } else {
        (
            A::TreatFileBinary,
            A::TreatExtensionBinary,
            msg::FM_TREAT_BINARY.get(),
        )
    };
    let row = |label: String, a: A| Entry::Action {
        label,
        action: a,
        destructive: false,
        key: None,
    };
    let mut items = vec![row(msg::fm_treat_name(name), file)];
    if let Some(ext) = ext {
        items.push(row(msg::fm_ignore_ext(ext), by_ext));
    }
    Entry::submenu(title, items)
}

/// A folder row's right-click menu, and the
/// empty-area menu's one row. Run by `status_list::apply_folder_context_action`;
/// `CreateFile` (and the editor it opens) needs state only the event loop
/// owns, so the loop runs that one itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FolderContextAction {
    /// Stage every change beneath the folder (Unstaged list).
    Stage,
    /// Unstage every change beneath the folder (Staged list).
    Unstage,
    /// Discard every uncommitted change beneath the folder, tracked and
    /// untracked alike - `GitBackend::discard` over every path
    /// `unstaged_paths_under` finds under it. Confirmed first.
    Discard,
    /// Append `<dir>/` to `.gitignore`.
    IgnoreFolder,
    /// Write the `.gitattributes` rule making git diff everything below the
    /// folder as binary / as text (through the "which file?" picker).
    TreatFolderBinary,
    TreatFolderText,
    /// `GitBackend::stash_push_paths` with the folder as the pathspec.
    StashFolder,
    /// A patch of the changes beneath the folder.
    CreatePatch,
    /// The `general.file_explorer` template, with the folder.
    OpenFolder,
    /// Ask for a name and create the file in the folder (the repo root for
    /// the empty-area menu).
    CreateFile,
    /// Stage / unstage / discard everything: the empty area's twins of the
    /// header buttons.
    StageAll,
    UnstageAll,
    DiscardAll,
    /// The repository's own folder, in the file explorer.
    OpenRepoFolder,
    /// Rewrite the encoding, line endings or indentation of the files
    /// beneath the folder (`Action::ConvertFiles`).
    Convert(gitpoulpi_core::textconv::ConvertOp),
    /// Expand/collapse all folders of the list, or a folder and its
    /// subfolders (`crate::tree_fold`).
    Fold(crate::tree_fold::TreeFold),
}

impl FolderContextAction {
    pub(crate) fn label(self) -> &'static str {
        match self {
            FolderContextAction::Stage => msg::FO_STAGE.get(),
            FolderContextAction::Unstage => msg::FO_UNSTAGE.get(),
            FolderContextAction::Discard => msg::FO_DISCARD.get(),
            FolderContextAction::IgnoreFolder => msg::FO_IGNORE.get(),
            FolderContextAction::TreatFolderBinary => msg::FO_TREAT_BINARY.get(),
            FolderContextAction::TreatFolderText => msg::FO_TREAT_TEXT.get(),
            FolderContextAction::StashFolder => msg::FO_STASH.get(),
            FolderContextAction::CreatePatch => msg::FO_CREATE_PATCH.get(),
            FolderContextAction::OpenFolder => msg::FO_OPEN.get(),
            FolderContextAction::CreateFile => msg::FO_CREATE_FILE.get(),
            FolderContextAction::StageAll => msg::STAGE_ALL.get(),
            FolderContextAction::UnstageAll => msg::UNSTAGE_ALL.get(),
            FolderContextAction::DiscardAll => gitpoulpi_i18n::msg::menu_log::DISCARD_ALL.get(),
            FolderContextAction::OpenRepoFolder => gitpoulpi_i18n::msg::pane_menu::OPEN_REPO.get(),
            FolderContextAction::Convert(_) => gitpoulpi_i18n::msg::pane_menu::CONVERT.get(),
            FolderContextAction::Fold(op) => op.label(),
        }
    }

    /// Every folder discard goes through the confirm modal - the same
    /// no-undo-possible category as a single file's Discard.
    pub(crate) fn is_destructive(self) -> bool {
        matches!(
            self,
            FolderContextAction::Discard | FolderContextAction::DiscardAll
        )
    }
}

/// The folder as the `.gitignore` line `dir/`, with forward slashes (the
/// menu label; the line written is relative to the file chosen).
pub(crate) fn folder_ignore_pattern(folder: &Path) -> String {
    format!("{}/", folder.to_string_lossy().replace('\\', "/"))
}

/// A folder row's menu. The Staged list swaps Stage
/// for Unstage and has no Discard or Ignore (nothing there to discard or
/// ignore for an indexed path).
pub(crate) fn folder_context_entries(
    section: StatusSection,
    folder: &Path,
) -> Vec<Entry<FolderContextAction>> {
    use FolderContextAction as A;
    let tool = |label: &str, a: A| Entry::Action {
        label: label.to_string(),
        action: a,
        destructive: a.is_destructive(),
        key: None,
    };
    let staged = section == StatusSection::Staged;
    let mut out = vec![if staged {
        tool(A::Unstage.label(), A::Unstage)
    } else {
        tool(A::Stage.label(), A::Stage)
    }];
    if !staged {
        out.push(tool(A::Discard.label(), A::Discard));
        out.push(tool(
            &msg::fo_ignore_named(folder_ignore_pattern(folder)),
            A::IgnoreFolder,
        ));
    }
    out.push(Entry::submenu(
        msg::FO_TREAT.get(),
        vec![
            tool(A::TreatFolderBinary.label(), A::TreatFolderBinary),
            tool(A::TreatFolderText.label(), A::TreatFolderText),
        ],
    ));
    out.push(crate::pane_menu::convert_submenu(A::Convert));
    out.push(tool(A::StashFolder.label(), A::StashFolder));
    out.push(tool(A::CreatePatch.label(), A::CreatePatch));
    out.push(Entry::Separator);
    out.push(tool(A::OpenFolder.label(), A::OpenFolder));
    out.push(tool(A::CreateFile.label(), A::CreateFile));
    out.push(Entry::Separator);
    out.extend(crate::tree_fold::entries(
        &[
            crate::tree_fold::TreeFold::ExpandBelow,
            crate::tree_fold::TreeFold::CollapseBelow,
        ],
        crate::tree_fold::FoldCtx::Status,
        A::Fold,
    ));
    out
}

/// The menu a right-click on the Unstaged pane's empty area opens.
pub(crate) fn empty_area_entries(section: StatusSection) -> Vec<Entry<FolderContextAction>> {
    crate::pane_menu::status_empty_entries(match section {
        StatusSection::Unstaged => crate::file_selection::FileList::Unstaged,
        StatusSection::Staged => crate::file_selection::FileList::Staged,
    })
}

/// Every Unstaged path at or beneath `folder`: what a folder's own Discard
/// acts on and, with `folder` being the tree's root (`""`), what the global
/// "Discard all changes" button acts on. Scoped to Unstaged only, matching
/// `folder_context_actions`: a file that is only staged has nothing to
/// discard without first unstaging it, a decision this menu leaves to the
/// user.
pub(crate) fn unstaged_paths_under(tree: &WorkingTree, folder: &Path) -> Vec<PathBuf> {
    section_paths(tree, StatusSection::Unstaged)
        .into_iter()
        .filter(|p| folder.as_os_str().is_empty() || p.starts_with(folder))
        .collect()
}

/// The confirm modal's prompt for discarding `paths`: the total count and,
/// when any are untracked, how many ("Discard 12 files (3 untracked)?").
/// Untracked is read from each path's Unstaged status in `tree`, not
/// assumed, since a caller could be asked to discard a path this working
/// tree no longer has an entry for (a race with another client).
pub(crate) fn discard_confirmation_prompt(tree: &WorkingTree, paths: &[PathBuf]) -> String {
    let untracked = paths
        .iter()
        .filter(|p| {
            tree.files
                .iter()
                .any(|f| &f.path == *p && matches!(f.worktree, Some(FileStatus::Untracked)))
        })
        .count();
    let n = paths.len() as u64;
    if untracked == 0 {
        msg::discard_prompt(n)
    } else {
        msg::discard_prompt_untracked(n, untracked)
    }
}

/// OSC 52: ask the *terminal* to put `text` on the system clipboard. A TUI
/// cannot reach the clipboard itself; OSC 52 is the standard way to ask, it
/// works over SSH and needs no dependency (like the OSC 22 cursor hints in
/// `cursor.rs`). Terminals that do not implement it ignore the sequence, so
/// this degrades to a no-op.
pub(crate) fn osc52_copy(text: &str) -> String {
    format!("\x1b]52;c;{}\x07", base64_encode(text.as_bytes()))
}

/// Minimal base64, since OSC 52 requires it and pulling a crate in for
/// one 20-line function isn't worth the dependency.
fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(list: &[&str]) -> Vec<PathBuf> {
        list.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn path_mode_is_one_flat_row_per_file_with_the_full_path() {
        let p = paths(&["crates/tui/src/lib.rs", "README.md"]);
        let rows = build_tree_rows(&p, FileListMode::Path, &HashSet::new(), "");
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0],
            TreeRow::File {
                depth: 0,
                label: "crates/tui/src/lib.rs".into(),
                index: 0,
            }
        );
    }

    #[test]
    fn tree_mode_emits_a_folder_row_per_directory_level() {
        let p = paths(&["crates/tui/src/lib.rs"]);
        let rows = build_tree_rows(&p, FileListMode::Tree, &HashSet::new(), "");
        // crates/ -> tui/ -> src/ -> lib.rs
        assert_eq!(rows.len(), 4);
        match &rows[0] {
            TreeRow::Folder { depth, label, .. } => {
                assert_eq!((*depth, label.as_str()), (0, "crates"));
            }
            other => panic!("expected a folder row, got {other:?}"),
        }
        assert_eq!(
            rows[3],
            TreeRow::File {
                depth: 3,
                label: "lib.rs".into(),
                index: 0,
            }
        );
    }

    #[test]
    fn a_shared_prefix_is_not_repeated_between_siblings() {
        let p = paths(&["crates/a.rs", "crates/b.rs"]);
        let rows = build_tree_rows(&p, FileListMode::Tree, &HashSet::new(), "");
        // one `crates` folder, then both files -- not a folder row each.
        assert_eq!(rows.len(), 3);
        assert!(matches!(rows[0], TreeRow::Folder { .. }));
        assert!(matches!(rows[1], TreeRow::File { .. }));
        assert!(matches!(rows[2], TreeRow::File { .. }));
    }

    /// The bug a naive implementation has: hiding a collapsed folder's
    /// files but still emitting its nested *folder* rows, so collapsing
    /// `crates` still showed `tui`, `src`, ...
    #[test]
    fn a_collapsed_folder_hides_its_nested_folders_too_not_just_its_files() {
        let p = paths(&["crates/tui/src/lib.rs", "crates/core/src/backend.rs"]);
        let mut collapsed = HashSet::new();
        collapsed.insert(PathBuf::from("crates"));
        let rows = build_tree_rows(&p, FileListMode::Tree, &collapsed, "");
        assert_eq!(rows.len(), 1, "only the collapsed `crates` row: {rows:?}");
        match &rows[0] {
            TreeRow::Folder {
                label, collapsed, ..
            } => {
                assert_eq!(label, "crates");
                assert!(collapsed);
            }
            other => panic!("expected the collapsed folder row, got {other:?}"),
        }
    }

    #[test]
    fn collapsing_one_subtree_leaves_its_siblings_visible() {
        let p = paths(&["a/one.rs", "b/two.rs"]);
        let mut collapsed = HashSet::new();
        collapsed.insert(PathBuf::from("a"));
        let rows = build_tree_rows(&p, FileListMode::Tree, &collapsed, "");
        // a (collapsed), b, two.rs
        assert_eq!(rows.len(), 3);
        assert_eq!(rows.iter().filter_map(TreeRow::file_index).count(), 1);
    }

    #[test]
    fn a_file_at_the_repo_root_gets_no_folder_row() {
        let p = paths(&["README.md"]);
        let rows = build_tree_rows(&p, FileListMode::Tree, &HashSet::new(), "");
        assert_eq!(
            rows,
            vec![TreeRow::File {
                depth: 0,
                label: "README.md".into(),
                index: 0,
            }]
        );
    }

    #[test]
    fn the_filter_is_case_insensitive_and_drops_folders_left_empty() {
        let p = paths(&["crates/tui/lib.rs", "docs/guide.md"]);
        let rows = build_tree_rows(&p, FileListMode::Tree, &HashSet::new(), "GUIDE");
        // `crates` must not survive as an empty folder row.
        assert_eq!(rows.len(), 2);
        assert!(matches!(&rows[0], TreeRow::Folder { label, .. } if label == "docs"));
        assert_eq!(rows[1].file_index(), Some(1));
    }

    #[test]
    fn file_index_survives_filtering_so_it_still_points_at_the_source_slice() {
        let p = paths(&["a.rs", "b.rs", "c.rs"]);
        let rows = build_tree_rows(&p, FileListMode::Path, &HashSet::new(), "c");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].file_index(),
            Some(2),
            "index into the ORIGINAL slice"
        );
    }

    #[test]
    fn all_folders_collects_every_intermediate_level() {
        let p = paths(&["crates/tui/src/lib.rs", "README.md"]);
        let folders = all_folders(&p);
        assert!(folders.contains(&PathBuf::from("crates")));
        assert!(folders.contains(&PathBuf::from("crates/tui")));
        assert!(folders.contains(&PathBuf::from("crates/tui/src")));
        assert_eq!(folders.len(), 3, "a root file contributes none");
    }

    #[test]
    fn a_partially_staged_file_shows_up_in_both_sections() {
        let tree = WorkingTree {
            files: vec![StatusEntry {
                path: PathBuf::from("both.rs"),
                index: Some(FileStatus::Staged),
                worktree: Some(FileStatus::Modified),
            }],
        };
        assert_eq!(section_paths(&tree, StatusSection::Staged).len(), 1);
        assert_eq!(section_paths(&tree, StatusSection::Unstaged).len(), 1);
    }

    /// A menu as plain text: one line per row, `---` for a rule, a
    /// submenu's rows indented beneath its `>` line.
    fn describe(entries: &[Entry<FileContextAction>]) -> Vec<String> {
        let mut out = Vec::new();
        for e in entries {
            match e {
                Entry::Action { label, .. } => out.push(label.clone()),
                Entry::Disabled { label } => out.push(format!("({label})")),
                Entry::Separator => out.push("---".into()),
                Entry::Submenu { label, items } => {
                    out.push(format!("{label} >"));
                    out.extend(describe(items).into_iter().map(|l| format!("  {l}")));
                }
            }
        }
        out
    }

    fn menu_for(
        section: StatusSection,
        status: FileStatus,
        path: &str,
        lfs: Vec<FileContextAction>,
    ) -> Vec<String> {
        // The Convert submenu is a tree of its own (`pane_menu` tests it): one
        // row here, not its forty.
        let entries: Vec<Entry<FileContextAction>> = file_context_entries(&FileMenuInput {
            section,
            status: Some(status),
            conflicted: false,
            path: Path::new(path),
            lfs,
            binary: Some(false),
            editor_name: "VS Code".into(),
        })
        .into_iter()
        .map(|e| match e {
            Entry::Submenu { label, .. } if label == "Convert" => Entry::submenu(label, vec![]),
            other => other,
        })
        .collect();
        describe(&entries)
    }

    /// The treat-as row offers the opposite of what the diff does now, in
    /// both lists, for the file and for its extension.
    #[test]
    fn the_treat_row_offers_the_opposite_and_exists_in_both_lists() {
        let rows = |section, binary: Option<bool>, name: &str| {
            let entries = file_context_entries(&FileMenuInput {
                section,
                status: Some(FileStatus::Modified),
                conflicted: false,
                path: Path::new(name),
                lfs: vec![],
                editor_name: "VS Code".into(),
                binary,
            });
            entries
                .into_iter()
                .find_map(|e| match e {
                    Entry::Submenu { label, items } if label.starts_with("Treat as") => Some((
                        label,
                        items
                            .into_iter()
                            .filter_map(|i| match i {
                                Entry::Action { action, .. } => Some(action),
                                _ => None,
                            })
                            .collect::<Vec<_>>(),
                    )),
                    _ => None,
                })
                .expect("a treat-as submenu")
        };
        use FileContextAction as A;
        for section in [StatusSection::Unstaged, StatusSection::Staged] {
            assert_eq!(
                rows(section, Some(false), "a/b.dat"),
                (
                    "Treat as binary".into(),
                    vec![A::TreatFileBinary, A::TreatExtensionBinary]
                )
            );
            assert_eq!(
                rows(section, Some(true), "a/b.dat"),
                (
                    "Treat as text".into(),
                    vec![A::TreatFileText, A::TreatExtensionText]
                )
            );
        }
        // No extension: only the file itself can be named.
        assert_eq!(
            rows(StatusSection::Unstaged, Some(true), "Makefile").1,
            vec![A::TreatFileText]
        );
    }

    #[test]
    fn a_modified_unstaged_file_gets_the_full_menu_in_order() {
        let menu = menu_for(
            StatusSection::Unstaged,
            FileStatus::Modified,
            "src/main.rs",
            vec![
                FileContextAction::LfsTrackFile,
                FileContextAction::LfsTrackExtension,
            ],
        );
        assert_eq!(
            menu,
            [
                "Stage",
                "Discard changes...",
                "Ignore >",
                "  Ignore 'main.rs'",
                "  All files with the extension '.rs'",
                "Treat as binary >",
                "  This file ('main.rs')",
                "  All files with the extension '.rs'",
                "Convert >",
                "LFS >",
                "  Track 'main.rs'",
                "  Track all files with the extension '.rs'",
                "Stash file",
                "---",
                "File History",
                "File Blame",
                "---",
                "Open in external diff tool",
                "Open in VS Code",
                "Open file in default program",
                "Show in folder",
                "---",
                "Copy file path",
                "Create patch from file changes...",
                "---",
                "Edit file",
                "Delete file...",
            ]
        );
    }

    #[test]
    fn an_untracked_file_loses_discard_but_keeps_delete() {
        let menu = menu_for(
            StatusSection::Unstaged,
            FileStatus::Untracked,
            "notes.txt",
            vec![],
        );
        assert!(!menu.contains(&"Discard changes...".to_string()));
        assert!(menu.contains(&"Delete file...".to_string()));
        assert!(menu.contains(&"Stage".to_string()));
        assert!(
            !menu.iter().any(|l| l.starts_with("LFS")),
            "no LFS rows when off"
        );
        assert!(menu.contains(&"(Open in external diff tool (untracked))".to_string()));
    }

    #[test]
    fn the_staged_list_swaps_stage_for_unstage_and_drops_discard_and_ignore() {
        let menu = menu_for(StatusSection::Staged, FileStatus::Staged, "a/b.txt", vec![]);
        assert_eq!(menu[0], "Unstage");
        assert!(!menu.contains(&"Stage".to_string()));
        assert!(!menu.contains(&"Discard changes...".to_string()));
        assert!(!menu.iter().any(|l| l.starts_with("Ignore")));
        assert!(menu.contains(&"Stash file".to_string()));
        assert!(menu.contains(&"Delete file...".to_string()));
    }

    #[test]
    fn a_file_without_an_extension_offers_no_extension_rows_and_a_deleted_one_no_disk_rows() {
        let menu = menu_for(
            StatusSection::Unstaged,
            FileStatus::Modified,
            "Makefile",
            vec![FileContextAction::LfsTrackFile],
        );
        assert!(!menu.iter().any(|l| l.contains("extension")));
        let gone = menu_for(
            StatusSection::Unstaged,
            FileStatus::Deleted,
            "old.txt",
            vec![],
        );
        for hidden in [
            "Show in folder",
            "Edit file",
            "Delete file...",
            "Open in VS Code",
        ] {
            assert!(
                !gone.contains(&hidden.to_string()),
                "{hidden} on a deleted file"
            );
        }
    }

    #[test]
    fn every_context_action_has_a_label_and_only_discard_and_delete_confirm() {
        use FileContextAction as A;
        for a in [
            A::Stage,
            A::Unstage,
            A::Discard,
            A::IgnoreFile,
            A::IgnoreExtension,
            A::StashFile,
            A::FileHistory,
            A::FileBlame,
            A::ExternalDiff,
            A::OpenInEditor,
            A::OpenInDefault,
            A::ShowInFolder,
            A::CopyPath,
            A::CreatePatch,
            A::EditFile,
            A::DeleteFile,
        ] {
            assert!(!a.label().is_empty());
            assert_eq!(a.is_destructive(), matches!(a, A::Discard | A::DeleteFile));
        }
    }

    /// Porcelain v2 reports an unmerged path with BOTH the index and
    /// worktree columns set, so the naive per-column split listed the same
    /// conflicted file in Unstaged *and* Staged - and the Staged copy
    /// then offered "Unstage", an operation with nothing behind it for an
    /// unmerged path.
    #[test]
    fn a_conflicted_file_belongs_to_the_unstaged_half_alone() {
        let tree = WorkingTree {
            files: vec![StatusEntry {
                path: PathBuf::from("clash.rs"),
                index: Some(FileStatus::Conflicted),
                worktree: Some(FileStatus::Conflicted),
            }],
        };
        assert_eq!(
            section_paths(&tree, StatusSection::Unstaged),
            vec![PathBuf::from("clash.rs")]
        );
        assert!(section_paths(&tree, StatusSection::Staged).is_empty());
    }

    /// The menu a conflicted row gets leads with the three resolutions and
    /// deliberately drops plain `Stage`: staging an unmerged path without
    /// picking a side is what commits a file full of `<<<<<<<` markers.
    #[test]
    fn a_conflicted_row_is_offered_the_three_sides_and_not_plain_stage() {
        let menu = describe(&file_context_entries(&FileMenuInput {
            section: StatusSection::Unstaged,
            status: Some(FileStatus::Conflicted),
            conflicted: true,
            path: Path::new("clash.rs"),
            lfs: vec![],
            binary: Some(false),
            editor_name: "VS Code".into(),
        }));
        // The merge editor leads, then the three whole-file resolutions.
        assert_eq!(
            &menu[..5],
            [
                "Open in merge editor",
                "---",
                "Resolve using ours (this branch)",
                "Resolve using theirs (incoming)",
                "Resolve using base (common ancestor)",
            ]
        );
        assert!(!menu.contains(&"Stage".to_string()));
        assert!(!menu.contains(&"Discard changes...".to_string()));
        // ... and an ordinary row offers no resolutions.
        let plain = menu_for(
            StatusSection::Unstaged,
            FileStatus::Modified,
            "a.rs",
            vec![],
        );
        assert!(plain.contains(&"Stage".to_string()));
        assert!(!plain.iter().any(|l| l.starts_with("Resolve")));
    }

    /// Taking a side is not in the confirm-modal category: the other two
    /// stages stay in the index until the merge is committed, so picking
    /// the wrong one is undone by picking the right one.
    #[test]
    fn taking_a_side_of_a_conflict_is_enabled_and_not_destructive() {
        for side in [ConflictSide::Ours, ConflictSide::Theirs, ConflictSide::Base] {
            let action = FileContextAction::ResolveUsing(side);
            assert!(!action.is_destructive(), "{action:?} should not confirm");
            assert!(!action.label().is_empty());
        }
    }

    #[test]
    fn discarding_changes_is_flagged_destructive_so_it_routes_via_the_confirm_modal() {
        assert!(FileContextAction::Discard.is_destructive());
        assert!(!FileContextAction::Stage.is_destructive());
        assert!(!FileContextAction::CopyPath.is_destructive());
    }

    #[test]
    fn base64_matches_known_vectors_including_both_pad_lengths() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn osc52_wraps_the_payload_in_the_clipboard_sequence() {
        assert_eq!(osc52_copy("foo"), "\x1b]52;c;Zm9v\x07");
    }

    fn entry(path: &str, worktree: Option<FileStatus>) -> StatusEntry {
        StatusEntry {
            path: PathBuf::from(path),
            index: None,
            worktree,
        }
    }

    /// Only paths *beneath* the folder, and only the
    /// Unstaged half - a file that is staged-clean (no further worktree
    /// change) is not something Discard on this folder should ever touch.
    #[test]
    fn unstaged_paths_under_a_folder_are_scoped_to_it_and_to_unstaged() {
        let tree = WorkingTree {
            files: vec![
                entry("src/a.rs", Some(FileStatus::Modified)),
                entry("src/nested/b.rs", Some(FileStatus::Untracked)),
                entry("other/c.rs", Some(FileStatus::Modified)),
                entry("src/staged_only.rs", None), // in the index only
            ],
        };
        let mut under_src = unstaged_paths_under(&tree, Path::new("src"));
        under_src.sort();
        assert_eq!(
            under_src,
            vec![PathBuf::from("src/a.rs"), PathBuf::from("src/nested/b.rs"),]
        );
        // The tree's own root ("") is the global "discard everything"
        // case: every Unstaged path, `other/` included.
        let mut everything = unstaged_paths_under(&tree, Path::new(""));
        everything.sort();
        assert_eq!(
            everything,
            vec![
                PathBuf::from("other/c.rs"),
                PathBuf::from("src/a.rs"),
                PathBuf::from("src/nested/b.rs"),
            ]
        );
    }

    /// The exact wording asks for: a plain count, or a
    /// count plus how many of those are untracked.
    #[test]
    fn discard_confirmation_names_the_count_and_the_untracked_share() {
        let tree = WorkingTree {
            files: vec![
                entry("a.rs", Some(FileStatus::Modified)),
                entry("b.rs", Some(FileStatus::Untracked)),
                entry("c.rs", Some(FileStatus::Untracked)),
            ],
        };
        let all: Vec<PathBuf> = tree.files.iter().map(|f| f.path.clone()).collect();
        assert_eq!(
            discard_confirmation_prompt(&tree, &all),
            "Discard 3 files (2 untracked)? This cannot be undone."
        );
        // No untracked paths among the ones asked about: no parenthetical.
        let just_a = vec![PathBuf::from("a.rs")];
        assert_eq!(
            discard_confirmation_prompt(&tree, &just_a),
            "Discard 1 file? This cannot be undone."
        );
    }

    #[test]
    fn the_unstaged_folder_menu_is_in_the_documented_order() {
        let labels: Vec<String> =
            folder_context_entries(StatusSection::Unstaged, Path::new("src/ui"))
                .iter()
                .map(|e| match e {
                    Entry::Action { label, .. } => label.clone(),
                    Entry::Separator => "---".into(),
                    Entry::Submenu { label, items } => {
                        let rows: Vec<String> = items
                            .iter()
                            .map(|i| match i {
                                Entry::Action { label, .. } => label.clone(),
                                _ => "?".into(),
                            })
                            .collect();
                        format!("{label} > {}", rows.join(" | "))
                    }
                    _ => "?".into(),
                })
                .collect();
        assert_eq!(
            labels,
            [
                "Stage folder",
                "Discard all changes in folder...",
                "Ignore all files in 'src/ui/'",
                "Treat files in folder as > Binary | Text",
                "Convert > ? | ? | ?",
                "Stash folder",
                "Create patch from changes in folder...",
                "---",
                "Open folder",
                "Create a file in this folder",
                "---",
                "Expand all subfolders",
                "Collapse all subfolders",
            ]
        );
    }

    #[test]
    fn the_staged_folder_menu_unstages_and_has_no_discard_or_ignore() {
        let entries = folder_context_entries(StatusSection::Staged, Path::new("src"));
        let actions: Vec<FolderContextAction> = entries
            .iter()
            .filter_map(|e| match e {
                Entry::Action { action, .. } => Some(*action),
                _ => None,
            })
            .collect();
        assert_eq!(actions[0], FolderContextAction::Unstage);
        assert!(!actions.contains(&FolderContextAction::Discard));
        assert!(!actions.contains(&FolderContextAction::IgnoreFolder));
        assert!(actions.contains(&FolderContextAction::CreateFile));
    }

    #[test]
    fn the_empty_area_offers_create_file_the_bulk_buttons_and_the_repo_folder() {
        let actions = |section| -> Vec<FolderContextAction> {
            empty_area_entries(section)
                .iter()
                .filter_map(|e| match e {
                    Entry::Action { action, .. } => Some(*action),
                    _ => None,
                })
                .collect()
        };
        assert_eq!(
            actions(StatusSection::Unstaged),
            vec![
                FolderContextAction::CreateFile,
                FolderContextAction::StageAll,
                FolderContextAction::DiscardAll,
                FolderContextAction::OpenRepoFolder,
                FolderContextAction::Fold(crate::tree_fold::TreeFold::ExpandAll),
                FolderContextAction::Fold(crate::tree_fold::TreeFold::CollapseAll),
            ]
        );
        assert_eq!(
            actions(StatusSection::Staged),
            vec![
                FolderContextAction::UnstageAll,
                FolderContextAction::OpenRepoFolder,
                FolderContextAction::Fold(crate::tree_fold::TreeFold::ExpandAll),
                FolderContextAction::Fold(crate::tree_fold::TreeFold::CollapseAll),
            ]
        );
    }
}
