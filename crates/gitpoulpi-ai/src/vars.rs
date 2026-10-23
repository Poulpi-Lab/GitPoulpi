//! Prompt variables: `${repo}`, `${uilang}`, ... in the user's own
//! instruction text, expanded right before a prompt is sent.
//!
//! [`expand`] is a pure function of the text and a [`PromptContext`] the
//! caller fills. [`PromptContext::from_repo`] reads the repository facts
//! (branch, HEAD, git user, remote) from the system `git`, so the TUI and
//! the CLI describe a repository identically.

use std::path::Path;
use std::process::Command;

/// One variable: its name (without `${}`), what it expands to, and an
/// English one-line description (the TUI translates its own).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VarInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub example: &'static str,
}

/// Every variable, in the order the editor lists them.
pub const VARIABLES: &[VarInfo] = &[
    VarInfo {
        name: "uilang",
        description: "UI language, English name",
        example: "French",
    },
    VarInfo {
        name: "repo",
        description: "Repository folder name",
        example: "gitpoulpi",
    },
    VarInfo {
        name: "branch",
        description: "Current branch",
        example: "main",
    },
    VarInfo {
        name: "ref",
        description: "Short sha of the commit concerned, or HEAD",
        example: "4c0443d",
    },
    VarInfo {
        name: "commit",
        description: "Same as ${ref}",
        example: "4c0443d",
    },
    VarInfo {
        name: "commit_subject",
        description: "Subject of that commit",
        example: "feat: add search",
    },
    VarInfo {
        name: "date",
        description: "Local date (ISO)",
        example: "2026-10-03",
    },
    VarInfo {
        name: "time",
        description: "Local time (HH:MM)",
        example: "14:05",
    },
    VarInfo {
        name: "datetime",
        description: "Local date and time",
        example: "2026-10-03 14:05",
    },
    VarInfo {
        name: "user",
        description: "Current user (git user.name)",
        example: "Ada Lovelace",
    },
    VarInfo {
        name: "email",
        description: "Current user's email (git user.email)",
        example: "ada@example.com",
    },
    VarInfo {
        name: "remote",
        description: "Default remote name",
        example: "origin",
    },
    VarInfo {
        name: "host",
        description: "Hosting provider: github, gitlab, other or none",
        example: "github",
    },
    VarInfo {
        name: "files",
        description: "Number of files changed in the action's diff",
        example: "3",
    },
    VarInfo {
        name: "action",
        description: "The AI action being run",
        example: "commit_message",
    },
];

/// The values the variables expand to. An empty field expands to nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PromptContext {
    pub uilang: String,
    pub repo: String,
    pub branch: String,
    pub commit: String,
    pub commit_subject: String,
    pub date: String,
    pub time: String,
    pub user: String,
    pub email: String,
    pub remote: String,
    pub host: String,
    pub files: String,
    pub action: String,
    /// The repository's folder (not a variable): where a local agent set
    /// to the `repository` working folder runs.
    pub repo_root: String,
}

impl PromptContext {
    /// The value of `name`, or `None` for a name that is not a variable.
    pub fn value(&self, name: &str) -> Option<String> {
        Some(match name {
            "uilang" => self.uilang.clone(),
            "repo" => self.repo.clone(),
            "branch" => self.branch.clone(),
            "ref" | "commit" => self.commit.clone(),
            "commit_subject" => self.commit_subject.clone(),
            "date" => self.date.clone(),
            "time" => self.time.clone(),
            "datetime" => format!("{} {}", self.date, self.time).trim().to_string(),
            "user" => self.user.clone(),
            "email" => self.email.clone(),
            "remote" => self.remote.clone(),
            "host" => self.host.clone(),
            "files" => self.files.clone(),
            "action" => self.action.clone(),
            _ => return None,
        })
    }

    /// Fills the local date and time (now).
    pub fn with_now(mut self) -> Self {
        let now = chrono::Local::now();
        self.date = now.format("%Y-%m-%d").to_string();
        self.time = now.format("%H:%M").to_string();
        self
    }

    /// Reads the repository facts at `root` from the system `git`.
    /// Whatever cannot be read stays empty.
    pub fn from_repo(root: &Path) -> Self {
        let git = |args: &[&str]| -> String {
            let mut cmd = Command::new("git");
            cmd.arg("-C").arg(root).args(args);
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x0800_0000);
            }
            cmd.output()
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default()
        };
        let remote = {
            let remotes = git(&["remote"]);
            let mut names = remotes.lines();
            let first = names.next().unwrap_or("").to_string();
            if remotes.lines().any(|r| r == "origin") {
                "origin".to_string()
            } else {
                first
            }
        };
        let url = if remote.is_empty() {
            String::new()
        } else {
            git(&["remote", "get-url", &remote])
        };
        let mut ctx = Self {
            repo: root
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            branch: git(&["branch", "--show-current"]),
            commit: git(&["rev-parse", "--short", "HEAD"]),
            commit_subject: git(&["log", "-1", "--format=%s"]),
            user: git(&["config", "user.name"]),
            email: git(&["config", "user.email"]),
            host: host_of(&url, !remote.is_empty()),
            remote,
            repo_root: root.to_string_lossy().into_owned(),
            ..Self::default()
        };
        if ctx.branch.is_empty() && !ctx.commit.is_empty() {
            ctx.branch = "HEAD".to_string();
        }
        ctx.with_now()
    }
}

