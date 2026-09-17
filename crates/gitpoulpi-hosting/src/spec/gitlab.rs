//! GitLab (gitlab.com and self-managed): URL shapes and head refs.

use super::*;
use crate::web_url::{finish, gitlab_labels, tidy};
use crate::{percent_encode_branch, PullRequestForm};

pub(super) static SPEC: FamilySpec = FamilySpec {
    pr_web_url: |base, project, n| Some(format!("{base}/{project}/-/merge_requests/{n}")),
    pr_edit_url: |url| format!("{}/edit", url.trim_end_matches('/')),
    new_pr_url: |base, project, source, target| {
        let base = base.trim_end_matches('/');
        let source = percent_encode_branch(source);
        let mut url = format!(
            "{base}/{project}/-/merge_requests/new?merge_request%5Bsource_branch%5D={source}"
        );
        if let Some(target) = target {
            let target = percent_encode_branch(target);
            url.push_str(&format!("&merge_request%5Btarget_branch%5D={target}"));
        }
        Some(url)
    },
    new_pr_form_url: |base, form| Some(new_pr_form(base, form)),
    issue_web_url: |base, project, reference| {
        Some(format!("{base}/{project}/-/issues/{reference}"))
    },
    new_issue_url: |base, project, title, body, labels| {
        let base = base.trim_end_matches('/');
        let title = title.trim();
        let mut params = vec![];
        if !title.is_empty() {
            params.push(("issue%5Btitle%5D".to_string(), title.to_string()));
        }
        let mut description = body.trim().to_string();
        let quick = gitlab_labels(labels);
        if !quick.is_empty() {
            if !description.is_empty() {
                description.push_str("\n\n");
            }
            description.push_str(&quick);
        }
        Some(tidy(finish(
            &format!("{base}/{project}/-/issues/new?"),
            &params,
            "issue%5Bdescription%5D",
            &description,
        )))
    },
    branch_url: |base, project, branch| Some(format!("{base}/{project}/-/tree/{branch}")),
    commit_url: |base, project, sha| Some(format!("{base}/{project}/-/commit/{sha}")),
    #[cfg(feature = "hosting")]
    checkout: |pr| CheckoutSource::HeadRef {
        remote_ref: format!("refs/merge-requests/{}/head", pr.number),
        local: format!("mr/{}", pr.number),
    },
    token_page: token_page_from_help,
    well_known: |host| {
        (host == "gitlab.com" || host.ends_with(".gitlab.com")).then_some(IntegrationKind::Gitlab)
    },
    ..FamilySpec::none(ApiFamily::Gitlab, "GL", '!')
};

fn new_pr_form(base: &str, form: &PullRequestForm<'_>) -> String {
    let base = base.trim_end_matches('/');
    let title = form.title.trim();
    // A fork's request starts from the fork's own page.
    let project = form.source_project.unwrap_or(form.project);
    let title = if form.draft && !title.is_empty() {
        format!("Draft: {title}")
    } else {
        title.to_string()
    };
    let mut params = vec![(
        "merge_request%5Bsource_branch%5D".to_string(),
        form.source_branch.to_string(),
    )];
    if let Some(t) = form.target_branch {
        params.push((
            "merge_request%5Btarget_branch%5D".to_string(),
            t.to_string(),
        ));
    }
    if !title.is_empty() {
        params.push(("merge_request%5Btitle%5D".to_string(), title));
    }
    tidy(finish(
        &format!("{base}/{project}/-/merge_requests/new?"),
        &params,
        "merge_request%5Bdescription%5D",
        form.body.trim(),
    ))
}
