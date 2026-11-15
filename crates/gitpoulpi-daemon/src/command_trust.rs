//! What a person confirms when they store a credential helper or a token
//! command (ADR-0010), and how the daemon checks it before every use.
//!
//! The command lives in the private store, never in the repository, so a
//! repository cannot make GitPoulpi run a command. What it *can* do is change
//! a tracked script that the command runs: a pull rewrites the file. So the
//! person confirms two hashes, kept with the profile:
//!
//! - `confirmed_sha256`: the helper string, or the argv, exactly;
//! - `script_sha256`: every file the command names that lies inside the work
//!   tree (path relative to the root and content), `None` when there is none.
//!
//! A command that no longer matches is refused: `RepoAuthUnavailable` for a
//! different command, `RepoAuthCommandChanged { path }` for an in-tree file
//! that changed. Confirming again re-pins both. The daemon's socket is not a
//! security boundary against local malware (as for hook and script trust);
//! this guards against agents and repositories.

use std::path::{Path, PathBuf};

use gitpoulpi_core::domain::HttpsAuth;
use gitpoulpi_core::Error;
use sha2::{Digest, Sha256};

fn unavailable(reason: impl Into<String>) -> Error {
    Error::RepoAuthUnavailable {
        reason: reason.into(),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The hash a person confirms for a helper string.
pub(crate) fn helper_hash(helper: &str) -> String {
    sha256_hex(helper.as_bytes())
}

/// The hash a person confirms for an argv: every element, NUL separated (an
/// element cannot hold one), so `["a b"]` and `["a", "b"]` differ.
pub(crate) fn argv_hash(argv: &[String]) -> String {
    sha256_hex(argv.join("\0").as_bytes())
}

/// The words of a helper string: `!` prefix dropped, split on white space
/// with `'...'` and `"..."` grouping (what an `sh` would do, no more).
pub(crate) fn helper_words(helper: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut started = false;
    for c in helper.trim().trim_start_matches('!').chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => current.push(c),
            (None, '\'') | (None, '"') => {
                quote = Some(c);
                started = true;
            }
            (None, c) if c.is_whitespace() => {
                if started || !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                    started = false;
                }
            }
            (None, c) => current.push(c),
        }
    }
    if started || !current.is_empty() {
        words.push(current);
    }
    words
}

/// The existing files among `words` that lie inside the work tree at `root`
/// (a relative word is relative to `root`, the command's directory), as
/// `(path relative to the root with `/`, canonical path)`, in order, without
/// repeats. `--opt=value` words are checked on their value too.
pub(crate) fn in_tree_files(root: &Path, words: &[String]) -> Vec<(String, PathBuf)> {
    let Ok(canonical_root) = std::fs::canonicalize(root) else {
        return Vec::new();
    };
    let mut found: Vec<(String, PathBuf)> = Vec::new();
    for word in words {
        let value = word.split_once('=').map(|(_, v)| v);
        for candidate in std::iter::once(word.as_str()).chain(value) {
            if candidate.is_empty() {
                continue;
            }
            let Ok(canonical) = std::fs::canonicalize(root.join(candidate)) else {
                continue;
            };
            if !canonical.is_file() {
                continue;
            }
            let Ok(relative) = canonical.strip_prefix(&canonical_root) else {
                continue;
            };
            let relative = relative.to_string_lossy().replace('\\', "/");
            if !found.iter().any(|(r, _)| *r == relative) {
                found.push((relative, canonical));
            }
        }
    }
    found
}

