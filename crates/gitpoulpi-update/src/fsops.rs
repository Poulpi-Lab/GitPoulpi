//! The file operations the updater needs, behind a trait so the tests run in
//! a temp folder (or in memory, simulating Windows' rules on Linux).

use std::io;
use std::path::Path;

pub trait FileOps: Send + Sync {
    fn exists(&self, path: &Path) -> bool;
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    /// Creates or replaces `path`; `executable` sets the Unix mode 0755.
    fn write(&self, path: &Path, data: &[u8], executable: bool) -> io::Result<()>;
    /// Renames `from` to `to`, replacing `to` when it exists (and is not
    /// protected by the OS).
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    /// A second name for `from`'s content (a hard link, else a copy): the
    /// previous version's `.old` on Unix, where the original is then
    /// replaced by an atomic rename.
    fn link_or_copy(&self, from: &Path, to: &Path) -> io::Result<()>;
    /// Whether another process holds `path` open so it cannot be moved
    /// aside or replaced (Windows: a running exe; elsewhere never).
    fn is_locked(&self, path: &Path) -> bool;
}

/// The real file system.
pub struct RealFs;

impl FileOps for RealFs {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn write(&self, path: &Path, data: &[u8], executable: bool) -> io::Result<()> {
        std::fs::write(path, data)?;
        #[cfg(unix)]
        if executable {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
        }
        #[cfg(not(unix))]
        let _ = executable;
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }

    fn link_or_copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::hard_link(from, to).or_else(|_| std::fs::copy(from, to).map(|_| ()))
    }

    fn is_locked(&self, path: &Path) -> bool {
        // Windows refuses a write handle on a running image; Linux says
        // "text file busy": both mean a process runs it.
        path.exists() && std::fs::OpenOptions::new().write(true).open(path).is_err()
    }
}
