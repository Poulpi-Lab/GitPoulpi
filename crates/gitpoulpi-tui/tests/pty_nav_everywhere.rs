//! `Ctrl+N` means the same thing on every surface: it shows a badge on the
//! surface's controls, the key after it presses one (a left click on it), `m`
//! arranges the dialog or popup in front and anything else cancels. The
//! palette, the pickers and the workspace search no longer take `Ctrl+N` as
//! "down", and `Alt+G` arranges whatever dialog is in front. Every wait
//! describes the whole state it expects (a terminal has no frame boundaries).

mod common;

use std::time::Duration;

use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};
use xtask::pty::{Screen, Terminal};

const STEP: Duration = Duration::from_secs(30);

fn repo() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "nav-everywhere-commit")
        .unwrap()
        .build()
}

fn launch_in(repo: &FixtureRepo, session: &str) -> (Terminal, tempfile::TempDir) {
    let config = tempfile::tempdir().unwrap();
    let term = common::launch(repo.root())
        .session(session)
        .config_dir(config.path())
        .size(200, 56)
        .no_exit_pause()
        .ready("nav-everywhere-commit")
        .mouse_capture()
        .spawn();
    (term, config)
}

fn wait(term: &mut Terminal, what: &str, f: impl FnMut(&Screen) -> bool) -> Screen {
    term.wait_for(what, STEP, f)
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()))
}

/// The status line names what `Ctrl+N` offers over a framed surface.
fn hint_up(s: &Screen) -> bool {
    s.contains("<badge>=") && s.contains("m=arrange (Esc to cancel)")
}

/// The badge character drawn just inside a popup's left border on `row`:
/// the cells ` X ` right after a `\u{2502}`.
fn badge_on_row(s: &Screen, row: u16) -> Option<char> {
    let line: Vec<char> = s.line(row).chars().collect();
    line.windows(4).find_map(|w| {
        (w[0] == '\u{2502}' && w[1] == ' ' && w[2].is_ascii_alphanumeric() && w[3] == ' ')
            .then_some(w[2])
    })
}

fn title_at(s: &Screen, title: &str) -> Option<(u16, u16)> {
    s.find_all(title).into_iter().min_by_key(|(c, r)| (*r, *c))
}

/// Opens the palette and filters it to the two `Dialog:` arrange rows.
fn palette_on_dialog_rows(term: &mut Terminal) -> Screen {
    term.send_keys("C-p").unwrap();
    wait(term, "the palette", |s| {
        title_at(s, "Command Palette").is_some()
    });
    term.send_text("dialog: ").unwrap();
    wait(term, "the two Dialog rows", |s| {
        s.contains("Dialog: Move or Resize") && s.contains("Dialog: Reset Position and Size")
    })
}

#[test]
fn ctrl_n_in_the_palette_badges_it_a_badge_focuses_its_target() {
    let repo = repo();
    let (mut term, _config) = launch_in(&repo, "pty-nav-everywhere-palette");
    let s = palette_on_dialog_rows(&mut term);
    let (_, move_row) = s.find("Dialog: Move or Resize").unwrap();
    let (_, reset_row) = s.find("Dialog: Reset Position and Size").unwrap();
    // Down puts the highlight on the second row (whichever it is).
    term.send_keys("Down").unwrap();
    let first_row = move_row.min(reset_row);
    term.send_keys("C-n").unwrap();
    let s = wait(
        &mut term,
        "the hint and a badge on the first list row",
        |s| hint_up(s) && badge_on_row(s, first_row).is_some(),
    );
    // The badge of the list presses its first row: the highlight goes back to
    // it, and Enter runs that row.
    let badge = badge_on_row(&s, first_row).unwrap();
    term.send_text(&badge.to_string()).unwrap();
    wait(&mut term, "the badges gone", |s| !hint_up(s));
    term.send_keys("Enter").unwrap();
    if first_row == move_row {
        wait(&mut term, "the first row ran: the arrange banner", |s| {
            s.contains("Arrange:")
        });
    } else {
        wait(&mut term, "the first row ran: the palette closed", |s| {
            title_at(s, "Command Palette").is_none()
        });
    }
    term.stop().unwrap();
}

#[test]
fn ctrl_n_then_m_arranges_the_palette_and_ctrl_n_no_longer_moves_the_highlight() {
    let repo = repo();
    let (mut term, _config) = launch_in(&repo, "pty-nav-everywhere-palette-m");
    let s = palette_on_dialog_rows(&mut term);
    let (tc, tr) = title_at(&s, "Command Palette").unwrap();
    // Ctrl+N then Esc: the badges go, the palette stays open.
    term.send_keys("C-n").unwrap();
    wait(&mut term, "the hint", hint_up);
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the badges cancelled, the palette open", |s| {
        !hint_up(s) && title_at(s, "Command Palette").is_some()
    });
    // Ctrl+N then m: the arrange mode of the palette, which stays open.
    term.send_keys("C-n").unwrap();
    wait(&mut term, "the hint", hint_up);
    term.send_text("m").unwrap();
    wait(&mut term, "the arrange banner", |s| s.contains("Arrange:"));
    term.send_keys("Right").unwrap();
    wait(&mut term, "the palette two columns right", |s| {
        title_at(s, "Command Palette") == Some((tc + 2, tr)) && s.contains("Arrange:")
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the mode left, the palette open", |s| {
        !s.contains("Arrange:") && title_at(s, "Command Palette") == Some((tc + 2, tr))
    });
    term.stop().unwrap();
}

