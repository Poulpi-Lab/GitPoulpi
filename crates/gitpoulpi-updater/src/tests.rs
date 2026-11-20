//! The updater against fakes: every machine effect is recorded in one log,
//! the files are real (a temp folder) so the swap is the engine's own.

use std::collections::BTreeMap;
use std::path::PathBuf;

use gitpoulpi_config::release::{self, InstallChannel, Marker, MARKER_FILE};
use gitpoulpi_testkit::signing::TestSigner;
use gitpoulpi_update::fsops::RealFs;
use gitpoulpi_update::job::{
    Action, JobElevation, Phase, Relaunch, Staged, UpdateJob, UpdateStatus, SCHEMA,
};
use gitpoulpi_update::managed_names;

use crate::env::Env;
use crate::fakes::*;
use crate::flow;
use crate::ops::{Clock, Proc, RunError};
use crate::outcome::Outcome;

const FROM: &str = "1.0.0";
const TO: &str = "2.0.0";

struct Fx {
    _tmp: tempfile::TempDir,
    install: PathBuf,
    updates: PathBuf,
    job_path: PathBuf,
    config: PathBuf,
    target: String,
}

impl Fx {
    fn status_path(&self) -> PathBuf {
        self.updates.join("status.json")
    }

    fn status(&self) -> UpdateStatus {
        UpdateStatus::parse(&std::fs::read_to_string(self.status_path()).unwrap()).unwrap()
    }

    fn read(&self, name: &str) -> String {
        std::fs::read_to_string(self.install.join(name)).unwrap_or_default()
    }

    fn exe(&self, binary: &str) -> String {
        release::exe_name(binary, &self.target)
    }
}

/// An install of version 1.0.0 with `channel` in its marker, and the
/// version 2.0.0 files staged beside it as `.new`.
fn fx(channel: Option<InstallChannel>) -> Fx {
    let tmp = tempfile::tempdir().unwrap();
    let install = tmp.path().join("install");
    let updates = tmp.path().join("data").join("updates");
    let config = tmp.path().join("config");
    for d in [&install, &updates, &config] {
        std::fs::create_dir_all(d).unwrap();
    }
    let target = release::host_target();
    let marker = |version: &str, channel| Marker {
        channel,
        ..Marker::new(version, &target)
    };
    for name in managed_names(&target) {
        if name == MARKER_FILE {
            std::fs::write(install.join(&name), marker(FROM, channel).render()).unwrap();
            std::fs::write(
                install.join(format!("{name}.new")),
                marker(TO, Some(InstallChannel::Archive)).render(),
            )
            .unwrap();
        } else {
            std::fs::write(install.join(&name), format!("v:{FROM}\n")).unwrap();
            std::fs::write(install.join(format!("{name}.new")), format!("v:{TO}\n")).unwrap();
        }
    }
    Fx {
        job_path: updates.join("job.json"),
        _tmp: tmp,
        install,
        updates,
        config,
        target,
    }
}

fn job(fx: &Fx, action: Action, elevation: JobElevation) -> UpdateJob {
    UpdateJob {
        schema: SCHEMA,
        action,
        install_dir: fx.install.clone(),
        from_version: FROM.into(),
        to_version: TO.into(),
        staged: Staged::InPlace,
        manager_argv: None,
        elevation,
        relaunch: Relaunch {
            exe: fx.install.join(fx.exe("gitpoulpi-tui")),
            args: vec!["--session".into(), "work".into()],
            cwd: fx.config.clone(),
        },
        requester_pid: 4242,
        config_dir: fx.config.clone(),
        status_path: fx.status_path(),
        environment: None,
        install_id: None,
    }
}

fn put(fx: &Fx, job: &UpdateJob) {
    std::fs::write(&fx.job_path, job.render()).unwrap();
}

struct World {
    log: Log,
    clock: FakeClock,
    proc: FakeProc,
    daemons: FakeDaemons,
    registry: FakeRegistry,
    transfer: FakeTransfer,
    chan: FakeChannelEnv,
    policy: FakePolicy,
    own_exe: PathBuf,
    signer: TestSigner,
    retired: &'static [&'static str],
}

impl World {
    fn new(fx: &Fx) -> Self {
        let log = new_log();
        World {
            clock: FakeClock::new(1_000),
            proc: FakeProc::new(&log),
            daemons: FakeDaemons {
                log: log.clone(),
                names: vec![],
                stubborn: false,
                down: Default::default(),
                lock_path: fx.updates.join("updating.lock"),
                scope: Default::default(),
            },
            registry: FakeRegistry {
                log: log.clone(),
                ..Default::default()
            },
            transfer: FakeTransfer {
                log: log.clone(),
                urls: BTreeMap::new(),
            },
            chan: FakeChannelEnv { writable: true },
            policy: FakePolicy {
                file: None,
                registry: None,
            },
            own_exe: fx.updates.join("bin").join(fx.exe("gitpoulpi-updater")),
            signer: TestSigner::new(),
            retired: &[],
            log,
        }
    }

