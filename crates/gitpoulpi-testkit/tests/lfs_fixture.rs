//! The LFS half of the fixture builder, proven against the real `git-lfs`
//! (skipped, or failed under `GITPOULPI_REQUIRE_LFS=1`, when it is not
//! installed): pointers in the object database, `file://` transfers,
//! `.lfsconfig` resolution in clones, and the loopback lock server.

use std::process::Command;

use gitpoulpi_testkit::lfs::LfsLockServer;
use gitpoulpi_testkit::{require_git_lfs, FixtureRepoBuilder};

const CONTENT: &[u8] = b"binary\0payload that is definitely not a pointer";

fn oid_of(bytes: &[u8]) -> String {
    // `git hash-object` is SHA-1; LFS wants SHA-256, so ask git-lfs itself.
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("f");
    std::fs::write(&file, bytes).unwrap();
    let out = hermetic(
        Command::new("git")
            .args(["lfs", "pointer", "--file"])
            .arg(&file),
    )
    .output()
    .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    text.lines()
        .find_map(|l| l.strip_prefix("oid sha256:"))
        .expect("git lfs pointer prints an oid")
        .to_string()
}

fn hermetic(cmd: &mut Command) -> &mut Command {
    cmd.env("GIT_CONFIG_GLOBAL", "")
        .env("GIT_CONFIG_SYSTEM", "")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
}

#[test]
fn a_tracked_file_is_a_pointer_in_git_and_real_bytes_on_disk() {
    require_git_lfs!("a_tracked_file_is_a_pointer_in_git_and_real_bytes_on_disk");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.bin")
        .unwrap()
        .commit_lfs_file("x.bin", CONTENT, "add x")
        .unwrap()
        .build();

    let stored = repo.git(&["cat-file", "blob", "HEAD:x.bin"]);
    assert!(stored.starts_with("version https://git-lfs"), "{stored}");
    assert!(stored.contains(&format!("size {}", CONTENT.len())));
    assert_eq!(std::fs::read(repo.root().join("x.bin")).unwrap(), CONTENT);
}

#[test]
fn a_push_to_the_file_origin_fills_its_lfs_store_and_a_clone_smudges() {
    require_git_lfs!("a_push_to_the_file_origin_fills_its_lfs_store_and_a_clone_smudges");
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs_origin()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track("*.bin")
        .unwrap()
        .commit_lfs_file("x.bin", CONTENT, "add x")
        .unwrap()
        .build();
    assert!(repo.remote_lfs_objects().is_empty());

    repo.git(&["push", "origin", "main"]);
    assert_eq!(repo.remote_lfs_objects(), [oid_of(CONTENT)]);

    // A clone finds the endpoint through the committed `.lfsconfig` and
    // smudges real content: proof the offline transfer path works end to
    // end (Windows `file:///C:/...` included).
    let dest_dir = tempfile::tempdir().unwrap();
    let dest = dest_dir.path().join("clone");
    let origin = repo.remote_path().unwrap().to_string_lossy().to_string();
    let out = hermetic(
        Command::new("git")
            .args([
                "-c",
                "filter.lfs.clean=git-lfs clean -- %f",
                "-c",
                "filter.lfs.smudge=git-lfs smudge -- %f",
                "-c",
                "filter.lfs.process=git-lfs filter-process",
                "-c",
                "filter.lfs.required=true",
                "clone",
                &origin,
            ])
            .arg(&dest),
    )
    .output()
    .unwrap();
    assert!(
        out.status.success(),
        "clone failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(std::fs::read(dest.join("x.bin")).unwrap(), CONTENT);
}

fn lfs(repo: &gitpoulpi_testkit::FixtureRepo, args: &[&str]) -> std::process::Output {
    hermetic(
        Command::new("git")
            .arg("lfs")
            .args(args)
            .current_dir(repo.root()),
    )
    .output()
    .unwrap()
}

#[test]
fn the_lock_server_round_trips_lock_list_unlock() {
    require_git_lfs!("the_lock_server_round_trips_lock_list_unlock");
    let server = LfsLockServer::start();
    let repo = FixtureRepoBuilder::new()
        .unwrap()
        .with_lfs()
        .unwrap()
        .lfs_track_lockable("*.bin")
        .unwrap()
        .commit_lfs_file("x.bin", CONTENT, "add x")
        .unwrap()
        .config("lfs.url", &server.url())
        .unwrap()
        .config("credential.helper", "")
        .unwrap()
        .build();

    let out = lfs(&repo, &["lock", "--json", "x.bin"]);
    assert!(
        out.status.success(),
        "lock failed: {} {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(server.locked_paths(), ["x.bin"]);

    let out = lfs(&repo, &["locks", "--json"]);
    let listed: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(listed[0]["path"], "x.bin");

    let out = lfs(&repo, &["unlock", "--json", "x.bin"]);
    assert!(
        out.status.success(),
        "unlock failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(server.locked_paths().is_empty());
}
