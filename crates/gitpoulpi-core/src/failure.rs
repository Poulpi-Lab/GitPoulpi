//! What went wrong, in enough detail to paste to someone (or an AI) who can
//! explain it: the operation, the git command line that ran, its exit code
//! and the tail of its stderr, or libgit2's class/code/message.
//!
//! Everything in an [`ErrorDetail`] has been through [`redact`], so it is
//! safe to show, log and copy: URLs with credentials, `Authorization`
//! headers, well-known token shapes and `password=`-style pairs never reach
//! it. [`FailureKind`] is the best-effort reading of git's English messages
//! that the frontends turn into a friendly sentence and a suggested action.
//! It is only ever a hint: an unknown or localised message stays
//! [`FailureKind::Other`] and the raw text is still there.

use serde::{Deserialize, Serialize};

/// How many trailing stderr lines an [`ErrorDetail`] keeps.
pub const STDERR_TAIL_LINES: usize = 30;

/// libgit2's own account of a failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Git2Detail {
    /// `git_error_t` class, e.g. `Net`, `Ssh`, `Http`.
    pub class: String,
    /// `git_error_code`, e.g. `Auth`, `NotFound`.
    pub code: String,
    pub message: String,
}

/// The facts of one failed backend operation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// What the user asked for (`Push`, `Pull`, ...), or git's subcommand
    /// until the daemon names the operation.
    pub operation: String,
    /// The git command line that ran, redacted. GitPoulpi's pinned `-c`
    /// output-shape overrides are left out: they are on every command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// The last [`STDERR_TAIL_LINES`] lines of stderr, redacted.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stderr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git2: Option<Git2Detail>,
}

impl ErrorDetail {
    /// A failed `git <args>`: `args` are the command's own arguments.
    pub fn from_git(args: &[&str], exit_code: Option<i32>, stderr: &str) -> Self {
        let command = redact(&format!("git {}", args.join(" ")));
        ErrorDetail {
            operation: args.first().copied().unwrap_or("git").to_string(),
            command: Some(command),
            exit_code,
            stderr: tail_lines(&redact(stderr), STDERR_TAIL_LINES),
            git2: None,
        }
    }

    /// A libgit2 failure.
    pub fn from_git2(
        operation: &str,
        class: impl Into<String>,
        code: impl Into<String>,
        message: &str,
    ) -> Self {
        ErrorDetail {
            operation: operation.to_string(),
            command: None,
            exit_code: None,
            stderr: String::new(),
            git2: Some(Git2Detail {
                class: class.into(),
                code: code.into(),
                message: redact(message),
            }),
        }
    }

    /// The same detail under the user-level operation's name.
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = operation.into();
        self
    }

    /// The best-effort reading of what failed.
    pub fn kind(&self) -> FailureKind {
        let mut text = self.stderr.clone();
        if let Some(g) = &self.git2 {
            text.push('\n');
            text.push_str(&g.message);
            if g.code == "Auth" || g.class == "Ssh" && g.message.contains("auth") {
                return FailureKind::AuthFailed;
            }
        }
        FailureKind::classify(&text)
    }

    /// The facts as plain text lines, for a report.
    pub fn lines(&self) -> Vec<String> {
        let mut out = vec![format!("operation: {}", self.operation)];
        if let Some(c) = &self.command {
            out.push(format!("command: {c}"));
        }
        if let Some(code) = self.exit_code {
            out.push(format!("exit code: {code}"));
        }
        if let Some(g) = &self.git2 {
            out.push(format!(
                "libgit2: class={} code={} message={}",
                g.class, g.code, g.message
            ));
        }
        if !self.stderr.trim().is_empty() {
            out.push("stderr:".to_string());
            out.extend(self.stderr.lines().map(|l| format!("  {l}")));
        }
        out
    }
}

