//! Windows: `.lnk` shortcuts in the Start menu and on the Desktop.
//!
//! The shortcut is written by PowerShell's `WScript.Shell` (`CreateShortcut`),
//! run with `-EncodedCommand` so no path ever needs shell quoting: it is
//! always present on Windows, handles Unicode paths and costs no dependency
//! (`mslnk` would add a crate for a feature used once). The spawn is slow
//! (hundreds of ms), so callers run it off the UI thread.
//!
//! State is read back without PowerShell: a `.lnk` stores its strings as
//! plain bytes (ANSI in the link info, UTF-16 in the string data), so the
//! marker in the description and the target paths are searched for directly.
//!
//! Taskbar: Windows has no supported API to pin, but a pin the user made is a
//! copy of the shortcut in `User Pinned\TaskBar`, with the `IconLocation` it
//! had. The taskbar caches an icon by its path, so rewriting the same `.ico`
//! never refreshes a pinned button: the file name carries a hash of the icon
//! (`gitpoulpi-<hash>.ico`), a new colour is a new path, and every shortcut of
//! ours, pinned copies included, is re-pointed at it (`repoint_shortcuts`)
//! before the older files are deleted (`cleanup_icons`).

use std::path::{Path, PathBuf};

use super::{terminals, Dirs, IconSet, Place, Request, State, Status, MARKER};
use crate::open_in::Os;

pub(super) const FILE: &str = "GitPoulpi.lnk";
/// The `.ico` before its name carried a hash; still ours, still cleaned up.
const LEGACY_ICON_FILE: &str = "gitpoulpi.ico";
const ICON_PREFIX: &str = "gitpoulpi-";
/// Hex digits of the hash in the file name (48 bits of the FNV-1a 64).
const ICON_HASH_DIGITS: usize = 12;

/// One argument of a Windows command line (`CommandLineToArgvW` rules).
pub fn quote_arg(arg: &str) -> String {
    if !arg.is_empty() && !arg.contains([' ', '\t', '"']) {
        return arg.to_string();
    }
    let mut out = String::from("\"");
    let mut backslashes = 0;
    for c in arg.chars() {
        match c {
            '\\' => backslashes += 1,
            '"' => {
                out.push_str(&"\\".repeat(backslashes * 2 + 1));
                out.push('"');
                backslashes = 0;
            }
            _ => {
                out.push_str(&"\\".repeat(backslashes));
                out.push(c);
                backslashes = 0;
            }
        }
    }
    out.push_str(&"\\".repeat(backslashes * 2));
    out.push('"');
    out
}

/// `(TargetPath, Arguments)` of the shortcut. With the console-less
/// `gitpoulpi-launch.exe` beside the exe it is that helper, which shows a
/// GitPoulpi window that is already open or else runs the rest of the line
/// (`-- <program> <args>`, [`inner`]); without it, [`inner`] itself.
pub(super) fn target(req: &Request) -> Result<(String, String), String> {
    let (program, args) = inner(req)?;
    Ok(match req.launch_exe {
        Some(launch) => (
            launch.to_string_lossy().to_string(),
            format!("-- {} {args}", quote_arg(&program))
                .trim_end()
                .to_string(),
        ),
        None => (program, args),
    })
}

/// What a click ends up running: the exe itself for the plain console, else
/// the terminal's program with the exe as its command.
pub(super) fn inner(req: &Request) -> Result<(String, String), String> {
    let exe = req.exe.to_string_lossy().to_string();
    if req.terminal == "console" {
        return Ok((exe, terminals::LAUNCHER_FLAG.to_string()));
    }
    let argv = terminals::argv(Os::Windows, req.terminal, &exe)
        .ok_or_else(|| format!("unknown terminal {}", req.terminal))?;
    let program = req
        .terminal_exe
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| argv[0].clone());
    let args = argv[1..]
        .iter()
        .map(|a| quote_arg(a))
        .collect::<Vec<_>>()
        .join(" ");
    Ok((program, args))
}

fn ps_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

/// The PowerShell script that writes one shortcut.
pub(super) fn script(lnk: &Path, target: &str, args: &str, workdir: &Path, icon: &Path) -> String {
    format!(
        "$ErrorActionPreference = 'Stop'\n\
         $s = (New-Object -ComObject WScript.Shell).CreateShortcut({lnk})\n\
         $s.TargetPath = {target}\n\
         $s.Arguments = {args}\n\
         $s.WorkingDirectory = {dir}\n\
         $s.IconLocation = {icon}\n\
         $s.Description = {desc}\n\
         $s.Save()\n{aumid}{notify}",
        lnk = ps_quote(&lnk.to_string_lossy()),
        target = ps_quote(target),
        args = ps_quote(args),
        dir = ps_quote(&workdir.to_string_lossy()),
        icon = ps_quote(&format!("{},0", icon.to_string_lossy())),
        desc = ps_quote(&format!("GitPoulpi {MARKER}")),
        aumid = aumid_script(&[lnk.to_path_buf()]),
        notify = notify_script(&[lnk.to_path_buf()], false),
    )
}

/// The PowerShell that gives each shortcut the AppUserModelID of
/// [`terminals::AUMID`] (`System.AppUserModel.ID`): `WScript.Shell` cannot
/// write it, the shell link's property store can. The terminal window GitPoulpi
/// runs in gets the same ID at run time (`gitpoulpi-tui`'s `window_group`),
/// which is what makes the taskbar show it under the pinned icon with the
/// "running" dot. Best effort: a failure here leaves a working shortcut that
/// just does not group.
pub(super) fn aumid_script(lnks: &[PathBuf]) -> String {
    if lnks.is_empty() {
        return String::new();
    }
    let mut s = String::from(AUMID_TYPE);
    for lnk in lnks {
        s.push_str(&format!(
            "try {{ [GpAumid]::Set({lnk}, {id}) }} catch {{ }}\n",
            lnk = ps_quote(&lnk.to_string_lossy()),
            id = ps_quote(terminals::AUMID),
        ));
    }
    s
}

