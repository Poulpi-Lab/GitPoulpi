//! The per-user Windows registry as a seam: `HKCU` keys and string values,
//! read and changed through the [`Registry`] trait. The real store runs
//! PowerShell ([`PowerShellRegistry`]); tests use [`MemoryRegistry`], and a
//! hand or PTY run sets `GITPOULPI_TEST_REGISTRY_FILE` to use a
//! [`FileRegistry`]. The logic that decides what to write is plain data, so it
//! compiles and is tested on every OS and never touches the real registry.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Environment variable naming a file that stands in for the registry.
pub const TEST_REGISTRY_FILE_ENV: &str = "GITPOULPI_TEST_REGISTRY_FILE";

/// `("", v)` is the key's (Default) value.
pub type Values = Vec<(String, String)>;

/// One change, applied in order. Keys are relative to `HKCU`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegOp {
    /// Creates the key if needed and sets a string value.
    Set {
        key: String,
        name: String,
        value: String,
    },
    /// Deletes the key and everything below it.
    DeleteTree(String),
    /// Deletes the key only when it has no subkeys and no values.
    DeleteIfEmpty(String),
}

pub trait Registry {
    /// For each key: `None` when absent, else its values and whether it has
    /// subkeys.
    fn read(&self, keys: &[String]) -> Result<Vec<Option<(Values, bool)>>, String>;
    fn apply(&self, ops: &[RegOp]) -> Result<(), String>;
}

/// The shared handle the callers hold.
pub type SharedRegistry = Arc<dyn Registry + Send + Sync>;

/// The real registry, or a file when [`TEST_REGISTRY_FILE_ENV`] is set.
pub fn from_env() -> SharedRegistry {
    match std::env::var_os(TEST_REGISTRY_FILE_ENV).filter(|v| !v.is_empty()) {
        Some(p) => Arc::new(FileRegistry(PathBuf::from(p))),
        None => Arc::new(PowerShellRegistry),
    }
}

