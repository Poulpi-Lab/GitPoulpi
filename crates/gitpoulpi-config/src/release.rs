//! How a release archive is named, shared by `xtask dist` (which writes the
//! files) and the update check (which looks for the one matching the running
//! build). One function, so the two can never disagree about a name.

/// The binaries a release archive ships (without the platform suffix).
///
/// `gitpoulpi-launch` is the console-less program the Windows shortcuts run
/// (`gitpoulpi launch` without a console flash); it ships everywhere so an
/// install is the same set of files on every OS.
pub const BINARIES: [&str; 5] = [
    "gitpoulpi-tui",
    "gitpoulpi",
    "gitpoulpi-mcp",
    "gitpoulpi-launch",
    "gitpoulpi-updater",
];

/// Program names a release no longer ships (without the platform suffix).
/// Empty for now: the archive still carries every name of [`BINARIES`].
/// When a name moves here, the updater deletes its file and its `.old` copy
/// after a successful swap, `gitpoulpi uninstall` removes it, and a restart
/// that was recorded against it (see the updater's relaunch mapping) starts
/// `gitpoulpi` instead. Never add a name here in the same release that
/// removes it from the archive: an old updater would leave a stale copy.
pub const RETIRED: [&str; 0] = [];

/// Whether `stem` (a program name without suffix, any case) is in `retired`.
pub fn is_retired_in(retired: &[&str], stem: &str) -> bool {
    retired.iter().any(|r| r.eq_ignore_ascii_case(stem))
}

/// The file names of `retired` for `target` (with the platform suffix).
pub fn retired_names_in(retired: &[&str], target: &str) -> Vec<String> {
    retired.iter().map(|b| exe_name(b, target)).collect()
}

/// The file names of [`RETIRED`] for `target`.
pub fn retired_names(target: &str) -> Vec<String> {
    retired_names_in(&RETIRED, target)
}

/// The file `xtask dist` writes next to the binaries inside a release
/// archive. A running build may replace itself (the self-updater of
/// the updates-and-install design, slice B) only when this
/// file sits next to its exe: a `cargo` build never has one.
pub const MARKER_FILE: &str = ".gitpoulpi-dist";

/// The Docker Hub repository of the container image. One constant: the update
/// panel, the docs and the release workflow all read this name, so a namespace
/// change is a one-line edit (GHCR mirrors it under `ghcr.io/poulpi-lab/`).
pub const DOCKER_IMAGE: &str = "poulpilab/gitpoulpi";

/// How an install got onto the machine: the marker's `channel=` line. It
/// decides what the app may do about an update (the install-channels design).
/// `cargo` is deliberately not a value: a cargo build has no marker, so it is
/// recognized by cargo's own install record instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallChannel {
    Archive,
    InnoUser,
    InnoMachine,
    Choco,
    ChocoInternal,
    Winget,
    Scoop,
    Homebrew,
    Apt,
    Msi,
    Aur,
    Docker,
}

impl InstallChannel {
    pub const ALL: [InstallChannel; 12] = [
        Self::Archive,
        Self::InnoUser,
        Self::InnoMachine,
        Self::Choco,
        Self::ChocoInternal,
        Self::Winget,
        Self::Scoop,
        Self::Homebrew,
        Self::Apt,
        Self::Msi,
        Self::Aur,
        Self::Docker,
    ];

    /// The value written after `channel=`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Archive => "archive",
            Self::InnoUser => "inno-user",
            Self::InnoMachine => "inno-machine",
            Self::Choco => "choco",
            Self::ChocoInternal => "choco-internal",
            Self::Winget => "winget",
            Self::Scoop => "scoop",
            Self::Homebrew => "homebrew",
            Self::Apt => "apt",
            Self::Msi => "msi",
            Self::Aur => "aur",
            Self::Docker => "docker",
        }
    }

    /// Exact spelling only; anything else is `None`.
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_str() == text)
    }
}

/// What the marker file says: the release the files belong to, and how they
/// were installed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    pub version: String,
    pub target: String,
    /// `None` for a marker written before channels existed, and for a
    /// `channel=` value this build does not know (a newer marker read by an
    /// older build): both are "Legacy", never a guess.
    pub channel: Option<InstallChannel>,
    /// An internal Chocolatey feed URL written by an internalised package.
    pub feed: Option<String>,
}

impl Marker {
    /// A marker with neither a channel nor a feed (what releases before the
    /// install-channels design wrote).
    pub fn new(version: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            target: target.into(),
            channel: None,
            feed: None,
        }
    }

    /// `version=...` and `target=...`, then `channel=` / `feed=` only when
    /// set, one per line. Old builds ignore the extra lines.
    pub fn render(&self) -> String {
        let mut out = format!(
            "version={}
target={}
",
            self.version, self.target
        );
        if let Some(c) = self.channel {
            out.push_str(&format!(
                "channel={}
",
                c.as_str()
            ));
        }
        if let Some(f) = &self.feed {
            out.push_str(&format!(
                "feed={f}
"
            ));
        }
        out
    }

    /// Reads [`Marker::render`]'s text; `None` when `version` or `target` is
    /// missing.
    pub fn parse(text: &str) -> Option<Self> {
        let field = |key: &str| {
            text.lines()
                .find_map(|l| l.trim().strip_prefix(key)?.strip_prefix('='))
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        };
        Some(Self {
            version: field("version")?,
            target: field("target")?,
            channel: field("channel").and_then(|c| InstallChannel::parse(&c)),
            feed: field("feed"),
        })
    }
}

/// Whether `target` (a Rust target triple) is a Windows one: it ships a
/// `.zip` of `.exe` files, everything else a `.tar.gz`.
pub fn is_windows(target: &str) -> bool {
    target.contains("windows")
}

