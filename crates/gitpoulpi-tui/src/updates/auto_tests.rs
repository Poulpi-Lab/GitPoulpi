//! The self-update's state machine over a temp install folder and a recorded
//! transport (no socket, no real exe touched).

use std::sync::atomic::AtomicBool;

use super::auto::{self, Phase};
use super::*;
use gitpoulpi_config::release::{Marker, MARKER_FILE};
use gitpoulpi_hosting::updates::Asset;
use gitpoulpi_hosting::RecordedTransport;
use gitpoulpi_update::fsops::RealFs;
use gitpoulpi_update::{managed_names, Updatable};
use xtask::dist::{marker_entry, write_tar_gz, Entry};

const TARGET: &str = "x86_64-unknown-linux-gnu";

fn lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// An install folder at `old` with the three binaries and the marker.
fn install_dir(old: &str) -> (tempfile::TempDir, Updatable) {
    let dir = tempfile::tempdir().unwrap();
    for name in managed_names(TARGET) {
        let text = if name == MARKER_FILE {
            Marker::new(old, TARGET).render()
        } else {
            format!("{name} {old}")
        };
        std::fs::write(dir.path().join(&name), text).unwrap();
    }
    let updatable = Updatable {
        dir: dir.path().to_path_buf(),
        exe_name: "gitpoulpi-tui".into(),
        marker: Marker::new(old, TARGET),
    };
    (dir, updatable)
}

fn archive(version: &str) -> Vec<u8> {
    let mut entries: Vec<Entry> = managed_names(TARGET)
        .into_iter()
        .filter(|n| n != MARKER_FILE)
        .map(|name| Entry {
            data: format!("new {name} {version}").into_bytes(),
            name,
            executable: true,
        })
        .collect();
    entries.push(marker_entry(version, TARGET));
    write_tar_gz(&format!("gitpoulpi-{version}-{TARGET}"), &entries).unwrap()
}

fn release(version: &str) -> Release {
    let name = format!("gitpoulpi-{version}-{TARGET}.tar.gz");
    let url = format!("https://github.com/o/r/releases/download/v{version}/{name}");
    Release {
        tag: format!("v{version}"),
        version: version.into(),
        prerelease: false,
        notes: String::new(),
        page_url: String::new(),
        asset: Some(Asset {
            name: name.clone(),
            url: url.clone(),
            size: 0,
        }),
        checksum: Some(Asset {
            name: format!("{name}.sha256"),
            url: format!("{url}.sha256"),
            size: 0,
        }),
    }
}

fn served(version: &str, hash_of: &[u8]) -> RecordedTransport {
    let bytes = archive(version);
    let sum = format!(
        "{}  gitpoulpi-{version}-{TARGET}.tar.gz\n",
        gitpoulpi_update::fetch::sha256_hex(hash_of)
    );
    RecordedTransport::new()
        .on_file(".tar.gz.sha256", sum.into_bytes())
        .on_file(".tar.gz", bytes)
}

fn reset(updatable: Option<Updatable>) {
    let mut s = shared();
    s.mode = UpdateMode::Notify;
    s.known = None;
    s.skipped = false;
    s.message = None;
    auto::set_updatable(&mut s, updatable, &RealFs);
}

#[test]
fn a_build_without_the_marker_never_updates_itself() {
    let _lock = lock();
    reset(None);
    let snap = snapshot();
    assert!(!snap.install.updatable);
    assert_eq!(
        auto::update_now().as_deref(),
        Some(msg::WHY_NOT_UPDATABLE.get())
    );
    assert_eq!(
        auto::restart_now().as_deref(),
        Some(msg::WHY_NOT_UPDATABLE.get())
    );
    let whys = whys();
    assert_eq!(whys.update_now, Some(msg::WHY_NOT_UPDATABLE.get()));
    assert_eq!(whys.revert, Some(msg::WHY_NOT_UPDATABLE.get()));
    assert_eq!(snapshot().install.phase, Phase::Idle, "nothing started");
}

