//! Workspaces: named groups of repositories, optionally with a default
//! commit profile (the workspaces design).
//!
//! **Beside favorites, for the same reason.** Which repositories belong
//! together is the *user's*, not a repository's and not a window's: two
//! frontends disagreeing about it is not a disagreement (ADR-0006), so it is
//! an ordinary file in the config directory, `workspaces.toml`, shared by
//! every session on the machine, read and written through this crate with no
//! IPC. The CLI and MCP edit it the same way the TUI does.
//!
//! ```toml
//! [[workspace]]
//! id = "acme"                    # stable slug, never changes on rename
//! name = "Acme"
//! color = "teal"                 # optional theme accent name
//! description = ""
//! profile = "work"               # optional `[[profiles]]` id
//! repos = ["/home/u/src/acme-api", "/home/u/src/acme-web"]
//!
//! [workspace.remotes]            # path -> origin URL, recorded on add
//! "/home/u/src/acme-api" = "git@example.com:acme/api.git"
//! ```
//!
//! A repository may belong to several workspaces. Its origin URL is kept so
//! a repository missing on disk stays listed (and can be cloned again).
//! Repositories are joined by exact path ([`same_repo_path`]), like
//! favorites, recent and tabs.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use gitpoulpi_config::{Config, Profile, RepoAiInstructions, WorkspaceAi, WorkspaceDefault};
use serde::{Deserialize, Serialize};

use crate::{same_repo_path, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    /// A stable slug (`acme`): what `--workspace`, `GITPOULPI_WORKSPACE`
    /// and the per-workspace tab lists refer to. Renaming keeps it.
    pub id: String,
    pub name: String,
    /// A theme accent name; `None` = the default accent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default)]
    pub description: String,
    /// The `[[profiles]]` id commits are made as in this workspace's
    /// repositories. It may name nothing (the profile was deleted): that is
    /// not an error, it just stops applying (`Config::profile_for`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(default)]
    pub repos: Vec<PathBuf>,
    /// Each repository's `origin` URL when it was added, keyed by the path
    /// as written in `repos`. Empty values are not stored.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub remotes: BTreeMap<String, String>,
    /// Instructions for every AI action in this workspace's repositories,
    /// and how they combine with the global ones (`add` after them,
    /// `replace` instead). Empty text changes nothing.
    #[serde(default, skip_serializing_if = "RepoAiInstructions::is_default")]
    pub ai_instructions: RepoAiInstructions,
    /// The same per action (the action's snake_case key), combined with the
    /// global instructions of that action.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ai_overrides: BTreeMap<String, RepoAiInstructions>,
}

impl Workspace {
    pub fn contains(&self, path: &Path) -> bool {
        self.repos.iter().any(|r| same_repo_path(r, path))
    }

    /// The recorded origin URL of `path`, if one was.
    pub fn remote_of(&self, path: &Path) -> Option<&str> {
        self.repos
            .iter()
            .find(|r| same_repo_path(r, path))
            .and_then(|r| self.remotes.get(r.to_string_lossy().as_ref()))
            .map(String::as_str)
    }

    /// Adds `path` (and its origin URL when known) unless it is already
    /// there. A repository already listed keeps its place but learns a URL
    /// it did not have. Returns whether the path was new.
    pub fn add_repo(&mut self, path: &Path, remote: Option<&str>) -> bool {
        let existing = self.repos.iter().find(|r| same_repo_path(r, path)).cloned();
        let key = existing.clone().unwrap_or_else(|| path.to_path_buf());
        if let Some(url) = remote.filter(|u| !u.trim().is_empty()) {
            self.remotes
                .entry(key.to_string_lossy().into_owned())
                .or_insert_with(|| url.trim().to_string());
        }
        if existing.is_some() {
            return false;
        }
        self.repos.push(key);
        true
    }

