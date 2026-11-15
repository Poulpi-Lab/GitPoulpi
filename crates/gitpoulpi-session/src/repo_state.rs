use std::collections::BTreeMap;
use std::path::Path;

use gitpoulpi_config::{AiActionSettings, AiConfig};
use gitpoulpi_core::domain::{Oid, OperationKind, RefVisibility, StashId};
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingAutoStashState {
    Applied,
    /// The pop left unmerged paths: the changes are in the working tree.
    Conflicted,
    /// The pop failed and applied nothing (stale lock, "would be
    /// overwritten", an untracked file in the way): the stash still holds
    /// the only copy of the changes.
    PopFailed,
}

/// Crash-safe record of an in-flight auto-stash (§5): written before the
/// wrapped operation runs, cleared on a clean pop, left in place (with
/// state = Conflicted) if the pop conflicts - never silently retried.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAutoStash {
    /// The stash's current index (`stash@{n}`) as last reconciled by the
    /// daemon. An index shifts when another stash is pushed or dropped, so
    /// it is only a cache of [`Self::stash_commit`], re-resolved before
    /// every pop/drop/show.
    pub stash_id: StashId,
    /// The stash's own commit: the stable identity of the record. `None`
    /// only in a record written before this field existed; the daemon
    /// resolves it once from the index and stores it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stash_commit: Option<Oid>,
    pub reason: OperationKind,
    pub created_at: i64,
    pub state: PendingAutoStashState,
    /// Why the last pop failed (`PopFailed` only), git's own words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The operation the stash was made for stopped half-way (a conflicted
    /// merge or rebase): the stash is popped once it has ended, however it
    /// ended. Set only after the operation's re-run, never while it runs.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub after_operation: bool,
}

/// The repo scope of the Preferences screen: settings that belong to
/// *this* repository rather than to the user.
///
/// They live here, beside the ref filters and the pending auto-stash,
/// rather than in a second store, so per-repo state has one home
/// (`.gitpoulpi/repo-state.toml`).
///
/// Every field is an `Option`, and `None` means *inherit* - from
/// `[git]` in `config.toml`, and beyond that from git's own
/// configuration. That distinction is the whole point: a repo that has
/// never been touched in this screen must not start overriding the
/// user's global `core.autocrlf` with a value nobody chose.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RepoPreferences {
    /// Encoding used to decode file contents for display, e.g. `utf-8`.
    pub encoding: Option<String>,
    /// `core.autocrlf`: `true` / `input` / `false`.
    pub autocrlf: Option<String>,
    /// `core.eol`: `native` / `lf` / `crlf`.
    pub eol: Option<String>,
    /// Whether Git LFS is handled for this repository.
    pub lfs_enabled: Option<bool>,
    /// Commit page: push the current branch to its upstream after every
    /// successful commit. `None` is off.
    pub push_after_commit: Option<bool>,
    /// Commit page: the commit panel's skip-hooks option starts checked.
    /// `None` is off.
    pub skip_hooks_by_default: Option<bool>,
    /// Merge options: a local branch merge squashes (stages the changes, no
    /// merge commit). `None` is off.
    pub squash_merge: Option<bool>,
    /// Merge options: whether a merge may fast-forward here, `never` /
    /// `allowed` / `only`. `None` follows `git.merge_fast_forward`.
    pub merge_fast_forward: Option<String>,
    /// Message template: an empty commit panel is pre-filled with it.
    pub template_apply: Option<bool>,
    /// Message template: comment lines (`core.commentChar`) are removed from
    /// the message when it is committed.
    pub template_strip_comments: Option<bool>,
    /// Conflict prevention page: proactive conflict detection for this
    /// repository. `None` follows `general.conflict_detection` (on).
    pub conflict_detection: Option<bool>,
    /// Conflict prevention page: the "Base branches" globs, one per line
    /// (`**` every branch, `origin/main`, `origin/feature/**`, `!origin/abc`).
    /// `None` is not set (see `conflict_base_source`).
    pub conflict_base_branches: Option<String>,
    /// Conflict prevention page: where the Base branches come from,
    /// `default` (the global default list), `list` (`conflict_base_list`) or
    /// `own` (`conflict_base_branches`). `None` (a file written before the
    /// choice existed): own patterns when set, else the default. Resolved by
    /// `gitpoulpi_config::ConflictPreventionConfig::resolve`.
    pub conflict_base_source: Option<String>,
    /// The named list (`[conflict_prevention.lists.<name>]`) this repository
    /// follows when `conflict_base_source` is `list`.
    pub conflict_base_list: Option<String>,
    /// Git page: remotes a new branch also pushes to, besides its upstream
    /// ("Extra push targets for new branches"). Copied into the branch's own
    /// `gitpoulpi-push-to` keys when it first gets an upstream; the branch
    /// keys stay the only thing a push reads.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub default_push_targets: Vec<String>,
    /// The commit identity profile (`[[profiles]]` id) this repository uses;
    /// `None` inherits `general.active_profile`. Chosen from the toolbar's
    /// profile combobox, so it is never mirrored into the Settings screen.
    pub profile: Option<String>,
    /// Per-action AI override: the action's snake_case key
    /// (`commit_message`) to the profile and/or instructions this
    /// repository wants instead of the global ones. An empty field, or an
    /// action absent here, inherits `config.toml`'s `[ai]`.
    pub ai_overrides: std::collections::BTreeMap<String, AiActionSettings>,
    /// The repository's instructions for every AI action and how they
    /// combine with the global ones (`add` after them, `replace` instead).
    #[serde(skip_serializing_if = "gitpoulpi_config::RepoAiInstructions::is_default")]
    pub ai_instructions: gitpoulpi_config::RepoAiInstructions,
    /// This repository's own `[ai.review]` values (CLAUDE.md / AGENTS.md /
    /// extra file / run in the repository); `None` fields inherit the global
    /// ones. A repository that is not cloned keeps the same struct in
    /// `remote-reviews.toml`.
    #[serde(skip_serializing_if = "gitpoulpi_config::RepoAiReview::is_empty")]
    pub ai_review: gitpoulpi_config::RepoAiReview,
    /// Read-only legacy key: the preset id chosen per action before the
    /// 2026-10-01 AI redesign. [`Self::migrate_ai`] turns it into
    /// `ai_overrides`; it is never written back.
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub ai_presets: std::collections::BTreeMap<String, String>,
}

