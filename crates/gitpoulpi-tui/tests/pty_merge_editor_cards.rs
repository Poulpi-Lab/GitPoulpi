//! The merge editor's cards, end to end through a pty (plan
//! the merge-conflict-editor design §6, slice S8): one test
//! per kind of conflict that is not text, on a real conflicted repository
//! from the testkit's `ConflictFixture`. Each opens the file from the Status
//! pane, checks the card names the conflict, answers with its keys (and one
//! with the mouse) and then looks at what git holds.

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::ConflictFixture;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn launch(repo: &Path, session: &str) -> (Terminal, tempfile::TempDir) {
    common::launch(repo)
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready_when("the conflicted file in the Status pane", |s| {
            s.contains("Unstaged") && s.contains("Merge in progress")
        })
        .mouse_capture()
        .start()
}

/// Opens `name` from the Status pane with Enter and waits for the card's
/// headline together with every text in `with` (one whole state: a torn
/// frame can show the headline before the rest). Returns that screen's text.
fn open_card(term: &mut Terminal, name: &str, headline: &str, with: &[&str]) -> String {
    // The conflicted row reads `! name`: a bare name may match other text.
    let (c, r) = term
        .wait_for_text(&format!("! {name}"), STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 2, r).unwrap();
    term.send_keys("Enter").unwrap();
    let t = term
        .wait_for("the card", STEP, |s| {
            s.contains(headline) && with.iter().all(|w| s.contains(w))
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
        .text();
    // A card has no text panes and no Output.
    assert!(!t.contains("Output \u{b7}"), "{t}");
    // Nothing edited the file: no "edited outside the editor" strip (a
    // card has no text to continue from).
    assert!(!t.contains("edited outside"), "{t}");
    t
}

fn wait_resolved(term: &mut Terminal, name: &str) {
    term.wait_for("the file resolved, the list back", STEP, |s| {
        s.contains(&format!("resolved {name}"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

fn unmerged(fixture: &ConflictFixture) -> Vec<String> {
    common::git(fixture.root(), &["diff", "--name-only", "--diff-filter=U"])
        .lines()
        .map(str::to_string)
        .collect()
}

/// `git ls-files -s <path>` in the index (mode, oid, stage).
fn index_entry(fixture: &ConflictFixture, path: &str) -> String {
    common::git(fixture.root(), &["ls-files", "-s", "--", path])
}

#[test]
fn a_binary_file_shows_a_card_and_b_takes_their_bytes() {
    let fixture = ConflictFixture::binary();
    let theirs = fixture.stage_bytes(3, "image.bin").unwrap();
    let (mut term, _config) = launch(fixture.root(), "pty-card-binary");
    let t = open_card(
        &mut term,
        "image.bin",
        "Binary file",
        &["Keep A", "Take B", "Take base"],
    );
    assert!(t.contains("Keep A") && t.contains("Take B") && t.contains("Take base"));
    term.send_keys("b").unwrap();
    wait_resolved(&mut term, "image.bin");
    assert_eq!(
        std::fs::read(fixture.root().join("image.bin")).unwrap(),
        theirs
    );
    assert!(unmerged(&fixture).is_empty());
    term.stop().unwrap();
}

/// The same card with the mouse: the Keep A button is where the paint
/// walk put it.
#[test]
fn a_card_button_works_with_the_mouse() {
    let fixture = ConflictFixture::binary();
    let ours = fixture.stage_bytes(2, "image.bin").unwrap();
    let (mut term, _config) = launch(fixture.root(), "pty-card-mouse");
    open_card(&mut term, "image.bin", "Binary file", &[]);
    let (c, r) = term
        .wait_for_text("Keep A", STEP)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.click(c + 1, r).unwrap();
    wait_resolved(&mut term, "image.bin");
    assert_eq!(
        std::fs::read(fixture.root().join("image.bin")).unwrap(),
        ours
    );
    term.stop().unwrap();
}

#[test]
fn an_lfs_pointer_shows_both_objects_and_a_keeps_ours() {
    let fixture = ConflictFixture::lfs_pointer();
    let ours = fixture.stage_bytes(2, "big.dat").unwrap();
    let (mut term, _config) = launch(fixture.root(), "pty-card-lfs");
    let t = open_card(
        &mut term,
        "big.dat",
        "Git LFS pointer",
        &["bbbbbbbbbbbb", "cccccccccccc"],
    );
    assert!(
        t.contains("bbbbbbbbbbbb") && t.contains("cccccccccccc"),
        "{t}"
    );
    term.send_keys("a").unwrap();
    wait_resolved(&mut term, "big.dat");
    assert_eq!(std::fs::read(fixture.root().join("big.dat")).unwrap(), ours);
    term.stop().unwrap();
}

/// Plan test 7: delete / modify, A deleted it. `D` accepts the deletion:
/// the file is gone and the deletion is staged.
#[test]
fn deleted_in_a_changed_in_b_d_accepts_the_deletion() {
    let fixture = ConflictFixture::delete_modify(true);
    let (mut term, _config) = launch(fixture.root(), "pty-card-delete-a");
    let t = open_card(
        &mut term,
        "gone.txt",
        "Deleted in A, changed in B",
        &["Keep the file", "Accept the deletion"],
    );
    assert!(t.contains("Keep the file") && t.contains("Accept the deletion"));
    term.send_text("D").unwrap();
    wait_resolved(&mut term, "gone.txt");
    assert!(!fixture.root().join("gone.txt").exists());
    assert_eq!(index_entry(&fixture, "gone.txt"), "");
    assert!(unmerged(&fixture).is_empty());
    term.stop().unwrap();
}

/// B deleted it, A changed it: `k` keeps the changed file.
#[test]
fn changed_in_a_deleted_in_b_k_keeps_the_file() {
    let fixture = ConflictFixture::delete_modify(false);
    let ours = fixture.stage_bytes(2, "gone.txt").unwrap();
    let (mut term, _config) = launch(fixture.root(), "pty-card-delete-b");
    open_card(&mut term, "gone.txt", "Changed in A, deleted in B", &[]);
    term.send_keys("k").unwrap();
    wait_resolved(&mut term, "gone.txt");
    assert_eq!(
        std::fs::read(fixture.root().join("gone.txt")).unwrap(),
        ours
    );
    assert!(index_entry(&fixture, "gone.txt").contains("100644"));
    term.stop().unwrap();
}

/// A rename that git could not finish leaves the path on one side only:
/// Delete removes it.
#[test]
fn a_rename_leftover_can_be_deleted() {
    let fixture = ConflictFixture::rename_delete();
    let (mut term, _config) = launch(fixture.root(), "pty-card-rename");
    // The base and A have it, B deleted it.
    let t = open_card(
        &mut term,
        "b.txt",
        "Changed in A, deleted in B",
        &["Keep the file", "Accept the deletion"],
    );
    assert!(
        t.contains("Keep the file") && t.contains("Accept the deletion"),
        "{t}"
    );
    term.send_text("D").unwrap();
    wait_resolved(&mut term, "b.txt");
    assert!(!fixture.root().join("b.txt").exists());
    term.stop().unwrap();
}

#[cfg(unix)]
#[test]
fn a_symlink_shows_both_targets_and_b_takes_theirs() {
    let fixture = ConflictFixture::symlink();
    let (mut term, _config) = launch(fixture.root(), "pty-card-symlink");
    let t = open_card(
        &mut term,
        "link",
        "Symbolic link",
        &["target-ours", "target-theirs"],
    );
    assert!(
        t.contains("target-ours") && t.contains("target-theirs"),
        "{t}"
    );
    term.send_keys("b").unwrap();
    wait_resolved(&mut term, "link");
    assert_eq!(
        std::fs::read_link(fixture.root().join("link")).unwrap(),
        Path::new("target-theirs")
    );
    term.stop().unwrap();
}

#[test]
fn a_mode_only_conflict_names_both_modes_and_a_keeps_ours() {
    let fixture = ConflictFixture::mode_only();
    let (mut term, _config) = launch(fixture.root(), "pty-card-mode");
    let t = open_card(
        &mut term,
        "run.sh",
        "Only the file mode differs",
        &["100644", "100755"],
    );
    assert!(t.contains("100644") && t.contains("100755"), "{t}");
    term.send_keys("b").unwrap();
    wait_resolved(&mut term, "run.sh");
    assert!(index_entry(&fixture, "run.sh").starts_with("100755"));
    term.stop().unwrap();
}

#[test]
fn a_submodule_shows_both_commits_and_b_takes_theirs() {
    let fixture = ConflictFixture::submodule();
    let (mut term, _config) = launch(fixture.root(), "pty-card-submodule");
    let t = open_card(
        &mut term,
        "sub",
        "Submodule",
        &["commit 3333333", "commit 2222222"],
    );
    assert!(
        t.contains("commit 3333333") && t.contains("commit 2222222"),
        "{t}"
    );
    term.send_keys("b").unwrap();
    wait_resolved(&mut term, "sub");
    let entry = index_entry(&fixture, "sub");
    assert!(
        entry.starts_with("160000 2222222222222222222222222222222222222222"),
        "{entry}"
    );
    term.stop().unwrap();
}

#[test]
fn a_file_over_the_size_cap_is_a_card_and_a_keeps_ours() {
    let fixture = ConflictFixture::huge();
    let ours = fixture.stage_bytes(2, "huge.txt").unwrap();
    let (mut term, _config) = launch(fixture.root(), "pty-card-huge");
    let t = open_card(
        &mut term,
        "huge.txt",
        "File too large",
        &["Open in merge tool"],
    );
    assert!(t.contains("Open in merge tool"), "{t}");
    term.send_keys("a").unwrap();
    wait_resolved(&mut term, "huge.txt");
    assert_eq!(
        std::fs::read(fixture.root().join("huge.txt")).unwrap(),
        ours
    );
    term.stop().unwrap();
}

/// Ctrl+S on a card never writes anything (it has no text).
#[test]
fn saving_on_a_card_does_nothing() {
    let fixture = ConflictFixture::binary();
    let before = std::fs::read(fixture.root().join("image.bin")).unwrap();
    let (mut term, _config) = launch(fixture.root(), "pty-card-save");
    open_card(&mut term, "image.bin", "Binary file", &[]);
    term.send_keys("C-s").unwrap();
    term.send_keys("Escape").unwrap();
    term.wait_for("the card closed", STEP, |s| !s.contains("Binary file"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(
        std::fs::read(fixture.root().join("image.bin")).unwrap(),
        before
    );
    assert_eq!(unmerged(&fixture), ["image.bin"]);
    term.stop().unwrap();
}

// -- picture previews (image view S8) ------------------------------------------

fn color_near(c: xtask::pty::Color, rgb: (u8, u8, u8)) -> bool {
    match c {
        xtask::pty::Color::Rgb(r, g, b) => {
            r.abs_diff(rgb.0) < 8 && g.abs_diff(rgb.1) < 8 && b.abs_diff(rgb.2) < 8
        }
        _ => false,
    }
}

/// Columns holding a half-block cell of that color.
fn picture_columns(s: &xtask::pty::Screen, rgb: (u8, u8, u8)) -> Vec<u16> {
    let mut cols = Vec::new();
    for row in 0..s.rows() {
        for col in 0..s.cols() {
            if let Some(c) = s.cell(col, row) {
                if c.text == "\u{2580}" && color_near(c.fg, rgb) && color_near(c.bg, rgb) {
                    cols.push(col);
                }
            }
        }
    }
    cols
}

/// A conflicted picture shows A (red), B (blue) and the base (green) inside
/// the card's boxes, read-only, and the buttons still resolve: `b` takes B.
#[test]
fn a_conflicted_picture_previews_a_b_and_base_and_the_buttons_still_work() {
    let fixture = ConflictFixture::image();
    let theirs = fixture.stage_bytes(3, "pic.png").unwrap();
    let (mut term, _config) = launch(fixture.root(), "pty-card-picture");
    open_card(&mut term, "pic.png", "Binary file", &[]);
    term.wait_for("the three pictures", STEP, |s| {
        ![(255, 0, 0), (0, 0, 255), (0, 255, 0)]
            .iter()
            .any(|c| picture_columns(s, *c).is_empty())
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let s = term.screen();
    let (a, b, base) = (
        picture_columns(&s, (255, 0, 0)),
        picture_columns(&s, (0, 0, 255)),
        picture_columns(&s, (0, 255, 0)),
    );
    // Side by side, in the order of the boxes: A, B, base.
    assert!(a.iter().max() < b.iter().min(), "{a:?} / {b:?}");
    assert!(b.iter().max() < base.iter().min(), "{b:?} / {base:?}");
    let t = term.text();
    assert!(t.contains("Keep A") && t.contains("Take B"), "{t}");
    term.send_keys("b").unwrap();
    wait_resolved(&mut term, "pic.png");
    assert_eq!(
        std::fs::read(fixture.root().join("pic.png")).unwrap(),
        theirs
    );
    term.stop().unwrap();
}

/// A binary that is not a picture keeps the plain card (no preview rows).
#[test]
fn a_conflicted_binary_that_is_not_a_picture_has_no_preview() {
    let fixture = ConflictFixture::binary();
    let (mut term, _config) = launch(fixture.root(), "pty-card-nopicture");
    open_card(&mut term, "image.bin", "Binary file", &[]);
    let s = term.screen();
    for c in [(255, 0, 0), (0, 0, 255), (0, 255, 0)] {
        assert!(picture_columns(&s, c).is_empty());
    }
    term.stop().unwrap();
}
