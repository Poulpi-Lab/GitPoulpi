//! With no terminal image protocol (auto-detection found
//! none, or `image_protocol = "none"` / `graph_renderer = "text"`), the
//! avatar option must not make the TUI attempt any picture: no image
//! escape on the wire (iTerm2 `OSC 1337`, Kitty `APC _G`, Sixel `DCS q`),
//! and the Author column keeps its text badge.

mod common;

use std::time::Duration;

use gitpoulpi_testkit::FixtureRepoBuilder;

const STARTUP: Duration = Duration::from_secs(60);
const AUTHOR: &str = "GitPoulp";

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn run(session: &str, toml: &str, terminal_image: bool) {
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("a no protocol commit")
        .unwrap()
        .build();
    let mut launch = common::launch(repo.root())
        .session(session)
        .config(toml)
        .size(200, 50)
        .ready_when("the commit row", |s| {
            s.contains("a no protocol commit") && s.contains(AUTHOR)
        });
    if terminal_image {
        launch = launch.terminal_image();
    }
    let (term, _config) = launch.start();
    term.wait_for_stable(Duration::from_millis(800), STARTUP)
        .unwrap();
    let raw = term.raw_output();
    for (what, needle) in [
        ("iTerm2 inline image", &b"]1337;File="[..]),
        ("Kitty graphics", &b"_Ga="[..]),
        ("Sixel", &b"\x1bPq"[..]),
        ("Sixel with params", &b"\x1bP0;"[..]),
    ] {
        assert!(
            !contains(&raw, needle),
            "{session}: a {what} escape was sent"
        );
    }
    let _ = term.stop();
}

#[test]
fn graph_avatars_without_a_protocol_draw_no_picture() {
    run(
        "noproto-graph",
        "[ui]\navatars = \"graph\"\navatar_sources = \"hosting\"\n",
        false,
    );
}

#[test]
fn author_avatars_without_a_protocol_draw_no_picture() {
    run(
        "noproto-author",
        "[ui]\navatars = \"author\"\navatar_sources = \"hosting\"\n",
        false,
    );
}

#[test]
fn an_explicit_protocol_none_overrides_a_capable_terminal() {
    run(
        "noproto-none",
        "[ui]\navatars = \"graph\"\navatar_sources = \"hosting\"\n\
         graph_renderer = \"image\"\nimage_protocol = \"none\"\n",
        true,
    );
}
