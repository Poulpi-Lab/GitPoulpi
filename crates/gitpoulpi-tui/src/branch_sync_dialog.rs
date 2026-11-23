//! "Pull all branches": the question asked before the run when a branch that
//! is behind its upstream is checked out in another worktree.
//!
//! Three numbered answers, with what each one does spelled out under the
//! highlighted row (master and detail, like `tag_delete_dialog`):
//!
//! 1. **Update those worktrees too**: the run fast-forwards them in their own
//!    folders (`worktrees: update`).
//! 2. **Skip them**: the run leaves them and reports them (`worktrees: skip`).
//! 3. **Cancel**: nothing runs.
//!
//! Keyboard: Up/Down/Tab/Shift+Tab (or `j`/`k`) move, `1`-`3` answer, Enter
//! answers the highlighted row, Esc cancels. Mouse: a click on a row answers
//! it, the close cross and a click away cancel.
//!
//! **How the branches are found, without waiting on the render thread.** The
//! window already holds the refs; a local branch with an upstream whose tip
//! differs from the upstream's is a candidate. Two answers from the daemon
//! (`Requester`, polled once a frame by [`tick`]) narrow it down: the worktree
//! list says which candidates are checked out elsewhere (and where), and a
//! `MergeBase` per such candidate says whether it is only behind (its tip is
//! the merge base) rather than diverged, which the run never touches anyway.
//! No such branch: the run starts at once, as `worktrees: skip`. Only the
//! tracking refs the window has are looked at, so a branch that becomes
//! behind with this very fetch is not asked about (it is skipped and shows up
//! in the summary). Process-wide state, like the other small modals, because
//! the window restarts on every tab switch; every stage is tied to the
//! repository root it began in.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use gitpoulpi_core::domain::{
    FetchOptions, Oid, PullAllBranchesRequest, PullMode, Ref, RefKind, Worktree, WorktreePolicy,
};
use gitpoulpi_core::Result as GitResult;
use gitpoulpi_i18n::msg::branch_sync as msg;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use crate::daemon_link::{Action, DaemonLink, Reply, Requester};
use crate::dialog_kit;
use crate::th;
use gitpoulpi_core::domain::OperationKind;

const WIN: dialog_kit::Win = dialog_kit::Win::movable("branch_sync");

/// The most branches listed before "and N more".
const MAX_LISTED: usize = 6;

/// What a run was asked for, apart from the worktree answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RunSpec {
    pub(crate) head_mode: Option<PullMode>,
    pub(crate) fetch_opts: FetchOptions,
}

impl RunSpec {
    fn action(&self, worktrees: WorktreePolicy) -> Action {
        Action::PullAllBranches(PullAllBranchesRequest {
            fetch: true,
            fetch_opts: self.fetch_opts.clone(),
            head_mode: self.head_mode,
            worktrees,
        })
    }
}

/// A branch the window's refs say may be behind its upstream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Candidate {
    pub(crate) branch: String,
    pub(crate) tip: Oid,
    pub(crate) upstream_tip: Oid,
}

/// A behind branch checked out in another worktree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Elsewhere {
    pub(crate) branch: String,
    pub(crate) path: PathBuf,
}

/// The answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Answer {
    Update,
    Skip,
    Cancel,
}

const ANSWERS: [Answer; 3] = [Answer::Update, Answer::Skip, Answer::Cancel];

enum Stage {
    /// Waiting for the worktree list.
    Worktrees {
        reply: Reply<Vec<Worktree>>,
        candidates: Vec<Candidate>,
    },
    /// Waiting for the merge bases of the candidates checked out elsewhere.
    Probing {
        probes: Vec<(Elsewhere, Oid, Reply<Option<Oid>>)>,
        settled: Vec<(Elsewhere, Oid, Option<Oid>)>,
    },
    /// The question is up.
    Asking(Open),
}

struct Open {
    list: Vec<Elsewhere>,
    selected: usize,
    /// The rect of each answer row, from the last paint.
    rows: Vec<Rect>,
    close: dialog_kit::CloseHit,
}

struct Pending {
    root: PathBuf,
    spec: RunSpec,
    stage: Stage,
}

