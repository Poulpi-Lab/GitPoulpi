//! The Status pane sections and file menu actions.
//!
//! Pure vocabulary: it names no terminal type, so a frontend other than the
//! terminal can share it. The module that owns the feature re-exports it.

use gitpoulpi_i18n::msg::status_pane as msg;

use gitpoulpi_core::domain::ConflictSide;

/// Which half of the working-changes pane a file row belongs to. A
/// partially-staged file legitimately appears in both, which is why this
/// is carried per-row rather than derived from the entry alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusSection {
    Unstaged,
    Staged,
}

/// One entry of a file's right-click menu (and its palette twin).
///
/// Every variant runs: `apply_file_context_action` (and
/// `apply_lfs_file_action` for the LFS rows) is the single place each
/// is carried out, for the mouse, the keyboard menu and the palette alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileContextAction {
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

impl StatusSection {
    /// Display order, top to bottom. Unstaged first: it's the half the
    /// user is usually acting on.
    pub const ORDER: [StatusSection; 2] = [StatusSection::Unstaged, StatusSection::Staged];

    pub fn title(self) -> &'static str {
        match self {
            StatusSection::Unstaged => msg::UNSTAGED_TITLE.get(),
            StatusSection::Staged => msg::STAGED_TITLE.get(),
        }
    }

    /// Label of the button in this section's header: the bulk action that
    /// empties it.
    pub fn bulk_label(self) -> &'static str {
        match self {
            StatusSection::Unstaged => msg::STAGE_ALL.get(),
            StatusSection::Staged => msg::UNSTAGE_ALL.get(),
        }
    }

    /// Label of the per-row button revealed on hover.
    pub fn row_label(self) -> &'static str {
        match self {
            StatusSection::Unstaged => msg::STAGE_FILE.get(),
            StatusSection::Staged => msg::UNSTAGE_FILE.get(),
        }
    }
}

impl FileContextAction {
    /// The row's label for actions whose text does not depend on the
    /// file. `file_context_entries` words the name-bearing ones itself.
    pub fn label(self) -> &'static str {
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
    pub fn is_destructive(self) -> bool {
        matches!(
            self,
            FileContextAction::Discard | FileContextAction::DeleteFile
        )
    }
}
