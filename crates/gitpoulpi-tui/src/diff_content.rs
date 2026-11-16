//! The Diff view's whole-file layout: fetching the file the diff is about,
//! and caching the rendered rows between frames.
//!
//! **Why a blob read rather than "ask git for unlimited context".** Hunk
//! and line staging name a `HunkId` the *backend* assigns, and the backend
//! resolves it against a diff it fetches itself with git's default three
//! lines of context (`staging.rs` in `gitpoulpi-cli-backend`). A diff
//! fetched with more context has different, fewer hunks -- the ids would
//! stop meaning the same thing and Stage Hunk would stage the wrong one.
//! So the diff stays exactly what staging expects, and the unchanged
//! stretches between its hunks are filled from the file's **new side**,
//! which is identical to the old side there by definition:
//!
//! | open diff          | new side read             |
//! |--------------------|---------------------------|
//! | a commit's         | its blob at that commit   |
//! | Unstaged (WT/index)| the bytes on disk         |
//! | Staged (index/HEAD)| the index                 |
//!
//! All three through the existing `read_blob` (`BlobSource`), sent on the
//! File Insight channel with a kind of its own (`FileInsightKind::
//! DiffContent`) and routed back here by `DiffContent::offer`. An added,
//! untracked or deleted file needs no read: its one hunk already is the
//! whole file.
//!
//! **Staying fresh.** The content is re-read whenever the diff it pairs
//! with changes (a refresh brought a different `FileDiff`): a working-tree
//! or index change that moves the unchanged text also changes the diff.
//!
//! **Staying responsive.** A whole-file diff has as many rows as the file
//! has lines. The render is cached (`Rc<DiffRender>`) keyed on the diff,
//! the content and every display toggle, so it is rebuilt when one of
//! those changes, never per frame; the event loop only copies the visible
//! window out of it. `longest_line` is cached with the content.

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use gitpoulpi_core::domain::{BlobContent, FileDiff, FileStatus, Oid};

use crate::daemon_link::{
    DiffContentSide, FileInsightKind, FileInsightOutcome, FileInsightPayload, FileInsightRequest,
};
use crate::diff_view::{
    longest_line, render_diff_with, DiffRender, DiffRenderOpts, DiffSource, DiffViewMode,
    DiffViewState,
};

/// Which content an open diff's whole-file layout needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContentKey {
    side: DiffContentSide,
    path: PathBuf,
    /// The commit for `DiffContentSide::Commit`; zero otherwise.
    at: Oid,
}

impl ContentKey {
    fn request(&self) -> FileInsightRequest {
        FileInsightRequest {
            path: self.path.clone(),
            at: self.at,
            kind: FileInsightKind::DiffContent(self.side),
            worktree: false,
            staged: false,
        }
    }

    fn answers(&self, request: &FileInsightRequest) -> bool {
        request.kind == FileInsightKind::DiffContent(self.side)
            && request.path == self.path
            && request.at == self.at
    }
}

/// The content for the whole-file layout, or why there is none.
#[derive(Debug, Clone)]
enum Loaded {
    Loading,
    Ready(Arc<Vec<String>>),
    /// Binary, too large to show, or the read failed - the view falls
    /// back to the hunks and says so.
    Unavailable(String),
}