/// What kind of failure git's message describes. The frontends map each to
/// a sentence and, where one exists, an action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    /// The remote has commits this branch lacks: pull first.
    NonFastForward,
    /// Credentials missing, wrong or refused.
    AuthFailed,
    /// The configured credential helper itself could not run (a missing
    /// script, a helper not on `PATH`, a helper that quit): git then sent no
    /// credentials and the remote's refusal that follows is a consequence,
    /// not the cause. [`FailureKind::credential_helper_line`] has the line.
    CredentialHelperFailed,
    /// A merge, rebase or pull stopped on conflicts.
    MergeConflict,
    /// Local changes would be overwritten: commit, stash or discard.
    WouldOverwrite,
    /// ssh does not know the remote's host key and could not ask: connect
    /// once in a terminal to verify and add it.
    HostKeyUnverified,
    /// The remote could not be reached.
    NetworkUnreachable,
    /// Another git process holds `index.lock` (or another lock file).
    LockExists,
    /// An abort refused because files have local changes it would overwrite
    /// (`ConflictRefusal::LocalChanges`). Never read from text: the typed
    /// error sets it.
    LocalChangesBlockAbort,
    Other,
}

impl FailureKind {
    /// Classifies git's English stderr. Order matters: the specific
    /// "would be overwritten" and lock messages come before the broad ones.
    pub fn classify(stderr: &str) -> FailureKind {
        let s = stderr.to_lowercase();
        let has = |needles: &[&str]| needles.iter().any(|n| s.contains(n));
        if has(&[
            "would be overwritten by",
            "please commit your changes or stash them",
        ]) {
            FailureKind::WouldOverwrite
        } else if has(&[
            ".lock': file exists",
            ".lock\": file exists",
            "another git process seems to be running",
        ]) {
            FailureKind::LockExists
        } else if has(&[
            "host key verification failed",
            "no matching host key type found",
            "remote host identification has changed",
        ]) {
            FailureKind::HostKeyUnverified
        } else if credential_helper_line(stderr).is_some_and(|(_, certain)| {
            certain
                || has(&[
                    "authentication failed",
                    "could not read username",
                    "could not read password",
                    "terminal prompts disabled",
                    "invalid username or",
                    "the requested url returned error: 401",
                    "the requested url returned error: 403",
                ])
        }) {
            FailureKind::CredentialHelperFailed
        } else if has(&[
            "authentication failed",
            "could not read username",
            "could not read password",
            "permission denied (publickey",
            "permission denied (password",
            "invalid username or password",
            "terminal prompts disabled",
            "the requested url returned error: 401",
            "the requested url returned error: 403",
            "http basic: access denied",
            "support for password authentication was removed",
        ]) {
            FailureKind::AuthFailed
        } else if has(&[
            "could not resolve host",
            "connection refused",
            "connection timed out",
            "operation timed out",
            "network is unreachable",
            "failed to connect to",
            "could not connect to server",
            "temporary failure in name resolution",
            "connection reset",
            "unable to access",
        ]) {
            FailureKind::NetworkUnreachable
        } else if has(&[
            "conflict (",
            "automatic merge failed",
            "fix conflicts and",
            "merge conflict in",
        ]) {
            FailureKind::MergeConflict
        } else if has(&[
            "non-fast-forward",
            "fetch first",
            "updates were rejected because",
            "not possible to fast-forward",
            "divergent branches",
            "diverging branches",
        ]) {
            FailureKind::NonFastForward
        } else {
            FailureKind::Other
        }
    }
}

impl FailureKind {
    /// The stderr line that shows a credential helper failing to run, for a
    /// [`FailureKind::CredentialHelperFailed`] reading (`None` otherwise).
    pub fn credential_helper_line(stderr: &str) -> Option<String> {
        credential_helper_line(stderr).map(|(line, _)| line.trim().to_string())
    }
}

