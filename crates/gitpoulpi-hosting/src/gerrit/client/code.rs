//! The code of a change through Gerrit's API, for review without a clone:
//! the files of a patch set (optionally against another patch set, `base`)
//! with each file's `/diff` JSON turned into hunks, the commit of a patch
//! set, the diff of a commit by sha, and a file's bytes (base64 body).
//!
//! Gerrit's diff is a list of sections (`a` lines only in the old side, `b`
//! only in the new, `ab` in both, `skip` for lines left out between two
//! hunks); this module writes them as unified-diff text and hands that to
//! [`gitpoulpi_core::diff_text::parse_hunks`], like the other hosts.

use std::collections::BTreeMap;

use gitpoulpi_core::domain::FileStatus;
use serde::Deserialize;

use super::{encode_component, GerritClient};
use crate::gerrit::map::{self, FileInfo};
use crate::review::api_code::{base64_decode, hunk_counts, ApiCommit, ApiFileDiff};
use crate::{Error, Result};

/// The lines of context around a change.
const CONTEXT: u32 = 3;

#[derive(Deserialize, Default)]
struct DiffInfo {
    #[serde(default)]
    content: Vec<Section>,
    #[serde(default)]
    binary: bool,
}

#[derive(Deserialize, Default)]
struct Section {
    #[serde(default)]
    a: Vec<String>,
    #[serde(default)]
    b: Vec<String>,
    #[serde(default)]
    ab: Vec<String>,
    #[serde(default)]
    skip: u32,
}

#[derive(Deserialize, Default)]
struct CommitInfo {
    #[serde(default)]
    commit: String,
    #[serde(default)]
    parents: Vec<map::CommitParent>,
    #[serde(default)]
    author: Person,
    #[serde(default)]
    subject: String,
}

#[derive(Deserialize, Default)]
struct Person {
    #[serde(default)]
    name: String,
    #[serde(default)]
    date: String,
}

/// An open hunk while the sections are walked.
struct Open {
    old_start: u32,
    new_start: u32,
    old_lines: u32,
    new_lines: u32,
    body: String,
}

impl Open {
    fn close(self, out: &mut String) {
        // A side without lines starts at the line before, as git prints it.
        let start = |start: u32, lines: u32| if lines == 0 { start - 1 } else { start };
        out.push_str(&format!(
            "@@ -{},{} +{},{} @@\n{}",
            start(self.old_start, self.old_lines),
            self.old_lines,
            start(self.new_start, self.new_lines),
            self.new_lines,
            self.body
        ));
    }
}

/// Gerrit's sections as unified-diff hunks (no file header).
fn unified(sections: &[Section]) -> String {
    let (mut old, mut new) = (1u32, 1u32);
    let mut out = String::new();
    let mut open: Option<Open> = None;
    for section in sections {
        if section.skip > 0 {
            if let Some(hunk) = open.take() {
                hunk.close(&mut out);
            }
            old += section.skip;
            new += section.skip;
            continue;
        }
        let hunk = open.get_or_insert_with(|| Open {
            old_start: old,
            new_start: new,
            old_lines: 0,
            new_lines: 0,
            body: String::new(),
        });
        for (mark, lines) in [(' ', &section.ab), ('-', &section.a), ('+', &section.b)] {
            for line in lines {
                hunk.body.push(mark);
                hunk.body.push_str(line);
                hunk.body.push('\n');
                if mark != '+' {
                    hunk.old_lines += 1;
                    old += 1;
                }
                if mark != '-' {
                    hunk.new_lines += 1;
                    new += 1;
                }
            }
        }
    }
    if let Some(hunk) = open {
        hunk.close(&mut out);
    }
    out
}

fn status_of(info: &FileInfo) -> FileStatus {
    match info.status.as_deref() {
        Some("A") | Some("C") => FileStatus::Added,
        Some("D") => FileStatus::Deleted,
        Some("R") => FileStatus::Renamed,
        _ => FileStatus::Modified,
    }
}

impl GerritClient {
    /// The files under `prefix` (`files/` of a revision or of a commit),
    /// each with its diff. `base` is a patch set number to compare with.
    async fn diff_files(
        &self,
        prefix: &str,
        base: Option<u32>,
        what: &str,
    ) -> Result<Vec<ApiFileDiff>> {
        let base = base.map_or_else(String::new, |n| format!("base={n}&"));
        let files: BTreeMap<String, FileInfo> = self
            .get_json(&format!("{prefix}/files/?{base}"), what)
            .await?;
        let mut out = Vec::new();
        // The magic files (`/COMMIT_MSG`, `/MERGE_LIST`) are not code.
        for (path, info) in files.into_iter().filter(|(p, _)| !p.starts_with('/')) {
            let diff: DiffInfo = self
                .get_json(
                    &format!(
                        "{prefix}/files/{}/diff?{base}context={CONTEXT}",
                        encode_component(&path)
                    ),
                    what,
                )
                .await?;
            let status = status_of(&info);
            let text = if diff.binary {
                String::new()
            } else {
                unified(&diff.content)
            };
            let file = gitpoulpi_core::diff_text::parse_hunks(&path, status, &text);
            let counted = hunk_counts(&file);
            out.push(ApiFileDiff {
                previous_path: info.old_path.filter(|p| !p.is_empty()),
                additions: if diff.binary {
                    info.lines_inserted
                } else {
                    counted.0
                },
                deletions: if diff.binary {
                    info.lines_deleted
                } else {
                    counted.1
                },
                diff: file,
                omitted: None,
            });
        }
        Ok(out)
    }