impl RepoPreferences {
    /// Whether anything here has been chosen at all - used to render
    /// "inherited" rather than a value the user never picked.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// This repository's AI override for an action, if it has a
    /// non-empty one. An old preset choice that [`Self::migrate_ai`] has
    /// not converted yet is read through `ai`'s `legacy_presets` archive,
    /// so a repository never loses its choice by being read before saved.
    pub fn ai_override(&self, action_key: &str, ai: &AiConfig) -> Option<AiActionSettings> {
        if let Some(o) = self.ai_overrides.get(action_key).filter(|o| !o.is_empty()) {
            return Some(o.clone());
        }
        let id = self.ai_presets.get(action_key)?;
        let preset = ai.legacy_preset(id)?;
        let o = AiActionSettings::from_preset(preset);
        (!o.is_empty()).then_some(o)
    }

    /// Converts the legacy `ai_presets` choices into `ai_overrides`
    /// (a choice naming no archived preset - a built-in, or one since
    /// deleted - carries nothing and is dropped; an override that already
    /// exists wins). Returns whether anything changed, i.e. whether the
    /// repo state needs saving.
    pub fn migrate_ai(&mut self, ai: &AiConfig) -> bool {
        if self.ai_presets.is_empty() {
            return false;
        }
        for (key, id) in std::mem::take(&mut self.ai_presets) {
            if self.ai_overrides.contains_key(&key) {
                continue;
            }
            if let Some(preset) = ai.legacy_preset(&id) {
                let o = AiActionSettings::from_preset(preset);
                if !o.is_empty() {
                    self.ai_overrides.insert(key, o);
                }
            }
        }
        true
    }
}

/// Which foldable panes the user has collapsed in this repository's
/// window, so a tab switch (which rebuilds the window) or a restart
/// brings them back as they were left. Titles/keys rather than enums:
/// this crate knows nothing about the TUI's sections, and an unknown key
/// is simply ignored on load.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FoldState {
    /// Collapsed left-sidebar sections, by title (`PULL REQUESTS`, ...).
    pub sidebar_sections: Vec<String>,
    /// Collapsed right-pane sections (`unstaged`, `staged`).
    pub status_sections: Vec<String>,
    /// Collapsed PULL REQUESTS groups. `None` means never chosen, so the
    /// built-in default (only "All" folded) applies; `Some(vec![])` is
    /// "the user expanded everything".
    pub pull_request_groups: Option<Vec<String>>,
    /// Collapsed folders (`src/ui`) of the Unstaged/Staged file trees,
    /// oldest first, bounded by the TUI.
    pub status_folders: Vec<String>,
    /// Collapsed folders of the commit-detail changed-files tree. Per
    /// repository, not per commit: applied to whichever commit is shown.
    pub commit_folders: Vec<String>,
}