#[test]
fn a_download_is_staged_beside_the_old_files_and_nothing_running_changes() {
    let _lock = lock();
    let (dir, updatable) = install_dir("0.1.0");
    let release = release("99.0.0");
    let bytes = archive("99.0.0");
    let transport = served("99.0.0", &bytes);
    auto::fetch_and_stage(
        &transport,
        &RealFs,
        &release,
        &updatable,
        &AtomicBool::new(false),
        &|_, _| {},
        &[],
    )
    .unwrap();
    for name in managed_names(TARGET) {
        assert!(
            dir.path().join(format!("{name}.new")).exists(),
            "{name}.new"
        );
    }
    assert_eq!(
        std::fs::read_to_string(dir.path().join("gitpoulpi-tui")).unwrap(),
        "gitpoulpi-tui 0.1.0",
        "the running binary is untouched until the restart"
    );
    // The next start finds it staged and offers the restart.
    reset(Some(updatable));
    assert_eq!(snapshot().install.phase, Phase::Ready("99.0.0".into()));
    assert_eq!(chip::text(), msg::CHIP_RESTART.get());
    assert_eq!(whys().restart, None);
    assert_eq!(whys().update_now, Some(msg::WHY_UPDATE_READY.get()));
}

#[test]
fn a_checksum_mismatch_stages_nothing() {
    let _lock = lock();
    let (dir, updatable) = install_dir("0.1.0");
    let transport = served("99.0.0", b"not the archive");
    let err = auto::fetch_and_stage(
        &transport,
        &RealFs,
        &release("99.0.0"),
        &updatable,
        &AtomicBool::new(false),
        &|_, _| {},
        &[],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        gitpoulpi_update::UpdateError::ChecksumMismatch { .. }
    ));
    assert!(!dir.path().join("gitpoulpi-tui.new").exists());
}

#[test]
fn a_missing_checksum_or_a_wrong_asset_name_stages_nothing() {
    let _lock = lock();
    let (dir, updatable) = install_dir("0.1.0");
    let bytes = archive("99.0.0");
    let transport = served("99.0.0", &bytes);
    let mut no_sum = release("99.0.0");
    no_sum.checksum = None;
    let err = auto::fetch_and_stage(
        &transport,
        &RealFs,
        &no_sum,
        &updatable,
        &AtomicBool::new(false),
        &|_, _| {},
        &[],
    )
    .unwrap_err();
    assert_eq!(err, gitpoulpi_update::UpdateError::NoChecksum);
    let mut wrong = release("99.0.0");
    wrong.asset.as_mut().unwrap().name = "gitpoulpi-99.0.0-aarch64-apple-darwin.tar.gz".into();
    let err = auto::fetch_and_stage(
        &transport,
        &RealFs,
        &wrong,
        &updatable,
        &AtomicBool::new(false),
        &|_, _| {},
        &[],
    )
    .unwrap_err();
    assert!(matches!(
        err,
        gitpoulpi_update::UpdateError::WrongAsset { .. }
    ));
    assert!(transport.requests().is_empty());
    assert!(!dir.path().join("gitpoulpi-tui.new").exists());
}

#[test]
fn finishing_a_download_sets_the_phase_the_message_and_the_job() {
    let _lock = lock();
    let (_dir, updatable) = install_dir("0.1.0");
    reset(Some(updatable));
    let job = crate::jobs_ui::local_start("x".into(), std::sync::Arc::new(|| {}));
    auto::finish_download(job, "99.0.0", Ok(()));
    assert_eq!(snapshot().install.phase, Phase::Ready("99.0.0".into()));
    assert_eq!(
        take_message().as_deref(),
        Some(msg::msg_update_ready("99.0.0").as_str())
    );
    auto::finish_download(job, "99.0.0", Err(gitpoulpi_update::UpdateError::Canceled));
    assert_eq!(snapshot().install.phase, Phase::Idle);
    auto::finish_download(
        job,
        "99.0.0",
        Err(gitpoulpi_update::UpdateError::NoChecksum),
    );
    assert!(matches!(snapshot().install.phase, Phase::Failed(_)));
}

