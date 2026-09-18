//! End-to-end checks of the right panel's commit details, in
//! order: `commit:` header, the author block, the change counts, a bordered
//! message block (summary + a scrollable, resizable description), then the
//! changed-files pane whose first rows are the sort / Path|Tree / View-all
//! row and the filter, above the file list -- driven through a pty by
//! `xtask::pty`.

mod common;

use common::git_ok;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(20);
const COLS: u16 = 140;
const ROWS: u16 = 44;
/// The right panel starts right of this column at 140 columns.
const RIGHT_FROM: u16 = 85;

/// HEAD adds `aa.txt` and `zz.txt` and modifies `a.txt`, with a summary
/// and a twenty-line description.
fn fixture() -> gitpoulpi_testkit::FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "initial")
        .unwrap()
        .build();
    let root = repo.root().to_path_buf();
    std::fs::write(root.join("a.txt"), "changed\n").unwrap();
    std::fs::write(root.join("aa.txt"), "x\n").unwrap();
    std::fs::write(root.join("zz.txt"), "y\n").unwrap();
    git_ok(&root, &["add", "-A"]);
    let body: String = (1..=20)
        .map(|i| format!("desc-{i:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    git_ok(&root, &["commit", "-q", "-m", "the-summary", "-m", &body]);
    repo
}

fn launch(repo: &Path, session: &str, config: &Path) -> Terminal {
    common::launch(repo)
        .session(session)
        .config_dir(config)
        .size(COLS, ROWS)
        .ready_when("the commit's details", |s| {
            s.contains("commit: ") && s.contains("desc-01") && s.contains("zz.txt")
        })
        .spawn()
}

/// First row where `needle` appears in the right panel.
fn right_row(screen: &Screen, needle: &str) -> Option<u16> {
    screen
        .find_all(needle)
        .into_iter()
        .find(|(col, _)| *col >= RIGHT_FROM)
        .map(|(_, row)| row)
}

fn right_col(screen: &Screen, needle: &str) -> Option<u16> {
    screen
        .find_all(needle)
        .into_iter()
        .find(|(col, _)| *col >= RIGHT_FROM)
        .map(|(col, _)| col)
}

fn settled(term: &Terminal) -> Screen {
    term.wait_for_stable(Duration::from_millis(400), STEP)
        .unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn the_right_panel_stacks_header_author_counts_message_then_the_files_pane_options_and_list() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let term = launch(repo.root(), "cm-order", config.path());
    let s = settled(&term);

    let header = right_row(&s, "commit: ").expect("commit header");
    let summary = right_row(&s, "the-summary").expect("summary");
    let desc = right_row(&s, "desc-01").expect("description");
    let parent = right_row(&s, "parent: ").expect("parent");
    let authored = right_row(&s, "authored ").expect("authored line");
    let counts = right_row(&s, "modified").expect("counts");
    let toolbar = right_row(&s, "View all files").expect("toolbar");
    let filter = right_row(&s, "Filter files:").expect("filter");
    let files = right_row(&s, "a.txt").expect("file list");
    let title = right_row(&s, "Changed files").expect("files pane title");
    assert!(
        header < parent,
        "the author block is right under the header"
    );
    assert_eq!(parent + 1, authored, "authored sits under the author line");
    assert!(authored < counts && counts < summary && summary < desc);
    assert!(desc < title, "the message is above the files pane");
    assert_eq!(toolbar, title + 1, "the options open the files pane");
    assert_eq!(filter, toolbar + 1);
    assert_eq!(files, filter + 1, "the list starts under the filter");
    assert!(s.contains("1 modified") && s.contains("2 added"));
    // `parent:` is right-aligned on the author's line: far right of the name.
    assert!(right_col(&s, "parent: ").unwrap() > RIGHT_FROM + 20);
    // The description is capped: its tail is scrolled out of view.
    assert!(!s.contains("desc-20"));
    term.stop().unwrap();
}

#[test]
fn dragging_the_splitter_under_the_message_resizes_it_and_the_size_persists() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "cm-drag", config.path());
    let s = settled(&term);
    assert!(!s.contains("desc-10"), "the description starts capped");

    // The message block's bottom border is two rows above the files pane's
    // title (the details box closes on the row between).
    let border = right_row(&s, "Changed files").expect("files pane title") - 2;
    let col = right_col(&s, "commit: ").unwrap() + 8;
    term.drag((col, border), (col, border + 5)).unwrap();
    term.wait_for("more of the description", STEP, |s| s.contains("desc-10"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();

    let term = launch(repo.root(), "cm-drag", config.path());
    term.wait_for("the persisted height", STEP, |s| s.contains("desc-10"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// The default theme's `ACCENT`, which `paint_resize_strip` fills a
/// hovered splitter with.
const ACCENT_BG: xtask::pty::Color = xtask::pty::Color::Rgb(64, 156, 255);

/// Hovering the splitter under the message lights its whole width, not just
/// the first cell.
#[test]
fn hovering_the_splitter_under_the_message_lights_its_whole_width() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "cm-hover", config.path());
    let s = settled(&term);
    let border = right_row(&s, "Changed files").expect("files pane title") - 2;
    let col = right_col(&s, "commit: ").unwrap() + 8;
    term.move_to(col, border).unwrap();
    let lit = |s: &Screen| {
        (RIGHT_FROM..COLS)
            .filter(|c| s.cell(*c, border).is_some_and(|cell| cell.bg == ACCENT_BG))
            .count()
    };
    let s = term
        .wait_for("the lit splitter", STEP, |s| lit(s) > 20)
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(lit(&s) > 20, "only {} cells lit", lit(&s));
    term.stop().unwrap();
}

#[test]
fn the_keyboard_and_the_palette_resize_and_scroll_the_message() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "cm-keys", config.path());
    assert!(!settled(&term).contains("desc-09"));

    // Ctrl+N ] grows the block by a row each time.
    for _ in 0..3 {
        term.send_keys("C-n").unwrap();
        term.send_keys("]").unwrap();
    }
    term.wait_for("three more rows", STEP, |s| s.contains("desc-09"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-n").unwrap();
    term.send_keys("[").unwrap();
    term.wait_for("one row back", STEP, |s| {
        s.contains("desc-08") && !s.contains("desc-09")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-p").unwrap();
    term.send_text("Reset Message Size").unwrap();
    term.send_keys("Enter").unwrap();
    term.send_keys("C-p").unwrap();
    term.send_text("Scroll Message Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the description scrolled", STEP, |s| {
        right_row(s, "the-summary").is_none() && s.contains("desc-0")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-p").unwrap();
    term.send_text("Scroll Message Up").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("scrolled back to the summary", STEP, |s| {
        right_row(s, "the-summary").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

#[test]
fn the_wheel_scrolls_the_message_block() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "cm-wheel", config.path());
    let s = settled(&term);
    let row = right_row(&s, "desc-01").unwrap();
    let col = right_col(&s, "desc-01").unwrap();
    term.scroll(true, col, row).unwrap();
    term.wait_for("the description scrolled", STEP, |s| {
        right_row(s, "the-summary").is_none()
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

#[test]
fn the_sort_button_and_its_palette_twin_reverse_the_file_order() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "cm-sort", config.path());
    let s = settled(&term);
    assert!(right_row(&s, "aa.txt").unwrap() < right_row(&s, "zz.txt").unwrap());

    let sort = right_row(&s, "A\u{2193}Z").expect("sort button");
    let sort_col = right_col(&s, "A\u{2193}Z").unwrap();
    term.click(sort_col, sort).unwrap();
    term.wait_for(
        "Z to A order",
        STEP,
        |s| matches!((right_row(s, "aa.txt"), right_row(s, "zz.txt")), (Some(a), Some(z)) if z < a),
    )
    .unwrap_or_else(|e| panic!("{e}"));

    term.send_keys("C-p").unwrap();
    term.send_text("Toggle Commit File Sort").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for(
        "A to Z order again",
        STEP,
        |s| matches!((right_row(s, "aa.txt"), right_row(s, "zz.txt")), (Some(a), Some(z)) if a < z),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// `(column, up arrow row, down arrow row)` of the message block's
/// scrollbar: the first arrow pair in the right panel.
fn message_bar(screen: &Screen) -> Option<(u16, u16, u16)> {
    let (col, up) = screen
        .find_all("\u{2191}")
        .into_iter()
        .filter(|(c, _)| *c >= RIGHT_FROM)
        .min_by_key(|(_, r)| *r)?;
    let down = screen
        .find_all("\u{2193}")
        .into_iter()
        .filter(|(c, r)| *c == col && *r > up)
        .map(|(_, r)| r)
        .min()?;
    Some((col, up, down))
}

/// The owner could not drag the commit description's scrollbar: the thumb
/// must follow the pointer, even once the pointer has left the track, and
/// the end arrows and the track must work too.
#[test]
fn the_message_scrollbar_drags_pages_and_steps() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(repo.root(), "cm-bar-drag", config.path());
    let s = settled(&term);
    term.wait_for_mouse_capture(STEP).unwrap();
    let (col, up, down) = message_bar(&s).unwrap_or_else(|| panic!("bar\n{}", s.text()));
    assert!(right_row(&s, "desc-01").is_some() && !s.contains("desc-20"));
    // Press on the thumb (first cell under the up arrow), pull it down past
    // the track's end and sideways out of the bar, release.
    term.press(col, up + 1).unwrap();
    term.drag_to(col - 12, down + 2).unwrap();
    term.wait_for("the tail of the description", STEP, |s| {
        s.contains("desc-20")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(col - 12, down + 2).unwrap();
    // The thumb is at the bottom now: drag it back up to the top.
    let (col, up, down) = message_bar(&term.screen()).expect("the bar again");
    term.press(col, down - 1).unwrap();
    term.drag_to(col, up).unwrap();
    term.wait_for("the head of the description", STEP, |s| {
        s.contains("desc-01")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.release(col, up).unwrap();
    // A click on the track below the thumb moves the view.
    term.click(col, down - 1).unwrap();
    term.wait_for("the track click scrolled", STEP, |s| !s.contains("desc-01"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();
}
