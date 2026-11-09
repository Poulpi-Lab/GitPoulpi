//! Reads of the config entries that decide how a repository authenticates,
//! with the scope and file each one comes from (`git config --show-scope
//! --show-origin`). The daemon builds a repository's [`NetworkAuth`] from
//! them (ADR-0010): which credential helpers are the user's own (system and
//! global scope) and which the repository brings (local, worktree, and
//! anything an included file adds on their behalf, which git scopes as the
//! including file), and which program the global `core.sshCommand` names.
//!
//! Read-only: `git config` runs nothing from the repository.
//!
//! [`NetworkAuth`]: gitpoulpi_core::domain::NetworkAuth

use std::path::Path;

use crate::process::git;

/// One value of a multi-valued config key, in the order git read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigEntry {
    /// `system`, `global`, `local`, `worktree` or `command`.
    pub scope: String,
    /// `file:<path>` (or `command line:`), as git prints it.
    pub origin: String,
    /// The key as git prints it: section and variable lowercased, the
    /// subsection (a URL) as written.
    pub key: String,
    pub value: String,
}

impl ConfigEntry {
    /// Whether the user's own configuration (not the repository's) set it.
    pub fn is_machine_wide(&self) -> bool {
        matches!(self.scope.as_str(), "system" | "global")
    }
}

/// Every entry whose key matches `pattern` (a regexp, as `--get-regexp`
/// takes it), in config order. `Err` when git cannot be asked; no match is an
/// empty list.
pub fn config_entries(repo_root: &Path, pattern: &str) -> Result<Vec<ConfigEntry>, String> {
    let out = git(
        repo_root,
        &[
            "config",
            "--show-scope",
            "--show-origin",
            "-z",
            "--get-regexp",
            pattern,
        ],
    )
    .output()
    .map_err(|e| format!("could not run git config: {e}"))?;
    match out.status.code() {
        Some(0) => {}
        // 1: nothing matched.
        Some(1) => return Ok(Vec::new()),
        _ => {
            return Err(format!(
                "git config failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        }
    }
    Ok(parse_entries(&String::from_utf8_lossy(&out.stdout)))
}

/// `scope NUL origin NUL key NL value NUL`, repeated (`-z`).
fn parse_entries(text: &str) -> Vec<ConfigEntry> {
    let mut fields = text.split('\0');
    let mut out = Vec::new();
    while let (Some(scope), Some(origin), Some(rest)) =
        (fields.next(), fields.next(), fields.next())
    {
        let (key, value) = rest.split_once('\n').unwrap_or((rest, ""));
        out.push(ConfigEntry {
            scope: scope.to_string(),
            origin: origin.to_string(),
            key: key.to_string(),
            value: value.to_string(),
        });
    }
    out
}

/// Every `credential.helper` and `credential.<url>.helper` entry.
pub fn credential_helper_entries(repo_root: &Path) -> Result<Vec<ConfigEntry>, String> {
    config_entries(repo_root, r"^credential\..*helper$")
}

/// Every `core.sshCommand` entry.
pub fn ssh_command_entries(repo_root: &Path) -> Result<Vec<ConfigEntry>, String> {
    config_entries(repo_root, r"^core\.sshcommand$")
}

/// The helpers that remain once the repository's own are dropped: the system
/// and global entries, in order, as `(None, helper)` for `credential.helper`
/// and `(Some(url), helper)` for `credential.<url>.helper`. An empty value
/// is git's reset: it clears what was collected before it for the same
/// scope of URLs, as git itself would.
pub fn machine_wide_helpers(entries: &[ConfigEntry]) -> Vec<(Option<String>, String)> {
    let mut kept: Vec<(Option<String>, String)> = Vec::new();
    for e in entries.iter().filter(|e| e.is_machine_wide()) {
        let Some(rest) = e.key.strip_prefix("credential.") else {
            continue;
        };
        let url = match rest.strip_suffix(".helper") {
            Some(url) if !url.is_empty() => Some(url.to_string()),
            _ if rest == "helper" => None,
            _ => continue,
        };
        if e.value.is_empty() {
            kept.retain(|(u, _)| *u != url);
        } else {
            kept.push((url, e.value.clone()));
        }
    }
    kept
}

/// The program word of the effective **global** `core.sshCommand` (system or
/// global scope; the repository's own is what the profile overrides), or
/// `ssh`. Users often point it at the Windows OpenSSH client to reach the
/// Windows agent, so a key file must be offered through the same program.
pub fn global_ssh_program(entries: &[ConfigEntry]) -> String {
    entries
        .iter()
        .rfind(|e| e.is_machine_wide() && !e.value.trim().is_empty())
        .and_then(|e| first_word(&e.value))
        .unwrap_or_else(|| "ssh".to_string())
}

/// The first word of a shell command line, quotes removed. Backslashes are
/// left alone so an unquoted Windows path stays whole.
fn first_word(command: &str) -> Option<String> {
    let s = command.trim_start();
    let mut chars = s.chars();
    match chars.next()? {
        q @ ('"' | '\'') => {
            let rest = &s[1..];
            let end = rest.find(q)?;
            Some(rest[..end].to_string()).filter(|w| !w.is_empty())
        }
        _ => s
            .split(char::is_whitespace)
            .next()
            .map(str::to_string)
            .filter(|w| !w.is_empty()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(scope: &str, key: &str, value: &str) -> ConfigEntry {
        ConfigEntry {
            scope: scope.into(),
            origin: "file:x".into(),
            key: key.into(),
            value: value.into(),
        }
    }

    #[test]
    fn the_null_separated_output_is_parsed() {
        let text = "global\0file:/h/.gitconfig\0credential.helper\nstore\0\
                    local\0file:.git/config\0credential.https://x.y.helper\n!f\0";
        let entries = parse_entries(text);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].scope, "global");
        assert_eq!(entries[0].value, "store");
        assert_eq!(entries[1].key, "credential.https://x.y.helper");
        assert_eq!(entries[1].origin, "file:.git/config");
    }

    #[test]
    fn only_machine_wide_helpers_are_kept_and_a_reset_clears_its_own_scope() {
        let entries = [
            entry("system", "credential.helper", "manager"),
            entry("global", "credential.helper", ""),
            entry("global", "credential.helper", "store"),
            entry("global", "credential.https://github.com.helper", "!gh"),
            entry("local", "credential.helper", "!/missing/script"),
            entry("local", "credential.https://github.com.helper", "!evil"),
            entry("worktree", "credential.helper", "!worktree"),
        ];
        assert_eq!(
            machine_wide_helpers(&entries),
            vec![
                (None, "store".to_string()),
                (Some("https://github.com".to_string()), "!gh".to_string()),
            ]
        );
    }

    #[test]
    fn the_ssh_program_is_the_first_word_of_the_global_command_only() {
        assert_eq!(global_ssh_program(&[]), "ssh");
        assert_eq!(
            global_ssh_program(&[entry("local", "core.sshcommand", "evil -i x")]),
            "ssh"
        );
        assert_eq!(
            global_ssh_program(&[
                entry("system", "core.sshcommand", "other"),
                entry(
                    "global",
                    "core.sshcommand",
                    "C:/Windows/System32/OpenSSH/ssh.exe -v"
                ),
                entry("local", "core.sshcommand", "evil"),
            ]),
            "C:/Windows/System32/OpenSSH/ssh.exe"
        );
        assert_eq!(
            global_ssh_program(&[entry(
                "global",
                "core.sshcommand",
                r#""C:\Program Files\OpenSSH\ssh.exe" -q"#
            )]),
            r"C:\Program Files\OpenSSH\ssh.exe"
        );
    }
}
