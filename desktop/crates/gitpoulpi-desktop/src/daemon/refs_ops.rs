//! Branches, tags and stashes as link actions: what each command turns into, the questions that come
//! first and the name rules.
//!
//! The rules are the terminal UI's (`ref_menu/apply.rs`, `git_actions.rs`, `log_inline_ref.rs` and
//! `tag_delete_dialog.rs` keep them private to that crate, so they are restated here, over the shared
//! `gitpoulpi_app::refs_tree` helpers where those exist). Everything here is pure: it reads a
//! [`Snapshot`] of the refs and answers with a [`Plan`], which the window then carries out. One
//! submitted action is one journal entry.

use gitpoulpi_app::refs_tree::{remote_checkout_plan, RefRowKind, RemoteCheckout};
use gitpoulpi_core::domain::{Oid, Ref, RefKind, Stash, TagSelection, TagWhere};
use gitpoulpi_i18n::msg;
use gitpoulpi_link::Action;

use super::Backend;
use crate::ref_target::RefTarget;

mod drop_menu;
mod tags;
pub use drop_menu::*;
pub use tags::*;

/// The refs and stashes a command decides from.
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub refs: Vec<Ref>,
    pub stashes: Vec<Stash>,
}

/// What a command does once decided.
#[derive(Clone, Debug, PartialEq)]
pub enum Plan {
    /// Submit now and say `said` in the status line.
    Submit { action: Action, said: String },
    /// Ask first. `danger` colours the confirming answer; nothing is highlighted before a key is pressed.
    Confirm {
        title: String,
        lines: Vec<String>,
        confirm: String,
        danger: bool,
        action: Action,
        said: String,
    },
    /// Nothing to submit: say why.
    Say(String),
}

impl Plan {
    fn submit(action: Action, said: String) -> Plan {
        Plan::Submit { action, said }
    }
}

/// Where a tag lives and which remotes have it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TagPlace {
    pub local: bool,
    /// Every remote that has the tag, at this commit or another one.
    pub remotes: Vec<String>,
    pub place: Option<TagWhere>,
    /// The remotes that have it at another commit.
    pub differs_on: Vec<String>,
}

/// One answer of the tag delete question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TagScope {
    Local,
    Remote(String),
    Both(String),
}

impl Snapshot {
    pub fn new(refs: Vec<Ref>, stashes: Vec<Stash>) -> Self {
        Self { refs, stashes }
    }

    /// The remotes that have a branch here, sorted.
    pub fn remote_names(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .refs
            .iter()
            .filter_map(|r| match &r.kind {
                RefKind::RemoteBranch { remote, .. } => Some(remote.clone()),
                _ => None,
            })
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// The checked-out branch and its commit.
    pub fn head(&self) -> Option<(&str, Oid)> {
        self.refs.iter().find_map(|r| match (&r.kind, r.is_head) {
            (RefKind::LocalBranch { name }, true) => Some((name.as_str(), r.target)),
            _ => None,
        })
    }

    pub fn find(&self, target: &RefTarget) -> Option<&Ref> {
        let kind = target.kind()?;
        let name = target.name();
        self.refs.iter().find(|r| match (&r.kind, kind) {
            (RefKind::LocalBranch { name: n }, RefRowKind::Local) => *n == name,
            (RefKind::RemoteBranch { remote, name: n }, RefRowKind::Remote) => {
                format!("{remote}/{n}") == name
            }
            (RefKind::Tag { name: n }, RefRowKind::Tag) => *n == name,
            _ => false,
        })
    }

    /// The commit a ref or stash points at.
    pub fn oid_of(&self, target: &RefTarget) -> Option<Oid> {
        match target {
            RefTarget::Stash(id) => self.stash(*id).and_then(|s| s.commit),
            other => self.find(other).map(|r| r.target),
        }
    }

    pub fn stash(&self, id: u32) -> Option<&Stash> {
        self.stashes.iter().find(|s| s.id.0 == id)
    }

    pub fn tag_place(&self, name: &str) -> TagPlace {
        let Some(r) = self.find(&RefTarget::Tag(name.to_string())) else {
            return TagPlace::default();
        };
        let sync = r.tag_sync.clone().unwrap_or_default();
        let mut remotes = sync.pushed_to.clone();
        remotes.extend(sync.differs_on.iter().cloned());
        remotes.sort();
        remotes.dedup();
        TagPlace {
            local: sync.local,
            place: r.tag_where(),
            remotes,
            differs_on: sync.differs_on,
        }
    }
}

// -- names ------------------------------------------------------------------------------------------

/// A new branch or a new tag.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewRef {
    Branch,
    Tag,
}

