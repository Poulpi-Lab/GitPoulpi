//! Buffer tests of the jobs indicator and panel. The cell is per test thread
//! (`jobs_ui::CELL` under `cfg(test)`), so seeding jobs here cannot show up in
//! another test's footer.

use gitpoulpi_core::domain::{OperationKind, RepoId};
use gitpoulpi_core::NotCancellableReason;
use gitpoulpi_ipc::{JobInfo, JobState};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::Terminal;

use super::*;
use crate::th;

fn job(id: u64, kind: OperationKind, repo: &str) -> JobInfo {
    JobInfo {
        id,
        kind,
        repo: Some(RepoId(repo.to_string())),
        dest: None,
        label: String::new(),
        state: JobState::Running,
        phase: None,
        progress: None,
        started_at: Some(0),
        joiners: 0,
        queued_after: None,
        not_cancellable: None,
        error: None,
    }
}

fn row_text(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width).map(|x| buf[(x, y)].symbol()).collect()
}

fn screen(buf: &Buffer) -> String {
    (0..buf.area.height)
        .map(|y| row_text(buf, y))
        .collect::<Vec<_>>()
        .join("\n")
}

fn footer_with_indicator() -> Buffer {
    let mut buf = Buffer::empty(Rect::new(0, 0, 120, 1));
    paint_indicator(&mut buf, Rect::new(0, 0, 120, 1), 108, 0);
    buf
}

#[test]
fn nothing_is_drawn_without_a_job() {
    let buf = footer_with_indicator();
    assert_eq!(row_text(&buf, 0).trim(), "");
    assert_eq!(width_for(), 0);
    assert!(indicator::hit(100, 0).is_none());
}

#[test]
fn one_job_shows_its_label_progress_and_a_clickable_cancel() {
    let mut j = job(1, OperationKind::Push, "/work/tool");
    j.progress = Some(42);
    changed(j);
    let buf = footer_with_indicator();
    let text = row_text(&buf, 0);
    assert!(text.contains("Push: tool"), "{text:?}");
    assert!(text.contains("42%"), "{text:?}");
    assert!(text.contains("[x]"), "{text:?}");
    let x = text
        .chars()
        .collect::<String>()
        .split("[x]")
        .next()
        .unwrap()
        .chars()
        .count() as u16;
    assert_eq!(
        indicator::hit(x + 1, 0),
        Some(indicator::IndicatorHit::Cancel)
    );
    assert_eq!(
        indicator::hit(x - 5, 0),
        Some(indicator::IndicatorHit::Label)
    );
    assert!(indicator::hit(0, 0).is_none());
}

#[test]
fn several_jobs_are_counted_with_the_oldest_ones_progress() {
    let mut a = job(1, OperationKind::Push, "/work/a");
    a.progress = Some(30);
    changed(a);
    changed(job(2, OperationKind::Fetch, "/work/b"));
    changed(job(3, OperationKind::Pull, "/work/c"));
    let text = row_text(&footer_with_indicator(), 0);
    assert!(text.contains("Jobs: 3"), "{text:?}");
    assert!(text.contains("30%"), "{text:?}");
}

#[test]
fn a_job_that_cannot_be_stopped_draws_the_cancel_disabled_and_says_why() {
    let mut j = job(1, OperationKind::Pull, "/work/tool");
    j.not_cancellable = Some(NotCancellableReason::Integrating);
    changed(j);
    let buf = footer_with_indicator();
    let text = row_text(&buf, 0);
    let x = text
        .chars()
        .collect::<String>()
        .split("[x]")
        .next()
        .unwrap()
        .chars()
        .count() as u16;
    assert_eq!(buf[(x, 0)].fg, th().ui.disabled, "drawn as disabled");
    // A press says why instead of sending anything.
    cancel_oldest();
    assert!(with(|c| c.pending.is_empty()), "nothing was sent");
    let said = take_notice().expect("a reason");
    assert!(said.contains("cannot be stopped safely"), "{said}");
}

