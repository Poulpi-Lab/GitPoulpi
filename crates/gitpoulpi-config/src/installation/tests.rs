use super::*;
use std::collections::{HashMap, HashSet};

const LINUX: &str = "x86_64-unknown-linux-gnu";
const WINDOWS: &str = "x86_64-pc-windows-msvc";

/// A file system of text paths. On Windows flavour it compares case- and
/// separator-insensitively, like the real one.
#[derive(Default)]
struct FakeFs {
    windows: bool,
    files: HashMap<String, String>,
    dirs: HashSet<String>,
    links: HashMap<String, String>,
}

impl FakeFs {
    fn new(windows: bool) -> Self {
        Self {
            windows,
            ..Self::default()
        }
    }
    fn key(&self, p: &str) -> String {
        norm(p, self.windows)
    }
    fn file(&mut self, p: &str, text: &str) -> &mut Self {
        let k = self.key(p);
        self.files.insert(k, text.to_string());
        self
    }
    fn dir(&mut self, p: &str) -> &mut Self {
        let k = self.key(p);
        self.dirs.insert(k);
        self
    }
    fn marker(&mut self, dir: &str, channel: Option<InstallChannel>) -> &mut Self {
        let target = if self.windows { WINDOWS } else { LINUX };
        let mut m = Marker::new("1.0.0", target);
        m.channel = channel;
        let p = join(dir, MARKER_FILE, self.windows);
        self.file(&p, &m.render())
    }
    fn checkout(&mut self, root: &str) -> &mut Self {
        let w = self.windows;
        self.file(&join(root, "Cargo.toml", w), "[workspace]");
        let crates = join(root, "crates", w);
        self.dir(&join(&crates, "gitpoulpi-tui", w))
    }
}

impl InstallFs for FakeFs {
    fn canonicalize(&self, p: &str) -> Option<String> {
        Some(
            self.links
                .get(&self.key(p))
                .cloned()
                .unwrap_or_else(|| p.to_string()),
        )
    }
    fn is_file(&self, p: &str) -> bool {
        self.files.contains_key(&self.key(p))
    }
    fn is_dir(&self, p: &str) -> bool {
        self.dirs.contains(&self.key(p))
    }
    fn read_to_string(&self, p: &str) -> Option<String> {
        self.files.get(&self.key(p)).cloned()
    }
}

fn env(fs: &FakeFs) -> IdentityEnv<'_> {
    IdentityEnv {
        windows: fs.windows,
        host_target: if fs.windows { WINDOWS } else { LINUX }.to_string(),
        cargo_home: None,
        install_id: None,
        env_override: None,
        fs,
    }
}

fn id_of(fs: &FakeFs, exe: &str) -> Identity {
    identify(Path::new(exe), &env(fs))
}

// ------------------------------------------------------------- identity

#[test]
fn a_dev_checkout_is_one_installation_for_dist_release_and_debug() {
    for (windows, root, sep) in [
        (false, "/src/gitpoulpi", "/"),
        (true, r"D:\src\gitpoulpi", "\\"),
    ] {
        let mut fs = FakeFs::new(windows);
        fs.checkout(root);
        let exe = |profile: &str| format!("{root}{sep}target{sep}{profile}{sep}gitpoulpi");
        let ids: Vec<Identity> = ["dist", "release", "debug", "custom"]
            .iter()
            .map(|p| id_of(&fs, &exe(p)))
            .collect();
        assert!(ids.iter().all(|i| i.kind == Kind::Dev), "{ids:?}");
        assert!(ids.iter().all(|i| i.id == ids[0].id), "{ids:?}");
        assert_eq!(ids[0].root, root);
    }
}

#[test]
fn the_checkout_wins_over_a_marker_under_target() {
    let mut fs = FakeFs::new(false);
    fs.checkout("/src/gitpoulpi");
    let plain = id_of(&fs, "/src/gitpoulpi/target/dist/gitpoulpi");
    fs.marker(
        "/src/gitpoulpi/target/release",
        Some(InstallChannel::Archive),
    );
    let with_marker = id_of(&fs, "/src/gitpoulpi/target/release/gitpoulpi");
    assert_eq!(with_marker.kind, Kind::Dev);
    assert_eq!(with_marker.id, plain.id);
}

