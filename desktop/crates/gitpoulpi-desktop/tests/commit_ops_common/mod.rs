//! Helpers shared by the commit-operation window tests (`commit_ops_window`, `operation_banner_window`).
#![allow(dead_code)]

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_core::domain::{Direction, Oid};
use gitpoulpi_desktop::app::{self, Overrides, Workspace};
use gitpoulpi_desktop::commands;
use gitpoulpi_desktop::daemon::Backend;
use gitpoulpi_desktop::dialogs;
use gitpoulpi_testkit::{isolate_process_env, FixtureRepo, FixtureRepoBuilder};
use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, Keystroke, Modifiers, TestAppContext};

pub type Opened = (AnyWindowHandle, Entity<Workspace>);

pub const NONE: Modifiers = Modifiers {
    control: false,
    alt: false,
    shift: false,
    platform: false,
    function: false,
};
pub const CTRL: Modifiers = Modifiers {
    control: true,
    ..NONE
};

pub fn git_commit(repo: &FixtureRepo, file: &str, contents: &str, message: &str) {
    std::fs::write(repo.root().join(file), contents).unwrap();
    repo.git(&["add", "--", file]);
    repo.git(&["commit", "-q", "-m", message]);
}

/// main: initial, add a, add b. side (from add b): s1, s2. The checkout is on main.
pub fn repo_with_side() -> FixtureRepo {
    isolate_process_env();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .commit_file("a.txt", "a\n", "add a")
        .unwrap()
        .commit_file("b.txt", "b\n", "add b")
        .unwrap()
        .build();
    let main = repo.git(&["branch", "--show-current"]).trim().to_string();
    repo.git(&["checkout", "-q", "-b", "side"]);
    git_commit(&repo, "s1.txt", "one\n", "side one");
    git_commit(&repo, "s2.txt", "two\n", "side two");
    repo.git(&["checkout", "-q", &main]);
    repo
}

pub fn oid(repo: &FixtureRepo, rev: &str) -> Oid {
    Oid::from_hex(repo.git(&["rev-parse", rev]).trim()).unwrap()
}

pub fn subjects(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%s", "HEAD"])
        .lines()
        .map(str::to_string)
        .collect()
}

pub fn open(cx: &mut TestAppContext, repo: &Path, prefs_dir: &Path) -> Opened {
    let backend = Backend::start(repo).unwrap();
    cx.executor().allow_parking();
    cx.update(|cx| app::init(cx, Some(prefs_dir.to_path_buf()), &Overrides::default()).unwrap());
    cx.update(|cx| app::open_main_window(backend, cx).unwrap())
}

pub fn wait_until(
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
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// Opens the window on `repo` and waits for the Log's rows.
pub fn open_log(cx: &mut TestAppContext, repo: &FixtureRepo, rows: usize) -> Opened {
    let prefs = tempfile::tempdir().unwrap();
    let opened = open(cx, repo.root(), prefs.path());
    std::mem::forget(prefs);
    let ws = opened.1.clone();
    wait_until(cx, "the Log", |cx| {
        ws.read_with(cx, |ws, cx| ws.log.read(cx).row_count()) >= rows
    });
    run(cx, opened.0, "palette.focus_log");
    opened
}

pub fn run(cx: &mut TestAppContext, window: AnyWindowHandle, id: &'static str) {
    cx.update_window(window, |_, w, cx| commands::run_id(id, w, cx))
        .unwrap();
    cx.run_until_parked();
}

pub fn press(cx: &mut TestAppContext, window: AnyWindowHandle, keys: &'static str) {
    for key in keys.split(' ') {
        cx.update_window(window, |_, w, cx| {
            w.dispatch_keystroke(Keystroke::parse(key).unwrap(), cx);
        })
        .unwrap();
        cx.run_until_parked();
    }
}

pub fn select(cx: &mut TestAppContext, ws: &Entity<Workspace>, oid: Oid) {
    ws.update(cx, |ws, cx| {
        ws.log.update(cx, |l, cx| l.select_oid(oid, cx))
    });
    cx.run_until_parked();
    assert_eq!(
        ws.read_with(cx, |ws, cx| ws.log.read(cx).selected_commit()),
        Some(oid)
    );
}

pub fn undo(cx: &mut TestAppContext, window: AnyWindowHandle, ws: &Entity<Workspace>) {
    wait_until(cx, "an entry to undo", |cx| {
        ws.read_with(cx, |ws, _| ws.backend().can_undo_redo(Direction::Undo))
    });
    run(cx, window, "global.undo");
}

pub fn dialog_open(cx: &mut TestAppContext, window: AnyWindowHandle) -> bool {
    cx.update_window(window, |_, w, cx| w.has_active_dialog(cx))
        .unwrap()
}

pub fn answer(cx: &mut TestAppContext, window: AnyWindowHandle, ix: usize) {
    let done = cx
        .update_window(window, |_, w, cx| dialogs::answer_open(ix, w, cx))
        .unwrap();
    assert!(done, "a question is open");
    cx.run_until_parked();
}

pub fn notice(cx: &mut TestAppContext, ws: &Entity<Workspace>) -> String {
    ws.read_with(cx, |ws, cx| {
        ws.notice(cx).map(|n| n.text).unwrap_or_default()
    })
}

pub fn click_row(
    cx: &mut TestAppContext,
    ws: &Entity<Workspace>,
    window: AnyWindowHandle,
    oid: Oid,
    m: Modifiers,
) {
    cx.update_window(window, |_, w, cx| {
        ws.update(cx, |ws, cx| {
            ws.log.update(cx, |l, cx| {
                let ix = l.row_of(oid).expect("the commit is in the Log");
                l.click_row(ix, m, w, cx)
            })
        })
    })
    .unwrap();
    cx.run_until_parked();
}

pub fn menu_labels(cx: &mut TestAppContext, ws: &Entity<Workspace>) -> Option<Vec<String>> {
    ws.read_with(cx, |ws, cx| {
        ws.log.read(cx).menu().map(|m| m.read(cx).labels())
    })
}

pub fn oid_of_head(repo: &FixtureRepo) -> Oid {
    oid(repo, "HEAD")
}
