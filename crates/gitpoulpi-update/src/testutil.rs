//! Test helpers: archives built by the real packer, releases pointing at
//! them, and an in-memory file system that follows Windows' rules.

use std::collections::{BTreeMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use gitpoulpi_config::release::{self, MARKER_FILE};
use gitpoulpi_hosting::updates::{Asset, Release};
use xtask::dist::{marker_entry, write_tar_gz, write_zip, Entry};

use crate::fsops::FileOps;
use crate::managed_names;

/// The files `xtask dist` would pack for `version` / `target`.
pub fn entries_for(version: &str, target: &str) -> Vec<Entry> {
    let mut entries: Vec<Entry> = managed_names(target)
        .into_iter()
        .filter(|n| n != MARKER_FILE)
        .map(|name| Entry {
            data: format!("binary {name} {version}").into_bytes(),
            name,
            executable: true,
        })
        .collect();
    entries.push(marker_entry(version, target));
    entries.push(Entry {
        name: "README.md".into(),
        data: b"readme".to_vec(),
        executable: false,
    });
    entries
}

/// The archive `xtask dist` would pack for `version` / `target`.
pub fn archive_for(version: &str, target: &str) -> Vec<u8> {
    let entries = entries_for(version, target);
    let stem = release::package_stem(version, target);
    if release::is_windows(target) {
        write_zip(&stem, &entries).unwrap()
    } else {
        write_tar_gz(&stem, &entries).unwrap()
    }
}

/// A release whose archive is at `url` and whose checksum is at `url` plus
/// `.sha256`. The asset names follow the target, `url` is free (the tests
/// serve by URL fragment).
pub fn release_for(version: &str, target: &str, url: &str) -> Release {
    let name = release::archive_name(version, target);
    Release {
        tag: format!("v{version}"),
        version: version.into(),
        prerelease: false,
        notes: String::new(),
        page_url: "https://github.com/o/r/releases".into(),
        asset: Some(Asset {
            name: name.clone(),
            url: url.into(),
            size: 0,
        }),
        checksum: Some(Asset {
            name: format!("{name}.sha256"),
            url: format!("{url}.sha256"),
            size: 0,
        }),
    }
}

struct Node {
    data: Vec<u8>,
    /// A process runs this file (Windows: it cannot be overwritten or
    /// deleted, only renamed). The flag travels with the file when renamed.
    running: bool,
}

/// An in-memory file system. `windows` makes a running file refuse removal
/// and replacement; `fail_rename_to` injects a failure (once per path).
pub struct FakeFs {
    files: Mutex<BTreeMap<PathBuf, Node>>,
    windows: bool,
    fail_rename_to: Mutex<HashSet<PathBuf>>,
}

fn denied(what: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!("{what}: used by another process"),
    )
}

impl FakeFs {
    pub fn new(windows: bool) -> Self {
        Self {
            files: Mutex::default(),
            windows,
            fail_rename_to: Mutex::default(),
        }
    }

    pub fn put(&self, path: impl Into<PathBuf>, data: &str) {
        self.files.lock().unwrap().insert(
            path.into(),
            Node {
                data: data.as_bytes().to_vec(),
                running: false,
            },
        );
    }

    pub fn set_running(&self, path: &Path, running: bool) {
        if let Some(n) = self.files.lock().unwrap().get_mut(path) {
            n.running = running;
        }
    }

    pub fn fail_rename_to(&self, path: impl Into<PathBuf>) {
        self.fail_rename_to.lock().unwrap().insert(path.into());
    }

    pub fn text(&self, path: &Path) -> Option<String> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .map(|n| String::from_utf8_lossy(&n.data).into_owned())
    }

    fn protected(&self, n: &Node) -> bool {
        self.windows && n.running
    }
}

impl FileOps for FakeFs {
    fn exists(&self, path: &Path) -> bool {
        self.files.lock().unwrap().contains_key(path)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        self.text(path)
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))
    }

    fn write(&self, path: &Path, data: &[u8], _executable: bool) -> io::Result<()> {
        let mut files = self.files.lock().unwrap();
        if files.get(path).is_some_and(|n| self.protected(n)) {
            return Err(denied("write"));
        }
        files.insert(
            path.to_path_buf(),
            Node {
                data: data.to_vec(),
                running: false,
            },
        );
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        if self.fail_rename_to.lock().unwrap().remove(to) {
            return Err(io::Error::other("injected failure"));
        }
        let mut files = self.files.lock().unwrap();
        if files.get(to).is_some_and(|n| self.protected(n)) {
            return Err(denied("rename onto a running file"));
        }
        let node = files
            .remove(from)
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        files.insert(to.to_path_buf(), node);
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        let mut files = self.files.lock().unwrap();
        match files.get(path) {
            None => Err(io::Error::from(io::ErrorKind::NotFound)),
            Some(n) if self.protected(n) => Err(denied("delete")),
            Some(_) => {
                files.remove(path);
                Ok(())
            }
        }
    }

    fn link_or_copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        let data = self
            .files
            .lock()
            .unwrap()
            .get(from)
            .map(|n| n.data.clone())
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        self.put(to.to_path_buf(), &String::from_utf8_lossy(&data));
        Ok(())
    }

    fn is_locked(&self, path: &Path) -> bool {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .is_some_and(|n| self.protected(n))
    }
}
