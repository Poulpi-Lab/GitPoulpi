//! The keymap installed from `config.toml` is process-wide: this test has a binary of its own so no
//! other window re-installs the defaults while it runs.

use std::path::Path;

use gitpoulpi_desktop::app::{self, Overrides, Workspace};
use gitpoulpi_desktop::daemon::Backend;
use gitpoulpi_testkit::{isolate_process_env, FixtureRepo, FixtureRepoBuilder};
use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, Keystroke, TestAppContext};

fn fixture() -> FixtureRepo {
    isolate_process_env();
    FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .commit_file("a.txt", "a", "second")
        .unwrap()
        .build()
}

fn open(
    cx: &mut TestAppContext,
    repo: &Path,
    config_dir: &Path,
) -> (AnyWindowHandle, Entity<Workspace>) {
    let backend = Backend::start(repo).unwrap();
    cx.executor().allow_parking();
    cx.update(|cx| app::init(cx, Some(config_dir.to_path_buf()), &Overrides::default()).unwrap());
    let opened = cx.update(|cx| app::open_main_window(backend, cx).unwrap());
    cx.run_until_parked();
    opened
}

fn press(cx: &mut TestAppContext, window: AnyWindowHandle, key: &'static str) {
    cx.update_window(window, |_, w, cx| {
        w.dispatch_keystroke(Keystroke::parse(key).unwrap(), cx);
    })
    .unwrap();
    cx.run_until_parked();
}

fn dialog_open(cx: &mut TestAppContext, window: AnyWindowHandle) -> bool {
    cx.update_window(window, |_, w, cx| w.has_active_dialog(cx))
        .unwrap()
}

#[gpui_kit::test]
fn a_remapped_key_follows_config_toml(cx: &mut TestAppContext) {
    let repo = fixture();
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.toml"),
        "[keybindings.global]\npalette = \"Ctrl+K\"\n",
    )
    .unwrap();
    let (window, _ws) = open(cx, repo.root(), dir.path());
    press(cx, window, "ctrl-p");
    assert!(!dialog_open(cx, window), "the old key no longer opens it");
    press(cx, window, "ctrl-k");
    assert!(dialog_open(cx, window), "the configured key does");
}
