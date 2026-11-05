//! Lazy (incremental) loading of the commit graph: when to ask for the
//! next page of history, what to ask for, and what to do with the answer.
//!
//! Everything decidable without a terminal is decided in [`LazyLog`] and
//! unit-tested against it; the Log's share of the feature is a few call
//! sites.
//!
//! # The shape
//!
//! The daemon's Log request takes a `skip`, so history is pageable: page `n`
//! is `skip = n * page_size`. The client holds one flat, contiguous
//! `Vec<CommitNode>` (the initial page plus every page appended under it)
//! and the graph is laid out over the whole of it.
//!
//! # Why the lanes survive a page boundary
//!
//! [`build_graph_grid`](super::layout::build_graph_grid) is a **fold**: it
//! walks the node list once, carrying lane state forward, and row `i` is a
//! function of `nodes[0..=i]` alone. That makes it **prefix-stable**:
//! appending rows cannot change a cell of the rows already produced, so a
//! commit in the blue lane stays in the blue lane when page 2 arrives.
//!
//! This is pinned by
//! `appending_a_page_does_not_disturb_the_rows_already_laid_out` in
//! `tests.rs`, which would fail if the fold were replaced by a global
//! two-pass lane allocator (that would renumber the whole graph on every
//! page). It is why this module can hold one flat list and re-lay it out
//! instead of carrying lane state across the boundary.
//!
//! # Never blocking the input loop
//!
//! `docs/developer/where-we-are.md`: never block the render/input loop on a
//! git call. A page is requested fire-and-forget on `daemon_link`'s thread
//! and arrives on a channel the loop polls; [`LazyLog`] holds the in-flight
//! bookkeeping so exactly one request is outstanding, and
//! [`is_loading`](LazyLog::is_loading) lets the Log say so while it waits.
//!
//! # Knowing when to stop
//!
//! Two independent reasons, and both must latch or the loader spins forever
//! at the bottom of the list:
//!
//! - **history ended.** A page shorter than asked for holds the root
//!   commit: [`exhausted`](LazyLog::is_exhausted) latches and nothing asks
//!   again.
//! - **the ceiling was reached.** `general.max_commits_in_graph` bounds how
//!   much the client will ever hold. The last page before it is *shortened*
//!   to land exactly on it, which is why [`next_page`](LazyLog::next_page)
//!   returns a `limit` and not just a `skip`.

use std::collections::HashSet;

use gitpoulpi_core::domain::{CommitNode, Oid};

use super::LogLoadSettings;

/// How many rows below the view the fetch is triggered at: three
/// screenfuls, and never fewer than 200 rows, so wheel/scrollbar-drag
/// users rarely reach the end while a page is still in flight.
pub(crate) fn lookahead_rows(visible_rows: usize) -> usize {
    (visible_rows.max(1) * 3).max(200)
}

/// One page to fetch: hand it straight to
/// `DaemonLink::request_log_page`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LogPageRequest {
    /// How many commits of the walk to drop, i.e. how many are already held.
    /// `LogOptions::skip`.
    pub skip: usize,
    /// How many to return. `LogOptions::limit`.
    pub limit: usize,
}

/// The lazy loader's whole state. One per Log.
#[derive(Debug, Clone)]
pub(crate) struct LazyLog {
    settings: LogLoadSettings,
    /// Commits currently held by the client, i.e. the next page's `skip`.
    loaded: usize,
    /// The `skip` of the request currently in flight, if any. At most one is
    /// ever outstanding: a second would duplicate the first's range or open
    /// a hole between them.
    in_flight: Option<usize>,
    /// Latched once history has been walked to its root.
    exhausted: bool,
}

impl LazyLog {
    pub(crate) fn new(settings: LogLoadSettings) -> LazyLog {
        LazyLog {
            settings,
            loaded: 0,
            in_flight: None,
            exhausted: false,
        }
    }

