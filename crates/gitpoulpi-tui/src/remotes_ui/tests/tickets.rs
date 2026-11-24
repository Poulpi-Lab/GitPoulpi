//! The diverged-push and auto-stash dialogs continue the action they held
//! under its own ticket (`daemon_link::tickets`): each flow below runs a real
//! tracked action against an embedded daemon, opens the dialog the way the
//! window does, answers it, and reads the ticket's reports.

use super::*;
use crate::daemon_link::ticket_tests::{
    diverged_fixture, harness, push_main, wait_final, wait_needs_user, AUTOSTASH_ENV,
};
use crate::daemon_link::{take_for, PushRejectedNotice, StepOutcome, Ticket, UserWait};

fn answer(
    ui: &mut RemoteUi,
    link: &DaemonLink,
    input_mode: &mut InputMode,
    action: RemoteUiAction,
) {
    let mut clipboard = clipboard::Clipboard::new();
    let mut last_message = None;
    let mut hidden = std::collections::HashSet::new();
    let solo = std::collections::HashSet::new();
    apply_remote_ui_action(
        action,
        RemoteUiCtx {
            ui,
            refs: &[],
            link,
            clipboard: &mut clipboard,
            input_mode,
            last_message: &mut last_message,
            hidden_from_graph: &mut hidden,
            solo_refs: &solo,
        },
    );
}

/// A push that the remote refuses, its notice, and the dialog open on it.
fn rejected_push(
    name: &str,
) -> (
    crate::daemon_link::ticket_tests::Harness,
    gitpoulpi_testkit::FixtureRepo,
    RemoteUi,
    Ticket,
    PushRejectedNotice,
) {
    let fixture = diverged_fixture();
    let h = harness(name, &fixture);
    let ticket = h.link.submit_tracked(push_main());
    let first = wait_needs_user(ticket);
    assert_eq!(first, vec![StepOutcome::NeedsUser(UserWait::PushRejected)]);
    let notice = h
        .link
        .push_rejected_rx
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("the refusal arrives typed");
    assert_eq!(notice.ticket, Some(ticket));
    let mut ui = RemoteUi::load(fixture.root());
    ui.on_push_rejected(&notice.rejection, notice.ticket, 0, 1);
    assert!(ui.menu.is_some(), "the diverged dialog is open");
    (h, fixture, ui, ticket, notice)
}

/// The owner's decision: "Pull then push" continues under the SAME ticket,
/// and the ticket's final outcome is the resolved push's.
#[test]
fn pull_then_push_continues_under_the_same_ticket() {
    let (h, fixture, mut ui, ticket, notice) = rejected_push("ticket-pull-push");
    let mut mode = InputMode::Normal;
    ui.menu = None; // the menu closes on the answer, as it does in the window
    answer(
        &mut ui,
        &h.link,
        &mut mode,
        RemoteUiAction::PullThenPush {
            remote: notice.rejection.remote.clone(),
            branch: notice.rejection.branch.clone(),
        },
    );
    assert_eq!(wait_final(ticket), vec![StepOutcome::Ok(None)]);
    assert_eq!(
        fixture.git(&["rev-parse", "main"]),
        fixture.git_in_remote(&["rev-parse", "main"]),
        "the resolved push really went through"
    );
    // Nothing is left to cancel when the window goes away.
    drop(ui);
    assert!(take_for(ticket).is_empty());
}

#[test]
fn closing_the_diverged_dialog_cancels_the_ticket() {
    let (_h, _fixture, mut ui, ticket, _notice) = rejected_push("ticket-diverged-closed");
    // Still open: nothing yet.
    ui.reap_ticket(None);
    assert!(take_for(ticket).is_empty());
    // Esc / a click away: the menu is gone.
    ui.menu = None;
    ui.reap_ticket(None);
    assert_eq!(take_for(ticket), vec![StepOutcome::Canceled]);
    // Once only.
    ui.reap_ticket(None);
    assert!(take_for(ticket).is_empty());
}

