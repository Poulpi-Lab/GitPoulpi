//! The API a frontend with several repositories open relies on: the link
//! identity on every hook, the one-call `drain`, the opt-in journal wake and
//! the refresh summary. Each test names what turns it red.

use std::sync::Mutex;

use gitpoulpi_core::domain::JournalState;

use super::*;

const WAIT: Duration = Duration::from_secs(60);

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build()
}

/// A host that remembers which link each wake came from.
#[derive(Default)]
struct RecordingHost {
    wakes: Mutex<Vec<(LinkId, std::path::PathBuf)>>,
    wake_journal: bool,
}

impl RecordingHost {
    fn arc(wake_journal: bool) -> Arc<RecordingHost> {
        Arc::new(RecordingHost {
            wake_journal,
            ..RecordingHost::default()
        })
    }

    fn wakes(&self) -> Vec<(LinkId, std::path::PathBuf)> {
        self.wakes.lock().unwrap().clone()
    }
}

impl LinkHost for RecordingHost {
    fn wake(&self, link: &LinkContext) {
        self.wakes
            .lock()
            .unwrap()
            .push((link.id(), link.root().to_path_buf()));
    }
    fn wake_on_journal(&self) -> bool {
        self.wake_journal
    }
    fn before_exit(&self, _link: &LinkContext) {}
    fn raise_window(&self, _link: &LinkContext) -> Result<String, String> {
        Err("no window".to_string())
    }
    fn ask_sensitive_stage(
        &self,
        _link: &LinkContext,
        _root: std::path::PathBuf,
        _action: Action,
        _files: Vec<gitpoulpi_core::privacy::Sensitive>,
        _ticket: Option<Ticket>,
    ) {
    }
    fn on_connected(&self, _link: &LinkContext, _requester: Requester) {}
    fn jobs(&self, _link: &LinkContext, _event: JobEvent) {}
    fn push_group(&self, _link: &LinkContext, _event: PushGroupEvent<'_>) {}
    fn report(&self, _link: &LinkContext, _report: Report<'_>) {}
    fn timing(
        &self,
        _link: &LinkContext,
        _kind: TimingKind,
        _repo: &Path,
        _label: String,
        _ms: u64,
    ) {
    }
    fn expect_background_fetch(&self, _link: &LinkContext) {}
    fn background_fetch_pending(&self, _link: &LinkContext) -> bool {
        false
    }
    fn job_label(&self, id: u64) -> String {
        format!("#{id}")
    }
    fn not_cancellable(&self, _reason: gitpoulpi_core::NotCancellableReason) -> &'static str {
        ""
    }
    fn initial_log_filter(&self, _root: &Path) -> gitpoulpi_core::domain::RefFilter {
        gitpoulpi_core::domain::RefFilter::default()
    }
    fn undo_key(&self, _which: UndoKey) -> String {
        // What the other tests' host says: the recorded transcripts name it.
        "Ctrl+Z".to_string()
    }
    fn daemon_job_live(&self) -> bool {
        false
    }
    fn is_own_hosting_change(&self, _change: &gitpoulpi_ipc::HostingChangeArgs) -> bool {
        false
    }
}

fn start_with(name: &str, repo: RepoRef, config: &Path, host: Arc<dyn LinkHost>) -> DaemonLink {
    start_at(
        name.into(),
        repo,
        Some(config.join("sessions").join(format!("{name}.port"))),
        host,
    )
}

/// Drains `link` until `pick` accepts an event, or fails after [`WAIT`].
fn drain_until<T>(
    link: &DaemonLink,
    what: &str,
    mut pick: impl FnMut(LinkEvent) -> Option<T>,
) -> T {
    let started = std::time::Instant::now();
    while started.elapsed() < WAIT {
        for event in link.drain() {
            if let Some(found) = pick(event) {
                return found;
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("timed out waiting for {what}");
}

/// Two links, two hosts, one process: each host hears only its own link, with
/// that link's id and root. Red when the thread context is not installed on
/// the runtime's threads (the wakes then name no link and go to the installed
/// host), or when the host is not kept per link.
#[test]
fn every_link_wakes_its_own_host_with_its_own_identity() {
    let (first_repo, second_repo) = (fixture(), fixture());
    let config = tempfile::tempdir().unwrap();
    let (host_a, host_b) = (RecordingHost::arc(false), RecordingHost::arc(false));
    let a = start_with(
        "two-links",
        first_repo.repo_ref(),
        config.path(),
        host_a.clone(),
    );
    let b = start_with(
        "two-links",
        second_repo.repo_ref(),
        config.path(),
        host_b.clone(),
    );
    assert_ne!(a.id(), b.id());
    assert_eq!(a.context().root(), first_repo.repo_ref().root);
    for (link, label) in [(&a, "first"), (&b, "second")] {
        let head = drain_until(
            link,
            &format!("the {label} link's first refresh"),
            |event| match event {
                LinkEvent::Refreshed(outcome) => Some(outcome.log_result.unwrap().nodes[0].id),
                _ => None,
            },
        );
        // An answer a spawned task sends: its wake comes from a runtime
        // thread, not from the link thread itself.
        link.request_commit_files(head, false);
        drain_until(link, &format!("the {label} link's commit files"), |event| {
            matches!(event, LinkEvent::CommitFiles(_)).then_some(())
        });
    }
    // Other tests of this process raise detached wakes (id 0) on whichever host
    // was installed last: only wakes that name a link are this test's.
    let named = |host: &RecordingHost| {
        host.wakes()
            .into_iter()
            .filter(|(id, _)| *id != LinkId::DETACHED)
            .collect::<Vec<_>>()
    };
    let (wakes_a, wakes_b) = (named(&host_a), named(&host_b));
    assert!(
        !wakes_a.is_empty() && !wakes_b.is_empty(),
        "both were woken"
    );
    assert!(
        wakes_a
            .iter()
            .all(|(id, root)| *id == a.id() && root == &first_repo.repo_ref().root),
        "host A heard another link: {wakes_a:?}"
    );
    assert!(
        wakes_b
            .iter()
            .all(|(id, root)| *id == b.id() && root == &second_repo.repo_ref().root),
        "host B heard another link: {wakes_b:?}"
    );
}

/// A thread without a link is detached; entering a link's context names it
/// for the guard's life only, and a nested one gives the outer one back. Red
/// when the guard does not restore what it replaced.
#[test]
fn a_thread_without_a_link_is_detached_and_entering_restores() {
    let host = RecordingHost::arc(false);
    let outer = LinkContext::new(LinkId::new(77), "/repo".into(), host.clone());
    let inner = LinkContext::new(LinkId::new(78), "/other".into(), host);
    let id_here = || super::link_ctx::current().map(|link| link.id().get());
    assert_eq!(id_here(), None);
    {
        let _outer = outer.enter();
        assert_eq!(id_here(), Some(77));
        {
            let _inner = inner.enter();
            assert_eq!(id_here(), Some(78));
        }
        assert_eq!(id_here(), Some(77), "the outer link is back");
    }
    assert_eq!(id_here(), None, "and the thread is detached again");
    assert!(LinkContext::detached().is_detached());
}

/// The journal changing wakes the host only when it opted in, and only when
/// the state really differs. Red when the wake is unconditional (the terminal
/// UI would draw twice) or missing.
#[test]
fn a_journal_change_wakes_only_a_host_that_asked_for_it() {
    let state = JournalState {
        undo_count: 1,
        ..JournalState::default()
    };
    for (opted_in, expected) in [(true, 1usize), (false, 0)] {
        let host = RecordingHost::arc(opted_in);
        let context = LinkContext::new(LinkId::new(5), "/repo".into(), host.clone());
        let activity = crate::activity::Activity::default();
        let _in = context.enter();
        apply_journal(&activity, state.clone());
        apply_journal(&activity, state.clone());
        assert_eq!(
            host.wakes().len(),
            expected,
            "opted in: {opted_in} (the same state twice is one change)"
        );
    }
}

/// `drain` hands the journal change over once, then nothing. Red when the
/// change flag is not set, or not consumed.
#[test]
fn drain_reports_a_journal_change_once() {
    let (repo, config) = (fixture(), tempfile::tempdir().unwrap());
    let link = start_with(
        "drain-journal",
        repo.repo_ref(),
        config.path(),
        RecordingHost::arc(false),
    );
    drain_until(&link, "the first refresh", |e| {
        matches!(e, LinkEvent::Refreshed(_)).then_some(())
    });
    let state = JournalState {
        undo_count: 3,
        ..JournalState::default()
    };
    link.activity.journal.set(state.clone());
    let mut seen = Vec::new();
    for event in link.drain() {
        if let LinkEvent::JournalChanged(s) = event {
            seen.push(s);
        }
    }
    assert_eq!(seen, vec![state], "the change arrives once");
    assert!(
        !link
            .drain()
            .iter()
            .any(|e| matches!(e, LinkEvent::JournalChanged(_))),
        "and is consumed"
    );
}

/// A link whose daemon hangs up ends with `Failed`, last, through the
/// same call. Red when `drain` leaves `failure_rx` out.
#[test]
fn drain_ends_with_the_failure_of_a_dead_link() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let port_file = config.path().join("sessions").join("dead.port");
    std::fs::create_dir_all(port_file.parent().unwrap()).unwrap();
    std::fs::write(
        &port_file,
        super::link_lifetime_tests::hang_up_listener().to_string(),
    )
    .unwrap();
    let link = start_at(
        "dead".into(),
        repo.repo_ref(),
        Some(port_file),
        RecordingHost::arc(false),
    );
    let reason = drain_until(&link, "the link's failure", |event| match event {
        LinkEvent::Failed(reason) => Some(reason),
        _ => None,
    });
    assert!(!reason.is_empty());
}

fn next_refresh(
    link: &DaemonLink,
    what: &str,
    accept: impl Fn(&RefreshSummary) -> bool,
) -> RefreshSummary {
    drain_until(link, what, |event| match event {
        LinkEvent::Refreshed(outcome) if accept(&outcome.summary) => Some(outcome.summary),
        _ => None,
    })
}

/// The summary names what changed: the first outcome is "first", an idle poll
/// changes nothing, a new file changes the working tree alone, and a commit
/// moves HEAD, the refs and the Log. Red when `observe` is not called on
/// delivery, or compares the wrong hash.
#[test]
fn a_refresh_says_which_parts_of_the_state_changed() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let link = start_with(
        "summary",
        repo.repo_ref(),
        config.path(),
        RecordingHost::arc(false),
    );
    let first = next_refresh(&link, "the first outcome", |_| true);
    assert!(
        first.first && first.working_tree && first.log && first.refs,
        "{first:?}"
    );

    // Settle: the deferred first-load reads, then an idle poll.
    next_refresh(&link, "an idle poll", |s| {
        // Keep asking until one outcome shows nothing new.
        !s.any()
    });
    // `drain_until` stopped on the first quiet outcome; ask again for a clean base.
    link.request_refresh();
    next_refresh(&link, "a quiet refresh", |s| !s.any());

    std::fs::write(repo.root().join("new.txt"), "hello\n").unwrap();
    link.request_refresh();
    let moved = next_refresh(&link, "the new file", |s| s.working_tree);
    assert!(!moved.first);
    assert!(
        !moved.head && !moved.refs && !moved.stashes && !moved.log,
        "only the working tree moved: {moved:?}"
    );

    repo.git(&["add", "-A"]);
    repo.git(&["commit", "-q", "-m", "second"]);
    link.request_refresh();
    let committed = next_refresh(&link, "the commit", |s| s.head);
    assert!(committed.log && committed.refs, "{committed:?}");
}
