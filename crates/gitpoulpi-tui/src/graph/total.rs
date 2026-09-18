//! The status bar's "loaded / total": how many commits the whole graph walk
//! holds, learned without loading them.
//!
//! # The shape
//!
//! The total is the daemon's `CountCommits` read: the same refs as the graph
//! (`all_refs`), counted without materialising a commit, cached daemon-side
//! until a ref moves. This side only decides *when to ask* and *what to
//! show*, and never waits: the request goes down the shared raw queue and
//! the answer lands in a [`Reply`] polled each frame.
//!
//! # When it asks
//!
//! - **Never when history is fully loaded** ([`LazyLog::is_exhausted`]): a
//!   short last page means loaded == total, known for free.
//! - **Once per ref state.** The key is a hash of every ref's kind and
//!   target, so it changes exactly when a fetch, commit, branch move or tag
//!   changes what the walk reaches, and not on a plain 2 s refresh.
//! - **At most one in flight.** A key change while one runs waits for it
//!   and then asks again; the older answer only stands in meanwhile.
//!
//! The old total stays on screen while a new one is counted, unless it
//! fell behind what is already loaded (then the placeholder shows, so the
//! bar can never read `5001 / 5000`).

use std::hash::{Hash, Hasher};

use gitpoulpi_config::CommitCountMethod;
use gitpoulpi_core::domain::Ref;

use crate::daemon_link::Reply;

/// What the status bar says about the total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TotalShown {
    /// Everything is loaded (or the count is not worth showing): just the
    /// loaded count, as before.
    Whole,
    /// More history exists and its size is not known yet: `loaded / ...`.
    Counting,
    /// `loaded / total`.
    Of(u64),
}

/// Hash of the ref state the walk depends on.
pub(crate) fn refs_key(refs: &[Ref]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for r in refs {
        // Not `Ref`'s other fields (visibility, upstream, sync): hiding a
        // branch or a tag sync does not change which commits exist.
        match &r.kind {
            gitpoulpi_core::domain::RefKind::LocalBranch { name } => (0u8, name).hash(&mut hasher),
            gitpoulpi_core::domain::RefKind::RemoteBranch { remote, name } => {
                (1u8, remote, name).hash(&mut hasher);
            }
            gitpoulpi_core::domain::RefKind::Tag { name } => (2u8, name).hash(&mut hasher),
        }
        r.target.hash(&mut hasher);
    }
    hasher.finish()
}

#[derive(Default)]
pub(crate) struct CommitTotal {
    /// The ref state `known` was counted for.
    known: Option<(u64, u64)>,
    /// The request in flight and the key it was sent for.
    pending: Option<(u64, Reply<u64>)>,
    /// A count that failed is not retried until the refs change.
    failed_key: Option<u64>,
    /// The `log.commit_count` the state above was counted under.
    method: Option<CommitCountMethod>,
}

impl CommitTotal {
    /// Polls the answer and, when the refs moved, starts the next count.
    /// `start` sends the request; it runs at most once per call.
    ///
    /// `method` is `log.commit_count`, read live: a change forgets the total
    /// and any answer still in flight (it was counted another way) and
    /// recounts once; `Off` never asks and shows the plain loaded count.
    pub(crate) fn tick(
        &mut self,
        method: CommitCountMethod,
        key: u64,
        exhausted: bool,
        start: impl FnOnce() -> Reply<u64>,
    ) {
        if self.method != Some(method) {
            self.method = Some(method);
            self.known = None;
            self.pending = None;
            self.failed_key = None;
        }
        if method == CommitCountMethod::Off {
            return;
        }
        if let Some((asked, reply)) = &self.pending {
            if let Some(answer) = reply.take() {
                let asked = *asked;
                self.pending = None;
                match answer {
                    Ok(total) => self.known = Some((asked, total)),
                    Err(_) => self.failed_key = Some(asked),
                }
            }
        }
        if exhausted || self.pending.is_some() {
            return;
        }
        let up_to_date = matches!(self.known, Some((k, _)) if k == key);
        if !up_to_date && self.failed_key != Some(key) {
            self.pending = Some((key, start()));
        }
    }

