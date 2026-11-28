//! What the undo-safety panel's actions do: the confirmations, the undo and redo steps, the restores and
//! the recoveries of the Reflog and Lost work views. A child of `safety` so it reaches the view's state.

use gitpoulpi_core::domain::{
    short, Direction, JournalState, Oid, SequenceGroup, UndoOutcome, UndoRefusal,
};
use gitpoulpi_core::Error as CoreError;
use gitpoulpi_i18n::msg;
use gitpoulpi_i18n::msg::undo_safety as m;
use gitpoulpi_link::error_text::error_text;
use gitpoulpi_link::undo_label as label;
use gpui_kit::{Context, Window};

use super::{NameFor, SafetyView};
use crate::dialogs::{self, Choice, Spec};
use crate::panels::activity::safety_model::{selected_commit, Act, Tab};

impl SafetyView {
    // ---- acting --------------------------------------------------------------------------------

    /// Runs `act` on the selected row, after the confirmation it needs.
    pub fn run(&mut self, act: Act, cx: &mut Context<Self>) {
        if self.busy {
            self.say(m::WHY_BUSY.get().to_string(), cx);
            return;
        }
        if !self.actions().contains(&act) {
            self.say(self.why_not(act), cx);
            return;
        }
        let state = self.journal_state();
        match act {
            Act::UndoTo => self.ask_steps(Direction::Undo, cx),
            Act::RedoTo => self.ask_steps(Direction::Redo, cx),
            Act::UndoSequence => self.ask_sequence(&state, cx),
            Act::RestoreOnly => {
                if let Some(id) = self.history.selected().map(|r| r.entry.summary.id) {
                    self.start_restore(id, None, true, cx);
                }
            }
            Act::Offer => {
                if let Some((refname, oid, _)) = label::offer_of(&state) {
                    self.start_reflog_restore(refname, oid, None, true, cx);
                }
            }
            Act::ShowInLog => {
                if let Some(oid) = selected_commit(self.tab, &self.reflog, &self.lost) {
                    self.close_and(cx, move |ws, _, cx| {
                        ws.log.update(cx, |l, cx| l.select_oid(oid, cx))
                    });
                }
            }
            Act::CherryPick => {
                if let Some(oid) = selected_commit(self.tab, &self.reflog, &self.lost) {
                    self.backend
                        .submit(crate::daemon::commit_ops::cherry_pick_action(oid));
                    self.say(m::cherry_picking(short(oid)), cx);
                }
            }
            Act::LoadMore => self.load_more(cx),
            Act::CreateBranch => {
                if let Some(at) = selected_commit(self.tab, &self.reflog, &self.lost) {
                    self.ask_name(
                        m::name_branch(short(at)),
                        format!("restored-{}", short(at)),
                        NameFor::Branch { at },
                        cx,
                    );
                }
            }
            Act::CreateTag => {
                if let Some(at) = selected_commit(self.tab, &self.reflog, &self.lost) {
                    self.ask_name(
                        m::name_tag(short(at)),
                        format!("restored-{}", short(at)),
                        NameFor::Tag { at },
                        cx,
                    );
                }
            }
            Act::ApplyToWorktree => {
                if let Some(commit) = selected_commit(self.tab, &self.reflog, &self.lost) {
                    self.start_apply(commit, cx);
                }
            }
            Act::SaveAsStash => {
                if let Some(commit) = selected_commit(self.tab, &self.reflog, &self.lost) {
                    self.start_stash(commit, cx);
                }
            }
            Act::RestoreRef => {
                if let (Some(target), Some(oid)) =
                    (self.reflog.target(), self.reflog.selected().map(|e| e.new))
                {
                    self.start_reflog_restore(target, oid, None, true, cx);
                }
            }
        }
    }

    /// Why `act` cannot run on the selected row, in words.
    fn why_not(&self, act: Act) -> String {
        match (act, self.history.selected()) {
            (_, None) if self.tab == Tab::History => m::WHY_NO_ROW.get().to_string(),
            (Act::UndoTo, Some(r)) if r.redo => m::WHY_ALREADY_UNDONE.get().to_string(),
            (Act::RedoTo, Some(r)) if !r.redo => m::WHY_NOT_UNDONE.get().to_string(),
            (Act::RestoreOnly, Some(r)) if r.redo => m::WHY_RESTORE_REDO.get().to_string(),
            (Act::RestoreOnly, _) => m::WHY_NO_RESTORE.get().to_string(),
            (Act::Offer, _) => m::WHY_OFFER_NEWEST.get().to_string(),
            (Act::RestoreRef, _) => m::WHY_NO_REF.get().to_string(),
            (Act::LoadMore, _) => m::WHY_NO_MORE.get().to_string(),
            _ => m::WHY_NO_ROW.get().to_string(),
        }
    }