/// `github`, `gitlab`, `other` (a remote on some other host) or `none`.
fn host_of(url: &str, has_remote: bool) -> String {
    let lower = url.to_ascii_lowercase();
    if lower.contains("github") {
        "github"
    } else if lower.contains("gitlab") {
        "gitlab"
    } else if has_remote {
        "other"
    } else {
        "none"
    }
    .to_string()
}

/// The variables `${name}` in `text`, in order, whether known or not,
/// with their byte ranges (covering `${` to `}`). `$${` is an escape and
/// yields nothing.
pub fn scan(text: &str) -> Vec<(std::ops::Range<usize>, &str)> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && bytes.get(i + 1) == Some(&b'$') && bytes.get(i + 2) == Some(&b'{') {
            i += 3;
            continue;
        }
        if bytes[i] == b'$' && bytes.get(i + 1) == Some(&b'{') {
            if let Some(close) = text[i + 2..].find('}') {
                let name = &text[i + 2..i + 2 + close];
                if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    out.push((i..i + 3 + close, name));
                    i += 3 + close;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

/// The names in `text` that are not variables, each once, in order.
pub fn unknown_in(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (_, name) in scan(text) {
        if !VARIABLES.iter().any(|v| v.name == name) && !out.iter().any(|o| o == name) {
            out.push(name.to_string());
        }
    }
    out
}

/// `text` with every known `${name}` replaced from `ctx`. Unknown ones
/// stay as they are; `$${` becomes a literal `${`.
pub fn expand(text: &str, ctx: &PromptContext) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    let mut cursor = 0;
    let bytes = text.as_bytes();
    let found = scan(text);
    let mut next = found.iter().peekable();
    while cursor < bytes.len() {
        if let Some((range, name)) = next.peek() {
            if range.start == cursor {
                if let Some(value) = ctx.value(name) {
                    out.push_str(&text[last..range.start]);
                    out.push_str(&value);
                    last = range.end;
                }
                cursor = range.end;
                next.next();
                continue;
            }
        }
        if bytes[cursor] == b'$'
            && bytes.get(cursor + 1) == Some(&b'$')
            && bytes.get(cursor + 2) == Some(&b'{')
        {
            out.push_str(&text[last..cursor]);
            out.push('$');
            last = cursor + 2;
            // Skip past the escaped `${` so it is never a variable.
            cursor += 3;
            continue;
        }
        cursor += 1;
    }
    out.push_str(&text[last..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> PromptContext {
        PromptContext {
            uilang: "French".into(),
            repo: "poulpi".into(),
            branch: "main".into(),
            commit: "abc1234".into(),
            commit_subject: "fix: x".into(),
            date: "2026-10-03".into(),
            time: "14:05".into(),
            user: "Ada".into(),
            email: "a@b.c".into(),
            remote: "origin".into(),
            host: "github".into(),
            files: "3".into(),
            repo_root: String::new(),
            action: "commit_message".into(),
        }
    }

    #[test]
    fn known_variables_expand() {
        let c = ctx();
        assert_eq!(
            expand("Write in ${uilang} for ${repo} (${branch}@${ref}).", &c),
            "Write in French for poulpi (main@abc1234)."
        );
        assert_eq!(
            expand("${datetime} ${commit} ${files}", &c),
            "2026-10-03 14:05 abc1234 3"
        );
        for v in VARIABLES {
            assert!(c.value(v.name).is_some(), "{}", v.name);
        }
    }

    #[test]
    fn unknown_variables_stay_and_are_reported() {
        let c = ctx();
        assert_eq!(expand("a ${nope} b ${repo}", &c), "a ${nope} b poulpi");
        assert_eq!(unknown_in("${nope} ${repo} ${nope} ${x}"), ["nope", "x"]);
        assert!(unknown_in("${repo} $${nope}").is_empty());
    }

    #[test]
    fn dollar_dollar_brace_escapes() {
        let c = ctx();
        assert_eq!(
            expand("literal $${repo} and ${repo}", &c),
            "literal ${repo} and poulpi"
        );
        assert_eq!(expand("cost $5, $, ${", &c), "cost $5, $, ${");
        assert_eq!(expand("${repo", &c), "${repo");
    }

    #[test]
    fn empty_value_and_non_ascii_text() {
        let mut c = ctx();
        c.user.clear();
        assert_eq!(expand("é[${user}]ü", &c), "é[]ü");
    }

    #[test]
    fn host_detection() {
        assert_eq!(host_of("git@github.com:a/b.git", true), "github");
        assert_eq!(host_of("https://gitlab.corp/x", true), "gitlab");
        assert_eq!(host_of("ssh://h/x", true), "other");
        assert_eq!(host_of("", false), "none");
    }
}
