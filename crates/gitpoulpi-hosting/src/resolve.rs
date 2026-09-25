//! Which configured profile answers questions about a repository.
//!
//! With several profiles (two self-managed GitLab hosts, or two accounts
//! on one host) a repository's remote does not identify a profile by
//! host alone. [`resolve_for_repo`] applies one deterministic rule set so
//! the TUI, CLI and MCP all pick the same profile:
//!
//! 1. an explicit per-repo override (`RepoState::hosting_integration`)
//!    wins, if it still names an enabled hosting profile;
//! 2. otherwise remotes are tried in the order `upstream`, `origin`, then
//!    the rest as given, and the first one a profile *owns* is used. Each
//!    family says what owning means (`FamilySpec::owns_remote`): the same
//!    hostname by default (ports are ignored: SSH and HTTPS remotes of one
//!    instance differ in them), the same organization for Azure DevOps;
//! 3. several profiles on that host: the one whose user owns the
//!    namespace (`login == first path segment`) wins, else the first
//!    configured -- and the result is flagged `ambiguous` so a frontend
//!    can offer to pin a choice.

use gitpoulpi_config::{Integration, IntegrationCategory};

use crate::spec::{spec_for, RemoteRef};

/// A remote as the resolver sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveRemote {
    pub name: String,
    pub url: String,
}

impl ResolveRemote {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution<'a> {
    pub integration: &'a Integration,
    /// The remote the project path was read from.
    pub remote: String,
    /// `group/sub/proj` (GitLab) or `owner/name` (GitHub).
    pub project_path: String,
    /// More than one enabled profile matched and none was pinned.
    pub ambiguous: bool,
}

/// The remote as `integration`'s family reads it, if it owns it.
fn owned_by(integration: &Integration, url: &str) -> Option<RemoteRef> {
    let spec = spec_for(integration.kind.family());
    let remote = (spec.parse_remote)(url)?;
    (spec.owns_remote)(integration, &remote).then_some(remote)
}

fn is_hosting(integration: &Integration) -> bool {
    integration.enabled && integration.kind.category() == IntegrationCategory::Hosting
}

fn remote_order(remotes: &[ResolveRemote]) -> Vec<&ResolveRemote> {
    let mut ordered: Vec<&ResolveRemote> = Vec::with_capacity(remotes.len());
    for special in ["upstream", "origin"] {
        ordered.extend(remotes.iter().filter(|r| r.name == special));
    }
    ordered.extend(
        remotes
            .iter()
            .filter(|r| r.name != "upstream" && r.name != "origin"),
    );
    ordered
}