    /// How deep the *full* refresh should read. It grows as pages are
    /// appended, so the 2-second poll (and any daemon-pushed refresh) returns
    /// everything the client shows in one contiguous walk instead of
    /// truncating back to the initial page under a user who scrolled into
    /// page three.
    #[cfg(test)]
    pub(crate) fn refresh_depth(&self) -> usize {
        // Deliberately *not* clamped to `max_commits`: `loaded` never
        // exceeds the ceiling (`next_page` shortens the last page to land on
        // it, and `GeneralConfig::max_commits_in_graph_effective` refuses a
        // ceiling below the initial load), so a `min` here would be
        // unreachable code that looks like a safeguard.
        self.loaded.max(self.settings.initial_commits)
    }

    /// Adopts settings changed in Preferences **without discarding what is
    /// loaded**.
    ///
    /// `exhausted` is cleared: it latched against the old numbers, and
    /// raising the ceiling (or enabling lazy loading) must make the loader
    /// willing to ask again. `in_flight` survives: dropping it would let a
    /// second request for the same range go out and open a hole.
    pub(crate) fn apply_settings(&mut self, settings: LogLoadSettings) {
        self.settings = settings;
        self.exhausted = false;
    }

    /// Forgets everything about what was held: the walk changed under the
    /// loader (the hidden-refs filter moved), so nothing held, in flight or
    /// latched as "the end" describes it any more. The next refresh that
    /// lands starts it again.
    pub(crate) fn restart(&mut self) {
        self.loaded = 0;
        self.in_flight = None;
        self.exhausted = false;
    }

    pub(crate) fn is_loading(&self) -> bool {
        self.in_flight.is_some()
    }

    /// Whether history has been walked to its root. The bottom of history
    /// is deliberately silent in the Log, but the status bar's total reads
    /// it: fully loaded means loaded == total, no count needed.
    pub(crate) fn is_exhausted(&self) -> bool {
        self.exhausted
    }

    /// Whether the configured ceiling has been reached. Unlike
    /// [`is_exhausted`](LazyLog::is_exhausted) this one *is* surfaced: the
    /// user has to be able to see that more history exists but the client
    /// declines to hold it, or the graph just looks truncated.
    pub(crate) fn is_at_ceiling(&self) -> bool {
        matches!(self.settings.max_commits, Some(max) if self.loaded >= max)
    }

    pub(crate) fn loaded(&self) -> usize {
        self.loaded
    }

    /// A full refresh landed, carrying `loaded` commits.
    ///
    /// Short of what was asked for means all of history fits in it, so there
    /// is nothing left to page. This is also how `exhausted` *un*-latches: a
    /// repository that grows past its initial page (a fetch) pages again.
    #[cfg(test)]
    pub(crate) fn on_refresh(&mut self, loaded: usize) {
        let asked = self.refresh_depth();
        self.on_refresh_asked(loaded, asked);
    }

    /// [`on_refresh`](LazyLog::on_refresh) for a refresh that was asked for
    /// `asked` commits. That is the depth at the time of the *request*: a
    /// page can land while the refresh is on its way, and judging the answer
    /// against the deeper depth of its arrival read a full answer as the end
    /// of history (paging stopped, the status bar lost its total, until the
    /// next refresh).
    pub(crate) fn on_refresh_asked(&mut self, loaded: usize, asked: usize) {
        self.loaded = loaded;
        self.exhausted = loaded < asked;
    }

    /// Should the next page be fetched?
    ///
    /// `rows_below_view` is how many Log rows are still below the bottom of
    /// the window, and `visible_rows` is how many the window shows.
    ///
    /// The trigger is one screenful of lookahead, which scales with the
    /// terminal instead of being a magic number (the next page is fetched on
    /// reaching the earliest commit).
    ///
    /// Counted in *rows below the view*, not absolute indices, so it is
    /// right while the Solo filter is on: the Log then shows fewer rows than
    /// the client holds commits, and a comparison against `loaded` would
    /// never fire.
    ///
    /// Returns `None` (and is cheap enough to call every frame) when lazy
    /// loading is off, a request is in flight, history is exhausted, the
    /// ceiling has been reached, or nothing is loaded yet.
    pub(crate) fn next_page(
        &self,
        rows_below_view: usize,
        visible_rows: usize,
    ) -> Option<LogPageRequest> {
        if !self.settings.lazy_load
            || self.exhausted
            || self.in_flight.is_some()
            || self.loaded == 0
        {
            return None;
        }
        if rows_below_view > lookahead_rows(visible_rows) {
            return None;
        }
        // Shortened to land *exactly* on the ceiling: asking for a full page
        // and discarding the tail would make the daemon walk commits nobody
        // sees. A page is exactly the configured size: it once grew with
        // what was held (1000 -> 2000 -> 4000), which the owner read as a bug
        // since "commits per lazy page" promised a fixed step.
        let page = self.settings.page_commits;
        let limit = match self.settings.max_commits {
            Some(max) => page.min(max.saturating_sub(self.loaded)),
            None => page,
        };
        if limit == 0 {
            return None;
        }
        Some(LogPageRequest {
            skip: self.loaded,
            limit,
        })
    }

