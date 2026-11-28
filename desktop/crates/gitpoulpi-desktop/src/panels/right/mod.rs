//! The right pane: it follows the Log's selection. A commit shows **Commit details** (hash, author,
//! parent, date, change counts, message) above the **Changed files** list with its controls; the WIP
//! row shows the working changes (Unstaged / Staged Files) over the commit form.

mod commit;
mod files;
mod widgets;
mod wip;

use std::sync::Arc;

use gitpoulpi_core::domain::{CommitNode, FileDiff, Oid, StatusEntry};
use gitpoulpi_i18n::msg;
use gpui_kit::component::input::{InputEvent, InputState, TextareaState};
use gpui_kit::{
    div, px, App, AppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, Styled as _, Subscription,
    Task, UniformListScrollHandle, Window,
};

use super::log::Selection;
use crate::daemon::Shared;
use crate::strings;
use crate::theme::Skin;
use files::{count_lines, tree_rows, FileRow};

pub enum RightEvent {
    /// Open file `ix` of this diff set in the centre pane.
    Open {
        files: Arc<Vec<FileDiff>>,
        ix: usize,
    },
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
            cx.subscribe(&summary, |this, s, ev: &InputEvent, cx| {
                if matches!(ev, InputEvent::Change) {
                    this.summary_len = s.read(cx).value().chars().count();
                    cx.notify();
                }
            }),
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

    /// Shows what the Log's selection stands for.
    pub fn show(&mut self, sel: Selection, status: &[StatusEntry], cx: &mut Context<Self>) {
        self.status = status.to_vec();
        self.selected = None;
        match sel {
            Selection::Wip => {
                self.mode = Mode::Wip;
                self.files = Arc::default();
                self.rebuild_rows(cx);
            }
            Selection::Commit(node) => self.load_commit(node.id, Mode::Commit(node, None), cx),
            Selection::Stash(st) => {
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
        self.status = status.to_vec();
        if matches!(self.mode, Mode::Wip) {
            self.rebuild_rows(cx);
        }
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
        });
        cx.notify();
    }

    fn open_working(&mut self, path: String, staged: bool, cx: &mut Context<Self>) {
        let fut = self.backend.working_diff(staged);
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
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let skin = cx.global::<Skin>().clone();
        let body = match &self.mode {
            Mode::Empty => div().into_any_element(),
            Mode::Wip => self.wip_view(&skin, cx).into_any_element(),
            Mode::Commit(node, stash) => {
                let (node, stash) = (node.clone(), stash.clone());
                self.commit_view(&node, &stash, &skin, cx)
                    .into_any_element()
            }
        };
        div()
            .size_full()
            .p(px(6.0))
            .bg(skin.side_bg)
            .track_focus(&self.focus)
            .child(body)
    }
}
