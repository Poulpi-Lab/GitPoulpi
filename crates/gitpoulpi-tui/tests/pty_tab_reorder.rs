//! Repository tabs reorder by dragging (tab bar and tab picker rows), by the
//! picker's Alt+Up/Down and by the palette's `Tab: Move Left/Right`. The
//! order is the session file's order, so it survives a restart; Home, the
//! workspace chip and the `+` are not tabs and never move.

mod common;

use std::path::{Path, PathBuf};
use std::time::Duration;

use gitpoulpi_session::GlobalSession;
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);
const SESSION: &str = "pty-tab-reorder";

fn repo(name: &str, file: &str, subject: &str) -> (gitpoulpi_testkit::FixtureRepo, PathBuf) {
    let fx = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file(file, "x\n", subject)
        .unwrap()
        .build();
    let root = fx.root().parent().unwrap().join(name);
    std::fs::rename(fx.root(), &root).unwrap();
    (fx, root)
}

struct Three {
    _keep: Vec<gitpoulpi_testkit::FixtureRepo>,
    orbit: PathBuf,
    moonbase: PathBuf,
    comet: PathBuf,
    config: tempfile::TempDir,
}

/// Three repositories, opened as tabs in the order orbit, moonbase, comet.
fn three() -> Three {
    let (a, orbit) = repo("orbit", "a.txt", "alpha-only-commit");
    let (b, moonbase) = repo("moonbase", "b.txt", "bravo-only-commit");
    let (c, comet) = repo("comet", "c.txt", "charlie-only-commit");
    let config = tempfile::tempdir().unwrap();
    let file = session_file(config.path());
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let mut s = GlobalSession::default();
    s.open(&moonbase);
    s.open(&comet);
    s.open(&orbit);
    s.activate(2);
    // orbit first, the others after it.
    s.tabs.rotate_right(1);
    s.save(&file).unwrap();
    Three {
        _keep: vec![a, b, c],
        orbit,
        moonbase,
        comet,
        config,
    }
}

fn session_file(config: &Path) -> PathBuf {
    config.join("sessions").join(format!("{SESSION}.toml"))
}