#[test]
fn a_cancel_of_a_running_job_is_sent_once_the_link_is_known() {
    let queue = crate::daemon_link::RawQueue::default();
    set_requester(queue.requester());
    changed(job(7, OperationKind::Push, "/work/tool"));
    cancel(7);
    assert_eq!(with(|c| c.pending.len()), 1, "a JobCancel went out");
}

#[test]
fn a_canceled_job_flashes_a_cross_and_a_fast_success_does_not() {
    let mut canceled = job(1, OperationKind::Push, "/work/tool");
    canceled.state = JobState::Canceled;
    changed(canceled);
    let text = row_text(&footer_with_indicator(), 0);
    assert!(text.contains("\u{2717} Push: tool"), "{text:?}");
    assert!(!text.contains("[x]"), "nothing left to cancel: {text:?}");

    removed(1);
    let mut done = job(2, OperationKind::Fetch, "/work/tool");
    done.state = JobState::Done;
    changed(done);
    assert_eq!(row_text(&footer_with_indicator(), 0).trim(), "");
}

fn draw_panel(width: u16, height: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(render_panel).unwrap();
    terminal.backend().buffer().clone()
}

fn rect_of(hit: PanelHit) -> Rect {
    with(|c| c.hits.iter().find(|(_, h)| *h == hit).map(|(r, _)| *r)).expect("a hit rect")
}

#[test]
fn the_panel_lists_each_job_with_its_cancel_and_the_reason_of_a_disabled_one() {
    let mut a = job(1, OperationKind::Push, "/work/alpha");
    a.progress = Some(50);
    changed(a);
    let mut b = job(2, OperationKind::Pull, "/work/beta");
    b.not_cancellable = Some(NotCancellableReason::Integrating);
    changed(b);
    panel::open();
    let buf = draw_panel(110, 30);
    let text = screen(&buf);
    assert!(text.contains("Background jobs"), "{text}");
    assert!(text.contains("Push: alpha"), "{text}");
    assert!(text.contains("Pull: beta"), "{text}");
    assert!(text.contains("50%"), "{text}");
    assert!(text.contains("Integrating changes: cannot be stopped safely"));
    // Each row has its Cancel; the disabled one is a rect too (a press says why).
    let cancel_a = rect_of(PanelHit::Cancel(1));
    assert_eq!(
        (0..cancel_a.width)
            .map(|dx| buf[(cancel_a.x + dx, cancel_a.y)].symbol())
            .collect::<String>()
            .trim(),
        "Cancel"
    );
    assert_eq!(row_text(&buf, cancel_a.y).matches("Cancel").count(), 1);
    // Pressing Cancel on the running row sends a request.
    let queue = crate::daemon_link::RawQueue::default();
    set_requester(queue.requester());
    let click = |r: Rect| crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: r.x + 1,
        row: r.y,
        modifiers: crossterm::event::KeyModifiers::NONE,
    };
    assert!(route_mouse(click(cancel_a)));
    assert_eq!(with(|c| c.pending.len()), 1);
    // The disabled one sends nothing and says why.
    draw_panel(110, 30);
    assert!(route_mouse(click(rect_of(PanelHit::Cancel(2)))));
    assert_eq!(with(|c| c.pending.len()), 1);
    assert!(take_notice().unwrap().contains("cannot be stopped safely"));
}

#[test]
fn the_panel_keys_select_cancel_and_go_to_the_repository() {
    use crossterm::event::{KeyCode, KeyEvent};
    let queue = crate::daemon_link::RawQueue::default();
    set_requester(queue.requester());
    changed(job(1, OperationKind::Push, "/work/alpha"));
    changed(job(2, OperationKind::Fetch, "/work/beta"));
    panel::open();
    draw_panel(110, 30);
    panel_key(KeyEvent::from(KeyCode::Down));
    assert_eq!(with(|c| c.selected), Some(2));
    panel_key(KeyEvent::from(KeyCode::Delete));
    assert_eq!(with(|c| c.pending.len()), 1);
    panel_key(KeyEvent::from(KeyCode::Enter));
    assert_eq!(take_goto(), Some(RepoId("/work/beta".into())));
    assert!(!panel_open(), "Enter closes the panel");
}

