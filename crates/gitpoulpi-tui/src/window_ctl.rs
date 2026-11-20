//! The terminal window GitPoulpi runs in, seen from outside: grouping it
//! under the launcher's taskbar icon (Windows) and bringing it to the front
//! when a launcher click finds GitPoulpi already open (`gitpoulpi launch`).
//!
//! A terminal-hosted app has no window of its own, so the taskbar shows the
//! terminal's icon, and Windows has no "running" dot under the pinned
//! GitPoulpi icon. The fix is the AppUserModelID: our shortcuts carry
//! `terminals::aumid_for`, and the TUI finds its terminal's top-level window and
//! gives it the same ID (and our icon).
//!
//! The window is found, first, through our ancestor processes: the terminal
//! that runs us is one of them (a process snapshot, [`ancestor_chain`]), and
//! its visible, unowned top-level window is ours ([`choose_window`]; a
//! process with several windows, WezTerm sharing one gui process or a Windows
//! Terminal with many windows, gives the one in the foreground, which a fresh
//! launcher start is). Only when that finds nothing (a console host that is
//! not our ancestor, a terminal multiplexer whose gui process is not our
//! parent) a unique title is tried: set it (`SetConsoleTitleW`, which the
//! terminal turns into its window title), `EnumWindows` for it, then put
//! GitPoulpi's own title back ([`TUI_TITLE`], never the console's stale one).
//! A terminal window with several tabs moves as a whole: all its tabs group
//! under GitPoulpi's icon while GitPoulpi runs in it. The foreground window
//! is deliberately NOT a last resort: grouping whatever the user switched to
//! meanwhile would be worse than not grouping.
//!
//! Everything that decides something is a pure function (the title token,
//! which methods to try, which Hyprland window is ours, which Windows window
//! is ours); the Windows calls and the processes are thin. The Windows block
//! is never compiled on the Linux development machine: keep it small.

use gitpoulpi_config::open_in::Os;

// Everything below that is text is a log line or a hook's error, not UI.
// i18n-scan: off

/// A title no other window has: our pid and the clock make it unique per
/// call, the prefix says whose it is if it is ever seen.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn title_token(pid: u32, nanos: u128) -> String {
    format!("GitPoulpi-{pid}-{nanos:x}")
}

/// The title GitPoulpi gives its terminal window: set at start and put back
/// after the title search, so the terminal never shows the console's original
/// title (the path of the exe).
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) const TUI_TITLE: &str = "GitPoulpi";

/// A top-level window, as the window enumeration reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) struct TopWindow {
    pub hwnd: isize,
    pub pid: u32,
    pub visible: bool,
    /// Has an owner window (a dialog, a tool window): never a terminal's.
    pub owned: bool,
}

/// The terminal window among `windows`: a visible, unowned one of the nearest
/// ancestor (`ancestors`, nearest first) that has any. When that process has
/// several, the `foreground` one if it is among them (a fresh launcher start
/// is in front), else the first listed.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn choose_window(
    ancestors: &[u32],
    windows: &[TopWindow],
    foreground: Option<isize>,
) -> Option<isize> {
    ancestors.iter().find_map(|pid| {
        let ours: Vec<&TopWindow> = windows
            .iter()
            .filter(|w| w.pid == *pid && w.visible && !w.owned)
            .collect();
        let front = foreground.filter(|f| ours.iter().any(|w| w.hwnd == *f));
        front.or_else(|| ours.first().map(|w| w.hwnd))
    })
}

/// Programs the walk up the process tree never goes into: their windows are
/// the desktop's (the taskbar, the shell), not a terminal's.
const SHELL_PROCESSES: [&str; 6] = [
    "explorer.exe",
    "services.exe",
    "wininit.exe",
    "winlogon.exe",
    "userinit.exe",
    "svchost.exe",
];

/// `pid` and its parents, nearest first, from a process table of
/// `(pid, parent pid, exe name)`. Stops at the root, at a loop (pids are
/// reused), after 32 steps, and before a desktop shell process.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn ancestor_chain(pid: u32, table: &[(u32, u32, String)]) -> Vec<u32> {
    let mut chain = vec![pid];
    while chain.len() < 32 {
        let current = *chain.last().unwrap_or(&pid);
        let Some(parent) = table
            .iter()
            .find(|(p, _, _)| *p == current)
            .map(|(_, parent, _)| *parent)
            .filter(|p| *p != 0 && !chain.contains(p))
        else {
            break;
        };
        let name = table
            .iter()
            .find(|(p, _, _)| *p == parent)
            .map(|(_, _, name)| name.as_str());
        if name.is_some_and(|n| SHELL_PROCESSES.iter().any(|s| n.eq_ignore_ascii_case(s))) {
            break;
        }
        chain.push(parent);
    }
    chain
}

