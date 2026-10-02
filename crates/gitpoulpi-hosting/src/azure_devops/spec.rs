//! The AzureDevops family spec: remote parsing, URL shapes, well-known hosts.
//!
//! A pull-request project is `Project/Repo`; an issue project is just
//! `Project` (plan D5). The web root is the organization (or Server
//! collection) root, e.g. `https://dev.azure.com/acme`.

use crate::client::encode_component;
use crate::spec::FamilySpec;
use gitpoulpi_config::{ApiFamily, IntegrationKind};

pub static SPEC: FamilySpec = FamilySpec {
    parse_remote: super::remote::parse_remote,
    owns_remote: super::remote::owns_remote,
    pr_web_url: |root, project, n| {
        let (p, r) = split_project(project);
        Some(format!(
            "{}/{}/_git/{}/pullrequest/{n}",
            trim_root(root),
            enc(p),
            enc(r)
        ))
    },
    new_pr_url: |root, project, source, target| {
        let (p, r) = split_project(project);
        let mut url = format!(
            "{}/{}/_git/{}/pullrequestcreate?sourceRef={}",
            trim_root(root),
            enc(p),
            enc(r),
            encode_component(source)
        );
        if let Some(target) = target {
            url.push_str(&format!("&targetRef={}", encode_component(target)));
        }
        Some(url)
    },
    new_pr_form_url: |root, form| {
        (SPEC.new_pr_url)(root, form.project, form.source_branch, form.target_branch)
    },
    issue_web_url: |root, project, reference| {
        let reference = reference.trim();
        let id = reference
            .strip_prefix("AB#")
            .or_else(|| reference.strip_prefix('#'))
            .unwrap_or(reference);
        let (p, _) = split_project(project);
        (!id.is_empty() && id.chars().all(|c| c.is_ascii_digit()))
            .then(|| format!("{}/{}/_workitems/edit/{id}", trim_root(root), enc(p)))
    },
    branch_url: |root, project, branch| {
        let (p, r) = split_project(project);
        Some(format!(
            "{}/{}/_git/{}?version=GB{}",
            trim_root(root),
            enc(p),
            enc(r),
            encode_component(branch)
        ))
    },
    commit_url: |root, project, sha| {
        let (p, r) = split_project(project);
        Some(format!(
            "{}/{}/_git/{}/commit/{sha}",
            trim_root(root),
            enc(p),
            enc(r)
        ))
    },
    well_known: |host| {
        let host = host.split('/').next().unwrap_or(host);
        (host == "dev.azure.com"
            || host == "ssh.dev.azure.com"
            || host.ends_with(".visualstudio.com"))
        .then_some(IntegrationKind::AzureDevops)
    },
    // A pull request is fetched by its source branch (`CheckoutSource::Branch`,
    // the default): `refs/pull/{id}/head` is not documented.
    ..FamilySpec::none(ApiFamily::AzureDevops, "AZ", '!')
};

fn trim_root(root: &str) -> &str {
    root.trim_end_matches('/')
}

fn enc(segment: &str) -> String {
    encode_component(segment)
}

/// `Project/Repo` to `(Project, Repo)`; a bare name serves as both.
pub(super) fn split_project(project: &str) -> (&str, &str) {
    project.split_once('/').unwrap_or((project, project))
}
