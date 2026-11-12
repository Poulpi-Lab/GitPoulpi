use super::*;
use crate::clipboard_route::Delivery;
use gitpoulpi_core::domain::{rule_pattern, RuleKind, RuleScope, Treat};
use gitpoulpi_i18n::msg::status_list as msg;

/// Builds the drill-in menu for a file row: the entries
/// `file_context_entries` lays out, captioned with the path and anchored
/// where the click (or the row, for the keyboard) was.
#[allow(clippy::too_many_arguments)]
pub(crate) fn open_file_menu(
    section: StatusSection,
    path: &std::path::Path,
    status_result: &gitpoulpi_core::Result<WorkingTree>,
    conflicts: &[gitpoulpi_core::domain::ConflictEntry],
    lfs: &crate::lfs_ui::LfsUi,
    repo_root: &std::path::Path,
    anchor_col: u16,
    anchor_row: u16,
) -> ContextMenu<FileContextAction> {
    let status = status_result
        .as_ref()
        .ok()
        .and_then(|tree| tree.files.iter().find(|f| f.path == path))
        .and_then(|entry| entry_status(entry, section));
    let entries = file_context_entries(&crate::status_pane::FileMenuInput {
        section,
        status,
        conflicted: conflicts.iter().any(|c| c.path == path),
        path,
        lfs: lfs.file_menu_actions(path),
        editor_name: crate::external_tools::editor_name(&crate::external_tools::current().editor),
        binary: crate::binary_hint::git_answer(repo_root, path),
    });
    ContextMenu::new(
        truncate_with_ellipsis(&path.to_string_lossy(), 40),
        entries,
        anchor_col,
        anchor_row,
    )
}

/// Rebuilds the rows of an open file menu in place (cursor kept): git's
/// binary answer landed after the menu opened.
#[allow(clippy::too_many_arguments)]
pub(crate) fn refresh_file_menu(
    menu: &mut ContextMenu<FileContextAction>,
    section: StatusSection,
    path: &std::path::Path,
    status_result: &gitpoulpi_core::Result<WorkingTree>,
    conflicts: &[gitpoulpi_core::domain::ConflictEntry],
    lfs: &crate::lfs_ui::LfsUi,
    repo_root: &std::path::Path,
) {
    let fresh = open_file_menu(
        section,
        path,
        status_result,
        conflicts,
        lfs,
        repo_root,
        menu.anchor_col,
        menu.anchor_row,
    );
    menu.replace_entries(fresh.into_entries());
}

/// Opens the "which file?" picker for a rule about `path` (a file or, with
/// `RuleScope::Folder`, a folder). Says so on the status line instead when
/// the path cannot be named (an extension rule on a file without one).
pub(crate) fn open_rule_file_picker(
    kind: RuleKind,
    path: &std::path::Path,
    scope: RuleScope,
    treat: Option<Treat>,
    repo_root: &std::path::Path,
    input_mode: &mut InputMode,
    last_message: &mut Option<String>,
) {
    open_rule_file_picker_many(
        kind,
        &[path.to_path_buf()],
        scope,
        treat,
        repo_root,
        input_mode,
        last_message,
    );
}

/// [`open_rule_file_picker`] for a selection: every path with a nameable
/// pattern goes into the one job (the picker's candidate files are those of
/// the first), the others are dropped. Says so instead when none can be named.
pub(crate) fn open_rule_file_picker_many(
    kind: RuleKind,
    paths: &[std::path::PathBuf],
    scope: RuleScope,
    treat: Option<Treat>,
    repo_root: &std::path::Path,
    input_mode: &mut InputMode,
    last_message: &mut Option<String>,
) {
    let paths: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .filter(|p| rule_pattern(kind, kind.file_name(), p, scope).is_some())
        .collect();
    if paths.is_empty() {
        *last_message = Some(msg::NO_EXTENSION.get().into());
        return;
    }
    let job = crate::language_picker::RuleFileJob {
        kind,
        paths,
        scope,
        treat,
    };
    *input_mode = InputMode::LanguagePicker(Box::new(
        crate::language_picker::LanguagePicker::open_rule_file(job, repo_root),
    ));
}

