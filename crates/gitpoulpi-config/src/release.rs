//! How a release archive is named, shared by `xtask dist` (which writes the
//! files) and the update check (which looks for the one matching the running
//! build). One function, so the two can never disagree about a name.

/// The binaries a release archive ships (without the platform suffix).
///
/// `gitpoulpi-launch` is the console-less program the Windows shortcuts run
/// (`gitpoulpi launch` without a console flash); it ships everywhere so an
/// install is the same set of files on every OS.
pub const BINARIES: [&str; 4] = [
    "gitpoulpi-tui",
    "gitpoulpi",
    "gitpoulpi-mcp",
    "gitpoulpi-launch",
];

/// The file `xtask dist` writes next to the binaries inside a release
/// archive. A running build may replace itself (the self-updater of
/// the updates-and-install design, slice B) only when this
/// file sits next to its exe: a `cargo` build never has one.
pub const MARKER_FILE: &str = ".gitpoulpi-dist";

/// What the marker file says: the release the files belong to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    pub version: String,
    pub target: String,
}

impl Marker {
    /// `version=...` and `target=...`, one per line.
    pub fn render(&self) -> String {
        format!("version={}\ntarget={}\n", self.version, self.target)
    }

    /// Reads [`Marker::render`]'s text; `None` when a field is missing.
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
        let m = Marker {
            version: "0.2.0".into(),
            target: "x86_64-unknown-linux-gnu".into(),
        };
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
}
