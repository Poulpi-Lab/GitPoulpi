//! Removing a worktree, and merging its branch into another one.
//!
//! ```text
//! ╭ Remove worktree repo-feature ────────────────────────────────╮
//! │ 3 uncommitted changes in /work/repo-feature                  │
//! │   ~ src/main.rs                                              │
//! │   + notes.txt                                                │
//! │ Commit message  fix the thing█                               │
//! │ [x] Also delete branch feature (merged into main)            │
//! │ [ Commit and remove ] [ Stash and remove ] [ Open to commit ]│
//! │ [ Remove anyway ] [ Cancel ]                                 │
//! ╰──────────────────────────────────────────────────────────────╯
//! ```
//!
//! One dialog for the whole "get rid of this worktree" question: a clean
//! worktree gets a plain confirmation, a dirty one the choices that keep
//! the work (commit it on its branch, stash it, open the worktree to
//! commit later) or throw it away (a second, explicit confirmation), and
//! a branch that is already merged offers to go with it. The same dialog,
//! in [`Mode::Merge`], is the target picker of "Merge <branch> into...".
//!
//! Pure like `worktree_dialog`: the daemon's answers arrive as
//! [`Reply`]s the paint polls (nothing here waits), and the owner turns a
//! [`RemoveOutcome`] into an `Action`.

use crate::th;
use std::path::{Path, PathBuf};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::domain::{
    Oid, OperationKind, Ref, RefKind, Worktree, WorktreeChange, WorktreeCommit,
    WorktreeRemoveOptions,
};
use gitpoulpi_core::Result as GitResult;
use gitpoulpi_i18n::msg::worktree_manage as msg;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use crate::daemon_link::{Reply, Requester};
use crate::dialog_kit::{self, ButtonState, CheckState};
use crate::keymap::chord::Key;
use crate::scrollbar::{BarDrag, Track};

/// What the dialog was opened for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Remove,
    /// Only the merge target picker (the row's "Merge into...").
    Merge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Loading,
    Clean,
    Dirty,
    /// "Remove anyway" asked for a second, explicit confirmation.
    Force,
    /// Choosing the branch to merge into.
    Merge,
}

/// The dialog's focusable parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    CommitMsg,
    StashMsg,
    NewBranch,
    DeleteBranch,
    List,
    CommitRemove,
    StashRemove,
    OpenToCommit,
    RemoveAnyway,
    Remove,
    MergeThenRemove,
    RunMerge,
    RunMergeRemove,
    ForceYes,
    Back,
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RemoveOutcome {
    Stay,
    Cancel,
    Remove {
        path: PathBuf,
        opts: WorktreeRemoveOptions,
        delete_branch: Option<String>,
    },
    /// Open the worktree as a tab with the commit message focused.
    OpenToCommit(PathBuf),
    /// Merge `from` into what is checked out at `into`; `then_remove` is
    /// the removal to run once it landed.
    Merge {
        into: PathBuf,
        from: Ref,
        then_remove: Option<(PathBuf, Option<String>)>,
    },
}

/// A branch the worktree's own branch is checked against: if the branch's
/// tip is an ancestor of `oid`, the branch is merged into `label`.
struct Probe {
    label: String,
    reply: Option<Reply<Option<Oid>>>,
    /// The answered merge base (`Some(None)`: no common history).
    base: Option<Option<Oid>>,
}

/// What `WorktreeRemoveDialog::new` needs from the window.
pub(crate) struct Seed {
    pub(crate) worktree: Worktree,
    /// Every worktree of the repository (the merge target must be checked
    /// out in one of them).
    pub(crate) worktrees: Vec<Worktree>,
    pub(crate) refs: Vec<Ref>,
    pub(crate) mode: Mode,
}

pub(crate) struct WorktreeRemoveDialog {
    wt: Worktree,
    worktrees: Vec<Worktree>,
    refs: Vec<Ref>,
    mode: Mode,
    stage: Stage,
    changes_reply: Option<Reply<Vec<WorktreeChange>>>,
    changes: Vec<WorktreeChange>,
    load_error: Option<String>,
    probes: Vec<Probe>,
    /// The branch the worktree's branch is merged into, once known.
    pub(crate) merged_into: Option<String>,
    pub(crate) delete_branch: bool,
    pub(crate) commit_msg: String,
    pub(crate) stash_msg: String,
    pub(crate) new_branch: String,
    /// Local branches a merge may go into (all but the worktree's own).
    targets: Vec<String>,
    pub(crate) target: usize,
    /// First visible row of the changes list / the branch list.
    offset: usize,
    list_rect: Option<Rect>,
    bar: Option<Track>,
    bar_drag: BarDrag,
    pub(crate) focus: Focus,
    hover: Option<Focus>,
    pub(crate) message: Option<String>,
    hits: Vec<(Rect, Focus)>,
    row_hits: Vec<(Rect, usize)>,
    close: dialog_kit::CloseHit,
    edit: crate::text_edit::FieldEditor<Focus>,
}

const WIN: dialog_kit::Win = dialog_kit::Win::movable("worktree_remove");

pub(crate) fn rect(frame: Rect) -> Rect {
    WIN.place(
        frame,
        dialog_kit::centered(frame, frame.width.saturating_sub(4).min(96), 22),
    )
}

/// `HEAD`'s name for a worktree directory: its last path component.
pub(crate) fn dir_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// The local branch names of `refs`, in list order.
fn local_branches(refs: &[Ref]) -> Vec<(&str, &Ref)> {
    refs.iter()
        .filter_map(|r| match &r.kind {
            RefKind::LocalBranch { name } => Some((name.as_str(), r)),
            _ => None,
        })
        .collect()
}

