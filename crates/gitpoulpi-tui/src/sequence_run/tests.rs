use super::machine::{ExecResult, PauseKind, Run, RunEvent, StepState};
use super::strip::{self, StripButton};
use super::*;
use gitpoulpi_config::sequences::{SequenceDef, StepSpec};
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::path::Path;

fn run(name: &str, steps: &[&str]) -> Run {
    let def = SequenceDef {
        id: name.to_lowercase(),
        name: name.into(),
        steps: steps.iter().map(|s| StepSpec::new(s)).collect(),
        ..SequenceDef::default()
    };
    Run::new(&def, Path::new("/repo"), Instant::now(), |id| {
        (None, id.to_string())
    })
}

#[test]
fn a_second_start_is_refused_with_the_running_one_and_its_step() {
    assert!(!active());
    begin(run("Backup", &["stage_all", "push"])).unwrap();
    assert!(active());
    let why = begin(run("Other", &["fetch"])).unwrap_err();
    assert!(
        why.contains("Backup") && why.contains("running") && why.contains("1/2"),
        "{why}"
    );
    // Also while the glue has the run out for a frame.
    let taken = take().expect("the run");
    assert!(active());
    let why = begin(run("Other", &["fetch"])).unwrap_err();
    assert!(why.contains("Backup"), "{why}");
    put_back(taken);
    // A canceled run leaves; another can start.
    let mut taken = take().unwrap();
    taken.on(RunEvent::Cancel);
    put_back(taken);
    assert!(!visible());
    begin(run("Other", &["fetch"])).unwrap();
}

#[test]
fn a_failed_run_stays_on_screen_until_dismissed_and_does_not_block_a_new_one() {
    let mut failed = run("Backup", &["push"]);
    failed.state = StepState::Failed {
        step: 0,
        text: "boom".into(),
    };
    {
        let mut slot_run = failed;
        slot_run.job_id = None;
        with(|s| s.run = Some(slot_run));
    }
    assert!(visible());
    assert!(!active());
    begin(run("Next", &["fetch"])).unwrap();
    assert!(active());
}

#[test]
fn requests_are_taken_once() {
    request_cancel();
    request_continue();
    let first = take_requests();
    assert!(first.cancel && first.cont && !first.go_there);
    assert_eq!(take_requests(), Requests::default());
}

fn paint(
    run: &Run,
    width: u16,
    hovered: Option<StripButton>,
) -> (Vec<(Rect, StripButton)>, String) {
    let mut terminal = Terminal::new(TestBackend::new(width, 1)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|f| hits = strip::render(f, f.area(), run, hovered))
        .unwrap();
    let buf = terminal.backend().buffer();
    let text: String = (0..width).map(|x| buf[(x, 0)].symbol()).collect();
    (hits, text)
}

#[test]
fn the_strip_has_a_close_cross_and_real_button_rects() {
    let mut r = run("Backup", &["ai_generate_commit_message", "commit"]);
    r.state = StepState::Paused(PauseKind::ReviewMessage);
    let (hits, text) = paint(&r, 100, None);
    assert!(text.contains("Backup") && text.contains("1/2"), "{text:?}");
    assert!(text.contains("Review the message"), "{text:?}");
    let button = |b| hits.iter().find(|(_, k)| *k == b).map(|(r, _)| *r);
    let cont = button(StripButton::Continue).expect("a Continue button");
    let close = button(StripButton::Close).expect("a close cross");
    assert_eq!(close.width, 3);
    assert!(cont.right() <= close.x, "{cont:?} {close:?}");
    let cell = |rect: Rect| -> String {
        let mut t = Terminal::new(TestBackend::new(100, 1)).unwrap();
        t.draw(|f| {
            strip::render(f, f.area(), &r, None);
        })
        .unwrap();
        (rect.x..rect.right())
            .map(|x| t.backend().buffer()[(x, 0)].symbol().to_string())
            .collect()
    };
    assert!(cell(cont).contains("Continue"));
    assert!(cell(close).contains('\u{2715}'));
}

#[test]
fn the_strip_offers_go_there_and_details_where_they_apply() {
    let mut r = run("Backup", &["push"]);
    r.state = StepState::WaitingForWindow;
    let (hits, text) = paint(&r, 100, None);
    assert!(hits.iter().any(|(_, b)| *b == StripButton::GoThere));
    assert!(!hits.iter().any(|(_, b)| *b == StripButton::Continue));
    assert!(text.contains("waiting for tab"), "{text:?}");
    r.state = StepState::Failed {
        step: 0,
        text: "no remote".into(),
    };
    let (hits, text) = paint(&r, 100, None);
    assert!(hits.iter().any(|(_, b)| *b == StripButton::Details));
    assert!(hits.iter().any(|(_, b)| *b == StripButton::Close));
    assert!(
        text.contains("stopped at step 1/1") && text.contains("no remote"),
        "{text:?}"
    );
}

#[test]
fn the_strip_stays_on_one_row_at_narrow_widths() {
    let mut r = run("Backup", &["push"]);
    r.state = StepState::Paused(PauseKind::BeforeStep);
    for width in [40u16, 60, 80] {
        let (hits, _) = paint(&r, width, None);
        assert!(
            hits.iter().any(|(_, b)| *b == StripButton::Close),
            "{width}"
        );
        assert!(
            hits.iter().all(|(rect, _)| rect.right() <= width),
            "{width}"
        );
    }
}

#[test]
fn executed_results_are_part_of_the_machine() {
    // The glue reports through Executed; a smoke check that the type is wired.
    let mut r = run("Backup", &["fetch"]);
    let _ = r.on(RunEvent::Executed(ExecResult::Instant));
}
