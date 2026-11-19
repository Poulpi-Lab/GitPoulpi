//! `cargo run -p xtask -- dist`: build the binaries with the `dist`
//! profile (thin LTO) and pack them into `target/dist/`.
//!
//! The archive is `gitpoulpi-<version>-<target>.zip` on Windows targets and
//! `.tar.gz` everywhere else, next to a `.sha256` file in the format
//! `sha256sum -c` reads. Everything here is plain Rust (no tar, zip or
//! sha256sum on PATH), so the same task runs on Windows and Linux. The
//! naming, the archive layout and the checksum line are pure functions with
//! tests; only `run` touches cargo and the disk layout.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

/// The file names the archive carries (the packages come from
/// `release::build_packages`: the shim stands in for tui and mcp).
const BINARIES: [&str; 5] = gitpoulpi_config::release::BINARIES;

const USAGE: &str = "\
Usage: cargo run -p xtask -- dist [--target <triple>] [--no-build]

Builds gitpoulpi-cli, gitpoulpi-updater and gitpoulpi-shim with `--profile dist` (gitpoulpi-tui
and gitpoulpi-mcp are copies of the shim) and
writes target/dist/gitpoulpi-<version>-<target>.<zip|tar.gz> plus a .sha256
file. Windows targets get a .zip, everything else a .tar.gz.

  --target <triple>  cross-build for this target (default: the host).
  --no-build         pack what is already in the dist profile directory.";

/// One file inside the archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Path relative to the archive's top-level directory.
    pub name: String,
    pub data: Vec<u8>,
    pub executable: bool,
}

pub use gitpoulpi_config::release::{
    archive_name, exe_name, installer_name, is_windows, package_stem,
};

/// The `.gitpoulpi-dist` file: it tells a running build it came from a
/// release archive and may update itself (version and target are checked
/// against the archive it downloads). The archive's marker says
/// `channel=archive`; a package (Inno, Chocolatey, apt, ...) replaces it with
/// its own through `xtask package-marker`, one code path for all of them.
pub fn marker_entry(version: &str, target: &str) -> Entry {
    Entry {
        name: gitpoulpi_config::release::MARKER_FILE.into(),
        data: gitpoulpi_config::release::Marker {
            channel: Some(gitpoulpi_config::release::InstallChannel::Archive),
            ..gitpoulpi_config::release::Marker::new(version, target)
        }
        .render()
        .into_bytes(),
        executable: false,
    }
}

const PACKAGE_MARKER_USAGE: &str = "\
Usage: cargo run -p xtask -- package-marker --channel <c> --version <v> --target <triple>
                                           [--feed <url>] --out <file>

Writes the .gitpoulpi-dist marker a package installs next to the binaries.
<c> is one of the install channels (archive, inno-user, inno-machine, choco,
choco-internal, winget, scoop, homebrew, apt, msi, aur, docker).";

/// The marker text for a package: `channel` must be a real install channel
/// (`cargo` is never a marker value).
pub fn package_marker_text(
    channel: &str,
    version: &str,
    target: &str,
    feed: Option<&str>,
) -> Result<String, String> {
    use gitpoulpi_config::release::{InstallChannel, Marker};
    let channel = InstallChannel::parse(channel).ok_or_else(|| {
        let all: Vec<&str> = InstallChannel::ALL.iter().map(|c| c.as_str()).collect();
        format!("unknown channel {channel:?} (one of: {})", all.join(", "))
    })?;
    if version.is_empty() || target.is_empty() {
        return Err("--version and --target must not be empty".into());
    }
    Ok(Marker {
        channel: Some(channel),
        feed: feed.filter(|f| !f.is_empty()).map(str::to_string),
        ..Marker::new(version, target)
    }
    .render())
}

