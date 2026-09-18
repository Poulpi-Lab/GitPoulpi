//! "Copy link to branch / to commit on remote": the page on the host that
//! shows a branch or a commit, built from the remote's URL and the host
//! family -- no network, no token.

use super::*;

use gitpoulpi_core::domain::Remote;
use gitpoulpi_hosting::{remote_web_base, well_known_kind, Integration};
use gitpoulpi_i18n::msg::remotes as msg;

/// What the link points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LinkTarget {
    /// A branch, by the name the remote spells it (`feature/x`).
    Branch(String),
    Commit(Oid),
}

/// The remote a link is about: the one named, else `origin`, else the
/// first.
pub(crate) fn pick_remote<'a>(
    remotes: &'a [Remote],
    preferred: Option<&str>,
) -> Option<&'a Remote> {
    preferred
        .and_then(|p| remotes.iter().find(|r| r.name == p))
        .or_else(|| remotes.iter().find(|r| r.name == "origin"))
        .or_else(|| remotes.first())
}

/// The browser URL of `target` on `remote`, or why there is none.
///
/// GitHub-family hosts use `/tree/<branch>` and `/commit/<sha>`, GitLab
/// ones `/-/tree/...` and `/-/commit/...`, Bitbucket `/branch/...` and
/// `/commits/...`. A host that is neither configured as an integration
/// nor `github.com`/`gitlab.com` is refused rather than guessed at.
pub(crate) fn remote_link_url(
    target: &LinkTarget,
    remote: &Remote,
    integrations: &[Integration],
) -> Result<String, String> {
    let Some((web_base, project)) = remote_web_base(&remote.fetch_url) else {
        return Err(msg::HOST_NOT_RECOGNIZED.get().into());
    };
    let matched = crate::pull_request::matching_integration(remote, integrations);
    let web_base = gitpoulpi_hosting::web_base_for(matched, web_base);
    let family = matched
        .map(|i| i.kind.family())
        .or_else(|| well_known_kind(&web_base).map(|k| k.family()));
    let Some(spec) = family.map(gitpoulpi_hosting::spec_for) else {
        return Err(msg::HOST_NOT_RECOGNIZED.get().into());
    };
    match target {
        LinkTarget::Branch(name) => (spec.branch_url)(&web_base, &project, name),
        LinkTarget::Commit(oid) => (spec.commit_url)(&web_base, &project, &oid.to_hex()),
    }
    .ok_or_else(|| msg::HOST_NOT_RECOGNIZED.get().into())
}

/// Puts the link on the clipboard; always returns the status-line text.
pub(crate) fn copy_remote_link(
    target: &LinkTarget,
    remotes: &[Remote],
    preferred_remote: Option<&str>,
    clipboard: &mut clipboard::Clipboard,
) -> String {
    let Some(remote) = pick_remote(remotes, preferred_remote) else {
        return msg::NO_REMOTE.get().into();
    };
    match remote_link_url(target, remote, &crate::pull_request::load_integrations()) {
        Ok(url) => {
            let what = match target {
                LinkTarget::Branch(_) => msg::LINK_BRANCH.get(),
                LinkTarget::Commit(_) => msg::LINK_COMMIT.get(),
            };
            clipboard.copy(what, &url)
        }
        Err(why) => why,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn remote(url: &str) -> Remote {
        Remote {
            name: "origin".into(),
            fetch_url: url.into(),
            push_url: None,
        }
    }

    #[test]
    fn github_and_gitlab_links_use_their_own_paths() {
        let oid = Oid::from_hex(&"ab".repeat(20)).expect("oid");
        assert_eq!(
            remote_link_url(
                &LinkTarget::Branch("feat/x".into()),
                &remote("git@github.com:me/thing.git"),
                &[]
            )
            .unwrap(),
            "https://github.com/me/thing/tree/feat/x"
        );
        assert_eq!(
            remote_link_url(
                &LinkTarget::Commit(oid),
                &remote("https://gitlab.com/g/sub/p.git"),
                &[]
            )
            .unwrap(),
            format!("https://gitlab.com/g/sub/p/-/commit/{}", oid.to_hex())
        );
    }

    /// A self-hosted Gitea in a plain http Docker container: the link
    /// keeps http and the port, whether the remote is http or ssh.
    #[test]
    fn a_plain_http_server_keeps_its_scheme_and_port() {
        let gitea = Integration::new(
            "g",
            gitpoulpi_hosting::IntegrationKind::Gitea,
            "http://localhost:3000",
        );
        for fetch in [
            "http://localhost:3000/me/thing.git",
            "ssh://git@localhost:2222/me/thing.git",
        ] {
            assert_eq!(
                remote_link_url(
                    &LinkTarget::Branch("feat/x".into()),
                    &remote(fetch),
                    std::slice::from_ref(&gitea)
                )
                .unwrap(),
                "http://localhost:3000/me/thing/src/branch/feat/x",
                "{fetch}"
            );
        }
    }

    #[test]
    fn an_unknown_host_is_refused() {
        assert!(remote_link_url(
            &LinkTarget::Branch("x".into()),
            &remote("https://git.internal.example/a/b.git"),
            &[]
        )
        .is_err());
    }

    #[test]
    fn the_preferred_remote_wins_then_origin_then_first() {
        let remotes = vec![
            Remote {
                name: "up".into(),
                fetch_url: "u".into(),
                push_url: None,
            },
            remote("o"),
        ];
        assert_eq!(pick_remote(&remotes, Some("up")).unwrap().name, "up");
        assert_eq!(pick_remote(&remotes, None).unwrap().name, "origin");
        assert_eq!(pick_remote(&remotes[..1], Some("zz")).unwrap().name, "up");
    }
}