    /// Closes the panel, then runs `f` on the main window's workspace.
    fn close_and(
        &mut self,
        cx: &mut Context<Self>,
        f: impl FnOnce(&mut crate::app::Workspace, &mut Window, &mut Context<crate::app::Workspace>)
            + 'static,
    ) {
        use gpui_kit::component::WindowExt as _;
        self.with_window(cx, move |window, cx| {
            window.close_dialog(cx);
            if let Some(ws) = crate::app::workspace_of(cx) {
                ws.update(cx, |ws, cx| f(ws, window, cx));
            }
        });
    }

    fn ask_steps(&mut self, dir: Direction, cx: &mut Context<Self>) {
        let Some((found, rows)) = self.history.steps_for_selection() else {
            return;
        };
        if found != dir {
            return;
        }
        let ids: Vec<u64> = rows.iter().map(|r| r.entry.summary.id).collect();
        let lines: Vec<String> = rows.iter().map(|r| label::entry_text(&r.entry)).collect();
        let n = ids.len() as u64;
        let (title, button, red) = match dir {
            Direction::Undo => (m::confirm_undo_to(n), m::BTN_UNDO_TO.get(), n > 1),
            Direction::Redo => (m::confirm_redo_to(n), m::BTN_REDO_TO.get(), false),
        };
        let spec = Self::confirm_spec(title, lines, button.to_string(), red);
        self.ask(
            spec,
            move |this, cx| this.start_steps(dir, ids, 0, false, None, cx),
            cx,
        );
    }

    /// "Undo whole sequence": the newest entries when one run made them, in the order they are undone.
    fn ask_sequence(&mut self, state: &JournalState, cx: &mut Context<Self>) {
        let Some(group) = state.sequence.clone().map(|g| *g) else {
            return;
        };
        let mut lines: Vec<String> = group
            .entries
            .iter()
            .map(|e| label::text(e.kind, &e.label))
            .collect();
        lines.push(m::WARNING_SEQUENCE_SCOPE.get().to_string());
        let spec = Self::confirm_spec(
            m::confirm_undo_sequence(&group.name),
            lines,
            m::BTN_UNDO_SEQUENCE.get().to_string(),
            true,
        );
        let ids: Vec<u64> = group.entries.iter().map(|e| e.id).collect();
        self.ask(
            spec,
            move |this, cx| this.start_steps(Direction::Undo, ids, 0, false, Some(group), cx),
            cx,
        );
    }

    /// Undoes or redoes `ids[done]`, then the next, one at a time.
    pub(super) fn start_steps(
        &mut self,
        dir: Direction,
        ids: Vec<u64>,
        done: usize,
        confirmed: bool,
        sequence: Option<SequenceGroup>,
        cx: &mut Context<Self>,
    ) {
        let Some(id) = ids.get(done).copied() else {
            return;
        };
        self.busy = true;
        let fut = self.backend.undo_step(dir, id, confirmed);
        self.job = Some(cx.spawn(async move |this, cx| {
            let result = fut.await;
            let _ = this.update(cx, |this, cx| {
                this.after_step(dir, ids, done, sequence, result, cx)
            });
        }));
        cx.notify();
    }

