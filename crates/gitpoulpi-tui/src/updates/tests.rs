//! The update check's wiring (no socket: a recorded transport and a temp
//! folder), the chip's text and the Release notes tab's buffer.

use super::*;
use gitpoulpi_config::Config;
use gitpoulpi_hosting::updates::CheckParams;
use gitpoulpi_hosting::RecordedTransport;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;

fn lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// A body whose newest release is far ahead of any real version.
fn releases_body() -> String {
    serde_json::json!([
        {"tag_name": "v99.0.0", "body": "## 99.0.0\n- shiny **new** thing",
         "html_url": "https://example.test/releases/v99.0.0", "assets": []},
        {"tag_name": "v0.0.1", "body": "old", "html_url": "x", "assets": []}
    ])
    .to_string()
}

fn params() -> CheckParams {
    CheckParams {
        repo: "o/r".into(),
        running: RUNNING.into(),
        channel: UpdateChannel::Stable,
        target: "x86_64-unknown-linux-gnu".into(),
        token: None,
    }
}

fn configured(dir: &Path, mode: UpdateMode) -> PathBuf {
    let path = dir.join("data").join("updates.json");
    let mut config = Config::default();
    config.updates.mode = mode;
    {
        let mut s = shared();
        s.known = None;
        s.skipped = false;
        s.checking = false;
        s.message = None;
        s.state_path = None;
        auto::set_updatable(&mut s, None, &gitpoulpi_update::fsops::RealFs);
    }
    configure(&config, Some(path.clone()));
    path
}

#[test]
fn the_chip_shows_the_version_and_turns_into_available_when_a_newer_release_is_known() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let path = configured(dir.path(), UpdateMode::Notify);
    assert_eq!(chip::text(), running_label());
    assert!(running_label().starts_with(&format!("v{RUNNING}")));
    assert!(!chip::is_available());

    let transport = RecordedTransport::new().on("/releases", 200, &releases_body());
    let outcome = perform(&transport, "http://x", &params(), &path, false, 1_000);
    assert!(matches!(outcome, Outcome::Answered), "{outcome:?}");
    finish(outcome, false);
    assert_eq!(chip::text(), "v99.0.0 available");
    assert!(chip::is_available());
}

#[test]
fn a_second_check_within_the_day_makes_no_request_and_a_manual_one_does() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let path = configured(dir.path(), UpdateMode::Notify);
    let transport = RecordedTransport::new().on("/releases", 200, &releases_body());
    perform(&transport, "http://x", &params(), &path, false, 1_000);
    assert_eq!(transport.requests().len(), 1);
    let again = perform(
        &transport,
        "http://x",
        &params(),
        &path,
        false,
        1_000 + 3_600,
    );
    assert!(matches!(again, Outcome::NotDue));
    assert_eq!(transport.requests().len(), 1, "throttled: no request");
    perform(
        &transport,
        "http://x",
        &params(),
        &path,
        true,
        1_000 + 3_601,
    );
    assert_eq!(transport.requests().len(), 2, "forced: asks again");
    let next_day = perform(
        &transport,
        "http://x",
        &params(),
        &path,
        false,
        1_000 + 86_400 + 3_601,
    );
    assert!(matches!(next_day, Outcome::Answered));
    assert_eq!(transport.requests().len(), 3);
}

#[test]
fn a_failed_check_is_silent_keeps_the_cache_and_still_counts_for_the_throttle() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let path = configured(dir.path(), UpdateMode::Notify);
    let ok = RecordedTransport::new().on("/releases", 200, &releases_body());
    finish(perform(&ok, "http://x", &params(), &path, false, 10), false);
    let down = RecordedTransport::new().on("/releases", 404, "{\"message\":\"Not Found\"}");
    let outcome = perform(&down, "http://x", &params(), &path, true, 20);
    assert!(matches!(outcome, Outcome::Failed(_)));
    finish(outcome, false);
    assert!(
        take_message().is_none(),
        "a background failure says nothing"
    );
    assert_eq!(chip::text(), "v99.0.0 available", "the known release stays");
    let state = UpdateState::load(&path);
    assert_eq!(state.checked_at, 20);
}

