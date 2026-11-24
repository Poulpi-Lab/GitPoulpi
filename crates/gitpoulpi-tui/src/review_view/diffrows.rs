//! The review's half of the diff pane's rows. The code rows (hunks, lines,
//! numbers, Inline or Split, hunks-only or the whole file) come from the
//! Diff view's own render (`diff_view::render_diff_annotated`); this module
//! is the [`Annotator`] that hands the review's rows - comment threads,
//! pending comments, the ✦ suggestions - to that render, to be drawn under the
//! line they sit on, and the way back from a drawn row to what it is.
//!
//! One screen line is exactly one row of the render, so scrolling, the
//! cursor and the hit test are plain row arithmetic; the rows are rebuilt when
//! the width, a toggle, the threads, the pending comments or a fold change.

use std::cell::RefCell;
use std::collections::HashSet;

use gitpoulpi_hosting::{DiffSide, PendingComment, ReviewThread};
use gitpoulpi_i18n::msg::review_ai as ai_msg;
use gitpoulpi_i18n::msg::review_view as msg;

use super::model::Item;
use crate::commit_detail::wrap_text;
use crate::diff_view::{AnnotBlock, AnnotRow, AnnotSide, AnnotTag, Annotator, DiffRow, RowChange};

// What an annotation row is (`AnnotTag::kind`).
pub(super) const K_FILE: u8 = 0;
pub(super) const K_THREAD_HEAD: u8 = 1;
pub(super) const K_THREAD_BODY: u8 = 2;
pub(super) const K_THREAD_META: u8 = 3;
pub(super) const K_PENDING_HEAD: u8 = 4;
pub(super) const K_PENDING_BODY: u8 = 5;
pub(super) const K_GHOST_HEAD: u8 = 6;
pub(super) const K_GHOST_BODY: u8 = 7;
pub(super) const K_HEADING: u8 = 8;

/// How a diff line changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Origin {
    Add,
    Delete,
    Context,
}

/// What one screen line of the diff pane is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowKind {
    /// A file's name, above its hunks (the Commits page, which shows several).
    FileHeader,
    Hunk,
    Line {
        origin: Origin,
        old: Option<u32>,
        new: Option<u32>,
    },
    /// The first comment of a thread: author, date, state and its buttons.
    ThreadHead {
        thread: usize,
    },
    /// A line of a thread's text; `meta` is the author line of a reply.
    ThreadBody {
        thread: usize,
        meta: bool,
    },
    PendingHead {
        pending: usize,
    },
    PendingBody {
        pending: usize,
    },
    /// The head of a ✦ suggestion: severity, line and its buttons.
    GhostHead {
        item: usize,
    },
    /// A line of a ✦ suggestion's text.
    GhostBody {
        item: usize,
    },
    /// "Comments that no longer match the code".
    Heading,
    /// A plain remark (no changes to show, still loading, ...).
    Note,
}

/// What the drawn row `row` is.
pub(crate) fn kind_of(row: &DiffRow) -> RowKind {
    if let Some(tag) = row.annot {
        let i = tag.index as usize;
        return match tag.kind {
            K_FILE => RowKind::FileHeader,
            K_THREAD_HEAD => RowKind::ThreadHead { thread: i },
            K_THREAD_BODY => RowKind::ThreadBody {
                thread: i,
                meta: false,
            },
            K_THREAD_META => RowKind::ThreadBody {
                thread: i,
                meta: true,
            },
            K_PENDING_HEAD => RowKind::PendingHead { pending: i },
            K_PENDING_BODY => RowKind::PendingBody { pending: i },
            K_GHOST_HEAD => RowKind::GhostHead { item: i },
            K_GHOST_BODY => RowKind::GhostBody { item: i },
            _ => RowKind::Heading,
        };
    }
    if row.header {
        return RowKind::Hunk;
    }
    let [old, new] = row.numbers;
    if old.is_none() && new.is_none() {
        return RowKind::Note;
    }
    let origin = match row.change {
        RowChange::Add | RowChange::Both => Origin::Add,
        RowChange::Del => Origin::Delete,
        RowChange::None => Origin::Context,
    };
    RowKind::Line { origin, old, new }
}

