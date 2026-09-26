//! What the repository's host protects, for `gitpoulpi rewrite` (S23).
//!
//! Opt-in per repository: the host is asked only when the user linked the
//! repository (a pulls or issues source, or a pinned profile); an unlinked
//! one gets the how-to links built from its remote's URL and nothing else.
//! A token that cannot read the protection is "unknown", never an error.

use gitpoulpi_core::domain::rewrite::HostProtection;
use gitpoulpi_core::domain::Remote;
use gitpoulpi_hosting::protection::{locate, lookup, ProtectionLookup};
use gitpoulpi_hosting::ResolveRemote;

use super::HostingEnv;

impl HostingEnv {
    /// The host of the repository, with what it protects when `ask` is set
    /// and the repository is linked. `remote_hint` is the remote a push went
    /// to. `None`: no host could be named (a plain server).
    pub async fn rewrite_host_protection(
        &self,
        remotes: &[Remote],
        state: &gitpoulpi_session::RepoState,
        remote_hint: Option<&str>,
        ask: bool,
    ) -> Option<HostProtection> {
        let integrations = self.integrations().unwrap_or_default();
        let resolve_remotes: Vec<ResolveRemote> = remotes
            .iter()
            .map(|r| ResolveRemote::new(r.name.clone(), r.fetch_url.clone()))
            .collect();
        let link = state
            .pulls_source
            .as_ref()
            .or(state.issues_source.as_ref())
            .map(|s| (s.integration.as_str(), s.project.as_str()));
        let located = locate(
            &resolve_remotes,
            &integrations,
            link,
            state.hosting_integration.as_deref(),
            remote_hint,
        )?;
        if !(ask && located.linked) {
            return Some(located.protection(None));
        }
        let answer = match &located.integration {
            Some(integration) => match self.client(integration, &integrations) {
                Ok(client) => lookup(&*client, &located.project).await,
                Err(why) => ProtectionLookup::Unknown(why),
            },
            None => ProtectionLookup::Unknown("no integration profile answers for it".into()),
        };
        Some(located.protection(Some(answer)))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use gitpoulpi_hosting::{HttpTransport, RecordedTransport};
    use gitpoulpi_secrets::{InMemoryTokens, TokenStore};

    use super::*;

    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    fn env(transport: Arc<RecordedTransport>) -> (tempfile::TempDir, HostingEnv) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(InMemoryTokens::default());
        store.store("integration:github", "tok").unwrap();
        gitpoulpi_config::set_list_in_file(
            &dir.path().join("config.toml"),
            "integrations",
            &[gitpoulpi_config::Integration::new(
                "github",
                gitpoulpi_config::IntegrationKind::Github,
                "https://github.com",
            )],
        )
        .unwrap();
        let env = HostingEnv::new(
            dir.path().join("config.toml"),
            store,
            Box::new(move || Ok(transport.clone() as Arc<dyn HttpTransport>)),
        );
        (dir, env)
    }

    fn remote(url: &str) -> Remote {
        Remote {
            name: "origin".into(),
            fetch_url: url.into(),
            push_url: None,
        }
    }

    fn link() -> gitpoulpi_session::RepoState {
        gitpoulpi_session::RepoState {
            pulls_source: Some(gitpoulpi_session::IssuesSource {
                integration: "github".into(),
                project: "octo/thing".into(),
                filter: None,
            }),
            ..Default::default()
        }
    }

    const BRANCHES: &str = r#"[{"name":"main","protected":true}]"#;

    #[test]
    fn a_linked_repository_is_asked_and_the_answer_names_the_protected_branches() {
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/branches?protected=true", 200, BRANCHES)
                .on("/rulesets", 200, "[]"),
        );
        let (_dir, env) = env(transport.clone());
        let host = block_on(env.rewrite_host_protection(
            &[remote("git@github.com:octo/thing.git")],
            &link(),
            None,
            true,
        ))
        .unwrap();
        assert!(host.asked && host.covers("main") && !host.covers("dev"));
        assert_eq!(host.project, "octo/thing");
    }

    #[test]
    fn an_unlinked_repository_is_never_asked_but_still_gets_the_links() {
        let transport = Arc::new(RecordedTransport::new());
        let (_dir, env) = env(transport.clone());
        let host = block_on(env.rewrite_host_protection(
            &[remote("git@github.com:octo/thing.git")],
            &Default::default(),
            None,
            true,
        ))
        .unwrap();
        assert!(!host.asked && host.patterns.is_empty());
        assert!(host.settings_urls()[0].ends_with("/octo/thing/settings/branches"));
        assert!(
            transport.requests().is_empty(),
            "nothing was fetched for an unlinked repository"
        );
    }

    #[test]
    fn a_token_without_the_scope_is_unknown_and_the_rewrite_goes_on() {
        let transport = Arc::new(
            RecordedTransport::new()
                .on("/branches?protected=true", 403, r#"{"message":"nope"}"#)
                .on("/rulesets", 403, r#"{"message":"nope"}"#),
        );
        let (_dir, env) = env(transport);
        let host = block_on(env.rewrite_host_protection(
            &[remote("https://github.com/octo/thing.git")],
            &link(),
            None,
            true,
        ))
        .unwrap();
        assert!(host.asked && host.patterns.is_empty());
        assert!(host.unknown.unwrap().contains("lacks the scope"));
    }
}
