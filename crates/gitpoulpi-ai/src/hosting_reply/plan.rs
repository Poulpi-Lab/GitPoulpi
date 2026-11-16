//! What of a conversation fits the request: the byte budget of
//! `ai.limits.hosting_reply_context_bytes` split between the item's
//! description, the thread being answered and the recent entries.
//!
//! Pure arithmetic over sizes, so it is unit tested without a conversation.
//! The rules, in priority order:
//!
//! 1. the thread being answered is always there, up to half the budget (its
//!    start is left out, the newest comments stay);
//! 2. the item's description gets up to a quarter;
//! 3. what remains goes to the other entries, **newest first**: an entry that
//!    does not fit is left out and the next older one is tried, so one huge
//!    log comment does not hide everything before it.
//!
//! Nothing here cuts silently: [`Plan`] says what was left out and the
//! caller turns it into [`crate::Cut`] entries.

/// How the budget is shared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Split {
    /// Bytes the description may use.
    pub description: usize,
    /// Bytes the thread being answered may use.
    pub thread: usize,
    /// Bytes left for the other entries.
    pub recent: usize,
}

/// Shares `budget` between a description of `description_len` bytes and a
/// thread of `thread_len` bytes; what they do not use flows to the recent
/// entries. `usize::MAX` (the caps are lifted) gives every part its whole size.
pub fn split_budget(budget: usize, description_len: usize, thread_len: usize) -> Split {
    if budget == usize::MAX {
        return Split {
            description: usize::MAX,
            thread: usize::MAX,
            recent: usize::MAX,
        };
    }
    let thread = thread_len.min(budget / 2);
    let description = description_len.min(budget / 4);
    Split {
        description,
        thread,
        recent: budget - thread - description,
    }
}

/// Which of the other entries are sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Indexes into the candidates, oldest first.
    pub kept: Vec<usize>,
    /// How many candidates were left out.
    pub omitted: usize,
    /// The newest entry alone is bigger than the room: it is sent cut to
    /// this many bytes (its end is left out) instead of not at all.
    pub newest_cap: Option<usize>,
}

/// Chooses among `sizes` (the rendered size of each candidate, oldest
/// first) what fits `room` bytes, newest first, skipping an entry that does
/// not fit. When the newest alone is bigger than `room`, it is the only one
/// kept, capped to `room`.
pub fn choose_recent(sizes: &[usize], room: usize) -> Plan {
    // The newest entry is what is being answered most often: it is never
    // dropped for being big, it is cut instead.
    if let Some(last) = sizes.len().checked_sub(1) {
        if sizes[last] > room && room > 0 {
            return Plan {
                kept: vec![last],
                omitted: last,
                newest_cap: Some(room),
            };
        }
    }
    let mut kept = Vec::new();
    let mut used = 0usize;
    for (index, size) in sizes.iter().enumerate().rev() {
        if used.saturating_add(*size) <= room {
            used += size;
            kept.push(index);
        }
    }
    kept.reverse();
    Plan {
        omitted: sizes.len() - kept.len(),
        kept,
        newest_cap: None,
    }
}

// -- what of the repository a reply needs ----------------------------------------

use std::collections::HashSet;

use super::repo_context::ContextLimits;
use super::{DiffRange, ReplyRequest, ReplyTarget};

/// A file the conversation points at, with the line when it said one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FileRef {
    pub path: String,
    pub line: Option<u32>,
}

/// What to read of the repository for one reply, chosen from the text of
/// the conversation. Pure: [`plan_context`] sees the request and the list of
/// paths and nothing else.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ContextSelection {
    /// Files named in the conversation that exist in the tree.
    pub files: Vec<FileRef>,
    /// Identifiers the conversation puts in backticks.
    pub symbols: Vec<String>,
    /// Quoted error messages and the rare words of the title.
    pub terms: Vec<String>,
    /// The pull request's range, when the item is one.
    pub diff: Option<DiffRange>,
    /// Include the repository map.
    pub repo_map: bool,
}

impl ContextSelection {
    /// A stable hash of what is selected (part of the cache key).
    pub fn hash(&self) -> u64 {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash_into(&mut hasher);
        hasher.finish()
    }

    fn hash_into(&self, state: &mut impl std::hash::Hasher) {
        use std::hash::Hash;
        self.files.hash(state);
        self.symbols.hash(state);
        self.terms.hash(state);
        self.diff.hash(state);
        self.repo_map.hash(state);
    }

    /// Every string to search for: the symbols, then the terms.
    pub fn search_terms(&self) -> Vec<String> {
        self.symbols.iter().chain(&self.terms).cloned().collect()
    }
}

/// Most files read for one reply.
const MAX_FILES: usize = 6;
/// A path-looking token that matches this many files by name is too vague.
const MAX_SUFFIX_MATCHES: usize = 3;

const STOP_WORDS: &[&str] = &[
    "about", "after", "again", "being", "could", "doesn", "error", "fails", "failed", "from",
    "going", "have", "issue", "might", "other", "please", "should", "still", "their", "there",
    "these", "thing", "those", "under", "using", "when", "where", "which", "while", "would",
    "wrong", "problem", "support", "request", "question", "update", "feature", "broken",
];

