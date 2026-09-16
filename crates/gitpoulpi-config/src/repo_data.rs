//! Where the data GitPoulpi keeps about one repository lives (ref filters,
//! fold state, caches, the personal workflow override...).
//!
//! The decision (`docs/architecture/session-and-state.md`): by default inside
//! the repository's git directory, so the working tree never gets a folder
//! the user did not create. `config.toml`'s `session.per_repo_state_dir`
//! chooses; every reader and writer goes through [`repo_data_file`] /
//! [`repo_data_dir`], never a `join` of its own.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::ConfigPaths;

/// The `session.per_repo_state_dir` default.
pub const DEFAULT_REPO_DATA_LOCATION: &str = "in_git_dir";

/// The folder older versions used inside the working tree.
pub const LEGACY_DIR_NAME: &str = ".gitpoulpi";

/// Where per-repository data is stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoDataLocation {
    /// `<git common dir>/gitpoulpi`: shared by all worktrees of a repository.
    InGitDir,
    /// `<repo>/.gitpoulpi`: the old behaviour.
    InWorktree,
    /// `<OS data dir>/repos/<name>-<id>`: outside the repository.
    UserData,
}

impl RepoDataLocation {
    pub const ALL: &'static [RepoDataLocation] = &[
        RepoDataLocation::InGitDir,
        RepoDataLocation::InWorktree,
        RepoDataLocation::UserData,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            RepoDataLocation::InGitDir => "in_git_dir",
            RepoDataLocation::InWorktree => "in_worktree",
            RepoDataLocation::UserData => "user_data",
        }
    }

    /// Reads the config string. The legacy value `".gitpoulpi"` (what the
    /// old default wrote when a config was saved in full) means the working
    /// tree folder; anything unknown or empty is the default.
    pub fn parse(setting: &str) -> Self {
        match setting.trim() {
            "in_worktree" | LEGACY_DIR_NAME => RepoDataLocation::InWorktree,
            "user_data" => RepoDataLocation::UserData,
            _ => RepoDataLocation::InGitDir,
        }
    }
}

/// The directory holding the data of the repository whose working tree is
/// `repo_root`, per `setting` (`session.per_repo_state_dir`). Not created.
pub fn repo_data_dir(repo_root: &Path, setting: &str) -> PathBuf {
    let paths = ConfigPaths::discover();
    repo_data_dir_with(repo_root, setting, paths.as_ref())
}

/// [`repo_data_dir`] with explicit OS paths (tests).
pub fn repo_data_dir_with(repo_root: &Path, setting: &str, paths: Option<&ConfigPaths>) -> PathBuf {
    let worktree = repo_root.join(LEGACY_DIR_NAME);
    match RepoDataLocation::parse(setting) {
        RepoDataLocation::InWorktree => worktree,
        // Not a git repository (or no `.git` we can read): the old place.
        RepoDataLocation::InGitDir => git_common_dir(repo_root)
            .map(|d| d.join("gitpoulpi"))
            .unwrap_or(worktree),
        RepoDataLocation::UserData => match paths {
            Some(p) => p.repos_data_dir().join(user_data_folder_name(repo_root)),
            None => worktree,
        },
    }
}

/// One file of the repository's data, with the migration from the legacy
/// `<repo>/.gitpoulpi` folder: when the chosen place has no such file but the
/// legacy folder has, its untracked files are moved over first (see
/// [`migrate_legacy`]). A file that stays in the legacy folder because git
/// tracks it (a committed `workflow.toml`) is still returned from there.
pub fn repo_data_file(repo_root: &Path, setting: &str, name: &str) -> PathBuf {
    let paths = ConfigPaths::discover();
    repo_data_file_with(repo_root, setting, name, paths.as_ref())
}

/// [`repo_data_file`] with explicit OS paths (tests).
pub fn repo_data_file_with(
    repo_root: &Path,
    setting: &str,
    name: &str,
    paths: Option<&ConfigPaths>,
) -> PathBuf {
    let dir = repo_data_dir_with(repo_root, setting, paths);
    let file = dir.join(name);
    let legacy = repo_root.join(LEGACY_DIR_NAME);
    if dir == legacy || file.exists() || !legacy.join(name).exists() {
        return file;
    }
    migrate_legacy(repo_root, &dir);
    if file.exists() || !legacy.join(name).exists() {
        file
    } else {
        legacy.join(name)
    }
}

