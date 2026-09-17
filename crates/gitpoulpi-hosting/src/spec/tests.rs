//! The per-family spec table: GitHub/GitLab behave exactly as the free
//! helpers did before they became wrappers over it, and the other four
//! families are inert stubs until their slice fills them in.

use super::*;
use crate::{remote_web_base, Integration, IntegrationKind};
use gitpoulpi_config::ApiFamily;

fn integration(kind: IntegrationKind, url: &str) -> Integration {
    Integration::new("t", kind, url)
}

#[test]
fn every_family_has_a_spec_with_its_chip_tag_and_sigil() {
    for (family, tag, sigil) in [
        (ApiFamily::Github, "GH", '#'),
        (ApiFamily::Gitlab, "GL", '!'),
        (ApiFamily::AzureDevops, "AZ", '!'),
        (ApiFamily::Bitbucket, "BB", '#'),
        (ApiFamily::Jira, "JI", '#'),
        (ApiFamily::Trello, "TR", '#'),
        (ApiFamily::Gitea, "GT", '#'),
    ] {
        let spec = spec_for(family);
        assert_eq!(spec.family, family);
        assert_eq!(spec.tag, tag, "{family:?}");
        assert_eq!(spec.pr_sigil, sigil, "{family:?}");
    }
}

#[test]
fn github_and_gitlab_urls_come_from_the_spec() {
    let gh = spec_for(ApiFamily::Github);
    let gl = spec_for(ApiFamily::Gitlab);
    assert_eq!(
        (gh.pr_web_url)("https://github.com", "me/thing", 12).as_deref(),
        Some("https://github.com/me/thing/pull/12")
    );
    assert_eq!(
        (gl.pr_web_url)("https://gl.example", "g/p", 53).as_deref(),
        Some("https://gl.example/g/p/-/merge_requests/53")
    );
    assert_eq!(
        (gh.issue_web_url)("https://github.com", "me/thing", "3").as_deref(),
        Some("https://github.com/me/thing/issues/3")
    );
    assert_eq!(
        (gl.issue_web_url)("https://gl.example", "g/p", "12").as_deref(),
        Some("https://gl.example/g/p/-/issues/12")
    );
    assert_eq!(
        (gh.pr_edit_url)("https://github.com/me/thing/pull/12"),
        "https://github.com/me/thing/pull/12"
    );
    assert_eq!(
        (gl.pr_edit_url)("https://gl.example/g/p/-/merge_requests/53/"),
        "https://gl.example/g/p/-/merge_requests/53/edit"
    );
    assert_eq!(
        (gh.new_pr_url)("https://github.com", "me/thing", "feat", Some("main")).as_deref(),
        Some("https://github.com/me/thing/compare/main...feat?expand=1")
    );
    assert_eq!(
        (gl.new_pr_url)("https://gl.example", "g/p", "feat", None).as_deref(),
        Some("https://gl.example/g/p/-/merge_requests/new?merge_request%5Bsource_branch%5D=feat")
    );
    assert_eq!(
        (gh.new_issue_url)("https://github.com", "o/r", "T", "", &[]).as_deref(),
        Some("https://github.com/o/r/issues/new?title=T")
    );
    assert_eq!(
        (gl.new_issue_url)("https://gl.example", "g/p", "T", "", &[]).as_deref(),
        Some("https://gl.example/g/p/-/issues/new?issue%5Btitle%5D=T")
    );
    assert_eq!(
        (gh.branch_url)("https://github.com", "o/r", "feat/x").as_deref(),
        Some("https://github.com/o/r/tree/feat/x")
    );
    assert_eq!(
        (gl.commit_url)("https://gl.example", "g/p", "abc").as_deref(),
        Some("https://gl.example/g/p/-/commit/abc")
    );
}

