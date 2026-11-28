//! The Radar worker's production wiring: reads each repository's own state,
//! its remotes and the configured providers from the disk. The worker itself
//! (`gitpoulpi_link::radar::worker`) only sees the [`Prepare`] trait.

use std::path::{Path, PathBuf};

use gitpoulpi_config::{ApiFamily, Integration};
use gitpoulpi_hosting::ResolveRemote;
use gitpoulpi_link::hosting_task::{cache_path, HostingRequest, IssueView, PrView};
use gitpoulpi_link::radar::account::{AccountPlan, CloneIndex};
use gitpoulpi_link::radar::worker::{lists_requests, PerRound, Prepare, Prepared};

/// Reads each repository's own state, remotes and the configured providers.
#[derive(Default)]
pub(crate) struct FsPrepare {
    /// The providers and the state directory: `config.toml` is read once for
    /// the round, not twice per repository.
    config: PerRound<(Vec<Integration>, String)>,
    /// Where the Radar's own files live (recents, workspaces, the account
    /// cache).
    config_dir: Option<PathBuf>,
}

impl FsPrepare {
    pub(crate) fn new(config_dir: Option<PathBuf>) -> Self {
        Self {
            config: PerRound::default(),
            config_dir,
        }
    }
}

impl Prepare for FsPrepare {
    fn begin_round(&self) {
        self.config.reset();
    }

    fn accounts(&self) -> AccountPlan {
        super::account_fs::plan_of(
            crate::pull_request::load_integrations(),
            self.config_dir.as_deref(),
        )
    }

    fn known_clones(&self, scope: &[PathBuf]) -> CloneIndex {
        super::account_fs::known_clones(scope, self.config_dir.as_deref())
    }

    fn prepare(&self, repo: &Path) -> Prepared {
        let state = gitpoulpi_session::RepoState::load(
            &gitpoulpi_link::repo_state_path::repo_state_path(repo),
        )
        .ok();
        let remotes = crate::repo_probe::git_config_path(repo)
            .and_then(|config| std::fs::read_to_string(config).ok())
            .map(|raw| crate::repo_probe::parse_remote_urls(&raw))
            .unwrap_or_default()
            .into_iter()
            .map(|(name, url)| ResolveRemote::new(name, url))
            .collect();
        // GitHub and GitLab (maintainer rule 6), plus the local Markdown issue
        // source, which is the repository's own.
        let config = self.config.get(|| {
            let integrations: Vec<Integration> = crate::pull_request::load_integrations()
                .into_iter()
                .filter(|i| lists_requests(i.kind) || i.kind.family() == ApiFamily::LocalMarkdown)
                .collect();
            let state_dir = gitpoulpi_config::Settings::resolve_ambient()
                .map(|s| s.config().session.per_repo_state_dir.clone())
                .unwrap_or_else(|_| gitpoulpi_config::DEFAULT_REPO_DATA_LOCATION.to_string());
            (integrations, state_dir)
        });
        let (integrations, state_dir) = (config.0.clone(), config.1.as_str());
        let issues_source = gitpoulpi_link::hosting_task::resolve_local_source(
            repo,
            &integrations,
            state.as_ref().and_then(|s| s.issues_source.clone()),
        );
        Prepared {
            request: HostingRequest {
                remotes,
                integrations,
                override_id: state.as_ref().and_then(|s| s.hosting_integration.clone()),
                head_branch: None,
                pr_view: PrView::Open,
                issue_view: IssueView::Open,
                issues_source,
                pulls_source: state.and_then(|s| s.pulls_source),
                repo_root: repo.to_path_buf(),
            },
            cache: Some(cache_path(repo, state_dir)),
        }
    }
}