#[test]
fn a_legacy_marker_keeps_its_id_when_the_channel_is_written() {
    let mut fs = FakeFs::new(true);
    let exe = r"C:\Tools\GitPoulpi\gitpoulpi.exe";
    fs.marker(r"C:\Tools\GitPoulpi", None);
    let before = id_of(&fs, exe);
    fs.marker(r"C:\Tools\GitPoulpi", Some(InstallChannel::Choco));
    let after = id_of(&fs, exe);
    assert_eq!(before.kind, Kind::Install);
    assert_eq!(before.channel, None);
    assert_eq!(after.channel, Some(InstallChannel::Choco));
    assert_eq!(before.id, after.id);
}

#[test]
fn the_same_folder_updated_in_place_keeps_its_identity() {
    let mut fs = FakeFs::new(true);
    let exe = r"C:\ProgramData\chocolatey\lib\gitpoulpi\tools\gitpoulpi.exe";
    let dir = r"C:\ProgramData\chocolatey\lib\gitpoulpi\tools";
    fs.file(
        &join(dir, MARKER_FILE, true),
        &Marker::new("1.0.0", WINDOWS).render(),
    );
    let old = id_of(&fs, exe);
    fs.file(
        &join(dir, MARKER_FILE, true),
        &Marker::new("1.4.0", WINDOWS).render(),
    );
    assert_eq!(old.id, id_of(&fs, exe).id);
}

#[test]
fn inno_user_and_all_users_are_two_installations() {
    let mut fs = FakeFs::new(true);
    let user = r"C:\Users\me\AppData\Local\Programs\GitPoulpi";
    let all = r"C:\Program Files\GitPoulpi";
    fs.marker(user, Some(InstallChannel::InnoUser));
    fs.marker(all, Some(InstallChannel::InnoMachine));
    let a = id_of(&fs, &format!(r"{user}\gitpoulpi.exe"));
    let b = id_of(&fs, &format!(r"{all}\gitpoulpi.exe"));
    assert_ne!(a.id, b.id);
    assert_eq!((a.kind, b.kind), (Kind::Install, Kind::Install));
}

