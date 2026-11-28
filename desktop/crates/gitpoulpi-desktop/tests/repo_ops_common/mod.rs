//! Helpers shared by the worktree, remote, LFS and file window tests (`repo_ops_window`, `repo_files_window`),
//! on top of `refs_ops_common`.
#![allow(dead_code)]

use gitpoulpi_desktop::app::{self, Overrides};
use gitpoulpi_desktop::commands::refs_ops::RefOp;
use gitpoulpi_desktop::commands::repo_ops::RepoOp;
use gitpoulpi_desktop::daemon::Backend;
use gitpoulpi_desktop::panels::refs::SectionKind;
use gitpoulpi_desktop::ref_target::RefTarget;
use gitpoulpi_testkit::FixtureRepo;
use gpui_kit::{AppContext as _, TestAppContext};

use crate::refs_ops_common::{run_op, wait_until, Opened};

/// Opens the window on a fixture that has no `topic` branch, once the Refs sidebar lists its branch.
pub fn open_plain(cx: &mut TestAppContext, repo: &FixtureRepo) -> Opened {
    let prefs = tempfile::tempdir().unwrap();
    let backend = Backend::start(repo.root()).unwrap();
    cx.executor().allow_parking();
    cx.update(|cx| {
        app::init(
            cx,
            Some(gitpoulpi_config::ConfigPaths::for_dir(prefs.path())),
            &Overrides::default(),
        )
        .unwrap()
    });
    std::mem::forget(prefs);
    let opened = cx.update(|cx| app::open_main_window(backend, cx).unwrap());
    let ws = opened.1.clone();
    wait_until(cx, "the branches", |cx| {
        ws.read_with(cx, |ws, cx| {
            !ws.refs.read(cx).row_labels(SectionKind::Local).is_empty()
        })
    });
    opened
}

pub fn open_menu(
    cx: &mut TestAppContext,
    window: gpui_kit::AnyWindowHandle,
    ws: &gpui_kit::Entity<gitpoulpi_desktop::app::Workspace>,
    target: RefTarget,
) {
    cx.update_window(window, |_, w, cx| {
        ws.update(cx, |ws, cx| {
            ws.open_ref_menu(
                target,
                gpui_kit::Point::new(gpui_kit::px(40.0), gpui_kit::px(40.0)),
                w,
                cx,
            )
        })
    })
    .unwrap();
    cx.run_until_parked();
}

/// Waits for the form a command opens once it has read what it needs from the daemon.
pub fn wait_form(cx: &mut TestAppContext) {
    wait_until(cx, "a form", |cx| {
        cx.update(|cx| gitpoulpi_desktop::form_dialog::current(cx))
            .is_some()
    });
}

pub fn run_repo(
    cx: &mut TestAppContext,
    window: gpui_kit::AnyWindowHandle,
    ws: &gpui_kit::Entity<gitpoulpi_desktop::app::Workspace>,
    op: RepoOp,
    target: Option<RefTarget>,
) {
    run_op(cx, window, ws, RefOp::Repo(op), target);
}

pub fn menu_labels(
    cx: &mut TestAppContext,
    ws: &gpui_kit::Entity<gitpoulpi_desktop::app::Workspace>,
) -> Vec<String> {
    ws.read_with(cx, |ws, cx| {
        ws.ref_menu()
            .map(|m| m.read(cx).labels())
            .unwrap_or_default()
    })
}