#[test]
fn skipping_hides_the_chip_notice_and_survives_a_restart() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let path = configured(dir.path(), UpdateMode::Notify);
    let ok = RecordedTransport::new().on("/releases", 200, &releases_body());
    finish(perform(&ok, "http://x", &params(), &path, false, 10), false);
    assert!(snapshot().available.is_some());
    assert_eq!(skip().as_deref(), Some("version 99.0.0 skipped"));
    assert!(snapshot().available.is_none());
    assert!(snapshot().skipped.is_some(), "still known, as skipped");
    assert_eq!(chip::text(), running_label());
    // A new process: the file says it.
    let _ = configured(dir.path(), UpdateMode::Notify);
    assert!(snapshot().available.is_none());
    assert_eq!(UpdateState::load(&path).skipped.as_deref(), Some("99.0.0"));
}

#[test]
fn mode_off_hides_the_notice_and_a_manual_check_says_why_without_asking() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let path = configured(dir.path(), UpdateMode::Notify);
    let ok = RecordedTransport::new().on("/releases", 200, &releases_body());
    finish(perform(&ok, "http://x", &params(), &path, false, 10), false);
    assert!(chip::is_available());
    let mut config = Config::default();
    config.updates.mode = UpdateMode::Off;
    configure(&config, None);
    assert!(!chip::is_available(), "off: no notice");
    assert_eq!(
        check_now().as_deref(),
        Some("update checks are turned off (Settings > Synchronization)")
    );
    assert!(!shared().checking, "nothing was started");
}

#[test]
fn auto_behaves_like_notify_for_a_build_that_cannot_update_itself() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let path = configured(dir.path(), UpdateMode::Auto);
    let ok = RecordedTransport::new().on("/releases", 200, &releases_body());
    finish(perform(&ok, "http://x", &params(), &path, false, 10), false);
    assert!(chip::is_available());
    assert!(snapshot().available.is_some());
}

#[test]
fn the_channel_decides_which_cached_release_counts() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let path = configured(dir.path(), UpdateMode::Notify);
    let body = serde_json::json!([
        {"tag_name": "v99.0.0-rc.1", "prerelease": true, "body": "rc", "html_url": "u", "assets": []}
    ])
    .to_string();
    let ok = RecordedTransport::new().on("/releases", 200, &body);
    let mut p = params();
    p.channel = UpdateChannel::Prerelease;
    let mut config = Config::default();
    config.updates.channel = UpdateChannel::Prerelease;
    configure(&config, None);
    finish(perform(&ok, "http://x", &p, &path, false, 10), false);
    assert_eq!(chip::text(), "v99.0.0-rc.1 available");
    configure(&Config::default(), None);
    assert_eq!(chip::text(), running_label(), "stable ignores the rc");
}

#[test]
fn a_manual_check_leaves_a_status_line() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let path = configured(dir.path(), UpdateMode::Notify);
    let ok = RecordedTransport::new().on("/releases", 200, &releases_body());
    finish(perform(&ok, "http://x", &params(), &path, true, 10), true);
    assert_eq!(
        take_message().as_deref(),
        Some("version 99.0.0 is available")
    );
    assert!(take_message().is_none(), "once");
    let down = RecordedTransport::new().on("/releases", 500, "boom");
    finish(perform(&down, "http://x", &params(), &path, true, 20), true);
    assert_eq!(
        take_message().as_deref(),
        Some("could not check for updates (see the Activity Log)")
    );
}

#[test]
fn the_changelog_is_compiled_in_and_the_running_section_is_found() {
    assert!(notes::section_start(CHANGELOG_FOR_TESTS, "0.1.0", false) < CHANGELOG_FOR_TESTS.len());
    let sample =
        "# Changelog\n\n## [Unreleased]\n- a\n\n## [0.2.0] - 2026\n- b\n\n## [0.1.0] - 2025\n- c\n";
    let at = |running, dev| &sample[notes::section_start(sample, running, dev)..];
    assert!(at("0.1.0", false).starts_with("## [0.1.0]"));
    assert!(at("0.2.0", false).starts_with("## [0.2.0]"));
    assert!(
        at("0.2.0", true).starts_with("## [Unreleased]"),
        "dev builds: Unreleased"
    );
    assert!(
        at("9.9.9", false).starts_with("## [Unreleased]"),
        "unknown: Unreleased"
    );
    assert_eq!(notes::section_start("no headings", "0.1.0", false), 0);
}

const CHANGELOG_FOR_TESTS: &str = include_str!("../../../../CHANGELOG.md");

