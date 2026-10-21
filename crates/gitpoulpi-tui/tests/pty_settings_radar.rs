//! Settings > Integrations > the Radar block of a connected code host, end to
//! end through a pty: the palette row lands on it, a key toggles a list and
//! `config.toml` holds `[integrations.radar]` at once. Temp config dir only.
//!
//! The rules are `pty_ui.rs`'s: nothing is located by a hard-coded
//! coordinate, nothing sleeps, every wait describes the state it waits for.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STEP: Duration = Duration::from_secs(30);

const PROFILE: &str = "[[integrations]]\nid = \"work\"\nkind = \"github\"\n\
    url = \"https://github.com\"\ntoken_ref = \"integration:work\"\nname = \"WorkAccount\"\n\n\
    [integrations.user]\nlogin = \"kai\"\nname = \"Kai\"\nid = \"7\"\n";

#[test]
fn the_palette_lands_on_the_radar_block_and_a_key_saves_a_list() {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("radar settings fixture")
        .unwrap()
        .build();
    let (mut term, config) = common::launch(repo.root())
        .session("pty-settings-radar")
        .config(PROFILE)
        .size(180, 60)
        .ready_when("the fixture loaded", |s| {
            s.contains("radar settings fixture")
        })
        .start();

    term.send_keys("C-p").unwrap();
    term.send_text("Radar Options of a Profile").unwrap();
    term.wait_for("the palette narrowed", STEP, |s| {
        s.contains("Command Palette (1/")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("Enter").unwrap();
    term.wait_for("the Radar block of the profile", STEP, |s| {
        s.contains("[ ] Requests waiting for my review") && s.contains("[ ] Include drafts")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));

    // The keyboard is already on the first checkbox.
    term.send_keys("Space").unwrap();
    term.wait_for("the box ticked", STEP, |s| {
        s.contains("[x] Requests waiting for my review")
    })
    .unwrap_or_else(|e| panic!("{e}\n{}", term.text()));
    let file = config.path().join("config.toml");
    let deadline = std::time::Instant::now() + STEP;
    loop {
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        if text.contains("[integrations.radar]") && text.contains("review_requests = true") {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "config.toml never held the list:\n{text}"
        );
        std::thread::yield_now();
    }
}
