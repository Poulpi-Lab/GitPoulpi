//! Forgetting an integration: its trust record and everything cached from it.

use gitpoulpi_config::{ConfigPaths, Integration};

use super::store::TrustStore;
use super::{clear_pending, origin, TrustSubject};

/// What [`forget`] removed.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ForgetReport {
    pub trust_record: bool,
    pub list_cache_files: usize,
    pub account_cache: bool,
    pub review_drafts: usize,
}

/// Removes `integration`'s trust record and the caches filled from its host
/// (the Clone dialog's saved repository lists, its Radar account answer, the
/// remote review drafts of its host). Called by logout and remove, so a later
/// connection to the same id starts from nothing.
///
/// Best effort per part: a file that cannot be read or written is left alone,
/// the rest still goes.
pub fn forget(paths: &ConfigPaths, integration: &Integration) -> ForgetReport {
    let mut report = ForgetReport::default();
    let id = integration.id.as_str();

    let file = TrustStore::file_in(paths);
    if let Ok(mut store) = TrustStore::load(&file) {
        let before = store.integration.len();
        store.integration.retain(|r| r.id != id);
        if store.integration.len() != before {
            report.trust_record = store.save(&file).is_ok();
        }
    }
    clear_pending(&TrustSubject::Integration(id.to_string()));

    #[cfg(feature = "hosting")]
    {
        report.list_cache_files =
            crate::list_cache::remove_integration(&paths.cache_dir().join("clone-lists"), id);
        let path = crate::account::cache_path(paths.config_dir());
        let mut cache = crate::account::AccountCache::load(&path);
        if cache.entries.remove(id).is_some() {
            report.account_cache = cache.save(&path).is_ok();
        }
    }

    if let Some(host) = origin::host_of(&integration.url) {
        report.review_drafts = forget_review_drafts(paths, &host);
    }
    report
}

/// Drops the remote-review drafts and AI review defaults keyed by `host/`.
fn forget_review_drafts(paths: &ConfigPaths, host: &str) -> usize {
    let file = paths.config_dir().join("remote-reviews.toml");
    let Ok(raw) = std::fs::read_to_string(&file) else {
        return 0;
    };
    let Ok(mut table) = toml::from_str::<toml::Table>(&raw) else {
        return 0;
    };
    let prefix = format!("{}/", host.to_ascii_lowercase());
    let mut removed = 0;
    for section in ["drafts", "ai_review"] {
        if let Some(toml::Value::Table(rows)) = table.get_mut(section) {
            let doomed: Vec<String> = rows
                .keys()
                .filter(|k| k.to_ascii_lowercase().starts_with(&prefix))
                .cloned()
                .collect();
            removed += doomed.len();
            for key in doomed {
                rows.remove(&key);
            }
        }
    }
    if removed > 0 {
        if let Ok(text) = toml::to_string_pretty(&table) {
            let _ = gitpoulpi_config::secure_fs::write_private(&file, text.as_bytes());
        }
    }
    removed
}
