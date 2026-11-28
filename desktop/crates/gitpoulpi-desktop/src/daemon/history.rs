//! Which commits the Log shows, and how it pages through them.
//!
//! The Log walks every ref (`--branches --tags --remotes`). On a detached HEAD that walk can miss HEAD
//! itself: a clone checked out at a tag or a commit has no local branch, and a repository whose only
//! branch was deleted has no ref at all, so the walk returns nothing. The first page therefore also
//! asks for the history of HEAD and puts the commits only HEAD reaches in front, which keeps every
//! child before its parents for the graph layout.

use std::collections::HashSet;

use gitpoulpi_core::domain::{CommitNode, Oid, Ref};

use super::queries::LogSource;
use super::Backend;

/// Commits fetched for the first page.
pub const FIRST_PAGE: u32 = 3000;
/// Commits fetched for each following page.
pub const NEXT_PAGE: u32 = 10_000;

/// How later pages continue the first one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HistoryPlan {
    pub source: LogSource,
    /// Commits at the front of the list that the paged walk does not contain (reachable from a
    /// detached HEAD only); subtracted from the offset of the next page.
    pub head_only: usize,
}

impl HistoryPlan {
    /// Offset of the next page given how many commits are loaded.
    pub fn next_skip(&self, loaded: usize) -> u32 {
        loaded.saturating_sub(self.head_only) as u32
    }
}

/// The first page and what is needed to continue it.
pub struct HistoryStart {
    pub nodes: Vec<CommitNode>,
    pub plan: HistoryPlan,
    /// Whether the paged walk may have more commits.
    pub more: bool,
    /// Commits the whole history holds, when the daemon could count them.
    pub total: Option<u64>,
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
    /// The first page of the Log for the current repository.
    pub fn history_start(
        &self,
        refs: &[Ref],
    ) -> impl std::future::Future<Output = Result<HistoryStart, String>> + Send + 'static {
        let detached = is_detached(refs);
        let refs_walk = self.log_page(LogSource::AllRefs, 0, FIRST_PAGE);
        let head_walk = detached.then(|| self.log_page(LogSource::Head, 0, FIRST_PAGE));
        let total_refs = self.count(LogSource::AllRefs);
        let total_head = detached.then(|| self.count(LogSource::Head));
        async move {
            let refs_nodes = refs_walk.await?;
            let more_refs = refs_nodes.len() as u32 >= FIRST_PAGE;
            let Some(head_walk) = head_walk else {
                return Ok(HistoryStart {
                    plan: HistoryPlan {
                        source: LogSource::AllRefs,
                        head_only: 0,
                    },
                    more: more_refs,
                    total: total_refs.await.ok(),
                    nodes: refs_nodes,
                });
            };
            let head_nodes = head_walk.await.unwrap_or_default();
            if refs_nodes.is_empty() {
                // No ref reaches any commit: the history of HEAD is the history, and it pages on its own.
                let more = head_nodes.len() as u32 >= FIRST_PAGE;
                let total = match total_head {
                    Some(c) => c.await.ok(),
                    None => None,
                };
                return Ok(HistoryStart {
                    plan: HistoryPlan {
                        source: LogSource::Head,
                        head_only: 0,
                    },
                    nodes: head_nodes,
                    more,
                    total,
                });
            }
            let (nodes, head_only) = merge_head_history(refs_nodes, head_nodes);
            let total = total_refs.await.ok().map(|t| t + head_only as u64);
            Ok(HistoryStart {
                plan: HistoryPlan {
                    source: LogSource::AllRefs,
                    head_only,
                },
                nodes,
                more: more_refs,
                total,
            })
        }
    }

    /// The page after the `loaded` commits already held.
    pub fn history_next(
        &self,
        plan: HistoryPlan,
        loaded: usize,
    ) -> impl std::future::Future<Output = Result<Vec<CommitNode>, String>> + Send + 'static {
        self.log_page(plan.source, plan.next_skip(loaded), NEXT_PAGE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_testkit::FixtureRepoBuilder;

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

    #[test]
    fn next_skip_ignores_the_head_only_prefix() {
        let plan = HistoryPlan {
            source: LogSource::AllRefs,
            head_only: 2,
        };
        assert_eq!(plan.next_skip(10), 8);
        assert_eq!(
            HistoryPlan {
                source: LogSource::Head,
                head_only: 0
            }
            .next_skip(10),
            10
        );
    }

    fn run<T>(fut: impl std::future::Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(fut)
    }

    /// Opens the fixture in the embedded daemon and returns the first page.
    fn start(repo: &gitpoulpi_testkit::FixtureRepo) -> HistoryStart {
        let backend = Backend::start(repo.root()).unwrap();
        let refs = run(backend.refs()).unwrap();
        run(backend.history_start(&refs)).unwrap()
    }

    /// What the plain all-refs walk returns for the fixture (the Log's old, only source).
    fn all_refs_walk(repo: &gitpoulpi_testkit::FixtureRepo) -> usize {
        let backend = Backend::start(repo.root()).unwrap();
        run(backend.log_page(LogSource::AllRefs, 0, FIRST_PAGE))
            .unwrap()
            .len()
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
    fn attached_head_shows_the_history_as_before() {
        gitpoulpi_testkit::isolate_process_env();
        let repo = three_commits().build();
        let page = start(&repo);
        assert_eq!(page.nodes.len(), 3);
        assert_eq!(page.plan.head_only, 0);
        assert_eq!(page.total, Some(3));
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
        assert_eq!(
            all_refs_walk(&repo),
            0,
            "the all-refs walk alone shows nothing here"
        );
        let page = start(&repo);
        assert_eq!(page.nodes.len(), 3, "the three commits of HEAD");
        assert_eq!(page.plan.source, LogSource::Head);
        assert_eq!(page.total, Some(3));
        assert_eq!(page.nodes[0].summary, "third");
    }

    #[test]
    fn detached_head_off_every_ref_keeps_its_own_commits_in_front() {
        gitpoulpi_testkit::isolate_process_env();
        let repo = three_commits().build();
        repo.git(&["checkout", "--detach", "HEAD~2"]);
        std::fs::write(repo.root().join("d.txt"), "d").unwrap();
        repo.git(&["add", "d.txt"]);
        repo.git(&["commit", "-m", "detached work"]);
        let page = start(&repo);
        assert_eq!(
            page.nodes.len(),
            4,
            "three on the branch plus the detached commit"
        );
        assert_eq!(page.nodes[0].summary, "detached work");
        assert_eq!(page.plan.head_only, 1);
        assert_eq!(page.total, Some(4));
    }

    #[test]
    fn detached_head_on_a_ref_adds_nothing() {
        gitpoulpi_testkit::isolate_process_env();
        let repo = three_commits().build();
        repo.git(&["checkout", "--detach", "HEAD~1"]);
        let page = start(&repo);
        assert_eq!(page.nodes.len(), 3);
        assert_eq!(page.plan.head_only, 0);
    }
}
