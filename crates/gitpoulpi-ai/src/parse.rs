//! Turning a model's text into something the app can use.

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::{Error, Result};

pub use crate::rules_answer::rewrite_rules;

mod sequence_answer;
pub use sequence_answer::{sequence_draft, SequenceProposal, StepProblem};

/// A commit message split the way the composer holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitMessage {
    pub summary: String,
    pub description: String,
}

impl CommitMessage {
    /// Back to one string, `git commit -m` style.
    pub fn full(&self) -> String {
        if self.description.is_empty() {
            self.summary.clone()
        } else {
            format!("{}\n\n{}", self.summary, self.description)
        }
    }
}

/// A pull request title and body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullRequestText {
    pub title: String,
    pub description: String,
}

/// One commit of a composer plan.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlannedCommit {
    pub message: String,
    pub files: Vec<String>,
}

/// Removes a wrapping markdown code fence, if there is one.
fn strip_fence(text: &str) -> &str {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix("```") {
        // Drop the info string (```text, ```json, ...) up to the newline.
        let rest = rest.split_once('\n').map(|(_, r)| r).unwrap_or(rest);
        if let Some(body) = rest.trim_end().strip_suffix("```") {
            return body.trim();
        }
    }
    text
}

fn strip_quotes(line: &str) -> &str {
    let line = line.trim();
    for (open, close) in [('"', '"'), ('\'', '\''), ('`', '`')] {
        if line.len() > 1 && line.starts_with(open) && line.ends_with(close) {
            return line[1..line.len() - 1].trim();
        }
    }
    line
}

/// Summary = the first non-empty line; description = whatever follows,
/// without the blank separator. Fences, wrapping quotes and a leading
/// "Commit message:" label are tolerated because models add them
/// regardless of instructions.
pub fn commit_message(answer: &str) -> Result<CommitMessage> {
    let body = strip_fence(answer);
    let mut lines = body.lines().skip_while(|l| l.trim().is_empty());
    let first = lines.next().ok_or(Error::Empty)?;
    let first = strip_quotes(first);
    let first = first
        .strip_prefix("Commit message:")
        .or_else(|| first.strip_prefix("commit message:"))
        .map(str::trim)
        .unwrap_or(first);
    let summary = strip_quotes(first).to_string();
    if summary.is_empty() {
        return Err(Error::Empty);
    }
    let rest: Vec<&str> = lines.collect();
    let description = rest.join("\n").trim().to_string();
    Ok(CommitMessage {
        summary,
        description,
    })
}

/// A stash name: the first non-empty line, unquoted.
pub fn stash_message(answer: &str) -> Result<String> {
    Ok(commit_message(answer)?.summary)
}

/// Title = first non-empty line (a leading `#`/`Title:` is dropped),
/// description = the rest.
pub fn pull_request(answer: &str) -> Result<PullRequestText> {
    let body = strip_fence(answer);
    let mut lines = body.lines().skip_while(|l| l.trim().is_empty());
    let first = lines.next().ok_or(Error::Empty)?;
    let title = first
        .trim()
        .trim_start_matches('#')
        .trim()
        .trim_start_matches("Title:")
        .trim();
    let title = strip_quotes(title).to_string();
    if title.is_empty() {
        return Err(Error::Empty);
    }
    let rest: Vec<&str> = lines.collect();
    Ok(PullRequestText {
        title,
        description: rest.join("\n").trim().to_string(),
    })
}