    fn after_step(
        &mut self,
        dir: Direction,
        ids: Vec<u64>,
        done: usize,
        sequence: Option<SequenceGroup>,
        result: Result<UndoOutcome, CoreError>,
        cx: &mut Context<Self>,
    ) {
        self.busy = false;
        match result {
            Ok(_) if done + 1 < ids.len() => {
                self.start_steps(dir, ids, done + 1, false, sequence, cx)
            }
            Ok(_) => {
                let n = ids.len() as u64;
                let text = match (&sequence, dir) {
                    (Some(group), _) => m::done_undo_sequence(&group.name),
                    (None, Direction::Undo) => m::done_undo_to(n),
                    (None, Direction::Redo) => m::done_redo_to(n),
                };
                self.say(text, cx);
                self.reload_current(cx);
            }
            Err(CoreError::NeedsConfirmation(prompt)) => {
                // The daemon found this step risky: ask, then resume from it with the answer.
                let title = self
                    .history
                    .rows
                    .iter()
                    .find(|r| r.entry.summary.id == ids[done])
                    .map(|r| label::entry_text(&r.entry))
                    .unwrap_or_default();
                let button = match dir {
                    Direction::Undo => msg::confirm_button::UNDO_ANYWAY,
                    Direction::Redo => msg::confirm_button::REDO_ANYWAY,
                };
                let spec = Self::confirm_spec(title, vec![prompt], button.get().to_string(), true);
                let rest = ids[done..].to_vec();
                self.ask(
                    spec,
                    move |this, cx| this.start_steps(dir, rest, 0, true, sequence, cx),
                    cx,
                );
            }
            Err(e) => {
                let stopped_at = sequence.as_ref().and_then(|g| {
                    let member = g.entries.iter().find(|x| Some(&x.id) == ids.get(done))?;
                    Some(m::sequence_stopped(
                        &g.name,
                        member.step,
                        g.of,
                        label::text(member.kind, &member.label),
                        error_text(&e),
                        (ids.len() - done) as u64,
                    ))
                });
                let text = if let Some(text) = stopped_at {
                    text
                } else if done > 0 {
                    m::stopped(done as u64, ids.len() as u64, error_text(&e))
                } else {
                    error_text(&e)
                };
                self.complain(text, cx);
                self.reload_current(cx);
            }
        }
    }

    /// Reloads the open view (and the history, which every action changes).
    pub fn reload_current(&mut self, cx: &mut Context<Self>) {
        self.history.loaded = false;
        self.load(Tab::History, cx);
        if self.tab != Tab::History {
            let tab = self.tab;
            match tab {
                Tab::Reflog => self.reflog.loaded = false,
                Tab::Lost => {
                    self.lost.loaded = false;
                    self.lost.commits.clear();
                }
                Tab::History => {}
            }
            self.load(tab, cx);
        }
    }

