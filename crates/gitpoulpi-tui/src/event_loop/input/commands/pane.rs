//! The right pane's file lists as one family: multi-select (the Log's
//! gestures), the menus on files, folders, the empty area and a selection,
//! and what their rows do. The mouse, the menu key and the palette all land
//! here, so none can drift.

use super::CommandCtx;
use crate::clipboard_route::Delivery;
use crate::event_loop::*;
use crate::file_panel::{open_wip_file, WipFileOpen};
use gitpoulpi_core::domain::{RuleKind, RuleScope, Treat};
use gitpoulpi_i18n::msg;

/// The list whose rows own the keyboard right now, if any.
pub(in crate::event_loop) fn focused_list(cx: &CommandCtx<'_>) -> Option<FileList> {
    match *cx.focus {
        Focus::Status if cx.wip_all.shown() && *cx.right_panel == RightPanel::Status => {
            Some(FileList::WipAll)
        }
        Focus::Status => Some(match *cx.active_status_section {
            StatusSection::Unstaged => FileList::Unstaged,
            StatusSection::Staged => FileList::Staged,
        }),
        Focus::Log if *cx.commit_file_focus && cx.compare.is_none() => Some(FileList::Commit),
        _ => None,
    }
}

fn section_of(list: FileList) -> Option<StatusSection> {
    match list {
        FileList::Unstaged => Some(StatusSection::Unstaged),
        FileList::Staged => Some(StatusSection::Staged),
        _ => None,
    }
}

/// A list's rows and files as the selection sees them.
pub(in crate::event_loop) fn list_rows(cx: &CommandCtx<'_>, list: FileList) -> ListRows {
    let from_tree = |rows: &[TreeRow], paths: &[PathBuf]| ListRows {
        rows: rows
            .iter()
            .filter_map(|r| match r {
                TreeRow::Folder { path, .. } => Some(SelRow {
                    path: path.clone(),
                    folder: true,
                }),
                TreeRow::File { index, .. } => paths.get(*index).map(|p| SelRow {
                    path: p.clone(),
                    folder: false,
                }),
            })
            .collect(),
        files: paths.to_vec(),
    };
    match list {
        FileList::Unstaged | FileList::Staged => {
            let section = section_of(list).unwrap_or(StatusSection::Unstaged);
            let lines = section_lines(&*cx.status_section_lines, section);
            ListRows {
                rows: lines
                    .iter()
                    .map(|l| match l {
                        StatusPaneLine::Folder { path, .. } => SelRow {
                            path: path.clone(),
                            folder: true,
                        },
                        StatusPaneLine::File { path, .. } => SelRow {
                            path: path.clone(),
                            folder: false,
                        },
                    })
                    .collect(),
                files: lines
                    .iter()
                    .filter_map(|l| l.file().map(|(_, p)| p.clone()))
                    .collect(),
            }
        }
        FileList::Commit => from_tree(&*cx.commit_file_rows, &*cx.commit_paths_cache),
        FileList::WipAll => from_tree(&cx.wip_all.rows, &cx.wip_all.paths),
    }
}

/// The row the list's own cursor is on.
pub(in crate::event_loop) fn list_cursor(cx: &CommandCtx<'_>, list: FileList) -> Option<usize> {
    match list {
        FileList::Unstaged | FileList::Staged => {
            cx.status_states.state(section_of(list)?).selected()
        }
        FileList::Commit => cx.commit_file_state.selected(),
        FileList::WipAll => cx.wip_all.state.selected(),
    }
}

fn set_cursor(cx: &mut CommandCtx<'_>, list: FileList, i: usize) {
    match list {
        FileList::Unstaged | FileList::Staged => {
            if let Some(section) = section_of(list) {
                cx.status_states.select(section, Some(i));
                *cx.active_status_section = section;
            }
        }
        FileList::Commit => {
            cx.commit_file_state.select(Some(i));
            *cx.commit_files_follow = true;
            *cx.commit_file_focus = true;
        }
        FileList::WipAll => {
            cx.wip_all.state.select(Some(i));
            cx.wip_all.follow = true;
        }
    }
}

fn say_selected(cx: &mut CommandCtx<'_>) {
    if cx.file_sel.len() > 1 {
        *cx.last_message = Some(msg::pane_menu::selected(cx.file_sel.len() as u64));
    }
}

