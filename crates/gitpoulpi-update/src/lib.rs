//! The self-updater's engine (the updates-and-install design,
//! slice B). No UI: `gitpoulpi-tui` drives it from a background thread.
//!
//! The flow, each step its own module so each is tested alone:
//!
//! 1. [`fetch`]: download the release archive and its `.sha256` through an
//!    `HttpTransport`, refuse a wrong asset name or a missing / mismatching
//!    checksum, and report progress (cancellable).
//! 2. [`archive`]: unpack the archive in memory (zip on Windows targets,
//!    tar.gz elsewhere), keeping only the release's own files.
//! 3. [`install::stage`]: write each new file as `<name>.new` next to the
//!    old one. Nothing running is touched; the UI then says "Restart to
//!    update".
//! 4. [`install::apply`] (on the user's action): keep the previous files as
//!    `<name>.old` and move the new ones in. Unix replaces with an atomic
//!    rename (a running exe keeps its inode); Windows cannot overwrite a
//!    running exe, so it renames the old one aside first and moves the new
//!    one in. Any failure rolls the whole set back.
//! 5. [`install::revert`] swaps the `.old` files back; [`install::cleanup`]
//!    deletes them once the new version has proved itself.
//!
//! Every file operation goes through [`fsops::FileOps`], so the Windows path
//! is unit-tested on Linux against an in-memory file system that follows
//! Windows' rules. Not journaled in git undo: it changes GitPoulpi's own
//! files, not a repository.

pub mod archive;
pub mod fetch;
pub mod fsops;
pub mod install;
#[cfg(test)]
pub(crate) mod testutil;

use std::path::{Path, PathBuf};

use gitpoulpi_config::release::{self, Marker, MARKER_FILE};

use fsops::FileOps;

/// Why an update step stopped. The text is English and technical: the UI
/// words the outcome and shows this as the reason.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum UpdateError {
    #[error("canceled")]
    Canceled,
    #[error("download failed: {0}")]
    Download(String),
    #[error("the release has no checksum file for this archive, so it is not installed")]
    NoChecksum,
    #[error("the checksum does not match (expected {expected}, got {actual}): the download is not installed")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("the release asset is named {found}, expected {expected}: not installed")]
    WrongAsset { expected: String, found: String },
    #[error("bad archive: {0}")]
    Archive(String),
    #[error("{what}: {error}")]
    Io { what: String, error: String },
    #[error("{0} is in use by another program: close it and try again")]
    Locked(String),
    #[error("no downloaded update is waiting")]
    NothingStaged,
    #[error("there is no previous version to go back to")]
    NothingToRevert,
}

pub type Result<T> = std::result::Result<T, UpdateError>;

pub(crate) fn io_err(what: impl Into<String>, error: impl std::fmt::Display) -> UpdateError {
    UpdateError::Io {
        what: what.into(),
        error: error.to_string(),
    }
}

/// A build that came from a release archive: its folder, its exe's file name
/// and what the marker says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Updatable {
    pub dir: PathBuf,
    pub exe_name: String,
    pub marker: Marker,
}

/// Whether the build running as `exe` may update itself: the marker sits next
/// to it and names `host_target` (a marker for another platform means the
/// files were copied around: not ours to replace). `None` for every `cargo`
/// build.
pub fn detect(fs: &dyn FileOps, exe: &Path, host_target: &str) -> Option<Updatable> {
    let dir = exe.parent()?.to_path_buf();
    let exe_name = exe.file_name()?.to_str()?.to_string();
    let marker = Marker::parse(&fs.read_to_string(&dir.join(MARKER_FILE)).ok()?)?;
    (marker.target == host_target).then_some(Updatable {
        dir,
        exe_name,
        marker,
    })
}

/// The file names an install consists of for `target`: the binaries with
/// their platform suffix, then the marker.
pub fn managed_names(target: &str) -> Vec<String> {
    release::BINARIES
        .iter()
        .map(|b| release::exe_name(b, target))
        .chain([MARKER_FILE.to_string()])
        .collect()
}
