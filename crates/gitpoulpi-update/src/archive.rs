//! Unpacking a release archive in memory.
//!
//! Only the release's own files are taken, by exact name, from the archive's
//! top-level directory (`gitpoulpi-<version>-<target>/`): the binaries and the
//! marker. Nothing else is written, whatever the archive contains, so a
//! crafted entry name (`../x`, an absolute path) has nothing to land on. A zip
//! entry whose name is not enclosed in the archive root is refused outright.

use std::io::Read;

use gitpoulpi_config::release::{self, Marker, MARKER_FILE};

use crate::{managed_names, Result, UpdateError};

/// One file taken out of the archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveFile {
    pub name: String,
    pub data: Vec<u8>,
    pub executable: bool,
}

/// No single file may be larger than this (a binary is a few tens of MiB).
const MAX_FILE: u64 = 512 * 1024 * 1024;

/// The files of `bytes`, an archive of `version` for `target`. Fails when the
/// marker is missing or names another version or target, or when no binary is
/// inside.
pub fn extract(bytes: &[u8], version: &str, target: &str) -> Result<Vec<ArchiveFile>> {
    let stem = release::package_stem(version, target);
    let wanted = managed_names(target);
    let files = if release::is_windows(target) {
        read_zip(bytes, &stem, &wanted)?
    } else {
        read_tar_gz(bytes, &stem, &wanted)?
    };
    let marker = files
        .iter()
        .find(|f| f.name == MARKER_FILE)
        .ok_or_else(|| UpdateError::Archive(format!("{MARKER_FILE} is missing")))?;
    let parsed = Marker::parse(&String::from_utf8_lossy(&marker.data))
        .ok_or_else(|| UpdateError::Archive(format!("{MARKER_FILE} is unreadable")))?;
    if parsed.version != version || parsed.target != target {
        return Err(UpdateError::Archive(format!(
            "the archive says {} for {}, expected {version} for {target}",
            parsed.version, parsed.target
        )));
    }
    if !files.iter().any(|f| f.name != MARKER_FILE) {
        return Err(UpdateError::Archive("no binary inside".into()));
    }
    Ok(files)
}

fn bad(e: impl std::fmt::Display) -> UpdateError {
    UpdateError::Archive(e.to_string())
}

fn read_tar_gz(bytes: &[u8], stem: &str, wanted: &[String]) -> Result<Vec<ArchiveFile>> {
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(bytes));
    let mut out = Vec::new();
    for entry in archive.entries().map_err(bad)? {
        let mut entry = entry.map_err(bad)?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry
            .path()
            .map_err(bad)?
            .to_string_lossy()
            .replace('\\', "/");
        let Some(name) = path
            .strip_prefix(stem)
            .and_then(|p| p.strip_prefix('/'))
            .filter(|n| wanted.iter().any(|w| w == n))
        else {
            continue;
        };
        if entry.size() > MAX_FILE {
            return Err(UpdateError::Archive(format!("{name} is too large")));
        }
        let executable = entry.header().mode().is_ok_and(|m| m & 0o111 != 0);
        let mut data = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut data).map_err(bad)?;
        out.push(ArchiveFile {
            name: name.to_string(),
            data,
            executable,
        });
    }
    Ok(out)
}

