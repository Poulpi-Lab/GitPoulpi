//! The two rows that look at the machine: an SSH key pinned in the
//! repository's own `core.sshCommand` (shown read-only, moved into
//! GitPoulpi), and the permissions of the key file.

use super::*;
use gitpoulpi_core::domain::AuthOverride;

const KEY: &str = "/keys/work_ed25519";

fn pinned_page(dir: &std::path::Path, command: &str) -> (PreferencesState, RawQueue) {
    let (mut st, queue) = page(dir);
    st.ra_take_view(
        RepoAuthView {
            overrides: vec![AuthOverride {
                key: "core.sshcommand".into(),
                value: command.into(),
                scope: "local".into(),
                origin: "file:.git/config".into(),
                overridden: false,
            }],
            ..RepoAuthView::default()
        },
        RepoAuthProfile::default(),
    );
    queue.queued();
    (st, queue)
}

fn pin_command() -> String {
    gitpoulpi_core::repo_config::ssh_command_for_key(KEY)
}

#[test]
fn a_pin_of_the_repository_is_shown_read_only_with_a_move_button() {
    let dir = tempfile::tempdir().unwrap();
    let (st, _q) = pinned_page(dir.path(), &pin_command());
    let fields = st.fields();
    let row = fields
        .iter()
        .find(|f| f.label == "Pinned in git's config")
        .expect("the pin row");
    assert!(
        matches!(&row.control, Control::Info(v) if *v == pin_command()),
        "read-only: an info row, not a field"
    );
    assert!(button_enabled(&st, "Move into GitPoulpi"));
    // No pin, no row.
    let dir = tempfile::tempdir().unwrap();
    let (mut st, _q) = page(dir.path());
    loaded(&mut st);
    assert!(!labels(&st).iter().any(|l| l == "Pinned in git's config"));
}

#[test]
fn a_command_that_is_not_a_plain_key_pin_cannot_be_moved() {
    let dir = tempfile::tempdir().unwrap();
    let (st, _q) = pinned_page(dir.path(), "ssh -i /k/id -F /elsewhere/config");
    assert!(labels(&st).iter().any(|l| l == "Pinned in git's config"));
    assert!(
        !labels(&st).iter().any(|l| l == "Move into GitPoulpi"),
        "no button for a command that does more than name a key"
    );
    assert!(
        labels(&st).iter().any(|l| l.contains("cannot move it")),
        "{:?}",
        labels(&st)
    );
}

/// The move saves the equivalent key-file profile, and only when that is
/// saved asks the daemon to unset the local `core.sshCommand`.
#[test]
fn moving_the_pin_saves_the_key_profile_then_unsets_the_pin() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, queue) = pinned_page(dir.path(), &pin_command());
    press(&mut st, "Move into GitPoulpi");
    let sent = queue.queued();
    assert_eq!(sent.len(), 1, "only the profile first: {sent:?}");
    assert_eq!(sent[0].0, OperationKind::RepoAuthSet);
    let args: RepoAuthSetArgs = serde_json::from_value(sent[0].1.clone()).unwrap();
    assert_eq!(
        args.profile.ssh,
        Some(SshAuth::KeyFile {
            key: KEY.into(),
            known_hosts: None,
            strict_host_key: false,
        }),
        "the equivalent key_file profile"
    );
    assert!(
        queue
            .queued()
            .iter()
            .all(|(op, _)| *op != OperationKind::SetSshCommand),
        "the pin stays until the profile is saved"
    );

    // The save lands: now the pin goes, through SetSshCommand.
    st.repo_auth.set = Some(Reply::ready(Ok(serde_json::Value::Null)));
    st.poll_repo_auth();
    let sent = queue.queued();
    let unset = sent
        .iter()
        .find(|(op, _)| *op == OperationKind::SetSshCommand)
        .unwrap_or_else(|| panic!("no unset: {sent:?}"));
    assert_eq!(unset.1, serde_json::json!({ "command": null }));

    // The unset lands: the page says so and reads the view again.
    st.repo_auth.pin_move.unset = Some(Reply::ready(Ok(serde_json::Value::Null)));
    st.poll_repo_auth();
    let status = st.repo_auth.status.clone().unwrap();
    assert!(status.starts_with("Moved: "), "{status}");
    assert!(status.contains(KEY), "{status}");
    assert!(
        queue
            .queued()
            .iter()
            .any(|(op, _)| *op == OperationKind::RepoAuthGet),
        "the view is read again"
    );
}