/// The branch a merge proposes: the main worktree's, else `main` /
/// `master`, else the first one.
pub(crate) fn default_target(
    worktrees: &[Worktree],
    own: Option<&str>,
    targets: &[String],
) -> usize {
    let main = worktrees
        .iter()
        .find(|w| w.is_main)
        .and_then(|w| w.branch.as_deref());
    [main, Some("main"), Some("master")]
        .into_iter()
        .flatten()
        .filter(|b| Some(*b) != own)
        .find_map(|b| targets.iter().position(|t| t == b))
        .unwrap_or(0)
}

impl WorktreeRemoveDialog {
    pub(crate) fn new(seed: Seed, requester: &Requester) -> Self {
        let Seed {
            worktree,
            worktrees,
            refs,
            mode,
        } = seed;
        let own = worktree.branch.clone();
        let targets: Vec<String> = local_branches(&refs)
            .into_iter()
            .map(|(n, _)| n.to_string())
            .filter(|n| Some(n.as_str()) != own.as_deref())
            .collect();
        let target = default_target(&worktrees, own.as_deref(), &targets);
        // Merged-ness: the branch's tip must be an ancestor of its
        // upstream, of the main worktree's branch, or of main / master.
        let mut probes = Vec::new();
        if let Some(branch) = own.as_deref() {
            let tip = local_branches(&refs)
                .into_iter()
                .find(|(n, _)| *n == branch)
                .map(|(_, r)| r.clone());
            if let Some(tip) = tip {
                let mut candidates: Vec<(String, Oid)> = Vec::new();
                if let Some(up) = &tip.upstream {
                    if let Some(r) = refs.iter().find(|r| match &r.kind {
                        RefKind::RemoteBranch { remote, name } => format!("{remote}/{name}") == *up,
                        _ => false,
                    }) {
                        candidates.push((up.clone(), r.target));
                    }
                }
                let main = worktrees
                    .iter()
                    .find(|w| w.is_main)
                    .and_then(|w| w.branch.clone());
                for name in main
                    .into_iter()
                    .chain(["main".to_string(), "master".to_string()])
                {
                    if name == branch || candidates.iter().any(|(n, _)| *n == name) {
                        continue;
                    }
                    if let Some((_, r)) =
                        local_branches(&refs).into_iter().find(|(n, _)| *n == name)
                    {
                        candidates.push((name, r.target));
                    }
                }
                for (label, oid) in candidates {
                    probes.push(Probe {
                        label,
                        reply: Some(requester.call::<Option<Oid>>(
                            OperationKind::MergeBase,
                            serde_json::json!({ "a": tip.target, "b": oid }),
                        )),
                        base: None,
                    });
                }
            }
        }
        let changes_reply = (mode == Mode::Remove).then(|| {
            requester.call::<Vec<WorktreeChange>>(
                OperationKind::WorktreeChanges,
                serde_json::json!({ "path": worktree.path }),
            )
        });
        let stage = if mode == Mode::Merge {
            Stage::Merge
        } else {
            Stage::Loading
        };
        let stash_msg = msg::stash_default(dir_name(&worktree.path));
        Self {
            wt: worktree,
            worktrees,
            refs,
            mode,
            stage,
            changes_reply,
            changes: Vec::new(),
            load_error: None,
            probes,
            merged_into: None,
            delete_branch: false,
            commit_msg: String::new(),
            stash_msg,
            new_branch: String::new(),
            targets,
            target,
            offset: 0,
            list_rect: None,
            bar: None,
            bar_drag: BarDrag::default(),
            focus: if mode == Mode::Merge {
                Focus::List
            } else {
                Focus::Cancel
            },
            hover: None,
            message: None,
            hits: Vec::new(),
            row_hits: Vec::new(),
            close: Default::default(),
            edit: Default::default(),
        }
    }

    pub(crate) fn nav_targets(&self) -> Vec<(Rect, (u16, u16))> {
        dialog_kit::nav_cells(self.hits.iter().map(|(r, _)| *r))
    }

    /// Takes the daemon's answers that have arrived.
    fn poll(&mut self) {
        if let Some(reply) = &self.changes_reply {
            if let Some(result) = reply.take() {
                self.changes_reply = None;
                match result {
                    Ok(changes) => {
                        self.stage = if changes.is_empty() {
                            Stage::Clean
                        } else {
                            Stage::Dirty
                        };
                        self.changes = changes;
                        self.focus = self.first_focus();
                    }
                    Err(e) => {
                        self.stage = Stage::Clean;
                        self.load_error = Some(e.to_string());
                        self.focus = Focus::Cancel;
                    }
                }
            }
        }
        let mut found = None;
        let tip = self
            .wt
            .branch
            .as_deref()
            .and_then(|b| {
                local_branches(&self.refs)
                    .into_iter()
                    .find(|(n, _)| *n == b)
            })
            .map(|(_, r)| r.target);
        for probe in &mut self.probes {
            if let Some(reply) = &probe.reply {
                if let Some(result) = reply.take() {
                    probe.reply = None;
                    probe.base = result.ok();
                }
            }
            if let Some(base) = probe.base {
                if base.is_some() && base == tip && found.is_none() {
                    found = Some(probe.label.clone());
                }
            }
        }
        if found.is_some() && self.merged_into.is_none() {
            self.merged_into = found;
            self.delete_branch = true;
        }
    }

    /// Whether the worktree's branch is known to be merged.
    pub(crate) fn merged(&self) -> bool {
        self.merged_into.is_some()
    }

    fn dirty_text(&self) -> String {
        msg::dirty_count(self.changes.len() as u64, self.wt.path.display())
    }

    fn detached(&self) -> bool {
        self.wt.branch.is_none()
    }