    /// Records that `request` was submitted. Split from
    /// [`next_page`](LazyLog::next_page) so the decision stays a pure query.
    pub(crate) fn mark_requested(&mut self, request: LogPageRequest) {
        self.in_flight = Some(request.skip);
    }

    /// A page came back. Appends whatever of it is genuinely new onto
    /// `nodes` and reports whether anything was added.
    ///
    /// Two guards, because a refresh and a page fetch race over a repository
    /// that can change between them:
    ///
    /// - **`skip` must still match what is loaded.** A refresh that landed
    ///   while this page was in flight may have changed the depth, which
    ///   moves every offset; the page is dropped and re-asked at the right
    ///   offset next frame.
    /// - **No commit may enter the list twice.** A commit pushed between the
    ///   refresh and the page shifts the walk by one, so a page can
    ///   re-deliver a held commit. The layout assumes each commit appears
    ///   once, and a repeat corrupts lane bookkeeping for every row below.
    pub(crate) fn on_page(
        &mut self,
        request: LogPageRequest,
        page: Vec<CommitNode>,
        nodes: &mut Vec<CommitNode>,
    ) -> bool {
        if self.in_flight == Some(request.skip) {
            self.in_flight = None;
        }
        if request.skip != nodes.len() {
            return false;
        }
        // A short page (the root commit is in it) ends the walk. Latched from
        // the *requested* size, before de-duplication, so a full page that
        // was entirely duplicate does not read as the end.
        if page.len() < request.limit {
            self.exhausted = true;
        }
        let held: HashSet<Oid> = nodes.iter().map(|n| n.id).collect();
        let before = nodes.len();
        nodes.extend(page.into_iter().filter(|n| !held.contains(&n.id)));
        self.loaded = nodes.len();
        nodes.len() > before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(byte: u8) -> CommitNode {
        CommitNode {
            id: Oid([byte; 20]),
            parents: Vec::new(),
            summary: String::new(),
            message: String::new(),
            author_name: String::new(),
            author_email: String::new(),
            timestamp: 0,
            refs: Vec::new(),
        }
    }

    fn settings(initial: usize, page: usize) -> LogLoadSettings {
        LogLoadSettings {
            initial_commits: initial,
            lazy_load: true,
            page_commits: page,
            max_commits: None,
        }
    }

    fn loaded(lazy: &mut LazyLog, n: usize) -> Vec<CommitNode> {
        lazy.on_refresh(n);
        (0..n).map(|i| node(i as u8)).collect()
    }

    #[test]
    fn a_restart_forgets_what_was_held_and_asks_for_nothing_until_a_refresh() {
        let mut lazy = LazyLog::new(settings(10, 5));
        loaded(&mut lazy, 10);
        lazy.mark_requested(LogPageRequest { skip: 10, limit: 5 });
        let mut nodes: Vec<CommitNode> = (0..3u8).map(node).collect();
        // The end had been reached once.
        lazy.on_page(LogPageRequest { skip: 3, limit: 5 }, vec![], &mut nodes);
        assert!(lazy.is_exhausted());

        lazy.restart();
        assert_eq!(lazy.loaded(), 0);
        assert!(
            !lazy.is_loading(),
            "the in-flight page belongs to the old walk"
        );
        assert!(!lazy.is_exhausted());
        assert_eq!(lazy.next_page(0, 20), None, "nothing before the refresh");

        lazy.on_refresh(10);
        assert_eq!(
            lazy.next_page(0, 20),
            Some(LogPageRequest { skip: 10, limit: 5 })
        );
    }

    #[test]
    fn nothing_is_requested_before_the_first_refresh_lands() {
        let lazy = LazyLog::new(settings(10, 5));
        assert_eq!(lazy.next_page(0, 20), None);
    }

    #[test]
    fn a_page_is_requested_once_the_earliest_commit_comes_within_the_lookahead() {
        let mut lazy = LazyLog::new(settings(1000, 25));
        loaded(&mut lazy, 1000);

        // 20 rows visible: lookahead is max(3 * 20, 200) = 200 rows.
        assert_eq!(lazy.next_page(800, 20), None);
        assert_eq!(lazy.next_page(201, 20), None);
        assert_eq!(
            lazy.next_page(200, 20),
            Some(LogPageRequest {
                skip: 1000,
                limit: 25
            })
        );
        // And at the very bottom.
        assert!(lazy.next_page(0, 20).is_some());
        // A tall window scales the lookahead past the 200-row floor.
        assert_eq!(lazy.next_page(301, 100), None);
        assert!(lazy.next_page(300, 100).is_some());
    }

    /// Red against the old auto-growing page (limit = max(page, loaded)):
    /// every page must add exactly the configured count, however much is held.
    #[test]
    fn every_page_adds_exactly_the_configured_count() {
        let mut lazy = LazyLog::new(settings(5000, 1000));
        loaded(&mut lazy, 5000);
        assert_eq!(lazy.next_page(0, 20).unwrap().limit, 1000);
        let mut lazy = LazyLog::new(settings(50_000, 1000));
        loaded(&mut lazy, 50_000);
        assert_eq!(lazy.next_page(0, 20).unwrap().limit, 1000);
    }

    #[test]
    fn successive_pages_grow_by_exactly_n_then_stop_at_the_root() {
        // 23 commits of history, 5 initial, pages of 5: 5,10,15,20,23.
        let history: Vec<CommitNode> = (0..23).map(node).collect();
        let mut lazy = LazyLog::new(settings(5, 5));
        let mut nodes: Vec<CommitNode> = history[..5].to_vec();
        lazy.on_refresh(5);
        let mut sizes = vec![nodes.len()];
        while let Some(request) = lazy.next_page(0, 10) {
            lazy.mark_requested(request);
            let end = (request.skip + request.limit).min(history.len());
            let page = history[request.skip..end].to_vec();
            lazy.on_page(request, page, &mut nodes);
            sizes.push(nodes.len());
        }
        assert_eq!(sizes, [5, 10, 15, 20, 23]);
        let ids: Vec<Oid> = nodes.iter().map(|n| n.id).collect();
        let want: Vec<Oid> = history.iter().map(|n| n.id).collect();
        assert_eq!(ids, want, "no gap, no duplicate");
    }

    /// The Solo filter shows fewer rows than the client holds commits, so a
    /// trigger phrased as "the selected index is near `loaded`" would never
    /// fire while it is on. Phrased as "rows still below the view", it does.
    #[test]
    fn the_trigger_fires_even_when_fewer_rows_are_shown_than_commits_held() {
        let mut lazy = LazyLog::new(settings(100, 25));
        loaded(&mut lazy, 100);
        // Solo left 6 displayed rows out of 100 loaded commits, and the
        // window shows 20 -- so every row is on screen.
        assert!(lazy.next_page(0, 20).is_some());
    }

    #[test]
    fn only_one_request_is_ever_in_flight() {
        let mut lazy = LazyLog::new(settings(100, 25));
        loaded(&mut lazy, 100);
        let request = lazy.next_page(0, 20).unwrap();
        lazy.mark_requested(request);
        assert!(lazy.is_loading());
        assert_eq!(
            lazy.next_page(0, 20),
            None,
            "a second request would either duplicate the first's range or leave a hole"
        );
    }

    #[test]
    fn an_arriving_page_is_appended_and_reopens_the_trigger() {
        let mut lazy = LazyLog::new(settings(4, 2));
        let mut nodes = loaded(&mut lazy, 4);
        let request = lazy.next_page(3, 10).unwrap();
        lazy.mark_requested(request);

        assert!(lazy.on_page(
            request,
            vec![node(4), node(5), node(6), node(7)],
            &mut nodes
        ));
        assert_eq!(nodes.len(), 8);
        assert_eq!(lazy.loaded(), 8);
        assert!(!lazy.is_loading());
        assert!(!lazy.is_exhausted());
        assert_eq!(
            lazy.next_page(5, 10),
            Some(LogPageRequest { skip: 8, limit: 2 })
        );
    }

    #[test]
    fn a_short_page_ends_history_and_nothing_asks_again() {
        let mut lazy = LazyLog::new(settings(4, 2));
        let mut nodes = loaded(&mut lazy, 4);
        let request = lazy.next_page(3, 10).unwrap();
        lazy.mark_requested(request);

        lazy.on_page(request, vec![node(4)], &mut nodes);
        assert!(lazy.is_exhausted());
        assert!(!lazy.is_loading());
        assert_eq!(
            lazy.next_page(4, 10),
            None,
            "the root commit must not leave a permanent loading state"
        );

        // And an empty page is the same story, from the other side.
        let mut lazy = LazyLog::new(settings(4, 2));
        let mut nodes = loaded(&mut lazy, 4);
        let request = lazy.next_page(3, 10).unwrap();
        lazy.mark_requested(request);
        assert!(!lazy.on_page(request, Vec::new(), &mut nodes));
        assert!(lazy.is_exhausted());
        assert_eq!(nodes.len(), 4);
    }

    #[test]
    fn a_refresh_shorter_than_it_asked_for_means_history_already_fits() {
        let mut lazy = LazyLog::new(settings(100, 25));
        loaded(&mut lazy, 12);
        assert!(lazy.is_exhausted());
        assert_eq!(lazy.next_page(11, 20), None);
    }

    #[test]
    fn a_repository_that_grows_past_its_initial_page_starts_paging_again() {
        let mut lazy = LazyLog::new(settings(4, 2));
        loaded(&mut lazy, 3);
        assert!(lazy.is_exhausted());
        // A fetch brought history down: the refresh now fills its page.
        let _ = loaded(&mut lazy, 4);
        assert!(!lazy.is_exhausted());
        assert!(lazy.next_page(3, 10).is_some());
    }

    #[test]
    fn a_page_whose_offset_no_longer_matches_is_dropped_not_stitched_in() {
        let mut lazy = LazyLog::new(settings(4, 2));
        let mut nodes = loaded(&mut lazy, 4);
        let request = lazy.next_page(3, 10).unwrap();
        lazy.mark_requested(request);

        // A refresh landed first and brought a fifth commit, so `skip: 4`
        // would now re-deliver a commit already held *and* leave the list
        // one short -- drop it.
        nodes.push(node(4));
        lazy.on_refresh(5);
        assert!(!lazy.on_page(request, vec![node(9), node(10)], &mut nodes));
        assert_eq!(nodes.len(), 5);
        assert_eq!(
            lazy.next_page(4, 10),
            Some(LogPageRequest { skip: 5, limit: 2 }),
            "the dropped page must be re-asked for at the corrected offset"
        );
    }

    #[test]
    fn a_commit_never_enters_the_list_twice() {
        let mut lazy = LazyLog::new(settings(4, 4));
        let mut nodes = loaded(&mut lazy, 4);
        let request = lazy.next_page(3, 10).unwrap();
        lazy.mark_requested(request);

        // The walk shifted by one under the page fetch, so the page
        // re-delivers `node(3)`. Appending it would give the layout a
        // commit twice and corrupt every lane below it.
        lazy.on_page(
            request,
            vec![node(3), node(4), node(5), node(6)],
            &mut nodes,
        );
        let ids: Vec<Oid> = nodes.iter().map(|n| n.id).collect();
        let unique: HashSet<Oid> = ids.iter().copied().collect();
        assert_eq!(ids.len(), unique.len(), "duplicate commit entered the log");
        assert_eq!(nodes.len(), 7);
    }

    #[test]
    fn lazy_loading_off_gives_the_initial_page_and_nothing_more() {
        let mut lazy = LazyLog::new(LogLoadSettings {
            lazy_load: false,
            ..settings(4, 2)
        });
        loaded(&mut lazy, 4);
        assert_eq!(lazy.next_page(3, 10), None);
        assert_eq!(lazy.next_page(3, 1), None);
        assert!(!lazy.is_loading());
    }

    /// The ceiling must stop the loader as hard as the root commit does, and
    /// land on the number exactly. Red without the `min` in `next_page`'s
    /// limit computation: the last page overshoots by up to a full page.
    #[test]
    fn the_ceiling_shortens_the_last_page_and_then_stops_the_loader() {
        let mut lazy = LazyLog::new(LogLoadSettings {
            max_commits: Some(9),
            ..settings(4, 4)
        });
        let mut nodes = loaded(&mut lazy, 4);

        let first = lazy.next_page(3, 10).unwrap();
        assert_eq!(first, LogPageRequest { skip: 4, limit: 4 });
        lazy.mark_requested(first);
        lazy.on_page(first, (4..8).map(|i| node(i as u8)).collect(), &mut nodes);
        assert!(!lazy.is_at_ceiling());

        // One commit short of the ceiling: the page is shortened to it
        // rather than asking for four and throwing three away.
        let last = lazy.next_page(3, 10).unwrap();
        assert_eq!(last, LogPageRequest { skip: 8, limit: 1 });
        lazy.mark_requested(last);
        lazy.on_page(last, vec![node(8)], &mut nodes);

        assert_eq!(nodes.len(), 9);
        assert!(lazy.is_at_ceiling());
        assert!(
            !lazy.is_exhausted(),
            "history did not end - the client chose to stop, and the UI must be able to say so"
        );
        assert_eq!(
            lazy.next_page(0, 10),
            None,
            "nothing may be asked for past the ceiling"
        );
    }

    /// Raising the ceiling in Preferences must make the loader ask again over
    /// the history already held. Red if `apply_settings` leaves `exhausted`
    /// alone: the graph stays frozen at the old ceiling.
    #[test]
    fn raising_the_ceiling_reopens_the_loader_without_dropping_what_is_held() {
        let mut lazy = LazyLog::new(LogLoadSettings {
            max_commits: Some(4),
            ..settings(4, 4)
        });
        let nodes = loaded(&mut lazy, 4);
        assert!(lazy.is_at_ceiling());
        assert_eq!(lazy.next_page(0, 10), None);

        lazy.apply_settings(LogLoadSettings {
            max_commits: Some(12),
            ..settings(4, 4)
        });
        assert_eq!(lazy.loaded(), nodes.len(), "the held history must survive");
        assert!(!lazy.is_at_ceiling());
        assert_eq!(
            lazy.next_page(0, 10),
            Some(LogPageRequest { skip: 4, limit: 4 })
        );
    }

    #[test]
    fn the_refresh_depth_grows_with_what_has_been_appended() {
        let mut lazy = LazyLog::new(settings(4, 2));
        let mut nodes = loaded(&mut lazy, 4);
        assert_eq!(lazy.refresh_depth(), 4);
        let request = lazy.next_page(3, 10).unwrap();
        lazy.mark_requested(request);
        lazy.on_page(request, vec![node(4), node(5)], &mut nodes);
        assert_eq!(
            lazy.refresh_depth(),
            6,
            "a refresh must not truncate the view back to the initial page"
        );
    }

    #[test]
    fn a_refresh_asked_before_a_page_landed_is_not_read_as_the_end_of_history() {
        let mut lazy = LazyLog::new(settings(4, 2));
        let mut nodes = loaded(&mut lazy, 4);
        // A refresh is sent at depth 4 ...
        let asked = lazy.refresh_depth();
        // ... and a page lands before its answer does.
        let request = lazy.next_page(3, 10).unwrap();
        lazy.mark_requested(request);
        lazy.on_page(request, vec![node(4), node(5)], &mut nodes);
        assert_eq!(lazy.refresh_depth(), 6);
        // The answer carries the 4 commits it was asked for: full, so not the end.
        lazy.on_refresh_asked(4, asked);
        assert!(
            !lazy.is_exhausted(),
            "4 of the 4 asked is not a short answer; the walk goes on"
        );
        assert!(lazy.next_page(0, 10).is_some(), "paging continues");
        // A really short answer still ends the walk.
        lazy.on_refresh_asked(3, 4);
        assert!(lazy.is_exhausted());
    }
}