    fn start_restore(&mut self, id: u64, name: Option<String>, dry: bool, cx: &mut Context<Self>) {
        self.busy = true;
        let fut = self.backend.restore_entry(id, name.as_deref(), dry);
        self.job = Some(cx.spawn(async move |this, cx| {
            let result = fut.await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(outcome) if dry => {
                        let spec = Self::confirm_spec(
                            m::BTN_RESTORE_ONLY.get().to_string(),
                            outcome.steps,
                            m::CONFIRM_RESTORE.get().to_string(),
                            false,
                        );
                        this.ask(
                            spec,
                            move |this, cx| this.start_restore(id, name, false, cx),
                            cx,
                        );
                    }
                    Ok(outcome) => {
                        let key = this.undo_key();
                        let text = if outcome.side_files.is_empty() {
                            m::done_restore(outcome.steps.join("; "), key)
                        } else {
                            m::done_restore_files(outcome.side_files.join(", "), key)
                        };
                        this.say(text, cx);
                        this.reload_current(cx);
                    }
                    Err(CoreError::UndoRefused(UndoRefusal::NameTaken { name, suggested })) => {
                        this.ask_name(m::name_taken(name), suggested, NameFor::Restore { id }, cx);
                    }
                    Err(e) => this.complain(error_text(&e), cx),
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn start_reflog_restore(
        &mut self,
        refname: String,
        oid: Oid,
        mode: Option<String>,
        dry: bool,
        cx: &mut Context<Self>,
    ) {
        self.busy = true;
        let fut = self
            .backend
            .reflog_restore(&refname, oid, mode.as_deref(), dry);
        self.job = Some(cx.spawn(async move |this, cx| {
            let result = fut.await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(outcome) if dry => {
                        let hard = mode.as_deref() == Some("hard");
                        let mut lines = outcome.steps;
                        if hard {
                            lines.push(m::mode_hard_warn(this.undo_key()));
                        }
                        let spec = Self::confirm_spec(
                            m::confirm_reflog(refname.clone(), short(oid)),
                            lines,
                            m::CONFIRM_RESTORE.get().to_string(),
                            hard,
                        );
                        this.ask(
                            spec,
                            move |this, cx| {
                                this.start_reflog_restore(refname, oid, mode, false, cx)
                            },
                            cx,
                        );
                    }
                    Ok(_) => {
                        let text = m::done_reflog(refname, short(oid), this.undo_key());
                        this.say(text, cx);
                        this.reload_current(cx);
                    }
                    Err(CoreError::UndoRefused(UndoRefusal::WouldOverwrite { .. }))
                        if mode.is_none() =>
                    {
                        this.ask_mode(refname, oid, cx);
                    }
                    Err(e) => this.complain(error_text(&e), cx),
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    /// The reflog restore would overwrite local changes: what should happen to them?
    fn ask_mode(&mut self, refname: String, oid: Oid, cx: &mut Context<Self>) {
        let me = cx.weak_entity();
        let spec = Spec {
            title: m::MODE_TITLE.get().to_string(),
            lines: vec![m::mode_explain(&refname)],
            choices: vec![
                Choice::new(m::MODE_KEEP.get()),
                Choice::new(m::MODE_SOFT.get()),
                Choice::new(m::MODE_MIXED.get()),
                Choice::destructive(m::MODE_HARD.get()),
            ],
            hint: msg::diff_actions::CONFIRM_HINT.get().to_string(),
            highlight_first: false,
            yes_no_keys: false,
        };
        self.with_window(cx, move |window, cx| {
            dialogs::open(
                window,
                cx,
                spec,
                move |ix, cx| {
                    let mode = ["keep", "soft", "mixed", "hard"][ix.min(3)];
                    let _ = me.update(cx, |this, cx| {
                        this.start_reflog_restore(refname, oid, Some(mode.to_string()), true, cx)
                    });
                },
                || {},
            );
        });
    }

    /// A typed name was accepted.
    pub(super) fn submit_name(&mut self, purpose: NameFor, value: String, cx: &mut Context<Self>) {
        let value = value.trim().to_string();
        if value.is_empty() {
            return;
        }
        match purpose {
            NameFor::Restore { id } => self.start_restore(id, Some(value), true, cx),
            NameFor::Branch { at } => {
                self.busy = true;
                let fut = self.backend.branch_at(&value, at);
                self.job = Some(cx.spawn(async move |this, cx| {
                    let result = fut.await;
                    let _ = this.update(cx, |this, cx| {
                        this.busy = false;
                        match result {
                            Ok(()) => this.say(m::done_branch(value, short(at)), cx),
                            Err(e) => this.complain(error_text(&e), cx),
                        }
                        this.reload_current(cx);
                    });
                }));
            }
            NameFor::Tag { at } => {
                self.busy = true;
                let fut = self.backend.tag_at(&value, at);
                self.job = Some(cx.spawn(async move |this, cx| {
                    let result = fut.await;
                    let _ = this.update(cx, |this, cx| {
                        this.busy = false;
                        match result {
                            Ok(()) => this.say(m::done_tag(value, short(at)), cx),
                            Err(e) => this.complain(error_text(&e), cx),
                        }
                        this.reload_current(cx);
                    });
                }));
            }
        }
        cx.notify();
    }

    fn start_apply(&mut self, commit: Oid, cx: &mut Context<Self>) {
        self.busy = true;
        let fut = self.backend.apply_commit(commit);
        self.job = Some(cx.spawn(async move |this, cx| {
            let result = fut.await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(_) => {
                        let text = m::done_apply(short(commit), this.undo_key());
                        this.say(text, cx);
                        this.reload_current(cx);
                    }
                    // Nothing changed: offer to keep the changes as a stash instead.
                    Err(e) => {
                        let spec = Self::confirm_spec(
                            m::APPLY_FAILED_TITLE.get().to_string(),
                            vec![error_text(&e), m::APPLY_FAILED_NOTHING.get().to_string()],
                            m::BTN_STASH.get().to_string(),
                            false,
                        );
                        this.ask(spec, move |this, cx| this.start_stash(commit, cx), cx);
                    }
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    fn start_stash(&mut self, commit: Oid, cx: &mut Context<Self>) {
        self.busy = true;
        let fut = self.backend.stash_from_commit(commit);
        self.job = Some(cx.spawn(async move |this, cx| {
            let result = fut.await;
            let _ = this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(_) => {
                        let text = m::done_stash(short(commit), this.undo_key());
                        this.say(text, cx);
                        this.reload_current(cx);
                    }
                    Err(e) => this.complain(error_text(&e), cx),
                }
                cx.notify();
            });
        }));
        cx.notify();
    }

    /// Whether an action is still running or a view still loads.
    pub fn is_busy(&self) -> bool {
        self.busy
    }
}
