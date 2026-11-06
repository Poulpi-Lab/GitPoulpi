//! The integration security check, end to end: the config's address for a
//! profile no longer is the address it was trusted for. The real binary opens
//! the "Integration security check" dialog instead of sending the token, and
//! the fake hosting service (`xtask::mock_host`) proves nothing was sent to
//! either address.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the whole state.

mod common;

use std::time::Duration;

use gitpoulpi_config::ConfigPaths;
use gitpoulpi_hosting::trust::{IntegrationRecord, TrustStore};
use gitpoulpi_testkit::FixtureRepoBuilder;
use xtask::mock_host::{self, MockHost, Options};
use xtask::pty::Terminal;

const STARTUP: Duration = Duration::from_secs(60);
const STEP: Duration = Duration::from_secs(30);

struct Setup {
    term: Terminal,
    /// Where the profile was trusted.
    old: MockHost,
    /// Where the config points now.
    new: MockHost,
    config: tempfile::TempDir,
    _start: gitpoulpi_testkit::FixtureRepo,
}

fn mock() -> MockHost {
    mock_host::start(Options {
        port: 0,
        bare_root: None,
    })
    .unwrap()
}

/// A profile `hub` trusted for the first mock while the config says the
/// second, and the Clone dialog opened (it lists the profile's repositories,
/// which asks the host).
fn open(session: &str) -> Setup {
    let old = mock();
    let new = mock();
    let start = FixtureRepoBuilder::new()
        .unwrap()
        .commit_file("a.txt", "a\n", "trust-dialog-start-commit")
        .unwrap()
        .build();
    let config = tempfile::tempdir().unwrap();
    std::fs::write(
        config.path().join("config.toml"),
        format!(
            "[[integrations]]\nid = \"hub\"\nkind = \"github_enterprise\"\nurl = \"http://127.0.0.1:{}\"\ntoken_ref = \"integration:hub\"\nname = \"hub\"\nuser = {{ login = \"{}\" }}\n",
            new.port,
            mock_host::GITHUB_LOGIN,
        ),
    )
    .unwrap();
    let paths = ConfigPaths::for_dir(config.path());
    let file = TrustStore::file_in(&paths);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    TrustStore {
        integration: vec![IntegrationRecord {
            id: "hub".into(),
            origin: format!("http://127.0.0.1:{}", old.port),
            account_id: String::new(),
            account_login: String::new(),
            recorded_at: 1,
            account_checked_at: 0,
            allow_plain_http: false,
            tls: None,
        }],
        ai: Vec::new(),
    }
    .save(&file)
    .unwrap();
    let mut term = common::launch(start.root())
        .session(session)
        .config_dir(config.path())
        .size(150, 50)
        .env("GITPOULPI_NO_BROWSER", "1")
        .env("GITPOULPI_TOKEN_HUB", "hub-token")
        .spawn();
    term.wait_for("the first window", STARTUP, |s| {
        s.contains("trust-dialog-start-commit")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("C-o").expect("the management tab");
    term.wait_for("the management tab", STEP, |s| {
        s.contains("Repository Management") && s.contains("Clone")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("c").expect("Clone");
    term.wait_for("the Clone dialog with the source", STEP, |s| {
        s.contains("Clone a repository") && s.contains("GitHub \u{00B7} hub")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    // Picking the source asks the host for its repositories.
    let (col, row) = term.screen().find("GitHub \u{00B7} hub").unwrap();
    term.click(col + 2, row).expect("pick the source");
    Setup {
        term,
        old,
        new,
        config,
        _start: start,
    }
}

fn address_text(s: &Setup) -> String {
    format!(
        "hub now points to http://127.0.0.1:{}, it was http://127.0.0.1:{}.",
        s.new.port, s.old.port
    )
}

#[test]
fn a_changed_address_opens_the_dialog_and_nothing_is_sent_to_either_host() {
    let mut s = open("pty-trust-dialog");
    let want = address_text(&s);
    let screen = s
        .term
        .wait_for("the security check dialog", STEP, |x| {
            x.contains("Integration security check") && x.contains("2  Trust the new address")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    // The sentence may wrap: compare without line breaks.
    let flat = screen
        .text()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let wanted = want.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flat.contains(&wanted) || flat.contains("The token was not sent."),
        "{}",
        screen.text()
    );
    assert!(screen.contains("1  Disconnect"), "{}", screen.text());
    assert!(screen.contains("3  Cancel"), "{}", screen.text());
    assert!(
        s.new.requests().is_empty() && s.old.requests().is_empty(),
        "the token went nowhere: {:?} / {:?}",
        s.new.requests(),
        s.old.requests()
    );

    // Esc cancels: the dialog goes, the trusted address is unchanged.
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("the dialog gone", STEP, |x| {
            !x.contains("Integration security check")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let store =
        TrustStore::load(&TrustStore::file_in(&ConfigPaths::for_dir(s.config.path()))).unwrap();
    assert_eq!(
        store.integration[0].origin,
        format!("http://127.0.0.1:{}", s.old.port)
    );
    assert!(s.new.requests().is_empty());

    // The palette brings it back.
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("the Clone dialog closed", STEP, |x| {
            !x.contains("Clone a repository")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("back on the repository", STEP, |x| {
            x.contains("trust-dialog-start-commit") && !x.contains("Repository Management")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("C-p").unwrap();
    s.term.send_text("security check").unwrap();
    s.term
        .wait_for("the palette entry", STEP, |x| {
            x.contains("Integrations: Review security check")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the dialog again", STEP, |x| {
            x.contains("Integration security check")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(s.new.requests().is_empty());
}

/// The palette reaches the review from the Repository Management tab too:
/// cancel the check (it is snoozed), then ask for it back from the palette.
#[test]
fn the_palette_brings_the_check_back_on_the_management_tab() {
    let mut s = open("pty-trust-dialog-palette-home");
    s.term
        .wait_for("the security check dialog", STEP, |x| {
            x.contains("2  Trust the new address")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Escape").unwrap();
    let screen = s
        .term
        .wait_for("the dialog gone, Home still shown", STEP, |x| {
            !x.contains("2  Trust the new address") && x.contains("Repository Management")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        screen.contains("Repository Management"),
        "{}",
        screen.text()
    );
    // The Clone dialog the setup opened is still up: close it, so the
    // palette is asked from the plain Home tab.
    s.term.send_keys("Escape").unwrap();
    s.term
        .wait_for("the Clone dialog closed", STEP, |x| {
            !x.contains("Shallow clone") && x.contains("Repository Management")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("C-p").unwrap();
    s.term.send_text("security check").unwrap();
    s.term
        .wait_for("the palette row", STEP, |x| {
            x.contains("Review security check")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("Enter").unwrap();
    s.term
        .wait_for("the dialog back from the palette", STEP, |x| {
            x.contains("2  Trust the new address")
        })
        .unwrap_or_else(|e| panic!("{e}"));
}

#[test]
fn trusting_the_new_address_opens_the_form_on_the_token() {
    let mut s = open("pty-trust-dialog-trust");
    s.term
        .wait_for("the security check dialog", STEP, |x| {
            x.contains("2  Trust the new address")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    s.term.send_keys("2").unwrap();
    let screen = s
        .term
        .wait_for("the form on the token", STEP, |x| {
            x.contains("Re-entering the token is recommended.")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("Integration security check"),
        "{}",
        screen.text()
    );
    let store =
        TrustStore::load(&TrustStore::file_in(&ConfigPaths::for_dir(s.config.path()))).unwrap();
    assert_eq!(
        store.integration[0].origin,
        format!("http://127.0.0.1:{}", s.new.port)
    );
}
