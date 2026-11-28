//! Helpers shared by the undo-history and Activity Log window tests (`activity_window`,
//! `activity_window_panels`). They sit on `refs_ops_common` (a fixture repository with a local bare
//! repository as its remote, the window, questions and forms).
#![allow(dead_code)]

use gitpoulpi_core::domain::Oid;
use gitpoulpi_desktop::app::Workspace;
use gitpoulpi_desktop::panels::activity::safety::{self, SafetyView};
use gitpoulpi_desktop::panels::activity::safety_model::Tab;
use gitpoulpi_desktop::panels::activity::{activity_log, ActivityView};
use gitpoulpi_link::undo_label::entry_text;
use gitpoulpi_link::Action;
use gitpoulpi_testkit::FixtureRepo;
use gpui_kit::{AnyWindowHandle, Entity, TestAppContext};

use crate::refs_ops_common::*;

pub fn head_oid(repo: &FixtureRepo) -> Oid {
    Oid::from_hex(repo.git(&["rev-parse", "HEAD"]).trim()).unwrap()
}

/// A branch created through the link: one undo entry.
pub fn new_branch(cx: &mut TestAppContext, ws: &Entity<Workspace>, repo: &FixtureRepo, name: &str) {
    let at = head_oid(repo);
    ws.read_with(cx, |ws, _| {
        ws.backend().submit(Action::BranchCreate {
            name: name.to_string(),
            at,
        })
    });
    wait_until(cx, "the branch", |_| has_branch(repo, name));
}

/// The open undo-safety panel.
pub fn safety_view(cx: &mut TestAppContext) -> Entity<SafetyView> {
    cx.update(|cx| safety::current(cx))
        .expect("the undo-safety panel is open")
}

pub fn activity_view(cx: &mut TestAppContext) -> Entity<ActivityView> {
    cx.update(|cx| activity_log::current(cx))
        .expect("the Activity Log is open")
}

/// Opens the panel by its command and waits until its view has loaded.
pub fn open_safety(
    cx: &mut TestAppContext,
    window: AnyWindowHandle,
    id: &'static str,
    tab: Tab,
) -> Entity<SafetyView> {
    run(cx, window, id);
    let view = safety_view(cx);
    wait_until(cx, "the view to load", |cx| {
        view.read_with(cx, |v, _| {
            v.tab == tab
                && match tab {
                    Tab::History => v.history.loaded,
                    Tab::Reflog => v.reflog.loaded,
                    Tab::Lost => v.lost.loaded,
                }
        })
    });
    view
}

/// The history rows' texts, newest first, redo rows after.
pub fn history_texts(cx: &mut TestAppContext, view: &Entity<SafetyView>) -> Vec<String> {
    view.read_with(cx, |v, _| {
        v.history
            .rows
            .iter()
            .map(|r| entry_text(&r.entry))
            .collect()
    })
}

/// Runs a panel action and waits for the work it started to end.
pub fn act(
    cx: &mut TestAppContext,
    view: &Entity<SafetyView>,
    act: gitpoulpi_desktop::panels::activity::safety_model::Act,
) {
    view.update(cx, |v, cx| v.run(act, cx));
    cx.run_until_parked();
}

/// Waits for the panel to finish what it started.
pub fn settle(cx: &mut TestAppContext, view: &Entity<SafetyView>) {
    wait_until(cx, "the panel to be idle", |cx| {
        view.read_with(cx, |v, _| !v.is_busy())
    });
}

/// A tag created through the link: one undo entry, HEAD stays where it is.
pub fn new_tag(cx: &mut TestAppContext, ws: &Entity<Workspace>, repo: &FixtureRepo, name: &str) {
    let at = head_oid(repo);
    ws.read_with(cx, |ws, _| {
        ws.backend().submit(Action::TagCreate {
            name: name.to_string(),
            at,
        })
    });
    wait_until(cx, "the tag", |_| has_tag(repo, name));
}

pub fn has_tag(repo: &FixtureRepo, name: &str) -> bool {
    repo.git(&["tag", "--list", name]).trim() == name
}