/// Per-repo state, gitignored under `.gitpoulpi/repo-state.toml` (§5),
/// shared across named sessions since it belongs to the repo, not a session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoState {
    #[serde(default)]
    pub ref_visibility: BTreeMap<String, RefVisibility>,
    /// Remotes hidden from the graph as a whole, sorted: every branch of
    /// the remote, the ones fetched later too. A separate field and not an
    /// entry of `ref_visibility`, so a branch that happens to be named like
    /// the remote cannot collide with it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hidden_remotes: Vec<String>,
    #[serde(default)]
    pub pending_auto_stash: Option<PendingAutoStash>,
    #[serde(default)]
    pub preferences: RepoPreferences,
    /// Collapsed/expanded state of the foldable panes.
    #[serde(default)]
    pub folds: FoldState,
    /// The left-pane sections this repository hides (stable lowercase
    /// names: `tags`, `pull_requests`, ...). `None` means no choice of its
    /// own: the global `[ui] sidebar_sections_hidden` default applies;
    /// `Some(vec![])` is "show everything here".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sidebar_sections_hidden: Option<Vec<String>>,
    /// Which folders of the Refs sidebar's LOCAL section the user has
    /// collapsed, by folder path (`feature`, `test/by`). Per repository,
    /// because a folder only exists in the repository whose branches
    /// make it - collapsing `feature` in one project says nothing about
    /// another.
    #[serde(default)]
    pub collapsed_local_ref_folders: Vec<String>,
    /// The same for the REMOTE section, whose paths start with the
    /// remote's own name (`origin`, `origin/feature`). Kept as a
    /// separate list rather than one list of `(section, path)` pairs so
    /// the file stays readable by hand and a LOCAL folder and a REMOTE
    /// folder of the same name can never be confused for each other.
    #[serde(default)]
    pub collapsed_remote_ref_folders: Vec<String>,
    /// The same for the TAGS section (`releases`, `backups`).
    #[serde(default)]
    pub collapsed_tag_ref_folders: Vec<String>,
    /// The remote this repository's one-click network actions target
    /// (toolbar Push/Fetch, tag push, ...), as chosen with "Set as
    /// default remote". `None` means "not chosen": see
    /// [`resolve_default_remote`] for what is used then.
    #[serde(default)]
    pub default_remote: Option<String>,
    /// The `[[integrations]]` profile id pinned for this repository
    /// ("Use integration profile for this repo"). `None` means resolve by
    /// remote host; a stale id (profile since removed) is ignored by
    /// `gitpoulpi_hosting::resolve_for_repo`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hosting_integration: Option<String>,
    /// Assignment is blocked for this repository's issues and pull requests:
    /// nothing can be assigned from GitPoulpi (every surface refuses), the
    /// existing assignees stay visible. For a project whose maintainers must
    /// not be named by a bot-driven workflow.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub assignment_blocked: bool,
    /// Where this repository's ISSUES section reads from: the provider
    /// profile and that provider's own repository/project, both chosen in
    /// the section. `None` means unlinked: nothing is fetched, and
    /// nothing ever links it on its own (the remotes only *suggest*).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issues_source: Option<IssuesSource>,
    /// The same choice for the PULL REQUESTS section: the provider profile
    /// and that provider's repository. `None` means unlinked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pulls_source: Option<IssuesSource>,
    /// The in-app code review's local state per request (pending comments,
    /// viewed files), keyed by `ReviewDraft::key`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub reviews: BTreeMap<String, crate::ReviewDraft>,
    /// The labels that are the columns of this repository's issue board on a
    /// host with no workflow (`status::todo`, `status::doing`), in board
    /// order. Empty: Open / Closed only.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub board_labels: Vec<String>,
    /// This user's tweaks to the branching workflow (`[workflow]`), layered
    /// over the shared `.gitpoulpi/workflow.toml`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow: Option<gitpoulpi_core::workflow::WorkflowOverride>,
    /// Issues made by a host-driven flow whose branch or merge request has
    /// not been picked up yet; every fetch looks for them
    /// (`gitpoulpi_core::workflow::branches_for_issue`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub watched_issues: Vec<WatchedIssue>,
    /// External history rewrite scripts (shell, Python, Lua) the user trusted
    /// in this repository, by content hash: a script that comes from the
    /// repository or a shared recipe runs only when its hash is here, so a
    /// changed script asks again. Only a human can add to it (the TUI's
    /// question, `rewrite script trust`); the MCP server cannot.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trusted_scripts: Vec<TrustedScript>,
    /// The committed workflow hooks (`.gitpoulpi/workflow.toml`'s `[hooks]`)
    /// the user trusted, by `gitpoulpi_core::workflow::hooks_hash`: hooks run
    /// only while the file's hash is here, so a changed file asks again. The
    /// commands are kept to show "what changed". Only a human adds to it
    /// (`FlowTrustHooks`; the MCP server cannot). Capped at 50.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trusted_hooks: Vec<TrustedHooks>,
}

