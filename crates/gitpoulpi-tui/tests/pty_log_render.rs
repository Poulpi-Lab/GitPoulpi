//! The Log pane's text, as drawn by the real binary, for a repository with
//! every row kind the renderer handles (WIP, a stash, a merge commit with
//! two lanes, a branch pill, a tag pill, a remote-less repo) at a wide and
//! a narrow terminal. The text graph renderer is forced so the lanes are
//! characters, and the volatile cells (short SHAs, dates) are masked, so
//! the digest is the same on every run and every machine.
//!
//! A golden by hand rather than `insta` (the TUI crate has no `insta`
//! dependency yet): `GOLDEN_WIDE` / `GOLDEN_NARROW` below. When a change
//! to the Log's look is intended, run with `LOG_RENDER_PRINT=1` and paste
//! the printed digest back in.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "one\n", "first commit")
        .unwrap()
        .commit_file("b.txt", "two\n", "second commit")
        .unwrap()
        .tag("v1")
        .unwrap()
        .build();
    repo.git(&["checkout", "-q", "-b", "side"]);
    std::fs::write(repo.root().join("s.txt"), "side\n").unwrap();
    repo.git(&["add", "s.txt"]);
    repo.git(&["commit", "-qm", "side work"]);
    repo.git(&["checkout", "-q", "-"]);
    std::fs::write(repo.root().join("m.txt"), "main\n").unwrap();
    repo.git(&["add", "m.txt"]);
    repo.git(&["commit", "-qm", "main work"]);
    repo.git(&["merge", "--no-ff", "-qm", "merge side", "side"]);
    // A stash, then a fresh dirty file so the WIP row is there too.
    std::fs::write(repo.root().join("a.txt"), "stashed\n").unwrap();
    repo.git(&["stash", "push", "-q", "-m", "parked"]);
    std::fs::write(repo.root().join("a.txt"), "dirty\n").unwrap();
    repo
}

fn launch(repo: &FixtureRepo, session: &str, cols: u16) -> (Terminal, tempfile::TempDir) {
    common::launch(repo.root())
        .session(session)
        .config("[ui]\ngraph_renderer = \"text\"\n")
        .size(cols, 40)
        .ready_when("every row kind drawn", |s| {
            s.contains("merge side") && s.contains("first commit") && s.contains("// WIP")
        })
        .start()
}

/// Masks the volatile cells, keeping every column where it was: a run of
/// 7 hex digits (a short SHA) becomes `SSSSSSS`, a `YYYY-MM-DD HH:MM`
/// stamp - or its `YYYY-MM-…` truncation in a narrow column - becomes `D`s.
fn mask(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let digits = |from: usize, n: usize| {
        from + n <= chars.len() && chars[from..from + n].iter().all(|c| c.is_ascii_digit())
    };
    let at = |i: usize, c: char| chars.get(i) == Some(&c);
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if digits(i, 4) && at(i + 4, '-') && digits(i + 5, 2) && at(i + 7, '-') {
            let mut end = i + 8;
            if digits(end, 2)
                && at(end + 2, ' ')
                && digits(end + 3, 2)
                && at(end + 5, ':')
                && digits(end + 6, 2)
            {
                end += 8;
            } else if at(end, '\u{2026}') {
                end += 1;
            }
            out.extend(std::iter::repeat_n('D', end - i));
            i = end;
        } else if i + 7 <= chars.len()
            && chars[i..i + 7]
                .iter()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            && chars[i..i + 7].iter().any(|c| c.is_ascii_digit())
            && (i == 0 || !chars[i - 1].is_alphanumeric())
            && (i + 7 == chars.len() || !chars[i + 7].is_alphanumeric())
        {
            out.push_str("SSSSSSS");
            i += 7;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// The Log pane's cells from its header row down to its last row, between
/// the `Ref` title and the end of the `SHA` title, masked and trimmed.
fn log_digest(s: &Screen) -> String {
    // The header row is the row of the `Message` title; `Ref` is the
    // first title on it (the sidebar's own `Refs` is on another row).
    let Some((_, header)) = s.find("Message") else {
        return String::new();
    };
    let on_header = |title: &str| {
        s.find_all(title)
            .into_iter()
            .find(|(_, r)| *r == header)
            .map(|(c, _)| c)
    };
    let Some(ref_x) = on_header("Ref") else {
        return String::new();
    };
    let end = match (on_header("SHA"), on_header("Date")) {
        (Some(sha), _) => sha + 7,
        (None, Some(date)) => date + 16,
        _ => ref_x + 80,
    };
    let mut lines = Vec::new();
    for row in header..s.rows() {
        let mut line = String::new();
        for col in ref_x..end {
            line.push_str(s.cell(col, row).map_or(" ", |c| c.display()));
        }
        let line = mask(line.trim_end());
        if row > header && line.is_empty() {
            break;
        }
        lines.push(line);
    }
    lines.join("\n")
}

fn check(golden: &str, name: &str, cols: u16) {
    let repo = fixture();
    let (term, _c) = launch(&repo, name, cols);
    // Wait for the real condition, the golden itself, not for "looks
    // settled": under load refs, avatars and graph cells can arrive after any
    // fixed settle window. On a timeout the last digest seen is asserted, so a
    // genuine render change still fails with its diff.
    let mut last = String::new();
    let waited = term.wait_for("the Log matching its golden", STEP, |s| {
        last = log_digest(s);
        last == golden
    });
    let digest = match waited {
        Ok(s) => log_digest(&s),
        Err(_) => last,
    };
    if std::env::var_os("LOG_RENDER_PRINT").is_some() {
        println!("{name}:\n{digest}\n");
    }
    assert_eq!(digest, golden, "{name}: the Log render changed:\n{digest}");
    term.stop().unwrap();
}

const GOLDEN_WIDE: &str = "\
Ref                  │Graph  │Message            │Author     │Date            │SHA
                      ◌       // WIP          ✎ 1
                      ┊ □     On main: parked     stash@{0}   DDDDDDDDDDDDDDDD SSSSSSS
 ✓ main 💻  ───────────◎─┴─╮   merge side          GitPoulpi … DDDDDDDDDDDDDDDD SSSSSSS
                      ○   │   main work           GitPoulpi … DDDDDDDDDDDDDDDD SSSSSSS
 side 💻  ─────────────│───●   side work           GitPoulpi … DDDDDDDDDDDDDDDD SSSSSSS
 v1 🔖  ⬆ ─────────────●───╯   second commit       GitPoulpi … DDDDDDDDDDDDDDDD SSSSSSS
                      ○       first commit        GitPoulpi … DDDDDDDDDDDDDDDD SSSSSSS";
const GOLDEN_NARROW: &str = "\
Ref      │Gra…Message        │Aut…Date     │SHA   │
          ◌   // WIP      ✎ 1
          ┊ □ On main: parked st… DDDDDDDDD SSSSSSS
 ✓ ma… 💻  ◎─┴─merge side      Gi… DDDDDDDDD SSSSSSS
          ○   main work       Gi… DDDDDDDDD SSSSSSS
 side 💻  ─│───side work       Gi… DDDDDDDDD SSSSSSS
 v1 🔖  ⬆ ─●───second commit   Gi… DDDDDDDDD SSSSSSS
          ○   first commit    Gi… DDDDDDDDD SSSSSSS";

#[test]
fn the_log_renders_every_row_kind_wide() {
    check(GOLDEN_WIDE, "pty-logrender-wide", 200);
}

#[test]
fn the_log_renders_every_row_kind_narrow() {
    check(GOLDEN_NARROW, "pty-logrender-narrow", 120);
}
