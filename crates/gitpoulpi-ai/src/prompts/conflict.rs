//! "Suggest a resolution" for ONE merge conflict: the prompt builder and
//! the parser of the answer.
//!
//! The model sees the file path and language, the conflict's base, ours
//! and theirs blocks (with the role labels the editor shows, never a bare
//! "ours" / "theirs": during a rebase git swaps their meaning), and up to
//! [`CONTEXT_LINES`] lines of the text git merged by itself on each side.
//! It answers with the resolved text of that region only. Everything is
//! capped ([`SIDE_BUDGET`], [`CONTEXT_BUDGET`], [`MAX_ANSWER_BYTES`]) so a
//! huge conflict fails with a sentence instead of a provider error.

use gitpoulpi_core::domain::{ConflictSide, MergeDocument, MergeRegion};

use crate::{Error, Result};

/// Lines of surrounding merged text sent on each side of the conflict.
pub const CONTEXT_LINES: usize = 20;
/// Most bytes the three blocks of the conflict may add up to.
pub const SIDE_BUDGET: usize = 40_000;
/// Most bytes of surrounding context, each side.
pub const CONTEXT_BUDGET: usize = 6_000;
/// An answer longer than this is rejected as junk.
pub const MAX_ANSWER_BYTES: usize = 200_000;

/// What the model is told about one conflict. Owned, so a worker thread
/// can carry it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictContext {
    /// The file's path, with `/` separators.
    pub path: String,
    /// 1-based, for "conflict 2 of 5".
    pub index: usize,
    pub total: usize,
    pub base: Vec<String>,
    /// Whether the file has a common ancestor at all (an empty `base` is
    /// then a real, empty ancestor block).
    pub has_base: bool,
    pub ours: Vec<String>,
    pub theirs: Vec<String>,
    /// The editor's role labels (`Current · main`, `Incoming · feature`).
    pub ours_label: String,
    pub theirs_label: String,
    /// The merged lines just before the conflict, nearest last.
    pub before: Vec<String>,
    /// The merged lines just after it.
    pub after: Vec<String>,
}

impl ConflictContext {
    /// The context of conflict `n` (0-based) of `doc`, or `None` when the
    /// document has no such conflict.
    pub fn from_document(doc: &MergeDocument, n: usize) -> Option<Self> {
        let at = doc
            .regions
            .iter()
            .enumerate()
            .filter(|(_, r)| r.is_conflict())
            .nth(n)
            .map(|(i, _)| i)?;
        let region = &doc.regions[at];
        let before = match at.checked_sub(1).map(|i| &doc.regions[i]) {
            Some(MergeRegion::Merged { lines }) => lines.clone(),
            _ => Vec::new(),
        };
        let after = match doc.regions.get(at + 1) {
            Some(MergeRegion::Merged { lines }) => lines.clone(),
            _ => Vec::new(),
        };
        let labels = &doc.labels;
        Some(Self {
            path: doc.entry.path.to_string_lossy().replace('\\', "/"),
            index: n + 1,
            total: doc.conflict_count(),
            base: region.lines_of(ConflictSide::Base).to_vec(),
            has_base: doc.base.is_some(),
            ours: region.lines_of(ConflictSide::Ours).to_vec(),
            theirs: region.lines_of(ConflictSide::Theirs).to_vec(),
            ours_label: labels.current.clone(),
            theirs_label: labels.incoming.clone(),
            before,
            after,
        })
    }
}

