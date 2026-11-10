//! The Windows registry reads and writes the updater and the TUI need: the
//! Inno uninstall key (its `InstallLocation` names the install folder, and its
//! `DisplayVersion` must follow every swap) and the administrators' update
//! policy (`HKLM\SOFTWARE\Policies\GitPoulpi\Updates`).
//!
//! Trap: Inno writes under the 32-bit view unless the script asks for the
//! 64-bit one, and which one a given install used is not recorded, so both
//! views are tried (64-bit first) for every read and write.

use std::collections::BTreeMap;

use windows_sys::Win32::Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegEnumValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, KEY_WOW64_32KEY,
    KEY_WOW64_64KEY, REG_DWORD, REG_EXPAND_SZ, REG_SZ,
};

use crate::channel::{Hive, RegistryOps};

/// The uninstall key of the Inno installer (`AppId` in `gitpoulpi.iss`).
pub const INNO_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{8C1D3E5A-6B7F-4A29-9E0C-47A1B2C3D4E5}_is1";

/// Where administrators put the update policy, with the policy file's names.
pub const POLICY_KEY: &str = r"SOFTWARE\Policies\GitPoulpi\Updates";

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn root(hive: Hive) -> HKEY {
    match hive {
        Hive::CurrentUser => HKEY_CURRENT_USER,
        Hive::LocalMachine => HKEY_LOCAL_MACHINE,
    }
}

struct Key(HKEY);

impl Drop for Key {
    fn drop(&mut self) {
        // SAFETY: the handle came from a successful RegOpenKeyExW.
        unsafe { RegCloseKey(self.0) };
    }
}

/// Opens `sub` under `root` in the 64-bit view, else the 32-bit one.
fn open(root: HKEY, sub: &str, access: u32) -> Option<Key> {
    let sub = wide(sub);
    for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
        let mut handle: HKEY = std::ptr::null_mut();
        // SAFETY: `sub` is NUL-terminated and outlives the call; `handle` is
        // a valid out pointer.
        let rc = unsafe { RegOpenKeyExW(root, sub.as_ptr(), 0, access | view, &mut handle) };
        if rc == ERROR_SUCCESS {
            return Some(Key(handle));
        }
    }
    None
}

fn utf16_text(buf: &[u16], bytes: u32) -> String {
    let n = (bytes as usize / 2).min(buf.len());
    let text = String::from_utf16_lossy(&buf[..n]);
    text.trim_end_matches('\0').to_string()
}

fn read_string(key: &Key, name: &str) -> Option<String> {
    let name = wide(name);
    let mut kind = 0u32;
    let mut len = 0u32;
    // SAFETY: a size query: no data pointer, valid out pointers.
    let rc = unsafe {
        RegQueryValueExW(
            key.0,
            name.as_ptr(),
            std::ptr::null(),
            &mut kind,
            std::ptr::null_mut(),
            &mut len,
        )
    };
    if rc != ERROR_SUCCESS || (kind != REG_SZ && kind != REG_EXPAND_SZ) {
        return None;
    }
    let mut buf = vec![0u16; len as usize / 2 + 1];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: `buf` holds `size` bytes.
    let rc = unsafe {
        RegQueryValueExW(
            key.0,
            name.as_ptr(),
            std::ptr::null(),
            &mut kind,
            buf.as_mut_ptr().cast::<u8>(),
            &mut size,
        )
    };
    (rc == ERROR_SUCCESS).then(|| utf16_text(&buf, size))
}

/// The real [`RegistryOps`].
pub struct WindowsRegistry;

impl RegistryOps for WindowsRegistry {
    fn inno_install_location(&self, hive: Hive) -> Option<String> {
        let key = open(root(hive), INNO_KEY, KEY_READ)?;
        read_string(&key, "InstallLocation")
    }
}

impl WindowsRegistry {
    /// Writes `DisplayVersion` of the Inno uninstall key in `hive`.
    pub fn set_display_version(&self, hive: Hive, version: &str) -> Result<(), String> {
        let key = open(root(hive), INNO_KEY, KEY_SET_VALUE)
            .ok_or_else(|| "the uninstall entry could not be opened".to_string())?;
        let name = wide("DisplayVersion");
        let data = wide(version);
        // SAFETY: `data` is NUL-terminated UTF-16 and `cbdata` counts its bytes.
        let rc = unsafe {
            RegSetValueExW(
                key.0,
                name.as_ptr(),
                0,
                REG_SZ,
                data.as_ptr().cast::<u8>(),
                (data.len() * 2) as u32,
            )
        };
        if rc == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!("registry error {rc}"))
        }
    }
}

/// The values of the machine policy key as text (`REG_SZ`, and `REG_DWORD`
/// as `1`/`0`-style numbers), names lower-cased. `None` without the key.
pub fn policy_values() -> Option<BTreeMap<String, String>> {
    let key = open(HKEY_LOCAL_MACHINE, POLICY_KEY, KEY_READ)?;
    let mut out = BTreeMap::new();
    for index in 0u32.. {
        let mut name = [0u16; 256];
        let mut name_len = name.len() as u32;
        let mut kind = 0u32;
        let mut data = [0u8; 4096];
        let mut data_len = data.len() as u32;
        // SAFETY: both buffers are as large as the lengths passed alongside.
        let rc = unsafe {
            RegEnumValueW(
                key.0,
                index,
                name.as_mut_ptr(),
                &mut name_len,
                std::ptr::null(),
                &mut kind,
                data.as_mut_ptr(),
                &mut data_len,
            )
        };
        if rc == ERROR_NO_MORE_ITEMS {
            break;
        }
        if rc != ERROR_SUCCESS {
            // A value too big for the buffer is not one of ours.
            continue;
        }
        let name = utf16_text(&name, name_len * 2).to_lowercase();
        let text = match kind {
            REG_SZ | REG_EXPAND_SZ => {
                let wide: Vec<u16> = data[..data_len as usize]
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|c| u16::from_le_bytes(*c))
                    .collect();
                utf16_text(&wide, data_len)
            }
            REG_DWORD if data_len >= 4 => {
                u32::from_le_bytes([data[0], data[1], data[2], data[3]]).to_string()
            }
            _ => continue,
        };
        out.insert(name, text);
    }
    Some(out)
}