/// `git check-ref-format`'s rules, as far as a short human message goes.
fn check_ref_format(name: &str) -> Result<(), String> {
    use msg::menu_inline as m;
    if name == "@" {
        return Err(m::NAME_AT.get().into());
    }
    if name.starts_with('/') || name.ends_with('/') || name.contains("//") {
        return Err(m::NAME_SLASH.get().into());
    }
    if name.ends_with('.') {
        return Err(m::NAME_DOT_END.get().into());
    }
    if name.contains("..") {
        return Err(m::NAME_DOTDOT.get().into());
    }
    if name.contains("@{") {
        return Err(m::NAME_AT_BRACE.get().into());
    }
    if name.starts_with('-') {
        return Err(m::NAME_DASH_START.get().into());
    }
    if let Some(bad) = name
        .chars()
        .find(|c| c.is_whitespace() || c.is_control() || "~^:?*[\\".contains(*c))
    {
        return Err(if bad.is_whitespace() {
            m::NAME_SPACES.get().to_string()
        } else {
            m::name_bad_char(bad.escape_default())
        });
    }
    for part in name.split('/') {
        if part.starts_with('.') {
            return Err(m::NAME_PART_DOT.get().into());
        }
        if part.ends_with(".lock") {
            return Err(m::NAME_PART_LOCK.get().into());
        }
    }
    Ok(())
}

/// The trimmed name if it can be created, else the message to show: git's format rules, an existing ref
/// of the same kind, and a directory/file clash (`a` next to `a/b`).
pub fn validate_new_ref(kind: NewRef, raw: &str, refs: &[Ref]) -> Result<String, String> {
    use msg::menu_inline as m;
    let name = raw.trim();
    if name.is_empty() {
        return Err(match kind {
            NewRef::Branch => m::ENTER_BRANCH.get().to_string(),
            NewRef::Tag => m::ENTER_TAG.get().to_string(),
        });
    }
    check_ref_format(name)?;
    let existing = refs.iter().filter_map(|r| match (&r.kind, kind) {
        (RefKind::LocalBranch { name }, NewRef::Branch) => Some(name.as_str()),
        (RefKind::Tag { name }, NewRef::Tag) => Some(name.as_str()),
        _ => None,
    });
    for other in existing {
        if other == name {
            return Err(match kind {
                NewRef::Branch => m::exists_branch(name),
                NewRef::Tag => m::exists_tag(name),
            });
        }
        if other.starts_with(&format!("{name}/")) || name.starts_with(&format!("{other}/")) {
            return Err(m::clashes(name, other));
        }
    }
    Ok(name.to_string())
}

/// [`validate_new_ref`] for a rename: the branch's own old name is not taken by itself.
pub fn validate_rename(old: &str, raw: &str, refs: &[Ref]) -> Result<String, String> {
    let others: Vec<Ref> = refs
        .iter()
        .filter(|r| !matches!(&r.kind, RefKind::LocalBranch { name } if name == old))
        .cloned()
        .collect();
    validate_new_ref(NewRef::Branch, raw, &others)
}

/// The upstream typed in the Set upstream form: empty unsets it, otherwise it must be a remote-tracking
/// branch the repository has (`origin/main`).
pub fn validate_upstream(raw: &str, refs: &[Ref]) -> Result<Option<String>, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(None);
    }
    let known = refs.iter().any(|r| match &r.kind {
        RefKind::RemoteBranch { remote, name } => format!("{remote}/{name}") == raw,
        _ => false,
    });
    if known {
        Ok(Some(raw.to_string()))
    } else {
        Err(msg::git_actions::no_remote_branch(raw))
    }
}

// -- checkout ---------------------------------------------------------------------------------------