    /// What to print for `loaded` commits held.
    pub(crate) fn shown(&self, loaded: usize, exhausted: bool) -> TotalShown {
        if exhausted || loaded == 0 || self.method == Some(CommitCountMethod::Off) {
            return TotalShown::Whole;
        }
        match self.known {
            Some((_, total)) if total <= loaded as u64 => TotalShown::Whole,
            Some((_, total)) => TotalShown::Of(total),
            None => TotalShown::Counting,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::CommitCountMethod as M;
    use std::cell::Cell;

    fn counting(total: u64) -> Reply<u64> {
        Reply::ready(Ok(total))
    }

    #[test]
    fn it_shows_a_placeholder_then_the_total_and_asks_once_per_key() {
        let asked = Cell::new(0);
        let mut total = CommitTotal::default();
        assert_eq!(total.shown(500, false), TotalShown::Counting);

        total.tick(M::SystemGit, 7, false, || {
            asked.set(asked.get() + 1);
            counting(2500)
        });
        // The answer is picked up on the next frame.
        total.tick(M::SystemGit, 7, false, || {
            unreachable!("one in flight, none needed")
        });
        assert_eq!(total.shown(500, false), TotalShown::Of(2500));
        for _ in 0..5 {
            total.tick(M::SystemGit, 7, false, || {
                unreachable!("same refs never recount")
            });
        }
        assert_eq!(asked.get(), 1);

        // A ref moved: recount, keep showing the old total meanwhile.
        total.tick(M::SystemGit, 8, false, || {
            asked.set(asked.get() + 1);
            Reply::pending()
        });
        assert_eq!(asked.get(), 2);
        assert_eq!(total.shown(500, false), TotalShown::Of(2500));
        // ... unless it fell behind what is loaded.
        assert_eq!(total.shown(2600, false), TotalShown::Whole);
    }

    #[test]
    fn off_never_asks_and_shows_the_plain_count() {
        let mut total = CommitTotal::default();
        for key in 1..4 {
            total.tick(M::Off, key, false, || {
                unreachable!("commit_count = off sends no CountCommits")
            });
        }
        assert_eq!(total.shown(500, false), TotalShown::Whole);
    }

    #[test]
    fn changing_the_method_forgets_the_total_and_recounts_once() {
        let asked = Cell::new(0);
        let mut total = CommitTotal::default();
        total.tick(M::SystemGit, 7, false, || {
            asked.set(asked.get() + 1);
            counting(2500)
        });
        total.tick(M::SystemGit, 7, false, || unreachable!("in flight"));
        assert_eq!(total.shown(500, false), TotalShown::Of(2500));

        // Off: the total is gone from the bar at once.
        total.tick(M::Off, 7, false, || unreachable!("off"));
        assert_eq!(total.shown(500, false), TotalShown::Whole);

        // Back on, with another method: asked again, exactly once.
        total.tick(M::Libgit2, 7, false, || {
            asked.set(asked.get() + 1);
            counting(2500)
        });
        total.tick(M::Libgit2, 7, false, || unreachable!("one recount only"));
        total.tick(M::Libgit2, 7, false, || unreachable!("one recount only"));
        assert_eq!(asked.get(), 2);
        assert_eq!(total.shown(500, false), TotalShown::Of(2500));
    }

    #[test]
    fn a_fully_loaded_history_needs_no_count() {
        let mut total = CommitTotal::default();
        total.tick(M::SystemGit, 1, true, || {
            unreachable!("exhausted: the total is the loaded count")
        });
        assert_eq!(total.shown(40, true), TotalShown::Whole);
    }

    #[test]
    fn a_stale_answer_is_dropped_and_the_new_key_asked_after_it() {
        let asked = Cell::new(0);
        let mut total = CommitTotal::default();
        let (reply, fill) = Reply::channel();
        total.tick(M::SystemGit, 1, false, || reply);
        // The refs move while the first count runs: nothing new is sent.
        total.tick(M::SystemGit, 2, false, || {
            unreachable!("one count in flight at a time")
        });
        fill(Ok(100));
        // The answer for key 1 lands, then key 2 is asked.
        total.tick(M::SystemGit, 2, false, || {
            asked.set(asked.get() + 1);
            Reply::pending()
        });
        assert_eq!(
            asked.get(),
            1,
            "the new key is asked once the old answer is in"
        );
    }

    #[test]
    fn a_failed_count_is_not_retried_until_the_refs_change() {
        let mut total = CommitTotal::default();
        total.tick(M::SystemGit, 1, false, || {
            Reply::ready(Err(gitpoulpi_core::Error::Backend("x".into())))
        });
        total.tick(M::SystemGit, 1, false, || {
            unreachable!("no retry on the same key")
        });
        total.tick(M::SystemGit, 1, false, || {
            unreachable!("no retry on the same key")
        });
        assert_eq!(total.shown(10, false), TotalShown::Counting);
        let asked = Cell::new(false);
        total.tick(M::SystemGit, 2, false, || {
            asked.set(true);
            Reply::pending()
        });
        assert!(asked.get());
    }
}