/// The diff line a comment started on `row` is about: `(side, line, other)`.
/// A Split row that pairs a deletion with an addition has two lines; `side`
/// (`0` old, `1` new) says which one the cursor is on.
pub(crate) fn code_line(row: &DiffRow, side: usize) -> Option<(DiffSide, u32, Option<u32>)> {
    if row.annot.is_some() || row.header {
        return None;
    }
    let [old, new] = row.numbers;
    let old_line = || old.map(|o| (DiffSide::Old, o, None));
    let new_line = || new.map(|n| (DiffSide::New, n, None));
    match row.change {
        RowChange::Add => new_line(),
        RowChange::Del => old_line(),
        RowChange::Both => {
            if side == 0 {
                old_line().or_else(new_line)
            } else {
                new_line().or_else(old_line)
            }
        }
        // The host wants both numbers of an unchanged line.
        RowChange::None => new.map(|n| (DiffSide::New, n, old)),
    }
}

/// Whether the row is where a thread / pending comment / suggestion begins
/// (the ticks the scrollbar's ruler shows).
pub(crate) fn is_marker(row: &DiffRow) -> bool {
    matches!(
        row.annot.map(|t| t.kind),
        Some(K_THREAD_HEAD | K_PENDING_HEAD | K_GHOST_HEAD)
    )
}

fn side_of(side: DiffSide) -> AnnotSide {
    match side {
        DiffSide::Old => AnnotSide::Old,
        DiffSide::New => AnnotSide::New,
    }
}

fn anchored(side: DiffSide, line: u32, old: Option<u32>, new: Option<u32>) -> bool {
    match side {
        DiffSide::New => new == Some(line),
        DiffSide::Old => old == Some(line),
    }
}

/// The first line of a comment, for a collapsed thread.
fn preview(body: &str) -> String {
    body.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// `author · 2026-10-11`, or just the author when the host gives no date
/// (GitLab's approvals): no dangling separator.
pub(super) fn who(author: &str, created: &str) -> String {
    let date = created.get(..10).unwrap_or(created);
    if date.is_empty() {
        author.to_string()
    } else {
        format!("{author} \u{b7} {date}")
    }
}

/// Everything the review puts into a file's diff.
#[derive(Debug)]
pub(crate) struct ReviewAnnotator<'a> {
    /// `(index in the view's list, thread)` of this file's threads.
    threads: &'a [(usize, &'a ReviewThread)],
    /// `(index in the view's list, comment)` of this file's pending comments.
    pending: &'a [(usize, &'a PendingComment)],
    /// `(index in the view's list, suggestion)` of this file's ✦ suggestions.
    ghosts: &'a [(usize, &'a Item)],
    collapsed: &'a HashSet<String>,
    placed: RefCell<Placed>,
}

#[derive(Debug, Default)]
struct Placed {
    threads: HashSet<usize>,
    pending: HashSet<usize>,
    ghosts: HashSet<usize>,
}

impl<'a> ReviewAnnotator<'a> {
    pub(crate) fn new(
        threads: &'a [(usize, &'a ReviewThread)],
        pending: &'a [(usize, &'a PendingComment)],
        ghosts: &'a [(usize, &'a Item)],
        collapsed: &'a HashSet<String>,
    ) -> Self {
        Self {
            threads,
            pending,
            ghosts,
            collapsed,
            placed: RefCell::default(),
        }
    }
}

fn row(kind: u8, index: usize, side: AnnotSide, text: impl Into<String>) -> AnnotRow {
    AnnotRow {
        tag: AnnotTag {
            kind,
            index: index as u32,
            side,
        },
        text: text.into(),
    }
}

fn thread_rows(
    index: usize,
    thread: &ReviewThread,
    collapsed: &HashSet<String>,
    side: AnnotSide,
    width: usize,
) -> Vec<AnnotRow> {
    let Some(first) = thread.comments.first() else {
        return Vec::new();
    };
    if collapsed.contains(&thread.id) {
        return vec![row(
            K_THREAD_HEAD,
            index,
            side,
            format!("{}: {}", first.author, preview(&first.body)),
        )];
    }
    let mut rows = vec![row(
        K_THREAD_HEAD,
        index,
        side,
        who(&first.author, &first.created_at),
    )];
    for (i, comment) in thread.comments.iter().enumerate() {
        if i > 0 {
            rows.push(row(
                K_THREAD_META,
                index,
                side,
                who(&comment.author, &comment.created_at),
            ));
        }
        for line in wrap_text(&comment.body, width) {
            rows.push(row(K_THREAD_BODY, index, side, line));
        }
    }
    rows
}