/// Chooses what of the repository to read for `request`, given the paths of
/// its tree (`tree`): the files the text names (`src/a.rs`, `a.rs:12`,
/// `x.rs#L12`, stack frames), identifiers in backticks, quoted error
/// messages, a few rare words of the title, and the pull request's range.
/// Paths that are not in the tree are dropped; the number of search terms is
/// held to `limits.terms`.
pub fn plan_context(
    request: &ReplyRequest,
    tree: &[String],
    limits: &ContextLimits,
) -> ContextSelection {
    let (title, mut texts) = request_texts(request);
    let by_path: HashSet<&str> = tree.iter().map(String::as_str).collect();

    let mut files: Vec<FileRef> = Vec::new();
    let mut symbols: Vec<String> = Vec::new();
    let mut terms: Vec<String> = Vec::new();
    let push_file = |f: FileRef, files: &mut Vec<FileRef>| {
        if files.len() < MAX_FILES && !files.iter().any(|g| g.path == f.path) {
            files.push(f);
        }
    };
    texts.push(title.clone());
    for text in &texts {
        for token in split_tokens(text) {
            if let Some((path, line)) = path_of(token) {
                for found in resolve_path(&path, tree, &by_path) {
                    push_file(FileRef { path: found, line }, &mut files);
                }
            }
        }
        for span in spans(text, '`') {
            if let Some(symbol) = identifier(span) {
                let is_path = path_of(span)
                    .is_some_and(|(p, _)| !resolve_path(&p, tree, &by_path).is_empty());
                if !is_path && !symbols.contains(&symbol) {
                    symbols.push(symbol);
                }
            }
        }
        for span in spans(text, '"') {
            let span = span.trim();
            if (12..=120).contains(&span.len())
                && span.contains(' ')
                && !span.contains('\n')
                && !terms.iter().any(|t| t == span)
            {
                terms.push(span.to_string());
            }
        }
    }
    // The rare words of the title: the longest ones are the rarest.
    let mut words: Vec<&str> = title
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.chars().count() >= 5 && !STOP_WORDS.contains(&w.to_lowercase().as_str()))
        .collect();
    words.sort_by_key(|w| std::cmp::Reverse(w.chars().count()));
    for word in words.into_iter().take(5) {
        if !terms.iter().any(|t| t == word) && !symbols.iter().any(|s| s == word) {
            terms.push(word.to_string());
        }
    }

    // Held to the number of terms the user allows, symbols first.
    let room = limits.terms.max(1);
    symbols.truncate(room);
    terms.truncate(room.saturating_sub(symbols.len()));
    ContextSelection {
        files,
        symbols,
        terms,
        diff: request.diff.clone(),
        repo_map: true,
    }
}

/// The title and every text of the conversation the planner reads.
fn request_texts(request: &ReplyRequest) -> (String, Vec<String>) {
    let mut title = String::new();
    let mut texts: Vec<String> = Vec::new();
    if let ReplyTarget::IssueBody { title: t } = &request.target {
        title = t.clone();
    }
    if let ReplyTarget::LineComment { path, line, .. } = &request.target {
        texts.push(match line {
            Some(line) => format!("{path}:{line}"),
            None => path.clone(),
        });
    }
    if let Some(item) = &request.item {
        // `Issue #12 "Title" (open) by ...`: the title is the quoted part.
        if title.is_empty() {
            title = item
                .summary
                .split_once('"')
                .and_then(|(_, rest)| rest.rsplit_once('"'))
                .map(|(t, _)| t.to_string())
                .unwrap_or_default();
        }
        texts.push(item.description.clone());
    }
    texts.extend(request.entries.iter().map(|e| e.text.clone()));
    // Within an exchange the selection stays the one of the first request, so
    // the context (and the provider cache over it) is the same for every
    // follow-up.
    let (hint, draft) = match request.exchange.first() {
        Some(first) => (first.hint.as_str(), first.draft.as_deref()),
        None => (request.hint.as_str(), request.draft.as_deref()),
    };
    texts.push(hint.to_string());
    if let Some(draft) = draft {
        texts.push(draft.to_string());
    }
    (title, texts)
}

/// The text split where a path cannot continue (spaces, quotes, brackets).
fn split_tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| {
        c.is_whitespace()
            || matches!(
                c,
                '(' | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '<'
                    | '>'
                    | '"'
                    | '\''
                    | '`'
                    | ','
                    | ';'
                    | '|'
                    | '*'
                    | '='
            )
    })
    .filter(|t| !t.is_empty())
}

/// What is between pairs of `quote` on one line.
fn spans(text: &str, quote: char) -> impl Iterator<Item = &str> {
    text.lines().flat_map(move |line| {
        let mut parts = line.split(quote);
        parts.next();
        parts
            .enumerate()
            .filter(|(i, _)| i % 2 == 0)
            .map(|(_, s)| s)
            .collect::<Vec<_>>()
            .into_iter()
    })
}

