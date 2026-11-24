//! Intra-line ("changed words") emphasis for the diff views.
//!
//! Inside a hunk, each run of removed lines followed by a run of added lines
//! is paired by order (the first removed line with the first added one, and
//! so on); a pair whose two lines are alike enough gets the words that differ
//! marked, so the row painter can lay a stronger background of the same hue
//! over just those words.
//!
//! The work is done once per render (`hunk_emphasis`, called by the row
//! emitters, whose result `diff_content::DiffContent` caches with the rows),
//! never per frame, and it is bounded: a line over [`MAX_LINE_CHARS`] gets
//! none, one pair's table over [`MAX_PAIR_CELLS`] falls back to marking the
//! whole differing middle, and a hunk spends at most [`HUNK_CELL_BUDGET`]
//! table cells in all (`docs/developer/perf-budgets.md`).
//!
//! Ranges are in terminal **cells** of the tab-expanded text (what the row
//! painter slices, scrolls and wraps by), never chars or bytes.

use super::layout::expand_tabs;
use crate::cell_text::char_cells;
use gitpoulpi_core::domain::{DiffLine, LineOrigin};
use ratatui::prelude::*;

/// A line longer than this (in chars, after tab expansion) gets no emphasis.
pub(super) const MAX_LINE_CHARS: usize = 1000;
/// The largest LCS table (tokens x tokens of the differing middle) one pair
/// may fill; past it the whole middle is marked instead.
const MAX_PAIR_CELLS: usize = 250_000;
/// The LCS table cells one hunk may fill in all (about a millisecond).
const HUNK_CELL_BUDGET: usize = 2_000_000;
/// A pair is marked only when at least this percent of its non-blank
/// characters are unchanged (Dice share over both lines): a rewritten line
/// stays flat rather than becoming a sea of emphasis.
const MIN_SHARE_PERCENT: usize = 40;

/// `[start, end)` in cells.
pub(super) type CellRange = (usize, usize);

/// The marked ranges of a paired removed line and added line.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct PairEmphasis {
    pub(super) old: Vec<CellRange>,
    pub(super) new: Vec<CellRange>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Word,
    Space,
    Other,
}

fn class_of(c: char) -> Class {
    if c.is_whitespace() {
        Class::Space
    } else if (c.is_alphanumeric() || c == '_') && char_cells(c) < 2 {
        Class::Word
    } else {
        Class::Other
    }
}

/// Words, whitespace runs and single punctuation characters, as char
/// ranges. A zero-width char (a combining mark, a variation selector) and
/// whatever follows a zero-width joiner stay with the token before them, so
/// an accented letter or an emoji sequence is never cut in two.
fn tokenize(chars: &[char]) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    let mut prev: Option<Class> = None;
    let mut join_next = false;
    for (i, &c) in chars.iter().enumerate() {
        let attach = !out.is_empty() && (char_cells(c) == 0 || join_next);
        join_next = c == '\u{200d}';
        if attach {
            if let Some(last) = out.last_mut() {
                last.1 = i + 1;
            }
            continue;
        }
        let class = class_of(c);
        match out.last_mut() {
            Some(last) if prev == Some(class) && class != Class::Other => last.1 = i + 1,
            _ => out.push((i, i + 1)),
        }
        prev = Some(class);
    }
    out
}

/// Cells before each char, plus the total.
fn cumulative_cells(chars: &[char]) -> Vec<usize> {
    let mut cum = Vec::with_capacity(chars.len() + 1);
    cum.push(0);
    for &c in chars {
        cum.push(cum[cum.len() - 1] + char_cells(c));
    }
    cum
}

