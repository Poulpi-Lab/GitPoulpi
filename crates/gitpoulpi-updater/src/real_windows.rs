//! The Windows half of the real machine: the process list (Toolhelp), the
//! processes holding files open (Restart Manager, which sees other users'
//! processes only when elevated), the elevation prompt
//! (`ShellExecuteExW` with the `runas` verb) and the registry.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_CANCELLED, ERROR_MORE_DATA, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::RestartManager::{
    RmEndSession, RmGetList, RmRegisterResources, RmStartSession, RM_PROCESS_INFO,
};
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, OpenProcess, QueryFullProcessImageNameW, TerminateProcess,
    WaitForSingleObject, INFINITE, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};
use windows_sys::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};

use gitpoulpi_ipc::discovery;
use gitpoulpi_update::channel::{Hive, RegistryOps};
use gitpoulpi_update::winreg::WindowsRegistry;

use crate::ops::{LaunchPlan, Proc, ProcessOps, Registry, RunError};
use crate::real::{relaunch_now, run_inherit, version_of};

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn norm(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/").to_lowercase()
}

/// Whether `exe` is a file inside `dir`.
fn inside(dir: &Path, exe: &Path) -> bool {
    let dir = norm(dir);
    norm(exe).starts_with(&format!("{}/", dir.trim_end_matches('/')))
}

fn pids() -> Vec<u32> {
    let mut found = Vec::new();
    // SAFETY: a snapshot handle, closed below; `entry` is sized as the API
    // requires.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return found;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut more = Process32FirstW(snapshot, &mut entry);
        while more != 0 {
            found.push(entry.th32ProcessID);
            more = Process32NextW(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
    }
    found
}

/// The full path of a process's executable; `None` when this user may not
/// look (another user's process, unless elevated).
fn image_path(pid: u32) -> Option<PathBuf> {
    // SAFETY: the handle is closed on every path; the buffer is as long as
    // the length passed.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
        CloseHandle(handle);
        (ok != 0).then(|| PathBuf::from(OsString::from_wide(&buf[..len as usize])))
    }
}

/// A command line argument as `CommandLineToArgvW` reads it back.
fn quote(arg: &str) -> String {
    if !arg.is_empty() && !arg.contains([' ', '\t', '"']) {
        return arg.to_string();
    }
    format!("\"{}\"", arg.replace('"', "\\\""))
}

pub struct RealProc;

impl ProcessOps for RealProc {
    fn is_alive(&self, pid: u32) -> bool {
        discovery::pid_alive(pid)
    }

    fn in_dir(&self, dir: &Path, exclude: &[u32]) -> Vec<Proc> {
        pids()
            .into_iter()
            .filter(|pid| !exclude.contains(pid))
            .filter_map(|pid| {
                let exe = image_path(pid)?;
                inside(dir, &exe).then_some(Proc { pid, exe })
            })
            .collect()
    }

    fn locking(&self, files: &[PathBuf], exclude: &[u32]) -> Vec<Proc> {
        let names: Vec<Vec<u16>> = files
            .iter()
            .filter(|f| f.exists())
            .map(|f| wide(&f.to_string_lossy()))
            .collect();
        if names.is_empty() {
            return Vec::new();
        }
        let ptrs: Vec<*const u16> = names.iter().map(|n| n.as_ptr()).collect();
        let mut found = Vec::new();
        // SAFETY: the session is ended on every path; every pointer passed
        // outlives its call and the info buffer has `needed` entries.
        unsafe {
            let mut session = 0u32;
            let mut key = [0u16; 33];
            if RmStartSession(&mut session, 0, key.as_mut_ptr()) != 0 {
                return found;
            }
            let registered = RmRegisterResources(
                session,
                ptrs.len() as u32,
                ptrs.as_ptr(),
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
            );
            if registered == 0 {
                let mut needed = 0u32;
                let mut count = 0u32;
                let mut reasons = 0u32;
                let rc = RmGetList(
                    session,
                    &mut needed,
                    &mut count,
                    std::ptr::null_mut(),
                    &mut reasons,
                );
                if rc == ERROR_MORE_DATA && needed > 0 {
                    let mut infos: Vec<RM_PROCESS_INFO> = vec![std::mem::zeroed(); needed as usize];
                    count = needed;
                    let rc = RmGetList(
                        session,
                        &mut needed,
                        &mut count,
                        infos.as_mut_ptr(),
                        &mut reasons,
                    );
                    if rc == 0 {
                        for info in &infos[..count as usize] {
                            let pid = info.Process.dwProcessId;
                            if !exclude.contains(&pid) {
                                let exe = image_path(pid).unwrap_or_default();
                                found.push(Proc { pid, exe });
                            }
                        }
                    }
                }
            }
            RmEndSession(session);
        }
        found
    }

    fn terminate(&self, pid: u32, _force: bool) -> Result<(), String> {
        // SAFETY: the handle is closed on every path.
        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
            if handle.is_null() {
                return Err(format!(
                    "could not open process {pid} (error {})",
                    GetLastError()
                ));
            }
            let ok = TerminateProcess(handle, 1);
            CloseHandle(handle);
            if ok == 0 {
                return Err(format!("could not end process {pid}"));
            }
        }
        Ok(())
    }

    fn run(&self, argv: &[String]) -> Result<i32, RunError> {
        run_inherit(argv)
    }

    fn run_elevated(&self, argv: &[String]) -> Result<i32, RunError> {
        let (program, rest) = argv
            .split_first()
            .ok_or_else(|| RunError::Failed("an empty command".into()))?;
        let verb = wide("runas");
        let file = wide(program);
        let params = wide(&rest.iter().map(|a| quote(a)).collect::<Vec<_>>().join(" "));
        // SAFETY: every string is NUL-terminated and outlives the call; the
        // process handle ShellExecuteExW returns is closed below.
        unsafe {
            let mut info: SHELLEXECUTEINFOW = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
            info.fMask = SEE_MASK_NOCLOSEPROCESS;
            info.lpVerb = verb.as_ptr();
            info.lpFile = file.as_ptr();
            info.lpParameters = params.as_ptr();
            info.nShow = 1;
            if ShellExecuteExW(&mut info) == 0 {
                let error = GetLastError();
                return Err(if error == ERROR_CANCELLED {
                    RunError::Declined
                } else {
                    RunError::Failed(format!("the elevation prompt failed (error {error})"))
                });
            }
            if info.hProcess.is_null() {
                return Ok(0);
            }
            WaitForSingleObject(info.hProcess, INFINITE);
            let mut code = 1u32;
            GetExitCodeProcess(info.hProcess, &mut code);
            CloseHandle(info.hProcess);
            Ok(code as i32)
        }
    }

    fn version_of(&self, exe: &Path) -> Option<String> {
        version_of(exe)
    }

    fn relaunch(&self, plan: &LaunchPlan) -> Result<(), String> {
        relaunch_now(plan)
    }
}

pub struct RealRegistry;

impl Registry for RealRegistry {
    fn reader(&self) -> &dyn RegistryOps {
        &WindowsRegistry
    }

    fn set_display_version(&self, hive: Hive, version: &str) -> Result<(), String> {
        WindowsRegistry.set_display_version(hive, version)
    }
}
