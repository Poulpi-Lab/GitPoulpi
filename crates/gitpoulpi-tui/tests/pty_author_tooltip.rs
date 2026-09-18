//! The commit-detail pane's author name: resting the pointer on it shows the
//! author's email in a tooltip; moving off it hides the tooltip.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);
const EMAIL: &str = "testkit@gitpoulpi.local";

#[test]
fn hovering_the_author_name_in_the_detail_pane_shows_the_email() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("the only commit")
        .unwrap()
        .build();
    let author = repo.git(&["log", "-1", "--format=%an"]).trim().to_string();
    let (mut term, _cfg) = common::launch(repo.root())
        .session("pty-author-tooltip")
        .size(200, 50)
        .ready("the only commit")
        .mouse_capture()
        .start();
    let (col, row) = term.wait_for_text("the only commit", STEP).unwrap();
    term.click(col + 1, row).unwrap();
    // The detail pane's author line also carries `parent:` / `(root)`; the
    // Log's author column does not.
    let detail = || {
        let screen = term.screen();
        screen.find_all(&author).into_iter().find(|&(_, r)| {
            let line = screen.line(r);
            line.contains("parent:") || line.contains("(root)")
        })
    };
    let deadline = std::time::Instant::now() + STEP;
    let (name_col, name_row) = loop {
        if let Some(at) = detail() {
            break at;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "no author line in the detail pane\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(!term.text().contains(EMAIL), "no email before the hover");
    term.move_to(name_col + 1, name_row).unwrap();
    term.move_to(name_col + 2, name_row).unwrap();
    term.wait_for("the email tooltip", STEP, |s| s.contains(EMAIL))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.move_to(0, 0).unwrap();
    term.wait_for_text_to_go(EMAIL, STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let _ = term.stop();
}
