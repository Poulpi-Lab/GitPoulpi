//! What a host protects (S23 of the history rewrite): the branches a rewrite
//! must not force-push unless the user lifts the protection first.
//!
//! Two pieces, both client side (hosting stays out of the daemon):
//!
//! - [`locate`]: which host and project a repository is on and whether the
//!   user *linked* it. Only a linked repository is ever queried (hosting is
//!   opt-in per repository); an unlinked one still gets the how-to links,
//!   built from its remote's URL, because a link is a string, not a request.
//! - [`lookup`]: what the host's API says is protected, through the
//!   provider's [`HostingClient::protected_branches`]. Whatever the token
//!   cannot read comes back as "unknown" with the reason; it is never an
//!   error that stops a rewrite.
//!
//! What each provider exposes, and with which token (the clients' own docs):
//! GitHub the protected flag of the branches plus the active branch rulesets
//! (a classic token with `repo`, a fine-grained one with Metadata read);
//! GitLab `protected_branches` (`read_api`); Azure DevOps the blocking branch
//! policies (Code read); Bitbucket Cloud the branch restrictions
//! (`repository:admin`), Data Center the branch permissions (repository
//! admin).

use gitpoulpi_config::{ApiFamily, Integration, IntegrationCategory};
use gitpoulpi_core::domain::rewrite::{HostKind, HostProtection};

use crate::client::HostingClient;
use crate::resolve::{resolve_for_repo, ResolveRemote};
use crate::spec::{spec_for, web_base};
use crate::Error;

/// The protected branch names and `*` patterns of one project (short names),
/// and what could not be read of them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProtectedBranches {
    pub patterns: Vec<String>,
    /// Part of the answer is missing (rulesets the token cannot read, a
    /// restriction that names a branching model): English, shown beside the
    /// list.
    pub note: Option<String>,
}

impl ProtectedBranches {
    /// Adds a branch name or pattern (a full `refs/heads/` name is
    /// shortened); empty and repeated ones are dropped.
    pub fn add(&mut self, name: &str) {
        let name = name.trim();
        let short = name.strip_prefix("refs/heads/").unwrap_or(name);
        if !short.is_empty() && !self.patterns.iter().any(|p| p == short) {
            self.patterns.push(short.to_string());
        }
    }

    /// Records one more thing that could not be read.
    pub fn note(&mut self, text: &str) {
        match &mut self.note {
            Some(existing) => {
                existing.push_str("; ");
                existing.push_str(text);
            }
            None => self.note = Some(text.to_string()),
        }
    }
}

/// The answer of [`lookup`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtectionLookup {
    Known(ProtectedBranches),
    /// The host could not say; the reason is English.
    Unknown(String),
}

/// Why a failed read is "unknown" rather than a failure.
pub fn unknown_reason(error: &Error) -> String {
    match error {
        Error::Unauthorized { .. } => {
            "the host rejected the token, so its branch protection could not be read".to_string()
        }
        Error::Forbidden { .. } => {
            "the token lacks the scope to read the host's branch protection".to_string()
        }
        Error::Api { status: 404, .. } => "the token cannot see this repository's branch protection (it needs admin or maintainer access)"
            .to_string(),
        Error::Unsupported(_) | Error::NotImplemented(_) => {
            "this host's client cannot read branch protection".to_string()
        }
        other => format!("the host's branch protection could not be read ({other})"),
    }
}

/// Asks the host which branches of `project` are protected.
pub async fn lookup(client: &dyn HostingClient, project: &str) -> ProtectionLookup {
    match client.protected_branches(project).await {
        Ok(found) => ProtectionLookup::Known(found),
        Err(error) => ProtectionLookup::Unknown(unknown_reason(&error)),
    }
}

/// The repository's host, as far as it is known without a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Located {
    pub kind: HostKind,
    /// The profile that answers for it, when one is configured.
    pub integration: Option<Integration>,
    pub web_base: String,
    pub project: String,
    pub remote: String,
    /// The user linked the repository (a pulls/issues source or a pinned
    /// profile): the only case where the host may be asked.
    pub linked: bool,
}

impl Located {
    /// The how-to's data with the answer of a lookup folded in (`None`: not
    /// asked).
    pub fn protection(&self, lookup: Option<ProtectionLookup>) -> HostProtection {
        let mut out = HostProtection::new(self.kind, self.project.clone(), self.web_base.clone());
        match lookup {
            None => {}
            Some(ProtectionLookup::Known(found)) => {
                out.asked = true;
                out.patterns = found.patterns;
                out.unknown = found.note;
            }
            Some(ProtectionLookup::Unknown(why)) => {
                out.asked = true;
                out.unknown = Some(why);
            }
        }
        out
    }
}

fn host_kind(family: ApiFamily) -> HostKind {
    match family {
        ApiFamily::Github => HostKind::GitHub,
        ApiFamily::Gitlab => HostKind::GitLab,
        ApiFamily::AzureDevops => HostKind::AzureDevOps,
        ApiFamily::Bitbucket => HostKind::Bitbucket,
        _ => HostKind::Other,
    }
}

fn family_of(kind: HostKind) -> Option<ApiFamily> {
    match kind {
        HostKind::GitHub => Some(ApiFamily::Github),
        HostKind::GitLab => Some(ApiFamily::Gitlab),
        HostKind::AzureDevOps => Some(ApiFamily::AzureDevops),
        HostKind::Bitbucket => Some(ApiFamily::Bitbucket),
        HostKind::Other => None,
    }
}