/// What the cached render was built from. `content` is compared by
/// pointer: a re-read that brought the same text keeps the old `Arc`
/// (see `DiffContent::offer`), so the cache survives it.
#[derive(Debug, Clone, PartialEq)]
struct RenderKey {
    mode: DiffViewMode,
    hunks_only: bool,
    show_whitespace: bool,
    word_wrap: bool,
    width: u16,
    h_offset: usize,
    content: ContentTag,
    lfs_raw: bool,
    lfs_local: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
enum ContentTag {
    None,
    Loading,
    Ready(usize),
    Unavailable(String),
}

/// The Diff view's whole-file content and render cache. One per window.
#[derive(Default)]
pub(crate) struct DiffContent {
    key: Option<ContentKey>,
    loaded: Option<Loaded>,
    /// The diff the content was last fetched for; a different one
    /// arriving triggers a re-read.
    fetched_for: Option<FileDiff>,
    /// `(diff, content pointer) -> longest line`.
    longest: Option<(FileDiff, usize, usize)>,
    render: Option<(FileDiff, RenderKey, Rc<DiffRender>)>,
    /// The new side's text from a re-read in a chosen encoding
    /// (`diff_recode`): it stands in for the blob read when set.
    forced: Option<Arc<Vec<String>>>,
}

/// The content an open diff's whole-file layout needs, or `None` when it
/// needs none: hunks-only, or a file whose one hunk is already all of it.
pub(crate) fn wanted(state: &DiffViewState, file: &FileDiff) -> Option<ContentKey> {
    if state.hunks_only
        // A chosen encoding brings the new side with it (`diff_recode`).
        || state.encoding.is_some()
        || matches!(
            file.status,
            FileStatus::Added | FileStatus::Untracked | FileStatus::Deleted
        )
    {
        return None;
    }
    let (side, at) = match state.source {
        DiffSource::Commit => (DiffContentSide::Commit, state.commit),
        // The new side of a comparison: the newer commit's blob, or the
        // bytes on disk when the working tree is the newer end.
        DiffSource::Compare => match state.compare {
            Some(
                gitpoulpi_core::domain::DiffTarget::CommitRange(_, to)
                | gitpoulpi_core::domain::DiffTarget::CommitSpan(_, to),
            ) => (DiffContentSide::Commit, to),
            _ => (DiffContentSide::WorkingTree, Oid([0u8; 20])),
        },
        DiffSource::Unstaged => (DiffContentSide::WorkingTree, Oid([0u8; 20])),
        DiffSource::Staged => (DiffContentSide::Index, Oid([0u8; 20])),
    };
    Some(ContentKey {
        side,
        path: state.path.clone(),
        at,
    })
}

/// The lines of a blob read for the whole-file layout, or why there are none
/// to show. Shared with the review's diff pane, which reads the same blobs.
pub(crate) fn blob_lines(blob: gitpoulpi_core::domain::Blob) -> Result<Arc<Vec<String>>, String> {
    match blob.content {
        BlobContent::Text { lines, .. } => Ok(Arc::new(lines)),
        BlobContent::Binary => Err(gitpoulpi_i18n::msg::diff_content::BINARY.get().into()),
        BlobContent::LfsPointer(_) => {
            Err(gitpoulpi_i18n::msg::diff_content::LFS_OBJECT.get().into())
        }
        BlobContent::TooLarge => Err(gitpoulpi_i18n::msg::diff_content::too_large(blob.size)),
    }
}

impl DiffContent {
    /// Called every frame the Diff view draws a file. Returns the read to
    /// send, if one is needed: the content this diff wants changed, or
    /// the diff itself changed under content already held.
    pub(crate) fn sync(
        &mut self,
        state: &DiffViewState,
        file: &FileDiff,
    ) -> Option<FileInsightRequest> {
        let Some(key) = wanted(state, file) else {
            self.key = None;
            self.loaded = None;
            self.fetched_for = None;
            return None;
        };
        let same_key = self.key.as_ref() == Some(&key);
        let same_diff = self.fetched_for.as_ref() == Some(file);
        if same_key && same_diff {
            return None;
        }
        if !same_key {
            // A different file: nothing held is worth showing meanwhile.
            self.loaded = Some(Loaded::Loading);
        }
        // Same file, new diff: keep showing what is held until the fresh
        // read lands, rather than flashing "loading" on every refresh.
        let request = key.request();
        self.key = Some(key);
        self.fetched_for = Some(file.clone());
        Some(request)
    }

    /// Takes the outcome if it is one of ours (a `DiffContent` read),
    /// handing anything else back for the File Insight panel. A stale
    /// answer - for a file no longer open - is swallowed, not shown.
    pub(crate) fn offer(&mut self, outcome: FileInsightOutcome) -> Option<FileInsightOutcome> {
        if !matches!(outcome.request.kind, FileInsightKind::DiffContent(_)) {
            return Some(outcome);
        }
        if !self
            .key
            .as_ref()
            .is_some_and(|k| k.answers(&outcome.request))
        {
            return None;
        }
        let next = match outcome.result {
            Ok(FileInsightPayload::Content(blob)) => match blob_lines(blob) {
                Ok(lines) => Loaded::Ready(lines),
                Err(why) => Loaded::Unavailable(why),
            },
            Ok(_) => {
                Loaded::Unavailable(gitpoulpi_i18n::msg::diff_content::UNEXPECTED.get().into())
            }
            Err(e) => Loaded::Unavailable(e.to_string()),
        };
        // The same text again keeps the same `Arc`, so the render cache
        // (keyed on the pointer) is not thrown away by a no-op re-read.
        let unchanged = matches!(
            (&self.loaded, &next),
            (Some(Loaded::Ready(old)), Loaded::Ready(new)) if old == new
        );
        if !unchanged {
            self.loaded = Some(next);
        }
        None
    }

