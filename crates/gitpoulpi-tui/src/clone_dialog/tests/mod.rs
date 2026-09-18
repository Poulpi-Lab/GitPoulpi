mod confirm;
mod form;
#[cfg(all(feature = "github", feature = "gitlab"))]
mod hosted;
mod i18n;
mod render;

use super::*;

fn abs(p: &str) -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(format!("C:\\{p}"))
    } else {
        PathBuf::from(format!("/{p}"))
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn typed(dialog: &mut CloneDialog, text: &str) {
    for c in text.chars() {
        dialog.on_key(key(KeyCode::Char(c)));
    }
}

#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
fn click(col: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: col,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

#[cfg(all(feature = "github", feature = "gitlab"))]
fn project(full_name: &str) -> HostedProject {
    HostedProject {
        integration_id: "corp".into(),
        id: full_name.into(),
        name: full_name.rsplit('/').next().unwrap().into(),
        full_name: full_name.into(),
        description: Some(format!("about {full_name}")),
        default_branch: Some("main".into()),
        ssh_url: Some(format!("git@gitlab.example.com:{full_name}.git")),
        https_url: Some(format!("https://gitlab.example.com/{full_name}.git")),
        web_url: None,
        private: false,
    }
}

#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
fn hosted() -> Vec<HostedSource> {
    vec![
        HostedSource {
            profile_id: "corp".into(),
            label: "GitLab \u{00B7} corp".into(),
            login: Some("kai".into()),
        },
        HostedSource {
            profile_id: "hub".into(),
            label: "GitHub \u{00B7} github.com".into(),
            login: None,
        },
    ]
}

#[cfg_attr(not(feature = "hosting"), allow(dead_code))]
fn with_hosted() -> CloneDialog {
    CloneDialog::new(Some(&abs("clones"))).with_hosted(hosted())
}

/// Lets the destination probe (a thread) answer, then returns what the
/// dialog did with it: a Clone pressed meanwhile carries on.
fn settle(dialog: &mut CloneDialog) -> CloneOutcome {
    for _ in 0..500 {
        let outcome = dialog.poll();
        if outcome != CloneOutcome::Stay || dialog.dest().is_none() || dialog.dest_kind().is_some()
        {
            return outcome;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("the destination probe never answered");
}

/// The highlighted source has rested long enough.
#[cfg(feature = "hosting")]
fn rest(dialog: &mut CloneDialog) {
    dialog.load_due = dialog.load_due.map(|_| Instant::now());
}

fn paint(dialog: &mut CloneDialog, width: u16, height: u16) -> (String, Rect) {
    let mut buf = Buffer::empty(Rect::new(0, 0, width, height));
    let rect = dialog.rect(buf.area);
    dialog.render(rect, &mut buf);
    (dialog_kit::buffer_text(&buf), rect)
}

/// With no provider built the dialog is the URL form only: a connected
/// profile in the config adds no source, so there is no provider list and
/// no repository browser.
#[cfg(not(feature = "hosting"))]
#[test]
fn a_build_with_no_provider_clones_by_url_only() {
    let mut profile = gitpoulpi_config::Integration::new(
        "hub",
        gitpoulpi_config::IntegrationKind::Github,
        "https://github.com",
    );
    profile.user = Some(gitpoulpi_config::ConnectedUser {
        login: "octo".into(),
        ..Default::default()
    });
    assert!(hosted_sources(&[profile]).is_empty());

    let mut dialog = CloneDialog::new(Some(&abs("clones"))).with_hosted(hosted_sources(&[]));
    let (text, _) = paint(&mut dialog, 120, 30);
    assert!(
        !text.contains("GitHub") && !text.contains("GitLab"),
        "{text}"
    );
    assert!(!text.contains(msg::clone::CLONE_FROM.get()), "{text}");
    let note_start = &crate::ui_kit::wrap_words(msg::clone::CONNECT.get(), 22)[0];
    assert!(!text.contains(note_start), "{text}");
    assert!(text.contains(msg::clone::SOURCE_URL.get()), "{text}");
    assert!(!dialog.hosted_mode());
    assert_eq!(dialog.sources_len(), 1);
}