#[test]
fn the_cancel_row_cancels_the_ticket() {
    let (h, _fixture, mut ui, ticket, _notice) = rejected_push("ticket-diverged-cancel");
    let mut mode = InputMode::Normal;
    ui.menu = None;
    answer(&mut ui, &h.link, &mut mode, RemoteUiAction::Cancel);
    assert_eq!(take_for(ticket), vec![StepOutcome::Canceled]);
}

/// Force push goes through its confirmation: the ticket rides in the
/// confirmation, and dismissing it cancels.
#[test]
fn the_force_push_confirmation_carries_the_ticket_and_dismissing_it_cancels() {
    let (h, _fixture, mut ui, ticket, notice) = rejected_push("ticket-diverged-force");
    let mut mode = InputMode::Normal;
    ui.menu = None;
    answer(
        &mut ui,
        &h.link,
        &mut mode,
        RemoteUiAction::ForcePush {
            remote: notice.rejection.remote.clone(),
            branch: notice.rejection.branch.clone(),
            after_fetch: true,
        },
    );
    let carried = match &mode {
        InputMode::Confirm {
            action: ConfirmAction::Destructive(pending),
            ..
        } => pending.ticket(),
        _ => panic!("the force push asks first"),
    };
    assert_eq!(carried, Some(ticket));
    // While the confirmation is up the ticket stays.
    ui.reap_ticket(carried);
    assert!(take_for(ticket).is_empty());
    // Esc on the confirmation: it is gone, and so is the ticket.
    ui.reap_ticket(None);
    assert_eq!(take_for(ticket), vec![StepOutcome::Canceled]);
}

/// Confirming goes out under the ticket (`confirm_now` submits
/// `pending.ticket()`), and the ticket's outcome is the force push's.
#[test]
fn a_confirmed_force_push_reports_under_the_ticket() {
    let (h, fixture, mut ui, ticket, notice) = rejected_push("ticket-diverged-force-ok");
    let mut mode = InputMode::Normal;
    ui.menu = None;
    answer(
        &mut ui,
        &h.link,
        &mut mode,
        RemoteUiAction::ForcePush {
            remote: notice.rejection.remote.clone(),
            branch: notice.rejection.branch.clone(),
            after_fetch: true,
        },
    );
    let InputMode::Confirm {
        action: ConfirmAction::Destructive(pending),
        ..
    } = mode
    else {
        panic!("the force push asks first");
    };
    let carried = pending.ticket();
    h.link.submit_as(carried, pending.into_action());
    // The held ticket is handed over: reaping must not cancel it.
    ui.reap_ticket(None);
    assert_eq!(wait_final(ticket), vec![StepOutcome::Ok(None)]);
    assert_eq!(
        fixture.git(&["rev-parse", "main"]),
        fixture.git_in_remote(&["rev-parse", "main"]),
    );
}

/// A tab switch drops the window and its dialog with it.
#[test]
fn a_window_rebuild_with_the_dialog_open_cancels_the_ticket() {
    let (_h, _fixture, ui, ticket, _notice) = rejected_push("ticket-diverged-rebuild");
    drop(ui);
    assert_eq!(take_for(ticket), vec![StepOutcome::Canceled]);
}

/// A refusal with no dialog (stale lease, protected branch) is the answer:
/// the ticket fails with the status line's text, it does not hang.
#[test]
fn a_refusal_without_a_dialog_fails_the_ticket() {
    let tmp = tempfile::tempdir().unwrap();
    let mut ui = RemoteUi::load(tmp.path());
    let ticket = crate::daemon_link::issue();
    crate::daemon_link::begin(ticket, 1, 0);
    crate::daemon_link::report_ticket(Some(ticket), StepOutcome::NeedsUser(UserWait::PushRejected));
    let rejection = PushRejection {
        remote: "origin".into(),
        branch: "main".into(),
        reason: PushRejectReason::StaleLease,
        detail: None,
    };
    let line = ui.on_push_rejected(&rejection, Some(ticket), 0, 1);
    assert!(ui.menu.is_none());
    assert_eq!(
        take_for(ticket),
        vec![
            StepOutcome::NeedsUser(UserWait::PushRejected),
            StepOutcome::Failed { text: line }
        ]
    );
}