#[test]
fn windows_paths_ignore_case_verbatim_prefix_and_trailing_separator() {
    let mut fs = FakeFs::new(true);
    fs.marker(
        r"C:\Program Files\GitPoulpi",
        Some(InstallChannel::InnoMachine),
    );
    let plain = id_of(&fs, r"C:\Program Files\GitPoulpi\gitpoulpi.exe");
    for spelling in [
        r"\\?\C:\Program Files\GitPoulpi\gitpoulpi.exe",
        r"c:/program files/gitpoulpi/GITPOULPI.EXE",
        r"C:\PROGRAM FILES\GITPOULPI\\gitpoulpi.exe",
    ] {
        assert_eq!(id_of(&fs, spelling).id, plain.id, "{spelling}");
    }
    // The cargo folder is compared the same way.
    let mut e = env(&fs);
    e.cargo_home = Some(r"c:\USERS\me\.cargo\".to_string());
    let cargo = identify(Path::new(r"\\?\C:\Users\me\.cargo\bin\gitpoulpi.exe"), &e);
    assert_eq!(cargo.kind, Kind::Cargo);
    assert_eq!(cargo.root, r"C:\Users\me\.cargo\bin");
    // A UNC verbatim path loses its prefix.
    assert_eq!(tidy(r"\\?\UNC\srv\share\x", true), r"\\srv\share\x");
    // Unix paths stay case-sensitive.
    let unix = FakeFs::new(false);
    assert_ne!(
        id_of(&unix, "/Opt/gp/gitpoulpi").id,
        id_of(&unix, "/opt/gp/gitpoulpi").id
    );
}

#[test]
fn scoop_versions_map_to_current() {
    let mut fs = FakeFs::new(true);
    let app = r"C:\Users\me\scoop\apps\gitpoulpi";
    fs.dir(&format!(r"{app}\current"));
    for v in ["1.0.0", "1.1.0", "current"] {
        fs.marker(&format!(r"{app}\{v}"), Some(InstallChannel::Scoop));
    }
    let ids: Vec<Identity> = ["1.0.0", "1.1.0", "current"]
        .iter()
        .map(|v| id_of(&fs, &format!(r"{app}\{v}\gitpoulpi.exe")))
        .collect();
    assert!(ids.iter().all(|i| i.id == ids[0].id), "{ids:?}");
    assert_eq!(ids[0].root, format!(r"{app}\current"));
    // Without a `current` sibling the version folder is the installation.
    let mut bare = FakeFs::new(true);
    bare.marker(r"C:\x\apps\g\1.0.0", None);
    assert_eq!(
        id_of(&bare, r"C:\x\apps\g\1.0.0\gitpoulpi.exe").root,
        r"C:\x\apps\g\1.0.0"
    );
}

#[test]
fn homebrew_cellar_versions_map_to_the_formula() {
    let mut fs = FakeFs::new(false);
    let formula = "/opt/homebrew/Cellar/gitpoulpi";
    fs.marker(
        &format!("{formula}/1.0.0/bin"),
        Some(InstallChannel::Homebrew),
    );
    fs.marker(
        &format!("{formula}/1.1.0/bin"),
        Some(InstallChannel::Homebrew),
    );
    let a = id_of(&fs, &format!("{formula}/1.0.0/bin/gitpoulpi"));
    let b = id_of(&fs, &format!("{formula}/1.1.0/bin/gitpoulpi"));
    assert_eq!(a.id, b.id);
    assert_eq!(a.root, formula);
    assert_eq!(a.kind, Kind::Install);
}

#[test]
fn cargo_bin_is_the_cargo_installation() {
    let fs = FakeFs::new(false);
    let mut e = env(&fs);
    e.cargo_home = Some("/home/me/.cargo".to_string());
    let i = identify(Path::new("/home/me/.cargo/bin/gitpoulpi"), &e);
    assert_eq!(i.kind, Kind::Cargo);
    assert_eq!(i.root, "/home/me/.cargo/bin");
    // A neighbouring folder is not it.
    let other = identify(Path::new("/home/me/.cargo/other/gitpoulpi"), &e);
    assert_eq!(other.kind, Kind::Portable);
}

#[test]
fn no_marker_or_foreign_target_is_portable() {
    let mut fs = FakeFs::new(false);
    let plain = id_of(&fs, "/opt/gp/gitpoulpi");
    assert_eq!(plain.kind, Kind::Portable);
    assert_eq!(plain.root, "/opt/gp");
    // A marker for another platform's archive says nothing about this exe.
    fs.file(
        "/opt/gp/.gitpoulpi-dist",
        &Marker::new("1.0.0", WINDOWS).render(),
    );
    assert_eq!(id_of(&fs, "/opt/gp/gitpoulpi").kind, Kind::Portable);
    // Nor does an unparseable one.
    fs.file("/opt/gp/.gitpoulpi-dist", "garbage");
    assert_eq!(id_of(&fs, "/opt/gp/gitpoulpi").kind, Kind::Portable);
}

#[test]
fn a_moved_checkout_is_a_new_installation() {
    let mut fs = FakeFs::new(false);
    fs.checkout("/src/gitpoulpi");
    fs.checkout("/src/moved/gitpoulpi");
    assert_ne!(
        id_of(&fs, "/src/gitpoulpi/target/release/gitpoulpi").id,
        id_of(&fs, "/src/moved/gitpoulpi/target/release/gitpoulpi").id
    );
}

#[test]
fn the_env_override_wins_and_is_sanitised() {
    let fs = FakeFs::new(false);
    let mut e = env(&fs);
    let me = identify(Path::new("/opt/gp/gitpoulpi"), &e);
    let mut config = Config::default();
    config.installation.settings.insert(
        me.id.clone(),
        InstallationEntry {
            env: "Shared Env".to_string(),
            ..InstallationEntry::default()
        },
    );
    assert_eq!(environment(&config, &me, &e), "shared-env");
    e.env_override = Some("../My Env!".to_string());
    assert_eq!(environment(&config, &me, &e), "my-env");
    // An override that sanitises to nothing is ignored, not an empty folder name.
    e.env_override = Some("../..".to_string());
    assert_eq!(environment(&config, &me, &e), "shared-env");
    // The pinned id is sanitised the same way.
    e.install_id = Some("Pty Test/1".to_string());
    let pinned = identify(Path::new("/opt/gp/gitpoulpi"), &e);
    assert_eq!(
        (pinned.id.as_str(), pinned.kind),
        ("p-pty-test-1", Kind::Pinned)
    );
}

#[test]
fn the_id_is_golden() {
    // FNV-1a-64 reference vectors: the empty input and "a".
    assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
    // The id is the hash of the normalised root: lower case on Windows.
    let mut win = FakeFs::new(true);
    win.checkout(r"D:\Src\GitPoulpi");
    let i = id_of(&win, r"D:\Src\GitPoulpi\target\release\gitpoulpi.exe");
    assert_eq!(i.id, "9d5b6e78a990a026");
    assert_eq!(i.short(), "i9d5b6e78");
    let unix = FakeFs::new(false);
    let u = id_of(&unix, "/home/seb/gitpoulpi/gitpoulpi");
    assert_eq!(u.id, "b7b0b24e011deb20");
    assert_eq!(
        short_id("p-test"),
        format!("i{}", &format!("{:016x}", fnv1a64(b"p-test"))[..8])
    );
}

// ------------------------------------------------------- settings entries

fn pinned(name: &str) -> Identity {
    Identity {
        id: format!("p-{name}"),
        kind: Kind::Portable,
        root: format!("/opt/{name}"),
        channel: None,
    }
}

struct NoAliases;
impl AliasEvidence for NoAliases {
    fn target_of(&self, _: &str) -> Option<String> {
        None
    }
}

struct Aliases(HashMap<&'static str, &'static str>);
impl AliasEvidence for Aliases {
    fn target_of(&self, alias: &str) -> Option<String> {
        self.0.get(alias).map(|t| t.to_string())
    }
}

fn live(_: &InstallationEntry) -> bool {
    true
}

fn with(config: &mut Config, id: &Identity, entry: InstallationEntry) {
    config.installation.settings.insert(id.id.clone(), entry);
}

#[test]
fn nobody_is_primary_by_default() {
    let config = Config::default();
    assert_eq!(config.installation.primary, None);
    let eff = effective(&config, &pinned("a"));
    assert!(!eff.primary);
    // Even a lone installation gets the suffixed name.
    assert_eq!(eff.name, "GitPoulpi (portable)");
    // Without an entry the kind decides the colour: dev wears sun.
    let dev = Identity {
        kind: Kind::Dev,
        ..pinned("d")
    };
    assert_eq!(effective(&config, &dev).logo_color, "sun");
    assert_eq!(effective(&config, &pinned("a")).logo_color, "");
    // A default config does not even mention the table.
    let text = toml::to_string(&config).unwrap();
    assert!(!text.contains("installation"), "{text}");
    assert!(Config::from_toml_str("").unwrap().0.installation.is_empty());
}

#[test]
fn becoming_primary_copies_the_old_primarys_values_into_its_entry() {
    let (a, b) = (pinned("a"), pinned("b"));
    let mut config = Config::default();
    config.ui.logo_color = "violet".into();
    config.ui.command_alias = "gp".into();
    config.ui.command_alias_add = "gpa".into();
    config.installation.primary = Some(a.id.clone());
    with(&mut config, &a, new_entry(&a, String::new()));
    with(
        &mut config,
        &b,
        InstallationEntry {
            logo_color: "sun".into(),
            command_alias: "gpd".into(),
            name: "GitPoulpi (dev)".into(),
            ..new_entry(&b, String::new())
        },
    );
    let plan = plan_primary(&config, &b).unwrap();
    let old = &plan.installation.settings[&a.id];
    assert_eq!(
        (
            old.logo_color.as_str(),
            old.command_alias.as_str(),
            old.command_alias_add.as_str()
        ),
        ("violet", "gp", "gpa")
    );
    assert_eq!(
        old.name, "GitPoulpi (portable)",
        "the plain name leaves with the role"
    );
    assert_eq!(plan.installation.primary.as_deref(), Some(b.id.as_str()));
    assert_eq!(plan.demoted.as_deref(), Some(a.id.as_str()));
    let ui = plan.ui.unwrap();
    assert_eq!(
        (
            ui.logo_color.as_str(),
            ui.command_alias.as_str(),
            ui.command_alias_add.as_str()
        ),
        ("sun", "gpd", "")
    );
    // B's own entry no longer outranks ui.*, so the primary keeps writing ui.*.
    let new = &plan.installation.settings[&b.id];
    assert!(new.logo_color.is_empty() && new.command_alias.is_empty());
    assert_eq!(new.name, "GitPoulpi (dev)");
    // Already primary: nothing to do. Resigning moves the values back.
    let mut after = config.clone();
    after.installation = plan.installation;
    after.ui.logo_color = ui.logo_color.clone();
    after.ui.command_alias = ui.command_alias.clone();
    assert!(plan_primary(&after, &b).is_none());
    let back = plan_resign(&after, &b.id).unwrap();
    assert_eq!(back.installation.primary, None);
    assert_eq!(back.installation.settings[&b.id].command_alias, "gpd");
    assert_eq!(back.installation.settings[&b.id].logo_color, "sun");
}

#[test]
fn seeding_gives_dev_sun_and_the_first_other_todays_colour() {
    let mut config = Config::default();
    config.ui.logo_color = "violet".into();
    let fs = FakeFs::new(false);
    let e = env(&fs);
    let dev = Identity {
        kind: Kind::Dev,
        ..pinned("dev")
    };
    let entry = seed(&config, &dev, &e, &NoAliases, &live).unwrap();
    assert_eq!(entry.logo_color, "sun");
    assert_eq!(entry.name, "GitPoulpi (dev)");
    with(&mut config, &dev, entry);
    let other = seed(&config, &pinned("x"), &e, &NoAliases, &live).unwrap();
    assert_eq!(other.logo_color, "violet");
    // Seeded once: a second look finds nothing to write.
    with(&mut config, &pinned("x"), other);
    assert!(seed(&config, &pinned("x"), &e, &NoAliases, &live).is_none());
    // A moved folder refreshes only the path.
    let moved = Identity {
        root: "/elsewhere".into(),
        ..pinned("x")
    };
    let refreshed = seed(&config, &moved, &e, &NoAliases, &live).unwrap();
    assert_eq!(refreshed.path, "/elsewhere");
    assert_eq!(refreshed.logo_color, "violet");
}

#[test]
fn a_colour_worn_by_a_live_installation_is_not_seeded_twice() {
    let mut config = Config::default();
    config.ui.logo_color = "violet".into();
    let fs = FakeFs::new(false);
    let e = env(&fs);
    let first = seed(&config, &pinned("a"), &e, &NoAliases, &live).unwrap();
    with(&mut config, &pinned("a"), first);
    let second = seed(&config, &pinned("b"), &e, &NoAliases, &live).unwrap();
    assert_eq!(second.logo_color, "teal");
    with(&mut config, &pinned("b"), second);
    let third = seed(&config, &pinned("c"), &e, &NoAliases, &live).unwrap();
    assert_eq!(third.logo_color, "orange");
    // Gone installations wear nothing and own no names.
    let gone = |_: &InstallationEntry| false;
    let again = seed(&config, &pinned("d"), &e, &NoAliases, &gone).unwrap();
    assert_eq!(again.logo_color, "violet");
    assert_eq!(again.name, "GitPoulpi (portable)");
    // With the live ones present the name is numbered.
    let numbered = seed(&config, &pinned("d"), &e, &NoAliases, &live).unwrap();
    assert_eq!(numbered.name, "GitPoulpi (portable) 3");
}

#[test]
fn legacy_alias_names_follow_the_installation_their_file_points_at() {
    let mut fs = FakeFs::new(false);
    fs.checkout("/src/gitpoulpi");
    let dev_exe = "/src/gitpoulpi/target/release/gitpoulpi";
    let other_exe = "/opt/gp/gitpoulpi";
    let mut config = Config::default();
    config.ui.command_alias = "gpd".into();
    config.ui.command_alias_add = "gpda".into();
    let e = env(&fs);
    let aliases = Aliases(HashMap::from([
        ("gpd", "/src/gitpoulpi/target/dist/gitpoulpi"),
        ("gpda", dev_exe),
    ]));
    let dev = identify(Path::new(dev_exe), &e);
    let mine = seed(&config, &dev, &e, &aliases, &live).unwrap();
    assert_eq!(
        (mine.command_alias.as_str(), mine.command_alias_add.as_str()),
        ("gpd", "gpda")
    );
    let other = identify(Path::new(other_exe), &e);
    let theirs = seed(&config, &other, &e, &aliases, &live).unwrap();
    assert!(theirs.command_alias.is_empty() && theirs.command_alias_add.is_empty());
    // ui.* stays: old builds still read it.
    assert_eq!(config.ui.command_alias, "gpd");
    // No alias file at all: nothing is claimed.
    let none = seed(&config, &dev, &e, &NoAliases, &live).unwrap();
    assert!(none.command_alias.is_empty());
}

#[test]
fn names_always_carry_a_suffix_unless_primary() {
    for (kind, channel, label) in [
        (Kind::Dev, None, "dev"),
        (Kind::Cargo, None, "cargo"),
        (Kind::Portable, None, "portable"),
        (Kind::Install, None, "portable"),
        (Kind::Install, Some(InstallChannel::InnoUser), "per-user"),
        (
            Kind::Install,
            Some(InstallChannel::InnoMachine),
            "all users",
        ),
        (Kind::Install, Some(InstallChannel::Choco), "Chocolatey"),
        (
            Kind::Install,
            Some(InstallChannel::ChocoInternal),
            "Chocolatey",
        ),
        (Kind::Install, Some(InstallChannel::Winget), "winget"),
        (Kind::Install, Some(InstallChannel::Scoop), "Scoop"),
        (Kind::Install, Some(InstallChannel::Msi), "MSI"),
        (Kind::Install, Some(InstallChannel::Homebrew), "Homebrew"),
        (Kind::Install, Some(InstallChannel::Apt), "apt"),
        (Kind::Install, Some(InstallChannel::Aur), "AUR"),
    ] {
        assert_eq!(default_name(kind, channel), format!("GitPoulpi ({label})"));
    }
    let id = pinned("a");
    let mut config = Config::default();
    assert_eq!(effective(&config, &id).name, "GitPoulpi (portable)");
    config.installation.primary = Some(id.id.clone());
    assert_eq!(effective(&config, &id).name, "GitPoulpi");
    // A frozen name of the entry is what a non-primary uses.
    config.installation.primary = None;
    with(
        &mut config,
        &id,
        InstallationEntry {
            name: "Mine".into(),
            ..InstallationEntry::default()
        },
    );
    assert_eq!(effective(&config, &id).name, "Mine");
}

#[test]
fn a_taken_name_gets_a_number() {
    let taken = |names: &[&str]| names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
    assert_eq!(unique_name("GitPoulpi (dev)", &[]), "GitPoulpi (dev)");
    assert_eq!(
        unique_name("GitPoulpi (dev)", &taken(&["gitpoulpi (DEV) "])),
        "GitPoulpi (dev) 2"
    );
    assert_eq!(
        unique_name(
            "GitPoulpi (dev)",
            &taken(&["GitPoulpi (dev)", "GitPoulpi (dev) 2"])
        ),
        "GitPoulpi (dev) 3"
    );
    // Name validation.
    assert_eq!(validate_name("GitPoulpi (dev) 2"), Ok(()));
    assert_eq!(validate_name(""), Err(NameError::Empty));
    assert_eq!(validate_name(&"x".repeat(65)), Err(NameError::TooLong));
    assert_eq!(validate_name(&"x".repeat(64)), Ok(()));
    assert_eq!(validate_name("a/b"), Err(NameError::BadChar('/')));
    assert_eq!(validate_name("a\tb"), Err(NameError::BadChar('\t')));
    assert_eq!(validate_name("a."), Err(NameError::TrailingDotOrSpace));
    assert_eq!(validate_name("a "), Err(NameError::TrailingDotOrSpace));
    assert!(matches!(validate_name("con"), Err(NameError::Reserved(_))));
    assert!(matches!(
        validate_name("LPT3.txt"),
        Err(NameError::Reserved(_))
    ));
    assert_eq!(validate_name("COM10"), Ok(()));
    assert_eq!(validate_name("console"), Ok(()));
}

#[test]
fn the_version_is_inserted_only_when_asked() {
    assert_eq!(
        launcher_name("GitPoulpi (Scoop)", false, "1.2.3"),
        "GitPoulpi (Scoop)"
    );
    assert_eq!(
        launcher_name("GitPoulpi (Scoop)", true, "1.2.3"),
        "GitPoulpi 1.2 (Scoop)"
    );
    assert_eq!(
        launcher_name("GitPoulpi", true, "1.2.3-rc1"),
        "GitPoulpi 1.2"
    );
    assert_eq!(launcher_name("My Octopus", true, "1.2.3"), "My Octopus 1.2");
    assert_eq!(launcher_name("GitPoulpiX", true, "1.2.3"), "GitPoulpiX 1.2");
    assert_eq!(major_minor("3"), "3.0");
}

#[test]
fn env_defaults_to_the_id_and_dist_release_share_it() {
    let mut fs = FakeFs::new(false);
    fs.checkout("/src/gitpoulpi");
    let e = env(&fs);
    let dist = identify(Path::new("/src/gitpoulpi/target/dist/gitpoulpi"), &e);
    let release = identify(Path::new("/src/gitpoulpi/target/release/gitpoulpi"), &e);
    let config = Config::default();
    let (a, b) = (
        environment(&config, &dist, &e),
        environment(&config, &release, &e),
    );
    assert_eq!(a, dist.id);
    assert_eq!(a, b);
    let other = identify(Path::new("/opt/gp/gitpoulpi"), &e);
    assert_ne!(environment(&config, &other, &e), a);
    assert_eq!(effective(&config, &dist).env, dist.id);
}

// ------------------------------------------------------------- the file

#[test]
fn set_installation_in_file_keeps_every_other_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        "[ui]\nlogo_color = \"violet\"\ncommand_alias = \"gp\"\n\n[general]\nundo_depth = 7\n\n[future]\nknob = 3\n",
    )
    .unwrap();
    set_installation_in_file(&path, |inst| {
        inst.settings.insert(
            "9c1e0b7a55d2e3f4".into(),
            InstallationEntry {
                path: "/src/gp".into(),
                kind: Kind::Dev,
                logo_color: "sun".into(),
                ..InstallationEntry::default()
            },
        );
    })
    .unwrap();
    let doc: toml::Value = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(doc["ui"]["logo_color"].as_str(), Some("violet"));
    assert_eq!(doc["ui"]["command_alias"].as_str(), Some("gp"));
    assert_eq!(doc["general"]["undo_depth"].as_integer(), Some(7));
    assert_eq!(doc["future"]["knob"].as_integer(), Some(3));
    let entry = &doc["installation"]["settings"]["9c1e0b7a55d2e3f4"];
    assert_eq!(entry["kind"].as_str(), Some("dev"));
    assert_eq!(entry["logo_color"].as_str(), Some("sun"));
    assert!(
        entry.get("env").is_none(),
        "unset fields stay out of the file"
    );
    let loaded = Config::load(&path).unwrap();
    assert_eq!(
        loaded.installation.settings["9c1e0b7a55d2e3f4"].path,
        "/src/gp"
    );
    // Emptying the table removes it again, other keys still intact.
    set_installation_in_file(&path, |inst| *inst = InstallationConfig::default()).unwrap();
    let doc: toml::Value = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert!(doc.get("installation").is_none());
    assert_eq!(doc["future"]["knob"].as_integer(), Some(3));
}

