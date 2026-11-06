//! The update panel's buffer: what each procedure kind shows, the close
//! cross out of the paint walk, and the keys and mouse of the window.

use super::*;
use crate::updates::{auto, plans::Facts, shared, TEST_LOCK};
use crossterm::event::{KeyEventKind, KeyEventState};
use gitpoulpi_config::policy::Policy;
use gitpoulpi_config::release::{InstallChannel, Marker};
use gitpoulpi_hosting::updates::Release;
use gitpoulpi_update::channel::{Channel, How, Resolved};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn facts(channel: Channel, policy: Policy) -> Facts {
    Facts {
        resolved: Resolved {
            channel,
            how: How::Marker,
            dir: "/opt/gp".into(),
            exe_name: "gitpoulpi-tui".into(),
            marker: Some(Marker::new(
                "0.1.0",
                gitpoulpi_config::release::host_target(),
            )),
        },
        policy,
        apt_source_present: true,
        aur_helper: None,
        binstall: false,
    }
}

/// The machine reads as `facts` and a newer release is known.
fn setup(facts: Option<Facts>) {
    let mut s = shared();
    s.mode = gitpoulpi_config::UpdateMode::Notify;
    s.skipped = false;
    s.checking = false;
    s.message = None;
    s.known = Some(Release {
        tag: "v99.0.0".into(),
        version: "99.0.0".into(),
        prerelease: false,
        notes: String::new(),
        page_url: "https://example.test/releases/v99.0.0".into(),
        asset: None,
        checksum: None,
    });
    auto::set_facts(&mut s, facts);
    drop(s);
    close();
}