    fn env(&self) -> Env<'_> {
        Env {
            fs: &RealFs,
            proc: &self.proc,
            daemons: &self.daemons,
            registry: &self.registry,
            clock: &self.clock,
            transfer: &self.transfer,
            channel_env: &self.chan,
            policy: &self.policy,
            host_target: release::host_target(),
            keys: vec![gitpoulpi_update::signing::PublicKey::decode(
                &self.signer.public_key_text(),
            )
            .expect("the test key parses")],
            own_exe: self.own_exe.clone(),
            own_pid: 77,
            echo: false,
            retired: self.retired,
        }
    }

    fn lines(&self) -> Vec<String> {
        self.log.borrow().clone()
    }

    fn position(&self, prefix: &str) -> Option<usize> {
        self.lines().iter().position(|l| l.starts_with(prefix))
    }
}

fn run(fx: &Fx, w: &World, job: &UpdateJob) -> Outcome {
    put(fx, job);
    flow::coordinator(&w.env(), &fx.job_path)
}

fn process(pid: u32, fx: &Fx) -> Proc {
    Proc {
        pid,
        exe: fx.install.join(fx.exe("gitpoulpi-tui")),
    }
}

// --- job validation -------------------------------------------------------

#[test]
fn a_folder_without_a_marker_is_a_bad_job() {
    let fx = fx(Some(InstallChannel::Archive));
    std::fs::remove_file(fx.install.join(MARKER_FILE)).unwrap();
    let w = World::new(&fx);
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out.code(), 6, "{out:?}");
    assert!(!fx.updates.join("updating.lock").exists());
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{FROM}\n"));
}

#[test]
fn a_job_of_another_schema_is_a_bad_job() {
    let fx = fx(Some(InstallChannel::Archive));
    let w = World::new(&fx);
    let mut value: serde_json::Value =
        serde_json::to_value(job(&fx, Action::Apply, JobElevation::None)).unwrap();
    value["schema"] = 2.into();
    std::fs::write(&fx.job_path, value.to_string()).unwrap();
    let out = flow::coordinator(&w.env(), &fx.job_path);
    assert_eq!(out.code(), 6, "{out:?}");
    assert!(w.lines().is_empty(), "nothing is touched: {:?}", w.lines());
}

#[test]
fn a_job_that_does_not_fit_the_channel_on_disk_is_a_bad_job() {
    // The marker says Chocolatey, the job asks for a file swap.
    let fx = fx(Some(InstallChannel::Choco));
    let w = World::new(&fx);
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out.code(), 6, "{out:?}");
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{FROM}\n"));
    assert!(w.position("shutdown").is_none() && w.position("terminate").is_none());
}

#[test]
fn a_policy_that_forbids_self_update_refuses_with_4_before_anything_stops() {
    for policy in [
        "[updates]\nallow_self_update = false\n",
        "[updates]\nmode = \"off\"\n",
    ] {
        let fx = fx(Some(InstallChannel::Archive));
        let mut w = World::new(&fx);
        w.policy.file = Some(policy.into());
        w.daemons.names = vec!["work".into()];
        let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
        assert_eq!(out.code(), 4, "{policy}: {out:?}");
        assert!(w.position("shutdown").is_none(), "nothing was stopped");
        assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{FROM}\n"));
    }
}

// --- stopping ---------------------------------------------------------------

#[test]
fn daemons_are_asked_first_then_the_grace_runs_and_only_then_processes_end() {
    let fx = fx(Some(InstallChannel::Archive));
    let mut w = World::new(&fx);
    w.daemons.names = vec!["a".into(), "b".into()];
    w.proc.running.borrow_mut().push(process(100, &fx));
    let started = w.clock.now();
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out.code(), 0, "{out:?}");
    let (a, b, t) = (
        w.position("shutdown:a").unwrap(),
        w.position("shutdown:b").unwrap(),
        w.position("terminate:100:soft").unwrap(),
    );
    assert!(a < b && b < t, "{:?}", w.lines());
    assert!(
        w.clock.now() - started >= 5,
        "the process got its grace before it was terminated"
    );
}

#[test]
fn a_process_that_will_not_stop_gives_5_and_changes_nothing() {
    let fx = fx(Some(InstallChannel::Archive));
    let w = World::new(&fx);
    w.proc.running.borrow_mut().push(process(100, &fx));
    w.proc.stubborn.borrow_mut().push(100);
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out.code(), 5, "{out:?}");
    assert!(out.message().contains("100"), "{out:?}");
    assert!(w.position("terminate:100:force").is_some());
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{FROM}\n"));
    assert!(!fx.updates.join("updating.lock").exists(), "lock released");
    assert_eq!(fx.status().phase, Phase::Failed);
    // GitPoulpi still starts again.
    assert!(w.position("relaunch").is_some());
}