/// Moves the untracked files of `<repo>/.gitpoulpi` into `dest`, once per
/// process and repository, then removes the legacy folder when nothing is left
/// in it. Never touches a file git tracks (the team's committed
/// `workflow.toml`); when git cannot tell, nothing moves. Never overwrites a
/// file already in `dest`. Best effort: a failure leaves the file where it was
/// (reads fall back to it).
pub fn migrate_legacy(repo_root: &Path, dest: &Path) {
    static DONE: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
    let legacy = repo_root.join(LEGACY_DIR_NAME);
    let done = DONE.get_or_init(|| Mutex::new(HashSet::new()));
    if !done.lock().is_ok_and(|mut d| d.insert(legacy.clone())) {
        return;
    }
    let Some(tracked) = tracked_in_legacy(repo_root) else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(&legacy) else {
        return;
    };
    if std::fs::create_dir_all(dest).is_err() {
        return;
    }
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        // The self-ignoring `.gitignore` is only meaningful in the working
        // tree: it is dropped with the folder below.
        if !entry.path().is_file()
            || tracked.contains(name_str.as_ref())
            || name_str == ".gitignore"
        {
            continue;
        }
        let target = dest.join(&name);
        if target.exists() {
            continue;
        }
        if std::fs::rename(entry.path(), &target).is_err()
            && std::fs::copy(entry.path(), &target).is_ok()
        {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    let ignore = legacy.join(".gitignore");
    if !tracked.contains(".gitignore")
        && std::fs::read_to_string(&ignore).is_ok_and(|t| t.starts_with("# Written by GitPoulpi:"))
    {
        let _ = std::fs::remove_file(&ignore);
    }
    // Only succeeds when empty.
    let _ = std::fs::remove_dir(&legacy);
}

/// The names (directly inside the legacy folder) git tracks, or `None` when
/// git could not be asked.
fn tracked_in_legacy(repo_root: &Path) -> Option<HashSet<String>> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["ls-files", "-z", "--", LEGACY_DIR_NAME])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Some(
        text.split('\0')
            .filter_map(|p| p.strip_prefix(".gitpoulpi/"))
            .map(|p| p.split('/').next().unwrap_or(p).to_string())
            .collect(),
    )
}

/// The repository's common git directory, read from the file system: `.git`
/// itself, or, for a linked worktree or submodule (`.git` is a `gitdir:`
/// file), the directory it points to, following `commondir` so all worktrees
/// of a repository agree.
pub fn git_common_dir(repo_root: &Path) -> Option<PathBuf> {
    let dotgit = repo_root.join(".git");
    if dotgit.is_dir() {
        return Some(dotgit);
    }
    let text = std::fs::read_to_string(&dotgit).ok()?;
    let target = text.lines().find_map(|l| l.strip_prefix("gitdir:"))?.trim();
    let gitdir = repo_root.join(target);
    if !gitdir.is_dir() {
        return None;
    }
    match std::fs::read_to_string(gitdir.join("commondir")) {
        Ok(common) => {
            let common = gitdir.join(common.trim());
            Some(std::fs::canonicalize(&common).unwrap_or(common))
        }
        Err(_) => Some(gitdir),
    }
}