/// The C# helper (`Add-Type`): load the `.lnk` through `IPersistFile`, set
/// PKEY_AppUserModel_ID (fmtid 9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3, pid 5)
/// as a VT_LPWSTR, commit and save.
const AUMID_TYPE: &str = r#"try {
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class GpAumid {
  [StructLayout(LayoutKind.Sequential, Pack = 4)]
  struct PROPERTYKEY { public Guid fmtid; public uint pid; }
  [StructLayout(LayoutKind.Explicit, Size = 24)]
  struct PROPVARIANT { [FieldOffset(0)] public ushort vt; [FieldOffset(8)] public IntPtr p; }
  [ComImport, Guid("886D8EEB-8CF2-4446-8D02-CDBA1DBDCF99"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
  interface IPropertyStore {
    [PreserveSig] int GetCount(out uint c);
    [PreserveSig] int GetAt(uint i, out PROPERTYKEY k);
    [PreserveSig] int GetValue(ref PROPERTYKEY k, out PROPVARIANT v);
    [PreserveSig] int SetValue(ref PROPERTYKEY k, ref PROPVARIANT v);
    [PreserveSig] int Commit();
  }
  [ComImport, Guid("0000010b-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
  interface IPersistFile {
    void GetClassID(out Guid g);
    [PreserveSig] int IsDirty();
    void Load([MarshalAs(UnmanagedType.LPWStr)] string f, uint mode);
    void Save([MarshalAs(UnmanagedType.LPWStr)] string f, [MarshalAs(UnmanagedType.Bool)] bool remember);
    void SaveCompleted([MarshalAs(UnmanagedType.LPWStr)] string f);
    void GetCurFile([MarshalAs(UnmanagedType.LPWStr)] out string f);
  }
  [ComImport, Guid("00021401-0000-0000-C000-000000000046")]
  class ShellLink { }
  public static void Set(string path, string id) {
    object link = new ShellLink();
    ((IPersistFile)link).Load(path, 2);
    var store = (IPropertyStore)link;
    var key = new PROPERTYKEY { fmtid = new Guid("9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3"), pid = 5 };
    var value = new PROPVARIANT { vt = 31, p = Marshal.StringToCoTaskMemUni(id) };
    store.SetValue(ref key, ref value);
    store.Commit();
    ((IPersistFile)link).Save(path, true);
    Marshal.FreeCoTaskMem(value.p);
  }
}
'@
} catch { }
"#;

/// `-EncodedCommand` payload: base64 of the script's UTF-16LE bytes.
pub(super) fn encode_command(script: &str) -> String {
    const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            B64[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Whether `bytes` holds `needle` as ANSI or UTF-16LE text.
pub(super) fn contains_text(bytes: &[u8], needle: &str) -> bool {
    let find = |hay: &[u8], pat: &[u8]| {
        !pat.is_empty() && hay.windows(pat.len()).any(|w| w.eq_ignore_ascii_case(pat))
    };
    let utf16: Vec<u8> = needle.encode_utf16().flat_map(u16::to_le_bytes).collect();
    find(bytes, needle.as_bytes()) || find(bytes, &utf16)
}

fn lnk_path(place: Place, dirs: &Dirs) -> Result<PathBuf, String> {
    match place {
        Place::StartMenu => dirs.start_menu.as_ref(),
        Place::Desktop => dirs.desktop.as_ref(),
        Place::Launcher => return Err("Windows has the Start menu, not a launcher".into()),
    }
    .map(|d| d.join(FILE))
    .ok_or_else(|| "that folder was not found".to_string())
}

/// `gitpoulpi-<hash of the .ico bytes>.ico`: a new colour is a new name.
/// FNV-1a: stable across builds, unlike `DefaultHasher`.
pub(super) fn ico_name(icons: &IconSet) -> String {
    let hash = icons.ico().iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    format!(
        "{ICON_PREFIX}{:0width$x}.ico",
        hash & ((1u64 << (4 * ICON_HASH_DIGITS)) - 1),
        width = ICON_HASH_DIGITS
    )
}

pub(super) fn ico_path(dirs: &Dirs, icons: &IconSet) -> PathBuf {
    dirs.assets.join(ico_name(icons))
}

/// Whether `file` is an icon we wrote: `gitpoulpi.ico` (before the names
/// carried a hash) or `gitpoulpi-<12 hex>.ico`. Nothing else in the folder is
/// ever touched.
pub(super) fn is_our_ico(file: &str) -> bool {
    if file == LEGACY_ICON_FILE {
        return true;
    }
    file.strip_prefix(ICON_PREFIX)
        .and_then(|r| r.strip_suffix(".ico"))
        .is_some_and(|h| {
            h.len() == ICON_HASH_DIGITS && h.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        })
}

fn our_icos(dirs: &Dirs) -> Vec<PathBuf> {
    std::fs::read_dir(&dirs.assets)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| is_our_ico(&e.file_name().to_string_lossy()))
        .map(|e| e.path())
        .collect()
}

/// Every shortcut that may name one of our icons: the Start menu and Desktop
/// entries and ALL shortcuts pinned to the taskbar (a pin whose marker
/// Windows stripped still points at our file, and must not lose it).
fn icon_holders(dirs: &Dirs) -> Vec<PathBuf> {
    let mut all: Vec<PathBuf> = [dirs.start_menu.as_ref(), dirs.desktop.as_ref()]
        .into_iter()
        .flatten()
        .map(|d| d.join(FILE))
        .collect();
    if let Some(pins) = dirs.taskbar_pinned.as_ref() {
        all.extend(
            std::fs::read_dir(pins)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk"))),
        );
    }
    all
}

/// Deletes the icon files of ours that are not `keep` and that no shortcut
/// points at any more.
pub(super) fn cleanup_icons(dirs: &Dirs, keep: &Path) {
    let holders: Vec<Vec<u8>> = icon_holders(dirs)
        .iter()
        .filter_map(|p| std::fs::read(p).ok())
        .collect();
    for ico in our_icos(dirs) {
        let name = ico.to_string_lossy();
        if ico == keep || holders.iter().any(|b| contains_text(b, &name)) {
            continue;
        }
        let _ = std::fs::remove_file(&ico);
    }
}

/// Writes `icons` as our current `.ico` (when missing or different) and says
/// where; the running GitPoulpi puts it on its terminal window.
pub(super) fn ensure_ico(dirs: &Dirs, icons: &IconSet) -> Option<PathBuf> {
    if icons.pngs.is_empty() {
        return None;
    }
    let path = ico_path(dirs, icons);
    let bytes = icons.ico();
    if std::fs::read(&path).ok().as_deref() != Some(bytes.as_slice()) {
        super::write_file(&path, &bytes, false).ok()?;
    }
    Some(path)
}

/// Does `bytes` name the exe? In newest mode the `dist` and `release` copies
/// of one checkout's exe count alike, so starting either build never makes a
/// shortcut read `Outdated`.
fn names_exe(bytes: &[u8], req: &Request) -> bool {
    exe_spellings(req.exe, req.newest)
        .iter()
        .any(|e| contains_text(bytes, e))
}

/// The launch helper counterpart of [`names_exe`]; true when there is none.
fn names_launch(bytes: &[u8], req: &Request) -> bool {
    req.launch_exe.map_or(true, |l| {
        exe_spellings(l, req.newest)
            .iter()
            .any(|e| contains_text(bytes, e))
    })
}

fn exe_spellings(exe: &Path, newest: bool) -> Vec<String> {
    if newest {
        crate::dev_mode::build_variants(exe)
    } else {
        vec![exe.to_path_buf()]
    }
    .iter()
    .map(|p| p.to_string_lossy().into_owned())
    .collect()
}

pub(super) fn status(place: Place, dirs: &Dirs, req: &Request) -> Status {
    let none = Status {
        state: State::NotInstalled,
        icon_stale: false,
    };
    let Ok(path) = lnk_path(place, dirs) else {
        return none;
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return none;
    };
    if !contains_text(&bytes, MARKER) {
        return Status {
            state: State::Foreign,
            icon_stale: false,
        };
    }
    // The flag counts: a shortcut written before `--launcher` existed has
    // the right exe and terminal but starts a "normal" start (Home), so it
    // must read `Outdated` for `sync_shortcuts` to rewrite it.
    // The launch helper and the AppUserModelID count too: a shortcut written
    // before them has the right exe and terminal but starts a new window on
    // every click and groups under the terminal, so it must read `Outdated`.
    let current = inner(req).is_ok_and(|(program, _)| {
        names_exe(&bytes, req)
            && (req.terminal == "console" || contains_text(&bytes, &program))
            && contains_text(&bytes, terminals::LAUNCHER_FLAG)
            && names_launch(&bytes, req)
            && contains_text(&bytes, terminals::AUMID)
    });
    // Stale: the current icon is not on disk, or this shortcut names another.
    let ico = ico_path(dirs, req.icons);
    let icon_stale = !req.icons.pngs.is_empty()
        && (std::fs::read(&ico).ok().as_deref() != Some(req.icons.ico().as_slice())
            || !contains_text(&bytes, &ico.to_string_lossy()));
    Status {
        state: if current {
            State::Installed
        } else {
            State::Outdated
        },
        icon_stale,
    }
}

pub(super) fn write_icon(dirs: &Dirs, req: &Request) -> Result<(), String> {
    if req.icons.pngs.is_empty() {
        return Ok(());
    }
    super::write_file(&ico_path(dirs, req.icons), &req.icons.ico(), false)
}

/// The logo colour changed: writes the new `.ico` under its new name,
/// re-points every shortcut of ours at it (PowerShell; skipped in tests),
/// then deletes the older files nothing points at.
pub(super) fn refresh_icon(dirs: &Dirs, req: &Request) -> Result<(), String> {
    write_icon(dirs, req)?;
    if req.refresh_icon_cache && !req.icons.pngs.is_empty() {
        repoint_shortcuts(dirs, req);
        flush_icon_cache();
    }
    cleanup_icons(dirs, &ico_path(dirs, req.icons));
    Ok(())
}

/// Best effort: asks Explorer to drop its cached icons (`ie4uinit -show`,
/// Windows 10 and later). Never in tests (`refresh_icon_cache` is off there).
pub(super) fn flush_icon_cache() {
    let _ = std::process::Command::new("ie4uinit.exe")
        .arg("-show")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

pub(super) fn install(place: Place, dirs: &Dirs, req: &Request) -> Result<PathBuf, String> {
    let path = lnk_path(place, dirs)?;
    if status(place, dirs, req).state == State::Foreign {
        return Err(format!(
            "{} is not ours; not overwriting it",
            path.display()
        ));
    }
    let (program, args) = target(req)?;
    write_icon(dirs, req)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let workdir = req.exe.parent().unwrap_or(Path::new("."));
    let icon = if req.icons.pngs.is_empty() {
        req.exe.to_path_buf()
    } else {
        ico_path(dirs, req.icons)
    };
    run_powershell(&script(&path, &program, &args, workdir, &icon))?;
    if req.refresh_icon_cache && !req.icons.pngs.is_empty() {
        repoint_shortcuts(dirs, req);
    }
    sync_pinned_args(dirs, req);
    cleanup_icons(dirs, &ico_path(dirs, req.icons));
    Ok(path)
}

/// The shortcuts in `User Pinned\TaskBar` that start GitPoulpi: they carry
/// our marker (the description survives the copy) or the path of our exe (the
/// target, or the argument of a terminal's command line). Sorted.
pub(super) fn pinned_ours(dirs: &Dirs, req: &Request) -> Vec<PathBuf> {
    let Some(dir) = dirs.taskbar_pinned.as_ref() else {
        return Vec::new();
    };
    let exes: Vec<String> = exe_spellings(req.exe, req.newest)
        .into_iter()
        .filter(|e| !e.is_empty())
        .collect();
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk")))
        .filter(|p| {
            std::fs::read(p).is_ok_and(|b| {
                contains_text(&b, MARKER) || exes.iter().any(|e| contains_text(&b, e))
            })
        })
        .collect();
    found.sort();
    found
}

/// The pinned copies whose icon is not our current `.ico` yet.
pub(super) fn pinned_to_repoint(dirs: &Dirs, req: &Request) -> Vec<PathBuf> {
    let ico = ico_path(dirs, req.icons).to_string_lossy().to_string();
    pinned_ours(dirs, req)
        .into_iter()
        .filter(|p| std::fs::read(p).is_ok_and(|b| !contains_text(&b, &ico)))
        .collect()
}

/// Every shortcut of ours (Start menu, Desktop, pinned copies) whose icon is
/// not the current `.ico`.
pub(super) fn shortcuts_to_repoint(dirs: &Dirs, req: &Request) -> Vec<PathBuf> {
    let ico = ico_path(dirs, req.icons).to_string_lossy().to_string();
    let own = [dirs.start_menu.as_ref(), dirs.desktop.as_ref()]
        .into_iter()
        .flatten()
        .map(|d| d.join(FILE))
        .filter(|p| {
            std::fs::read(p).is_ok_and(|b| contains_text(&b, MARKER) && !contains_text(&b, &ico))
        });
    let mut all: Vec<PathBuf> = own.chain(pinned_to_repoint(dirs, req)).collect();
    all.sort();
    all.dedup();
    all
}

/// The script that re-points shortcuts at `icon` and tells Explorer: each
/// touched `.lnk` (`SHCNE_UPDATEITEM`, which is what makes the taskbar read
/// it again) and that icons changed (`SHCNE_ASSOCCHANGED`). Saving through
/// `WScript.Shell` drops the AppUserModelID of a shortcut, so it is written
/// again after.
pub(super) fn pinned_script(lnks: &[PathBuf], icon: &Path) -> String {
    let mut s = String::from("$ErrorActionPreference = 'Stop'\n");
    if !lnks.is_empty() {
        s.push_str("$sh = New-Object -ComObject WScript.Shell\n");
    }
    for lnk in lnks {
        s.push_str(&format!(
            "$p = $sh.CreateShortcut({lnk})\n$p.IconLocation = {icon}\n$p.Save()\n",
            lnk = ps_quote(&lnk.to_string_lossy()),
            icon = ps_quote(&format!("{},0", icon.to_string_lossy())),
        ));
    }
    s.push_str(&aumid_script(lnks));
    s.push_str(&notify_script(lnks, true));
    s
}

/// `Add-Type` of `SHChangeNotify`: the plain form (event, flags, two
/// pointers) and the path form (`SHCNF_PATHW`, the item as a string).
const NOTIFY_DECL: &str = "Add-Type -Namespace Gp -Name Shell -MemberDefinition \
     '[DllImport(\"shell32.dll\")] public static extern void SHChangeNotify(int e, uint f, IntPtr a, IntPtr b); \
     [DllImport(\"shell32.dll\", CharSet = CharSet.Unicode, EntryPoint = \"SHChangeNotify\")] public static extern void SHChangeNotifyPath(int e, uint f, string a, IntPtr b);'\n";
const NOTIFY_ASSOC: &str =
    "[Gp.Shell]::SHChangeNotify(0x08000000, 0, [IntPtr]::Zero, [IntPtr]::Zero)\n";
/// `SHCNE_UPDATEITEM` and `SHCNF_PATHW` (shlobj_core.h).
const SHCNE_UPDATEITEM: u32 = 0x0000_2000;
const SHCNF_PATHW: u32 = 0x0005;

/// The PowerShell that notifies the shell of `lnks` (updated items) and, when
/// `assoc`, that associations (icons) changed. Empty for nothing to say.
fn notify_script(lnks: &[PathBuf], assoc: bool) -> String {
    if lnks.is_empty() && !assoc {
        return String::new();
    }
    let mut s = String::from(NOTIFY_DECL);
    for lnk in lnks {
        s.push_str(&format!(
            "[Gp.Shell]::SHChangeNotifyPath({SHCNE_UPDATEITEM:#x}, {SHCNF_PATHW:#x}, {lnk}, [IntPtr]::Zero)\n",
            lnk = ps_quote(&lnk.to_string_lossy()),
        ));
    }
    if assoc {
        s.push_str(NOTIFY_ASSOC);
    }
    s
}

/// Best effort: re-points our shortcuts (Start menu, Desktop, and the copies
/// pinned to the taskbar) at the current `.ico` and notifies the shell. A pin
/// the user made from the Start menu entry is a copy; if Windows stripped our
/// marker AND the exe path it is not found and the user unpins and pins again
/// (documented in the settings guide).
pub(super) fn repoint_shortcuts(dirs: &Dirs, req: &Request) {
    let lnks = shortcuts_to_repoint(dirs, req);
    let _ = run_powershell(&pinned_script(&lnks, &ico_path(dirs, req.icons)));
}

/// The pinned copies to rewrite, each with whether it carries our marker
/// (it was a copy of our shortcut, so its whole target is ours to rewrite:
/// `true`) or only our exe path identifies it (a shortcut the user made:
/// `false`, only the missing launcher flag is appended, the rest is theirs).
/// A copy of ours is stale when it lacks the flag, the launch helper (when
/// there is one) or the AppUserModelID; a user's copy when it lacks the flag.
pub(super) fn pinned_stale(dirs: &Dirs, req: &Request) -> Vec<(PathBuf, bool)> {
    pinned_ours(dirs, req)
        .into_iter()
        .filter_map(|p| {
            let bytes = std::fs::read(&p).ok()?;
            let owned = contains_text(&bytes, MARKER);
            let flag = contains_text(&bytes, terminals::LAUNCHER_FLAG);
            let launch = names_launch(&bytes, req);
            let current = if owned {
                flag && launch && contains_text(&bytes, terminals::AUMID)
            } else {
                flag
            };
            (!current).then_some((p, owned))
        })
        .collect()
}

/// The script that gives pinned copies the current command line: a copy of
/// our shortcut (`owned`) gets our `TargetPath` and `Arguments`, one the user
/// made only gets the flag appended. Explorer is told afterwards.
pub(super) fn pinned_args_script(items: &[(PathBuf, bool)], program: &str, args: &str) -> String {
    let mut s = String::from("$ErrorActionPreference = 'Stop'\n");
    if !items.is_empty() {
        s.push_str("$sh = New-Object -ComObject WScript.Shell\n");
    }
    for (lnk, owned) in items {
        let lnk = ps_quote(&lnk.to_string_lossy());
        if *owned {
            s.push_str(&format!(
                "$p = $sh.CreateShortcut({lnk})\n$p.TargetPath = {t}\n$p.Arguments = {a}\n$p.Save()\n",
                t = ps_quote(program),
                a = ps_quote(args),
            ));
        } else {
            s.push_str(&format!(
                "$p = $sh.CreateShortcut({lnk})\n\
                 if ($p.Arguments -notlike '*{flag}*') {{ $p.Arguments = ($p.Arguments + ' {flag}').Trim(); $p.Save() }}\n",
                flag = terminals::LAUNCHER_FLAG,
            ));
        }
    }
    let owned: Vec<PathBuf> = items
        .iter()
        .filter(|(_, owned)| *owned)
        .map(|(lnk, _)| lnk.clone())
        .collect();
    s.push_str(&aumid_script(&owned));
    let all: Vec<PathBuf> = items.iter().map(|(lnk, _)| lnk.clone()).collect();
    s.push_str(&notify_script(&all, true));
    s
}

/// Best effort: pinned taskbar copies made before `--launcher` existed keep
/// their old command line (Explorer copies the shortcut when pinning), so the
/// start they make is a normal one. They are rewritten here. True when a
/// rewrite was run. Skipped when `refresh_icon_cache` is off (tests: no
/// PowerShell is spawned).
pub(super) fn sync_pinned_args(dirs: &Dirs, req: &Request) -> bool {
    if !req.refresh_icon_cache {
        return false;
    }
    let items = pinned_stale(dirs, req);
    let Ok((program, args)) = target(req) else {
        return false;
    };
    !items.is_empty() && run_powershell(&pinned_args_script(&items, &program, &args)).is_ok()
}

fn run_powershell(script: &str) -> Result<(), String> {
    let mut cmd = std::process::Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-EncodedCommand",
        &encode_command(script),
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let out = cmd.output().map_err(|e| format!("PowerShell: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

pub(super) fn remove(place: Place, dirs: &Dirs) -> Result<bool, String> {
    let path = lnk_path(place, dirs)?;
    let Ok(bytes) = std::fs::read(&path) else {
        return Ok(false);
    };
    if !contains_text(&bytes, MARKER) {
        return Err(format!("{} is not ours; not removing it", path.display()));
    }
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    let other = match place {
        Place::StartMenu => dirs.desktop.as_ref(),
        _ => dirs.start_menu.as_ref(),
    };
    if !other.is_some_and(|d| d.join(FILE).exists()) {
        for ico in our_icos(dirs) {
            let _ = std::fs::remove_file(ico);
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::desktop::IconSet;

    fn req<'a>(exe: &'a Path, term: &'a str, icons: &'a IconSet) -> Request<'a> {
        Request {
            os: Os::Windows,
            exe,
            terminal: term,
            terminal_exe: None,
            launch_exe: None,
            newest: false,
            icons,
            refresh_icon_cache: false,
        }
    }

    #[test]
    fn the_console_shortcut_targets_the_exe_and_a_terminal_wraps_it() {
        let icons = IconSet::default();
        let exe = PathBuf::from(r"C:\Program Files\GitPoulpi\gitpoulpi.exe");
        assert_eq!(
            target(&req(&exe, "console", &icons)).unwrap(),
            (exe.to_string_lossy().to_string(), "--launcher".to_string())
        );
        let (program, args) = target(&req(&exe, "windows-terminal", &icons)).unwrap();
        assert_eq!(program, "wt.exe");
        assert_eq!(
            args,
            r#""C:\Program Files\GitPoulpi\gitpoulpi.exe" --launcher"#
        );
        let (_, args) = target(&req(&exe, "wezterm", &icons)).unwrap();
        assert_eq!(
            args,
            r#"start -- "C:\Program Files\GitPoulpi\gitpoulpi.exe" --launcher"#
        );
    }

    #[test]
    fn arguments_are_quoted_like_the_windows_command_line_wants() {
        assert_eq!(quote_arg("plain"), "plain");
        assert_eq!(quote_arg(r"C:\a b\"), r#""C:\a b\\""#);
        assert_eq!(quote_arg(r#"a"b"#), r#""a\"b""#);
    }

    #[test]
    fn the_script_sets_every_property_and_doubles_single_quotes() {
        let s = script(
            Path::new(r"C:\Users\O'Neil\Desktop\GitPoulpi.lnk"),
            "wt.exe",
            "\"x y\"",
            Path::new(r"C:\gp"),
            Path::new(r"C:\gp\gitpoulpi.ico"),
        );
        assert!(s.contains(r"CreateShortcut('C:\Users\O''Neil\Desktop\GitPoulpi.lnk')"));
        assert!(s.contains("$s.TargetPath = 'wt.exe'"));
        assert!(s.contains("$s.Arguments = '\"x y\"'"));
        assert!(s.contains(r"$s.IconLocation = 'C:\gp\gitpoulpi.ico,0'"));
        assert!(s.contains(&format!("$s.Description = 'GitPoulpi {MARKER}'")));
    }

    fn pinned_dirs(tmp: &Path) -> Dirs {
        Dirs {
            taskbar_pinned: Some(tmp.join("TaskBar")),
            assets: tmp.join("assets"),
            ..Dirs::default()
        }
    }

    fn utf16(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    fn icon_set(n: u8) -> IconSet {
        IconSet {
            pngs: vec![(16, vec![n])],
        }
    }

    fn ico_files(dirs: &Dirs) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(&dirs.assets)
            .map(|d| {
                d.flatten()
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();
        names.sort();
        names
    }

    #[test]
    fn the_ico_path_follows_the_icon_so_the_taskbar_cannot_keep_the_old_picture() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = pinned_dirs(tmp.path());
        let (a, b) = (icon_set(1), icon_set(2));
        let (pa, pb) = (ico_path(&dirs, &a), ico_path(&dirs, &b));
        assert_ne!(pa, pb, "a new colour is a new path");
        assert_eq!(pa, ico_path(&dirs, &a), "the same icon, the same path");
        let name = pa.file_name().unwrap().to_string_lossy().to_string();
        assert!(
            name.starts_with("gitpoulpi-") && name.ends_with(".ico"),
            "{name}"
        );
        assert!(is_our_ico(&name));
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        write_icon(&dirs, &req(&exe, "console", &a)).unwrap();
        write_icon(&dirs, &req(&exe, "console", &b)).unwrap();
        assert_eq!(std::fs::read(&pa).unwrap(), a.ico());
        assert_eq!(std::fs::read(&pb).unwrap(), b.ico());
        assert_eq!(ico_files(&dirs).len(), 2, "the old one stays until cleaned");
    }

    #[test]
    fn only_our_own_icon_names_are_ours() {
        for ours in ["gitpoulpi.ico", "gitpoulpi-0123456789ab.ico"] {
            assert!(is_our_ico(ours), "{ours}");
        }
        for other in [
            "gitpoulpi-0123456789ab.png",
            "gitpoulpi-0123456789a.ico",
            "gitpoulpi-0123456789abc.ico",
            "gitpoulpi-0123456789AB.ico",
            "gitpoulpi-extra.ico",
            "other-0123456789ab.ico",
            "notes.ico",
        ] {
            assert!(!is_our_ico(other), "{other}");
        }
    }

    #[test]
    fn cleanup_deletes_old_icons_nothing_points_at_and_keeps_the_rest() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dirs = pinned_dirs(tmp.path());
        dirs.start_menu = Some(tmp.path().join("Programs"));
        let pins = dirs.taskbar_pinned.clone().unwrap();
        std::fs::create_dir_all(&pins).unwrap();
        std::fs::create_dir_all(dirs.start_menu.as_ref().unwrap()).unwrap();
        std::fs::create_dir_all(&dirs.assets).unwrap();
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let (new, mid, old, legacy) = (icon_set(3), icon_set(2), icon_set(1), "gitpoulpi.ico");
        for set in [&new, &mid, &old] {
            write_icon(&dirs, &req(&exe, "console", set)).unwrap();
        }
        std::fs::write(dirs.assets.join(legacy), b"old").unwrap();
        std::fs::write(dirs.assets.join("notes.ico"), b"mine").unwrap();
        std::fs::write(dirs.assets.join("gitpoulpi-extra.ico"), b"mine").unwrap();
        // A pin (any shortcut, ours or not) still names the middle icon.
        std::fs::write(
            pins.join("pin.lnk"),
            utf16(&ico_path(&dirs, &mid).to_string_lossy()),
        )
        .unwrap();
        cleanup_icons(&dirs, &ico_path(&dirs, &new));
        let left = ico_files(&dirs);
        assert!(left.contains(&ico_name(&new)), "{left:?}");
        assert!(left.contains(&ico_name(&mid)), "a pin still points at it");
        assert!(!left.contains(&ico_name(&old)), "{left:?}");
        assert!(!left.contains(&legacy.to_string()), "{left:?}");
        assert!(left.contains(&"notes.ico".to_string()));
        assert!(left.contains(&"gitpoulpi-extra.ico".to_string()));
        // The pin is repointed later: now nothing names the middle one.
        std::fs::remove_file(pins.join("pin.lnk")).unwrap();
        cleanup_icons(&dirs, &ico_path(&dirs, &new));
        assert!(!ico_files(&dirs).contains(&ico_name(&mid)));
    }

    #[test]
    fn a_shortcut_naming_another_colour_is_stale_until_repointed() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dirs = pinned_dirs(tmp.path());
        dirs.start_menu = Some(tmp.path().join("Programs"));
        std::fs::create_dir_all(dirs.start_menu.as_ref().unwrap()).unwrap();
        let path = dirs.start_menu.as_ref().unwrap().join(FILE);
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let (old, new) = (icon_set(1), icon_set(2));
        let lnk = |icons: &IconSet| {
            utf16(&format!(
                "{} --launcher {} {MARKER} {}",
                exe.display(),
                terminals::AUMID,
                ico_path(&dirs, icons).display()
            ))
        };
        std::fs::write(&path, lnk(&old)).unwrap();
        let r = req(&exe, "console", &new);
        write_icon(&dirs, &r).unwrap();
        let st = status(Place::StartMenu, &dirs, &r);
        assert_eq!(st.state, State::Installed);
        assert!(st.icon_stale, "the shortcut still names the old file");
        assert_eq!(shortcuts_to_repoint(&dirs, &r), std::slice::from_ref(&path));
        std::fs::write(&path, lnk(&new)).unwrap();
        assert!(!status(Place::StartMenu, &dirs, &r).icon_stale);
        assert!(shortcuts_to_repoint(&dirs, &r).is_empty());
    }

    #[test]
    fn refreshing_the_icon_writes_the_new_file_and_drops_the_old_one_nobody_uses() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = pinned_dirs(tmp.path());
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let (old, new) = (icon_set(1), icon_set(2));
        write_icon(&dirs, &req(&exe, "console", &old)).unwrap();
        refresh_icon(&dirs, &req(&exe, "console", &new)).unwrap();
        assert_eq!(ico_files(&dirs), [ico_name(&new)]);
    }

    #[test]
    fn removing_the_last_shortcut_removes_every_icon_of_ours_only() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dirs = pinned_dirs(tmp.path());
        dirs.start_menu = Some(tmp.path().join("Programs"));
        std::fs::create_dir_all(dirs.start_menu.as_ref().unwrap()).unwrap();
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        for n in [1, 2] {
            write_icon(&dirs, &req(&exe, "console", &icon_set(n))).unwrap();
        }
        std::fs::write(dirs.assets.join("gitpoulpi.ico"), b"old").unwrap();
        std::fs::write(dirs.assets.join("notes.ico"), b"mine").unwrap();
        std::fs::write(
            dirs.start_menu.as_ref().unwrap().join(FILE),
            utf16(&format!("GitPoulpi {MARKER}")),
        )
        .unwrap();
        assert!(remove(Place::StartMenu, &dirs).unwrap());
        assert_eq!(ico_files(&dirs), ["notes.ico"]);
    }

    #[test]
    fn repointing_tells_the_shell_about_each_shortcut_and_keeps_the_app_id() {
        let lnk = PathBuf::from(r"C:\u\Pins\GitPoulpi.lnk");
        let s = pinned_script(&[lnk], Path::new(r"C:\gp\gitpoulpi-0123456789ab.ico"));
        assert!(s.contains(r"$p.IconLocation = 'C:\gp\gitpoulpi-0123456789ab.ico,0'"));
        assert!(s.contains(
            r"SHChangeNotifyPath(0x2000, 0x5, 'C:\u\Pins\GitPoulpi.lnk', [IntPtr]::Zero)"
        ));
        assert!(s.contains("SHChangeNotify(0x08000000"));
        assert!(s.contains("[GpAumid]::Set('C:\\u\\Pins\\GitPoulpi.lnk'"));
        // Declared before it is used.
        assert!(
            s.find("Add-Type -Namespace Gp").unwrap() < s.find("SHChangeNotifyPath(0x").unwrap()
        );
        assert!(!pinned_script(&[], Path::new("x.ico")).contains("SHChangeNotifyPath(0x"));
    }

    #[test]
    fn pinned_copies_are_ours_by_marker_or_by_exe_path_only() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = pinned_dirs(tmp.path());
        let pins = dirs.taskbar_pinned.clone().unwrap();
        std::fs::create_dir_all(&pins).unwrap();
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let icons = IconSet::default();
        let r = req(&exe, "console", &icons);
        let wt = utf16(r#"wt.exe "C:\gp\gitpoulpi.exe""#);
        std::fs::write(
            pins.join("marker.lnk"),
            utf16(&format!("GitPoulpi {MARKER}")),
        )
        .unwrap();
        std::fs::write(pins.join("terminal.LNK"), &wt).unwrap();
        std::fs::write(
            pins.join("other.lnk"),
            utf16(
                r"C:\Windows
otepad.exe",
            ),
        )
        .unwrap();
        std::fs::write(pins.join("notes.txt"), utf16(MARKER)).unwrap();
        let names: Vec<_> = pinned_ours(&dirs, &r)
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(names, ["marker.lnk", "terminal.LNK"]);
        assert!(pinned_ours(&Dirs::default(), &r).is_empty());
    }

    #[test]
    fn only_pinned_copies_with_another_icon_are_repointed() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = pinned_dirs(tmp.path());
        let pins = dirs.taskbar_pinned.clone().unwrap();
        std::fs::create_dir_all(&pins).unwrap();
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let icons = IconSet::default();
        let r = req(&exe, "console", &icons);
        let ico = ico_path(&dirs, &icons).to_string_lossy().to_string();
        std::fs::write(
            pins.join("old.lnk"),
            utf16(&format!("{MARKER} C:\\elsewhere.ico")),
        )
        .unwrap();
        std::fs::write(pins.join("new.lnk"), utf16(&format!("{MARKER} {ico},0"))).unwrap();
        let todo = pinned_to_repoint(&dirs, &r);
        assert_eq!(todo, [pins.join("old.lnk")]);
        let s = pinned_script(&todo, &ico_path(&dirs, &icons));
        assert!(s.contains("old.lnk"));
        assert!(!s.contains("new.lnk"));
        assert!(s.contains(&format!("$p.IconLocation = '{ico},0'")));
        assert!(s.contains("SHChangeNotify(0x08000000"));
        assert!(!pinned_script(&[], &ico_path(&dirs, &icons)).contains("CreateShortcut"));
    }

    #[test]
    fn a_shortcut_without_the_launcher_flag_reads_outdated_and_with_it_installed() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = Dirs {
            start_menu: Some(tmp.path().join("Programs")),
            assets: tmp.path().join("assets"),
            ..Dirs::default()
        };
        std::fs::create_dir_all(dirs.start_menu.as_ref().unwrap()).unwrap();
        let path = dirs.start_menu.as_ref().unwrap().join(FILE);
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let icons = IconSet::default();
        for (term, program) in [("console", ""), ("windows-terminal", "wt.exe")] {
            let r = req(&exe, term, &icons);
            let old = utf16(&format!("{program} {} GitPoulpi {MARKER}", exe.display()));
            std::fs::write(&path, &old).unwrap();
            assert_eq!(
                status(Place::StartMenu, &dirs, &r).state,
                State::Outdated,
                "{term}"
            );
            let new = utf16(&format!(
                "{program} {} --launcher {} GitPoulpi {MARKER}",
                exe.display(),
                terminals::AUMID
            ));
            std::fs::write(&path, &new).unwrap();
            assert_eq!(
                status(Place::StartMenu, &dirs, &r).state,
                State::Installed,
                "{term}"
            );
        }
    }

    #[test]
    fn pinned_copies_without_the_flag_are_found_and_rewritten_by_ownership() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = pinned_dirs(tmp.path());
        let pins = dirs.taskbar_pinned.clone().unwrap();
        std::fs::create_dir_all(&pins).unwrap();
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let icons = IconSet::default();
        let r = req(&exe, "windows-terminal", &icons);
        // A pin of our shortcut from before the flag: marker, no flag.
        std::fs::write(
            pins.join("a-ours-old.lnk"),
            utf16(&format!("wt.exe {} GitPoulpi {MARKER}", exe.display())),
        )
        .unwrap();
        // The user's own shortcut to our exe, marker stripped, no flag.
        std::fs::write(
            pins.join("b-user.lnk"),
            utf16(&format!("wt.exe -p Mine {}", exe.display())),
        )
        .unwrap();
        // Already current.
        std::fs::write(
            pins.join("c-current.lnk"),
            utf16(&format!(
                "wt.exe {} --launcher {} {MARKER}",
                exe.display(),
                terminals::AUMID
            )),
        )
        .unwrap();
        // Not ours.
        std::fs::write(pins.join("d-other.lnk"), utf16(r"C:\Windows\notepad.exe")).unwrap();
        let todo = pinned_stale(&dirs, &r);
        assert_eq!(
            todo,
            [
                (pins.join("a-ours-old.lnk"), true),
                (pins.join("b-user.lnk"), false)
            ]
        );
        let (program, args) = target(&r).unwrap();
        let s = pinned_args_script(&todo, &program, &args);
        assert!(s.contains("a-ours-old.lnk"));
        assert!(s.contains("$p.TargetPath = 'wt.exe'"));
        assert!(s.contains(r#"$p.Arguments = 'C:\gp\gitpoulpi.exe --launcher'"#));
        // Our copy gets the AppUserModelID too; the user's does not.
        assert!(
            s.contains("[GpAumid]::Set('") && s.contains("a-ours-old.lnk', 'GitPoulpi.GitPoulpi')")
        );
        assert!(!s.contains("b-user.lnk', 'GitPoulpi.GitPoulpi')"));
        // The user's copy only gets the flag appended, never a new target.
        let user = s.split("b-user.lnk").nth(1).unwrap();
        assert!(user.contains("' --launcher'"));
        assert!(!user.contains("TargetPath"));
        assert!(!s.contains("c-current.lnk") && !s.contains("d-other.lnk"));
        // Nothing is spawned without the cache-refresh switch (tests).
        assert!(!sync_pinned_args(&dirs, &r));
    }

    fn launch_req<'a>(
        exe: &'a Path,
        launch: &'a Path,
        term: &'a str,
        icons: &'a IconSet,
    ) -> Request<'a> {
        Request {
            launch_exe: Some(launch),
            ..req(exe, term, icons)
        }
    }

    #[test]
    fn with_the_helper_the_shortcut_targets_it_and_the_rest_of_the_line_follows_a_double_dash() {
        let icons = IconSet::default();
        let exe = PathBuf::from(r"C:\Program Files\GitPoulpi\gitpoulpi.exe");
        let launch = PathBuf::from(r"C:\Program Files\GitPoulpi\gitpoulpi-launch.exe");
        let (program, args) =
            target(&launch_req(&exe, &launch, "windows-terminal", &icons)).unwrap();
        assert_eq!(program, launch.to_string_lossy());
        assert_eq!(
            args,
            r#"-- wt.exe "C:\Program Files\GitPoulpi\gitpoulpi.exe" --launcher"#
        );
        let (_, args) = target(&launch_req(&exe, &launch, "console", &icons)).unwrap();
        assert_eq!(
            args,
            r#"-- "C:\Program Files\GitPoulpi\gitpoulpi.exe" --launcher"#
        );
        // A terminal found by its full path (with spaces) stays one word.
        let wez = PathBuf::from(r"C:\Apps\Wez Term\wezterm-gui.exe");
        let mut r = launch_req(&exe, &launch, "wezterm", &icons);
        r.terminal_exe = Some(&wez);
        let (_, args) = target(&r).unwrap();
        assert!(
            args.starts_with(r#"-- "C:\Apps\Wez Term\wezterm-gui.exe" start -- "#),
            "{args}"
        );
    }

    #[test]
    fn a_shortcut_without_the_helper_or_the_app_id_reads_outdated() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = Dirs {
            start_menu: Some(tmp.path().join("Programs")),
            assets: tmp.path().join("assets"),
            ..Dirs::default()
        };
        std::fs::create_dir_all(dirs.start_menu.as_ref().unwrap()).unwrap();
        let path = dirs.start_menu.as_ref().unwrap().join(FILE);
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let launch = PathBuf::from(r"C:\gp\gitpoulpi-launch.exe");
        let icons = IconSet::default();
        let r = launch_req(&exe, &launch, "console", &icons);
        let words = |helper: bool, aumid: bool| {
            utf16(&format!(
                "{} {} --launcher {} {MARKER}",
                if helper {
                    launch.display().to_string()
                } else {
                    String::new()
                },
                exe.display(),
                if aumid { terminals::AUMID } else { "" },
            ))
        };
        for (helper, aumid, want) in [
            (false, true, State::Outdated),
            (true, false, State::Outdated),
            (true, true, State::Installed),
        ] {
            std::fs::write(&path, words(helper, aumid)).unwrap();
            assert_eq!(
                status(Place::StartMenu, &dirs, &r).state,
                want,
                "helper {helper}, aumid {aumid}"
            );
        }
    }

    #[test]
    fn the_script_gives_every_shortcut_the_app_user_model_id_and_never_fails_the_install() {
        let s = aumid_script(&[
            PathBuf::from(r"C:\u\Start\GitPoulpi.lnk"),
            PathBuf::from(r"C:\u\Pins\O'Neil.lnk"),
        ]);
        assert!(
            s.contains("9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3"),
            "PKEY_AppUserModel_ID"
        );
        assert!(s.contains("pid = 5"));
        assert!(s.contains(r"[GpAumid]::Set('C:\u\Start\GitPoulpi.lnk', 'GitPoulpi.GitPoulpi')"));
        assert!(s.contains(r"O''Neil.lnk"));
        // Every call, and the type itself, are inside try/catch.
        assert_eq!(s.matches("catch").count(), 3);
        assert!(aumid_script(&[]).is_empty());
        // The install script carries it.
        let full = script(
            Path::new(r"C:\u\GitPoulpi.lnk"),
            "x.exe",
            "",
            Path::new(r"C:\gp"),
            Path::new(r"C:\gp\i.ico"),
        );
        assert!(full.contains("[GpAumid]::Set("));
    }

    #[test]
    fn encoded_command_is_base64_of_utf16le() {
        // "A" is 41 00 -> "QQA=".
        assert_eq!(encode_command("A"), "QQA=");
        assert_eq!(encode_command("Hi"), "SABpAA==");
    }

    #[test]
    fn contains_text_finds_ansi_and_utf16_strings_without_case() {
        let utf16: Vec<u8> = "xx C:\\Gp\\a.exe"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert!(contains_text(&utf16, r"c:\gp\a.exe"));
        assert!(contains_text(b"..C:\\Gp\\a.exe..", r"C:\Gp\a.exe"));
        assert!(!contains_text(b"nothing", "x.exe"));
    }

    #[test]
    fn status_and_remove_read_the_marker_from_the_file_bytes() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = Dirs {
            start_menu: Some(tmp.path().join("Programs")),
            assets: tmp.path().join("assets"),
            ..Dirs::default()
        };
        std::fs::create_dir_all(dirs.start_menu.as_ref().unwrap()).unwrap();
        let path = dirs.start_menu.as_ref().unwrap().join(FILE);
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let icons = IconSet::default();
        let r = req(&exe, "console", &icons);

        assert_eq!(
            status(Place::StartMenu, &dirs, &r).state,
            State::NotInstalled
        );
        std::fs::write(&path, b"foreign").unwrap();
        assert_eq!(status(Place::StartMenu, &dirs, &r).state, State::Foreign);
        assert!(remove(Place::StartMenu, &dirs).is_err());

        let ours: Vec<u8> = format!(
            "{} --launcher {} GitPoulpi {MARKER}",
            exe.display(),
            terminals::AUMID
        )
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
        std::fs::write(&path, &ours).unwrap();
        assert_eq!(status(Place::StartMenu, &dirs, &r).state, State::Installed);
        let moved = PathBuf::from(r"D:\other\gitpoulpi.exe");
        assert_eq!(
            status(Place::StartMenu, &dirs, &req(&moved, "console", &icons)).state,
            State::Outdated
        );
        assert!(remove(Place::StartMenu, &dirs).unwrap());
        assert!(!path.exists());
    }

    #[test]
    fn newest_mode_reads_a_shortcut_naming_either_build_as_installed() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("co");
        std::fs::create_dir_all(root.join("crates/gitpoulpi-tui")).unwrap();
        std::fs::write(root.join("Cargo.toml"), "").unwrap();
        let build = |p: &str| root.join("target").join(p).join("gitpoulpi-tui.exe");
        let (dist, release) = (build("dist"), build("release"));
        let dirs = Dirs {
            start_menu: Some(tmp.path().join("Programs")),
            assets: tmp.path().join("assets"),
            ..Dirs::default()
        };
        std::fs::create_dir_all(dirs.start_menu.as_ref().unwrap()).unwrap();
        let path = dirs.start_menu.as_ref().unwrap().join(FILE);
        let icons = IconSet::default();
        // The shortcut was written for the dist build.
        let ours: Vec<u8> = format!(
            "{} --launcher {} GitPoulpi {MARKER}",
            dist.display(),
            terminals::AUMID
        )
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
        std::fs::write(&path, &ours).unwrap();
        // The release TUI starts: without newest it would rewrite it.
        let r = req(&release, "console", &icons);
        assert_eq!(status(Place::StartMenu, &dirs, &r).state, State::Outdated);
        let newest = Request { newest: true, ..r };
        assert_eq!(
            status(Place::StartMenu, &dirs, &newest).state,
            State::Installed
        );
        let other = tmp.path().join("elsewhere/gitpoulpi-tui.exe");
        let elsewhere = Request {
            exe: &other,
            newest: true,
            ..r
        };
        assert_eq!(
            status(Place::StartMenu, &dirs, &elsewhere).state,
            State::Outdated
        );
    }
}
