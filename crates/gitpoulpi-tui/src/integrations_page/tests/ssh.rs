use super::*;
use crate::th;

fn ssh_page(h: &mut H) -> (IntegrationsPage, Arc<RecordedTransport>, PathBuf) {
    let t = Arc::new(
        RecordedTransport::new()
            .on("/user/keys/77", 204, "")
            .on("/user/keys?", 200, "[]")
            .on(
                "/user/keys",
                201,
                r#"{"id":77,"title":"t","key":"ssh-ed25519 AAAA"}"#,
            ),
    );
    let ssh_dir = h._dir.path().join("dot-ssh");
    let mut page = IntegrationsPage {
        transport: Some(t.clone()),
        ..Default::default()
    };
    page.ssh.ssh_dir = Some(ssh_dir.clone());
    h.connected(IntegrationKind::GitlabSelfManaged, "kai");
    page.select_kind(GITLAB_SM, &h.config);
    page.zone = Zone::Body;
    (page, t, ssh_dir)
}

fn wait_ssh(page: &mut IntegrationsPage, h: &mut H) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while page.ssh.job.is_some() {
        page.poll(&mut h.env());
        assert!(
            std::time::Instant::now() < deadline,
            "key job never finished"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

fn act(page: &mut IntegrationsPage, h: &mut H, item: Item) {
    page.activate(item, &mut h.env());
}

#[test]
fn the_ssh_section_offers_default_generate_and_custom() {
    let mut h = H::new();
    let (mut page, _, _) = ssh_page(&mut h);
    let screen = text(&draw(&mut page, &h, 90, 40, None));
    for want in [
        "SSH key",
        "Default key",
        "Generate new key",
        "Custom key",
        "default SSH key",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    act(&mut page, &mut h, Item::SshGenerate);
    let screen = text(&draw(&mut page, &h, 90, 40, None));
    for want in [
        "Key title",
        "GitPoulpi gitlab.example.com 20",
        "[x] Add the public key",
        " Generate key ",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    act(&mut page, &mut h, Item::SshCustom);
    let screen = text(&draw(&mut page, &h, 90, 40, None));
    for want in ["Private key file", " Browse... ", " Use this key "] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
}

#[test]
fn generating_uploads_after_listing_and_shows_the_key_line() {
    let mut h = H::new();
    let (mut page, t, ssh_dir) = ssh_page(&mut h);
    act(&mut page, &mut h, Item::SshGenerate);
    act(&mut page, &mut h, Item::SshGo);
    wait_ssh(&mut page, &mut h);
    let ssh = h.config.integrations[0].ssh.clone().expect("attached");
    assert_eq!(ssh.mode, SshMode::Generated);
    assert_eq!(ssh.remote_key_id.as_deref(), Some("77"));
    assert!(ssh_dir
        .join("gitpoulpi_gitlab-self-managed_ed25519")
        .is_file());
    assert_eq!(t.requests()[0].method, gitpoulpi_hosting::Method::Get);
    let saved = std::fs::read_to_string(&h.path).unwrap();
    assert!(saved.contains("remote_key_id"), "{saved}");
    let screen = text(&draw(&mut page, &h, 90, 40, None));
    for want in [
        "generated key",
        "SHA256:",
        " Remove ",
        " Delete ",
        "added to gitlab.example.com",
    ] {
        assert!(screen.contains(want), "missing {want:?}:\n{screen}");
    }
    // The radio is gone once a key is attached.
    assert!(!screen.contains("Generate new key"));
}

#[test]
fn remove_detaches_and_keeps_files_delete_confirms_naming_the_paths() {
    let mut h = H::new();
    let (mut page, t, ssh_dir) = ssh_page(&mut h);
    act(&mut page, &mut h, Item::SshGenerate);
    act(&mut page, &mut h, Item::SshGo);
    wait_ssh(&mut page, &mut h);
    let private = ssh_dir.join("gitpoulpi_gitlab-self-managed_ed25519");

    act(&mut page, &mut h, Item::SshRemove);
    wait_ssh(&mut page, &mut h);
    assert!(h.config.integrations[0].ssh.is_none());
    assert!(private.is_file(), "Remove keeps the files");
    assert!(t
        .requests()
        .iter()
        .any(|r| r.method == gitpoulpi_hosting::Method::Delete));

    // Delete: nothing happens until the confirmation is accepted.
    h.config.integrations[0].ssh = Some(IntegrationSsh {
        mode: SshMode::Generated,
        private_key_path: Some(private.display().to_string()),
        public_key_path: Some(format!("{}.pub", private.display())),
        title: Some("t".into()),
        remote_key_id: None,
    });
    act(&mut page, &mut h, Item::SshDelete);
    let screen = text(&draw(&mut page, &h, 140, 40, None));
    assert!(
        screen.contains("Delete this key and its files?"),
        "{screen}"
    );
    assert!(
        screen.contains("gitpoulpi_gitlab-self-managed_ed25519.pub"),
        "{screen}"
    );
    assert!(
        screen.contains("gitpoulpi_gitlab-self-managed_ed25519"),
        "{screen}"
    );
    act(&mut page, &mut h, Item::SshCancelDelete);
    assert!(private.is_file(), "Keep changes nothing");
    assert!(h.config.integrations[0].ssh.is_some());

    act(&mut page, &mut h, Item::SshDelete);
    act(&mut page, &mut h, Item::SshConfirmDelete);
    wait_ssh(&mut page, &mut h);
    assert!(!private.exists());
    assert!(h.config.integrations[0].ssh.is_none());
}

#[test]
fn a_custom_key_is_attached_from_a_typed_path_and_a_missing_file_is_refused() {
    let mut h = H::new();
    let (mut page, _, ssh_dir) = ssh_page(&mut h);
    std::fs::create_dir_all(&ssh_dir).unwrap();
    let k = ssh_dir.join("id_work");
    act(&mut page, &mut h, Item::SshCustom);
    page.ssh.path = k.display().to_string();
    act(&mut page, &mut h, Item::SshUse);
    assert!(page.ssh.error.is_some(), "the file does not exist yet");
    assert!(h.config.integrations[0].ssh.is_none());
    std::fs::write(&k, "x").unwrap();
    act(&mut page, &mut h, Item::SshUse);
    let ssh = h.config.integrations[0].ssh.clone().unwrap();
    assert_eq!(ssh.mode, SshMode::Custom);
    assert_eq!(ssh.remote_key_id, None);
}

#[test]
fn browse_opens_the_file_picker_and_choosing_fills_the_path() {
    let mut h = H::new();
    let (mut page, _, ssh_dir) = ssh_page(&mut h);
    act(&mut page, &mut h, Item::SshCustom);
    act(&mut page, &mut h, Item::SshBrowse);
    assert!(page.ssh.picker.is_some());
    assert!(page.text_focused(&h.config), "keys go to the picker");
    let screen = text(&draw(&mut page, &h, 120, 40, None));
    assert!(screen.contains("Choose a private key file"), "{screen}");
    page.picker_outcome(PickerOutcome::Choose(ssh_dir.join("id_x")));
    assert!(page.ssh.picker.is_none());
    assert!(page.ssh.path.ends_with("id_x"));
    // Esc in the picker only closes the picker.
    act(&mut page, &mut h, Item::SshBrowse);
    let nav = key(&mut page, &mut h, KeyCode::Esc);
    assert_eq!(nav, Nav::Stay);
    assert!(page.ssh.picker.is_none());
}

#[test]
fn every_ssh_control_takes_the_strong_hover_and_is_reachable_by_keyboard() {
    let mut h = H::new();
    let (mut page, _, _) = ssh_page(&mut h);
    for item in [Item::SshGenerate, Item::SshCustom] {
        act(&mut page, &mut h, item);
        let _ = draw(&mut page, &h, 90, 40, None);
        let items = page.items(&h.config);
        let targets: Vec<(Rect, Hit, bool)> = page
            .hits
            .iter()
            .filter(|(_, hit, _)| matches!(hit, Hit::Item(i) if items.contains(i) && i.is_ssh()))
            .cloned()
            .collect();
        assert!(!targets.is_empty());
        for (r, hit, hoverable) in targets {
            if !hoverable || matches!(hit, Hit::Item(Item::SshTitle | Item::SshPath)) {
                continue;
            }
            let b = draw(&mut page, &h, 90, 40, Some((r.x + 1, r.y)));
            assert_eq!(b[(r.x + 1, r.y)].bg, th().ui.button_hover, "{hit:?}");
        }
    }
    // Keyboard: every SSH item is in the focus order.
    let items = page.items(&h.config);
    for want in [
        Item::SshDefault,
        Item::SshGenerate,
        Item::SshCustom,
        Item::SshPath,
    ] {
        assert!(items.contains(&want), "{want:?} in {items:?}");
    }
}
