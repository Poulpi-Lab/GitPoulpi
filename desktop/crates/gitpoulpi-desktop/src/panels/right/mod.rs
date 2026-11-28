//! The right pane: it follows the Log's selection. A commit shows **Commit details** (hash, author,
//! parent, date, change counts, message) above the **Changed files** list with its controls; the WIP
//! row shows the working changes (Unstaged / Staged Files) over the commit form.
//!
//! | Module     | Role |
//! |------------|------|
//! | `status`   | the working-tree lists and their selection as plain data |
//! | `stage`    | stage, unstage, discard and the selection gestures (each one submitted action = one undo entry) |
//! | `wip`      | the working-changes view: the two lists with their buttons, drag and drop |
//! | `composer` | the commit form: rules, commit, amend, skip hooks, the template |
//! | `commit`   | the selected commit's details and files |

mod commit;
mod composer;
mod files;
mod stage;
mod status;
mod widgets;
mod wip;

use std::collections::HashSet;
use std::sync::Arc;

use gitpoulpi_core::domain::{CommitNode, FileDiff, Oid, StatusEntry};
use gitpoulpi_i18n::msg;
use gitpoulpi_link::commit_settings::CommitSettings;
use gpui_kit::component::input::{InputEvent, InputState, TextareaState};
use gpui_kit::{
    div, px, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _, Subscription,
    Task, UniformListScrollHandle, Window,
};

pub use stage::DraggedFiles;

use super::diff::DiffSource;
use super::log::Selection as LogSelection;
use crate::daemon::Shared;
use crate::strings;
use crate::theme::Skin;
use files::{count_lines, tree_rows, FileRow};
use status::Selection;

pub enum RightEvent {
    /// Open file `ix` of this diff set in the centre pane.
    Open {
        files: Arc<Vec<FileDiff>>,
        ix: usize,
        source: DiffSource,
    },
    /// A line for the status line (why a command did nothing, what it started).
    Say(String),
}

enum Mode {
    Empty,
    Wip,
    /// A commit, with the stash label when the selection is a stash.
    Commit(CommitNode, Option<String>),
}

pub struct RightPanel {
    backend: Shared,
    mode: Mode,
    files: Arc<Vec<FileDiff>>,
    rows: Vec<FileRow>,
    status: Vec<StatusEntry>,
    branch: String,
    tree: bool,
    sort_az: bool,
    view_all: bool,
    filter: Entity<InputState>,
    summary: Entity<InputState>,
    description: Entity<TextareaState>,
    summary_len: usize,
    amend: bool,
    skip_hooks: bool,
    /// The repository's commit settings and the identity the commit is made as.
    settings: CommitSettings,
    author: Option<String>,
    template_applied: bool,
    /// The commit that was sent and has not been answered yet.
    commit_ticket: Option<gitpoulpi_link::Ticket>,
    selection: Selection,
    /// Folders folded in Tree mode.
    collapsed: HashSet<String>,
    selected: Option<usize>,
    scroll: UniformListScrollHandle,
    scroll_staged: UniformListScrollHandle,
    revision: u64,
    focus: FocusHandle,
    _task: Option<Task<()>>,
    _subs: Vec<Subscription>,
}

impl EventEmitter<RightEvent> for RightPanel {}

