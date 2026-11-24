//! Search across the commit graph: the pure half.
//!
//! This is a *search*, not a filter. Non-matching commits stay in the table
//! and are merely dimmed; the user steps between hits with the up/down
//! arrows and the counter says where they are (`3 of 16`). The Refs
//! sidebar's solo mechanism genuinely removes commits from the graph, and
//! conflating the two would make "search" silently destroy the view it was
//! supposed to help read.
//!
//! Everything here is a pure function over already-fetched data (no daemon
//! round trip, no I/O), so the render loop can recompute matches on every
//! keystroke and the whole thing is testable without a terminal.

use crate::th;

/// ⌕ (telephone recorder) reads as a magnifier and has text presentation;
/// shared by the graph search bar and the tab bar's Global search button.
pub(crate) const MAGNIFIER: &str = "\u{2315}";
use gitpoulpi_core::domain::{CommitNode, Ref, RefKind};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

/// A clickable control in the search bar. Mirrors the `HitTarget` +
/// `frame_hits` convention the toolbar and the diff bar already use, so
/// the bar doesn't invent a third way of being clickable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchBarButton {
    Prev,
    Next,
    Close,
}

/// Live state of an open search. Absent (`None` at the call site) means
/// the bar is closed and the Log renders exactly as it always did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogSearch {
    pub query: String,
    /// Position *within the match list*, not a commit index. Kept clamped
    /// by `clamp_current` after every query edit, since the match list
    /// shrinks and grows as the user types.
    pub current: usize,
}

