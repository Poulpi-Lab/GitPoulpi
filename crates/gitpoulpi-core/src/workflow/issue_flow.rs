//! Host-driven flows, the pure half: the issue made from the template, the
//! name a host or bot gives the branch of an issue, and which branches
//! belong to which issue. Talking to the host is `gitpoulpi-commands`'
//! business (the daemon has no hosting credentials).

use std::time::Duration;

use super::{BranchPattern, IssueFlow, PatternError};

/// The name the issue's branch gets when the workflow does not say.
pub const DEFAULT_BRANCH_FROM_ISSUE: &str = "{type}/{issue}-{slug}";

/// How long `wait` polls by default (seconds).
pub const DEFAULT_WAIT_SECS: u64 = 600;

/// What `wait` is waiting for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WaitFor {
    Branch,
    MergeRequest,
    #[default]
    BranchOrMergeRequest,
}

impl WaitFor {
    /// `branch`, `mr` / `merge-request` / `pr`, `branch-or-mr` (anything
    /// else, and nothing, is the default).
    pub fn parse(word: &str) -> WaitFor {
        match word.trim().to_ascii_lowercase().as_str() {
            "branch" => WaitFor::Branch,
            "mr" | "pr" | "merge-request" | "pull-request" => WaitFor::MergeRequest,
            _ => WaitFor::BranchOrMergeRequest,
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            WaitFor::Branch => "branch",
            WaitFor::MergeRequest => "mr",
            WaitFor::BranchOrMergeRequest => "branch-or-mr",
        }
    }

    pub fn wants_branch(self) -> bool {
        !matches!(self, WaitFor::MergeRequest)
    }

    pub fn wants_request(self) -> bool {
        !matches!(self, WaitFor::Branch)
    }
}

impl IssueFlow {
    pub fn wait_kind(&self) -> WaitFor {
        self.wait_for
            .as_deref()
            .map(WaitFor::parse)
            .unwrap_or_default()
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs.unwrap_or(DEFAULT_WAIT_SECS).max(1))
    }

    pub fn branch_pattern_source(&self) -> &str {
        self.branch_from_issue
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(DEFAULT_BRANCH_FROM_ISSUE)
    }

    /// Whether the host should also open the merge request (default yes).
    pub fn wants_merge_request(&self) -> bool {
        self.merge_request.unwrap_or(true)
    }
}

/// The title, description and labels of the issue to create.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct IssueDraft {
    pub title: String,
    pub body: String,
    pub labels: Vec<String>,
    pub assignees: Vec<String>,
}

fn fill(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (name, value) in vars {
        out = out.replace(&format!("{{{name}}}"), value);
    }
    out
}

/// The issue of type `type_id` titled `title`: the configured title and
/// label templates and the description template's text, `{type}` `{title}`
/// `{user}` `{date}` replaced. Without a title template the title is
/// used as typed.
pub fn issue_draft(
    flow: &IssueFlow,
    type_id: &str,
    title: &str,
    template_text: Option<&str>,
    user: &str,
    date: &str,
) -> IssueDraft {
    let vars = [
        ("type", type_id),
        ("title", title.trim()),
        ("user", user),
        ("date", date),
    ];
    let title = match flow.issue_title.as_deref().filter(|t| !t.trim().is_empty()) {
        Some(template) => fill(template, &vars),
        None => title.trim().to_string(),
    };
    IssueDraft {
        title,
        body: template_text.map(|t| fill(t, &vars)).unwrap_or_default(),
        labels: flow
            .issue_labels
            .iter()
            .map(|l| fill(l, &vars))
            .filter(|l| !l.trim().is_empty())
            .collect(),
        assignees: flow.assignees.clone(),
    }
}

/// The pattern of the branch a host or bot makes for an issue of `type_id`
/// (`{type}` is replaced by the id, the rest are branch pattern tokens).
pub fn issue_branch_pattern(
    flow: &IssueFlow,
    type_id: &str,
) -> Result<BranchPattern, PatternError> {
    BranchPattern::parse(&flow.branch_pattern_source().replace("{type}", type_id))
}