impl RightPanel {
    pub fn new(backend: Shared, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let filter = cx.new(|cx| InputState::new(window, cx));
        let summary =
            cx.new(|cx| InputState::new(window, cx).placeholder(msg::composer::SUMMARY.get()));
        let description = cx.new(|cx| {
            TextareaState::new(window, cx)
                .auto_grow(3, 5)
                .placeholder(msg::composer::DESCRIPTION.get())
        });
        let subs = vec![
            cx.subscribe(&filter, |this, _, ev: &InputEvent, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.rebuild_rows(cx);
                }
            }),
            cx.subscribe_in(
                &summary,
                window,
                |this, s, ev: &InputEvent, window, cx| match ev {
                    InputEvent::Change => {
                        this.summary_len = s.read(cx).value().chars().count();
                        cx.notify();
                    }
                    // Enter in the summary moves on to the description; Ctrl+Enter commits.
                    InputEvent::PressEnter { secondary, .. } => {
                        if *secondary {
                            this.commit(window, cx);
                        } else {
                            this.description.read(cx).focus_handle(cx).focus(window, cx);
                        }
                    }
                    _ => {}
                },
            ),
            cx.subscribe_in(
                &description,
                window,
                |this, _, ev: &InputEvent, window, cx| {
                    if let InputEvent::PressEnter {
                        secondary: true, ..
                    } = ev
                    {
                        this.commit(window, cx);
                    }
                },
            ),
        ];
        Self {
            backend,
            mode: Mode::Empty,
            files: Arc::default(),
            rows: Vec::new(),
            status: Vec::new(),
            branch: String::new(),
            tree: false,
            sort_az: true,
            view_all: false,
            filter,
            summary,
            description,
            summary_len: 0,
            amend: false,
            skip_hooks: false,
            settings: CommitSettings::default(),
            author: None,
            template_applied: false,
            commit_ticket: None,
            selection: Selection::default(),
            collapsed: HashSet::new(),
            selected: None,
            scroll: UniformListScrollHandle::new(),
            scroll_staged: UniformListScrollHandle::new(),
            revision: 0,
            focus: cx.focus_handle(),
            _task: None,
            _subs: subs,
        }
    }

    pub fn set_branch(&mut self, branch: Option<String>) {
        self.branch = branch.unwrap_or_default();
    }

    /// Whether the working changes are what the pane shows.
    pub fn shows_working_changes(&self) -> bool {
        matches!(self.mode, Mode::Wip)
    }

    fn say(&mut self, text: impl Into<String>, cx: &mut Context<Self>) {
        cx.emit(RightEvent::Say(text.into()));
    }

    /// Shows what the Log's selection stands for.
    pub fn show(&mut self, sel: LogSelection, status: &[StatusEntry], cx: &mut Context<Self>) {
        self.set_status(status);
        self.selected = None;
        match sel {
            LogSelection::Wip => {
                self.mode = Mode::Wip;
                self.files = Arc::default();
                self.rebuild_rows(cx);
                self.load_commit_context(cx);
            }
            LogSelection::Commit(node) => self.load_commit(node.id, Mode::Commit(node, None), cx),
            LogSelection::Stash(st) => {
                if let Some(c) = st.commit {
                    let stand_in = CommitNode {
                        id: c,
                        parents: vec![],
                        summary: st.message.clone(),
                        message: st.message.clone(),
                        author_name: String::new(),
                        author_email: String::new(),
                        timestamp: st.timestamp,
                        refs: vec![],
                    };
                    self.load_commit(
                        c,
                        Mode::Commit(stand_in, Some(strings::stash_name(st.id.0))),
                        cx,
                    );
                }
            }
        }
        cx.notify();
    }

    pub fn update_status(&mut self, status: &[StatusEntry], cx: &mut Context<Self>) {
        self.set_status(status);
        if matches!(self.mode, Mode::Wip) {
            self.rebuild_rows(cx);
        }
        self.settle(cx);
    }

    /// Takes a fresh working tree: the selection drops what left its list, and the link learns which
    /// files are one rename.
    fn set_status(&mut self, status: &[StatusEntry]) {
        self.status = status.to_vec();
        self.backend.set_rename_pairs(&self.status);
        let status = &self.status;
        self.selection.retain(|section, path| {
            status::section_paths(status, section)
                .iter()
                .any(|p| p == path)
        });
    }

    fn load_commit(&mut self, oid: Oid, mode: Mode, cx: &mut Context<Self>) {
        self.mode = mode;
        self.revision += 1;
        let rev = self.revision;
        self.files = Arc::default();
        self.rows.clear();
        let fut = self.backend.commit_diff(oid);
        self._task = Some(cx.spawn(async move |this, cx| {
            let files = fut.await.unwrap_or_default();
            let _ = this.update(cx, |this, cx| {
                if this.revision != rev {
                    return;
                }
                this.files = Arc::new(files);
                this.rebuild_rows(cx);
            });
        }));
        cx.notify();
    }

    fn rebuild_rows(&mut self, cx: &mut Context<Self>) {
        let q = self.filter.read(cx).value().to_lowercase();
        let mut rows: Vec<FileRow> = Vec::new();
        if matches!(self.mode, Mode::Commit(..)) {
            for (ix, f) in self.files.iter().enumerate() {
                let path = f.path.to_string_lossy().into_owned();
                if !q.is_empty() && !path.to_lowercase().contains(&q) {
                    continue;
                }
                let (adds, dels) = count_lines(f);
                rows.push(FileRow::file(path, f.status, adds, dels, ix));
            }
        }
        rows.sort_by(|a, b| a.path.cmp(&b.path));
        if !self.sort_az {
            rows.reverse();
        }
        if self.tree {
            rows = tree_rows(rows);
        }
        self.rows = rows;
        cx.notify();
    }

    fn open_commit_file(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.selected = Some(ix);
        cx.emit(RightEvent::Open {
            files: self.files.clone(),
            ix,
            source: DiffSource::Commit,
        });
        cx.notify();
    }

    /// Opens the diff of a working-tree file: the unstaged half or the staged one.
    fn open_working(&mut self, path: String, staged: bool, cx: &mut Context<Self>) {
        let fut = self.backend.working_diff(staged);
        let source = if staged {
            DiffSource::Staged
        } else {
            DiffSource::Unstaged
        };
        cx.spawn(async move |this, cx| {
            let files = fut.await.unwrap_or_default();
            let ix = files
                .iter()
                .position(|f| f.path.to_string_lossy() == path)
                .unwrap_or(0);
            let _ = this.update(cx, |_, cx| {
                cx.emit(RightEvent::Open {
                    files: Arc::new(files),
                    ix,
                    source,
                })
            });
        })
        .detach();
    }
}

impl Focusable for RightPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for RightPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let body = match &self.mode {
            Mode::Empty => div().into_any_element(),
            Mode::Wip => self.wip_view(window, &skin, cx).into_any_element(),
            Mode::Commit(node, stash) => {
                let (node, stash) = (node.clone(), stash.clone());
                self.commit_view(&node, &stash, &skin, cx)
                    .into_any_element()
            }
        };
        div()
            .id("right-pane")
            .size_full()
            .p(px(6.0))
            .bg(skin.side_bg)
            .key_context("GpStatusPane")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev, window, cx| this.key_down(ev, window, cx)))
            .child(body)
    }
}