/// Parses a composer plan and validates it against the real change set:
/// every changed file assigned to exactly one commit, no unknown files,
/// no empty commit, no empty message.
pub fn composer_plan(answer: &str, changed: &[String]) -> Result<Vec<PlannedCommit>> {
    let body = strip_fence(answer);
    let start = body
        .find('[')
        .ok_or_else(|| Error::Decode("no JSON array in the answer".into()))?;
    let end = body
        .rfind(']')
        .ok_or_else(|| Error::Decode("no JSON array in the answer".into()))?;
    if end < start {
        return Err(Error::Decode("no JSON array in the answer".into()));
    }
    let plan: Vec<PlannedCommit> =
        serde_json::from_str(&body[start..=end]).map_err(|e| Error::Decode(e.to_string()))?;
    if plan.is_empty() {
        return Err(Error::Invalid("the plan has no commit".into()));
    }
    let known: BTreeSet<&str> = changed.iter().map(String::as_str).collect();
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for commit in &plan {
        if commit.message.trim().is_empty() {
            return Err(Error::Invalid("a planned commit has no message".into()));
        }
        if commit.files.is_empty() {
            return Err(Error::Invalid(format!(
                "the planned commit {:?} has no file",
                commit.message
            )));
        }
        for file in &commit.files {
            if !known.contains(file.as_str()) {
                return Err(Error::Invalid(format!(
                    "the plan mentions {file}, which is not a changed file"
                )));
            }
            if !seen.insert(file.as_str()) {
                return Err(Error::Invalid(format!(
                    "the plan puts {file} in more than one commit"
                )));
            }
        }
    }
    let missing: Vec<&str> = known.difference(&seen).copied().collect();
    if !missing.is_empty() {
        return Err(Error::Invalid(format!(
            "the plan leaves out: {}",
            missing.join(", ")
        )));
    }
    Ok(plan)
}

/// One commit of a split plan: a message and the ids (`H1`, `F2`, ...,
/// trimmed and upper-cased) of the pieces it takes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PlannedSplit {
    pub message: String,
    pub pieces: Vec<String>,
}

/// Parses a split plan and checks it against the piece `ids` the prompt
/// offered: the array is found among fences or prose, every id is used
/// exactly once, no unknown id, no empty message or commit, at least two
/// commits. Order is kept (oldest commit first).
pub fn split_plan(answer: &str, ids: &[String]) -> Result<Vec<PlannedSplit>> {
    let body = strip_fence(answer);
    let end = body
        .rfind(']')
        .ok_or_else(|| Error::Decode("no JSON array in the answer".into()))?;
    let mut parsed = None;
    let mut first_error = None;
    // Prose may hold its own brackets ("see [1]"): try each opening one.
    for (start, _) in body[..end].match_indices('[') {
        match serde_json::from_str::<Vec<PlannedSplit>>(&body[start..=end]) {
            Ok(plan) => {
                parsed = Some(plan);
                break;
            }
            Err(e) => {
                first_error.get_or_insert(e);
            }
        }
    }
    let mut plan = parsed.ok_or_else(|| match first_error {
        Some(e) => Error::Decode(e.to_string()),
        None => Error::Decode("no JSON array in the answer".into()),
    })?;
    if plan.len() < 2 {
        return Err(Error::Invalid("the plan needs at least two commits".into()));
    }
    let known: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for commit in &mut plan {
        if commit.message.trim().is_empty() {
            return Err(Error::Invalid("a planned commit has no message".into()));
        }
        if commit.pieces.is_empty() {
            return Err(Error::Invalid(format!(
                "the planned commit {:?} has no piece",
                commit.message
            )));
        }
        for id in &mut commit.pieces {
            *id = id.trim().to_uppercase();
            if !known.contains(id.as_str()) {
                return Err(Error::Invalid(format!(
                    "the plan mentions {id}, which is not a piece"
                )));
            }
            if !seen.insert(id.clone()) {
                return Err(Error::Invalid(format!(
                    "the plan puts {id} in more than one commit"
                )));
            }
        }
    }
    let missing: Vec<&str> = ids
        .iter()
        .map(String::as_str)
        .filter(|id| !seen.contains(*id))
        .collect();
    if !missing.is_empty() {
        return Err(Error::Invalid(format!(
            "the plan leaves out: {}",
            missing.join(", ")
        )));
    }
    Ok(plan)
}

/// The header a script gets when the model wrote none.
pub const DEFAULT_SCRIPT_HEADER: &str = "--! name: AI script\n--! kinds: commit\n";

