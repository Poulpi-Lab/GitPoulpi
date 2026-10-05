//! The Gitea / Forgejo family spec: URL shapes, head refs, `codeberg.org`.

use crate::spec::{token_page_from_help, CheckoutSource, FamilySpec};
use crate::web_url::{finish, tidy};
use crate::{percent_encode_branch, PullRequestForm};
use gitpoulpi_config::{ApiFamily, IntegrationKind};

pub static SPEC: FamilySpec = FamilySpec {
    pr_web_url: |base, project, n| Some(format!("{base}/{project}/pulls/{n}")),
    new_pr_url: |base, project, source, target| {
        let base = base.trim_end_matches('/');
        let source = percent_encode_branch(source);
        Some(match target {
            Some(target) => format!(
                "{base}/{project}/compare/{}...{source}",
                percent_encode_branch(target)
            ),
            None => format!("{base}/{project}/compare/{source}"),
        })
    },
    new_pr_form_url: |base, form| Some(new_pr_form(base, form)),
    issue_web_url: |base, project, reference| Some(format!("{base}/{project}/issues/{reference}")),
    new_issue_url: |base, project, title, body, _labels| {
        let base = base.trim_end_matches('/');
        let title = title.trim();
        let mut params = vec![];
        if !title.is_empty() {
            params.push(("title".to_string(), title.to_string()));
        }
        Some(tidy(finish(
            &format!("{base}/{project}/issues/new?"),
            &params,
            "body",
            body.trim(),
        )))
    },
    branch_url: |base, project, branch| Some(format!("{base}/{project}/src/branch/{branch}")),
    commit_url: |base, project, sha| Some(format!("{base}/{project}/commit/{sha}")),
    // Same special ref as GitHub: fork pull requests are fetched from the
    // target repository.
    #[cfg(feature = "hosting")]
    checkout: |pr| CheckoutSource::HeadRef {
        remote_ref: format!("refs/pull/{}/head", pr.number),
        local: format!("pr/{}", pr.number),
    },
    token_page: token_page_from_help,
    well_known: |host| match host {
        "codeberg.org" => Some(IntegrationKind::Forgejo),
        "gitea.com" => Some(IntegrationKind::Gitea),
        _ => None,
    },
    ..FamilySpec::none(ApiFamily::Gitea, "GT", '#')
};

fn new_pr_form(base: &str, form: &PullRequestForm<'_>) -> String {
    let base = base.trim_end_matches('/');
    let branch = percent_encode_branch;
    let head = match form.source_project {
        Some(fork) => format!(
            "{}:{}",
            fork.split('/').next().unwrap_or(fork),
            branch(form.source_branch)
        ),
        None => branch(form.source_branch),
    };
    let range = match form.target_branch {
        Some(t) => format!("{}...{head}", branch(t)),
        None => head,
    };
    let mut params = vec![];
    let title = form.title.trim();
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
