//! Which commits the Log shows, and how it fetches them.
//!
//! Paging itself is the shared `gitpoulpi_app::graph::LazyLog` (page size, ceiling, lazy on/off from
//! `LogLoadSettings`); this file only fetches. The Log walks every ref (`--branches --tags --remotes`)
//! minus the hidden ones. On a detached HEAD that walk can miss HEAD itself: a clone checked out at a
//! tag or a commit has no local branch, and a repository whose only branch was deleted has no ref at
//! all. A refresh therefore also asks for the history of HEAD and puts the commits only HEAD reaches
//! in front (`LazyLog::set_front`), which keeps every child before its parents for the graph layout.
//! When no ref reaches any commit the history of HEAD is the walk and pages on its own.
//!
//! The fetching of one page is [`Backend::history_page`]; the rest of the daemon link is not touched.

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

use gitpoulpi_app::graph::LogPageRequest;
use gitpoulpi_core::domain::{CommitGraph, CommitNode, Oid, OperationKind, Ref, RefFilter};

use super::queries::LogSource;
use super::Backend;

type Fut<T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send + 'static>>;

/// What the pages of one Log walk continue: where it starts and which refs it leaves out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Walk {
    pub source: LogSource,
    pub filter: RefFilter,
}

/// A full refresh: the held list from the top.
pub struct Refresh {
    /// The detached-HEAD commits first (`front` of them), then the walk.
    pub nodes: Vec<CommitNode>,
    /// How many leading commits the paged walk does not contain.
    pub front: usize,
    /// How many walk commits were asked for (the `LazyLog` refresh depth at the time of the request).
    pub asked: usize,
    pub walk: Walk,
}

/// Pure part of the detached-HEAD handling: puts the commits only HEAD reaches in front of the ref walk.
pub fn merge_head_history(
    refs_walk: Vec<CommitNode>,
    head_walk: Vec<CommitNode>,
) -> (Vec<CommitNode>, usize) {
    let known: HashSet<Oid> = refs_walk.iter().map(|n| n.id).collect();
    let mut out: Vec<CommitNode> = head_walk
        .into_iter()
        .filter(|n| !known.contains(&n.id))
        .collect();
    let head_only = out.len();
    out.extend(refs_walk);
    (out, head_only)
}

/// True when no ref is the checked-out branch: HEAD is detached (or the repository has no commits).
pub fn is_detached(refs: &[Ref]) -> bool {
    !refs.iter().any(|r| r.is_head)
}

impl Backend {
    fn walk_page(&self, walk: &Walk, skip: usize, limit: Option<usize>) -> Fut<Vec<CommitNode>> {
        let mut opts = walk.source.options(skip as u32, limit.map(|l| l as u32));
        if walk.source == LogSource::AllRefs {
            opts.hidden = walk.filter.clone();
        }
        let fut = self.call::<CommitGraph>(OperationKind::Log, serde_json::json!({ "opts": opts }));
        Box::pin(async move { fut.await.map(|g| g.nodes) })
    }

    /// The one page fetch: the commits of `walk` from `request.skip` (the walk offset, front commits
    /// not counted), `request.limit` of them.
    pub fn history_page(&self, walk: &Walk, request: LogPageRequest) -> Fut<Vec<CommitNode>> {
        self.walk_page(walk, request.skip, Some(request.limit))
    }

    /// How many commits the whole walk holds, `front` detached-HEAD commits included.
    pub fn history_total(&self, walk: &Walk, front: usize) -> Fut<u64> {
        let mut opts = walk.source.options(0, None);
        if walk.source == LogSource::AllRefs {
            opts.hidden = walk.filter.clone();
        }
        let fut = self.call::<u64>(
            OperationKind::CountCommits,
            serde_json::json!({ "opts": opts }),
        );
        Box::pin(async move { fut.await.map(|n| n + front as u64) })
    }

