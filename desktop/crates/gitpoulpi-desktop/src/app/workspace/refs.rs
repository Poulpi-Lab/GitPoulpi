//! What the window does about branches, tags and stashes: running a [`RefOp`] on a target, checkout,
//! merge, delete and the like (a plan from `daemon::refs_ops`, asked about first when it says to), the
//! stash and tag commands, and Copy. The forms are in `ref_forms.rs`, the menu in `ref_menu_host.rs`.
//!
//! The rules live in `daemon::refs_ops` (pure, tested); this file only asks and submits. One submitted
//! action is one undo entry.

use gitpoulpi_core::domain::Oid;
use gitpoulpi_i18n::msg;
use gitpoulpi_link::Action;
use gpui_kit::{ClipboardItem, Context, Window};

use super::Workspace;
use crate::app::actions::with_workspace;
use crate::commands::refs_ops::RefOp;
use crate::commands::repo_ops::RepoOp;
use crate::daemon::refs_ops::{self as rules, Plan, Snapshot, TagDelete};
use crate::dialogs::{self, Choice, Spec};
use crate::ref_target::RefTarget;
use crate::strings;

impl Workspace {
    pub(super) fn snapshot(&self, cx: &gpui_kit::App) -> Snapshot {
        self.refs.read(cx).snapshot()
    }

    /// Runs a ref command. `target` is what a menu or a click names; without one the Refs sidebar's
    /// selection is used.
    pub fn run_ref_op(
        &mut self,
        op: RefOp,
        target: Option<RefTarget>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let snap = self.snapshot(cx);
        let selected = target.or_else(|| self.refs.read(cx).selected_target());
        let say = |this: &mut Self, text: &str, cx: &mut Context<Self>| this.say(text, cx);
        match op {
            RefOp::NewBranch => self.start_new_branch(None, window, cx),
            RefOp::NewBranchAtCommit => match self.log.read(cx).selected_commit() {
                Some(at) => self.start_new_branch(Some(at), window, cx),
                None => say(self, msg::input::SELECT_LOG_COMMIT_NOT_WIP.get(), cx),
            },
            RefOp::NewTag => self.start_new_tag(None, false, window, cx),
            RefOp::NewTagAtCommit | RefOp::NewAnnotatedTagAtCommit => {
                match self.log.read(cx).selected_commit() {
                    Some(at) => self.start_new_tag(
                        Some(at),
                        op == RefOp::NewAnnotatedTagAtCommit,
                        window,
                        cx,
                    ),
                    None => say(self, msg::input::SELECT_LOG_COMMIT_NOT_WIP.get(), cx),
                }
            }
            RefOp::Checkout => match selected {
                Some(t) => self.carry_out(rules::checkout_plan(&t, &snap), window, cx),
                None => say(self, msg::input::SELECT_REF_REFS.get(), cx),
            },
            RefOp::Merge => match selected {
                Some(RefTarget::Local(n)) => {
                    self.carry_out(rules::merge_plan(&snap, &n), window, cx)
                }
                _ => say(self, msg::input::SELECT_BRANCH_REFS.get(), cx),
            },
            RefOp::Rebase => match selected {
                Some(RefTarget::Local(n)) => {
                    self.carry_out(rules::rebase_plan(&snap, &n), window, cx)
                }
                _ => say(self, msg::input::SELECT_LOCAL_BRANCH_REBASE.get(), cx),
            },
            RefOp::Rename => match selected {
                Some(RefTarget::Local(n)) => self.start_rename(n, snap, window, cx),
                _ => say(self, msg::input::SELECT_BRANCH_REFS.get(), cx),
            },
            RefOp::SetUpstream => {
                let branch = match selected {
                    Some(RefTarget::Local(n)) => Some(n),
                    _ => snap.head().map(|h| h.0.to_string()),
                };
                match branch {
                    Some(n) => self.start_upstream(n, snap, window, cx),
                    None => say(self, msg::input::SELECT_BRANCH_REFS.get(), cx),
                }
            }
            RefOp::FastForwardToCurrent | RefOp::FastForwardCurrentToThis => match selected {
                Some(t @ (RefTarget::Local(_) | RefTarget::Remote(_))) => {
                    let to_current = op == RefOp::FastForwardToCurrent;
                    if to_current && !matches!(t, RefTarget::Local(_)) {
                        say(self, msg::input::SELECT_BRANCH_REFS.get(), cx);
                    } else {
                        self.carry_out(rules::ff_plan(&snap, &t, to_current), window, cx);
                    }
                }
                _ => say(self, msg::input::SELECT_BRANCH_REFS.get(), cx),
            },
            RefOp::DeleteSelected => match selected {
                Some(RefTarget::Local(n)) => {
                    self.carry_out(rules::delete_branch_plan(&snap, &n), window, cx)
                }
                Some(t @ RefTarget::Remote(_)) => {
                    self.carry_out(rules::delete_remote_plan(&snap, &t, false), window, cx)
                }
                Some(RefTarget::Tag(n)) => self.start_tag_delete(n, snap, window, cx),
                Some(RefTarget::Stash(id)) => self.drop_stash(id, window, cx),
                Some(_) | None => say(self, msg::input::NO_REF_TO_DELETE.get(), cx),
            },
            RefOp::DeleteRemoteBranch | RefOp::DeleteBranchAndRemote => match selected {
                Some(t @ (RefTarget::Local(_) | RefTarget::Remote(_))) => self.carry_out(
                    rules::delete_remote_plan(&snap, &t, op == RefOp::DeleteBranchAndRemote),
                    window,
                    cx,
                ),
                _ => say(self, msg::input::SELECT_BRANCH_REFS.get(), cx),
            },
            RefOp::DeleteTagFromRemote => match selected {
                Some(RefTarget::Tag(n)) => self.start_tag_delete(n, snap, window, cx),
                _ => say(self, msg::input::SELECT_REF_REFS.get(), cx),
            },
            RefOp::PushTag => match selected {
                Some(RefTarget::Tag(n)) => {
                    self.carry_out(rules::push_tag_plan(&snap, &n), window, cx)
                }
                _ => say(self, msg::input::SELECT_REF_REFS.get(), cx),
            },
            RefOp::PushAllTags => self.carry_out(rules::push_all_tags_plan(&snap), window, cx),
            RefOp::AnnotateTag => match selected {
                Some(RefTarget::Tag(n)) => self.start_annotate(n, window, cx),
                _ => say(self, msg::input::SELECT_REF_REFS.get(), cx),
            },
            RefOp::Copy(field) => match selected {
                Some(t) => self.copy_field(field, &t, &snap, cx),
                None => say(self, msg::input::SELECT_REF_REFS.get(), cx),
            },
            RefOp::StashPush => {
                self.backend.stash_push(None);
                self.say(msg::input::STASHING.get(), cx);
            }
            RefOp::StashMessage => self.start_stash(window, cx),
            RefOp::StashPop | RefOp::StashApply | RefOp::StashDrop | RefOp::StashShow => {
                // The selected stash, else the latest one.
                let id = match selected {
                    Some(RefTarget::Stash(id)) => Some(id),
                    _ => snap.stashes.first().map(|s| s.id.0),
                };
                match id {
                    None => say(self, msg::input::NO_STASH_TO_POP.get(), cx),
                    Some(id) => self.run_stash(op, id, &snap, window, cx),
                }
            }
            RefOp::Reveal => match selected {
                Some(t) => {
                    if !self.refs.update(cx, |r, cx| r.reveal(&t, cx)) {
                        say(self, &msg::git_actions::gone(t.name(), ""), cx);
                    }
                }
                None => say(self, msg::input::SELECT_REF_REFS.get(), cx),
            },
            RefOp::ToggleHide | RefOp::ToggleSolo => match selected {
                Some(t) if t.kind().is_some() => {
                    let name = t.name();
                    self.refs.update(cx, |r, cx| {
                        if op == RefOp::ToggleHide {
                            r.toggle_hidden(&name, cx)
                        } else {
                            r.toggle_solo(&name, cx)
                        }
                    });
                }
                _ => say(self, msg::input::SELECT_REF_REFS.get(), cx),
            },
            RefOp::Repo(op) => self.run_repo_op(op, selected, window, cx),
            RefOp::DropOnCurrent => match (selected, snap.head()) {
                (Some(source), Some((head, _))) => {
                    let at = self.menu_origin(cx);
                    self.open_drop_menu(source, RefTarget::Local(head.to_string()), at, window, cx);
                }
                (None, _) => say(self, msg::input::SELECT_REF_REFS.get(), cx),
                (_, None) => say(self, msg::menu_ref::FF_DETACHED.get(), cx),
            },
        }
    }