#[test]
fn set_in_file_of_another_key_keeps_the_installation_table() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    set_installation_in_file(&path, |inst| {
        inst.primary = Some("abc".into());
        inst.settings.insert(
            "abc".into(),
            InstallationEntry {
                name: "GitPoulpi (cargo)".into(),
                env: "shared".into(),
                ..InstallationEntry::default()
            },
        );
    })
    .unwrap();
    crate::set_in_file(&path, "ui.logo_color", "teal").unwrap();
    crate::set_in_file(&path, "general.undo_depth", "9").unwrap();
    let config = Config::load(&path).unwrap();
    assert_eq!(config.ui.logo_color, "teal");
    assert_eq!(config.installation.primary.as_deref(), Some("abc"));
    let e = &config.installation.settings["abc"];
    assert_eq!(
        (e.name.as_str(), e.env.as_str()),
        ("GitPoulpi (cargo)", "shared")
    );
}

#[test]
fn become_primary_in_file_is_one_write_that_moves_the_values() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        "[ui]\nlogo_color = \"violet\"\ncommand_alias = \"gp\"\n",
    )
    .unwrap();
    let (a, b) = (pinned("a"), pinned("b"));
    become_primary_in_file(&path, &a).unwrap().unwrap();
    let first = Config::load(&path).unwrap();
    assert_eq!(first.installation.primary.as_deref(), Some("p-a"));
    assert_eq!(
        effective(&first, &a).command_alias,
        "gp",
        "the first primary inherits ui.*"
    );
    let plan = become_primary_in_file(&path, &b).unwrap().unwrap();
    assert_eq!(plan.demoted.as_deref(), Some("p-a"));
    let config = Config::load(&path).unwrap();
    assert_eq!(config.installation.primary.as_deref(), Some("p-b"));
    assert_eq!(effective(&config, &b).name, "GitPoulpi");
    assert!(become_primary_in_file(&path, &b).unwrap().is_none());
}

