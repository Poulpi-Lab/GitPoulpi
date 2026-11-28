//! The file context menu as a part of the window: opening it (a right click on a row of the working-changes
//! lists or of a commit's files, or the menu key) and carrying out its rows. The rows are
//! `repo_menu::file_items`; the programs they start are `commands::file_ops`.
//!
//! Stage, Unstage and Discard are the working-changes pane's own ops on its selection (a right click selects
//! the row first, like any list), so a menu row and the keyboard do the same thing.

use gitpoulpi_core::domain::RuleScope;
use gitpoulpi_i18n::msg;
use gitpoulpi_link::Action;
use gpui_kit::{ClipboardItem, Context, Pixels, Point, Window};

use super::Workspace;
use crate::commands::file_ops::{self, FileOp};
use crate::commands::ops::StatusOp;
use crate::commands::staging;
use crate::daemon::repo_ops as rules;
use crate::ref_target::{FileSide, RefTarget};
use crate::repo_menu::{self, FileMenuInput};
use crate::strings;

impl Workspace {
    /// A right click (or the menu key) on a file row: the file's menu at `at`.
    pub fn open_file_menu(
        &mut self,
        path: String,
        side: FileSide,
        at: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let status = self.right.read(cx).status_of(&path, side);
        let input = FileMenuInput {
            path: &path,
            side,
            status,
            lfs: self.lfs_rows_for(&path, side),
            editor_name: file_ops::editor_name(&file_ops::External::current(cx)),
        };
        let items = repo_menu::file_items(&input);
        self.show_items(items, RefTarget::File(path, side), at, window, cx);
    }

    /// The file a file command acts on: the menu's, else the selected one.
    fn file_of(&self, target: Option<RefTarget>, cx: &gpui_kit::App) -> Option<(String, FileSide)> {
        match target {
            Some(RefTarget::File(p, s)) => Some((p, s)),
            _ => self.right.read(cx).current_file(),
        }
    }

    pub(in crate::app) fn run_file_op(
        &mut self,
        op: FileOp,
        target: Option<RefTarget>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((path, side)) = self.file_of(target, cx) else {
            return self.say(strings::SELECT_FILE, cx);
        };
        let root = self.backend.repo_root();
        match op {
            FileOp::Stage | FileOp::Unstage | FileOp::Discard => {
                let status_op = match op {
                    FileOp::Stage => StatusOp::StageSelected,
                    FileOp::Unstage => StatusOp::UnstageSelected,
                    _ => StatusOp::DiscardSelected,
                };
                self.right.update(cx, |r, cx| {
                    r.select_file(&path, side, cx);
                    staging::run_status(r, status_op, window, cx)
                });
            }
            FileOp::IgnoreFile | FileOp::IgnoreExtension | FileOp::IgnoreFolder => {
                let scope = match op {
                    FileOp::IgnoreFile => RuleScope::File,
                    FileOp::IgnoreExtension => RuleScope::Extension,
                    _ => RuleScope::Folder,
                };
                let target_path = if scope == RuleScope::Folder {
                    match rules::folder_of(&path) {
                        Some(f) => f,
                        None => return self.say(strings::SELECT_FILE, cx),
                    }
                } else {
                    path.clone()
                };
                match rules::ignore_action(&target_path, scope) {
                    Some(action) => {
                        let pattern =
                            rules::ignore_pattern(&target_path, scope).unwrap_or_default();
                        self.backend.submit(action);
                        self.say(msg::status_list::ignoring_in(&pattern, ".gitignore"), cx);
                    }
                    None => self.say(msg::status_list::NO_EXTENSION.get(), cx),
                }
            }
            FileOp::TreatFileBinary
            | FileOp::TreatFileText
            | FileOp::TreatExtensionBinary
            | FileOp::TreatExtensionText => {
                use gitpoulpi_core::domain::{RuleKind, Treat};
                let (scope, treat) = match op {
                    FileOp::TreatFileBinary => (RuleScope::File, Treat::Binary),
                    FileOp::TreatFileText => (RuleScope::File, Treat::Text),
                    FileOp::TreatExtensionBinary => (RuleScope::Extension, Treat::Binary),
                    _ => (RuleScope::Extension, Treat::Text),
                };
                let Some(pattern) = gitpoulpi_core::domain::rule_pattern(
                    RuleKind::Attributes,
                    ".gitattributes",
                    &path,
                    scope,
                ) else {
                    return self.say(msg::status_list::NO_EXTENSION.get(), cx);
                };
                self.backend
                    .submit(rules::treat_action(&path, scope, treat));
                let said = if treat == Treat::Binary {
                    msg::status_list::treating_binary(&path, &pattern, ".gitattributes")
                } else {
                    msg::status_list::treating_text(&path, &pattern, ".gitattributes")
                };
                self.say(said, cx);
            }
            FileOp::StashFile => {
                self.backend.submit(Action::StashFile(path.clone().into()));
                self.say(msg::status_list::stashing(&path), cx);
            }
            FileOp::OpenInEditor | FileOp::OpenInDefault | FileOp::ShowInFolder => {
                let file = file_ops::absolute(&root, &path);
                if !file.exists() {
                    return self.say(strings::FILE_NOT_ON_DISK, cx);
                }
                let ext = file_ops::External::current(cx);
                let started = match op {
                    FileOp::OpenInEditor => file_ops::open_in_editor(&ext, &file),
                    FileOp::OpenInDefault => file_ops::open_in_default(&file),
                    _ => file_ops::show_in_folder(&ext, &file),
                };
                match started {
                    Ok(what) => self.say(strings::launched(&what), cx),
                    Err(why) => crate::commands::status::set_error(
                        cx,
                        strings::launch_failed(&file.to_string_lossy(), &why),
                    ),
                }
            }
            FileOp::CopyPath => {
                cx.write_to_clipboard(ClipboardItem::new_string(path.clone()));
                self.say(msg::commit_files::copied_path(&path), cx);
            }
            FileOp::History => self.show_file_history(path, window, cx),
        }
    }

