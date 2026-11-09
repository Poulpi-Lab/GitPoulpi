//! `<private>/repo-auth.toml`: which identity each repository's network git
//! processes use (ADR-0010).
//!
//! Private directory, not `RepoState`: with `per_repo_state_dir = in_worktree`
//! that file lives inside the working tree, where a repository could ship
//! one, and `.git/config` is what the owner does not want touched. Written by
//! `secure_fs::write_private` (0600 on Unix, a user-only DACL on Windows).
//!
//! Records are keyed by `user_data_folder_name(repo_root)` (so every worktree
//! of a repository shares one) and keep the common git directory for display
//! and for noticing a moved repository.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use gitpoulpi_config::ConfigPaths;
use gitpoulpi_core::domain::{HttpsAuth, RepoAuthProfile, SshAuth};
use serde::{Deserialize, Serialize};

/// File name inside [`ConfigPaths::private_dir`].
pub const REPO_AUTH_FILE: &str = "repo-auth.toml";

/// One repository's stored profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoAuthRecord {
    /// `user_data_folder_name(repo_root)`.
    pub key: String,
    /// The common git directory at the time of saving (display, and moved
    /// repository detection).
    #[serde(default)]
    pub git_common_dir: String,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub https: Option<HttpsAuth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshAuth>,
}

impl RepoAuthRecord {
    pub fn profile(&self) -> RepoAuthProfile {
        RepoAuthProfile {
            https: self.https.clone(),
            ssh: self.ssh.clone(),
        }
    }
}

/// The whole file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoAuthStore {
    #[serde(default, rename = "repo", skip_serializing_if = "Vec::is_empty")]
    pub repo: Vec<RepoAuthRecord>,
}

/// Serializes read-modify-write cycles inside one process (the daemon is the
/// only writer, but its connections run concurrently).
static WRITE_LOCK: Mutex<()> = Mutex::new(());

impl RepoAuthStore {
    /// `<private>/repo-auth.toml`.
    pub fn file_in(paths: &ConfigPaths) -> PathBuf {
        paths.private_dir().join(REPO_AUTH_FILE)
    }

