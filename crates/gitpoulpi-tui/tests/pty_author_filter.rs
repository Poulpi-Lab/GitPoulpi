//! The Log's author filter: the funnel in the Author header opens a popup
//! (search field, Clear filter row, one row per author with one avatar), the
//! chosen authors' commits stay lit and every other row dims exactly as the
//! graph search dims, a text search narrows it further, and the palette
//! reaches all of it. Three generic authors on a temp fixture repository,
//! temp config dir, no network (the initials badge is the avatar; terminal
//! images are off).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const FUNNEL: &str = "\u{25BD}";
const FUNNEL_ON: &str = "\u{25BC}";

/// Alice and Bob two commits each, Carol one; newest first on screen:
/// epsilon (Bob), delta (Alice), gamma (Carol), beta (Bob), alpha (Alice).
fn fixture() -> FixtureRepo {
    let mut b = FixtureRepoBuilder::new().unwrap();
    for (who, email, message) in [
        ("Alice Example", "alice@example.com", "alpha init"),
        ("Bob Example", "bob@example.com", "beta fix"),
        ("Carol Example", "carol@example.com", "gamma add"),
        ("Alice Example", "alice@example.com", "delta more"),
        ("Bob Example", "bob@example.com", "epsilon work"),
    ] {
        b = b
            .config("user.name", who)
            .unwrap()
            .config("user.email", email)
            .unwrap()
            .commit(message)
            .unwrap();
    }
    b.build()
}

