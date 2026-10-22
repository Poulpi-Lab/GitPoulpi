//! The "Create Rules with AI" action of the history rewrite workbench: the
//! fixed contract (the rule vocabulary and the answer's JSON shape) and the
//! user message (the description, and the repository context only when the
//! user ticked the box).
//!
//! The vocabulary is a table of kind, meaning and a JSON example. A test
//! parses every example into the workbench's own `Rule` and validates it, so
//! the prompt cannot drift from the types the answer is parsed into.

/// One rule kind the model may use: its `type`, what it does, and an example
/// of the `kind` object.
pub struct VocabEntry {
    pub kind: &'static str,
    pub does: &'static str,
    pub example: &'static str,
}

/// Every rule kind the AI may create. `script` (arbitrary code) and the
/// secret-literal form of `content_replace` are deliberately absent: the
/// first has its own action (a reviewed Lua script), the second would put
/// secret values in the model's hands.
pub const VOCABULARY: &[VocabEntry] = &[
    VocabEntry {
        kind: "identity",
        does: "replace an author and/or committer (and optionally annotated tag taggers and \
               Name <email> trailers) matching `when` by `set`. `when` matches the whole value; \
               emails ignore case, names do not; `regex: true` makes both regular expressions. \
               `apply_to`: author | committer | both (default both).",
        example: r#"{"type":"identity","when":{"email":"old@example.org"},"set":{"name":"Ada Lovelace","email":"ada@example.com"},"apply_to":"both","include_taggers":false,"map_trailers":false}"#,
    },
    VocabEntry {
        kind: "mailmap",
        does: "make authors/committers what the worktree's .mailmap says (`apply_to` as above).",
        example: r#"{"type":"mailmap","apply_to":"both"}"#,
    },
    VocabEntry {
        kind: "anonymise",
        does: "replace every identity by a numbered placeholder; `keep` lists identities left as \
               they are (same shape as `when`).",
        example: r#"{"type":"anonymise","keep":[{"email":"bot@example.org"}],"name_pattern":"Contributor {n}","email_pattern":"contributor-{n}@anonymous.invalid","apply_to":"both"}"#,
    },
    VocabEntry {
        kind: "date_shift",
        does: "move dates by `by_seconds` (negative = earlier, never 0). +2h is 7200. \
               `apply_to`: author_date | committer_date | both.",
        example: r#"{"type":"date_shift","by_seconds":7200,"apply_to":"both"}"#,
    },
    VocabEntry {
        kind: "date_timezone",
        does: "write dates in another UTC offset (`offset_minutes`, minutes east of UTC); `keep`: \
               instant (same moment, the clock changes) | wall_clock (same clock, the moment \
               changes).",
        example: r#"{"type":"date_timezone","offset_minutes":60,"keep":"instant","apply_to":"both"}"#,
    },
    VocabEntry {
        kind: "date_window",
        does: "move commits that fall outside the allowed hours to the nearest allowed time. \
               `windows`: days (mon..sun), start_minute and end_minute (0..1440, end exclusive). \
               `direction`: nearest | later_only | earlier_only. `offset`: {\"zone\":\"commit\"} \
               or {\"zone\":\"fixed\",\"minutes\":60}. `keep_order` keeps a commit after its parent.",
        example: r#"{"type":"date_window","windows":[{"days":["mon","tue","wed","thu","fri"],"start_minute":1080,"end_minute":1410}],"offset":{"zone":"commit"},"direction":"nearest","keep_order":true,"min_gap_seconds":60,"seed":0,"apply_to":"both"}"#,
    },
    VocabEntry {
        kind: "committer_from_author",
        does: "the committer (name, email, date) becomes the author's.",
        example: r#"{"type":"committer_from_author"}"#,
    },
    VocabEntry {
        kind: "message_replace",
        does: "search and replace in commit messages; `regex` (default false) and \
               `case_sensitive` (default true); an empty `with` deletes; `$1` works in regex mode.",
        example: r#"{"type":"message_replace","find":{"pattern":"WIP: ","regex":false,"case_sensitive":true},"with":""}"#,
    },
    VocabEntry {
        kind: "message_delete_lines",
        does: "delete the message lines that match `pattern` (same shape as `find` above).",
        example: r#"{"type":"message_delete_lines","pattern":{"pattern":"^Change-Id: ","regex":true,"case_sensitive":true}}"#,
    },
    VocabEntry {
        kind: "trailer",
        does: "`action` add | remove | map on the trailers named `key`. add: `value`, `if_exists` \
               add | replace | skip. remove: optional `find` (a pattern) limits it to matching \
               values. map: `find` pattern replaced by `with` in the values.",
        example: r#"{"type":"trailer","action":"add","key":"Signed-off-by","value":"Ada Lovelace <ada@example.com>","if_exists":"skip"}"#,
    },
    VocabEntry {
        kind: "content_replace",
        does: "search and replace inside files whose path matches the `paths` globs (empty = \
               every file). Only the pattern form: `\"find\":\"pattern\"` with `pattern`, `regex`, \
               `case_sensitive`, and `with`. Optional `max_blob_bytes`.",
        example: r#"{"type":"content_replace","paths":["*.md"],"find":"pattern","pattern":"http://old.example.org","regex":false,"case_sensitive":true,"with":"https://new.example.org"}"#,
    },
    VocabEntry {
        kind: "remove_paths",
        does: "delete the files and folders matching the globs from every commit.",
        example: r#"{"type":"remove_paths","globs":["secrets.txt","config/*.key"]}"#,
    },
    VocabEntry {
        kind: "remove_blobs_over",
        does: "delete every file larger than `bytes` from every commit.",
        example: r#"{"type":"remove_blobs_over","bytes":10485760}"#,
    },
    VocabEntry {
        kind: "move_paths",
        does: "`move`: rename (`from`, `to`) | into_subdir (`dir`) | extract_subdir (`dir`, \
               becomes the root; leave `mode` out, the user is asked).",
        example: r#"{"type":"move_paths","move":"rename","from":"src/old","to":"src/new"}"#,
    },
    VocabEntry {
        kind: "lfs_import",
        does: "move the files matching `patterns` into Git LFS (optional `min_size` in bytes).",
        example: r#"{"type":"lfs_import","patterns":["*.psd"],"min_size":1048576}"#,
    },
    VocabEntry {
        kind: "lfs_export",
        does: "bring the files matching `patterns` back from Git LFS as real files.",
        example: r#"{"type":"lfs_export","patterns":["*.psd"]}"#,
    },
    VocabEntry {
        kind: "line_endings",
        does: "give the text files matching `paths` (empty = all) one line ending, `target`: lf | \
               crlf | cr; `exclude` globs are left alone.",
        example: r#"{"type":"line_endings","target":"lf","paths":[],"exclude":["*.bat"]}"#,
    },
    VocabEntry {
        kind: "strip_exec",
        does: "remove the executable bit from the files matching `paths` (at least one glob).",
        example: r#"{"type":"strip_exec","paths":["*.txt"]}"#,
    },
    VocabEntry {
        kind: "reencode",
        does: "convert the text files matching `paths` to `target`: utf8 | utf8_bom | utf16_le | \
               utf16_be | windows1252 | latin1. Optional `from` (else detected) and `fallback`.",
        example: r#"{"type":"reencode","target":"utf8","from":"windows1252","paths":["*.txt"]}"#,
    },
    VocabEntry {
        kind: "remove_signatures",
        does: "strip the GPG/SSH signature of every commit in scope.",
        example: r#"{"type":"remove_signatures"}"#,
    },
    VocabEntry {
        kind: "message_encoding",
        does: "re-encode commit messages to UTF-8 (`names`: also the author and committer names).",
        example: r#"{"type":"message_encoding","names":false}"#,
    },
];