    /// `patch_set`: the one to show (the current by default); `compare`: the
    /// patch set to show it against (the parent commit by default).
    pub(super) async fn review_diff(
        &self,
        project: &str,
        number: u64,
        patch_set: Option<u32>,
        compare: Option<u32>,
    ) -> Result<Vec<ApiFileDiff>> {
        let revision = patch_set.map_or_else(|| "current".to_string(), |n| n.to_string());
        let prefix = format!(
            "/changes/{}/revisions/{revision}",
            Self::change_id(project, number)
        );
        self.diff_files(&prefix, compare, "reading the diff of a Gerrit change")
            .await
    }

    /// A change is one commit per patch set: the newest patch set's.
    pub(super) async fn review_commits(
        &self,
        project: &str,
        number: u64,
    ) -> Result<Vec<ApiCommit>> {
        let info: CommitInfo = self
            .get_json(
                &format!(
                    "/changes/{}/revisions/current/commit",
                    Self::change_id(project, number)
                ),
                "reading the commit of a Gerrit change",
            )
            .await?;
        Ok(vec![ApiCommit {
            sha: info.commit,
            parents: info.parents.into_iter().map(|p| p.commit).collect(),
            author: info.author.name,
            date: map::iso(&info.author.date),
            summary: info.subject,
        }])
    }

    pub(super) async fn review_commit_diff(
        &self,
        project: &str,
        sha: &str,
    ) -> Result<Vec<ApiFileDiff>> {
        let prefix = format!(
            "/projects/{}/commits/{}",
            encode_component(project.trim()),
            encode_component(sha)
        );
        self.diff_files(&prefix, None, "reading a commit of a Gerrit project")
            .await
    }

    /// `branches/<branch>/files/<path>/content` (a branch, `HEAD` by
    /// default) or `commits/<sha>/files/<path>/content` for a commit id:
    /// the body is the file in base64.
    pub(super) async fn review_file(
        &self,
        project: &str,
        path: &str,
        rev: Option<&str>,
    ) -> Result<Option<Vec<u8>>> {
        let rev = rev.map(str::trim).filter(|r| !r.is_empty());
        let scope = match rev {
            Some(r) if r.len() >= 7 && r.bytes().all(|c| c.is_ascii_hexdigit()) => {
                format!("commits/{}", encode_component(r))
            }
            other => format!("branches/{}", encode_component(other.unwrap_or("HEAD"))),
        };
        let url = format!(
            "{}/projects/{}/{scope}/files/{}/content",
            self.api(),
            encode_component(project.trim()),
            encode_component(path.trim_start_matches('/'))
        );
        let response = self
            .transport
            .send(self.authed(crate::http::HttpRequest::get(url)))
            .await?;
        if response.status == 404 {
            return Ok(None);
        }
        response.error_for_status("reading a file of a Gerrit project")?;
        base64_decode(response.body.trim())
            .map(Some)
            .ok_or_else(|| Error::Decode(format!("the content of {path} is not base64")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(a: &[&str], b: &[&str], ab: &[&str], skip: u32) -> Section {
        let own = |l: &[&str]| l.iter().map(|s| s.to_string()).collect();
        Section {
            a: own(a),
            b: own(b),
            ab: own(ab),
            skip,
        }
    }

    #[test]
    fn sections_become_hunks_with_their_line_numbers_and_skips_split_them() {
        let text = unified(&[
            section(&[], &[], &["x", "y"], 0),
            section(&["old"], &["new1", "new2"], &[], 0),
            section(&[], &[], &["z"], 0),
            section(&[], &[], &[], 40),
            section(&[], &[], &["q"], 0),
            section(&[], &["tail"], &[], 0),
        ]);
        assert_eq!(
            text,
            "@@ -1,4 +1,5 @@\n x\n y\n-old\n+new1\n+new2\n z\n@@ -45,1 +46,2 @@\n q\n+tail\n"
        );
    }

    #[test]
    fn a_side_without_lines_starts_at_the_line_before() {
        assert_eq!(
            unified(&[section(&[], &["a", "b"], &[], 0)]),
            "@@ -0,0 +1,2 @@\n+a\n+b\n"
        );
        assert_eq!(
            unified(&[section(&["a"], &[], &[], 0)]),
            "@@ -1,1 +0,0 @@\n-a\n"
        );
    }
}
