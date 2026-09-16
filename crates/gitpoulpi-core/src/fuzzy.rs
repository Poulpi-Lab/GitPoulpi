//! The one fuzzy scorer: the command palette and the workspace search rank
//! with it, so a query means the same thing everywhere.
//!
//! Three tiers, so a short candidate cannot be buried by a long one that
//! merely happens to contain the same letters:
//!
//! 1. the query appears verbatim *after the category prefix* (`Category: `,
//!    the palette's labels; with [`Anchor::Whole`] the whole text is the body,
//!    so tier 1 is "verbatim anywhere") - what you almost always mean;
//! 2. it appears verbatim anywhere (so typing `git` gathers the group);
//! 3. its characters appear in order but apart (a genuine fuzzy match).
//!
//! Within a tier an earlier position and a shorter text win. Pure: no I/O,
//! no allocation on the scoring path (a search runs it over every commit
//! subject of a repository per keystroke).
//!
//! Matching is on lower-cased text: callers lower-case the candidate once
//! (the palette and the search index cache it) and the query once.

use serde::{Deserialize, Serialize};

/// Where tier 1 starts looking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// After a leading `Category:` (the palette's labels; a translation may
    /// put a no-break space before the colon and an ordinary one after, CJK
    /// catalogs write the full-width colon).
    AfterCategory,
    /// The whole text is the body (refs, commit subjects: a colon in
    /// `feat: x` is not a category).
    Whole,
}

/// A match with the characters to highlight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Match {
    pub score: i32,
    /// Indices (in characters) into the lower-cased text, ascending. For
    /// ASCII text these are the indices of the original too.
    pub positions: Vec<u32>,
}

/// Score `text` against a lower-cased `query`, higher is better, `None` for
/// no match. An empty query matches everything with `0`.
pub fn score(text: &str, query: &str, anchor: Anchor) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    score_lower(&text.to_lowercase(), query, anchor)
}

fn body_start(lower: &str, anchor: Anchor) -> usize {
    if anchor == Anchor::Whole {
        return 0;
    }
    lower
        .find([':', '\u{ff1a}'])
        .map(|i| {
            let after = i + lower[i..].chars().next().map_or(1, char::len_utf8);
            let rest = &lower[after..];
            after + rest.len() - rest.trim_start().len()
        })
        .unwrap_or(0)
}

/// [`score`] over an already lower-cased text and query.
pub fn score_lower(lower: &str, query: &str, anchor: Anchor) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    // Characters, not bytes: a CJK label is not three times "longer".
    let len_penalty = || lower.chars().count() as i32;
    let start = body_start(lower, anchor);
    if let Some(pos) = lower[start..].find(query) {
        return Some(30_000 - pos as i32 * 10 - len_penalty());
    }
    // With no category the whole text was just searched: tier 2 is tier 1.
    if start > 0 {
        if let Some(pos) = lower.find(query) {
            return Some(20_000 - pos as i32 * 10 - len_penalty());
        }
    }
    let (first, last) = subsequence_span(lower, query)?;
    let spread = (last - first) as i32;
    Some(10_000 - spread * 10 - len_penalty())
}

