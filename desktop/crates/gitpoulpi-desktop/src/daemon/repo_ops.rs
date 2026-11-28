//! Worktrees, submodules, remotes, LFS and file commands as link actions: what each one turns into, the
//! questions that come first and the rules of their forms.
//!
//! The rules are the terminal UI's (`worktree_dialog.rs`, `remotes_ui`, `lfs_ui.rs`, `status_pane.rs` keep
//! them private to that crate, so they are restated here over the shared `gitpoulpi_core::domain` helpers).
//! Everything is pure: it reads what the panels hold and answers with an [`Action`] or a reason, and the
//! window submits. One submitted action is one journal entry.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::{
    rule_pattern, short, worktree_generated_name, worktree_id, LfsTrackOptions, Oid, Ref, RefKind,
    Remote, RuleKind, RuleScope, Submodule, SubmoduleState, Treat, Worktree, WorktreeAddOptions,
    WorktreeStart,
};
use gitpoulpi_i18n::msg;
use gitpoulpi_link::Action;

use super::Backend;
use super::OperationKind;

// -- worktrees -----------------------------------------------------------------------------------------

/// Where a new worktree starts and how the form words it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeSeed {
    pub start: WorktreeStart,
    /// What the form calls the start: `feature`, `origin/feature`, `v1.0`, `a1b2c3d`.
    pub label: String,
    /// A branch name the caller wants for the worktree (a remote branch's local name).
    pub new_branch: String,
    /// The start branch is checked out in a worktree already, so a new branch is required.
    pub branch_busy: bool,
}

fn busy(branch: &str, worktrees: &[Worktree]) -> bool {
    worktrees
        .iter()
        .any(|w| w.branch.as_deref() == Some(branch))
}

/// "From the current branch": the checked-out branch cannot be checked out twice, so the form proposes a
/// new branch on it; a detached HEAD starts a detached worktree there.
pub fn seed_from_head(refs: &[Ref], head: Option<Oid>) -> Option<WorktreeSeed> {
    if let Some(r) = refs.iter().find(|r| r.is_head) {
        if let RefKind::LocalBranch { name } = &r.kind {
            return Some(WorktreeSeed {
                start: WorktreeStart::Branch(name.clone()),
                label: name.clone(),
                new_branch: String::new(),
                branch_busy: true,
            });
        }
    }
    head.map(|oid| WorktreeSeed {
        start: WorktreeStart::Commit(oid),
        label: short(oid),
        new_branch: String::new(),
        branch_busy: false,
    })
}

/// The seed of a branch or tag row: a local branch starts there (a new branch is required when it is busy),
/// a remote branch or a tag starts at its commit with the local branch name proposed.
pub fn seed_for_ref(
    kind: SeedKind<'_>,
    refs: &[Ref],
    worktrees: &[Worktree],
) -> Option<WorktreeSeed> {
    match kind {
        SeedKind::Local(name) => Some(WorktreeSeed {
            start: WorktreeStart::Branch(name.to_string()),
            label: name.to_string(),
            new_branch: String::new(),
            branch_busy: busy(name, worktrees),
        }),
        SeedKind::Remote(full) => {
            let target = refs.iter().find_map(|r| match &r.kind {
                RefKind::RemoteBranch { remote, name } if format!("{remote}/{name}") == full => {
                    Some((r.target, name.clone()))
                }
                _ => None,
            })?;
            Some(WorktreeSeed {
                start: WorktreeStart::Commit(target.0),
                label: full.to_string(),
                new_branch: target.1,
                branch_busy: false,
            })
        }
        SeedKind::Tag(name) => {
            let at = refs
                .iter()
                .find(|r| matches!(&r.kind, RefKind::Tag { name: n } if n == name))?
                .target;
            Some(WorktreeSeed {
                start: WorktreeStart::Commit(at),
                label: name.to_string(),
                new_branch: String::new(),
                branch_busy: false,
            })
        }
    }
}