/// `general.auto_stash = ask`: the refused checkout waits for the dialog,
/// "Stash changes and continue" resends it under the same ticket.
#[test]
fn auto_stash_continue_reports_under_the_same_ticket() {
    let _env = AUTOSTASH_ENV.blocking_lock();
    std::env::set_var("GITPOULPI_GENERAL_AUTO_STASH", "ask");
    let (h, fixture, mut ui, ticket, mut mode) = blocked_checkout("ticket-autostash-ok");
    answer(
        &mut ui,
        &h.link,
        &mut mode,
        RemoteUiAction::AutoStashContinue,
    );
    assert_eq!(wait_final(ticket), vec![StepOutcome::Ok(None)]);
    assert_eq!(
        fixture.git(&["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "other"
    );
    std::env::remove_var("GITPOULPI_GENERAL_AUTO_STASH");
}

#[test]
fn auto_stash_cancel_reports_canceled() {
    let _env = AUTOSTASH_ENV.blocking_lock();
    std::env::set_var("GITPOULPI_GENERAL_AUTO_STASH", "ask");
    let (h, _fixture, mut ui, ticket, mut mode) = blocked_checkout("ticket-autostash-cancel");
    ui.menu = None;
    answer(&mut ui, &h.link, &mut mode, RemoteUiAction::Cancel);
    assert_eq!(take_for(ticket), vec![StepOutcome::Canceled]);
    std::env::remove_var("GITPOULPI_GENERAL_AUTO_STASH");
}

#[test]
fn auto_stash_dialog_closed_without_an_answer_reports_canceled() {
    let _env = AUTOSTASH_ENV.blocking_lock();
    std::env::set_var("GITPOULPI_GENERAL_AUTO_STASH", "ask");
    let (_h, _fixture, mut ui, ticket, _mode) = blocked_checkout("ticket-autostash-esc");
    ui.menu = None;
    ui.reap_ticket(None);
    assert_eq!(take_for(ticket), vec![StepOutcome::Canceled]);
    std::env::remove_var("GITPOULPI_GENERAL_AUTO_STASH");
}

/// A tracked checkout that local changes block, and the dialog open on it.
fn blocked_checkout(
    name: &str,
) -> (
    crate::daemon_link::ticket_tests::Harness,
    gitpoulpi_testkit::FixtureRepo,
    RemoteUi,
    Ticket,
    InputMode,
) {
    let fixture = gitpoulpi_testkit::FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "1\n2\n3\n4\n5\n", "base")
        .unwrap()
        .branch("other")
        .unwrap()
        .checkout("other")
        .unwrap()
        .commit_file("a.txt", "1 other\n2\n3\n4\n5\n", "other edit")
        .unwrap()
        .checkout("main")
        .unwrap()
        .write_worktree("a.txt", b"1\n2\n3\n4\n5 local\n")
        .unwrap()
        .build();
    let h = harness(name, &fixture);
    let ticket = h
        .link
        .submit_tracked(Action::CheckoutBranch("other".into()));
    assert_eq!(
        wait_needs_user(ticket),
        vec![StepOutcome::NeedsUser(UserWait::AutoStash)]
    );
    let needed = h
        .link
        .autostash_rx
        .recv_timeout(std::time::Duration::from_secs(30))
        .expect("the dialog's request");
    assert_eq!(needed.ticket, Some(ticket));
    let mut ui = RemoteUi::load(fixture.root());
    ui.on_autostash_needed(needed, 0, 1);
    assert!(ui.menu.is_some());
    (h, fixture, ui, ticket, InputMode::Normal)
}
