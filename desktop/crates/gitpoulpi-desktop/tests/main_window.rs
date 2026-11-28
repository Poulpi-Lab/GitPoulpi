//! Headless test: builds the real main window on GPUI's test platform, against a fixture repository
//! and a temporary config directory, and checks what the Log loaded and how its columns were laid out.

use std::path::Path;
use std::time::{Duration, Instant};

use gitpoulpi_desktop::app::{self, Overrides, Workspace};
use gitpoulpi_desktop::daemon::Backend;
use gitpoulpi_desktop::prefs::{Col, DesktopPrefs, MESSAGE_MIN};
use gitpoulpi_testkit::{isolate_process_env, FixtureRepo, FixtureRepoBuilder};
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};

fn fixture() -> FixtureRepo {
    isolate_process_env();
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .commit_file("a.txt", "a", "second")
        .unwrap()
        .commit_file("b.txt", "b", "third")
        .unwrap()
        .build()
}

/// Opens the main window for `repo`; the preferences live in `prefs_dir`.
fn open(
    cx: &mut TestAppContext,
    repo: &Path,
    prefs_dir: &Path,
) -> (AnyWindowHandle, Entity<Workspace>) {
    let backend = Backend::start(repo).unwrap();
    // The daemon runs on its own runtime: let the executor wait for it instead of declaring a deadlock.
    cx.executor().allow_parking();
    cx.update(|cx| app::init(cx, Some(prefs_dir.to_path_buf()), &Overrides::default()).unwrap());
    cx.update(|cx| app::open_main_window(backend, cx).unwrap())
}

/// Lets the app run until the Log has loaded `expected` commits.
fn wait_for_commits(cx: &mut TestAppContext, ws: &Entity<Workspace>, expected: usize) -> usize {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        cx.run_until_parked();
        let loaded = ws.read_with(cx, |ws, cx| ws.log.read(cx).loaded_commits());
        if loaded >= expected || Instant::now() > deadline {
            return loaded;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

#[gpui_kit::test]
fn the_main_window_builds_and_the_log_loads(cx: &mut TestAppContext) {
    let repo = fixture();
    let prefs = tempfile::tempdir().unwrap();
    let (_window, ws) = open(cx, repo.root(), prefs.path());
    assert_eq!(wait_for_commits(cx, &ws, 3), 3);
    let rows = ws.read_with(cx, |ws, cx| ws.log.read(cx).row_count());
    assert_eq!(rows, 3, "three commits, no WIP row on a clean tree");
}

#[gpui_kit::test]
fn a_detached_head_without_any_branch_still_lists_its_history(cx: &mut TestAppContext) {
    let repo = fixture();
    let branch = repo
        .git(&["rev-parse", "--abbrev-ref", "HEAD"])
        .trim()
        .to_string();
    repo.git(&["checkout", "--detach"]);
    repo.git(&["branch", "-D", &branch]);
    let prefs = tempfile::tempdir().unwrap();
    let (_window, ws) = open(cx, repo.root(), prefs.path());
    assert_eq!(wait_for_commits(cx, &ws, 3), 3);
}

#[gpui_kit::test]
fn at_1280_px_every_log_column_keeps_room(cx: &mut TestAppContext) {
    let repo = fixture();
    let prefs = tempfile::tempdir().unwrap();
    let mut stored = DesktopPrefs::default();
    stored.window.width = 1280.0;
    stored.window.height = 800.0;
    stored.save(prefs.path()).unwrap();
    let (window, ws) = open(cx, repo.root(), prefs.path());
    assert_eq!(wait_for_commits(cx, &ws, 3), 3);
    // Draw a few frames so the Log measures its header.
    for _ in 0..3 {
        cx.update_window(window, |_, window, _| window.refresh())
            .ok();
        cx.run_until_parked();
    }
    let (measured, layout) = ws.read_with(cx, |ws, cx| {
        let log = ws.log.read(cx);
        (log.measured_width(), log.column_layout())
    });
    assert!(
        (400.0..1280.0).contains(&measured) && (measured - 900.0).abs() > 0.5,
        "the Log measured its own width: {measured}"
    );
    for col in Col::ALL {
        assert!(
            layout.get(col) >= col.min() - 0.01,
            "{col:?} is {}",
            layout.get(col)
        );
    }
    assert!(
        layout.message_w >= MESSAGE_MIN - 0.01,
        "message is {}",
        layout.message_w
    );
}
