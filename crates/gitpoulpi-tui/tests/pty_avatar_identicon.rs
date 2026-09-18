//! The avatar fallback, end to end through a pty. The pty has no image
//! protocol (`GITPOULPI_TERMINAL_IMAGE=off`), so the Author column shows
//! its one-cell glyph: the identicon as a quadrant block character by
//! default, the initials letter with `ui.avatar_fallback = "initials"`.
//! `ui.avatar_sources = "hosting"` in a repository with no remote keeps
//! the run off the network entirely. Same rules as `pty_ui.rs`.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

const STARTUP: Duration = Duration::from_secs(60);

/// The Author column truncates the name ("GitPoulp...") at this width.
const AUTHOR: &str = "GitPoulp";

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("an avatar commit")
        .unwrap()
        .build()
}

/// The character just before the author's name on the commit's row.
fn glyph_before_author(repo: &FixtureRepo, config_toml: &str, session: &str) -> char {
    let config = tempfile::tempdir().unwrap();
    std::fs::write(config.path().join("config.toml"), config_toml).unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 50)
        .spawn();
    let screen = term
        .wait_for("the commit row with its author", STARTUP, |s| {
            s.contains("an avatar commit") && s.contains(AUTHOR)
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (_, row) = screen.find("an avatar commit").unwrap();
    let line = screen.line(row);
    let before = line.split(AUTHOR).next().unwrap();
    // `glyph` + the gutter space.
    let mut chars = before.trim_end_matches(' ').chars().rev();
    chars.next().expect("a glyph before the author's name")
}

fn is_block(c: char) -> bool {
    ('\u{2596}'..='\u{259F}').contains(&c) || ('\u{2580}'..='\u{2588}').contains(&c)
}

/// Without an image protocol the identicon fallback degrades to a one-cell
/// block that is blank for some authors, so the Author column shows the
/// colored initials badge instead (never a block).
#[test]
fn a_text_terminal_shows_initials_even_with_the_identicon_fallback() {
    let repo = fixture();
    let glyph = glyph_before_author(
        &repo,
        "[ui]\navatars = \"author\"\navatar_sources = \"hosting\"\n",
        "pty-avatar-identicon",
    );
    assert!(!is_block(glyph) && glyph != ' ', "got {glyph:?}");
    assert_eq!(glyph, 'G', "the first initial of the author");
}

/// `initials` keeps the colored letter badge.
#[test]
fn the_initials_fallback_still_shows_a_letter() {
    let repo = fixture();
    let glyph = glyph_before_author(
        &repo,
        "[ui]\navatars = \"author\"\navatar_sources = \"hosting\"\navatar_fallback = \"initials\"\n",
        "pty-avatar-initials",
    );
    assert_eq!(glyph, 'G', "the first initial of the author");
}