/// A click on row `i` with Ctrl (toggle), Shift/Alt (range) or neither
/// (start over). `true`: it was a selection gesture and the caller must not
/// also open the row; `false`: a plain click, the caller does what it
/// always did.
pub(in crate::event_loop) fn click_select(
    cx: &mut CommandCtx<'_>,
    list: FileList,
    i: usize,
    mods: KeyModifiers,
) -> bool {
    let rows = list_rows(cx, list);
    let cursor = list_cursor(cx, list);
    if mods.contains(KeyModifiers::CONTROL) {
        cx.file_sel.toggle(list, &rows, i, cursor);
    } else if mods.intersects(KeyModifiers::SHIFT | KeyModifiers::ALT) {
        cx.file_sel.extend_to(list, &rows, i, cursor);
    } else {
        cx.file_sel
            .reset_to(list, rows.rows.get(i).map(|r| r.path.clone()));
        return false;
    }
    set_cursor(cx, list, i);
    say_selected(cx);
    true
}

/// Space: toggles the cursor row; Shift+Up/Down (`step`): moves the cursor
/// and takes the range; Esc: clears. `true` when the key was ours.
pub(in crate::event_loop) fn select_key(
    cx: &mut CommandCtx<'_>,
    key: KeyEvent,
    escape_clears: bool,
) -> bool {
    let Some(list) = focused_list(cx) else {
        return false;
    };
    let rows = list_rows(cx, list);
    let cursor = list_cursor(cx, list);
    let km = crate::keymap::current();
    match key.code {
        // Space (and Insert) toggles the row in every list, as in the Log;
        // staging, the action with consequences, is Ctrl+Space
        // (`status.stage_toggle`). A Space rebound to staging stays staging.
        KeyCode::Insert | KeyCode::Char(' ')
            if key.code == KeyCode::Insert
                || (key.modifiers.is_empty()
                    && !(matches!(list, FileList::Unstaged | FileList::Staged)
                        && km.is(crate::keymap::Action::StatusStageToggle, &key))) =>
        {
            let Some(c) = cursor else {
                return true;
            };
            cx.file_sel.toggle(list, &rows, c, Some(c));
            say_selected(cx);
            true
        }
        KeyCode::Up | KeyCode::Down
            if key
                .modifiers
                .intersects(KeyModifiers::SHIFT | KeyModifiers::ALT) =>
        {
            let Some(c) = cursor else {
                return true;
            };
            let target = if key.code == KeyCode::Up {
                c.saturating_sub(1)
            } else {
                (c + 1).min(rows.rows.len().saturating_sub(1))
            };
            cx.file_sel.extend_to(list, &rows, target, Some(c));
            set_cursor(cx, list, target);
            say_selected(cx);
            true
        }
        // Select all, the palette's "Files: Select All" (a fixed key like
        // the text fields' Ctrl+A: it is only ours while a file list has
        // the focus, and then no text field can).
        KeyCode::Char('a') | KeyCode::Char('A') if key.modifiers == KeyModifiers::CONTROL => {
            select_all(cx);
            true
        }
        KeyCode::Esc if escape_clears && !cx.file_sel.is_empty() => {
            cx.file_sel.clear();
            *cx.last_message = Some(msg::input::SELECTION_CLEARED.get().into());
            true
        }
        _ => false,
    }
}

/// The palette's twins of the gestures.
pub(in crate::event_loop) fn select_all(cx: &mut CommandCtx<'_>) {
    match focused_list(cx) {
        Some(list) => {
            let rows = list_rows(cx, list);
            cx.file_sel.select_all(list, &rows);
            say_selected(cx);
        }
        None => *cx.last_message = Some(msg::pane_menu::SELECT_FILES_FIRST.get().into()),
    }
}

pub(in crate::event_loop) fn clear_selection(cx: &mut CommandCtx<'_>) {
    cx.file_sel.clear();
    *cx.last_message = Some(msg::input::SELECTION_CLEARED.get().into());
}

pub(in crate::event_loop) fn toggle_cursor(cx: &mut CommandCtx<'_>) {
    let key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    if !select_key(cx, key, false) {
        *cx.last_message = Some(msg::pane_menu::SELECT_FILES_FIRST.get().into());
    }
}