/// Checkout of a row: a local branch or a tag directly; a remote branch through the local branch that
/// tracks it, or an offer to create one (never a raw checkout of the remote ref, which would detach).
pub fn checkout_plan(target: &RefTarget, snap: &Snapshot) -> Plan {
    match target {
        RefTarget::Local(name) => Plan::submit(
            Action::CheckoutBranch(name.clone()),
            msg::input::checking_out(name),
        ),
        RefTarget::Tag(name) => {
            if snap.tag_place(name).place == Some(TagWhere::Remote) {
                let place = snap.tag_place(name);
                return Plan::Say(msg::tag_where::msg_not_local(
                    name,
                    place.remotes.join(", "),
                ));
            }
            Plan::submit(
                Action::CheckoutBranch(name.clone()),
                msg::input::checking_out(name),
            )
        }
        RefTarget::Remote(name) => {
            match remote_checkout_plan(name, &snap.refs, &snap.remote_names()) {
                RemoteCheckout::CheckoutLocal { local } => {
                    let said = msg::git_actions::checking_out_tracks(&local, name);
                    Plan::submit(Action::CheckoutBranch(local), said)
                }
                RemoteCheckout::AlreadyOnIt { local } => {
                    Plan::Say(msg::git_actions::already_on(&local, name))
                }
                RemoteCheckout::OfferCreate { local, upstream } => Plan::Confirm {
                    title: msg::git_actions::offer_create(&upstream, &local),
                    lines: Vec::new(),
                    confirm: msg::confirm_button::CREATE_BRANCH.get().to_string(),
                    danger: false,
                    said: msg::git_actions::creating_tracking(&local, &upstream),
                    action: Action::BranchCreateTracking {
                        local_name: local,
                        upstream,
                    },
                },
                RemoteCheckout::NameTaken { local, tracks } => Plan::Say(match tracks {
                    Some(other) => msg::git_actions::name_taken_other(&local, other, name),
                    None => msg::git_actions::name_taken_none(&local),
                }),
            }
        }
        RefTarget::Stash(_) => Plan::Say(msg::input::SELECT_REF_REFS.get().to_string()),
    }
}

/// Detach HEAD onto a commit.
pub fn checkout_commit(at: Oid) -> Plan {
    Plan::submit(
        Action::CheckoutCommit(at),
        msg::input::checking_out(gitpoulpi_core::domain::short(at)),
    )
}

// -- integrate --------------------------------------------------------------------------------------

fn local_ref<'a>(snap: &'a Snapshot, name: &str) -> Option<&'a Ref> {
    snap.find(&RefTarget::Local(name.to_string()))
}

/// Merge the local branch `name` into the checked-out one.
pub fn merge_plan(snap: &Snapshot, name: &str) -> Plan {
    match local_ref(snap, name) {
        Some(from) => Plan::submit(Action::Merge(from.clone()), msg::menu_ref::merging(name)),
        None => Plan::Say(msg::git_actions::NO_SUCH_LOCAL_BRANCH.get().to_string()),
    }
}

/// Rebase the checked-out branch onto the local branch `name`.
pub fn rebase_plan(snap: &Snapshot, name: &str) -> Plan {
    match local_ref(snap, name) {
        Some(onto) => Plan::submit(
            Action::Rebase(onto.clone()),
            msg::menu_ref::rebasing_onto(name),
        ),
        None => Plan::Say(msg::git_actions::NO_SUCH_LOCAL_BRANCH.get().to_string()),
    }
}

/// Why a fast-forward row is grayed out, from what the loaded refs already say.
pub fn ff_refusal(moving: &str, moving_at: Oid, to_at: Oid) -> Option<String> {
    let _ = moving;
    (moving_at == to_at).then(|| msg::menu_ref::FF_ALREADY_THERE.get().to_string())
}

/// Fast-forward `target` (a local branch, not the checked-out one) to the checked-out branch
/// (`to_current`), or the checked-out branch to `target` (a local or remote branch).
pub fn ff_plan(snap: &Snapshot, target: &RefTarget, to_current: bool) -> Plan {
    let Some((head_name, head_at)) = snap.head() else {
        return Plan::Say(msg::menu_ref::FF_DETACHED.get().to_string());
    };
    let name = target.name();
    let Some(this_at) = snap.oid_of(target) else {
        return Plan::Say(msg::git_actions::gone(&name, ""));
    };
    let (branch, to_at, label) = if to_current {
        (name.clone(), head_at, head_name.to_string())
    } else {
        (head_name.to_string(), this_at, name.clone())
    };
    let moving_at = if to_current { this_at } else { head_at };
    if let Some(why) = ff_refusal(&branch, moving_at, to_at) {
        return Plan::Say(why);
    }
    let said = msg::menu_ref::fast_forwarding(&branch, &label);
    Plan::submit(
        Action::FastForward {
            branch,
            target: to_at,
            target_label: label,
        },
        said,
    )
}

// -- delete -----------------------------------------------------------------------------------------

/// What a Delete row acts on: the local branch and/or the remote one (`(remote, branch)`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeleteTargets {
    pub local: Option<String>,
    pub remote: Option<(String, String)>,
}

fn remote_parts(snap: &Snapshot, display: &str) -> Option<(String, String)> {
    snap.refs.iter().find_map(|r| match &r.kind {
        RefKind::RemoteBranch { remote, name } if format!("{remote}/{name}") == display => {
            Some((remote.clone(), name.clone()))
        }
        _ => None,
    })
}

