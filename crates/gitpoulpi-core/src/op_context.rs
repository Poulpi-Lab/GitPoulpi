//! What produced an error: the operation (a human label and the key it is
//! known by), its main arguments, the repository, the frontend that asked and
//! the request's id.
//!
//! The daemon builds an [`OpContext`] for every request that fails (the
//! request's arguments are summarised by `gitpoulpi_ipc::op_context`, which
//! has the JSON) and sends it beside the error (`ResponseError::context`),
//! logs it in one line
//! ([`OpContext::failure_line`]) and the Activity Log reads it back: from the
//! error itself, or from that line ([`parse_failure_line`]) when the failure
//! happened in another frontend.
//!
//! Everything here is English and redacted: it ends up in logs and in the
//! text a person pastes into a chat. Arguments are picked from a short list of
//! keys (a name, a branch, a path, a URL): never a message body, never a value
//! that could be a secret, and the result goes through [`crate::redact`].

use serde::{Deserialize, Serialize};

use crate::domain::OperationKind;

/// The longest an argument summary is.
const ARGS_MAX: usize = 160;

/// Where an error came from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpContext {
    /// The operation's key (`RemoteRemove`).
    pub operation: String,
    /// What a person calls it ("Remove remote").
    pub label: String,
    /// The main arguments, redacted ("remote=cm6, url=https://host/r.git").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub args: String,
    /// The repository's root path, when the request had one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    /// Who asked: `TUI`, `CLI` or `MCP`, when the connection said.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontend: Option<String>,
    /// The request's id on its connection (the operation id).
    #[serde(default)]
    pub request: u64,
    /// The job the request became, when it was one (network operations).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job: Option<u64>,
}

impl OpContext {
    /// A context that is not a daemon request (a hosting call, an external
    /// program): `label` is what it is called, `args` its main arguments.
    pub fn named(label: impl Into<String>, args: impl AsRef<str>) -> Self {
        let label = label.into();
        OpContext {
            operation: label.clone(),
            label,
            args: clip(&crate::redact(args.as_ref()), ARGS_MAX),
            ..OpContext::default()
        }
    }

    /// The repository's folder name.
    pub fn repo_name(&self) -> Option<String> {
        let repo = self.repo.as_deref()?.trim_end_matches(['/', '\\']);
        let name = repo.rsplit(['/', '\\']).next().unwrap_or(repo);
        (!name.is_empty()).then(|| name.to_string())
    }

    /// "Remove remote (remote=cm6)".
    pub fn describe(&self) -> String {
        if self.args.is_empty() {
            self.label.clone()
        } else {
            format!("{} ({})", self.label, self.args)
        }
    }

    /// The bracketed tag a log line ends with: `[op #12, CLI, repo name]`.
    pub fn tag(&self) -> String {
        let mut parts = vec![format!("op #{}", self.request)];
        if let Some(frontend) = &self.frontend {
            parts.push(frontend.clone());
        }
        if let Some(name) = self.repo_name() {
            parts.push(format!("repo {}", tag_safe(&name)));
        }
        format!("[{}]", parts.join(", "))
    }

    /// The application log's line for a failure: "Remove remote failed
    /// (remote=cm6) [op #12, CLI, repo r]: backend error: no such remote: cm6".
    /// What produced it comes first, so a narrow window still shows it.
    pub fn failure_line(&self, message: &str) -> String {
        let message = crate::redact(message);
        let mut first = message.lines().next().unwrap_or("").to_string();
        if message.lines().count() > 1 {
            first.push_str(" ...");
        }
        let args = if self.args.is_empty() {
            String::new()
        } else {
            format!(" ({})", self.args)
        };
        format!("{} failed{args} {}: {first}", self.label, self.tag())
    }

    /// The facts as the lines of an error report.
    pub fn report_lines(&self) -> Vec<String> {
        let mut lines = vec![format!("operation: {}", self.operation_line())];
        if !self.args.is_empty() {
            lines.push(format!("arguments: {}", self.args));
        }
        if let Some(repo) = &self.repo {
            lines.push(format!("repository: {}", crate::redact(repo)));
        }
        if let Some(frontend) = &self.frontend {
            lines.push(format!("requested by: {frontend}"));
        }
        if self.request > 0 {
            let job = self
                .job
                .map(|job| format!(" (job {job})"))
                .unwrap_or_default();
            lines.push(format!("operation id: {}{job}", self.request));
        }
        lines
    }

    fn operation_line(&self) -> String {
        if self.operation == self.label {
            self.label.clone()
        } else {
            format!("{} ({})", self.label, self.operation)
        }
    }
}

/// Keeps `text` from breaking the tag (`]` and `,` are its delimiters).
fn tag_safe(text: &str) -> String {
    text.chars()
        .map(|c| if matches!(c, ']' | ',' | '[') { '_' } else { c })
        .collect()
}

