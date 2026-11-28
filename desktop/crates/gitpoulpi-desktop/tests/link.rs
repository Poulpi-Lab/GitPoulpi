//! Headless tests of the window over the shared daemon link: live updates from another client, and the
//! first writes (Fetch, Pull, Push, Undo, Redo) with their status line. Real daemon, fixture
//! repositories, a temporary config directory; no network.

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{Direction, RepoRef};
use gitpoulpi_desktop::app::{self, Overrides, Workspace};
use gitpoulpi_desktop::commands;
use gitpoulpi_desktop::daemon::transfer::Transfer;
use gitpoulpi_desktop::daemon::{Backend, DesktopHost};
use gitpoulpi_link::Action;
use gitpoulpi_testkit::{isolate_process_env, FixtureRepo, FixtureRepoBuilder};
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

fn fixture(with_origin: bool) -> FixtureRepo {
    isolate_process_env();
    let mut b = FixtureRepoBuilder::new().unwrap().with_initial().unwrap();
    if with_origin {
        b = b.with_origin().unwrap();
    }
    b.commit_file("a.txt", "a", "second")
        .unwrap()
        .commit_file("b.txt", "b", "third")
        .unwrap()
        .build()
}

struct Opened {
    window: AnyWindowHandle,
    ws: Entity<Workspace>,
    session: String,
    _prefs: tempfile::TempDir,
}

fn open(cx: &mut TestAppContext, repo: &Path) -> Opened {
    let prefs = tempfile::tempdir().unwrap();
    let backend = Backend::start(repo).unwrap();
    let session = backend.session().to_string();
    cx.executor().allow_parking();
    cx.update(|cx| app::init(cx, Some(prefs.path().to_path_buf()), &Overrides::default()).unwrap());
    let (window, ws) = cx.update(|cx| app::open_main_window(backend, cx).unwrap());
    Opened {
        window,
        ws,
        session,
        _prefs: prefs,
    }
}

