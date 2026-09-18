//! `pull_request` for a build with no hosting provider (custom builds,
//! plan section 3.2): no Start / Create pull request flow. What stays is
//! what other modules read from it without being about pull requests:
//! the `[[integrations]]` table (it still parses) and the remote ->
//! profile match "open remote on web" uses.
#![allow(dead_code)]

use super::*;

use gitpoulpi_core::domain::{Ref, Remote};
use gitpoulpi_hosting::Integration;

/// The configured integrations, via the shared settings loader.
pub(crate) fn load_integrations() -> Vec<Integration> {
    gitpoulpi_config::Settings::resolve_ambient()
        .map(|settings| settings.config().integrations.clone())
        .unwrap_or_default()
}

/// A plain ref-menu context: there is no "Start a pull request" row.
pub(crate) fn menu_context_for_ref(
    refs: &gitpoulpi_core::Result<Vec<Ref>>,
    log: &gitpoulpi_core::Result<gitpoulpi_core::domain::CommitGraph>,
    section: SidebarSection,
    name: &str,
    _remotes: &[Remote],
) -> MenuContext {
    MenuContext::for_ref(refs, log, section, name)
}

/// Never filled in this build.
pub(crate) struct PullRequestSeed;

/// `host` of a `host[:port]` or `https://host[:port]` string, lowercased,
/// without `www.` and without the port (an instance's SSH and HTTPS
/// remotes differ in them).
fn bare_host(text: &str) -> String {
    let rest = text.split("://").last().unwrap_or(text);
    let host = rest.split('/').next().unwrap_or(rest);
    let host = host.rsplit('@').next().unwrap_or(host);
    let host = host.split(':').next().unwrap_or(host);
    host.to_lowercase().trim_start_matches("www.").to_string()
}

/// The enabled profile whose host is the remote's host (the provider
/// clients are not built here, so this is a plain host match).
pub(crate) fn integration_for_url<'a>(
    url: &str,
    integrations: &'a [Integration],
) -> Option<&'a Integration> {
    let (base, _) = gitpoulpi_hosting::remote_web_base(url)?;
    let host = bare_host(&base);
    integrations
        .iter()
        .filter(|i| i.enabled)
        .find(|i| i.host().is_some_and(|h| bare_host(&h) == host))
}

pub(crate) fn matching_integration<'a>(
    remote: &Remote,
    integrations: &'a [Integration],
) -> Option<&'a Integration> {
    integration_for_url(&remote.fetch_url, integrations)
}

pub(crate) fn start_pull_request_now(
    _branch: &str,
    _refs: &[Ref],
    _remotes: &[Remote],
    _seed: &mut Option<PullRequestSeed>,
) -> String {
    gitpoulpi_i18n::msg::hosting_ui::WHY_NOT_BUILT
        .get()
        .to_string()
}

pub(crate) fn create_pull_request_now(
    _refs: &[Ref],
    _remotes: &[Remote],
    _seed: &mut Option<PullRequestSeed>,
) -> String {
    gitpoulpi_i18n::msg::hosting_ui::WHY_NOT_BUILT
        .get()
        .to_string()
}