#[test]
fn a_daemon_that_never_goes_does_not_block_when_no_process_is_left() {
    let fx = fx(Some(InstallChannel::Archive));
    let mut w = World::new(&fx);
    w.daemons.names = vec!["work".into()];
    w.daemons.stubborn = true;
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out.code(), 0, "{out:?}");
}

// --- locks ----------------------------------------------------------------

#[test]
fn the_lock_is_held_while_stopping_and_gone_afterwards() {
    let fx = fx(Some(InstallChannel::Archive));
    let mut w = World::new(&fx);
    w.daemons.names = vec!["work".into()];
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out.code(), 0, "{out:?}");
    assert_eq!(w.lines()[0], "shutdown:work:lock=true");
    assert!(!fx.updates.join("updating.lock").exists());
    assert!(!fx.install.join(".gitpoulpi-updating").exists());
}

#[test]
fn a_live_lock_of_another_update_gives_3_and_a_stale_one_is_ignored() {
    let fx = fx(Some(InstallChannel::Archive));
    let w = World::new(&fx);
    let lock = fx.updates.join("updating.lock");
    std::fs::write(
        &lock,
        gitpoulpi_config::update_lock::render(5, w.clock.now() - 10),
    )
    .unwrap();
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out.code(), 3, "{out:?}");
    assert!(
        lock.exists(),
        "the other update's lock is not ours to remove"
    );
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{FROM}\n"));

    std::fs::write(
        &lock,
        gitpoulpi_config::update_lock::render(5, w.clock.now() - 601),
    )
    .unwrap();
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out.code(), 0, "{out:?}");
    assert!(!lock.exists());
}

// --- swapping, verifying, rolling back --------------------------------------

#[test]
fn an_inno_user_swap_updates_the_files_hkcu_and_relaunches_unelevated() {
    let fx = fx(Some(InstallChannel::InnoUser));
    let w = World::new(&fx);
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out, Outcome::Done(format!("Updated to {TO}")));
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{TO}\n"));
    assert_eq!(
        fx.read(&format!("{}.old", fx.exe("gitpoulpi"))),
        format!("v:{FROM}\n")
    );
    // The marker keeps the channel the install recorded.
    let marker = Marker::parse(&fx.read(MARKER_FILE)).unwrap();
    assert_eq!(
        (marker.version.as_str(), marker.channel),
        (TO, Some(InstallChannel::InnoUser))
    );
    assert!(
        w.lines().contains(&format!("display:CurrentUser:{TO}")),
        "{:?}",
        w.lines()
    );
    assert!(!w.lines().iter().any(|l| l.starts_with("run_elevated")));
    assert_eq!(fx.status().phase, Phase::Done);
    let relaunch = w
        .lines()
        .into_iter()
        .find(|l| l.starts_with("relaunch"))
        .unwrap();
    // Unix replaces the process (`true`); Windows starts a detached
    // launcher instead (`false`).
    let replace = if cfg!(windows) { "false" } else { "true" };
    assert!(
        relaunch.ends_with(&format!("--session work:{replace}")),
        "{relaunch}"
    );
}

#[test]
fn a_build_that_does_not_run_is_rolled_back_with_its_registry_entry() {
    let fx = fx(Some(InstallChannel::InnoUser));
    let w = World::new(&fx);
    *w.proc.reports.borrow_mut() = Some("9.9.9".into());
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out.code(), 1, "{out:?}");
    assert!(out.message().contains("put back"), "{out:?}");
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{FROM}\n"));
    assert_eq!(Marker::parse(&fx.read(MARKER_FILE)).unwrap().version, FROM);
    let display: Vec<String> = w
        .lines()
        .into_iter()
        .filter(|l| l.starts_with("display"))
        .collect();
    assert_eq!(
        display,
        [
            format!("display:CurrentUser:{TO}"),
            format!("display:CurrentUser:{FROM}")
        ]
    );
    assert_eq!(fx.status().phase, Phase::Failed);
}

#[test]
fn a_revert_swaps_the_old_files_back_and_verifies_them() {
    let fx = fx(Some(InstallChannel::Archive));
    for name in managed_names(&fx.target) {
        let _ = std::fs::remove_file(fx.install.join(format!("{name}.new")));
        let old = if name == MARKER_FILE {
            Marker {
                channel: Some(InstallChannel::Archive),
                ..Marker::new("0.9.0", &fx.target)
            }
            .render()
        } else {
            "v:0.9.0\n".to_string()
        };
        std::fs::write(fx.install.join(format!("{name}.old")), old).unwrap();
    }
    let w = World::new(&fx);
    let mut j = job(&fx, Action::Revert, JobElevation::None);
    j.from_version = FROM.into();
    j.to_version = "0.9.0".into();
    let out = run(&fx, &w, &j);
    assert_eq!(out, Outcome::Done("Went back to 0.9.0".into()));
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), "v:0.9.0\n");
}