/// A language name for the path's extension, for the prompt (and the code
/// fence). `None` for an extension this does not know.
pub fn language_of(path: &str) -> Option<&'static str> {
    let name = path.rsplit('/').next().unwrap_or(path);
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    Some(match ext.as_deref() {
        Some("rs") => "Rust",
        Some("py") => "Python",
        Some("js" | "mjs" | "cjs") => "JavaScript",
        Some("ts") => "TypeScript",
        Some("tsx") => "TypeScript (TSX)",
        Some("jsx") => "JavaScript (JSX)",
        Some("go") => "Go",
        Some("java") => "Java",
        Some("kt") => "Kotlin",
        Some("cs") => "C#",
        Some("c" | "h") => "C",
        Some("cc" | "cpp" | "cxx" | "hpp") => "C++",
        Some("rb") => "Ruby",
        Some("php") => "PHP",
        Some("swift") => "Swift",
        Some("sh" | "bash") => "shell",
        Some("ps1") => "PowerShell",
        Some("toml") => "TOML",
        Some("yaml" | "yml") => "YAML",
        Some("json") => "JSON",
        Some("md") => "Markdown",
        Some("html" | "htm") => "HTML",
        Some("css") => "CSS",
        Some("scss") => "SCSS",
        Some("sql") => "SQL",
        Some("xml") => "XML",
        Some("lock") => "lock file",
        _ => return None,
    })
}

fn bytes_of(lines: &[String]) -> usize {
    lines.iter().map(|l| l.len() + 1).sum()
}

/// The last `CONTEXT_LINES` lines of `lines` (the first ones when
/// `from_end` is false) that also fit `CONTEXT_BUDGET` bytes.
fn trim_context(lines: &[String], from_end: bool) -> Vec<&str> {
    let mut picked: Vec<&str> = Vec::new();
    let mut used = 0;
    let iter: Box<dyn Iterator<Item = &String>> = if from_end {
        Box::new(lines.iter().rev())
    } else {
        Box::new(lines.iter())
    };
    for line in iter.take(CONTEXT_LINES) {
        used += line.len() + 1;
        if used > CONTEXT_BUDGET {
            break;
        }
        picked.push(line);
    }
    if from_end {
        picked.reverse();
    }
    picked
}

fn block(title: &str, lines: &[String], fence_lang: &str) -> String {
    let mut out = format!("### {title}\n```{fence_lang}\n");
    for line in lines {
        out.push_str(line.trim_end_matches('\r'));
        out.push('\n');
    }
    out.push_str("```\n\n");
    out
}

/// The user message of "suggest a resolution". `Err` when the conflict is
/// too large to send.
pub fn conflict_resolve(c: &ConflictContext) -> Result<String> {
    let total = bytes_of(&c.base) + bytes_of(&c.ours) + bytes_of(&c.theirs);
    if total > SIDE_BUDGET {
        return Err(Error::Invalid(format!(
            "this conflict is too large to send to the AI ({total} bytes; the limit is {SIDE_BUDGET})"
        )));
    }
    let language = language_of(&c.path);
    let fence = match language {
        Some(l) => l.split(' ').next().unwrap_or("").to_ascii_lowercase(),
        None => String::new(),
    };
    let mut out = format!(
        "Resolve conflict {} of {} in `{}`{}.\n\n",
        c.index,
        c.total.max(1),
        c.path,
        language.map(|l| format!(" ({l})")).unwrap_or_default(),
    );
    let ours_label = if c.ours_label.is_empty() {
        "Current"
    } else {
        &c.ours_label
    };
    let theirs_label = if c.theirs_label.is_empty() {
        "Incoming"
    } else {
        &c.theirs_label
    };
    let before: Vec<String> = trim_context(&c.before, true)
        .into_iter()
        .map(str::to_string)
        .collect();
    let after: Vec<String> = trim_context(&c.after, false)
        .into_iter()
        .map(str::to_string)
        .collect();
    if !before.is_empty() {
        out.push_str(&block(
            "Context before the conflict (already merged)",
            &before,
            &fence,
        ));
    }
    if c.has_base {
        out.push_str(&block("Common ancestor (base)", &c.base, &fence));
    } else {
        out.push_str("### Common ancestor (base)\nnone: both sides added this.\n\n");
    }
    out.push_str(&block(&format!("Side A: {ours_label}"), &c.ours, &fence));
    out.push_str(&block(
        &format!("Side B: {theirs_label}"),
        &c.theirs,
        &fence,
    ));
    if !after.is_empty() {
        out.push_str(&block(
            "Context after the conflict (already merged)",
            &after,
            &fence,
        ));
    }
    out.push_str(
        "Answer with the resolved text that replaces the conflict, in one code block. \
         Keep both intents when they are independent. No conflict markers, no explanation.",
    );
    Ok(out)
}

