//! The rows of the menus the worktree, submodule, remote, LFS and file commands open. They are the ref menu's
//! [`Item`]s (one view, one keyboard behaviour), each carrying a [`RepoOp`]: the terminal UI's rows in its
//! order, with the ones the desktop cannot run (the daemon has no such operation) present and dimmed with
//! the reason.

use gitpoulpi_config::OpenInEntry;
use gitpoulpi_core::domain::{FileStatus, LfsRepoStatus, Submodule, SubmoduleState, Worktree};
use gitpoulpi_i18n::msg;

use crate::commands::file_ops::FileOp;
use crate::commands::refs_ops::RefOp;
use crate::commands::repo_ops::RepoOp;
use crate::daemon::repo_ops as rules;
use crate::ref_menu::{glyph, Item};
use crate::ref_target::FileSide;
use crate::strings;

fn row(g: &str, label: impl Into<String>, op: RepoOp) -> Item {
    Item::act(glyph(g, label.into()), RefOp::Repo(op))
}

fn danger(g: &str, label: impl Into<String>, op: RepoOp) -> Item {
    Item::danger(glyph(g, label.into()), RefOp::Repo(op))
}

fn later(g: &str, label: impl Into<String>) -> Item {
    Item::later(glyph(g, label.into()))
}

fn file_row(g: &str, label: impl Into<String>, op: FileOp) -> Item {
    row(g, label, RepoOp::File(op))
}

fn sub(label: impl Into<String>, items: Vec<Item>) -> Item {
    Item::Submenu {
        label: label.into(),
        items,
    }
}

/// The WORKTREES header's `+` menu.
pub fn worktree_plus() -> Vec<Item> {
    use msg::worktree_manage as m;
    vec![
        row("\u{229E}", m::PLUS_HEAD.get(), RepoOp::WorktreeFromHead),
        row("\u{229E}", m::PLUS_COMMIT.get(), RepoOp::WorktreeAtCommit),
        row("\u{229E}", m::PLUS_CHOOSE.get(), RepoOp::WorktreeChoose),
    ]
}

/// The "Open in" entries of a worktree.
pub fn open_in_rows(entries: &[OpenInEntry]) -> Vec<Item> {
    entries
        .iter()
        .take(u8::MAX as usize)
        .enumerate()
        .map(|(i, e)| row("\u{25B8}", e.label.clone(), RepoOp::WorktreeOpenIn(i as u8)))
        .collect()
}

/// The menu of a worktree row. The main worktree is the repository itself: nothing to open or remove.
pub fn worktree_items(w: &Worktree, entries: &[OpenInEntry]) -> Vec<Item> {
    use msg::worktree_manage as m;
    let mut rows = Vec::new();
    if !w.is_main {
        rows.push(row(
            "\u{2197}",
            m::MENU_OPEN_TAB.get(),
            RepoOp::WorktreeOpen,
        ));
    }
    rows.push(if entries.is_empty() {
        Item::off(
            glyph("\u{25B8}", strings::OPEN_IN_NONE.to_string()),
            m::MENU_OPEN_IN_NONE_WHY.get().to_string(),
        )
    } else {
        sub(strings::OPEN_IN, open_in_rows(entries))
    });
    if let Some(branch) = &w.branch {
        // The terminal UI merges the branch into another worktree's branch; the daemon's
        // worktree merge has no desktop form yet.
        rows.push(Item::later(glyph("\u{21C6}", m::menu_merge(branch))));
    }
    rows.push(Item::Separator);
    rows.push(if w.is_main {
        Item::off(
            glyph("\u{2717}", m::MENU_REMOVE.get().to_string()),
            m::MAIN_CANNOT_BE_REMOVED.get().to_string(),
        )
    } else {
        danger("\u{2717}", m::MENU_REMOVE.get(), RepoOp::WorktreeRemove)
    });
    rows
}

