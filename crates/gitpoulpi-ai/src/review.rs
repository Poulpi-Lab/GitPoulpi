//! Review suggestions: what the model's answer to the "code review" prompt
//! (`prompts::code_review`) becomes.
//!
//! The parser is deliberately tolerant. Models wrap JSON in prose or code
//! fences, name keys differently, send line numbers as strings, or ignore
//! the format and write a list. Whatever can be read is kept; a suggestion
//! whose line is not in the diff is **kept as a file-level suggestion**
//! (`anchored == false`) with the line the model named, never dropped. Text
//! that holds no suggestion at all becomes the summary, so an answer is
//! never lost either.

use std::collections::{HashMap, HashSet};

use gitpoulpi_core::domain::{FileDiff, LineOrigin};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Which column of the diff a line number is in (the hosting crate's
/// `DiffSide`, which this crate cannot name without its `hosting` feature).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    /// The file before the change: removed and context lines.
    Old,
    /// The file after the change: added and context lines.
    #[default]
    New,
}

impl Side {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Old => "old",
            Self::New => "new",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "old" | "left" | "base" | "before" => Some(Self::Old),
            "new" | "right" | "head" | "after" => Some(Self::New),
            _ => None,
        }
    }
}

/// How much the model thinks a remark matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    #[default]
    Info,
    Warning,
    Issue,
}

impl Severity {
    pub fn key(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Issue => "issue",
        }
    }

    /// Reads the usual spellings (`error`, `blocker`, `nit`, ...); unknown
    /// words are `Info`.
    pub fn parse(text: &str) -> Self {
        match text.trim().to_ascii_lowercase().as_str() {
            "issue" | "error" | "critical" | "blocker" | "blocking" | "high" | "bug" | "major" => {
                Self::Issue
            }
            "warning" | "warn" | "medium" | "moderate" | "minor" | "risk" => Self::Warning,
            _ => Self::Info,
        }
    }
}

/// A line a comment can be attached to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    pub side: Side,
    pub line: u32,
    /// For a context line, its number on the other side (GitLab needs it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other_line: Option<u32>,
}

/// One suggested review comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Suggestion {
    pub path: String,
    /// The line the model named (on `side`); `0` when it named none.
    pub line: u32,
    pub side: Side,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    pub severity: Severity,
    pub body: String,
    /// `true`: `line` is a line of the diff. `false`: the comment is about
    /// the file (or a line the diff does not show) and is listed as such.
    pub anchored: bool,
    /// For an anchored context line, its number on the other side.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other_line: Option<u32>,
    /// Where an unanchored suggestion goes when accepted: the file's first
    /// changed line. `None` when the diff does not contain the file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Anchor>,
}

impl Suggestion {
    /// The line an accepted suggestion becomes a comment on, with the text
    /// to send. A file-level suggestion lands on the file's first changed
    /// line and says which line it was about. `None`: the file is not in
    /// the diff, so there is nowhere to put it.
    pub fn as_comment(&self, text: &str) -> Option<(Anchor, Option<u32>, String)> {
        if self.anchored {
            let anchor = Anchor {
                side: self.side,
                line: self.line,
                other_line: self.other_line,
            };
            return Some((anchor, self.start_line, text.to_string()));
        }
        let anchor = self.fallback?;
        let body = if self.line > 0 {
            format!("(about line {}) {text}", self.line)
        } else {
            text.to_string()
        };
        Some((anchor, None, body))
    }
}

/// The overall verdict the model recommends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftVerdict {
    Approve,
    RequestChanges,
    #[default]
    Comment,
}

impl DraftVerdict {
    pub fn key(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::RequestChanges => "request_changes",
            Self::Comment => "comment",
        }
    }

    /// Reads the usual spellings ("approved", "LGTM", "changes requested",
    /// "request-changes", "reject"); anything else is a plain comment.
    pub fn parse(text: &str) -> Self {
        let t = text.trim().to_ascii_lowercase().replace(['_', '-'], " ");
        if (t.contains("request") && t.contains("change"))
            || t.contains("changes requested")
            || t.contains("reject")
            || t.contains("needs work")
            || t.contains("block")
        {
            Self::RequestChanges
        } else if t.contains("approv") || t.contains("lgtm") || t == "accept" || t == "ok" {
            Self::Approve
        } else {
            Self::Comment
        }
    }
}