/// One hook command of a trust record.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HookLine {
    /// `after_start` or `before_finish`.
    pub phase: String,
    pub command: String,
}

/// A list of committed workflow hooks the user trusted (see
/// `RepoState::trusted_hooks`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TrustedHooks {
    /// `hooks_hash` of the list.
    pub hash: String,
    pub hooks: Vec<HookLine>,
    /// Unix seconds.
    pub trusted_at: i64,
}

/// A script the user trusted (see `RepoState::trusted_scripts`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TrustedScript {
    /// `gitpoulpi_core::domain::rewrite::script_hash` of its text.
    pub hash: String,
    /// What it was called when trusted (a path or a name), for humans.
    pub label: String,
    /// Unix seconds.
    pub trusted_at: i64,
}

/// An issue a host-driven flow is waiting on (see `RepoState::watched_issues`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct WatchedIssue {
    pub number: u64,
    /// The topic type the issue was made for (`feature`).
    pub type_id: String,
    pub title: String,
    /// Unix seconds.
    pub created: i64,
}

impl RepoState {
    /// Trusts the script whose hash is `hash` (a second trust of the same
    /// hash only refreshes its label); the list is capped at 200 entries,
    /// forgetting the oldest.
    pub fn trust_script(&mut self, hash: &str, label: &str, now: i64) {
        self.trusted_scripts.retain(|t| t.hash != hash);
        self.trusted_scripts.push(TrustedScript {
            hash: hash.to_string(),
            label: label.to_string(),
            trusted_at: now,
        });
        let extra = self.trusted_scripts.len().saturating_sub(200);
        self.trusted_scripts.drain(..extra);
    }

    /// Trusts the committed hooks `hooks` whose hash is `hash` (trusting the
    /// same hash again only refreshes it); the list is capped at 50 entries,
    /// forgetting the oldest.
    pub fn trust_hooks(&mut self, hash: &str, hooks: Vec<HookLine>, now: i64) {
        self.trusted_hooks.retain(|t| t.hash != hash);
        self.trusted_hooks.push(TrustedHooks {
            hash: hash.to_string(),
            hooks,
            trusted_at: now,
        });
        let extra = self.trusted_hooks.len().saturating_sub(50);
        self.trusted_hooks.drain(..extra);
    }

    /// Whether the committed hooks with this hash were trusted.
    pub fn trusts_hooks(&self, hash: &str) -> bool {
        self.trusted_hooks.iter().any(|t| t.hash == hash)
    }

    /// Whether `command` of `phase` is in any trusted list (what a resumed
    /// flow checks per step).
    pub fn trusts_hook_command(&self, phase: &str, command: &str) -> bool {
        self.trusted_hooks.iter().any(|t| {
            t.hooks
                .iter()
                .any(|h| h.phase == phase && h.command == command)
        })
    }

    /// Whether the user trusted the script with this hash.
    pub fn trusts_script(&self, hash: &str) -> bool {
        self.trusted_scripts.iter().any(|t| t.hash == hash)
    }

    /// Starts watching `issue` (replaces an entry of the same number) and
    /// forgets the oldest past 50, so the list never grows without bound.
    pub fn watch_issue(&mut self, issue: WatchedIssue) {
        self.watched_issues.retain(|w| w.number != issue.number);
        self.watched_issues.push(issue);
        let extra = self.watched_issues.len().saturating_sub(50);
        self.watched_issues.drain(..extra);
    }

    /// Stops watching issue `number`; whether it was watched.
    pub fn unwatch_issue(&mut self, number: u64) -> bool {
        let before = self.watched_issues.len();
        self.watched_issues.retain(|w| w.number != number);
        self.watched_issues.len() != before
    }
}

/// A repository's chosen issue tracker: an `[[integrations]]` profile and a
/// project of that provider (`owner/name`, `group/sub/proj`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssuesSource {
    pub integration: String,
    #[serde(default)]
    pub project: String,
    /// A provider-native extra clause ANDed onto the list query: a Jira
    /// JQL fragment, an Azure WIQL condition, Trello list names. Other
    /// providers ignore it. Absent in files written before it existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
}