fn menu_from(cx: &CommandCtx<'_>, target: PaneTarget, col: u16, row: u16) -> OpenMenu {
    let entries = pane_menu_entries(cx.repo_root, &target);
    let menu = ContextMenu::new(pane_caption(&target), entries, col, row);
    OpenMenu::Pane((target, menu))
}

/// The rows of a pane menu. Also rebuilt in place when git's binary answer
/// for the single file lands after the menu opened.
pub(in crate::event_loop) fn pane_menu_entries(
    root: &Path,
    target: &PaneTarget,
) -> Vec<crate::context_menu::Entry<PaneAction>> {
    let first = target.first().map(Path::to_path_buf);
    let on_disk = match target.kind {
        PaneKind::File => first.as_ref().is_some_and(|p| root.join(p).exists()),
        PaneKind::Folder => target
            .folder
            .as_ref()
            .is_some_and(|f| root.join(f).is_dir()),
        _ => true,
    };
    // Only a single file reads the hint; `None` until git has answered.
    let binary = match (target.kind == PaneKind::File, first.as_ref()) {
        (true, Some(p)) => crate::binary_hint::git_answer(root, p),
        _ => Some(false),
    };
    pane_entries(&PaneMenuInput {
        target,
        on_disk,
        binary,
        editor_name: crate::external_tools::editor_name(&crate::external_tools::current().editor),
    })
}

/// The menu of a multi-selection of `list`, when the selection is more than
/// one file and `at` (the row under the pointer, if any) is part of it.
pub(in crate::event_loop) fn selection_menu(
    cx: &CommandCtx<'_>,
    list: FileList,
    at: Option<usize>,
    col: u16,
    row: u16,
) -> Option<OpenMenu> {
    if cx.file_sel.list() != Some(list) || cx.file_sel.len() < 2 {
        return None;
    }
    let rows = list_rows(cx, list);
    if let Some(i) = at {
        let r = rows.rows.get(i)?;
        if !cx.file_sel.contains(list, &r.path, r.folder) {
            return None;
        }
    }
    let paths = cx.file_sel.files_in_order(&rows);
    if paths.len() < 2 {
        return None;
    }
    let target = PaneTarget {
        list,
        kind: PaneKind::Selection,
        commit: (list == FileList::Commit)
            .then(|| cx.commit_files.as_ref().map(|o| o.commit))
            .flatten(),
        paths,
        folder: None,
        file_index: None,
    };
    Some(menu_from(cx, target, col, row))
}

/// The menu of a row (or the empty area, `at: None`) of the commit's files
/// or of the working tree's View all files. For a WIP file that has changes
/// this is the working-changes file menu, the same as in Unstaged / Staged.
pub(in crate::event_loop) fn list_row_menu(
    cx: &CommandCtx<'_>,
    list: FileList,
    at: Option<usize>,
    col: u16,
    row: u16,
) -> Option<OpenMenu> {
    let commit = match list {
        FileList::Commit => Some(cx.commit_files.as_ref()?.commit),
        _ => None,
    };
    let rows = list_rows(cx, list);
    let (kind, paths, folder, file_index) = match at {
        None => (PaneKind::Empty, Vec::new(), None, None),
        Some(i) => {
            let r = rows.rows.get(i)?;
            if r.folder {
                let files = rows
                    .files
                    .iter()
                    .filter(|f| f.starts_with(&r.path))
                    .cloned()
                    .collect();
                (PaneKind::Folder, files, Some(r.path.clone()), None)
            } else {
                let file_index = match (list, cx.commit_files.as_ref()) {
                    (FileList::Commit, Some(outcome)) => {
                        let index = cx.commit_paths_cache.iter().position(|p| *p == r.path)?;
                        changed_file_index(outcome, &*cx.commit_paths_cache, index)
                    }
                    _ => None,
                };
                (PaneKind::File, vec![r.path.clone()], None, file_index)
            }
        }
    };
    if list == FileList::WipAll && kind == PaneKind::File {
        if let Ok(tree) = cx.status_result.as_ref() {
            if let Some(entry) = wip_entry(tree, &paths[0]) {
                let section = if entry.worktree.is_some() {
                    StatusSection::Unstaged
                } else {
                    StatusSection::Staged
                };
                let path = paths[0].clone();
                let menu = open_file_menu(
                    section,
                    &path,
                    &*cx.status_result,
                    &*cx.conflicts,
                    &*cx.lfs,
                    cx.repo_root,
                    col,
                    row,
                );
                return Some(OpenMenu::File((section, path, menu)));
            }
        }
    }
    let target = PaneTarget {
        list,
        kind,
        commit,
        paths,
        folder,
        file_index,
    };
    Some(menu_from(cx, target, col, row))
}