/// What a log line's tag says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpTag {
    pub request: u64,
    pub frontend: Option<String>,
    pub repo_name: Option<String>,
}

/// The tag in a log line ([`OpContext::tag`]), if it has one.
pub fn parse_tag(message: &str) -> Option<OpTag> {
    let first = message.lines().next()?;
    let start = first.find("[op #")?;
    let end = first[start..].find(']')? + start;
    let mut parts = first[start + 5..end].split(", ");
    let request = parts.next()?.trim().parse().ok()?;
    let mut tag = OpTag {
        request,
        frontend: None,
        repo_name: None,
    };
    for part in parts {
        if let Some(name) = part.strip_prefix("repo ") {
            tag.repo_name = Some(name.to_string());
        } else if !part.is_empty() {
            tag.frontend = Some(part.to_string());
        }
    }
    Some(tag)
}

/// A failure line taken apart ([`OpContext::failure_line`]'s inverse).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedFailure {
    pub label: String,
    pub args: String,
    pub message: String,
    pub tag: OpTag,
}

/// Reads a line [`OpContext::failure_line`] wrote.
pub fn parse_failure_line(message: &str) -> Option<ParsedFailure> {
    let tag = parse_tag(message)?;
    let first = message.lines().next()?;
    let at = first.find(" [op #")?;
    let head = &first[..at];
    let after = &first[at + 1..];
    let close = after.find(']')?;
    let text = after[close + 1..].strip_prefix(": ")?;
    let (label, args) = match head
        .strip_suffix(')')
        .and_then(|h| h.split_once(" failed ("))
    {
        Some((label, args)) => (label, args),
        None => (head.strip_suffix(" failed")?, ""),
    };
    Some(ParsedFailure {
        label: label.to_string(),
        args: args.to_string(),
        message: text.to_string(),
        tag,
    })
}

pub fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let mut out: String = text.chars().take(max.saturating_sub(3)).collect();
    out.push_str("...");
    out
}

/// What a person calls `kind`: "Remove remote". The operations without a
/// hand-written name are their key split into words.
pub fn operation_label(kind: OperationKind) -> String {
    use OperationKind as K;
    let named = match kind {
        K::Fetch => "Fetch",
        K::FetchAll => "Fetch all remotes",
        K::FetchRefAsBranch => "Fetch a ref as a branch",
        K::FetchRefspec => "Fetch a refspec",
        K::Pull => "Pull",
        K::Push => "Push",
        K::PushForReview => "Push for review",
        K::Clone => "Clone",
        K::Init => "Create repository",
        K::RemoteAdd => "Add remote",
        K::RemoteRemove => "Remove remote",
        K::RemoteRename => "Rename remote",
        K::RemoteSetUrl => "Set remote URL",
        K::RemoteSetPushUrl => "Set remote push URL",
        K::RemoteAccessCheck => "Test remote access",
        K::BranchCreate => "Create branch",
        K::BranchCreateTracking => "Create tracking branch",
        K::BranchDelete => "Delete branch",
        K::BranchDeleteRemote => "Delete remote branch",
        K::BranchRename => "Rename branch",
        K::BranchSetUpstream => "Set upstream",
        K::BranchSetRemotes => "Set remotes",
        K::RemoteGroupStatus => "Remote group status",
        K::RemoteGroupPlan => "Plan a grouped push or pull",
        K::RemoteGroupRun => "Run a grouped push or pull",
        K::PullAllBranches => "Pull all branches",
        K::TagCreate => "Create tag",
        K::TagDelete => "Delete tag",
        K::TagDeleteRemote => "Delete remote tag",
        K::TagPush => "Push tags",
        K::TagFetch => "Fetch tag",
        K::Checkout => "Checkout",
        K::Commit => "Commit",
        K::Merge => "Merge",
        K::Rebase => "Rebase",
        K::RebaseInteractive => "Interactive rebase",
        K::CherryPick => "Cherry-pick",
        K::Revert => "Revert",
        K::Reset => "Reset",
        K::StashPush => "Stash",
        K::StashPop => "Pop stash",
        K::StashApply => "Apply stash",
        K::StashDrop => "Drop stash",
        K::StageFile => "Stage file",
        K::StageFolder => "Stage folder",
        K::StageAll => "Stage all",
        K::UnstageFile => "Unstage file",
        K::UnstageFolder => "Unstage folder",
        K::UnstageAll => "Unstage all",
        K::Discard => "Discard changes",
        K::DeleteFile => "Delete file",
        K::Undo => "Undo",
        K::Redo => "Redo",
        K::RepoConfig => "Read repository settings",
        K::SetRepoConfig => "Change repository setting",
        K::RepoAuthTest => "Test repository sign-in",
        K::WorktreeAdd => "Add worktree",
        K::WorktreeChanges => "List worktree changes",
        K::WorktreeRemove => "Remove worktree",
        K::ResolveConflict => "Resolve conflict",
        K::FlowRun => "Run workflow",
        K::LfsPull => "LFS pull",
        K::LfsPush => "LFS push",
        K::LfsFetch => "LFS fetch",
        K::LfsLock => "LFS lock",
        K::LfsUnlock => "LFS unlock",
        _ => return words_of_key(&format!("{kind:?}")),
    };
    named.to_string()
}

