//! What the in-app code review keeps per repository between runs: the
//! comments written but not yet submitted (a pending review) and the files
//! the reviewer ticked as viewed. Lives in `RepoState::reviews`, keyed by
//! `<integration>|<project>#<number>`, so a restart loses nothing.
//!
//! The comment shape mirrors `gitpoulpi_hosting::review::PendingComment`
//! (this crate does not depend on the hosting crate); the review view
//! converts both ways.

use serde::{Deserialize, Serialize};

/// A line comment waiting for the review to be submitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftComment {
    /// The path at the head.
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_path: Option<String>,
    /// The (last) line commented.
    pub line: u32,
    /// `old` or `new`: which column of the diff `line` is in.
    pub side: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other_line: Option<u32>,
    pub body: String,
}

/// Where an accepted ✦ suggestion becomes a comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiPlace {
    /// `old` or `new`.
    pub side: String,
    pub line: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other_line: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
}

/// One open ✦ suggestion of the draft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiItem {
    pub path: String,
    #[serde(default)]
    pub line: u32,
    /// `info`, `warning` or `issue`.
    pub severity: String,
    pub body: String,
    #[serde(default)]
    pub anchored: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<AiPlace>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub prefix: String,
}

/// The model's full review draft (S6): the text to submit, the verdict it
/// recommends and the suggestions the reviewer has not decided on yet.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiDraft {
    #[serde(default)]
    pub body: String,
    /// `approve`, `request-changes` or `comment`.
    #[serde(default)]
    pub verdict: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub items: Vec<AiItem>,
    /// The head commit the draft was written against; a different head
    /// marks it stale.
    #[serde(default)]
    pub head_sha: String,
}

/// Where the reviewer stood in the view when the window ended with the
/// review open (`ReviewResume`). Row numbers are of the rendered diff, which
/// is rebuilt the same for the same request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumePos {
    /// `overview`, `files` or `commits`.
    #[serde(default)]
    pub tab: String,
    /// The keyboard was on the diff, not the list.
    #[serde(default)]
    pub diff_focus: bool,
    /// The Files list's file (by path).
    #[serde(default)]
    pub file: String,
    /// The Commits list's commit index.
    #[serde(default)]
    pub commit: usize,
    #[serde(default)]
    pub cursor: usize,
    #[serde(default)]
    pub offset: usize,
}

/// A review that was in progress (the view open, not closed by the user and
/// not submitted) when its window ended: the tab is reopened on it.
/// `RepoState::open_review` for a cloned repository, `RemoteReviews::open_review`
/// for one reviewed through the host's API (which needs the request's
/// own fields to build the view with no repository list).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewResume {
    pub integration: String,
    pub project: String,
    pub number: u64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub label: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub title: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub web_url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source_branch: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target_branch: String,
    #[serde(default)]
    pub pos: ResumePos,
}

/// One request's local review state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewDraft {
    #[serde(default)]
    pub comments: Vec<DraftComment>,
    /// Paths ticked as viewed.
    #[serde(default)]
    pub viewed: Vec<String>,
    /// The head commit the viewed ticks and comments were made against.
    #[serde(default)]
    pub head_sha: String,
    /// The ✦ review draft, when the model wrote one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai: Option<AiDraft>,
}

impl ReviewDraft {
    pub fn is_empty(&self) -> bool {
        self.comments.is_empty() && self.viewed.is_empty() && self.ai.is_none()
    }

    /// The key a request's draft is stored under.
    pub fn key(integration: &str, project: &str, number: u64) -> String {
        format!("{integration}|{project}#{number}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RepoState;

    #[test]
    fn a_draft_survives_a_save_and_a_load_and_an_old_file_has_none() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("repo-state.toml");
        std::fs::write(&path, "default_remote = \"origin\"\n").unwrap();
        let mut state = RepoState::load(&path).unwrap();
        assert!(state.reviews.is_empty(), "files from before have no drafts");
        let key = ReviewDraft::key("corp", "g/p", 9);
        state.reviews.insert(
            key.clone(),
            ReviewDraft {
                comments: vec![DraftComment {
                    path: "src/a.rs".into(),
                    previous_path: None,
                    line: 12,
                    side: "new".into(),
                    start_line: Some(10),
                    other_line: None,
                    body: "why?".into(),
                }],
                viewed: vec!["src/a.rs".into()],
                head_sha: "abc".into(),
                ai: Some(AiDraft {
                    body: "Fix the loop.".into(),
                    verdict: "request-changes".into(),
                    summary: "s".into(),
                    items: vec![AiItem {
                        path: "src/a.rs".into(),
                        line: 3,
                        severity: "issue".into(),
                        body: "bug".into(),
                        anchored: true,
                        place: Some(AiPlace {
                            side: "new".into(),
                            line: 3,
                            other_line: None,
                            start_line: None,
                        }),
                        prefix: String::new(),
                    }],
                    head_sha: "abc".into(),
                }),
            },
        );
        state.save(&path).unwrap();
        let back = RepoState::load(&path).unwrap();
        assert_eq!(back.reviews[&key].comments[0].start_line, Some(10));
        assert_eq!(back.reviews[&key].viewed, vec!["src/a.rs".to_string()]);
        let ai = back.reviews[&key].ai.as_ref().unwrap();
        assert_eq!(ai.verdict, "request-changes");
        assert_eq!(ai.items[0].place.as_ref().unwrap().line, 3);
        assert_eq!(back.default_remote.as_deref(), Some("origin"));
    }
}