fn pending_rows(
    index: usize,
    comment: &PendingComment,
    side: AnnotSide,
    width: usize,
) -> Vec<AnnotRow> {
    let range = comment
        .start_line
        .filter(|s| *s != comment.line)
        .map_or_else(
            || comment.line.to_string(),
            |s| format!("{s}-{}", comment.line),
        );
    let mut rows = vec![row(K_PENDING_HEAD, index, side, msg::pending_head(&range))];
    for line in wrap_text(&comment.body, width) {
        rows.push(row(K_PENDING_BODY, index, side, line));
    }
    rows
}

fn ghost_rows(index: usize, item: &Item, side: AnnotSide, width: usize) -> Vec<AnnotRow> {
    let word = item.severity.word();
    let head = match (item.anchored, item.place) {
        (true, Some(place)) => {
            let range = place
                .start_line
                .filter(|s| *s != place.line)
                .map_or_else(|| place.line.to_string(), |s| format!("{s}-{}", place.line));
            ai_msg::ghost_head(word, &range)
        }
        _ if item.line > 0 => ai_msg::ghost_head_about(word, item.line),
        _ => word.to_string(),
    };
    let mut rows = vec![row(K_GHOST_HEAD, index, side, head)];
    for line in wrap_text(&item.body, width) {
        rows.push(row(K_GHOST_BODY, index, side, line));
    }
    rows
}

impl Annotator for ReviewAnnotator<'_> {
    fn after_line(&self, old: Option<u32>, new: Option<u32>, width: usize) -> Vec<AnnotBlock> {
        let mut placed = self.placed.borrow_mut();
        let mut blocks = Vec::new();
        for (index, thread) in self.threads {
            if placed.threads.contains(index) {
                continue;
            }
            if let Some((side, at)) = thread.anchor() {
                if anchored(side, at, old, new) {
                    placed.threads.insert(*index);
                    let side = side_of(side);
                    blocks.push(AnnotBlock {
                        side,
                        rows: thread_rows(*index, thread, self.collapsed, side, width),
                    });
                }
            }
        }
        for (index, comment) in self.pending {
            if placed.pending.contains(index) {
                continue;
            }
            if anchored(comment.side, comment.line, old, new) {
                placed.pending.insert(*index);
                let side = side_of(comment.side);
                blocks.push(AnnotBlock {
                    side,
                    rows: pending_rows(*index, comment, side, width),
                });
            }
        }
        for (index, item) in self.ghosts {
            if placed.ghosts.contains(index) || !item.anchored {
                continue;
            }
            if let Some(place) = item.place {
                if anchored(place.side, place.line, old, new) {
                    placed.ghosts.insert(*index);
                    let side = side_of(place.side);
                    blocks.push(AnnotBlock {
                        side,
                        rows: ghost_rows(*index, item, side, width),
                    });
                }
            }
        }
        blocks
    }

    fn trailing(&self, width: usize) -> Vec<AnnotRow> {
        let placed = self.placed.borrow();
        let mut rows = Vec::new();
        let loose_threads: Vec<_> = self
            .threads
            .iter()
            .filter(|(i, _)| !placed.threads.contains(i))
            .collect();
        let loose_pending: Vec<_> = self
            .pending
            .iter()
            .filter(|(i, _)| !placed.pending.contains(i))
            .collect();
        let loose_ghosts: Vec<_> = self
            .ghosts
            .iter()
            .filter(|(i, _)| !placed.ghosts.contains(i))
            .collect();
        if !loose_threads.is_empty() || !loose_pending.is_empty() {
            rows.push(row(
                K_HEADING,
                0,
                AnnotSide::Full,
                msg::OUTDATED_HEADING.get(),
            ));
            for (index, thread) in loose_threads {
                rows.extend(thread_rows(
                    *index,
                    thread,
                    self.collapsed,
                    AnnotSide::Span,
                    width,
                ));
            }
            for (index, comment) in loose_pending {
                rows.extend(pending_rows(*index, comment, AnnotSide::Span, width));
            }
        }
        if !loose_ghosts.is_empty() {
            rows.push(row(
                K_HEADING,
                1,
                AnnotSide::Full,
                ai_msg::FILE_LEVEL_HEADING.get(),
            ));
            for (index, item) in loose_ghosts {
                rows.extend(ghost_rows(*index, item, AnnotSide::Span, width));
            }
        }
        rows
    }
}

