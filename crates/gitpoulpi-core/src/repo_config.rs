//! The repository-local git settings GitPoulpi may read and write through
//! [`crate::GitBackend::repo_config`] / [`crate::GitBackend::set_repo_config`]:
//! an allow-list, deliberately not a general config API.

use crate::{Error, Result};

/// Line-ending conversion on checkout/commit (`true`, `input`, `false`).
pub const CORE_AUTOCRLF: &str = "core.autocrlf";
/// Line ending of text files in the working tree (`lf`, `crlf`, `native`).
pub const CORE_EOL: &str = "core.eol";

/// The file the commit message is pre-filled from (`commit.template`). Free
/// text: a path, `~/` and a path relative to the work tree included.
pub const COMMIT_TEMPLATE: &str = "commit.template";
/// The character that starts a comment line in a commit message
/// (`core.commentChar`: one character, or `auto`). Read to strip comments.
pub const CORE_COMMENT_CHAR: &str = "core.commentChar";

/// Whether commits are signed by default (`commit.gpgSign`, `true` or
/// `false`). Read by the split editor to pre-check "Sign commits".
pub const COMMIT_GPGSIGN: &str = "commit.gpgSign";

/// The commit identity's name: what a repository's chosen profile writes.
pub const USER_NAME: &str = "user.name";
/// The commit identity's e-mail.
pub const USER_EMAIL: &str = "user.email";

/// Every key the repo-config calls accept, with the values each takes.
/// An empty list means free text (any non-blank, single-line value).
pub const REPO_CONFIG_KEYS: &[(&str, &[&str])] = &[
    (CORE_AUTOCRLF, &["true", "input", "false"]),
    (CORE_EOL, &["lf", "crlf", "native"]),
    (USER_NAME, &[]),
    (USER_EMAIL, &[]),
    (COMMIT_TEMPLATE, &[]),
    (CORE_COMMENT_CHAR, &[]),
    (COMMIT_GPGSIGN, &["true", "false"]),
];

/// The character that starts a comment line, from `core.commentChar`'s value:
/// its first character, `#` when unset, blank or `auto` (`auto` picks a
/// character absent from the message; for stripping a user's own template
/// `#` is the answer).
pub fn comment_char(configured: Option<&str>) -> char {
    match configured.map(str::trim) {
        Some(v) if v != "auto" => v.chars().next().unwrap_or('#'),
        _ => '#',
    }
}

/// `message` without its comment lines (lines starting with `comment`), as
/// `--cleanup=strip` would drop them, and without the blank lines that leaves
/// at either end. GitPoulpi does it itself because it commits with `-m`,
/// which git does not clean up.
pub fn strip_comment_lines(message: &str, comment: char) -> String {
    let kept: Vec<&str> = message
        .lines()
        .filter(|line| !line.starts_with(comment))
        .collect();
    kept.join("\n").trim().to_string()
}

/// `Err` unless `key` is on the allow-list.
pub fn check_repo_config_key(key: &str) -> Result<()> {
    if REPO_CONFIG_KEYS.iter().any(|(k, _)| *k == key) {
        Ok(())
    } else {
        Err(Error::Backend(format!(
            "`{key}` is not a settable repository config key (allowed: {})",
            REPO_CONFIG_KEYS
                .iter()
                .map(|(k, _)| *k)
                .collect::<Vec<_>>()
                .join(", ")
        )))
    }
}

/// `Err` unless `key` is allowed and `value` is one of its accepted values.
pub fn check_repo_config_value(key: &str, value: &str) -> Result<()> {
    check_repo_config_key(key)?;
    let allowed = REPO_CONFIG_KEYS
        .iter()
        .find(|(k, _)| *k == key)
        .map_or(&[][..], |(_, v)| *v);
    if allowed.is_empty() {
        return if value.trim().is_empty() || value.contains(['\n', '\r']) {
            Err(Error::Backend(format!(
                "{key} must be a non-blank single line"
            )))
        } else {
            Ok(())
        };
    }
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(Error::Backend(format!(
            "`{value}` is not a valid value of {key} (one of: {})",
            allowed.join(", ")
        )))
    }
}