/// Marks which tokens of `a` and `b` are not part of the longest common
/// subsequence of tokens (after trimming the shared head and tail).
/// `budget` is what is left of the hunk's table cells.
fn changed_tokens(a: &[&[char]], b: &[&[char]], budget: &mut usize) -> (Vec<bool>, Vec<bool>) {
    let (na, nb) = (a.len(), b.len());
    let mut ca = vec![true; na];
    let mut cb = vec![true; nb];
    let mut head = 0;
    while head < na.min(nb) && a[head] == b[head] {
        ca[head] = false;
        cb[head] = false;
        head += 1;
    }
    let mut tail = 0;
    while tail < na.min(nb) - head && a[na - 1 - tail] == b[nb - 1 - tail] {
        ca[na - 1 - tail] = false;
        cb[nb - 1 - tail] = false;
        tail += 1;
    }
    let (ma, mb) = (&a[head..na - tail], &b[head..nb - tail]);
    let (n, m) = (ma.len(), mb.len());
    let cells = (n + 1) * (m + 1);
    if n == 0 || m == 0 || cells > MAX_PAIR_CELLS || cells > *budget {
        return (ca, cb);
    }
    *budget -= cells;
    // lcs[i][j]: the common length of ma[i..] and mb[j..].
    let w = m + 1;
    let mut lcs = vec![0u16; cells];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i * w + j] = if ma[i] == mb[j] {
                lcs[(i + 1) * w + j + 1] + 1
            } else {
                lcs[(i + 1) * w + j].max(lcs[i * w + j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if ma[i] == mb[j] {
            ca[head + i] = false;
            cb[head + j] = false;
            i += 1;
            j += 1;
        } else if lcs[(i + 1) * w + j] >= lcs[i * w + j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    (ca, cb)
}

/// The changed tokens as cell ranges. Two marked stretches with exactly one
/// short blank between them are joined: `word word` reads as one change, not
/// two flickering ones.
fn ranges(tokens: &[(usize, usize)], changed: &[bool], chars: &[char]) -> Vec<CellRange> {
    let cum = cumulative_cells(chars);
    let blank = |t: usize| {
        let (s, e) = tokens[t];
        e - s <= 2 && chars[s..e].iter().all(|c| c.is_whitespace())
    };
    let mut out: Vec<CellRange> = Vec::new();
    let mut t = 0;
    while t < tokens.len() {
        if !changed[t] {
            t += 1;
            continue;
        }
        let (from, mut to) = (cum[tokens[t].0], cum[tokens[t].1]);
        let mut next = t + 1;
        loop {
            if next < tokens.len() && changed[next] {
                to = cum[tokens[next].1];
                next += 1;
            } else if next + 1 < tokens.len() && blank(next) && changed[next + 1] {
                to = cum[tokens[next + 1].1];
                next += 2;
            } else {
                break;
            }
        }
        if to > from {
            match out.last_mut() {
                Some(last) if last.1 == from => last.1 = to,
                _ => out.push((from, to)),
            }
        }
        t = next;
    }
    out
}

/// The emphasis of one removed line against the added line that replaces it,
/// or `None` when either is too long, nothing differs, or they are too unlike.
/// Both texts are already tab-expanded.
pub(super) fn pair_emphasis(old: &str, new: &str, budget: &mut usize) -> Option<PairEmphasis> {
    if old == new {
        return None;
    }
    let oc: Vec<char> = old.chars().collect();
    let nc: Vec<char> = new.chars().collect();
    if oc.len() > MAX_LINE_CHARS || nc.len() > MAX_LINE_CHARS {
        return None;
    }
    let ot = tokenize(&oc);
    let nt = tokenize(&nc);
    let os: Vec<&[char]> = ot.iter().map(|&(s, e)| &oc[s..e]).collect();
    let ns: Vec<&[char]> = nt.iter().map(|&(s, e)| &nc[s..e]).collect();
    let (co, cn) = changed_tokens(&os, &ns, budget);
    // The unchanged share, over the non-blank characters of both lines.
    let weight = |toks: &[(usize, usize)], chars: &[char], changed: &[bool], keep: bool| {
        toks.iter()
            .zip(changed)
            .filter(|(_, c)| **c != keep)
            .map(|(&(s, e), _)| chars[s..e].iter().filter(|c| !c.is_whitespace()).count())
            .sum::<usize>()
    };
    let same = weight(&ot, &oc, &co, true) + weight(&nt, &nc, &cn, true);
    let all = weight(&ot, &oc, &co, true)
        + weight(&ot, &oc, &co, false)
        + weight(&nt, &nc, &cn, true)
        + weight(&nt, &nc, &cn, false);
    if all > 0 && same * 100 < MIN_SHARE_PERCENT * all {
        return None;
    }
    let result = PairEmphasis {
        old: ranges(&ot, &co, &oc),
        new: ranges(&nt, &cn, &nc),
    };
    (!result.old.is_empty() || !result.new.is_empty()).then_some(result)
}

/// The emphasis of every line of a hunk, by line index: a removed line gets
/// the old side of its pair, an added line the new side. Lines with no
/// partner, context lines and unlike pairs get an empty list.
pub(super) fn hunk_emphasis(lines: &[DiffLine]) -> Vec<Vec<CellRange>> {
    let mut out: Vec<Vec<CellRange>> = vec![Vec::new(); lines.len()];
    let mut budget = HUNK_CELL_BUDGET;
    let mut i = 0;
    while i < lines.len() {
        if lines[i].origin != LineOrigin::Delete {
            i += 1;
            continue;
        }
        let del_start = i;
        while i < lines.len() && lines[i].origin == LineOrigin::Delete {
            i += 1;
        }
        let add_start = i;
        while i < lines.len() && lines[i].origin == LineOrigin::Add {
            i += 1;
        }
        let pairs = (add_start - del_start).min(i - add_start);
        for n in 0..pairs {
            let old = expand_tabs(&lines[del_start + n].content);
            let new = expand_tabs(&lines[add_start + n].content);
            if let Some(p) = pair_emphasis(&old, &new, &mut budget) {
                out[del_start + n] = p.old;
                out[add_start + n] = p.new;
            }
        }
    }
    out
}

/// `piece` (what one drawn row shows of a line) as spans: the cells inside
/// `marked` in `emph`, the rest in `base`.
///
/// Column `c` of the piece is cell `cell_base + c - lead` of the line's
/// tab-expanded text; the first `lead` columns (a scroll marker) are never
/// marked. A zero-width char takes the mark of the char before it.
pub(super) fn emph_spans(
    piece: &str,
    (cell_base, lead): (usize, usize),
    marked: &[CellRange],
    base: Style,
    emph: Style,
) -> Vec<Span<'static>> {
    if marked.is_empty() {
        return vec![Span::styled(piece.to_string(), base)];
    }
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut run = String::new();
    let mut run_marked = false;
    let mut col = 0usize;
    let mut at = 0usize; // first range that may still contain the cell
    for c in piece.chars() {
        let w = char_cells(c);
        let is_marked = if w == 0 {
            run_marked
        } else if col < lead {
            false
        } else {
            let cell = cell_base + col - lead;
            while at < marked.len() && marked[at].1 <= cell {
                at += 1;
            }
            at < marked.len() && marked[at].0 <= cell
        };
        if is_marked != run_marked && !run.is_empty() {
            spans.push(Span::styled(
                std::mem::take(&mut run),
                if run_marked { emph } else { base },
            ));
        }
        run_marked = is_marked;
        run.push(c);
        col += w;
    }
    if !run.is_empty() {
        spans.push(Span::styled(run, if run_marked { emph } else { base }));
    }
    spans
}

/// The style of the marked words of an added or removed line: `base` with the
/// stronger background `emph_bg`. When the terminal's color depth would draw
/// `emph_bg` the same as the line's own `line_bg` (16 colors, or a theme with
/// no background of its own), bold and underline mark the words instead.
pub(super) fn emph_style(base: Style, line_bg: Color, emph_bg: Color) -> Style {
    emph_style_at(base, line_bg, emph_bg, crate::color_depth::active())
}

fn emph_style_at(
    base: Style,
    line_bg: Color,
    emph_bg: Color,
    depth: crate::color_depth::Depth,
) -> Style {
    if crate::color_depth::distinct_at(line_bg, emph_bg, depth) {
        base.bg(emph_bg)
    } else {
        base.add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
    }
}

/// The marked-words style for a line of this origin, `None` for context.
pub(super) fn emph_for(origin: &LineOrigin, base: Style) -> Option<Style> {
    let diff = &crate::th().diff;
    match origin {
        LineOrigin::Add => Some(emph_style(base, diff.add_bg, diff.add_emph_bg)),
        LineOrigin::Delete => Some(emph_style(base, diff.del_bg, diff.del_emph_bg)),
        LineOrigin::Context => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(old: &str, new: &str) -> Option<PairEmphasis> {
        pair_emphasis(old, new, &mut HUNK_CELL_BUDGET.clone())
    }

    /// The marked text of `s` for `ranges` (ASCII or a char per cell).
    fn marked(s: &str, ranges: &[CellRange]) -> Vec<String> {
        let mut cell = 0;
        let mut out: Vec<String> = vec![String::new(); ranges.len()];
        for c in s.chars() {
            for (k, r) in ranges.iter().enumerate() {
                if cell >= r.0 && cell < r.1 {
                    out[k].push(c);
                }
            }
            cell += char_cells(c);
        }
        out
    }

    #[test]
    fn tokens_split_words_blanks_and_punctuation() {
        let chars: Vec<char> = "foo_bar  = (x);".chars().collect();
        let toks: Vec<String> = tokenize(&chars)
            .into_iter()
            .map(|(s, e)| chars[s..e].iter().collect())
            .collect();
        assert_eq!(toks, ["foo_bar", "  ", "=", " ", "(", "x", ")", ";"]);
    }

    #[test]
    fn accents_stay_inside_their_word() {
        let chars: Vec<char> = "récupère les branches".chars().collect();
        let toks: Vec<String> = tokenize(&chars)
            .into_iter()
            .map(|(s, e)| chars[s..e].iter().collect())
            .collect();
        assert_eq!(toks, ["récupère", " ", "les", " ", "branches"]);
        // A decomposed accent (e + combining acute) is not cut off its letter.
        let chars: Vec<char> = "re\u{301}cupe".chars().collect();
        assert_eq!(tokenize(&chars), vec![(0, 7)]);
    }

    #[test]
    fn an_emoji_sequence_is_one_token() {
        let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
        let chars: Vec<char> = format!("a {family} b").chars().collect();
        let toks: Vec<String> = tokenize(&chars)
            .into_iter()
            .map(|(s, e)| chars[s..e].iter().collect())
            .collect();
        assert_eq!(toks, ["a", " ", family, " ", "b"]);
    }

    #[test]
    fn only_the_differing_words_are_marked() {
        let old = "Tire chaque branche locale qui a un amont : r\u{e9}cup\u{e8}re les commits";
        let new = "Met \u{e0} jour toutes les branches locales qui ont un amont : r\u{e9}cup\u{e8}re les commits";
        let p = pair(old, new).expect("alike enough");
        let removed = marked(old, &p.old);
        let added = marked(new, &p.new);
        assert!(removed.iter().any(|m| m.contains("Tire chaque branche")));
        assert!(added.iter().any(|m| m.contains("Met")));
        // The shared tail is never marked, accents included.
        assert!(removed.iter().all(|m| !m.contains("amont")));
        assert!(added.iter().all(|m| !m.contains("r\u{e9}cup")));
    }

    #[test]
    fn one_word_change_marks_that_word_only() {
        let p = pair("let value = compute(a, b);", "let value = compute(a, c);").unwrap();
        assert_eq!(marked("let value = compute(a, b);", &p.old), ["b"]);
        assert_eq!(marked("let value = compute(a, c);", &p.new), ["c"]);
    }

    #[test]
    fn a_whitespace_only_change_marks_the_blanks() {
        let p = pair("    return x;", "        return x;").unwrap();
        assert_eq!(marked("    return x;", &p.old), ["    "]);
        assert_eq!(marked("        return x;", &p.new), ["        "]);
    }

    #[test]
    fn unlike_lines_get_no_emphasis() {
        assert_eq!(pair("alpha beta gamma", "one two three four"), None);
        assert_eq!(pair("same", "same"), None);
    }

    #[test]
    fn a_line_over_the_cap_gets_none() {
        let long = "word ".repeat(MAX_LINE_CHARS / 5 + 10);
        let other = format!("{long}x");
        assert_eq!(pair(&long, &other), None);
    }

    #[test]
    fn a_blank_between_two_changes_joins_them() {
        let (old, new) = ("keep aaa bbb keep", "keep ccc ddd keep");
        let p = pair(old, new).unwrap();
        assert_eq!(marked(old, &p.old), ["aaa bbb"]);
        assert_eq!(marked(new, &p.new), ["ccc ddd"]);
    }

    #[test]
    fn wide_chars_count_two_cells() {
        let (old, new) = ("\u{65e5}\u{672c} aaa end", "\u{65e5}\u{672c} bbb end");
        let p = pair(old, new).unwrap();
        assert_eq!(p.old, vec![(5, 8)]);
        assert_eq!(p.new, vec![(5, 8)]);
    }

    #[test]
    fn a_huge_differing_middle_falls_back_to_marking_all_of_it() {
        let a: String = (0..150).map(|i| format!("a{i} ")).collect();
        let b: String = (0..150).map(|i| format!("b{i} ")).collect();
        let (old, new) = (format!("head {a}tail"), format!("head {b}tail"));
        // Unlike enough to be refused, or marked wholesale: never slow.
        let mut budget = 10;
        let _ = pair_emphasis(&old, &new, &mut budget);
        assert_eq!(budget, 10, "the table was not built");
    }

    fn l(origin: LineOrigin, s: &str) -> DiffLine {
        DiffLine {
            origin,
            content: s.to_string(),
            staged: false,
        }
    }

    #[test]
    fn runs_pair_by_order_and_the_extra_lines_stay_flat() {
        use LineOrigin::{Add, Context, Delete};
        let lines = vec![
            l(Context, "ctx"),
            l(Delete, "let one = 1;"),
            l(Delete, "let two = 2;"),
            l(Delete, "let three = 3;"),
            l(Add, "let one = 10;"),
            l(Add, "let two = 20;"),
            l(Context, "ctx"),
        ];
        let e = hunk_emphasis(&lines);
        assert!(e[0].is_empty() && e[6].is_empty());
        assert!(!e[1].is_empty() && !e[4].is_empty(), "1st with 1st");
        assert!(!e[2].is_empty() && !e[5].is_empty(), "2nd with 2nd");
        assert!(e[3].is_empty(), "the unpaired removed line stays flat");
    }

    #[test]
    fn no_distinct_background_falls_back_to_bold_and_underline() {
        use crate::color_depth::Depth;
        let base = Style::default().fg(Color::Green);
        let (flat, strong) = (Color::Rgb(24, 52, 38), Color::Rgb(36, 88, 60));
        // Truecolor: the stronger background itself.
        assert_eq!(
            emph_style_at(base, flat, strong, Depth::True),
            base.bg(strong)
        );
        // A theme with no background of its own (the Terminal preset).
        let marked = emph_style_at(base, Color::Reset, Color::Reset, Depth::True);
        assert_eq!(marked.bg, None);
        assert!(marked
            .add_modifier
            .contains(Modifier::BOLD | Modifier::UNDERLINED));
        // Two tints the 16 colors draw alike: still told apart.
        assert!(!crate::color_depth::distinct_at(flat, strong, Depth::X16));
        let marked = emph_style_at(base, flat, strong, Depth::X16);
        assert!(marked
            .add_modifier
            .contains(Modifier::BOLD | Modifier::UNDERLINED));
    }

    #[test]
    fn spans_split_at_the_marked_cells() {
        let base = Style::default().fg(Color::Green);
        let emph = base.bg(Color::Red);
        let spans = emph_spans("abcdef", (0, 0), &[(2, 4)], base, emph);
        let parts: Vec<(&str, bool)> = spans
            .iter()
            .map(|s| (s.content.as_ref(), s.style == emph))
            .collect();
        assert_eq!(parts, [("ab", false), ("cd", true), ("ef", false)]);
        // A scrolled row: one marker column, then the line from cell 3.
        let spans = emph_spans("\u{ab}defg", (3, 1), &[(4, 5)], base, emph);
        let parts: Vec<(&str, bool)> = spans
            .iter()
            .map(|s| (s.content.as_ref(), s.style == emph))
            .collect();
        assert_eq!(parts, [("\u{ab}d", false), ("e", true), ("fg", false)]);
    }
}
