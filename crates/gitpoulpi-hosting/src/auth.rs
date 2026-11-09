//! Which request headers carry a token, per integration kind. Pure: no
//! I/O, so every scheme is a table-test row.
//!
//! | Kind | Header(s) |
//! |---|---|
//! | github* | `Authorization: Bearer <token>` |
//! | gitlab* | `PRIVATE-TOKEN: <token>` + `Authorization: Bearer <token>` |
//! | azure_devops* | `Authorization: Basic b64(":" + token)` |
//! | bitbucket | `Authorization: Basic b64(account + ":" + token)`; empty account: `Bearer` (workspace / repository access tokens) |
//! | gitea, forgejo | `Authorization: token <token>` |
//! | bitbucket_data_center, jira_data_center | `Authorization: Bearer <token>` |
//! | jira_cloud | `Authorization: Basic b64(account + ":" + token)` |
//! | linear | `Authorization: <api key>` (no `Bearer`) |
//! | youtrack | `Authorization: Bearer <permanent token>` |
//! | redmine | `X-Redmine-API-Key: <api key>` |
//! | gerrit | `Authorization: Basic b64(account + ":" + HTTP password)` |
//! | trello | `Authorization: OAuth oauth_consumer_key="<account>", oauth_token="<token>"` |
//!
//! A Jira Cloud profile with no account still yields a (useless) Basic
//! header: the host answers 401 and the form asks for the email, rather
//! than this layer inventing a second error path.

use crate::integration::{Integration, IntegrationKind};

/// The headers that authenticate `token` for `integration`.
pub fn auth_headers(integration: &Integration, token: &str) -> Vec<(String, String)> {
    let authorization = |value: String| ("Authorization".to_string(), value);
    match integration.kind {
        IntegrationKind::Github | IntegrationKind::GithubEnterprise => {
            vec![authorization(format!("Bearer {token}"))]
        }
        IntegrationKind::Gitlab | IntegrationKind::GitlabSelfManaged => vec![
            ("PRIVATE-TOKEN".to_string(), token.to_string()),
            authorization(format!("Bearer {token}")),
        ],
        IntegrationKind::AzureDevops | IntegrationKind::AzureDevopsServer => {
            vec![authorization(basic("", token))]
        }
        IntegrationKind::Bitbucket => {
            let account = integration.account.trim();
            if account.is_empty() {
                vec![authorization(format!("Bearer {token}"))]
            } else {
                vec![authorization(basic(account, token))]
            }
        }
        // Gitea and Forgejo: `token <t>` is their documented scheme.
        IntegrationKind::Gitea | IntegrationKind::Forgejo => {
            vec![authorization(format!("token {token}"))]
        }
        // Linear's personal API keys go bare; OAuth tokens would be `Bearer`.
        IntegrationKind::Linear => vec![authorization(token.to_string())],
        IntegrationKind::BitbucketDataCenter
        | IntegrationKind::JiraDataCenter
        | IntegrationKind::Youtrack => {
            vec![authorization(format!("Bearer {token}"))]
        }
        IntegrationKind::JiraCloud => {
            vec![authorization(basic(integration.account.trim(), token))]
        }
        // Redmine's REST API key travels in its own header.
        IntegrationKind::Redmine => vec![("X-Redmine-API-Key".to_string(), token.to_string())],
        // Gerrit: HTTP basic with the user name and the generated HTTP password.
        IntegrationKind::Gerrit => vec![authorization(basic(integration.account.trim(), token))],
        IntegrationKind::LocalMarkdown => Vec::new(),
        IntegrationKind::Trello => vec![authorization(format!(
            "OAuth oauth_consumer_key=\"{}\", oauth_token=\"{token}\"",
            integration.account.trim()
        ))],
    }
}

fn basic(user: &str, token: &str) -> String {
    format!("Basic {}", base64(format!("{user}:{token}").as_bytes()))
}

/// The user name git's HTTPS transport sends together with `token` for
/// `kind`, or `None` for a kind that is not a code host (issue trackers, a
/// folder of notes). Pure, so every host is a table-test row.
///
/// | Kind | User name |
/// |---|---|
/// | github, github enterprise | `x-access-token` |
/// | gitlab* | `oauth2` |
/// | bitbucket cloud | the account, or `x-token-auth` for an access token (no account) |
/// | bitbucket data center | the account, else the login |
/// | azure devops* | any non-empty name: the account, else the login, else `gitpoulpi` |
/// | gitea, forgejo | the login, else the account |
/// | gerrit | the account (HTTP password auth), else the login |
///
/// `account` is the profile's configured account, `login` the one the host
/// reported on the last check. A profile's own `username` overrides this.
pub fn git_username(kind: IntegrationKind, account: &str, login: &str) -> Option<String> {
    let (account, login) = (account.trim(), login.trim());
    let first = |names: &[&str], fallback: &str| {
        names
            .iter()
            .find(|n| !n.is_empty())
            .copied()
            .unwrap_or(fallback)
            .to_string()
    };
    Some(match kind {
        IntegrationKind::Github | IntegrationKind::GithubEnterprise => "x-access-token".into(),
        IntegrationKind::Gitlab | IntegrationKind::GitlabSelfManaged => "oauth2".into(),
        IntegrationKind::Bitbucket => first(&[account], "x-token-auth"),
        IntegrationKind::BitbucketDataCenter => first(&[account, login], "x-token-auth"),
        IntegrationKind::AzureDevops | IntegrationKind::AzureDevopsServer => {
            first(&[account, login], "gitpoulpi")
        }
        IntegrationKind::Gitea | IntegrationKind::Forgejo => first(&[login, account], "git"),
        IntegrationKind::Gerrit => first(&[account, login], "git"),
        IntegrationKind::JiraCloud
        | IntegrationKind::JiraDataCenter
        | IntegrationKind::Trello
        | IntegrationKind::Linear
        | IntegrationKind::Youtrack
        | IntegrationKind::Redmine
        | IntegrationKind::LocalMarkdown => return None,
    })
}