fn is_hosting(integration: &Integration) -> bool {
    integration.enabled && integration.kind.category() == IntegrationCategory::Hosting
}

/// The host of a repository.
///
/// `link` is the saved pulls or issues source (profile id and project),
/// `pinned` the repository's pinned profile; either makes the repository
/// linked. Without one the profile that owns a remote answers (not linked),
/// and without a profile the remote's own URL names the host by its name
/// (`github`, `gitlab`, `dev.azure.com`, `bitbucket`). `remote_hint` is the
/// remote a push went to, tried first.
pub fn locate(
    remotes: &[ResolveRemote],
    integrations: &[Integration],
    link: Option<(&str, &str)>,
    pinned: Option<&str>,
    remote_hint: Option<&str>,
) -> Option<Located> {
    if let Some((id, project)) = link {
        if let Some(integration) = integrations.iter().find(|i| i.id == id && is_hosting(i)) {
            let project = if project.is_empty() {
                resolve_for_repo(remotes, std::slice::from_ref(integration), Some(id))
                    .map(|r| r.project_path)
                    .unwrap_or_default()
            } else {
                project.to_string()
            };
            if let (Some(base), false) = (web_base(integration), project.is_empty()) {
                return Some(Located {
                    kind: host_kind(integration.kind.family()),
                    integration: Some(integration.clone()),
                    web_base: base,
                    project,
                    remote: remote_hint.unwrap_or_default().to_string(),
                    linked: true,
                });
            }
        }
    }
    let ordered: Vec<ResolveRemote> = match remote_hint {
        Some(hint) => remotes
            .iter()
            .filter(|r| r.name == hint)
            .chain(remotes.iter().filter(|r| r.name != hint))
            .cloned()
            .collect(),
        None => remotes.to_vec(),
    };
    if let Some(found) = resolve_for_repo(&ordered, integrations, pinned) {
        if let Some(base) = web_base(found.integration) {
            return Some(Located {
                kind: host_kind(found.integration.kind.family()),
                integration: Some(found.integration.clone()),
                web_base: base,
                project: found.project_path,
                remote: found.remote,
                linked: pinned.is_some_and(|p| p == found.integration.id),
            });
        }
    }
    // No profile: the remote's URL says which host it is.
    for remote in &ordered {
        let kind = HostKind::of_url(&remote.url);
        let Some(family) = family_of(kind) else {
            continue;
        };
        if let Some(parsed) = (spec_for(family).parse_remote)(&remote.url) {
            return Some(Located {
                kind,
                integration: None,
                web_base: parsed.web_base,
                project: parsed.project,
                remote: remote.name.clone(),
                linked: false,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::IntegrationKind;

    fn remote(name: &str, url: &str) -> ResolveRemote {
        ResolveRemote::new(name, url)
    }

    #[test]
    fn an_unlinked_repository_is_located_from_its_remote_url_and_never_asked() {
        let found = locate(
            &[remote("origin", "git@github.com:octo/thing.git")],
            &[],
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(found.kind, HostKind::GitHub);
        assert_eq!(found.web_base, "https://github.com");
        assert_eq!(found.project, "octo/thing");
        assert!(!found.linked, "no link: the host is not to be queried");
        let how = found.protection(None).howto(&[]);
        assert!(how.ways[0].links[0].starts_with("https://github.com/octo/thing/settings"));
    }

    #[test]
    fn a_saved_link_decides_the_project_and_makes_the_repository_linked() {
        let gitlab = Integration::new(
            "corp",
            IntegrationKind::GitlabSelfManaged,
            "https://git.corp.example",
        );
        let found = locate(
            &[remote("origin", "https://elsewhere.example/x/y.git")],
            &[gitlab],
            Some(("corp", "team/sub/proj")),
            None,
            None,
        )
        .unwrap();
        assert!(found.linked);
        assert_eq!(found.kind, HostKind::GitLab);
        assert_eq!(found.project, "team/sub/proj");
        assert_eq!(found.web_base, "https://git.corp.example");
    }

    #[test]
    fn a_failed_read_is_unknown_with_the_reason_never_an_error() {
        let forbidden = Error::Forbidden {
            what: "x".into(),
            detail: "no".into(),
        };
        assert!(unknown_reason(&forbidden).contains("lacks the scope"));
        let missing = Error::Api {
            what: "x".into(),
            status: 404,
            detail: "Not Found".into(),
        };
        assert!(unknown_reason(&missing).contains("admin or maintainer"));
        let located = locate(
            &[remote("origin", "https://github.com/o/r")],
            &[],
            None,
            None,
            None,
        )
        .unwrap();
        let protection = located.protection(Some(ProtectionLookup::Unknown("because".into())));
        assert!(protection.asked && protection.patterns.is_empty());
        assert_eq!(protection.unknown.as_deref(), Some("because"));
    }

    #[test]
    fn names_are_shortened_and_deduplicated() {
        let mut list = ProtectedBranches::default();
        list.add("refs/heads/main");
        list.add("main");
        list.add(" ");
        list.add("release/*");
        assert_eq!(list.patterns, ["main", "release/*"]);
    }
}
