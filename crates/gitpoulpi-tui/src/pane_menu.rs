//! The right pane's file-list menus that act on paths, not on one status
//! row: the commit's changed files and the View all files lists (a file, a
//! folder or the empty area), and any multi-selection (`file_selection`).
//! Plus the shared "Convert" submenu the single-file and folder menus offer
//! too.
//!
//! Pure: this builds rows; `event_loop::input::commands::pane` runs them, so
//! the mouse click, the menu key and the palette cannot drift.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::Oid;
use gitpoulpi_core::textconv::{ConvertOp, Eol};
use gitpoulpi_i18n::msg::pane_menu as pm;
use gitpoulpi_i18n::msg::status_pane as sp;

use crate::context_menu::Entry;
use crate::file_selection::FileList;
use crate::tree_fold::{FoldCtx, TreeFold};

/// What the menu was opened on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaneKind {
    /// One file row.
    File,
    /// One folder row: its files are `paths`.
    Folder,
    /// Outside any row.
    Empty,
    /// Several selected rows.
    Selection,
}

/// Everything an action of the menu acts on, resolved when the menu opened
/// (the lists move on under an open menu, the target does not).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PaneTarget {
    pub(crate) list: FileList,
    pub(crate) kind: PaneKind,
    /// The commit whose files the list shows (`FileList::Commit`).
    pub(crate) commit: Option<Oid>,
    /// The files acted on: the row's file, the folder's files or the
    /// selection, repo-relative.
    pub(crate) paths: Vec<PathBuf>,
    /// The folder row the menu was opened on.
    pub(crate) folder: Option<PathBuf>,
    /// Commit list, a file the commit changed: its index in the changed
    /// files (for the diff). `None`: unchanged (View all files).
    pub(crate) file_index: Option<usize>,
}