/// Extracts the Lua script from an answer: the code block that defines
/// `rewrite_commit` (fenced or not, with prose around), with the default
/// header added when the model left it out. It does not run the script: the
/// daemon's check does, and a compile error is sent back once.
pub fn rewrite_script(answer: &str) -> Result<String> {
    let mut blocks: Vec<String> = Vec::new();
    let mut open: Option<Vec<&str>> = None;
    for line in answer.lines() {
        if line.trim_start().starts_with("```") {
            match open.take() {
                Some(body) => blocks.push(body.join("\n")),
                None => open = Some(Vec::new()),
            }
        } else if let Some(body) = open.as_mut() {
            body.push(line);
        }
    }
    // An unclosed fence is a block cut short; keep what there is.
    if let Some(body) = open {
        blocks.push(body.join("\n"));
    }
    let defines = |t: &str| t.contains("rewrite_commit");
    let mut code = blocks.into_iter().find(|b| defines(b));
    if code.is_none() && !answer.contains("```") && defines(answer) {
        // No fence: the script starts at its first header or code line.
        let lines: Vec<&str> = answer.lines().collect();
        let start = lines
            .iter()
            .position(|l| {
                let l = l.trim_start();
                l.starts_with("--") || l.starts_with("function ") || l.starts_with("local ")
            })
            .unwrap_or(0);
        code = Some(lines[start..].join("\n"));
    }
    let code = code.ok_or_else(|| {
        if answer.trim().is_empty() {
            Error::Empty
        } else {
            Error::Invalid("the answer has no script that defines rewrite_commit".into())
        }
    })?;
    let mut code = code.trim_matches('\n').trim_end().to_string();
    if !code.lines().any(|l| l.trim_start().starts_with("--!")) {
        code = format!("{DEFAULT_SCRIPT_HEADER}{code}");
    }
    code.push('\n');
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_answer_splits_into_summary_and_description() {
        let m = commit_message("feat: add x\n\nDoes the thing.\nAnd more.\n").unwrap();
        assert_eq!(m.summary, "feat: add x");
        assert_eq!(m.description, "Does the thing.\nAnd more.");
        assert_eq!(m.full(), "feat: add x\n\nDoes the thing.\nAnd more.");
    }

    #[test]
    fn fences_quotes_and_labels_are_tolerated() {
        let m = commit_message("```text\n\"fix: y\"\n\nbody\n```").unwrap();
        assert_eq!(m.summary, "fix: y");
        assert_eq!(m.description, "body");
        let m = commit_message("Commit message: fix: z").unwrap();
        assert_eq!(m.summary, "fix: z");
        assert!(m.description.is_empty());
    }

    #[test]
    fn an_empty_answer_is_an_error() {
        assert_eq!(commit_message("  \n "), Err(Error::Empty));
        assert_eq!(pull_request(""), Err(Error::Empty));
    }

    #[test]
    fn a_stash_name_is_one_line() {
        assert_eq!(
            stash_message("\"WIP: parser\"\nextra").unwrap(),
            "WIP: parser"
        );
    }

    #[test]
    fn a_pull_request_has_a_title_and_a_body() {
        let p = pull_request("# Add login\n\n## Summary\n- x").unwrap();
        assert_eq!(p.title, "Add login");
        assert_eq!(p.description, "## Summary\n- x");
    }

    fn files(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_valid_plan_parses_even_inside_a_fence_with_prose() {
        let answer = "Here you go:\n```json\n[{\"message\":\"a\",\"files\":[\"x.rs\"]},\
                      {\"message\":\"b\",\"files\":[\"y.rs\",\"z.rs\"]}]\n```";
        let plan = composer_plan(answer, &files(&["x.rs", "y.rs", "z.rs"])).unwrap();
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[1].files, ["y.rs", "z.rs"]);
    }

    #[test]
    fn a_plan_that_does_not_cover_the_change_set_exactly_is_rejected() {
        let changed = files(&["x.rs", "y.rs"]);
        let one = |m: &str, f: &str| format!("[{{\"message\":\"{m}\",\"files\":[{f}]}}]");
        // Leaves y.rs out.
        assert!(composer_plan(&one("a", "\"x.rs\""), &changed).is_err());
        // Unknown file.
        assert!(composer_plan(&one("a", "\"x.rs\",\"y.rs\",\"w.rs\""), &changed).is_err());
        // Duplicate across commits.
        let dup = "[{\"message\":\"a\",\"files\":[\"x.rs\",\"y.rs\"]},{\"message\":\"b\",\"files\":[\"y.rs\"]}]";
        assert!(composer_plan(dup, &changed).is_err());
        // Empty message, empty file list, not JSON.
        assert!(composer_plan(&one(" ", "\"x.rs\",\"y.rs\""), &changed).is_err());
        assert!(composer_plan("[{\"message\":\"a\",\"files\":[]}]", &changed).is_err());
        assert!(matches!(
            composer_plan("no json", &changed),
            Err(Error::Decode(_))
        ));
    }
}