static STATE: Mutex<Option<Pending>> = Mutex::new(None);

fn state() -> std::sync::MutexGuard<'static, Option<Pending>> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

/// The local branches (not the checked-out one of this window) whose tip
/// differs from their remote-tracking branch's: the only ones that can be
/// behind.
pub(crate) fn candidates(refs: &[Ref]) -> Vec<Candidate> {
    refs.iter()
        .filter(|r| !r.is_head)
        .filter_map(|r| {
            let RefKind::LocalBranch { name } = &r.kind else {
                return None;
            };
            let upstream = r.upstream.as_deref()?;
            let upstream_tip = refs.iter().find_map(|o| match &o.kind {
                RefKind::RemoteBranch { remote, name }
                    if format!("{remote}/{name}") == upstream =>
                {
                    Some(o.target)
                }
                _ => None,
            })?;
            (upstream_tip != r.target).then(|| Candidate {
                branch: name.clone(),
                tip: r.target,
                upstream_tip,
            })
        })
        .collect()
}

/// Starts "Pull all branches" for the window of `root`: at once when no
/// candidate exists, else after the worktree check (and the question, if a
/// behind branch is checked out elsewhere).
pub(crate) fn begin(
    link: &DaemonLink,
    root: &Path,
    refs: &[Ref],
    head_mode: Option<PullMode>,
    fetch_opts: FetchOptions,
) {
    let requester = link.activity_handle().raw.requester();
    if let Some(action) = start(&requester, root, refs, head_mode, fetch_opts) {
        link.submit_action(action);
    }
}

/// [`begin`] without the link: the action to send now, if any.
pub(crate) fn start(
    requester: &Requester,
    root: &Path,
    refs: &[Ref],
    head_mode: Option<PullMode>,
    fetch_opts: FetchOptions,
) -> Option<Action> {
    let spec = RunSpec {
        head_mode,
        fetch_opts,
    };
    let candidates = candidates(refs);
    if candidates.is_empty() {
        return Some(spec.action(WorktreePolicy::Skip));
    }
    let reply = requester.call::<Vec<Worktree>>(OperationKind::Worktrees, serde_json::json!({}));
    *state() = Some(Pending {
        root: root.to_path_buf(),
        spec,
        stage: Stage::Worktrees { reply, candidates },
    });
    None
}

/// Moves the check along, once a frame, from the window of `root`. Sends the
/// run when there is nothing to ask.
pub(crate) fn tick(root: &Path, requester: &Requester, link: &DaemonLink) {
    if let Some(action) = poll(root, requester) {
        link.submit_action(action);
    }
}

/// One step of the check: the action to send now, if the check ended without
/// a question.
pub(crate) fn poll(root: &Path, requester: &Requester) -> Option<Action> {
    let mut slot = state();
    let pending = slot.as_mut().filter(|p| p.root == root)?;
    match &mut pending.stage {
        Stage::Worktrees { reply, candidates } => {
            let answer: GitResult<Vec<Worktree>> = reply.take()?;
            let worktrees = answer.unwrap_or_default();
            let probes: Vec<(Elsewhere, Oid, Reply<Option<Oid>>)> = candidates
                .iter()
                .filter_map(|c| {
                    let path = worktrees
                        .iter()
                        .find(|w| w.branch.as_deref() == Some(c.branch.as_str()))?
                        .path
                        .clone();
                    let reply = requester.call::<Option<Oid>>(
                        OperationKind::MergeBase,
                        serde_json::json!({ "a": c.tip, "b": c.upstream_tip }),
                    );
                    Some((
                        Elsewhere {
                            branch: c.branch.clone(),
                            path,
                        },
                        c.tip,
                        reply,
                    ))
                })
                .collect();
            pending.stage = Stage::Probing {
                probes,
                settled: Vec::new(),
            };
            drop(slot);
            poll(root, requester)
        }
        Stage::Probing { probes, settled } => {
            let mut waiting = Vec::new();
            for (place, tip, reply) in std::mem::take(probes) {
                match reply.take() {
                    Some(answer) => settled.push((place, tip, answer.ok().flatten())),
                    None => waiting.push((place, tip, reply)),
                }
            }
            *probes = waiting;
            if !probes.is_empty() {
                return None;
            }
            // Only behind: the merge base is the branch's own tip.
            let mut list: Vec<Elsewhere> = settled
                .drain(..)
                .filter(|(_, tip, base)| *base == Some(*tip))
                .map(|(place, _, _)| place)
                .collect();
            list.sort_by(|a, b| a.branch.cmp(&b.branch));
            if list.is_empty() {
                let spec = pending.spec.clone();
                *slot = None;
                return Some(spec.action(WorktreePolicy::Skip));
            }
            pending.stage = Stage::Asking(Open {
                list,
                selected: 0,
                rows: Vec::new(),
                close: Default::default(),
            });
            crate::redraw::poke();
            None
        }
        Stage::Asking(_) => None,
    }
}

