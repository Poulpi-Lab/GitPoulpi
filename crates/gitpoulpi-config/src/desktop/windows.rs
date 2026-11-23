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

use super::owner::{self, Facts, Owner};
use super::{
    terminals, Dirs, IconSet, InstallCtx, Place, Request, State, Status, MARKER, OWNER_TAG,
};
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
pub(super) fn script(
    lnk: &Path,
    target: &str,
    args: &str,
    workdir: &Path,
    icon: &Path,
    desc: &str,
    aumid: &str,
) -> String {
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
        desc = ps_quote(desc),
        aumid = aumid_script(&[lnk.to_path_buf()], aumid),
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
pub(super) fn aumid_script(lnks: &[PathBuf], aumid: &str) -> String {
    if lnks.is_empty() {
        return String::new();
    }
    let mut s = String::from(AUMID_TYPE);
    for lnk in lnks {
        s.push_str(&format!(
            "try {{ [GpAumid]::Set({lnk}, {id}) }} catch {{ }}\n",
            lnk = ps_quote(&lnk.to_string_lossy()),
            id = ps_quote(aumid),
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

/// The file name of `ctx`'s shortcut: the primary keeps [`FILE`] (the name old
/// builds look for), every other installation is `<launcher name>.lnk`.
pub(super) fn file_name(ctx: &InstallCtx) -> String {
    if ctx.is_primary() {
        FILE.to_string()
    } else {
        named_file(ctx)
    }
}

/// `<launcher name>.lnk`, the name a non-primary installation uses (and the
/// one a primary handover renames the old primary's legacy file to).
fn named_file(ctx: &InstallCtx) -> String {
    let clean: String = ctx
        .name
        .chars()
        .map(|c| {
            if c.is_control() || "\\/:*?\"<>|".contains(c) {
                '_'
            } else {
                c
            }
        })
        .collect();
    let clean = clean.trim_end_matches(['.', ' ']).trim_start();
    if clean.is_empty() {
        format!("GitPoulpi ({}).lnk", ctx.short)
    } else {
        format!("{clean}.lnk")
    }
}

fn dir_of(place: Place, dirs: &Dirs) -> Result<&Path, String> {
    match place {
        Place::StartMenu => dirs.start_menu.as_deref(),
        Place::Desktop => dirs.desktop.as_deref(),
        Place::Launcher => return Err("Windows has the Start menu, not a launcher".into()),
    }
    .ok_or_else(|| "that folder was not found".to_string())
}

fn lnk_path(place: Place, dirs: &Dirs, ctx: &InstallCtx) -> Result<PathBuf, String> {
    dir_of(place, dirs).map(|d| d.join(file_name(ctx)))
}

/// The owner tag written into the `.lnk` description: `X-GitPoulpi-Install=<id>`.
/// Nothing without an id (the id-less legacy context of tests and fakes).
fn tag_text(ctx: &InstallCtx) -> Option<String> {
    (!ctx.id.is_empty()).then(|| format!("{OWNER_TAG}={}", ctx.id))
}

/// The description of `ctx`'s shortcut. The legacy marker is written by the
/// primary only: old builds look for exactly that, so a non-primary shortcut
/// is invisible to them.
pub(super) fn description(ctx: &InstallCtx) -> String {
    let mut d = String::from("GitPoulpi");
    if ctx.is_primary() {
        d.push(' ');
        d.push_str(MARKER);
    }
    if let Some(tag) = tag_text(ctx) {
        d.push(' ');
        d.push_str(&tag);
    }
    d
}

/// The value of the owner tag in a `.lnk`'s bytes (ANSI or UTF-16LE).
///
/// A `.lnk` StringData field has no terminator: a 16-bit character count, the
/// characters, then the next field's count right away. That count can itself
/// be an id character (0x35 is `5`), so reading "while id characters" can
/// swallow it. The UTF-16 description is therefore bounded by its own count
/// prefix; where that cannot be located, a hashed id is cut to its exact 16
/// lowercase hex digits (pinned `p-<name>` ids have no fixed length).
pub(super) fn read_tag(bytes: &[u8]) -> Option<String> {
    let key = format!("{OWNER_TAG}=");
    let is_id = |c: u8| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_' | b'.');
    let at = |hay: &[u8], pat: &[u8]| hay.windows(pat.len()).position(|w| w == pat);
    let bound = |mut v: String| {
        if !v.starts_with("p-") && v.len() > 16 && v.as_bytes()[..16].iter().all(is_lower_hex) {
            v.truncate(16);
        }
        v
    };
    if let Some(p) = at(bytes, key.as_bytes()) {
        let value: Vec<u8> = bytes[p + key.len()..]
            .iter()
            .copied()
            .take_while(|b| is_id(*b))
            .collect();
        return Some(bound(String::from_utf8_lossy(&value).into_owned()));
    }
    let wide_of = |t: &str| -> Vec<u8> { t.encode_utf16().flat_map(u16::to_le_bytes).collect() };
    let wide = wide_of(&key);
    let p = at(bytes, &wide)?;
    let from = p + wide.len();
    // The description starts with "GitPoulpi" (see `description`); the count
    // prefix sits right before it. A hit inside the description (the legacy
    // marker `X-GitPoulpi=1`) is skipped, hence the `-` test below.
    let head = wide_of("GitPoulpi");
    let counted_end = bytes[..p]
        .windows(head.len())
        .rposition(|w| w == head.as_slice())
        // The legacy marker `X-GitPoulpi=1` inside the description is no start.
        .filter(|ds| *ds >= 2 && bytes[ds - 2..*ds] != [b'-', 0])
        .and_then(|ds| {
            let n = u16::from_le_bytes([bytes[ds - 2], bytes[ds - 1]]) as usize;
            let end = ds + 2 * n;
            (end >= from && end <= bytes.len()).then_some(end)
        });
    let limit = counted_end.unwrap_or(bytes.len());
    let value: Vec<u8> = bytes[from..limit]
        .as_chunks::<2>()
        .0
        .iter()
        .take_while(|c| c[1] == 0 && is_id(c[0]))
        .map(|c| c[0])
        .collect();
    let v = String::from_utf8_lossy(&value).into_owned();
    Some(if counted_end.is_some() { v } else { bound(v) })
}

fn is_lower_hex(c: &u8) -> bool {
    matches!(c, b'0'..=b'9' | b'a'..=b'f')
}

/// Facts of a `.lnk` for [`owner::decide`]; `exes` are the spellings of the
/// running exe (a pin the user made names it, with neither tag nor marker).
fn facts_of<'a>(bytes: &[u8], tag: Option<&'a str>, exes: &[String]) -> Facts<'a> {
    Facts {
        tag,
        claimed_by: None,
        legacy_marker: contains_text(bytes, MARKER),
        names_my_exe: exes.iter().any(|e| contains_text(bytes, e)),
    }
}

/// Who owns the shortcut at `ctx`'s own path. An untagged legacy file there
/// can only be one the primary handover renamed to this installation's name
/// (contents untouched): it is mine, and the next sync rewrites it.
fn owner_at_own_path(bytes: &[u8], ctx: &InstallCtx) -> Owner {
    let tag = read_tag(bytes);
    let facts = facts_of(bytes, tag.as_deref(), &[]);
    if !ctx.is_primary() && facts.tag.is_none() && facts.legacy_marker {
        return Owner::Mine;
    }
    owner::decide(&facts, ctx)
}

fn is_mine(owner: &Owner) -> bool {
    matches!(owner, Owner::Mine | Owner::Adopt)
}

/// `gitpoulpi-<hash of the .ico bytes>.ico` (the primary) or
/// `gitpoulpi-i<8hex>-<hash>.ico` (every other installation): a new colour is
/// a new name. FNV-1a: stable across builds, unlike `DefaultHasher`.
pub(super) fn ico_name_for(icons: &IconSet, ctx: &InstallCtx) -> String {
    let hash = icons.ico().iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    let digits = format!(
        "{:0width$x}",
        hash & ((1u64 << (4 * ICON_HASH_DIGITS)) - 1),
        width = ICON_HASH_DIGITS
    );
    if ctx.is_primary() {
        format!("{ICON_PREFIX}{digits}.ico")
    } else {
        format!("{ICON_PREFIX}{}-{digits}.ico", ctx.short)
    }
}

/// [`ico_name_for`] the primary.
#[cfg(test)]
pub(super) fn ico_name(icons: &IconSet) -> String {
    ico_name_for(icons, InstallCtx::legacy_ref())
}

pub(super) fn ico_path_for(dirs: &Dirs, icons: &IconSet, ctx: &InstallCtx) -> PathBuf {
    dirs.assets.join(ico_name_for(icons, ctx))
}

/// [`ico_path_for`] the primary.
pub(super) fn ico_path(dirs: &Dirs, icons: &IconSet) -> PathBuf {
    ico_path_for(dirs, icons, InstallCtx::legacy_ref())
}

fn is_hash(h: &str) -> bool {
    h.len() == ICON_HASH_DIGITS && h.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Whether `file` is an icon the primary wrote: `gitpoulpi.ico` (before the
/// names carried a hash) or `gitpoulpi-<12 hex>.ico`.
pub(super) fn is_our_ico(file: &str) -> bool {
    file == LEGACY_ICON_FILE
        || file
            .strip_prefix(ICON_PREFIX)
            .and_then(|r| r.strip_suffix(".ico"))
            .is_some_and(is_hash)
}

/// Whether `file` is an icon `ctx` wrote: the primary's names, or for another
/// installation `gitpoulpi-i<8hex>-<12 hex>.ico` with its own short id. Nothing
/// else in the folder is ever touched, another installation's icons included.
pub(super) fn is_my_ico(file: &str, ctx: &InstallCtx) -> bool {
    if ctx.is_primary() {
        return is_our_ico(file);
    }
    file.strip_prefix(ICON_PREFIX)
        .and_then(|r| r.strip_prefix(ctx.short.as_str()))
        .and_then(|r| r.strip_prefix('-'))
        .and_then(|r| r.strip_suffix(".ico"))
        .is_some_and(is_hash)
}

fn my_icos(dirs: &Dirs, ctx: &InstallCtx) -> Vec<PathBuf> {
    std::fs::read_dir(&dirs.assets)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| is_my_ico(&e.file_name().to_string_lossy(), ctx))
        .map(|e| e.path())
        .collect()
}

/// Every `.lnk` Explorer keeps for taskbar pins: `User Pinned\TaskBar` and
/// the folders (one deep) of `ImplicitAppShortcuts`. Sorted.
pub(super) fn pinned_lnks(dirs: &Dirs) -> Vec<PathBuf> {
    fn lnks_in(dir: &Path, out: &mut Vec<PathBuf>) {
        out.extend(
            std::fs::read_dir(dir)
                .into_iter()
                .flatten()
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk"))),
        );
    }
    let mut out = Vec::new();
    if let Some(dir) = dirs.taskbar_pinned.as_ref() {
        lnks_in(dir, &mut out);
    }
    if let Some(dir) = dirs.implicit_pinned.as_ref() {
        lnks_in(dir, &mut out);
        for sub in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            if sub.path().is_dir() {
                lnks_in(&sub.path(), &mut out);
            }
        }
    }
    out.sort();
    out
}

/// Every shortcut that may name one of `ctx`'s icons: its own Start menu and
/// Desktop entries and ALL shortcuts pinned to the taskbar (a pin whose marker
/// Windows stripped still points at our file, and must not lose it).
fn icon_holders(dirs: &Dirs, ctx: &InstallCtx) -> Vec<PathBuf> {
    let mut all: Vec<PathBuf> = [dirs.start_menu.as_ref(), dirs.desktop.as_ref()]
        .into_iter()
        .flatten()
        .map(|d| d.join(file_name(ctx)))
        .collect();
    all.extend(pinned_lnks(dirs));
    all
}

/// Deletes the icon files of `ctx` that are not `keep` and that no shortcut
/// points at any more.
pub(super) fn cleanup_icons(dirs: &Dirs, keep: &Path, ctx: &InstallCtx) {
    let holders: Vec<Vec<u8>> = icon_holders(dirs, ctx)
        .iter()
        .filter_map(|p| std::fs::read(p).ok())
        .collect();
    for ico in my_icos(dirs, ctx) {
        let name = ico.to_string_lossy();
        if ico == keep || holders.iter().any(|b| contains_text(b, &name)) {
            continue;
        }
        let _ = std::fs::remove_file(&ico);
    }
}

/// Deletes every icon `ctx` wrote when it keeps no shortcut of its own in the
/// Start menu or on the Desktop and nothing else points at it: a pin, or the
/// legacy-named `GitPoulpi.lnk` (the primary's name, which the holders of a
/// primary `ctx` include). The icons the window or a handed-over shortcut left
/// behind otherwise stay for good. Another installation's icons are never
/// touched.
pub(super) fn release_icons(dirs: &Dirs, ctx: &InstallCtx) {
    let kept = [dirs.start_menu.as_ref(), dirs.desktop.as_ref()]
        .into_iter()
        .flatten()
        .any(|d| d.join(file_name(ctx)).exists());
    if !kept {
        cleanup_icons(dirs, Path::new(""), ctx);
    }
}

/// Writes `icons` as `ctx`'s current `.ico` (when missing or different) and
/// says where; the running GitPoulpi puts it on its terminal window.
pub(super) fn ensure_ico(dirs: &Dirs, icons: &IconSet, ctx: &InstallCtx) -> Option<PathBuf> {
    if icons.pngs.is_empty() {
        return None;
    }
    let path = ico_path_for(dirs, icons, ctx);
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
    req.launch_exe.is_none_or(|l| {
        exe_spellings(l, req.newest)
            .iter()
            .any(|e| contains_text(bytes, e))
    })
}

/// Does `bytes` name the retired `gitpoulpi-tui` spelling of the exe? Such a
/// shortcut still works (the stub) but is stale: it is rewritten to name
/// `gitpoulpi`.
fn names_legacy_exe(bytes: &[u8], req: &Request) -> bool {
    exe_spellings(req.exe, req.newest)
        .iter()
        .filter_map(|e| super::alias::legacy_tui_exe(Path::new(e)))
        .any(|l| contains_text(bytes, &l.to_string_lossy()))
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
    let Ok(path) = lnk_path(place, dirs, req.ctx) else {
        return none;
    };
    let Ok(bytes) = std::fs::read(&path) else {
        return none;
    };
    // Another installation's file, a legacy one without a primary, or not ours
    // at all: shown as `Foreign`, never rewritten.
    if !is_mine(&owner_at_own_path(&bytes, req.ctx)) {
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
    let aumid = terminals::aumid_for(req.ctx);
    let current = inner(req).is_ok_and(|(program, _)| {
        names_exe(&bytes, req)
            && !names_legacy_exe(&bytes, req)
            && (req.terminal == "console" || contains_text(&bytes, &program))
            && contains_text(&bytes, terminals::LAUNCHER_FLAG)
            && names_launch(&bytes, req)
            && contains_text(&bytes, &aumid)
    });
    // Stale: the current icon is not on disk, or this shortcut names another.
    let ico = ico_path_for(dirs, req.icons, req.ctx);
    let icon_stale = !req.icons.pngs.is_empty()
        && (std::fs::read(&ico).ok().as_deref() != Some(req.icons.ico().as_slice())
            || !contains_text(&bytes, &ico.to_string_lossy())
            // A pinned copy still on the old icon: the Start menu entry may
            // already be current (an earlier run, the CLI) while the pin is not.
            || !pinned_to_repoint(dirs, req).is_empty());
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
    super::write_file(
        &ico_path_for(dirs, req.icons, req.ctx),
        &req.icons.ico(),
        false,
    )
}

/// The logo colour changed: writes the new `.ico` under its new name,
/// re-points every shortcut of ours at it (PowerShell; skipped in tests),
/// then deletes the older files of this installation nothing points at.
pub(super) fn refresh_icon(dirs: &Dirs, req: &Request) -> Result<(), String> {
    write_icon(dirs, req)?;
    if req.refresh_icon_cache && !req.icons.pngs.is_empty() {
        repoint_shortcuts(dirs, req, &run_powershell);
        flush_icon_cache();
    }
    cleanup_icons(dirs, &ico_path_for(dirs, req.icons, req.ctx), req.ctx);
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

/// Runs one PowerShell script; a test swaps in a fake that records it.
pub(super) type Run<'a> = &'a dyn Fn(&str) -> Result<(), String>;

pub(super) fn install(place: Place, dirs: &Dirs, req: &Request) -> Result<PathBuf, String> {
    install_with(place, dirs, req, &run_powershell)
}

pub(super) fn install_with(
    place: Place,
    dirs: &Dirs,
    req: &Request,
    run: Run,
) -> Result<PathBuf, String> {
    let path = lnk_path(place, dirs, req.ctx)?;
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
        ico_path_for(dirs, req.icons, req.ctx)
    };
    let renamed = !path.exists();
    run(&script(
        &path,
        &program,
        &args,
        workdir,
        &icon,
        &description(req.ctx),
        &terminals::aumid_for(req.ctx),
    ))?;
    // The version in the name (or the primary toggle) changed the file name:
    // this installation's own tagged files under the old name go, the shell is
    // told. Another installation's file is never looked at.
    let old = stale_own_lnks(&path, req.ctx);
    for lnk in &old {
        let _ = std::fs::remove_file(lnk);
    }
    if !old.is_empty() || renamed {
        let _ = run(&gone_script(&old, Some(&path)));
    }
    if req.refresh_icon_cache && !req.icons.pngs.is_empty() {
        repoint_shortcuts(dirs, req, run);
    }
    sync_pinned_args_with(dirs, req, run);
    cleanup_icons(dirs, &ico_path_for(dirs, req.icons, req.ctx), req.ctx);
    Ok(path)
}

/// The other `.lnk` files in `current`'s folder tagged with this
/// installation's id: what its shortcut was called before the name changed.
fn stale_own_lnks(current: &Path, ctx: &InstallCtx) -> Vec<PathBuf> {
    let Some(dir) = current.parent() else {
        return Vec::new();
    };
    if ctx.id.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p != current && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("lnk")))
        .filter(|p| {
            std::fs::read(p).is_ok_and(|b| read_tag(&b).as_deref() == Some(ctx.id.as_str()))
        })
        .collect();
    out.sort();
    out
}

/// The shortcuts in `User Pinned` that start this installation's GitPoulpi,
/// each with whether it is a copy of a shortcut of ours (it carries our tag, or
/// the legacy marker while this is the primary: its whole target is ours to
/// rewrite) or one the user made (only our exe path identifies it: just the
/// missing launcher flag is appended). Every pin goes through
/// [`owner::decide`]: another installation's pin, a legacy one without a
/// primary and a foreign one are never touched. Sorted.
pub(super) fn pinned_owned(dirs: &Dirs, req: &Request) -> Vec<(PathBuf, bool)> {
    let mut exes: Vec<String> = exe_spellings(req.exe, req.newest)
        .into_iter()
        .filter(|e| !e.is_empty())
        .collect();
    // A pin an older release wrote names `gitpoulpi-tui`: still ours.
    let legacy: Vec<String> = exes
        .iter()
        .filter_map(|e| super::alias::legacy_tui_exe(Path::new(e)))
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    exes.extend(legacy);
    let mut found: Vec<(PathBuf, bool)> = pinned_lnks(dirs)
        .into_iter()
        .filter_map(|p| {
            let bytes = std::fs::read(&p).ok()?;
            let tag = read_tag(&bytes);
            let facts = facts_of(&bytes, tag.as_deref(), &exes);
            is_mine(&owner::decide(&facts, req.ctx))
                .then_some((p, facts.tag.is_some() || facts.legacy_marker))
        })
        .collect();
    found.sort();
    found
}

/// [`pinned_owned`] without the flag.
pub(super) fn pinned_ours(dirs: &Dirs, req: &Request) -> Vec<PathBuf> {
    pinned_owned(dirs, req)
        .into_iter()
        .map(|(p, _)| p)
        .collect()
}

/// The pinned copies whose icon is not our current `.ico` yet.
pub(super) fn pinned_to_repoint(dirs: &Dirs, req: &Request) -> Vec<PathBuf> {
    let ico = ico_path_for(dirs, req.icons, req.ctx)
        .to_string_lossy()
        .to_string();
    pinned_ours(dirs, req)
        .into_iter()
        .filter(|p| std::fs::read(p).is_ok_and(|b| !contains_text(&b, &ico)))
        .collect()
}

/// Every shortcut of this installation (Start menu, Desktop, pinned copies)
/// whose icon is not the current `.ico`.
pub(super) fn shortcuts_to_repoint(dirs: &Dirs, req: &Request) -> Vec<PathBuf> {
    let ico = ico_path_for(dirs, req.icons, req.ctx)
        .to_string_lossy()
        .to_string();
    let own = [dirs.start_menu.as_ref(), dirs.desktop.as_ref()]
        .into_iter()
        .flatten()
        .map(|d| d.join(file_name(req.ctx)))
        .filter(|p| {
            std::fs::read(p)
                .is_ok_and(|b| is_mine(&owner_at_own_path(&b, req.ctx)) && !contains_text(&b, &ico))
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
pub(super) fn pinned_script(lnks: &[PathBuf], icon: &Path, aumid: &str) -> String {
    // One shortcut that cannot be saved (locked, read-only) must not stop the
    // others nor the notification: each is its own `try`.
    let mut s = String::from("$ErrorActionPreference = 'Stop'\n");
    if !lnks.is_empty() {
        s.push_str("$sh = New-Object -ComObject WScript.Shell\n");
    }
    for lnk in lnks {
        s.push_str(&format!(
            "try {{ $p = $sh.CreateShortcut({lnk}); $p.IconLocation = {icon}; $p.Save() }} catch {{ }}\n",
            lnk = ps_quote(&lnk.to_string_lossy()),
            icon = ps_quote(&format!("{},0", icon.to_string_lossy())),
        ));
    }
    s.push_str(&aumid_script(lnks, aumid));
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
/// `SHCNE_UPDATEITEM`, `SHCNF_PATHW` and `SHCNF_FLUSHNOWAIT` (shlobj_core.h).
const SHCNE_UPDATEITEM: u32 = 0x0000_2000;
/// `SHCNE_UPDATEDIR`: the folder of a pinned shortcut changed.
const SHCNE_UPDATEDIR: u32 = 0x0000_1000;
/// Path form of the item, plus `SHCNF_FLUSHNOWAIT` (0x3000 = `SHCNF_FLUSH`
/// 0x1000 | 0x2000): Explorer handles the notification now instead of
/// queueing it, and the call still returns without waiting for it.
const SHCNF_PATHW_NOWAIT: u32 = 0x0005 | 0x3000;

/// The PowerShell that notifies the shell of `lnks` (updated items) and, when
/// `assoc`, that associations (icons) changed. Empty for nothing to say.
fn notify_script(lnks: &[PathBuf], assoc: bool) -> String {
    if lnks.is_empty() && !assoc {
        return String::new();
    }
    let mut s = String::from(NOTIFY_DECL);
    for lnk in lnks {
        s.push_str(&format!(
            "[Gp.Shell]::SHChangeNotifyPath({SHCNE_UPDATEITEM:#x}, {SHCNF_PATHW_NOWAIT:#x}, {lnk}, [IntPtr]::Zero)\n",
            lnk = ps_quote(&lnk.to_string_lossy()),
        ));
    }
    let mut dirs: Vec<&Path> = lnks.iter().filter_map(|l| l.parent()).collect();
    dirs.sort();
    dirs.dedup();
    for dir in dirs {
        s.push_str(&format!(
            "[Gp.Shell]::SHChangeNotifyPath({SHCNE_UPDATEDIR:#x}, {SHCNF_PATHW_NOWAIT:#x}, {dir}, [IntPtr]::Zero)\n",
            dir = ps_quote(&dir.to_string_lossy()),
        ));
    }
    if assoc {
        s.push_str(NOTIFY_ASSOC);
    }
    s
}

/// `SHCNE_DELETE` / `SHCNE_CREATE`: a file went away / appeared.
const SHCNE_DELETE: u32 = 0x0000_0004;
const SHCNE_CREATE: u32 = 0x0000_0002;

/// The PowerShell that tells the shell `gone` were deleted and `created` (when
/// there is one) appeared: a rename is both. Empty for nothing to say.
pub(super) fn gone_script(gone: &[PathBuf], created: Option<&Path>) -> String {
    if gone.is_empty() && created.is_none() {
        return String::new();
    }
    let mut s = String::from(NOTIFY_DECL);
    let item = |event: u32, p: &Path| {
        format!(
            "[Gp.Shell]::SHChangeNotifyPath({event:#x}, {SHCNF_PATHW_NOWAIT:#x}, {p}, [IntPtr]::Zero)\n",
            p = ps_quote(&p.to_string_lossy()),
        )
    };
    for p in gone {
        s.push_str(&item(SHCNE_DELETE, p));
    }
    if let Some(p) = created {
        s.push_str(&item(SHCNE_CREATE, p));
    }
    let mut dirs: Vec<&Path> = gone
        .iter()
        .map(PathBuf::as_path)
        .chain(created)
        .filter_map(Path::parent)
        .collect();
    dirs.sort();
    dirs.dedup();
    for dir in dirs {
        s.push_str(&item(SHCNE_UPDATEDIR, dir));
    }
    s
}

/// Best effort: re-points our shortcuts (Start menu, Desktop, and the copies
/// pinned to the taskbar) at the current `.ico` and notifies the shell. A pin
/// the user made from the Start menu entry is a copy; if Windows stripped our
/// marker AND the exe path it is not found and the user unpins and pins again
/// (documented in the settings guide).
pub(super) fn repoint_shortcuts(dirs: &Dirs, req: &Request, run: Run) {
    let lnks = shortcuts_to_repoint(dirs, req);
    let _ = run(&pinned_script(
        &lnks,
        &ico_path_for(dirs, req.icons, req.ctx),
        &terminals::aumid_for(req.ctx),
    ));
}

/// The pinned copies to rewrite, each with whether it carries our marker
/// (it was a copy of our shortcut, so its whole target is ours to rewrite:
/// `true`) or only our exe path identifies it (a shortcut the user made:
/// `false`, only the missing launcher flag is appended, the rest is theirs).
/// A copy of ours is stale when it lacks the flag, the launch helper (when
/// there is one) or the AppUserModelID; a user's copy when it lacks the flag.
pub(super) fn pinned_stale(dirs: &Dirs, req: &Request) -> Vec<(PathBuf, bool)> {
    let aumid = terminals::aumid_for(req.ctx);
    pinned_owned(dirs, req)
        .into_iter()
        .filter_map(|(p, owned)| {
            let bytes = std::fs::read(&p).ok()?;
            let flag = contains_text(&bytes, terminals::LAUNCHER_FLAG);
            let launch = names_launch(&bytes, req);
            let current = if owned {
                flag && launch && !names_legacy_exe(&bytes, req) && contains_text(&bytes, &aumid)
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
pub(super) fn pinned_args_script(
    items: &[(PathBuf, bool)],
    program: &str,
    args: &str,
    aumid: &str,
) -> String {
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
    s.push_str(&aumid_script(&owned, aumid));
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
    sync_pinned_args_with(dirs, req, &run_powershell)
}

fn sync_pinned_args_with(dirs: &Dirs, req: &Request, run: Run) -> bool {
    if !req.refresh_icon_cache {
        return false;
    }
    let items = pinned_stale(dirs, req);
    let Ok((program, args)) = target(req) else {
        return false;
    };
    !items.is_empty()
        && run(&pinned_args_script(
            &items,
            &program,
            &args,
            &terminals::aumid_for(req.ctx),
        ))
        .is_ok()
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

pub(super) fn remove(place: Place, dirs: &Dirs, ctx: &InstallCtx) -> Result<bool, String> {
    remove_with(place, dirs, ctx, &run_powershell)
}

/// Deletes this installation's own shortcut (and, once no other place keeps
/// one, its own icons). Another installation's shortcut and icons stay.
pub(super) fn remove_with(
    place: Place,
    dirs: &Dirs,
    ctx: &InstallCtx,
    run: Run,
) -> Result<bool, String> {
    let path = lnk_path(place, dirs, ctx)?;
    let Ok(bytes) = std::fs::read(&path) else {
        // No shortcut here; with none anywhere, icons left by the window or an
        // earlier shortcut go too (unless something still points at them).
        release_icons(dirs, ctx);
        return Ok(false);
    };
    if !is_mine(&owner_at_own_path(&bytes, ctx)) {
        return Err(format!("{} is not ours; not removing it", path.display()));
    }
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    let other = match place {
        Place::StartMenu => dirs.desktop.as_ref(),
        _ => dirs.start_menu.as_ref(),
    };
    if !other.is_some_and(|d| d.join(file_name(ctx)).exists()) {
        for ico in my_icos(dirs, ctx) {
            let _ = std::fs::remove_file(ico);
        }
    }
    let _ = run(&gone_script(std::slice::from_ref(&path), None));
    Ok(true)
}

/// The primary handover: `old` (the installation that was primary) keeps its
/// launchers under its own name. Each place's legacy-named `GitPoulpi.lnk` that
/// is `old`'s (tagged with its id, or untagged with the legacy marker) is
/// renamed to `<old name>.lnk`, contents untouched; `old`'s next sync rewrites
/// it. The caller then writes the new primary's own `GitPoulpi.lnk`. Returns
/// the new paths.
pub(super) fn hand_over(dirs: &Dirs, old: &InstallCtx) -> Result<Vec<PathBuf>, String> {
    hand_over_with(dirs, old, &run_powershell)
}

pub(super) fn hand_over_with(
    dirs: &Dirs,
    old: &InstallCtx,
    run: Run,
) -> Result<Vec<PathBuf>, String> {
    let named = named_file(old);
    if named == FILE {
        return Err("the old primary has no name of its own".into());
    }
    let mut moved = Vec::new();
    for dir in [dirs.start_menu.as_deref(), dirs.desktop.as_deref()]
        .into_iter()
        .flatten()
    {
        let (from, to) = (dir.join(FILE), dir.join(&named));
        let Ok(bytes) = std::fs::read(&from) else {
            continue;
        };
        let tag = read_tag(&bytes);
        let is_old = match tag.as_deref() {
            Some(t) => t == old.id,
            None => contains_text(&bytes, MARKER),
        };
        if !is_old || to.exists() {
            continue;
        }
        std::fs::rename(&from, &to).map_err(|e| format!("{}: {e}", from.display()))?;
        let _ = run(&gone_script(std::slice::from_ref(&from), Some(&to)));
        moved.push(to);
    }
    Ok(moved)
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
            ctx: crate::desktop::InstallCtx::legacy_ref(),
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
            &description(InstallCtx::legacy_ref()),
            terminals::AUMID,
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
        cleanup_icons(&dirs, &ico_path(&dirs, &new), InstallCtx::legacy_ref());
        let left = ico_files(&dirs);
        assert!(left.contains(&ico_name(&new)), "{left:?}");
        assert!(left.contains(&ico_name(&mid)), "a pin still points at it");
        assert!(!left.contains(&ico_name(&old)), "{left:?}");
        assert!(!left.contains(&legacy.to_string()), "{left:?}");
        assert!(left.contains(&"notes.ico".to_string()));
        assert!(left.contains(&"gitpoulpi-extra.ico".to_string()));
        // The pin is repointed later: now nothing names the middle one.
        std::fs::remove_file(pins.join("pin.lnk")).unwrap();
        cleanup_icons(&dirs, &ico_path(&dirs, &new), InstallCtx::legacy_ref());
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
        assert!(remove(Place::StartMenu, &dirs, InstallCtx::legacy_ref()).unwrap());
        assert_eq!(ico_files(&dirs), ["notes.ico"]);
    }

    #[test]
    fn releasing_deletes_the_primary_named_icons_nothing_names_and_keeps_the_rest() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dirs = pinned_dirs(tmp.path());
        dirs.start_menu = Some(tmp.path().join("Programs"));
        std::fs::create_dir_all(dirs.start_menu.as_ref().unwrap()).unwrap();
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let primary = InstallCtx::legacy_ref();
        let other = ctx_of(ID_A, "GitPoulpi (a)", Some(ID_B));
        // The window's icons: the primary's and another installation's.
        let (new, theirs) = (icon_set(1), icon_set(2));
        write_icon(&dirs, &req(&exe, "console", &new)).unwrap();
        let rt = req_with(&exe, &theirs, &other);
        write_icon(&dirs, &rt).unwrap();
        let mine = ico_name_for(&new, primary);
        let others = ico_name_for(&theirs, &other);
        assert_eq!(ico_files(&dirs).len(), 2);

        // The legacy GitPoulpi.lnk of somebody still names it: it stays.
        let legacy = dirs.start_menu.as_ref().unwrap().join(FILE);
        std::fs::write(&legacy, utf16(&ico_path(&dirs, &new).to_string_lossy())).unwrap();
        release_icons(&dirs, primary);
        assert!(ico_files(&dirs).contains(&mine));
        std::fs::remove_file(&legacy).unwrap();

        // A pin that names it keeps it too.
        let pin = tmp.path().join("TaskBar").join("GitPoulpi.lnk");
        std::fs::create_dir_all(pin.parent().unwrap()).unwrap();
        std::fs::write(&pin, utf16(&ico_path(&dirs, &new).to_string_lossy())).unwrap();
        release_icons(&dirs, primary);
        assert!(ico_files(&dirs).contains(&mine));
        std::fs::remove_file(&pin).unwrap();

        // Nothing names it: gone, and only that one.
        release_icons(&dirs, primary);
        assert_eq!(ico_files(&dirs), [others]);
    }

    #[test]
    fn removing_with_no_shortcut_left_still_drops_the_icons_of_ours() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dirs = pinned_dirs(tmp.path());
        dirs.start_menu = Some(tmp.path().join("Programs"));
        std::fs::create_dir_all(dirs.start_menu.as_ref().unwrap()).unwrap();
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        write_icon(&dirs, &req(&exe, "console", &icon_set(1))).unwrap();
        assert_eq!(ico_files(&dirs).len(), 1);
        assert!(!remove(Place::StartMenu, &dirs, InstallCtx::legacy_ref()).unwrap());
        assert!(ico_files(&dirs).is_empty(), "{:?}", ico_files(&dirs));
    }

    #[test]
    fn repointing_tells_the_shell_about_each_shortcut_and_keeps_the_app_id() {
        let lnk = PathBuf::from(r"C:\u\Pins\GitPoulpi.lnk");
        let s = pinned_script(
            &[lnk],
            Path::new(r"C:\gp\gitpoulpi-0123456789ab.ico"),
            terminals::AUMID,
        );
        assert!(s.contains(r"$p.IconLocation = 'C:\gp\gitpoulpi-0123456789ab.ico,0'"));
        assert!(s.contains(
            r"SHChangeNotifyPath(0x2000, 0x3005, 'C:\u\Pins\GitPoulpi.lnk', [IntPtr]::Zero)"
        ));
        assert!(s.contains("SHChangeNotify(0x08000000"));
        assert!(s.contains("[GpAumid]::Set('C:\\u\\Pins\\GitPoulpi.lnk'"));
        // Declared before it is used.
        assert!(
            s.find("Add-Type -Namespace Gp").unwrap() < s.find("SHChangeNotifyPath(0x").unwrap()
        );
        assert!(!pinned_script(&[], Path::new("x.ico"), terminals::AUMID)
            .contains("SHChangeNotifyPath(0x"));
    }

    #[test]
    fn implicit_app_shortcuts_count_as_pins_and_a_stale_pin_makes_the_icon_stale() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dirs = pinned_dirs(tmp.path());
        let implicit = tmp.path().join("Implicit");
        dirs.implicit_pinned = Some(implicit.clone());
        let sub = implicit.join("ab12");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::create_dir_all(dirs.taskbar_pinned.as_ref().unwrap()).unwrap();
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let icons = icon_set(1);
        let r = req(&exe, "console", &icons);
        std::fs::write(
            sub.join("GitPoulpi.lnk"),
            utf16(&format!("{MARKER} C:\\old.ico")),
        )
        .unwrap();
        std::fs::write(sub.join("Other.lnk"), utf16("notepad")).unwrap();
        assert_eq!(pinned_ours(&dirs, &r), [sub.join("GitPoulpi.lnk")]);
        assert_eq!(pinned_lnks(&dirs).len(), 2);
        // The Start menu entry already names the current icon, the pin does not.
        let start = tmp.path().join("Start");
        std::fs::create_dir_all(&start).unwrap();
        dirs.start_menu = Some(start.clone());
        let ico = ico_path(&dirs, &icons);
        write_icon(&dirs, &r).unwrap();
        std::fs::write(
            start.join(FILE),
            utf16(&format!("{MARKER} {}", ico.display())),
        )
        .unwrap();
        assert!(status(Place::StartMenu, &dirs, &r).icon_stale);
        std::fs::write(
            sub.join("GitPoulpi.lnk"),
            utf16(&format!("{MARKER} {}", ico.display())),
        )
        .unwrap();
        assert!(!status(Place::StartMenu, &dirs, &r).icon_stale);
    }

    #[test]
    fn one_unsaveable_pin_does_not_stop_the_others_or_the_notification() {
        let a = PathBuf::from("C:/u/Pins/A.lnk");
        let b = PathBuf::from("C:/u/Pins/B.lnk");
        let s = pinned_script(&[a, b], Path::new("x.ico"), terminals::AUMID);
        assert_eq!(s.matches("try { $p = $sh.CreateShortcut(").count(), 2);
        assert!(s.contains(r"SHChangeNotifyPath(0x1000, 0x3005, 'C:/u/Pins', [IntPtr]::Zero)"));
        assert!(s.contains("SHChangeNotify(0x08000000"));
    }

    /// Real PowerShell, real `.lnk` files: a new colour reaches the Start menu
    /// entry and every pinned copy (`TaskBar` and `ImplicitAppShortcuts`),
    /// never another app's pin, and the old icon file goes away.
    #[cfg(windows)]
    #[test]
    fn windows_a_new_colour_reaches_every_pinned_copy_and_only_ours() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dirs = pinned_dirs(tmp.path());
        dirs.start_menu = Some(tmp.path().join("Start"));
        dirs.implicit_pinned = Some(tmp.path().join("Implicit"));
        let sub = tmp.path().join("Implicit").join("ab12");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::create_dir_all(dirs.taskbar_pinned.as_ref().unwrap()).unwrap();
        let exe = tmp.path().join("gitpoulpi.exe");
        std::fs::write(&exe, b"x").unwrap();
        let (a, b) = (icon_set(1), icon_set(2));
        let mut r = req(&exe, "console", &a);
        r.refresh_icon_cache = true;

        let start = install(Place::StartMenu, &dirs, &r).unwrap();
        let pin = dirs.taskbar_pinned.as_ref().unwrap().join("GitPoulpi.lnk");
        let implicit = sub.join("GitPoulpi.lnk");
        std::fs::copy(&start, &pin).unwrap();
        std::fs::copy(&start, &implicit).unwrap();
        let other = dirs.taskbar_pinned.as_ref().unwrap().join("Notepad.lnk");
        run_powershell(&format!(
            "$s = (New-Object -ComObject WScript.Shell).CreateShortcut({}); \
             $s.TargetPath = 'C:\\Windows\\notepad.exe'; $s.IconLocation = 'C:\\Windows\\notepad.exe,0'; $s.Save()",
            ps_quote(&other.to_string_lossy())
        ))
        .unwrap();
        let other_before = std::fs::read(&other).unwrap();
        let (old_ico, new_ico) = (ico_path(&dirs, &a), ico_path(&dirs, &b));
        let names = |p: &Path, ico: &Path| {
            contains_text(&std::fs::read(p).unwrap(), &ico.to_string_lossy())
        };
        assert!(names(&pin, &old_ico), "the pin copies the Start menu entry");

        let mut rb = req(&exe, "console", &b);
        rb.refresh_icon_cache = true;
        refresh_icon(&dirs, &rb).unwrap();

        for lnk in [&start, &pin, &implicit] {
            assert!(names(lnk, &new_ico), "{} names the new icon", lnk.display());
            assert!(
                !names(lnk, &old_ico),
                "{} dropped the old icon",
                lnk.display()
            );
        }
        assert_eq!(
            std::fs::read(&other).unwrap(),
            other_before,
            "not ours: untouched"
        );
        assert!(new_ico.exists());
        assert!(!old_ico.exists(), "the old icon file is deleted");
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
        let s = pinned_script(&todo, &ico_path(&dirs, &icons), terminals::AUMID);
        assert!(s.contains("old.lnk"));
        assert!(!s.contains("new.lnk"));
        assert!(s.contains(&format!("$p.IconLocation = '{ico},0'")));
        assert!(s.contains("SHChangeNotify(0x08000000"));
        assert!(
            !pinned_script(&[], &ico_path(&dirs, &icons), terminals::AUMID)
                .contains("CreateShortcut")
        );
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
    fn a_shortcut_or_pin_naming_the_retired_tui_is_outdated_and_rewritten() {
        let tmp = tempfile::tempdir().unwrap();
        let mut dirs = pinned_dirs(tmp.path());
        dirs.start_menu = Some(tmp.path().join("Programs"));
        std::fs::create_dir_all(dirs.start_menu.as_ref().unwrap()).unwrap();
        let pins = dirs.taskbar_pinned.clone().unwrap();
        std::fs::create_dir_all(&pins).unwrap();
        let exe = PathBuf::from(r"C:\gp\gitpoulpi.exe");
        let old = r"C:\gp\gitpoulpi-tui.exe";
        let icons = IconSet::default();
        let r = req(&exe, "console", &icons);
        // Everything else current: flag, AUMID, marker; only the target is old.
        let text = |target: &str| {
            utf16(&format!(
                "{target} --launcher {} GitPoulpi {MARKER}",
                terminals::AUMID
            ))
        };
        let lnk = dirs.start_menu.as_ref().unwrap().join(FILE);
        std::fs::write(&lnk, text(&exe.to_string_lossy())).unwrap();
        assert_eq!(status(Place::StartMenu, &dirs, &r).state, State::Installed);
        std::fs::write(&lnk, text(old)).unwrap();
        assert_eq!(status(Place::StartMenu, &dirs, &r).state, State::Outdated);
        // Pins: found by the old path alone, stale while they name it.
        std::fs::write(pins.join("old.lnk"), text(old)).unwrap();
        std::fs::write(pins.join("new.lnk"), text(&exe.to_string_lossy())).unwrap();
        std::fs::write(pins.join("user.lnk"), utf16(&format!("wt.exe {old}"))).unwrap();
        let ours: Vec<_> = pinned_ours(&dirs, &r)
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert_eq!(ours, ["new.lnk", "old.lnk", "user.lnk"]);
        let stale: Vec<_> = pinned_stale(&dirs, &r)
            .into_iter()
            .map(|(p, owned)| (p.file_name().unwrap().to_string_lossy().to_string(), owned))
            .collect();
        assert_eq!(
            stale,
            [
                ("old.lnk".to_string(), true),
                ("user.lnk".to_string(), false)
            ]
        );
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
        let s = pinned_args_script(&todo, &program, &args, terminals::AUMID);
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
        let s = aumid_script(
            &[
                PathBuf::from(r"C:\u\Start\GitPoulpi.lnk"),
                PathBuf::from(r"C:\u\Pins\O'Neil.lnk"),
            ],
            terminals::AUMID,
        );
        assert!(
            s.contains("9F4C2855-9F79-4B39-A8D0-E1D42DE1D5F3"),
            "PKEY_AppUserModel_ID"
        );
        assert!(s.contains("pid = 5"));
        assert!(s.contains(r"[GpAumid]::Set('C:\u\Start\GitPoulpi.lnk', 'GitPoulpi.GitPoulpi')"));
        assert!(s.contains(r"O''Neil.lnk"));
        // Every call, and the type itself, are inside try/catch.
        assert_eq!(s.matches("catch").count(), 3);
        assert!(aumid_script(&[], terminals::AUMID).is_empty());
        // The install script carries it.
        let full = script(
            Path::new(r"C:\u\GitPoulpi.lnk"),
            "x.exe",
            "",
            Path::new(r"C:\gp"),
            Path::new(r"C:\gp\i.ico"),
            &description(InstallCtx::legacy_ref()),
            terminals::AUMID,
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
        assert!(remove(Place::StartMenu, &dirs, InstallCtx::legacy_ref()).is_err());

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
        assert!(remove(Place::StartMenu, &dirs, InstallCtx::legacy_ref()).unwrap());
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

    // ---- several installations on one machine -------------------------------

    use std::cell::RefCell;

    const ID_A: &str = "9c1e0b7a55d2e3f4";
    const ID_B: &str = "1111222233334444";

    fn ctx_of(id: &str, name: &str, primary: Option<&str>) -> InstallCtx {
        let exists = |i: &str| crate::desktop::KnownInstall {
            id: i.to_string(),
            exists: true,
        };
        InstallCtx {
            id: id.to_string(),
            short: format!("i{}", &id[..8]),
            name: name.to_string(),
            primary: primary.map(str::to_string),
            alias_claims: Vec::new(),
            known: vec![exists(ID_A), exists(ID_B)],
        }
    }

    fn req_with<'a>(exe: &'a Path, icons: &'a IconSet, ctx: &'a InstallCtx) -> Request<'a> {
        Request {
            ctx,
            ..req(exe, "console", icons)
        }
    }

    /// A PowerShell that records its scripts and, for a shortcut script, writes
    /// the `.lnk` as the script's own text (UTF-16), so what the real spawn
    /// would store (description, target, icon, AppUserModelID) can be read back.
    fn fake(log: &RefCell<Vec<String>>) -> impl Fn(&str) -> Result<(), String> + '_ {
        move |script: &str| {
            log.borrow_mut().push(script.to_string());
            if script.contains("$s.Save()") {
                let from = script.find("CreateShortcut('").unwrap() + "CreateShortcut('".len();
                let len = script[from..].find("')").unwrap();
                let lnk = PathBuf::from(script[from..from + len].replace("''", "'"));
                std::fs::create_dir_all(lnk.parent().unwrap()).unwrap();
                // Like a real .lnk, the description is preceded by its count
                // (a character of that code, right before the text).
                let mut text = script.to_string();
                if let Some(d) = script.find("$s.Description = '") {
                    let d = d + "$s.Description = '".len();
                    let n = script[d..].find('\'').unwrap_or(0) as u32;
                    text.insert(d, char::from_u32(n).unwrap());
                }
                std::fs::write(lnk, utf16(&text)).unwrap();
            }
            Ok(())
        }
    }

    struct Machine {
        _tmp: tempfile::TempDir,
        dirs: Dirs,
        exe: PathBuf,
    }

    fn machine() -> Machine {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = Dirs {
            start_menu: Some(tmp.path().join("Programs")),
            desktop: Some(tmp.path().join("Desktop")),
            taskbar_pinned: Some(tmp.path().join("TaskBar")),
            assets: tmp.path().join("assets"),
            ..Dirs::default()
        };
        for d in [
            dirs.start_menu.as_ref().unwrap(),
            dirs.desktop.as_ref().unwrap(),
            dirs.taskbar_pinned.as_ref().unwrap(),
        ] {
            std::fs::create_dir_all(d).unwrap();
        }
        let exe = tmp.path().join("apps").join("gitpoulpi.exe");
        Machine {
            _tmp: tmp,
            dirs,
            exe,
        }
    }

    fn start(m: &Machine) -> &Path {
        m.dirs.start_menu.as_deref().unwrap()
    }

    /// A `.lnk` StringData field: 16-bit count, then the UTF-16 characters.
    fn string_data(text: &str) -> Vec<u8> {
        let mut out = (text.encode_utf16().count() as u16).to_le_bytes().to_vec();
        out.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
        out
    }

    #[test]
    fn the_tag_is_bounded_by_its_stringdata_count_not_by_the_next_field() {
        // The next field's count is 0x35 ('5'): an id character right after the tag.
        for next in [0x35u16, 0x61, 0x30] {
            let mut b = vec![0u8; 8];
            b.extend(string_data(&format!("GitPoulpi {OWNER_TAG}={ID_A}")));
            b.extend(next.to_le_bytes());
            b.extend(
                "C:\\Program Files\\GitPoulpi"
                    .encode_utf16()
                    .flat_map(u16::to_le_bytes),
            );
            assert_eq!(read_tag(&b).as_deref(), Some(ID_A), "next count {next:#x}");
        }
        // Without a locatable description the hashed id is cut to 16 digits.
        let b = format!("{OWNER_TAG}={ID_A}5").into_bytes();
        assert_eq!(read_tag(&b).as_deref(), Some(ID_A));
        // A pinned id is read whole.
        let mut b = string_data(&format!("GitPoulpi {OWNER_TAG}=p-pty"));
        b.extend(0x2eu16.to_le_bytes());
        assert_eq!(read_tag(&b).as_deref(), Some("p-pty"));
    }

    #[test]
    fn a_desktop_shortcut_whose_next_field_count_is_an_id_character_stays_ours() {
        // A real .lnk: the Description StringData, then the next field's count
        // (0x35) and a target path. The fake writes that layout.
        let log = RefCell::new(Vec::new());
        let m = machine();
        let exe_text = m.exe.display().to_string();
        let write_real = |script: &str| -> Result<(), String> {
            log.borrow_mut().push(script.to_string());
            if script.contains("$s.Save()") {
                let from = script.find("CreateShortcut('").unwrap() + "CreateShortcut('".len();
                let len = script[from..].find("')").unwrap();
                let lnk = PathBuf::from(script[from..from + len].replace("''", "'"));
                let d = script.find("$s.Description = '").unwrap() + "$s.Description = '".len();
                let desc = &script[d..d + script[d..].find('\'').unwrap()];
                let mut b = vec![0u8; 76];
                b.extend(string_data(desc));
                b.extend(0x35u16.to_le_bytes());
                b.extend(exe_text.encode_utf16().flat_map(u16::to_le_bytes));
                // The rest of the facts (arguments, icon, AppUserModelID).
                b.extend(utf16(script));
                std::fs::create_dir_all(lnk.parent().unwrap()).unwrap();
                std::fs::write(lnk, b).unwrap();
            }
            Ok(())
        };
        let icons = icon_set(1);
        let ctx = ctx_of(ID_A, "GitPoulpi (dev)", None);
        let r = req_with(&m.exe, &icons, &ctx);
        let path = install_with(Place::Desktop, &m.dirs, &r, &write_real).unwrap();
        assert_eq!(status(Place::Desktop, &m.dirs, &r).state, State::Installed);
        // A second install is not refused as foreign.
        install_with(Place::Desktop, &m.dirs, &r, &write_real).unwrap();
        assert!(remove_with(Place::Desktop, &m.dirs, &ctx, &write_real).unwrap());
        assert!(!path.exists());
    }

    #[test]
    fn a_non_primary_shortcut_is_named_after_its_installation_and_tagged() {
        let m = machine();
        let icons = icon_set(1);
        let ctx = ctx_of(ID_A, "GitPoulpi (dev)", None);
        let log = RefCell::new(Vec::new());
        let r = req_with(&m.exe, &icons, &ctx);
        let path = install_with(Place::StartMenu, &m.dirs, &r, &fake(&log)).unwrap();
        assert_eq!(path, start(&m).join("GitPoulpi (dev).lnk"));
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(read_tag(&bytes).as_deref(), Some(ID_A));
        assert!(contains_text(
            &bytes,
            &format!("GitPoulpi {OWNER_TAG}={ID_A}'")
        ));
        // Its own AppUserModelID, its own icon name.
        assert!(contains_text(&bytes, &format!("GitPoulpi.I{ID_A}")));
        assert!(contains_text(&bytes, "gitpoulpi-i9c1e0b7a-"));
        assert_eq!(
            status(Place::StartMenu, &m.dirs, &r).state,
            State::Installed
        );
        // The primary keeps the legacy name, marker and AppUserModelID.
        let primary = ctx_of(ID_B, "GitPoulpi", Some(ID_B));
        let rp = req_with(&m.exe, &icons, &primary);
        let p = install_with(Place::StartMenu, &m.dirs, &rp, &fake(&log)).unwrap();
        assert_eq!(p, start(&m).join(FILE));
        let pb = std::fs::read(&p).unwrap();
        assert!(contains_text(&pb, MARKER) && contains_text(&pb, terminals::AUMID));
        assert_eq!(read_tag(&pb).as_deref(), Some(ID_B));
    }

    #[test]
    fn non_primary_bytes_never_contain_the_legacy_marker() {
        let m = machine();
        let icons = icon_set(1);
        let ctx = ctx_of(ID_A, "GitPoulpi (dev)", Some(ID_B));
        let log = RefCell::new(Vec::new());
        let r = req_with(&m.exe, &icons, &ctx);
        for place in [Place::StartMenu, Place::Desktop] {
            let path = install_with(place, &m.dirs, &r, &fake(&log)).unwrap();
            assert!(
                !contains_text(&std::fs::read(path).unwrap(), MARKER),
                "{place:?}"
            );
        }
        assert!(!description(&ctx).contains(MARKER));
        for s in log.borrow().iter() {
            assert!(!s.contains(MARKER), "{s}");
        }
        // Old builds look for exactly the marker: nothing of ours matches.
        assert!(description(&ctx_of(ID_B, "GitPoulpi", Some(ID_B))).contains(MARKER));
    }

    #[test]
    fn sync_never_rewrites_another_installations_shortcut() {
        let m = machine();
        let icons = icon_set(1);
        let ctx = ctx_of(ID_A, "GitPoulpi (dev)", None);
        let r = req_with(&m.exe, &icons, &ctx);
        let theirs = start(&m).join("GitPoulpi (B).lnk");
        let before = utf16(&format!("other.exe GitPoulpi {OWNER_TAG}={ID_B} old.ico"));
        std::fs::write(&theirs, &before).unwrap();
        // The same name, but tagged with another installation: not mine.
        let clash = start(&m).join("GitPoulpi (dev).lnk");
        std::fs::write(&clash, &before).unwrap();
        assert_eq!(status(Place::StartMenu, &m.dirs, &r).state, State::Foreign);
        let log = RefCell::new(Vec::new());
        assert!(install_with(Place::StartMenu, &m.dirs, &r, &fake(&log)).is_err());
        assert!(remove_with(Place::StartMenu, &m.dirs, &ctx, &fake(&log)).is_err());
        assert_eq!(std::fs::read(&clash).unwrap(), before);
        // Free the name: only my own file is written, theirs is never touched.
        std::fs::remove_file(&clash).unwrap();
        install_with(Place::StartMenu, &m.dirs, &r, &fake(&log)).unwrap();
        assert_eq!(std::fs::read(&theirs).unwrap(), before);
        for s in log.borrow().iter() {
            assert!(!s.contains("GitPoulpi (B).lnk"), "{s}");
        }
        assert!(shortcuts_to_repoint(&m.dirs, &r).is_empty());
    }

    #[test]
    fn legacy_shortcut_is_untouched_without_a_primary() {
        let m = machine();
        let icons = icon_set(1);
        let ctx = ctx_of(ID_A, "GitPoulpi (dev)", None);
        let mut r = req_with(&m.exe, &icons, &ctx);
        r.refresh_icon_cache = true;
        let legacy = start(&m).join(FILE);
        let before = utf16(&format!("{} GitPoulpi {MARKER} old.ico", m.exe.display()));
        std::fs::write(&legacy, &before).unwrap();
        let pin = m
            .dirs
            .taskbar_pinned
            .as_ref()
            .unwrap()
            .join("GitPoulpi.lnk");
        std::fs::write(&pin, &before).unwrap();
        let log = RefCell::new(Vec::new());
        install_with(Place::StartMenu, &m.dirs, &r, &fake(&log)).unwrap();
        install_with(Place::Desktop, &m.dirs, &r, &fake(&log)).unwrap();
        refresh_icon(&m.dirs, &r).ok();
        assert_eq!(std::fs::read(&legacy).unwrap(), before);
        assert_eq!(std::fs::read(&pin).unwrap(), before);
        assert!(pinned_ours(&m.dirs, &r).is_empty());
        assert!(shortcuts_to_repoint(&m.dirs, &r).is_empty());
        assert!(!sync_pinned_args_with(&m.dirs, &r, &fake(&log)));
        for s in log.borrow().iter() {
            assert!(!s.contains("GitPoulpi.lnk"), "{s}");
        }
        // Removing mine leaves it alone too.
        assert!(remove_with(Place::StartMenu, &m.dirs, &ctx, &fake(&log)).unwrap());
        assert_eq!(std::fs::read(&legacy).unwrap(), before);
    }

    #[test]
    fn pins_of_another_installation_are_not_repointed() {
        let m = machine();
        let icons = icon_set(1);
        let ctx = ctx_of(ID_A, "GitPoulpi (dev)", Some(ID_B));
        let mut r = req_with(&m.exe, &icons, &ctx);
        r.refresh_icon_cache = true;
        let pins = m.dirs.taskbar_pinned.clone().unwrap();
        let exe = m.exe.display();
        // Mine (tagged), B's (tagged), a legacy one (B is the primary), a pin
        // the user made by hand naming my exe, and an unrelated one.
        std::fs::write(
            pins.join("a-mine.lnk"),
            [
                utf16(&format!("{exe} ")),
                string_data(&format!("GitPoulpi {OWNER_TAG}={ID_A}")),
                utf16(" old.ico"),
            ]
            .concat(),
        )
        .unwrap();
        let theirs = [
            utf16("b.exe "),
            string_data(&format!("GitPoulpi {OWNER_TAG}={ID_B}")),
            utf16(" old.ico"),
        ]
        .concat();
        std::fs::write(pins.join("b-theirs.lnk"), &theirs).unwrap();
        let legacy = utf16(&format!("{exe} GitPoulpi {MARKER} old.ico"));
        std::fs::write(pins.join("c-legacy.lnk"), &legacy).unwrap();
        std::fs::write(
            pins.join("d-user.lnk"),
            utf16(&format!("wt.exe -p X {exe}")),
        )
        .unwrap();
        std::fs::write(pins.join("e-other.lnk"), utf16("notepad.exe")).unwrap();
        let names = |v: Vec<PathBuf>| -> Vec<String> {
            v.iter()
                .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                .collect()
        };
        assert_eq!(
            names(pinned_ours(&m.dirs, &r)),
            ["a-mine.lnk", "d-user.lnk"]
        );
        let log = RefCell::new(Vec::new());
        repoint_shortcuts(&m.dirs, &r, &fake(&log));
        assert!(sync_pinned_args_with(&m.dirs, &r, &fake(&log)));
        let all = log.borrow().join("\n");
        assert!(
            all.contains("a-mine.lnk") && all.contains("d-user.lnk"),
            "{all}"
        );
        for other in ["b-theirs.lnk", "c-legacy.lnk", "e-other.lnk"] {
            assert!(!all.contains(other), "{other} was touched: {all}");
        }
        assert_eq!(std::fs::read(pins.join("b-theirs.lnk")).unwrap(), theirs);
        assert_eq!(std::fs::read(pins.join("c-legacy.lnk")).unwrap(), legacy);
        // The primary has its own pin and adopts the untagged legacy one.
        let primary = ctx_of(ID_B, "GitPoulpi", Some(ID_B));
        let rb = req_with(&m.exe, &icons, &primary);
        assert_eq!(
            names(pinned_ours(&m.dirs, &rb)),
            ["b-theirs.lnk", "c-legacy.lnk", "d-user.lnk"]
        );
    }

    #[test]
    fn icon_cleanup_keeps_another_installations_icons() {
        let m = machine();
        let (old, new) = (icon_set(1), icon_set(2));
        let a = ctx_of(ID_A, "GitPoulpi (dev)", None);
        let b = ctx_of(ID_B, "GitPoulpi (cargo)", None);
        let primary = ctx_of(ID_B, "GitPoulpi", Some(ID_B));
        // Names: `i<8hex>-<12hex>` for the others, the old shape for the primary.
        let name_a = ico_name_for(&new, &a);
        assert!(
            is_my_ico(&name_a, &a) && !is_my_ico(&name_a, &b),
            "{name_a}"
        );
        assert!(name_a.starts_with("gitpoulpi-i9c1e0b7a-"), "{name_a}");
        let name_p = ico_name_for(&new, &primary);
        assert!(is_our_ico(&name_p) && !is_our_ico(&name_a), "{name_p}");
        assert!(!is_my_ico(&name_p, &a));
        std::fs::create_dir_all(&m.dirs.assets).unwrap();
        let mut files = vec!["gitpoulpi.ico".to_string(), "notes.ico".to_string()];
        for set in [&old, &new] {
            for c in [&a, &b, &primary] {
                files.push(ico_name_for(set, c));
            }
        }
        for f in &files {
            std::fs::write(m.dirs.assets.join(f), b"x").unwrap();
        }
        cleanup_icons(&m.dirs, &ico_path_for(&m.dirs, &new, &a), &a);
        let left = ico_files(&m.dirs);
        // Mine: the old one is gone, the current one stays.
        assert!(!left.contains(&ico_name_for(&old, &a)), "{left:?}");
        assert!(left.contains(&ico_name_for(&new, &a)), "{left:?}");
        // Everybody else's, the primary's and the legacy one included, stay.
        assert!(left.contains(&ico_name_for(&old, &b)), "{left:?}");
        assert!(left.contains(&ico_name_for(&new, &b)), "{left:?}");
        assert!(left.contains(&ico_name_for(&old, &primary)), "{left:?}");
        for keep in ["gitpoulpi.ico", "notes.ico"] {
            assert!(left.contains(&keep.to_string()), "{left:?}");
        }
        // The primary cleans only its own shape.
        cleanup_icons(&m.dirs, &ico_path_for(&m.dirs, &new, &primary), &primary);
        let left = ico_files(&m.dirs);
        assert!(!left.contains(&ico_name_for(&old, &primary)), "{left:?}");
        assert!(!left.contains(&"gitpoulpi.ico".to_string()), "{left:?}");
        assert!(left.contains(&ico_name_for(&old, &b)), "{left:?}");
    }

    #[test]
    fn a_version_change_renames_only_my_shortcut() {
        let m = machine();
        let icons = icon_set(1);
        let before = ctx_of(ID_A, "GitPoulpi (Scoop)", None);
        let after = ctx_of(ID_A, "GitPoulpi 1.2 (Scoop)", None);
        let log = RefCell::new(Vec::new());
        let old = install_with(
            Place::StartMenu,
            &m.dirs,
            &req_with(&m.exe, &icons, &before),
            &fake(&log),
        )
        .unwrap();
        let theirs = start(&m).join("GitPoulpi (B).lnk");
        let theirs_bytes = utf16(&format!("b.exe GitPoulpi {OWNER_TAG}={ID_B}"));
        std::fs::write(&theirs, &theirs_bytes).unwrap();
        let legacy = start(&m).join(FILE);
        std::fs::write(&legacy, utf16(&format!("GitPoulpi {MARKER}"))).unwrap();
        log.borrow_mut().clear();
        let new = install_with(
            Place::StartMenu,
            &m.dirs,
            &req_with(&m.exe, &icons, &after),
            &fake(&log),
        )
        .unwrap();
        assert_eq!(new, start(&m).join("GitPoulpi 1.2 (Scoop).lnk"));
        assert!(new.exists() && !old.exists(), "renamed, not duplicated");
        assert_eq!(std::fs::read(&theirs).unwrap(), theirs_bytes);
        assert!(legacy.exists());
        // The shell hears about the deletion and the creation.
        let all = log.borrow().join("\n");
        assert!(
            all.contains(&format!(
                "SHChangeNotifyPath(0x4, 0x3005, '{}'",
                old.display()
            )),
            "{all}"
        );
        assert!(
            all.contains(&format!(
                "SHChangeNotifyPath(0x2, 0x3005, '{}'",
                new.display()
            )),
            "{all}"
        );
        assert!(!all.contains("GitPoulpi (B).lnk"), "{all}");
    }

    #[test]
    fn handover_renames_the_old_primarys_lnk_without_rewriting_it() {
        let m = machine();
        let old = ctx_of(ID_A, "GitPoulpi (dev)", Some(ID_A));
        let tagged = utf16(&format!(
            "a.exe GitPoulpi {MARKER} {OWNER_TAG}={ID_A} i.ico"
        ));
        let untagged = utf16(&format!("a.exe GitPoulpi {MARKER} i.ico"));
        let desktop = m.dirs.desktop.clone().unwrap();
        std::fs::write(start(&m).join(FILE), &tagged).unwrap();
        std::fs::write(desktop.join(FILE), &untagged).unwrap();
        let log = RefCell::new(Vec::new());
        let moved = hand_over_with(&m.dirs, &old, &fake(&log)).unwrap();
        assert_eq!(
            moved,
            [
                start(&m).join("GitPoulpi (dev).lnk"),
                desktop.join("GitPoulpi (dev).lnk")
            ]
        );
        assert!(!start(&m).join(FILE).exists() && !desktop.join(FILE).exists());
        assert_eq!(
            std::fs::read(&moved[0]).unwrap(),
            tagged,
            "contents untouched"
        );
        assert_eq!(std::fs::read(&moved[1]).unwrap(), untagged);
        // Nothing was written: the shell was only told.
        for s in log.borrow().iter() {
            assert!(
                !s.contains("CreateShortcut") && !s.contains("Save()"),
                "{s}"
            );
            assert!(s.contains("SHChangeNotifyPath(0x4,"), "{s}");
        }
        // After the handover B is primary; A still owns its renamed files
        // (tagged, or untagged at its own name) and is free to fix them.
        let a_now = ctx_of(ID_A, "GitPoulpi (dev)", Some(ID_B));
        let icons = icon_set(1);
        for place in [Place::StartMenu, Place::Desktop] {
            let st = status(place, &m.dirs, &req_with(&m.exe, &icons, &a_now));
            assert_ne!(st.state, State::Foreign, "{place:?}");
        }
        // A file of someone else, or a name already taken, is not renamed.
        for p in &moved {
            std::fs::remove_file(p).unwrap();
        }
        std::fs::write(
            start(&m).join(FILE),
            utf16(&format!("GitPoulpi {MARKER} {OWNER_TAG}={ID_B}")),
        )
        .unwrap();
        assert!(hand_over_with(&m.dirs, &old, &fake(&log))
            .unwrap()
            .is_empty());
        assert!(start(&m).join(FILE).exists());
        std::fs::write(desktop.join(FILE), &untagged).unwrap();
        std::fs::write(&moved[1], b"taken").unwrap();
        assert!(hand_over_with(&m.dirs, &old, &fake(&log))
            .unwrap()
            .is_empty());
        assert!(desktop.join(FILE).exists(), "never overwrites a file");
    }

    #[test]
    fn remove_deletes_only_my_shortcut_and_icons() {
        let m = machine();
        let icons = icon_set(1);
        let a = ctx_of(ID_A, "GitPoulpi (dev)", None);
        let b = ctx_of(ID_B, "GitPoulpi (cargo)", None);
        let log = RefCell::new(Vec::new());
        for c in [&a, &b] {
            for place in [Place::StartMenu, Place::Desktop] {
                install_with(place, &m.dirs, &req_with(&m.exe, &icons, c), &fake(&log)).unwrap();
            }
        }
        let desktop = m.dirs.desktop.clone().unwrap();
        let legacy = start(&m).join(FILE);
        std::fs::write(&legacy, utf16(&format!("GitPoulpi {MARKER}"))).unwrap();
        std::fs::write(m.dirs.assets.join("notes.ico"), b"mine").unwrap();
        let (ico_a, ico_b) = (ico_name_for(&icons, &a), ico_name_for(&icons, &b));
        assert!(m.dirs.assets.join(&ico_a).exists() && m.dirs.assets.join(&ico_b).exists());
        log.borrow_mut().clear();
        // One place: my icon stays while the other place still has my shortcut.
        assert!(remove_with(Place::StartMenu, &m.dirs, &a, &fake(&log)).unwrap());
        assert!(!start(&m).join("GitPoulpi (dev).lnk").exists());
        assert!(start(&m).join("GitPoulpi (cargo).lnk").exists());
        assert!(m.dirs.assets.join(&ico_a).exists());
        assert!(log
            .borrow()
            .iter()
            .any(|s| s.contains("SHChangeNotifyPath(0x4,")));
        // The last one: my icons go, nobody else's.
        assert!(remove_with(Place::Desktop, &m.dirs, &a, &fake(&log)).unwrap());
        assert!(!desktop.join("GitPoulpi (dev).lnk").exists());
        assert!(!m.dirs.assets.join(&ico_a).exists());
        assert!(m.dirs.assets.join(&ico_b).exists());
        assert!(m.dirs.assets.join("notes.ico").exists());
        assert!(desktop.join("GitPoulpi (cargo).lnk").exists());
        assert!(legacy.exists(), "the legacy file is not mine");
        // Nothing left to remove.
        assert!(!remove_with(Place::Desktop, &m.dirs, &a, &fake(&log)).unwrap());
    }
}