    /// Whether what the whole-file layout needs has settled: nothing is
    /// asked for, or the read answered (with the text or with why not).
    pub(crate) fn settled(&self) -> bool {
        !matches!(self.loaded, Some(Loaded::Loading))
    }

    /// Hands over (or takes back, with `None`) the new side's text of a
    /// re-read in a chosen encoding; called every frame before `sync`.
    pub(crate) fn set_forced(&mut self, lines: Option<Arc<Vec<String>>>) {
        let same = match (&self.forced, &lines) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        if !same {
            self.forced = lines;
        }
    }

    fn content_tag(&self) -> ContentTag {
        if let Some(lines) = &self.forced {
            return ContentTag::Ready(Arc::as_ptr(lines) as usize);
        }
        match (&self.key, &self.loaded) {
            (None, _) | (_, None) => ContentTag::None,
            (Some(_), Some(Loaded::Loading)) => ContentTag::Loading,
            (Some(_), Some(Loaded::Ready(c))) => ContentTag::Ready(Arc::as_ptr(c) as usize),
            (Some(_), Some(Loaded::Unavailable(why))) => ContentTag::Unavailable(why.clone()),
        }
    }

    fn ready(&self) -> Option<&[String]> {
        if let Some(lines) = &self.forced {
            return Some(lines.as_slice());
        }
        match (&self.key, &self.loaded) {
            (Some(_), Some(Loaded::Ready(c))) => Some(c.as_slice()),
            _ => None,
        }
    }

    /// The longest line of what is drawn, cached with the content. It is the
    /// upper bound of `DiffViewState::scroll_h` and the content length of the
    /// diff's horizontal scrollbar (`scrollbar::Track::horizontal`, in
    /// `event_loop.rs`), which runs outside `terminal.draw`'s closure where
    /// this cache does not survive, so the render pass publishes the answer
    /// into `diff_longest_line`.
    pub(crate) fn longest(&mut self, file: &FileDiff) -> usize {
        let ptr = match self.content_tag() {
            ContentTag::Ready(p) => p,
            _ => 0,
        };
        if let Some((f, p, n)) = &self.longest {
            if *p == ptr && f == file {
                return *n;
            }
        }
        let n = longest_line(file, self.ready());
        self.longest = Some((file.clone(), ptr, n));
        n
    }

