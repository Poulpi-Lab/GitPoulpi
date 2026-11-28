//! The Status pane sections and file menu actions.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

use gitpoulpi_core::domain::ConflictSide;

/// Which half of the working-changes pane a file row belongs to. A
/// partially-staged file legitimately appears in both, which is why this
/// is carried per-row rather than derived from the entry alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StatusSection {
    Unstaged,
    Staged,
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