pub fn resolve_for_repo<'a>(
    remotes: &[ResolveRemote],
    integrations: &'a [Integration],
    override_id: Option<&str>,
) -> Option<Resolution<'a>> {
    let ordered = remote_order(remotes);

    if let Some(id) = override_id {
        // A stale override (deleted, disabled, or now an issue tracker)
        // silently falls through to automatic resolution.
        if let Some(chosen) = integrations.iter().find(|i| i.id == id && is_hosting(i)) {
            let spec = spec_for(chosen.kind.family());
            let pick = ordered
                .iter()
                .find_map(|r| owned_by(chosen, &r.url).map(|p| (r, p)))
                .or_else(|| {
                    ordered
                        .iter()
                        .find_map(|r| (spec.parse_remote)(&r.url).map(|p| (r, p)))
                })?;
            return Some(Resolution {
                integration: chosen,
                remote: pick.0.name.clone(),
                project_path: pick.1.project,
                ambiguous: false,
            });
        }
    }

    for remote in &ordered {
        let candidates: Vec<(&Integration, RemoteRef)> = integrations
            .iter()
            .filter(|i| is_hosting(i))
            .filter_map(|i| owned_by(i, &remote.url).map(|r| (i, r)))
            .collect();
        if candidates.is_empty() {
            continue;
        }
        let namespace = candidates[0].1.project.split('/').next().unwrap_or("");
        let owner = candidates.iter().find(|(i, _)| {
            i.user
                .as_ref()
                .is_some_and(|u| !u.login.is_empty() && u.login.eq_ignore_ascii_case(namespace))
        });
        let (chosen, parsed) = owner.unwrap_or(&candidates[0]);
        return Some(Resolution {
            integration: chosen,
            remote: remote.name.clone(),
            project_path: parsed.project.clone(),
            ambiguous: candidates.len() > 1,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::{ConnectedUser, IntegrationKind};

    fn gl(id: &str, url: &str) -> Integration {
        Integration::new(id, IntegrationKind::GitlabSelfManaged, url)
    }

    fn with_user(mut i: Integration, login: &str) -> Integration {
        i.user = Some(ConnectedUser {
            login: login.into(),
            ..Default::default()
        });
        i
    }

    fn origin(url: &str) -> Vec<ResolveRemote> {
        vec![ResolveRemote::new("origin", url)]
    }

    #[test]
    fn two_self_managed_hosts_resolve_by_host() {
        let ints = vec![
            gl("a", "https://gitlab.a.example"),
            gl("b", "https://gitlab.b.example"),
        ];
        let r = resolve_for_repo(&origin("git@gitlab.b.example:g/sub/p.git"), &ints, None).unwrap();
        assert_eq!(r.integration.id, "b");
        assert_eq!(r.project_path, "g/sub/p");
        assert_eq!(r.remote, "origin");
        assert!(!r.ambiguous);
    }

    #[test]
    fn every_remote_spelling_reaches_the_same_profile() {
        let ints = vec![gl("corp", "https://gitlab.example.com")];
        for url in [
            "https://gitlab.example.com/g/p.git",
            "https://gitlab.example.com:8443/g/p.git",
            "ssh://git@gitlab.example.com:2222/g/p.git",
            "git@gitlab.example.com:g/p.git",
            "HTTPS://GitLab.Example.COM/g/p",
        ] {
            let r = resolve_for_repo(&origin(url), &ints, None).unwrap();
            assert_eq!(r.integration.id, "corp", "{url}");
            assert_eq!(r.project_path, "g/p", "{url}");
        }
    }

    #[test]
    fn a_profile_url_with_a_port_still_matches() {
        let ints = vec![gl("corp", "https://gitlab.example.com:8443")];
        assert!(resolve_for_repo(&origin("git@gitlab.example.com:g/p.git"), &ints, None).is_some());
    }

    #[test]
    fn two_users_on_one_host_prefer_the_namespace_owner() {
        let ints = vec![
            with_user(gl("work", "https://gitlab.example.com"), "corp"),
            with_user(gl("me", "https://gitlab.example.com"), "sg"),
        ];
        let r = resolve_for_repo(
            &origin("https://gitlab.example.com/sg/thing.git"),
            &ints,
            None,
        )
        .unwrap();
        assert_eq!(r.integration.id, "me");
        assert!(r.ambiguous);

        let r = resolve_for_repo(
            &origin("https://gitlab.example.com/other/thing.git"),
            &ints,
            None,
        )
        .unwrap();
        assert_eq!(
            r.integration.id, "work",
            "first configured when nobody owns it"
        );
        assert!(r.ambiguous);
    }

    #[test]
    fn an_override_beats_the_automatic_choice() {
        let ints = vec![
            with_user(gl("work", "https://gitlab.example.com"), "corp"),
            with_user(gl("me", "https://gitlab.example.com"), "sg"),
        ];
        let r = resolve_for_repo(
            &origin("https://gitlab.example.com/sg/t.git"),
            &ints,
            Some("work"),
        )
        .unwrap();
        assert_eq!(r.integration.id, "work");
        assert!(!r.ambiguous);
    }

    #[test]
    fn a_stale_or_disabled_override_falls_back_to_automatic() {
        let mut off = gl("off", "https://gitlab.example.com");
        off.enabled = false;
        let ints = vec![off, gl("on", "https://gitlab.example.com")];
        for id in ["deleted", "off"] {
            let r = resolve_for_repo(
                &origin("https://gitlab.example.com/g/p.git"),
                &ints,
                Some(id),
            )
            .unwrap();
            assert_eq!(r.integration.id, "on", "{id}");
        }
    }

    #[test]
    fn upstream_is_tried_before_origin_before_the_rest() {
        let ints = vec![
            gl("a", "https://gitlab.a.example"),
            gl("b", "https://gitlab.b.example"),
        ];
        let remotes = vec![
            ResolveRemote::new("zzz", "https://gitlab.b.example/z/p.git"),
            ResolveRemote::new("origin", "https://gitlab.a.example/me/p.git"),
            ResolveRemote::new("upstream", "https://gitlab.b.example/team/p.git"),
        ];
        let r = resolve_for_repo(&remotes, &ints, None).unwrap();
        assert_eq!(
            (r.integration.id.as_str(), r.remote.as_str()),
            ("b", "upstream")
        );
        assert_eq!(r.project_path, "team/p");

        // Only `origin` unmatched: the next remote is used.
        let ints = vec![gl("b", "https://gitlab.b.example")];
        let r = resolve_for_repo(&remotes, &ints, None).unwrap();
        assert_eq!(r.remote, "upstream");
    }

    #[test]
    fn unknown_hosts_disabled_profiles_and_issue_trackers_match_nothing() {
        let mut off = gl("off", "https://gitlab.example.com");
        off.enabled = false;
        let jira = Integration::new(
            "j",
            IntegrationKind::JiraCloud,
            "https://gitlab.example.com",
        );
        let ints = vec![off, jira];
        assert!(
            resolve_for_repo(&origin("https://gitlab.example.com/g/p.git"), &ints, None).is_none()
        );
        assert!(
            resolve_for_repo(&origin("https://elsewhere.example/g/p.git"), &ints, None).is_none()
        );
        assert!(resolve_for_repo(&[], &ints, None).is_none());
    }

    #[test]
    fn github_dot_com_matches_its_public_profile() {
        let ints = vec![Integration::new(
            "gh",
            IntegrationKind::Github,
            "https://github.com",
        )];
        let r = resolve_for_repo(&origin("git@github.com:me/thing.git"), &ints, None).unwrap();
        assert_eq!(r.integration.id, "gh");
        assert_eq!(r.project_path, "me/thing");
    }
}

/// One proposal for linking a repository: a remote whose host an enabled
/// hosting profile serves, with the project path read from it. Only ever a
/// *suggestion* - nothing links a repository on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    pub integration_id: String,
    pub remote: String,
    pub project_path: String,
}

/// The profiles that match the repository's remotes, `origin` first (then
/// `upstream`, then the rest as given); every enabled hosting profile on a
/// remote's host is proposed, once per `(profile, project)`.
pub fn suggest_for_repo(
    remotes: &[ResolveRemote],
    integrations: &[Integration],
) -> Vec<Suggestion> {
    let mut ordered: Vec<&ResolveRemote> = Vec::with_capacity(remotes.len());
    for special in ["origin", "upstream"] {
        ordered.extend(remotes.iter().filter(|r| r.name == special));
    }
    ordered.extend(
        remotes
            .iter()
            .filter(|r| r.name != "upstream" && r.name != "origin"),
    );
    let mut out: Vec<Suggestion> = Vec::new();
    for remote in ordered {
        for integration in integrations.iter().filter(|i| is_hosting(i)) {
            let Some(parsed) = owned_by(integration, &remote.url) else {
                continue;
            };
            let dup = out
                .iter()
                .any(|s| s.integration_id == integration.id && s.project_path == parsed.project);
            if !dup {
                out.push(Suggestion {
                    integration_id: integration.id.clone(),
                    remote: remote.name.clone(),
                    project_path: parsed.project,
                });
            }
        }
    }
    out
}