/// Runs the app until `done` holds, or fails after a deadline.
fn wait_for(
    cx: &mut TestAppContext,
    what: &str,
    mut done: impl FnMut(&mut TestAppContext) -> bool,
) {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        cx.run_until_parked();
        if done(cx) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out waiting for: {what}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn commits(cx: &mut TestAppContext, ws: &Entity<Workspace>) -> usize {
    ws.read_with(cx, |ws, cx| ws.log.read(cx).loaded_commits())
}

/// Runs the command a toolbar button or key stands for, through the one dispatcher.
fn press(cx: &mut TestAppContext, o: &Opened, what: Transfer) {
    let id = match what {
        Transfer::Fetch => "git_prefix.fetch_all",
        Transfer::Pull => "git_prefix.pull",
        Transfer::Push => "git_prefix.push",
        Transfer::Undo => "global.undo",
        Transfer::Redo => "global.redo",
    };
    cx.update_window(o.window, |_, w, cx| commands::run_id(id, w, cx))
        .unwrap();
}

fn notice_text(cx: &mut TestAppContext, ws: &Entity<Workspace>) -> Option<String> {
    ws.read_with(cx, |ws, _| ws.notice().map(|n| n.text.clone()))
}

fn can(cx: &mut TestAppContext, ws: &Entity<Workspace>, direction: Direction) -> bool {
    ws.read_with(cx, |ws, _| ws.backend().can_undo_redo(direction))
}

fn branch_of(repo: &FixtureRepo) -> String {
    repo.git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string()
}

/// A second frontend of the same session: its own link, committing through the daemon.
fn commit_through_another_client(repo: &FixtureRepo, session: &str, file: &str, message: &str) {
    std::fs::write(repo.root().join(file), message).unwrap();
    let link = gitpoulpi_link::start(
        session.to_string(),
        RepoRef::from_root(repo.root()),
        DesktopHost::global(),
    );
    link.submit_action(Action::StageAll);
    link.submit_action(Action::Commit {
        message: message.to_string(),
        amend: false,
        no_verify: true,
        push: false,
    });
    // Queued on the link's thread: wait for the commit to land before the link is dropped.
    let deadline = Instant::now() + Duration::from_secs(20);
    while !repo.git(&["log", "--format=%s"]).contains(message) {
        assert!(
            Instant::now() < deadline,
            "the second client's commit never landed"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[gpui_kit::test]
fn a_commit_made_through_another_client_shows_in_the_log(cx: &mut TestAppContext) {
    let repo = fixture(false);
    let o = open(cx, repo.root());
    wait_for(cx, "the first load", |cx| commits(cx, &o.ws) == 3);
    commit_through_another_client(&repo, &o.session, "c.txt", "from another client");
    wait_for(cx, "the Log to show the new commit", |cx| {
        commits(cx, &o.ws) == 4
    });
    assert!(o.ws.read_with(cx, |ws, _| ws.live_refreshes()) >= 1);
}

#[gpui_kit::test]
fn a_live_reload_keeps_the_selection_on_its_commit(cx: &mut TestAppContext) {
    let repo = fixture(false);
    let o = open(cx, repo.root());
    wait_for(cx, "the first load", |cx| commits(cx, &o.ws) == 3);
    let log = o.ws.read_with(cx, |ws, _| ws.log.clone());
    // Row 0 is the newest commit; select the one below it.
    log.update(cx, |l, cx| l.select(1, cx));
    let chosen = log.read_with(cx, |l, _| l.selected_commit()).unwrap();
    commit_through_another_client(&repo, &o.session, "c.txt", "pushes the rows down");
    wait_for(cx, "the Log to show the new commit", |cx| {
        commits(cx, &o.ws) == 4
    });
    wait_for(cx, "the selection to follow its commit", |cx| {
        log.read_with(cx, |l, _| l.selected_row() == 2)
    });
    assert_eq!(log.read_with(cx, |l, _| l.selected_commit()), Some(chosen));
}

#[gpui_kit::test]
fn a_plain_git_commit_is_picked_up_without_user_action(cx: &mut TestAppContext) {
    let repo = fixture(false);
    let o = open(cx, repo.root());
    wait_for(cx, "the first load", |cx| commits(cx, &o.ws) == 3);
    std::fs::write(repo.root().join("d.txt"), "d").unwrap();
    repo.git(&["add", "d.txt"]);
    repo.git(&[
        "-c",
        "user.name=T",
        "-c",
        "user.email=t@example.org",
        "commit",
        "-m",
        "outside",
    ]);
    wait_for(cx, "the periodic refresh to show the commit", |cx| {
        commits(cx, &o.ws) == 4
    });
}

#[gpui_kit::test]
fn undo_and_redo_through_the_window_move_the_log(cx: &mut TestAppContext) {
    let repo = fixture(false);
    let o = open(cx, repo.root());
    wait_for(cx, "the first load", |cx| commits(cx, &o.ws) == 3);
    commit_through_another_client(&repo, &o.session, "c.txt", "to be undone");
    wait_for(cx, "four commits", |cx| commits(cx, &o.ws) == 4);
    wait_for(cx, "the journal to offer Undo", |cx| {
        can(cx, &o.ws, Direction::Undo)
    });

    press(cx, &o, Transfer::Undo);
    wait_for(cx, "the Log to drop the commit", |cx| {
        commits(cx, &o.ws) == 3
    });
    assert!(!repo.git(&["log", "--format=%s"]).contains("to be undone"));
    wait_for(cx, "the Undo result on the status line", |cx| {
        notice_text(cx, &o.ws).is_some_and(|t| !t.is_empty())
    });

    wait_for(cx, "the journal to offer Redo", |cx| {
        can(cx, &o.ws, Direction::Redo)
    });
    press(cx, &o, Transfer::Redo);
    wait_for(cx, "the Log to show the commit again", |cx| {
        commits(cx, &o.ws) == 4
    });
    assert!(repo.git(&["log", "--format=%s"]).contains("to be undone"));
}

#[gpui_kit::test]
fn undo_with_nothing_to_undo_says_why_and_changes_nothing(cx: &mut TestAppContext) {
    let repo = fixture(false);
    let o = open(cx, repo.root());
    wait_for(cx, "the first load", |cx| commits(cx, &o.ws) == 3);
    press(cx, &o, Transfer::Undo);
    wait_for(cx, "the refusal", |cx| {
        o.ws.read_with(cx, |ws, _| ws.notice().is_some_and(|n| n.error))
    });
    assert_eq!(commits(cx, &o.ws), 3);
}

#[gpui_kit::test]
fn push_fetch_and_pull_run_through_the_link(cx: &mut TestAppContext) {
    let repo = fixture(true);
    let branch = branch_of(&repo);
    let o = open(cx, repo.root());
    wait_for(cx, "the first load", |cx| commits(cx, &o.ws) == 3);

    // Push: the branch has no upstream, so it goes to origin and sets it.
    press(cx, &o, Transfer::Push);
    wait_for(cx, "the push to reach origin", |_| {
        repo.try_git(&["ls-remote", "--heads", "origin", &branch])
            .is_some_and(|out| out.contains(&branch))
    });
    wait_for(cx, "the push result on the status line", |cx| {
        // The link words the result itself ("pushed ..."); the "started" line is gone by then.
        o.ws.read_with(cx, |ws, _| {
            ws.notice()
                .is_some_and(|n| !n.error && n.text != gitpoulpi_i18n::msg::link::ACT_PUSHING.get())
        })
    });

    // Fetch: another user pushed; the remote-tracking ref moves.
    repo.push_from_elsewhere(&branch, "elsewhere");
    let before = repo.git(&["rev-parse", &format!("origin/{branch}")]);
    press(cx, &o, Transfer::Fetch);
    wait_for(cx, "the fetch to move origin/<branch>", |_| {
        repo.git(&["rev-parse", &format!("origin/{branch}")]) != before
    });

    // Pull: fast-forward onto it, and the Log follows.
    press(cx, &o, Transfer::Pull);
    wait_for(cx, "the Log to show the pulled commit", |cx| {
        commits(cx, &o.ws) == 4
    });
    assert!(repo.git(&["log", "--format=%s"]).contains("elsewhere"));
}

#[gpui_kit::test]
fn push_without_a_remote_says_so(cx: &mut TestAppContext) {
    let repo = fixture(false);
    let o = open(cx, repo.root());
    wait_for(cx, "the first load", |cx| commits(cx, &o.ws) == 3);
    press(cx, &o, Transfer::Push);
    wait_for(cx, "the refusal", |cx| {
        notice_text(cx, &o.ws).is_some_and(|t| t == gitpoulpi_desktop::strings::PUSH_NO_REMOTE)
    });
}
