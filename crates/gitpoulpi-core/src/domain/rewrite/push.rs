//! Pushing a rewrite (the history-rewrite design, "Already
//! pushed"): the request, the per-ref answer, the record of what an apply
//! moved, and the note for the people who have the old history.
//!
//! Pure data and text; the push itself is
//! [`GitBackend::push_rewritten`](crate::GitBackend::push_rewritten).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{PushState, RefChange, RunId};
use crate::domain::Oid;

/// `<git-dir>/gitpoulpi/rewrite/pushed.json`: what was pushed, kept after a
/// run's directory is swept, so a later undo of the rewrite can still warn.
pub const PUSHED_FILE: &str = "pushed.json";

/// Protected branch patterns when the config says nothing (`rewrite.protected_branches`).
pub const DEFAULT_PROTECTED_BRANCHES: &[&str] = &["main", "master", "develop", "release/*"];

fn yes() -> bool {
    true
}

/// Push the refs an applied run moved, each with an explicit lease.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushRewritten {
    pub run: RunId,
    /// Only the refs of this remote (default: every remote the run's refs
    /// were on).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
    /// Protected refs to push anyway (short or full names). Protected refs
    /// are left out unless listed here or [`Self::include_all_protected`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_protected: Vec<String>,
    #[serde(default)]
    pub include_all_protected: bool,
    /// Only these refs (short or full names): the ticks of the TUI's push
    /// list. `None` is every ref the run moved that a remote has.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refs: Option<Vec<String>>,
    /// Ask the remote what would happen (`git push --dry-run`): the leases
    /// are checked, nothing is sent.
    #[serde(default)]
    pub dry_run: bool,
    /// Push each remote's refs in one `--atomic` push (default). Servers
    /// that refuse `--atomic` get one push per ref, and the answer says so.
    #[serde(default = "yes")]
    pub atomic: bool,
}

impl PushRewritten {
    pub fn new(run: RunId) -> Self {
        Self {
            run,
            remote: None,
            include_protected: Vec::new(),
            include_all_protected: false,
            refs: None,
            dry_run: false,
            atomic: true,
        }
    }

    /// Whether the push is about the ref `name` (a full ref name): every ref
    /// when no selection was made.
    pub fn selects(&self, name: &str) -> bool {
        let short = name
            .strip_prefix("refs/heads/")
            .or_else(|| name.strip_prefix("refs/tags/"))
            .unwrap_or(name);
        match &self.refs {
            None => true,
            Some(only) => only.iter().any(|n| n == name || n == short),
        }
    }

    /// Whether the user asked for the protected ref `name` (a full ref name).
    pub fn includes(&self, name: &str) -> bool {
        let short = name.strip_prefix("refs/heads/").unwrap_or(name);
        self.include_all_protected
            || self
                .include_protected
                .iter()
                .any(|n| n == name || n == short)
    }
}

/// What happened to one ref.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PushRefStatus {
    Pushed,
    /// A dry run: the remote would take it.
    WouldPush,
    /// The remote already has the new id.
    UpToDate,
    /// The lease failed: the remote ref is not where the preview saw it
    /// (someone pushed since). Fetch and review before trying again.
    RejectedRemoteMoved,
    /// The host refused the force push (a protected branch rule).
    RejectedProtectedByHost,
    /// Any other refusal; the detail has git's words.
    RejectedOther,
    /// Left out on purpose: a protected ref nobody asked to push.
    SkippedProtected,
    /// Not sent: an atomic push failed on another ref, so nothing was
    /// pushed, or the local ref is no longer the rewritten one.
    NotPushed,
}

impl PushRefStatus {
    pub fn is_rejected(self) -> bool {
        matches!(
            self,
            Self::RejectedRemoteMoved | Self::RejectedProtectedByHost | Self::RejectedOther
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushRefResult {
    /// The full ref name, same on both sides.
    pub name: String,
    pub remote: String,
    /// The lease: what the remote had when the preview ran.
    pub expected_old: Oid,
    /// The rewritten id; `None` when the rewrite deleted the ref.
    pub new: Option<Oid>,
    pub status: PushRefStatus,
    /// git's own words for a refusal, when it gave some.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushRewrittenOutcome {
    pub run: RunId,
    pub dry_run: bool,
    pub results: Vec<PushRefResult>,
    /// At least one remote refused `--atomic`, so its refs went one by one.
    #[serde(default)]
    pub atomic_fallback: bool,
}

impl PushRewrittenOutcome {
    pub fn count(&self, status: PushRefStatus) -> usize {
        self.results.iter().filter(|r| r.status == status).count()
    }

    /// Nothing was refused or held back (skipped protected refs do not
    /// count: the user chose that).
    pub fn is_clean(&self) -> bool {
        self.results
            .iter()
            .all(|r| !r.status.is_rejected() && r.status != PushRefStatus::NotPushed)
    }
}

/// One ref of a pushed-record: enough for the undo warning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushedRef {
    pub name: String,
    pub remote: String,
    pub new: Oid,
}

/// What an applied run moved, as `moved.json` in its run directory keeps it,
/// with the directory the maps are in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewriteRecord {
    pub run: RunId,
    pub dir: PathBuf,
    pub refs: Vec<RefChange>,
    /// The refs among `refs` that the remote has under that name (a branch
    /// with a same-named remote-tracking ref, a tag in the tag mirror): the
    /// ones a push sends and a note is about. A branch that only contains
    /// pushed commits is in `refs` but not here.
    pub on_remote: Vec<String>,
}

/// The files `rewrite maps` copied or points at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewriteMaps {
    pub run: RunId,
    pub dir: PathBuf,
    pub commit_map: PathBuf,
    pub ref_map: PathBuf,
    /// The maps were copied to a directory the caller chose.
    pub copied: bool,
}