/// The operation a label names ([`operation_label`]'s inverse): a log line
/// carries the label, and the report wants the key.
pub fn operation_of_label(label: &str) -> Option<OperationKind> {
    OperationKind::ALL
        .into_iter()
        .find(|kind| operation_label(*kind) == label)
}

/// "RewriteScanSecrets" as "Rewrite scan secrets".
pub fn words_of_key(key: &str) -> String {
    let mut out = String::new();
    for (i, c) in key.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push(' ');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remove_remote() -> OpContext {
        OpContext {
            operation: "RemoteRemove".into(),
            label: operation_label(OperationKind::RemoteRemove),
            args: "remote=cm6".into(),
            repo: Some("/data/Projects/demo".into()),
            frontend: None,
            request: 12,
            job: None,
        }
    }

    #[test]
    fn a_remote_operation_is_labelled_by_what_a_person_calls_it() {
        let ctx = remove_remote();
        assert_eq!(ctx.label, "Remove remote");
        assert_eq!(ctx.repo_name().as_deref(), Some("demo"));
        assert_eq!(ctx.describe(), "Remove remote (remote=cm6)");
    }

    #[test]
    fn an_operation_without_a_hand_written_name_is_its_key_in_words() {
        assert_eq!(
            operation_label(OperationKind::RewriteScanSecrets),
            "Rewrite scan secrets"
        );
        assert_eq!(operation_label(OperationKind::Push), "Push");
    }

    #[test]
    fn a_failure_line_reads_back_as_the_same_facts() {
        let mut ctx = remove_remote();
        ctx.frontend = Some("CLI".into());
        let line = ctx.failure_line("backend error: no such remote: cm6");
        assert_eq!(
            line,
            "Remove remote failed (remote=cm6) [op #12, CLI, repo demo]: \
             backend error: no such remote: cm6"
        );
        let parsed = parse_failure_line(&line).expect("parses");
        assert_eq!(parsed.label, "Remove remote");
        assert_eq!(parsed.args, "remote=cm6");
        assert_eq!(parsed.message, "backend error: no such remote: cm6");
        assert_eq!(parsed.tag.request, 12);
        assert_eq!(parsed.tag.frontend.as_deref(), Some("CLI"));
        assert_eq!(parsed.tag.repo_name.as_deref(), Some("demo"));
    }

    #[test]
    fn a_line_without_arguments_or_frontend_parses_too() {
        let ctx = OpContext {
            operation: "FetchAll".into(),
            label: operation_label(OperationKind::FetchAll),
            request: 3,
            ..OpContext::default()
        };
        let line = ctx.failure_line("boom");
        assert_eq!(line, "Fetch all remotes failed [op #3]: boom");
        let parsed = parse_failure_line(&line).expect("parses");
        assert_eq!(parsed.label, "Fetch all remotes");
        assert_eq!(parsed.args, "");
        assert_eq!(parsed.tag.frontend, None);
        assert!(parse_failure_line("some other line").is_none());
    }

    #[test]
    fn the_failure_line_hides_secrets_in_the_message() {
        let ctx = OpContext::named("Fetch", "");
        let line = ctx.failure_line("unable to access 'https://me:hunter2@host.test/r.git'");
        assert!(!line.contains("hunter2"), "{line}");
    }

    #[test]
    fn the_report_lines_name_operation_repository_and_id() {
        let mut ctx = remove_remote();
        ctx.request = 9;
        ctx.repo = Some("/p/demo".into());
        ctx.frontend = Some("TUI".into());
        ctx.job = Some(4);
        let lines = ctx.report_lines();
        assert_eq!(lines[0], "operation: Remove remote (RemoteRemove)");
        assert!(lines.contains(&"arguments: remote=cm6".to_string()));
        assert!(lines.contains(&"repository: /p/demo".to_string()));
        assert!(lines.contains(&"requested by: TUI".to_string()));
        assert!(lines.contains(&"operation id: 9 (job 4)".to_string()));
    }

    #[test]
    fn every_operation_has_a_label_of_its_own_that_names_it_back() {
        for kind in OperationKind::ALL {
            let label = operation_label(kind);
            assert!(!label.is_empty(), "{kind:?}");
            assert_eq!(operation_of_label(&label), Some(kind), "{label}");
        }
        assert_eq!(operation_of_label("Not an operation"), None);
    }
}
