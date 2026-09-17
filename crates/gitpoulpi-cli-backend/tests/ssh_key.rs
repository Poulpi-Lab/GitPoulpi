//! `ssh.private_key_path` at the git boundary: a transfer's git process gets
//! `GIT_SSH_COMMAND` with the key only when nothing the user chose is already
//! in force. A stub `ssh` first on `PATH` records what git ran it with, so no
//! network is involved. The key and `PATH` are process-wide, so everything runs
//! in one test.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;

use gitpoulpi_cli_backend::{set_ssh_key, SystemGitBackend};
use gitpoulpi_core::domain::FetchOptions;
use gitpoulpi_core::GitBackend;
use gitpoulpi_testkit::FixtureRepoBuilder;

#[tokio::test]
async fn the_configured_key_fills_the_gap_and_never_overrides_the_users_own_choice() {
    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let log = dir.path().join("ssh-args.txt");
    let stub = bin.join("ssh");
    std::fs::write(
        &stub,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexit 255\n",
            log.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&stub, std::fs::Permissions::from_mode(0o755)).unwrap();
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![bin];
    paths.extend(std::env::split_paths(&path));
    std::env::set_var("PATH", std::env::join_paths(paths).unwrap());
    std::env::set_var("GIT_TERMINAL_PROMPT", "0");
    std::env::remove_var("GIT_SSH_COMMAND");
    std::env::remove_var("GIT_SSH");

    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .commit("initial")
        .unwrap()
        .build();
    repo.git(&["remote", "add", "origin", "ssh://git@example.invalid/r.git"]);
    let backend = SystemGitBackend::new();
    let fetch = || async {
        let _ = std::fs::remove_file(&log);
        let result = backend
            .fetch(&repo.repo_ref(), "origin", &FetchOptions::default())
            .await;
        assert!(result.is_err(), "the stub ssh always fails");
        std::fs::read_to_string(&log).unwrap_or_default()
    };

    // No key: ssh is left alone.
    set_ssh_key(None);
    let args = fetch().await;
    assert!(args.contains("git@example.invalid"), "ssh ran: {args}");
    assert!(!args.contains("IdentitiesOnly"), "{args}");

    // A key with spaces: offered exactly, and only that one.
    let key = dir.path().join("my keys").join("id work");
    set_ssh_key(Some(key.display().to_string()));
    let args = fetch().await;
    let lines: Vec<&str> = args.lines().collect();
    let at = lines.iter().position(|l| *l == "-i").expect(&args);
    assert_eq!(lines[at + 1], key.display().to_string(), "{args}");
    assert!(lines.contains(&"IdentitiesOnly=yes"), "{args}");

    // The repository's own pin (core.sshCommand) wins over the setting.
    repo.git(&["config", "core.sshCommand", "ssh -o Pinned=yes"]);
    let args = fetch().await;
    assert!(args.contains("Pinned=yes"), "{args}");
    assert!(!args.contains("IdentitiesOnly"), "{args}");
    repo.git(&["config", "--unset", "core.sshCommand"]);

    // So does a GIT_SSH_COMMAND the user exported.
    std::env::set_var("GIT_SSH_COMMAND", "ssh -o FromEnv=yes");
    let args = fetch().await;
    assert!(args.contains("FromEnv=yes"), "{args}");
    assert!(!args.contains("IdentitiesOnly"), "{args}");
    std::env::remove_var("GIT_SSH_COMMAND");

    // Cleared again: back to ssh's own defaults.
    set_ssh_key(Some("  ".into()));
    let args = fetch().await;
    assert!(!args.contains("IdentitiesOnly"), "{args}");
    set_ssh_key(None);
}
