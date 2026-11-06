//! Checks on user-, API- or agent-supplied values that end up as a
//! positional argument of a network operation. Both backends run them, so a
//! value is refused the same way whichever one is chosen.

use crate::{Error, Result};

/// A remote name, URL or local path given to a network operation (clone,
/// fetch, pull, push, `remote add`, `remote set-url`).
///
/// Refused: an empty value, a leading `-` (git would read it as an option,
/// for example `--upload-pack=...`), control characters, and the
/// `<helper>::<address>` form (`ext::sh -c ...` runs a command through a
/// remote helper).
pub fn check_remote_arg(what: &str, value: &str) -> Result<()> {
    let bad = |why: &str| {
        Err(Error::Backend(format!(
            "`{value}` is not a usable {what}: {why}"
        )))
    };
    if value.is_empty() {
        return bad("it is empty");
    }
    if value.starts_with('-') {
        return bad("it starts with `-`");
    }
    if value.chars().any(char::is_control) {
        return bad("it contains control characters");
    }
    if let Some((helper, _)) = value.split_once("::") {
        let is_helper = !helper.is_empty()
            && helper
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '.' | '-' | '_'));
        if is_helper {
            return bad("remote helper addresses (`ext::`, `fd::`, ...) are not allowed");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_remotes_and_urls_pass() {
        for v in [
            "origin",
            "https://example.com/r.git",
            "git@github.com:o/r.git",
            "ssh://git@host/r",
            "file:///tmp/r",
            "/tmp/r",
            "C:\\repos\\r",
            "../r",
        ] {
            assert!(check_remote_arg("remote", v).is_ok(), "{v}");
        }
    }

    #[test]
    fn option_lookalikes_helpers_and_control_chars_are_refused() {
        for v in [
            "",
            "-x",
            "--upload-pack=touch /tmp/x",
            "ext::sh -c touch% /tmp/x",
            "EXT::sh -c id",
            "fd::3",
            "hg::https://x",
            "https://x/\nfoo",
            "a\0b",
        ] {
            assert!(check_remote_arg("remote", v).is_err(), "{v:?}");
        }
    }
}