/// The branches (of `branches`, remote names without the remote) that belong
/// to issue `number` of one of `type_ids`: they match the branch pattern
/// with `{issue}` equal to the number (or `{issue_key}` ending `-number`).
pub fn branches_for_issue(
    flow: &IssueFlow,
    type_ids: &[&str],
    branches: &[String],
    number: u64,
) -> Vec<String> {
    let wanted = number.to_string();
    let suffix = format!("-{number}");
    let patterns: Vec<BranchPattern> = if flow.branch_pattern_source().contains("{type}") {
        type_ids
            .iter()
            .filter_map(|t| issue_branch_pattern(flow, t).ok())
            .collect()
    } else {
        issue_branch_pattern(flow, "").ok().into_iter().collect()
    };
    let mut out: Vec<String> = Vec::new();
    for branch in branches {
        let hit = patterns.iter().any(|p| {
            p.parse_name(branch).is_some_and(|f| {
                f.get("issue").is_some_and(|v| *v == wanted)
                    || f.get("issue_key").is_some_and(|k| k.ends_with(&suffix))
            })
        });
        if hit && !out.contains(branch) {
            out.push(branch.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flow() -> IssueFlow {
        IssueFlow {
            issue_title: Some("[{type}] {title}".into()),
            issue_labels: vec!["workflow::{type}".into(), "bot".into()],
            ..IssueFlow::default()
        }
    }

    #[test]
    fn the_draft_fills_the_templates() {
        let d = issue_draft(
            &flow(),
            "feature",
            " Login page ",
            Some("Type: {type}\nBy {user} on {date}\n{title}"),
            "sg",
            "2026-10-16",
        );
        assert_eq!(d.title, "[feature] Login page");
        assert_eq!(d.labels, ["workflow::feature", "bot"]);
        assert_eq!(d.body, "Type: feature\nBy sg on 2026-10-16\nLogin page");
    }

    #[test]
    fn without_a_title_template_the_title_is_as_typed() {
        let d = issue_draft(&IssueFlow::default(), "bug", "Crash", None, "", "");
        assert_eq!(d.title, "Crash");
        assert_eq!(d.body, "");
        assert!(d.labels.is_empty());
    }

    #[test]
    fn branches_are_matched_by_issue_number_not_by_prefix() {
        let f = IssueFlow::default();
        let branches: Vec<String> = [
            "feature/12-login-page",
            "feature/123-other",
            "feature/12",
            "bugfix/12-crash",
            "main",
        ]
        .map(String::from)
        .to_vec();
        let found = branches_for_issue(&f, &["feature", "bugfix"], &branches, 12);
        assert_eq!(found, ["feature/12-login-page", "bugfix/12-crash"]);
        assert_eq!(
            branches_for_issue(&f, &["feature"], &branches, 123),
            ["feature/123-other"]
        );
    }

    #[test]
    fn a_pattern_without_the_type_matches_any_branch_of_the_shape() {
        let f = IssueFlow {
            branch_from_issue: Some("{issue}-{slug}".into()),
            ..IssueFlow::default()
        };
        let branches = vec!["7-fix".to_string(), "feature/7-fix".to_string()];
        assert_eq!(
            branches_for_issue(&f, &["feature"], &branches, 7),
            ["7-fix"]
        );
    }

    #[test]
    fn an_issue_key_matches_by_its_number() {
        let f = IssueFlow {
            branch_from_issue: Some("{type}/{issue_key}-{slug}".into()),
            ..IssueFlow::default()
        };
        let branches = vec!["feature/PROJ-12-login".to_string()];
        assert_eq!(
            branches_for_issue(&f, &["feature"], &branches, 12),
            ["feature/PROJ-12-login"]
        );
    }

    #[test]
    fn wait_for_words() {
        assert_eq!(WaitFor::parse("branch"), WaitFor::Branch);
        assert_eq!(WaitFor::parse("MR"), WaitFor::MergeRequest);
        assert_eq!(
            WaitFor::parse("branch-or-mr"),
            WaitFor::BranchOrMergeRequest
        );
        assert_eq!(WaitFor::parse(""), WaitFor::BranchOrMergeRequest);
        assert!(WaitFor::Branch.wants_branch() && !WaitFor::Branch.wants_request());
    }
}