#[test]
fn restart_swaps_the_files_keeps_the_old_ones_and_revert_brings_them_back() {
    let _lock = lock();
    let (dir, updatable) = install_dir("0.1.0");
    let bytes = archive("99.0.0");
    auto::fetch_and_stage(
        &served("99.0.0", &bytes),
        &RealFs,
        &release("99.0.0"),
        &updatable,
        &AtomicBool::new(false),
        &|_, _| {},
        &[],
    )
    .unwrap();
    reset(Some(updatable.clone()));
    auto::swap_files(&RealFs, &updatable, false).unwrap();
    let live = |n: &str| std::fs::read_to_string(dir.path().join(n)).unwrap();
    assert_eq!(live("gitpoulpi-tui"), "new gitpoulpi-tui 99.0.0");
    assert_eq!(live("gitpoulpi-tui.old"), "gitpoulpi-tui 0.1.0");
    // The new start offers the revert.
    reset(Some(updatable.clone()));
    assert_eq!(snapshot().install.revert_to.as_deref(), Some("0.1.0"));
    assert_eq!(whys().revert, None);
    auto::swap_files(&RealFs, &updatable, true).unwrap();
    assert_eq!(live("gitpoulpi-tui"), "gitpoulpi-tui 0.1.0");
    // The cleanup after a good start removes what is left to go back to.
    auto::cleanup_now(&RealFs);
    assert!(!dir.path().join("gitpoulpi-tui.old").exists());
    assert_eq!(snapshot().install.revert_to, None);
}

#[test]
fn a_written_job_asks_the_loop_to_quit_and_a_failed_handoff_stays_ready() {
    let _lock = lock();
    let (_dir, updatable) = install_dir("0.1.0");
    reset(Some(updatable));
    let prepared = handoff::Prepared {
        updater: "/data/updates/bin/gitpoulpi-updater".into(),
        job: "/data/updates/job.json".into(),
    };
    auto::finish_handoff(Ok(prepared), "99.0.0", Phase::Ready("99.0.0".into()));
    assert!(auto::take_quit_request());
    assert!(!auto::take_quit_request(), "once");
    assert!(take_message().unwrap().contains("99.0.0"));
    // The updater could not be prepared: nothing ends, the download stays.
    shared().install.phase = Phase::Installing("99.0.0".into());
    auto::finish_handoff(
        Err("this installation has no updater program".into()),
        "99.0.0",
        Phase::Ready("99.0.0".into()),
    );
    assert_eq!(snapshot().install.phase, Phase::Ready("99.0.0".into()));
    assert!(!auto::take_quit_request());
    let said = take_message().unwrap();
    assert!(said.contains("no updater"), "{said}");
}

#[test]
fn restart_with_nothing_staged_or_for_a_build_without_files_says_why_and_starts_nothing() {
    let _lock = lock();
    reset(None);
    assert_eq!(
        auto::restart_now().as_deref(),
        Some(msg::WHY_NOT_UPDATABLE.get()),
        "no install record"
    );
    let (_dir, updatable) = install_dir("0.1.0");
    reset(Some(updatable));
    assert_eq!(
        auto::restart_now().as_deref(),
        Some(msg::WHY_NOTHING_STAGED.get())
    );
    assert_eq!(snapshot().install.phase, Phase::Idle);
    assert!(!auto::take_quit_request());
}

#[test]
fn a_staged_download_of_an_older_version_is_dropped() {
    let _lock = lock();
    let (dir, updatable) = install_dir("0.1.0");
    // Staged marker says 0.0.5: not newer than this build (0.1.0 and up).
    for name in managed_names(TARGET) {
        let text = if name == MARKER_FILE {
            "version=0.0.5\ntarget=x86_64-unknown-linux-gnu\n".to_string()
        } else {
            "stale".into()
        };
        std::fs::write(dir.path().join(format!("{name}.new")), text).unwrap();
    }
    reset(Some(updatable));
    assert_eq!(snapshot().install.phase, Phase::Idle);
    assert!(!dir.path().join("gitpoulpi-tui.new").exists());
}