    /// The store at `path`. A missing file is an empty store; one that is
    /// there but cannot be read or parsed is an **error**, never an empty
    /// store: an empty one would silently send a repository back to the
    /// machine's own credentials.
    pub fn load(path: &Path) -> Result<Self, String> {
        match std::fs::read_to_string(path) {
            Ok(raw) => {
                toml::from_str(&raw).map_err(|e| format!("{} is unreadable: {e}", path.display()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(format!("{} is unreadable: {e}", path.display())),
        }
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            gitpoulpi_config::secure_fs::create_private_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        gitpoulpi_config::secure_fs::write_private(path, text.as_bytes())
    }

    pub fn get(&self, key: &str) -> Option<&RepoAuthRecord> {
        self.repo.iter().find(|r| r.key == key)
    }
}

/// The key of the repository at `repo_root`.
pub fn repo_auth_key(repo_root: &Path) -> String {
    gitpoulpi_config::user_data_folder_name(repo_root)
}

/// The stored record of the repository at `repo_root`, if any. `Err` when the
/// file exists and cannot be read (callers refuse the transfer).
pub fn load_repo_auth(
    paths: &ConfigPaths,
    repo_root: &Path,
) -> Result<Option<RepoAuthRecord>, String> {
    let store = RepoAuthStore::load(&RepoAuthStore::file_in(paths))?;
    Ok(store.get(&repo_auth_key(repo_root)).cloned())
}

/// Stores `profile` for the repository at `repo_root`, replacing its record.
/// A profile with no source at all removes the record instead.
pub fn save_repo_auth(
    paths: &ConfigPaths,
    repo_root: &Path,
    profile: &RepoAuthProfile,
    now: i64,
) -> Result<(), String> {
    if profile.https.is_none() && profile.ssh.is_none() {
        return clear_repo_auth(paths, repo_root).map(|_| ());
    }
    let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let file = RepoAuthStore::file_in(paths);
    let mut store = RepoAuthStore::load(&file)?;
    let key = repo_auth_key(repo_root);
    let record = RepoAuthRecord {
        key: key.clone(),
        git_common_dir: gitpoulpi_config::git_common_dir(repo_root)
            .unwrap_or_else(|| repo_root.to_path_buf())
            .to_string_lossy()
            .replace('\\', "/"),
        updated_at: now,
        https: profile.https.clone(),
        ssh: profile.ssh.clone(),
    };
    match store.repo.iter_mut().find(|r| r.key == key) {
        Some(slot) => *slot = record,
        None => store.repo.push(record),
    }
    store
        .save(&file)
        .map_err(|e| format!("{} could not be written: {e}", file.display()))
}

/// Removes the repository's record. Whether there was one.
pub fn clear_repo_auth(paths: &ConfigPaths, repo_root: &Path) -> Result<bool, String> {
    let _guard = WRITE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let file = RepoAuthStore::file_in(paths);
    let mut store = RepoAuthStore::load(&file)?;
    let key = repo_auth_key(repo_root);
    let before = store.repo.len();
    store.repo.retain(|r| r.key != key);
    if store.repo.len() == before {
        return Ok(false);
    }
    store
        .save(&file)
        .map_err(|e| format!("{} could not be written: {e}", file.display()))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(dir: &Path) -> PathBuf {
        let root = dir.join("proj");
        std::fs::create_dir_all(root.join(".git")).unwrap();
        root
    }

    #[test]
    fn a_profile_round_trips_through_the_private_file() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path().join("cfg"));
        let root = repo(dir.path());
        let profile = RepoAuthProfile {
            https: Some(HttpsAuth::System {
                username: Some("alice".into()),
                ignore_repo_helpers: true,
            }),
            ssh: Some(SshAuth::KeyFile {
                key: "/k/work".into(),
                known_hosts: Some("/k/kh".into()),
                strict_host_key: true,
            }),
        };
        assert_eq!(load_repo_auth(&paths, &root).unwrap(), None);
        save_repo_auth(&paths, &root, &profile, 42).unwrap();
        let record = load_repo_auth(&paths, &root).unwrap().expect("stored");
        assert_eq!(record.profile(), profile);
        assert_eq!(record.updated_at, 42);
        assert!(record.git_common_dir.ends_with("proj/.git"), "{record:?}");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(RepoAuthStore::file_in(&paths))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600, "the file must be user-only");
        }
        assert!(clear_repo_auth(&paths, &root).unwrap());
        assert!(!clear_repo_auth(&paths, &root).unwrap());
        assert_eq!(load_repo_auth(&paths, &root).unwrap(), None);
    }

    #[test]
    fn two_repositories_keep_their_own_records() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path().join("cfg"));
        let (a, b) = (repo(&dir.path().join("a")), repo(&dir.path().join("b")));
        let system = |user: &str| RepoAuthProfile {
            https: Some(HttpsAuth::System {
                username: Some(user.into()),
                ignore_repo_helpers: false,
            }),
            ssh: None,
        };
        save_repo_auth(&paths, &a, &system("ann"), 1).unwrap();
        save_repo_auth(&paths, &b, &system("bob"), 2).unwrap();
        save_repo_auth(&paths, &a, &system("anna"), 3).unwrap();
        assert_eq!(
            load_repo_auth(&paths, &a).unwrap().unwrap().profile(),
            system("anna")
        );
        assert_eq!(
            load_repo_auth(&paths, &b).unwrap().unwrap().profile(),
            system("bob")
        );
    }

    #[test]
    fn a_source_that_is_not_available_yet_is_parsed_and_kept() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path().join("cfg"));
        let root = repo(dir.path());
        let key = repo_auth_key(&root);
        let file = RepoAuthStore::file_in(&paths);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(
            &file,
            format!(
                "[[repo]]\nkey = \"{key}\"\ngit_common_dir = \"x\"\nupdated_at = 1\n\n\
                 [repo.https]\nsource = \"token_command\"\nargv = [\"pwsh\", \"-File\", \"t.ps1\"]\n\
                 hosts = [\"github.com\"]\ncache_minutes = 50\n\n\
                 [repo.ssh]\nsource = \"integration_key\"\nintegration = \"corp\"\n"
            ),
        )
        .unwrap();
        let record = load_repo_auth(&paths, &root).unwrap().unwrap();
        assert!(matches!(
            record.https,
            Some(HttpsAuth::TokenCommand {
                cache_minutes: Some(50),
                ..
            })
        ));
        assert!(matches!(record.ssh, Some(SshAuth::IntegrationKey { .. })));
    }

    #[test]
    fn an_unreadable_file_is_an_error_not_an_empty_store() {
        let dir = tempfile::tempdir().unwrap();
        let paths = ConfigPaths::for_dir(dir.path().join("cfg"));
        let root = repo(dir.path());
        let file = RepoAuthStore::file_in(&paths);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "[[repo]\nnot toml").unwrap();
        let err = load_repo_auth(&paths, &root).unwrap_err();
        assert!(err.contains("unreadable"), "{err}");
        // And a write does not paper over it.
        assert!(save_repo_auth(&paths, &root, &RepoAuthProfile::default(), 1).is_err());
    }
}
