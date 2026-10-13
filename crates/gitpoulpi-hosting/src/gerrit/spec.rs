//! The Gerrit family spec: change pages, the `refs/changes` head ref and
//! remotes with the authenticated `/a/` prefix.

use crate::spec::{CheckoutSource, FamilySpec, RemoteRef};
use gitpoulpi_config::ApiFamily;

pub static SPEC: FamilySpec = FamilySpec {
    // `{url}/c/{project}/+/{number}`.
    pr_web_url: |base, project, n| {
        Some(format!("{}/c/{project}/+/{n}", base.trim_end_matches('/')))
    },
    // The change ref of the patch set the list carries in `source_branch`
    // (`refs/changes/NN/<change>/<ps>`), falling back on patch set 1.
    #[cfg(feature = "hosting")]
    checkout: |pr| CheckoutSource::HeadRef {
        remote_ref: if super::map::patch_set_of_ref(&pr.source_branch).is_some() {
            pr.source_branch.clone()
        } else {
            super::map::change_ref(pr.number, 1)
        },
        local: format!("change/{}", pr.number),
    },
    parse_remote,
    ..FamilySpec::none(ApiFamily::Gerrit, "GR", '#')
};

/// A remote of a Gerrit server: `ssh://user@host:29418/project`,
/// `https://host/project` or `https://host/a/project` (the authenticated
/// clone URL carries the `/a/` prefix, which is not part of the name).
fn parse_remote(remote_url: &str) -> Option<RemoteRef> {
    let (web_base, project) = crate::remote_web_base(remote_url)?;
    let project = if remote_url.contains("://") && !remote_url.starts_with("ssh://") {
        project
            .strip_prefix("a/")
            .map(str::to_string)
            .unwrap_or(project)
    } else {
        project
    };
    Some(RemoteRef { web_base, project })
}
