use super::*;
use crate::testutil::FakeFs;

const LINUX: &str = "x86_64-unknown-linux-gnu";
const WIN: &str = "x86_64-pc-windows-msvc";

#[derive(Default)]
struct FakeReg {
    user: Option<String>,
    machine: Option<String>,
}

impl RegistryOps for FakeReg {
    fn inno_install_location(&self, hive: Hive) -> Option<String> {
        match hive {
            Hive::CurrentUser => self.user.clone(),
            Hive::LocalMachine => self.machine.clone(),
        }
    }
}

struct FakeEnv {
    cargo_home: Option<PathBuf>,
    writable: bool,
}

impl ChannelEnv for FakeEnv {
    fn cargo_home(&self) -> Option<PathBuf> {
        self.cargo_home.clone()
    }
    fn dir_writable(&self, _dir: &Path) -> bool {
        self.writable
    }
}

fn env(writable: bool) -> FakeEnv {
    FakeEnv {
        cargo_home: Some(PathBuf::from("/home/u/.cargo")),
        writable,
    }
}

fn marker_text(target: &str, channel: Option<InstallChannel>, feed: Option<&str>) -> String {
    Marker {
        channel,
        feed: feed.map(str::to_string),
        ..Marker::new("0.1.0", target)
    }
    .render()
}

fn put_marker(fs: &FakeFs, dir: &str, text: &str) {
    fs.put(format!("{dir}/{MARKER_FILE}"), text);
}

#[test]
fn a_marker_with_a_channel_decides() {
    for c in InstallChannel::ALL {
        let fs = FakeFs::new(false);
        put_marker(&fs, "/opt/gp", &marker_text(LINUX, Some(c), None));
        let r = resolve(
            &fs,
            &NoRegistry,
            &env(false),
            Path::new("/opt/gp/gitpoulpi-tui"),
            LINUX,
        );
        assert_eq!(r.channel, Channel::Install(c));
        assert_eq!(r.how, How::Marker);
        assert_eq!(r.dir, Path::new("/opt/gp"));
        assert_eq!(r.exe_name, "gitpoulpi-tui");
    }
}

#[test]
fn an_internal_feed_is_kept() {
    let fs = FakeFs::new(false);
    let text = marker_text(
        WIN,
        Some(InstallChannel::ChocoInternal),
        Some("https://nexus.example/choco/"),
    );
    put_marker(&fs, "/c/gp", &text);
    let r = resolve(
        &fs,
        &NoRegistry,
        &env(false),
        Path::new("/c/gp/gitpoulpi.exe"),
        WIN,
    );
    assert_eq!(r.feed(), Some("https://nexus.example/choco/"));
}

#[test]
fn a_marker_for_another_platform_is_dev_not_legacy() {
    let fs = FakeFs::new(false);
    put_marker(
        &fs,
        "/opt/gp",
        &marker_text(WIN, Some(InstallChannel::Archive), None),
    );
    let r = resolve(
        &fs,
        &NoRegistry,
        &env(true),
        Path::new("/opt/gp/gitpoulpi-tui"),
        LINUX,
    );
    assert_eq!((r.channel, r.how), (Channel::Dev, How::Dev));
    assert_eq!(r.marker, None);
}

#[test]
fn legacy_in_a_writable_folder_resolves_to_archive() {
    let fs = FakeFs::new(false);
    put_marker(&fs, "/opt/gp", &marker_text(LINUX, None, None));
    let r = resolve(
        &fs,
        &NoRegistry,
        &env(true),
        Path::new("/opt/gp/gitpoulpi-tui"),
        LINUX,
    );
    assert_eq!(r.channel, Channel::Install(InstallChannel::Archive));
    assert_eq!(r.how, How::Legacy);
    assert!(r.marker.is_some(), "the old marker is kept for the rewrite");
}

#[test]
fn legacy_in_a_read_only_folder_only_announces() {
    let fs = FakeFs::new(false);
    put_marker(&fs, "/opt/gp", &marker_text(LINUX, None, None));
    let r = resolve(
        &fs,
        &NoRegistry,
        &env(false),
        Path::new("/opt/gp/gitpoulpi-tui"),
        LINUX,
    );
    assert_eq!(r.channel, Channel::LegacyReadOnly);
}

#[test]
fn an_unknown_channel_value_is_legacy() {
    let fs = FakeFs::new(false);
    put_marker(
        &fs,
        "/opt/gp",
        "version=9.0.0\ntarget=x86_64-unknown-linux-gnu\nchannel=snap\n",
    );
    let r = resolve(
        &fs,
        &NoRegistry,
        &env(true),
        Path::new("/opt/gp/gitpoulpi-tui"),
        LINUX,
    );
    assert_eq!(r.how, How::Legacy);
}

