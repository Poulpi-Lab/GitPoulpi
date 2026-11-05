//! A saved copy of an integration's repository list, so the Clone dialog
//! shows it at once while a fresh request runs.
//!
//! One JSON file per key (`integration id`, `host`, `login`) in a cache
//! directory the caller names. Read leniently: a missing, corrupt, foreign
//! version or colliding-name file is no cache. Writes go through a temp file
//! and a rename. Never holds a token: the key is the profile id, the host and
//! the connected login, so another account on the same profile is another
//! entry.

use std::path::{Path, PathBuf};

use gitpoulpi_config::Integration;
use serde::{Deserialize, Serialize};

use crate::HostedProject;

/// File format version; any other value reads as no cache.
pub const VERSION: u32 = 1;
/// A list longer than this is cut when stored (bounds a file to a few MB).
pub const MAX_PROJECTS: usize = 5000;
/// Stored descriptions are cut to this many characters.
pub const MAX_DESCRIPTION: usize = 200;
/// [`prune`] keeps at most this many files, newest first.
pub const MAX_FILES: usize = 32;

/// What a saved list belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListKey {
    pub integration_id: String,
    pub host: String,
    /// The connected login, empty when none.
    pub login: String,
}

impl ListKey {
    /// The key of `integration` (`login` is the connected user's login), or
    /// `None` when it has no host to key on.
    pub fn of(integration: &Integration, login: Option<&str>) -> Option<Self> {
        Some(Self {
            integration_id: integration.id.clone(),
            host: integration.host()?,
            login: login.unwrap_or("").to_string(),
        })
    }

    fn file_name(&self) -> String {
        let clean = |s: &str| -> String {
            s.chars()
                .map(|c| {
                    if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                        c
                    } else {
                        '_'
                    }
                })
                .collect()
        };
        format!(
            "{}@{}@{}.json",
            clean(&self.integration_id),
            clean(&self.host),
            clean(&self.login)
        )
    }
}

/// One saved list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedList {
    pub version: u32,
    pub key: ListKey,
    /// Unix seconds.
    pub fetched_at: u64,
    pub projects: Vec<HostedProject>,
}

impl CachedList {
    pub fn new(key: ListKey, fetched_at: u64, projects: Vec<HostedProject>) -> Self {
        Self {
            version: VERSION,
            key,
            fetched_at,
            projects,
        }
    }
}

fn path_of(dir: &Path, key: &ListKey) -> PathBuf {
    dir.join(key.file_name())
}

/// The saved list of `key`, or `None` when absent or unreadable.
pub fn load(dir: &Path, key: &ListKey) -> Option<CachedList> {
    let raw = std::fs::read_to_string(path_of(dir, key)).ok()?;
    let list: CachedList = serde_json::from_str(&raw).ok()?;
    (list.version == VERSION && list.key == *key).then_some(list)
}

/// Removes any `user:password@` from an `https://` clone URL.
fn strip_userinfo(url: &str) -> String {
    let Some(rest) = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
    else {
        return url.to_string();
    };
    let scheme_len = url.len() - rest.len();
    let end = rest.find('/').unwrap_or(rest.len());
    match rest[..end].rfind('@') {
        Some(at) => format!("{}{}", &url[..scheme_len], &rest[at + 1..]),
        None => url.to_string(),
    }
}