/// Where a menu opened from the keyboard anchors: the cursor row of `list`.
pub(in crate::event_loop) fn keyboard_anchor(cx: &CommandCtx<'_>, list: FileList) -> (u16, u16) {
    match list {
        FileList::Unstaged | FileList::Staged => {
            let section = section_of(list).unwrap_or(StatusSection::Unstaged);
            let area = cx
                .status_section_areas
                .iter()
                .find(|(s, _)| *s == section)
                .map(|(_, a)| *a)
                .unwrap_or_default();
            let state = cx.status_states.state(section);
            let sel = state.selected().unwrap_or(0);
            (
                area.x + 2,
                area.y + 1 + (sel - state.offset().min(sel)) as u16,
            )
        }
        FileList::Commit => {
            let sel = cx.commit_file_state.selected().unwrap_or(0);
            (
                cx.commit_files_area.x + 2,
                cx.commit_files_area.y
                    + 1
                    + commit_detail::FILES_HEADER_ROWS
                    + (sel - cx.commit_file_state.offset().min(sel)) as u16,
            )
        }
        FileList::WipAll => {
            let sel = cx.wip_all.state.selected().unwrap_or(0);
            (
                cx.wip_all.area.x + 2,
                cx.wip_all.area.y + 1 + (sel - cx.wip_all.state.offset().min(sel)) as u16,
            )
        }
    }
}

/// The keyboard twin of a right-click on the cursor row of the focused
/// list (the palette's `Files: Open Menu`, and the menu key when a
/// selection or a commit / View all files row is focused). `true` when it
/// opened a menu or explained why not.
pub(in crate::event_loop) fn open_menu_from_keyboard(cx: &mut CommandCtx<'_>) -> bool {
    let Some(list) = focused_list(cx) else {
        return false;
    };
    let (col, row) = keyboard_anchor(cx, list);
    let cursor = list_cursor(cx, list);
    if let Some(menu) = selection_menu(cx, list, cursor, col, row) {
        *cx.open_menu = menu;
        return true;
    }
    match list {
        FileList::Commit | FileList::WipAll => {
            if cursor.is_none() {
                *cx.last_message = Some(msg::input::SELECT_FILE_FIRST.get().into());
                return true;
            }
            match list_row_menu(cx, list, cursor, col, row) {
                Some(menu) => *cx.open_menu = menu,
                None => *cx.last_message = Some(msg::input::FILE_LIST_LOADING.get().into()),
            }
            true
        }
        // The working-changes lists keep their own file / folder menus.
        FileList::Unstaged | FileList::Staged => false,
    }
}

/// The menu key on the Unstaged / Staged lists: the cursor row's file or
/// folder menu, anchored on its own row.
pub(in crate::event_loop) fn open_status_row_menu(cx: &mut CommandCtx<'_>) {
    let active = *cx.active_status_section;
    let state = cx.status_states.state(active);
    let lines = section_lines(&*cx.status_section_lines, active);
    let selected_row = state.selected().and_then(|i| lines.get(i));
    let area = cx
        .status_section_areas
        .iter()
        .find(|(s, _)| *s == active)
        .map(|(_, a)| *a)
        .unwrap_or_default();
    let sel = state.selected().unwrap_or(0);
    // +1 for the sub-pane's own top border.
    let row = area.y + 1 + (sel - state.offset().min(sel)) as u16;
    match selected_row {
        Some(StatusPaneLine::File { .. }) => {
            if let Some((section, path)) = selected_row.and_then(StatusPaneLine::file) {
                *cx.open_menu = OpenMenu::File((
                    section,
                    path.clone(),
                    open_file_menu(
                        section,
                        path,
                        &*cx.status_result,
                        &*cx.conflicts,
                        &*cx.lfs,
                        cx.repo_root,
                        area.x + 2,
                        row,
                    ),
                ));
            }
        }
        Some(StatusPaneLine::Folder { path, .. }) => {
            *cx.open_menu = OpenMenu::Folder((
                active,
                path.clone(),
                open_folder_menu(active, path, area.x + 2, row),
            ));
        }
        _ => {}
    }
}