/// The parsed answer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewSuggestions {
    pub summary: String,
    /// The recommended verdict; `None` when the answer held no structure.
    #[serde(default)]
    pub verdict: Option<DraftVerdict>,
    /// The review text to submit; falls back to the summary.
    #[serde(default)]
    pub body: String,
    pub suggestions: Vec<Suggestion>,
}

/// Which lines the diff shows, to place suggestions.
#[derive(Debug, Clone, Default)]
pub struct DiffIndex {
    /// `(path, side, line)` to the number on the other side for a context line.
    lines: HashMap<(String, Side, u32), Option<u32>>,
    /// Paths in diff order.
    order: Vec<String>,
    first: HashMap<String, Anchor>,
}

impl DiffIndex {
    pub fn new(files: &[FileDiff]) -> Self {
        let mut ix = Self::default();
        for fd in files {
            let path = fd.path.to_string_lossy().replace('\\', "/");
            ix.order.push(path.clone());
            let (mut first_changed, mut first_any): (Option<Anchor>, Option<Anchor>) = (None, None);
            for hunk in &fd.hunks {
                let (mut old, mut new) = (hunk.old_start, hunk.new_start);
                for line in &hunk.lines {
                    let anchor = match line.origin {
                        LineOrigin::Context => {
                            ix.lines.insert((path.clone(), Side::Old, old), Some(new));
                            ix.lines.insert((path.clone(), Side::New, new), Some(old));
                            let a = Anchor {
                                side: Side::New,
                                line: new,
                                other_line: Some(old),
                            };
                            old += 1;
                            new += 1;
                            first_any.get_or_insert(a);
                            continue;
                        }
                        LineOrigin::Add => {
                            ix.lines.insert((path.clone(), Side::New, new), None);
                            new += 1;
                            Anchor {
                                side: Side::New,
                                line: new - 1,
                                other_line: None,
                            }
                        }
                        LineOrigin::Delete => {
                            ix.lines.insert((path.clone(), Side::Old, old), None);
                            old += 1;
                            Anchor {
                                side: Side::Old,
                                line: old - 1,
                                other_line: None,
                            }
                        }
                    };
                    first_changed.get_or_insert(anchor);
                    first_any.get_or_insert(anchor);
                }
            }
            if let Some(a) = first_changed.or(first_any) {
                ix.first.insert(path, a);
            }
        }
        ix
    }

    /// The paths of the diff, in order.
    pub fn paths(&self) -> &[String] {
        &self.order
    }

    /// `path` as the diff spells it: exact, or the one file it is a suffix
    /// (or prefix-trimmed form) of.
    fn resolve(&self, path: &str) -> Option<String> {
        let path = path.trim().trim_start_matches("./").replace('\\', "/");
        let path = path
            .strip_prefix("a/")
            .or_else(|| path.strip_prefix("b/"))
            .filter(|p| {
                self.order
                    .iter()
                    .any(|o| o == p || o.ends_with(&format!("/{p}")))
            })
            .map(str::to_string)
            .unwrap_or(path);
        if self.order.contains(&path) {
            return Some(path);
        }
        let mut hits = self
            .order
            .iter()
            .filter(|o| o.ends_with(&format!("/{path}")) || path.ends_with(&format!("/{o}")));
        let first = hits.next()?;
        hits.next().is_none().then(|| first.clone())
    }

    fn has(&self, path: &str, side: Side, line: u32) -> Option<Option<u32>> {
        self.lines.get(&(path.to_string(), side, line)).copied()
    }
}

// -- reading the answer ---------------------------------------------------------

fn text_of(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn field<'a>(obj: &'a serde_json::Map<String, Value>, names: &[&str]) -> Option<&'a Value> {
    names.iter().find_map(|n| obj.get(*n))
}

/// A line number given as 12, "12" or "12-14" / "12:14" (a range).
fn lines_of(v: &Value) -> Option<(Option<u32>, u32)> {
    let text = text_of(v)?;
    let text = text.trim().trim_start_matches('L');
    let nums: Vec<u32> = text
        .split(|c: char| !c.is_ascii_digit())
        .filter(|p| !p.is_empty())
        .filter_map(|p| p.parse().ok())
        .collect();
    match nums.as_slice() {
        [] => None,
        [one] => Some((None, *one)),
        [a, .., b] if a < b => Some((Some(*a), *b)),
        [a, .., _] => Some((None, *a)),
    }
}