/// `binary` with the platform's executable suffix.
pub fn exe_name(binary: &str, target: &str) -> String {
    if is_windows(target) {
        format!("{binary}.exe")
    } else {
        binary.to_string()
    }
}

/// `gitpoulpi-<version>-<target>`: the archive's stem and its top-level
/// directory, so extracting never spills files into the current directory.
pub fn package_stem(version: &str, target: &str) -> String {
    format!("gitpoulpi-{version}-{target}")
}

/// `gitpoulpi-<version>-<target>.zip` on Windows targets, `.tar.gz` elsewhere.
pub fn archive_name(version: &str, target: &str) -> String {
    let ext = if is_windows(target) { "zip" } else { "tar.gz" };
    format!("{}.{ext}", package_stem(version, target))
}

/// `gitpoulpi-<version>-<target>-setup.exe`: the Windows installer
/// (`packaging/windows/gitpoulpi.iss`) the release workflow builds from the
/// zip, uploaded next to it. Kept here so the script, the workflow and the
/// tests that tie them together agree on one name.
pub fn installer_name(version: &str, target: &str) -> String {
    format!("{}-setup.exe", package_stem(version, target))
}

/// The target triple of the running build, as far as the release workflow
/// names them (`x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`,
/// `aarch64-apple-darwin`, ...). Other combinations still get a plausible
/// triple: no release asset will match it, and the update notice then points
/// at the release page instead of an archive.
pub fn host_target() -> String {
    let arch = std::env::consts::ARCH;
    match std::env::consts::OS {
        "windows" => format!("{arch}-pc-windows-msvc"),
        "macos" => format!("{arch}-apple-darwin"),
        "linux" => format!("{arch}-unknown-linux-gnu"),
        other => format!("{arch}-unknown-{other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_is_zip_on_windows_and_tar_gz_elsewhere() {
        assert_eq!(
            archive_name("0.1.0", "x86_64-pc-windows-msvc"),
            "gitpoulpi-0.1.0-x86_64-pc-windows-msvc.zip"
        );
        assert_eq!(
            archive_name("0.1.0", "x86_64-unknown-linux-gnu"),
            "gitpoulpi-0.1.0-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            archive_name("0.2.0", "aarch64-apple-darwin"),
            "gitpoulpi-0.2.0-aarch64-apple-darwin.tar.gz"
        );
    }

    #[test]
    fn the_marker_round_trips_and_needs_both_fields() {
        let m = Marker::new("0.2.0", "x86_64-unknown-linux-gnu");
        assert_eq!(Marker::parse(&m.render()), Some(m));
        assert_eq!(Marker::parse("version=0.2.0\n"), None);
        assert_eq!(Marker::parse(""), None);
    }

    #[test]
    fn windows_binaries_get_exe() {
        assert_eq!(
            exe_name("gitpoulpi", "x86_64-pc-windows-msvc"),
            "gitpoulpi.exe"
        );
        assert_eq!(
            exe_name("gitpoulpi", "x86_64-unknown-linux-gnu"),
            "gitpoulpi"
        );
    }

    #[test]
    fn the_host_target_names_this_build() {
        let t = host_target();
        assert!(t.starts_with(std::env::consts::ARCH), "{t}");
        assert_eq!(is_windows(&t), cfg!(windows));
    }

    #[test]
    fn the_marker_round_trips_with_channel_and_feed() {
        let m = Marker {
            channel: Some(InstallChannel::ChocoInternal),
            feed: Some("https://nexus.example/repository/choco/".into()),
            ..Marker::new("0.2.0", "x86_64-pc-windows-msvc")
        };
        let text = m.render();
        assert!(text.contains("channel=choco-internal\n"), "{text}");
        assert_eq!(Marker::parse(&text), Some(m));
    }

    #[test]
    fn channel_and_feed_lines_are_written_only_when_set() {
        let text = Marker::new("0.2.0", "t").render();
        assert_eq!(text, "version=0.2.0\ntarget=t\n");
    }

    #[test]
    fn an_old_marker_parses_as_legacy_and_old_builds_read_a_new_one() {
        let old = Marker::parse("version=0.1.0\ntarget=t\n").unwrap();
        assert_eq!((old.channel, old.feed), (None, None));
        // The extra lines of a new marker must not break the old parse rule.
        let new = "version=0.3.0\ntarget=t\nchannel=apt\nfuture=1\n";
        assert_eq!(
            Marker::parse(new).unwrap().channel,
            Some(InstallChannel::Apt)
        );
    }

    #[test]
    fn every_channel_spelling_round_trips_and_is_unique() {
        let mut seen = std::collections::HashSet::new();
        for c in InstallChannel::ALL {
            assert_eq!(InstallChannel::parse(c.as_str()), Some(c));
            assert!(seen.insert(c.as_str()), "{c:?} repeats a spelling");
            let m = Marker {
                channel: Some(c),
                ..Marker::new("1.0.0", "t")
            };
            assert_eq!(Marker::parse(&m.render()).unwrap().channel, Some(c));
        }
    }

    #[test]
    fn an_unknown_channel_is_legacy_not_a_parse_failure() {
        assert_eq!(InstallChannel::parse("snap"), None);
        assert_eq!(InstallChannel::parse("Archive"), None);
        assert_eq!(InstallChannel::parse("cargo"), None, "never a marker value");
        let m = Marker::parse("version=1.0.0\ntarget=t\nchannel=snap\n").unwrap();
        assert_eq!(m.channel, None);
    }

    #[test]
    fn the_docker_image_is_the_hub_name() {
        assert_eq!(DOCKER_IMAGE, "poulpilab/gitpoulpi");
    }
}