/// The first stderr line that reads as a credential helper that could not
/// run, with whether the line names the helper itself (`certain`) or is a
/// bare shell failure (`sh: /c/Users/x/token.sh: No such file or directory`)
/// that only counts next to an authentication failure.
///
/// The realistic shapes: a `credential.helper = !script` or an absolute path
/// run through `sh` (`sh: ...: No such file or directory`, `... not found`,
/// `Permission denied`, `/bin/sh: 1: x: not found`), `helper = foo` with no
/// `git-credential-foo` on `PATH` (`git: 'credential-foo' is not a git
/// command`), a helper that exited asking git to stop (`fatal: credential
/// helper '...' told us to quit`), and cmd.exe's "is not recognized".
fn credential_helper_line(stderr: &str) -> Option<(&str, bool)> {
    for line in stderr.lines() {
        // A message made of the command and its first stderr line:
        // `backend error: git fetch ... failed: sh: x: No such file`.
        let line = line
            .find(" failed: ")
            .map_or(line, |i| &line[i + " failed: ".len()..]);
        let lower = line.trim().to_lowercase();
        let l = lower.strip_prefix("error: ").unwrap_or(&lower);
        let l = l.strip_prefix("fatal: ").unwrap_or(l);
        let shell_failure = [
            "no such file or directory",
            "not found",
            "permission denied",
            "cannot execute",
            "exec format error",
            "bad interpreter",
        ]
        .iter()
        .any(|n| l.contains(n));
        let names_helper = l.contains("credential");
        // bash as `sh` names the script and the line instead:
        // `/x/helper get: line 1: /x/helper: No such file or directory`.
        let shell_prefix = [
            "sh: ",
            "/bin/sh: ",
            "bash: ",
            "/bin/bash: ",
            "dash: ",
            "/bin/dash: ",
        ]
        .iter()
        .any(|p| l.starts_with(p))
            || l.contains(": line ");
        if (l.contains("is not a git command") && l.contains("credential-"))
            || (l.contains("credential helper") && l.contains("told us to quit"))
            || (names_helper && shell_failure && (shell_prefix || l.contains("cannot run")))
            || (names_helper && l.contains("is not recognized as an internal or external command"))
        {
            return Some((line, true));
        }
        if shell_prefix && shell_failure {
            return Some((line, false));
        }
    }
    None
}

/// The last `n` lines of `text`, trimmed of surrounding blank lines.
pub fn tail_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.trim().lines().collect();
    let skip = lines.len().saturating_sub(n);
    let mut out = lines[skip..].join("\n");
    if skip > 0 {
        out.insert_str(0, &format!("[{skip} earlier lines omitted]\n"));
    }
    out
}

const MASK: &str = "***";

/// Keys whose value is a secret when written `key=value` or `key: value`.
const SECRET_KEYS: &[&str] = &[
    "password",
    "passwd",
    "pwd",
    "token",
    "secret",
    "api_key",
    "apikey",
    "api-key",
    "access_token",
    "private_token",
    "private-token",
    "client_secret",
    "authorization",
    "proxy-authorization",
    "x-api-key",
];

/// `text` with credentials masked: userinfo in URLs, `Authorization`
/// style headers, `Bearer`/`Basic` values, token-shaped words and
/// `password=...` pairs. Idempotent, line structure preserved.
pub fn redact(text: &str) -> String {
    let masked = text
        .split_inclusive('\n')
        .map(redact_line)
        .collect::<String>();
    mask_registered(masked)
}

/// Secrets this process knows exactly (a token it resolved for a grant),
/// masked wherever they appear whatever they look like: a token with no
/// known prefix is invisible to the shape rules above.
fn registered_secrets() -> &'static std::sync::Mutex<Vec<String>> {
    static SECRETS: std::sync::OnceLock<std::sync::Mutex<Vec<String>>> = std::sync::OnceLock::new();
    SECRETS.get_or_init(Default::default)
}

/// Shorter values would mask ordinary words.
const MIN_REGISTERED_SECRET: usize = 6;

/// Remembers `secret` for the life of the process: [`redact`] masks every
/// occurrence of it from now on. A value shorter than six characters is
/// ignored (it would mask ordinary text); registering twice is harmless.
pub fn redact_register_secret(secret: &str) {
    let secret = secret.trim();
    if secret.len() < MIN_REGISTERED_SECRET {
        return;
    }
    let mut all = registered_secrets()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if !all.iter().any(|s| s == secret) {
        all.push(secret.to_string());
    }
}

fn mask_registered(text: String) -> String {
    let all = registered_secrets()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    all.iter().fold(text, |text, secret| {
        if text.contains(secret.as_str()) {
            text.replace(secret.as_str(), "***")
        } else {
            text
        }
    })
}

fn redact_line(line: &str) -> String {
    let line = redact_urls(line);
    let line = redact_pairs(&line);
    let line = redact_bearer(&line);
    redact_prefixed_tokens(&line)
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '~' | '+' | '/' | '=')
}