struct Raw {
    path: String,
    line: u32,
    start: Option<u32>,
    side: Option<Side>,
    severity: Severity,
    body: String,
}

fn raw_from_object(obj: &serde_json::Map<String, Value>) -> Option<Raw> {
    let body = field(
        obj,
        &[
            "body",
            "comment",
            "message",
            "text",
            "suggestion",
            "description",
        ],
    )
    .and_then(text_of)?
    .trim()
    .to_string();
    if body.is_empty() {
        return None;
    }
    let path = field(obj, &["path", "file", "filename", "file_path"])
        .and_then(text_of)
        .unwrap_or_default();
    let (mut start, mut line) = field(obj, &["line", "end_line", "line_number", "lines"])
        .and_then(lines_of)
        .unwrap_or((None, 0));
    if let Some((_, first)) = field(obj, &["start_line", "from_line"]).and_then(lines_of) {
        if line > 0 && first < line {
            start = Some(first);
        } else if line == 0 {
            line = first;
        }
    }
    let side = field(obj, &["side"])
        .and_then(text_of)
        .and_then(|s| Side::parse(&s));
    let severity = field(obj, &["severity", "level", "priority", "type"])
        .and_then(text_of)
        .map(|s| Severity::parse(&s))
        .unwrap_or_default();
    Some(Raw {
        path,
        line,
        start,
        side,
        severity,
        body,
    })
}

/// The JSON value in `answer`: the text itself, a fenced block, or the
/// outermost `{...}` / `[...]` in the middle of prose.
fn json_in(answer: &str) -> Option<Value> {
    let text = answer.trim();
    if let Ok(v) = serde_json::from_str::<Value>(text) {
        return Some(v);
    }
    // Fenced blocks, any info string.
    let mut rest = text;
    while let Some(open) = rest.find("```") {
        let after = &rest[open + 3..];
        let body_start = after.find('\n').map_or(0, |n| n + 1);
        let Some(close) = after[body_start..].find("```") else {
            break;
        };
        let body = &after[body_start..body_start + close];
        if let Ok(v) = serde_json::from_str::<Value>(body.trim()) {
            return Some(v);
        }
        rest = &after[body_start + close + 3..];
    }
    for (open, close) in [('{', '}'), ('[', ']')] {
        if let (Some(a), Some(b)) = (text.find(open), text.rfind(close)) {
            if a < b {
                if let Ok(v) = serde_json::from_str::<Value>(&text[a..=b]) {
                    return Some(v);
                }
            }
        }
    }
    None
}

/// `path:12: text`, `- path:12-14 - text`, `1. path line 12: text`.
fn loose_item(line: &str) -> Option<Raw> {
    let line = line.trim().trim_start_matches(['-', '*', '+']).trim();
    let line = line
        .split_once(". ")
        .filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
        .map_or(line, |(_, r)| r)
        .trim();
    let line = line.trim_start_matches('`');
    let (head, body) = line.split_once(": ").or_else(|| line.split_once(" - "))?;
    let head = head.trim().trim_end_matches('`');
    let (path, numbers) = head
        .rsplit_once(':')
        .or_else(|| head.rsplit_once(" line "))
        .or_else(|| head.rsplit_once(" L"))?;
    if path.is_empty() || path.contains(' ') || !path.contains(['.', '/']) {
        return None;
    }
    let (start, line) = lines_of(&Value::String(numbers.to_string()))?;
    let body = body.trim();
    (!body.is_empty()).then(|| Raw {
        path: path.trim().to_string(),
        line,
        start,
        side: None,
        severity: Severity::parse(
            body.split(|c: char| !c.is_alphabetic())
                .next()
                .unwrap_or_default(),
        ),
        body: body.to_string(),
    })
}

