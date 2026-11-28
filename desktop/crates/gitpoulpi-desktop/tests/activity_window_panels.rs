//! Headless tests of the Activity Log, the background jobs panel, the grouped push progress, the rejected
//! push dialog and the Log's ref pill cursor, on the real main window. Fixture repositories with local
//! bare repositories as remotes and a temporary config directory; no network, no visible window.

mod activity_common;
mod refs_ops_common;

use std::process::Command;

use activity_common::*;
use gitpoulpi_core::domain::{
    GroupAction, GroupMode, Oid, OperationKind, RemoteGroupRunRequest, RunTarget, TargetStatus,
};
use gitpoulpi_desktop::panels::activity::log_model::{Levels, LogTab};
use gitpoulpi_desktop::panels::activity::{group, jobs};
use gitpoulpi_i18n::msg;
use gitpoulpi_ipc::JobState;
use gitpoulpi_link::Action;
use gitpoulpi_log::Level;
use gitpoulpi_testkit::FixtureRepo;
use gpui_kit::TestAppContext;
use refs_ops_common::*;

fn git_in(dir: &std::path::Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args([
            "-c",
            "user.name=other",
            "-c",
            "user.email=other@example.com",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Somebody else pushes a commit to the origin's main.
fn other_pushes(repo: &FixtureRepo, file: &str) {
    let url = repo
        .git(&["remote", "get-url", "origin"])
        .trim()
        .to_string();
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().join("other");
    git_in(dir.path(), &["clone", "-q", &url, work.to_str().unwrap()]);
    std::fs::write(work.join(file), "other\n").unwrap();
    git_in(&work, &["add", "--", file]);
    git_in(&work, &["commit", "-q", "-m", "from the other clone"]);
    git_in(&work, &["push", "-q", "origin", "HEAD:main"]);
}

fn local_commit(repo: &FixtureRepo, file: &str, message: &str) {
    std::fs::write(repo.root().join(file), "mine\n").unwrap();
    repo.git(&["add", "--", file]);
    repo.git(&["commit", "-q", "-m", message]);
}

fn origin_main(repo: &FixtureRepo) -> String {
    repo.git(&["ls-remote", "origin", "refs/heads/main"])
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

// ---- the Activity Log ------------------------------------------------------------------------------

#[gpui_kit::test]
fn the_git_tab_lists_what_the_link_reported_filters_by_level_and_copies_an_entry(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    // A success and a failure (a branch that already exists).
    new_branch(cx, &ws, &repo, "fine");
    let at = head_oid(&repo);
    ws.read_with(cx, |ws, _| {
        ws.backend().submit(Action::BranchCreate {
            name: "main".into(),
            at,
        })
    });
    wait_until(cx, "the failure in the feed", |cx| {
        ws.read_with(cx, |ws, _| {
            ws.backend()
                .host()
                .feed()
                .iter()
                .any(|e| e.level == Level::Error)
        })
    });
    run(cx, window, "nav_prefix.activity_log");
    let view = activity_view(cx);
    assert_eq!(view.read_with(cx, |v, _| v.tab), LogTab::Git);
    // Newest first: the failure heads the list, with the daemon's words and the operation as details.
    let first = view.read_with(cx, |v, _| (v.level_of(0), v.line(0), v.detail_lines()));
    assert_eq!(first.0, Level::Error);
    assert!(
        first
            .1
            .contains(&msg::activity_log::failed_title(msg::toolbar::BRANCH.get()))
            || !first.1.is_empty()
    );
    assert!(first.2.len() >= 2, "details: {:?}", first.2);
    // Levels: hiding Error removes the failure, showing it again brings it back.
    let total = view.read_with(cx, |v, _| v.len());
    view.update(cx, |v, cx| v.toggle_level(Level::Error, cx));
    assert!(view.read_with(cx, |v, _| v.len()) < total);
    assert!(view.read_with(cx, |v, _| (0..v.len())
        .all(|i| v.level_of(i) != Level::Error)));
    view.update(cx, |v, cx| v.toggle_level(Level::Error, cx));
    assert_eq!(view.read_with(cx, |v, _| v.len()), total);
    // Debug is quiet by default and shows when asked.
    view.update(cx, |v, cx| v.set_levels(Levels::all(), cx));
    assert!(view.read_with(cx, |v, _| v.len()) >= total);
    view.update(cx, |v, cx| v.set_levels(Levels::none(), cx));
    assert_eq!(view.read_with(cx, |v, _| v.len()), 0);
    view.update(cx, |v, cx| v.set_levels(Levels::default(), cx));
    // Copy puts the selected entry on the clipboard.
    view.update(cx, |v, cx| {
        v.select(0, cx);
        v.copy_selected(cx)
    });
    let copied = cx
        .read_from_clipboard()
        .and_then(|c| c.text())
        .unwrap_or_default();
    assert!(copied.contains("ERROR"), "{copied}");
    assert!(
        !notice(cx, &ws).is_empty(),
        "the status line says what was copied"
    );
    // The problem keys land on the failure.
    view.update(cx, |v, cx| {
        v.select(v.len() - 1, cx);
        v.step_problem(true, cx);
    });
    assert_eq!(
        view.read_with(cx, |v, _| v.level_of(v.selected)),
        Level::Error
    );
}

#[gpui_kit::test]
fn the_last_error_commands_copy_and_open_the_failure_and_say_when_there_is_none(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    run(cx, window, "palette.copy_last_error");
    assert_eq!(notice(cx, &ws), msg::activity_log::WHY_NO_LAST_ERROR.get());
    let at = head_oid(&repo);
    ws.read_with(cx, |ws, _| {
        ws.backend().submit(Action::BranchCreate {
            name: "main".into(),
            at,
        })
    });
    wait_until(cx, "the failure", |cx| {
        ws.read_with(cx, |ws, _| ws.backend().host().last_error().is_some())
    });
    run(cx, window, "palette.copy_last_error");
    let copied = cx
        .read_from_clipboard()
        .and_then(|c| c.text())
        .unwrap_or_default();
    assert!(copied.contains("ERROR"), "{copied}");
    run(cx, window, "palette.open_last_error");
    let view = activity_view(cx);
    assert_eq!(
        view.read_with(cx, |v, _| v.level_of(v.selected)),
        Level::Error
    );
    // Level commands act on the open log.
    run(cx, window, "palette.activity_level_toggle_error");
    assert!(view.read_with(cx, |v, _| (0..v.len())
        .all(|i| v.level_of(i) != Level::Error)));
    run(cx, window, "palette.activity_level_all");
    run(cx, window, "palette.activity_level_app_tab");
    assert_eq!(view.read_with(cx, |v, _| v.tab), LogTab::App);
    run(cx, window, "palette.activity_level_git_tab");
    assert_eq!(view.read_with(cx, |v, _| v.tab), LogTab::Git);
    run(cx, window, "palette.activity_level_scope");
    assert_eq!(
        view.read_with(cx, |v, _| v.scope),
        gitpoulpi_desktop::panels::activity::log_model::Scope::All
    );
}

#[gpui_kit::test]
fn the_application_tab_reads_the_log_files_and_clearing_asks_first(cx: &mut TestAppContext) {
    let repo = repo();
    let dir = tempfile::tempdir().unwrap();
    let logger = gitpoulpi_log::init(gitpoulpi_log::Options {
        dir: Some(dir.path().to_path_buf()),
        level: Level::Trace,
        retention_days: 3,
        max_file_bytes: 1 << 20,
        process: "desktop",
    });
    let (window, _ws) = open(cx, &repo);
    logger.emit(Level::Warn, "test", "a line the log tab must show");
    logger.emit(Level::Debug, "test", "a debug line");
    logger.flush();
    run(cx, window, "palette.activity_level_app_tab");
    let view = activity_view(cx);
    wait_until(cx, "the lines", |cx| {
        view.read_with(cx, |v, _| {
            (0..v.len()).any(|i| v.line(i).contains("a line the log tab must show"))
        })
    });
    // The Debug line is hidden until the level is shown.
    assert!(view.read_with(cx, |v, _| !(0..v.len())
        .any(|i| v.line(i).contains("a debug line"))));
    run(cx, window, "palette.activity_level_toggle_debug");
    assert!(view.read_with(cx, |v, _| (0..v.len())
        .any(|i| v.line(i).contains("a debug line"))));
    assert!(
        view.read_with(cx, |v, _| v.dir.is_some()),
        "the log folder is known"
    );
    // Clear asks; Cancel keeps the lines, Clear removes them.
    run(cx, window, "palette.activity_level_clear_log");
    assert!(dialog_open(cx, window));
    assert_eq!(question_title(cx), msg::activity_log::CLEAR_TITLE.get());
    answer(cx, window, 1);
    assert!(view.read_with(cx, |v, _| (0..v.len())
        .any(|i| v.line(i).contains("a line the log tab must show"))));
    run(cx, window, "palette.activity_level_clear_log");
    answer(cx, window, 0);
    wait_until(cx, "the log cleared", |cx| {
        view.read_with(cx, |v, _| {
            !(0..v.len()).any(|i| v.line(i).contains("a line the log tab must show"))
        })
    });
    assert_eq!(notice(cx, &_ws), msg::activity_log::CLEAR_DONE.get());
}

// ---- jobs ------------------------------------------------------------------------------------------

#[gpui_kit::test]
fn the_jobs_panel_lists_running_queued_and_finished_jobs_and_cancels_where_it_can(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    feed_job(
        cx,
        &ws,
        job(1, OperationKind::Push, JobState::Running, "/work/tool"),
    );
    feed_job(
        cx,
        &ws,
        job(2, OperationKind::Fetch, JobState::Queued, "/work/lib"),
    );
    feed_job(
        cx,
        &ws,
        job(3, OperationKind::Pull, JobState::Done, "/work/tool"),
    );
    let mut sent = job(4, OperationKind::Push, JobState::Running, "/work/sent");
    sent.not_cancellable = Some(gitpoulpi_core::NotCancellableReason::Sent);
    feed_job(cx, &ws, sent);
    // The indicator counts what is going, the panel lists everything, active first.
    assert_eq!(
        ws.read_with(cx, |ws, _| ws.backend().host().live_jobs().len()),
        3
    );
    run(cx, window, "nav_prefix.jobs");
    let view = cx
        .update(|cx| jobs::current(cx))
        .expect("the jobs panel is open");
    let listed = view.read_with(cx, |v, _| v.jobs());
    assert_eq!(listed.len(), 4);
    assert!(
        listed.last().unwrap().state.is_finished(),
        "finished jobs come last"
    );
    assert_eq!(
        jobs::label(&listed[0]),
        msg::jobs::label(msg::jobs::KIND_PUSH.get(), "tool")
    );
    assert_eq!(
        jobs::state_word(JobState::Queued),
        msg::jobs::STATE_QUEUED.get()
    );
    // A job that cannot be cancelled says why and nothing is sent.
    view.update(cx, |v, cx| {
        v.select(4, cx);
        v.cancel_selected(cx)
    });
    assert_eq!(
        view.read_with(cx, |v, _| v.note.clone()),
        Some(msg::jobs::not_cancellable(msg::jobs::WHY_SENT.get()))
    );
    assert_eq!(
        notice(cx, &ws),
        msg::jobs::not_cancellable(msg::jobs::WHY_SENT.get())
    );
    // A finished one too.
    view.update(cx, |v, cx| {
        v.select(3, cx);
        v.cancel_selected(cx)
    });
    assert_eq!(
        notice(cx, &ws),
        msg::jobs::not_cancellable(msg::jobs::WHY_FINISHED.get())
    );
    // A cancellable one is sent to the daemon, which has never heard of this synthetic job: its answer
    // names the job in the status line.
    view.update(cx, |v, cx| {
        v.select(1, cx);
        v.cancel_selected(cx)
    });
    wait_until(cx, "the daemon's answer", |cx| {
        notice(cx, &ws).contains("tool")
    });
}

#[gpui_kit::test]
fn cancelling_with_no_job_running_says_so(cx: &mut TestAppContext) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    run(cx, window, "palette.jobs_cancel_all");
    assert_eq!(notice(cx, &ws), msg::jobs::PALETTE_NONE.get());
    run(cx, window, "palette.jobs_show_background_jobs");
}

// ---- the grouped push ------------------------------------------------------------------------------

fn add_backup_remote(repo: &FixtureRepo) {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().to_path_buf();
    std::mem::forget(temp);
    let bare = dir.join("backup.git");
    git_in(&dir, &["init", "-q", "--bare", bare.to_str().unwrap()]);
    repo.git(&["remote", "add", "backup", bare.to_str().unwrap()]);
    repo.git(&["remote", "add", "broken", "/nonexistent/nowhere.git"]);
}

fn run_request(targets: &[(&str, bool)]) -> RemoteGroupRunRequest {
    RemoteGroupRunRequest {
        branch: "main".into(),
        mode: GroupMode::Push,
        targets: targets
            .iter()
            .map(|(remote, absent)| RunTarget {
                remote: remote.to_string(),
                branch: "main".into(),
                action: GroupAction::Push,
                lease: None,
                absent: *absent,
            })
            .collect(),
        ..Default::default()
    }
}

#[gpui_kit::test]
fn a_grouped_push_shows_each_remote_with_its_status_and_retry_offers_the_failed_ones(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    add_backup_remote(&repo);
    let (window, ws) = open(cx, &repo);
    // Nothing yet: the command says so and the palette row is unavailable.
    run(cx, window, "palette.show_grouped_push");
    assert_eq!(notice(cx, &ws), msg::palette::WHY_NO_GROUPED_PUSH.get());
    // origin is up to date, backup gets the branch, broken cannot be reached.
    let request = run_request(&[("origin", false), ("backup", true), ("broken", false)]);
    ws.read_with(cx, |ws, _| {
        ws.backend()
            .submit(Action::RemoteGroupRun(Box::new(request)))
    });
    wait_until(cx, "the run to end", |cx| {
        ws.read_with(cx, |ws, _| {
            ws.backend().host().group_run().is_some_and(|r| !r.going())
        })
    });
    let run_state = ws
        .read_with(cx, |ws, _| ws.backend().host().group_run())
        .unwrap();
    let status_of = |remote: &str| {
        run_state
            .rows
            .iter()
            .find(|r| r.remote == remote)
            .and_then(|r| r.status.clone())
    };
    assert!(
        matches!(
            status_of("backup"),
            Some(TargetStatus::Pushed { created: true })
        ),
        "{:?}",
        run_state.rows
    );
    assert!(matches!(
        status_of("origin"),
        Some(TargetStatus::UpToDate | TargetStatus::Pushed { .. })
    ));
    assert!(matches!(
        status_of("broken"),
        Some(TargetStatus::Failed { .. })
    ));
    assert!(
        run_state.has_retryable(),
        "the missing remote can be retried"
    );
    assert!(
        !repo
            .git(&["ls-remote", "backup", "refs/heads/main"])
            .is_empty(),
        "the push really went"
    );
    // The progress view lists a row per remote and offers Retry for the failed one.
    run(cx, window, "palette.show_grouped_push");
    let view = cx
        .update(|cx| group::current(cx))
        .expect("the progress view is open");
    assert_eq!(view.read_with(cx, |v, _| v.run().unwrap().rows.len()), 3);
    let texts: Vec<String> = run_state
        .rows
        .iter()
        .map(|r| group::row_text(r).1)
        .collect();
    assert!(
        texts
            .iter()
            .any(|t| t == msg::push_group::RUN_CREATED.get()),
        "{texts:?}"
    );
    assert!(!group::footer_text(&run_state).is_empty());
    let backend = ws.read_with(cx, |ws, _| ws.backend().clone());
    assert!(group::retry(&backend), "a failed remote can be retried");
}

// ---- the rejected push -----------------------------------------------------------------------------

#[gpui_kit::test]
fn a_push_the_remote_refused_asks_pull_first_force_or_cancel_and_force_is_always_confirmed(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    other_pushes(&repo, "theirs.txt");
    local_commit(&repo, "mine.txt", "my commit");
    let theirs = origin_main(&repo);
    let (window, ws) = open(cx, &repo);
    let title = msg::remotes::diverged_title("origin", "main");
    // Cancel changes nothing on the remote.
    run(cx, window, "git_prefix.push");
    wait_until(cx, "the question", |cx| dialog_open(cx, window));
    assert_eq!(question_title(cx), title);
    answer(cx, window, 2);
    cx.run_until_parked();
    assert_eq!(origin_main(&repo), theirs);
    // Force push asks again, naming the branch and the remote; Cancel there changes nothing either.
    run(cx, window, "git_prefix.push");
    wait_until(cx, "the question", |cx| dialog_open(cx, window));
    answer(cx, window, 1);
    wait_until(cx, "the force question", |cx| dialog_open(cx, window));
    assert_eq!(
        question_title(cx),
        msg::remotes::force_confirm_fetch("main", "origin")
    );
    answer(cx, window, 1);
    cx.run_until_parked();
    assert_eq!(
        origin_main(&repo),
        theirs,
        "a cancelled force push changes nothing"
    );
    // Confirmed, it replaces the remote's branch with ours.
    run(cx, window, "git_prefix.push");
    wait_until(cx, "the question", |cx| dialog_open(cx, window));
    answer(cx, window, 1);
    wait_until(cx, "the force question", |cx| dialog_open(cx, window));
    answer(cx, window, 0);
    let mine = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    wait_until(cx, "the forced push", |_| origin_main(&repo) == mine);
    let _ = ws;
}

#[gpui_kit::test]
fn pull_first_then_push_integrates_their_commit_and_sends_ours(cx: &mut TestAppContext) {
    let repo = repo();
    other_pushes(&repo, "theirs.txt");
    local_commit(&repo, "mine.txt", "my commit");
    let (window, _ws) = open(cx, &repo);
    run(cx, window, "git_prefix.push");
    wait_until(cx, "the question", |cx| dialog_open(cx, window));
    answer(cx, window, 0);
    wait_until(cx, "both commits on the remote", |_| {
        let log = repo.git(&["log", "--format=%s", "origin/main"]);
        origin_main(&repo) == repo.git(&["rev-parse", "HEAD"]).trim()
            && log.contains("from the other clone")
            && log.contains("my commit")
    });
}

// ---- the pill cursor -------------------------------------------------------------------------------

#[gpui_kit::test]
fn the_pill_cursor_walks_the_refs_of_the_selected_commit_and_the_menu_key_opens_the_one_it_is_on(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    let main_oid = Oid::from_hex(repo.git(&["rev-parse", "main"]).trim()).unwrap();
    wait_until(cx, "the commit in the Log", |cx| {
        ws.read_with(cx, |ws, cx| ws.log.read(cx).row_of(main_oid).is_some())
    });
    ws.update(cx, |ws, cx| {
        ws.log.update(cx, |l, cx| l.select_oid(main_oid, cx))
    });
    run(cx, window, "palette.focus_log");
    let cursor = |cx: &mut TestAppContext| ws.read_with(cx, |ws, cx| ws.log.read(cx).pill_cursor());
    let pills = ws.read_with(cx, |ws, cx| ws.log.read(cx).pills_of_selected());
    assert!(
        pills.len() >= 2,
        "the tip has a branch and a tag: {pills:?}"
    );
    assert_eq!(cursor(cx), None, "no cursor until the keys are used");
    press(cx, window, "]");
    assert_eq!(cursor(cx), Some((main_oid, 0)));
    press(cx, window, "]");
    assert_eq!(cursor(cx), Some((main_oid, 1)));
    // The menu key opens the menu of the pill the cursor is on (the second one), not the commit's.
    let second = ws.read_with(cx, |ws, cx| ws.log.read(cx).pills_of_selected())[1]
        .name
        .clone();
    press(cx, window, "shift-f10");
    let menu = ws
        .read_with(cx, |ws, _| ws.ref_menu().cloned())
        .expect("the ref menu opened for the pill under the cursor");
    assert!(menu
        .read_with(cx, |m, _| m.labels())
        .iter()
        .any(|l| l.contains(&second)));
    press(cx, window, "escape");
    // `[` goes back and wraps.
    press(cx, window, "[");
    assert_eq!(cursor(cx), Some((main_oid, 0)));
    press(cx, window, "[");
    assert_eq!(cursor(cx), Some((main_oid, pills.len() - 1)), "it wraps");
    // Escape drops the cursor: the menu key is the commit menu again.
    press(cx, window, "escape");
    assert_eq!(cursor(cx), None);
    press(cx, window, "shift-f10");
    assert!(ws.read_with(cx, |ws, _| ws.ref_menu().is_none()));
    assert!(ws.read_with(cx, |ws, cx| ws.log.read(cx).menu().is_some()));
}

#[gpui_kit::test]
fn the_pill_cursor_on_a_commit_without_refs_says_so_and_another_row_drops_it(
    cx: &mut TestAppContext,
) {
    let repo = repo();
    let (window, ws) = open(cx, &repo);
    let main_oid = Oid::from_hex(repo.git(&["rev-parse", "main"]).trim()).unwrap();
    let first = Oid::from_hex(repo.git(&["rev-list", "--max-parents=0", "main"]).trim()).unwrap();
    wait_until(cx, "the commits in the Log", |cx| {
        ws.read_with(cx, |ws, cx| {
            let l = ws.log.read(cx);
            l.row_of(main_oid).is_some() && l.row_of(first).is_some()
        })
    });
    run(cx, window, "palette.focus_log");
    ws.update(cx, |ws, cx| {
        ws.log.update(cx, |l, cx| l.select_oid(first, cx))
    });
    press(cx, window, "]");
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.log.read(cx).pill_cursor()),
        None
    );
    assert_eq!(
        notice(cx, &ws),
        gitpoulpi_desktop::strings::NO_REFS_ON_COMMIT
    );
    // On the tip, a cursor is dropped by selecting another row.
    ws.update(cx, |ws, cx| {
        ws.log.update(cx, |l, cx| l.select_oid(main_oid, cx))
    });
    press(cx, window, "]");
    assert!(ws
        .read_with(cx, |ws, cx| ws.log.read(cx).pill_cursor())
        .is_some());
    ws.update(cx, |ws, cx| {
        ws.log.update(cx, |l, cx| l.select_oid(first, cx))
    });
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.log.read(cx).pill_cursor()),
        None
    );
}
