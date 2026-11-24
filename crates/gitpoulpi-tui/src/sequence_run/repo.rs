//! The repository's own sequences (`.gitpoulpi/sequences.toml`) in the
//! window: what the active repository's file holds, whether the user trusted
//! exactly that, and the notice asking when not.
//!
//! The file can come from other people, so it is read through
//! `gitpoulpi_session::inspect_repo_sequences`: the sequences and the hash
//! they were judged by come from one read. Only a trusted file reaches the
//! registry (palette rows, shortcuts, runs); an untrusted or changed one only
//! raises the notice, whose Review lists every step before anything is
//! trusted. The trust record is `RepoState::trusted_sequences`, written here
//! and nowhere an agent can reach.
//!
//! The window calls [`sync`] once a frame with its repository's root: it
//! looks at two file stamps and re-reads only when one moved, so a file that
//! changed under a running window asks again within a frame.

use super::registry::{self, with, Registry};
use gitpoulpi_config::sequences::SequenceDef;
use gitpoulpi_session::{inspect_repo_sequences, RepoSequences, RepoState, SequencesTrust};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

type Stamp = [Option<(SystemTime, u64)>; 2];

/// What the registry knows about the active repository's file.
pub(super) struct Entry {
    root: PathBuf,
    stamp: Stamp,
    seen: RepoSequences,
    /// The hash the user chose to ignore for this session.
    ignored: Option<String>,
}

fn stamp_of(path: &Path) -> Option<(SystemTime, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

fn state_file(root: &Path) -> PathBuf {
    // Unit tests never read the machine's config.
    #[cfg(test)]
    {
        RepoState::path_for(root, ".gitpoulpi")
    }
    #[cfg(not(test))]
    {
        crate::fold_state::state_path(root)
    }
}

fn stamps(root: &Path) -> Stamp {
    [
        stamp_of(&gitpoulpi_config::sequences::repo_file(root)),
        stamp_of(&state_file(root)),
    ]
}

fn read(root: &Path) -> RepoSequences {
    let state = RepoState::load(&state_file(root)).unwrap_or_default();
    inspect_repo_sequences(root, &state)
}

/// What the notice says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Notice {
    /// How many sequences the file defines.
    pub(crate) count: usize,
    /// Trusted once and changed since.
    pub(crate) changed: bool,
}

/// Once a frame: follows `root` and its file. Cheap (two `stat`s) unless
/// something moved.
pub(crate) fn sync(root: &Path) {
    let now = stamps(root);
    let moved = with(|reg| {
        reg.repo
            .as_ref()
            .is_none_or(|e| e.root != root || e.stamp != now)
    });
    if moved {
        load(root);
    }
}

/// Re-reads the file for `root` now and tells the registry (and, in a
/// running window, the keymap).
pub(crate) fn load(root: &Path) {
    let seen = read(root);
    let stamp = stamps(root);
    with(|reg| install(reg, root, stamp, seen));
    registry::rebuild();
    crate::redraw::poke();
}

/// Stores `seen` for `root`. An "ignore" holds only while the file stays
/// what it was when the user ignored it.
fn install(reg: &mut Registry, root: &Path, stamp: Stamp, seen: RepoSequences) {
    let ignored = reg
        .repo
        .take()
        .filter(|e| e.root == root)
        .and_then(|e| e.ignored)
        .filter(|h| *h == seen.hash);
    reg.repo = Some(Entry {
        root: root.to_path_buf(),
        stamp,
        seen,
        ignored,
    });
}

/// The repository sequences that may be offered: the file's, when trusted.
pub(crate) fn trusted_defs() -> Vec<SequenceDef> {
    with(|reg| {
        reg.repo
            .as_ref()
            .map(|e| e.seen.trusted().to_vec())
            .unwrap_or_default()
    })
}

/// The trusted sequences of `root` read from disk right now (what a run
/// starts from: what runs is what is trusted at that moment).
pub(crate) fn fresh_trusted(root: &Path) -> Vec<SequenceDef> {
    read(root).trusted().to_vec()
}

/// The question to ask, when there is one for the active repository.
pub(crate) fn notice() -> Option<Notice> {
    with(|reg| {
        let e = reg.repo.as_ref()?;
        if !e.seen.needs_answer() || e.ignored.as_deref() == Some(e.seen.hash.as_str()) {
            return None;
        }
        Some(Notice {
            count: e.seen.defs.len(),
            changed: e.seen.trust == SequencesTrust::Changed,
        })
    })
}

/// The file waits for an answer (ignored or not): what the palette commands
/// need.
pub(crate) fn pending() -> Option<Notice> {
    with(|reg| {
        let e = reg.repo.as_ref()?;
        e.seen.needs_answer().then(|| Notice {
            count: e.seen.defs.len(),
            changed: e.seen.trust == SequencesTrust::Changed,
        })
    })
}

/// The file's sequences, for the review (trusted or not).
pub(crate) fn reviewed() -> Vec<SequenceDef> {
    with(|reg| {
        reg.repo
            .as_ref()
            .map(|e| e.seen.defs.clone())
            .unwrap_or_default()
    })
}