impl LogSearch {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Whether one commit matches `query`, case-insensitively.
///
/// Deliberately wider than the commit message alone: it matches
/// author and SHA too, which suits "who wrote this" or a SHA pasted from a
/// bug report. Ref names are included so typing a branch name jumps to its
/// tip.
///
/// An all-whitespace or empty query matches nothing (`find_matches`
/// short-circuits before calling this).
pub fn commit_matches(commit: &CommitNode, refs: &[Ref], query: &str) -> bool {
    let needle = query.to_lowercase();
    if commit.message.to_lowercase().contains(&needle)
        || commit.author_name.to_lowercase().contains(&needle)
        || commit.author_email.to_lowercase().contains(&needle)
        // Hex is lowercase already, but normalise anyway so a SHA pasted
        // in uppercase still matches.
        || commit.id.to_hex().contains(&needle)
    {
        return true;
    }
    refs.iter().filter(|r| r.target == commit.id).any(|r| {
        let name = match &r.kind {
            RefKind::LocalBranch { name } | RefKind::Tag { name } => name.clone(),
            RefKind::RemoteBranch { remote, name } => format!("{remote}/{name}"),
        };
        name.to_lowercase().contains(&needle)
    })
}

/// Indices into `nodes` of every commit matching `query`, in display order.
///
/// An empty/whitespace-only query yields **no** matches rather than all of
/// them: matching everything would make the counter read `1 of 847` and dim
/// nothing the instant the bar opens, which reads as broken.
pub fn find_matches(nodes: &[CommitNode], refs: &[Ref], query: &str) -> Vec<usize> {
    if query.trim().is_empty() {
        return Vec::new();
    }
    nodes
        .iter()
        .enumerate()
        .filter(|(_, c)| commit_matches(c, refs, query))
        .map(|(i, _)| i)
        .collect()
}

/// The hits when an author filter may be on: the text hits (`None`: no text
/// search, a blank query) narrowed to the chosen authors' commits; with no text
/// search the author filter's own commits are the hits. No filter and no text
/// is no hits, as ever. This is the one place the two combine (AND), used by
/// the paint's cache and by the key handlers, so the counter, the stepping and
/// the dimming can never disagree.
pub fn combine_hits(
    nodes: &[CommitNode],
    text: Option<Vec<usize>>,
    authors: &std::collections::BTreeSet<String>,
) -> Vec<usize> {
    if authors.is_empty() {
        return text.unwrap_or_default();
    }
    let by_author = |i: &usize| {
        nodes
            .get(*i)
            .is_some_and(|c| crate::author_filter::commit_by(c, authors))
    };
    match text {
        Some(t) => t.into_iter().filter(by_author).collect(),
        None => (0..nodes.len()).filter(by_author).collect(),
    }
}

/// [`find_matches`] with the author filter applied ([`combine_hits`]).
pub fn find_matches_with_authors(
    nodes: &[CommitNode],
    refs: &[Ref],
    query: &str,
    authors: &std::collections::BTreeSet<String>,
) -> Vec<usize> {
    let text = (!query.trim().is_empty()).then(|| find_matches(nodes, refs, query));
    combine_hits(nodes, text, authors)
}

/// Where an arrow key lands while hits are lit: the display row of the
/// `count`-th hit from `selected` in the direction of `delta`'s sign, never a
/// dimmed row in between. `hits` are display rows, ascending. A selection on a
/// dimmed row goes to the nearest hit that way; fewer hits than `count` stop at
/// the last one; none that way (or no hits) is `None`, the selection stays.
pub(crate) fn step_among_hits(
    hits: &[usize],
    selected: Option<usize>,
    delta: i32,
) -> Option<usize> {
    let n = delta.unsigned_abs() as usize;
    if n == 0 || hits.is_empty() {
        return None;
    }
    let Some(sel) = selected else {
        return if delta > 0 { hits.first() } else { hits.last() }.copied();
    };
    if delta > 0 {
        let first = hits.partition_point(|&h| h <= sel);
        hits.get((first + n - 1).min(hits.len() - 1))
            .copied()
            .filter(|_| first < hits.len())
    } else {
        let before = hits.partition_point(|&h| h < sel);
        before
            .checked_sub(1)
            .map(|last| hits[last.saturating_sub(n - 1)])
    }
}

/// PageUp/PageDown over hits: the first hit at or past the row a plain page
/// would reach (the last hit when that is beyond them all), strictly beyond
/// the selection. `None` keeps the selection.
pub(crate) fn page_among_hits(
    hits: &[usize],
    selected: Option<usize>,
    delta: i32,
) -> Option<usize> {
    let sel = selected.unwrap_or(0);
    if delta > 0 {
        let target = sel.saturating_add(delta as usize);
        let at = hits.partition_point(|&h| h < target);
        let pick = hits.get(at).or(hits.last()).copied()?;
        (pick > sel || selected.is_none()).then_some(pick)
    } else {
        let target = sel.saturating_sub(delta.unsigned_abs() as usize);
        let at = hits.partition_point(|&h| h <= target);
        let pick = at
            .checked_sub(1)
            .map(|i| hits[i])
            .or(hits.first().copied())?;
        (pick < sel || selected.is_none()).then_some(pick)
    }
}

/// Lowercased haystacks for a node list, built once so each keystroke is
/// a `contains` scan rather than five `to_lowercase` allocations per
/// commit. Matches exactly what [`commit_matches`] matches (message,
/// author name, author email, sha, names of refs on the commit), with the
/// fields joined by a control character a query will not contain.
pub struct SearchIndex {
    hay: Vec<String>,
}

impl SearchIndex {
    pub fn new(nodes: &[CommitNode], refs: &[Ref]) -> SearchIndex {
        let mut names: std::collections::HashMap<_, Vec<String>> = std::collections::HashMap::new();
        for r in refs {
            let name = match &r.kind {
                RefKind::LocalBranch { name } | RefKind::Tag { name } => name.clone(),
                RefKind::RemoteBranch { remote, name } => format!("{remote}/{name}"),
            };
            names.entry(r.target).or_default().push(name.to_lowercase());
        }
        let hay = nodes
            .iter()
            .map(|c| {
                let mut h = format!(
                    "{}\u{1}{}\u{1}{}\u{1}{}",
                    c.message.to_lowercase(),
                    c.author_name.to_lowercase(),
                    c.author_email.to_lowercase(),
                    c.id.to_hex()
                );
                for n in names.get(&c.id).into_iter().flatten() {
                    h.push('\u{1}');
                    h.push_str(n);
                }
                h
            })
            .collect();
        SearchIndex { hay }
    }