#[test]
fn esc_and_a_click_outside_close_the_panel() {
    use crossterm::event::{KeyCode, KeyEvent};
    changed(job(1, OperationKind::Push, "/work/alpha"));
    panel::open();
    draw_panel(110, 30);
    panel_key(KeyEvent::from(KeyCode::Esc));
    assert!(!panel_open());
    panel::open();
    draw_panel(110, 30);
    assert!(route_mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: 0,
        row: 0,
        modifiers: crossterm::event::KeyModifiers::NONE,
    }));
    assert!(!panel_open());
}

#[test]
fn the_palette_offers_a_cancel_per_cancellable_job_and_cancel_all() {
    changed(job(1, OperationKind::Push, "/work/alpha"));
    let mut stuck = job(2, OperationKind::Pull, "/work/beta");
    stuck.not_cancellable = Some(NotCancellableReason::Integrating);
    changed(stuck);
    let rows = crate::palette::jobs_entries();
    assert_eq!(rows.len(), 1, "only the job that can be stopped");
    assert_eq!(rows[0].name(), "Jobs: Cancel Push: alpha");
    assert!(any_cancellable());
}

#[test]
fn a_local_job_draws_the_indicator_and_its_cancel_calls_the_stop() {
    let stopped = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let flag = std::sync::Arc::clone(&stopped);
    let id = local_start(
        "\u{2726} Commit message".into(),
        std::sync::Arc::new(move || flag.store(true, std::sync::atomic::Ordering::SeqCst)),
    );
    assert!(is_local(id));
    let text = row_text(&footer_with_indicator(), 0);
    assert!(text.contains("\u{2726} Commit message"), "{text:?}");
    assert!(text.contains("[x]"), "{text:?}");
    assert!(any_cancellable());
    assert_eq!(wording::label_of_id(id), "\u{2726} Commit message");

    // A click on the [x] stops it, through the same path as a daemon job.
    let rect = with(|c| c.ind_cancel);
    assert!(rect.width > 0);
    assert!(route_mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: rect.x + 1,
        row: rect.y,
        modifiers: crossterm::event::KeyModifiers::NONE,
    }));
    assert!(stopped.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(find(id).unwrap().state, JobState::Cancelling);

    local_finish(id, JobState::Canceled);
    assert!(active().is_empty());
    // A daemon list arriving later does not forget a job of this process.
    let other = local_start("w".into(), std::sync::Arc::new(|| {}));
    seed(Vec::new());
    assert!(find(other).is_some());
    local_finish(other, JobState::Done);
}

#[test]
fn a_running_fetch_job_is_in_the_indicator_and_not_in_the_in_flight_text() {
    use crate::activity::Activity;
    changed(job(1, OperationKind::Fetch, "/r/tool"));
    let activity = Activity::default();
    activity.begin_as("fetching\u{2026}".into(), false, true);
    assert_eq!(
        activity.in_flight_text(std::time::Duration::ZERO),
        None,
        "the breadcrumb stays quiet while a job is on the indicator"
    );
    assert!(row_text(&footer_with_indicator(), 0).contains("Fetch:"));
}

#[test]
fn elapsed_text_counts_seconds_then_minutes_then_hours() {
    use std::time::Duration;
    assert_eq!(indicator::elapsed_text(Duration::from_millis(900)), "");
    assert_eq!(indicator::elapsed_text(Duration::from_secs(12)), "12s");
    assert_eq!(indicator::elapsed_text(Duration::from_secs(65)), "1m 05s");
    assert_eq!(indicator::elapsed_text(Duration::from_secs(3720)), "1h 02m");
}