/// `scheme://user:secret@host` becomes `scheme://***@host`.
fn redact_urls(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(pos) = rest.find("://") {
        let (head, tail) = rest.split_at(pos + 3);
        out.push_str(head);
        let end = tail
            .find(|c: char| c.is_whitespace() || matches!(c, '/' | '"' | '\'' | '<' | '>'))
            .unwrap_or(tail.len());
        let authority = &tail[..end];
        match authority.rfind('@') {
            Some(at) if at > 0 => {
                out.push_str(MASK);
                out.push_str(&authority[at..]);
            }
            _ => out.push_str(authority),
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

/// `key=value` / `key: value` for the secret keys (case-insensitive).
fn redact_pairs(line: &str) -> String {
    let lower = line.to_lowercase();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    let bytes = line.as_bytes();
    while i < line.len() {
        let mut matched = None;
        // Only at a key boundary: start of line or after a non-word char.
        let boundary = i == 0
            || !line[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
        if boundary && line.is_char_boundary(i) {
            for key in SECRET_KEYS {
                if lower[i..].starts_with(key) {
                    let after = i + key.len();
                    let mut j = after;
                    while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'"') {
                        j += 1;
                    }
                    if j < bytes.len() && (bytes[j] == b'=' || bytes[j] == b':') && j > after - 1 {
                        // key, optional spaces, separator, optional spaces
                        let mut k = j + 1;
                        while k < bytes.len() && bytes[k] == b' ' {
                            k += 1;
                        }
                        // A header value runs to the end of the line; a
                        // pair's value to whitespace, `&` or a quote.
                        let header = *key == "authorization" || *key == "proxy-authorization";
                        let mut end = k;
                        while end < line.len() {
                            let c = line[end..].chars().next().unwrap();
                            let stop = if header {
                                c == '\n' || c == '\r' || c == '"' || c == '\''
                            } else {
                                c.is_whitespace() || matches!(c, '&' | '"' | '\'' | ',' | ';')
                            };
                            if stop {
                                break;
                            }
                            end += c.len_utf8();
                        }
                        if end > k {
                            matched = Some((after, j + 1, k, end));
                        }
                    }
                    break;
                }
            }
        }
        match matched {
            Some((after, sep_end, value_start, end)) => {
                out.push_str(&line[i..after]);
                out.push_str(&line[after..sep_end]);
                out.push_str(&line[sep_end..value_start]);
                out.push_str(MASK);
                i = end;
            }
            None => {
                let c = line[i..].chars().next().unwrap();
                out.push(c);
                i += c.len_utf8();
            }
        }
    }
    out
}

/// `Bearer abc` and `Basic abc` (any case).
fn redact_bearer(line: &str) -> String {
    let lower = line.to_lowercase();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < line.len() {
        let rest = &lower[i..];
        let scheme = ["bearer ", "basic ", "token "]
            .into_iter()
            .find(|s| rest.starts_with(s));
        let boundary = i == 0
            || !line[..i]
                .chars()
                .next_back()
                .is_some_and(|c| c.is_ascii_alphanumeric());
        if let (Some(s), true) = (scheme, boundary) {
            let value_start = i + s.len();
            let value: String = line[value_start..]
                .chars()
                .take_while(|c| is_word_char(*c))
                .collect();
            // "basic " / "token " are ordinary words: only mask a value that
            // looks like a credential (long enough, no plain-word look).
            let credential = value.len() >= 12 || (s.starts_with("bearer") && value.len() >= 6);
            if credential && !value.contains(MASK) {
                out.push_str(&line[i..value_start]);
                out.push_str(MASK);
                i = value_start + value.len();
                continue;
            }
        }
        let c = line[i..].chars().next().unwrap();
        out.push(c);
        i += c.len_utf8();
    }
    out
}

/// Words that start with a known token prefix.
fn redact_prefixed_tokens(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        if !word.is_empty() {
            // The prefixes are the history scanner's table (`rewrite::secrets`),
            // so the log and the scanner mask the same shapes.
            let is_token = crate::domain::rewrite::secrets::token_prefixes()
                .any(|p| word.starts_with(p) && word.len() >= p.len() + 8);
            if is_token {
                out.push_str(MASK);
            } else {
                out.push_str(word);
            }
            word.clear();
        }
    };
    for c in line.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
            word.push(c);
        } else {
            flush(&mut word, &mut out);
            out.push(c);
        }
    }
    flush(&mut word, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_in_urls_are_masked() {
        assert_eq!(
            redact("fatal: unable to access 'https://user:hunter2@example.com/r.git/': x"),
            "fatal: unable to access 'https://***@example.com/r.git/': x"
        );
        assert_eq!(
            redact("remote https://ghp_abcdefghijklmnop@github.com/o/r"),
            "remote https://***@github.com/o/r"
        );
        // No userinfo: untouched.
        assert_eq!(
            redact("https://github.com/o/r.git"),
            "https://github.com/o/r.git"
        );
    }

    #[test]
    fn auth_headers_and_secret_pairs_are_masked() {
        let out = redact("-c http.extraheader=Authorization: Basic dXNlcjpwYXNz\nnext line");
        assert!(!out.contains("dXNlcjpwYXNz"), "{out}");
        assert!(out.ends_with("next line"), "{out}");
        let out = redact("curl -H 'Authorization: Bearer abcdef123456' x");
        assert!(!out.contains("abcdef123456"), "{out}");
        let out = redact("url?access_token=abc123xyz&per_page=5");
        assert!(
            !out.contains("abc123xyz") && out.contains("per_page=5"),
            "{out}"
        );
        let out = redact("password = s3cret value");
        assert!(!out.contains("s3cret"), "{out}");
        let out = redact("PRIVATE-TOKEN: glpat-xxxxxxxxxxxxxxxxxxxx");
        assert!(!out.contains("glpat-xxxx"), "{out}");
    }

    #[test]
    fn token_shaped_words_are_masked_but_prose_is_not() {
        let out = redact("pushed with ghp_0123456789abcdefghij to origin");
        assert_eq!(out, "pushed with *** to origin");
        let out = redact("key sk-ant-api03-ABCDEFGHIJKLMN end");
        assert!(!out.contains("ABCDEFGH"), "{out}");
        // Ordinary text survives, including words that merely contain keys.
        let plain = "error: failed to push some refs to 'origin' (tokenizer, basic setup)";
        assert_eq!(redact(plain), plain);
    }

    #[test]
    fn a_registered_secret_is_masked_even_without_a_known_prefix() {
        let secret = "zq81-plain-secret-value-4417";
        let line = format!("remote: denied for {secret} at host\n");
        assert!(redact(&line).contains(secret), "not registered yet");
        redact_register_secret(secret);
        redact_register_secret(secret);
        redact_register_secret("abc");
        assert_eq!(redact(&line), "remote: denied for *** at host\n");
        // Too short to be a secret: ordinary text stays readable.
        assert_eq!(redact("abc def"), "abc def");
    }

    #[test]
    fn redaction_is_idempotent() {
        let once = redact("https://u:p@h/x Authorization: Bearer abcdefghijkl token=zzz");
        assert_eq!(redact(&once), once);
    }

    #[test]
    fn a_detail_never_holds_the_secret_it_was_built_from() {
        let d = ErrorDetail::from_git(
            &["push", "https://me:topsecret@host/r.git"],
            Some(128),
            "fatal: Authentication failed for 'https://me:topsecret@host/r.git/'\n",
        );
        let all = format!("{:?}", d);
        assert!(!all.contains("topsecret"), "{all}");
        assert_eq!(d.kind(), FailureKind::AuthFailed);
    }

    #[test]
    fn stderr_is_cut_to_its_tail() {
        let long: String = (0..100).map(|i| format!("line {i}\n")).collect();
        let d = ErrorDetail::from_git(&["fetch"], Some(1), &long);
        assert!(d.stderr.contains("line 99"));
        assert!(!d.stderr.contains("line 10\n"));
        assert!(d.stderr.starts_with("[70 earlier lines omitted]"));
    }

    #[test]
    fn git_messages_map_to_a_friendly_kind() {
        use FailureKind as K;
        let cases = [
            (
                " ! [rejected]        main -> main (non-fast-forward)\nerror: failed to push some refs",
                K::NonFastForward,
            ),
            ("hint: Updates were rejected because the tip of your current branch is behind", K::NonFastForward),
            ("fatal: Not possible to fast-forward, aborting.", K::NonFastForward),
            ("remote: Invalid username or password.\nfatal: Authentication failed for 'https://h/r'", K::AuthFailed),
            ("git@h: Permission denied (publickey).\nfatal: Could not read from remote repository.", K::AuthFailed),
            ("fatal: could not read Username for 'https://h': terminal prompts disabled", K::AuthFailed),
            ("CONFLICT (content): Merge conflict in a.txt\nAutomatic merge failed; fix conflicts", K::MergeConflict),
            ("error: Your local changes to the following files would be overwritten by merge:\n\ta.txt", K::WouldOverwrite),
            ("fatal: Unable to create '/r/.git/index.lock': File exists.", K::LockExists),
            ("fatal: unable to access 'https://h/r/': Could not resolve host: h", K::NetworkUnreachable),
            ("Host key verification failed.\nfatal: Could not read from remote repository.", K::HostKeyUnverified),
            ("ssh: connect to host h port 22: Network is unreachable", K::NetworkUnreachable),
            ("something unexpected", K::Other),
            (HELPER_STDERR, K::CredentialHelperFailed),
        ];
        for (stderr, kind) in cases {
            assert_eq!(FailureKind::classify(stderr), kind, "{stderr}");
        }
    }

    #[test]
    fn libgit2_auth_failures_are_recognized() {
        let d = ErrorDetail::from_git2("fetch", "Http", "Auth", "authentication required");
        assert_eq!(d.kind(), FailureKind::AuthFailed);
    }

    const HELPER_STDERR: &str = "sh: /c/Users/X/.config/poulpi-lab/token.sh: No such file or directory\nremote: Invalid username or token. Password authentication is not supported for Git operations.\nfatal: Authentication failed for 'https://github.com/Poulpi-Lab/GitPoulpi.git/'";

    #[test]
    fn a_credential_helper_that_cannot_run_is_not_read_as_a_refusal() {
        use FailureKind as K;
        let line = K::credential_helper_line(HELPER_STDERR).unwrap();
        assert_eq!(
            line,
            "sh: /c/Users/X/.config/poulpi-lab/token.sh: No such file or directory"
        );
        for stderr in [
            "git: 'credential-foo' is not a git command. See 'git --help'.\nfatal: Authentication failed for 'https://h/r'",
            "/bin/sh: 1: /home/me/helper.sh: not found\nfatal: could not read Username for 'https://h': terminal prompts disabled",
            "sh: /usr/bin/git-credential-manager: Permission denied\nfatal: Authentication failed for 'https://h/r'",
            "fatal: credential helper '/x/y' told us to quit",
            "'git-credential-foo' is not recognized as an internal or external command,\nfatal: Authentication failed for 'https://h/r'",
            "error: cannot run git-credential-foo: No such file or directory\nfatal: Authentication failed for 'https://h/r'",
            "/x/credential-script get: line 1: /x/credential-script: No such file or directory\nfatal: could not read Username for 'http://h': No such device or address",
        ] {
            assert_eq!(K::classify(stderr), K::CredentialHelperFailed, "{stderr}");
            assert!(K::credential_helper_line(stderr).is_some(), "{stderr}");
        }
        // A plain refusal, and a bare shell failure without an auth failure
        // (not provably the helper), keep their readings.
        let refusal = "fatal: Authentication failed for 'https://h/r'";
        assert_eq!(K::classify(refusal), K::AuthFailed);
        let flat = format!("backend error: git fetch --all failed: {HELPER_STDERR}");
        assert_eq!(K::classify(&flat), K::CredentialHelperFailed);
        assert_eq!(
            K::credential_helper_line(&flat).as_deref(),
            Some("sh: /c/Users/X/.config/poulpi-lab/token.sh: No such file or directory")
        );
        assert_eq!(K::credential_helper_line(refusal), None);
        assert_eq!(
            K::classify("sh: ./hook.sh: not found"),
            K::Other,
            "a bare shell failure is not a helper failure on its own"
        );
        assert_eq!(
            K::classify("ssh: connect to host h port 22: Network is unreachable"),
            K::NetworkUnreachable
        );
    }
}