/// First and last character index of the leftmost in-order match of every
/// query character, `None` when one is missing.
fn subsequence_span(lower: &str, query: &str) -> Option<(usize, usize)> {
    if query.is_ascii() && lower.is_ascii() {
        // The hot path of the commit search: bytes, no UTF-8 decoding.
        let bytes = lower.as_bytes();
        let (mut from, mut first, mut last) = (0usize, None::<usize>, 0usize);
        for q in query.bytes() {
            let at = from + bytes[from..].iter().position(|&c| c == q)?;
            first.get_or_insert(at);
            last = at;
            from = at + 1;
        }
        return Some((first.unwrap_or(0), last));
    }
    let mut chars = lower.chars();
    let mut first: Option<usize> = None;
    let mut last = 0usize;
    let mut idx = 0usize;
    for qc in query.chars() {
        let mut found = false;
        for c in chars.by_ref() {
            let here = idx;
            idx += 1;
            if c == qc {
                first.get_or_insert(here);
                last = here;
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
    }
    Some((first.unwrap_or(0), last))
}

/// [`score_lower`] plus the matched character positions (computed only for
/// the rows a caller shows, not for every candidate).
pub fn matched(lower: &str, query: &str, anchor: Anchor) -> Option<Match> {
    let score = score_lower(lower, query, anchor)?;
    if query.is_empty() {
        return Some(Match {
            score,
            positions: Vec::new(),
        });
    }
    let start = body_start(lower, anchor);
    let verbatim = lower[start..].find(query).map(|p| start + p).or_else(|| {
        if start > 0 {
            lower.find(query)
        } else {
            None
        }
    });
    let positions = match verbatim {
        Some(byte) => {
            let first = lower[..byte].chars().count() as u32;
            (first..first + query.chars().count() as u32).collect()
        }
        None => {
            let mut chars = lower.chars().enumerate();
            let mut out = Vec::with_capacity(query.len());
            for qc in query.chars() {
                for (i, c) in chars.by_ref() {
                    if c == qc {
                        out.push(i as u32);
                        break;
                    }
                }
            }
            out
        }
    };
    Some(Match { score, positions })
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: Anchor = Anchor::Whole;
    const C: Anchor = Anchor::AfterCategory;

    #[test]
    fn an_empty_query_matches_everything_with_zero() {
        assert_eq!(score("anything", "", W), Some(0));
        assert!(matched("x", "", W).unwrap().positions.is_empty());
    }

    #[test]
    fn the_three_tiers_rank_in_order() {
        let after_category = score("Git: Fetch", "fetch", C).unwrap();
        let anywhere = score("Git: Fetch", "git", C).unwrap();
        let fuzzy = score("Git: Fetch", "gfh", C).unwrap();
        assert!(after_category > anywhere && anywhere > fuzzy);
        assert!(after_category >= 30_000 - 100 && anywhere >= 20_000 - 100);
        assert!(fuzzy <= 10_000 && fuzzy > 9_000);
    }

    #[test]
    fn earlier_and_shorter_win_inside_a_tier() {
        let early = score("fetch all", "fetch", W).unwrap();
        let late = score("please fetch all", "fetch", W).unwrap();
        let long = score("fetch all the things", "fetch", W).unwrap();
        assert!(early > late && early > long);
    }

    #[test]
    fn a_missing_character_is_no_match() {
        assert_eq!(score("fetch", "fz", W), None);
        assert_eq!(score("fetch", "hf", W), None);
    }

    #[test]
    fn a_colon_is_a_category_only_for_the_palette() {
        // Whole-text anchor: `login` is at position 6 of "feat: login".
        assert_eq!(score("feat: login", "login", W), Some(30_000 - 60 - 11));
        // Palette anchor: the body starts after the colon and its space.
        assert_eq!(score("feat: login", "login", C), Some(30_000 - 11));
        // Full-width colon, and a no-break space before the colon.
        assert_eq!(score("提交：拉取", "拉取", C), Some(30_000 - 5));
        assert_eq!(score("Git\u{a0}: Fetch", "fetch", C), Some(30_000 - 11));
    }

    #[test]
    fn the_ascii_fast_path_agrees_with_the_char_path() {
        for (text, query) in [("hello world", "hwd"), ("a-b-c", "abc"), ("xyz", "zy")] {
            let ascii = subsequence_span(text, query);
            // A trailing non-ASCII char forces the general path and cannot
            // change the span of an ASCII query (a miss stays a miss).
            let general = subsequence_span(&format!("{text}é"), query);
            assert_eq!(ascii, general, "{text} / {query}");
        }
    }

    #[test]
    fn positions_name_the_matched_characters() {
        let p = |t, q, a| matched(t, q, a).unwrap().positions;
        assert_eq!(p("fix login bug", "login", W), [4, 5, 6, 7, 8]);
        assert_eq!(p("fix login bug", "flb", W), [0, 4, 10]);
        assert_eq!(p("git: fetch", "fetch", C), [5, 6, 7, 8, 9]);
        // Multi-byte text counts characters, not bytes.
        assert_eq!(p("é fix", "fix", W), [2, 3, 4]);
    }
}