pub(super) fn run_powershell(script: &str, envs: &[(&str, &str)]) -> Result<String, String> {
    let mut cmd = std::process::Command::new("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-EncodedCommand",
        &super::windows::encode_command(script),
    ]);
    cmd.envs(envs.iter().copied());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let out = cmd.output().map_err(|e| format!("PowerShell: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

fn check(s: &str) -> Result<(), String> {
    if s.contains(['\t', '\n', '\r']) {
        return Err("a registry value cannot hold a tab or a line break".to_string());
    }
    Ok(())
}

fn ops_text(ops: &[RegOp]) -> Result<String, String> {
    let mut out = String::new();
    for op in ops {
        match op {
            RegOp::Set { key, name, value } => {
                for s in [key, name, value] {
                    check(s)?;
                }
                out.push_str(&format!("S\t{key}\t{name}\t{value}\n"));
            }
            RegOp::DeleteTree(k) => {
                check(k)?;
                out.push_str(&format!("T\t{k}\n"));
            }
            RegOp::DeleteIfEmpty(k) => {
                check(k)?;
                out.push_str(&format!("E\t{k}\n"));
            }
        }
    }
    Ok(out)
}

const APPLY_SCRIPT: &str = "$ErrorActionPreference='Stop';\
$root=[Microsoft.Win32.Registry]::CurrentUser;\
foreach($l in ($env:GP_OPS -split \"`n\")){\
if($l -eq ''){continue};\
$f=$l -split \"`t\",4;\
if($f[0] -eq 'S'){$k=$root.CreateSubKey($f[1]);$k.SetValue($f[2],$f[3],[Microsoft.Win32.RegistryValueKind]::String);$k.Close()}\
elseif($f[0] -eq 'T'){$root.DeleteSubKeyTree($f[1],$false)}\
elseif($f[0] -eq 'E'){$k=$root.OpenSubKey($f[1]);if($k){$e=($k.SubKeyCount -eq 0 -and $k.ValueCount -eq 0);$k.Close();if($e){$root.DeleteSubKey($f[1],$false)}}}}";

const READ_SCRIPT: &str = "$root=[Microsoft.Win32.Registry]::CurrentUser;\
foreach($key in ($env:GP_KEYS -split \"`n\")){\
if($key -eq ''){continue};\
$k=$root.OpenSubKey($key);\
if($k -eq $null){[Console]::Out.Write('K'+\"`t\"+$key+\"`t\"+'-'+\"`n\");continue};\
$sub=0;if($k.SubKeyCount -gt 0){$sub=1};\
[Console]::Out.Write('K'+\"`t\"+$key+\"`t\"+$sub+\"`n\");\
foreach($n in $k.GetValueNames()){$v=$k.GetValue($n,$null,[Microsoft.Win32.RegistryValueOptions]::DoNotExpandEnvironmentNames);\
[Console]::Out.Write('V'+\"`t\"+$n+\"`t\"+$v+\"`n\")};\
$k.Close()}";

/// Parses [`READ_SCRIPT`]'s output into one entry per requested key, in order.
pub fn parse_read(keys: &[String], out: &str) -> Vec<Option<(Values, bool)>> {
    let mut found: BTreeMap<String, Option<(Values, bool)>> = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in out.lines() {
        let line = line.trim_end_matches('\r');
        let mut f = line.splitn(3, '\t');
        match (f.next(), f.next(), f.next()) {
            (Some("K"), Some(key), Some(flag)) => {
                found.insert(
                    key.to_string(),
                    (flag != "-").then(|| (Vec::new(), flag == "1")),
                );
                current = Some(key.to_string());
            }
            (Some("V"), Some(name), value) => {
                if let Some(Some((vals, _))) = current.as_ref().and_then(|k| found.get_mut(k)) {
                    vals.push((name.to_string(), value.unwrap_or("").to_string()));
                }
            }
            _ => {}
        }
    }
    keys.iter()
        .map(|k| found.get(k).cloned().unwrap_or(None))
        .collect()
}

/// `HKCU` through PowerShell: one process per read or apply.
pub struct PowerShellRegistry;

impl Registry for PowerShellRegistry {
    fn read(&self, keys: &[String]) -> Result<Vec<Option<(Values, bool)>>, String> {
        for k in keys {
            check(k)?;
        }
        let joined = keys.join("\n");
        run_powershell(READ_SCRIPT, &[("GP_KEYS", &joined)]).map(|o| parse_read(keys, &o))
    }

    fn apply(&self, ops: &[RegOp]) -> Result<(), String> {
        if ops.is_empty() {
            return Ok(());
        }
        let text = ops_text(ops)?;
        run_powershell(APPLY_SCRIPT, &[("GP_OPS", &text)]).map(|_| ())
    }
}

/// Lower-cased key path -> (value name -> value). Names keep their case.
type Tree = BTreeMap<String, BTreeMap<String, String>>;

fn norm(key: &str) -> String {
    key.trim_matches('\\').to_ascii_lowercase()
}

fn read_tree(tree: &Tree, keys: &[String]) -> Vec<Option<(Values, bool)>> {
    keys.iter()
        .map(|k| {
            let n = norm(k);
            let vals = tree.get(&n)?;
            let prefix = format!("{n}\\");
            let sub = tree.keys().any(|o| o.starts_with(&prefix));
            Some((
                vals.iter().map(|(a, b)| (a.clone(), b.clone())).collect(),
                sub,
            ))
        })
        .collect()
}

fn apply_tree(tree: &mut Tree, ops: &[RegOp]) -> Result<(), String> {
    ops_text(ops)?;
    for op in ops {
        match op {
            RegOp::Set { key, name, value } => {
                // Creating a key creates its ancestors, as the registry does.
                let n = norm(key);
                let mut path = String::new();
                for part in n.split('\\') {
                    if !path.is_empty() {
                        path.push('\\');
                    }
                    path.push_str(part);
                    tree.entry(path.clone()).or_default();
                }
                tree.entry(n)
                    .or_default()
                    .insert(name.clone(), value.clone());
            }
            RegOp::DeleteTree(k) => {
                let n = norm(k);
                let prefix = format!("{n}\\");
                tree.retain(|key, _| *key != n && !key.starts_with(&prefix));
            }
            RegOp::DeleteIfEmpty(k) => {
                let n = norm(k);
                let prefix = format!("{n}\\");
                let has_sub = tree.keys().any(|o| o.starts_with(&prefix));
                if tree.get(&n).is_some_and(|v| v.is_empty()) && !has_sub {
                    tree.remove(&n);
                }
            }
        }
    }
    Ok(())
}

/// A registry in memory, for tests.
#[derive(Default)]
pub struct MemoryRegistry(Mutex<Tree>);

impl MemoryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Every key and value, sorted: what a test compares before and after.
    /// A key with no values shows as `(key, "", "<key>")`.
    pub fn dump(&self) -> Vec<(String, String, String)> {
        let tree = self.0.lock().unwrap();
        let mut out = Vec::new();
        for (k, vals) in tree.iter() {
            if vals.is_empty() {
                out.push((k.clone(), String::new(), "<key>".to_string()));
            }
            for (n, v) in vals {
                out.push((k.clone(), n.clone(), v.clone()));
            }
        }
        out
    }
}

impl Registry for MemoryRegistry {
    fn read(&self, keys: &[String]) -> Result<Vec<Option<(Values, bool)>>, String> {
        Ok(read_tree(&self.0.lock().unwrap(), keys))
    }

    fn apply(&self, ops: &[RegOp]) -> Result<(), String> {
        apply_tree(&mut self.0.lock().unwrap(), ops)
    }
}

/// The same model kept in a file: `key<TAB>name<TAB>value` per value, and
/// `key<TAB>NUL<TAB>` for a key with no values.
pub struct FileRegistry(pub PathBuf);

impl FileRegistry {
    fn load(&self) -> Tree {
        let mut tree = Tree::new();
        let Ok(text) = std::fs::read_to_string(&self.0) else {
            return tree;
        };
        for line in text.lines() {
            let mut f = line.splitn(3, '\t');
            match (f.next(), f.next(), f.next()) {
                (Some(k), Some("\u{0}"), _) => {
                    tree.entry(k.to_string()).or_default();
                }
                (Some(k), Some(n), Some(v)) => {
                    tree.entry(k.to_string())
                        .or_default()
                        .insert(n.to_string(), v.to_string());
                }
                _ => {}
            }
        }
        tree
    }

    fn save(&self, tree: &Tree) -> Result<(), String> {
        let mut out = String::new();
        for (k, vals) in tree {
            if vals.is_empty() {
                out.push_str(&format!("{k}\t\u{0}\t\n"));
            }
            for (n, v) in vals {
                out.push_str(&format!("{k}\t{n}\t{v}\n"));
            }
        }
        std::fs::write(&self.0, out).map_err(|e| format!("{}: {e}", self.0.display()))
    }
}

impl Registry for FileRegistry {
    fn read(&self, keys: &[String]) -> Result<Vec<Option<(Values, bool)>>, String> {
        Ok(read_tree(&self.load(), keys))
    }

    fn apply(&self, ops: &[RegOp]) -> Result<(), String> {
        let mut tree = self.load();
        apply_tree(&mut tree, ops)?;
        self.save(&tree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(key: &str, name: &str, value: &str) -> RegOp {
        RegOp::Set {
            key: key.into(),
            name: name.into(),
            value: value.into(),
        }
    }

    #[test]
    fn memory_round_trips_values_and_reports_subkeys() {
        let r = MemoryRegistry::new();
        r.apply(&[set("A\\B", "", "d"), set("A\\B", "x", "1")])
            .unwrap();
        let got = r.read(&["A\\B".into(), "A".into(), "Z".into()]).unwrap();
        let (vals, sub) = got[0].clone().unwrap();
        assert!(!sub);
        assert!(
            vals.contains(&("".into(), "d".into())) && vals.contains(&("x".into(), "1".into()))
        );
        assert_eq!(got[1], Some((vec![], true)));
        assert_eq!(got[2], None);
    }

    #[test]
    fn delete_if_empty_keeps_a_key_with_values_or_subkeys() {
        let r = MemoryRegistry::new();
        r.apply(&[set("P\\C", "v", "1"), set("Q", "v", "1")])
            .unwrap();
        r.apply(&[
            RegOp::DeleteIfEmpty("P".into()),
            RegOp::DeleteIfEmpty("Q".into()),
        ])
        .unwrap();
        assert!(r
            .read(&["P".into(), "Q".into()])
            .unwrap()
            .iter()
            .all(Option::is_some));
        r.apply(&[
            RegOp::DeleteTree("P\\C".into()),
            RegOp::DeleteIfEmpty("P".into()),
        ])
        .unwrap();
        assert_eq!(r.read(&["P".into()]).unwrap()[0], None);
        r.apply(&[RegOp::DeleteTree("Q".into())]).unwrap();
        assert!(r.dump().is_empty());
    }

    #[test]
    fn a_tab_or_newline_in_a_value_is_rejected() {
        let r = MemoryRegistry::new();
        assert!(r.apply(&[set("K", "n", "a\tb")]).is_err());
        assert!(r.apply(&[set("K", "n", "a\nb")]).is_err());
        assert!(r.dump().is_empty());
    }

    #[test]
    fn the_read_script_output_is_parsed_in_request_order() {
        let keys = vec!["B".to_string(), "A".to_string(), "C".to_string()];
        let out = "K\tA\t1\nV\t\tdef\nV\tx\ta b\nK\tB\t-\n";
        let got = parse_read(&keys, out);
        assert_eq!(got[0], None);
        assert_eq!(
            got[1],
            Some((
                vec![("".into(), "def".into()), ("x".into(), "a b".into())],
                true
            ))
        );
        assert_eq!(got[2], None);
    }

    #[test]
    fn the_file_registry_persists_between_instances() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("reg.txt");
        FileRegistry(f.clone())
            .apply(&[set("K\\S", "", "v")])
            .unwrap();
        let got = FileRegistry(f.clone())
            .read(&["K\\S".into(), "K".into()])
            .unwrap();
        assert_eq!(got[0], Some((vec![("".into(), "v".into())], false)));
        assert_eq!(got[1], Some((vec![], true)));
        FileRegistry(f.clone())
            .apply(&[RegOp::DeleteTree("K".into())])
            .unwrap();
        assert_eq!(FileRegistry(f).read(&["K\\S".into()]).unwrap()[0], None);
    }
}
