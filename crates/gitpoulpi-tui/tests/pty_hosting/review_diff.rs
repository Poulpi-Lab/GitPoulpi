//! The review's diff pane has the Diff view's options: Inline / Split, the
//! hunks or the whole file, whitespace, wrap, the encoding dropdown, and the
//! scrollbar's ruler - against the shared `xtask mock-host`, over a real
//! repository whose request changes line 12 of a 150-line file (so the whole
//! file scrolls and the ruler has room to show where things are).

use crate::review::*;
use crate::support::*;
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::mock_host::{self, Options};

fn long_fixture() -> FixtureRepo {
    let base: String = (1..=150).map(|n| format!("// line {n}\n")).collect();
    let moons = base.replace("// line 12\n", "let moons = Vec::new();\nmoons.push(1);\n");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("src/planets.rs", &base, "Start the planet table")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit_file("src/planets.rs", &moons, "Add moons to the table")
        .unwrap()
        .commit_file("docs/moons.md", "# Moons\ntext\n", "Document the moons")
        .unwrap()
        .push("feature:refs/merge-requests/58/head")
        .unwrap()
        .build();
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[pulls_source]\nintegration = \"corp\"\nproject = \"o/orbit\"\n",
    )
    .expect("seed repo-state.toml");
    repo
}