/// `xtask package-marker`.
pub fn package_marker(args: &[String]) -> Result<i32, String> {
    let (mut channel, mut version, mut target, mut feed, mut out) = (None, None, None, None, None);
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or(format!("{name} needs a value"));
        match arg.as_str() {
            "--channel" => channel = Some(value("--channel")?),
            "--version" => version = Some(value("--version")?),
            "--target" => target = Some(value("--target")?),
            "--feed" => feed = Some(value("--feed")?),
            "--out" => out = Some(value("--out")?),
            "-h" | "--help" => {
                println!("{PACKAGE_MARKER_USAGE}");
                return Ok(0);
            }
            other => {
                return Err(format!(
                    "unknown option {other:?}\n\n{PACKAGE_MARKER_USAGE}"
                ))
            }
        }
    }
    let need = |v: Option<String>, name: &str| {
        v.ok_or(format!("{name} is required\n\n{PACKAGE_MARKER_USAGE}"))
    };
    let text = package_marker_text(
        &need(channel, "--channel")?,
        &need(version, "--version")?,
        &need(target, "--target")?,
        feed.as_deref(),
    )?;
    let out = need(out, "--out")?;
    fs::write(&out, text).map_err(|e| format!("cannot write {out}: {e}"))?;
    Ok(0)
}

const PACKAGE_MANIFESTS_USAGE: &str = "\
Usage: cargo run -p xtask -- package-manifests [--dir <dir>] [--version <v>]
                                               [--notes <file>] [--out <file>]

Writes latest.json from the .sha256 files in <dir> (default target/dist):
{\"version\", \"notes\", \"assets\": {file name: sha256}}. The release attaches
it so a mirror only has to copy files.";

/// One `.sha256` file's line (`<hex>  <name>`, as `sha256sum` writes it).
pub fn parse_checksum_line(line: &str) -> Option<(String, String)> {
    let (hex, name) = line.trim().split_once(char::is_whitespace)?;
    let name = name.trim().trim_start_matches('*');
    let valid = hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit());
    (valid && !name.is_empty()).then(|| (hex.to_ascii_lowercase(), name.to_string()))
}

/// The `latest.json` text: assets sorted by name so the file is stable.
pub fn latest_json(version: &str, notes: &str, checksums: &[(String, String)]) -> String {
    let assets: std::collections::BTreeMap<&str, &str> = checksums
        .iter()
        .map(|(hex, name)| (name.as_str(), hex.as_str()))
        .collect();
    let value = serde_json::json!({ "version": version, "notes": notes, "assets": assets });
    let mut text = serde_json::to_string_pretty(&value).expect("json serializes");
    text.push('\n');
    text
}

/// `xtask package-manifests`.
pub fn package_manifests(args: &[String]) -> Result<i32, String> {
    let (mut dir, mut version, mut notes, mut out) = (None, None, None, None);
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or(format!("{name} needs a value"));
        match arg.as_str() {
            "--dir" => dir = Some(value("--dir")?),
            "--version" => version = Some(value("--version")?),
            "--notes" => notes = Some(value("--notes")?),
            "--out" => out = Some(value("--out")?),
            "-h" | "--help" => {
                println!("{PACKAGE_MANIFESTS_USAGE}");
                return Ok(0);
            }
            other => {
                return Err(format!(
                    "unknown option {other:?}\n\n{PACKAGE_MANIFESTS_USAGE}"
                ))
            }
        }
    }
    let dir = PathBuf::from(dir.unwrap_or_else(|| "target/dist".into()));
    let mut checksums = Vec::new();
    let entries = fs::read_dir(&dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("sha256") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let (hex, name) = text
            .lines()
            .find_map(parse_checksum_line)
            .ok_or_else(|| format!("{} holds no checksum line", path.display()))?;
        checksums.push((hex, name));
    }
    if checksums.is_empty() {
        return Err(format!("no .sha256 files in {}", dir.display()));
    }
    let notes = match notes {
        Some(f) => fs::read_to_string(&f).map_err(|e| format!("cannot read {f}: {e}"))?,
        None => String::new(),
    };
    let version = version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    let out = out
        .map(PathBuf::from)
        .unwrap_or_else(|| dir.join("latest.json"));
    fs::write(&out, latest_json(&version, &notes, &checksums))
        .map_err(|e| format!("cannot write {}: {e}", out.display()))?;
    println!("{}", out.display());
    Ok(0)
}

