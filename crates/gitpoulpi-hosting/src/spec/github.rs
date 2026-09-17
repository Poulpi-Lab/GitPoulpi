//! GitHub (github.com and Enterprise Server): URL shapes and head refs.

use super::*;
use crate::web_url::{finish, tidy};
use crate::{percent_encode_branch, PullRequestForm};

pub(super) static SPEC: FamilySpec = FamilySpec {
    pr_web_url: |base, project, n| Some(format!("{base}/{project}/pull/{n}")),
    new_pr_url: |base, project, source, target| {
        let base = base.trim_end_matches('/');
        let source = percent_encode_branch(source);
        Some(match target {
            Some(target) => {
                let target = percent_encode_branch(target);
                format!("{base}/{project}/compare/{target}...{source}?expand=1")
            }
            None => format!("{base}/{project}/compare/{source}?expand=1"),
        })
    },
    new_pr_form_url: |base, form| Some(new_pr_form(base, form)),
    issue_web_url: |base, project, reference| Some(format!("{base}/{project}/issues/{reference}")),
    new_issue_url: |base, project, title, body, labels| {
        let base = base.trim_end_matches('/');
        let title = title.trim();
        let mut params = vec![];
        if !title.is_empty() {
            params.push(("title".to_string(), title.to_string()));
        }
        let labels: Vec<&str> = labels
            .iter()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        if !labels.is_empty() {
            params.push(("labels".to_string(), labels.join(",")));
        }
        Some(tidy(finish(
            &format!("{base}/{project}/issues/new?"),
            &params,
            "body",
            body.trim(),
        )))
    },
    branch_url: |base, project, branch| Some(format!("{base}/{project}/tree/{branch}")),
    commit_url: |base, project, sha| Some(format!("{base}/{project}/commit/{sha}")),
    #[cfg(feature = "hosting")]
    checkout: |pr| CheckoutSource::HeadRef {
        remote_ref: format!("refs/pull/{}/head", pr.number),
        local: format!("pr/{}", pr.number),
    },
    token_page: token_page_from_help,
    well_known: |host| {
        (host == "github.com" || host.ends_with(".github.com")).then_some(IntegrationKind::Github)
    },
    ..FamilySpec::none(ApiFamily::Github, "GH", '#')
};

fn new_pr_form(base: &str, form: &PullRequestForm<'_>) -> String {
    let base = base.trim_end_matches('/');
    let branch = percent_encode_branch;
    let title = form.title.trim();
    let head = match form.source_project {
        Some(fork) => {
            let (owner, repo) = fork.split_once('/').unwrap_or((fork, ""));
            let same_name = form.project.rsplit('/').next() == Some(repo);
            if same_name || repo.is_empty() {
                format!("{owner}:{}", branch(form.source_branch))
            } else {
                format!("{owner}:{repo}:{}", branch(form.source_branch))
            }
        }
        None => branch(form.source_branch),
    };
    let range = match form.target_branch {
        Some(t) => format!("{}...{head}", branch(t)),
        None => head,
    };
    let mut params = vec![("expand".to_string(), "1".to_string())];
    if !title.is_empty() {
        params.push(("title".to_string(), title.to_string()));
    }
    tidy(finish(
        &format!("{base}/{}/compare/{range}?", form.project),
        &params,
        "body",
        form.body.trim(),
    ))
}