#[test]
fn a_legacy_windows_install_is_told_apart_by_the_inno_uninstall_key() {
    let exe = Path::new("/progs/GitPoulpi/gitpoulpi-tui.exe");
    // Per user: HKCU names this folder (Inno's trailing separator and case
    // differ from the path).
    let fs = FakeFs::new(true);
    put_marker(&fs, "/progs/GitPoulpi", &marker_text(WIN, None, None));
    let user = FakeReg {
        user: Some("/PROGS/GitPoulpi/".into()),
        ..FakeReg::default()
    };
    let r = resolve(&fs, &user, &env(true), exe, WIN);
    assert_eq!(r.channel, Channel::Install(InstallChannel::InnoUser));
    assert_eq!(r.how, How::InnoUninstallKey);
    // All users: HKLM.
    let machine = FakeReg {
        machine: Some("/progs/GitPoulpi".into()),
        ..FakeReg::default()
    };
    let r = resolve(&fs, &machine, &env(false), exe, WIN);
    assert_eq!(r.channel, Channel::Install(InstallChannel::InnoMachine));
    // A key for another folder proves nothing: back to the writability rule.
    let other = FakeReg {
        user: Some("/elsewhere".into()),
        ..FakeReg::default()
    };
    let r = resolve(&fs, &other, &env(true), exe, WIN);
    assert_eq!(r.how, How::Legacy);
}

#[test]
fn inno_backslash_locations_compare_equal() {
    assert!(same_dir(
        r"C:\Program Files\GitPoulpi\",
        r"c:\program files\gitpoulpi"
    ));
    assert!(!same_dir(r"C:\a\b", r"C:\a\c"));
}

#[test]
fn the_inno_key_is_not_asked_on_unix() {
    let fs = FakeFs::new(false);
    put_marker(&fs, "/opt/gp", &marker_text(LINUX, None, None));
    let reg = FakeReg {
        user: Some("/opt/gp".into()),
        ..FakeReg::default()
    };
    let r = resolve(
        &fs,
        &reg,
        &env(true),
        Path::new("/opt/gp/gitpoulpi-tui"),
        LINUX,
    );
    assert_eq!(r.how, How::Legacy);
}

fn cargo_fs(record: &str) -> FakeFs {
    let fs = FakeFs::new(false);
    fs.put("/home/u/.cargo/.crates2.json", record);
    fs
}

#[test]
fn a_cargo_install_is_known_by_cargos_own_record() {
    let fs = cargo_fs(
        r#"{"installs":{"gitpoulpi-cli 0.3.0 (git+https://github.com/Poulpi-Lab/GitPoulpi?tag=releases/v0.3.0#abc)":{"bins":["gitpoulpi"]}}}"#,
    );
    let r = resolve(
        &fs,
        &NoRegistry,
        &env(true),
        Path::new("/home/u/.cargo/bin/gitpoulpi"),
        LINUX,
    );
    assert_eq!((r.channel, r.how), (Channel::Cargo, How::CargoRecord));
}

#[test]
fn cargo_bin_without_a_record_of_ours_is_dev() {
    let exe = Path::new("/home/u/.cargo/bin/gitpoulpi");
    let other = cargo_fs(r#"{"installs":{"ripgrep 14.0.0 (registry+x)":{}}}"#);
    let r = resolve(&other, &NoRegistry, &env(true), exe, LINUX);
    assert_eq!(r.channel, Channel::Dev);
    let none = FakeFs::new(false);
    assert_eq!(
        resolve(&none, &NoRegistry, &env(true), exe, LINUX).channel,
        Channel::Dev
    );
    let garbage = cargo_fs("not json");
    assert_eq!(
        resolve(&garbage, &NoRegistry, &env(true), exe, LINUX).channel,
        Channel::Dev
    );
}

#[test]
fn a_record_does_not_make_a_binary_elsewhere_cargo() {
    let fs = cargo_fs(r#"{"installs":{"gitpoulpi-cli 0.3.0 (x)":{}}}"#);
    let r = resolve(
        &fs,
        &NoRegistry,
        &env(true),
        Path::new("/data/Projects/gitpoulpi/target/release/gitpoulpi"),
        LINUX,
    );
    assert_eq!(r.channel, Channel::Dev, "a repo build is never updated");
}

#[test]
fn no_marker_and_no_cargo_home_is_dev() {
    let fs = FakeFs::new(false);
    let env = FakeEnv {
        cargo_home: None,
        writable: true,
    };
    let r = resolve(&fs, &NoRegistry, &env, Path::new("/x/gitpoulpi-tui"), LINUX);
    assert_eq!((r.channel, r.how), (Channel::Dev, How::Dev));
}

#[test]
fn the_real_env_probes_writability_and_cleans_up() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(RealEnv.dir_writable(tmp.path()));
    assert_eq!(std::fs::read_dir(tmp.path()).unwrap().count(), 0);
    assert!(!RealEnv.dir_writable(&tmp.path().join("missing")));
}