/// `src/a.rs`, `a.rs:12`, `a.rs:12:5`, `a.rs#L12`, `a.rs#L12-L20`, `./a.rs`,
/// `a/src/b.rs` (a diff header): the path and the line, when the token looks
/// like one.
fn path_of(token: &str) -> Option<(String, Option<u32>)> {
    let token = token.trim_matches(|c: char| matches!(c, '.' | ',' | ':' | ';' | '!' | '?'));
    if token.contains("://") || token.len() < 3 {
        return None;
    }
    let (head, line) = if let Some((head, anchor)) = token.split_once("#L") {
        let digits: String = anchor.chars().take_while(char::is_ascii_digit).collect();
        (head, digits.parse::<u32>().ok())
    } else {
        let mut parts = token.split(':');
        let head = parts.next()?;
        let line = parts.next().and_then(|n| n.parse::<u32>().ok());
        (head, line)
    };
    let head = head.trim_start_matches("./");
    if !(head.contains('/') || head.contains('.')) || head.ends_with('/') || head.starts_with('.') {
        return None;
    }
    // A name needs an extension or a directory to be a path and not a word
    // ending a sentence ("done.").
    let name = head.rsplit('/').next().unwrap_or(head);
    if !head.contains('/')
        && !name.rsplit_once('.').is_some_and(|(stem, ext)| {
            !stem.is_empty()
                && !ext.is_empty()
                && ext.len() <= 8
                && ext.chars().all(char::is_alphanumeric)
        })
    {
        return None;
    }
    Some((head.to_string(), line))
}

/// The tree paths `path` stands for: itself, or without a diff's `a/` / `b/`
/// prefix, or the few files that end with it.
fn resolve_path(path: &str, tree: &[String], by_path: &HashSet<&str>) -> Vec<String> {
    if by_path.contains(path) {
        return vec![path.to_string()];
    }
    for prefix in ["a/", "b/"] {
        if let Some(rest) = path.strip_prefix(prefix) {
            if by_path.contains(rest) {
                return vec![rest.to_string()];
            }
        }
    }
    let suffix = format!("/{path}");
    let found: Vec<String> = tree
        .iter()
        .filter(|p| p.ends_with(&suffix))
        .take(MAX_SUFFIX_MATCHES + 1)
        .cloned()
        .collect();
    if found.len() <= MAX_SUFFIX_MATCHES {
        found
    } else {
        Vec::new()
    }
}

/// An identifier worth searching for: `Foo::bar`, `parse_date`, `render()`.
fn identifier(span: &str) -> Option<String> {
    let span = span.trim().trim_end_matches("()");
    let ok = (3..=64).contains(&span.len())
        && span
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        && span
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | ':' | '.' | '-' | '$'));
    ok.then(|| span.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_budget_is_shared_thread_first_then_description_then_the_rest() {
        let s = split_budget(1000, 5000, 5000);
        assert_eq!(s.thread, 500, "the thread gets up to half");
        assert_eq!(s.description, 250, "the description up to a quarter");
        assert_eq!(s.recent, 250);
        assert_eq!(s.thread + s.description + s.recent, 1000);
    }

    #[test]
    fn what_a_part_does_not_use_flows_to_the_recent_entries() {
        let s = split_budget(1000, 40, 0);
        assert_eq!((s.description, s.thread, s.recent), (40, 0, 960));
    }

    #[test]
    fn lifted_caps_give_every_part_its_whole_size() {
        let s = split_budget(usize::MAX, 9, 9);
        assert_eq!(s.description, usize::MAX);
        assert_eq!(s.recent, usize::MAX);
    }

    #[test]
    fn the_newest_entries_are_kept_first() {
        let p = choose_recent(&[10, 10, 10, 10], 25);
        assert_eq!(p.kept, [2, 3], "the two newest fit");
        assert_eq!(p.omitted, 2);
        assert_eq!(p.newest_cap, None);
    }

    #[test]
    fn a_huge_entry_is_skipped_and_older_ones_still_fit() {
        let p = choose_recent(&[10, 10_000, 10], 25);
        assert_eq!(p.kept, [0, 2]);
        assert_eq!(p.omitted, 1);
    }

    #[test]
    fn everything_fits_when_there_is_room() {
        let p = choose_recent(&[10, 20, 30], 60);
        assert_eq!(p.kept, [0, 1, 2]);
        assert_eq!(p.omitted, 0);
    }

    #[test]
    fn a_newest_entry_bigger_than_the_room_is_kept_cut_not_dropped() {
        let p = choose_recent(&[5, 900], 100);
        assert_eq!(p.kept, [1], "the newest wins over an older small entry");
        assert_eq!(p.newest_cap, Some(100));
        assert_eq!(p.omitted, 1);
        let p = choose_recent(&[900], 100);
        assert_eq!(p.kept, [0]);
        assert_eq!(p.newest_cap, Some(100));
        assert_eq!(p.omitted, 0);
    }

    #[test]
    fn nothing_to_choose_is_nothing_kept() {
        let p = choose_recent(&[], 100);
        assert_eq!((p.kept.len(), p.omitted, p.newest_cap), (0, 0, None));
        let p = choose_recent(&[10], 0);
        assert_eq!((p.kept.len(), p.omitted), (0, 1));
    }
}