/// The question is on screen.
pub(crate) fn is_open() -> bool {
    state()
        .as_ref()
        .is_some_and(|p| matches!(p.stage, Stage::Asking(_)))
}

/// A check is still running (the window keeps drawing and ticking).
pub(crate) fn is_checking() -> bool {
    state()
        .as_ref()
        .is_some_and(|p| !matches!(p.stage, Stage::Asking(_)))
}

/// What an answer does: the run to send, or nothing for Cancel.
fn answered(answer: Answer, spec: &RunSpec) -> Option<Action> {
    match answer {
        Answer::Update => Some(spec.action(WorktreePolicy::Update)),
        Answer::Skip => Some(spec.action(WorktreePolicy::Skip)),
        Answer::Cancel => None,
    }
}

/// Closes the question with `answer` and sends what it says.
fn finish(slot: &mut Option<Pending>, answer: Answer) -> Option<Action> {
    let pending = slot.take()?;
    answered(answer, &pending.spec)
}

/// A key while the question is up (it swallows them all).
pub(crate) fn on_key(key: KeyEvent, link: &DaemonLink) {
    if let Some(action) = on_key_action(key) {
        link.submit_action(action);
    }
}

fn on_key_action(key: KeyEvent) -> Option<Action> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    let mut slot = state();
    let Some(Pending {
        stage: Stage::Asking(open),
        ..
    }) = slot.as_mut()
    else {
        return None;
    };
    let last = ANSWERS.len() - 1;
    match key.code {
        KeyCode::Esc => finish(&mut slot, Answer::Cancel),
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            open.selected = open.selected.saturating_sub(1);
            None
        }
        KeyCode::Down | KeyCode::Char('j') => {
            open.selected = (open.selected + 1).min(last);
            None
        }
        KeyCode::Tab => {
            open.selected = if key.modifiers.contains(KeyModifiers::SHIFT) {
                open.selected.saturating_sub(1)
            } else {
                (open.selected + 1).min(last)
            };
            None
        }
        KeyCode::Char(c @ '1'..='3') => {
            let i = c as usize - '1' as usize;
            finish(&mut slot, ANSWERS[i])
        }
        KeyCode::Enter => {
            let answer = ANSWERS[open.selected.min(last)];
            finish(&mut slot, answer)
        }
        _ => None,
    }
}

/// A mouse event while the question is up (it swallows them all).
pub(crate) fn on_mouse(mouse: MouseEvent, link: &DaemonLink) {
    if let Some(action) = on_mouse_action(mouse) {
        link.submit_action(action);
    }
}

fn on_mouse_action(mouse: MouseEvent) -> Option<Action> {
    let mut slot = state();
    let Some(Pending {
        stage: Stage::Asking(open),
        ..
    }) = slot.as_mut()
    else {
        return None;
    };
    let over = open
        .rows
        .iter()
        .position(|r| r.contains((mouse.column, mouse.row).into()));
    match mouse.kind {
        MouseEventKind::Moved | MouseEventKind::Drag(_) => {
            open.close.set_hover(mouse.column, mouse.row);
            if let Some(i) = over {
                open.selected = i;
            }
            None
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if open.close.hit(mouse.column, mouse.row) {
                return finish(&mut slot, Answer::Cancel);
            }
            match over {
                Some(i) => finish(&mut slot, ANSWERS[i]),
                None => {
                    // A click away cancels and is consumed.
                    let inside = open
                        .rows
                        .first()
                        .zip(open.rows.last())
                        .is_some_and(|(a, b)| {
                            mouse.row >= a.y.saturating_sub(MAX_LISTED as u16 + 4)
                                && mouse.row <= b.y + 8
                        });
                    if inside {
                        None
                    } else {
                        finish(&mut slot, Answer::Cancel)
                    }
                }
            }
        }
        _ => None,
    }
}