/// Which kind of ref a worktree seed is made from.
#[derive(Clone, Copy, Debug)]
pub enum SeedKind<'a> {
    Local(&'a str),
    Remote(&'a str),
    Tag(&'a str),
}

/// "From the selected commit": a detached start there.
pub fn seed_for_commit(oid: Oid) -> WorktreeSeed {
    WorktreeSeed {
        start: WorktreeStart::Commit(oid),
        label: short(oid),
        new_branch: String::new(),
        branch_busy: false,
    }
}

/// A fresh `worktree-<id>` name (8 hex digits from the clock: this crate has no randomness of its own).
pub fn generated_worktree_name() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| {
            d.subsec_nanos() ^ (d.as_secs() as u32).rotate_left(11)
        });
    worktree_generated_name(&worktree_id(nanos.to_le_bytes()))
}

/// `<repo parent>/<repo name>-<label>`: a sibling of the repository named after what it holds.
pub fn default_worktree_path(repo_root: &Path, label: &str) -> PathBuf {
    let safe: String = label
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || "-_.".contains(c) {
                c
            } else {
                '-'
            }
        })
        .collect();
    let name = repo_root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "worktree".into());
    repo_root
        .parent()
        .unwrap_or(repo_root)
        .join(format!("{name}-{safe}"))
}

/// The worktree the form would add, or why it cannot: the folder must be a new or empty absolute path; the
/// branch field is a new branch's name, the name of an existing local branch that no worktree has (checked
/// out as it is), or empty (the start is checked out as it is).
pub fn plan_worktree(
    seed: &WorktreeSeed,
    folder: &str,
    branch: &str,
    refs: &[Ref],
    worktrees: &[Worktree],
) -> Result<(PathBuf, WorktreeAddOptions), String> {
    use msg::worktree as t;
    let folder = folder.trim();
    if folder.is_empty() {
        return Err(t::CHOOSE_FOLDER.get().into());
    }
    let path = PathBuf::from(folder);
    if !path.is_absolute() {
        return Err(t::NOT_ABSOLUTE.get().into());
    }
    if path.is_file() {
        return Err(t::is_a_file(path.display()));
    }
    if std::fs::read_dir(&path).is_ok_and(|mut d| d.next().is_some()) {
        return Err(t::not_empty(path.display()));
    }
    let branch = branch.trim();
    if branch.is_empty() {
        if seed.branch_busy {
            return Err(t::already_checked_out(&seed.label));
        }
        return Ok((
            path,
            WorktreeAddOptions {
                start: seed.start.clone(),
                new_branch: None,
            },
        ));
    }
    let exists = refs
        .iter()
        .any(|r| matches!(&r.kind, RefKind::LocalBranch { name } if name == branch));
    if exists {
        if busy(branch, worktrees) {
            return Err(t::already_checked_out(branch));
        }
        return Ok((
            path,
            WorktreeAddOptions {
                start: WorktreeStart::Branch(branch.to_string()),
                new_branch: None,
            },
        ));
    }
    super::refs_ops::validate_new_ref(super::refs_ops::NewRef::Branch, branch, refs)?;
    Ok((
        path,
        WorktreeAddOptions {
            start: seed.start.clone(),
            new_branch: Some(branch.to_string()),
        },
    ))
}

/// The line under the form's fields: what Create will do.
pub fn worktree_summary(path: &Path, opts: &WorktreeAddOptions) -> String {
    use msg::worktree as t;
    let tail = match (&opts.new_branch, &opts.start) {
        (Some(n), _) => t::on_new_branch(n),
        (None, WorktreeStart::Branch(b)) => t::on_branch(b),
        (None, WorktreeStart::Commit(_)) => t::DETACHED.get().to_string(),
    };
    format!("{}{}{tail}", t::WILL_CREATE.get(), path.display())
}

pub fn worktree_add_action(path: PathBuf, opts: WorktreeAddOptions) -> Action {
    Action::WorktreeAdd { path, opts }
}

/// The worktrees other than the main one.
pub fn is_linked(w: &Worktree) -> bool {
    !w.is_main
}

/// What removing a worktree asks, by what its folder holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemoveAsk {
    /// Nothing uncommitted: one question.
    Clean,
    /// `n` uncommitted changes: removing discards them.
    Dirty(usize),
}

pub fn remove_ask(changes: usize) -> RemoveAsk {
    if changes == 0 {
        RemoveAsk::Clean
    } else {
        RemoveAsk::Dirty(changes)
    }
}