#[test]
fn the_review_diff_has_split_the_whole_file_whitespace_the_ruler_and_comments_in_split() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = long_fixture();
    let (mut term, config) = launch_review(&mock, &repo, "pty-review-diff-options");
    open_from_menu(&mut term);
    open_files(&mut term);

    // The Diff view's bar is on the pane, with the encoding dropdown.
    let screen = term
        .wait_for("the Diff view's bar and the code", STEP, |s| {
            ["Inline", "Split", "Hunks", "WS", "Wrap", "Auto"]
                .iter()
                .all(|label| s.contains(label))
                && s.contains("let moons = Vec::new();")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    for label in ["Inline", "Split", "Hunks", "WS", "Wrap", "Auto"] {
        assert!(screen.contains(label), "{label}:\n{}", screen.text());
    }
    let (inline_col, _) = screen.find("let moons = Vec::new();").unwrap();

    // Split, by its button: the added lines move to the right half, and the
    // thread goes with them.
    let (c, r) = screen.find("Split").unwrap();
    term.click(c + 1, r).expect("Split");
    let screen = term
        .wait_for(
            "the Split layout, the thread under the new side",
            STEP,
            |s| {
                // The thread is part of the wait: the code lines move to the
                // right half a frame before the thread is laid out under them.
                match (
                    s.find("let moons = Vec::new();"),
                    s.find("Why a Vec here and not a slice?"),
                ) {
                    (Some((moons, _)), Some((thread, _))) => {
                        moons > inline_col + 20 && thread + 3 >= moons
                    }
                    _ => false,
                }
            },
        )
        .unwrap_or_else(|e| panic!("{e}"));
    let (moons_col, _) = screen.find("let moons = Vec::new();").unwrap();
    let (thread_col, _) = screen
        .find("Why a Vec here and not a slice?")
        .unwrap_or_else(|| panic!("the thread:\n{}", screen.text()));
    assert!(
        thread_col + 3 >= moons_col,
        "the thread sits under the new side:\n{}",
        screen.text()
    );

    // The whole file, by its key: the lines the hunks leave out are there and
    // the ruler marks the thread apart from the change.
    term.send_text("H").expect("the Hunks toggle");
    term.wait_for("the whole file", STEP, |s| {
        s.contains("// line 5") && s.contains("// line 30")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let screen = term.screen();
    let right = screen.cols() / 2;
    assert!(
        screen
            .find_all("\u{25cf}")
            .iter()
            .any(|(col, _)| *col > right),
        "a thread dot on the scrollbar:\n{}",
        screen.text()
    );
    assert!(
        screen
            .find_all("\u{2590}")
            .iter()
            .any(|(col, _)| *col > right),
        "a change mark on the scrollbar:\n{}",
        screen.text()
    );

    // Whitespace, by its key and by its button.
    term.send_text("S").expect("the whitespace toggle");
    let screen = term.screen();
    let (c, r) = screen.find("WS").unwrap();
    term.click(c, r).expect("WS");
    // All of it is remembered the way the Diff view's own toggles are.
    wait_until("the toggles were written to tui.toml", || {
        let text = std::fs::read_to_string(config.path().join("tui.toml")).unwrap_or_default();
        text.contains("diff_layout = \"split\"")
            && text.contains("diff_hunks_only = false")
            && text.contains("diff_show_whitespace = true")
    });

    // A comment on a line, in Split: pending, under the line, in its half.
    // A click on a torn frame can be lost: retry until the box shows.
    crate::common::retry_until(
        &mut term,
        "the comment box",
        6,
        std::time::Duration::from_secs(5),
        |t| {
            let screen = t.screen();
            if screen.contains("Comment on code") {
                return;
            }
            if let Some((c, r)) = screen.find("moons.push(1);") {
                t.click(c + 2, r).expect("the second added line");
                t.send_keys("c").expect("comment");
            }
        },
        |s| s.contains("Comment on code") && s.contains("src/planets.rs:13"),
    );
    term.send_text("needs a doc comment").unwrap();
    term.send_keys("C-s").expect("keep it");
    let screen = term
        .wait_for("the pending comment under its line", STEP, |s| {
            let (Some((_, pending)), Some((_, code))) =
                (s.find("Pending - line 13"), s.find("moons.push(1);"))
            else {
                return false;
            };
            s.contains("1 pending comment")
                && pending > code
                && s.contains("needs a doc comment")
                && s.find_all("\u{25cf}")
                    .iter()
                    .any(|(col, _)| *col > s.cols() / 2)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (pending_col, pending_row) = screen.find("Pending - line 13").unwrap();
    let (_, code_row) = screen.find("moons.push(1);").unwrap();
    assert!(pending_row > code_row, "under its line:\n{}", screen.text());
    assert!(
        pending_col + 3 >= moons_col,
        "in the half of its side:\n{}",
        screen.text()
    );
    assert!(
        screen
            .find_all("\u{25cf}")
            .iter()
            .filter(|(col, _)| *col > right)
            .count()
            >= 1,
        "the ruler still marks the threads:\n{}",
        screen.text()
    );
}

/// The request changes line 12 of `src/planets.rs`, then adds a Latin-1 line
/// and a very long one right under it (the file is not UTF-8).
fn latin_fixture() -> FixtureRepo {
    let base: String = (1..=30).map(|n| format!("// line {n}\n")).collect();
    let mut moons: Vec<u8> = base
        .replace("// line 12\n", "let moons = Vec::new();\nmoons.push(1);\n")
        .into_bytes();
    moons.extend_from_slice(b"// caf\xe9 d\xe9j\xe0 vu\n");
    moons.extend_from_slice(format!("// {}END_OF_LONG_LINE\n", "w".repeat(220)).as_bytes());
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("src/planets.rs", &base, "Start the planet table")
        .unwrap()
        .with_origin()
        .unwrap()
        .push("main")
        .unwrap()
        .branch("feature")
        .unwrap()
        .commit_bytes("src/planets.rs", &moons, "Add moons to the table")
        .unwrap()
        .commit_file("docs/moons.md", "# Moons\ntext\n", "Document the moons")
        .unwrap()
        .push("feature:refs/merge-requests/58/head")
        .unwrap()
        .build();
    let dir = repo.state_dir();
    std::fs::create_dir_all(&dir).expect("state dir");
    std::fs::write(
        dir.join("repo-state.toml"),
        "[pulls_source]\nintegration = \"corp\"\nproject = \"o/orbit\"\n",
    )
    .expect("seed repo-state.toml");
    repo
}

#[test]
fn the_review_diff_reads_a_file_in_another_encoding_scrolls_sideways_and_copies_text() {
    let mock = mock_host::start(Options::default()).expect("the mock host");
    let repo = latin_fixture();
    let (mut term, _config) = launch_review(&mock, &repo, "pty-review-diff-parts");
    open_from_menu(&mut term);
    open_files(&mut term);

    // Auto-detect: the backend's reading of a Latin-1 file.
    let screen = term
        .wait_for("the Latin-1 line as the backend read it", STEP, |s| {
            s.contains("// caf")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("// caf\u{e9} d\u{e9}j\u{e0} vu"),
        "not decoded yet:\n{}",
        screen.text()
    );

    // The encoding dropdown: the key opens it, the search line narrows it, and
    // Enter reads both sides of the file again in that encoding.
    term.send_text("E").expect("the encoding key");
    term.wait_for("the encoding list", STEP, |s| {
        s.contains("Auto-detect") && s.contains("Windows-1252")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("1252").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the file read as Windows-1252", STEP, |s| {
        s.contains("// caf\u{e9} d\u{e9}j\u{e0} vu") && s.contains("Windows-1252 \u{25be}")
    })
    .unwrap_or_else(|e| panic!("{e}"));

    // Sideways: the long line is cut at the pane's edge; Right (on the diff)
    // scrolls it until its end shows.
    assert!(!term.screen().contains("END_OF_LONG_LINE"));
    term.send_keys("Tab").expect("focus the diff");
    for _ in 0..60 {
        term.send_keys("Right").unwrap();
    }
    let screen = term
        .wait_for("the end of the long line", STEP, |s| {
            s.contains("END_OF_LONG_LINE")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("\u{ab}"),
        "the scrolled-row marker:\n{}",
        screen.text()
    );

    // A drag in the code selects text, Ctrl+C copies it.
    let (c, r) = screen.find("END_OF_LONG_LINE").unwrap();
    term.drag((c, r), (c + 6, r)).expect("select");
    term.send_keys("C-c").expect("copy");
    term.wait_for("the copy confirmed", STEP, |s| {
        s.contains("copied the selection") && s.contains("END_OF_")
    })
    .unwrap_or_else(|e| panic!("{e}"));
}