/// `(hash, display names)` of the in-tree files of the command, or `None`
/// when it names none. An unreadable file hashes as empty, which no
/// confirmation holds.
pub(crate) fn script_pin(root: &Path, words: &[String]) -> Option<(String, String)> {
    let files = in_tree_files(root, words);
    if files.is_empty() {
        return None;
    }
    let mut all = String::new();
    for (relative, canonical) in &files {
        let content = std::fs::read(canonical)
            .map(|c| sha256_hex(&c))
            .unwrap_or_default();
        all.push_str(&format!("{relative}\0{content}\n"));
    }
    let names = files
        .iter()
        .map(|(r, _)| r.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Some((sha256_hex(all.as_bytes()), names))
}

/// What a confirmation pins, computed now.
pub(crate) struct Pin {
    pub confirmed_sha256: String,
    pub script_sha256: Option<String>,
}

/// The command's own hash and the words it runs (for the in-tree check).
fn command_of(https: &HttpsAuth) -> Option<(String, Vec<String>)> {
    match https {
        HttpsAuth::Helper { helper, .. } => Some((helper_hash(helper), helper_words(helper))),
        HttpsAuth::TokenCommand { argv, .. } => Some((argv_hash(argv), argv.clone())),
        _ => None,
    }
}

/// Whether `https` is a command-running source.
pub(crate) fn is_command(https: &HttpsAuth) -> bool {
    command_of(https).is_some()
}

/// The pin of the command of `https` as it is on disk now, `None` when
/// `https` runs no command.
pub(crate) fn pin(root: &Path, https: &HttpsAuth) -> Option<Pin> {
    let (hash, words) = command_of(https)?;
    Some(Pin {
        confirmed_sha256: hash,
        script_sha256: script_pin(root, &words).map(|(h, _)| h),
    })
}

/// Stores `pin` and `now` in the confirmation fields of `https`.
pub(crate) fn apply_pin(https: &mut HttpsAuth, pin: Pin, now: i64) {
    match https {
        HttpsAuth::Helper {
            confirmed_sha256,
            script_sha256,
            confirmed_at,
            ..
        }
        | HttpsAuth::TokenCommand {
            confirmed_sha256,
            script_sha256,
            confirmed_at,
            ..
        } => {
            *confirmed_sha256 = Some(pin.confirmed_sha256);
            *script_sha256 = pin.script_sha256;
            *confirmed_at = Some(now);
        }
        _ => {}
    }
}

/// The confirmation fields of a stored command source.
fn confirmation(https: &HttpsAuth) -> Option<(Option<&str>, Option<&str>, Option<i64>)> {
    match https {
        HttpsAuth::Helper {
            confirmed_sha256,
            script_sha256,
            confirmed_at,
            ..
        }
        | HttpsAuth::TokenCommand {
            confirmed_sha256,
            script_sha256,
            confirmed_at,
            ..
        } => Some((
            confirmed_sha256.as_deref(),
            script_sha256.as_deref(),
            *confirmed_at,
        )),
        _ => None,
    }
}

/// Copies the confirmation of `stored` into `new` when both run the very
/// same command that `stored` had confirmed and its files are unchanged
/// (editing a source's hosts or cache time needs no new confirmation).
/// Whether it did.
pub(crate) fn carry_confirmation(root: &Path, new: &mut HttpsAuth, stored: &HttpsAuth) -> bool {
    let (Some((new_hash, _)), Some((stored_hash, _))) = (command_of(new), command_of(stored))
    else {
        return false;
    };
    let Some((Some(confirmed), script, at)) = confirmation(stored) else {
        return false;
    };
    if confirmed != new_hash || new_hash != stored_hash || verify(root, stored).is_err() {
        return false;
    }
    let carried = Pin {
        confirmed_sha256: confirmed.to_string(),
        script_sha256: script.map(str::to_string),
    };
    apply_pin(new, carried, at.unwrap_or_default());
    true
}

/// Checks the stored confirmation of a command source against the command
/// and its in-tree files as they are now. `Ok` for a source that runs no
/// command.
pub(crate) fn verify(root: &Path, https: &HttpsAuth) -> Result<(), Error> {
    let Some((hash, words)) = command_of(https) else {
        return Ok(());
    };
    let (confirmed, script, _) = confirmation(https).unwrap_or((None, None, None));
    match confirmed {
        None => return Err(unavailable(
            "the command was never confirmed: confirm it in GitPoulpi's authentication settings",
        )),
        Some(c) if c != hash => {
            return Err(unavailable(
                "the command is not the one that was confirmed: confirm it again",
            ))
        }
        Some(_) => {}
    }
    let now = script_pin(root, &words);
    if now.as_ref().map(|(h, _)| h.as_str()) != script {
        let path = now
            .map(|(_, names)| names)
            .unwrap_or_else(|| "a file of the command".to_string());
        return Err(Error::RepoAuthCommandChanged { path });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_command(argv: &[&str]) -> HttpsAuth {
        HttpsAuth::TokenCommand {
            argv: argv.iter().map(|s| s.to_string()).collect(),
            username: None,
            hosts: Vec::new(),
            cache_minutes: None,
            use_for_hosting: false,
            confirmed_sha256: None,
            script_sha256: None,
            confirmed_at: None,
        }
    }

    #[test]
    fn helper_words_split_like_a_shell_would() {
        assert_eq!(helper_words("!/a/b --x"), ["/a/b", "--x"]);
        assert_eq!(
            helper_words("!\"C:/Program Files/x/get.sh\" 'a b' c"),
            ["C:/Program Files/x/get.sh", "a b", "c"]
        );
        assert_eq!(helper_words("store"), ["store"]);
    }

    #[test]
    fn argv_boundaries_matter_to_the_hash() {
        assert_ne!(
            argv_hash(&["a b".into()]),
            argv_hash(&["a".into(), "b".into()])
        );
    }

    #[test]
    fn only_files_inside_the_tree_are_pinned_and_a_change_is_detected() {
        let tree = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(tree.path().join("get.sh"), "echo one").unwrap();
        let outer = outside.path().join("outer.sh");
        std::fs::write(&outer, "echo outer").unwrap();
        let outer = outer.to_string_lossy().into_owned();

        let mut https = token_command(&["sh", "get.sh", &outer]);
        let pinned = pin(tree.path(), &https).unwrap();
        assert!(pinned.script_sha256.is_some());
        apply_pin(&mut https, pinned, 1);
        verify(tree.path(), &https).unwrap();

        // The file outside the tree is not pinned: editing it changes nothing.
        std::fs::write(&outer, "echo changed").unwrap();
        verify(tree.path(), &https).unwrap();

        std::fs::write(tree.path().join("get.sh"), "echo two").unwrap();
        match verify(tree.path(), &https) {
            Err(Error::RepoAuthCommandChanged { path }) => assert_eq!(path, "get.sh"),
            other => panic!("{other:?}"),
        }
        // Confirming again re-pins.
        let again = pin(tree.path(), &https).unwrap();
        apply_pin(&mut https, again, 2);
        verify(tree.path(), &https).unwrap();
    }

    #[test]
    fn a_command_without_a_confirmation_or_with_another_hash_is_refused() {
        let tree = tempfile::tempdir().unwrap();
        let mut https = token_command(&["tool"]);
        assert!(matches!(
            verify(tree.path(), &https),
            Err(Error::RepoAuthUnavailable { .. })
        ));
        let pinned = pin(tree.path(), &https).unwrap();
        apply_pin(&mut https, pinned, 1);
        verify(tree.path(), &https).unwrap();
        if let HttpsAuth::TokenCommand { argv, .. } = &mut https {
            argv.push("--other".into());
        }
        assert!(matches!(
            verify(tree.path(), &https),
            Err(Error::RepoAuthUnavailable { .. })
        ));
    }

    #[test]
    fn a_confirmed_script_that_vanished_is_a_change() {
        let tree = tempfile::tempdir().unwrap();
        let file = tree.path().join("g.sh");
        std::fs::write(&file, "x").unwrap();
        let mut https = token_command(&["sh", "g.sh"]);
        let pinned = pin(tree.path(), &https).unwrap();
        apply_pin(&mut https, pinned, 1);
        std::fs::remove_file(&file).unwrap();
        assert!(matches!(
            verify(tree.path(), &https),
            Err(Error::RepoAuthCommandChanged { .. })
        ));
    }
}