    /// Removes `path` and its recorded URL. Returns whether it was there.
    pub fn remove_repo(&mut self, path: &Path) -> bool {
        let Some(at) = self.repos.iter().position(|r| same_repo_path(r, path)) else {
            return false;
        };
        let gone = self.repos.remove(at);
        self.remotes.remove(gone.to_string_lossy().as_ref());
        true
    }

    /// This workspace's AI instructions, empty entries left out.
    pub fn ai(&self) -> WorkspaceAi {
        WorkspaceAi {
            general: self.ai_instructions.clone(),
            actions: self
                .ai_overrides
                .iter()
                .filter(|(_, v)| !v.is_default())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

    /// What [`Config::profile_for`] needs of this workspace for `repo`.
    pub fn default_for(&self, repo: &Path) -> WorkspaceDefault<'_> {
        WorkspaceDefault {
            profile: self.profile.as_deref(),
            repo_is_member: self.contains(repo),
        }
    }

    /// The profile commits in `repo` are made as, when working from this
    /// workspace: its own default, else `general.active_profile`.
    pub fn profile_for<'c>(&self, config: &'c Config, repo: &Path) -> Option<&'c Profile> {
        config.profile_for(Some(self.default_for(repo)))
    }
}

/// `Acme API!` -> `acme-api`: ASCII letters and digits, single dashes, never
/// empty.
pub fn slugify(name: &str) -> String {
    let mut slug = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_end_matches('-').to_string();
    if slug.is_empty() {
        "workspace".to_string()
    } else {
        slug
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspaces {
    #[serde(default, rename = "workspace")]
    pub workspaces: Vec<Workspace>,
}

impl Workspaces {
    /// `<config dir>/workspaces.toml`.
    pub fn file_in(config_dir: &Path) -> PathBuf {
        config_dir.join("workspaces.toml")
    }

    /// A missing or corrupt file is an empty list, for the reason
    /// `FavoriteRepos::load` gives: a convenience file must never make the
    /// app unusable.
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(raw) => toml::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Written through a temporary file and a rename, so a window reading it
    /// while the CLI saves never sees half a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, toml::to_string_pretty(self)?)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Workspace> {
        self.workspaces.iter().find(|w| w.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Workspace> {
        self.workspaces.iter_mut().find(|w| w.id == id)
    }

    /// Every workspace `path` belongs to.
    pub fn containing<'a>(&'a self, path: &'a Path) -> impl Iterator<Item = &'a Workspace> {
        self.workspaces.iter().filter(move |w| w.contains(path))
    }

    /// The workspace whose AI instructions apply to `repo`: the active one
    /// (`active`, an id) when it contains the repository, else the first
    /// workspace that does.
    pub fn for_repo<'a>(&'a self, repo: &'a Path, active: Option<&str>) -> Option<&'a Workspace> {
        active
            .and_then(|id| self.get(id))
            .filter(|w| w.contains(repo))
            .or_else(|| self.containing(repo).next())
    }

    /// The AI instructions that apply to `repo` (see [`Self::for_repo`]);
    /// empty when no workspace has the repository.
    pub fn ai_for(&self, repo: &Path, active: Option<&str>) -> WorkspaceAi {
        self.for_repo(repo, active)
            .map(Workspace::ai)
            .unwrap_or_default()
    }

    /// A slug from `name` that no workspace uses yet (`acme`, `acme-2`...).
    pub fn free_id(&self, name: &str) -> String {
        let base = slugify(name);
        let mut id = base.clone();
        let mut n = 2;
        while self.get(&id).is_some() {
            id = format!("{base}-{n}");
            n += 1;
        }
        id
    }

    /// Appends a workspace named `name` and returns its id. A name no
    /// workspace has: the caller checks `find_by_name` when names must be
    /// unique (the CLI does).
    pub fn create(&mut self, name: &str) -> &mut Workspace {
        let id = self.free_id(name);
        self.workspaces.push(Workspace {
            id,
            name: name.trim().to_string(),
            ..Workspace::default()
        });
        self.workspaces.last_mut().expect("just pushed")
    }

    /// A workspace by its name, ignoring case.
    pub fn find_by_name(&self, name: &str) -> Option<&Workspace> {
        let name = name.trim();
        self.workspaces
            .iter()
            .find(|w| w.name.eq_ignore_ascii_case(name))
    }

    pub fn delete(&mut self, id: &str) -> bool {
        let before = self.workspaces.len();
        self.workspaces.retain(|w| w.id != id);
        self.workspaces.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(n: usize) -> PathBuf {
        PathBuf::from(format!("/repos/r{n}"))
    }

    #[test]
    fn slugs_are_stable_ascii_and_never_empty() {
        assert_eq!(slugify("Acme API!"), "acme-api");
        assert_eq!(slugify("  --Mixed__Case 42 "), "mixed-case-42");
        assert_eq!(slugify("日本語"), "workspace");
        assert_eq!(slugify(""), "workspace");
    }

    #[test]
    fn creating_gives_unique_ids_and_renaming_keeps_them() {
        let mut all = Workspaces::default();
        assert_eq!(all.create("Acme").id, "acme");
        assert_eq!(all.create("ACME").id, "acme-2");
        assert_eq!(all.create("acme").id, "acme-3");
        all.get_mut("acme").unwrap().name = "Renamed".into();
        assert_eq!(all.get("acme").unwrap().name, "Renamed");
        assert!(all.find_by_name("renamed").is_some());
        assert!(all.delete("acme-2"));
        assert!(!all.delete("acme-2"));
        // A freed id is reused.
        assert_eq!(all.create("Acme").id, "acme-2");
    }

    #[test]
    fn repos_join_by_exact_path_and_remember_their_origin() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        std::fs::create_dir_all(&real).unwrap();
        let mut w = Workspace::default();
        assert!(w.add_repo(&real, Some("git@h:o/r.git")));
        // Another spelling of the same directory is not a second repository
        // and does not replace the URL.
        assert!(!w.add_repo(&dir.path().join("real/"), Some("other")));
        assert_eq!(w.repos.len(), 1);
        assert!(w.contains(&real.join(".")));
        assert_eq!(
            w.remote_of(&dir.path().join("real/")),
            Some("git@h:o/r.git")
        );
        // A URL learned later is kept when none was.
        assert!(w.add_repo(&at(2), None));
        assert_eq!(w.remote_of(&at(2)), None);
        w.add_repo(&at(2), Some("u"));
        assert_eq!(w.remote_of(&at(2)), Some("u"));
        assert!(w.remove_repo(&real));
        assert!(!w.remove_repo(&real));
        assert!(w.remotes.keys().all(|k| k != &real.to_string_lossy()));
    }

    #[test]
    fn a_repository_may_belong_to_several_workspaces() {
        let mut all = Workspaces::default();
        all.create("A").add_repo(&at(1), None);
        all.create("B").add_repo(&at(1), None);
        all.create("C").add_repo(&at(2), None);
        let one = at(1);
        let ids: Vec<_> = all.containing(&one).map(|w| w.id.as_str()).collect();
        assert_eq!(ids, ["a", "b"]);
    }

    #[test]
    fn a_round_trip_through_the_file_preserves_everything() {
        let dir = tempfile::tempdir().unwrap();
        let file = Workspaces::file_in(dir.path());
        let mut all = Workspaces::default();
        let w = all.create("Acme");
        w.color = Some("teal".into());
        w.description = "the customer".into();
        w.profile = Some("work".into());
        w.add_repo(&at(1), Some("git@h:o/a.git"));
        w.add_repo(&at(2), None);
        all.create("Bare");
        all.save(&file).unwrap();
        assert_eq!(Workspaces::load(&file), all);
        let raw = std::fs::read_to_string(&file).unwrap();
        assert!(raw.contains("[[workspace]]"), "{raw}");
        assert!(!dir.path().join("workspaces.toml.tmp").exists());
    }

    #[test]
    fn ai_instructions_round_trip_and_stay_out_of_the_file_when_empty() {
        use gitpoulpi_config::RepoInstructionsMode;
        let dir = tempfile::tempdir().unwrap();
        let file = Workspaces::file_in(dir.path());
        let mut all = Workspaces::default();
        all.create("Plain");
        let w = all.create("Acme");
        w.ai_instructions = RepoAiInstructions {
            text: "Use the Acme glossary.\nBe terse.".into(),
            mode: RepoInstructionsMode::Replace,
        };
        w.ai_overrides.insert(
            "code_review".into(),
            RepoAiInstructions {
                text: "Flag missing tests.".into(),
                mode: RepoInstructionsMode::Add,
            },
        );
        all.save(&file).unwrap();
        assert_eq!(Workspaces::load(&file), all);
        let raw = std::fs::read_to_string(&file).unwrap();
        assert!(
            raw.contains("ai_instructions") && raw.contains("code_review"),
            "{raw}"
        );
        assert_eq!(raw.matches("ai_instructions").count(), 1, "{raw}");
        let ai = all.get("acme").unwrap().ai();
        assert_eq!(ai.general.mode, RepoInstructionsMode::Replace);
        assert_eq!(
            ai.for_action(gitpoulpi_config::AiAction::CodeReview).text,
            "Flag missing tests."
        );
        assert!(all.get("plain").unwrap().ai().is_empty());
        // An old file without the keys still loads.
        let old = "[[workspace]]\nid = \"a\"\nname = \"A\"\n";
        std::fs::write(&file, old).unwrap();
        assert!(Workspaces::load(&file).get("a").unwrap().ai().is_empty());
    }

    #[test]
    fn the_active_workspace_wins_when_it_has_the_repository_else_the_first_one() {
        let mut all = Workspaces::default();
        all.create("A").add_repo(&at(1), None);
        all.create("B").add_repo(&at(1), None);
        all.create("C").add_repo(&at(2), None);
        let id = |w: Option<&Workspace>| w.map(|w| w.id.clone());
        let one = at(1);
        assert_eq!(id(all.for_repo(&one, Some("b"))), Some("b".into()));
        assert_eq!(id(all.for_repo(&one, None)), Some("a".into()));
        // The active workspace does not hold the repository: the first that does.
        assert_eq!(id(all.for_repo(&one, Some("c"))), Some("a".into()));
        assert_eq!(id(all.for_repo(&one, Some("gone"))), Some("a".into()));
        assert_eq!(id(all.for_repo(&at(9), Some("a"))), None);
    }

    #[test]
    fn a_missing_or_corrupt_file_is_an_empty_list() {
        let dir = tempfile::tempdir().unwrap();
        let file = Workspaces::file_in(dir.path());
        assert!(Workspaces::load(&file).workspaces.is_empty());
        std::fs::write(&file, "this is [not toml").unwrap();
        assert!(Workspaces::load(&file).workspaces.is_empty());
    }

    #[test]
    fn the_workspace_profile_applies_only_inside_it() {
        let mut config = Config::default();
        config.profiles.push(Profile::new("work", "Work"));
        config.profiles.push(Profile::new("home", "Home"));
        config.general.active_profile = "home".into();
        let mut w = Workspace::default();
        w.add_repo(&at(1), None);
        w.profile = Some("work".into());
        let id = |p: Option<&Profile>| p.map(|p| p.id.clone());
        assert_eq!(id(w.profile_for(&config, &at(1))), Some("work".into()));
        assert_eq!(id(w.profile_for(&config, &at(2))), Some("home".into()));
        w.profile = Some("deleted".into());
        assert_eq!(id(w.profile_for(&config, &at(1))), Some("home".into()));
    }
}