/// [`base_prompt`], built once (the prompt table hands out `&'static str`).
pub fn base_prompt_static() -> &'static str {
    static BASE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    BASE.get_or_init(base_prompt)
}

/// The base prompt: the answer's shape, the scope and options vocabulary,
/// then the rule vocabulary from [`VOCABULARY`].
pub fn base_prompt() -> String {
    let mut out = String::from(
        "\
You prepare a history rewrite recipe for GitPoulpi from the user's description. You only \
PREPARE it: the user reads, edits, previews and applies it themselves. You never run anything. \
Answer with one JSON object in exactly one ```json code block and nothing else:

{
  \"scope\": {\"scope\": \"current_branch\"},
  \"scope_options\": {\"include_tags\": true},
  \"options\": {\"committer\": \"keep\", \"prune_empty\": \"auto\"},
  \"rules\": [ {\"label\": \"short title\", \"kind\": { ...one rule kind... }} ],
  \"notes\": [\"anything you could not express as a rule, or must warn about\"]
}

Every key but `rules` is optional: leave out what the user did not ask to change.

scope: {\"scope\":\"current_branch\"} (default), {\"scope\":\"all_local\"} (every local branch and \
tag) or {\"scope\":\"refs\",\"refs\":[\"main\",\"release\"]} (only names that exist). Nothing else \
can be expressed: a time limit such as \"since March\", a commit range or a selection is not a \
scope; say so in `notes` (a date window is not a rule either: do not invent one).
scope_options: include_tags (default true) moves the tags that point at rewritten commits.
options: committer \"keep\" (default: only a rule changes the committer) or \"me_now\"; \
prune_empty \"auto\" (default, drops commits a content rule emptied) or \"never\"; \
update_message_hashes (bool, default false); keep_backup_refs (bool, default false). \
Never set signatures: the user is asked when it matters.

Rules run in order. Use the fewest rules that do what was asked, only the kinds below, and \
exactly the fields shown (JSON, no comments). Paths are globs relative to the repository root. \
Never invent an email, name, path or date the user did not give or the context does not list: \
put the question in `notes` instead of guessing. Never create a rule that runs code.

Rule kinds (`kind.type`), each with an example of its `kind` object:
",
    );
    for entry in VOCABULARY {
        out.push_str(&format!(
            "\n- {}: {}\n  {}\n",
            entry.kind, entry.does, entry.example
        ));
    }
    out
}

/// What the model may be told about the repository. Empty unless the user
/// ticked the box: the instruction alone is the default.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RulesContext {
    /// The scope the workbench shows ("the current branch", "every local
    /// branch and tag").
    pub scope: String,
    pub current_branch: Option<String>,
    /// Local branch and tag names.
    pub refs: Vec<String>,
    /// Distinct authors of the scope: (name, email).
    pub authors: Vec<(String, String)>,
}

