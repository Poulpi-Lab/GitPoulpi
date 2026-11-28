//! Helpers shared by the branch, tag, stash and ref-menu window tests (`refs_branches_window`,
//! `refs_tags_stashes_window`, `refs_menu_window`): a fixture repository with a local bare repository as its
//! remote, the window, commands, questions and forms.
#![allow(dead_code)]

use std::time::{Duration, Instant};

use gitpoulpi_core::domain::Direction;
use gitpoulpi_desktop::app::{self, Overrides, Workspace};
use gitpoulpi_desktop::commands::refs_ops::RefOp;
use gitpoulpi_desktop::commands::{self};
use gitpoulpi_desktop::daemon::Backend;
use gitpoulpi_desktop::form_dialog::{self, FormView};
use gitpoulpi_desktop::ref_target::RefTarget;
use gitpoulpi_desktop::{dialogs, panels::refs::SectionKind};
use gitpoulpi_testkit::{isolate_process_env, FixtureRepo, FixtureRepoBuilder};
use gpui_kit::component::WindowExt as _;
use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, Keystroke, TestAppContext};

pub fn repo() -> FixtureRepo {
    isolate_process_env();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_initial()
        .unwrap()
        .commit_file("a.txt", "a\n", "add a")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("HEAD:refs/heads/main")
        .unwrap()
        .track("main", "origin")
        .unwrap()
        .branch("topic")
        .unwrap()
        .commit_file("t.txt", "t\n", "topic work")
        .unwrap()
        .push("topic")
        .unwrap()
        .branch("remote-only")
        .unwrap()
        .push("remote-only")
        .unwrap()
        .checkout("main")
        .unwrap()
        .tag("v1")
        .unwrap()
        .build();
    repo.git(&["branch", "-D", "remote-only"]);
    repo
}

pub type Opened = (AnyWindowHandle, Entity<Workspace>);

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

/// Opens the window and waits until the Refs sidebar lists the fixture's branches.
pub fn open(cx: &mut TestAppContext, repo: &FixtureRepo) -> Opened {
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
            let labels = ws.refs.read(cx).row_labels(SectionKind::Local);
            labels.iter().any(|l| l == "topic")
                && !ws.refs.read(cx).row_labels(SectionKind::Remote).is_empty()
        })
    });
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

pub fn run_op(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    ws: &Entity<Workspace>,
    op: RefOp,
    target: Option<RefTarget>,
) {
    cx.update_window(window, |_, w, cx| {
        ws.update(cx, |ws, cx| ws.run_ref_op(op, target, w, cx))
    })
    .unwrap();
    cx.run_until_parked();
}

pub fn select(cx: &mut TestAppContext, ws: &Entity<Workspace>, target: RefTarget) {
    let found = ws.update(cx, |ws, cx| {
        ws.refs.update(cx, |r, cx| r.select_target(&target, cx))
    });
    assert!(found, "{target:?} is listed");
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

pub fn question_title(cx: &mut TestAppContext) -> String {
    cx.update(|cx| dialogs::current(cx))
        .expect("a question")
        .read_with(cx, |q, _| q.spec().title.clone())
}

pub fn form(cx: &mut TestAppContext) -> Entity<FormView> {
    cx.update(|cx| form_dialog::current(cx)).expect("a form")
}

pub fn type_into(cx: &mut TestAppContext, window: AnyWindowHandle, ix: usize, text: &str) {
    let f = form(cx);
    cx.update_window(window, |_, w, cx| {
        f.update(cx, |f, cx| f.set_text(ix, text, w, cx))
    })
    .unwrap();
    cx.run_until_parked();
}

pub fn submit(cx: &mut TestAppContext, window: AnyWindowHandle) {
    let f = form(cx);
    cx.update_window(window, |_, w, cx| f.update(cx, |f, cx| f.submit(w, cx)))
        .unwrap();
    cx.run_until_parked();
}

pub fn form_error(cx: &mut TestAppContext) -> Option<String> {
    form(cx).read_with(cx, |f, _| f.error().map(str::to_string))
}

pub fn notice(cx: &mut TestAppContext, ws: &Entity<Workspace>) -> String {
    ws.read_with(cx, |ws, cx| {
        ws.notice(cx).map(|n| n.text).unwrap_or_default()
    })
}

pub fn head(repo: &FixtureRepo) -> String {
    repo.git(&["branch", "--show-current"]).trim().to_string()
}

pub fn branches(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["branch", "--format=%(refname:short)"])
        .lines()
        .map(str::to_string)
        .collect()
}

pub fn has_branch(repo: &FixtureRepo, name: &str) -> bool {
    branches(repo).iter().any(|b| b == name)
}

pub fn remote_has(repo: &FixtureRepo, refname: &str) -> bool {
    !repo
        .git(&["ls-remote", "origin", refname])
        .trim()
        .is_empty()
}

pub fn is_ancestor(repo: &FixtureRepo, ancestor: &str, of: &str) -> bool {
    repo.try_git(&["merge-base", "--is-ancestor", ancestor, of])
        .is_some()
}

pub fn local(name: &str) -> RefTarget {
    RefTarget::Local(name.into())
}

// -- the mouse --------------------------------------------------------------------------------------

pub fn frames(cx: &mut TestAppContext, window: AnyWindowHandle, n: usize) {
    for _ in 0..n {
        cx.update_window(window, |_, window, _| window.refresh())
            .ok();
        cx.run_until_parked();
    }
}

pub fn sim<E: gpui_kit::InputEvent>(cx: &mut TestAppContext, window: AnyWindowHandle, event: E) {
    let mut v = gpui_kit::VisualTestContext::from_window(window, cx);
    v.simulate_event(event);
    v.run_until_parked();
}

/// A point inside the selected Refs row, from where it was last drawn.
pub fn row_point(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    ws: &Entity<Workspace>,
    target: RefTarget,
) -> gpui_kit::Point<gpui_kit::Pixels> {
    select(cx, ws, target);
    frames(cx, window, 3);
    let p = ws
        .read_with(cx, |ws, cx| ws.refs.read(cx).selected_row_origin())
        .expect("the selected row was drawn");
    gpui_kit::Point::new(p.x, p.y - gpui_kit::px(10.0))
}

pub fn dirty(repo: &FixtureRepo) {
    std::fs::write(repo.root().join("a.txt"), "changed\n").unwrap();
}

pub fn stashes(repo: &FixtureRepo) -> usize {
    repo.git(&["stash", "list"]).lines().count()
}