pub fn worktree_remove_action(path: PathBuf, force: bool) -> Action {
    Action::WorktreeRemove {
        path,
        opts: Box::new(gitpoulpi_core::domain::WorktreeRemoveOptions {
            force,
            ..Default::default()
        }),
        delete_branch: None,
    }
}

// -- submodules ----------------------------------------------------------------------------------------

/// The submodule at `path`.
pub fn find_submodule<'a>(list: &'a [Submodule], path: &str) -> Option<&'a Submodule> {
    list.iter().find(|s| s.path == path)
}

/// `update` of `path` (all when `None`), cloning the ones not cloned yet.
pub fn submodule_update_action(path: Option<&str>) -> Action {
    Action::SubmoduleUpdate {
        paths: path.map(|p| vec![p.to_string()]).unwrap_or_default(),
        init: true,
    }
}

pub fn submodule_init_action(path: &str) -> Action {
    Action::SubmoduleInit {
        paths: vec![path.to_string()],
    }
}

/// The folder a submodule opens as, or why it cannot be (the terminal UI's words).
pub fn submodule_folder(root: &Path, sub: &Submodule) -> Result<PathBuf, String> {
    if sub.state == SubmoduleState::Uninitialized {
        return Err(msg::submodules_ui::not_cloned(&sub.path));
    }
    Ok(root.join(&sub.path))
}

/// The glyph of a submodule's state, as the terminal UI draws it.
pub fn submodule_mark(state: SubmoduleState) -> &'static str {
    match state {
        SubmoduleState::UpToDate => "\u{2713}",
        SubmoduleState::Modified => "\u{2191}",
        SubmoduleState::Uninitialized => "\u{25CB}",
        SubmoduleState::Conflicted => "\u{2717}",
    }
}

// -- remotes -------------------------------------------------------------------------------------------

/// `scheme://host/path`, `ssh://...`, `git@host:path`, or a local path (an existing folder or a `file://`).
fn url_ok(url: &str) -> bool {
    let url = url.trim();
    if url.contains(char::is_whitespace) {
        return false;
    }
    if let Some((scheme, rest)) = url.split_once("://") {
        return !scheme.is_empty() && !rest.trim_matches('/').is_empty();
    }
    if let Some((host, path)) = url.split_once(':') {
        // `git@host:owner/repo.git`; a Windows drive (`C:\x`) is a path.
        if host.len() > 1 && host.contains('@') && !path.is_empty() {
            return true;
        }
    }
    Path::new(url).is_absolute()
}

/// The fetch (or push) URL is acceptable, or the reason.
pub fn validate_remote_url(url: &str) -> Result<(), String> {
    let url = url.trim();
    if url.is_empty() {
        return Err(msg::remotes::URL_EMPTY.get().into());
    }
    if url_ok(url) {
        Ok(())
    } else {
        Err(msg::remotes::not_remote_url(url))
    }
}

/// Git's refname rules as they apply to a remote name, and "not already taken" (the name an edit keeps does
/// not count).
pub fn validate_remote_name(
    name: &str,
    taken: &[Remote],
    keeping: Option<&str>,
) -> Result<(), String> {
    if name.is_empty() {
        return Err(msg::remotes::NAME_EMPTY.get().into());
    }
    if name.chars().any(|c| c.is_whitespace() || c.is_control())
        || name.chars().any(|c| "~^:?*[\\".contains(c))
        || name.contains("..")
        || name.contains("@{")
        || name.starts_with(['-', '/'])
        || name.ends_with(['/', '.'])
        || name.ends_with(".lock")
    {
        return Err(msg::remotes::invalid_name(name));
    }
    if taken.iter().any(|r| r.name == name) && keeping != Some(name) {
        return Err(msg::remotes::name_taken(name));
    }
    Ok(())
}

/// The actions an edited remote needs: a rename, a new fetch URL, a new push URL, each only when it
/// changed. They are separate journal entries (the daemon journals each write on its own).
pub fn remote_edit_actions(
    old: &Remote,
    name: &str,
    fetch_url: &str,
    push_url: &str,
) -> Vec<Action> {
    let mut out = Vec::new();
    if name != old.name {
        out.push(Action::RemoteRename {
            old_name: old.name.clone(),
            new_name: name.to_string(),
        });
    }
    if fetch_url != old.fetch_url {
        out.push(Action::RemoteSetUrl {
            name: name.to_string(),
            url: fetch_url.to_string(),
        });
    }
    let new_push = (!push_url.is_empty()).then(|| push_url.to_string());
    if new_push != old.push_url {
        out.push(Action::RemoteSetPushUrl {
            name: name.to_string(),
            url: new_push,
        });
    }
    out
}