/// `Ctrl+N` over a dialog that badges nothing of its own shows its controls
/// (the close cross at least) and the hint before `m` arranges it.
fn badges_then_m(term: &mut Terminal, title: &str) {
    let s = wait(term, "the dialog with its close cross", |s| {
        title_at(s, title).is_some_and(|(_, r)| s.line(r).contains('\u{2715}'))
    });
    let (tc, tr) = title_at(&s, title).unwrap();
    term.send_keys("C-n").unwrap();
    // The close cross is the first control: its badge replaces it.
    wait(term, "the close cross badged and the hint naming m", |s| {
        hint_up(s) && !s.line(tr).contains('\u{2715}') && s.line(tr).contains(" 1 ")
    });
    term.send_text("m").unwrap();
    wait(term, "the arrange banner", |s| {
        s.contains("Arrange:") && !hint_up(s)
    });
    term.send_keys("Right").unwrap();
    wait(term, "the dialog two columns right", |s| {
        title_at(s, title) == Some((tc + 2, tr)) && s.contains("Arrange:")
    });
    term.send_keys("Escape").unwrap();
    wait(term, "the mode left, the dialog kept", |s| {
        !s.contains("Arrange:") && title_at(s, title) == Some((tc + 2, tr))
    });
    term.send_keys("Escape").unwrap();
    wait(term, "the dialog closed", |s| title_at(s, title).is_none());
}

#[test]
fn ctrl_n_shows_badges_then_m_arranges_a_move_only_and_a_resizable_dialog() {
    let repo = repo();
    let (mut term, _config) = launch_in(&repo, "pty-nav-everywhere-dialogs");
    // About: move only.
    term.send_keys("C-n").unwrap();
    term.send_text("a").unwrap();
    badges_then_m(&mut term, "About GitPoulpi");
    // The background jobs panel: resizable.
    term.send_keys("C-n").unwrap();
    term.send_text("J").unwrap();
    badges_then_m(&mut term, "Background jobs");
    term.stop().unwrap();
}

#[test]
fn alt_g_arranges_the_dialog_in_front_whatever_it_is() {
    let repo = repo();
    let (mut term, _config) = launch_in(&repo, "pty-nav-everywhere-altg");
    term.send_keys("C-n").unwrap();
    term.send_text("a").unwrap();
    let s = wait(&mut term, "the About window", |s| {
        title_at(s, "About GitPoulpi").is_some()
    });
    let (tc, tr) = title_at(&s, "About GitPoulpi").unwrap();
    term.send_keys("M-g").unwrap();
    wait(&mut term, "the arrange banner", |s| s.contains("Arrange:"));
    term.send_keys("Right").unwrap();
    wait(&mut term, "About two columns right", |s| {
        title_at(s, "About GitPoulpi") == Some((tc + 2, tr)) && s.contains("Arrange:")
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the mode left, About kept", |s| {
        !s.contains("Arrange:") && title_at(s, "About GitPoulpi") == Some((tc + 2, tr))
    });
    term.stop().unwrap();
}

#[test]
fn ctrl_n_badges_a_picker_and_the_help_overlay() {
    let repo = repo();
    let (mut term, _config) = launch_in(&repo, "pty-nav-everywhere-picker");
    // The theme picker, opened by its own key: its filter box and list.
    term.send_keys("C-n").unwrap();
    term.send_text("t").unwrap();
    wait(&mut term, "the theme list", |s| {
        s.contains("System (follows terminal)")
    });
    term.send_keys("C-n").unwrap();
    wait(&mut term, "the picker's badges and hint", |s| {
        hint_up(s) && s.contains(" 1 ") && s.contains(" 2 ") && s.contains(" 3 ")
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the badges cancelled, the picker open", |s| {
        !hint_up(s) && s.contains("System (follows terminal)")
    });
    term.send_keys("Escape").unwrap();
    wait(&mut term, "the picker closed", |s| {
        !s.contains("System (follows terminal)")
    });
    // The help overlay: filter box, tabs, body, link.
    term.send_text("?").unwrap();
    wait(&mut term, "the help overlay", |s| {
        s.contains("Keybindings - type to filter")
    });
    term.send_keys("C-n").unwrap();
    wait(&mut term, "the help overlay's badges and hint", |s| {
        hint_up(s) && s.contains(" 2 ")
    });
    term.stop().unwrap();
}

#[cfg(feature = "global-search")]
#[test]
fn ctrl_n_badges_the_workspace_search() {
    let repo = repo();
    let (mut term, _config) = launch_in(&repo, "pty-nav-everywhere-search");
    term.send_keys("M-C-f").unwrap();
    wait(&mut term, "the search popup", |s| {
        s.contains("Search open repositories")
    });
    term.send_keys("C-n").unwrap();
    wait(&mut term, "the search's badges and hint", |s| {
        hint_up(s) && s.contains(" 1 ") && s.contains(" 2 ") && s.contains(" 3 ")
    });
    term.send_text("m").unwrap();
    wait(&mut term, "the arrange banner", |s| s.contains("Arrange:"));
    term.stop().unwrap();
}