    fn first_focus(&self) -> Focus {
        match self.stage {
            Stage::Dirty => Focus::CommitMsg,
            Stage::Clean => Focus::Remove,
            Stage::Merge => Focus::List,
            Stage::Force => Focus::ForceYes,
            Stage::Loading => Focus::Cancel,
        }
    }

    /// The focus ring of the current stage, in `Tab` order.
    fn order(&self) -> Vec<Focus> {
        let mut out = Vec::new();
        match self.stage {
            Stage::Loading => {}
            Stage::Clean => {
                if self.merged() {
                    out.push(Focus::DeleteBranch);
                }
                out.push(Focus::Remove);
                if !self.detached() && self.targets_exist() {
                    out.push(Focus::MergeThenRemove);
                }
            }
            Stage::Dirty => {
                out.push(Focus::List);
                out.push(Focus::CommitMsg);
                if self.detached() {
                    out.push(Focus::NewBranch);
                }
                out.push(Focus::StashMsg);
                if self.merged() {
                    out.push(Focus::DeleteBranch);
                }
                out.extend([
                    Focus::CommitRemove,
                    Focus::StashRemove,
                    Focus::OpenToCommit,
                    Focus::RemoveAnyway,
                ]);
            }
            Stage::Force => out.push(Focus::ForceYes),
            Stage::Merge => {
                out.push(Focus::List);
                out.push(Focus::RunMerge);
                if self.mode == Mode::Remove {
                    out.push(Focus::RunMergeRemove);
                }
            }
        }
        out.push(if matches!(self.stage, Stage::Force) {
            Focus::Back
        } else {
            Focus::Cancel
        });
        out
    }

    fn targets_exist(&self) -> bool {
        !self.targets.is_empty()
    }

    fn field_parts(
        &mut self,
    ) -> (
        &mut crate::text_edit::FieldEditor<Focus>,
        Option<(Focus, &mut String)>,
    ) {
        let focus = self.focus;
        let field = match focus {
            Focus::CommitMsg => Some(&mut self.commit_msg),
            Focus::StashMsg => Some(&mut self.stash_msg),
            Focus::NewBranch => Some(&mut self.new_branch),
            _ => None,
        };
        self.edit.sync(field.is_some().then_some(focus));
        (&mut self.edit, field.map(|f| (focus, f)))
    }

    pub(crate) fn paste(&mut self, text: &str) {
        let clean: String = text.chars().filter(|c| !c.is_control()).collect();
        if let (edit, Some((id, field))) = self.field_parts() {
            edit.caret_mut(id).insert(field, &clean);
        }
    }

    /// The branch name the worktree's branch commit would go on.
    fn commit_plan(&self) -> Result<WorktreeCommit, String> {
        if self.commit_msg.trim().is_empty() {
            return Err(msg::NEED_MESSAGE.get().into());
        }
        let branch = if self.detached() {
            let name = self.new_branch.trim();
            if name.is_empty() {
                return Err(msg::NEED_BRANCH.get().into());
            }
            if let Some(reason) = crate::init_dialog::branch_name_error(name) {
                return Err(reason);
            }
            Some(name.to_string())
        } else {
            None
        };
        Ok(WorktreeCommit {
            message: self.commit_msg.trim().to_string(),
            branch,
        })
    }

    fn delete_choice(&self) -> Option<String> {
        if self.delete_branch && self.merged() {
            self.wt.branch.clone()
        } else {
            None
        }
    }

    fn merge_plan(&self) -> Result<(PathBuf, Ref), String> {
        let Some(own) = self.wt.branch.as_deref() else {
            return Err(msg::DETACHED_NO_MERGE.get().into());
        };
        let Some(target) = self.targets.get(self.target) else {
            return Err(msg::MERGE_NONE.get().into());
        };
        let into = self
            .worktrees
            .iter()
            .find(|w| w.branch.as_deref() == Some(target.as_str()))
            .ok_or_else(|| msg::merge_not_checked_out(target))?;
        let from = local_branches(&self.refs)
            .into_iter()
            .find(|(n, _)| *n == own)
            .map(|(_, r)| r.clone())
            .ok_or_else(|| msg::merge_gone(own))?;
        Ok((into.path.clone(), from))
    }

    fn activate(&mut self, part: Focus) -> RemoveOutcome {
        self.message = None;
        match part {
            Focus::Cancel => RemoveOutcome::Cancel,
            Focus::Back => {
                self.stage = Stage::Dirty;
                self.focus = Focus::RemoveAnyway;
                RemoveOutcome::Stay
            }
            Focus::DeleteBranch => {
                self.delete_branch = !self.delete_branch;
                RemoveOutcome::Stay
            }
            Focus::Remove => RemoveOutcome::Remove {
                path: self.wt.path.clone(),
                opts: WorktreeRemoveOptions::default(),
                delete_branch: self.delete_choice(),
            },
            Focus::CommitRemove => match self.commit_plan() {
                Ok(commit) => RemoveOutcome::Remove {
                    path: self.wt.path.clone(),
                    opts: WorktreeRemoveOptions {
                        commit: Some(commit),
                        ..Default::default()
                    },
                    // The commit makes the branch newer than anything it
                    // was merged into: it never goes with the worktree.
                    delete_branch: None,
                },
                Err(why) => {
                    self.message = Some(why);
                    RemoveOutcome::Stay
                }
            },
            Focus::StashRemove => RemoveOutcome::Remove {
                path: self.wt.path.clone(),
                opts: WorktreeRemoveOptions {
                    stash: Some(self.stash_msg.trim().to_string()).filter(|m| !m.is_empty()),
                    ..Default::default()
                },
                delete_branch: self.delete_choice(),
            },
            Focus::OpenToCommit => RemoveOutcome::OpenToCommit(self.wt.path.clone()),
            Focus::RemoveAnyway => {
                self.stage = Stage::Force;
                self.focus = Focus::Back;
                RemoveOutcome::Stay
            }
            Focus::ForceYes => RemoveOutcome::Remove {
                path: self.wt.path.clone(),
                opts: WorktreeRemoveOptions {
                    force: true,
                    ..Default::default()
                },
                delete_branch: self.delete_choice(),
            },
            Focus::MergeThenRemove => {
                self.stage = Stage::Merge;
                self.focus = Focus::List;
                RemoveOutcome::Stay
            }
            Focus::RunMerge | Focus::RunMergeRemove => match self.merge_plan() {
                Ok((into, from)) => RemoveOutcome::Merge {
                    into,
                    from,
                    then_remove: (part == Focus::RunMergeRemove)
                        .then(|| (self.wt.path.clone(), self.delete_choice())),
                },
                Err(why) => {
                    self.message = Some(why);
                    RemoveOutcome::Stay
                }
            },
            Focus::CommitMsg | Focus::StashMsg | Focus::NewBranch | Focus::List => {
                RemoveOutcome::Stay
            }
        }
    }