/// Runs one entry of a file's context menu.
///
/// Disabled entries report what they'd need instead of doing nothing;
/// destructive ones are routed into the confirm modal rather than run
/// directly, which is why this takes `input_mode`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_file_context_action(
    action: FileContextAction,
    section: StatusSection,
    repo_root: &std::path::Path,
    path: &std::path::Path,
    link: &DaemonLink,
    refs: &[gitpoulpi_core::domain::Ref],
    input_mode: &mut InputMode,
    file_insight: &mut Option<FileInsight>,
    last_message: &mut Option<String>,
    patch_seed: &mut Option<crate::repo_dialogs::PatchSeed>,
) {
    let tools = crate::external_tools::current();
    let absolute = repo_root.join(path);
    let report = |last_message: &mut Option<String>, ok: String, result: Result<(), String>| {
        *last_message = Some(match result {
            Ok(()) => ok,
            Err(why) => crate::activity_log::report_problem_clear(
                crate::activity_log::ProblemSource::Tool,
                &why,
            ),
        });
    };
    match action {
        // Opened by the caller (`apply_file_menu_action`): it needs the
        // editor's state, which this plain function does not have.
        FileContextAction::OpenMergeEditor => {}
        FileContextAction::StashFile => {
            link.submit_action(Action::StashFile(path.to_path_buf()));
            *last_message = Some(msg::stashing(path.display()));
        }
        // Every rule row asks which file it goes into first.
        FileContextAction::IgnoreFile | FileContextAction::IgnoreExtension => {
            let scope = if action == FileContextAction::IgnoreExtension {
                RuleScope::Extension
            } else {
                RuleScope::File
            };
            open_rule_file_picker(
                RuleKind::Ignore,
                path,
                scope,
                None,
                repo_root,
                input_mode,
                last_message,
            );
        }
        FileContextAction::TreatFileBinary
        | FileContextAction::TreatFileText
        | FileContextAction::TreatExtensionBinary
        | FileContextAction::TreatExtensionText => {
            let (scope, treat) = match action {
                FileContextAction::TreatFileBinary => (RuleScope::File, Treat::Binary),
                FileContextAction::TreatFileText => (RuleScope::File, Treat::Text),
                FileContextAction::TreatExtensionBinary => (RuleScope::Extension, Treat::Binary),
                _ => (RuleScope::Extension, Treat::Text),
            };
            open_rule_file_picker(
                RuleKind::Attributes,
                path,
                scope,
                Some(treat),
                repo_root,
                input_mode,
                last_message,
            );
        }
        FileContextAction::DeleteFile => {
            *input_mode = InputMode::Confirm {
                prompt: msg::delete_prompt(path.display()),
                action: ConfirmAction::Destructive(PendingDestructive::DeleteFile(
                    path.to_path_buf(),
                )),
            };
        }
        FileContextAction::ExternalDiff => {
            crate::external_tools::open_external_diff(
                repo_root.to_path_buf(),
                path.to_path_buf(),
                section == StatusSection::Staged,
                tools.diff_tool.clone(),
            );
            *last_message = Some(msg::opening_diff_tool(path.display()));
        }
        FileContextAction::OpenInEditor => report(
            last_message,
            msg::opened_editor(path.display()),
            crate::external_tools::open_in_editor(&tools.editor, &absolute),
        ),
        FileContextAction::OpenInDefault => match crate::open_with::pointer_notice(&absolute) {
            Some(notice) => *last_message = Some(notice),
            None => report(
                last_message,
                msg::opened_default(path.display()),
                crate::external_tools::open_default(&absolute),
            ),
        },
        // The image editors list on the working-tree file; the editor saves in
        // place and the status refresh picks the change up.
        FileContextAction::EditImageIn => crate::image_view::edit_in::open_from_menu(
            crate::image_view::edit_in::Target::File(absolute.clone()),
        ),
        FileContextAction::OpenWith => {
            crate::open_with::popup::open_from_menu(crate::open_with::Subject {
                path: path.to_path_buf(),
                source: if section == StatusSection::Staged {
                    crate::open_with::Source::Index
                } else {
                    crate::open_with::Source::WorkingTree
                },
            })
        }
        FileContextAction::ShowInFolder => report(
            last_message,
            msg::showing_explorer(path.display()),
            crate::external_tools::show_in_folder(&tools.file_explorer, &absolute),
        ),
        FileContextAction::Stage => link.submit_action(Action::StageFile(path.to_path_buf())),
        FileContextAction::Unstage => link.submit_action(Action::UnstageFile(path.to_path_buf())),
        // No confirm modal: taking a side of a conflict discards nothing
        // that git isn't still holding. The other two stages stay in the
        // index until the merge is committed, so picking the wrong one is
        // undone by picking the right one.
        FileContextAction::ResolveUsing(side) => {
            link.submit_action(Action::ResolveConflict {
                path: path.to_path_buf(),
                side,
            });
            *last_message = Some(msg::resolving(path.display(), side_word(side)));
        }
        FileContextAction::Discard => match head_oid(refs) {
            Some(source) => {
                *input_mode = InputMode::Confirm {
                    prompt: msg::discard_file_prompt(path.display()),
                    action: ConfirmAction::Destructive(PendingDestructive::DiscardFile {
                        path: path.to_path_buf(),
                        source,
                    }),
                };
            }
            None => {
                *last_message = Some(msg::CANT_DISCARD.get().into());
            }
        },
        FileContextAction::Convert(op) => {
            link.submit_action(Action::ConvertFiles {
                paths: vec![path.to_string_lossy().replace('\\', "/")],
                op,
            });
        }
        FileContextAction::CopyPath => {
            *last_message = Some(
                match crate::clipboard::deliver_shared(&path.to_string_lossy()) {
                    Delivery::Clipboard => msg::copied_clipboard(path.display()),
                    Delivery::Terminal => {
                        gitpoulpi_i18n::msg::input::copied_via_terminal(path.display())
                    }
                    Delivery::Nowhere(why) => gitpoulpi_i18n::msg::input::no_clipboard(why),
                },
            );
        }
        // The menu already names the file, so unlike the palette's
        // version these two never have to go looking for a target.
        FileContextAction::FileHistory => open_wip_insight(
            FileInsightKind::History,
            path.to_path_buf(),
            section == StatusSection::Staged,
            refs,
            link,
            file_insight,
        ),
        FileContextAction::FileBlame => open_wip_insight(
            FileInsightKind::Blame,
            path.to_path_buf(),
            section == StatusSection::Staged,
            refs,
            link,
            file_insight,
        ),
        FileContextAction::CreatePatch => {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "wip".to_string());
            *patch_seed = Some(crate::repo_dialogs::PatchSeed {
                source: gitpoulpi_core::domain::PatchSource::WorkingTree {
                    paths: vec![path.to_string_lossy().into_owned()],
                },
                name: format!("wip-{name}"),
            });
        }
        // The LFS rows (`apply_lfs_file_action`) and `EditFile` (which needs
        // the editor window) are run by the event loop.
        _ => {}
    }
}

