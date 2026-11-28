//! The status bar's "loaded / total": the count state machine lives in
//! `gitpoulpi_app::graph::total` (when to ask, what to show). This is the
//! thin adapter that gives it the daemon link's [`Reply`] and polls it.

use gitpoulpi_config::CommitCountMethod;

use crate::daemon_link::Reply;

use gitpoulpi_app::graph::{CommitTotal as Machine, TotalShown};

/// The machine over the link's reply type.
#[derive(Default)]
pub(crate) struct CommitTotal(Machine<Reply<u64>>);

impl CommitTotal {
    /// Polls the answer and, when the refs moved, starts the next count
    /// (`start` runs at most once per call). See the app crate's docs.
    pub(crate) fn tick(
        &mut self,
        method: CommitCountMethod,
        key: u64,
        exhausted: bool,
        start: impl FnOnce() -> Reply<u64>,
    ) {
        self.0.tick(method, key, exhausted, start, |reply| {
            reply.take().map(Result::ok)
        });
    }

    pub(crate) fn forget(&mut self) {
        self.0.forget();
    }

    pub(crate) fn shown(&self, loaded: usize, exhausted: bool) -> TotalShown {
        self.0.shown(loaded, exhausted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_config::CommitCountMethod as M;

    #[test]
    fn the_adapter_polls_the_link_reply_and_maps_a_failure() {
        let mut total = CommitTotal::default();
        assert_eq!(total.shown(500, false), TotalShown::Counting);
        total.tick(M::SystemGit, 7, false, || Reply::ready(Ok(2500)));
        total.tick(M::SystemGit, 7, false, || unreachable!("in flight"));
        assert_eq!(total.shown(500, false), TotalShown::Of(2500));

        let mut failing = CommitTotal::default();
        failing.tick(M::SystemGit, 1, false, || {
            Reply::ready(Err(gitpoulpi_core::Error::Backend("x".into())))
        });
        failing.tick(M::SystemGit, 1, false, || unreachable!("no retry"));
        assert_eq!(failing.shown(10, false), TotalShown::Counting);
    }
}