fn tab_text(slot: &mut notes::ReleaseNotesSlot) -> String {
    use ratatui::backend::TestBackend;
    let mut terminal = ratatui::Terminal::new(TestBackend::new(110, 30)).unwrap();
    terminal
        .draw(|frame| slot.render(frame, ratatui::layout::Rect::new(0, 1, 110, 28)))
        .unwrap();
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

fn known_release() {
    let mut s = shared();
    s.known = Some(Release {
        asset: None,
        checksum: None,
        notes: "the notes".into(),
        ..release("99.0.0")
    });
}

#[test]
fn the_tab_of_a_cargo_build_explains_why_it_cannot_update_and_has_no_update_button() {
    let _lock = lock();
    reset(None);
    known_release();
    let mut slot = notes::ReleaseNotesSlot::default();
    slot.open();
    let text = tab_text(&mut slot);
    assert!(text.contains("cannot update itself"), "{text}");
    assert!(text.contains("Download"), "{text}");
    assert!(
        !text.lines().nth(2).unwrap().contains("Update now"),
        "{text}"
    );
    assert!(!text.contains("u update now"), "{text}");
}

#[test]
fn the_tab_of_a_release_build_offers_update_now_then_restart_then_revert() {
    let _lock = lock();
    let (dir, updatable) = install_dir("0.1.0");
    reset(Some(updatable.clone()));
    known_release();
    let mut slot = notes::ReleaseNotesSlot::default();
    slot.open();
    let text = tab_text(&mut slot);
    assert!(
        text.lines().nth(2).unwrap().contains("Update now"),
        "{text}"
    );
    assert!(text.contains("u update now"), "the key hint: {text}");
    assert!(!text.contains("cannot update itself"), "{text}");
    // A release without an archive for this platform says so instead of starting.
    let said = auto::update_now().unwrap();
    assert!(said.contains("no archive"), "{said}");
    assert_eq!(snapshot().install.phase, Phase::Idle, "nothing started");

    // Staged: the header, the button and the chip say restart.
    let bytes = archive("99.0.0");
    auto::fetch_and_stage(
        &served("99.0.0", &bytes),
        &RealFs,
        &release("99.0.0"),
        &updatable,
        &AtomicBool::new(false),
        &|_, _| {},
        &[],
    )
    .unwrap();
    reset(Some(updatable.clone()));
    known_release();
    let text = tab_text(&mut slot);
    assert!(
        text.contains("Version 99.0.0 is ready: restart to update"),
        "{text}"
    );
    assert!(text.contains("Restart to update"), "{text}");
    assert!(
        !text.lines().nth(2).unwrap().contains(" Update now "),
        "{text}"
    );
    assert_eq!(chip::text(), "Restart to update");

    // After the swap the previous version can be restored.
    auto::swap_files(&RealFs, &updatable, false).unwrap();
    reset(Some(updatable));
    let text = tab_text(&mut slot);
    assert!(text.contains("Revert to 0.1.0"), "{text}");
    assert!(dir.path().join("gitpoulpi-tui.old").exists());
}

#[test]
fn a_running_download_shows_its_percentage_and_no_second_start() {
    let _lock = lock();
    let (_dir, updatable) = install_dir("0.1.0");
    reset(Some(updatable));
    known_release();
    {
        let mut s = shared();
        s.install.phase = Phase::Downloading("99.0.0".into());
    }
    auto::set_progress(50, Some(200));
    let mut slot = notes::ReleaseNotesSlot::default();
    slot.open();
    let text = tab_text(&mut slot);
    assert!(text.contains("Downloading version 99.0.0... 25%"), "{text}");
    assert!(
        !text.lines().nth(2).unwrap().contains("Update now"),
        "{text}"
    );
    assert_eq!(
        auto::update_now().as_deref(),
        Some(msg::WHY_UPDATE_BUSY.get())
    );
    assert_eq!(whys().restart, Some(msg::WHY_UPDATE_BUSY.get()));
}

/// The policy's `download_base` mirror: staged from only when it is passed.
#[test]
fn a_policy_mirror_download_is_staged_only_when_the_mirror_is_allowed() {
    let _lock = lock();
    let (_dir, updatable) = install_dir("0.1.0");
    let mut mirrored = release("99.0.0");
    let name = format!("gitpoulpi-99.0.0-{TARGET}.tar.gz");
    let url = format!("https://mirror.example/gp/{name}");
    mirrored.asset.as_mut().unwrap().url = url.clone();
    mirrored.checksum.as_mut().unwrap().url = format!("{url}.sha256");
    let bytes = archive("99.0.0");
    let stage = |hosts: &[String]| {
        auto::fetch_and_stage(
            &served("99.0.0", &bytes),
            &RealFs,
            &mirrored,
            &updatable,
            &AtomicBool::new(false),
            &|_, _| {},
            hosts,
        )
    };
    assert!(matches!(
        stage(&[]).unwrap_err(),
        gitpoulpi_update::UpdateError::Download(_)
    ));
    stage(&["https://mirror.example/gp/".to_string()]).unwrap();
}