/// The menu of a submodule row: what the state allows.
pub fn submodule_items(s: &Submodule) -> Vec<Item> {
    use msg::menu_ref as m;
    let cloned = s.state != SubmoduleState::Uninitialized;
    let mut rows = vec![row(
        "\u{27F3}",
        m::SUBMODULE_UPDATE.get(),
        RepoOp::SubmoduleUpdate,
    )];
    rows.push(row(
        "\u{25CB}",
        m::SUBMODULE_INIT.get(),
        RepoOp::SubmoduleInit,
    ));
    rows.push(if cloned {
        row("\u{2197}", m::SUBMODULE_OPEN.get(), RepoOp::SubmoduleOpen)
    } else {
        Item::off(
            glyph("\u{2197}", m::SUBMODULE_OPEN.get().to_string()),
            msg::submodules_ui::not_cloned(&s.path),
        )
    });
    rows
}

/// The menu of a remote (its name in the REMOTE section).
pub fn remote_items(name: &str) -> Vec<Item> {
    use msg::remotes as m;
    vec![
        row("\u{27F2}", m::fetch_remote(name), RepoOp::RemoteFetch),
        Item::Separator,
        row("\u{2398}", m::COPY_NAME.get(), RepoOp::RemoteCopyName),
        later("\u{2398}", m::COPY_LINK.get()),
        later("\u{2713}", m::SET_DEFAULT.get()),
        later("\u{25C9}", m::HIDE_REMOTE_IN_GRAPH.get()),
        Item::Separator,
        row("\u{270E}", m::edit_remote(name), RepoOp::RemoteEdit),
        danger("\u{2717}", m::remove_remote(name), RepoOp::RemoteRemove),
    ]
}

/// The LFS button's menu. `usable` is `Some(reason)` while git-lfs cannot run.
pub fn lfs_items(status: Option<&LfsRepoStatus>, usable: Option<String>) -> Vec<Item> {
    use msg::lfs_ui as m;
    if let Some(why) = usable {
        return vec![
            Item::off(
                glyph("\u{21E3}", m::MENU_FETCH_ALL.get().to_string()),
                why.clone(),
            ),
            row(
                "\u{2197}",
                msg::palette::LFS_OPEN_INSTALL_PAGE.get(),
                RepoOp::LfsInstallPage,
            ),
        ];
    }
    let mut rows = Vec::new();
    if status.is_some_and(|s| !s.filters_configured || !s.hooks_installed) {
        rows.push(row(
            "\u{2699}",
            msg::palette::LFS_INITIALIZE_FOR_THIS_REPOSITORY.get(),
            RepoOp::LfsInit,
        ));
        rows.push(Item::Separator);
    }
    rows.push(row(
        "\u{21E3}",
        m::MENU_FETCH_ALL.get(),
        RepoOp::LfsFetchAll,
    ));
    rows.push(row(
        "\u{2713}",
        m::MENU_CHECKOUT_ALL.get(),
        RepoOp::LfsCheckout,
    ));
    rows.push(row("\u{21E9}", m::MENU_PULL_ALL.get(), RepoOp::LfsPull));
    rows.push(row("\u{21E7}", m::MENU_PUSH_ALL.get(), RepoOp::LfsPushAll));
    rows.push(row("\u{2702}", m::MENU_PRUNE.get(), RepoOp::LfsPrune));
    rows.push(Item::Separator);
    rows.push(row(
        "+",
        msg::palette::LFS_TRACK_PATTERN.get(),
        RepoOp::LfsTrackPattern,
    ));
    rows.push(row(
        "-",
        msg::palette::LFS_UNTRACK_PATTERN.get(),
        RepoOp::LfsUntrackPattern,
    ));
    rows.push(row(
        "\u{1F512}",
        msg::palette::LFS_SHOW_LOCKS.get(),
        RepoOp::LfsShowLocks,
    ));
    rows
}

/// What a file row's menu is built from.
pub struct FileMenuInput<'a> {
    pub path: &'a str,
    pub side: FileSide,
    /// The row's own status (the index status in Staged, the working tree's in Unstaged).
    pub status: Option<FileStatus>,
    /// The LFS rows the file gets (none when LFS is off or unusable).
    pub lfs: Vec<RepoOp>,
    /// "code", "notepad++", ... for the `Open in <editor>` row; `None` words it without a name.
    pub editor_name: Option<String>,
}