/// Whether `line` is a git conflict marker (seven or more repeats).
fn is_marker(line: &str) -> bool {
    let line = line.trim_end_matches('\r');
    ['<', '=', '>', '|'].into_iter().any(|ch| {
        let run = line.chars().take_while(|&x| x == ch).count();
        run >= 7 && (line.len() == run || line.as_bytes()[run] == b' ')
    })
}

/// Text before a code block that shows the model chatted: when there is no
/// fence at all, an answer opening like this is rejected.
fn looks_like_prose(first_line: &str) -> bool {
    let l = first_line.trim().to_ascii_lowercase();
    const OPENERS: [&str; 10] = [
        "here is",
        "here's",
        "here are",
        "sure",
        "certainly",
        "the resolved",
        "i'm sorry",
        "i am sorry",
        "i cannot",
        "i can't",
    ];
    OPENERS.iter().any(|o| l.starts_with(o)) || l.starts_with("sorry")
}

/// The first fenced block of `answer` (the body between the fence lines,
/// indentation kept), if there is one. `Some(None)` is an unterminated
/// fence.
fn fenced_block(answer: &str) -> Option<Option<String>> {
    let mut lines = answer.lines();
    let mut found_open = false;
    let mut body: Vec<&str> = Vec::new();
    for line in lines.by_ref() {
        if !found_open {
            if line.trim_start().starts_with("```") {
                found_open = true;
            }
            continue;
        }
        if line.trim() == "```" {
            return Some(Some(body.join("\n")));
        }
        body.push(line);
    }
    found_open.then_some(None)
}