/// The branch's counterpart on the other side: for a local branch its upstream (when loaded) or a remote
/// branch of the same name; for a remote branch the local branch tracking it, else one of the same name.
pub fn delete_targets(snap: &Snapshot, target: &RefTarget) -> DeleteTargets {
    match target {
        RefTarget::Local(name) => {
            let upstream = snap.refs.iter().find_map(|r| match &r.kind {
                RefKind::LocalBranch { name: n } if n == name => r.upstream.clone(),
                _ => None,
            });
            let twin = upstream
                .filter(|u| remote_parts(snap, u).is_some())
                .or_else(|| {
                    snap.refs.iter().find_map(|r| match &r.kind {
                        RefKind::RemoteBranch { remote, name: n } if n == name && n != "HEAD" => {
                            Some(format!("{remote}/{n}"))
                        }
                        _ => None,
                    })
                });
            DeleteTargets {
                local: Some(name.clone()),
                remote: twin.and_then(|t| remote_parts(snap, &t)),
            }
        }
        RefTarget::Remote(full) => {
            let Some((_, branch)) = remote_parts(snap, full) else {
                return DeleteTargets::default();
            };
            let local = snap
                .refs
                .iter()
                .find_map(|r| match &r.kind {
                    RefKind::LocalBranch { name } if r.upstream.as_deref() == Some(full) => {
                        Some(name.clone())
                    }
                    _ => None,
                })
                .or_else(|| {
                    snap.refs.iter().find_map(|r| match &r.kind {
                        RefKind::LocalBranch { name } if *name == branch => Some(name.clone()),
                        _ => None,
                    })
                });
            DeleteTargets {
                local,
                remote: remote_parts(snap, full),
            }
        }
        _ => DeleteTargets::default(),
    }
}

/// The refusal for deleting the checked-out branch.
pub fn checked_out_refusal(snap: &Snapshot, branch: &str) -> Option<String> {
    (snap.head().map(|h| h.0) == Some(branch))
        .then(|| msg::menu_ref::cannot_delete_checked_out(branch))
}

fn confirm(title: String, confirm: String, action: Action, said: String) -> Plan {
    Plan::Confirm {
        title,
        lines: Vec::new(),
        confirm,
        danger: true,
        action,
        said,
    }
}

fn delete_label() -> String {
    msg::confirm_button::DELETE.get().to_string()
}

/// Delete a local branch: refused on the checked-out one (said, not hidden), asked about otherwise.
pub fn delete_branch_plan(snap: &Snapshot, name: &str) -> Plan {
    if let Some(why) = checked_out_refusal(snap, name) {
        return Plan::Say(why);
    }
    confirm(
        msg::menu_ref::confirm_delete_branch(name),
        delete_label(),
        Action::BranchDelete(name.to_string()),
        msg::input::delete_branch(name),
    )
}

/// Delete the remote twin of a branch, and with `both` the local branch as well.
pub fn delete_remote_plan(snap: &Snapshot, target: &RefTarget, both: bool) -> Plan {
    let name = target.name();
    let t = delete_targets(snap, target);
    match (t.local, t.remote) {
        (_, None) => Plan::Say(msg::menu_ref::no_remote_branch(&name)),
        (None, Some(_)) if both => Plan::Say(msg::menu_ref::no_local_branch(&name)),
        (local, Some((remote, branch))) => {
            if let Some(why) = both
                .then(|| local.as_deref().and_then(|l| checked_out_refusal(snap, l)))
                .flatten()
            {
                return Plan::Say(why);
            }
            let and_local = both.then_some(local).flatten();
            let prompt = match &and_local {
                Some(l) => msg::menu_ref::confirm_delete_both(l, &remote, &branch),
                None => msg::menu_ref::confirm_delete_remote(&branch, &remote),
            };
            let said = msg::menu_ref::fetching(&remote);
            confirm(
                prompt,
                delete_label(),
                Action::BranchDeleteRemote {
                    remote,
                    name: branch,
                    and_local,
                },
                said,
            )
        }
    }
}

/// The rows of the tag delete question, as the terminal UI orders them: local first, then per remote its
/// remote-only and its both-places delete. A tag with no local ref has no local rows.
pub fn tag_scopes(has_local: bool, remotes: &[String]) -> Vec<TagScope> {
    let mut out = Vec::new();
    if has_local {
        out.push(TagScope::Local);
    }
    for r in remotes {
        out.push(TagScope::Remote(r.clone()));
        if has_local {
            out.push(TagScope::Both(r.clone()));
        }
    }
    out
}