/// Saves `list` (temp file, then rename), capped and stripped of user info.
pub fn store(dir: &Path, list: &CachedList) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let mut list = list.clone();
    list.projects.truncate(MAX_PROJECTS);
    for p in &mut list.projects {
        if let Some(d) = &mut p.description {
            if d.chars().count() > MAX_DESCRIPTION {
                *d = d.chars().take(MAX_DESCRIPTION).collect();
            }
        }
        p.https_url = p.https_url.as_deref().map(strip_userinfo);
    }
    let path = path_of(dir, &list.key);
    let text = serde_json::to_string(&list).map_err(std::io::Error::other)?;
    let tmp = path.with_extension(format!("json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, &path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

/// Deletes the saved list of `key`.
pub fn remove(dir: &Path, key: &ListKey) {
    let _ = std::fs::remove_file(path_of(dir, key));
}

/// Deletes every file whose key is not in `live` (a removed profile, a
/// disconnected or other account), then keeps only the [`MAX_FILES`] newest.
/// Files that do not parse are deleted too.
pub fn prune(dir: &Path, live: &[ListKey]) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut kept: Vec<(u64, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let parsed = std::fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<CachedList>(&raw).ok())
            .filter(|l| l.version == VERSION && live.contains(&l.key));
        match parsed {
            Some(l) => kept.push((l.fetched_at, path)),
            None => {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    kept.sort_by_key(|(at, _)| std::cmp::Reverse(*at));
    for (_, path) in kept.into_iter().skip(MAX_FILES) {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(id: &str, login: &str) -> ListKey {
        ListKey {
            integration_id: id.into(),
            host: "github.com".into(),
            login: login.into(),
        }
    }

    fn project(full: &str) -> HostedProject {
        HostedProject {
            integration_id: "gh".into(),
            id: full.into(),
            name: full.rsplit('/').next().unwrap().into(),
            full_name: full.into(),
            description: None,
            default_branch: None,
            ssh_url: None,
            https_url: None,
            web_url: None,
            private: false,
        }
    }

    #[test]
    fn round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let list = CachedList::new(
            key("gh", "ann"),
            5,
            vec![project("ann/a"), project("org/b")],
        );
        store(dir.path(), &list).unwrap();
        assert_eq!(load(dir.path(), &key("gh", "ann")), Some(list));
        assert_eq!(load(dir.path(), &key("gh", "bob")), None);
    }

    #[test]
    fn corrupt_wrong_version_and_collision_read_as_none() {
        let dir = tempfile::tempdir().unwrap();
        let k = key("gh", "ann");
        std::fs::write(path_of(dir.path(), &k), "{ not json").unwrap();
        assert_eq!(load(dir.path(), &k), None);

        let mut list = CachedList::new(k.clone(), 1, vec![]);
        list.version = 99;
        std::fs::write(
            path_of(dir.path(), &k),
            serde_json::to_string(&list).unwrap(),
        )
        .unwrap();
        assert_eq!(load(dir.path(), &k), None);

        // "a b" and "a_b" sanitize to the same file name; the stored key wins.
        let (k1, k2) = (key("a b", "x"), key("a_b", "x"));
        store(dir.path(), &CachedList::new(k1.clone(), 1, vec![])).unwrap();
        assert_eq!(path_of(dir.path(), &k1), path_of(dir.path(), &k2));
        assert_eq!(load(dir.path(), &k2), None);
        assert!(load(dir.path(), &k1).is_some());
    }

    #[test]
    fn store_caps_and_strips_user_info() {
        let dir = tempfile::tempdir().unwrap();
        let mut many: Vec<HostedProject> = (0..MAX_PROJECTS + 3)
            .map(|i| project(&format!("o/r{i}")))
            .collect();
        many[0].description = Some("x".repeat(MAX_DESCRIPTION + 50));
        many[0].https_url = Some("https://user:secret@host.example/o/r0.git".into());
        many[1].https_url = Some("https://host.example/o/r1.git".into());
        let k = key("gh", "ann");
        store(dir.path(), &CachedList::new(k.clone(), 1, many)).unwrap();
        let back = load(dir.path(), &k).unwrap();
        assert_eq!(back.projects.len(), MAX_PROJECTS);
        assert_eq!(
            back.projects[0].description.as_ref().unwrap().len(),
            MAX_DESCRIPTION
        );
        assert_eq!(
            back.projects[0].https_url.as_deref(),
            Some("https://host.example/o/r0.git")
        );
        assert_eq!(
            back.projects[1].https_url.as_deref(),
            Some("https://host.example/o/r1.git")
        );
        let raw = std::fs::read_to_string(path_of(dir.path(), &k)).unwrap();
        assert!(!raw.contains("secret"));
    }

    #[test]
    fn prune_drops_unknown_ids_and_changed_login() {
        let dir = tempfile::tempdir().unwrap();
        for k in [key("gh", "ann"), key("gh", "bob"), key("gone", "ann")] {
            store(dir.path(), &CachedList::new(k, 1, vec![])).unwrap();
        }
        prune(dir.path(), &[key("gh", "bob")]);
        assert_eq!(load(dir.path(), &key("gh", "ann")), None);
        assert_eq!(load(dir.path(), &key("gone", "ann")), None);
        assert!(load(dir.path(), &key("gh", "bob")).is_some());
    }

    #[test]
    fn prune_keeps_the_newest_files() {
        let dir = tempfile::tempdir().unwrap();
        let keys: Vec<ListKey> = (0..MAX_FILES + 3)
            .map(|i| key(&format!("p{i}"), ""))
            .collect();
        for (i, k) in keys.iter().enumerate() {
            store(dir.path(), &CachedList::new(k.clone(), i as u64, vec![])).unwrap();
        }
        prune(dir.path(), &keys);
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), MAX_FILES);
        assert!(load(dir.path(), &keys[0]).is_none());
        assert!(load(dir.path(), keys.last().unwrap()).is_some());
    }
}
