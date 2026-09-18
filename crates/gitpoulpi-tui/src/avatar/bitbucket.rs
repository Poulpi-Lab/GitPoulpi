//! Bitbucket Cloud as an avatar source: pure URL building and parsing, no
//! I/O (the fetching is `resolve`'s, over an injected `Fetch`).
//!
//! Only used when an enabled `[[integrations]]` entry of kind `bitbucket`
//! exists and the repository has a `bitbucket.org` remote -- a host the
//! user has not connected is never asked about their authors.
//!
//! Verified 2026-10-08 against Atlassian's published OpenAPI document
//! (`https://api.bitbucket.org/swagger.json`): `GET
//! /2.0/repositories/{workspace}/{repo_slug}/commit/{commit}` answers the
//! commit with `author.raw` (the git identity) and, when Bitbucket linked
//! the commit's email to an account, `author.user.links.avatar.href`; an
//! unknown commit or an invisible repository answers 404. The rate limit
//! is a rolling hour and answers 429 (Atlassian's "Bitbucket Cloud rate
//! limit troubleshooting" page); `Retry-After` is not documented there, so
//! the shared hour-long default applies when it is absent.
//!
//! Bitbucket Data Center and Server have no such endpoint that could be
//! verified (and no public avatar host), so they are not a source.

use gitpoulpi_config::{Integration, IntegrationKind};

/// The Bitbucket Cloud repository a remote points at, and the integration
/// that owns it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BitbucketRepo {
    pub(crate) workspace: String,
    pub(crate) slug: String,
    /// The integration whose credentials the lookup may use (never
    /// another host's: the API host is fixed).
    pub(crate) integration: Integration,
}

impl BitbucketRepo {
    /// `workspace/slug`, lowercased: the session's "unreachable" key.
    pub(crate) fn key(&self) -> String {
        format!("bb:{}/{}", self.workspace, self.slug).to_ascii_lowercase()
    }
}

/// An enabled Bitbucket Cloud profile.
pub(crate) fn is_bitbucket(integration: &Integration) -> bool {
    integration.enabled && integration.kind == IntegrationKind::Bitbucket
}

/// `(workspace, slug)` of a `bitbucket.org` remote, for every spelling a
/// clone leaves in `.git/config`: `https://[user@]bitbucket.org/w/r(.git)`,
/// `git@bitbucket.org:w/r.git`, `ssh://git@bitbucket.org(:22)/w/r.git`.
/// `None` for any other host.
pub(crate) fn repo_of_remote(url: &str) -> Option<(String, String)> {
    let url = url.trim();
    let (host, path) = if let Some((_, rest)) = url.split_once("://") {
        let rest = rest.rsplit_once('@').map_or(rest, |(_, r)| r);
        let (host, path) = rest.split_once('/')?;
        (host.split(':').next()?, path)
    } else {
        let rest = url.rsplit_once('@').map_or(url, |(_, r)| r);
        rest.split_once(':')?
    };
    if !matches!(
        host.to_ascii_lowercase().as_str(),
        "bitbucket.org" | "www.bitbucket.org"
    ) {
        return None;
    }
    let path = path.trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut parts = path.split('/');
    let (workspace, slug) = (parts.next()?, parts.next()?);
    if parts.next().is_some() || workspace.is_empty() || slug.is_empty() {
        return None;
    }
    Some((workspace.to_string(), slug.to_string()))
}

/// The repository to ask: `origin` when it is on bitbucket.org, else the
/// first remote that is, provided an enabled Bitbucket integration exists.
pub(crate) fn pick_repo<'a>(
    remotes: impl IntoIterator<Item = (&'a str, &'a str)>,
    integrations: &[Integration],
) -> Option<BitbucketRepo> {
    let integration = integrations.iter().find(|i| is_bitbucket(i))?;
    let mut first = None;
    for (name, url) in remotes {
        if let Some((workspace, slug)) = repo_of_remote(url) {
            let repo = BitbucketRepo {
                workspace,
                slug,
                integration: integration.clone(),
            };
            if name == "origin" {
                return Some(repo);
            }
            first.get_or_insert(repo);
        }
    }
    first
}

/// Percent-encodes everything but unreserved characters.
fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// `GET /2.0/repositories/{workspace}/{repo_slug}/commit/{commit}`.
pub(crate) fn commit_api_url(repo: &BitbucketRepo, sha: &str) -> String {
    format!(
        "https://api.bitbucket.org/2.0/repositories/{}/{}/commit/{}",
        encode(&repo.workspace),
        encode(&repo.slug),
        encode(sha)
    )
}