/// The path the built-in editor has open when it is among `paths`: converting
/// it under the editor would overwrite unsaved edits, so the caller refuses and
/// says which file to close.
pub(in crate::event_loop) fn editor_blocks<'a>(
    cx: &CommandCtx<'_>,
    paths: impl IntoIterator<Item = &'a Path>,
) -> Option<PathBuf> {
    let open = cx.editor.as_ref().map(|e| e.path.clone())?;
    paths.into_iter().find(|p| *p == open).map(|_| open)
}

fn copy_to_clipboard(text: &str) -> crate::clipboard_route::Delivery {
    crate::clipboard::deliver_shared(text)
}

fn slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn short(commit: Oid) -> String {
    commit.to_hex()[..SHORT_SHA_LEN].to_string()
}

/// Runs one row of a pane menu.
pub(in crate::event_loop) fn apply_pane_action(
    cx: &mut CommandCtx<'_>,
    target: PaneTarget,
    action: PaneAction,
) {
    use PaneAction as A;
    let root = cx.repo_root.clone();
    let tools = crate::external_tools::current();
    let first = target.first().map(Path::to_path_buf).unwrap_or_default();
    let absolute = root.join(&first);
    let head = head_oid(cx.refs_result.as_deref().unwrap_or(&[]));
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
        A::Fold(op) => super::fold_files(cx, target.list, op, target.folder.as_deref()),
        A::OpenDiff | A::OpenFile => match (target.list, target.commit) {
            (FileList::Commit, Some(commit)) => {
                let open = match (action, target.file_index) {
                    (A::OpenDiff, Some(file_index)) => CommitFileOpen::Diff {
                        file_index,
                        path: first,
                    },
                    _ => CommitFileOpen::Content { path: first },
                };
                open_commit_file(
                    open,
                    commit,
                    cx.link,
                    &mut *cx.file_insight,
                    &mut *cx.diff_view_state,
                    &mut *cx.central_view,
                    None,
                );
                *cx.ref_popup = None;
                cx.open_menu.close_column_menu();
            }
            _ => open_wip_file(
                WipFileOpen::Content { path: first },
                head,
                cx.link,
                &mut *cx.file_insight,
                &mut *cx.diff_view_state,
                &mut *cx.central_view,
            ),
        },
        A::FileHistory | A::FileBlame => {
            let kind = if action == A::FileHistory {
                FileInsightKind::History
            } else {
                FileInsightKind::Blame
            };
            match target.commit {
                Some(commit) => {
                    open_file_insight_at(kind, first, commit, cx.link, &mut *cx.file_insight)
                }
                None => open_wip_insight(
                    kind,
                    first,
                    false,
                    cx.refs_result.as_deref().unwrap_or(&[]),
                    cx.link,
                    &mut *cx.file_insight,
                ),
            }
        }
        A::OpenInEditor => report(
            &mut *cx.last_message,
            msg::status_list::opened_editor(first.display()),
            crate::external_tools::open_in_editor(&tools.editor, &absolute),
        ),
        A::OpenInDefault => match crate::open_with::pointer_notice(&absolute) {
            Some(notice) => *cx.last_message = Some(notice),
            None => report(
                &mut *cx.last_message,
                msg::status_list::opened_default(first.display()),
                crate::external_tools::open_default(&absolute),
            ),
        },
        A::ShowInFolder => report(
            &mut *cx.last_message,
            msg::status_list::showing_explorer(first.display()),
            crate::external_tools::show_in_folder(&tools.file_explorer, &absolute),
        ),
        A::OpenFolder => {
            let folder = target.folder.clone().unwrap_or_default();
            report(
                &mut *cx.last_message,
                msg::status_list::opened_folder(folder.display()),
                crate::external_tools::show_in_folder(&tools.file_explorer, &root.join(&folder)),
            );
        }
        A::OpenRepoFolder => report(
            &mut *cx.last_message,
            msg::status_list::opened_folder(root.display()),
            crate::external_tools::show_in_folder(&tools.file_explorer, &root),
        ),
        A::EditFile => {
            if let Err(why) = open_editor(
                &root,
                &first,
                head,
                cx.link,
                &mut *cx.editor,
                &mut *cx.file_insight,
                &*cx.diff_view_state,
            ) {
                *cx.last_message = Some(why);
            }
        }
        A::EditImage => match target.commit {
            Some(commit) => {
                open_commit_file(
                    CommitFileOpen::Content { path: first },
                    commit,
                    cx.link,
                    &mut *cx.file_insight,
                    &mut *cx.diff_view_state,
                    &mut *cx.central_view,
                    None,
                );
                crate::image_view::edit_in::request_when_ready();
            }
            None => crate::image_view::edit_in::open_from_menu(
                crate::image_view::edit_in::Target::File(absolute),
            ),
        },
        A::OpenWith => crate::open_with::popup::open_from_menu(crate::open_with::Subject {
            path: first,
            source: match target.commit {
                Some(commit) => crate::open_with::Source::Commit(commit),
                None => crate::open_with::Source::WorkingTree,
            },
        }),
        A::CopyPath | A::CopyFullPath => {
            let lines: Vec<String> = if target.kind == PaneKind::Folder {
                vec![slash(target.folder.as_deref().unwrap_or(Path::new("")))]
            } else {
                target.paths.iter().map(|p| slash(p)).collect()
            };
            let lines: Vec<String> = if action == A::CopyFullPath {
                lines
                    .iter()
                    .map(|p| root.join(p).to_string_lossy().into_owned())
                    .collect()
            } else {
                lines
            };
            *cx.last_message = Some(match copy_to_clipboard(&lines.join("\n")) {
                Delivery::Clipboard => match lines.as_slice() {
                    [one] => msg::commit_files::copied_path(one),
                    many => msg::pane_menu::copied_paths(many.len() as u64),
                },
                Delivery::Terminal => {
                    msg::input::copied_via_terminal(msg::rewrite_push::WHAT_PATH.get())
                }
                Delivery::Nowhere(why) => msg::input::no_clipboard(why),
            });
        }
        A::IgnoreFiles | A::IgnoreExtensions => open_rule_file_picker_many(
            RuleKind::Ignore,
            &target.paths,
            if action == A::IgnoreFiles {
                RuleScope::File
            } else {
                RuleScope::Extension
            },
            None,
            &root,
            &mut *cx.input_mode,
            &mut *cx.last_message,
        ),
        A::IgnoreFolder => open_rule_file_picker(
            RuleKind::Ignore,
            target.folder.as_deref().unwrap_or(Path::new("")),
            RuleScope::Folder,
            None,
            &root,
            &mut *cx.input_mode,
            &mut *cx.last_message,
        ),
        A::TreatBinary
        | A::TreatText
        | A::TreatExtensionBinary
        | A::TreatExtensionText
        | A::TreatFolderBinary
        | A::TreatFolderText => {
            let (scope, treat) = match action {
                A::TreatBinary => (RuleScope::File, Treat::Binary),
                A::TreatText => (RuleScope::File, Treat::Text),
                A::TreatExtensionBinary => (RuleScope::Extension, Treat::Binary),
                A::TreatExtensionText => (RuleScope::Extension, Treat::Text),
                A::TreatFolderBinary => (RuleScope::Folder, Treat::Binary),
                _ => (RuleScope::Folder, Treat::Text),
            };
            let paths = match (scope, target.folder.as_ref()) {
                (RuleScope::Folder, Some(folder)) => vec![folder.clone()],
                _ => target.paths.clone(),
            };
            open_rule_file_picker_many(
                RuleKind::Attributes,
                &paths,
                scope,
                Some(treat),
                &root,
                &mut *cx.input_mode,
                &mut *cx.last_message,
            );
        }
        A::Delete => {
            let n = target.paths.len() as u64;
            *cx.input_mode = InputMode::Confirm {
                prompt: if let [one] = target.paths.as_slice() {
                    msg::status_list::delete_prompt(one.display())
                } else {
                    msg::pane_menu::delete_many_prompt(n)
                },
                action: ConfirmAction::Destructive(if let [one] = target.paths.as_slice() {
                    PendingDestructive::DeleteFile(one.clone())
                } else {
                    PendingDestructive::DeleteFiles(target.paths.clone())
                }),
            };
        }
        A::CreatePatch => {
            let paths: Vec<String> = target.paths.iter().map(|p| slash(p)).collect();
            let stem = |p: &Path| {
                p.file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default()
            };
            let what = match (target.kind, target.paths.as_slice()) {
                (PaneKind::File, [one]) => stem(one),
                (PaneKind::Folder, _) => stem(target.folder.as_deref().unwrap_or(Path::new(""))),
                _ => format!("{}-files", paths.len()),
            };
            *cx.patch_seed = Some(crate::repo_dialogs::PatchSeed {
                source: match target.commit {
                    Some(commit) => {
                        gitpoulpi_core::domain::PatchSource::CommitPaths { commit, paths }
                    }
                    None => gitpoulpi_core::domain::PatchSource::WorkingTree { paths },
                },
                name: match target.commit {
                    Some(commit) => format!("{}-{what}", short(commit)),
                    None => format!("wip-{what}"),
                },
            });
        }
        A::DiffWorkingTree => {
            if let Some(commit) = target.commit {
                *cx.file_insight = None;
                *cx.central_view = CentralView::Diff;
                *cx.diff_view_state = Some(
                    DiffViewState::for_compare(
                        commit,
                        gitpoulpi_core::domain::DiffTarget::CommitVsWorkingTree(commit),
                        target.file_index.unwrap_or(usize::MAX),
                        first,
                    )
                    .resume_last_used(),
                );
                *cx.ref_popup = None;
                cx.open_menu.close_column_menu();
            }
        }
        A::Restore => {
            if let Some(commit) = target.commit {
                let n = target.paths.len() as u64;
                *cx.input_mode = InputMode::Confirm {
                    prompt: if let [one] = target.paths.as_slice() {
                        msg::pane_menu::restore_prompt(one.display(), short(commit))
                    } else {
                        msg::pane_menu::restore_many_prompt(n, short(commit))
                    },
                    action: ConfirmAction::Destructive(PendingDestructive::RestoreFiles {
                        paths: target.paths.clone(),
                        source: commit,
                    }),
                };
            }
        }
        A::Stage => {
            for p in &target.paths {
                cx.link.submit_action(Action::StageFile(p.clone()));
            }
        }
        A::Unstage => {
            for p in &target.paths {
                cx.link.submit_action(Action::UnstageFile(p.clone()));
            }
        }
        A::Discard => match cx.status_result.as_ref() {
            Ok(tree) => {
                *cx.input_mode = InputMode::Confirm {
                    prompt: discard_confirmation_prompt(tree, &target.paths),
                    action: ConfirmAction::Destructive(PendingDestructive::DiscardPaths(
                        target.paths.clone(),
                    )),
                };
            }
            Err(_) => *cx.last_message = Some(msg::input::NO_WORKING_TREE.get().into()),
        },
        A::Convert(op) => {
            if target.paths.is_empty() {
                *cx.last_message = Some(msg::pane_menu::NOTHING_TO_CONVERT.get().into());
            } else if let Some(open) = editor_blocks(cx, target.paths.iter().map(PathBuf::as_path))
            {
                *cx.last_message = Some(msg::pane_menu::editor_open(open.display()));
            } else {
                let paths: Vec<String> = target.paths.iter().map(|p| slash(p)).collect();
                *cx.last_message = Some(msg::pane_menu::converting(paths.len() as u64));
                cx.link.submit_action(Action::ConvertFiles { paths, op });
            }
        }
        A::CreateFile => start_create_file(
            target.folder.as_deref().unwrap_or(Path::new("")),
            &mut *cx.input_mode,
        ),
    }
    // A selection is spent once its action ran (the files it named may
    // have moved to another list).
    if target.kind == PaneKind::Selection
        && matches!(action, A::Stage | A::Unstage | A::Delete | A::Restore)
    {
        cx.file_sel.clear();
    }
}