    /// Same contract as [`find_matches`].
    pub fn find(&self, query: &str) -> Vec<usize> {
        if query.trim().is_empty() {
            return Vec::new();
        }
        let needle = query.to_lowercase();
        self.hay
            .iter()
            .enumerate()
            .filter(|(_, h)| h.contains(&needle))
            .map(|(i, _)| i)
            .collect()
    }
}

/// Keeps `current` inside `0..len` after the match list changed under it.
/// An empty list parks it at 0 so the counter can render `0 of 0`.
pub fn clamp_current(current: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        current.min(len - 1)
    }
}

/// Step to the next/previous match, wrapping at both ends.
///
/// Wrapping is what makes Enter-Enter-Enter a usable way to tour every
/// hit without watching the counter for the end of the list.
pub fn step(current: usize, len: usize, forward: bool) -> usize {
    if len == 0 {
        return 0;
    }
    if forward {
        (current + 1) % len
    } else {
        (current + len - 1) % len
    }
}

/// The counter's text: `3 of 16`, or `0 of 0` when nothing matches.
///
/// `current` is 0-based internally and 1-based here, as in
/// every find bar.
pub fn counter_text(current: usize, len: usize) -> String {
    if len == 0 {
        gitpoulpi_i18n::msg::find_bar::counter(0, 0)
    } else {
        gitpoulpi_i18n::msg::find_bar::counter(current + 1, len)
    }
}

/// The widest counter a graph of `total` commits can ever produce, in
/// columns: the last match of all of them (`847 of 847`).
///
/// The counter sits between the query field and the buttons, so sizing it
/// to the *current* match count would undo the fixed-width field: the
/// buttons would jump a column sideways when the hit count crossed 9, 99,
/// ... while typing. `total` does not change under the keyboard, so a slot
/// sized from it is stable.
fn counter_slot_width(total: usize) -> usize {
    crate::display_width(&counter_text(total.saturating_sub(1), total))
}

/// Renders the overlay bar and returns it with the click regions of its
/// three buttons as `(x_offset, width, button)`, for the caller to register
/// against `frame_hits` at the bar's own origin.
///
/// Laid out right-aligned within `width`, as a find bar does:
/// magnifier, the query, the counter, then ↑ ↓ ✕.
///
/// **Every part of this layout has a fixed width.** Drawing the query
/// verbatim would slide the counter and the buttons with every keystroke,
/// moving the button the user is aiming at out from under the mouse. The
/// query gets a reserved slot it scrolls inside (`text_field::field_window`)
/// and the counter a slot sized from `total` rather than the live match
/// count, so only the *text* changes as the user types, never the geometry.
#[cfg(test)]
pub fn search_bar(
    search: &LogSearch,
    match_count: usize,
    total: usize,
    width: u16,
) -> (Line<'static>, Vec<(u16, u16, SearchBarButton)>) {
    let (line, hits, _) = search_bar_edit(search, None, match_count, total, width);
    (line, hits)
}

