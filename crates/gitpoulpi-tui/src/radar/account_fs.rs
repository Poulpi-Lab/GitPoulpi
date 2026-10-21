//! The Radar's account-wide plan, read from the disk: which integrations to
//! ask and which to offer, and the clones the machine knows about. Both run
//! on the worker, never on the thread that draws.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gitpoulpi_config::Integration;
use gitpoulpi_hosting::account::{self, AccountQuery};

use super::account::{AccountJob, AccountPlan, CloneIndex, Offer};
use super::scope;

/// The most clones read for the index: a home folder must not turn into
/// thousands of `.git/config` reads.
const MAX_CLONES: usize = 600;

/// The question an integration with its lists on is asked. Drafts are always
/// fetched: the Radar's own `Drafts` checkbox hides them, so toggling it
/// needs no new round.
pub(crate) fn query_of(integration: &Integration) -> AccountQuery {
    let radar = integration.radar_options();
    AccountQuery {
        review_requests: radar.review_requests,
        open_requests: radar.open_requests,
        assigned_issues: radar.assigned_issues,
        include_drafts: true,
        namespaces: radar.namespaces,
        limit: 0,
    }
}

/// Whether the Radar can list account-wide rows for `integration`.
fn askable(integration: &Integration) -> bool {
    integration.enabled
        && account::supports(integration.kind)
        && crate::build_features::kind_built(integration.kind)
}

/// Jobs for the integrations with a list on, offers for the other code hosts.
pub(crate) fn plan_of(integrations: Vec<Integration>, config_dir: Option<&Path>) -> AccountPlan {
    let labels = crate::pull_request::pr_tabs(&integrations);
    let label_of = |id: &str| {
        labels
            .iter()
            .find(|t| t.integration.id == id)
            .map(|t| t.label.clone())
            .unwrap_or_else(|| id.to_string())
    };
    let mut jobs = Vec::new();
    let mut offers = Vec::new();
    for integration in integrations.iter().filter(|i| askable(i)) {
        if integration.radar_options().any_list() {
            jobs.push(AccountJob {
                integration: integration.clone(),
                query: query_of(integration),
                label: label_of(&integration.id),
            });
        } else {
            offers.push(Offer {
                id: integration.id.clone(),
                label: label_of(&integration.id),
            });
        }
    }
    AccountPlan {
        all: integrations,
        jobs,
        offers,
        cache: config_dir.map(account::cache_path),
    }
}

/// Every clone this machine knows: the scope's repositories, the recent ones,
/// the workspaces' and what the folder scans found, indexed by remote.
pub(crate) fn known_clones(scope_repos: &[PathBuf], config_dir: Option<&Path>) -> CloneIndex {
    let mut candidates: Vec<PathBuf> = scope_repos.to_vec();
    if let Some(dir) = config_dir {
        candidates.extend(
            gitpoulpi_session::RecentRepos::load(&gitpoulpi_session::RecentRepos::file_in(dir))
                .repos
                .into_iter()
                .map(|r| r.path),
        );
        candidates.extend(
            gitpoulpi_session::Workspaces::load(&gitpoulpi_session::Workspaces::file_in(dir))
                .workspaces
                .into_iter()
                .flat_map(|w| w.repos),
        );
        for folder in [scope::CLONE_FOLDER, scope::BROWSE_FOLDER] {
            if let Some(found) =
                scope::folder_of(dir, folder).and_then(|root| scope::cached_repos(&root))
            {
                candidates.extend(found);
            }
        }
    }
    let mut seen = HashSet::new();
    let mut index = CloneIndex::default();
    for path in candidates
        .into_iter()
        .filter(|p| seen.insert(gitpoulpi_session::repo_path_key(p)))
        .take(MAX_CLONES)
    {
        let Some(config) = crate::repo_probe::git_config_path(&path) else {
            continue;
        };
        let Ok(raw) = std::fs::read_to_string(config) else {
            continue;
        };
        for (_, url) in crate::repo_probe::parse_remote_urls(&raw) {
            index.add_remote(&url, &path);
        }
    }
    index
}

/// Writes `integrations.radar.review_requests = true` for the profile `id`
/// in `<config_dir>/config.toml`, leaving the profile's other Radar options
/// as they are.
pub(crate) fn turn_on_review_requests(config_dir: &Path, id: &str) -> Result<(), String> {
    let file = config_dir.join("config.toml");
    let mut list = gitpoulpi_config::Config::load(&file)
        .map_err(|e| e.to_string())?
        .integrations;
    let integration = list
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| gitpoulpi_i18n::msg::radar::why_no_integration(id))?;
    let mut radar = integration.radar_options();
    radar.review_requests = true;
    integration.radar = Some(radar);
    gitpoulpi_config::set_list_in_file(&file, "integrations", &list).map_err(|e| e.to_string())
}

/// The profile `id` of `<config_dir>/config.toml`, if it is there.
pub(crate) fn integration_in(config_dir: &Path, id: &str) -> Option<Integration> {
    gitpoulpi_config::Config::load(&config_dir.join("config.toml"))
        .ok()?
        .integrations
        .into_iter()
        .find(|i| i.id == id)
}

/// Writes `integrations.radar.review_mode` of the profile `id` in
/// `<config_dir>/config.toml`, leaving its other Radar options as they are
/// (the "Remember" checkbox of the review choice).
pub(crate) fn set_review_mode(
    config_dir: &Path,
    id: &str,
    mode: gitpoulpi_config::RemoteReviewMode,
) -> Result<(), String> {
    let file = config_dir.join("config.toml");
    let mut list = gitpoulpi_config::Config::load(&file)
        .map_err(|e| e.to_string())?
        .integrations;
    let integration = list
        .iter_mut()
        .find(|i| i.id == id)
        .ok_or_else(|| gitpoulpi_i18n::msg::radar::why_no_integration(id))?;
    let mut radar = integration.radar_options();
    radar.review_mode = mode;
    integration.radar = Some(radar);
    gitpoulpi_config::set_list_in_file(&file, "integrations", &list).map_err(|e| e.to_string())
}
