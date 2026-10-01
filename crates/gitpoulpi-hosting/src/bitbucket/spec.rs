//! The Bitbucket family spec: URL shapes, fork checkout, `bitbucket.org`.

use crate::percent_encode_branch;
use crate::spec::{token_page_from_help, CheckoutSource, FamilySpec};
use gitpoulpi_config::{ApiFamily, IntegrationKind};

pub static SPEC: FamilySpec = FamilySpec {
    pr_web_url: |base, project, n| Some(format!("{base}/{project}/pull-requests/{n}")),
    new_pr_url: |base, project, source, target| {
        let base = base.trim_end_matches('/');
        let mut url = format!(
            "{base}/{project}/pull-requests/new?source={}",
            percent_encode_branch(source)
        );
        if let Some(target) = target {
            url.push_str(&format!("&dest={}", percent_encode_branch(target)));
        }
        Some(url)
    },
    new_pr_form_url: |base, form| {
        // Bitbucket takes no title or description in the url.
        let project = form.source_project.unwrap_or(form.project);
        let base = base.trim_end_matches('/');
        let mut url = format!(
            "{base}/{project}/pull-requests/new?source={}",
            percent_encode_branch(form.source_branch)
        );
        if let Some(target) = form.target_branch {
            url.push_str(&format!("&dest={}", percent_encode_branch(target)));
        }
        Some(url)
    },
    branch_url: |base, project, branch| Some(format!("{base}/{project}/branch/{branch}")),
    commit_url: |base, project, sha| Some(format!("{base}/{project}/commits/{sha}")),
    checkout: |pr| {
        // Data Center keeps the head in `refs/pull-requests/{id}/from` of
        // the target repository (forks included); its web url is
        // `.../projects/KEY/repos/slug/pull-requests/N`.
        if pr.web_url.contains("/projects/") && pr.web_url.contains("/pull-requests/") {
            return CheckoutSource::HeadRef {
                remote_ref: format!("refs/pull-requests/{}/from", pr.number),
                local: format!("pr/{}", pr.number),
            };
        }
        // Cloud keeps no fetchable head ref: check the source branch out,
        // from the fork's clone url when the request comes from one.
        let fork_url = pr.source_project.as_deref().and_then(|fork| {
            let (scheme, rest) = pr.web_url.split_once("://")?;
            let host = rest.split('/').next()?;
            Some(format!("{scheme}://{host}/{fork}.git"))
        });
        CheckoutSource::Branch {
            fork_url,
            branch: pr.source_branch.clone(),
            local: format!("pr/{}", pr.number),
        }
    },
    token_page: token_page_from_help,
    well_known: |host| (host == "bitbucket.org").then_some(IntegrationKind::Bitbucket),
    ..FamilySpec::none(ApiFamily::Bitbucket, "BB", '#')
};