fn order(config: &Path) -> Vec<String> {
    GlobalSession::load(&session_file(config))
        .unwrap()
        .tabs
        .iter()
        .map(|t| {
            t.repo_path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect()
}

fn start(t: &Three) -> Terminal {
    common::launch(&t.orbit)
        .session(SESSION)
        .config_dir(t.config.path())
        .size(200, 50)
        .no_exit_pause()
        .ready("alpha-only-commit")
        .mouse_capture()
        .spawn()
}

fn bar(term: &Terminal) -> String {
    term.text().lines().nth(1).unwrap_or("").to_string()
}

/// Where `name` is on the tab bar (row 1; the title bar above also names
/// the repository).
fn tab_pos(term: &Terminal, name: &str) -> (u16, u16) {
    term.screen()
        .find_all(name)
        .into_iter()
        .find(|(_, r)| *r == 1)
        .unwrap_or_else(|| panic!("no {name} tab\n{}", term.text()))
}

/// The names in the bar, left to right.
fn bar_order(s: &str) -> Vec<&'static str> {
    let mut found: Vec<(usize, &'static str)> = ["orbit", "moonbase", "comet"]
        .into_iter()
        .filter_map(|n| s.find(n).map(|i| (i, n)))
        .collect();
    found.sort();
    found.into_iter().map(|(_, n)| n).collect()
}

fn wait_bar(term: &mut Terminal, want: &[&str]) {
    let want = want.to_vec();
    term.wait_for("the tab order", STEP, |s| {
        let line = s.text().lines().nth(1).unwrap_or("").to_string();
        bar_order(&line) == want
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn dragging_a_tab_shows_a_ghost_and_a_marker_and_a_drop_reorders() {
    let t = three();
    let mut term = start(&t);
    wait_bar(&mut term, &["orbit", "moonbase", "comet"]);
    let (ox, row) = tab_pos(&term, "orbit");
    let (cx, _) = tab_pos(&term, "comet");
    term.press(ox + 1, row).unwrap();
    term.drag_to(ox + 5, row).unwrap();
    term.drag_to(cx + 12, row).unwrap();
    // Mid-drag: the ghost label by the pointer, the marker on the bar.
    term.wait_for("the ghost and the insertion marker", STEP, |s| {
        let text = s.text();
        text.contains("\u{21C4} orbit") && text.lines().nth(1).unwrap_or("").contains('\u{2503}')
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        order(t.config.path())[0],
        "orbit",
        "nothing moved before the drop"
    );
    term.release(cx + 12, row).unwrap();
    // One predicate: the new order, and the ghost gone.
    term.wait_for("the new order, the ghost gone", STEP, |s| {
        let line = s.text().lines().nth(1).unwrap_or("").to_string();
        bar_order(&line) == ["moonbase", "comet", "orbit"] && !s.text().contains("\u{21C4} orbit")
    })
    .unwrap_or_else(|e| panic!("the ghost is gone: {e}\n{}", term.text()));
    assert_eq!(order(t.config.path()), ["moonbase", "comet", "orbit"]);
    term.stop().unwrap();
}

#[test]
fn a_press_and_release_without_movement_still_selects_the_tab() {
    let t = three();
    let mut term = start(&t);
    wait_bar(&mut term, &["orbit", "moonbase", "comet"]);
    let (mx, row) = tab_pos(&term, "moonbase");
    term.click(mx + 1, row).unwrap();
    term.wait_for("moonbase shown", STEP, |s| s.contains("bravo-only-commit"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(order(t.config.path()), ["orbit", "moonbase", "comet"]);
    term.stop().unwrap();
}

#[test]
fn escape_cancels_a_tab_drag_and_the_release_changes_nothing() {
    let t = three();
    let mut term = start(&t);
    wait_bar(&mut term, &["orbit", "moonbase", "comet"]);
    let (ox, row) = tab_pos(&term, "orbit");
    let (cx, _) = tab_pos(&term, "comet");
    term.press(ox + 1, row).unwrap();
    term.drag_to(ox + 5, row).unwrap();
    term.drag_to(cx + 12, row).unwrap();
    term.wait_for("the ghost", STEP, |s| s.contains("\u{21C4} orbit"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the drag canceled", STEP, |s| {
        !s.contains("\u{21C4} orbit") && s.contains("drag of orbit canceled")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(cx + 12, row).unwrap();
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(bar_order(&bar(&term)), ["orbit", "moonbase", "comet"]);
    assert_eq!(order(t.config.path()), ["orbit", "moonbase", "comet"]);
    term.stop().unwrap();
}

#[test]
fn the_tab_picker_reorders_with_alt_arrows_and_shows_the_hint() {
    let t = three();
    let mut term = start(&t);
    wait_bar(&mut term, &["orbit", "moonbase", "comet"]);
    term.send_keys("C-n").unwrap();
    term.send_text("t").unwrap();
    term.wait_for("the tab picker", STEP, |s| {
        s.contains("Alt+Up/Down to reorder")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The highlight starts on the first row (orbit): move it down twice.
    term.send_keys("M-Down").unwrap();
    wait_bar(&mut term, &["moonbase", "orbit", "comet"]);
    term.send_keys("M-Down").unwrap();
    wait_bar(&mut term, &["moonbase", "comet", "orbit"]);
    term.send_keys("M-Up").unwrap();
    wait_bar(&mut term, &["moonbase", "orbit", "comet"]);
    assert_eq!(order(t.config.path()), ["moonbase", "orbit", "comet"]);
    // The picker is still open and the highlight followed orbit: Enter
    // switches to it (it is the shown tab already, so the view stays).
    term.send_keys("Escape").unwrap();
    term.stop().unwrap();
}

#[test]
fn dragging_a_picker_row_reorders_the_tabs() {
    let t = three();
    let mut term = start(&t);
    wait_bar(&mut term, &["orbit", "moonbase", "comet"]);
    term.send_keys("C-n").unwrap();
    term.send_text("t").unwrap();
    term.wait_for("the tab picker", STEP, |s| {
        s.contains("Alt+Up/Down to reorder")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term.screen();
    // The picker rows carry the names after a two-cell marker; the bar
    // (row 1) also has them, so look from row 2 down.
    let find_row = |name: &str| {
        screen
            .find_all(name)
            .into_iter()
            .find(|(_, r)| *r > 3)
            .unwrap_or_else(|| panic!("no picker row for {name}\n{}", term.text()))
    };
    let (ox, orow) = find_row("orbit");
    let (_, crow) = find_row("comet");
    term.press(ox + 1, orow).unwrap();
    term.drag_to(ox + 1, orow + 1).unwrap();
    term.drag_to(ox + 1, crow).unwrap();
    term.wait_for("the ghost over the picker", STEP, |s| {
        s.contains("\u{21C4} orbit")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(ox + 1, crow).unwrap();
    wait_bar(&mut term, &["moonbase", "comet", "orbit"]);
    assert_eq!(order(t.config.path()), ["moonbase", "comet", "orbit"]);
    term.stop().unwrap();
}

#[test]
fn the_palette_moves_the_current_tab_and_refuses_at_the_ends() {
    let t = three();
    let mut term = start(&t);
    wait_bar(&mut term, &["orbit", "moonbase", "comet"]);
    // orbit is the first tab and is shown: moving left has no room, and the
    // palette says why instead of doing nothing.
    term.send_keys("C-p").unwrap();
    term.send_text("Tab: Move Left").unwrap();
    term.wait_for("the command, unavailable", STEP, |s| {
        s.contains("Command Palette (1/")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    term.wait_for("the reason", STEP, |s| s.contains("already the first one"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(order(t.config.path()), ["orbit", "moonbase", "comet"]);
    term.send_keys("C-p").unwrap();
    term.send_text("Tab: Move Right").unwrap();
    term.wait_for("the command", STEP, |s| s.contains("Command Palette (1/"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_bar(&mut term, &["moonbase", "orbit", "comet"]);
    assert_eq!(order(t.config.path()), ["moonbase", "orbit", "comet"]);
    // And back.
    term.send_keys("C-p").unwrap();
    term.send_text("Tab: Move Left").unwrap();
    term.wait_for("the command", STEP, |s| s.contains("Command Palette (1/"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Enter").unwrap();
    wait_bar(&mut term, &["orbit", "moonbase", "comet"]);
    term.stop().unwrap();
}

#[test]
fn the_new_order_survives_a_restart() {
    let t = three();
    let mut term = start(&t);
    wait_bar(&mut term, &["orbit", "moonbase", "comet"]);
    let (ox, row) = tab_pos(&term, "orbit");
    let (cx, _) = tab_pos(&term, "comet");
    term.drag((ox + 1, row), (cx + 12, row)).unwrap();
    wait_bar(&mut term, &["moonbase", "comet", "orbit"]);
    term.stop().unwrap();
    // A fresh process, same session: the bar comes back in the new order.
    let mut term = start(&t);
    wait_bar(&mut term, &["moonbase", "comet", "orbit"]);
    term.stop().unwrap();
}

#[test]
fn the_pinned_chips_stay_put_and_a_drop_before_the_first_tab_lands_first() {
    let t = three();
    let mut term = start(&t);
    wait_bar(&mut term, &["orbit", "moonbase", "comet"]);
    let home_before = term
        .wait_for_text("\u{2302}", STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (cx, row) = tab_pos(&term, "comet");
    // Drag the last tab as far left as the bar goes: over the Home button.
    term.drag((cx + 1, row), (0, row)).unwrap();
    wait_bar(&mut term, &["comet", "orbit", "moonbase"]);
    assert_eq!(
        term.wait_for_text("\u{2302}", STEP)
            .unwrap_or_else(|e| panic!("{e}")),
        home_before,
        "Home did not move"
    );
    assert_eq!(order(t.config.path()), ["comet", "orbit", "moonbase"]);
    let _ = (&t.moonbase, &t.comet);
    term.stop().unwrap();
}

/// When the tabs do not fit, the `...` after the last one that did is a
/// button: it opens the tab picker (the twin of `t`).
#[test]
fn the_overflow_ellipsis_opens_the_tab_picker() {
    let t = three();
    let mut term = common::launch(&t.orbit)
        .session(SESSION)
        .config_dir(t.config.path())
        .size(44, 30)
        .no_exit_pause()
        .ready("orbit")
        .mouse_capture()
        .spawn();
    let _ = term.wait_for_stable(Duration::from_millis(500), STEP);
    let (x, row) = term
        .screen()
        .find_all("\u{2026}")
        .into_iter()
        .find(|(_, r)| *r == 1)
        .unwrap_or_else(|| panic!("no overflow ellipsis\n{}", term.text()));
    term.click(x + 1, row).unwrap();
    term.wait_for("the tab picker", STEP, |s| s.contains("Select Tab (3/3)"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