/// The drill-in menu for a folder row, captioned with the folder.
pub(crate) fn open_folder_menu(
    section: StatusSection,
    folder: &std::path::Path,
    anchor_col: u16,
    anchor_row: u16,
) -> ContextMenu<FolderContextAction> {
    ContextMenu::new(
        truncate_with_ellipsis(&format!("{}/", folder.to_string_lossy()), 40),
        crate::status_pane::folder_context_entries(section, folder),
        anchor_col,
        anchor_row,
    )
}

/// The menu of a right-click on the Unstaged pane outside any item.
pub(crate) fn open_empty_area_menu(
    section: StatusSection,
    anchor_col: u16,
    anchor_row: u16,
) -> ContextMenu<FolderContextAction> {
    ContextMenu::new(
        msg::WORKING_CHANGES.get(),
        crate::status_pane::empty_area_entries(section),
        anchor_col,
        anchor_row,
    )
}

/// Opens the name prompt of "Create File" in `folder` (empty = the repo root).
pub(crate) fn start_create_file(folder: &std::path::Path, input_mode: &mut InputMode) {
    *input_mode = InputMode::ComposingRefName {
        op: RefNameOp::CreateFile(folder.to_path_buf()),
        value: String::new(),
    };
}

/// Runs one entry of a folder's context menu: the menu's click, its Enter
/// and the palette all land here.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_folder_context_action(
    action: FolderContextAction,
    repo_root: &std::path::Path,
    path: &std::path::Path,
    tree: &WorkingTree,
    link: &DaemonLink,
    input_mode: &mut InputMode,
    last_message: &mut Option<String>,
    patch_seed: &mut Option<crate::repo_dialogs::PatchSeed>,
) {
    match action {
        FolderContextAction::Stage => link.submit_action(Action::StageFolder(path.to_path_buf())),
        FolderContextAction::Unstage => {
            link.submit_action(Action::UnstageFolder(path.to_path_buf()))
        }
        FolderContextAction::Discard | FolderContextAction::CreateFile => {
            apply_folder_context_action(action, path, tree, input_mode, last_message)
        }
        FolderContextAction::IgnoreFolder => open_rule_file_picker(
            RuleKind::Ignore,
            path,
            RuleScope::Folder,
            None,
            repo_root,
            input_mode,
            last_message,
        ),
        FolderContextAction::TreatFolderBinary | FolderContextAction::TreatFolderText => {
            let treat = if action == FolderContextAction::TreatFolderBinary {
                Treat::Binary
            } else {
                Treat::Text
            };
            open_rule_file_picker(
                RuleKind::Attributes,
                path,
                RuleScope::Folder,
                Some(treat),
                repo_root,
                input_mode,
                last_message,
            );
        }
        FolderContextAction::StashFolder => {
            link.submit_action(Action::StashFile(path.to_path_buf()));
            *last_message = Some(msg::stashing(format!("{}/", path.display())));
        }
        FolderContextAction::CreatePatch => {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "wip".to_string());
            *patch_seed = Some(crate::repo_dialogs::PatchSeed {
                source: gitpoulpi_core::domain::PatchSource::WorkingTree {
                    paths: vec![path.to_string_lossy().replace('\\', "/")],
                },
                name: format!("wip-{name}"),
            });
        }
        FolderContextAction::Convert(op) => {
            let paths: Vec<String> = tree
                .files
                .iter()
                .filter(|f| f.path.starts_with(path))
                .map(|f| f.path.to_string_lossy().replace('\\', "/"))
                .collect();
            if paths.is_empty() {
                *last_message = Some(msg::nothing_under(path.display()));
            } else {
                *last_message = Some(gitpoulpi_i18n::msg::pane_menu::converting(
                    paths.len() as u64
                ));
                link.submit_action(Action::ConvertFiles { paths, op });
            }
        }
        FolderContextAction::StageAll => link.submit_action(Action::StageAll),
        FolderContextAction::UnstageAll => link.submit_action(Action::UnstageAll),
        FolderContextAction::DiscardAll => {
            discard_all_changes(Some(tree), input_mode, last_message)
        }
        // Folding needs the window's state: `menu_actions` runs it.
        FolderContextAction::Fold(_) => {}
        FolderContextAction::OpenRepoFolder => {
            let tools = crate::external_tools::current();
            *last_message = Some(
                match crate::external_tools::show_in_folder(&tools.file_explorer, repo_root) {
                    Ok(()) => msg::opened_folder(repo_root.display()),
                    Err(why) => crate::activity_log::report_problem_clear(
                        crate::activity_log::ProblemSource::Tool,
                        &why,
                    ),
                },
            );
        }
        FolderContextAction::OpenFolder => {
            let tools = crate::external_tools::current();
            *last_message = Some(
                match crate::external_tools::show_in_folder(
                    &tools.file_explorer,
                    &repo_root.join(path),
                ) {
                    Ok(()) => msg::opened_folder(path.display()),
                    Err(why) => crate::activity_log::report_problem_clear(
                        crate::activity_log::ProblemSource::Tool,
                        &why,
                    ),
                },
            );
        }
    }
}