/// The ssh command that offers one key and only that one:
/// `ssh -i "<key>" -o IdentitiesOnly=yes`. Used for the repository pin
/// (`core.sshCommand`) and for `GIT_SSH_COMMAND` when `ssh.private_key_path` /
/// `ssh.public_key_path` is set; git runs it through `sh`, so the path sits in
/// double quotes with `"`, `$` and the backtick escaped. Backslashes become
/// forward slashes, which git's `sh` accepts on Windows too.
pub fn ssh_command_for_key(key_path: &str) -> String {
    format!("ssh -i {} -o IdentitiesOnly=yes", sh_quote_path(key_path))
}

/// `path` as one word for the `sh` git runs a command through (a credential
/// helper, `GIT_SSH_COMMAND`): in double quotes with `"`, `$` and the
/// backtick escaped, backslashes turned into forward slashes (Git for
/// Windows' `sh` accepts them, and a quoted backslash would be an escape).
pub fn sh_quote_path(path: &str) -> String {
    let mut quoted = String::with_capacity(path.len() + 2);
    quoted.push('"');
    for c in path.chars() {
        match c {
            '\\' => quoted.push('/'),
            '"' | '$' | '`' => {
                quoted.push('\\');
                quoted.push(c);
            }
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_key_path_is_quoted_for_sh() {
        use super::ssh_command_for_key as c;
        assert_eq!(c("/k/a b"), r#"ssh -i "/k/a b" -o IdentitiesOnly=yes"#);
        assert_eq!(
            c(r"C:\Users\me\.ssh\my key"),
            r#"ssh -i "C:/Users/me/.ssh/my key" -o IdentitiesOnly=yes"#
        );
        assert_eq!(
            c(r#"/k/$HOME/`x`/"q""#),
            r#"ssh -i "/k/\$HOME/\`x\`/\"q\"" -o IdentitiesOnly=yes"#
        );
    }

    #[test]
    fn a_path_is_one_double_quoted_word() {
        use super::sh_quote_path as q;
        assert_eq!(q("/a b/c"), r#""/a b/c""#);
        assert_eq!(q(r"C:\Program Files\x.exe"), r#""C:/Program Files/x.exe""#);
        assert_eq!(q(r#"a"$`"#), r#""a\"\$\`""#);
    }

    use super::*;

    #[test]
    fn only_the_listed_keys_and_values_pass() {
        assert!(check_repo_config_value(CORE_AUTOCRLF, "input").is_ok());
        assert!(check_repo_config_value(CORE_EOL, "native").is_ok());
        assert!(check_repo_config_value(CORE_EOL, "true").is_err());
        assert!(check_repo_config_key("core.sshCommand").is_err());
        assert!(check_repo_config_key("user.email").is_ok());
        assert!(check_repo_config_value(USER_NAME, "Ada Lovelace").is_ok());
        assert!(check_repo_config_value(USER_NAME, "  ").is_err());
        assert!(check_repo_config_value(USER_EMAIL, "a\nb").is_err());
        assert!(check_repo_config_key("user.signingkey").is_err());
    }

    #[test]
    fn comment_lines_are_stripped_with_the_configured_character() {
        assert_eq!(comment_char(None), '#');
        assert_eq!(comment_char(Some("auto")), '#');
        assert_eq!(comment_char(Some(";")), ';');
        assert_eq!(comment_char(Some("  ")), '#');
        assert_eq!(
            strip_comment_lines("# note\nfix: a\n\n# more\nbody\n", '#'),
            "fix: a\n\nbody"
        );
        assert_eq!(strip_comment_lines("# a\n  # b\nx", '#'), "# b\nx");
        assert_eq!(strip_comment_lines("; a\n# b", ';'), "# b");
        assert_eq!(strip_comment_lines("# only", '#'), "");
    }
}