fn screen(terminal: &Terminal<TestBackend>) -> String {
    let buf = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            out.push_str(buf[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

fn draw(slot: &mut notes::ReleaseNotesSlot, terminal: &mut Terminal<TestBackend>) -> String {
    let area = Rect::new(0, 1, 100, 28);
    terminal
        .draw(|frame| slot.render(frame, area))
        .expect("draw");
    screen(terminal)
}

#[test]
fn the_release_notes_tab_shows_the_new_release_body_above_the_changelog_with_its_buttons() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let path = configured(dir.path(), UpdateMode::Notify);
    let ok = RecordedTransport::new().on("/releases", 200, &releases_body());
    finish(perform(&ok, "http://x", &params(), &path, false, 10), false);
    let mut slot = notes::ReleaseNotesSlot::default();
    slot.open();
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let text = draw(&mut slot, &mut terminal);
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[1].contains("Version 99.0.0 is available"), "{text}");
    assert!(
        lines[2].contains("Download")
            && lines[2].contains("Skip this version")
            && lines[2].contains("Check now"),
        "{text}"
    );
    assert!(
        lines.iter().any(|l| l.contains("shiny new thing")),
        "the body, markdown rendered: {text}"
    );
    assert!(lines[28].contains("d download"), "the key hint: {text}");
    // Skipping from the keyboard rewrites the header and drops Skip.
    let event = slot.on_key(crossterm::event::KeyEvent::from(
        crossterm::event::KeyCode::Char('s'),
    ));
    assert_eq!(
        event,
        notes::NotesEvent::Message("version 99.0.0 skipped".into())
    );
    let text = draw(&mut slot, &mut terminal);
    assert!(text.contains("(skipped)"), "{text}");
    assert!(!text.contains("Skip this version"), "{text}");
}

#[test]
fn with_nothing_newer_the_tab_opens_on_the_changelog_at_the_running_version() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let _ = configured(dir.path(), UpdateMode::Notify);
    let mut slot = notes::ReleaseNotesSlot::default();
    slot.open();
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let text = draw(&mut slot, &mut terminal);
    assert!(
        text.lines()
            .nth(1)
            .unwrap()
            .contains(&format!("You are running version {RUNNING}")),
        "{text}"
    );
    assert!(text.lines().nth(2).unwrap().contains("Check now"));
    assert!(!text.lines().nth(2).unwrap().contains("Download"));
    // The first body row is the Unreleased heading of the compiled-in changelog.
    assert!(
        text.lines().nth(4).unwrap().contains("Unreleased"),
        "{text}"
    );
}

#[test]
fn escape_closes_the_tab_and_the_keys_scroll() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let _ = configured(dir.path(), UpdateMode::Notify);
    let mut slot = notes::ReleaseNotesSlot::default();
    slot.open();
    let mut terminal = Terminal::new(TestBackend::new(100, 12)).unwrap();
    let area = Rect::new(0, 0, 100, 12);
    terminal.draw(|f| slot.render(f, area)).unwrap();
    let before = screen(&terminal);
    let key = |c| crossterm::event::KeyEvent::from(c);
    slot.on_key(key(crossterm::event::KeyCode::PageDown));
    terminal.draw(|f| slot.render(f, area)).unwrap();
    assert_ne!(before, screen(&terminal), "PageDown scrolled");
    slot.on_key(key(crossterm::event::KeyCode::Home));
    terminal.draw(|f| slot.render(f, area)).unwrap();
    assert_eq!(
        slot.on_key(key(crossterm::event::KeyCode::Esc)),
        notes::NotesEvent::Close
    );
}

#[test]
fn the_release_notes_buttons_show_the_pointing_hand_and_the_text_the_arrow() {
    let _lock = lock();
    let dir = tempfile::tempdir().unwrap();
    let path = configured(dir.path(), UpdateMode::Notify);
    let ok = RecordedTransport::new().on("/releases", 200, &releases_body());
    finish(perform(&ok, "http://x", &params(), &path, false, 10), false);
    let mut slot = notes::ReleaseNotesSlot::default();
    slot.open();
    let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let text = draw(&mut slot, &mut terminal);
    let lines: Vec<&str> = text.lines().collect();
    let button_row = lines[2];
    let col = |needle: &str| {
        button_row
            .find(needle)
            .map(|b| button_row[..b].chars().count())
    };
    let download = col("Download").expect("Download") as u16;
    assert!(slot.pointer_hand_at(download + 1, 2), "Download");
    let check = col("Check now").expect("Check now") as u16;
    assert!(slot.pointer_hand_at(check + 1, 2), "Check now");
    assert!(!slot.pointer_hand_at(99, 2), "past the buttons");
    let body_row = lines
        .iter()
        .position(|l| l.contains("shiny new thing"))
        .expect("the body") as u16;
    assert!(!slot.pointer_hand_at(3, body_row), "release text");
}
