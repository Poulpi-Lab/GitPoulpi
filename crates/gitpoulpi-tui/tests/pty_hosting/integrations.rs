use std::time::Duration;

use crate::mock::*;
use crate::support::*;

/// A profile that is not connected opens in its form, which must offer
/// Remove.
#[test]
fn a_disconnected_profile_can_be_removed_from_settings_and_stays_removed() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let config = format!(
        "{}\n[[integrations]]\nid = \"gitlab\"\nkind = \"gitlab\"\nurl = \"https://gitlab.com\"\ntoken_ref = \"integration:gitlab\"\nname = \"gitlab\"\n",
        config_toml_ui(mock.port, "", "auto")
    );
    let (mut term, dir) = launch(
        &repo,
        "pty-hosting-remove",
        &config,
        COLS,
        "Add the feature",
    );
    term.send_keys("C-n").unwrap();
    term.send_keys(",").unwrap();
    term.wait_for("Settings open", STEP, |s| s.contains("Git & encoding"))
        .unwrap_or_else(|e| panic!("{e}"));
    // Each target is located afresh and clicked until the screen answers:
    // the page lays out in stages under load, and a click at the position
    // of the first paint hits whatever moved there. Selecting a page, a
    // kind and opening the confirmation are all idempotent gestures.
    let dwell = Duration::from_millis(150);
    term.act_until(
        "the provider list",
        Duration::from_secs(4),
        STEP,
        |t| {
            let (c, r) = t.wait_for_settled_text("Integrations", dwell, STEP)?;
            t.click(c + 1, r)
        },
        |s| s.contains("\u{25CB} GitLab"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.act_until(
        "the disconnected gitlab.com form with Remove",
        Duration::from_secs(4),
        STEP,
        |t| {
            // The kind's row (its bullet), not a "GitLab" in a help text above it.
            let (c, r) = t.wait_for_settled_text("\u{25CB} GitLab", dwell, STEP)?;
            t.click(c + 3, r)
        },
        |s| s.contains("[ Remove ]") && s.contains("[ Connect ]"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    term.act_until(
        "the confirmation",
        Duration::from_secs(4),
        STEP,
        |t| {
            let (c, r) = t.wait_for_settled_text("[ Remove ]", dwell, STEP)?;
            t.click(c + 2, r)
        },
        |s| s.contains("Remove profile"),
    )
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let screen = term
        .wait_for_settled_text("[ Remove ]", dwell, STEP)
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen;
    term.click(c + 2, r).unwrap();
    term.wait_for("the profile gone", STEP, |s| !s.contains("Remove profile"))
        .unwrap_or_else(|e| panic!("{e}"));
    let saved = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(!saved.contains("id = \"gitlab\""), "{saved}");
    assert!(saved.contains("id = \"corp\""), "{saved}");
    term.stop().unwrap();
}

/// Settings > Integrations lists only the products GitPoulpi has a client
/// for: no grayed "soon" rows for Bitbucket, Azure DevOps, Jira or Trello.
#[test]
fn the_integrations_page_lists_only_supported_products() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _dir) = launch(
        &repo,
        "pty-hosting-only-supported",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    term.send_keys("C-n").unwrap();
    term.send_keys(",").unwrap();
    term.wait_for("Settings open", STEP, |s| s.contains("Git & encoding"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term.wait_for_text("Integrations", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    term.wait_for("the provider list", STEP, |s| {
        s.contains("GitLab Self-Managed")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// Opens Settings > Integrations and selects the product `name` in the
/// left list, then starts its connect form.
fn open_form(term: &mut xtask::pty::Terminal, name: &str) {
    term.send_keys("C-n").unwrap();
    term.send_keys(",").unwrap();
    term.wait_for("Settings open", STEP, |s| s.contains("Git & encoding"))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = term.wait_for_text("Integrations", STEP).unwrap();
    term.click(c + 1, r).unwrap();
    let screen = term
        .wait_for("the provider list", STEP, |s| s.contains(name))
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find(name).unwrap();
    term.click(c + 1, r).unwrap();
    let screen = term
        .wait_for("the product's empty view", STEP, |s| {
            s.contains("[ Connect ]") && s.contains("No ")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("[ Connect ]").unwrap();
    term.click(c + 2, r).unwrap();
}

/// The Issue trackers group lists Jira and Trello, and a Jira Cloud
/// profile connects against the mock entirely from the keyboard: Site
/// URL, Atlassian email, token, then Connect. The saved profile carries
/// the kind and the (non-secret) account and never the token.
#[test]
fn a_jira_cloud_profile_connects_from_the_keyboard_and_saves_its_account() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, dir) = launch(
        &repo,
        "pty-hosting-jira-connect",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    open_form(&mut term, "Jira Cloud");
    let screen = term
        .wait_for("the Jira Cloud form", STEP, |s| {
            s.contains("Site URL") && s.contains("Atlassian account email")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("Issue trackers"), "{}", screen.text());
    term.send_text(&format!("http://127.0.0.1:{}", mock.port))
        .unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("me@x").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("jira-secret-token").unwrap();
    // Token -> Generate -> Connect, by Tab only (rule 4).
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the connected card", STEP, |s| {
        s.contains("Connected") && s.contains("me@x")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(mock.saw("GET", "/rest/api/3/myself"));
    let saved = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(saved.contains("kind = \"jira_cloud\""), "{saved}");
    assert!(saved.contains("account = \"me@x\""), "{saved}");
    assert!(!saved.contains("jira-secret-token"), "{saved}");
    // Isolation: the token went through the throwaway credential helper the
    // PTY driver wires into the child's own global git config (inside the
    // temp config dir), never the machine's real helper or keychain.
    let creds = dir.path().join("git-credentials");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !creds.exists() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    let stored = std::fs::read_to_string(&creds)
        .unwrap_or_else(|e| panic!("the token must land in the test's own helper file: {e}"));
    assert!(stored.contains("gitpoulpi.invalid"), "{stored}");
    assert!(dir.path().join("home").is_dir());
    term.stop().unwrap();
}

/// Connect without the Atlassian email says what is missing.
#[test]
fn connecting_jira_cloud_without_an_email_asks_for_it() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _dir) = launch(
        &repo,
        "pty-hosting-jira-noemail",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    open_form(&mut term, "Jira Cloud");
    term.wait_for("the form", STEP, |s| s.contains("Site URL"))
        .unwrap_or_else(|e| panic!("{e}"));
    term.send_text(&format!("http://127.0.0.1:{}", mock.port))
        .unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("tok").unwrap();
    let screen = term
        .wait_for("Connect disabled hint reachable", STEP, |s| {
            s.contains("[ Connect ]")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("[ Connect ]").unwrap();
    term.click(c + 2, r).unwrap();
    term.wait_for("the email error", STEP, |s| {
        s.contains("Enter your Atlassian account email first.")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(!mock.saw("GET", "/myself"));
    term.stop().unwrap();
}

/// Trello: the token link needs the API key. It is disabled until the key
/// is typed, then opens Trello's authorize page with that key; the key
/// link opens the Power-Up admin page. Esc leaves without saving.
#[test]
fn trello_generate_link_needs_the_api_key_and_esc_saves_nothing() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, dir) = launch(
        &repo,
        "pty-hosting-trello-link",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    open_form(&mut term, "Trello");
    term.wait_for("the Trello form, link disabled", STEP, |s| {
        s.contains("[ Get an API key on Trello ]")
            && s.contains("[ Generate a token (enter the API key first) ]")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_text("abc123key").unwrap();
    let screen = term
        .wait_for("the link enabled", STEP, |s| {
            s.contains("[ Generate a token on trello.com ]")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    let (c, r) = screen.find("[ Get an API key on Trello ]").unwrap();
    term.click(c + 2, r).unwrap();
    let urls = wait_opened(&dir, 1);
    assert!(
        urls.iter()
            .any(|u| u.contains("trello.com/power-ups/admin")),
        "{urls:?}"
    );
    let (c, r) = screen.find("[ Generate a token on trello.com ]").unwrap();
    term.click(c + 2, r).unwrap();
    let urls = wait_opened(&dir, 2);
    assert!(
        urls.iter()
            .any(|u| u.contains("trello.com/1/authorize") && u.contains("key=abc123key")),
        "{urls:?}"
    );
    term.send_keys("Escape").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(300));
    let saved = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(!saved.contains("trello"), "{saved}");
    term.stop().unwrap();
}

/// Gitea: Host, token, then Connect, from the keyboard. The check is
/// `GET /api/v1/user` and the saved profile carries the kind, never the token.
#[test]
fn a_gitea_profile_connects_from_the_keyboard_and_saves_its_kind() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, dir) = launch(
        &repo,
        "pty-hosting-gitea-connect",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    open_form(&mut term, "Gitea");
    let screen = term
        .wait_for("the Gitea form", STEP, |s| s.contains("Host domain"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("Hosting"), "{}", screen.text());
    term.send_text(&format!("http://127.0.0.1:{}", mock.port))
        .unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("gitea-secret-token").unwrap();
    // Token -> Generate -> Connect, by Tab only (rule 4).
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the connected card", STEP, |s| {
        s.contains("Connected") && s.contains("kai")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(mock.saw("GET", "/api/v1/user"));
    let saved = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(saved.contains("kind = \"gitea\""), "{saved}");
    assert!(!saved.contains("gitea-secret-token"), "{saved}");
    term.stop().unwrap();
}

/// A self-hosted server is often a Docker service named without a dot:
/// `gitea:3000` is a host (the check goes out), while a bare `gitlab` is
/// still refused with the host hint.
#[test]
fn a_dotless_docker_host_with_a_port_is_accepted_but_a_bare_name_is_not() {
    for (typed, accepted) in [("gitlab", false), ("gitea:3000", true)] {
        let mock = start_mock();
        let repo = fixture(mock.port);
        let (mut term, _dir) = launch(
            &repo,
            if accepted {
                "pty-hosting-dotless-ok"
            } else {
                "pty-hosting-dotless-bad"
            },
            &config_toml(mock.port, ""),
            COLS,
            "Add the feature",
        );
        open_form(&mut term, "Gitea");
        term.wait_for("the Gitea form", STEP, |s| s.contains("Host domain"))
            .unwrap_or_else(|e| panic!("{e}"));
        term.send_text(typed).unwrap();
        term.send_keys("Tab").unwrap();
        term.send_text("tok").unwrap();
        term.send_keys("Tab").unwrap();
        term.send_keys("Tab").unwrap();
        term.send_keys("Enter").unwrap();
        if accepted {
            term.wait_for("the check going out", STEP, |s| {
                // "Connecting to" is a moment long: on a slow machine only the
                // failed check (the test transport refuses a non-loopback
                // host) is left to see.
                s.contains("Connecting to")
                    || s.contains("could not reach")
                    || s.contains("refusing to connect")
            })
            .unwrap_or_else(|e| panic!("{e}"));
            assert!(
                !term.text().contains("valid host domain"),
                "{}",
                term.text()
            );
        } else {
            term.wait_for("the host hint", STEP, |s| s.contains("valid host domain"))
                .unwrap_or_else(|e| panic!("{e}"));
        }
        term.stop().unwrap();
    }
}

/// Forgejo starts on Codeberg: the host field is prefilled and stays
/// editable.
#[test]
fn the_forgejo_form_starts_on_codeberg() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _dir) = launch(
        &repo,
        "pty-hosting-forgejo-form",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    open_form(&mut term, "Forgejo");
    term.wait_for("the prefilled host", STEP, |s| {
        s.contains("Host domain") && s.contains("https://codeberg.org")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.stop().unwrap();
}

/// YouTrack: Server URL, token, then Connect, from the keyboard. The check is
/// `GET /api/users/me` with the token as a Bearer; the saved profile carries
/// the kind and never the token.
#[test]
fn a_youtrack_profile_connects_from_the_keyboard_and_saves_its_kind() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, dir) = launch(
        &repo,
        "pty-hosting-youtrack-connect",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    open_form(&mut term, "YouTrack");
    let screen = term
        .wait_for("the YouTrack form", STEP, |s| {
            s.contains("Site URL") && s.contains("acme.youtrack.cloud")
        })
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(screen.contains("Issue trackers"), "{}", screen.text());
    term.send_text(&format!("http://127.0.0.1:{}", mock.port))
        .unwrap();
    term.send_keys("Tab").unwrap();
    term.send_text("perm:youtrack-secret").unwrap();
    // Token -> Generate -> Connect, by Tab only (rule 4).
    term.send_keys("Tab").unwrap();
    term.send_keys("Tab").unwrap();
    term.send_keys("Enter").unwrap();
    term.wait_for("the connected card", STEP, |s| {
        s.contains("Connected") && s.contains("kai")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(mock
        .requests()
        .iter()
        .any(|r| r.target.starts_with("/api/users/me")
            && r.headers
                .to_lowercase()
                .contains("authorization: bearer perm:youtrack-secret")));
    let saved = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(saved.contains("kind = \"youtrack\""), "{saved}");
    assert!(!saved.contains("youtrack-secret"), "{saved}");
    term.stop().unwrap();
}

/// Linear is one public host: its form asks for the key only (no address),
/// and is listed under Issue trackers.
#[test]
fn the_linear_form_asks_for_the_key_only() {
    let mock = start_mock();
    let repo = fixture(mock.port);
    let (mut term, _dir) = launch(
        &repo,
        "pty-hosting-linear-form",
        &config_toml(mock.port, ""),
        COLS,
        "Add the feature",
    );
    open_form(&mut term, "Linear");
    let screen = term
        .wait_for("the Linear form", STEP, |s| s.contains("Issue trackers"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(
        !screen.contains("Host domain") && !screen.contains("Site URL"),
        "no address to type:\n{}",
        screen.text()
    );
    term.stop().unwrap();
}