#[test]
fn a_refused_save_leaves_the_pin_alone() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, queue) = pinned_page(dir.path(), &pin_command());
    press(&mut st, "Move into GitPoulpi");
    queue.queued();
    st.repo_auth.set = Some(Reply::ready(Err(
        gitpoulpi_core::Error::RepoAuthUnavailable {
            reason: "no".into(),
        },
    )));
    st.poll_repo_auth();
    assert!(
        queue
            .queued()
            .iter()
            .all(|(op, _)| *op != OperationKind::SetSshCommand),
        "nothing unsets the pin when the key was not saved"
    );
    assert!(st.repo_auth.pin_move.key.is_none());
}

#[test]
fn the_move_waits_for_unsaved_edits_and_for_a_free_ssh_choice() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, queue) = pinned_page(dir.path(), &pin_command());
    st.repo_auth.draft.username = "alice".into();
    press(&mut st, "Move into GitPoulpi");
    assert_eq!(
        st.repo_auth.status.as_deref(),
        Some("Save or revert your edits first.")
    );
    assert!(queue.queued().is_empty());

    let dir = tempfile::tempdir().unwrap();
    let (mut st, queue) = pinned_page(dir.path(), &pin_command());
    st.repo_auth.stored.ssh = Some(SshAuth::KeyFile {
        key: "/keys/other".into(),
        known_hosts: None,
        strict_host_key: false,
    });
    press(&mut st, "Move into GitPoulpi");
    assert!(
        st.repo_auth
            .status
            .as_deref()
            .is_some_and(|s| s.contains("already has a source")),
        "{:?}",
        st.repo_auth.status
    );
    assert!(
        queue.queued().is_empty(),
        "the existing choice is not overwritten"
    );
}

#[cfg(unix)]
mod acl {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    /// Polls the page until `done` or five seconds pass.
    fn poll_until(st: &mut PreferencesState, what: &str, done: impl Fn(&PreferencesState) -> bool) {
        let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !done(st) {
            assert!(std::time::Instant::now() < end, "timed out: {what}");
            st.poll_repo_auth();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    fn mode(path: &std::path::Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn a_key_other_accounts_can_read_gets_a_warning_and_a_button_that_fixes_it() {
        let dir = tempfile::tempdir().unwrap();
        let key = dir.path().join("id_work");
        std::fs::write(&key, "-----BEGIN OPENSSH PRIVATE KEY-----\n").unwrap();
        std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o644)).unwrap();
        let (mut st, _q) = page(dir.path());
        loaded(&mut st);
        st.repo_auth.draft.ssh = SshKind::KeyFile;
        st.repo_auth.draft.key = key.to_string_lossy().into_owned();

        poll_until(&mut st, "the warning", |st| st.ra_key_exposed());
        let fields = st.fields();
        let row = fields
            .iter()
            .find(|f| f.label == "Private key file")
            .expect("the key row");
        let warning = row.warning.clone().unwrap_or_default();
        assert!(
            warning.contains("OpenSSH refuses such a key"),
            "the warning explains why: {warning}"
        );
        assert!(button_enabled(&st, "Restrict to my account"));

        // The button (Enter on it is the keyboard twin: `press` activates).
        press(&mut st, "Restrict to my account");
        poll_until(&mut st, "the fix", |st| {
            mode(&key) == 0o600 && !st.ra_key_exposed()
        });
        assert_eq!(mode(&key), 0o600);
        assert!(
            !labels(&st).iter().any(|l| l == "Restrict to my account"),
            "the button goes with the warning"
        );
        assert_eq!(
            st.repo_auth.status.as_deref(),
            Some("The key file can now be read by your account only.")
        );
    }

    #[test]
    fn a_private_key_and_an_http_choice_show_no_warning() {
        let dir = tempfile::tempdir().unwrap();
        let key = dir.path().join("id_work");
        std::fs::write(&key, "k").unwrap();
        std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o600)).unwrap();
        let (mut st, _q) = page(dir.path());
        loaded(&mut st);
        st.repo_auth.draft.ssh = SshKind::KeyFile;
        st.repo_auth.draft.key = key.to_string_lossy().into_owned();
        // Give the check time to answer "private".
        for _ in 0..50 {
            st.poll_repo_auth();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert!(!st.ra_key_exposed());
        assert!(!labels(&st).iter().any(|l| l == "Restrict to my account"));
    }
}
