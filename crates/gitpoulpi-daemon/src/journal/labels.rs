use gitpoulpi_core::domain::ResetMode;
use gitpoulpi_core::domain::{short, CheckoutTarget, CommitOptions, Oid, OperationKind};
use gitpoulpi_ipc::Request;
use serde_json::Value;

use super::capture::arg_str;

pub(super) fn label_for(request: &Request) -> String {
    if let Some(label) = crate::journal_snap::label(request) {
        return label;
    }
    if let Some(label) = crate::journal_stashes_remotes::label(request) {
        return label;
    }
    let arg = |k: &str| arg_str(request, k).unwrap_or_default();
    match request.command {
        OperationKind::Commit => {
            let amend = request
                .args
                .get("opts")
                .and_then(|o| serde_json::from_value::<CommitOptions>(o.clone()).ok())
                .is_some_and(|o| o.amend);
            let first_line = arg("message").lines().next().unwrap_or("").to_string();
            format!(
                "{} \"{first_line}\"",
                if amend { "Amend" } else { "Commit" }
            )
        }
        OperationKind::Checkout => {
            match request
                .args
                .get("target")
                .and_then(|t| serde_json::from_value::<CheckoutTarget>(t.clone()).ok())
            {
                Some(CheckoutTarget::Branch(name)) => format!("Checkout {name}"),
                Some(CheckoutTarget::Commit(oid)) => format!("Checkout {}", short(oid)),
                _ => "Checkout".to_string(),
            }
        }
        OperationKind::BranchCreate => format!("Create branch {}", arg("name")),
        OperationKind::BranchDelete => format!("Delete branch {}", arg("name")),
        OperationKind::BranchCreateTracking => format!(
            "Create branch {} tracking {}",
            arg("local_name"),
            arg("upstream")
        ),
        OperationKind::BranchRename => {
            format!("Rename branch {} to {}", arg("old_name"), arg("new_name"))
        }
        OperationKind::BranchSetUpstream => match arg_str(request, "upstream") {
            Some(upstream) => format!("Set upstream of {} to {upstream}", arg("branch")),
            None => format!("Unset upstream of {}", arg("branch")),
        },
        OperationKind::BranchSetRemotes => format!("Set remotes of {}", arg("branch")),
        OperationKind::SetRepoConfig => match arg_str(request, "value") {
            Some(value) => format!("Set {} to {value}", arg("key")),
            None => format!("Unset {}", arg("key")),
        },
        OperationKind::SparseSet => "Set sparse checkout".to_string(),
        OperationKind::SparseDisable => "Disable sparse checkout".to_string(),
        OperationKind::TagCreate => format!("Create tag {}", arg("name")),
        OperationKind::TagDelete => format!("Delete tag {}", arg("name")),
        OperationKind::BranchDeleteRemote => {
            format!("Delete branch {} on {}", arg("name"), arg("remote"))
        }
        OperationKind::TagDeleteRemote => {
            format!("Delete tag {} on {}", arg("name"), arg("remote"))
        }
        OperationKind::TagAnnotate => format!("Annotate tag {}", arg("name")),
        OperationKind::FastForward => format!(
            "Fast-forward {} to {}",
            arg("branch"),
            arg_str(request, "target")
                .map(|t| t.chars().take(7).collect::<String>())
                .unwrap_or_default()
        ),
        OperationKind::AmendCommit => format!(
            "Amend {}",
            arg_str(request, "commit")
                .map(|t| t.chars().take(7).collect::<String>())
                .unwrap_or_default()
        ),
        OperationKind::Reword => format!(
            "Reword {}",
            arg_str(request, "commit")
                .map(|t| t.chars().take(7).collect::<String>())
                .unwrap_or_default()
        ),
        OperationKind::SplitCommit => format!(
            "Split {} into {} commits",
            request
                .args
                .get("req")
                .and_then(|r| r.get("commit"))
                .and_then(|c| c.as_str())
                .map(|t| t.chars().take(7).collect::<String>())
                .unwrap_or_default(),
            request
                .args
                .get("req")
                .and_then(|r| r.get("parts"))
                .and_then(|p| p.as_array())
                .map_or(0, |p| p.len())
        ),
        OperationKind::FetchRefAsBranch => {
            format!("Fetch {} as branch {}", arg("remote_ref"), arg("branch"))
        }
        // The write-ahead label, from the live plan; the outcome's replaces it.
        OperationKind::RewriteApply => {
            let plan = crate::dispatch::repo_ref(request)
                .ok()
                .and_then(|repo| super::capture::rewrite_plan(None, &repo, request));
            match plan {
                Some(plan) => {
                    let names: Vec<&str> = plan.refs.iter().map(|r| r.name.as_str()).collect();
                    rewrite_label(plan.counts.commits_changed, &names)
                }
                None => "Rewrite history".to_string(),
            }
        }
        OperationKind::Merge => match request
            .args
            .get("from")
            .and_then(|r| serde_json::from_value::<gitpoulpi_core::domain::Ref>(r.clone()).ok())
        {
            Some(r) => format!("Merge {}", ref_name(&r)),
            None => "Merge".to_string(),
        },
        OperationKind::Rebase => match request
            .args
            .get("onto")
            .and_then(|r| serde_json::from_value::<gitpoulpi_core::domain::Ref>(r.clone()).ok())
        {
            Some(r) => format!("Rebase onto {}", ref_name(&r)),
            None => "Rebase".to_string(),
        },
        OperationKind::RebaseInteractive => "Interactive rebase".to_string(),
        OperationKind::CherryPick => {
            let n = request
                .args
                .get("commits")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            if n == 1 {
                "Cherry-pick 1 commit".to_string()
            } else {
                format!("Cherry-pick {n} commits")
            }
        }
        OperationKind::Revert => format!(
            "Revert {}",
            request
                .args
                .get("commit")
                .and_then(|c| serde_json::from_value::<Oid>(c.clone()).ok())
                .map(short)
                .unwrap_or_default()
        ),
        OperationKind::Reset => {
            let mode = match request
                .args
                .get("mode")
                .and_then(|m| serde_json::from_value::<ResetMode>(m.clone()).ok())
            {
                Some(ResetMode::Soft) => "soft",
                Some(ResetMode::Mixed) => "mixed",
                _ => "hard",
            };
            format!(
                "Reset ({mode}) to {}",
                request
                    .args
                    .get("target")
                    .and_then(|c| serde_json::from_value::<Oid>(c.clone()).ok())
                    .map(short)
                    .unwrap_or_default()
            )
        }
        OperationKind::Pull => "Pull".to_string(),
        OperationKind::FlowRun => crate::flow::label(request),
        OperationKind::RemoteGroupRun => crate::remote_group::label(request),
        OperationKind::ComposeCommits => {
            let n = request
                .args
                .get("commits")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            format!("Compose {n} commit{}", if n == 1 { "" } else { "s" })
        }
        other => format!("{other:?}"),
    }
}