#[test]
fn a_seed_write_keeps_every_byte_outside_the_installation_table() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let before = "\n# my notes\n[ui]\nzeta = 1 # last\nlogo_color = \"violet\"\n\n\n[general]\nundo_depth = 7\n  # indented\n\n[installation.settings.old]\nname = \"Old\"\n\n[future]\nknob = 3\n";
    std::fs::write(&path, before).unwrap();
    set_installation_in_file(&path, |inst| {
        inst.settings.insert(
            "new".into(),
            InstallationEntry {
                name: "New".into(),
                ..InstallationEntry::default()
            },
        );
    })
    .unwrap();
    let after = std::fs::read_to_string(&path).unwrap();
    let (head, tail) = before.split_once("[installation.settings.old]").unwrap();
    let tail = tail.split_once("[future]").unwrap().1;
    assert!(after.starts_with(head), "{after}");
    assert!(after.ends_with(&format!("[future]{tail}")), "{after}");
    assert!(after.contains("[installation.settings.new]"), "{after}");
    assert_eq!(
        Config::load(&path).unwrap().installation.settings["old"].name,
        "Old"
    );

    // No existing table: the new one is appended, nothing else moves.
    let path = dir.path().join("second.toml");
    let plain = "\n# top\nb = 1\n\n[ui]\nz = 2\n";
    std::fs::write(&path, plain).unwrap();
    set_installation_in_file(&path, |inst| {
        inst.settings
            .insert("x".into(), InstallationEntry::default());
    })
    .unwrap();
    assert!(std::fs::read_to_string(&path).unwrap().starts_with(plain));
}