    /// Clamps the view's horizontal scroll against what is drawn, then
    /// returns the rendered rows - from the cache when nothing they
    /// depend on changed.
    ///
    /// Clamped *before* rendering: the render consumes `h_offset`, so
    /// clamping afterwards would hand it one frame of a stale, too-large
    /// offset every time the file changes.
    pub(crate) fn render(
        &mut self,
        file: &FileDiff,
        state: &mut DiffViewState,
        width: u16,
    ) -> Rc<DiffRender> {
        if state.word_wrap {
            state.h_offset = 0;
        } else {
            let max_h = crate::diff_view::h_scroll_max(self.longest(file), state.mode, width);
            state.h_offset = state.h_offset.min(max_h);
        }
        let key = RenderKey {
            mode: state.mode,
            hunks_only: state.hunks_only,
            show_whitespace: state.show_whitespace,
            word_wrap: state.word_wrap,
            width,
            h_offset: state.h_offset,
            content: self.content_tag(),
            lfs_raw: state.lfs_raw,
            lfs_local: state.lfs_local,
        };
        if let Some((f, k, r)) = &self.render {
            if *k == key && f == file {
                return Rc::clone(r);
            }
        }
        let opts = DiffRenderOpts::for_state(state, width, self.ready());
        // A Git LFS file's card needs neither the whole file nor a notice
        // about it.
        let lfs_card = !state.lfs_raw && gitpoulpi_core::domain::lfs_change(file).is_some();
        let render = match &key.content {
            _ if lfs_card => render_diff_with(file, &opts),
            ContentTag::Loading => {
                DiffRender::notice(gitpoulpi_i18n::msg::diff_content::LOADING_WHOLE.get())
            }
            ContentTag::Unavailable(why) => render_diff_with(file, &opts)
                .with_notice_on_top(gitpoulpi_i18n::msg::diff_content::whole_unavailable(why)),
            ContentTag::None | ContentTag::Ready(_) => render_diff_with(file, &opts),
        };
        let render = Rc::new(render);
        self.render = Some((file.clone(), key, Rc::clone(&render)));
        render
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gitpoulpi_core::domain::{Blob, BlobSource, DiffLine, Hunk, HunkId, LineOrigin};
    use ratatui::text::Line;

    fn line(origin: LineOrigin, content: &str) -> DiffLine {
        DiffLine {
            origin,
            content: content.into(),
            staged: false,
        }
    }

    /// A 20-line file `l1..l20`; the diff changes line 3 and line 15 (two
    /// hunks, git's three lines of context, the first clipped at the top),
    /// leaving 7..=11 between them and 19..=20 after.
    fn content() -> Vec<String> {
        (1..=20).map(|n| format!("l{n}")).collect()
    }

    fn two_hunks() -> FileDiff {
        let h0 = Hunk {
            id: HunkId(0),
            old_start: 1,
            old_lines: 6,
            new_start: 1,
            new_lines: 6,
            lines: vec![
                line(LineOrigin::Context, "l1"),
                line(LineOrigin::Context, "l2"),
                line(LineOrigin::Delete, "old3"),
                line(LineOrigin::Add, "l3"),
                line(LineOrigin::Context, "l4"),
                line(LineOrigin::Context, "l5"),
                line(LineOrigin::Context, "l6"),
            ],
        };
        let h1 = Hunk {
            id: HunkId(1),
            old_start: 12,
            old_lines: 7,
            new_start: 12,
            new_lines: 7,
            lines: vec![
                line(LineOrigin::Context, "l12"),
                line(LineOrigin::Context, "l13"),
                line(LineOrigin::Context, "l14"),
                line(LineOrigin::Delete, "old15"),
                line(LineOrigin::Add, "l15"),
                line(LineOrigin::Context, "l16"),
                line(LineOrigin::Context, "l17"),
                line(LineOrigin::Context, "l18"),
            ],
        };
        FileDiff {
            renamed_from: None,
            similarity: None,
            path: PathBuf::from("f.txt"),
            status: FileStatus::Modified,
            hunks: vec![h0, h1],
        }
    }

    /// A 30-line file with one change at line 15: the whole file has a
    /// gap of 11 unchanged lines on each side of the hunk.
    fn long_content() -> Vec<String> {
        (1..=30).map(|n| format!("line {n}")).collect()
    }

    fn one_hunk_in_the_middle() -> FileDiff {
        FileDiff {
            renamed_from: None,
            similarity: None,
            path: PathBuf::from("f.txt"),
            status: FileStatus::Modified,
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 12,
                old_lines: 7,
                new_start: 12,
                new_lines: 7,
                lines: vec![
                    line(LineOrigin::Context, "line 12"),
                    line(LineOrigin::Context, "line 13"),
                    line(LineOrigin::Context, "line 14"),
                    line(LineOrigin::Delete, "old 15"),
                    line(LineOrigin::Add, "line 15"),
                    line(LineOrigin::Context, "line 16"),
                    line(LineOrigin::Context, "line 17"),
                    line(LineOrigin::Context, "line 18"),
                ],
            }],
        }
    }

    fn state(source: DiffSource, mode: DiffViewMode, hunks_only: bool) -> DiffViewState {
        let mut s = match source {
            DiffSource::Commit => DiffViewState::for_commit(Oid([3; 20]), 0, "f.txt".into()),
            other => DiffViewState::for_working(other, "f.txt".into()),
        };
        s.mode = mode;
        s.hunks_only = hunks_only;
        s
    }

    fn blob(lines: Vec<String>) -> FileInsightPayload {
        FileInsightPayload::Content(Blob {
            path: "f.txt".into(),
            source: BlobSource::WorkingTree,
            size: 0,
            image: None,
            content: BlobContent::Text {
                lines,
                crlf: false,
                final_newline: true,
            },
        })
    }