fn launch(repo: &FixtureRepo, config: &std::path::Path, session: &str) -> Terminal {
    std::fs::write(config.join("config.toml"), "[ui]\navatars = \"author\"\n").unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config)
        .size(200, 50)
        .spawn();
    term.wait_for("the fixture loaded", STARTUP, |s| {
        s.contains("epsilon work") && s.contains("Author")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term
}

/// Where the funnel is drawn (`glyph`: outline or filled), one cell right of
/// the "Author" title.
fn funnel_at(s: &Screen, glyph: &str) -> Option<(u16, u16)> {
    s.find(&format!("Author {glyph}")).map(|(c, r)| (c + 7, r))
}

fn cell_of(s: &Screen, text: &str) -> Option<xtask::pty::Cell> {
    s.find(text).and_then(|(c, r)| s.cell(c, r).cloned())
}

/// A row is dimmed when its message is neither bold nor in the full text
/// color the lit rows have (the emulator has no DIM attribute: the search's
/// own PTY test asserts the same way).
fn dimmed(s: &Screen, lit: &str, other: &str) -> Option<bool> {
    let (lit, other) = (cell_of(s, lit)?, cell_of(s, other)?);
    Some(other.fg != lit.fg && !other.bold)
}

fn open_with_funnel(term: &mut Terminal) {
    let s = term
        .wait_for("the funnel", STEP, |s| funnel_at(s, FUNNEL).is_some())
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = funnel_at(&s, FUNNEL).unwrap();
    common::retry_until(
        term,
        "the author popup",
        6,
        Duration::from_secs(4),
        |t| t.click(c, r).unwrap(),
        |s| s.contains("Clear filter"),
    );
}

/// The popup's author rows (below `Clear filter`), top to bottom.
fn author_rows(s: &Screen) -> Vec<String> {
    let (_, clear) = s.find("Clear filter").expect("the Clear filter row");
    (1..=3).map(|i| s.line(clear + i)).collect()
}

#[test]
fn the_funnel_opens_a_popup_with_one_avatar_per_author_and_a_pick_dims_the_rest() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&repo, config.path(), "author-filter-mouse");
    open_with_funnel(&mut term);
    let s = term.screen();
    assert!(
        s.contains("filter by author"),
        "the placeholder:\n{}",
        s.text()
    );
    let rows = author_rows(&s);
    // One avatar (the initials badge) before each name, the email muted after
    // it, most commits first and then by name.
    for (row, (badge, name, email)) in rows.iter().zip([
        ("AE", "Alice Example", "alice@example.com"),
        ("BE", "Bob Example", "bob@example.com"),
        ("CE", "Carol Example", "carol@example.com"),
    ]) {
        assert!(
            row.contains(&format!("{badge} {name}")) && row.contains(email),
            "{row:?}"
        );
        assert_eq!(row.matches(badge).count(), 1, "one avatar only: {row:?}");
    }
    // Nothing is dimmed yet.
    assert_eq!(dimmed(&s, "gamma add", "delta more"), Some(false));

    // Click Carol: ticked, the popup stays open, the other authors dim.
    let (_, clear) = s.find("Clear filter").unwrap();
    let (name_col, _) = s.find("Carol Example").unwrap();
    // The popup row, not the Log's own: it is the one under Clear filter.
    let carol_row = clear + 3;
    term.click(name_col.max(1), carol_row).unwrap();
    let s = term
        .wait_for("Carol ticked and the others dimmed", STEP, |s| {
            s.contains("Clear filter")
                && s.line(clear + 3).contains('\u{2713}')
                && dimmed(s, "gamma add", "delta more") == Some(true)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(dimmed(&s, "gamma add", "beta fix"), Some(true));

    // Esc closes the popup and keeps the filter: the funnel is filled.
    term.send_keys("Escape").unwrap();
    let s = term
        .wait_for("the popup closed, the funnel lit", STEP, |s| {
            !s.contains("Clear filter") && funnel_at(s, FUNNEL_ON).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(dimmed(&s, "gamma add", "delta more"), Some(true));

    // Reopen: Clear filter restores the Log and closes the popup.
    let (c, r) = funnel_at(&s, FUNNEL_ON).unwrap();
    common::retry_until(
        &mut term,
        "the popup again",
        6,
        Duration::from_secs(4),
        |t| t.click(c, r).unwrap(),
        |s| s.contains("Clear filter"),
    );
    let s = term.screen();
    let (cc, cr) = s.find("Clear filter").unwrap();
    term.click(cc + 2, cr).unwrap();
    term.wait_for("the filter cleared", STEP, |s| {
        !s.contains("Clear filter")
            && funnel_at(s, FUNNEL).is_some()
            && dimmed(s, "gamma add", "delta more") == Some(false)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn the_keyboard_and_the_palette_reach_the_whole_filter() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&repo, config.path(), "author-filter-keys");
    // The palette opens the popup; typing narrows it.
    term.send_keys("C-p").unwrap();
    term.send_text("filter by author").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Log: Filter by Author...")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the popup", STEP, |s| s.contains("Clear filter"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("bob").unwrap();
    let s = term
        .wait_for("the list narrowed to Bob", STEP, |s| {
            s.contains("Clear filter")
                && !s
                    .line(s.find("Clear filter").unwrap().1 + 1)
                    .contains("Alice")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(
        author_rows(&s)[0].contains("Bob Example"),
        "{}",
        author_rows(&s)[0]
    );
    // Enter toggles the highlighted author; Esc closes and keeps the filter.
    term.send_keys("Enter").unwrap();
    term.wait_for("Bob ticked", STEP, |s| {
        s.find("Clear filter")
            .is_some_and(|(_, r)| s.line(r + 1).contains('\u{2713}'))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The first Esc empties the typed text, the second closes.
    term.send_keys("Escape").unwrap();
    term.send_keys("Escape").unwrap();
    term.send_keys("Escape").unwrap();
    let s = term
        .wait_for("the funnel lit and Alice's row dimmed", STEP, |s| {
            !s.contains("Clear filter")
                && funnel_at(s, FUNNEL_ON).is_some()
                && dimmed(s, "epsilon work", "delta more") == Some(true)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(dimmed(&s, "epsilon work", "beta fix"), Some(false));

    // The palette clears it.
    term.send_keys("C-p").unwrap();
    term.send_text("clear author filter").unwrap();
    term.wait_for("the palette entry", STEP, |s| {
        s.contains("Log: Clear Author Filter")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the filter cleared", STEP, |s| {
        s.contains("Author filter cleared")
            && funnel_at(s, FUNNEL).is_some()
            && dimmed(s, "epsilon work", "delta more") == Some(false)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn a_text_search_narrows_the_filter_and_a_right_click_on_the_funnel_clears_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&repo, config.path(), "author-filter-search");
    open_with_funnel(&mut term);
    // Choose Bob (the second row).
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("Bob ticked", STEP, |s| {
        s.find("Clear filter")
            .is_some_and(|(_, r)| s.line(r + 2).contains('\u{2713}'))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the popup closed, the funnel lit", STEP, |s| {
        !s.contains("Clear filter") && funnel_at(s, FUNNEL_ON).is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // "alpha" is Alice's commit: with Bob chosen nothing matches both (AND).
    term.send_text("f").unwrap();
    term.send_text("alpha").unwrap();
    term.wait_for("no hit under both", STEP, |s| s.contains("0 of 0"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // "beta" is Bob's: one hit.
    term.send_keys("C-u").unwrap();
    term.send_text("beta").unwrap();
    term.wait_for("one hit", STEP, |s| s.contains("1 of 1"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    // A right-click on the lit funnel drops the filter.
    let s = term
        .wait_for("the bar closed", STEP, |s| {
            funnel_at(s, FUNNEL_ON).is_some() && !s.contains("1 of 1")
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let (c, r) = funnel_at(&s, FUNNEL_ON).unwrap();
    common::retry_until(
        &mut term,
        "the filter cleared by the right-click",
        6,
        Duration::from_secs(4),
        |t| t.right_click(c, r).unwrap(),
        |s| funnel_at(s, FUNNEL).is_some(),
    );
}

#[test]
fn the_filter_is_remembered_per_repository_across_a_restart() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&repo, config.path(), "author-filter-persist");
    open_with_funnel(&mut term);
    term.send_keys("Enter").unwrap();
    term.wait_for("Alice ticked", STEP, |s| {
        s.find("Clear filter")
            .is_some_and(|(_, r)| s.line(r + 1).contains('\u{2713}'))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.stop().unwrap();

    let term = launch(&repo, config.path(), "author-filter-persist");
    term.wait_for("the funnel lit again", STEP, |s| {
        funnel_at(s, FUNNEL_ON).is_some() && dimmed(s, "delta more", "gamma add") == Some(true)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let s = term.screen();
    // Alice's commits are lit, Bob's and Carol's dimmed.
    assert_eq!(dimmed(&s, "delta more", "beta fix"), Some(true));
}

#[test]
fn a_narrow_log_still_says_the_filter_is_on_and_the_key_still_opens_it() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&repo, config.path(), "author-filter-narrow");
    open_with_funnel(&mut term);
    term.send_keys("Enter").unwrap();
    term.wait_for("Alice ticked", STEP, |s| {
        s.find("Clear filter")
            .is_some_and(|(_, r)| s.line(r + 1).contains('\u{2713}'))
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Escape").unwrap();
    // Too narrow for the Author column: it gives way, the Log's border carries
    // the note, and Shift+A (Log focus) still opens the popup.
    term.resize(70, 40).unwrap();
    term.wait_for("the note on the Log's border", STEP, |s| {
        s.contains("Author: Alice Example") && !s.contains("Author \u{25BC}")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("A").unwrap();
    term.wait_for("the popup by key", STEP, |s| s.contains("Clear filter"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

/// The Log row whose message cell carries the selection background: the only
/// row of the four whose background differs from the others.
fn selected_message(s: &Screen) -> Option<&'static str> {
    let names = [
        "epsilon work",
        "delta more",
        "gamma add",
        "beta fix",
        "alpha init",
    ];
    let bgs: Vec<_> = names
        .iter()
        .map(|n| cell_of(s, n).map(|c| c.bg))
        .collect::<Option<_>>()?;
    let odd: Vec<_> = (0..names.len())
        .filter(|&i| bgs.iter().filter(|b| **b == bgs[i]).count() == 1)
        .collect();
    (odd.len() == 1).then(|| names[odd[0]])
}

#[test]
fn the_arrows_step_between_lit_commits_while_the_author_filter_is_on() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&repo, config.path(), "author-filter-arrows");
    open_with_funnel(&mut term);
    // Choose Alice: delta and alpha stay lit, with Carol's and Bob's rows between.
    term.send_keys("Enter").unwrap();
    term.wait_for("Alice ticked", STEP, |s| {
        s.find("Clear filter")
            .is_some_and(|(_, r)| s.line(r + 1).contains('\u{2713}'))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the funnel lit", STEP, |s| {
        !s.contains("Clear filter") && dimmed(s, "delta more", "gamma add") == Some(true)
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // The selection sits on the dimmed HEAD row (epsilon): Down goes to the
    // nearest lit commit, the next Down skips gamma and beta, the last stays.
    term.send_keys("Down").unwrap();
    term.wait_for("delta selected", STEP, |s| {
        selected_message(s) == Some("delta more")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Down").unwrap();
    term.wait_for("alpha selected, skipping the dimmed rows", STEP, |s| {
        selected_message(s) == Some("alpha init")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Down").unwrap();
    term.send_keys("Up").unwrap();
    term.wait_for("Up goes back to delta", STEP, |s| {
        selected_message(s) == Some("delta more")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
}

#[test]
fn esc_in_the_popup_clears_the_text_then_closes_and_esc_in_the_log_drops_the_filter() {
    let repo = fixture();
    let config = tempfile::tempdir().unwrap();
    let mut term = launch(&repo, config.path(), "author-filter-esc");
    open_with_funnel(&mut term);
    // Choose Bob, then type text: the first Esc empties the field and keeps
    // the popup open, the second closes it with the filter kept.
    term.send_keys("Down").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("Bob ticked", STEP, |s| {
        s.find("Clear filter")
            .is_some_and(|(_, r)| s.line(r + 2).contains('\u{2713}'))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_text("carol").unwrap();
    term.wait_for("the list narrowed", STEP, |s| {
        s.find("Clear filter")
            .is_some_and(|(_, r)| !s.line(r + 1).contains("Alice"))
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    term.wait_for("the text cleared, the popup still open", STEP, |s| {
        s.contains("Clear filter") && s.contains("filter by author")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.send_keys("Escape").unwrap();
    let s = term
        .wait_for("the popup closed, the funnel lit", STEP, |s| {
            !s.contains("Clear filter") && funnel_at(s, FUNNEL_ON).is_some()
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(dimmed(&s, "epsilon work", "delta more"), Some(true));
    // Esc in the Log drops the filter: funnel back to normal, every commit lit.
    term.send_keys("Escape").unwrap();
    let s = term
        .wait_for("the filter dropped by Esc", STEP, |s| {
            funnel_at(s, FUNNEL).is_some() && dimmed(s, "epsilon work", "delta more") == Some(false)
        })
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert_eq!(dimmed(&s, "gamma add", "beta fix"), Some(false));
    assert!(s.contains("Author filter cleared"), "{}", s.text());
}