/// What [`search_bar_edit`] drew: the bar, its buttons' click spans
/// (column offset, width, button), and the query text's own span
/// (column offset, width) - where a click in the query lands.
pub(crate) type SearchBarDrawn = (Line<'static>, Vec<(u16, u16, SearchBarButton)>, (u16, u16));

/// [`search_bar`] with the query field's caret and selection, also returning where the query's
/// text sits in the bar as
/// (column offset from the bar's left edge, width).
pub(crate) fn search_bar_edit(
    search: &LogSearch,
    caret: Option<&crate::text_edit::Caret>,
    match_count: usize,
    total: usize,
    width: u16,
) -> SearchBarDrawn {
    // All BMP box-drawing/technical glyphs with no emoji presentation --
    // a VS16-dependent magnifier or arrow set is what produced the ref
    // pill's rendering "hole" (fixed in 58200e1), so none are used here.
    const UP: &str = "\u{2191}";
    const DOWN: &str = "\u{2193}";
    const CLOSE: &str = "\u{2715}";

    let counter = counter_text(search.current, match_count);
    // `total` is the upper bound on `match_count`, so the slot always
    // holds the counter -- but clamp defensively rather than let a
    // mismatched pair silently overflow the bar's rect.
    let counter_w = counter_slot_width(total).max(crate::display_width(&counter));

    // Fixed overhead, counted exactly: " ⌕ " (3) + "  " and the trailing
    // space around the counter (3) + three 3-cell buttons (9) = 15, plus the
    // counter's slot. The block cursor is not counted: it lives *inside* the
    // query field. Guessing this number is how the bar ends up one cell too
    // wide and pushes its own close button off the edge.
    const BUTTONS_W: usize = 9;
    let chrome = 3 + 3 + counter_w + BUTTONS_W;
    // In a pane too narrow for all three, the counter is what goes: a
    // one-column query field would be useless, and the buttons have
    // keyboard equivalents but no other visible affordance.
    let (field_w, counter_w) = if (width as usize) > chrome {
        (width as usize - chrome, counter_w)
    } else {
        ((width as usize).saturating_sub(3 + BUTTONS_W), 0)
    };
    // The query scrolls inside its slot, elided from the left so the
    // caret -- where the user is actually typing -- stays visible.

    let no_hits = match_count == 0 && !search.query.trim().is_empty();
    let query_style = if no_hits {
        // A query that matches nothing says so in the field itself, not
        // only in the counter -- the counter is small and easy to miss.
        Style::default()
            .fg(th().status.warning)
            .bg(crate::input_focus_bg())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(th().ui.accent)
            .bg(crate::input_focus_bg())
            .add_modifier(Modifier::BOLD)
    };

    let mut spans = vec![Span::styled(
        format!(" {MAGNIFIER} "),
        Style::default().fg(th().ui.muted),
    )];
    if field_w > 0 {
        // The block cursor makes it obvious the field has the keyboard,
        // which matters because every other key is being swallowed. The
        // field's unused tail is blank, but reserved: this is the padding
        // that stops the counter and the buttons from moving.
        let look = crate::text_field::EditLook::new(query_style, "\u{2588}").caret_style(
            Style::default()
                .fg(th().ui.accent)
                .bg(crate::input_focus_bg()),
        );
        spans.extend(crate::text_field::edit_line(&search.query, caret, field_w, true, look).spans);
    }
    if counter_w > 0 {
        spans.push(Span::styled(
            format!("  {counter:<counter_w$} "),
            Style::default().fg(th().ui.muted),
        ));
    }

    let mut hits = Vec::new();
    // Terminal columns, not chars: a click region computed by char count
    // lands left of the button it names the moment anything wide (a CJK
    // query, an emoji pasted into the field) is on the bar.
    let mut x: u16 = spans.iter().map(|s| s.width() as u16).sum();
    for (label, button) in [
        (UP, SearchBarButton::Prev),
        (DOWN, SearchBarButton::Next),
        (CLOSE, SearchBarButton::Close),
    ] {
        let text = format!(" {label} ");
        let w = crate::display_width(&text) as u16;
        spans.push(Span::styled(
            text,
            Style::default().fg(th().ui.accent).bg(th().ui.selection),
        ));
        hits.push((x, w, button));
        x += w;
    }

    (Line::from(spans), hits, (3, field_w as u16))
}

#[cfg(test)]
mod tests {

    #[test]
    fn arrows_skip_dimmed_rows_between_hits() {
        let hits = [2, 5, 9];
        assert_eq!(step_among_hits(&hits, Some(2), 1), Some(5));
        assert_eq!(step_among_hits(&hits, Some(5), -1), Some(2));
        // On a dimmed row: nearest hit that way.
        assert_eq!(step_among_hits(&hits, Some(3), 1), Some(5));
        assert_eq!(step_among_hits(&hits, Some(3), -1), Some(2));
        // Nothing that way: stay.
        assert_eq!(step_among_hits(&hits, Some(9), 1), None);
        assert_eq!(step_among_hits(&hits, Some(2), -1), None);
        assert_eq!(step_among_hits(&hits, Some(1), -1), None);
        // A count stops at the last hit.
        assert_eq!(step_among_hits(&hits, Some(2), 5), Some(9));
        assert_eq!(step_among_hits(&hits, Some(9), -2), Some(2));
        assert_eq!(step_among_hits(&[], Some(1), 1), None);
        assert_eq!(step_among_hits(&hits, None, 1), Some(2));
    }

    #[test]
    fn pages_land_on_a_hit() {
        let hits = [2, 5, 9];
        assert_eq!(page_among_hits(&hits, Some(2), 4), Some(9));
        assert_eq!(page_among_hits(&hits, Some(2), 3), Some(5));
        assert_eq!(page_among_hits(&hits, Some(5), 40), Some(9));
        assert_eq!(page_among_hits(&hits, Some(9), 4), None);
        assert_eq!(page_among_hits(&hits, Some(9), -4), Some(5));
        assert_eq!(page_among_hits(&hits, Some(5), -40), Some(2));
        assert_eq!(page_among_hits(&hits, Some(2), -4), None);
    }
    use super::*;
    use gitpoulpi_core::domain::Oid;

    fn oid(n: u8) -> Oid {
        let mut bytes = [0u8; 20];
        bytes[0] = n;
        Oid(bytes)
    }

    fn commit(n: u8, summary: &str, author: &str) -> CommitNode {
        CommitNode {
            id: oid(n),
            parents: Vec::new(),
            summary: summary.to_string(),
            message: summary.to_string(),
            author_name: author.to_string(),
            author_email: format!("{}@example.com", author.to_lowercase()),
            timestamp: 0,
            refs: Vec::new(),
        }
    }

    fn nodes() -> Vec<CommitNode> {
        vec![
            commit(1, "Merge branch 'feature'", "Alice"),
            commit(2, "fix: overflow guard", "Bob"),
            commit(3, "Merge branch 'hotfix'", "Bob"),
        ]
    }

    #[test]
    fn search_is_case_insensitive_across_message_and_author() {
        let refs: Vec<Ref> = Vec::new();
        assert_eq!(find_matches(&nodes(), &refs, "merg"), vec![0, 2]);
        assert_eq!(find_matches(&nodes(), &refs, "MERG"), vec![0, 2]);
        // Author, not message.
        assert_eq!(find_matches(&nodes(), &refs, "bob"), vec![1, 2]);
    }

    #[test]
    fn a_sha_prefix_matches_its_commit() {
        let refs: Vec<Ref> = Vec::new();
        let hex = oid(2).to_hex();
        assert_eq!(find_matches(&nodes(), &refs, &hex[..6]), vec![1]);
    }

    #[test]
    fn a_ref_name_matches_the_commit_it_points_at() {
        let refs = vec![Ref {
            kind: RefKind::LocalBranch {
                name: "release/1.0".into(),
            },
            target: oid(3),
            visibility: Default::default(),
            is_head: false,
            upstream: None,
            tag_sync: None,
            multi_remote: None,
        }];
        assert_eq!(find_matches(&nodes(), &refs, "release"), vec![2]);
    }

    /// An empty query must match NOTHING, not everything - otherwise
    /// opening the bar instantly claims "1 of 847" and dims no rows,
    /// which reads as a broken search rather than an empty one.
    #[test]
    fn an_empty_query_matches_nothing() {
        let refs: Vec<Ref> = Vec::new();
        assert!(find_matches(&nodes(), &refs, "").is_empty());
        assert!(find_matches(&nodes(), &refs, "   ").is_empty());
        assert_eq!(counter_text(0, 0), "0 of 0");
    }

    /// Enter/Enter/Enter should tour every hit and come back round,
    /// rather than sticking on the last one.
    #[test]
    fn stepping_wraps_at_both_ends() {
        assert_eq!(step(0, 3, true), 1);
        assert_eq!(
            step(2, 3, true),
            0,
            "forward past the end wraps to the first"
        );
        assert_eq!(
            step(0, 3, false),
            2,
            "backward past the start wraps to the last"
        );
        assert_eq!(step(0, 0, true), 0, "no matches: stays put, no panic");
    }

    #[test]
    fn current_is_clamped_when_the_match_list_shrinks() {
        assert_eq!(clamp_current(7, 3), 2);
        assert_eq!(clamp_current(7, 0), 0);
        assert_eq!(clamp_current(1, 3), 1);
    }

    #[test]
    fn counter_is_one_based_for_display() {
        assert_eq!(counter_text(0, 16), "1 of 16");
        assert_eq!(counter_text(15, 16), "16 of 16");
    }

    /// A query longer than the bar must not push the buttons off the
    /// edge; it's elided from the left so the typing end stays visible.
    #[test]
    fn a_long_query_is_elided_and_the_bar_still_fits() {
        let search = LogSearch {
            query: "a".repeat(200),
            current: 0,
        };
        let (line, hits) = search_bar(&search, 1, 400, 60);
        assert_eq!(hits.len(), 3);
        assert!(
            line.width() <= 60,
            "bar rendered {} cells wide, wider than its 60-cell area",
            line.width()
        );
    }

    /// The query field's slot, and therefore the counter and the three
    /// buttons beside it, must sit at exactly the same columns whatever has
    /// been typed. A verbatim-drawn field put ✕ at x=25 for `fix` and x=41
    /// for a 20-char query, moving the button out from under the mouse
    /// mid-click.
    #[test]
    fn the_field_and_the_buttons_keep_their_columns_whatever_is_typed() {
        const WIDTH: u16 = 46;
        let baseline = search_bar(
            &LogSearch {
                query: String::new(),
                current: 0,
            },
            0,
            400,
            WIDTH,
        );
        for (query, match_count, current) in [
            ("fix".to_string(), 3usize, 0usize),
            ("fix: overflow".to_string(), 12, 9),
            ("a".repeat(60), 0, 0),
            ("日本語のコミット".to_string(), 7, 6),
            ("🚀 rocket".to_string(), 128, 99),
        ] {
            let (line, hits) = search_bar(
                &LogSearch {
                    query: query.clone(),
                    current,
                },
                match_count,
                400,
                WIDTH,
            );
            assert_eq!(
                line.width(),
                baseline.0.width(),
                "{query:?}: the bar rendered {} columns, not the {} an empty query does",
                line.width(),
                baseline.0.width()
            );
            assert_eq!(
                line.width(),
                WIDTH as usize,
                "{query:?}: the bar must fill its fixed rect exactly"
            );
            assert_eq!(
                hits, baseline.1,
                "{query:?}: the ↑ ↓ ✕ buttons moved with the query"
            );
            // The caret has to still be on the bar -- a fixed width that
            // scrolls the caret out of view trades one bug for another.
            assert!(
                line.spans.iter().any(|s| s.content.contains('\u{2588}')),
                "{query:?}: the block cursor fell off the field"
            );
        }
    }

    /// The same property, but against real painted cells rather than the
    /// `Line`'s arithmetic: a headless width calculation can agree with
    /// itself and still be wrong about what lands on screen.
    #[test]
    fn painted_cells_put_the_chrome_in_the_same_columns_at_any_query_length() {
        use ratatui::buffer::Buffer;
        use ratatui::layout::Rect;
        use ratatui::widgets::{Paragraph, Widget};

        const WIDTH: u16 = 46;
        let paint = |query: &str, match_count: usize| -> Vec<String> {
            let (line, _) = search_bar(
                &LogSearch {
                    query: query.to_string(),
                    current: 0,
                },
                match_count,
                400,
                WIDTH,
            );
            let area = Rect::new(0, 0, WIDTH, 1);
            let mut buf = Buffer::empty(area);
            Paragraph::new(line).render(area, &mut buf);
            (0..WIDTH)
                .map(|x| buf[(x, 0)].symbol().to_string())
                .collect()
        };

        let short = paint("fix", 3);
        let long = paint(&"overflowing query ".repeat(4), 0);
        // The chrome: magnifier, the ↑ ↓ ✕ buttons. Same columns, or the
        // field grew and shoved them along.
        for glyph in ["\u{2315}", "\u{2191}", "\u{2193}", "\u{2715}"] {
            let a = short.iter().position(|s| s == glyph);
            let b = long.iter().position(|s| s == glyph);
            assert!(a.is_some(), "{glyph} never got painted");
            assert_eq!(
                a, b,
                "{glyph} sits at column {a:?} with a 3-char query and {b:?} with a long one"
            );
        }
        // And the caret is still on screen in both, inside the field --
        // left of the counter, not clipped off the right edge.
        let caret = |cells: &[String]| cells.iter().position(|s| s == "\u{2588}");
        let up = short.iter().position(|s| s == "\u{2191}").unwrap();
        for cells in [&short, &long] {
            let c = caret(cells).expect("the block cursor fell off the field");
            assert!(c < up, "the caret was painted at {c}, past the buttons");
        }
    }

    /// Whatever the pane's width, the bar fills it exactly and never
    /// overflows it - an overflowing `Line` is silently clipped by
    /// ratatui, which is how the close button disappeared off the edge.
    #[test]
    fn the_bar_fills_its_rect_exactly_at_every_usable_width() {
        let search = LogSearch {
            query: "overflow".into(),
            current: 0,
        };
        // 12 columns is the magnifier plus the three buttons: below that
        // the caller doesn't draw the bar at all.
        for width in 12..=60u16 {
            let (line, _) = search_bar(&search, 3, 400, width);
            assert_eq!(
                line.width(),
                width as usize,
                "at {width} columns the bar drew {}",
                line.width()
            );
        }
    }

    /// The counter's slot is sized from the graph, not from the live hit
    /// count - otherwise the buttons still jump a column the moment the
    /// count crosses 9 or 99 under the keyboard.
    #[test]
    fn a_growing_hit_count_does_not_move_the_buttons() {
        let search = LogSearch {
            query: "fix".into(),
            current: 0,
        };
        let (_, few) = search_bar(&search, 9, 900, 46);
        let (_, many) = search_bar(&search, 900, 900, 46);
        assert_eq!(few, many);
    }

    #[test]
    fn the_bar_exposes_three_click_targets_in_order() {
        let search = LogSearch {
            query: "merg".into(),
            current: 0,
        };
        let (_, hits) = search_bar(&search, 2, 40, 80);
        let buttons: Vec<_> = hits.iter().map(|(_, _, b)| *b).collect();
        assert_eq!(
            buttons,
            vec![
                SearchBarButton::Prev,
                SearchBarButton::Next,
                SearchBarButton::Close
            ]
        );
        // Regions must not overlap, or a click would be ambiguous.
        for pair in hits.windows(2) {
            let (x0, w0, _) = pair[0];
            let (x1, ..) = pair[1];
            assert!(x0 + w0 <= x1);
        }
    }
}