fn label(answer: Answer) -> &'static str {
    match answer {
        Answer::Update => msg::OPT_UPDATE.get(),
        Answer::Skip => msg::OPT_SKIP.get(),
        Answer::Cancel => msg::OPT_CANCEL.get(),
    }
}

fn detail(answer: Answer) -> &'static str {
    match answer {
        Answer::Update => msg::DETAIL_UPDATE.get(),
        Answer::Skip => msg::DETAIL_SKIP.get(),
        Answer::Cancel => msg::DETAIL_CANCEL.get(),
    }
}

/// Paints the question, last, over the panes.
pub(crate) fn render(frame: &mut Frame) {
    let mut slot = state();
    let Some(Pending {
        stage: Stage::Asking(open),
        ..
    }) = slot.as_mut()
    else {
        return;
    };
    let area = frame.area();
    let width = 84u16.min(area.width.saturating_sub(4));
    if width < 30 || area.height < 16 {
        dialog_kit::too_small::paint_closable(
            frame.buffer_mut(),
            area,
            msg::DLG_TITLE.get(),
            &mut open.close,
        );
        return;
    }
    let inner = width.saturating_sub(4) as usize;
    let shown = open.list.len().min(MAX_LISTED);
    let hidden = open.list.len() - shown;
    let detail_lines = crate::commit_detail::wrap_text(
        detail(ANSWERS[open.selected.min(ANSWERS.len() - 1)]),
        inner,
    );
    let height = (shown as u16 + u16::from(hidden > 0) + detail_lines.len() as u16 + 11)
        .min(area.height.saturating_sub(2));
    let outer = WIN.place(area, dialog_kit::centered(area, width, height));
    let buf = frame.buffer_mut();
    open.close
        .framed(buf, outer, msg::DLG_TITLE.get(), &WIN, true);
    let text = Style::default().fg(th().ui.text).bg(th().ui.panel);
    let muted = Style::default().fg(th().ui.muted).bg(th().ui.panel);
    let room = outer.width.saturating_sub(4);
    dialog_kit::put(
        buf,
        outer.x + 2,
        outer.y + 1,
        msg::DLG_QUESTION.get(),
        room,
        text,
    );
    let mut y = outer.y + 3;
    for item in open.list.iter().take(shown) {
        let line = msg::row(&item.branch, item.path.display());
        dialog_kit::put(buf, outer.x + 4, y, &line, room.saturating_sub(2), text);
        y += 1;
    }
    if hidden > 0 {
        dialog_kit::put(
            buf,
            outer.x + 4,
            y,
            &msg::more(hidden as u64),
            room.saturating_sub(2),
            muted,
        );
        y += 1;
    }
    y += 1;
    open.rows.clear();
    for (i, answer) in ANSWERS.iter().enumerate() {
        let selected = i == open.selected;
        let style = if selected {
            Style::default()
                .fg(th().ui.text)
                .bg(th().ui.selection)
                .add_modifier(Modifier::BOLD)
        } else {
            text
        };
        let row = Rect::new(outer.x + 1, y, outer.width.saturating_sub(2), 1);
        dialog_kit::fill(buf, row, style);
        dialog_kit::put(
            buf,
            outer.x + 2,
            y,
            &format!("{}  {}", i + 1, label(*answer)),
            room,
            style,
        );
        crate::pointer_map::plain(row);
        open.rows.push(row);
        y += 1;
    }
    y += 1;
    for line in detail_lines {
        dialog_kit::put(buf, outer.x + 2, y, &line, room, muted);
        y += 1;
    }
    let hint_y = outer.y + outer.height.saturating_sub(2);
    dialog_kit::put(buf, outer.x + 2, hint_y, msg::DLG_HINT.get(), room, muted);
}

#[cfg(test)]
mod tests;