pub fn install_text(version: &str, target: &str) -> String {
    let exe = |b| exe_name(b, target);
    let (place, path_hint) = if is_windows(target) {
        (
            "Copy the .exe files to a folder of your choice, for example\n\
             %LOCALAPPDATA%\\Programs\\GitPoulpi, and add that folder to your PATH\n\
             (Settings > System > About > Advanced system settings > Environment Variables).",
            "gitpoulpi.exe --version",
        )
    } else {
        (
            "Copy the binaries to a folder on your PATH, for example\n\
             ~/.local/bin (`install -m 755 gitpoulpi gitpoulpi-tui gitpoulpi-mcp gitpoulpi-launch gitpoulpi-updater ~/.local/bin`)\n\
             or /usr/local/bin.",
            "gitpoulpi --version",
        )
    };
    format!(
        "GitPoulpi {version} ({target})\n\
         \n\
         Contents\n\
         \x20 {cli}      the CLI; with no subcommand it opens the terminal UI\n\
         \x20 {tui}  starts the terminal UI (a small stub that runs {cli})\n\
         \x20 {mcp}  starts the MCP server for AI agents (a small stub that runs {cli})\n\
         \x20 {launch}  what the shortcuts run: shows the open window, else starts one\n\
         \x20 {updater}  replaces GitPoulpi during an update; GitPoulpi runs it, not you\n\
         \n\
         Install\n\
         {place}\n\
         \n\
         Check\n\
         \x20 {path_hint}\n\
         \n\
         Requirements\n\
         \x20 A system `git` on PATH (2.30 or newer is recommended). `git-lfs` 3.0+ is\n\
         \x20 optional and only needed for repositories that use Git LFS.\n\
         \x20 A terminal with true color; the commit graph is drawn as an image when\n\
         \x20 the terminal supports the Kitty or iTerm2 graphics protocols (WezTerm,\n\
         \x20 Kitty, Ghostty) and falls back to characters otherwise.\n\
         \n\
         Updating: replace the binaries. A running background daemon from an older\n\
         version is reported by the TUI; `gitpoulpi daemon restart` swaps it.\n",
        cli = exe("gitpoulpi"),
        tui = exe("gitpoulpi-tui"),
        mcp = exe("gitpoulpi-mcp"),
        launch = exe("gitpoulpi-launch"),
        updater = exe("gitpoulpi-updater"),
    )
}

/// Lower-case hex SHA-256 of `data`.
pub fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// One line in the format `sha256sum -c` and `shasum -a 256 -c` accept.
pub fn checksum_line(hex: &str, file_name: &str) -> String {
    format!("{hex}  {file_name}\n")
}

pub fn write_tar_gz(stem: &str, entries: &[Entry]) -> std::io::Result<Vec<u8>> {
    let encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    let mut builder = tar::Builder::new(encoder);
    for entry in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(entry.data.len() as u64);
        header.set_mode(if entry.executable { 0o755 } else { 0o644 });
        header.set_mtime(0);
        header.set_cksum();
        builder.append_data(
            &mut header,
            format!("{stem}/{}", entry.name),
            &entry.data[..],
        )?;
    }
    builder.into_inner()?.finish()
}

pub fn write_zip(stem: &str, entries: &[Entry]) -> std::io::Result<Vec<u8>> {
    let to_io = |e: zip::result::ZipError| std::io::Error::other(e.to_string());
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for entry in entries {
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(if entry.executable { 0o755 } else { 0o644 });
        writer
            .start_file(format!("{stem}/{}", entry.name), options)
            .map_err(to_io)?;
        writer.write_all(&entry.data)?;
    }
    Ok(writer.finish().map_err(to_io)?.into_inner())
}

