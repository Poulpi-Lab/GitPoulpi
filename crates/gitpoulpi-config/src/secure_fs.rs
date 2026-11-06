//! User-only files and directories for the local state GitPoulpi keeps.
//!
//! The IPC token, the config, the sessions and the logs are the owner's: no
//! other account on the machine has a reason to read them. This module is the
//! one place that decides how they are created.
//!
//! * [`create_private_dir_all`] makes directories `0700` on Unix.
//! * [`write_private`] writes atomically (temporary file in the same
//!   directory, then rename) and creates the file `0600` from the start, so
//!   there is never a window where it is readable and no `chmod` after the
//!   write.
//! * [`tighten_owned`] narrows an entry that already exists (a file written
//!   by an older build) when it is ours and not a symlink.
//! * [`harden_app_dirs`] runs that over the app-owned roots once per binary
//!   start. It never touches working-tree files and sets no process-wide
//!   `umask` (a umask would also change the modes of checked-out files).
//!
//! Windows has no mode bits: [`protect_user_only`] replaces the DACL with the
//! current user plus SYSTEM, and [`has_foreign_access`] says whether Everyone,
//! Users or Authenticated Users hold an allow entry.

use std::io;
use std::path::{Path, PathBuf};

use crate::ConfigPaths;

/// Creates `path` and its missing parents, `0700` on Unix.
pub fn create_private_dir_all(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(path)
    }
    #[cfg(not(unix))]
    {
        std::fs::create_dir_all(path)
    }
}

/// Where a write to `path` lands: the target of a symlink (a config file
/// kept in a dotfiles repository stays a link), else `path` itself.
fn write_target(path: &Path) -> PathBuf {
    match std::fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => {
            std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
        }
        _ => path.to_path_buf(),
    }
}