impl RulesContext {
    pub fn is_empty(&self) -> bool {
        self.scope.is_empty()
            && self.current_branch.is_none()
            && self.refs.is_empty()
            && self.authors.is_empty()
    }
}

/// How many refs and authors one request carries at most.
pub const MAX_CONTEXT_ITEMS: usize = 100;

/// The user message: the description, then the context (only when `context`
/// is not empty) and how many rules the workbench already holds.
pub fn rewrite_rules(description: &str, context: &RulesContext) -> String {
    let mut out = format!(
        "What the history rewrite must do:\n{}\n",
        description.trim()
    );
    if !context.is_empty() {
        out.push_str("\nThe repository (sent because the user allowed it):\n");
        if !context.scope.is_empty() {
            out.push_str(&format!("- scope on screen: {}\n", context.scope));
        }
        if let Some(branch) = &context.current_branch {
            out.push_str(&format!("- current branch: {branch}\n"));
        }
        if !context.refs.is_empty() {
            out.push_str("- branches and tags:\n");
            for r in context.refs.iter().take(MAX_CONTEXT_ITEMS) {
                out.push_str(&format!("  - {r}\n"));
            }
        }
        if !context.authors.is_empty() {
            out.push_str("- authors of the history (name <email>):\n");
            for (name, email) in context.authors.iter().take(MAX_CONTEXT_ITEMS) {
                out.push_str(&format!("  - {name} <{email}>\n"));
            }
        }
    }
    out
}

/// The second attempt: the prompt, then why the first answer was rejected.
pub fn rewrite_rules_retry(prompt: &str, reason: &str) -> String {
    format!(
        "{prompt}\nYour previous answer was rejected: {reason}\nAnswer again with the complete \
         JSON object in one ```json code block only."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_base_prompt_names_every_kind_with_its_example() {
        let p = base_prompt();
        for e in VOCABULARY {
            assert!(p.contains(&format!("- {}:", e.kind)), "{}", e.kind);
            assert!(p.contains(e.example), "{}", e.kind);
        }
        assert!(p.contains("current_branch") && p.contains("all_local"));
        assert!(p.contains("You never run anything"));
    }

    #[test]
    fn without_context_the_message_holds_only_the_description() {
        let p = rewrite_rules("  drop secrets.txt ", &RulesContext::default());
        assert_eq!(p, "What the history rewrite must do:\ndrop secrets.txt\n");
    }

    #[test]
    fn the_context_lists_scope_refs_and_authors() {
        let c = RulesContext {
            scope: "the current branch".into(),
            current_branch: Some("main".into()),
            refs: vec!["main".into(), "v1".into()],
            authors: vec![("Ada".into(), "ada@example.org".into())],
        };
        let p = rewrite_rules("x", &c);
        for needle in [
            "scope on screen: the current branch",
            "current branch: main",
            "  - v1\n",
            "  - Ada <ada@example.org>\n",
        ] {
            assert!(p.contains(needle), "{needle}\n{p}");
        }
    }
}
