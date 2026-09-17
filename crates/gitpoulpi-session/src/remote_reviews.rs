//! What the in-app review keeps for repositories that are **not cloned**
//! (a request reviewed through the host's API): the pending comments and
//! the ✦ draft of each request, and the per-project AI review defaults.
//!
//! A cloned repository keeps these in its own state file
//! (`RepoState::reviews`); with no repository there is no such file, so
//! they live in one file of the user: `<config dir>/remote-reviews.toml`,
//! shared by every session on the machine and keyed by `host/project`
//! (`github.com/owner/name`).
//!
//! Best-effort I/O, like `radar.toml`: a missing or corrupt file is an empty
//! state, and a save that fails loses a draft, never the window.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use gitpoulpi_config::RepoAiReview;

use crate::{Result, ReviewDraft};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteReviews {
    /// `host/project` -> draft key (`ReviewDraft::key`) -> draft.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub drafts: BTreeMap<String, BTreeMap<String, ReviewDraft>>,
    /// `host/project` -> AI review defaults.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ai_review: BTreeMap<String, RepoAiReview>,
}

impl RemoteReviews {
    /// `<config dir>/remote-reviews.toml`.
    pub fn file_in(config_dir: &Path) -> PathBuf {
        config_dir.join("remote-reviews.toml")
    }

    /// The key of a project: its host (without scheme, port or case) and
    /// its path.
    pub fn project_key(host_url: &str, project: &str) -> String {
        let rest = host_url.split_once("://").map_or(host_url, |(_, r)| r);
        let rest = rest.split('/').next().unwrap_or(rest);
        let host = rest.split(':').next().unwrap_or(rest).to_ascii_lowercase();
        let project = project.trim_matches('/').trim_end_matches(".git");
        format!("{host}/{project}")
    }

    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(raw) => toml::from_str(&raw).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Temp file + rename, so a crash never leaves half a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, toml::to_string_pretty(self)?)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn draft(&self, project: &str, key: &str) -> Option<&ReviewDraft> {
        self.drafts.get(project)?.get(key)
    }

    /// Stores `draft`, or forgets the request's entry when it is empty (and
    /// the project's when that leaves it with none).
    pub fn set_draft(&mut self, project: &str, key: &str, draft: ReviewDraft) {
        if draft.is_empty() {
            if let Some(of_project) = self.drafts.get_mut(project) {
                of_project.remove(key);
                if of_project.is_empty() {
                    self.drafts.remove(project);
                }
            }
        } else {
            self.drafts
                .entry(project.to_string())
                .or_default()
                .insert(key.to_string(), draft);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DraftComment;

    fn draft() -> ReviewDraft {
        ReviewDraft {
            comments: vec![DraftComment {
                path: "src/a.rs".into(),
                previous_path: None,
                line: 4,
                side: "new".into(),
                start_line: None,
                other_line: None,
                body: "why?".into(),
            }],
            viewed: vec!["src/a.rs".into()],
            head_sha: "abc".into(),
            ai: None,
        }
    }

    #[test]
    fn drafts_and_ai_defaults_round_trip_per_host_and_project() {
        let dir = tempfile::tempdir().unwrap();
        let path = RemoteReviews::file_in(dir.path());
        assert_eq!(RemoteReviews::load(&path), RemoteReviews::default());
        let project = RemoteReviews::project_key("https://GitHub.com:443/", "o/r.git");
        assert_eq!(project, "github.com/o/r");
        let key = ReviewDraft::key("corp", "o/r", 7);
        let mut file = RemoteReviews::default();
        file.set_draft(&project, &key, draft());
        file.ai_review.insert(
            project.clone(),
            RepoAiReview {
                include_claude_md: Some(true),
                include_agents_md: None,
                run_in_repository: None,
                extra_file: Some("docs/REVIEW.md".into()),
                no_extra_file: Some(true),
            },
        );
        file.save(&path).unwrap();
        let back = RemoteReviews::load(&path);
        assert_eq!(back, file);
        assert_eq!(back.draft(&project, &key).unwrap().comments[0].line, 4);
        assert_eq!(back.ai_review[&project].include_claude_md, Some(true));
        assert_eq!(back.ai_review[&project].include_agents_md, None);
    }

    #[test]
    fn an_empty_draft_removes_its_entry_and_an_empty_project() {
        let mut file = RemoteReviews::default();
        file.set_draft("h/p", "k", draft());
        file.set_draft("h/p", "k", ReviewDraft::default());
        assert!(file.drafts.is_empty());
    }

    #[test]
    fn a_corrupt_file_is_an_empty_state() {
        let dir = tempfile::tempdir().unwrap();
        let path = RemoteReviews::file_in(dir.path());
        std::fs::write(&path, "drafts = [[[").unwrap();
        assert_eq!(RemoteReviews::load(&path), RemoteReviews::default());
    }
}