/// One way to bring the terminal window forward.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Step {
    /// Windows: `SetForegroundWindow` on the terminal's top-level window.
    Windows,
    /// `wezterm cli activate-pane --pane-id <id>`.
    WezTerm(String),
    /// Hyprland: focus the window that owns one of our ancestor processes.
    Hyprland,
}

/// The methods to try on this machine, from `os` and the environment (`var`
/// answers a variable, `None` when unset or empty). A terminal running here
/// can match several (WezTerm on Hyprland): each is tried, the first that
/// works counts.
pub(crate) fn raise_plan(os: Os, var: &dyn Fn(&str) -> Option<String>) -> Vec<Step> {
    match os {
        Os::Windows => vec![Step::Windows],
        Os::Macos => Vec::new(),
        Os::Linux => {
            let mut steps = Vec::new();
            if let Some(pane) = var("WEZTERM_PANE") {
                steps.push(Step::WezTerm(pane));
            }
            if var("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
                steps.push(Step::Hyprland);
            }
            steps
        }
    }
}

/// The parent pid in the text of `/proc/<pid>/stat`: the field after the
/// state, found after the *last* `)` because the command name may hold both
/// spaces and parentheses.
#[cfg_attr(windows, allow(dead_code))]
pub(crate) fn parent_pid(stat: &str) -> Option<u32> {
    let after = &stat[stat.rfind(')')? + 1..];
    after.split_whitespace().nth(1)?.parse().ok()
}

/// The address of the Hyprland window to focus: `clients_json` is `hyprctl
/// clients -j`, `ancestors` our process and its parents, nearest first. The
/// window owned by the nearest ancestor wins (the terminal emulator itself).
#[cfg_attr(windows, allow(dead_code))]
pub(crate) fn hyprland_window(clients_json: &str, ancestors: &[u32]) -> Option<String> {
    let clients: serde_json::Value = serde_json::from_str(clients_json).ok()?;
    let clients = clients.as_array()?;
    ancestors.iter().find_map(|pid| {
        clients.iter().find_map(|c| {
            (c.get("pid")?.as_u64()? == u64::from(*pid))
                .then(|| c.get("address")?.as_str().map(str::to_string))
                .flatten()
        })
    })
}

/// Tries each method of `plan` through `run` (which says whether it worked)
/// and describes the first that did, else why none did.
#[cfg_attr(windows, allow(dead_code))]
pub(crate) fn raise_with(
    plan: &[Step],
    run: &mut dyn FnMut(&Step) -> Result<(), String>,
) -> Result<String, String> {
    if plan.is_empty() {
        return Err("no way to bring this terminal window forward here".to_string());
    }
    let mut last = String::new();
    for step in plan {
        match run(step) {
            Ok(()) => return Ok(format!("{step:?}")),
            Err(e) => last = e,
        }
    }
    Err(last)
}

/// Brings this window's terminal to the front, as well as this machine
/// lets us. Blocking (it runs programs): call it from a thread, never the
/// render loop. `Ok`: what worked; `Err`: why nothing did.
pub(crate) fn raise() -> Result<String, String> {
    // A test hook (and an escape hatch for a window manager we do not know):
    // run this program instead of the built-in methods.
    if let Some(hook) = std::env::var_os("GITPOULPI_RAISE_HOOK").filter(|h| !h.is_empty()) {
        return run_raise_hook(&hook);
    }
    let var = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
    let plan = raise_plan(Os::current(), &var);
    raise_with(&plan, &mut |step| run_step(step))
}

/// Runs the raise hook with nothing tied to this terminal: the TUI owns it,
/// and a line the hook writes to an inherited stderr would land on the
/// screen. Its stderr is kept for the error instead.
fn run_raise_hook(hook: &std::ffi::OsStr) -> Result<String, String> {
    let out = std::process::Command::new(hook)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        return Ok("hook".to_string());
    }
    let said = String::from_utf8_lossy(&out.stderr);
    let said = said.trim();
    Err(if said.is_empty() {
        format!("the raise hook exited with {}", out.status)
    } else {
        format!("the raise hook exited with {}: {said}", out.status)
    })
}

