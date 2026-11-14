//! "Worktree name": from a description of the task, a short worktree name
//! and a branch name. The prompt shows the repository's own branch names so
//! the answer follows its conventions (`feature/`, `fix/`, lowercase
//! kebab-case); the parser is strict about what becomes a git ref and a
//! folder name, because both end up on disk.

use serde::Deserialize;

use crate::{Error, Result};

/// The model's instructions: the answer is one JSON object.
pub const BASE_PROMPT: &str = "You name a git worktree and its branch. You get a description of \
    the work (a feature, a bug fix or any task) and some of the repository's existing branch \
    names. Answer with one JSON object only, no prose, no code fences: \
    {\"name\": \"<short-kebab-name>\", \"branch\": \"<branch-name>\"}. The name is two to four \
    lowercase words joined by hyphens (at most 30 characters) saying what the work is. The \
    branch is the same idea written the way the existing branches are (reuse their prefix such \
    as feature/ or fix/ and their lowercase kebab-case style when they have one; without any \
    convention, use a plain lowercase kebab-case name). Use only a-z, 0-9, hyphens and at most \
    one slash in the branch, never spaces or other punctuation.";

/// Most characters of a worktree name (the part after `worktree-`).
pub const MAX_NAME: usize = 30;
/// Most characters of a branch name.
pub const MAX_BRANCH: usize = 60;
/// Most branch names the prompt carries, and most characters of the
/// description it carries.
pub const MAX_BRANCHES: usize = 40;
pub const MAX_DESCRIPTION: usize = 2000;

/// The user message: the description and the repository's branch names.
pub fn prompt(description: &str, branches: &[String]) -> String {
    let description: String = description.trim().chars().take(MAX_DESCRIPTION).collect();
    let mut out = format!("Work to do:\n{description}\n\nExisting branches:\n");
    if branches.is_empty() {
        out.push_str("(none)\n");
    }
    for branch in branches.iter().take(MAX_BRANCHES) {
        out.push_str(&format!("- {branch}\n"));
    }
    out.push_str("\nName the worktree and its branch.");
    out
}

/// What the model proposed, both already valid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeNames {
    /// A lowercase kebab slug, at most [`MAX_NAME`] characters (the caller
    /// prefixes `worktree-` unless it has the word already).
    pub name: String,
    /// A valid git branch name, at most [`MAX_BRANCH`] characters.
    pub branch: String,
}

#[derive(Deserialize)]
struct Answer {
    name: String,
    branch: String,
}

/// A lowercase kebab slug of `text`: letters and digits kept (ASCII
/// only), anything else a single hyphen, cut to [`MAX_NAME`] on a word
/// boundary where possible.
pub fn slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.trim().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_matches('-');
    if out.len() <= MAX_NAME {
        return out.to_string();
    }
    let cut = &out[..MAX_NAME];
    // Back to the last whole word, unless that leaves nothing.
    let cut = match cut.rfind('-') {
        Some(at) if at > 0 && out.as_bytes()[MAX_NAME] != b'-' => &cut[..at],
        _ => cut,
    };
    cut.trim_matches('-').to_string()
}

/// Why `branch` cannot be used as a branch name, or `None` when it can:
/// the subset of git's rules the prompt asks for (ASCII letters, digits,
/// `-`, `_`, `.` and `/` between non-empty parts).
fn branch_problem(branch: &str) -> Option<&'static str> {
    if branch.is_empty() {
        return Some("the branch name is empty");
    }
    if branch.chars().count() > MAX_BRANCH {
        return Some("the branch name is too long");
    }
    if !branch
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "-_./".contains(c))
    {
        return Some("the branch name has characters a branch cannot use");
    }
    if branch.contains("..") || branch.ends_with('.') || branch.ends_with(".lock") {
        return Some("the branch name is not a valid git ref");
    }
    if branch
        .split('/')
        .any(|part| part.is_empty() || part.starts_with(['-', '.']) || part.ends_with(".lock"))
    {
        return Some("the branch name is not a valid git ref");
    }
    None
}