    /// A double click on a Refs row: a branch or tag is checked out, a stash shows its changes.
    pub(in crate::app) fn open_ref_row(
        &mut self,
        target: RefTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let snap = self.snapshot(cx);
        match target {
            RefTarget::Worktree(_) => {
                self.run_repo_op(RepoOp::WorktreeOpen, Some(target), window, cx)
            }
            RefTarget::Submodule(_) => {
                self.run_repo_op(RepoOp::SubmoduleOpen, Some(target), window, cx)
            }
            RefTarget::Stash(id) => self.show_stash(snap.stash(id).and_then(|s| s.commit), cx),
            t => self.carry_out(rules::checkout_plan(&t, &snap), window, cx),
        }
    }

    /// Submits what a plan says, asking first when it says to.
    pub fn carry_out(&mut self, plan: Plan, window: &mut Window, cx: &mut Context<Self>) {
        match plan {
            Plan::Say(text) => self.say(text, cx),
            Plan::Submit { action, said } => {
                self.backend.submit(action);
                self.say(said, cx);
            }
            Plan::Confirm {
                title,
                lines,
                confirm,
                danger,
                action,
                said,
            } => {
                let spec = Spec {
                    title,
                    lines,
                    choices: vec![
                        if danger {
                            Choice::destructive(confirm)
                        } else {
                            Choice::new(confirm)
                        },
                        Choice::new(msg::dialog::CANCEL.get()),
                    ],
                    hint: msg::diff_actions::CONFIRM_HINT.get().to_string(),
                    // A destructive answer is never the default: a stray Enter answers nothing.
                    highlight_first: !danger,
                    yes_no_keys: true,
                };
                dialogs::open(
                    window,
                    cx,
                    spec,
                    move |ix, cx| {
                        if ix == 0 {
                            with_workspace(cx, move |ws, _, cx| {
                                ws.backend.submit(action);
                                ws.say(said, cx);
                            });
                        }
                    },
                    || {},
                );
            }
        }
    }