#[cfg(not(windows))]
fn run_step(step: &Step) -> Result<(), String> {
    match step {
        Step::Windows => Err("not Windows".into()),
        Step::WezTerm(pane) => succeeded(
            std::process::Command::new("wezterm")
                .args(["cli", "activate-pane", "--pane-id", pane])
                .output(),
        ),
        Step::Hyprland => {
            let clients = std::process::Command::new("hyprctl")
                .args(["clients", "-j"])
                .output()
                .map_err(|e| format!("hyprctl: {e}"))?;
            let mut ancestors = vec![std::process::id()];
            while let Some(parent) = ancestors
                .last()
                .and_then(|p| std::fs::read_to_string(format!("/proc/{p}/stat")).ok())
                .and_then(|s| parent_pid(&s))
                .filter(|p| *p > 1 && ancestors.len() < 32)
            {
                ancestors.push(parent);
            }
            let address = hyprland_window(&String::from_utf8_lossy(&clients.stdout), &ancestors)
                .ok_or("Hyprland has no window for this terminal")?;
            succeeded(
                std::process::Command::new("hyprctl")
                    .args(["dispatch", "focuswindow", &format!("address:{address}")])
                    .output(),
            )
        }
    }
}

#[cfg(not(windows))]
fn succeeded(out: std::io::Result<std::process::Output>) -> Result<(), String> {
    match out {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => Err(String::from_utf8_lossy(&o.stderr).trim().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(windows)]
fn run_step(step: &Step) -> Result<(), String> {
    match step {
        Step::Windows => win::raise(),
        _ => Err("not on this system".into()),
    }
}

/// Starts grouping this terminal window under the launcher's icon, on a
/// thread (the terminal needs a moment to apply the title). Windows only,
/// and only for a start that came from our shortcut or a double-click.
pub(crate) fn group_in_background(launcher: bool) {
    #[cfg(windows)]
    if launcher {
        let _ = std::thread::Builder::new()
            .name("gitpoulpi-group".into())
            .spawn(win::group);
    }
    #[cfg(not(windows))]
    let _ = launcher;
}

/// The logo colour changed: puts the new icon on the terminal window we
/// grouped, if we did. Blocking (a file write): call it from a thread.
pub(crate) fn refresh_window_icon() {
    #[cfg(windows)]
    win::refresh_icon();
}

#[cfg(windows)]
mod win {
    //! Hand-declared FFI for the few calls (no `windows` crate in this
    //! crate's dependencies). Never compiled outside Windows.
    // The FFI structs mirror the C layouts: fields the code never reads are
    // still part of the layout.
    #![allow(dead_code)]
    use super::{ancestor_chain, choose_window, title_token, TopWindow, TUI_TITLE};
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicIsize, Ordering};

    type Hwnd = isize;

    #[repr(C)]
    struct Guid {
        data1: u32,
        data2: u16,
        data3: u16,
        data4: [u8; 8],
    }

    #[repr(C)]
    struct PropKey {
        fmtid: Guid,
        pid: u32,
    }

    /// `PROPVARIANT` with `vt` and the pointer arm of its union; the union is
    /// two pointers wide (`BRECORD`), which the second word keeps.
    #[repr(C)]
    struct PropVariant {
        vt: u16,
        reserved: [u16; 3],
        value: *mut u16,
        pad: usize,
    }

    #[repr(C)]
    struct PropertyStoreVtbl {
        query_interface: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(*mut c_void) -> u32,
        get_count: usize,
        get_at: usize,
        get_value: usize,
        set_value:
            unsafe extern "system" fn(*mut c_void, *const PropKey, *const PropVariant) -> i32,
        commit: unsafe extern "system" fn(*mut c_void) -> i32,
    }

    #[repr(C)]
    struct FlashInfo {
        size: u32,
        hwnd: Hwnd,
        flags: u32,
        count: u32,
        timeout: u32,
    }

    /// `PROCESSENTRY32W` (tlhelp32.h): `ULONG_PTR` is `usize`, `LONG` `i32`;
    /// `repr(C)` inserts the padding the compiler's layout has too.
    #[repr(C)]
    struct ProcessEntry {
        size: u32,
        usage: u32,
        process_id: u32,
        default_heap_id: usize,
        module_id: u32,
        threads: u32,
        parent_process_id: u32,
        pri_class_base: i32,
        flags: u32,
        exe_file: [u16; 260],
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn SetConsoleTitleW(title: *const u16) -> i32;
        fn CreateToolhelp32Snapshot(flags: u32, process_id: u32) -> isize;
        fn Process32FirstW(snapshot: isize, entry: *mut ProcessEntry) -> i32;
        fn Process32NextW(snapshot: isize, entry: *mut ProcessEntry) -> i32;
        fn CloseHandle(handle: isize) -> i32;
    }

    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(
            callback: unsafe extern "system" fn(Hwnd, isize) -> i32,
            lparam: isize,
        ) -> i32;
        fn GetWindowTextW(hwnd: Hwnd, buffer: *mut u16, size: i32) -> i32;
        fn GetWindowThreadProcessId(hwnd: Hwnd, process_id: *mut u32) -> u32;
        fn IsWindowVisible(hwnd: Hwnd) -> i32;
        fn IsWindow(hwnd: Hwnd) -> i32;
        fn GetWindow(hwnd: Hwnd, command: u32) -> Hwnd;
        fn GetForegroundWindow() -> Hwnd;
        fn IsIconic(hwnd: Hwnd) -> i32;
        fn ShowWindow(hwnd: Hwnd, command: i32) -> i32;
        fn SetForegroundWindow(hwnd: Hwnd) -> i32;
        fn FlashWindowEx(info: *const FlashInfo) -> i32;
        fn SendMessageW(hwnd: Hwnd, message: u32, wparam: usize, lparam: isize) -> isize;
        fn LoadImageW(
            instance: isize,
            name: *const u16,
            kind: u32,
            width: i32,
            height: i32,
            flags: u32,
        ) -> isize;
    }

    #[link(name = "shell32")]
    extern "system" {
        fn SHGetPropertyStoreForWindow(hwnd: Hwnd, riid: *const Guid, out: *mut *mut c_void)
            -> i32;
    }

    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(reserved: *mut c_void, flags: u32) -> i32;
        fn CoUninitialize();
    }

    const IID_PROPERTY_STORE: Guid = Guid {
        data1: 0x886d_8eeb,
        data2: 0x8cf2,
        data3: 0x4446,
        data4: [0x8d, 0x02, 0xcd, 0xba, 0x1d, 0xbd, 0xcf, 0x99],
    };
    /// PKEY_AppUserModel_ID.
    const KEY_APP_USER_MODEL_ID: PropKey = PropKey {
        fmtid: Guid {
            data1: 0x9f4c_2855,
            data2: 0x9f79,
            data3: 0x4b39,
            data4: [0xa8, 0xd0, 0xe1, 0xd4, 0x2d, 0xe1, 0xd5, 0xf3],
        },
        pid: 5,
    };
    const VT_LPWSTR: u16 = 31;
    /// `TH32CS_SNAPPROCESS`.
    const TH32CS_SNAPPROCESS: u32 = 0x0000_0002;
    /// `GW_OWNER`.
    const GW_OWNER: u32 = 4;
    const SW_RESTORE: i32 = 9;
    const WM_SETICON: u32 = 0x0080;
    const ICON_SMALL: usize = 0;
    const ICON_BIG: usize = 1;
    const IMAGE_ICON: u32 = 1;
    const LR_LOADFROMFILE: u32 = 0x0010;
    /// `FLASHW_ALL | FLASHW_TIMERNOFG`: flash until the window is in front.
    const FLASH_UNTIL_FOREGROUND: u32 = 0x0003 | 0x000c;

    /// The terminal's top-level window, once found; 0 before.
    static TERMINAL: AtomicIsize = AtomicIsize::new(0);

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn set_console_title(title: &str) {
        let w = wide(title);
        // SAFETY: `w` is NUL-terminated and outlives the call.
        unsafe {
            SetConsoleTitleW(w.as_ptr());
        }
    }

    struct Search {
        token: Vec<u16>,
        found: Hwnd,
    }

    unsafe extern "system" fn visit(hwnd: Hwnd, lparam: isize) -> i32 {
        // SAFETY: `lparam` is the `&mut Search` `find_by_title_token` passed.
        let search = unsafe { &mut *(lparam as *mut Search) };
        let mut buf = [0u16; 512];
        // SAFETY: the buffer is valid for the length passed.
        let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
        if n > 0 {
            let title = &buf[..n as usize];
            if title.windows(search.token.len()).any(|w| w == search.token) {
                search.found = hwnd;
                return 0; // stop
            }
        }
        1
    }

    fn find_by_title_token(token: &str) -> Option<Hwnd> {
        let mut search = Search {
            token: token.encode_utf16().collect(),
            found: 0,
        };
        // SAFETY: `search` outlives the call; the callback only reads it back.
        unsafe {
            EnumWindows(visit, &mut search as *mut Search as isize);
        }
        (search.found != 0).then_some(search.found)
    }

    /// How the terminal window was found, for the log.
    #[derive(Clone, Copy)]
    enum Via {
        Ancestors,
        Title,
        Cached,
    }

    impl Via {
        fn as_str(self) -> &'static str {
            match self {
                Via::Ancestors => "ancestors",
                Via::Title => "title",
                Via::Cached => "cache",
            }
        }
    }

    fn window_pid(hwnd: Hwnd) -> u32 {
        let mut pid = 0u32;
        // SAFETY: `pid` outlives the call.
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut pid);
        }
        pid
    }

    unsafe extern "system" fn collect(hwnd: Hwnd, lparam: isize) -> i32 {
        // SAFETY: `lparam` is the `&mut Vec<TopWindow>` `top_windows` passed.
        let all = unsafe { &mut *(lparam as *mut Vec<TopWindow>) };
        // SAFETY: plain handle arguments.
        let (visible, owner) = unsafe { (IsWindowVisible(hwnd) != 0, GetWindow(hwnd, GW_OWNER)) };
        all.push(TopWindow {
            hwnd,
            pid: window_pid(hwnd),
            visible,
            owned: owner != 0,
        });
        1
    }

    fn top_windows() -> Vec<TopWindow> {
        let mut all = Vec::new();
        // SAFETY: `all` outlives the call; the callback only pushes to it.
        unsafe {
            EnumWindows(collect, &mut all as *mut Vec<TopWindow> as isize);
        }
        all
    }

    /// `(pid, parent pid, exe name)` of every process.
    fn process_table() -> Vec<(u32, u32, String)> {
        let mut table = Vec::new();
        // SAFETY: the entry is zeroed with `size` set as the API requires, the
        // snapshot handle is closed, the buffers outlive each call.
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == -1 {
                return table;
            }
            let mut entry: ProcessEntry = std::mem::zeroed();
            entry.size = std::mem::size_of::<ProcessEntry>() as u32;
            let mut more = Process32FirstW(snapshot, &mut entry) != 0;
            while more {
                let len = entry.exe_file.iter().position(|c| *c == 0).unwrap_or(260);
                table.push((
                    entry.process_id,
                    entry.parent_process_id,
                    String::from_utf16_lossy(&entry.exe_file[..len]),
                ));
                more = Process32NextW(snapshot, &mut entry) != 0;
            }
            CloseHandle(snapshot);
        }
        table
    }

    /// The terminal window of our nearest ancestor process that has one.
    fn find_by_ancestors() -> Option<Hwnd> {
        let chain = ancestor_chain(std::process::id(), &process_table());
        // SAFETY: no arguments.
        let foreground = unsafe { GetForegroundWindow() };
        let found = choose_window(
            &chain,
            &top_windows(),
            (foreground != 0).then_some(foreground),
        );
        if found.is_none() {
            tracing::info!("window grouping: no ancestor owns a visible window (chain {chain:?})");
        }
        found
    }

    /// Finds the terminal window by giving it a unique title for a moment,
    /// then puts GitPoulpi's own title back. Polls for up to three seconds:
    /// the terminal applies the title asynchronously.
    fn find_by_title() -> Option<Hwnd> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let token = title_token(std::process::id(), nanos);
        set_console_title(&token);
        let mut found = None;
        for _ in 0..60 {
            found = find_by_title_token(&token);
            if found.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        // Never the title the console started with: that is the exe's path.
        set_console_title(TUI_TITLE);
        found
    }

    /// The terminal's top-level window: the cached one, else through our
    /// ancestors (polled for up to three seconds: a fresh terminal may not
    /// have shown its window yet; `patient`), else by title.
    fn find_terminal_window(patient: bool) -> Option<(Hwnd, Via)> {
        let cached = TERMINAL.load(Ordering::SeqCst);
        // SAFETY: plain handle argument.
        if cached != 0 && unsafe { IsWindow(cached) } != 0 {
            return Some((cached, Via::Cached));
        }
        let mut found = None;
        let attempts = if patient { 12 } else { 1 };
        for attempt in 0..attempts {
            if let Some(h) = find_by_ancestors() {
                found = Some((h, Via::Ancestors));
                break;
            }
            if attempt + 1 < attempts {
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        }
        let found = found.or_else(|| find_by_title().map(|h| (h, Via::Title)));
        if let Some((h, _)) = found {
            TERMINAL.store(h, Ordering::SeqCst);
        }
        found
    }

    /// Sets the AppUserModelID of `hwnd` through its property store; the
    /// HRESULTs of the two steps on success.
    fn set_app_id(hwnd: Hwnd, id: &str) -> Result<(i32, i32), String> {
        let mut text = wide(id);
        // SAFETY: COM calls on this thread; every pointer outlives its call,
        // the store is released, and COM is uninitialized only if we
        // initialized it (a failure other than "already initialized" is
        // not paired).
        unsafe {
            let hr = CoInitializeEx(std::ptr::null_mut(), 0x2); // apartment threaded
            let initialized = hr >= 0;
            let mut store: *mut c_void = std::ptr::null_mut();
            let hr = SHGetPropertyStoreForWindow(hwnd, &IID_PROPERTY_STORE, &mut store);
            let result = if hr < 0 || store.is_null() {
                Err(format!("SHGetPropertyStoreForWindow failed (hr {hr:#x})"))
            } else {
                let vtbl = *(store as *mut *const PropertyStoreVtbl);
                let value = PropVariant {
                    vt: VT_LPWSTR,
                    reserved: [0; 3],
                    value: text.as_mut_ptr(),
                    pad: 0,
                };
                let set = ((*vtbl).set_value)(store, &KEY_APP_USER_MODEL_ID, &value);
                let committed = if set >= 0 {
                    ((*vtbl).commit)(store)
                } else {
                    set
                };
                ((*vtbl).release)(store);
                if committed >= 0 {
                    Ok((set, committed))
                } else {
                    Err(format!(
                        "setting the AppUserModelID failed (SetValue hr {set:#x}, Commit hr {committed:#x})"
                    ))
                }
            };
            if initialized {
                CoUninitialize();
            }
            result
        }
    }

    /// The `.ico` of the colour the title bar wears now (written when it is
    /// not on disk yet: not everyone has shortcuts).
    fn current_ico() -> Option<std::path::PathBuf> {
        let home = gitpoulpi_config::home_dir().unwrap_or_default();
        let dirs = gitpoulpi_config::desktop::Dirs::discover(
            gitpoulpi_config::open_in::Os::current(),
            &|k| std::env::var(k).ok(),
            &home,
        );
        let slug = crate::logo_colors::ways()[crate::about::way_index()]
            .slug
            .clone();
        let ctx = crate::launcher::this_ctx();
        gitpoulpi_config::desktop::ensure_ico_for(&dirs, &crate::launcher::icon_set(&slug), &ctx)
    }

    /// Our icon (the current colour's `.ico`) on the window, so the taskbar
    /// button and the title bar wear it instead of the terminal's.
    fn set_icons(hwnd: Hwnd) {
        let Some(ico) = current_ico() else {
            tracing::info!("window grouping: no icon file to put on the window");
            return;
        };
        let path = wide(&ico.to_string_lossy());
        let mut loaded = [0isize; 2];
        for (i, (which, size)) in [(ICON_SMALL, 16), (ICON_BIG, 32)].into_iter().enumerate() {
            // SAFETY: `path` is NUL-terminated; the icon handle is passed on
            // to the window, which keeps it for its lifetime.
            unsafe {
                let icon = LoadImageW(0, path.as_ptr(), IMAGE_ICON, size, size, LR_LOADFROMFILE);
                loaded[i] = icon;
                if icon != 0 {
                    SendMessageW(hwnd, WM_SETICON, which, icon);
                }
            }
        }
        tracing::info!(
            "window grouping: window icon set (hwnd {hwnd:#x}, {}, small loaded {}, big loaded {})",
            ico.display(),
            loaded[0] != 0,
            loaded[1] != 0
        );
    }

    /// The icon of the colour chosen in About, on the window already found.
    pub(super) fn refresh_icon() {
        let hwnd = TERMINAL.load(Ordering::SeqCst);
        // SAFETY: plain handle argument.
        if hwnd != 0 && unsafe { IsWindow(hwnd) } != 0 {
            set_icons(hwnd);
        }
    }

    /// The ID and the icon on `hwnd`, logged step by step.
    fn apply(hwnd: Hwnd, round: &str) {
        let pid = window_pid(hwnd);
        let aumid = gitpoulpi_config::desktop::terminals::aumid_for(&crate::launcher::this_ctx());
        match set_app_id(hwnd, &aumid) {
            Ok((set, commit)) => tracing::info!(
                "window grouping: AppUserModelID set ({round}; hwnd {hwnd:#x}, pid {pid}, SetValue hr {set:#x}, Commit hr {commit:#x})"
            ),
            Err(e) => tracing::warn!(
                "window grouping: AppUserModelID not set ({round}; hwnd {hwnd:#x}, pid {pid}): {e}"
            ),
        }
        set_icons(hwnd);
    }

    /// Groups the terminal window under the launcher's icon, and once more
    /// two seconds later: a terminal sometimes resets the window's
    /// properties right after showing it.
    pub(super) fn group() {
        // The terminal shows GitPoulpi's title, not the exe's path.
        set_console_title(TUI_TITLE);
        let Some((hwnd, via)) = find_terminal_window(true) else {
            tracing::info!("window grouping: the terminal window was not found");
            return;
        };
        tracing::info!(
            "window grouping: found terminal window {hwnd:#x} (pid {}) via {}",
            window_pid(hwnd),
            via.as_str()
        );
        apply(hwnd, "first");
        std::thread::sleep(std::time::Duration::from_secs(2));
        // SAFETY: plain handle argument.
        if unsafe { IsWindow(hwnd) } != 0 {
            apply(hwnd, "again after 2 s");
        }
    }

    /// Restores (when minimized) and shows the terminal window; when Windows
    /// refuses the foreground, flashes its taskbar button instead.
    pub(super) fn raise() -> Result<(), String> {
        let (hwnd, _) = find_terminal_window(false).ok_or("the terminal window was not found")?;
        // SAFETY: plain handle and value arguments; `info` outlives the call.
        unsafe {
            if IsIconic(hwnd) != 0 {
                ShowWindow(hwnd, SW_RESTORE);
            }
            if SetForegroundWindow(hwnd) == 0 {
                let info = FlashInfo {
                    size: std::mem::size_of::<FlashInfo>() as u32,
                    hwnd,
                    flags: FLASH_UNTIL_FOREGROUND,
                    count: 0,
                    timeout: 0,
                };
                FlashWindowEx(&info);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |k| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_string())
        }
    }

    #[test]
    fn the_title_token_is_unique_per_process_and_instant_and_names_us() {
        let a = title_token(42, 0x1a2b);
        assert_eq!(a, "GitPoulpi-42-1a2b");
        assert_ne!(a, title_token(43, 0x1a2b));
        assert_ne!(a, title_token(42, 0x1a2c));
        // A token is never a prefix of another one: `contains` is exact.
        assert!(!title_token(4, 1).contains(&title_token(42, 1)));
    }

    const HOOK_NOISE: &str = "NOISE-FROM-THE-RAISE-HOOK";
    const HOOK_CHILD: &str = "GITPOULPI_RAISE_HOOK_TEST_HOOK";

    /// Only does anything inside the child process of
    /// `a_raise_hook_that_writes_to_stderr_does_not_reach_ours`.
    #[test]
    fn child_runs_a_noisy_raise_hook() {
        let Some(hook) = std::env::var_os(HOOK_CHILD) else {
            return;
        };
        let err = run_raise_hook(&hook).expect_err("the hook fails");
        assert!(err.contains(HOOK_NOISE), "the message keeps stderr: {err}");
    }

    /// The TUI owns the terminal while the hook runs: its stderr must be
    /// captured, not inherited (it would be painted over the screen).
    #[cfg(unix)]
    #[test]
    fn a_raise_hook_that_writes_to_stderr_does_not_reach_ours() {
        use std::os::unix::fs::PermissionsExt;
        if std::env::var_os(HOOK_CHILD).is_some() {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let hook = dir.path().join("hook.sh");
        std::fs::write(&hook, format!("#!/bin/sh\necho {HOOK_NOISE} >&2\nexit 1\n")).unwrap();
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
        let out = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "window_ctl::tests::child_runs_a_noisy_raise_hook",
                "--nocapture",
            ])
            .env(HOOK_CHILD, &hook)
            .stdin(std::process::Stdio::null())
            .output()
            .expect("run the child");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "the child failed:\n{stderr}");
        assert!(
            !stderr.contains(HOOK_NOISE),
            "the hook's stderr reached the process's own:\n{stderr}"
        );
    }

    fn win(hwnd: isize, pid: u32, visible: bool, owned: bool) -> TopWindow {
        TopWindow {
            hwnd,
            pid,
            visible,
            owned,
        }
    }

    #[test]
    fn the_nearest_ancestor_with_a_visible_unowned_window_owns_ours() {
        let windows = [
            win(1, 900, true, false),  // a far ancestor's window
            win(2, 200, false, false), // hidden
            win(3, 200, true, true),   // a dialog
            win(4, 200, true, false),  // the terminal
            win(5, 777, true, false),  // a stranger's
        ];
        // Our shell 1000, the terminal 200, a launcher 900.
        assert_eq!(choose_window(&[1000, 200, 900], &windows, None), Some(4));
        assert_eq!(choose_window(&[1000, 900], &windows, None), Some(1));
        assert_eq!(choose_window(&[1000, 555], &windows, None), None);
        assert_eq!(choose_window(&[], &windows, Some(5)), None);
        // A foreground window that is not an ancestor's is never taken.
        assert_eq!(choose_window(&[1000, 200], &windows, Some(5)), Some(4));
    }

    #[test]
    fn a_process_with_several_windows_gives_the_foreground_one() {
        let windows = [
            win(10, 200, true, false),
            win(11, 200, true, false),
            win(12, 200, true, false),
        ];
        assert_eq!(choose_window(&[200], &windows, Some(11)), Some(11));
        assert_eq!(choose_window(&[200], &windows, None), Some(10));
        // The foreground one is hidden or owned: not eligible.
        let windows = [win(10, 200, true, false), win(11, 200, false, false)];
        assert_eq!(choose_window(&[200], &windows, Some(11)), Some(10));
    }

    fn proc(pid: u32, parent: u32, name: &str) -> (u32, u32, String) {
        (pid, parent, name.to_string())
    }

    #[test]
    fn the_ancestor_chain_climbs_to_the_root_but_never_into_the_desktop_shell() {
        let table = [
            proc(1000, 900, "gitpoulpi.exe"),
            proc(900, 800, "pwsh.exe"),
            proc(800, 700, "wezterm-gui.exe"),
            proc(700, 600, "Explorer.EXE"),
            proc(600, 0, "wininit.exe"),
        ];
        assert_eq!(ancestor_chain(1000, &table), [1000, 900, 800]);
        assert_eq!(ancestor_chain(600, &table), [600]);
        assert_eq!(ancestor_chain(42, &table), [42], "unknown pid: just itself");
    }

    #[test]
    fn the_ancestor_chain_survives_a_reused_pid_that_loops() {
        let table = [
            proc(1, 2, "a.exe"),
            proc(2, 3, "b.exe"),
            proc(3, 1, "c.exe"),
        ];
        assert_eq!(ancestor_chain(1, &table), [1, 2, 3]);
    }

    #[test]
    fn which_methods_are_tried_follows_the_os_and_the_terminal() {
        assert_eq!(raise_plan(Os::Windows, &env(&[])), [Step::Windows]);
        assert!(raise_plan(Os::Macos, &env(&[("WEZTERM_PANE", "3")])).is_empty());
        assert!(raise_plan(Os::Linux, &env(&[])).is_empty());
        assert_eq!(
            raise_plan(Os::Linux, &env(&[("WEZTERM_PANE", "3")])),
            [Step::WezTerm("3".into())]
        );
        // WezTerm on Hyprland: the pane is activated, then the window focused.
        assert_eq!(
            raise_plan(
                Os::Linux,
                &env(&[("WEZTERM_PANE", "3"), ("HYPRLAND_INSTANCE_SIGNATURE", "x")])
            ),
            [Step::WezTerm("3".into()), Step::Hyprland]
        );
    }

    #[test]
    fn the_parent_pid_survives_a_command_name_with_spaces_and_parentheses() {
        assert_eq!(parent_pid("1234 (bash) S 1200 1234 1234 0"), Some(1200));
        assert_eq!(parent_pid("77 (my (weird) name) R 55 77 77 0 -1"), Some(55));
        assert_eq!(parent_pid("garbage"), None);
    }

    #[test]
    fn hyprland_focuses_the_window_of_the_nearest_ancestor() {
        let clients = r#"[
            {"address":"0xaaa","pid":100,"class":"kitty"},
            {"address":"0xbbb","pid":200,"class":"foot"},
            {"address":"0xccc","pid":300,"class":"other"}
        ]"#;
        // Our shell (900) and its terminal (200) and a login manager (100).
        assert_eq!(
            hyprland_window(clients, &[1000, 900, 200, 100]).as_deref(),
            Some("0xbbb")
        );
        assert_eq!(hyprland_window(clients, &[1000, 900]), None);
        assert_eq!(hyprland_window("not json", &[200]), None);
    }

    #[test]
    fn the_first_method_that_works_counts_and_none_says_why() {
        let plan = [Step::WezTerm("1".into()), Step::Hyprland];
        let worked = raise_with(&plan, &mut |s| match s {
            Step::WezTerm(_) => Err("mux not running".into()),
            _ => Ok(()),
        });
        assert_eq!(worked.unwrap(), "Hyprland");
        let failed = raise_with(&plan, &mut |_| Err("nope".into()));
        assert_eq!(failed.unwrap_err(), "nope");
        let empty = raise_with(&[], &mut |_| Ok(()));
        assert!(empty.unwrap_err().contains("no way"));
    }
}
