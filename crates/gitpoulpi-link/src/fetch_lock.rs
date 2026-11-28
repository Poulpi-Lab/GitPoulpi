//! When a failed background fetch is dropped without a word.

/// Whether git's own message is a lock another process holds: an
/// `index.lock`, a ref's `<name>.lock`, `packed-refs.lock` (every git
/// message names the `.lock` file, whatever the language) or the English
/// "cannot lock ref".
pub fn is_git_lock_error(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.contains(".lock") || text.contains("cannot lock ref") // i18n-english
}

/// Whether a failed fetch is dropped without a word: only the window's own
/// background fetch, and only for a git lock. Concurrent fetches across
/// environments can meet the ref locks (the installations design, 1.5); git's
/// lock makes one of them lose, nothing is corrupted, and the next interval
/// tries again. A fetch the user asked for always reports its failure.
pub fn skip_silently(background: bool, error_text: &str) -> bool {
    background && is_git_lock_error(error_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_fetch_skips_silently_on_a_git_lock() {
        let lock = "fatal: Unable to create '/r/.git/refs/remotes/origin/main.lock': File exists.";
        assert!(skip_silently(true, lock));
        assert!(skip_silently(
            true,
            "error: cannot lock ref 'refs/remotes/origin/x'"
        ));
        // A fetch the user asked for reports it; so does any other failure.
        assert!(!skip_silently(false, lock));
        assert!(!skip_silently(
            true,
            "fatal: could not read from remote repository"
        ));
    }
}