    fn step_list(&mut self, delta: isize) {
        match self.stage {
            Stage::Merge => {
                let len = self.targets.len();
                if len > 0 {
                    self.target =
                        (self.target as isize + delta).clamp(0, (len as isize - 1).max(0)) as usize;
                }
            }
            _ => {
                let max = self.changes.len().saturating_sub(1);
                self.offset = (self.offset as isize + delta).clamp(0, max as isize) as usize;
            }
        }
    }

    pub(crate) fn on_key(&mut self, key: KeyEvent) -> RemoveOutcome {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);
        if let (edit, Some((id, field))) = self.field_parts() {
            if let Some((result, copied)) = edit.dialog_key(id, field, &key) {
                if result == crate::text_edit::KeyEdit::Edited {
                    self.message = None;
                }
                if copied.is_some() {
                    self.message = copied;
                }
                return RemoveOutcome::Stay;
            }
        }
        let list_focused = self.focus == Focus::List;
        match key.code {
            KeyCode::Esc => {
                if self.stage == Stage::Force {
                    return self.activate(Focus::Back);
                }
                return RemoveOutcome::Cancel;
            }
            KeyCode::Down if list_focused => self.step_list(1),
            KeyCode::Up if list_focused => self.step_list(-1),
            KeyCode::Char('j') if list_focused && key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.step_list(1)
            }
            KeyCode::Char('k') if list_focused && key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.step_list(-1)
            }
            KeyCode::PageDown if list_focused => self.step_list(8),
            KeyCode::PageUp if list_focused => self.step_list(-8),
            KeyCode::Tab | KeyCode::BackTab | KeyCode::Down | KeyCode::Up => {
                let order = self.order();
                let forward =
                    matches!(key.code, KeyCode::Down) || (key.code == KeyCode::Tab && !shift);
                let at = order.iter().position(|f| *f == self.focus).unwrap_or(0);
                let next = if forward {
                    (at + 1) % order.len()
                } else {
                    (at + order.len() - 1) % order.len()
                };
                self.focus = order[next];
            }
            KeyCode::Enter => return self.enter(),
            KeyCode::Char(' ') if self.field_parts().1.is_none() && !list_focused => {
                return self.activate(self.focus)
            }
            _ => {}
        }
        RemoveOutcome::Stay
    }

    /// `Enter`: a button does its thing; a field or the list runs the
    /// stage's default button.
    fn enter(&mut self) -> RemoveOutcome {
        let default = match (self.stage, self.focus) {
            (_, f)
                if !matches!(
                    f,
                    Focus::CommitMsg | Focus::StashMsg | Focus::NewBranch | Focus::List
                ) =>
            {
                f
            }
            (Stage::Dirty, Focus::StashMsg) => Focus::StashRemove,
            (Stage::Dirty, _) => Focus::CommitRemove,
            (Stage::Merge, _) => Focus::RunMerge,
            (Stage::Clean, _) => Focus::Remove,
            _ => return RemoveOutcome::Stay,
        };
        self.activate(default)
    }

    fn field_mouse(&mut self, mouse: MouseEvent) -> bool {
        match self.field_parts() {
            (edit, Some((id, field))) => {
                edit.dialog_mouse(Some(id), field, mouse.kind, mouse.column, mouse.row)
            }
            (edit, None) => edit.dialog_mouse(None, "", mouse.kind, mouse.column, mouse.row),
        }
    }

    pub(crate) fn on_mouse(&mut self, mouse: MouseEvent) -> RemoveOutcome {
        let top = match self.stage {
            Stage::Merge => self.target.saturating_sub(self.offset),
            _ => self.offset,
        };
        let content = match self.stage {
            Stage::Merge => self.targets.len(),
            _ => self.changes.len(),
        };
        if self.stage != Stage::Merge {
            if let Some(new) = self.bar_drag.on_mouse(self.bar, mouse, content, top) {
                self.offset = new;
                return RemoveOutcome::Stay;
            }
        }
        let hit = dialog_kit::hit_at(&self.hits, mouse.column, mouse.row);
        if !matches!(mouse.kind, MouseEventKind::Down(_)) && self.field_mouse(mouse) {
            return RemoveOutcome::Stay;
        }
        match mouse.kind {
            MouseEventKind::Moved | MouseEventKind::Drag(_) => self.hover = hit,
            MouseEventKind::ScrollDown
                if self
                    .list_rect
                    .is_some_and(|r| r.contains((mouse.column, mouse.row).into())) =>
            {
                self.step_list(1)
            }
            MouseEventKind::ScrollUp
                if self
                    .list_rect
                    .is_some_and(|r| r.contains((mouse.column, mouse.row).into())) =>
            {
                self.step_list(-1)
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(part) = hit {
                    self.focus = part;
                    if !matches!(
                        part,
                        Focus::CommitMsg | Focus::StashMsg | Focus::NewBranch | Focus::List
                    ) {
                        return self.activate(part);
                    }
                    self.field_mouse(mouse);
                } else if self.stage == Stage::Merge {
                    let row = self
                        .row_hits
                        .iter()
                        .find(|(r, _)| r.contains((mouse.column, mouse.row).into()))
                        .map(|(_, i)| *i);
                    if let Some(i) = row {
                        self.target = i;
                        self.focus = Focus::List;
                    }
                }
            }
            _ => {}
        }
        RemoveOutcome::Stay
    }

    fn title(&self) -> String {
        match self.mode {
            Mode::Merge => msg::merge_title(self.wt.branch.as_deref().unwrap_or("")),
            Mode::Remove => msg::remove_title(dir_name(&self.wt.path)),
        }
    }

    pub(crate) fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.poll();
        self.field_parts();
        self.edit.begin_frame();
        let title = self.title();
        self.close.framed(buf, area, &title, &WIN, true);
        let mut hits: Vec<(Rect, Focus)> = Vec::new();
        self.row_hits.clear();
        self.list_rect = None;
        self.bar = None;
        let inner = Rect::new(
            area.x + 2,
            area.y + 1,
            area.width.saturating_sub(4),
            area.height.saturating_sub(2),
        );
        if inner.width < 30 || inner.height < 8 {
            dialog_kit::too_small::note(buf, area);
            self.hits = hits;
            return;
        }
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let text = Style::default().fg(th().ui.text).bg(th().ui.panel);
        let mut y = inner.y;
        let bottom = inner.y + inner.height;
        // Two button rows and a status row sit at the bottom.
        let list_bottom = bottom.saturating_sub(7);

        match self.stage {
            Stage::Loading => {
                dialog_kit::put(buf, inner.x, y, msg::LOADING.get(), inner.width, muted);
            }
            Stage::Clean => {
                let line = match self.wt.branch.as_deref() {
                    Some(b) => msg::clean_text(self.wt.path.display(), b),
                    None => msg::clean_text_detached(self.wt.path.display()),
                };
                for part in crate::ui_kit::wrap_words(&line, inner.width as usize) {
                    dialog_kit::put(buf, inner.x, y, &part, inner.width, text);
                    y += 1;
                }
                y += 1;
                if let Some(error) = &self.load_error {
                    dialog_kit::put(
                        buf,
                        inner.x,
                        y,
                        error,
                        inner.width,
                        Style::default().fg(th().status.warning).bg(th().ui.panel),
                    );
                    y += 1;
                }
                y = self.paint_branch_option(buf, inner, y, &mut hits);
                let _ = y;
            }
            Stage::Dirty => {
                dialog_kit::put(buf, inner.x, y, &self.dirty_text(), inner.width, text);
                y += 1;
                let rows = (list_bottom.saturating_sub(y)).clamp(2, 8) as usize;
                let body = Rect::new(inner.x, y, inner.width, rows as u16);
                self.paint_changes(buf, body, &mut hits);
                y += rows as u16 + 1;
                y = self.paint_dirty_fields(buf, inner, y, &mut hits);
                let _ = y;
            }
            Stage::Force => {
                let line = msg::force_text(self.changes.len() as u64, self.wt.path.display());
                for part in crate::ui_kit::wrap_words(&line, inner.width as usize) {
                    dialog_kit::put(
                        buf,
                        inner.x,
                        y,
                        &part,
                        inner.width,
                        Style::default().fg(th().status.danger).bg(th().ui.panel),
                    );
                    y += 1;
                }
            }
            Stage::Merge => {
                let own = self.wt.branch.clone().unwrap_or_default();
                dialog_kit::put(buf, inner.x, y, &msg::merge_pick(&own), inner.width, text);
                y += 1;
                let rows = (list_bottom.saturating_sub(y)).clamp(2, 10) as usize;
                let body = Rect::new(inner.x, y, inner.width, rows as u16);
                self.paint_targets(buf, body, &mut hits);
                y += rows as u16;
                if let Some(target) = self.targets.get(self.target) {
                    let note = match self
                        .worktrees
                        .iter()
                        .find(|w| w.branch.as_deref() == Some(target))
                    {
                        Some(w) => msg::merge_checked_out(target, w.path.display()),
                        None => msg::merge_not_checked_out(target),
                    };
                    dialog_kit::put(buf, inner.x, y, &note, inner.width, muted);
                } else {
                    dialog_kit::put(buf, inner.x, y, msg::MERGE_NONE.get(), inner.width, muted);
                }
            }
        }

        let status_y = bottom.saturating_sub(4);
        if let Some(message) = &self.message {
            dialog_kit::put(
                buf,
                inner.x,
                status_y,
                message,
                inner.width,
                Style::default().fg(th().status.danger).bg(th().ui.panel),
            );
        }

        // Buttons, wrapped over two rows when they do not fit one.
        let buttons = self.buttons();
        let mut cells: Vec<(Rect, Focus)> = Vec::new();
        let mut row = bottom.saturating_sub(3);
        let mut x = inner.x;
        for (focus, label, kind) in buttons {
            let w = dialog_kit::button_width(label);
            if x + w > inner.right() {
                row += 1;
                x = inner.x;
            }
            let hovered = self.hover == Some(focus);
            let focused = self.focus == focus;
            let state = match kind {
                Kind::Primary => ButtonState::Primary { hovered, focused },
                Kind::Normal => ButtonState::Normal { hovered, focused },
                Kind::Disabled => ButtonState::Disabled,
            };
            let r = dialog_kit::button(buf, x, row, label, state);
            cells.push((r, focus));
            x += w + 1;
        }
        // Registered inner-first: buttons before the fields and rows.
        for cell in cells.into_iter().rev() {
            hits.insert(0, cell);
        }
        let tab = dialog_kit::key_name(Key::Tab);
        let enter = dialog_kit::key_name(Key::Enter);
        dialog_kit::hints(
            buf,
            inner.x,
            bottom.saturating_sub(1),
            inner.width,
            &[
                (tab.as_str(), msg::HINT_NEXT.get()),
                (enter.as_str(), msg::HINT_DEFAULT.get()),
            ],
        );
        self.hits = hits;
    }

    /// The "Also delete branch" checkbox or the reason there is none.
    fn paint_branch_option(
        &mut self,
        buf: &mut Buffer,
        inner: Rect,
        y: u16,
        hits: &mut Vec<(Rect, Focus)>,
    ) -> u16 {
        let Some(branch) = self.wt.branch.clone() else {
            return y;
        };
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        match self.merged_into.clone() {
            Some(into) => {
                let r = dialog_kit::checkbox(
                    buf,
                    inner.x,
                    y,
                    inner.width,
                    &msg::delete_branch(&branch, &into),
                    self.delete_branch,
                    CheckState::Enabled {
                        hovered: self.hover == Some(Focus::DeleteBranch),
                        focused: self.focus == Focus::DeleteBranch,
                    },
                );
                hits.push((r, Focus::DeleteBranch));
            }
            None => {
                dialog_kit::put(
                    buf,
                    inner.x,
                    y,
                    &msg::branch_not_merged(&branch),
                    inner.width,
                    muted,
                );
            }
        }
        y + 1
    }

    fn paint_dirty_fields(
        &mut self,
        buf: &mut Buffer,
        inner: Rect,
        mut y: u16,
        hits: &mut Vec<(Rect, Focus)>,
    ) -> u16 {
        let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
        let label_w = [msg::COMMIT_FIELD, msg::STASH_FIELD, msg::BRANCH_FIELD]
            .iter()
            .map(|m| crate::display_width(m.get()) as u16 + 2)
            .max()
            .unwrap_or(14);
        let field_x = inner.x + label_w;
        let field_w = inner.width.saturating_sub(label_w);
        let field = |buf: &mut Buffer,
                     y: u16,
                     label: &str,
                     value: &str,
                     hint: &str,
                     id: Focus,
                     edit: &crate::text_edit::FieldEditor<Focus>,
                     hover: Option<Focus>,
                     focus: Focus,
                     hits: &mut Vec<(Rect, Focus)>| {
            dialog_kit::put(buf, inner.x, y, label, label_w, muted);
            let r = Rect::new(field_x, y, field_w, 1);
            dialog_kit::text_input(
                buf,
                r,
                value,
                hint,
                focus == id,
                hover == Some(id),
                (edit, id),
            );
            hits.push((r, id));
        };
        field(
            buf,
            y,
            msg::COMMIT_FIELD.get(),
            &self.commit_msg,
            msg::COMMIT_HINT.get(),
            Focus::CommitMsg,
            &self.edit,
            self.hover,
            self.focus,
            hits,
        );
        y += 1;
        if self.detached() {
            field(
                buf,
                y,
                msg::BRANCH_FIELD.get(),
                &self.new_branch,
                msg::BRANCH_HINT.get(),
                Focus::NewBranch,
                &self.edit,
                self.hover,
                self.focus,
                hits,
            );
            y += 1;
        }
        field(
            buf,
            y,
            msg::STASH_FIELD.get(),
            &self.stash_msg,
            "",
            Focus::StashMsg,
            &self.edit,
            self.hover,
            self.focus,
            hits,
        );
        y += 1;
        dialog_kit::put(buf, field_x, y, msg::STASH_NOTE.get(), field_w, muted);
        y += 1;
        self.paint_branch_option(buf, inner, y, hits)
    }

    fn paint_changes(&mut self, buf: &mut Buffer, body: Rect, hits: &mut Vec<(Rect, Focus)>) {
        let rows = body.height as usize;
        let max = self.changes.len().saturating_sub(rows);
        self.offset = self.offset.min(max);
        let bar = Track::vertical(
            Rect::new(body.right().saturating_sub(1), body.y, 1, body.height),
            self.changes.len(),
            self.offset,
        );
        let text_w = body.width.saturating_sub(if bar.is_some() { 2 } else { 0 });
        for (i, change) in self.changes.iter().skip(self.offset).take(rows).enumerate() {
            let mark = if change.untracked { "+" } else { "~" };
            let style = Style::default()
                .fg(if change.untracked {
                    th().status.success
                } else {
                    th().status.warning
                })
                .bg(th().ui.panel);
            let line = format!("  {mark} {}", change.path);
            dialog_kit::put(buf, body.x, body.y + i as u16, &line, text_w, style);
        }
        hits.push((body, Focus::List));
        self.list_rect = Some(body);
        if let Some(track) = bar {
            crate::scrollbar::render_buf(buf, track, true, false);
        }
        self.bar = bar;
    }

    fn paint_targets(&mut self, buf: &mut Buffer, body: Rect, hits: &mut Vec<(Rect, Focus)>) {
        let rows = body.height as usize;
        // Keep the selection in view.
        if self.target < self.offset {
            self.offset = self.target;
        } else if self.target >= self.offset + rows {
            self.offset = self.target + 1 - rows;
        }
        for (i, name) in self.targets.iter().enumerate().skip(self.offset).take(rows) {
            let y = body.y + (i - self.offset) as u16;
            let selected = i == self.target;
            let style = if selected {
                Style::default()
                    .fg(th().ui.text)
                    .bg(th().ui.selection)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                Style::default().fg(th().refs.local).bg(th().ui.panel)
            };
            let line = format!("  {name}");
            let r = Rect::new(body.x, y, body.width, 1);
            for x in r.x..r.right() {
                buf[(x, y)].set_style(style);
            }
            dialog_kit::put(buf, body.x, y, &line, body.width, style);
            self.row_hits.push((r, i));
        }
        let _ = hits;
        self.list_rect = Some(body);
    }

    fn buttons(&self) -> Vec<(Focus, &'static str, Kind)> {
        let cancel = (Focus::Cancel, msg::CANCEL.get(), Kind::Normal);
        match self.stage {
            Stage::Loading => vec![cancel],
            Stage::Clean => {
                let mut out = vec![(Focus::Remove, msg::REMOVE.get(), Kind::Primary)];
                if !self.detached() && self.targets_exist() {
                    out.push((
                        Focus::MergeThenRemove,
                        msg::MERGE_THEN_REMOVE.get(),
                        Kind::Normal,
                    ));
                }
                out.push(cancel);
                out
            }
            Stage::Dirty => {
                let commit_ok = self.commit_plan().is_ok();
                vec![
                    (
                        Focus::CommitRemove,
                        msg::COMMIT_REMOVE.get(),
                        if commit_ok {
                            Kind::Primary
                        } else {
                            Kind::Disabled
                        },
                    ),
                    (Focus::StashRemove, msg::STASH_REMOVE.get(), Kind::Normal),
                    (Focus::OpenToCommit, msg::OPEN_TO_COMMIT.get(), Kind::Normal),
                    (Focus::RemoveAnyway, msg::REMOVE_ANYWAY.get(), Kind::Normal),
                    cancel,
                ]
            }
            Stage::Force => vec![
                (Focus::ForceYes, msg::FORCE_YES.get(), Kind::Primary),
                (Focus::Back, msg::BACK.get(), Kind::Normal),
            ],
            Stage::Merge => {
                let ok = self.merge_plan().is_ok();
                let kind = if ok { Kind::Primary } else { Kind::Disabled };
                let mut out = vec![(Focus::RunMerge, msg::MERGE_RUN.get(), kind)];
                if self.mode == Mode::Remove {
                    out.push((
                        Focus::RunMergeRemove,
                        msg::MERGE_REMOVE_RUN.get(),
                        if ok { Kind::Normal } else { Kind::Disabled },
                    ));
                }
                out.push(cancel);
                out
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Primary,
    Normal,
    Disabled,
}

impl dialog_kit::Dismiss for WorktreeRemoveDialog {
    fn is_dirty(&self) -> bool {
        false
    }

    fn close_hit(&self) -> &dialog_kit::CloseHit {
        &self.close
    }

    fn close_hit_mut(&mut self) -> &mut dialog_kit::CloseHit {
        &mut self.close
    }
}

/// The result type of a decoded reply, for tests.
#[allow(dead_code)]
type Decoded<T> = GitResult<T>;

/// The worktree a window should focus the commit message of when it opens.
static COMMIT_FOCUS: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/// "Open to commit": remembers which worktree's window gets the caret in
/// its commit message (windows are rebuilt on a tab switch, so this
/// outlives the one asking).
pub(crate) fn want_commit_focus(path: &Path) {
    if let Ok(mut held) = COMMIT_FOCUS.lock() {
        *held = Some(path.to_path_buf());
    }
}

/// Whether the window of the repository at `root` is the one waiting for
/// it; true once.
pub(crate) fn take_commit_focus(root: &Path) -> bool {
    let canonical = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let Ok(mut held) = COMMIT_FOCUS.lock() else {
        return false;
    };
    match held.as_deref() {
        Some(want) if canonical(want) == canonical(root) => {
            *held = None;
            true
        }
        _ => false,
    }
}

/// What a worktree row's menu or the palette asks the window to open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Remove the worktree at this folder.
    Remove(PathBuf),
    /// Merge the branch of the worktree at this folder into another one.
    Merge(PathBuf),
}

static REQUESTS: std::sync::Mutex<Vec<Request>> = std::sync::Mutex::new(Vec::new());

/// A menu row or palette command asks for a dialog; the window opens it on
/// its next tick (it owns the daemon link the dialog needs).
pub(crate) fn ask(request: Request) {
    if let Ok(mut held) = REQUESTS.lock() {
        held.push(request);
    }
    crate::redraw::poke();
}

pub(crate) fn take_request() -> Option<Request> {
    let mut held = REQUESTS.lock().ok()?;
    (!held.is_empty()).then(|| held.remove(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon_link::RawQueue;
    use gitpoulpi_core::domain::RefVisibility;

    fn branch(name: &str, byte: u8) -> Ref {
        Ref {
            kind: RefKind::LocalBranch { name: name.into() },
            target: Oid([byte; 20]),
            visibility: RefVisibility::Shown,
            is_head: name == "main",
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        }
    }

    fn worktrees() -> Vec<Worktree> {
        vec![
            Worktree {
                path: PathBuf::from("/r"),
                branch: Some("main".into()),
                is_main: true,
                is_locked: false,
            },
            Worktree {
                path: PathBuf::from("/wt-feature"),
                branch: Some("feature".into()),
                is_main: false,
                is_locked: false,
            },
        ]
    }

    fn dialog(mode: Mode, linked: usize) -> WorktreeRemoveDialog {
        let queue = RawQueue::default();
        let all = worktrees();
        WorktreeRemoveDialog::new(
            Seed {
                worktree: all[linked].clone(),
                worktrees: all,
                refs: vec![branch("main", 1), branch("feature", 2), branch("other", 3)],
                mode,
            },
            &queue.requester(),
        )
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn a_dirty_worktree_never_removes_without_a_choice_and_force_is_asked_twice() {
        let mut d = dialog(Mode::Remove, 1);
        d.changes = vec![
            WorktreeChange {
                path: "a.txt".into(),
                untracked: false,
            },
            WorktreeChange {
                path: "b.txt".into(),
                untracked: true,
            },
        ];
        d.stage = Stage::Dirty;
        // Commit needs a message.
        assert_eq!(d.activate(Focus::CommitRemove), RemoveOutcome::Stay);
        assert!(d.message.is_some());
        // Remove anyway: a second stage, nothing is sent yet.
        assert_eq!(d.activate(Focus::RemoveAnyway), RemoveOutcome::Stay);
        assert_eq!(d.stage, Stage::Force);
        // Esc goes back, not out.
        assert_eq!(d.on_key(key(KeyCode::Esc)), RemoveOutcome::Stay);
        assert_eq!(d.stage, Stage::Dirty);
        d.activate(Focus::RemoveAnyway);
        match d.activate(Focus::ForceYes) {
            RemoveOutcome::Remove { opts, .. } => assert!(opts.force),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn commit_and_stash_ask_for_what_they_need() {
        let mut d = dialog(Mode::Remove, 1);
        d.stage = Stage::Dirty;
        d.commit_msg = "  save it ".into();
        match d.activate(Focus::CommitRemove) {
            RemoveOutcome::Remove {
                opts,
                delete_branch,
                ..
            } => {
                let commit = opts.commit.unwrap();
                assert_eq!(commit.message, "save it");
                assert_eq!(commit.branch, None);
                assert!(!opts.force && opts.stash.is_none());
                assert_eq!(delete_branch, None, "a fresh commit is never merged");
            }
            other => panic!("{other:?}"),
        }
        match d.activate(Focus::StashRemove) {
            RemoveOutcome::Remove { opts, .. } => {
                assert_eq!(opts.stash.as_deref(), Some("WIP on wt-feature"));
            }
            other => panic!("{other:?}"),
        }
        // A detached worktree must name a branch for the commit.
        d.wt.branch = None;
        assert_eq!(d.activate(Focus::CommitRemove), RemoveOutcome::Stay);
        d.new_branch = "rescue".into();
        match d.activate(Focus::CommitRemove) {
            RemoveOutcome::Remove { opts, .. } => {
                assert_eq!(opts.commit.unwrap().branch.as_deref(), Some("rescue"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_branch_goes_only_when_it_is_known_to_be_merged_and_ticked() {
        let mut d = dialog(Mode::Remove, 1);
        d.stage = Stage::Clean;
        match d.activate(Focus::Remove) {
            RemoveOutcome::Remove { delete_branch, .. } => assert_eq!(delete_branch, None),
            other => panic!("{other:?}"),
        }
        d.merged_into = Some("main".into());
        d.delete_branch = true;
        match d.activate(Focus::Remove) {
            RemoveOutcome::Remove { delete_branch, .. } => {
                assert_eq!(delete_branch.as_deref(), Some("feature"))
            }
            other => panic!("{other:?}"),
        }
        d.activate(Focus::DeleteBranch);
        match d.activate(Focus::Remove) {
            RemoveOutcome::Remove { delete_branch, .. } => assert_eq!(delete_branch, None),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn merge_defaults_to_the_main_worktrees_branch_and_runs_where_it_is_checked_out() {
        let mut d = dialog(Mode::Merge, 1);
        assert_eq!(d.targets, vec!["main".to_string(), "other".to_string()]);
        assert_eq!(d.targets[d.target], "main");
        match d.activate(Focus::RunMerge) {
            RemoveOutcome::Merge {
                into,
                from,
                then_remove,
            } => {
                assert_eq!(into, PathBuf::from("/r"));
                assert_eq!(from.target, Oid([2; 20]));
                assert_eq!(then_remove, None);
            }
            other => panic!("{other:?}"),
        }
        // `other` is checked out nowhere: refused with the reason.
        d.target = 1;
        assert_eq!(d.activate(Focus::RunMerge), RemoveOutcome::Stay);
        assert!(d.message.as_deref().unwrap().contains("not checked out"));
        // A detached worktree has nothing to merge.
        d.wt.branch = None;
        assert!(d.merge_plan().unwrap_err().contains("detached"));
    }

    #[test]
    fn it_renders_every_stage_inside_its_frame_with_real_buttons() {
        let area = Rect::new(0, 0, 120, 40);
        for (stage, needle) in [
            (Stage::Loading, "Checking the worktree"),
            (Stage::Clean, "Remove"),
            (Stage::Dirty, "Stash and remove"),
            (Stage::Force, "Discard changes and remove"),
            (Stage::Merge, "Merge feature into:"),
        ] {
            let mut d = dialog(Mode::Remove, 1);
            d.changes = vec![WorktreeChange {
                path: "a.txt".into(),
                untracked: true,
            }];
            d.stage = stage;
            let mut buf = Buffer::empty(area);
            d.render(rect(area), &mut buf);
            let text = dialog_kit::buffer_text(&buf);
            assert!(text.contains(needle), "{stage:?}:\n{text}");
            assert!(
                text.contains("Cancel") || stage == Stage::Force,
                "{stage:?}"
            );
            for (r, _) in &d.hits {
                assert!(
                    r.right() <= rect(area).right(),
                    "{stage:?}: {r:?} spills out"
                );
            }
        }
    }
}
