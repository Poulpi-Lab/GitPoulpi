//! The push menu says who the push signs in as, only when the repository's
//! profile is not all default, and the row opens the SSH & credentials page.

use super::*;
use gitpoulpi_core::domain::{ActiveIdentity, HttpsIdentity, IdentityVia};

fn remote(name: &str) -> Remote {
    Remote {
        name: name.into(),
        fetch_url: format!("https://github.com/acme/{name}.git"),
        push_url: None,
    }
}

fn head() -> Vec<Ref> {
    let mut r = local_branch_at("main", oid(1));
    r.is_head = true;
    vec![r]
}

/// The menu as a screen: every row's text.
fn drawn(ui: &RemoteUi) -> String {
    use ratatui::backend::TestBackend;
    let menu = ui.menu.as_ref().expect("the push menu is open");
    let mut terminal = ratatui::Terminal::new(TestBackend::new(80, 20)).unwrap();
    terminal
        .draw(|f| {
            menu.render(f, None);
        })
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect()
}

fn account() -> ActiveIdentity {
    ActiveIdentity {
        https: Some(HttpsIdentity {
            via: IdentityVia::Token,
            name: "github-corp".into(),
        }),
        ssh: None,
        integration: true,
    }
}

#[test]
fn the_push_menu_shows_the_identity_line_only_when_the_profile_is_not_default() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = RemoteUi::load(dir.path());
    ui.remotes = vec![remote("origin")];

    // Everything default: the menu is the remotes and nothing else.
    assert_eq!(ui.open_push_menu(&head(), 2, 2), None);
    let text = drawn(&ui);
    assert!(text.contains("origin"), "{text}");
    assert!(!text.contains("Signs in as"), "{text}");

    // An account chosen for this repository: one line, the text the log uses.
    crate::activity_log::identity::note(dir.path(), Some(&account()));
    assert_eq!(ui.open_push_menu(&head(), 2, 2), None);
    let text = drawn(&ui);
    assert!(text.contains("Signs in as: github-corp (token)"), "{text}");
    let menu = ui.menu.as_ref().unwrap();
    let opens_page = (0..menu.row_count())
        .filter_map(|i| menu.row_action(i))
        .any(|a| matches!(a, RemoteUiAction::OpenRepoAuth));
    assert!(opens_page, "the line is a row that opens the page");

    // Back to default: gone again.
    crate::activity_log::identity::note(dir.path(), None);
    ui.open_push_menu(&head(), 2, 2);
    assert!(!drawn(&ui).contains("Signs in as"));
}