    // -- tags and stashes ----------------------------------------------------------------------------

    pub(super) fn start_tag_delete(
        &mut self,
        name: String,
        snap: Snapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match rules::tag_delete_start(&snap, &name) {
            TagDelete::Say(text) => self.say(text, cx),
            TagDelete::Plan(plan) => self.carry_out(*plan, window, cx),
            TagDelete::Ask(rows) => {
                use msg::tag_where as t;
                let on = snap.tag_place(&name).remotes;
                let spec = Spec {
                    title: t::DLG_TITLE.get().to_string(),
                    lines: vec![t::dlg_question(&name)],
                    choices: rows
                        .iter()
                        .map(|r| Choice::destructive(rules::tag_scope_label(r)))
                        .collect(),
                    hint: t::DLG_HINT.get().to_string(),
                    // Nothing is chosen until a key or a click says so.
                    highlight_first: false,
                    yes_no_keys: false,
                };
                dialogs::open(
                    window,
                    cx,
                    spec,
                    move |ix, cx| {
                        let Some(scope) = rows.get(ix).cloned() else {
                            return;
                        };
                        with_workspace(cx, move |ws, window, cx| {
                            let snap = ws.snapshot(cx);
                            let plan = rules::tag_scope_plan(&snap, &name, &scope, &on);
                            ws.carry_out(plan, window, cx);
                        });
                    },
                    || {},
                );
            }
        }
    }

    pub(super) fn drop_stash(&mut self, id: u32, window: &mut Window, cx: &mut Context<Self>) {
        let key = gitpoulpi_app::keymap::current().shown(gitpoulpi_app::keymap::Action::GlobalUndo);
        let plan = Plan::Confirm {
            title: msg::menu_ref::confirm_drop_stash(id, key),
            lines: Vec::new(),
            confirm: msg::menu_ref::STASH_DROP.get().to_string(),
            danger: true,
            action: Action::StashDrop(gitpoulpi_core::domain::StashId(id)),
            said: msg::input::drop_stash(id),
        };
        self.carry_out(plan, window, cx);
    }

    pub(super) fn run_stash(
        &mut self,
        op: RefOp,
        id: u32,
        snap: &Snapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let sid = gitpoulpi_core::domain::StashId(id);
        match op {
            RefOp::StashPop => {
                self.backend.submit(Action::StashPop(sid));
                self.say(msg::menu_ref::popping_stash(id), cx);
            }
            RefOp::StashApply => {
                self.backend.submit(Action::StashApply(sid));
                self.say(msg::menu_ref::applying_stash(id), cx);
            }
            RefOp::StashDrop => self.drop_stash(id, window, cx),
            _ => self.show_stash(snap.stash(id).and_then(|s| s.commit), cx),
        }
    }

    /// Shows the changes of a stash in the diff pane.
    pub(super) fn show_stash(&mut self, commit: Option<Oid>, cx: &mut Context<Self>) {
        let Some(commit) = commit else {
            return self.say(strings::STASH_EMPTY, cx);
        };
        let diff = self.backend.commit_diff(commit);
        cx.spawn(async move |this, cx| {
            let files = diff.await.unwrap_or_default();
            let _ = this.update(cx, |this, cx| {
                if files.is_empty() {
                    return this.say(strings::STASH_EMPTY, cx);
                }
                this.diff.update(cx, |d, cx| {
                    d.show(
                        std::sync::Arc::new(files),
                        0,
                        crate::panels::diff::DiffSource::Commit,
                        cx,
                    )
                });
                this.center = super::Center::Diff;
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn copy_field(
        &mut self,
        field: gitpoulpi_app::palette::vocab::ref_menu::RefField,
        target: &RefTarget,
        snap: &Snapshot,
        cx: &mut Context<Self>,
    ) {
        use gitpoulpi_app::palette::vocab::ref_menu::RefField as F;
        let text = match field {
            F::Name => Some(target.name()),
            F::FullRef => rules::qualified_ref_name(target),
            F::TargetSha => snap.oid_of(target).map(|o| o.to_hex()),
        };
        match text {
            Some(text) => {
                let preview: String = text.chars().take(24).collect();
                cx.write_to_clipboard(ClipboardItem::new_string(text));
                self.say(msg::input::copied(field.label(), preview), cx);
            }
            None => self.say(msg::git_actions::gone(target.name(), ""), cx),
        }
    }
}
