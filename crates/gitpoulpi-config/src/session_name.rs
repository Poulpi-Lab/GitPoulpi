//! Session names, and the rule for the *default* one.
//!
//! A session name is used verbatim as a filename (`sessions/<name>.toml`,
//! `sessions/<name>.port`), so it has to survive the strictest filesystem
//! we target, Windows'. This crate carries no `cfg(windows)`: every
//! platform derives the same conservative name, so a session described in
//! a bug report (or a config dir copied between machines) means the same
//! thing everywhere.
//!
//! The *default* session is derived from the repository root rather than
//! being the literal `"default"`: one shared daemon would serve two
//! unrelated repos correctly, but closing either frontend would take it
//! down under the other, and both would contend for one `.port` file.
//! Every frontend opened *anywhere inside one repo* still resolves to the
//! same name and daemon (architecture principle 3). `-s <name>` forces
//! sharing across repos or names a session explicitly.

use std::path::{Path, PathBuf};

/// Filename length bound. Windows' own limit is 255 per path component,
/// but the name also shows up in the TUI footer and in `session list`, so
/// this is deliberately far below it: a truncated stem plus the hash
/// suffix still identifies the repo, and the hash still disambiguates.
pub const MAX_SESSION_NAME_LEN: usize = 64;

/// Device names Windows reserves at *every* directory level, with or
/// without an extension (`CON.txt` is still `CON`). Opening one doesn't
/// fail cleanly - it silently talks to a device - so they get a `_`
/// prefix rather than a rejection.
const WINDOWS_RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM0", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
    "COM8", "COM9", "LPT0", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Makes `raw` safe to use as a filename on every platform we target.
///
/// Applied to user-supplied `-s` values too: `-s feature/login` would
/// otherwise write `sessions/feature/login.port` into a directory that
/// does not exist, and `-s C:\repo` would be rejected by Windows. Two raw
/// names can map to the same sanitised one; they then share a session,
/// which beats an I/O error at startup.
pub fn sanitize_session_name(raw: &str) -> String {
    // An allow-list, not the Windows deny-list (`\ / : * ? " < > |`): the
    // deny-list omits control characters, and a path separator sneaking
    // through creates a file somewhere nobody looks for it.
    let mut out = String::with_capacity(raw.len().min(MAX_SESSION_NAME_LEN));
    let mut last_was_dash = false;
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-') {
            out.push(ch);
            last_was_dash = false;
        } else if !last_was_dash && !out.is_empty() {
            out.push('-');
            last_was_dash = true;
        }
    }

    // Truncate before trimming, so a name that ends in a dot only *after*
    // truncation still gets it stripped.
    out.truncate(MAX_SESSION_NAME_LEN);
    // Windows silently strips trailing dots and spaces from filenames, so
    // `work.` and `work` would be one file wearing two session names.
    // Spaces are already gone via the allow-list; dots are not.
    while out.ends_with('.') || out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        return "session".to_string();
    }

    let stem = out.split('.').next().unwrap_or(&out);
    if WINDOWS_RESERVED
        .iter()
        .any(|reserved| stem.eq_ignore_ascii_case(reserved))
    {
        let mut escaped = String::with_capacity(out.len() + 1);
        escaped.push('_');
        escaped.push_str(&out);
        escaped.truncate(MAX_SESSION_NAME_LEN);
        return escaped;
    }

    out
}

/// The default session for a process started in `start_dir`: the session
/// of the repository that directory belongs to, or of the directory
/// itself when it is not inside a repository.
pub fn default_session_name(start_dir: &Path) -> String {
    let root = find_repo_root(start_dir).unwrap_or_else(|| start_dir.to_path_buf());
    default_session_name_from_root(&root)
}