#[test]
fn nothing_staged_is_exit_3() {
    let fx = fx(Some(InstallChannel::Archive));
    for name in managed_names(&fx.target) {
        let _ = std::fs::remove_file(fx.install.join(format!("{name}.new")));
    }
    let w = World::new(&fx);
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out.code(), 3, "{out:?}");
}

// --- the elevated step --------------------------------------------------------

fn worker_world(fx: &Fx) -> World {
    let mut w = World::new(fx);
    w.own_exe = fx.install.join(fx.exe("gitpoulpi-updater"));
    w.proc.alive.borrow_mut().push(55);
    w
}

fn archive_bytes(fx: &Fx) -> Vec<u8> {
    use xtask::dist::{marker_entry, write_tar_gz, write_zip, Entry};
    let mut entries: Vec<Entry> = release::BINARIES
        .iter()
        .map(|b| Entry {
            name: release::exe_name(b, &fx.target),
            data: format!("v:{TO}\n").into_bytes(),
            executable: true,
        })
        .collect();
    entries.push(marker_entry(TO, &fx.target));
    let stem = release::package_stem(TO, &fx.target);
    if release::is_windows(&fx.target) {
        write_zip(&stem, &entries).unwrap()
    } else {
        write_tar_gz(&stem, &entries).unwrap()
    }
}

const SUM_URL: &str = "https://github.com/Poulpi-Lab/GitPoulpi/releases/download/v2.0.0/x.sha256";
const SIG_URL: &str = "https://github.com/Poulpi-Lab/GitPoulpi/releases/download/v2.0.0/x.minisig";

/// A machine install whose archive sits in the user's cache.
fn archive_job(fx: &Fx, bytes: &[u8]) -> UpdateJob {
    for name in managed_names(&fx.target) {
        let _ = std::fs::remove_file(fx.install.join(format!("{name}.new")));
    }
    let cache = fx.updates.join(release::archive_name(TO, &fx.target));
    std::fs::write(&cache, bytes).unwrap();
    let mut j = job(fx, Action::Apply, JobElevation::Uac);
    j.staged = Staged::Archive {
        archive: cache,
        sha256_url: SUM_URL.into(),
        minisig_url: SIG_URL.into(),
    };
    j
}

/// What the release published as the archive's `.minisig`.
fn signature_for(w: &World, fx: &Fx, bytes: &[u8]) -> Vec<u8> {
    let name = release::archive_name(TO, &fx.target);
    w.signer.sign(&name, TO, bytes).into_bytes()
}

fn checksum_for(fx: &Fx, bytes: &[u8]) -> Vec<u8> {
    let name = release::archive_name(TO, &fx.target);
    xtask::dist::checksum_line(&xtask::dist::sha256_hex(bytes), &name).into_bytes()
}

#[test]
fn the_elevated_worker_verifies_stages_swaps_and_writes_the_machine_hive() {
    let fx = fx(Some(InstallChannel::InnoMachine));
    let mut w = worker_world(&fx);
    let bytes = archive_bytes(&fx);
    let j = archive_job(&fx, &bytes);
    put(&fx, &j);
    w.transfer
        .urls
        .insert(SUM_URL.into(), checksum_for(&fx, &bytes));
    let sig = signature_for(&w, &fx, &bytes);
    w.transfer.urls.insert(SIG_URL.into(), sig);
    // Another user's process the elevated look can see.
    w.proc.other_users.borrow_mut().push(Proc {
        pid: 900,
        exe: PathBuf::from("x"),
    });
    let out = flow::elevated(&w.env(), &fx.job_path, 55);
    assert_eq!(
        out,
        Outcome::Done(format!("Updated to {TO}")),
        "{:?}",
        w.lines()
    );
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{TO}\n"));
    assert!(w.lines().contains(&format!("display:LocalMachine:{TO}")));
    assert!(
        w.position("terminate:900").is_some(),
        "other users' processes end"
    );
    assert!(!w
        .lines()
        .iter()
        .any(|l| l.starts_with("display:CurrentUser")));
    assert!(!fx.install.join(".gitpoulpi-updating").exists());
}

