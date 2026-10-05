//! The code of a request through Gitea's / Forgejo's API, for review without
//! a clone: `pulls/N.diff` and `git/commits/SHA.diff` (a whole `git diff`
//! each, split per file), `pulls/N/commits`, and a file through the JSON
//! `contents` route (base64, because the transport hands bodies over as
//! text and a raw route would corrupt a binary file).

use serde::Deserialize;

use super::GiteaClient;
use crate::client::encode_component;
use crate::review::api_code::{
    base64_decode, encode_path, hunk_counts, summary_of, ApiCommit, ApiFileDiff,
};
use crate::review::invalid;
use crate::{Error, Result};

#[derive(Deserialize)]
struct CommitRow {
    #[serde(default)]
    sha: String,
    #[serde(default)]
    parents: Vec<ParentRow>,
    author: Option<super::Login>,
    commit: Option<CommitBody>,
}

#[derive(Deserialize)]
struct ParentRow {
    #[serde(default)]
    sha: String,
}

#[derive(Deserialize, Default)]
struct CommitBody {
    #[serde(default)]
    message: String,
    author: Option<Person>,
}

#[derive(Deserialize, Default)]
struct Person {
    #[serde(default)]
    name: String,
    #[serde(default)]
    date: String,
}

#[derive(Deserialize)]
struct ContentRow {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    encoding: Option<String>,
    #[serde(default)]
    size: u64,
}

/// A whole `git diff` as one [`ApiFileDiff`] per `diff --git` section.
/// Gitea sends every file whole, so nothing is ever `omitted`; a binary
/// file has no hunks.
pub(super) fn files_of(raw: &str) -> Vec<ApiFileDiff> {
    let mut sections: Vec<String> = Vec::new();
    for line in raw.lines() {
        if line.starts_with("diff --git ") {
            sections.push(String::new());
        }
        if let Some(current) = sections.last_mut() {
            current.push_str(line);
            current.push('\n');
        }
    }
    sections
        .iter()
        .filter_map(|section| {
            let diff = gitpoulpi_core::diff_text::parse_unified(section)
                .into_iter()
                .next()?;
            let previous_path = section
                .lines()
                .find_map(|l| l.strip_prefix("rename from "))
                .map(str::to_string);
            let (additions, deletions) = hunk_counts(&diff);
            Some(ApiFileDiff {
                diff,
                previous_path,
                additions,
                deletions,
                omitted: None,
            })
        })
        .collect()
}

impl GiteaClient {
    async fn diff_text(&self, url: String, what: &str) -> Result<String> {
        let response = self.transport.send(self.get(url)).await?;
        response.error_for_status(what)?;
        Ok(response.body)
    }

    pub(super) async fn review_diff(&self, project: &str, number: u64) -> Result<Vec<ApiFileDiff>> {
        let raw = self
            .diff_text(
                self.repo_url(project, &format!("pulls/{number}.diff")),
                &format!("reading the diff of a {} pull request", self.label()),
            )
            .await?;
        Ok(files_of(&raw))
    }

    pub(super) async fn review_commits(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ApiCommit>> {
        let rows: Vec<CommitRow> = self
            .pages(
                &self.repo_url(project, &format!("pulls/{number}/commits")),
                &format!("listing the commits of a {} pull request", self.label()),
            )
            .await?;
        // Servers disagree on the order; a commit has its parent listed
        // before it, so a first row whose parent is in the list is the
        // newest of a newest-first list.
        let newest_first = rows.first().is_some_and(|f| {
            f.parents
                .iter()
                .any(|p| rows.iter().any(|r| r.sha == p.sha))
        });
        let mut commits: Vec<ApiCommit> = rows
            .into_iter()
            .map(|r| {
                let body = r.commit.unwrap_or_default();
                let person = body.author.unwrap_or_default();
                ApiCommit {
                    sha: r.sha,
                    parents: r.parents.into_iter().map(|p| p.sha).collect(),
                    author: r.author.map(|a| a.login).unwrap_or(person.name),
                    date: person.date,
                    summary: summary_of(&body.message),
                }
            })
            .collect();
        if newest_first {
            commits.reverse();
        }
        Ok(commits)
    }

    pub(super) async fn review_commit_diff(
        &self,
        project: &str,
        sha: &str,
    ) -> Result<Vec<ApiFileDiff>> {
        let raw = self
            .diff_text(
                self.repo_url(
                    project,
                    &format!("git/commits/{}.diff", encode_component(sha)),
                ),
                &format!("reading a commit of a {} repository", self.label()),
            )
            .await?;
        Ok(files_of(&raw))
    }

    /// `contents/<path>?ref=`: JSON with the bytes in base64. Without a
    /// `ref` the server reads the default branch.
    pub(super) async fn review_file(
        &self,
        project: &str,
        path: &str,
        rev: Option<&str>,
    ) -> Result<Option<Vec<u8>>> {
        let mut url = self.repo_url(project, &format!("contents/{}", encode_path(path)));
        if let Some(rev) = rev {
            url.push_str(&format!("?ref={}", encode_component(rev)));
        }
        let response = self.transport.send(self.get(url)).await?;
        if response.status == 404 {
            return Ok(None);
        }
        response.error_for_status(&format!("reading a file of a {} repository", self.label()))?;
        // A directory answers with a list.
        let row: ContentRow = response
            .json()
            .map_err(|_| invalid("reading a file", format!("{path} is not a file")))?;
        match (row.encoding.as_deref(), row.content) {
            (Some("base64"), Some(content)) => base64_decode(&content)
                .map(Some)
                .ok_or_else(|| Error::Decode(format!("the content of {path} is not base64"))),
            _ => Err(invalid(
                "reading a file",
                format!(
                    "{path} is not served inline by the contents API ({} bytes)",
                    row.size
                ),
            )),
        }
    }
}