/// The default session for a known repository root: `<basename>-<hash>`.
///
/// The basename alone collides across the very common case of two
/// checkouts of one project (`~/work/api` and `~/oss/api`, or any git
/// worktree layout), and the full path is neither a legal filename nor a
/// short one - so the readable stem is kept for humans and a hash of the
/// *whole* path carries the actual identity.
pub fn default_session_name_from_root(root: &Path) -> String {
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let full = canonical.to_string_lossy();
    let stem = canonical
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "repo".to_string());

    let hash = fnv1a64(full.as_bytes());
    // Reserve the `-` plus 16 hex digits, so the hash is never what gets
    // truncated away -- it is the part that makes the name unique.
    let budget = MAX_SESSION_NAME_LEN.saturating_sub(17);
    let stem = sanitize_session_name(&stem);
    let stem: String = stem.chars().take(budget).collect();
    let stem = stem.trim_end_matches(['.', '-']);
    let stem = if stem.is_empty() { "repo" } else { stem };

    sanitize_session_name(&format!("{stem}-{hash:016x}"))
}

/// FNV-1a, written out rather than reaching for `DefaultHasher`: the std
/// hasher's output is explicitly *not* guaranteed stable across Rust
/// releases, so a CLI and a TUI built from different toolchains would
/// derive different default sessions for the same repo and silently stop
/// seeing each other - exactly the bug this module exists to prevent.
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        // A linked worktree's `.git` is a *file*, not a directory, and it
        // is still its own root as far as sessions are concerned -- two
        // worktrees of one repo are two working trees and deserve two
        // daemons, which `.exists()` gives us for free.
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Every `<name>.port` discovery file in `sessions_dir`, as
/// `(session name, port)` sorted by name. Unparseable files are skipped
/// rather than reported: a half-written one is a transient state, not a
/// session. A missing directory means no daemon has ever run here, which
/// is an empty list, not an error.
///
/// A `.port` file existing says nothing about the daemon still being
/// alive - a crash leaves the file behind. Callers must probe the port
/// before presenting a row as a live session.
pub fn list_session_port_files(sessions_dir: &Path) -> Vec<(String, u16)> {
    let Ok(entries) = std::fs::read_dir(sessions_dir) else {
        return Vec::new();
    };

    let mut found: Vec<(String, u16)> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()? != "port" {
                return None;
            }
            let name = path.file_stem()?.to_string_lossy().to_string();
            let port = std::fs::read_to_string(&path).ok()?.trim().parse().ok()?;
            Some((name, port))
        })
        .collect();
    found.sort();
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn sanitiser_rejects_every_character_windows_forbids() {
        for bad in [':', '\\', '/', '<', '>', '"', '|', '?', '*'] {
            let name = sanitize_session_name(&format!("a{bad}b"));
            assert!(
                !name.contains(bad),
                "{bad:?} survived sanitisation: {name:?}"
            );
        }
    }

    #[test]
    fn sanitiser_drops_control_characters_the_deny_list_would_miss() {
        let name = sanitize_session_name("a\u{0}b\tc\nd");
        assert!(
            name.chars().all(|c| !c.is_control()),
            "control character survived: {name:?}"
        );
    }

    #[test]
    fn sanitiser_strips_trailing_dots_and_spaces() {
        assert_eq!(sanitize_session_name("work. "), "work");
        assert_eq!(sanitize_session_name("work..."), "work");
    }

    #[test]
    fn sanitiser_escapes_windows_reserved_device_names() {
        for reserved in ["CON", "con", "PRN", "AUX", "NUL", "COM1", "lpt9", "CON.txt"] {
            let name = sanitize_session_name(reserved);
            assert!(
                name.starts_with('_'),
                "reserved device name {reserved:?} was left usable as {name:?}"
            );
        }
    }

    #[test]
    fn sanitiser_leaves_ordinary_names_alone() {
        assert_eq!(sanitize_session_name("default"), "default");
        assert_eq!(sanitize_session_name("work-2"), "work-2");
        assert_eq!(sanitize_session_name("feature/login"), "feature-login");
    }

    #[test]
    fn sanitiser_bounds_the_length() {
        let name = sanitize_session_name(&"x".repeat(500));
        assert!(name.len() <= MAX_SESSION_NAME_LEN, "{} chars", name.len());
    }

    #[test]
    fn sanitiser_never_returns_an_empty_name() {
        assert!(!sanitize_session_name("").is_empty());
        assert!(!sanitize_session_name("///").is_empty());
        assert!(!sanitize_session_name("...").is_empty());
    }

    #[test]
    fn default_session_differs_between_repositories() {
        assert_ne!(
            default_session_name_from_root(Path::new("/home/kai/Projects/alpha")),
            default_session_name_from_root(Path::new("/home/kai/Projects/beta"))
        );
    }

    #[test]
    fn default_session_is_stable_across_calls_and_filename_safe() {
        let a = default_session_name_from_root(Path::new("C:\\Users\\kai\\Dev\\My Repo"));
        let b = default_session_name_from_root(Path::new("C:\\Users\\kai\\Dev\\My Repo"));
        assert_eq!(a, b);
        assert_eq!(
            sanitize_session_name(&a),
            a,
            "derived name is not a sanitisation fixpoint"
        );
        assert!(a.len() <= MAX_SESSION_NAME_LEN, "{a}");
    }

    #[test]
    fn default_session_keeps_the_repo_name_readable() {
        let name = default_session_name_from_root(Path::new("/nonexistent/Projects/gitpoulpi"));
        assert!(name.starts_with("gitpoulpi-"), "{name}");
    }

    #[test]
    fn a_very_long_repo_name_keeps_its_whole_hash() {
        let long = format!("/tmp/{}", "z".repeat(300));
        let name = default_session_name_from_root(Path::new(&long));
        assert!(name.len() <= MAX_SESSION_NAME_LEN, "{} chars", name.len());
        let hash = name.rsplit('-').next().unwrap();
        assert_eq!(hash.len(), 16, "hash was truncated away: {name}");
    }

    #[test]
    fn two_repos_with_the_same_basename_do_not_collide() {
        assert_ne!(
            default_session_name_from_root(Path::new("/nonexistent/a/work/api")),
            default_session_name_from_root(Path::new("/nonexistent/b/work/api"))
        );
    }

    #[test]
    fn default_session_walks_up_to_the_repo_root() {
        // Architecture principle 3 in miniature: a CLI run deep inside the
        // tree and a TUI opened at the root must land on the same daemon,
        // or they stop seeing each other live.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("repo");
        let deep = root.join("crates").join("thing").join("src");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::create_dir_all(root.join(".git")).unwrap();

        assert_eq!(default_session_name(&deep), default_session_name(&root));
    }

    #[test]
    fn sibling_repos_under_one_parent_get_different_sessions() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["alpha", "beta"] {
            std::fs::create_dir_all(dir.path().join(name).join(".git")).unwrap();
        }
        assert_ne!(
            default_session_name(&dir.path().join("alpha")),
            default_session_name(&dir.path().join("beta"))
        );
    }

    #[test]
    fn a_linked_worktree_is_its_own_session() {
        // `.git` is a file in a linked worktree; a directory-only check
        // would walk past it and hand the worktree the parent's session.
        let dir = tempfile::tempdir().unwrap();
        let main = dir.path().join("repo");
        let linked = main.join("wt").join("feature");
        std::fs::create_dir_all(&linked).unwrap();
        std::fs::create_dir_all(main.join(".git")).unwrap();
        std::fs::write(linked.join(".git"), "gitdir: ../../.git/worktrees/feature").unwrap();

        assert_ne!(default_session_name(&linked), default_session_name(&main));
    }

    #[test]
    fn port_files_are_listed_with_their_ports_and_nothing_else_is() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("work.port"), "40401\n").unwrap();
        std::fs::write(dir.path().join("oss.port"), "40402").unwrap();
        std::fs::write(dir.path().join("work.toml"), "irrelevant").unwrap();
        std::fs::write(dir.path().join("broken.port"), "not-a-port").unwrap();

        let found = list_session_port_files(dir.path());
        let names: Vec<_> = found.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(
            names,
            vec!["oss", "work"],
            "sorted names only, got {found:?}"
        );
        assert_eq!(found[0].1, 40402);
        assert_eq!(found[1].1, 40401);
    }

    #[test]
    fn listing_a_missing_directory_is_empty_not_an_error() {
        let dir = tempfile::tempdir().unwrap();
        assert!(list_session_port_files(&dir.path().join("nope")).is_empty());
    }
}