#[test]
fn the_elevated_worker_refuses_an_archive_that_does_not_match_the_published_checksum() {
    let fx = fx(Some(InstallChannel::InnoMachine));
    let mut w = worker_world(&fx);
    let bytes = archive_bytes(&fx);
    let j = archive_job(&fx, &bytes);
    put(&fx, &j);
    // The checksum the release published is for other bytes: the staged
    // archive was swapped by a user-level process.
    w.transfer
        .urls
        .insert(SUM_URL.into(), checksum_for(&fx, b"the real archive"));
    // The signature is the real one: the checksum is what fails here.
    let sig = signature_for(&w, &fx, &bytes);
    w.transfer.urls.insert(SIG_URL.into(), sig);
    let out = flow::elevated(&w.env(), &fx.job_path, 55);
    assert_eq!(out.code(), 1, "{out:?}");
    assert!(out.message().contains("checksum"), "{out:?}");
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{FROM}\n"));
    for name in managed_names(&fx.target) {
        assert!(
            !fx.install.join(format!("{name}.new")).exists(),
            "{name} was staged"
        );
    }
}

/// The elevated worker's refusal of a staged archive: nothing staged, nothing
/// swapped, the reason names the signature.
fn assert_refused_untouched(fx: &Fx, out: &Outcome, reason: &str) {
    assert_eq!(out.code(), 1, "{out:?}");
    assert!(out.message().contains(reason), "{out:?}");
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{FROM}\n"));
    for name in managed_names(&fx.target) {
        assert!(
            !fx.install.join(format!("{name}.new")).exists(),
            "{name} was staged"
        );
    }
}

#[test]
fn the_elevated_worker_refuses_a_tampered_staged_archive_even_with_a_matching_checksum() {
    let fx = fx(Some(InstallChannel::InnoMachine));
    let mut w = worker_world(&fx);
    let real = archive_bytes(&fx);
    // A user-level process swapped the staged file for its own build, and
    // cannot sign: the release's checksum and signature are for `real`.
    let mut tampered = real.clone();
    tampered.extend_from_slice(b"extra");
    let j = archive_job(&fx, &tampered);
    put(&fx, &j);
    w.transfer
        .urls
        .insert(SUM_URL.into(), checksum_for(&fx, &tampered));
    let sig = signature_for(&w, &fx, &real);
    w.transfer.urls.insert(SIG_URL.into(), sig);
    let out = flow::elevated(&w.env(), &fx.job_path, 55);
    assert_refused_untouched(&fx, &out, "signature");
}

#[test]
fn the_elevated_worker_refuses_an_archive_without_a_signature() {
    let fx = fx(Some(InstallChannel::InnoMachine));
    let mut w = worker_world(&fx);
    let bytes = archive_bytes(&fx);
    let j = archive_job(&fx, &bytes);
    put(&fx, &j);
    w.transfer
        .urls
        .insert(SUM_URL.into(), checksum_for(&fx, &bytes));
    // Nothing at the .minisig URL (404).
    let out = flow::elevated(&w.env(), &fx.job_path, 55);
    assert_refused_untouched(&fx, &out, "no signature");
}

#[test]
fn the_elevated_worker_refuses_a_signature_for_another_file_or_version_or_key() {
    let fx = fx(Some(InstallChannel::InnoMachine));
    let bytes = archive_bytes(&fx);
    let name = release::archive_name(TO, &fx.target);
    let stranger = TestSigner::new();
    // (the signature text: which signer, which file, which version, the
    // reason the refusal gives)
    for (case, reason) in [
        ("other file", "signature is for"),
        ("other version", "signature is for"),
        ("other key", "signature does not match"),
    ] {
        let mut w = worker_world(&fx);
        let j = archive_job(&fx, &bytes);
        put(&fx, &j);
        w.transfer
            .urls
            .insert(SUM_URL.into(), checksum_for(&fx, &bytes));
        let sig = match case {
            "other file" => w.signer.sign("other.zip", TO, &bytes),
            "other version" => w.signer.sign(&name, "1.0.0", &bytes),
            _ => stranger.sign(&name, TO, &bytes),
        };
        w.transfer.urls.insert(SIG_URL.into(), sig.into_bytes());
        let out = flow::elevated(&w.env(), &fx.job_path, 55);
        assert_refused_untouched(&fx, &out, reason);
    }
}

#[test]
fn the_elevated_worker_refuses_a_signature_from_a_host_the_policy_does_not_allow() {
    let fx = fx(Some(InstallChannel::InnoMachine));
    let mut w = worker_world(&fx);
    let bytes = archive_bytes(&fx);
    let mut j = archive_job(&fx, &bytes);
    if let Staged::Archive { minisig_url, .. } = &mut j.staged {
        *minisig_url = "https://evil.example/x.minisig".into();
    }
    put(&fx, &j);
    w.transfer
        .urls
        .insert(SUM_URL.into(), checksum_for(&fx, &bytes));
    let sig = signature_for(&w, &fx, &bytes);
    w.transfer
        .urls
        .insert("https://evil.example/x.minisig".into(), sig);
    let out = flow::elevated(&w.env(), &fx.job_path, 55);
    assert_eq!(out.code(), 1, "{out:?}");
    assert!(
        w.position("get:https://evil.example").is_none(),
        "no request to a host that is not allowed"
    );
}