// ------------------------------------------------------------------- note

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteRefKind {
    Branch,
    Tag,
}

/// One pushed ref that collaborators must re-sync.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteRef {
    /// The full ref name.
    pub name: String,
    /// `main`, `v1.2`.
    pub short: String,
    pub kind: NoteRefKind,
    pub remote: String,
    pub old: Oid,
    pub new: Oid,
}

/// The note for collaborators: structured (the TUI renders it in its own
/// language) and as English text (CLI, MCP, copy).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewriteNote {
    pub run: RunId,
    pub refs: Vec<NoteRef>,
    pub text: String,
}

/// The refs a note is about: moved, not deleted, and on a remote.
pub fn note_refs(record: &RewriteRecord) -> Vec<NoteRef> {
    record
        .refs
        .iter()
        .filter(|r| record.on_remote.contains(&r.name))
        .filter_map(|r| {
            let PushState::OnRemote { remote, .. } = &r.pushed else {
                return None;
            };
            let new = r.new.filter(|n| *n != r.old)?;
            let (kind, short) = if let Some(s) = r.name.strip_prefix("refs/heads/") {
                (NoteRefKind::Branch, s)
            } else {
                (NoteRefKind::Tag, r.name.strip_prefix("refs/tags/")?)
            };
            Some(NoteRef {
                name: r.name.clone(),
                short: short.to_string(),
                kind,
                remote: remote.clone(),
                old: r.old,
                new,
            })
        })
        .collect()
}

fn short(oid: Oid) -> String {
    oid.to_hex()[..12].to_string()
}

/// The English note: what happened, and what everyone who has the old
/// history does, per ref, with the old and new tips.
pub fn note_text(refs: &[NoteRef]) -> String {
    if refs.is_empty() {
        return "No pushed ref was rewritten: nobody else needs to do anything.\n".to_string();
    }
    let mut out = String::from(
        "The history of this repository was rewritten and force-pushed.\n\
         Your clone still has the old history. Do not push your old branches \
         back: that would bring the old commits back.\n\n",
    );
    for r in refs {
        let (old, new) = (short(r.old), short(r.new));
        match r.kind {
            NoteRefKind::Branch => out.push_str(&format!(
                "Branch {b} on {rem}: {old} -> {new}\n  \
                 No local work on it:\n    \
                 git fetch {rem}\n    \
                 git checkout {b}\n    \
                 git reset --hard {rem}/{b}\n  \
                 Local commits on top of the old {b}:\n    \
                 git fetch {rem}\n    \
                 git rebase --onto {rem}/{b} {old} {b}\n\n",
                b = r.short,
                rem = r.remote,
            )),
            NoteRefKind::Tag => out.push_str(&format!(
                "Tag {t} on {rem}: {old} -> {new}\n  \
                 Replace your copy:\n    \
                 git fetch {rem} +refs/tags/{t}:refs/tags/{t}\n\n",
                t = r.short,
                rem = r.remote,
            )),
        }
    }
    out.push_str(
        "Other branches of yours that contain old commits keep them until you \
         rebase them with `git rebase --onto <new base> <old base> <branch>` \
         (the old and new ids above, and the commit map, tell which).\n",
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(name: &str, pushed: PushState) -> RefChange {
        RefChange {
            name: name.into(),
            old: Oid([1; 20]),
            new: Some(Oid([2; 20])),
            pushed,
            protected: false,
            included: true,
            checked_out_in: None,
        }
    }

    fn on_origin() -> PushState {
        PushState::OnRemote {
            remote: "origin".into(),
            remote_oid: Oid([1; 20]),
            rewritten_on_remote: 1,
        }
    }

    #[test]
    fn a_note_lists_only_pushed_branches_and_tags() {
        let record = RewriteRecord {
            run: RunId("r".into()),
            dir: PathBuf::new(),
            refs: vec![
                change("refs/heads/main", on_origin()),
                change("refs/heads/local", PushState::LocalOnly),
                change("refs/heads/odd", on_origin()),
                change("refs/tags/v1", on_origin()),
            ],
            on_remote: vec!["refs/heads/main".into(), "refs/tags/v1".into()],
        };
        let refs = note_refs(&record);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].kind, NoteRefKind::Branch);
        assert_eq!(refs[1].short, "v1");
        let text = note_text(&refs);
        assert!(text.contains("git reset --hard origin/main"), "{text}");
        assert!(
            text.contains("git rebase --onto origin/main 010101010101 main"),
            "{text}"
        );
        assert!(
            text.contains("git fetch origin +refs/tags/v1:refs/tags/v1"),
            "{text}"
        );
        assert!(!text.contains('\u{2014}'));
    }

    #[test]
    fn a_request_defaults_to_atomic_and_asks_for_protected_refs_by_name() {
        let args: PushRewritten = serde_json::from_str(r#"{"run":"r"}"#).unwrap();
        assert!(args.atomic && !args.dry_run && !args.include_all_protected);
        let mut asked = PushRewritten::new(RunId("r".into()));
        asked.include_protected = vec!["main".into()];
        assert!(asked.includes("refs/heads/main"));
        assert!(!asked.includes("refs/heads/develop"));
        asked.include_all_protected = true;
        assert!(asked.includes("refs/heads/develop"));
        assert!(asked.selects("refs/heads/anything"), "no selection: all");
        asked.refs = Some(vec!["main".into(), "refs/tags/v1".into()]);
        assert!(asked.selects("refs/heads/main") && asked.selects("refs/tags/v1"));
        assert!(!asked.selects("refs/heads/dev"));
    }
}