/// The menu of a file, in the terminal UI's order. A working-tree file has Stage or Unstage, Discard
/// (tracked, unstaged), Ignore, Treat as, LFS and Stash; a commit's file has the viewing rows only.
pub fn file_items(input: &FileMenuInput<'_>) -> Vec<Item> {
    use gitpoulpi_app::palette::vocab::status_pane::FileContextAction as A;
    use msg::status_pane as m;
    let path = input.path;
    let name = rules::name_of(path);
    let ext = rules::ext_of(path);
    let working = input.side != FileSide::Commit;
    let staged = input.side == FileSide::Staged;
    let untracked = matches!(input.status, Some(FileStatus::Untracked));
    let on_disk = !matches!(input.status, Some(FileStatus::Deleted));
    let mut rows = Vec::new();
    if working {
        rows.push(if staged {
            file_row("\u{2212}", A::Unstage.label(), FileOp::Unstage)
        } else {
            file_row("+", A::Stage.label(), FileOp::Stage)
        });
        if !staged && !untracked {
            rows.push(danger(
                "\u{2717}",
                A::Discard.label(),
                RepoOp::File(FileOp::Discard),
            ));
        }
        if !staged {
            let mut ignore = vec![file_row(
                "\u{2298}",
                m::fm_ignore_name(&name),
                FileOp::IgnoreFile,
            )];
            if let Some(ext) = &ext {
                ignore.push(file_row(
                    "\u{2298}",
                    m::fm_ignore_ext(ext),
                    FileOp::IgnoreExtension,
                ));
            }
            if let Some(folder) = rules::folder_of(path) {
                ignore.push(file_row(
                    "\u{2298}",
                    m::fo_ignore_named(&folder),
                    FileOp::IgnoreFolder,
                ));
            }
            rows.push(sub(m::FM_IGNORE.get(), ignore));
        }
        for (title, file, by_ext) in [
            (
                m::FM_TREAT_BINARY.get(),
                FileOp::TreatFileBinary,
                FileOp::TreatExtensionBinary,
            ),
            (
                m::FM_TREAT_TEXT.get(),
                FileOp::TreatFileText,
                FileOp::TreatExtensionText,
            ),
        ] {
            let mut items = vec![file_row("\u{25A4}", m::fm_treat_name(&name), file)];
            if let Some(ext) = &ext {
                items.push(file_row("\u{25A4}", m::fm_ignore_ext(ext), by_ext));
            }
            rows.push(sub(title, items));
        }
        if !input.lfs.is_empty() {
            let items = input
                .lfs
                .iter()
                .map(|&op| {
                    let label = match op {
                        RepoOp::LfsTrackFile => m::fm_track_name(&name),
                        RepoOp::LfsTrackExtension => m::fm_track_ext(ext.as_deref().unwrap_or("")),
                        RepoOp::LfsLock => A::LfsLock.label().to_string(),
                        RepoOp::LfsUnlock => A::LfsUnlock.label().to_string(),
                        _ => A::LfsForceUnlock.label().to_string(),
                    };
                    row("\u{25A3}", label, op)
                })
                .collect();
            rows.push(sub("LFS", items));
        }
        rows.push(file_row(
            "\u{2398}",
            A::StashFile.label(),
            FileOp::StashFile,
        ));
        rows.push(Item::Separator);
    }
    if !untracked {
        rows.push(file_row(
            "\u{29D6}",
            A::FileHistory.label(),
            FileOp::History,
        ));
        rows.push(later("\u{2630}", A::FileBlame.label()));
        rows.push(Item::Separator);
    }
    if on_disk {
        rows.push(file_row(
            "\u{270E}",
            match &input.editor_name {
                Some(name) => m::fm_open_in(name),
                None => A::OpenInEditor.label().to_string(),
            },
            FileOp::OpenInEditor,
        ));
        rows.push(file_row(
            "\u{2197}",
            A::OpenInDefault.label(),
            FileOp::OpenInDefault,
        ));
        rows.push(file_row(
            "\u{1F4C1}",
            A::ShowInFolder.label(),
            FileOp::ShowInFolder,
        ));
        rows.push(Item::Separator);
    }
    rows.push(file_row("\u{2398}", A::CopyPath.label(), FileOp::CopyPath));
    rows
}