    /// Opens `file` in `s`, answers the read with `lines`, and renders.
    fn loaded(
        s: &mut DiffViewState,
        file: &FileDiff,
        lines: Vec<String>,
        width: u16,
    ) -> (DiffContent, Rc<DiffRender>) {
        let mut c = DiffContent::default();
        let request = c.sync(s, file).expect("the whole file is asked for");
        assert!(c
            .offer(FileInsightOutcome {
                request,
                result: Ok(blob(lines)),
            })
            .is_none());
        let r = c.render(file, s, width);
        (c, r)
    }

    fn plain(lines: &[Line<'_>]) -> Vec<String> {
        lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect()
    }

    /// Draws the rendered rows into a `TestBackend`, the way the event
    /// loop's `Paragraph` does, and returns the screen text per row.
    fn draw(render: &DiffRender, width: u16) -> Vec<String> {
        use ratatui::backend::TestBackend;
        use ratatui::widgets::Paragraph;
        use ratatui::Terminal;
        let height = render.lines.len() as u16;
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(Paragraph::new(render.lines.clone()), frame.area());
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        (0..height)
            .map(|y| (0..width).map(|x| buf[(x, y)].symbol()).collect::<String>())
            .collect()
    }

    // ---- which content, and when it is read ----------------------------

    #[test]
    fn each_diff_source_reads_its_own_new_side() {
        let file = two_hunks();
        let side = |source| match wanted(&state(source, DiffViewMode::Inline, false), &file)
            .unwrap()
            .request()
            .kind
        {
            FileInsightKind::DiffContent(side) => side,
            other => panic!("{other:?}"),
        };
        assert_eq!(side(DiffSource::Commit), DiffContentSide::Commit);
        assert_eq!(side(DiffSource::Unstaged), DiffContentSide::WorkingTree);
        assert_eq!(side(DiffSource::Staged), DiffContentSide::Index);
        let commit = wanted(
            &state(DiffSource::Commit, DiffViewMode::Inline, false),
            &file,
        );
        assert_eq!(commit.unwrap().at, Oid([3; 20]), "the commit's own blob");
    }

    #[test]
    fn nothing_is_read_for_hunks_only_or_for_a_file_that_is_all_one_hunk() {
        let file = two_hunks();
        assert!(wanted(
            &state(DiffSource::Unstaged, DiffViewMode::Inline, true),
            &file
        )
        .is_none());
        for status in [
            FileStatus::Added,
            FileStatus::Untracked,
            FileStatus::Deleted,
        ] {
            let f = FileDiff {
                renamed_from: None,
                similarity: None,
                status,
                ..two_hunks()
            };
            let s = state(DiffSource::Unstaged, DiffViewMode::Inline, false);
            assert!(wanted(&s, &f).is_none(), "{status:?}");
        }
    }

    /// Read once per (file, diff): not every frame, and again when a
    /// refresh brings a different diff (the file changed under it).
    #[test]
    fn the_content_is_read_once_and_again_only_when_the_diff_changes() {
        let s = state(DiffSource::Unstaged, DiffViewMode::Inline, false);
        let file = two_hunks();
        let mut c = DiffContent::default();
        assert!(c.sync(&s, &file).is_some());
        assert!(c.sync(&s, &file).is_none(), "not per frame");
        let mut changed = two_hunks();
        changed.hunks.pop();
        assert!(c.sync(&s, &changed).is_some(), "a new diff re-reads");
        assert!(c.sync(&s, &changed).is_none());
    }

    #[test]
    fn a_file_view_answer_is_handed_back_and_a_stale_diff_answer_is_dropped() {
        let s = state(DiffSource::Unstaged, DiffViewMode::Inline, false);
        let file = two_hunks();
        let mut c = DiffContent::default();
        c.sync(&s, &file);
        let insight = FileInsightOutcome {
            request: FileInsightRequest {
                path: "f.txt".into(),
                at: Oid([0; 20]),
                kind: FileInsightKind::Content,
                worktree: true,
                staged: false,
            },
            result: Ok(blob(content())),
        };
        assert!(c.offer(insight).is_some(), "the File View's own read");
        let stale = FileInsightOutcome {
            request: FileInsightRequest {
                path: "other.txt".into(),
                at: Oid([0; 20]),
                kind: FileInsightKind::DiffContent(DiffContentSide::WorkingTree),
                worktree: false,
                staged: false,
            },
            result: Ok(blob(content())),
        };
        assert!(c.offer(stale).is_none(), "swallowed");
        assert!(c.ready().is_none(), "and not shown");
    }

    // ---- the four layouts, rendered ------------------------------------

    /// Inline, hunks only: the two hunks and nothing between them - the
    /// pre-item-30 Inline view, unchanged.
    #[test]
    fn inline_hunks_only_draws_just_the_hunks() {
        let mut s = state(DiffSource::Unstaged, DiffViewMode::Inline, true);
        let file = one_hunk_in_the_middle();
        let mut c = DiffContent::default();
        assert!(c.sync(&s, &file).is_none(), "hunks only reads nothing");
        let r = c.render(&file, &mut s, 60);
        let screen = draw(&r, 60);
        assert_eq!(screen.len(), 1 + 8, "header + the hunk's 8 lines");
        assert!(screen[0].starts_with("@@ -12,7 +12,7 @@"));
        assert!(!screen.iter().any(|l| l.contains("line 1 ")));
        assert!(!screen.iter().any(|l| l.contains("line 30")));
    }

    /// Inline, whole file: every line of the file, in order, numbered on both
    /// sides, with the hunk (and its header) in place. Red with
    /// `full_file: None` (the gaps vanish: 9 rows).
    #[test]
    fn inline_whole_file_draws_every_line_with_the_hunk_in_place() {
        let mut s = state(DiffSource::Unstaged, DiffViewMode::Inline, false);
        let file = one_hunk_in_the_middle();
        let (_, r) = loaded(&mut s, &file, long_content(), 60);
        let screen = draw(&r, 60);
        // 11 lines before, header, 8 hunk lines, 12 after = 32 rows.
        assert_eq!(screen.len(), 11 + 1 + 8 + 12, "{screen:#?}");
        assert!(
            screen[0].trim_start().starts_with("1     1  line 1"),
            "{:?}",
            screen[0]
        );
        assert!(screen[10].contains("line 11"));
        assert!(screen[11].starts_with("@@ -12,7 +12,7 @@"));
        assert_eq!(r.hunk_starts, vec![11]);
        let deleted = screen.iter().position(|l| l.contains("old 15")).unwrap();
        assert!(screen[deleted].contains('-'));
        assert!(screen.last().unwrap().contains("line 30"));
        // Both numbers on an unchanged row after the hunk: 19 on each side.
        let after = screen.iter().find(|l| l.contains("line 19")).unwrap();
        assert!(after.contains("   19    19 "), "{after:?}");
        // The unchanged rows belong to no hunk.
        assert_eq!(r.rows[0].hunk, None);
        assert_eq!(r.rows[31].hunk, None);
        assert_eq!(r.rows[12].hunk, Some(HunkId(0)));
    }

    /// Split, hunks only: two columns, only the hunk.
    #[test]
    fn split_hunks_only_draws_the_hunk_side_by_side() {
        let mut s = state(DiffSource::Staged, DiffViewMode::Split, true);
        let file = one_hunk_in_the_middle();
        let mut c = DiffContent::default();
        let r = c.render(&file, &mut s, 80);
        let screen = draw(&r, 80);
        // header, 3 context, the paired change, 3 context = 8 rows.
        assert_eq!(screen.len(), 8, "{screen:#?}");
        let pair = screen.iter().find(|l| l.contains("old 15")).unwrap();
        assert!(
            pair.contains("line 15"),
            "delete and add on one row: {pair:?}"
        );
        assert!(pair.contains('\u{2502}'));
    }

    /// Split, whole file: every line on both sides, the change paired in
    /// place.
    #[test]
    fn split_whole_file_draws_every_line_on_both_sides() {
        let mut s = state(DiffSource::Staged, DiffViewMode::Split, false);
        let file = one_hunk_in_the_middle();
        let (_, r) = loaded(&mut s, &file, long_content(), 80);
        let screen = draw(&r, 80);
        assert_eq!(screen.len(), 11 + 1 + 7 + 12, "{screen:#?}");
        let first = &screen[0];
        assert_eq!(first.matches("line 1 ").count(), 2, "both sides: {first:?}");
        let last = screen.last().unwrap();
        assert_eq!(last.matches("line 30").count(), 2, "both sides: {last:?}");
        let pair = screen.iter().find(|l| l.contains("old 15")).unwrap();
        assert!(pair.contains("line 15"));
    }

    /// A row's text without its gutter and `+`/`-` marker.
    fn last_word(row: &str) -> &str {
        row.split_whitespace()
            .last()
            .unwrap_or("")
            .trim_start_matches(['+', '-'])
    }

    /// Two hunks: the gaps (between them and after the last) are exactly
    /// the lines neither covers - every line of the new file once, in
    /// order, each on its own new-side number.
    #[test]
    fn the_gap_between_two_hunks_is_exactly_the_uncovered_lines() {
        let mut s = state(DiffSource::Commit, DiffViewMode::Inline, false);
        let file = two_hunks();
        let (_, r) = loaded(&mut s, &file, content(), 60);
        let text = plain(&r.lines);
        let new_side: Vec<(u32, String)> = text
            .iter()
            .filter(|l| !l.starts_with("@@") && !l.contains("old"))
            .map(|l| {
                let new: u32 = l[6..11].trim().parse().expect("a new-side number");
                (new, last_word(l).to_string())
            })
            .collect();
        let expected: Vec<(u32, String)> = content()
            .into_iter()
            .enumerate()
            .map(|(i, l)| (i as u32 + 1, l))
            .collect();
        assert_eq!(new_side, expected, "{text:#?}");
        assert_eq!(r.hunk_starts.len(), 2);
    }

    /// A pure deletion (`+N,0`) and a pure addition (`-N,0`) put the gap
    /// on the right side of git's "after line N" spelling.
    #[test]
    fn pure_additions_and_deletions_line_the_gaps_up() {
        // New file: a b X c d; X added after line 2 (`-2,0 +3,1`, no context).
        let add = FileDiff {
            renamed_from: None,
            similarity: None,
            path: "f.txt".into(),
            status: FileStatus::Modified,
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 2,
                old_lines: 0,
                new_start: 3,
                new_lines: 1,
                lines: vec![line(LineOrigin::Add, "X")],
            }],
        };
        let mut s = state(DiffSource::Unstaged, DiffViewMode::Inline, false);
        let (_, r) = loaded(
            &mut s,
            &add,
            vec!["a".into(), "b".into(), "X".into(), "c".into(), "d".into()],
            40,
        );
        let text = plain(&r.lines);
        let last_words: Vec<&str> = text.iter().map(|l| last_word(l)).collect();
        assert_eq!(last_words, ["a", "b", "@@", "X", "c", "d"], "{text:#?}");
        assert!(
            text[4].contains("    3     4 "),
            "old 3 / new 4 after the add: {:?}",
            text[4]
        );