/// What deleting a tag asks. A tag only here is deleted after a plain confirm; a tag on one remote and
/// nowhere local goes straight to that remote's confirm; every other case asks where first (`Err`
/// carries the rows). `want_remote` (the palette's "delete from remote") starts on the remote row.
pub enum TagDelete {
    Say(String),
    Plan(Box<Plan>),
    /// Ask where: the rows, and the one to highlight first.
    Ask(Vec<TagScope>),
}

pub fn tag_delete_start(snap: &Snapshot, name: &str) -> TagDelete {
    let place = snap.tag_place(name);
    match (place.local, place.remotes.as_slice()) {
        (true, []) => TagDelete::Plan(Box::new(tag_scope_plan(snap, name, &TagScope::Local, &[]))),
        (false, []) => TagDelete::Say(msg::menu_ref::no_remote_has(name)),
        (false, [only]) => TagDelete::Plan(Box::new(tag_scope_plan(
            snap,
            name,
            &TagScope::Remote(only.clone()),
            &place.remotes,
        ))),
        _ => TagDelete::Ask(tag_scopes(place.local, &place.remotes)),
    }
}

/// The plan of one scope answer. Local never asks when no remote has the tag; with a remote involved it
/// goes through a confirm naming the remote (outward-facing, not undoable on the remote side).
pub fn tag_scope_plan(_snap: &Snapshot, name: &str, scope: &TagScope, on: &[String]) -> Plan {
    use msg::tag_where as t;
    match scope {
        TagScope::Local if on.is_empty() => confirm(
            msg::menu_ref::confirm_delete_tag(name),
            delete_label(),
            Action::TagDelete(name.to_string()),
            msg::input::delete_tag(name),
        ),
        // Runs at once: it loses no work (Undo restores it, a fetch can bring it back).
        TagScope::Local => Plan::submit(
            Action::TagDelete(name.to_string()),
            t::msg_deleted_local_still(name, on.join(", ")),
        ),
        TagScope::Remote(remote) => confirm(
            t::confirm_remote(name, remote),
            delete_label(),
            Action::TagDeleteRemote {
                remote: remote.clone(),
                name: name.to_string(),
            },
            t::msg_deleted_remote(name, remote),
        ),
        TagScope::Both(remote) => confirm(
            t::confirm_both(name, remote),
            delete_label(),
            Action::TagDeleteBoth {
                remote: remote.clone(),
                name: name.to_string(),
            },
            t::msg_deleted_both(name, remote),
        ),
    }
}

/// The label of one answer of the tag delete question.
pub fn tag_scope_label(scope: &TagScope) -> String {
    use msg::tag_where as t;
    match scope {
        TagScope::Local => t::OPT_LOCAL.get().to_string(),
        TagScope::Remote(r) => t::opt_remote(r),
        TagScope::Both(r) => t::opt_both(r),
    }
}

// -- the calls --------------------------------------------------------------------------------------

impl Backend {
    /// Submits a plan's action (the caller has asked first when the plan said to).
    pub fn run_plan_action(&self, action: Action) {
        self.submit(action);
    }

    /// Creates the branch `name` at `at` and checks it out, as the terminal UI does (the link action
    /// creates, then checks out).
    pub fn create_branch_and_checkout(&self, name: String, at: Oid) {
        self.submit(Action::BranchCreate { name, at });
    }

    /// Creates the branch `name` at `at` and leaves HEAD where it is: the daemon operation itself, which
    /// the daemon journals like any other (Undo removes the branch).
    pub fn create_branch_only(
        &self,
        name: &str,
        at: Oid,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send + 'static {
        let call = self.call::<serde_json::Value>(
            gitpoulpi_core::domain::OperationKind::BranchCreate,
            serde_json::json!({ "name": name, "at": at }),
        );
        async move { call.await.map(|_| ()) }
    }

    /// Creates a tag: lightweight, or annotated when `message` is given.
    pub fn create_tag(&self, name: String, at: Oid, message: Option<String>) {
        self.submit(match message {
            Some(message) => Action::TagCreateAnnotated { name, at, message },
            None => Action::TagCreate { name, at },
        });
    }

    /// Stashes the working changes (untracked files included, as the terminal UI does), with a message
    /// when one is given.
    pub fn stash_push(&self, message: Option<String>) {
        self.submit(match message {
            Some(message) => Action::StashPushWith { message },
            None => Action::StashPush,
        });
    }

    /// Sets (or with `None` unsets) a branch's upstream.
    pub fn set_upstream(&self, branch: String, upstream: Option<String>) {
        self.submit(Action::BranchSetUpstream { branch, upstream });
    }
}

#[cfg(test)]
mod tests;