/// The active repository's root, when the registry follows one.
pub(crate) fn root() -> Option<PathBuf> {
    with(|reg| reg.repo.as_ref().map(|e| e.root.clone()))
}

/// Hides the notice until the file changes (or the window restarts).
pub(crate) fn ignore() {
    with(|reg| {
        if let Some(e) = reg.repo.as_mut() {
            e.ignored = Some(e.seen.hash.clone());
        }
    });
    crate::redraw::poke();
}

/// Trusts the file as it is on disk now: the hash recorded is the one of the
/// sequences just read, so what was reviewed is what is trusted. Returns the
/// number of sequences trusted, or why it could not be saved.
pub(crate) fn trust() -> Result<usize, String> {
    let root = root().ok_or_else(|| "no repository".to_string())?;
    trust_at(&root)
}

fn trust_at(root: &Path) -> Result<usize, String> {
    let seen = read(root);
    if seen.defs.is_empty() {
        return Ok(0);
    }
    let path = state_file(root);
    let mut state = RepoState::load(&path).unwrap_or_default();
    state.trust_sequences(
        &seen.hash,
        seen.defs.iter().map(|d| d.id.clone()).collect(),
        gitpoulpi_session::now_seconds(),
    );
    state.save(&path).map_err(|e| e.to_string())?;
    load(root);
    Ok(seen.defs.len())
}

/// Whether the user's own edit of the file may count as trusted: it was
/// trusted, or there was nothing to trust, before they wrote. A file other
/// people wrote and nobody trusted stays untrusted whatever the user adds to
/// it.
pub(crate) fn trusted_before_edit(root: &Path) -> bool {
    matches!(
        read(root).trust,
        SequencesTrust::Trusted | SequencesTrust::None
    )
}

/// After the user wrote the file in Settings: trusts it again when it was
/// trusted before the edit.
pub(crate) fn retrust_after_edit(root: &Path, was_trusted: bool) {
    if was_trusted {
        let _ = trust_at(root);
    } else {
        load(root);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    fn write(root: &Path, text: &str) {
        let file = gitpoulpi_config::sequences::repo_file(root);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
    }

    const ONE: &str = "[[sequences]]\nid = \"ship\"\nname = \"Ship it\"\n[[sequences.steps]]\nrun = \"stage_all\"\n";

    /// A repository folder whose state file is inside it (the tests' config
    /// says nothing, so the default location applies).
    fn repo_dir() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn nothing_is_offered_until_trusted_and_the_notice_counts_them() {
        let dir = repo_dir();
        write(dir.path(), ONE);
        load(dir.path());
        assert!(trusted_defs().is_empty(), "never offered untrusted");
        assert_eq!(
            notice(),
            Some(Notice {
                count: 1,
                changed: false
            })
        );
        assert_eq!(trust(), Ok(1));
        assert_eq!(trusted_defs().len(), 1);
        assert_eq!(notice(), None);
    }

    #[test]
    fn only_a_trusted_file_reaches_the_registry_and_so_the_palette_and_the_keys() {
        let dir = repo_dir();
        write(dir.path(), ONE);
        load(dir.path());
        let config = gitpoulpi_config::Config::default();
        registry::refresh(&config, "x");
        assert_eq!(registry::slot_of("ship"), None, "untrusted: not offered");
        trust().unwrap();
        registry::refresh(&config, "x");
        let slot = registry::slot_of("ship").expect("trusted: offered");
        assert_eq!(registry::info(slot).unwrap().name, "Ship it");
        write(dir.path(), &ONE.replace("stage_all", "push"));
        load(dir.path());
        registry::refresh(&config, "x");
        assert_eq!(registry::slot_of("ship"), None, "changed: asks again");
    }

    #[test]
    fn a_change_asks_again_and_ignore_lasts_until_the_next_change() {
        let dir = repo_dir();
        write(dir.path(), ONE);
        load(dir.path());
        trust().unwrap();
        write(dir.path(), &ONE.replace("stage_all", "push"));
        load(dir.path());
        assert!(trusted_defs().is_empty());
        assert_eq!(notice().map(|n| n.changed), Some(true));
        ignore();
        assert_eq!(notice(), None, "ignored");
        write(dir.path(), &ONE.replace("stage_all", "fetch"));
        load(dir.path());
        assert!(notice().is_some(), "a newer change asks again");
    }

    #[test]
    fn an_own_edit_keeps_a_trusted_file_trusted_but_not_a_foreign_one() {
        let dir = repo_dir();
        write(dir.path(), ONE);
        load(dir.path());
        let was = trusted_before_edit(dir.path());
        assert!(!was, "foreign and untrusted");
        write(dir.path(), &ONE.replace("Ship it", "Ship"));
        retrust_after_edit(dir.path(), was);
        assert!(trusted_defs().is_empty());
        trust().unwrap();
        let was = trusted_before_edit(dir.path());
        assert!(was);
        write(dir.path(), &ONE.replace("stage_all", "push"));
        retrust_after_edit(dir.path(), was);
        assert_eq!(trusted_defs().len(), 1, "the user's own edit stays trusted");
    }
}
