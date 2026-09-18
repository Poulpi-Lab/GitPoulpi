//! Settings > General > Launcher: the command alias and "Add GitPoulpi to
//! PATH" rows. HOME, XDG_* and the config directory point at scratch
//! directories, so nothing here can reach the real `~/.local/bin` or the real
//! shell startup files.

#![cfg(unix)]

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);
const MESSAGE: &str = "alias probe commit";

/// The "Alias status" row's value is `value` (the shortcut rows around it
/// also say "not created").
fn alias_status_is(screen: &str, value: &str) -> bool {
    screen.lines().any(|l| {
        // Label column, then value column: `Alias status    created`.
        l.find("Alias status").is_some_and(|i| {
            l[i + "Alias status".len()..].trim_matches(|c: char| c == ' ' || c == '\u{2502}')
                == value
        })
    })
}

fn palette(term: &mut Terminal, query: &str, entry: &str) {
    term.send_keys("C-p").unwrap();
    term.send_text(query).unwrap();
    term.wait_for("the palette entry", STEP, |s| s.contains(entry))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
}

fn wait_until(what: &str, term: &Terminal, ok: impl Fn() -> bool) {
    let deadline = std::time::Instant::now() + STARTUP;
    while !ok() {
        assert!(
            std::time::Instant::now() < deadline,
            "{what}\n{}",
            term.text()
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

#[test]
fn create_gp_from_the_palette_then_remove_it_from_the_row() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\ncommand_alias = \"gp\"\n",
    )
    .unwrap();
    let mut term = common::launch(repo.root())
        .session("command-alias")
        .config_dir(config.path())
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("HOME", home.path())
        .env("XDG_DATA_HOME", home.path().join("data"))
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .size(200, 70)
        .no_exit_pause()
        .ready(MESSAGE)
        .spawn();
    let link = home.path().join(".local/bin/gp");
    assert!(std::fs::symlink_metadata(&link).is_err());

    palette(&mut term, "command alias", "Launcher: Create Command Alias");
    // The page scrolled to the acted-on button: on a 70-row terminal the
    // Launcher group sits below the fold otherwise.
    term.wait_for("the alias row reads created", STARTUP, |s| {
        s.contains("Remove: command alias") && alias_status_is(&s.text(), "created")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let target = std::fs::read_link(&link).expect("the alias is a symlink");
    assert_eq!(
        target.file_name().unwrap().to_string_lossy(),
        "gitpoulpi-tui",
        "it points at the running TUI: {}",
        target.display()
    );
    assert!(target.exists(), "{}", target.display());

    // The palette selected the Remove button already, so one click runs it;
    // it deletes only what we made.
    term.act_until(
        "the removal confirmation",
        Duration::from_secs(5),
        STEP,
        |t| {
            if let Some((col, row)) = t.screen().find("Remove: command alias") {
                t.click(col + 1, row)?;
            }
            Ok(())
        },
        |s| s.contains("This cannot be") && s.contains("Cancel"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    common::confirm_removal(&mut term, STEP);
    wait_until("the link is still there", &term, || {
        std::fs::symlink_metadata(&link).is_err()
    });
    let _ = term.stop();
}

#[test]
fn pick_the_name_in_the_combobox_then_add_and_remove_gitpoulpi_on_path() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit(MESSAGE)
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        "[ui]\ngraph_renderer = \"text\"\n",
    )
    .unwrap();
    std::fs::write(home.path().join(".bashrc"), "export KEEP=1\n").unwrap();
    let mut term = common::launch(repo.root())
        .session("command-alias-path")
        .config_dir(config.path())
        .env("GITPOULPI_DESKTOP_NO_HOOKS", "1")
        .env("HOME", home.path())
        .env("XDG_DATA_HOME", home.path().join("data"))
        .env("XDG_CONFIG_HOME", home.path().join("config"))
        .size(200, 70)
        .no_exit_pause()
        .ready(MESSAGE)
        .spawn();
    let link = home.path().join(".local/bin/gp");
    let rc = home.path().join(".bashrc");

    // No name yet: the palette command lands on the alias row and creates
    // nothing.
    palette(&mut term, "command alias", "Launcher: Create Command Alias");
    term.wait_for("the page on the alias row", STEP, |s| {
        s.contains("Command alias") && common::choice_at(s, "None").is_some()
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    assert!(std::fs::symlink_metadata(&link).is_err());

    // Pick gp in the dropdown (never a cycle), then press Create.
    term.act_until(
        "the alias list",
        Duration::from_secs(5),
        STEP,
        |t| {
            if let Some((vc, vr)) = common::choice_at(&t.screen(), "None") {
                t.click(vc + 1, vr)?;
            }
            Ok(())
        },
        |s| s.contains("\u{2713} None"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // A combobox that takes typed names selects on the first click and
    // picks on the second one within the double-click window (same as
    // Enter), so a double click. The row under None is located afresh for
    // every attempt, and it is only sent while the list is still up, so a
    // slow first pick is never clicked through; a TUI too stalled to see the
    // pair as a double click gets another one.
    term.act_until(
        "the name is gp",
        Duration::from_secs(3),
        STEP,
        |t| {
            if let Some((tc, tr)) = t.screen().find("\u{2713} None") {
                t.double_click(tc + 3, tr + 1)?; // gp, the row under None
            }
            Ok(())
        },
        |s| common::choice_at(s, "gp").is_some(),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    // Press Create by clicking it, located afresh every attempt: keys
    // (Down, Down, Enter) are not state-checked, so under load one landing
    // while the dropdown was still closing selected another row and the
    // alias was never created. The click is only repeated while no link
    // exists.
    // The button sits below the fold until the page is scrolled to it.
    common::scroll_settings_to(&mut term, "Create: command alias", STEP);
    let link_made = link.clone();
    term.act_until(
        "the alias was never created",
        Duration::from_secs(5),
        STARTUP,
        |t| {
            if std::fs::symlink_metadata(&link).is_err() {
                if let Some((cc, cr)) = t.screen().find("Create: command alias") {
                    t.click(cc + 1, cr)?;
                }
            }
            Ok(())
        },
        |_| std::fs::symlink_metadata(&link_made).is_ok(),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // PATH: the palette twin (from the main window) writes a marked block,
    // and only that.
    term.send_keys("Esc").unwrap();
    term.wait_for("Settings closed", STEP, |s| !s.contains("General - global"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette(&mut term, "add to path", "Launcher: Add to PATH");
    wait_until("no block in .bashrc", &term, || {
        std::fs::read_to_string(&rc).is_ok_and(|t| t.contains("X-GitPoulpi=1"))
    });
    let text = std::fs::read_to_string(&rc).unwrap();
    assert!(text.starts_with("export KEEP=1\n"), "{text}");
    assert!(
        text.contains(&home.path().join(".local/bin").display().to_string()),
        "{text}"
    );
    term.wait_for("the PATH row reads added", STEP, |s| {
        s.contains("added (new terminals)")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    term.send_keys("Esc").unwrap();
    term.wait_for("Settings closed", STEP, |s| !s.contains("General - global"))
        .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    palette(&mut term, "remove from path", "Launcher: Remove from PATH");
    wait_until("the block is still there", &term, || {
        std::fs::read_to_string(&rc).is_ok_and(|t| t == "export KEEP=1\n")
    });
    let _ = term.stop();
}