fn draw(width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(render).unwrap();
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

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

fn click(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

fn panel_for(channel: Channel, policy: Policy) -> String {
    setup(Some(facts(channel, policy)));
    open();
    draw(100, 30)
}

#[test]
fn a_swap_install_offers_update_and_restart_and_says_how_it_was_installed() {
    let _lock = lock();
    let text = panel_for(Channel::Install(InstallChannel::Archive), Policy::default());
    assert!(text.contains("Installed with: a release archive"), "{text}");
    assert!(text.contains("Version 99.0.0 is available"), "{text}");
    assert!(text.contains("Update and restart"), "{text}");
    assert!(text.contains("closes, replaces its files"), "{text}");
    assert!(!text.contains("Copy command"), "{text}");
    close();
}

#[test]
fn a_package_manager_install_runs_its_command_and_can_copy_it() {
    let _lock = lock();
    let text = panel_for(Channel::Install(InstallChannel::Choco), Policy::default());
    assert!(text.contains("Installed with: Chocolatey"), "{text}");
    assert!(text.contains("Run update"), "{text}");
    assert!(text.contains("choco upgrade gitpoulpi -y"), "{text}");
    assert!(text.contains("Copy command"), "{text}");
    assert!(!text.contains("Open in terminal"), "{text}");
    assert!(!text.contains("Update and restart"), "{text}");
    close();
}

#[test]
fn apt_shows_the_command_with_copy_and_open_in_terminal_and_never_a_run_button() {
    let _lock = lock();
    let text = panel_for(Channel::Install(InstallChannel::Apt), Policy::default());
    assert!(text.contains("Installed with: apt"), "{text}");
    assert!(
        text.contains("sudo apt update && sudo apt install --only-upgrade gitpoulpi"),
        "{text}"
    );
    assert!(text.contains("Copy command"), "{text}");
    assert!(text.contains("Open in terminal"), "{text}");
    assert!(!text.contains("Update and restart"), "{text}");
    assert!(!text.contains("Run update"), "{text}");
    close();
}

#[test]
fn a_managed_install_announces_and_shows_the_administrators_message() {
    let _lock = lock();
    let policy = Policy {
        message: Some("Updates are deployed by IT.".into()),
        ..Policy::default()
    };
    let text = panel_for(Channel::Install(InstallChannel::Msi), policy);
    assert!(text.contains("managed by your administrator"), "{text}");
    assert!(
        text.contains("From your administrator: Updates are deployed by IT."),
        "{text}"
    );
    for absent in ["Update and restart", "Run update", "Copy command"] {
        assert!(!text.contains(absent), "{absent}: {text}");
    }
    close();
}

#[test]
fn a_container_announces_and_lists_the_pull_commands_from_the_one_image_constant() {
    let _lock = lock();
    let text = panel_for(Channel::Install(InstallChannel::Docker), Policy::default());
    assert!(text.contains("Download a newer image"), "{text}");
    assert!(
        text.contains(&format!(
            "docker pull {}:99.0.0",
            gitpoulpi_config::release::DOCKER_IMAGE
        )),
        "{text}"
    );
    assert!(text.contains("Copy command"), "{text}");
    close();
}

#[test]
fn a_policy_that_blocks_self_update_turns_the_swap_into_an_announcement() {
    let _lock = lock();
    let policy = Policy {
        allow_self_update: Some(false),
        ..Policy::default()
    };
    let text = panel_for(Channel::Install(InstallChannel::Archive), policy);
    assert!(text.contains("turned off updating"), "{text}");
    assert!(!text.contains("Update and restart"), "{text}");
    close();
}

#[test]
fn a_development_build_says_it_has_no_install_record() {
    let _lock = lock();
    let text = panel_for(Channel::Dev, Policy::default());
    assert!(text.contains("no install record"), "{text}");
    assert!(!text.contains("Update and restart"), "{text}");
    close();
}

#[test]
fn the_close_cross_comes_from_the_paint_walk_and_every_way_out_closes() {
    let _lock = lock();
    setup(Some(facts(
        Channel::Install(InstallChannel::Archive),
        Policy::default(),
    )));
    open();
    let text = draw(100, 30);
    let (window, cross) = with(|s| (s.window, s.close));
    assert_eq!(cross.y, window.y, "on the top border\n{text}");
    assert_eq!(cross.width, 3);
    let row: String = text.lines().nth(usize::from(cross.y)).unwrap().to_string();
    assert!(row.contains('\u{2715}') || row.contains('x'), "{row}");
    // The cross.
    assert!(is_open());
    on_mouse(click(cross.x + 1, cross.y));
    assert!(!is_open(), "the cross closes");
    // Escape.
    open();
    draw(100, 30);
    on_key(key(KeyCode::Esc));
    assert!(!is_open(), "Esc closes");
    // A click outside.
    open();
    draw(100, 30);
    on_mouse(click(0, 0));
    assert!(!is_open(), "a click outside closes");
    // A click inside (on nothing) keeps it open.
    open();
    draw(100, 30);
    let inside = with(|s| s.window);
    on_mouse(click(inside.x + 3, inside.y + 3));
    assert!(is_open());
    close();
}

#[test]
fn every_button_is_reached_by_a_key_and_the_mouse_through_one_hit_list() {
    let _lock = lock();
    setup(Some(facts(
        Channel::Install(InstallChannel::Apt),
        Policy::default(),
    )));
    open();
    draw(100, 30);
    let hits = with(|s| s.hits.clone());
    let shown: Vec<Act> = hits.iter().map(|(_, a)| *a).collect();
    for act in [
        Act::Copy,
        Act::Terminal,
        Act::Download,
        Act::Notes,
        Act::Skip,
        Act::Check,
        Act::Close,
    ] {
        assert!(shown.contains(&act), "{act:?} in {shown:?}");
    }
    // The pointing hand sits on each button and nowhere else.
    for (rect, _) in &hits {
        assert!(pointer_hand(rect.x, rect.y));
    }
    // Copy by the mouse: the status says what was copied.
    let (rect, _) = hits.iter().find(|(_, a)| *a == Act::Copy).unwrap();
    let event = on_mouse(click(rect.x + 1, rect.y));
    let Event::Message(line) = event else {
        panic!("{event:?}")
    };
    assert!(line.contains("sudo apt update"), "{line}");
    // And by its key.
    let Event::Message(line) = on_key(key(KeyCode::Char('c'))) else {
        panic!("no message")
    };
    assert!(line.contains("the update command"), "{line}");
    // Notes: closes the panel and asks for the tab.
    assert_eq!(on_key(key(KeyCode::Char('n'))), Event::ReleaseNotes);
    assert!(!is_open());
    close();
}

#[test]
fn the_body_scrolls_with_a_bar_in_a_short_window() {
    let _lock = lock();
    let policy = Policy {
        message: Some("A long message from the administrator. ".repeat(8)),
        ..Policy::default()
    };
    setup(Some(facts(
        Channel::Install(InstallChannel::Docker),
        policy,
    )));
    open();
    let text = draw(60, 14);
    assert!(with(|s| s.lines) > usize::from(with(|s| s.body.height)));
    assert!(
        text.contains('\u{2191}') && text.contains('\u{2193}'),
        "a scrollbar is drawn\n{text}"
    );
    let before = with(|s| s.offset);
    on_key(key(KeyCode::Down));
    draw(60, 14);
    assert_eq!(with(|s| s.offset), before + 1);
    // The wheel too.
    on_mouse(MouseEvent {
        kind: MouseEventKind::ScrollDown,
        column: 10,
        row: 6,
        modifiers: KeyModifiers::NONE,
    });
    assert!(with(|s| s.offset) > before + 1);
    close();
}

#[test]
fn the_buttons_match_the_procedure_and_the_pure_list_has_no_empty_label() {
    let _lock = lock();
    for channel in InstallChannel::ALL {
        setup(Some(facts(Channel::Install(channel), Policy::default())));
        let list = actions(&crate::updates::snapshot());
        assert!(!list.is_empty());
        assert!(
            list.iter().all(|(_, label)| !label.is_empty()),
            "{channel:?}"
        );
        assert_eq!(list.last().map(|(a, _)| *a), Some(Act::Close));
    }
    close();
}