#[test]
fn a_running_job_shows_its_elapsed_time_before_the_cancel() {
    let mut j = job(1, OperationKind::Fetch, "/work/PMM");
    j.progress = Some(42);
    changed(j);
    with(|c| {
        c.jobs.get_mut(&1).unwrap().first_seen =
            Instant::now() - std::time::Duration::from_secs(65);
    });
    let text = row_text(&footer_with_indicator(), 0);
    assert!(text.contains("Fetch: PMM 42% 1m 05s [x]"), "{text:?}");
}

#[test]
fn a_long_label_is_cut_but_the_elapsed_time_stays() {
    let mut j = job(
        1,
        OperationKind::Fetch,
        "/work/a-very-long-repository-folder-name-here",
    );
    j.progress = Some(42);
    changed(j);
    with(|c| {
        c.jobs.get_mut(&1).unwrap().first_seen =
            Instant::now() - std::time::Duration::from_secs(12);
    });
    let text = row_text(&footer_with_indicator(), 0);
    assert!(text.contains("\u{2026} 12s [x]"), "{text:?}");
}

/// The pointing hand: the panel's Cancel buttons and cross, the
/// status bar's indicator; the arrow on the empty part of the window.
#[test]
fn the_panel_and_the_indicator_show_the_pointing_hand_on_what_a_click_acts_on() {
    changed(job(1, OperationKind::Push, "/work/alpha"));
    let buf = footer_with_indicator();
    let text = row_text(&buf, 0);
    let x = text.chars().position(|c| c == '[').expect("the [x]") as u16;
    assert!(pointer_hand(x + 1, 0), "the indicator's [x]");
    assert!(!pointer_hand(0, 0), "empty status bar");

    panel::open();
    draw_panel(110, 30);
    let cancel = rect_of(PanelHit::Cancel(1));
    assert!(pointer_hand(cancel.x + 1, cancel.y), "the Cancel button");
    let row = rect_of(PanelHit::Row(1));
    assert!(!pointer_hand(row.x + 1, row.y), "a job row keeps the arrow");
    let close = with(|c| c.close);
    assert!(pointer_hand(close.x + 1, close.y), "the cross");
    let window = with(|c| c.window);
    assert!(
        !pointer_hand(window.x + 1, window.bottom() - 2),
        "blank space inside the window"
    );
    assert!(!pointer_hand(0, 0), "outside the window");
}

/// A job row is the arrow, lit as a whole under the pointer; its Cancel
/// button keeps the hand.
#[test]
fn a_job_row_is_the_arrow_and_lights_while_its_cancel_button_keeps_the_hand() {
    use crate::pointer_map::{self, Kind};
    changed(job(1, OperationKind::Push, "/work/alpha"));
    changed(job(2, OperationKind::Pull, "/work/beta"));
    panel::open();
    pointer_map::begin_frame();
    draw_panel(110, 30);
    let row = rect_of(PanelHit::Row(2));
    let cancel = rect_of(PanelHit::Cancel(2));
    assert_eq!(pointer_map::at(row.x + 2, row.y), Some(Kind::Plain));
    assert_eq!(pointer_map::at(cancel.x + 1, cancel.y), Some(Kind::Hand));
    with(|c| c.pointer = Some((row.x + 2, row.y)));
    let lit = draw_panel(110, 30);
    assert_eq!(lit[(row.x + 2, row.y)].bg, crate::th().ui.hover);
    with(|c| c.pointer = None);
}

#[test]
fn the_jobs_panel_moves_resizes_and_arranges() {
    changed(job(1, OperationKind::Push, "/work/alpha"));
    panel::open();
    crate::dialog_kit::frame::testing::check_gestures(true, || {
        draw_panel(160, 50);
    });
    panel::close();
}