/// `<readable name>-<stable id>` for the user-data folder of a repository.
/// The id hashes the canonical path of the common git directory (or the
/// working tree when there is none), so every worktree of a repository maps
/// to the same folder and the name alone can never collide.
pub fn user_data_folder_name(repo_root: &Path) -> String {
    let common = git_common_dir(repo_root);
    let identity = common.clone().unwrap_or_else(|| repo_root.to_path_buf());
    let canonical = std::fs::canonicalize(&identity).unwrap_or(identity);
    let mut text = canonical.to_string_lossy().replace('\\', "/");
    if let Some(rest) = text.strip_prefix("//?/") {
        text = rest.to_string();
    }
    if cfg!(windows) {
        text = text.to_lowercase();
    }
    // FNV-1a: stable across platforms and releases (unlike `DefaultHasher`).
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    // The readable name: the repository folder (for a linked worktree the
    // main repository's, i.e. the parent of the common `.git`).
    let name_source = match &common {
        Some(d) if d.file_name().is_some_and(|n| n == ".git") => {
            d.parent().map(Path::to_path_buf).unwrap_or_default()
        }
        _ => repo_root.to_path_buf(),
    };
    let name: String = name_source
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .take(40)
        .collect();
    let name = name.trim_matches('.');
    if name.is_empty() {
        format!("{hash:016x}")
    } else {
        format!("{name}-{hash:016x}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn git(dir: &Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "git {args:?}");
    }

    fn init(dir: &Path) {
        git(dir, &["init", "-q"]);
        git(dir, &["config", "user.email", "t@example.com"]);
        git(dir, &["config", "user.name", "T"]);
    }

    #[test]
    fn the_setting_strings_parse_and_the_legacy_value_means_the_worktree() {
        use RepoDataLocation::*;
        assert_eq!(RepoDataLocation::parse("in_git_dir"), InGitDir);
        assert_eq!(RepoDataLocation::parse("in_worktree"), InWorktree);
        assert_eq!(RepoDataLocation::parse("user_data"), UserData);
        assert_eq!(RepoDataLocation::parse(".gitpoulpi"), InWorktree);
        assert_eq!(RepoDataLocation::parse(""), InGitDir);
        assert_eq!(RepoDataLocation::parse("nonsense"), InGitDir);
        for l in RepoDataLocation::ALL {
            assert_eq!(RepoDataLocation::parse(l.as_str()), *l);
        }
        assert_eq!(DEFAULT_REPO_DATA_LOCATION, "in_git_dir");
    }

    #[test]
    fn each_location_resolves_where_it_says() {
        let repo = tempfile::tempdir().unwrap();
        init(repo.path());
        let cfg = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(cfg.path());
        assert_eq!(
            repo_data_dir_with(repo.path(), "in_git_dir", Some(&paths)),
            repo.path().join(".git").join("gitpoulpi")
        );
        assert_eq!(
            repo_data_dir_with(repo.path(), "in_worktree", Some(&paths)),
            repo.path().join(".gitpoulpi")
        );
        let user = repo_data_dir_with(repo.path(), "user_data", Some(&paths));
        assert_eq!(user.parent().unwrap(), paths.repos_data_dir());
        // A readable name, a stable id.
        let folder = user.file_name().unwrap().to_string_lossy().into_owned();
        assert!(folder.contains('-'), "{folder}");
        assert_eq!(
            user,
            repo_data_dir_with(repo.path(), "user_data", Some(&paths))
        );
    }

    #[test]
    fn a_non_repository_falls_back_to_the_working_tree_folder() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            repo_data_dir_with(dir.path(), "in_git_dir", None),
            dir.path().join(".gitpoulpi")
        );
    }

    #[test]
    fn linked_worktrees_share_the_main_repositorys_folder() {
        let main = tempfile::tempdir().unwrap();
        init(main.path());
        std::fs::write(main.path().join("f"), "x").unwrap();
        git(main.path(), &["add", "f"]);
        git(main.path(), &["commit", "-qm", "c"]);
        let wt_parent = tempfile::tempdir().unwrap();
        let wt = wt_parent.path().join("wt");
        git(
            main.path(),
            &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "other"],
        );
        let a = repo_data_dir_with(main.path(), "in_git_dir", None);
        let b = repo_data_dir_with(&wt, "in_git_dir", None);
        assert!(b.ends_with("gitpoulpi"));
        assert_eq!(
            std::fs::canonicalize(a.parent().unwrap()).unwrap(),
            std::fs::canonicalize(b.parent().unwrap()).unwrap()
        );
        let cfg = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(cfg.path());
        assert_eq!(
            repo_data_dir_with(main.path(), "user_data", Some(&paths)),
            repo_data_dir_with(&wt, "user_data", Some(&paths))
        );
    }

    #[test]
    fn untracked_legacy_files_move_and_the_empty_folder_goes() {
        let repo = tempfile::tempdir().unwrap();
        init(repo.path());
        let legacy = repo.path().join(".gitpoulpi");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("repo-state.toml"), "a = 1\n").unwrap();
        std::fs::write(
            legacy.join(".gitignore"),
            "# Written by GitPoulpi: per-repository UI state, not project content.\n*\n",
        )
        .unwrap();
        let file = repo_data_file_with(repo.path(), "in_git_dir", "repo-state.toml", None);
        assert_eq!(file, repo.path().join(".git/gitpoulpi/repo-state.toml"));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "a = 1\n");
        assert!(!legacy.exists(), "the empty legacy folder is removed");
    }

    #[test]
    fn a_tracked_workflow_stays_put_and_stays_readable() {
        let repo = tempfile::tempdir().unwrap();
        init(repo.path());
        let legacy = repo.path().join(".gitpoulpi");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("workflow.toml"), "team = true\n").unwrap();
        std::fs::write(legacy.join("repo-state.toml"), "a = 1\n").unwrap();
        git(repo.path(), &["add", "-f", ".gitpoulpi/workflow.toml"]);
        git(repo.path(), &["commit", "-qm", "workflow"]);
        let wf = repo_data_file_with(repo.path(), "in_git_dir", "workflow.toml", None);
        // Still read from the committed file.
        assert_eq!(wf, legacy.join("workflow.toml"));
        assert_eq!(std::fs::read_to_string(&wf).unwrap(), "team = true\n");
        // The untracked state moved; the tracked file was never touched.
        assert!(repo.path().join(".git/gitpoulpi/repo-state.toml").is_file());
        assert!(!legacy.join("repo-state.toml").exists());
        assert!(legacy.join("workflow.toml").is_file());
    }

    #[test]
    fn existing_data_in_the_new_place_is_never_overwritten() {
        let repo = tempfile::tempdir().unwrap();
        init(repo.path());
        let legacy = repo.path().join(".gitpoulpi");
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("hosting-cache.json"), "old").unwrap();
        let new = repo.path().join(".git/gitpoulpi");
        std::fs::create_dir_all(&new).unwrap();
        std::fs::write(new.join("hosting-cache.json"), "new").unwrap();
        let f = repo_data_file_with(repo.path(), "in_git_dir", "hosting-cache.json", None);
        assert_eq!(std::fs::read_to_string(f).unwrap(), "new");
    }
}