impl PaneTarget {
    pub(crate) fn first(&self) -> Option<&Path> {
        self.paths.first().map(PathBuf::as_path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PaneAction {
    OpenDiff,
    /// The file's content (File View): at the commit, or on disk.
    OpenFile,
    FileHistory,
    FileBlame,
    OpenInEditor,
    OpenInDefault,
    /// The built-in editor tab, on the working copy.
    EditFile,
    /// The image editors, on a read-only copy of the commit's version.
    EditImage,
    OpenWith,
    ShowInFolder,
    /// The repo-relative path(s).
    CopyPath,
    CopyFullPath,
    IgnoreFiles,
    IgnoreExtensions,
    IgnoreFolder,
    TreatBinary,
    TreatText,
    TreatExtensionBinary,
    TreatExtensionText,
    TreatFolderBinary,
    TreatFolderText,
    /// Remove from disk, after a confirmation.
    Delete,
    /// The Create patch dialog for these files: the commit's version of them
    /// (commit list), or their changes in the working tree.
    CreatePatch,
    /// The diff of the commit's version against the file on disk.
    DiffWorkingTree,
    /// Overwrite the working copy with the commit's version, after a
    /// confirmation (journaled: Ctrl+Z puts the old bytes back).
    Restore,
    Stage,
    Unstage,
    Discard,
    Convert(ConvertOp),
    OpenFolder,
    CreateFile,
    OpenRepoFolder,
    /// Expand/collapse all folders of the list, or a folder and its
    /// subfolders (`crate::tree_fold`).
    Fold(crate::tree_fold::TreeFold),
}

impl From<TreeFold> for PaneAction {
    fn from(op: TreeFold) -> Self {
        PaneAction::Fold(op)
    }
}

impl PaneAction {
    /// Rows that stop at a confirm modal (painted red).
    pub(crate) fn is_destructive(self) -> bool {
        matches!(
            self,
            PaneAction::Delete | PaneAction::Restore | PaneAction::Discard
        )
    }
}

/// What the entries are built from.
pub(crate) struct PaneMenuInput<'a> {
    pub(crate) target: &'a PaneTarget,
    /// The working copy exists (a file), or the folder does.
    pub(crate) on_disk: bool,
    /// Whether git diffs the single file as binary (`None`: not answered).
    pub(crate) binary: Option<bool>,
    pub(crate) editor_name: String,
}

/// The menu caption: the file (or folder) name, or "N files".
pub(crate) fn pane_caption(target: &PaneTarget) -> String {
    match target.kind {
        PaneKind::Selection => pm::title_files(target.paths.len() as u64),
        PaneKind::Folder => crate::truncate_with_ellipsis(
            &format!(
                "{}/",
                target.folder.as_deref().unwrap_or(Path::new("")).display()
            ),
            40,
        ),
        PaneKind::File => crate::truncate_with_ellipsis(
            &target.first().unwrap_or(Path::new("")).to_string_lossy(),
            40,
        ),
        PaneKind::Empty => gitpoulpi_i18n::msg::status_list::WORKING_CHANGES
            .get()
            .to_string(),
    }
}

fn tool(label: impl Into<String>, action: PaneAction) -> Entry<PaneAction> {
    Entry::Action {
        label: label.into(),
        action,
        destructive: action.is_destructive(),
        key: None,
    }
}

/// A row grayed out with its reason when the working copy is gone.
fn needs_disk(on_disk: bool, label: impl Into<String>, action: PaneAction) -> Entry<PaneAction> {
    if on_disk {
        tool(label, action)
    } else {
        Entry::disabled(label, pm::NOT_IN_TREE.get())
    }
}

/// The tab stops and widths the Indentation rows offer.
const SPACE_SIZES: [usize; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
const TAB_SIZES: [usize; 4] = [2, 3, 4, 8];

/// `Convert >` with Encoding, Line endings and Indentation inside, for any
/// menu whose action type can carry a [`ConvertOp`].
pub(crate) fn convert_submenu<A: Clone>(wrap: impl Fn(ConvertOp) -> A) -> Entry<A> {
    let row = |label: String, op: ConvertOp| Entry::action(label, wrap(op));
    let encodings = gitpoulpi_core::textconv::ALL
        .iter()
        .map(|c| row(c.name().to_string(), ConvertOp::Encoding(*c)))
        .collect();
    let eols = [Eol::Lf, Eol::Crlf, Eol::Cr]
        .into_iter()
        .map(|e| row(e.code().unwrap_or("").to_string(), ConvertOp::Eol(e)))
        .collect();
    let mut indent = Vec::new();
    for size in SPACE_SIZES {
        indent.push(row(
            pm::indent_spaces(size as u64),
            ConvertOp::Indent { tabs: false, size },
        ));
    }
    for size in TAB_SIZES {
        indent.push(row(
            pm::indent_tabs(size as u64),
            ConvertOp::Indent { tabs: true, size },
        ));
    }
    Entry::submenu(
        pm::CONVERT.get(),
        vec![
            Entry::submenu(pm::CONVERT_ENCODING.get(), encodings),
            Entry::submenu(pm::CONVERT_EOL.get(), eols),
            Entry::submenu(pm::CONVERT_INDENT.get(), indent),
        ],
    )
}

fn convert_entry(input: &PaneMenuInput<'_>) -> Entry<PaneAction> {
    let t = input.target;
    // A single file known to be on disk and binary cannot be converted.
    if t.kind == PaneKind::File && !input.on_disk {
        return Entry::disabled(pm::CONVERT.get(), pm::NOT_IN_TREE.get());
    }
    if t.kind == PaneKind::File && input.binary == Some(true) {
        return Entry::disabled(pm::CONVERT.get(), pm::BINARY_FILE.get());
    }
    convert_submenu(PaneAction::Convert)
}

/// The fold ("Expand all", "Collapse all subfolders"...) rows of a list's menus.
pub(crate) fn fold_entries<A>(ops: &[TreeFold], list: FileList) -> Vec<Entry<A>>
where
    A: From<TreeFold>,
{
    crate::tree_fold::entries(ops, fold_ctx(list), A::from)
}

/// The key table a list's fold rows show their keys from.
pub(crate) fn fold_ctx(list: FileList) -> FoldCtx {
    match list {
        FileList::Commit => FoldCtx::LogFiles,
        _ => FoldCtx::Status,
    }
}

/// The menu's rows.
pub(crate) fn pane_entries(input: &PaneMenuInput<'_>) -> Vec<Entry<PaneAction>> {
    use PaneAction as A;
    let t = input.target;
    match t.kind {
        PaneKind::Empty => {
            let mut out = vec![
                tool(sp::EMPTY_CREATE_FILE.get(), A::CreateFile),
                tool(pm::OPEN_REPO.get(), A::OpenRepoFolder),
                Entry::Separator,
            ];
            out.extend(fold_entries(
                &[TreeFold::ExpandAll, TreeFold::CollapseAll],
                t.list,
            ));
            out
        }
        PaneKind::Folder => {
            let folder = t.folder.clone().unwrap_or_default();
            let mut out = vec![
                needs_disk(input.on_disk, sp::FO_OPEN.get(), A::OpenFolder),
                tool(sp::FA_COPY_PATH.get(), A::CopyPath),
                tool(pm::COPY_FULL_PATH.get(), A::CopyFullPath),
                Entry::Separator,
                tool(
                    sp::fo_ignore_named(crate::status_pane::folder_ignore_pattern(&folder)),
                    A::IgnoreFolder,
                ),
                Entry::submenu(
                    sp::FO_TREAT.get(),
                    vec![
                        tool(sp::FO_TREAT_BINARY.get(), A::TreatFolderBinary),
                        tool(sp::FO_TREAT_TEXT.get(), A::TreatFolderText),
                    ],
                ),
                convert_entry(input),
                Entry::Separator,
            ];
            if t.list == FileList::Commit {
                out.push(tool(sp::FO_CREATE_PATCH.get(), A::CreatePatch));
            }
            out.push(needs_disk(
                input.on_disk,
                sp::FO_CREATE_FILE.get(),
                A::CreateFile,
            ));
            out.push(Entry::Separator);
            out.extend(fold_entries(
                &[TreeFold::ExpandBelow, TreeFold::CollapseBelow],
                t.list,
            ));
            out
        }
        PaneKind::File => file_entries(input),
        PaneKind::Selection => selection_entries(input),
    }
}

fn file_entries(input: &PaneMenuInput<'_>) -> Vec<Entry<PaneAction>> {
    use PaneAction as A;
    let t = input.target;
    let path = t.first().unwrap_or(Path::new(""));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = path.extension().map(|e| e.to_string_lossy().into_owned());
    let commit = t.list == FileList::Commit;
    let on_disk = input.on_disk;
    let mut out = Vec::new();
    if commit {
        out.push(tool(
            gitpoulpi_i18n::msg::commit_files::OPEN_DIFF.get(),
            A::OpenDiff,
        ));
        if t.file_index.is_some() {
            out.push(tool(pm::OPEN_FILE.get(), A::OpenFile));
        }
        out.push(needs_disk(
            on_disk,
            pm::DIFF_WORKING_TREE.get(),
            A::DiffWorkingTree,
        ));
    } else {
        out.push(tool(pm::OPEN_FILE.get(), A::OpenFile));
    }
    out.push(tool(sp::FA_FILE_HISTORY.get(), A::FileHistory));
    out.push(tool(sp::FA_FILE_BLAME.get(), A::FileBlame));
    out.push(Entry::Separator);
    out.push(needs_disk(
        on_disk,
        sp::fm_open_in(&input.editor_name),
        A::OpenInEditor,
    ));
    out.push(needs_disk(
        on_disk,
        sp::FA_OPEN_IN_DEFAULT.get(),
        A::OpenInDefault,
    ));
    // The apps for the file's type, on a read-only copy of the version the
    // list shows (the commit's, or the working copy's).
    if crate::image_view::edit_in::is_image_path(path) {
        out.push(tool(
            gitpoulpi_i18n::msg::image_edit_in::MENU.get(),
            A::EditImage,
        ));
    } else if crate::open_with::menu_offered(&path.to_string_lossy()) {
        out.push(tool(
            gitpoulpi_i18n::msg::open_with::MENU.get(),
            A::OpenWith,
        ));
    }
    out.push(needs_disk(
        on_disk,
        sp::FA_SHOW_IN_FOLDER.get(),
        A::ShowInFolder,
    ));
    out.push(needs_disk(on_disk, sp::FA_EDIT_FILE.get(), A::EditFile));
    out.push(Entry::Separator);
    out.push(tool(sp::FA_COPY_PATH.get(), A::CopyPath));
    out.push(tool(pm::COPY_FULL_PATH.get(), A::CopyFullPath));
    if (commit && t.file_index.is_some()) || t.list == FileList::WipAll {
        out.push(tool(sp::FA_CREATE_PATCH.get(), A::CreatePatch));
    }
    out.push(Entry::Separator);
    let mut ignore = vec![tool(sp::fm_ignore_name(&name), A::IgnoreFiles)];
    if let Some(ext) = &ext {
        ignore.push(tool(sp::fm_ignore_ext(ext), A::IgnoreExtensions));
    }
    out.push(Entry::submenu(sp::FM_IGNORE.get(), ignore));
    out.push(match input.binary {
        Some(binary) => treat_submenu(binary, &name, ext.as_deref()),
        None => Entry::Disabled {
            label: sp::FM_TREAT_PENDING.get().into(),
        },
    });
    out.push(convert_entry(input));
    out.push(Entry::Separator);
    if commit {
        out.push(tool(pm::RESTORE.get(), A::Restore));
    }
    out.push(needs_disk(on_disk, sp::FA_DELETE_FILE.get(), A::Delete));
    out
}

/// "Treat as Binary / Text" for one file, as the status list's file menu.
fn treat_submenu(binary: bool, name: &str, ext: Option<&str>) -> Entry<PaneAction> {
    use PaneAction as A;
    let (file, by_ext, title) = if binary {
        (A::TreatText, A::TreatExtensionText, sp::FM_TREAT_TEXT.get())
    } else {
        (
            A::TreatBinary,
            A::TreatExtensionBinary,
            sp::FM_TREAT_BINARY.get(),
        )
    };
    let mut items = vec![tool(sp::fm_treat_name(name), file)];
    if let Some(ext) = ext {
        items.push(tool(sp::fm_ignore_ext(ext), by_ext));
    }
    Entry::submenu(title, items)
}

fn selection_entries(input: &PaneMenuInput<'_>) -> Vec<Entry<PaneAction>> {
    use PaneAction as A;
    let t = input.target;
    let mut out = Vec::new();
    match t.list {
        FileList::Unstaged => {
            out.push(tool(sp::FA_STAGE.get(), A::Stage));
            out.push(tool(sp::FA_DISCARD.get(), A::Discard));
        }
        FileList::Staged => out.push(tool(sp::FA_UNSTAGE.get(), A::Unstage)),
        FileList::Commit | FileList::WipAll => {}
    }
    if t.list != FileList::Staged {
        out.push(Entry::submenu(
            sp::FM_IGNORE.get(),
            vec![
                tool(pm::IGNORE_THESE.get(), A::IgnoreFiles),
                tool(pm::IGNORE_EXTENSIONS.get(), A::IgnoreExtensions),
            ],
        ));
    }
    out.push(tool(sp::FM_TREAT_BINARY.get(), A::TreatBinary));
    out.push(tool(sp::FM_TREAT_TEXT.get(), A::TreatText));
    out.push(convert_entry(input));
    out.push(Entry::Separator);
    out.push(tool(pm::COPY_PATHS.get(), A::CopyPath));
    out.push(tool(pm::COPY_FULL_PATHS.get(), A::CopyFullPath));
    if t.list == FileList::Commit {
        out.push(tool(sp::FA_CREATE_PATCH.get(), A::CreatePatch));
    }
    if t.list != FileList::Staged {
        out.push(Entry::Separator);
        if t.list == FileList::Commit {
            out.push(tool(pm::RESTORE_MANY.get(), A::Restore));
        }
        out.push(tool(pm::DELETE_FILES.get(), A::Delete));
    }
    out
}

/// The empty-area menu of the Unstaged / Staged lists (the working
/// changes): the bulk buttons' twins and the folder-level rows.
pub(crate) fn status_empty_entries(
    list: FileList,
) -> Vec<Entry<crate::status_pane::FolderContextAction>> {
    use crate::status_pane::FolderContextAction as F;
    let mut out = Vec::new();
    match list {
        FileList::Staged => out.push(Entry::action(sp::UNSTAGE_ALL.get(), F::UnstageAll)),
        _ => {
            out.push(Entry::action(sp::EMPTY_CREATE_FILE.get(), F::CreateFile));
            out.push(Entry::action(sp::STAGE_ALL.get(), F::StageAll));
            out.push(Entry::danger(
                gitpoulpi_i18n::msg::menu_log::DISCARD_ALL.get(),
                F::DiscardAll,
            ));
        }
    }
    out.push(Entry::Separator);
    out.push(Entry::action(pm::OPEN_REPO.get(), F::OpenRepoFolder));
    out.push(Entry::Separator);
    out.extend(crate::tree_fold::entries(
        &[TreeFold::ExpandAll, TreeFold::CollapseAll],
        FoldCtx::Status,
        F::Fold,
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(kind: PaneKind, list: FileList, paths: &[&str]) -> PaneTarget {
        PaneTarget {
            list,
            kind,
            commit: None,
            paths: paths.iter().map(PathBuf::from).collect(),
            folder: (kind == PaneKind::Folder).then(|| PathBuf::from("src")),
            file_index: Some(0),
        }
    }

    fn input(t: &PaneTarget, on_disk: bool) -> PaneMenuInput<'_> {
        PaneMenuInput {
            target: t,
            on_disk,
            binary: Some(false),
            editor_name: "VS Code".into(),
        }
    }

    fn actions(entries: &[Entry<PaneAction>]) -> Vec<PaneAction> {
        fn walk(entries: &[Entry<PaneAction>], out: &mut Vec<PaneAction>) {
            for e in entries {
                match e {
                    Entry::Action { action, .. } => out.push(*action),
                    Entry::Submenu { items, .. } => walk(items, out),
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        walk(entries, &mut out);
        out
    }

    #[test]
    fn a_pane_menu_has_the_documented_set() {
        let t = target(PaneKind::File, FileList::Commit, &["src/a.rs"]);
        let found = actions(&pane_entries(&input(&t, true)));
        for want in [
            PaneAction::OpenDiff,
            PaneAction::OpenFile,
            PaneAction::FileHistory,
            PaneAction::FileBlame,
            PaneAction::OpenInEditor,
            PaneAction::OpenInDefault,
            PaneAction::EditFile,
            PaneAction::ShowInFolder,
            PaneAction::CopyPath,
            PaneAction::CopyFullPath,
            PaneAction::IgnoreFiles,
            PaneAction::TreatBinary,
            PaneAction::Restore,
            PaneAction::Delete,
            PaneAction::DiffWorkingTree,
            PaneAction::CreatePatch,
        ] {
            assert!(found.contains(&want), "{want:?} missing from {found:?}");
        }
        assert!(found.contains(&PaneAction::Convert(ConvertOp::Eol(Eol::Lf))));
    }

    #[test]
    fn a_file_gone_from_the_working_tree_grays_its_working_copy_rows() {
        let t = target(PaneKind::File, FileList::Commit, &["src/a.rs"]);
        let found = actions(&pane_entries(&input(&t, false)));
        for gone in [
            PaneAction::OpenInEditor,
            PaneAction::EditFile,
            PaneAction::ShowInFolder,
            PaneAction::Delete,
            PaneAction::DiffWorkingTree,
            PaneAction::Convert(ConvertOp::Eol(Eol::Lf)),
        ] {
            assert!(!found.contains(&gone), "{gone:?} must be grayed");
        }
        // The commit's own version stays reachable.
        assert!(found.contains(&PaneAction::OpenDiff) && found.contains(&PaneAction::Restore));
        assert!(found.contains(&PaneAction::CreatePatch));
    }

    #[test]
    fn create_patch_is_offered_for_changed_commit_files_and_commit_selections_only() {
        let mut t = target(PaneKind::File, FileList::Commit, &["src/a.rs"]);
        t.file_index = None;
        let found = actions(&pane_entries(&input(&t, true)));
        assert!(!found.contains(&PaneAction::CreatePatch), "unchanged file");
        let t = target(PaneKind::Selection, FileList::Commit, &["a", "b"]);
        let found = actions(&pane_entries(&input(&t, true)));
        assert!(found.contains(&PaneAction::CreatePatch));
        let t = target(PaneKind::Selection, FileList::Staged, &["a", "b"]);
        let found = actions(&pane_entries(&input(&t, true)));
        assert!(!found.contains(&PaneAction::CreatePatch));
    }

    #[test]
    fn the_selection_menu_follows_the_list() {
        let t = target(PaneKind::Selection, FileList::Unstaged, &["a", "b"]);
        let found = actions(&pane_entries(&input(&t, true)));
        assert!(found.contains(&PaneAction::Stage) && found.contains(&PaneAction::Discard));
        assert!(!found.contains(&PaneAction::Unstage));
        let t = target(PaneKind::Selection, FileList::Staged, &["a", "b"]);
        let found = actions(&pane_entries(&input(&t, true)));
        assert!(found.contains(&PaneAction::Unstage));
        assert!(!found.contains(&PaneAction::Delete) && !found.contains(&PaneAction::IgnoreFiles));
        assert_eq!(pane_caption(&t), "2 files");
    }

    #[test]
    fn the_empty_area_and_folder_menus() {
        let t = target(PaneKind::Empty, FileList::Commit, &[]);
        let found = actions(&pane_entries(&input(&t, true)));
        assert_eq!(
            found,
            vec![
                PaneAction::CreateFile,
                PaneAction::OpenRepoFolder,
                PaneAction::Fold(TreeFold::ExpandAll),
                PaneAction::Fold(TreeFold::CollapseAll),
            ]
        );
        let t = target(PaneKind::Folder, FileList::Commit, &["src/a", "src/b"]);
        let found = actions(&pane_entries(&input(&t, true)));
        assert!(found.contains(&PaneAction::IgnoreFolder));
        assert!(found.contains(&PaneAction::TreatFolderBinary));
        assert!(found.contains(&PaneAction::CreateFile));
        assert!(found.contains(&PaneAction::OpenFolder));
        // A folder folds below itself, the empty area folds the whole list.
        assert!(found.contains(&PaneAction::Fold(TreeFold::ExpandBelow)));
        assert!(found.contains(&PaneAction::Fold(TreeFold::CollapseBelow)));
        assert!(!found.contains(&PaneAction::Fold(TreeFold::ExpandAll)));
    }

    #[test]
    fn convert_offers_every_encoding_eol_and_indent_size() {
        let Entry::Submenu { items, .. } = convert_submenu(|op| op) else {
            panic!("a submenu");
        };
        let count = |i: usize| match &items[i] {
            Entry::Submenu { items, .. } => items.len(),
            _ => 0,
        };
        assert_eq!(count(0), gitpoulpi_core::textconv::ALL.len());
        assert_eq!(count(1), 3);
        assert_eq!(count(2), SPACE_SIZES.len() + TAB_SIZES.len());
    }
}