/// RFC 4648 base64 with padding. Twenty lines beat a dependency.
pub fn base64(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let n = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_the_rfc_4648_vectors() {
        for (input, want) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(input.as_bytes()), want, "{input:?}");
        }
    }

    #[test]
    fn git_usernames_per_host() {
        use IntegrationKind as K;
        let name = git_username;
        assert_eq!(name(K::Github, "me", "me").as_deref(), Some("x-access-token"));
        assert_eq!(name(K::GithubEnterprise, "", "").as_deref(), Some("x-access-token"));
        assert_eq!(name(K::Gitlab, "me", "").as_deref(), Some("oauth2"));
        assert_eq!(name(K::GitlabSelfManaged, "", "").as_deref(), Some("oauth2"));
        assert_eq!(name(K::Bitbucket, "ada", "").as_deref(), Some("ada"));
        assert_eq!(name(K::Bitbucket, " ", "ada").as_deref(), Some("x-token-auth"));
        assert_eq!(name(K::BitbucketDataCenter, "", "ada").as_deref(), Some("ada"));
        assert_eq!(name(K::AzureDevops, "", "").as_deref(), Some("gitpoulpi"));
        assert_eq!(name(K::AzureDevops, "", "ada").as_deref(), Some("ada"));
        assert_eq!(name(K::Gitea, "acct", "ada").as_deref(), Some("ada"));
        assert_eq!(name(K::Forgejo, "acct", "").as_deref(), Some("acct"));
        assert_eq!(name(K::Gerrit, "ada", "other").as_deref(), Some("ada"));
        for k in [
            K::JiraCloud,
            K::JiraDataCenter,
            K::Trello,
            K::Linear,
            K::Youtrack,
            K::Redmine,
            K::LocalMarkdown,
        ] {
            assert_eq!(name(k, "a", "b"), None, "{k:?}");
        }
    }

    fn profile(kind: IntegrationKind, account: &str) -> Integration {
        let mut i = Integration::new("x", kind, "https://h.example");
        i.account = account.into();
        i
    }

    fn headers(kind: IntegrationKind, account: &str) -> Vec<(String, String)> {
        auth_headers(&profile(kind, account), "tok")
    }

    #[test]
    fn every_kind_has_its_scheme() {
        let bearer = vec![("Authorization".to_string(), "Bearer tok".to_string())];
        let basic = |v: &str| vec![("Authorization".to_string(), format!("Basic {v}"))];
        // ":tok", "me@x:tok", ":tok" (Jira with no account).
        assert_eq!(base64(b":tok"), "OnRvaw==");
        assert_eq!(base64(b"me@x:tok"), "bWVAeDp0b2s=");

        assert_eq!(headers(IntegrationKind::Github, ""), bearer);
        assert_eq!(headers(IntegrationKind::GithubEnterprise, ""), bearer);
        for kind in [IntegrationKind::Gitlab, IntegrationKind::GitlabSelfManaged] {
            assert_eq!(
                headers(kind, ""),
                vec![
                    ("PRIVATE-TOKEN".to_string(), "tok".to_string()),
                    ("Authorization".to_string(), "Bearer tok".to_string()),
                ]
            );
        }
        for kind in [
            IntegrationKind::AzureDevops,
            IntegrationKind::AzureDevopsServer,
        ] {
            assert_eq!(headers(kind, ""), basic("OnRvaw=="));
        }
        assert_eq!(
            headers(IntegrationKind::Bitbucket, "me@x"),
            basic("bWVAeDp0b2s=")
        );
        assert_eq!(
            headers(IntegrationKind::Bitbucket, ""),
            bearer,
            "empty account"
        );
        assert_eq!(headers(IntegrationKind::Bitbucket, "  "), bearer);
        assert_eq!(headers(IntegrationKind::BitbucketDataCenter, ""), bearer);
        assert_eq!(headers(IntegrationKind::JiraDataCenter, ""), bearer);
        assert_eq!(
            headers(IntegrationKind::JiraCloud, "me@x"),
            basic("bWVAeDp0b2s=")
        );
        assert_eq!(
            headers(IntegrationKind::Redmine, ""),
            vec![("X-Redmine-API-Key".to_string(), "tok".to_string())]
        );
        // Gerrit: the user name and the HTTP password, as basic auth.
        assert_eq!(
            headers(IntegrationKind::Gerrit, "me@x"),
            basic("bWVAeDp0b2s=")
        );
        assert_eq!(
            headers(IntegrationKind::Trello, "KEY"),
            vec![(
                "Authorization".to_string(),
                "OAuth oauth_consumer_key=\"KEY\", oauth_token=\"tok\"".to_string()
            )]
        );
    }

    #[test]
    fn a_token_never_lands_in_a_url_scheme() {
        // Trello's key/token go in the header, never in the query.
        let h = headers(IntegrationKind::Trello, "KEY");
        assert!(h.iter().all(|(k, _)| k == "Authorization"));
    }
}