    /// The list from the top, `depth` walk commits deep (`LazyLog::refresh_depth`).
    pub fn history_refresh(&self, refs: &[Ref], filter: RefFilter, depth: usize) -> Fut<Refresh> {
        let all = Walk {
            source: LogSource::AllRefs,
            filter,
        };
        let head = Walk {
            source: LogSource::Head,
            filter: RefFilter::default(),
        };
        let refs_walk = self.walk_page(&all, 0, Some(depth));
        let head_walk = is_detached(refs).then(|| self.walk_page(&head, 0, Some(depth)));
        Box::pin(async move {
            let refs_nodes = refs_walk.await?;
            let Some(head_walk) = head_walk else {
                return Ok(Refresh {
                    nodes: refs_nodes,
                    front: 0,
                    asked: depth,
                    walk: all,
                });
            };
            let head_nodes = head_walk.await.unwrap_or_default();
            if refs_nodes.is_empty() {
                // No ref reaches any commit: the history of HEAD is the history, and it pages on its own.
                return Ok(Refresh {
                    nodes: head_nodes,
                    front: 0,
                    asked: depth,
                    walk: head,
                });
            }
            let (nodes, front) = merge_head_history(refs_nodes, head_nodes);
            Ok(Refresh {
                nodes,
                front,
                asked: depth,
                walk: all,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_app::graph::{LazyLog, LogLoadSettings};
    use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

    fn node(n: u8) -> CommitNode {
        CommitNode {
            id: Oid([n; 20]),
            parents: vec![],
            summary: format!("c{n}"),
            message: format!("c{n}"),
            author_name: String::new(),
            author_email: String::new(),
            timestamp: 0,
            refs: vec![],
        }
    }

    #[test]
    fn head_only_commits_go_first_and_known_ones_are_not_repeated() {
        let (nodes, head_only) = merge_head_history(vec![node(1), node(2)], vec![node(9), node(2)]);
        let ids: Vec<u8> = nodes.iter().map(|n| n.id.0[0]).collect();
        assert_eq!(ids, vec![9, 1, 2]);
        assert_eq!(head_only, 1);
    }

    fn run<T>(fut: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(fut)
    }

    /// The Log's own sequence, without the window: refresh, then pages until the walk ends.
    struct Driver {
        backend: super::super::Shared,
        lazy: LazyLog,
        nodes: Vec<CommitNode>,
        walk: Walk,
    }

    fn small(initial: usize, page: usize, max: Option<usize>) -> LogLoadSettings {
        LogLoadSettings {
            initial_commits: initial,
            lazy_load: true,
            page_commits: page,
            max_commits: max,
        }
    }

    impl Driver {
        fn open(repo: &FixtureRepo, settings: LogLoadSettings, filter: RefFilter) -> Self {
            let backend = Backend::start(repo.root()).unwrap();
            let mut lazy = LazyLog::new(settings);
            let refs = run(backend.refs()).unwrap();
            let depth = lazy.refresh_depth();
            let r = run(backend.history_refresh(&refs, filter, depth)).unwrap();
            lazy.restart();
            lazy.set_front(r.front);
            lazy.on_refresh_asked(r.nodes.len(), r.asked);
            Self {
                backend,
                lazy,
                nodes: r.nodes,
                walk: r.walk,
            }
        }

        /// Fetches and appends the next page; the request it made, `None` when nothing is due.
        fn page(&mut self) -> Option<LogPageRequest> {
            let req = self.lazy.next_page(0, 1)?;
            self.lazy.mark_requested(req);
            let page = run(self.backend.history_page(&self.walk, req)).unwrap();
            self.lazy.on_page(req, page, &mut self.nodes);
            Some(req)
        }

        fn summaries(&self) -> Vec<&str> {
            self.nodes.iter().map(|n| n.summary.as_str()).collect()
        }
    }

    fn commits(n: usize) -> FixtureRepoBuilder {
        let mut b = FixtureRepoBuilder::new().unwrap().with_initial().unwrap();
        for i in 1..n {
            b = b
                .commit_file(&format!("f{i}.txt"), "x", &format!("c{i}"))
                .unwrap();
        }
        b
    }

    fn three_commits() -> FixtureRepoBuilder {
        FixtureRepoBuilder::new()
            .unwrap()
            .with_initial()
            .unwrap()
            .commit_file("a.txt", "a", "second")
            .unwrap()
            .commit_file("b.txt", "b", "third")
            .unwrap()
    }

    #[test]
    fn attached_head_shows_the_history_with_no_front_and_a_total() {
        gitpoulpi_testkit::isolate_process_env();
        let repo = three_commits().build();
        let mut d = Driver::open(&repo, LogLoadSettings::default(), RefFilter::default());
        assert_eq!(d.nodes.len(), 3);
        assert_eq!(d.lazy.front(), 0);
        assert!(
            d.lazy.is_exhausted(),
            "a short first page is the whole history"
        );
        assert!(d.page().is_none());
        assert_eq!(run(d.backend.history_total(&d.walk, 0)).unwrap(), 3);
    }

    #[test]
    fn pages_follow_the_configured_size_and_stop_at_the_end_of_the_walk() {
        gitpoulpi_testkit::isolate_process_env();
        let repo = commits(5).build();
        let mut d = Driver::open(&repo, small(2, 2, None), RefFilter::default());
        assert_eq!(d.nodes.len(), 2);
        assert!(!d.lazy.is_exhausted());
        let req = d.page().unwrap();
        assert_eq!(
            (req.skip, req.limit),
            (2, 2),
            "the page boundary is the held count"
        );
        assert_eq!(d.nodes.len(), 4);
        let req = d.page().unwrap();
        assert_eq!(req.skip, 4);
        assert_eq!(d.nodes.len(), 5, "a short page holds the root commit");
        assert!(d.lazy.is_exhausted());
        assert!(
            d.page().is_none(),
            "an exhausted walk asks for nothing more"
        );
        assert_eq!(d.summaries().first(), Some(&"c4"));
    }

    #[test]
    fn the_ceiling_shortens_the_last_page() {
        gitpoulpi_testkit::isolate_process_env();
        let repo = commits(6).build();
        let mut d = Driver::open(&repo, small(2, 3, Some(4)), RefFilter::default());
        let req = d.page().unwrap();
        assert_eq!(
            (req.skip, req.limit),
            (2, 2),
            "3 per page, but only 2 fit under 4"
        );
        assert_eq!(d.nodes.len(), 4);
        assert!(d.lazy.is_at_ceiling());
        assert!(d.page().is_none());
    }

    #[test]
    fn detached_head_with_no_ref_at_all_shows_the_history_of_head() {
        gitpoulpi_testkit::isolate_process_env();
        let repo = three_commits().build();
        let branch = repo
            .git(&["rev-parse", "--abbrev-ref", "HEAD"])
            .trim()
            .to_string();
        repo.git(&["checkout", "--detach"]);
        repo.git(&["branch", "-D", &branch]);
        let d = Driver::open(&repo, LogLoadSettings::default(), RefFilter::default());
        assert_eq!(d.nodes.len(), 3, "the three commits of HEAD");
        assert_eq!(d.walk.source, LogSource::Head);
        assert_eq!(d.nodes[0].summary, "third");
        assert_eq!(run(d.backend.history_total(&d.walk, 0)).unwrap(), 3);
    }

    #[test]
    fn detached_head_off_every_ref_keeps_its_own_commits_in_front() {
        gitpoulpi_testkit::isolate_process_env();
        let repo = three_commits().build();
        repo.git(&["checkout", "--detach", "HEAD"]);
        std::fs::write(repo.root().join("d.txt"), "d").unwrap();
        repo.git(&["add", "d.txt"]);
        repo.git(&["commit", "-m", "detached work"]);
        let d = Driver::open(&repo, LogLoadSettings::default(), RefFilter::default());
        assert_eq!(
            d.nodes.len(),
            4,
            "three on the branch plus the detached commit"
        );
        assert_eq!(d.nodes[0].summary, "detached work");
        assert_eq!(d.lazy.front(), 1);
        assert_eq!(
            run(d.backend.history_total(&d.walk, d.lazy.front())).unwrap(),
            4
        );
    }

    #[test]
    fn the_front_does_not_count_towards_the_page_offset() {
        gitpoulpi_testkit::isolate_process_env();
        let repo = three_commits().build();
        repo.git(&["checkout", "--detach", "HEAD"]);
        std::fs::write(repo.root().join("d.txt"), "d").unwrap();
        repo.git(&["add", "d.txt"]);
        repo.git(&["commit", "-m", "detached work"]);
        let mut d = Driver::open(&repo, small(2, 2, None), RefFilter::default());
        assert_eq!(d.lazy.front(), 1);
        assert_eq!(d.nodes.len(), 3, "one in front, two of the walk");
        let req = d.page().unwrap();
        assert_eq!(
            req.skip, 2,
            "two walk commits held, the front one not counted"
        );
        assert_eq!(d.nodes.len(), 4);
        let mut ids: Vec<_> = d.nodes.iter().map(|n| n.id).collect();
        ids.sort_by_key(|i| i.0);
        ids.dedup();
        assert_eq!(ids.len(), 4, "no commit twice");
        assert!(d.lazy.is_exhausted());
        assert_eq!(
            d.summaries(),
            vec!["detached work", "third", "second", "initial"]
        );
    }

    #[test]
    fn detached_head_on_a_ref_adds_nothing() {
        gitpoulpi_testkit::isolate_process_env();
        let repo = three_commits().build();
        repo.git(&["checkout", "--detach", "HEAD~1"]);
        let d = Driver::open(&repo, LogLoadSettings::default(), RefFilter::default());
        assert_eq!(d.nodes.len(), 3);
        assert_eq!(d.lazy.front(), 0);
    }

    #[test]
    fn a_hidden_branch_leaves_its_commits_out_of_every_page() {
        gitpoulpi_testkit::isolate_process_env();
        let repo = three_commits().build();
        let main = repo
            .git(&["rev-parse", "--abbrev-ref", "HEAD"])
            .trim()
            .to_string();
        repo.git(&["checkout", "-b", "side"]);
        std::fs::write(repo.root().join("s.txt"), "s").unwrap();
        repo.git(&["add", "s.txt"]);
        repo.git(&["commit", "-m", "side work"]);
        repo.git(&["checkout", &main]);
        let all = Driver::open(&repo, LogLoadSettings::default(), RefFilter::default());
        assert_eq!(all.nodes.len(), 4);
        let hidden = gitpoulpi_app::graph::walk_filter(
            &["side".to_string()].into_iter().collect(),
            &Default::default(),
        );
        let d = Driver::open(&repo, LogLoadSettings::default(), hidden);
        assert_eq!(d.nodes.len(), 3);
        assert!(d.summaries().iter().all(|s| *s != "side work"));
        assert_eq!(run(d.backend.history_total(&d.walk, 0)).unwrap(), 3);
    }
}