/// Asks `job` (an [`AiAction::WorktreeName`](gitpoulpi_config::AiAction)
/// job) to name a worktree and its branch for `description`; shared by the
/// TUI and the CLI.
#[cfg(feature = "network")]
pub async fn suggest(
    job: &crate::assist::Job,
    description: &str,
    branches: &[String],
) -> Result<WorktreeNames> {
    let answer = job.run(&prompt(description, branches)).await?;
    parse(&answer)
}

/// Reads the model's answer: a JSON object (fences or prose around it are
/// tolerated) with a `name` and a `branch`. The name is slugged; the
/// branch must already be valid and short, or the answer is refused.
pub fn parse(answer: &str) -> Result<WorktreeNames> {
    let start = answer
        .find('{')
        .ok_or_else(|| Error::Decode("no JSON object in the answer".into()))?;
    let end = answer
        .rfind('}')
        .filter(|end| *end > start)
        .ok_or_else(|| Error::Decode("no JSON object in the answer".into()))?;
    let parsed: Answer = serde_json::from_str(&answer[start..=end])
        .map_err(|e| Error::Decode(format!("not the expected object: {e}")))?;
    let branch = parsed.branch.trim().to_string();
    if let Some(problem) = branch_problem(&branch) {
        return Err(Error::Invalid(problem.into()));
    }
    let mut name = slug(&parsed.name);
    if name.is_empty() {
        // No usable name: the branch's last part says it as well.
        name = slug(branch.rsplit('/').next().unwrap_or(&branch));
    }
    if name.is_empty() {
        return Err(Error::Invalid("the worktree name is empty".into()));
    }
    Ok(WorktreeNames { name, branch })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_good_answer_gives_a_slug_and_a_branch() {
        let n = parse("{\"name\": \"Login Form Fix\", \"branch\": \"fix/login-form\"}").unwrap();
        assert_eq!(n.name, "login-form-fix");
        assert_eq!(n.branch, "fix/login-form");
    }

    #[test]
    fn fences_and_prose_around_the_object_are_tolerated() {
        let n = parse("Sure!\n```json\n{\"name\":\"a-b\",\"branch\":\"feature/a-b\"}\n```").unwrap();
        assert_eq!((n.name.as_str(), n.branch.as_str()), ("a-b", "feature/a-b"));
    }

    #[test]
    fn garbage_is_refused() {
        for answer in ["", "no idea", "{\"name\": 3}", "{\"name\":\"x\"}", "} {"] {
            assert!(matches!(parse(answer), Err(Error::Decode(_))), "{answer:?}");
        }
    }

    #[test]
    fn an_overlong_branch_is_refused_and_an_overlong_name_is_cut() {
        let long = "a".repeat(MAX_BRANCH + 1);
        let answer = format!("{{\"name\":\"x\",\"branch\":\"{long}\"}}");
        assert!(matches!(parse(&answer), Err(Error::Invalid(_))));
        let name = "word ".repeat(20);
        let answer = format!("{{\"name\":\"{name}\",\"branch\":\"ok\"}}");
        let n = parse(&answer).unwrap();
        assert!(n.name.len() <= MAX_NAME, "{}", n.name);
        assert!(!n.name.ends_with('-') && n.name.starts_with("word"));
    }

    #[test]
    fn a_branch_that_is_not_a_valid_ref_is_refused() {
        for branch in [
            "has space",
            "a..b",
            "/lead",
            "trail/",
            "a//b",
            "-dash",
            "x.lock",
            "a~b",
            "a:b",
            "a?b",
            "caf\u{e9}",
            "a.",
        ] {
            let answer = format!("{{\"name\":\"x\",\"branch\":\"{branch}\"}}");
            assert!(
                matches!(parse(&answer), Err(Error::Invalid(_))),
                "{branch:?} must be refused"
            );
        }
    }

    #[test]
    fn an_empty_name_falls_back_to_the_branchs_last_part() {
        let n = parse("{\"name\":\"!!!\",\"branch\":\"feature/Fast_Sync\"}").unwrap();
        assert_eq!(n.name, "fast-sync");
    }

    #[test]
    fn the_prompt_carries_the_description_and_the_branches() {
        let p = prompt("  fix the crash  ", &["feature/a".into(), "fix/b".into()]);
        assert!(p.contains("fix the crash") && p.contains("- feature/a") && p.contains("- fix/b"));
        assert!(prompt("x", &[]).contains("(none)"));
    }
}