#[test]
fn the_elevated_worker_refuses_a_checksum_from_a_host_the_policy_does_not_allow() {
    let fx = fx(Some(InstallChannel::InnoMachine));
    let mut w = worker_world(&fx);
    let bytes = archive_bytes(&fx);
    let mut j = archive_job(&fx, &bytes);
    if let Staged::Archive { sha256_url, .. } = &mut j.staged {
        *sha256_url = "https://evil.example/x.sha256".into();
    }
    put(&fx, &j);
    w.transfer.urls.insert(
        "https://evil.example/x.sha256".into(),
        checksum_for(&fx, &bytes),
    );
    let out = flow::elevated(&w.env(), &fx.job_path, 55);
    assert_eq!(out.code(), 1, "{out:?}");
    assert!(
        w.position("get:").is_none(),
        "no request to a host that is not allowed"
    );
}

#[test]
fn the_elevated_worker_only_runs_from_the_installed_copy_and_for_a_live_parent() {
    let fx = fx(Some(InstallChannel::InnoMachine));
    put(&fx, &job(&fx, Action::Apply, JobElevation::Uac));
    // The user-writable copy under the data folder.
    let mut w = worker_world(&fx);
    w.own_exe = fx.updates.join("bin").join(fx.exe("gitpoulpi-updater"));
    assert_eq!(flow::elevated(&w.env(), &fx.job_path, 55).code(), 6);
    // Installed copy, but the coordinator that asked is gone.
    let w = worker_world(&fx);
    w.proc.alive.borrow_mut().clear();
    assert_eq!(flow::elevated(&w.env(), &fx.job_path, 55).code(), 6);
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{FROM}\n"));
}

#[test]
fn a_declined_elevation_prompt_gives_2_and_changes_nothing() {
    let fx = fx(Some(InstallChannel::InnoMachine));
    let w = World::new(&fx);
    *w.proc.elevated.borrow_mut() = Some(Err(RunError::Declined));
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::Uac));
    assert_eq!(out, Outcome::Declined);
    assert_eq!(out.code(), 2);
    assert_eq!(fx.status().phase, Phase::Declined);
    assert_eq!(fx.read(&fx.exe("gitpoulpi")), format!("v:{FROM}\n"));
    assert!(
        fx.install
            .join(format!("{}.new", fx.exe("gitpoulpi")))
            .exists(),
        "still staged"
    );
    let elevated = w
        .lines()
        .into_iter()
        .find(|l| l.starts_with("run_elevated"))
        .unwrap();
    assert!(
        elevated.contains(&format!("{} elevated --job", fx.exe("gitpoulpi-updater"))),
        "{elevated}"
    );
    assert!(elevated.ends_with("--parent 77"), "{elevated}");
}

// --- the package manager ------------------------------------------------------

#[test]
fn the_manager_command_is_the_plans_run_elevated_only_where_the_plan_says() {
    // Chocolatey: through the elevation prompt.
    let fx1 = fx(Some(InstallChannel::Choco));
    let w = World::new(&fx1);
    *w.proc.reports.borrow_mut() = Some(TO.into());
    let mut j = job(&fx1, Action::Manager, JobElevation::Uac);
    j.manager_argv = Some(
        ["choco", "upgrade", "gitpoulpi", "-y"]
            .map(String::from)
            .to_vec(),
    );
    let out = run(&fx1, &w, &j);
    assert_eq!(
        out,
        Outcome::Done(format!("Updated to {TO}")),
        "{:?}",
        w.lines()
    );
    assert!(w
        .lines()
        .contains(&"run_elevated:choco upgrade gitpoulpi -y".to_string()));

    // Scoop: plain run, and its revert command is accepted too.
    let fx2 = fx(Some(InstallChannel::Scoop));
    let w = World::new(&fx2);
    *w.proc.reports.borrow_mut() = Some(TO.into());
    let mut j = job(&fx2, Action::Manager, JobElevation::None);
    j.manager_argv = Some(
        ["scoop", "reset", "gitpoulpi@2.0.0"]
            .map(String::from)
            .to_vec(),
    );
    let out = run(&fx2, &w, &j);
    assert_eq!(out.code(), 0, "{out:?} {:?}", w.lines());
    assert!(w
        .lines()
        .contains(&"run:scoop reset gitpoulpi@2.0.0".to_string()));
}

#[test]
fn a_manager_command_that_is_not_the_plans_is_refused_and_never_run() {
    let fx = fx(Some(InstallChannel::Choco));
    let w = World::new(&fx);
    let mut j = job(&fx, Action::Manager, JobElevation::Uac);
    j.manager_argv = Some(["cmd", "/c", "evil"].map(String::from).to_vec());
    let out = run(&fx, &w, &j);
    assert_eq!(out.code(), 6, "{out:?}");
    assert!(w.position("run").is_none(), "{:?}", w.lines());
}