/// Turns the model's answer into suggestions placed on `diff`.
///
/// Never fails: an answer that holds no suggestion is returned as the
/// summary alone, so the reviewer still sees what the model said.
pub fn parse_suggestions(answer: &str, diff: &DiffIndex) -> ReviewSuggestions {
    let mut summary = String::new();
    let mut body = String::new();
    let mut verdict = None;
    let mut raws: Vec<Raw> = Vec::new();
    match json_in(answer) {
        Some(Value::Object(obj)) => {
            summary = field(&obj, &["summary", "overview", "overall"])
                .and_then(text_of)
                .unwrap_or_default();
            let has_place = field(&obj, &["path", "file", "filename", "file_path"]).is_some();
            if !has_place {
                body = field(&obj, &["body", "review_text", "text"])
                    .and_then(text_of)
                    .unwrap_or_default();
            }
            verdict = Some(
                field(
                    &obj,
                    &["verdict", "decision", "recommendation", "event", "state"],
                )
                .and_then(text_of)
                .map(|v| DraftVerdict::parse(&v))
                .unwrap_or_default(),
            );
            if let Some(Value::Array(items)) = field(
                &obj,
                &["suggestions", "comments", "findings", "issues", "review"],
            ) {
                raws.extend(
                    items
                        .iter()
                        .filter_map(|i| i.as_object())
                        .filter_map(raw_from_object),
                );
            } else if has_place {
                if let Some(one) = raw_from_object(&obj) {
                    raws.push(one);
                }
            }
        }
        Some(Value::Array(items)) => {
            raws.extend(
                items
                    .iter()
                    .filter_map(|i| i.as_object())
                    .filter_map(raw_from_object),
            );
        }
        _ => {
            let mut prose = Vec::new();
            for line in answer.lines() {
                match loose_item(line) {
                    Some(raw) => raws.push(raw),
                    None => prose.push(line),
                }
            }
            summary = prose.join("\n").trim().to_string();
        }
    }

    let mut seen: HashSet<(String, u32, String)> = HashSet::new();
    let mut out: Vec<Suggestion> = Vec::new();
    for raw in raws {
        let resolved = diff.resolve(&raw.path);
        let path = resolved
            .clone()
            .unwrap_or_else(|| raw.path.trim().to_string());
        if !seen.insert((path.clone(), raw.line, raw.body.clone())) {
            continue;
        }
        let placed = resolved.as_ref().filter(|_| raw.line > 0).and_then(|p| {
            let preferred = raw.side.unwrap_or(Side::New);
            let other = match preferred {
                Side::New => Side::Old,
                Side::Old => Side::New,
            };
            [preferred, other]
                .into_iter()
                .find_map(|side| diff.has(p, side, raw.line).map(|o| (side, o)))
        });
        let fallback = resolved.as_ref().and_then(|p| diff.first.get(p).copied());
        match placed {
            Some((side, other_line)) => {
                let start_line = raw
                    .start
                    .filter(|s| *s < raw.line && diff.has(&path, side, *s).is_some());
                out.push(Suggestion {
                    path,
                    line: raw.line,
                    side,
                    start_line,
                    severity: raw.severity,
                    body: raw.body,
                    anchored: true,
                    other_line,
                    fallback,
                });
            }
            None => out.push(Suggestion {
                path,
                line: raw.line,
                side: raw.side.unwrap_or_default(),
                start_line: None,
                severity: raw.severity,
                body: raw.body,
                anchored: false,
                other_line: None,
                fallback,
            }),
        }
    }
    // Diff order, then line; the unplaced of a file after its placed ones.
    let rank = |p: &str| diff.order.iter().position(|o| o == p).unwrap_or(usize::MAX);
    out.sort_by_key(|s| (rank(&s.path), !s.anchored, s.line));
    let summary = summary.trim().to_string();
    let body = body.trim().to_string();
    ReviewSuggestions {
        body: if body.is_empty() {
            summary.clone()
        } else {
            body
        },
        summary,
        verdict,
        suggestions: out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{DiffLine, FileStatus, Hunk, HunkId};

    fn line(origin: LineOrigin, text: &str) -> DiffLine {
        DiffLine {
            origin,
            content: format!("{text}\n"),
            staged: false,
        }
    }

    /// `src/a.rs`: 10 keep (10/10), - 11 gone, + 11 added one, + 12 added two,
    /// 12/13 tail.
    fn index() -> DiffIndex {
        let a = FileDiff {
            renamed_from: None,
            similarity: None,
            path: "src/a.rs".into(),
            status: FileStatus::Modified,
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 10,
                old_lines: 3,
                new_start: 10,
                new_lines: 4,
                lines: vec![
                    line(LineOrigin::Context, "keep"),
                    line(LineOrigin::Delete, "gone"),
                    line(LineOrigin::Add, "added one"),
                    line(LineOrigin::Add, "added two"),
                    line(LineOrigin::Context, "tail"),
                ],
            }],
        };
        let b = FileDiff {
            renamed_from: None,
            similarity: None,
            path: "docs/b.md".into(),
            status: FileStatus::Added,
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 0,
                old_lines: 0,
                new_start: 1,
                new_lines: 1,
                lines: vec![line(LineOrigin::Add, "# B")],
            }],
        };
        DiffIndex::new(&[a, b])
    }

    #[test]
    fn plain_json_is_read_and_placed_on_the_diff() {
        let answer = r#"{"summary": "Adds a column.", "suggestions": [
            {"path": "src/a.rs", "line": 12, "side": "new", "severity": "warning", "body": "Why a Vec?"},
            {"path": "src/a.rs", "line": 11, "side": "old", "severity": "issue", "body": "Dropped a check."}]}"#;
        let got = parse_suggestions(answer, &index());
        assert_eq!(got.summary, "Adds a column.");
        assert_eq!(got.suggestions.len(), 2);
        // Sorted by line: the old-side 11 first.
        let s = &got.suggestions[0];
        assert_eq!(
            (s.line, s.side, s.severity, s.anchored),
            (11, Side::Old, Severity::Issue, true)
        );
        let s = &got.suggestions[1];
        assert_eq!(
            (s.line, s.side, s.severity, s.anchored),
            (12, Side::New, Severity::Warning, true)
        );
    }

    #[test]
    fn fenced_json_inside_prose_is_found() {
        let answer = "Sure! Here is my review:\n\n```json\n{\"summary\":\"ok\",\"suggestions\":[{\"path\":\"docs/b.md\",\"line\":1,\"body\":\"Add a title line.\"}]}\n```\nHope it helps.";
        let got = parse_suggestions(answer, &index());
        assert_eq!(got.summary, "ok");
        assert_eq!(got.suggestions.len(), 1);
        assert!(got.suggestions[0].anchored);
        assert_eq!(
            got.suggestions[0].severity,
            Severity::Info,
            "no severity means info"
        );
    }

    #[test]
    fn json_in_the_middle_of_text_and_odd_key_names_are_tolerated() {
        let answer = r#"My findings: {"overview": "meh", "findings": [{"file": "./src/a.rs", "line": "12", "level": "error", "message": "Off by one."}]} Done."#;
        let got = parse_suggestions(answer, &index());
        assert_eq!(got.summary, "meh");
        let s = &got.suggestions[0];
        assert_eq!(
            (s.path.as_str(), s.line, s.severity),
            ("src/a.rs", 12, Severity::Issue)
        );
    }

    #[test]
    fn a_bare_array_and_a_range_work() {
        let answer = r#"[{"path": "src/a.rs", "line": "11-12", "body": "Extract this."}]"#;
        let got = parse_suggestions(answer, &index());
        let s = &got.suggestions[0];
        assert_eq!((s.start_line, s.line, s.anchored), (Some(11), 12, true));
        assert_eq!(got.summary, "");
    }

    #[test]
    fn a_line_not_in_the_diff_is_listed_as_file_level_never_dropped() {
        let answer = r#"{"suggestions": [
            {"path": "src/a.rs", "line": 400, "body": "Far away."},
            {"path": "src/a.rs", "body": "No line at all."},
            {"path": "unknown/file.rs", "line": 3, "body": "Not in the diff."}]}"#;
        let got = parse_suggestions(answer, &index());
        assert_eq!(got.suggestions.len(), 3);
        assert!(got.suggestions.iter().all(|s| !s.anchored));
        let far = got.suggestions.iter().find(|s| s.line == 400).unwrap();
        // The first changed line of the file is where it would go.
        assert_eq!(
            far.fallback.map(|a| (a.side, a.line)),
            Some((Side::Old, 11))
        );
        let (anchor, start, body) = far.as_comment("Far away.").unwrap();
        assert_eq!((anchor.line, start), (11, None));
        assert!(body.starts_with("(about line 400) "), "{body}");
        let ghost = got
            .suggestions
            .iter()
            .find(|s| s.path == "unknown/file.rs")
            .unwrap();
        assert!(ghost.fallback.is_none() && ghost.as_comment("x").is_none());
        let noline = got.suggestions.iter().find(|s| s.line == 0).unwrap();
        assert_eq!(
            noline.as_comment("No line at all.").unwrap().2,
            "No line at all."
        );
    }

    #[test]
    fn a_wrong_side_is_corrected_when_the_line_exists_on_the_other_one() {
        // Line 11 exists on both sides; line 12 only as an added line.
        let answer =
            r#"{"suggestions": [{"path": "src/a.rs", "line": 12, "side": "old", "body": "x"}]}"#;
        let got = parse_suggestions(answer, &index());
        let s = &got.suggestions[0];
        assert!(s.anchored);
        assert_eq!(
            (s.side, s.line),
            (Side::Old, 12),
            "old 12 is the context tail"
        );
        let answer = r#"{"suggestions": [{"path": "src/a.rs", "line": 11, "side": "new", "body": "added one"}]}"#;
        assert_eq!(
            parse_suggestions(answer, &index()).suggestions[0].side,
            Side::New
        );
    }

    #[test]
    fn a_context_line_carries_its_number_on_the_other_side() {
        let answer =
            r#"{"suggestions": [{"path": "src/a.rs", "line": 13, "side": "new", "body": "tail"}]}"#;
        let s = &parse_suggestions(answer, &index()).suggestions[0];
        assert!(s.anchored);
        assert_eq!(s.other_line, Some(12));
    }

    #[test]
    fn loose_text_lists_are_read_and_the_rest_is_the_summary() {
        let answer = "Looks mostly fine.\n\n- src/a.rs:12: warning - the Vec grows without bound\n2. docs/b.md line 1: add a title\nThat is all.";
        let got = parse_suggestions(answer, &index());
        assert_eq!(got.suggestions.len(), 2, "{got:?}");
        assert_eq!(got.suggestions[0].path, "src/a.rs");
        assert_eq!(got.suggestions[0].severity, Severity::Warning);
        assert!(got.summary.contains("Looks mostly fine.") && got.summary.contains("That is all."));
    }

    #[test]
    fn text_without_any_suggestion_is_the_summary() {
        let got = parse_suggestions("Nothing to report, the change is small and safe.", &index());
        assert!(got.suggestions.is_empty());
        assert_eq!(
            got.summary,
            "Nothing to report, the change is small and safe."
        );
        assert_eq!(
            parse_suggestions("", &index()),
            ReviewSuggestions::default()
        );
    }

    #[test]
    fn duplicates_collapse_and_a_suffix_path_resolves() {
        let answer = r#"{"suggestions": [
            {"path": "a.rs", "line": 12, "body": "same"},
            {"path": "src/a.rs", "line": 12, "body": "same"}]}"#;
        let got = parse_suggestions(answer, &index());
        assert_eq!(got.suggestions.len(), 1);
        assert_eq!(got.suggestions[0].path, "src/a.rs");
    }

    #[test]
    fn severities_are_read_from_the_usual_words() {
        for (word, want) in [
            ("issue", Severity::Issue),
            ("Critical", Severity::Issue),
            ("warning", Severity::Warning),
            ("warn", Severity::Warning),
            ("nit", Severity::Info),
            ("", Severity::Info),
        ] {
            assert_eq!(Severity::parse(word), want, "{word}");
        }
    }

    #[test]
    fn the_verdict_and_body_are_read_leniently() {
        for (word, want) in [
            ("approve", DraftVerdict::Approve),
            ("Approved", DraftVerdict::Approve),
            ("LGTM", DraftVerdict::Approve),
            ("request_changes", DraftVerdict::RequestChanges),
            ("Changes requested", DraftVerdict::RequestChanges),
            ("request-changes", DraftVerdict::RequestChanges),
            ("comment", DraftVerdict::Comment),
            ("hmm", DraftVerdict::Comment),
        ] {
            assert_eq!(DraftVerdict::parse(word), want, "{word}");
        }
        let got = parse_suggestions(
            r#"{"verdict": "changes requested", "body": "Fix the loop.", "summary": "s",
                "suggestions": [{"path": "src/a.rs", "line": 11, "body": "x"}]}"#,
            &index(),
        );
        assert_eq!(got.verdict, Some(DraftVerdict::RequestChanges));
        assert_eq!(got.body, "Fix the loop.");
        assert_eq!(got.suggestions.len(), 1);
    }

    #[test]
    fn a_missing_verdict_is_a_comment_and_a_missing_body_is_the_summary() {
        let got = parse_suggestions(r#"{"summary": "Looks fine.", "suggestions": []}"#, &index());
        assert_eq!(got.verdict, Some(DraftVerdict::Comment));
        assert_eq!(got.body, "Looks fine.");
        assert!(got.suggestions.is_empty(), "a body is not a suggestion");
        let prose = parse_suggestions("Just some words.", &index());
        assert_eq!(prose.verdict, None);
        assert_eq!(prose.body, "Just some words.");
    }
}
