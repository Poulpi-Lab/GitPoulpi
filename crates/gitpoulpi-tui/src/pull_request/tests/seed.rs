use super::*;

fn local_branch(name: &str, upstream: Option<&str>) -> Ref {
    Ref {
        kind: RefKind::LocalBranch { name: name.into() },
        target: gitpoulpi_core::domain::Oid::from_hex(&"1".repeat(40)).unwrap(),
        visibility: Default::default(),
        is_head: false,
        upstream: upstream.map(str::to_string),
        tag_sync: None,
    }
}

#[test]
fn no_upstream_is_blocked_with_push_first() {
    let refs = vec![local_branch("feature", None)];
    let reason = pull_request_reason(
        "feature",
        &Ok(refs),
        &[remote("origin", "https://github.com/me/thing.git")],
        &[],
    );
    assert_eq!(reason.as_deref(), Some("push feature first"));
}

#[test]
fn an_unrecognized_remote_host_is_blocked() {
    let refs = vec![local_branch("feature", Some("origin/feature"))];
    let reason = pull_request_reason(
        "feature",
        &Ok(refs),
        &[remote("origin", "https://git.example.com/me/thing.git")],
        &[],
    );
    assert_eq!(reason.as_deref(), Some("remote host not recognized"));
}

/// A well-known host with no `[[integrations]]` entry is still
/// offered - there is a URL to build even with no token.
#[test]
fn github_with_no_integration_is_offered() {
    let refs = vec![local_branch("feature", Some("origin/feature"))];
    let reason = pull_request_reason(
        "feature",
        &Ok(refs),
        &[remote("origin", "https://github.com/me/thing.git")],
        &[],
    );
    assert_eq!(reason, None);
}

/// The ref menu asks this on every open: a matched integration is
/// offered on the remote alone, no token looked up (the credential
/// helper costs seconds on Windows).
#[test]
fn a_matched_integration_is_offered_without_a_token() {
    let refs = vec![local_branch("feature", Some("origin/feature"))];
    let reason = pull_request_reason(
        "feature",
        &Ok(refs),
        &[remote("origin", "https://gitlab.example.com/g/p.git")],
        &[integration(
            "corp",
            IntegrationKind::GitlabSelfManaged,
            "https://gitlab.example.com",
        )],
    );
    assert_eq!(reason, None);
}

#[test]
fn a_matched_integration_with_no_reachable_token_still_falls_back_to_a_url() {
    struct NoToken;
    impl TokenSource for NoToken {
        fn token(&self, _integration: &Integration) -> gitpoulpi_hosting::Result<String> {
            Err(gitpoulpi_hosting::Error::Http(
                "no token in this test".into(),
            ))
        }
    }

    let refs = vec![local_branch("feature", Some("origin/feature"))];
    let remotes = [remote("origin", "https://gitlab.example.com/g/p.git")];
    let integrations = [integration(
        "corp",
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example.com",
    )];
    match resolve("feature", &refs, &remotes, &integrations, &NoToken) {
        Resolution::DirectUrl(url) => {
            assert!(
                url.contains("gitlab.example.com/g/p/-/merge_requests/new"),
                "{url}"
            );
            assert!(url.contains("source_branch"), "{url}");
        }
        _ => panic!("expected a direct URL with no reachable token"),
    }
}

#[test]
fn default_target_prefers_a_remote_head_then_main_then_master() {
    let with_head = vec![
        Ref {
            kind: RefKind::RemoteBranch {
                remote: "origin".into(),
                name: "HEAD".into(),
            },
            target: gitpoulpi_core::domain::Oid::from_hex(&"1".repeat(40)).unwrap(),
            visibility: Default::default(),
            is_head: false,
            upstream: None,
            tag_sync: None,
        },
        Ref {
            kind: RefKind::RemoteBranch {
                remote: "origin".into(),
                name: "main".into(),
            },
            target: gitpoulpi_core::domain::Oid::from_hex(&"1".repeat(40)).unwrap(),
            visibility: Default::default(),
            is_head: false,
            upstream: None,
            tag_sync: None,
        },
    ];
    assert_eq!(
        default_target(&with_head, "origin").as_deref(),
        Some("HEAD")
    );

    let with_master = vec![Ref {
        kind: RefKind::RemoteBranch {
            remote: "origin".into(),
            name: "master".into(),
        },
        target: gitpoulpi_core::domain::Oid::from_hex(&"1".repeat(40)).unwrap(),
        visibility: Default::default(),
        is_head: false,
        upstream: None,
        tag_sync: None,
    }];
    assert_eq!(
        default_target(&with_master, "origin").as_deref(),
        Some("master")
    );
    assert_eq!(default_target(&[], "origin"), None);
}
