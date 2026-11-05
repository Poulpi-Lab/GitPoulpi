//! The infinite "Still loading... (large repository?)" -- see
//! `session-and-state.md`, "The daemon died with the window". Every test
//! here passes an explicit discovery file (`start_at`), never
//! `GITPOULPI_CONFIG_DIR`.

use super::*;

/// Generous: a first refresh on an idle machine is well under a
/// second. The point is failing instead of hanging CI.
const WAIT: Duration = Duration::from_secs(60);

fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build()
}

fn port_in(port_file: &std::path::Path) -> u16 {
    std::fs::read_to_string(port_file)
        .expect("the discovery file was written")
        .trim()
        .parse()
        .unwrap()
}

/// Each window (each tab switch, each visit to the `⌂` tab) starts its own
/// link and drops the previous one. The embedded daemon must not live on the
/// runtime of the link that started it: it would die with that window, and
/// the next window would connect to it mid-shutdown and wait forever for its
/// first refresh.
///
/// Asserted two ways, so a lucky interleaving cannot pass: every window gets
/// its first refresh, *and* it is the same daemon throughout (the discovery
/// file never changes). Red if `Daemon::spawn` goes back on the caller's
/// runtime in `ensure_daemon_at`.
#[test]
fn the_embedded_daemon_outlives_the_window_that_started_it() {
    let fixture = fixture();
    let config = tempfile::tempdir().unwrap();
    let port_file = config.path().join("sessions").join("lifetime.port");

    let mut first_port = None;
    for window in 0..8 {
        let link = start_at(
            "lifetime".into(),
            fixture.repo_ref(),
            Some(port_file.clone()),
        );
        let outcome = link.outcome_rx.recv_timeout(WAIT).unwrap_or_else(|e| {
            panic!(
                "window {window} never got its first refresh ({e}) - this is the \
                 infinite \"Still loading...\"; link failure: {:?}",
                link.failure_rx.try_recv()
            )
        });
        assert!(
            outcome.log_result.is_ok(),
            "window {window}: {:?}",
            outcome.log_result.err()
        );
        let port = port_in(&port_file);
        assert_eq!(
            *first_port.get_or_insert(port),
            port,
            "window {window} found the daemon gone and started another - the \
             embedded daemon died with the window before it"
        );
        drop(link);
    }
}

/// A stand-in for a daemon that reads one request and hangs up
/// without answering it - a daemon stopping mid-request, or one from
/// another build dropping a request it cannot parse. Reading the
/// request first matters: it is what leaves the link *waiting* for a
/// response (rather than failing to write), the state that could hang.
/// Returns its port; it serves until the process ends.
fn hang_up_listener() -> u16 {
    use std::io::Read;
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            std::thread::spawn(move || {
                let mut len = [0u8; 4];
                if stream.read_exact(&mut len).is_ok() {
                    let mut payload = vec![0u8; u32::from_be_bytes(len) as usize];
                    let _ = stream.read_exact(&mut payload);
                }
                // Dropped here: the connection closes, unanswered.
            });
        }
    });
    port
}

/// A link whose daemon hangs up must *say so*, promptly, on `failure_rx`;
/// the render loop turns that into the error with Retry. Red (times out)
/// without the `pending.lock().unwrap().close(reason)` call in the reader
/// task, which leaves the first refresh awaiting a response that never comes.
#[test]
fn a_daemon_that_hangs_up_is_reported_not_waited_on() {
    let fixture = fixture();
    let config = tempfile::tempdir().unwrap();
    let port_file = config.path().join("sessions").join("hangup.port");
    std::fs::create_dir_all(port_file.parent().unwrap()).unwrap();
    std::fs::write(&port_file, hang_up_listener().to_string()).unwrap();

    let link = start_at("hangup".into(), fixture.repo_ref(), Some(port_file));
    let reason = link
        .failure_rx
        .recv_timeout(WAIT)
        .expect("the link must report that it stopped, not wait forever");
    assert!(
        reason.contains("daemon"),
        "the reason should say what happened: {reason}"
    );
}

/// The other silent `return` the link had: nothing listening at all.
#[tokio::test]
async fn a_daemon_that_cannot_be_reached_is_reported() {
    let port = {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.local_addr().unwrap().port()
    };
    let (_manual_refresh_tx, manual_refresh_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_commit_files_tx, commit_files_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_working_diff_tx, working_diff_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_file_insight_tx, file_insight_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let (_log_page_tx, log_page_task_rx) = tokio::sync::mpsc::unbounded_channel();
    let reason = tokio::time::timeout(
        WAIT,
        link_loop_with_port(
            port,
            "unreachable".into(),
            fixture().repo_ref(),
            LinkChannels {
                manual_refresh_rx,
                action_rx,
                commit_files_task_rx,
                working_diff_task_rx,
                file_insight_task_rx,
                merge_task_rx: tokio::sync::mpsc::unbounded_channel().1,
                outcome_tx: std_mpsc::channel().0,
                action_error_tx: std_mpsc::channel().0,
                action_message_tx: std_mpsc::channel().0,
                push_rejected_tx: std_mpsc::channel().0,
                autostash_tx: std_mpsc::channel().0,
                commit_files_out_tx: std_mpsc::channel().0,
                working_diff_out_tx: std_mpsc::channel().0,
                file_insight_out_tx: std_mpsc::channel().0,
                merge_out_tx: std_mpsc::channel().0,
                log_page_task_rx,
                log_page_out_tx: std_mpsc::channel().0,
                stats_worker: None,
                split_worker: None,
                log_depth: Arc::new(AtomicUsize::new(0)),
                log_filter: Default::default(),
                lfs_gate: Default::default(),
                activity: Default::default(),
            },
            Default::default(),
        ),
    )
    .await
    .expect("the link must give up on an unreachable daemon, not wait");
    let reason = reason.expect("an unreachable daemon is a failure, with a reason");
    assert!(reason.contains("could not connect"), "{reason}");
}