#[cfg(feature = "hosting")]
#[test]
fn github_and_gitlab_check_pull_requests_out_from_their_head_refs() {
    use crate::model::{PullRequestState, PullRequestSummary};
    let pr = |n: u64| PullRequestSummary {
        number: n,
        iid_label: format!("#{n}"),
        title: String::new(),
        state: PullRequestState::Open,
        source_branch: "feat".into(),
        target_branch: "main".into(),
        source_project: None,
        author: String::new(),
        web_url: String::new(),
        updated_at: String::new(),
        review_requested: false,
        assignees: vec![],
        reviewers: vec![],
        checks: None,
        has_conflicts: None,
    };
    assert_eq!(
        (spec_for(ApiFamily::Github).checkout)(&pr(7)),
        CheckoutSource::HeadRef {
            remote_ref: "refs/pull/7/head".into(),
            local: "pr/7".into()
        }
    );
    assert_eq!(
        (spec_for(ApiFamily::Gitlab).checkout)(&pr(7)),
        CheckoutSource::HeadRef {
            remote_ref: "refs/merge-requests/7/head".into(),
            local: "mr/7".into()
        }
    );
    // Azure DevOps (and a stub family) fetches by branch.
    assert!(matches!(
        (spec_for(ApiFamily::AzureDevops).checkout)(&pr(7)),
        CheckoutSource::Branch { .. }
    ));
}

#[test]
fn the_default_parse_remote_is_remote_web_base() {
    for remote in [
        "https://gitlab.example.com/group/sub/proj.git",
        "ssh://git@gitlab.example.com:2222/group/sub/proj.git",
        "git@gitlab.example.com:group/sub/proj.git",
        "https://github.com/me/thing.git",
        "",
        "not a url",
    ] {
        for family in [ApiFamily::Github, ApiFamily::Gitlab] {
            let parsed = (spec_for(family).parse_remote)(remote).map(|r| (r.web_base, r.project));
            assert_eq!(parsed, remote_web_base(remote), "{remote}");
        }
    }
}

#[test]
fn the_default_owner_compares_hostnames_ignoring_ports_and_api_prefixes() {
    let remote = |url| (spec_for(ApiFamily::Gitlab).parse_remote)(url).unwrap();
    let gl = integration(
        IntegrationKind::GitlabSelfManaged,
        "https://gl.example:8443",
    );
    let owns = spec_for(ApiFamily::Gitlab).owns_remote;
    assert!(owns(&gl, &remote("git@gl.example:g/p.git")));
    assert!(owns(&gl, &remote("https://GL.example/g/p")));
    assert!(!owns(&gl, &remote("https://other.example/g/p")));
    let gh = integration(IntegrationKind::Github, "");
    assert!((spec_for(ApiFamily::Github).owns_remote)(
        &gh,
        &remote("git@github.com:me/thing.git")
    ));
}

#[test]
fn token_pages_follow_the_profiles_kind() {
    assert_eq!(
        (spec_for(ApiFamily::Github).token_page)(&integration(IntegrationKind::Github, ""))
            .as_deref(),
        Some("https://github.com/settings/tokens/new?description=GitPoulpi&scopes=repo,read:user,admin:public_key")
    );
    assert_eq!(
        (spec_for(ApiFamily::Jira).token_page)(&integration(
            IntegrationKind::JiraCloud,
            "https://x.atlassian.net"
        ))
        .as_deref(),
        Some("https://id.atlassian.com/manage-profile/security/api-tokens")
    );
}

#[cfg(feature = "bitbucket")]
#[test]
fn bitbucket_keeps_its_branch_and_commit_links() {
    // Bitbucket's branch/commit links predate the spec and keep working.
    let bb = spec_for(ApiFamily::Bitbucket);
    assert_eq!(
        (bb.branch_url)("https://bitbucket.org", "w/r", "x").as_deref(),
        Some("https://bitbucket.org/w/r/branch/x")
    );
    assert_eq!(
        (bb.commit_url)("https://bitbucket.org", "w/r", "abc").as_deref(),
        Some("https://bitbucket.org/w/r/commits/abc")
    );
}
