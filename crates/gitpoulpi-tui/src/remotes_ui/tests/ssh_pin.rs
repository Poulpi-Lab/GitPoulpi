use super::*;

fn profile(id: &str, name: &str, key: Option<&str>) -> gitpoulpi_config::Integration {
    let mut i = gitpoulpi_config::Integration::new(
        id,
        gitpoulpi_config::IntegrationKind::Github,
        "https://github.com",
    );
    i.name = name.to_string();
    i.ssh = key.map(|k| gitpoulpi_config::IntegrationSsh {
        mode: gitpoulpi_config::SshMode::Custom,
        private_key_path: Some(k.to_string()),
        ..Default::default()
    });
    i
}

#[test]
fn only_profiles_with_a_private_key_are_offered_for_the_pin() {
    let choices = ssh_key_choices(&[
        profile("a", "Work", Some("/k/work")),
        profile("b", "", None),
        profile("c", "", Some("/k/personal")),
    ]);
    let got: Vec<(&str, &str)> = choices
        .iter()
        .map(|c| (c.profile.as_str(), c.private_key_path.as_str()))
        .collect();
    assert_eq!(got, [("Work", "/k/work"), ("c", "/k/personal")]);
}

#[test]
fn the_pin_menu_marks_the_pinned_key_and_offers_clear_only_when_pinned() {
    let keys = ssh_key_choices(&[profile("a", "Work", Some("/k/work"))]);
    let pinned = gitpoulpi_hosting::ssh_key::ssh_command_for_key("/k/work");

    let unpinned = ssh_pin_entries(None, &keys);
    assert!(matches!(&unpinned[0], Entry::Disabled { label } if label.contains("Not pinned")));
    assert!(!unpinned.iter().any(|e| matches!(
        e,
        Entry::Action {
            action: RemoteUiAction::PinSshKey(None),
            ..
        }
    )));

    let entries = ssh_pin_entries(Some(&pinned), &keys);
    assert!(matches!(&entries[0], Entry::Disabled { label } if label.contains(&pinned)));
    assert!(entries.iter().any(|e| matches!(
        e,
        Entry::Action { label, action: RemoteUiAction::PinSshKey(Some(c)), .. }
            if c == &pinned && label.starts_with('\u{25C9}')
    )));
    assert!(entries.iter().any(|e| matches!(
        e,
        Entry::Action {
            action: RemoteUiAction::PinSshKey(None),
            ..
        }
    )));
}

#[test]
fn the_pin_menu_says_when_no_profile_has_a_key() {
    let entries = ssh_pin_entries(None, &[]);
    assert!(entries.iter().any(
        |e| matches!(e, Entry::Disabled { label } if label.contains("No profile has an SSH key"))
    ));
}