/// The row that names a file above its hunks (the Commits page stacks several).
pub(crate) fn file_header(index: usize, path: &str) -> AnnotRow {
    row(K_FILE, index, AnnotSide::Full, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff_view::{render_diff_annotated, DiffRender, DiffRenderOpts, DiffViewMode};
    use gitpoulpi_core::domain::{DiffLine, FileDiff, FileStatus, Hunk, HunkId, LineOrigin};
    use gitpoulpi_hosting::ReviewComment;

    fn line(origin: LineOrigin, text: &str) -> DiffLine {
        DiffLine {
            origin,
            content: format!("{text}\n"),
            staged: false,
        }
    }

    /// Old lines 10-12, new lines 10-13; the whole file has 14 lines.
    pub(crate) fn file() -> FileDiff {
        FileDiff {
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
        }
    }

    fn whole() -> Vec<String> {
        (1..=14)
            .map(|n| match n {
                10 => "keep".to_string(),
                11 => "added one".to_string(),
                12 => "added two".to_string(),
                13 => "tail".to_string(),
                n => format!("plain {n}"),
            })
            .collect()
    }

    fn thread(id: &str, line: Option<u32>, side: DiffSide, body: &str) -> ReviewThread {
        ReviewThread {
            id: id.into(),
            path: "src/a.rs".into(),
            line,
            original_line: Some(12),
            side,
            start_line: None,
            resolved: false,
            outdated: line.is_none(),
            resolvable: true,
            comments: vec![ReviewComment {
                id: "1".into(),
                author: "tomas".into(),
                body: body.into(),
                created_at: "2026-10-11T08:00:00Z".into(),
                in_reply_to: None,
                web_url: None,
            }],
        }
    }

    fn render(
        threads: &[ReviewThread],
        pending: &[PendingComment],
        mode: DiffViewMode,
        full: Option<&[String]>,
    ) -> DiffRender {
        let f = file();
        let t: Vec<(usize, &ReviewThread)> = threads.iter().enumerate().collect();
        let p: Vec<(usize, &PendingComment)> = pending.iter().enumerate().collect();
        let collapsed = HashSet::new();
        let annotator = ReviewAnnotator::new(&t, &p, &[], &collapsed);
        render_diff_annotated(
            &f,
            &DiffRenderOpts {
                mode,
                hunks_only: full.is_none(),
                show_whitespace: true,
                word_wrap: false,
                width: 80,
                h_offset: 0,
                full_file: full,
                lfs_raw: false,
                lfs_local: None,
                intraline: false,
            },
            Some(&annotator),
        )
    }

    fn text(r: &DiffRender, i: usize) -> String {
        r.lines[i]
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect()
    }

    fn at(r: &DiffRender, needle: &str) -> usize {
        (0..r.lines.len())
            .find(|i| text(r, *i).contains(needle))
            .unwrap_or_else(|| panic!("no row with {needle}"))
    }

    fn kinds(r: &DiffRender) -> Vec<RowKind> {
        r.rows.iter().map(kind_of).collect()
    }

    #[test]
    fn lines_are_numbered_on_both_sides_from_the_hunk_start() {
        let r = render(&[], &[], DiffViewMode::Inline, None);
        let nums: Vec<_> = kinds(&r)
            .into_iter()
            .filter_map(|k| match k {
                RowKind::Line { old, new, .. } => Some((old, new)),
                _ => None,
            })
            .collect();
        assert_eq!(
            nums,
            vec![
                (Some(10), Some(10)),
                (Some(11), None),
                (None, Some(11)),
                (None, Some(12)),
                (Some(12), Some(13)),
            ]
        );
    }

    #[test]
    fn inline_a_thread_sits_right_under_its_line_and_a_deleted_line_takes_old_side_threads() {
        let threads = [
            thread("t1", Some(12), DiffSide::New, "why a Vec?"),
            thread("t2", Some(11), DiffSide::Old, "was this needed?"),
        ];
        let r = render(&threads, &[], DiffViewMode::Inline, None);
        let k = kinds(&r);
        // New line 12 is "added two"; old line 11 is "gone".
        let two = at(&r, "added two");
        assert!(matches!(k[two + 1], RowKind::ThreadHead { thread: 0 }));
        assert!(text(&r, two + 2).contains("why a Vec?"));
        let gone = at(&r, "gone");
        assert!(matches!(k[gone + 1], RowKind::ThreadHead { thread: 1 }));
        assert!(text(&r, gone + 2).contains("was this needed?"));
    }

    #[test]
    fn split_a_thread_sits_under_the_side_it_belongs_to() {
        let threads = [
            thread("t1", Some(11), DiffSide::New, "new side"),
            thread("t2", Some(11), DiffSide::Old, "old side"),
        ];
        let r = render(&threads, &[], DiffViewMode::Split, None);
        // Old line 11 ("gone") and new line 11 ("added one") share one row.
        let pair = at(&r, "gone");
        assert!(text(&r, pair).contains("added one"));
        let k = kinds(&r);
        let old_head = (pair + 1..r.lines.len())
            .find(|i| matches!(k[*i], RowKind::ThreadHead { thread: 1 }))
            .expect("the old-side thread");
        let new_head = (pair + 1..r.lines.len())
            .find(|i| matches!(k[*i], RowKind::ThreadHead { thread: 0 }))
            .expect("the new-side thread");
        let sep = text(&r, pair)
            .chars()
            .position(|c| c == '\u{2502}')
            .unwrap();
        let col = |row: usize, needle: &str| {
            let t = text(&r, row);
            let byte = t.find(needle).unwrap();
            t[..byte].chars().count()
        };
        assert!(col(old_head + 1, "old side") < sep, "left half");
        assert!(col(new_head + 1, "new side") > sep, "right half");
        // Every annotation row keeps the separator column.
        assert_eq!(
            text(&r, old_head + 1).chars().nth(sep),
            Some('\u{2502}'),
            "{}",
            text(&r, old_head + 1)
        );
    }

    #[test]
    fn a_thread_outside_the_hunks_is_listed_in_hunks_only_and_sits_under_its_line_in_the_whole_file(
    ) {
        let threads = [thread("t1", Some(3), DiffSide::New, "far away")];
        for mode in [DiffViewMode::Inline, DiffViewMode::Split] {
            let hunks = render(&threads, &[], mode, None);
            let k = kinds(&hunks);
            assert!(
                k.contains(&RowKind::Heading),
                "{mode:?}: listed after the hunks"
            );
            let heading = k.iter().position(|k| *k == RowKind::Heading).unwrap();
            assert!(matches!(k[heading + 1], RowKind::ThreadHead { thread: 0 }));

            let content = whole();
            let full = render(&threads, &[], mode, Some(&content));
            let k = kinds(&full);
            assert!(!k.contains(&RowKind::Heading), "{mode:?}: no list");
            let line3 = at(&full, "plain 3");
            assert!(
                matches!(k[line3 + 1], RowKind::ThreadHead { thread: 0 }),
                "{mode:?}: right under line 3"
            );
        }
    }

    #[test]
    fn a_collapsed_thread_keeps_one_row_with_a_preview() {
        let f = file();
        let threads = [thread("t1", Some(12), DiffSide::New, "first line\nsecond")];
        let t: Vec<(usize, &ReviewThread)> = threads.iter().enumerate().collect();
        let collapsed: HashSet<String> = ["t1".to_string()].into();
        let annotator = ReviewAnnotator::new(&t, &[], &[], &collapsed);
        let r = render_diff_annotated(
            &f,
            &DiffRenderOpts {
                mode: DiffViewMode::Inline,
                hunks_only: true,
                show_whitespace: true,
                word_wrap: false,
                width: 80,
                h_offset: 0,
                full_file: None,
                lfs_raw: false,
                lfs_local: None,
                intraline: false,
            },
            Some(&annotator),
        );
        let head = at(&r, "tomas: first line");
        assert!(matches!(kinds(&r)[head], RowKind::ThreadHead { .. }));
        assert!(!kinds(&r)
            .iter()
            .any(|k| matches!(k, RowKind::ThreadBody { .. })));
    }

    #[test]
    fn a_pending_comment_shows_under_its_line_with_its_range() {
        let pending = [PendingComment {
            path: "src/a.rs".into(),
            previous_path: None,
            line: 12,
            side: DiffSide::New,
            start_line: Some(11),
            other_line: None,
            body: "please rename".into(),
        }];
        for mode in [DiffViewMode::Inline, DiffViewMode::Split] {
            let r = render(&[], &pending, mode, None);
            let head = kinds(&r)
                .iter()
                .position(|k| matches!(k, RowKind::PendingHead { pending: 0 }))
                .unwrap();
            assert!(text(&r, head).contains("11-12"), "{}", text(&r, head));
            assert!(text(&r, head + 1).contains("please rename"));
            assert!(text(&r, head - 1).contains("added two"), "{mode:?}");
        }
    }

    #[test]
    fn a_context_line_reports_both_numbers_for_the_host() {
        let r = render(&[], &[], DiffViewMode::Inline, None);
        let context = &r.rows[at(&r, "keep")];
        assert_eq!(
            code_line(context, 1),
            Some((DiffSide::New, 10, Some(10))),
            "the new number, and the old one as `other`"
        );
        assert_eq!(
            code_line(&r.rows[at(&r, "gone")], 1),
            Some((DiffSide::Old, 11, None))
        );
        assert_eq!(
            code_line(&r.rows[at(&r, "added one")], 1),
            Some((DiffSide::New, 11, None))
        );
    }

    #[test]
    fn a_split_row_with_both_sides_comments_on_the_side_asked_for() {
        let r = render(&[], &[], DiffViewMode::Split, None);
        let pair = &r.rows[at(&r, "gone")];
        assert_eq!(code_line(pair, 0), Some((DiffSide::Old, 11, None)));
        assert_eq!(code_line(pair, 1), Some((DiffSide::New, 11, None)));
    }

    #[test]
    fn the_markers_are_the_heads_only() {
        let threads = [thread("t1", Some(12), DiffSide::New, "a\nb\nc")];
        let r = render(&threads, &[], DiffViewMode::Inline, None);
        assert_eq!(r.rows.iter().filter(|r| is_marker(r)).count(), 1);
    }

    #[test]
    fn stacked_files_keep_their_row_map_hunk_starts_and_text_in_step() {
        let one = render(&[], &[], DiffViewMode::Split, None);
        let two = render(&[], &[], DiffViewMode::Split, None);
        let (n, hunk) = (one.lines.len(), one.hunk_starts[0]);
        let stacked = DiffRender::stack(
            vec![(file_header(0, "a.rs"), one), (file_header(1, "b.rs"), two)],
            DiffViewMode::Split,
            80,
        );
        assert_eq!(stacked.lines.len(), 2 * (n + 1));
        assert_eq!(stacked.rows.len(), stacked.lines.len());
        assert_eq!(stacked.segs.len(), stacked.lines.len());
        assert_eq!(stacked.hunk_starts, vec![1 + hunk, n + 2 + hunk]);
        assert!(matches!(kind_of(&stacked.rows[0]), RowKind::FileHeader));
        assert!(matches!(kind_of(&stacked.rows[n + 1]), RowKind::FileHeader));
        // The second file's text segments point at the second file's lines.
        let first_line = stacked.segs[n + 2 + hunk + 1][0].unwrap().line;
        assert!(stacked.text.sides[0][first_line].contains("keep"));
        assert!(first_line >= stacked.text.sides[0].len() / 2);
        assert_eq!(stacked.annot_text[&0], "a.rs");
        assert_eq!(stacked.annot_text[&(n + 1)], "b.rs");
        assert!(stacked.split_x.is_some());
    }
}