#[test]
fn a_manager_that_fails_or_leaves_another_version_is_a_failure() {
    let fx = fx(Some(InstallChannel::Homebrew));
    let w = World::new(&fx);
    w.proc.run_code.set(1);
    let mut j = job(&fx, Action::Manager, JobElevation::None);
    j.manager_argv = Some(["brew", "upgrade", "gitpoulpi"].map(String::from).to_vec());
    assert_eq!(run(&fx, &w, &j).code(), 1);
    w.proc.run_code.set(0);
    *w.proc.reports.borrow_mut() = Some("1.0.0".into());
    let out = run(&fx, &w, &j);
    assert_eq!(out.code(), 1, "{out:?}");
}

// --- relaunch -------------------------------------------------------------------

#[test]
fn unix_replaces_the_process_and_windows_goes_through_the_launcher() {
    let fx = fx(Some(InstallChannel::Archive));
    let w = World::new(&fx);
    let j = job(&fx, Action::Apply, JobElevation::None);

    let unix = flow::relaunch_plan(&w.env(), &j);
    assert!(unix.replace_process || release::is_windows(&fx.target));

    let mut env = w.env();
    env.host_target = "x86_64-pc-windows-msvc".into();
    std::fs::write(fx.install.join("gitpoulpi-launch.exe"), b"").unwrap();
    let win = flow::relaunch_plan(&env, &j);
    assert!(!win.replace_process);
    assert_eq!(win.program, fx.install.join("gitpoulpi-launch.exe"));
    assert_eq!(
        win.args,
        [
            "--new-window".to_string(),
            "--".into(),
            j.relaunch.exe.to_string_lossy().into_owned(),
            "--session".into(),
            "work".into()
        ]
    );
    assert_eq!(win.cwd, j.relaunch.cwd);
    std::fs::remove_file(fx.install.join("gitpoulpi-launch.exe")).unwrap();
    let bare = flow::relaunch_plan(&env, &j);
    assert_eq!(bare.program, j.relaunch.exe);
    assert!(!bare.replace_process);

    let mut env = w.env();
    env.host_target = "x86_64-unknown-linux-gnu".into();
    let unix = flow::relaunch_plan(&env, &j);
    assert!(unix.replace_process);
    assert_eq!(
        (unix.program, unix.args),
        (j.relaunch.exe.clone(), j.relaunch.args.clone())
    );
}

#[test]
fn a_missing_or_retired_relaunch_exe_restarts_gitpoulpi() {
    let fx = fx(Some(InstallChannel::Archive));
    let mut w = World::new(&fx);
    let j = job(&fx, Action::Apply, JobElevation::None);
    let new = fx.install.join(fx.exe("gitpoulpi"));

    // Recorded exe present and not retired: kept.
    let kept = flow::relaunch_plan(&w.env(), &j);
    assert_eq!(kept.program, fx.install.join(fx.exe("gitpoulpi-tui")));

    // Retired name: gitpoulpi, same arguments.
    w.retired = &["gitpoulpi-tui"];
    let plan = flow::relaunch_plan(&w.env(), &j);
    assert_eq!(plan.program, new);
    assert_eq!(plan.args, j.relaunch.args);
    assert!(plan.replace_process || release::is_windows(&fx.target));

    // Missing file (a user who skipped versions): gitpoulpi as well.
    w.retired = &[];
    std::fs::remove_file(fx.install.join(fx.exe("gitpoulpi-tui"))).unwrap();
    assert_eq!(flow::relaunch_plan(&w.env(), &j).program, new);

    // Windows goes through the launcher with the mapped target.
    let mut env = w.env();
    env.host_target = "x86_64-pc-windows-msvc".into();
    std::fs::write(fx.install.join("gitpoulpi-launch.exe"), b"").unwrap();
    let win = flow::relaunch_plan(&env, &j);
    assert_eq!(
        win.args[2],
        env.exe_in(&fx.install, "gitpoulpi").to_string_lossy()
    );
}