/// `author.user.links.avatar.href` out of a "get a commit" response. `None`
/// when Bitbucket linked no account to the commit's email, or the body is
/// not what the endpoint documents.
pub(crate) fn author_avatar_url(body: &[u8]) -> Option<String> {
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let url = value
        .get("author")?
        .get("user")?
        .get("links")?
        .get("avatar")?
        .get("href")?
        .as_str()?
        .trim();
    url.starts_with("https://").then(|| url.to_string())
}

/// How a Bitbucket API call authenticates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Auth {
    /// `email:api_token`.
    Basic { user: String, token: String },
    /// A token alone.
    Bearer(String),
}

/// The credentials of `integration`, if a token is stored: the account
/// (email) with it makes Basic, a bare token Bearer - the same rule as
/// `gitpoulpi-hosting`'s `auth_headers`, restated because this crate does
/// not depend on it. Runs `git credential fill` at worst: call on the
/// worker, never on the input thread.
pub(crate) fn auth_of(integration: &Integration) -> Option<Auth> {
    let token = super::token_of(integration)?;
    let user = integration.account.trim();
    Some(if user.is_empty() {
        Auth::Bearer(token)
    } else {
        Auth::Basic {
            user: user.to_string(),
            token,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(enabled: bool) -> Integration {
        let mut i = Integration::new("bb", IntegrationKind::Bitbucket, "https://bitbucket.org");
        i.enabled = enabled;
        i
    }

    #[test]
    fn every_clone_spelling_names_the_repository() {
        for url in [
            "https://bitbucket.org/ws/repo.git",
            "https://me@bitbucket.org/ws/repo",
            "git@bitbucket.org:ws/repo.git",
            "ssh://git@bitbucket.org:22/ws/repo.git",
            "https://www.bitbucket.org/ws/repo/",
        ] {
            assert_eq!(
                repo_of_remote(url),
                Some(("ws".into(), "repo".into())),
                "{url}"
            );
        }
    }

    #[test]
    fn other_hosts_and_other_shapes_are_not_bitbucket_cloud() {
        for url in [
            "https://bitbucket.example.com/scm/ws/repo.git",
            "https://github.com/ws/repo.git",
            "https://bitbucket.org/ws",
            "https://bitbucket.org/ws/repo/extra",
            "not a url",
        ] {
            assert_eq!(repo_of_remote(url), None, "{url}");
        }
    }

    #[test]
    fn a_remote_matches_only_with_an_enabled_integration_and_origin_wins() {
        let remotes = [
            ("up", "git@bitbucket.org:other/fork.git"),
            ("origin", "https://bitbucket.org/ws/repo.git"),
        ];
        let repo = pick_repo(remotes, &[profile(true)]).unwrap();
        assert_eq!(
            (repo.workspace.as_str(), repo.slug.as_str()),
            ("ws", "repo")
        );
        assert_eq!(repo.key(), "bb:ws/repo");
        assert!(pick_repo(remotes, &[profile(false)]).is_none());
        assert!(pick_repo(remotes, &[]).is_none());
        let only_up = [("up", "git@bitbucket.org:other/fork.git")];
        assert_eq!(pick_repo(only_up, &[profile(true)]).unwrap().slug, "fork");
        assert!(pick_repo([("origin", "https://gitlab.com/a/b")], &[profile(true)]).is_none());
    }

    #[test]
    fn the_commit_url_is_built_on_the_api_host_with_encoded_parts() {
        let repo = pick_repo(
            [("origin", "https://bitbucket.org/ws/re po.git")],
            &[profile(true)],
        )
        .unwrap();
        assert_eq!(
            commit_api_url(&repo, "abc123"),
            "https://api.bitbucket.org/2.0/repositories/ws/re%20po/commit/abc123"
        );
    }

    #[test]
    fn the_account_picture_is_read_from_the_linked_author() {
        let linked = br#"{"author":{"raw":"A <a@x>","user":{"display_name":"A","uuid":"{u}",
            "links":{"avatar":{"href":"https://secure.gravatar.com/avatar/x?d=y"}}}}}"#;
        assert_eq!(
            author_avatar_url(linked).as_deref(),
            Some("https://secure.gravatar.com/avatar/x?d=y")
        );
        // No account linked to the email.
        assert_eq!(author_avatar_url(br#"{"author":{"raw":"A <a@x>"}}"#), None);
        // Not https, not JSON.
        assert_eq!(
            author_avatar_url(
                br#"{"author":{"user":{"links":{"avatar":{"href":"http://x/y.png"}}}}}"#
            ),
            None
        );
        assert_eq!(author_avatar_url(b"<html>"), None);
    }
}