/// The folder entries that only touch UI state: `Discard` always routes
/// through the confirm modal - same shape as `apply_file_context_action`'s
/// own `Discard` arm, but over every Unstaged path beneath the folder --
/// and `CreateFile` opens the name prompt. The rest is
/// `run_folder_context_action`.
///
/// An empty selection (nothing Unstaged under this folder - can happen
/// if the tree changed between the menu opening and the click landing)
/// says so instead of opening a confirm modal with nothing to confirm.
pub(crate) fn apply_folder_context_action(
    action: FolderContextAction,
    path: &std::path::Path,
    tree: &WorkingTree,
    input_mode: &mut InputMode,
    last_message: &mut Option<String>,
) {
    match action {
        FolderContextAction::Discard => {
            let paths = unstaged_paths_under(tree, path);
            if paths.is_empty() {
                *last_message = Some(msg::nothing_under(path.display()));
                return;
            }
            *input_mode = InputMode::Confirm {
                prompt: discard_confirmation_prompt(tree, &paths),
                action: ConfirmAction::Destructive(PendingDestructive::DiscardPaths(paths)),
            };
        }
        FolderContextAction::CreateFile => start_create_file(path, input_mode),
        _ => {}
    }
}

/// The global "Discard all changes" trash button and its palette/key
/// twins: every Unstaged path, confirmed exactly like
/// a folder's own Discard (`apply_folder_context_action`) - this is that
/// same action with the whole tree as the "folder".
///
/// `tree` is `None` while the working tree hasn't loaded yet (same
/// window `commit_readiness` and friends already treat as "nothing to
/// act on"), which reads as "nothing to discard" rather than an error.
pub(crate) fn discard_all_changes(
    tree: Option<&WorkingTree>,
    input_mode: &mut InputMode,
    last_message: &mut Option<String>,
) {
    let Some(tree) = tree else {
        *last_message = Some(msg::NO_TREE.get().into());
        return;
    };
    let paths = section_paths(tree, StatusSection::Unstaged);
    if paths.is_empty() {
        *last_message = Some(msg::NOTHING_TO_DISCARD.get().into());
        return;
    }
    *input_mode = InputMode::Confirm {
        prompt: discard_confirmation_prompt(tree, &paths),
        action: ConfirmAction::Destructive(PendingDestructive::DiscardPaths(paths)),
    };
}

/// The word naming a conflict side in the "resolving ..." status message.
fn side_word(side: gitpoulpi_core::domain::ConflictSide) -> &'static str {
    use gitpoulpi_core::domain::ConflictSide;
    match side {
        ConflictSide::Ours => msg::SIDE_OURS.get(),
        ConflictSide::Theirs => msg::SIDE_THEIRS.get(),
        ConflictSide::Base => msg::SIDE_BASE.get(),
    }
}