/// The remote a branch pushes to and pulls from: its upstream's remote, else `origin`, else the only one.
pub fn remote_of_branch(refs: &[Ref], remotes: &[Remote], branch: &str) -> Option<String> {
    let upstream = refs.iter().find_map(|r| match &r.kind {
        RefKind::LocalBranch { name } if name == branch => r.upstream.as_ref(),
        _ => None,
    });
    if let Some(up) = upstream {
        if let Some(remote) = remotes
            .iter()
            .find(|r| up.starts_with(&format!("{}/", r.name)))
        {
            return Some(remote.name.clone());
        }
    }
    remotes
        .iter()
        .find(|r| r.name == "origin")
        .or(if remotes.len() == 1 {
            remotes.first()
        } else {
            None
        })
        .map(|r| r.name.clone())
}

// -- LFS -----------------------------------------------------------------------------------------------

/// `*.psd` for `art/hero.psd`; `None` for a file with no extension.
pub fn lfs_extension_pattern(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?;
    (!ext.is_empty()).then(|| format!("*.{ext}"))
}

/// A path as `.gitattributes` and `git lfs` want it: forward slashes.
pub fn lfs_file_pattern(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn lfs_track_action(patterns: Vec<String>) -> Action {
    Action::LfsTrack {
        patterns,
        opts: LfsTrackOptions::default(),
    }
}

/// A pattern typed in a form: not empty, no line break.
pub fn validate_lfs_pattern(text: &str) -> Result<String, String> {
    let text = text.trim();
    if text.is_empty() || text.contains(['\n', '\r']) {
        return Err(crate::strings::LFS_PATTERN_NEEDED.into());
    }
    Ok(text.to_string())
}

// -- files ---------------------------------------------------------------------------------------------

/// The `.gitignore` line for `path` at `scope`, `None` when the scope does not fit (an extension of a file
/// without one).
pub fn ignore_pattern(path: &str, scope: RuleScope) -> Option<String> {
    rule_pattern(RuleKind::Ignore, ".gitignore", path, scope)
}

pub fn ignore_action(path: &str, scope: RuleScope) -> Option<Action> {
    ignore_pattern(path, scope).map(|pattern| Action::AddToGitignore {
        pattern,
        file: None,
    })
}

pub fn treat_action(path: &str, scope: RuleScope, treat: Treat) -> Action {
    Action::TreatAs {
        paths: vec![path.to_string()],
        scope,
        treat,
        file: None,
    }
}

/// The folder a file sits in (`None` at the repository root).
pub fn folder_of(path: &str) -> Option<String> {
    let path = path.replace('\\', "/");
    path.rsplit_once('/')
        .map(|(dir, _)| dir.to_string())
        .filter(|d| !d.is_empty())
}

/// The file's own name.
pub fn name_of(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_string()
}

/// The file's extension, without the dot.
pub fn ext_of(path: &str) -> Option<String> {
    let name = name_of(path);
    name.rsplit_once('.')
        .filter(|(stem, ext)| !stem.is_empty() && !ext.is_empty())
        .map(|(_, e)| e.to_string())
}

// -- reads ---------------------------------------------------------------------------------------------

type Fut<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send>>;

impl Backend {
    pub fn remotes(&self) -> Fut<Vec<Remote>> {
        Box::pin(self.call(OperationKind::Remotes, serde_json::json!({})))
    }

    /// The uncommitted changes of the worktree at `path` (what removing it would lose).
    pub fn worktree_changes(
        &self,
        path: &Path,
    ) -> Fut<Vec<gitpoulpi_core::domain::WorktreeChange>> {
        Box::pin(self.call(
            OperationKind::WorktreeChanges,
            serde_json::json!({ "path": path }),
        ))
    }

    /// The commits that changed `path`, newest first, from `start`.
    pub fn file_history(
        &self,
        path: &str,
        start: Oid,
        skip: usize,
        limit: usize,
    ) -> Fut<gitpoulpi_core::domain::FileHistoryPage> {
        Box::pin(self.call(
            OperationKind::FileHistory,
            serde_json::json!({ "path": path, "start": start, "skip": skip, "limit": limit }),
        ))
    }

    pub fn lfs_status(&self) -> Fut<gitpoulpi_core::domain::LfsRepoStatus> {
        Box::pin(self.call(OperationKind::LfsStatus, serde_json::json!({})))
    }

    pub fn lfs_files(&self) -> Fut<Vec<gitpoulpi_core::domain::LfsFile>> {
        Box::pin(self.call(OperationKind::LfsFiles, serde_json::json!({})))
    }

    pub fn lfs_locks(&self, remote: &str) -> Fut<Vec<gitpoulpi_core::domain::LfsLock>> {
        Box::pin(self.call(
            OperationKind::LfsLocks,
            serde_json::json!({ "remote": remote }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remote(name: &str) -> Remote {
        Remote {
            name: name.into(),
            fetch_url: "https://example.org/a/b.git".into(),
            push_url: None,
        }
    }

    #[test]
    fn remote_names_follow_git_and_may_keep_their_own() {
        let taken = [remote("origin")];
        assert!(validate_remote_name("up", &taken, None).is_ok());
        assert!(validate_remote_name("origin", &taken, None).is_err());
        assert!(validate_remote_name("origin", &taken, Some("origin")).is_ok());
        for bad in ["", "a b", "-x", "a..b", "x.lock", "a:b"] {
            assert!(validate_remote_name(bad, &taken, None).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn remote_urls_are_web_ssh_scp_or_local_paths() {
        for ok in [
            "https://example.org/a/b.git",
            "ssh://git@example.org/a/b.git",
            "git@example.org:a/b.git",
            "file:///srv/a.git",
        ] {
            assert!(validate_remote_url(ok).is_ok(), "{ok}");
        }
        for bad in ["", "nonsense", "https://", "a b"] {
            assert!(validate_remote_url(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn an_edited_remote_sends_only_what_changed() {
        let old = remote("origin");
        assert!(remote_edit_actions(&old, "origin", &old.fetch_url, "").is_empty());
        let acts = remote_edit_actions(&old, "up", "https://example.org/x.git", "ssh://h/p.git");
        assert!(matches!(acts[0], Action::RemoteRename { .. }));
        assert!(matches!(acts[1], Action::RemoteSetUrl { .. }));
        assert!(matches!(
            acts[2],
            Action::RemoteSetPushUrl { url: Some(_), .. }
        ));
    }

    #[test]
    fn ignore_patterns_are_anchored_and_extension_needs_one() {
        assert_eq!(
            ignore_pattern("src/a.log", RuleScope::File).as_deref(),
            Some("/src/a.log")
        );
        assert_eq!(
            ignore_pattern("src/a.log", RuleScope::Extension).as_deref(),
            Some("*.log")
        );
        assert_eq!(ignore_pattern("Makefile", RuleScope::Extension), None);
        assert_eq!(
            ignore_pattern("build/out", RuleScope::Folder).as_deref(),
            Some("/build/out/")
        );
        assert_eq!(folder_of("a/b/c.txt").as_deref(), Some("a/b"));
        assert_eq!(folder_of("c.txt"), None);
        assert_eq!(ext_of("a/.env"), None);
        assert_eq!(ext_of("a/x.tar.gz").as_deref(), Some("gz"));
    }

    #[test]
    fn lfs_patterns_use_slashes_and_extensions() {
        assert_eq!(
            lfs_extension_pattern(Path::new("art/hero.psd")).as_deref(),
            Some("*.psd")
        );
        assert_eq!(lfs_extension_pattern(Path::new("Makefile")), None);
        assert!(validate_lfs_pattern("  ").is_err());
        assert_eq!(validate_lfs_pattern(" *.bin ").unwrap(), "*.bin");
    }

    #[test]
    fn the_default_worktree_folder_is_a_sibling_named_after_the_label() {
        let p = default_worktree_path(Path::new("/w/repo"), "feat/x y");
        assert_eq!(p, Path::new("/w/repo-feat-x-y"));
    }
}