pub(super) fn ref_name(r: &gitpoulpi_core::domain::Ref) -> String {
    match &r.kind {
        gitpoulpi_core::domain::RefKind::LocalBranch { name }
        | gitpoulpi_core::domain::RefKind::Tag { name } => name.clone(),
        gitpoulpi_core::domain::RefKind::RemoteBranch { remote, name } => {
            format!("{remote}/{name}")
        }
    }
}

/// `Rewrite history: 412 commits, 2 branches, 4 tags`. The shape is fixed:
/// the history rows peel it back into words (`safety_panel/label.rs`).
/// Notes refs ride along and are not counted.
pub(super) fn rewrite_label(commits: u64, ref_names: &[&str]) -> String {
    let branches = ref_names
        .iter()
        .filter(|n| n.starts_with("refs/heads/") || **n == "HEAD")
        .count() as u64;
    let tags = ref_names
        .iter()
        .filter(|n| n.starts_with("refs/tags/"))
        .count() as u64;
    let counted =
        |n: u64, one: &str, other: &str| format!("{n} {}", if n == 1 { one } else { other });
    format!(
        "Rewrite history: {}, {}, {}",
        counted(commits, "commit", "commits"),
        counted(branches, "branch", "branches"),
        counted(tags, "tag", "tags"),
    )
}

#[cfg(test)]
mod tests {
    use super::rewrite_label;

    #[test]
    fn the_rewrite_label_counts_commits_branches_and_tags() {
        assert_eq!(
            rewrite_label(
                412,
                &[
                    "refs/heads/main",
                    "refs/heads/topic",
                    "refs/tags/v1",
                    "refs/tags/v2",
                    "refs/tags/v3",
                    "refs/tags/v4",
                    "refs/notes/commits",
                ]
            ),
            "Rewrite history: 412 commits, 2 branches, 4 tags"
        );
        assert_eq!(
            rewrite_label(1, &["refs/heads/main"]),
            "Rewrite history: 1 commit, 1 branch, 0 tags"
        );
        assert_eq!(
            rewrite_label(3, &["HEAD"]),
            "Rewrite history: 3 commits, 1 branch, 0 tags"
        );
    }
}