pub fn run(args: &[String]) -> Result<i32, String> {
    let mut target: Option<String> = None;
    let mut build = true;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--target" => {
                target = Some(it.next().ok_or("--target needs a triple")?.clone());
            }
            "--no-build" => build = false,
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(0);
            }
            other => return Err(format!("unknown option {other:?}\n\n{USAGE}")),
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("cannot locate the workspace root")?
        .to_path_buf();
    let version = env!("CARGO_PKG_VERSION");
    let host = host_triple()?;
    let triple = target.clone().unwrap_or_else(|| host.clone());
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target"));

    if build {
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let mut cmd = Command::new(cargo);
        // A release archive shows no commit hash next to the version.
        cmd.env("GITPOULPI_DIST", "1");
        cmd.current_dir(&root).args(["build", "--profile", "dist"]);
        for p in gitpoulpi_config::release::build_packages() {
            cmd.args(["-p", p]);
        }
        if let Some(t) = &target {
            cmd.args(["--target", t]);
        }
        eprintln!("xtask dist: building with --profile dist (thin LTO, this takes a while)");
        let status = cmd.status().map_err(|e| format!("cannot run cargo: {e}"))?;
        if !status.success() {
            return Err("cargo build --profile dist failed".into());
        }
    }

    let bin_dir = match &target {
        Some(t) => target_dir.join(t).join("dist"),
        None => target_dir.join("dist"),
    };

    let mut entries = Vec::new();
    let mut sizes = Vec::new();
    for b in BINARIES {
        let file = exe_name(b, &triple);
        let built = exe_name(
            gitpoulpi_config::release::built_binary(b)
                .ok_or_else(|| format!("{b} has no built-from entry"))?,
            &triple,
        );
        let data = fs::read(bin_dir.join(&built))
            .map_err(|e| format!("cannot read {}: {e}", bin_dir.join(&built).display()))?;
        sizes.push((file.clone(), data.len()));
        entries.push(Entry {
            name: file,
            data,
            executable: true,
        });
    }
    entries.push(marker_entry(version, &triple));
    entries.push(Entry {
        name: "INSTALL.txt".into(),
        data: install_text(version, &triple).into_bytes(),
        executable: false,
    });
    for doc in [
        "README.md",
        "LICENSE",
        crate::licenses::OUTPUT,
        "CHANGELOG.md",
    ] {
        match fs::read(root.join(doc)) {
            Ok(data) => entries.push(Entry {
                name: doc.into(),
                data: if doc == "README.md" {
                    absolute_image_links(&String::from_utf8_lossy(&data)).into_bytes()
                } else {
                    data
                },
                executable: false,
            }),
            // The MIT notice and the third-party notices are license
            // obligations: no archive without them.
            Err(e) if doc == crate::licenses::OUTPUT => {
                return Err(format!(
                    "{doc} is missing ({e}): run `cargo run -p xtask -- licenses`"
                ))
            }
            Err(e) if doc == "LICENSE" => return Err(format!("{doc} is missing ({e})")),
            Err(_) => eprintln!("xtask dist: warning: {doc} is missing, not included"),
        }
    }

    let stem = package_stem(version, &triple);
    let bytes = if is_windows(&triple) {
        write_zip(&stem, &entries)
    } else {
        write_tar_gz(&stem, &entries)
    }
    .map_err(|e| format!("cannot write the archive: {e}"))?;

    let out_dir = target_dir.join("dist");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let name = archive_name(version, &triple);
    let archive_path = out_dir.join(&name);
    fs::write(&archive_path, &bytes).map_err(|e| e.to_string())?;
    let checksum_path = out_dir.join(format!("{name}.sha256"));
    fs::write(&checksum_path, checksum_line(&sha256_hex(&bytes), &name))
        .map_err(|e| e.to_string())?;

    for (file, size) in sizes {
        println!("  {file}: {:.1} MiB", size as f64 / 1048576.0);
    }
    println!(
        "{} ({:.1} MiB)\n{}",
        archive_path.display(),
        bytes.len() as f64 / 1048576.0,
        checksum_path.display()
    );
    Ok(0)
}

/// The README's screenshots are repo-relative (`docs/user/images/x.png`),
/// which break once the README sits in a release archive. Point them at
/// GitHub's raw view instead of bundling the images.
fn absolute_image_links(readme: &str) -> String {
    const REL: &str = "](docs/user/images/";
    const ABS: &str = "](https://github.com/Poulpi-Lab/GitPoulpi/blob/main/docs/user/images/";
    let mut out = String::with_capacity(readme.len() + 512);
    let mut rest = readme;
    while let Some(i) = rest.find(REL) {
        out.push_str(&rest[..i]);
        out.push_str(ABS);
        rest = &rest[i + REL.len()..];
        match rest.find(')') {
            Some(j) => {
                out.push_str(&rest[..j]);
                out.push_str("?raw=true");
                rest = &rest[j..];
            }
            None => break,
        }
    }
    out.push_str(rest);
    out
}