#[test]
fn a_swap_deletes_the_retired_files_and_their_old_copies() {
    let fx = fx(Some(InstallChannel::Archive));
    let mut w = World::new(&fx);
    w.retired = &["gitpoulpi-mcp"];
    let name = fx.exe("gitpoulpi-mcp");
    for suffix in [".swap", ".new"] {
        std::fs::write(fx.install.join(format!("{name}{suffix}")), b"x").unwrap();
    }
    let out = run(&fx, &w, &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(out.code(), 0, "{out:?}");
    for suffix in ["", ".old", ".swap", ".new"] {
        assert!(
            !fx.install.join(format!("{name}{suffix}")).exists(),
            "{name}{suffix} should be gone"
        );
    }
    // What is still shipped stays.
    assert!(fx.install.join(fx.exe("gitpoulpi")).exists());
    assert!(fx.install.join(fx.exe("gitpoulpi-tui")).exists());
}

// --- the command line -------------------------------------------------------------

#[test]
fn the_command_line_is_parsed() {
    use crate::{parse_args, Command};
    let a = |s: &[&str]| s.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    assert_eq!(parse_args(&a(&["--version"])), Ok(Command::Version));
    assert_eq!(
        parse_args(&a(&["run", "--job", "j.json"])),
        Ok(Command::Run {
            job: "j.json".into()
        })
    );
    assert_eq!(
        parse_args(&a(&["elevated", "--job", "j.json", "--parent", "9"])),
        Ok(Command::Elevated {
            job: "j.json".into(),
            parent: 9
        })
    );
    assert!(parse_args(&a(&["run"])).is_err());
    assert!(parse_args(&a(&["elevated", "--job", "j"])).is_err());
    assert!(parse_args(&a(&[])).is_err());
}

// --- installations: whose daemons, whose files ------------------------------

fn port(dir: &std::path::Path, name: &str, pid: Option<u32>) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join(format!("{name}.port")), "1").unwrap();
    if let Some(pid) = pid {
        std::fs::write(dir.join(format!("{name}.pid")), pid.to_string()).unwrap();
    }
}

fn names(found: Vec<crate::ops::SessionDaemon>) -> Vec<String> {
    found.into_iter().map(|d| d.name).collect()
}

#[test]
fn the_updater_stops_only_its_envs_daemons() {
    let tmp = tempfile::tempdir().unwrap();
    let sessions = tmp.path().join("sessions");
    port(&sessions.join("env").join("a"), "mine", None);
    port(&sessions.join("env").join("b"), "theirs", None);
    let scope = |env: &str| crate::ops::StopScope {
        config_dir: tmp.path().to_path_buf(),
        env: Some(env.into()),
        install_pids: vec![],
    };
    assert_eq!(names(crate::real::scan_sessions(&scope("a"))), vec!["mine"]);
    assert_eq!(
        names(crate::real::scan_sessions(&scope("b"))),
        vec!["theirs"]
    );

    // The job's environment is what reaches the daemon lookup.
    let fx = fx(Some(InstallChannel::Archive));
    let w = World::new(&fx);
    let mut j = job(&fx, Action::Apply, JobElevation::None);
    j.environment = Some("a".into());
    crate::stop::stop_daemons(&w.env(), &j);
    let seen = w.daemons.scope.borrow().clone().unwrap();
    assert_eq!(seen.env.as_deref(), Some("a"));
}

#[test]
fn a_flat_layout_daemon_is_stopped_only_when_hosted_from_this_install_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let sessions = tmp.path().join("sessions");
    port(&sessions, "old-mine", Some(100));
    port(&sessions, "old-other", Some(200));
    port(&sessions, "old-unknown", None);
    port(&sessions.join("env").join("a"), "mine", None);
    let scope = crate::ops::StopScope {
        config_dir: tmp.path().to_path_buf(),
        env: Some("a".into()),
        install_pids: vec![100],
    };
    assert_eq!(
        names(crate::real::scan_sessions(&scope)),
        vec!["mine", "old-mine"]
    );
    // A job without an environment (an old build) still reaches its own
    // flat daemons, and only those.
    let flat_only = crate::ops::StopScope { env: None, ..scope };
    assert_eq!(
        names(crate::real::scan_sessions(&flat_only)),
        vec!["old-mine"]
    );

    // The pids come from the process image lookup of the install folder.
    let fx = fx(Some(InstallChannel::Archive));
    let w = World::new(&fx);
    w.proc.running.borrow_mut().push(process(100, &fx));
    crate::stop::stop_daemons(&w.env(), &job(&fx, Action::Apply, JobElevation::None));
    assert_eq!(
        w.daemons.scope.borrow().clone().unwrap().install_pids,
        vec![100]
    );
}

#[test]
fn status_and_lock_live_under_the_installation() {
    let fx = fx(Some(InstallChannel::Archive));
    let own = fx.updates.join("abc123");
    std::fs::create_dir_all(&own).unwrap();
    let mut w = World::new(&fx);
    w.daemons.names = vec!["work".into()];
    w.daemons.lock_path = own.join("updating.lock");
    let mut j = job(&fx, Action::Apply, JobElevation::None);
    j.status_path = own.join("status.json");
    j.install_id = Some("abc123".into());
    let out = run(&fx, &w, &j);
    assert_eq!(out.code(), 0, "{out:?}");
    assert_eq!(
        w.lines()[0],
        "shutdown:work:lock=true",
        "the lock sat in the installation's folder"
    );
    assert!(own.join("status.json").exists());
    assert!(!own.join("updating.lock").exists());
    assert!(
        !fx.updates.join("status.json").exists() && !fx.updates.join("updating.lock").exists(),
        "nothing in the shared folder"
    );
}
