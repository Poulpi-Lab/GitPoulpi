//! The history-rewrite script action: the fixed contract part (the system
//! prompt's base) and the user message (the instruction, and a sample of
//! commit metadata only when the user asked for it).

use gitpoulpi_core::domain::CommitNode;

/// How many sample commits one request may carry at most.
pub const MAX_SAMPLE: usize = 50;

/// The base prompt: what a script is, the contract and the sandbox, as
/// short as it can be while the model has everything it needs.
pub const BASE_PROMPT: &str = "\
You write one Lua 5.4 script that rewrites git history, for GitPoulpi. Answer with the script in \
exactly one ```lua code block and nothing else.

The script starts with a header of `--!` lines, then defines `rewrite_commit(c, ctx)`, which is \
called once per commit, oldest first:

--! name: <short title>
--! kinds: commit
--! window: 0
--! param <name>: text|number|bool \"<label>\" = <default>

Header: `name` and `kinds: commit` are required; `window: N` (0 to 50) is how many neighbours \
ctx.window.before / ctx.window.after hold; one `--! param` line per value the user may want to \
change (read it as ctx.params.<name>; give a default when there is a sensible one).

`c` is the commit: c.id, c.parents, c.message, c.signed, and c.author and c.committer, each with \
name, email, time (Unix seconds, UTC) and offset_minutes (the time zone, minutes east of UTC). \
`ctx` holds index, total (counts), params, state (a table carried from one call to the next: \
return `state = {...}` to change it) and window.before / window.after (neighbouring commits with \
id, author, committer, subject).

Return nothing to leave the commit as it is, or a table with only what changes: \
`author = { name, email, time, offset_minutes }` (any subset), `committer = { ... }` (same), \
`message = \"...\"`, `state = {...}`. Any other field is an error. Never return tree or parents.

Helpers: regex_replace(text, pattern, replacement) (Rust regex syntax, $1 in the replacement), \
regex_match(text, pattern) -> boolean, stable_hash(text, salt) -> integer, rand(seed) -> number \
in [0, 1), weekday(time, offset_minutes) (1 = Monday .. 7 = Sunday), hour(time, offset_minutes) \
(0 to 23), trailers(message) -> list of { key, value }, set_trailer(message, key, value) -> message \
(replaces the key's trailers; a fourth argument 'add' or 'skip' changes that), \
remove_trailer(message, key) -> message, at('YYYY-MM-DD HH:MM', offset_minutes) -> unix time, log(text).

Sandbox: only the string, table, math and utf8 libraries exist. There is no io, os, require, \
load, dofile, print, pcall or clock, and math.random is missing (use rand(seed) or stable_hash). \
The script must be deterministic. A call is limited to 10 million instructions and 64 MiB.

Keep it short, readable and commented where the intent is not obvious. Change only what the user \
asked for.";

/// What the model sees of one commit when the user ticked "Send a sample":
/// who, when and the subject. Never file contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleCommit {
    pub author_name: String,
    pub author_email: String,
    /// Unix seconds, UTC.
    pub timestamp: i64,
    pub subject: String,
}

impl SampleCommit {
    pub fn from_node(node: &CommitNode) -> Self {
        Self {
            author_name: node.author_name.clone(),
            author_email: node.author_email.clone(),
            timestamp: node.timestamp,
            subject: node.summary.clone(),
        }
    }

    fn line(&self) -> String {
        let when = chrono::DateTime::from_timestamp(self.timestamp, 0)
            .map(|d| d.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            .unwrap_or_else(|| self.timestamp.to_string());
        let subject: String = self
            .subject
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(120)
            .collect();
        format!(
            "- {} <{}> | {when} | {subject}",
            self.author_name, self.author_email
        )
    }
}

/// The user message: the instruction, then (only with a non-empty `sample`,
/// which the caller builds only when the user ticked the box) the commit
/// headers.
pub fn rewrite_script(instruction: &str, sample: &[SampleCommit]) -> String {
    let mut out = format!("What the script must do:\n{}\n", instruction.trim());
    if !sample.is_empty() {
        out.push_str(&format!(
            "\nA sample of {} commits of the history (author, date, subject), to show what the \
             data looks like:\n",
            sample.len().min(MAX_SAMPLE)
        ));
        for commit in sample.iter().take(MAX_SAMPLE) {
            out.push_str(&commit.line());
            out.push('\n');
        }
    }
    out
}

/// The second attempt: the prompt, then why the first answer was rejected.
pub fn rewrite_script_retry(prompt: &str, reason: &str) -> String {
    format!(
        "{prompt}\nYour previous answer was rejected: {reason}\nAnswer again with the complete \
         corrected script in one ```lua code block only."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_a_sample_the_message_holds_only_the_instruction() {
        let p = rewrite_script("  move night commits  ", &[]);
        assert_eq!(p, "What the script must do:\nmove night commits\n");
    }

    #[test]
    fn a_sample_lists_who_when_and_subject_only() {
        let c = SampleCommit {
            author_name: "Ada".into(),
            author_email: "ada@example.org".into(),
            timestamp: 86_400,
            subject: "fix: x\nbody".into(),
        };
        let p = rewrite_script("do it", &[c]);
        assert!(
            p.contains("- Ada <ada@example.org> | 1970-01-02 00:00:00 UTC | fix: x\n"),
            "{p}"
        );
        assert!(!p.contains("body"), "{p}");
    }
}