/// Which remote "the default remote" is, for a repository.
///
/// In order: the remote the user **chose** (`chosen`, from
/// [`RepoState::default_remote`]) if it still exists; the remote of the
/// checked-out branch's **upstream**; `origin` if there is one; the
/// **only** remote if there is exactly one. `None` when none of those
/// applies (no remote at all, or several and none of them `origin`) --
/// a caller then asks rather than guesses.
///
/// A chosen remote that has since been removed or renamed is ignored
/// rather than returned: pushing to a remote that is not there is an
/// error the user could not connect to the setting that caused it.
pub fn resolve_default_remote(
    chosen: Option<&str>,
    head_upstream_remote: Option<&str>,
    remotes: &[String],
) -> Option<String> {
    let exists = |name: &str| remotes.iter().any(|r| r == name);
    if let Some(chosen) = chosen.filter(|c| exists(c)) {
        return Some(chosen.to_string());
    }
    if let Some(upstream) = head_upstream_remote.filter(|u| exists(u)) {
        return Some(upstream.to_string());
    }
    if exists("origin") {
        return Some("origin".to_string());
    }
    match remotes {
        [only] => Some(only.clone()),
        _ => None,
    }
}

/// Makes the state directory ignore itself, except `workflow.toml` (the
/// shared branching workflow, which the user commits). Written when absent;
/// one GitPoulpi wrote earlier gets the exception appended; an ignore file the
/// user wrote is theirs and is left alone.
pub(crate) fn ensure_state_gitignore(dir: &Path) -> Result<()> {
    // Only the working-tree folder needs hiding: `.git/gitpoulpi` and the
    // user data folder are outside what git looks at.
    if dir
        .file_name()
        .is_none_or(|n| n != gitpoulpi_config::LEGACY_DIR_NAME)
    {
        return Ok(());
    }
    const HEADER: &str = "# Written by GitPoulpi: per-repository UI state, not project content.\n";
    let ignore = dir.join(".gitignore");
    if !ignore.exists() {
        std::fs::write(&ignore, format!("{HEADER}*\n!workflow.toml\n"))?;
        return Ok(());
    }
    let current = std::fs::read_to_string(&ignore)?;
    if current.starts_with(HEADER) && !current.lines().any(|l| l.trim() == "!workflow.toml") {
        let mut next = current;
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push_str("!workflow.toml\n");
        std::fs::write(&ignore, next)?;
    }
    Ok(())
}