fn host_triple() -> Result<String, String> {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let out = Command::new(rustc)
        .arg("-vV")
        .output()
        .map_err(|e| format!("cannot run rustc: {e}"))?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|l| l.strip_prefix("host: "))
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "rustc -vV printed no host line".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn readme_images_become_absolute_urls() {
        let out = absolute_image_links("a ![x](docs/user/images/a.png) b [d](docs/user/x.md)");
        assert_eq!(
            out,
            "a ![x](https://github.com/Poulpi-Lab/GitPoulpi/blob/main/docs/user/images/a.png?raw=true) b [d](docs/user/x.md)"
        );
    }

    use super::*;
    use std::io::Read;

    fn sample() -> Vec<Entry> {
        vec![
            Entry {
                name: "gitpoulpi".into(),
                data: b"binary".to_vec(),
                executable: true,
            },
            Entry {
                name: "INSTALL.txt".into(),
                data: b"hello".to_vec(),
                executable: false,
            },
        ]
    }

    #[test]
    fn the_archive_marker_says_channel_archive() {
        let e = marker_entry("0.2.0", "x86_64-unknown-linux-gnu");
        let m = gitpoulpi_config::release::Marker::parse(&String::from_utf8(e.data).unwrap());
        assert_eq!(
            m.unwrap().channel,
            Some(gitpoulpi_config::release::InstallChannel::Archive)
        );
    }

    #[test]
    fn package_marker_writes_each_channel_and_refuses_others() {
        use gitpoulpi_config::release::{InstallChannel, Marker};
        for c in InstallChannel::ALL {
            let text = package_marker_text(c.as_str(), "1.2.3", "t", None).unwrap();
            let m = Marker::parse(&text).unwrap();
            assert_eq!(
                (m.version.as_str(), m.target.as_str(), m.channel, m.feed),
                ("1.2.3", "t", Some(c), None)
            );
        }
        let with_feed = package_marker_text("choco-internal", "1.2.3", "t", Some("https://f/"));
        assert_eq!(
            Marker::parse(&with_feed.unwrap()).unwrap().feed.as_deref(),
            Some("https://f/")
        );
        assert!(package_marker_text("cargo", "1", "t", None).is_err());
        assert!(package_marker_text("snap", "1", "t", None).is_err());
        assert!(package_marker_text("apt", "", "t", None).is_err());
    }

    #[test]
    fn package_marker_writes_the_file() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("marker");
        let args: Vec<String> = [
            "--channel",
            "apt",
            "--version",
            "0.3.0",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--out",
            out.to_str().unwrap(),
        ]
        .map(String::from)
        .to_vec();
        assert_eq!(package_marker(&args), Ok(0));
        let m = gitpoulpi_config::release::Marker::parse(&fs::read_to_string(&out).unwrap());
        assert_eq!(
            m.unwrap().channel,
            Some(gitpoulpi_config::release::InstallChannel::Apt)
        );
        assert!(package_marker(&args[..6]).is_err(), "--out is required");
    }

    #[test]
    fn checksum_lines_parse_and_latest_json_is_sorted_and_complete() {
        let hex = "ab".repeat(32);
        let line = checksum_line(&hex, "gitpoulpi-1.0.0-x.tar.gz");
        assert_eq!(
            parse_checksum_line(&line),
            Some((hex.clone(), "gitpoulpi-1.0.0-x.tar.gz".into()))
        );
        assert_eq!(parse_checksum_line("short  name"), None);
        assert_eq!(parse_checksum_line(""), None);
        let json = latest_json(
            "1.0.0",
            "Notes",
            &[
                (hex.clone(), "b.zip".into()),
                ("cd".repeat(32), "a.tar.gz".into()),
            ],
        );
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["version"], "1.0.0");
        assert_eq!(v["notes"], "Notes");
        assert_eq!(v["assets"]["b.zip"], hex);
        let names: Vec<&String> = v["assets"].as_object().unwrap().keys().collect();
        assert_eq!(names, ["a.tar.gz", "b.zip"]);
    }

    #[test]
    fn package_manifests_reads_the_sha256_files_of_a_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let hex = "0f".repeat(32);
        fs::write(
            tmp.path().join("gitpoulpi-1.0.0-x.zip.sha256"),
            checksum_line(&hex, "gitpoulpi-1.0.0-x.zip"),
        )
        .unwrap();
        fs::write(tmp.path().join("other.txt"), "ignored").unwrap();
        let args: Vec<String> = ["--dir", tmp.path().to_str().unwrap(), "--version", "1.0.0"]
            .map(String::from)
            .to_vec();
        assert_eq!(package_manifests(&args), Ok(0));
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(tmp.path().join("latest.json")).unwrap())
                .unwrap();
        assert_eq!(v["assets"]["gitpoulpi-1.0.0-x.zip"], hex);
        let empty = tempfile::tempdir().unwrap();
        let bad: Vec<String> = ["--dir", empty.path().to_str().unwrap()]
            .map(String::from)
            .to_vec();
        assert!(package_manifests(&bad).is_err());
    }

    #[test]
    fn the_marker_names_the_version_and_target() {
        let e = marker_entry("0.2.0", "x86_64-pc-windows-msvc");
        assert_eq!(e.name, ".gitpoulpi-dist");
        let m = gitpoulpi_config::release::Marker::parse(&String::from_utf8(e.data).unwrap());
        assert_eq!(m.unwrap().target, "x86_64-pc-windows-msvc");
    }

    #[test]
    fn dist_files_are_the_archive_names_and_tui_and_mcp_come_from_the_shim() {
        use gitpoulpi_config::release::{build_packages, built_binary, BINARIES};
        assert_eq!(BINARIES, super::BINARIES);
        assert_eq!(built_binary("gitpoulpi-tui"), Some("gitpoulpi-shim"));
        assert_eq!(built_binary("gitpoulpi-mcp"), Some("gitpoulpi-shim"));
        assert_eq!(built_binary("gitpoulpi"), Some("gitpoulpi"));
        assert_eq!(built_binary("gitpoulpi-launch"), Some("gitpoulpi-launch"));
        let p = build_packages();
        assert!(!p.contains(&"gitpoulpi-tui") && !p.contains(&"gitpoulpi-mcp"));
        assert!(p.contains(&"gitpoulpi-shim"));
    }

    #[test]
    fn install_text_names_the_files_of_that_platform() {
        let w = install_text("0.1.0", "x86_64-pc-windows-msvc");
        assert!(w.contains("gitpoulpi-mcp.exe") && w.contains("PATH"));
        let l = install_text("0.1.0", "x86_64-unknown-linux-gnu");
        assert!(l.contains("~/.local/bin") && !l.contains(".exe"));
    }

    #[test]
    fn sha256_matches_the_known_vector_and_the_line_is_sha256sum_shaped() {
        let hex = sha256_hex(b"abc");
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(checksum_line(&hex, "a.zip"), format!("{hex}  a.zip\n"));
    }

    #[test]
    fn tar_gz_has_a_top_level_dir_and_keeps_modes() {
        let bytes = write_tar_gz("gitpoulpi-0.1.0-x", &sample()).unwrap();
        let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(&bytes[..]));
        let mut seen = Vec::new();
        for entry in archive.entries().unwrap() {
            let mut entry = entry.unwrap();
            let path = entry.path().unwrap().to_string_lossy().into_owned();
            let mode = entry.header().mode().unwrap();
            let mut body = Vec::new();
            entry.read_to_end(&mut body).unwrap();
            seen.push((path, mode, body));
        }
        assert_eq!(
            seen,
            vec![
                (
                    "gitpoulpi-0.1.0-x/gitpoulpi".into(),
                    0o755,
                    b"binary".to_vec()
                ),
                (
                    "gitpoulpi-0.1.0-x/INSTALL.txt".into(),
                    0o644,
                    b"hello".to_vec()
                ),
            ]
        );
    }

    #[test]
    fn zip_has_a_top_level_dir_and_the_contents() {
        let bytes = write_zip("gitpoulpi-0.1.0-x", &sample()).unwrap();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 2);
        let mut body = String::new();
        archive
            .by_name("gitpoulpi-0.1.0-x/INSTALL.txt")
            .unwrap()
            .read_to_string(&mut body)
            .unwrap();
        assert_eq!(body, "hello");
        assert!(archive.by_name("gitpoulpi-0.1.0-x/gitpoulpi").is_ok());
    }
}