        // New file: a b d; line c deleted after line 2 (`-3,1 +2,0`).
        let del = FileDiff {
            renamed_from: None,
            similarity: None,
            path: "f.txt".into(),
            status: FileStatus::Modified,
            hunks: vec![Hunk {
                id: HunkId(0),
                old_start: 3,
                old_lines: 1,
                new_start: 2,
                new_lines: 0,
                lines: vec![line(LineOrigin::Delete, "c")],
            }],
        };
        let (_, r) = loaded(&mut s, &del, vec!["a".into(), "b".into(), "d".into()], 40);
        let text = plain(&r.lines);
        let last_words: Vec<&str> = text.iter().map(|l| last_word(l)).collect();
        assert_eq!(last_words, ["a", "b", "@@", "c", "d"], "{text:#?}");
        assert!(
            text[4].contains("    4     3 "),
            "old 4 / new 3 after the delete: {:?}",
            text[4]
        );
    }

    // ---- loading, unavailable, and the cache ---------------------------

    #[test]
    fn while_loading_the_view_says_so_and_a_binary_file_falls_back_to_the_hunks() {
        let mut s = state(DiffSource::Unstaged, DiffViewMode::Inline, false);
        let file = one_hunk_in_the_middle();
        let mut c = DiffContent::default();
        let request = c.sync(&s, &file).unwrap();
        let r = c.render(&file, &mut s, 60);
        assert!(plain(&r.lines)[0].contains("loading the whole file"));
        assert!(r.hunk_starts.is_empty());

        c.offer(FileInsightOutcome {
            request,
            result: Ok(FileInsightPayload::Content(Blob {
                path: "f.txt".into(),
                source: BlobSource::WorkingTree,
                size: 9,
                image: None,
                content: BlobContent::Binary,
            })),
        });
        let r = c.render(&file, &mut s, 60);
        let text = plain(&r.lines);
        assert!(
            text[0].contains("whole file unavailable: binary"),
            "{text:#?}"
        );
        assert_eq!(r.hunk_starts, vec![1], "the hunk, shifted under the notice");
        assert!(r.rows[1].header);
        assert_eq!(r.lines.len(), r.rows.len());
    }

    /// Rendered once per change, not per frame: the same inputs hand back
    /// the very same `Rc`; a toggle or a new diff rebuilds it.
    #[test]
    fn the_render_is_cached_until_something_it_depends_on_changes() {
        let mut s = state(DiffSource::Unstaged, DiffViewMode::Inline, false);
        let file = one_hunk_in_the_middle();
        let (mut c, first) = loaded(&mut s, &file, long_content(), 60);
        let again = c.render(&file, &mut s, 60);
        assert!(Rc::ptr_eq(&first, &again), "same inputs, same render");
        s.mode = DiffViewMode::Split;
        let split = c.render(&file, &mut s, 60);
        assert!(!Rc::ptr_eq(&first, &split));
        let wider = c.render(&file, &mut s, 90);
        assert!(!Rc::ptr_eq(&split, &wider));
        // A re-read (a refresh brought a new diff) with the same text
        // keeps the same content, so the render is not rebuilt for it.
        let mut changed = one_hunk_in_the_middle();
        changed.hunks[0].lines[3].content = "older 15".into();
        let request = c.sync(&s, &changed).expect("a new diff re-reads");
        c.offer(FileInsightOutcome {
            request,
            result: Ok(blob(long_content())),
        });
        let before = c.render(&file, &mut s, 90);
        assert!(Rc::ptr_eq(&wider, &before), "same text, same render");
    }

    /// A large file stays cheap: the whole-file render of 50 000 lines is one
    /// linear walk (built once; the event loop copies only the window it
    /// draws). Guards against an accidental per-row search.
    /// A file read in a chosen encoding brings its own new side: the blob
    /// read is not asked for, and the whole-file layout is drawn from that
    /// text (`diff_recode`).
    #[test]
    fn a_chosen_encodings_text_stands_in_for_the_blob_read() {
        let file = one_hunk_in_the_middle();
        let mut state = DiffViewState::for_commit(Oid([1u8; 20]), 0, PathBuf::from("f.txt"));
        state.hunks_only = false;
        let mut content = DiffContent::default();
        assert!(content.sync(&state, &file).is_some(), "the blob is read");
        state.encoding = Some(crate::editor_charset::Charset::Windows1252);
        assert!(wanted(&state, &file).is_none());
        assert!(content.sync(&state, &file).is_none(), "no read of its own");
        let text: Vec<String> = (1..=30).map(|n| format!("r\u{e9}f {n}")).collect();
        content.set_forced(Some(Arc::new(text)));
        assert!(content.settled());
        let render = content.render(&file, &mut state, 80);
        let shown: String = render
            .lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();
        assert!(
            shown.contains("r\u{e9}f 1") && shown.contains("r\u{e9}f 30"),
            "the whole file from the re-read text"
        );
        // Gone with the choice: the blob is wanted again.
        content.set_forced(None);
        state.encoding = None;
        assert!(wanted(&state, &file).is_some());
    }

    #[test]
    fn a_large_file_renders_in_linear_time() {
        let n = 50_000u32;
        let big: Vec<String> = (1..=n).map(|i| format!("line {i}")).collect();
        let hunks: Vec<Hunk> = (0..500)
            .map(|k| {
                let at = 50 + k * 100;
                Hunk {
                    id: HunkId(k),
                    old_start: at,
                    old_lines: 1,
                    new_start: at,
                    new_lines: 1,
                    lines: vec![
                        line(LineOrigin::Delete, "old"),
                        line(LineOrigin::Add, &format!("line {at}")),
                    ],
                }
            })
            .collect();
        let file = FileDiff {
            renamed_from: None,
            similarity: None,
            path: "f.txt".into(),
            status: FileStatus::Modified,
            hunks,
        };
        let mut s = state(DiffSource::Commit, DiffViewMode::Inline, false);
        let started = std::time::Instant::now();
        let (_, r) = loaded(&mut s, &file, big, 100);
        let took = started.elapsed();
        assert_eq!(r.hunk_starts.len(), 500);
        // n unchanged-or-changed lines + 500 headers + 500 extra deletes.
        assert_eq!(r.lines.len(), n as usize + 1000);
        assert!(
            took < std::time::Duration::from_secs(5),
            "whole-file render of {n} lines took {took:?}"
        );
    }
}