    /// The file's history as a read-only list: a line per commit that changed it, newest first, and a
    /// button that shows the newest one in the Log.
    pub(in crate::app) fn show_file_history(
        &mut self,
        path: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.with_head(window, cx, move |this, head, window, cx| {
            let page = this.backend.file_history(&path, head, 0, HISTORY_LIMIT);
            cx.spawn_in(window, async move |this, cx| {
                let page = page.await;
                let _ = this.update_in(cx, |this, window, cx| match page {
                    Ok(p) if !p.entries.is_empty() => this.history_dialog(path, p, window, cx),
                    Ok(_) => this.say(strings::NO_FILE_HISTORY, cx),
                    Err(why) => crate::commands::status::set_error(cx, why),
                });
            })
            .detach();
        });
    }

    fn history_dialog(
        &mut self,
        path: String,
        page: gitpoulpi_core::domain::FileHistoryPage,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use crate::dialogs::{self, Choice, Spec};
        let dates = crate::prefs::PrefsStore::get(cx).dates().clone();
        let lines: Vec<String> = page
            .entries
            .iter()
            .map(|e| {
                format!(
                    "{}  {}  {}  {}",
                    gitpoulpi_core::domain::short(e.commit),
                    dates.format_short(e.timestamp),
                    e.author_name,
                    e.summary
                )
            })
            .collect();
        let newest = page.entries.first().map(|e| e.commit);
        let spec = Spec {
            title: strings::history_title(&path),
            lines,
            choices: vec![
                Choice::new(msg::history_view::ACTION_SHOW_IN_LOG.get()),
                Choice::new(strings::CLOSE),
            ],
            hint: String::new(),
            highlight_first: true,
            yes_no_keys: false,
        };
        dialogs::open(
            window,
            cx,
            spec,
            move |ix, cx| {
                if let (0, Some(oid)) = (ix, newest) {
                    crate::app::actions::with_workspace(cx, move |ws, _, cx| {
                        ws.show_log(cx);
                        ws.log.update(cx, |l, cx| l.select_oid(oid, cx));
                    });
                }
            },
            || {},
        );
    }
}

/// How many commits the history list reads.
const HISTORY_LIMIT: usize = 200;