/// Writes `bytes` to `path` so that only the current user can read it: a
/// temporary file next to it, created `0600` on Unix, then renamed over
/// `path`. The parent directory must exist.
pub fn write_private(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let target = write_target(path);
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = target.with_file_name(format!(
        ".{name}.{}.{}.tmp",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| {
        let mut file = options.open(&tmp)?;
        file.write_all(bytes)?;
        file.flush()?;
        drop(file);
        std::fs::rename(&tmp, &target)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Clears the group and other permission bits of `path` when it is owned by
/// the current user and is not a symlink. Anything else is left alone. A
/// no-op off Unix.
pub fn tighten_owned(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let meta = std::fs::symlink_metadata(path)?;
        if meta.file_type().is_symlink() {
            return Ok(());
        }
        // SAFETY: geteuid has no preconditions and cannot fail.
        if meta.uid() != unsafe { libc::geteuid() } {
            return Ok(());
        }
        let mode = meta.mode() & 0o7777;
        if mode & 0o077 != 0 {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode & !0o077))?;
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

/// How deep and how wide [`harden_app_dirs`] walks below a root: the roots
/// hold a handful of files, the cap only stops a runaway cache.
#[cfg(unix)]
const WALK_DEPTH: usize = 4;
#[cfg(unix)]
const WALK_ENTRIES: usize = 20_000;

#[cfg(unix)]
fn tighten_tree(path: &Path, depth: usize, budget: &mut usize) {
    if *budget == 0 {
        return;
    }
    *budget -= 1;
    let _ = tighten_owned(path);
    let is_dir = std::fs::symlink_metadata(path)
        .map(|m| m.file_type().is_dir())
        .unwrap_or(false);
    if !is_dir || depth == 0 {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            tighten_tree(&entry.path(), depth - 1, budget);
        }
    }
}

/// Hardens the app-owned roots (config, sessions, cache, data, logs, the
/// private directory): creates the private directory and, on Unix, clears
/// group and other access on what an older build left behind; on Windows
/// protects the config root when it shows foreign access (a portable
/// install on a shared drive). `updates/` is never protected explicitly on
/// Windows: the elevated updater may run as another administrator account
/// and must still read `job.json`. Best effort, never fails a start.
pub fn harden_app_dirs(paths: &ConfigPaths) {
    let _ = create_private_dir_all(&paths.private_dir());
    #[cfg(unix)]
    {
        let mut roots = vec![
            paths.config_dir().to_path_buf(),
            paths.cache_dir(),
            paths.data_dir(),
            paths.log_dir(),
        ];
        roots.sort();
        roots.dedup();
        for root in roots {
            let mut budget = WALK_ENTRIES;
            tighten_tree(&root, WALK_DEPTH, &mut budget);
        }
    }
    #[cfg(windows)]
    {
        let root = paths.config_dir();
        if root.exists() && has_foreign_access(root) {
            let _ = protect_user_only(root);
        }
    }
}

/// Replaces the DACL of `path` with a protected one granting full control to
/// the current user and SYSTEM only (inheritable, so a directory's children
/// follow). Windows only.
#[cfg(windows)]
pub fn protect_user_only(path: &Path) -> io::Result<()> {
    win::protect_user_only(path)
}

/// Whether `path`'s DACL has an allow entry for Everyone, BUILTIN\Users or
/// Authenticated Users. `false` when it cannot be read. Windows only.
#[cfg(windows)]
pub fn has_foreign_access(path: &Path) -> bool {
    win::has_foreign_access(path)
}

#[cfg(windows)]
mod win {
    use std::io;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    use windows_sys::Win32::Foundation::{CloseHandle, LocalFree, ERROR_SUCCESS, HANDLE};
    use windows_sys::Win32::Security::Authorization::{
        ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
        ConvertStringSidToSidW, GetNamedSecurityInfoW, SetNamedSecurityInfoW, SDDL_REVISION_1,
        SE_FILE_OBJECT,
    };
    use windows_sys::Win32::Security::{
        EqualSid, GetAce, GetSecurityDescriptorDacl, GetTokenInformation, TokenUser, ACL,
        DACL_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID,
        TOKEN_QUERY, TOKEN_USER,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    fn wide(s: &std::ffi::OsStr) -> Vec<u16> {
        s.encode_wide().chain(std::iter::once(0)).collect()
    }

    /// The current user's SID as a string, from the process token.
    fn current_user_sid() -> io::Result<String> {
        // SAFETY: plain token calls; the handle and the allocated string are
        // released on every path.
        unsafe {
            let mut token: HANDLE = std::ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return Err(io::Error::last_os_error());
            }
            let mut needed = 0u32;
            GetTokenInformation(token, TokenUser, std::ptr::null_mut(), 0, &mut needed);
            let mut buf = vec![0u8; needed.max(1) as usize];
            let ok = GetTokenInformation(
                token,
                TokenUser,
                buf.as_mut_ptr().cast(),
                needed,
                &mut needed,
            );
            CloseHandle(token);
            if ok == 0 {
                return Err(io::Error::last_os_error());
            }
            let user = &*(buf.as_ptr() as *const TOKEN_USER);
            let mut raw: *mut u16 = std::ptr::null_mut();
            if ConvertSidToStringSidW(user.User.Sid, &mut raw) == 0 {
                return Err(io::Error::last_os_error());
            }
            let mut len = 0;
            while *raw.add(len) != 0 {
                len += 1;
            }
            let sid = String::from_utf16_lossy(std::slice::from_raw_parts(raw, len));
            LocalFree(raw.cast());
            Ok(sid)
        }
    }

    pub fn protect_user_only(path: &Path) -> io::Result<()> {
        let sid = current_user_sid()?;
        let sddl = wide(std::ffi::OsStr::new(&format!(
            "D:PAI(A;OICI;FA;;;{sid})(A;OICI;FA;;;SY)"
        )));
        let name = wide(path.as_os_str());
        // SAFETY: the descriptor comes from the conversion call and is freed
        // after the DACL pointer inside it was used.
        unsafe {
            let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
            if ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                SDDL_REVISION_1,
                &mut sd,
                std::ptr::null_mut(),
            ) == 0
            {
                return Err(io::Error::last_os_error());
            }
            let (mut present, mut defaulted) = (0, 0);
            let mut dacl: *mut ACL = std::ptr::null_mut();
            let got = GetSecurityDescriptorDacl(sd, &mut present, &mut dacl, &mut defaulted);
            let result = if got == 0 || present == 0 {
                Err(io::Error::last_os_error())
            } else {
                let status = SetNamedSecurityInfoW(
                    name.as_ptr(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    dacl,
                    std::ptr::null(),
                );
                if status == ERROR_SUCCESS {
                    Ok(())
                } else {
                    Err(io::Error::from_raw_os_error(status as i32))
                }
            };
            LocalFree(sd);
            result
        }
    }

    pub fn has_foreign_access(path: &Path) -> bool {
        let name = wide(path.as_os_str());
        // Everyone, BUILTIN\Users, Authenticated Users.
        let foreign: Vec<Vec<u16>> = ["S-1-1-0", "S-1-5-32-545", "S-1-5-11"]
            .iter()
            .map(|s| wide(std::ffi::OsStr::new(s)))
            .collect();
        // SAFETY: the security descriptor and the SIDs are freed on every
        // path; the ACE pointers are only read while the descriptor lives.
        unsafe {
            let mut dacl: *mut ACL = std::ptr::null_mut();
            let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
            let status = GetNamedSecurityInfoW(
                name.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut dacl,
                std::ptr::null_mut(),
                &mut sd,
            );
            if status != ERROR_SUCCESS {
                return false;
            }
            let mut found = false;
            if !dacl.is_null() {
                let sids: Vec<PSID> = foreign
                    .iter()
                    .filter_map(|s| {
                        let mut sid: PSID = std::ptr::null_mut();
                        (ConvertStringSidToSidW(s.as_ptr(), &mut sid) != 0).then_some(sid)
                    })
                    .collect();
                for index in 0..(*dacl).AceCount as u32 {
                    let mut ace: *mut core::ffi::c_void = std::ptr::null_mut();
                    if GetAce(dacl, index, &mut ace) == 0 || ace.is_null() {
                        continue;
                    }
                    // ACCESS_ALLOWED_ACE_TYPE (0): header, mask, then the SID.
                    if *(ace as *const u8) != 0 {
                        continue;
                    }
                    let sid = (ace as *mut u8).add(8) as PSID;
                    if sids.iter().any(|f| EqualSid(sid, *f) != 0) {
                        found = true;
                        break;
                    }
                }
                for sid in sids {
                    LocalFree(sid);
                }
            }
            LocalFree(sd);
            found
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_private_replaces_the_file_and_leaves_no_temporary() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.toml");
        write_private(&file, b"one").unwrap();
        write_private(&file, b"two").unwrap();
        assert_eq!(std::fs::read(&file).unwrap(), b"two");
        let names: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(names, vec![std::ffi::OsString::from("a.toml")]);
    }

    #[cfg(unix)]
    mod unix {
        use super::*;
        use std::os::unix::fs::PermissionsExt;

        fn mode(path: &Path) -> u32 {
            std::fs::metadata(path).unwrap().permissions().mode() & 0o777
        }

        #[test]
        fn a_private_file_is_created_user_only_even_under_a_loose_umask() {
            let dir = tempfile::tempdir().unwrap();
            let file = dir.path().join("secret");
            // SAFETY: umask is process-wide; restored right after the write.
            let old = unsafe { libc::umask(0) };
            let result = write_private(&file, b"x");
            unsafe { libc::umask(old) };
            result.unwrap();
            assert_eq!(mode(&file), 0o600);
        }

        #[test]
        fn a_private_dir_is_0700_with_its_new_parents() {
            let dir = tempfile::tempdir().unwrap();
            let deep = dir.path().join("a").join("b");
            create_private_dir_all(&deep).unwrap();
            assert_eq!(mode(&deep), 0o700);
            assert_eq!(mode(&dir.path().join("a")), 0o700);
        }

        #[test]
        fn a_symlinked_target_stays_a_link() {
            let dir = tempfile::tempdir().unwrap();
            let real = dir.path().join("real.toml");
            std::fs::write(&real, "old").unwrap();
            let link = dir.path().join("config.toml");
            std::os::unix::fs::symlink(&real, &link).unwrap();
            write_private(&link, b"new").unwrap();
            assert!(std::fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink());
            assert_eq!(std::fs::read(&real).unwrap(), b"new");
        }

        #[test]
        fn tighten_owned_clears_group_and_other_bits() {
            let dir = tempfile::tempdir().unwrap();
            let file = dir.path().join("old.toml");
            std::fs::write(&file, "x").unwrap();
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o664)).unwrap();
            tighten_owned(&file).unwrap();
            assert_eq!(mode(&file), 0o600);
        }

        #[test]
        fn tighten_owned_skips_a_symlink_and_its_target() {
            let dir = tempfile::tempdir().unwrap();
            let target = dir.path().join("target");
            std::fs::write(&target, "x").unwrap();
            std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644)).unwrap();
            let link = dir.path().join("link");
            std::os::unix::fs::symlink(&target, &link).unwrap();
            tighten_owned(&link).unwrap();
            assert_eq!(
                mode(&target),
                0o644,
                "the link's target is not ours to change"
            );
        }

        #[test]
        fn harden_app_dirs_tightens_what_an_older_build_left_and_makes_private() {
            let dir = tempfile::tempdir().unwrap();
            let paths = ConfigPaths::for_dir(dir.path().join("cfg"));
            std::fs::create_dir_all(paths.sessions_dir()).unwrap();
            let config = paths.config_file();
            std::fs::write(&config, "x").unwrap();
            std::fs::set_permissions(&config, std::fs::Permissions::from_mode(0o644)).unwrap();
            std::fs::set_permissions(paths.sessions_dir(), std::fs::Permissions::from_mode(0o755))
                .unwrap();
            harden_app_dirs(&paths);
            assert_eq!(mode(&config), 0o600);
            assert_eq!(mode(&paths.sessions_dir()), 0o700);
            assert_eq!(mode(&paths.private_dir()), 0o700);
        }
    }

    #[cfg(windows)]
    #[test]
    fn protect_user_only_removes_foreign_access() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("secret");
        std::fs::write(&file, "x").unwrap();
        protect_user_only(&file).unwrap();
        assert!(!has_foreign_access(&file));
        assert_eq!(std::fs::read(&file).unwrap(), b"x");
    }
}