/// The resolved lines in the model's answer. Takes the first code block
/// when there is one (indentation kept), otherwise the whole answer;
/// rejects an empty answer, conflict markers left in, an unterminated
/// fence, a chatty answer with no code block, and one over
/// [`MAX_ANSWER_BYTES`]. An explicitly empty code block is a valid, empty
/// resolution.
pub fn conflict_resolution(answer: &str) -> Result<Vec<String>> {
    if answer.trim().is_empty() {
        return Err(Error::Empty);
    }
    if answer.len() > MAX_ANSWER_BYTES {
        return Err(Error::Invalid(
            "the AI answer is too long to be one conflict's resolution".into(),
        ));
    }
    let text = match fenced_block(answer) {
        Some(Some(body)) => body,
        Some(None) => {
            return Err(Error::Decode(
                "the answer's code block was never closed".into(),
            ))
        }
        None => {
            let first = answer.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
            if looks_like_prose(first) {
                return Err(Error::Decode(
                    "the answer is an explanation, not the resolved text".into(),
                ));
            }
            answer.trim_matches(['\r', '\n']).to_string()
        }
    };
    let lines: Vec<String> = text
        .split('\n')
        .map(|l| l.trim_end_matches('\r').to_string())
        .collect();
    // `"a\nb\n"` splits into a trailing empty piece that is not a line.
    let mut lines = lines;
    if lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    if lines.iter().any(|l| is_marker(l)) {
        return Err(Error::Decode(
            "the answer still contains conflict markers".into(),
        ));
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(str::to_string).collect()
    }

    fn ctx() -> ConflictContext {
        ConflictContext {
            path: "src/lib.rs".into(),
            index: 2,
            total: 3,
            base: lines("let x = 1;"),
            has_base: true,
            ours: lines("let x = 2;"),
            theirs: lines("let x = 3;"),
            ours_label: "Current · main".into(),
            theirs_label: "Incoming · feature".into(),
            before: (1..=30).map(|i| format!("before {i}")).collect(),
            after: (1..=30).map(|i| format!("after {i}")).collect(),
        }
    }

    #[test]
    fn the_prompt_names_the_file_language_labels_and_all_three_blocks() {
        let p = conflict_resolve(&ctx()).unwrap();
        assert!(p.contains("conflict 2 of 3 in `src/lib.rs` (Rust)"), "{p}");
        assert!(p.contains("Side A: Current · main"), "{p}");
        assert!(p.contains("Side B: Incoming · feature"), "{p}");
        assert!(p.contains("let x = 1;") && p.contains("let x = 2;") && p.contains("let x = 3;"));
        assert!(p.contains("```rust\n"), "{p}");
    }

    #[test]
    fn the_context_is_capped_to_twenty_lines_nearest_the_conflict() {
        let p = conflict_resolve(&ctx()).unwrap();
        assert!(p.contains("before 30") && p.contains("before 11"), "{p}");
        assert!(!p.contains("before 10\n"), "{p}");
        assert!(p.contains("after 1\n") && p.contains("after 20\n"), "{p}");
        assert!(!p.contains("after 21"), "{p}");
    }

    #[test]
    fn an_add_add_conflict_says_there_is_no_base() {
        let mut c = ctx();
        c.has_base = false;
        c.base.clear();
        let p = conflict_resolve(&c).unwrap();
        assert!(p.contains("none: both sides added this"), "{p}");
    }

    #[test]
    fn a_huge_conflict_is_refused_with_a_sentence() {
        let mut c = ctx();
        c.ours = vec!["x".repeat(SIDE_BUDGET + 1)];
        let err = conflict_resolve(&c).unwrap_err().to_string();
        assert!(err.contains("too large"), "{err}");
    }

    #[test]
    fn unknown_extensions_have_no_language() {
        assert_eq!(language_of("a/b.rs"), Some("Rust"));
        assert_eq!(language_of("a/b.weird"), None);
        assert_eq!(language_of("Makefile"), None);
    }

    #[test]
    fn the_first_code_block_is_the_answer_and_keeps_its_indentation() {
        let a = "Here you go:\n```rust\n    let x = 5;\n\n    y();\n```\nHope it helps.\n```\nignored\n```";
        assert_eq!(
            conflict_resolution(a).unwrap(),
            ["    let x = 5;", "", "    y();"]
        );
    }

    #[test]
    fn a_bare_answer_is_taken_whole() {
        assert_eq!(conflict_resolution("a\nb\n").unwrap(), ["a", "b"]);
        assert_eq!(conflict_resolution("a\r\nb\r\n").unwrap(), ["a", "b"]);
    }

    #[test]
    fn an_empty_code_block_is_an_empty_resolution() {
        assert_eq!(
            conflict_resolution("```\n```").unwrap(),
            Vec::<String>::new()
        );
    }

    #[test]
    fn junk_is_rejected() {
        assert_eq!(conflict_resolution("  \n"), Err(Error::Empty));
        for bad in [
            "```rust\n<<<<<<< HEAD\nx\n=======\ny\n>>>>>>> b\n```",
            "Here is the resolution:\nlet x = 1;",
            "I'm sorry, I cannot help with that.",
            "```rust\nlet x = 1;",
        ] {
            assert!(conflict_resolution(bad).is_err(), "{bad}");
        }
        let huge = "x\n".repeat(MAX_ANSWER_BYTES);
        assert!(conflict_resolution(&huge).is_err());
    }

    #[test]
    fn a_line_of_equals_in_code_is_not_a_marker_unless_it_is_seven() {
        assert!(conflict_resolution("a\n=====\nb").is_ok());
        assert!(conflict_resolution("a\n=======\nb").is_err());
    }
}