#[test]
fn a_handover_write_keeps_the_formatting_too() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let before = "# hi\n[ui]\nz = 1\ncommand_alias = \"gp\"\n\n[general]\nundo_depth = 7\n";
    std::fs::write(&path, before).unwrap();
    let id = pinned("a");
    let done = become_primary_in_file(&path, &id).unwrap();
    assert!(done.is_some());
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(after.starts_with("# hi\n[ui]\nz = 1\n"), "{after}");
    assert!(after.contains("[general]\nundo_depth = 7\n"), "{after}");
    assert_eq!(
        Config::load(&path).unwrap().installation.primary.as_deref(),
        Some("p-a")
    );
}

#[test]
fn a_pinned_id_is_not_seeded_into_the_file_but_a_discovered_one_is() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    let entry = InstallationEntry::default();
    let mut id = pinned("pty");
    id.kind = Kind::Pinned;
    assert!(persist_seed(&path, &id, &entry));
    assert!(!path.exists(), "a pinned id writes nothing");

    let found = pinned("found");
    assert!(persist_seed(&path, &found, &entry));
    let config = Config::load(&path).unwrap();
    assert!(config.installation.settings.contains_key("p-found"));
}

#[test]
fn seed_returns_nothing_for_an_unchanged_entry() {
    let id = pinned("same");
    let fs = FakeFs::default();
    let mut config = Config::default();
    let entry = seed(&config, &id, &env(&fs), &NoAliases, &live).unwrap();
    with(&mut config, &id, entry);
    assert!(seed(&config, &id, &env(&fs), &NoAliases, &live).is_none());
}
