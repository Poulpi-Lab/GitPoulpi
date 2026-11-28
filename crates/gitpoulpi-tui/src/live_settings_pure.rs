//! The live settings the daemon client reads, without a `Config` in hand:
//! plain atomics and locks, no terminal types. Re-exported by
//! `live_settings`, so every call site keeps its old path.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::RwLock;

/// `general.auto_prune`, published like the date patterns so `daemon_link`'s
/// `Fetch`/`FetchAll` handlers can read it without a `Config`. It governs
/// whether those fetches (background auto-fetch and the toolbar's Fetch
/// included) pass `--prune`; the diverged-push dialog's lease-check fetch
/// never prunes, on purpose. On by default (like the config); pruning is not
/// undoable, but the refs come back on the next fetch if the branch returns.
static AUTO_PRUNE: AtomicBool = AtomicBool::new(true);

/// Serializes the tests that touch the process-wide `AUTO_PRUNE`: the Settings
/// toggle test flips it, and the `Fetch` wire golden expects it off.
#[cfg(test)]
pub(crate) static AUTO_PRUNE_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub(crate) fn install_auto_prune(on: bool) {
    AUTO_PRUNE.store(on, Ordering::Relaxed);
}

/// `general.auto_prune_tags`, published like `AUTO_PRUNE`.
static AUTO_PRUNE_TAGS: AtomicBool = AtomicBool::new(false);

pub(crate) fn install_auto_prune_tags(on: bool) {
    AUTO_PRUNE_TAGS.store(on, Ordering::Relaxed);
}

/// What the Fetch / Fetch All paths send: both prune settings, read live.
pub(crate) fn fetch_options() -> gitpoulpi_core::domain::FetchOptions {
    gitpoulpi_core::domain::FetchOptions {
        prune: AUTO_PRUNE.load(Ordering::Relaxed),
        prune_tags: AUTO_PRUNE_TAGS.load(Ordering::Relaxed),
    }
}

/// `ui.image_view_max_mib`, published like `AUTO_PRUNE`.
static IMAGE_VIEW_MAX_MIB: AtomicU32 = AtomicU32::new(32);

pub(crate) fn install_image_view_max_mib(mib: u32) {
    IMAGE_VIEW_MAX_MIB.store(mib, Ordering::Relaxed);
}

/// The byte cap on an image read, from `ui.image_view_max_mib`.
pub(crate) fn image_view_max_bytes() -> u64 {
    u64::from(IMAGE_VIEW_MAX_MIB.load(Ordering::Relaxed)) * 1024 * 1024
}

/// `general.delete_orig_files_after_merge`, published like `AUTO_PRUNE`. Read
/// by `daemon_link::run_action`'s `Merge` arm after a successful merge.
static DELETE_ORIG_AFTER_MERGE: AtomicBool = AtomicBool::new(false);

pub(crate) fn install_delete_orig_after_merge(on: bool) {
    DELETE_ORIG_AFTER_MERGE.store(on, Ordering::Relaxed);
}

pub(crate) fn delete_orig_after_merge() -> bool {
    DELETE_ORIG_AFTER_MERGE.load(Ordering::Relaxed)
}

/// `general.active_profile`, resolved to the `CommitAuthor` a commit should
/// carry, or `None` when it names no profile (`Config::active_profile`'s
/// "falls back to `git config user.*`" rule). Read by
/// `daemon_link::run_action`'s `Commit` arm.
///
/// A profile with an empty name or email counts as no profile: half an
/// identity would make git refuse the commit (`fatal: empty ident`).
static ACTIVE_PROFILE_AUTHOR: RwLock<Option<gitpoulpi_core::domain::CommitAuthor>> =
    RwLock::new(None);

/// Publishes the author the resolved profile gives (`None` for no profile or
/// half an identity).
pub(crate) fn publish_author(profile: Option<gitpoulpi_config::Profile>) {
    let author = profile.and_then(|p| {
        if p.author_name.is_empty() || p.author_email.is_empty() {
            None
        } else {
            Some(gitpoulpi_core::domain::CommitAuthor {
                name: p.author_name,
                email: p.author_email,
            })
        }
    });
    if let Ok(mut slot) = ACTIVE_PROFILE_AUTHOR.write() {
        *slot = author;
    }
}

pub(crate) fn active_profile_author() -> Option<gitpoulpi_core::domain::CommitAuthor> {
    ACTIVE_PROFILE_AUTHOR.read().ok().and_then(|s| s.clone())
}