fn read_zip(bytes: &[u8], stem: &str, wanted: &[String]) -> Result<Vec<ArchiveFile>> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(bad)?;
    let mut out = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(bad)?;
        // `enclosed_name` is the crate's own check: no NUL byte, no absolute
        // path, nothing that climbs out of the archive root. An entry that
        // fails it is a crafted archive, refused as a whole. A symlink is
        // never taken (its target is not a file of the release).
        let Some(enclosed) = file.enclosed_name() else {
            return Err(UpdateError::Archive(
                "an archive entry has an unsafe path".to_string(),
            ));
        };
        if !file.is_file() {
            continue;
        }
        let path = enclosed.to_string_lossy().replace('\\', "/");
        let Some(name) = path
            .strip_prefix(stem)
            .and_then(|p| p.strip_prefix('/'))
            .filter(|n| wanted.iter().any(|w| w == n))
        else {
            continue;
        };
        if file.size() > MAX_FILE {
            return Err(UpdateError::Archive(format!("{name} is too large")));
        }
        let mut data = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut data).map_err(bad)?;
        // Windows has no mode bits to keep: the `.exe` suffix is enough.
        out.push(ArchiveFile {
            name: name.to_string(),
            data,
            executable: true,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::entries_for as entries;
    use xtask::dist::{marker_entry, write_tar_gz, write_zip, Entry};

    const LINUX: &str = "x86_64-unknown-linux-gnu";
    const WIN: &str = "x86_64-pc-windows-msvc";

    #[test]
    fn a_tar_gz_gives_the_binaries_and_the_marker_only() {
        let bytes = write_tar_gz(
            &release::package_stem("0.2.0", LINUX),
            &entries("0.2.0", LINUX),
        )
        .unwrap();
        let files = extract(&bytes, "0.2.0", LINUX).unwrap();
        let mut names: Vec<_> = files.iter().map(|f| f.name.as_str()).collect();
        names.sort();
        assert_eq!(
            names,
            [
                ".gitpoulpi-dist",
                "gitpoulpi",
                "gitpoulpi-launch",
                "gitpoulpi-mcp",
                "gitpoulpi-tui",
                "gitpoulpi-updater"
            ]
        );
        assert!(
            files
                .iter()
                .find(|f| f.name == "gitpoulpi")
                .unwrap()
                .executable
        );
        assert!(
            !files
                .iter()
                .find(|f| f.name == MARKER_FILE)
                .unwrap()
                .executable
        );
    }

    #[test]
    fn a_zip_gives_the_exes() {
        let bytes =
            write_zip(&release::package_stem("0.2.0", WIN), &entries("0.2.0", WIN)).unwrap();
        let files = extract(&bytes, "0.2.0", WIN).unwrap();
        assert!(files.iter().any(|f| f.name == "gitpoulpi-tui.exe"));
        assert!(files.iter().any(|f| f.name == MARKER_FILE));
    }

    #[test]
    fn a_zip_with_an_unsafe_entry_path_is_refused() {
        let stem = release::package_stem("0.2.0", WIN);
        for bad_name in ["../../evil", "../../../etc/passwd", "sub/../../../evil"] {
            let mut e = entries("0.2.0", WIN);
            e.push(Entry {
                name: bad_name.into(),
                data: b"x".to_vec(),
                executable: false,
            });
            let bytes = write_zip(&stem, &e).unwrap();
            assert!(
                matches!(extract(&bytes, "0.2.0", WIN), Err(UpdateError::Archive(_))),
                "{bad_name} must be refused"
            );
        }
        // An absolute entry name (an empty stem puts a leading slash).
        let bytes = write_zip("", &entries("0.2.0", WIN)).unwrap();
        assert!(matches!(
            extract(&bytes, "0.2.0", WIN),
            Err(UpdateError::Archive(_))
        ));
        // A plain archive still extracts.
        let bytes = write_zip(&stem, &entries("0.2.0", WIN)).unwrap();
        assert!(extract(&bytes, "0.2.0", WIN).is_ok());
    }

    #[test]
    fn a_marker_for_another_version_or_target_refuses() {
        let stem = release::package_stem("0.2.0", LINUX);
        let mut e = entries("0.2.0", LINUX);
        e.retain(|x| x.name != MARKER_FILE);
        e.push(marker_entry("0.9.9", LINUX));
        let bytes = write_tar_gz(&stem, &e).unwrap();
        assert!(matches!(
            extract(&bytes, "0.2.0", LINUX),
            Err(UpdateError::Archive(_))
        ));
        // No marker at all.
        let mut e = entries("0.2.0", LINUX);
        e.retain(|x| x.name != MARKER_FILE);
        let bytes = write_tar_gz(&stem, &e).unwrap();
        assert!(extract(&bytes, "0.2.0", LINUX).is_err());
    }

    #[test]
    fn files_outside_the_release_directory_are_ignored() {
        // Right names, wrong top-level directory: nothing is taken.
        let bytes = write_tar_gz("evil", &entries("0.2.0", LINUX)).unwrap();
        assert!(extract(&bytes, "0.2.0", LINUX).is_err());
        // A name that is not a managed one inside the right directory is skipped
        // (the tar writer itself refuses `..`, so a nested name stands in).
        let mut e = entries("0.2.0", LINUX);
        e.push(Entry {
            name: "sub/escape".into(),
            data: b"x".to_vec(),
            executable: false,
        });
        let bytes = write_tar_gz(&release::package_stem("0.2.0", LINUX), &e).unwrap();
        let files = extract(&bytes, "0.2.0", LINUX).unwrap();
        assert!(files.iter().all(|f| !f.name.contains('/')));
    }

    #[test]
    fn garbage_is_an_archive_error() {
        assert!(matches!(
            extract(b"not an archive", "0.2.0", LINUX),
            Err(UpdateError::Archive(_))
        ));
        assert!(matches!(
            extract(b"not an archive", "0.2.0", WIN),
            Err(UpdateError::Archive(_))
        ));
    }
}