impl RepoState {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&raw)?)
    }

    /// Writes the file, and makes its directory ignore itself.
    ///
    /// The directory sits in the working tree (`<repo>/.gitpoulpi/` by
    /// default), so without an ignore it makes the repository "dirty" with
    /// a WIP row for a file the user never wrote. A `.gitignore` of `*`
    /// inside the directory hides it - itself included - without
    /// touching the user's own `.gitignore` or `.git/info/exclude`. An
    /// existing one is left alone: if the user decided to commit this
    /// directory, that is their call.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            gitpoulpi_config::secure_fs::create_private_dir_all(parent)?;
            ensure_state_gitignore(parent)?;
        }
        let raw = toml::to_string_pretty(self)?;
        gitpoulpi_config::secure_fs::write_private(path, raw.as_bytes())?;
        Ok(())
    }

    /// Where a repository's state file lives, given its working-tree
    /// root and `session.per_repo_state_dir` from `config.toml` (a
    /// [`gitpoulpi_config::RepoDataLocation`] name).
    ///
    /// One function rather than a `join` at each call site: the TUI, and
    /// anything that follows it, must agree on the path or a setting
    /// saved by one is invisible to the other. Also migrates a legacy
    /// `<repo>/.gitpoulpi` folder (see [`gitpoulpi_config::repo_data_file`]).
    pub fn path_for(repo_root: &Path, state_dir: &str) -> std::path::PathBuf {
        gitpoulpi_config::repo_data_file(repo_root, state_dir, "repo-state.toml")
    }

    /// Where the hosting snapshot cache (`hosting-cache.json`) sits: beside
    /// the state file. The TUI writes it; the workspace search reads it.
    pub fn hosting_cache_path_for(repo_root: &Path, state_dir: &str) -> std::path::PathBuf {
        gitpoulpi_config::repo_data_file(repo_root, state_dir, "hosting-cache.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Saving state must not make the repository dirty: the state
    /// directory lives in the working tree.
    #[test]
    fn trusted_scripts_round_trip_and_are_keyed_by_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        let mut state = RepoState::default();
        state.trust_script("aa", "scripts/a.py", 10);
        state.trust_script("bb", "scripts/b.py", 11);
        state.trust_script("aa", "renamed.py", 12);
        assert_eq!(state.trusted_scripts.len(), 2);
        assert!(state.trusts_script("aa") && state.trusts_script("bb"));
        assert!(!state.trusts_script("cc"), "another hash is not trusted");
        state.save(&path).unwrap();
        let back = RepoState::load(&path).unwrap();
        assert_eq!(back.trusted_scripts, state.trusted_scripts);
        assert_eq!(back.trusted_scripts[1].label, "renamed.py");
    }

    #[test]
    fn trusted_hooks_are_keyed_by_hash_and_capped_at_fifty() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        let line = |c: &str| HookLine {
            phase: "after_start".into(),
            command: c.into(),
        };
        let mut state = RepoState::default();
        state.trust_hooks("aa", vec![line("make")], 1);
        state.trust_hooks("aa", vec![line("make")], 2);
        assert_eq!(state.trusted_hooks.len(), 1);
        assert!(state.trusts_hooks("aa") && !state.trusts_hooks("bb"));
        assert!(state.trusts_hook_command("after_start", "make"));
        assert!(!state.trusts_hook_command("before_finish", "make"));
        assert!(!state.trusts_hook_command("after_start", "make; rm"));
        for i in 0..60 {
            state.trust_hooks(&format!("h{i}"), vec![line("x")], 3);
        }
        assert_eq!(state.trusted_hooks.len(), 50);
        assert!(!state.trusts_hooks("aa"), "the oldest is forgotten");
        state.save(&path).unwrap();
        assert_eq!(
            RepoState::load(&path).unwrap().trusted_hooks,
            state.trusted_hooks
        );
    }

    #[test]
    fn fold_state_round_trips_and_leaves_other_fields_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        let mut state = RepoState {
            default_remote: Some("up".into()),
            ..Default::default()
        };
        state.folds = FoldState {
            sidebar_sections: vec!["ISSUES".into(), "PULL REQUESTS".into()],
            status_sections: vec!["staged".into()],
            pull_request_groups: Some(vec![]),
            status_folders: vec!["src/a".into()],
            commit_folders: vec!["docs".into()],
        };
        state.save(&path).unwrap();
        let back = RepoState::load(&path).unwrap();
        assert_eq!(back.folds, state.folds);
        assert_eq!(back.default_remote.as_deref(), Some("up"));
    }

    #[test]
    fn hidden_sidebar_sections_distinguish_no_choice_from_show_all() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        RepoState::default().save(&path).unwrap();
        assert_eq!(
            RepoState::load(&path).unwrap().sidebar_sections_hidden,
            None
        );
        let own = RepoState {
            sidebar_sections_hidden: Some(vec!["tags".into()]),
            ..Default::default()
        };
        own.save(&path).unwrap();
        assert_eq!(
            RepoState::load(&path).unwrap().sidebar_sections_hidden,
            Some(vec!["tags".to_string()])
        );
        let all = RepoState {
            sidebar_sections_hidden: Some(vec![]),
            ..Default::default()
        };
        all.save(&path).unwrap();
        assert_eq!(
            RepoState::load(&path).unwrap().sidebar_sections_hidden,
            Some(vec![])
        );
    }

    #[test]
    fn a_state_file_without_folds_loads_as_never_chosen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.toml");
        std::fs::write(&path, "default_remote = \"o\"\n").unwrap();
        let back = RepoState::load(&path).unwrap();
        assert_eq!(back.folds, FoldState::default());
        assert!(back.folds.pull_request_groups.is_none());
    }

    #[test]
    fn saving_state_leaves_git_status_clean() {
        let dir = tempfile::tempdir().unwrap();
        let git = |args: &[&str]| {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(dir.path())
                .output()
                .expect("run git");
            assert!(out.status.success(), "git {args:?}");
            String::from_utf8_lossy(&out.stdout).into_owned()
        };
        git(&["init", "-q"]);
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        RepoState::default().save(&path).unwrap();
        assert!(path.exists());
        assert_eq!(git(&["status", "--porcelain", "--untracked-files=all"]), "");
    }

    /// A `.gitignore` the user put there themselves is theirs.
    #[test]
    fn an_existing_gitignore_in_the_state_directory_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let ignore = path.parent().unwrap().join(".gitignore");
        std::fs::write(&ignore, "mine\n").unwrap();
        RepoState::default().save(&path).unwrap();
        assert_eq!(std::fs::read_to_string(&ignore).unwrap(), "mine\n");
    }

    /// Round-tripping must not turn "inherit" into a value: a `None` that
    /// comes back as `Some("native")` silently starts overriding the
    /// user's git configuration for every repo they ever opened.
    #[test]
    fn unset_repo_preferences_stay_unset_across_a_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        RepoState::default().save(&path).unwrap();

        let back = RepoState::load(&path).unwrap();
        assert!(back.preferences.is_empty());
        assert_eq!(back.preferences.autocrlf, None);
        assert_eq!(back.preferences.lfs_enabled, None);
    }

    #[test]
    fn a_chosen_repo_preference_round_trips_and_leaves_the_others_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        let mut state = RepoState::default();
        state.preferences.autocrlf = Some("input".into());
        state.preferences.lfs_enabled = Some(true);
        state.preferences.profile = Some("work".into());
        state.save(&path).unwrap();

        let back = RepoState::load(&path).unwrap();
        assert_eq!(back.preferences.autocrlf.as_deref(), Some("input"));
        assert_eq!(back.preferences.lfs_enabled, Some(true));
        assert_eq!(back.preferences.profile.as_deref(), Some("work"));
        assert_eq!(back.preferences.encoding, None);
        assert_eq!(back.preferences.eol, None);
    }

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    /// Each rung of the ladder wins over the ones below it, and a rung
    /// naming a remote that no longer exists is skipped, not returned.
    #[test]
    fn the_default_remote_ladder() {
        let all = names(&["fork", "origin", "upstream"]);
        assert_eq!(
            resolve_default_remote(Some("fork"), Some("upstream"), &all).as_deref(),
            Some("fork"),
            "the user's choice wins"
        );
        assert_eq!(
            resolve_default_remote(Some("gone"), Some("upstream"), &all).as_deref(),
            Some("upstream"),
            "a chosen remote that was removed falls through to the upstream's"
        );
        assert_eq!(
            resolve_default_remote(None, None, &all).as_deref(),
            Some("origin")
        );
        assert_eq!(
            resolve_default_remote(None, None, &names(&["gh"])).as_deref(),
            Some("gh"),
            "the only remote, whatever its name"
        );
        assert_eq!(
            resolve_default_remote(None, None, &names(&["a", "b"])),
            None,
            "several remotes and no origin: ask, do not guess"
        );
        assert_eq!(resolve_default_remote(Some("x"), Some("y"), &[]), None);
    }

    /// The review context defaults are per repository, every field optional:
    /// nothing chosen is nothing written, and each field survives alone.
    #[test]
    fn the_review_context_defaults_round_trip_as_none_until_chosen() {
        use gitpoulpi_config::{RepoAiReview, RunInRepo};
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        RepoState::default().save(&path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("ai_review"), "unset is not written: {raw}");
        assert_eq!(
            RepoState::load(&path).unwrap().preferences.ai_review,
            RepoAiReview::default()
        );
        let mut state = RepoState::default();
        state.preferences.ai_review = RepoAiReview {
            run_in_repository: Some(RunInRepo::No),
            include_claude_md: Some(false),
            include_agents_md: None,
            extra_file: Some(String::new()),
            no_extra_file: Some(true),
        };
        state.save(&path).unwrap();
        let back = RepoState::load(&path).unwrap().preferences.ai_review;
        assert_eq!(back.run_in_repository, Some(RunInRepo::No));
        assert_eq!(back.include_claude_md, Some(false));
        assert_eq!(back.include_agents_md, None, "left to inherit");
        assert_eq!(back.extra_file.as_deref(), Some(""), "cleared is not unset");
        assert_eq!(back.no_extra_file, Some(true));
    }

    /// The choice is per repository and survives a save/load.
    #[test]
    fn the_chosen_default_remote_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        let state = RepoState {
            default_remote: Some("fork".into()),
            ..RepoState::default()
        };
        state.save(&path).unwrap();
        assert_eq!(
            RepoState::load(&path).unwrap().default_remote.as_deref(),
            Some("fork")
        );
        RepoState::default().save(&path).unwrap();
        assert_eq!(RepoState::load(&path).unwrap().default_remote, None);
    }

    /// The pinned integration profile survives a save/load, and an old
    /// file without the key loads as "not pinned".
    #[test]
    fn the_pinned_integration_profile_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        let state = RepoState {
            hosting_integration: Some("corp".into()),
            ..RepoState::default()
        };
        state.save(&path).unwrap();
        assert_eq!(
            RepoState::load(&path)
                .unwrap()
                .hosting_integration
                .as_deref(),
            Some("corp")
        );
        RepoState::default().save(&path).unwrap();
        assert_eq!(RepoState::load(&path).unwrap().hosting_integration, None);
        assert_eq!(
            toml::from_str::<RepoState>("").unwrap().hosting_integration,
            None
        );
    }

    #[test]
    fn the_issues_source_round_trips_and_defaults_to_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        let state = RepoState {
            issues_source: Some(IssuesSource {
                integration: "corp".into(),
                project: "g/sub/p".into(),
                filter: None,
            }),
            ..RepoState::default()
        };
        state.save(&path).unwrap();
        assert_eq!(
            RepoState::load(&path).unwrap().issues_source,
            state.issues_source
        );
        RepoState::default().save(&path).unwrap();
        assert_eq!(RepoState::load(&path).unwrap().issues_source, None);
        assert_eq!(toml::from_str::<RepoState>("").unwrap().issues_source, None);
    }

    #[test]
    fn the_source_filter_round_trips_and_old_files_load() {
        let source = IssuesSource {
            integration: "jira".into(),
            project: "ABC".into(),
            filter: Some("component = UI".into()),
        };
        let raw = toml::to_string(&source).unwrap();
        assert!(raw.contains("filter = \"component = UI\""), "{raw}");
        assert_eq!(toml::from_str::<IssuesSource>(&raw).unwrap(), source);
        let unfiltered = IssuesSource {
            filter: None,
            ..source
        };
        assert!(!toml::to_string(&unfiltered).unwrap().contains("filter"));
        // A file written before the field existed.
        let old: IssuesSource =
            toml::from_str("integration = \"corp\"\nproject = \"g/p\"\n").unwrap();
        assert_eq!(old.filter, None);
        let state: RepoState =
            toml::from_str("[issues_source]\nintegration = \"corp\"\nproject = \"g/p\"\n").unwrap();
        assert_eq!(state.issues_source.unwrap().filter, None);
    }

    #[test]
    fn the_pulls_source_round_trips_and_defaults_to_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = RepoState::path_for(dir.path(), ".gitpoulpi");
        let state = RepoState {
            pulls_source: Some(IssuesSource {
                integration: "corp".into(),
                project: "g/p".into(),
                filter: None,
            }),
            ..RepoState::default()
        };
        state.save(&path).unwrap();
        assert_eq!(
            RepoState::load(&path).unwrap().pulls_source,
            state.pulls_source
        );
        RepoState::default().save(&path).unwrap();
        assert_eq!(RepoState::load(&path).unwrap().pulls_source, None);
    }

    /// The path is derived in exactly one place, so a setting written by
    /// one component is found by the next.
    #[test]
    fn the_state_file_hangs_off_the_repo_root_and_the_configured_directory() {
        let path = RepoState::path_for(Path::new("/repos/gitpoulpi"), ".gitpoulpi");
        assert!(path.ends_with(std::path::Path::new(".gitpoulpi").join("repo-state.toml")));
        assert!(path.starts_with("/repos/gitpoulpi"));
    }

    fn ai_with_archive() -> AiConfig {
        let (config, _) = gitpoulpi_config::Config::from_toml_str(
            r#"
[ai]
enabled = true

[[ai.presets]]
id = "fr"
action = "commit_message"
profile = "cc"
system_prompt = "En francais."
"#,
        )
        .unwrap();
        config.ai
    }

    #[test]
    fn an_old_repo_preset_choice_is_read_and_migrated_to_an_override() {
        let ai = ai_with_archive();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("repo-state.toml");
        std::fs::write(
            &path,
            "[preferences.ai_presets]
commit_message = \"fr\"
stash_message = \"builtin:stash_message\"
",
        )
        .unwrap();
        let mut state = RepoState::load(&path).unwrap();
        let expected = AiActionSettings {
            profile: "cc".into(),
            instructions: "En francais.".into(),
        };
        // Readable before any migration, so no frontend has to care.
        assert_eq!(
            state.preferences.ai_override("commit_message", &ai),
            Some(expected.clone())
        );
        assert_eq!(state.preferences.ai_override("stash_message", &ai), None);

        assert!(state.preferences.migrate_ai(&ai));
        assert!(!state.preferences.migrate_ai(&ai));
        state.save(&path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("ai_presets"), "{raw}");
        let reloaded = RepoState::load(&path).unwrap();
        assert_eq!(
            reloaded.preferences.ai_override("commit_message", &ai),
            Some(expected)
        );
        assert!(reloaded.preferences.ai_presets.is_empty());
    }

    #[test]
    fn default_push_targets_round_trip_and_an_empty_list_is_not_written() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("repo-state.toml");
        let mut state = RepoState::default();
        state.save(&path).unwrap();
        assert!(!std::fs::read_to_string(&path)
            .unwrap()
            .contains("default_push_targets"));
        state.preferences.default_push_targets = vec!["gitlab".into(), "backup".into()];
        state.save(&path).unwrap();
        let back = RepoState::load(&path).unwrap();
        assert_eq!(back.preferences.default_push_targets, ["gitlab", "backup"]);
    }

    #[test]
    fn an_empty_override_inherits() {
        let mut prefs = RepoPreferences::default();
        prefs
            .ai_overrides
            .insert("commit_message".into(), AiActionSettings::default());
        assert_eq!(
            prefs.ai_override("commit_message", &AiConfig::default()),
            None
        );
    }
}
