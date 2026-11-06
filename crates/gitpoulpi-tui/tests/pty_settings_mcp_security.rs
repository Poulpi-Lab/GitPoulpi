//! Settings > Agents & MCP, the MCP HTTP endpoint's security rows: the
//! authentication switch (with its danger warning), the access token (masked,
//! Create, Show, Copy, Rotate with its confirmation, and the keyboard twin of
//! the buttons), the TLS choice and the "behind a tunnel or reverse proxy"
//! switch. Real binary through a pty; the config dir is a temp dir.
#![cfg(feature = "mcp")]

mod common;

use std::path::Path;
use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::pty::Terminal;

const STEP: Duration = Duration::from_secs(30);

fn stored(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("config.toml")).unwrap_or_default()
}

fn wait_for_file(path: &Path, what: &str) -> String {
    let deadline = std::time::Instant::now() + STEP;
    loop {
        if let Ok(text) = std::fs::read_to_string(path) {
            if !text.trim().is_empty() {
                return text.trim().to_string();
            }
        }
        assert!(std::time::Instant::now() < deadline, "{what} never written");
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn wait_stored(dir: &Path, needle: &str) {
    let deadline = std::time::Instant::now() + STEP;
    while !stored(dir).contains(needle) {
        assert!(
            std::time::Instant::now() < deadline,
            "{needle:?} never reached config.toml: {}",
            stored(dir)
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// The token row's line, scrolled into view.
fn token_line(term: &mut Terminal) -> String {
    common::scroll_settings_to(term, "Access token", STEP);
    term.text()
        .lines()
        .find(|l| l.contains("Access token") && !l.contains("Clients send"))
        .unwrap_or_default()
        .to_string()
}

/// Clicks the first cell of `text` on the token row.
fn click_on_token_row(term: &mut Terminal, text: &str) {
    common::scroll_settings_to(term, "Access token", STEP);
    let screen = term.screen();
    let (_, row) = screen
        .find_all("Access token")
        .into_iter()
        .find(|&(c, _)| c > 28)
        .expect("the token row");
    let (col, _) = screen
        .find_all(text)
        .into_iter()
        .find(|&(_, r)| r == row)
        .unwrap_or_else(|| panic!("no {text:?} on the token row\n{}", term.text()));
    term.click(col + 1, row).expect("click the button");
}

#[test]
fn the_security_rows_work_by_mouse_and_keyboard() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("mcp sec base")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    let dump = tempfile::tempdir().unwrap();
    let copied = dump.path().join("clipboard.txt");
    let mut term = common::launch(repo.root())
        .session("pty-settings-mcp-security")
        .config_dir(config.path())
        .env("GITPOULPI_CLIPBOARD_DUMP", &copied)
        .size(200, 100)
        .ready("mcp sec base")
        .spawn();
    let (gear_col, gear_row) = term.wait_for_text("\u{2699}", STEP).unwrap();
    term.click(gear_col, gear_row).expect("click the gear");
    term.wait_for("Settings open", STEP, |s| {
        s.contains("Repo-specific settings")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    common::settings_page(&mut term, "Agents & MCP");
    let token_file = config.path().join("data/private/mcp-http.token");

    // Authentication is on by default; turning it off warns.
    let (col, row) = common::scroll_settings_to(&mut term, "HTTP authentication", STEP);
    assert!(!term.screen().contains("Authentication is off"));
    term.click(col + 1, row).expect("click the switch");
    wait_stored(config.path(), "http_auth = false");
    term.wait_for("the danger warning", STEP, |s| {
        s.contains("Authentication is off")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.click(col + 1, row).expect("click the switch again");
    wait_stored(config.path(), "http_auth = true");

    // No token yet: Create makes one, masked on screen.
    assert!(
        token_line(&mut term).contains("(none yet)"),
        "{}",
        term.text()
    );
    click_on_token_row(&mut term, "Create");
    let token = wait_for_file(&token_file, "the token");
    let line = loop {
        let line = token_line(&mut term);
        if !line.contains("(none yet)") {
            break line;
        }
        std::thread::sleep(Duration::from_millis(200));
    };
    assert!(line.contains("..."), "{line}");
    assert!(!term.text().contains(&token), "masked\n{}", term.text());

    // The masked value is the Show button; the keyboard twin is Enter on the
    // focused button.
    click_on_token_row(&mut term, "...");
    term.wait_for("the full token", STEP, |s| s.contains(&token))
        .unwrap_or_else(|e| panic!("{e}"));
    click_on_token_row(&mut term, "...");
    term.wait_for("the token hidden again", STEP, |s| !s.contains(&token))
        .unwrap_or_else(|e| panic!("{e}"));

    // Copy: the clipboard holds the whole token, the status line none of it.
    click_on_token_row(&mut term, "Copy");
    let deadline = std::time::Instant::now() + STEP;
    while std::fs::read_to_string(&copied).unwrap_or_default() != token {
        assert!(std::time::Instant::now() < deadline, "nothing copied");
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(!term.text().contains(&token[..12]), "{}", term.text());

    // Rotate asks first; Cancel (focused) keeps the token.
    click_on_token_row(&mut term, "Rotate");
    term.wait_for("the confirmation", STEP, |s| s.contains("Cancel"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(wait_for_file(&token_file, "the token"), token);

    // Confirming replaces it. Keyboard: the dialog's buttons by Left/Right.
    click_on_token_row(&mut term, "Rotate");
    term.wait_for("the confirmation", STEP, |s| s.contains("Cancel"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Right").unwrap();
    term.send_keys("Enter").unwrap();
    let deadline = std::time::Instant::now() + STEP;
    while wait_for_file(&token_file, "the token") == token {
        assert!(std::time::Instant::now() < deadline, "token not rotated");
        std::thread::sleep(Duration::from_millis(200));
    }

    // TLS choice and the proxy statement.
    let (col, row) =
        common::scroll_settings_to(&mut term, "Behind a tunnel or reverse proxy", STEP);
    term.click(col + 1, row).expect("click the proxy switch");
    wait_stored(config.path(), "behind_proxy = true");
    term.wait_for("the proxy warning", STEP, |s| {
        s.contains("plain text on that network")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    let (col, row) = common::scroll_settings_to_choice(&mut term, "Off", STEP);
    term.click(col, row).expect("open the TLS choice");
    let screen = term
        .wait_for("the TLS options", STEP, |s| s.contains("\u{2713} Off"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Off (ticked), then Self-signed one row below.
    let (tc, tr) = screen.find("\u{2713} Off").unwrap();
    term.click(tc + 2, tr + 1).expect("pick self-signed");
    wait_stored(config.path(), "tls = \"self_signed\"");
}
