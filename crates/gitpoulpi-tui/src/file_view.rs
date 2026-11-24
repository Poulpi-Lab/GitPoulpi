//! The File Insight overlay: one pane serving three
//! file-centric views -- **File View** (contents at a revision), **Blame**
//! (contents with a per-line attribution gutter) and **File History**
//! (the commits touching a path).
//!
//! One module rather than three because they are the same widget with a
//! different left gutter, opened from the same places, and backed by the
//! three `GitBackend` reads added together (`read_blob`, `blame`, and
//! `LogOptions::paths`). Keeping them apart would have meant three copies
//! of the scroll/selection/row arithmetic -- and `lib.rs`'s own history
//! says duplicated row arithmetic is where this project's real bugs come
//! from.
//!
//! Everything here is pure: state plus a `&FileInsight -> Vec<Line>` row
//! builder. Nothing touches the daemon or the terminal, so the rows can
//! be asserted on directly (see the tests at the bottom) the way
//! `status_pane` and `palette` are.

use gitpoulpi_core::domain::{describe_pointer, Blame, BlameContent, Blob, BlobContent, Oid};
use gitpoulpi_i18n::msg::file_view as msg;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};

use crate::daemon_link::{FileInsightKind, FileInsightPayload, FileInsightRequest};
use crate::live_settings;
use crate::text_select::{self, LineSource, RowSegs, Seg, Tabs, TextSelection};
use crate::th;

/// Width of the blame gutter: 8 sha characters, a space, then the author
/// column, then the date. Fixed rather than content-derived so the file
/// text starts at the same column on every line - a gutter that shifted
/// per row would make the code beside it unreadable.
const SHA_WIDTH: usize = 8;
const AUTHOR_WIDTH: usize = 14;
const DATE_WIDTH: usize = 10;

/// What the overlay is showing, and what it was asked for.
pub(crate) struct FileInsight {
    pub(crate) request: FileInsightRequest,
    pub(crate) data: FileInsightData,
    /// Index of the highlighted row within `rows`, and the first row
    /// drawn. Both are clamped by [`FileInsight::clamp`] whenever the data
    /// or the viewport changes, so a selection can't survive past the end
    /// of a shorter payload.
    pub(crate) selected: usize,
    pub(crate) offset: usize,
    /// First *character column* drawn of each row's text (never the fixed
    /// gutter - see `insight_rows`). Independent of `offset`/`selected`,
    /// the way a text editor's horizontal scroll is independent of which
    /// line is selected.
    pub(crate) h_offset: usize,
    /// A mouse text selection over the text column --
    /// the file's lines, Blame's code, History's summaries. Anchored in
    /// the text, so it survives scrolling; cleared when new data lands.
    pub(crate) text_sel: Option<TextSelection>,
    /// The rows (0-based, inclusive) an agent's `ui show file` / `ui
    /// highlight` tinted; painted over the selection's own band.
    pub(crate) highlight: Option<(usize, usize)>,
    /// Which answer `data` is: a fresh number at every accepted payload, so
    /// the image pane re-reads a file that was re-loaded (an edit saved).
    pub(crate) load_id: u64,
    /// The Blame view's own state (hunks, highlighting, popups), built when
    /// a blame result lands; `None` for every other kind of result.
    pub(crate) blame: Option<crate::blame_view::BlameView>,
    /// Set when the commit column re-pointed this view at another commit
    /// (`commit_column::follow`): what the view says about the file there.
    pub(crate) follow_note: Option<FollowNote>,
}

/// What the commit column found out about the open file at the commit it
/// followed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FollowNote {
    /// The commit did not touch the file: this is the file as it was there.
    Unchanged,
    /// The commit has no such file: the view says so and stays usable.
    Missing,
}

pub(crate) enum FileInsightData {
    /// The request is out; no answer yet.
    Loading,
    Content(Blob),
    Blame(Blame),
    Failed(String),
}

impl FileInsight {
    pub(crate) fn loading(request: FileInsightRequest) -> Self {
        Self {
            request,
            data: FileInsightData::Loading,
            selected: 0,
            offset: 0,
            h_offset: 0,
            text_sel: None,
            highlight: None,
            load_id: 0,
            blame: None,
            follow_note: None,
        }
    }

    /// Whether row `i` is inside the agent's highlight band.
    fn highlighted(&self, i: usize) -> bool {
        self.highlight.is_some_and(|(a, b)| (a..=b).contains(&i))
    }

    /// Accepts an answer, *if* it answers the request currently on
    /// screen. A blame of a big file can land after the user has already
    /// switched to another file or another view, and applying it then
    /// would silently replace what they are looking at with something
    /// they abandoned.
    pub(crate) fn accept(&mut self, request: &FileInsightRequest, payload: FileInsightPayload) {
        if *request != self.request {
            return;
        }
        self.data = match payload {
            FileInsightPayload::Notice(message) => FileInsightData::Failed(message),
            FileInsightPayload::Content(blob) => FileInsightData::Content(blob),
            FileInsightPayload::Blame(blame) => FileInsightData::Blame(blame),
            // Routed to the image pane and the History view before it gets here.
            FileInsightPayload::HistoryPage(_)
            | FileInsightPayload::ImageBytes(_)
            | FileInsightPayload::BinaryState(_)
            | FileInsightPayload::Recoded { .. } => return,
        };
        self.selected = 0;
        self.offset = 0;
        self.h_offset = 0;
        self.text_sel = None;
        self.highlight = None;
        self.blame = match &self.data {
            FileInsightData::Blame(Blame {
                content: BlameContent::Lines(lines),
                ..
            }) => Some(crate::blame_view::BlameView::new(&self.request.path, lines)),
            _ => None,
        };
        static LOADS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        self.load_id = LOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    pub(crate) fn fail(&mut self, request: &FileInsightRequest, message: String) {
        if *request == self.request {
            // A file the commit column followed to a commit that has none:
            // a clear sentence, whatever the backend said.
            let message = if self.follow_note.is_some() {
                self.follow_note = Some(FollowNote::Missing);
                msg::MISSING_AT_COMMIT.get().to_string()
            } else {
                message
            };
            self.data = FileInsightData::Failed(message);
        }
    }

    /// Title bar text: what is being shown, of what, at which revision.
    pub(crate) fn title(&self) -> String {
        let kind = match self.request.kind {
            FileInsightKind::Content
            | FileInsightKind::DiffContent(_)
            | FileInsightKind::DiffRecoded { .. }
            | FileInsightKind::ImageBytes(_)
            | FileInsightKind::BinaryState => msg::TITLE_CONTENT.get(),
            FileInsightKind::Blame => msg::TITLE_BLAME.get(),
            FileInsightKind::History | FileInsightKind::HistoryPage(_) => msg::TITLE_HISTORY.get(),
        };
        let at = if self.request.worktree {
            if self.request.staged {
                msg::WIP_STAGED.get()
            } else {
                msg::WIP.get()
            }
            .to_string()
        } else {
            short_sha(self.request.at)
        };
        let title = format!("{kind} - {} @ {at}", self.request.path.display());
        match self.follow_note {
            Some(FollowNote::Unchanged) => format!("{title} - {}", msg::UNCHANGED_IN_COMMIT.get()),
            _ => title,
        }
    }

    /// Number of selectable rows.
    pub(crate) fn len(&self) -> usize {
        match &self.data {
            FileInsightData::Content(blob) => blob.content.lines().len(),
            FileInsightData::Blame(blame) => blame.content.lines().len(),
            FileInsightData::Loading | FileInsightData::Failed(_) => 0,
        }
    }

    /// Width, in characters, of the longest row's *text* column (never the
    /// fixed gutter). The horizontal scroll clamps against this - there is
    /// nothing to the right of it worth showing.
    pub(crate) fn longest_line(&self) -> usize {
        match &self.data {
            FileInsightData::Content(blob) => match &blob.content {
                BlobContent::Text { lines, .. } => {
                    lines.iter().map(|l| l.chars().count()).max().unwrap_or(0)
                }
                _ => 0,
            },
            FileInsightData::Blame(blame) => match &blame.content {
                BlameContent::Lines(lines) => lines
                    .iter()
                    .map(|l| l.content.chars().count())
                    .max()
                    .unwrap_or(0),
                _ => 0,
            },
            FileInsightData::Loading | FileInsightData::Failed(_) => 0,
        }
    }

    /// The commit a row points at, for "jump to this commit in the Log".
    /// `None` for File View, whose rows are text and attribute nothing.
    pub(crate) fn commit_at(&self, row: usize) -> Option<Oid> {
        match &self.data {
            FileInsightData::Blame(blame) => blame
                .content
                .lines()
                .get(row)
                .map(|l| l.commit)
                .filter(|c| !crate::wip_view::is_uncommitted(*c)),
            _ => None,
        }
    }

    pub(crate) fn select(&mut self, row: usize, viewport: usize) {
        self.selected = row;
        self.clamp(viewport);
    }

    /// Moves the selection by `delta` rows, saturating at both ends.
    ///
    /// `isize` rather than a direction enum because the caller already has
    /// +1/-1/+page/-page. Clamping at zero is the whole subtlety: `selected - page` on
    /// a `usize` near the top would wrap to a huge value on PageUp.
    pub(crate) fn move_selection(&mut self, delta: isize, viewport: usize) {
        let len = self.len();
        if len == 0 {
            return;
        }
        let next = self.selected as isize + delta;
        self.selected = next.clamp(0, (len as isize - 1).max(0)) as usize;
        self.clamp(viewport);
    }

    /// Keeps `selected` inside the data and `offset` inside the window
    /// that shows `selected`.
    pub(crate) fn clamp(&mut self, viewport: usize) {
        let len = self.len();
        if len == 0 {
            self.selected = 0;
            self.offset = 0;
            return;
        }
        self.selected = self.selected.min(len - 1);
        let viewport = viewport.max(1);
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset + viewport {
            self.offset = self.selected + 1 - viewport;
        }
        self.offset = self.offset.min(len.saturating_sub(1));
    }

    /// Scrolls without moving the selection - the mouse wheel.
    ///
    /// `viewport` matters twice here. First, the clamp: the reachable range
    /// is `[0, len - viewport]`, not `[0, len - 1]`, since an offset past that
    /// is one screen beyond where the last line is visible at the bottom and
    /// looks "stuck" once nothing more scrolls into view.
    /// Second, the selection: `clamp` (called every frame, before render) pulls
    /// `offset` back to `selected` whenever `selected` falls outside the visible
    /// window, which would undo a wheel scroll on the next frame. Dragging `selected`
    /// back inside the new window here, as `clamp` would, keeps the scroll from
    /// fighting itself.
    pub(crate) fn scroll(&mut self, delta: isize, viewport: usize) {
        let len = self.len();
        if len == 0 {
            return;
        }
        let viewport = viewport.max(1);
        let max_offset = len.saturating_sub(viewport);
        let next = self.offset as isize + delta;
        self.offset = next.clamp(0, max_offset as isize) as usize;
        if self.selected < self.offset {
            self.selected = self.offset;
        } else if self.selected >= self.offset + viewport {
            self.selected = self.offset + viewport - 1;
        }
    }

    /// Horizontal wheel/key scroll of the text column. Independent of
    /// `offset`/`selected` and of `clamp`, and clamped to the longest
    /// line - there is nothing to scroll to past it.
    pub(crate) fn scroll_h(&mut self, delta: isize) {
        let max = self.longest_line().saturating_sub(1) as isize;
        let next = self.h_offset as isize + delta;
        self.h_offset = next.clamp(0, max.max(0)) as usize;
    }
}

/// Builds the visible rows.
///
/// `width` is the *inner* width of the overlay (borders already removed),
/// and `height` how many rows fit. Truncation happens here rather than
/// being left to ratatui so that the gutter is never the thing that gets
/// cut off: on a narrow terminal the file text is what shrinks, because a
/// blame line with no sha is useless while a blame line with no tail is
/// still readable.
pub(crate) fn insight_rows(insight: &FileInsight, width: usize, height: usize) -> Vec<Line<'_>> {
    let dim = Style::default().fg(th().ui.muted);
    let accent = Style::default().fg(th().ui.accent);

    match &insight.data {
        FileInsightData::Loading => vec![Line::styled(msg::READING.get(), dim)],
        FileInsightData::Failed(message) => vec![Line::styled(message.clone(), accent)],
        FileInsightData::Content(blob) => match &blob.content {
            BlobContent::Binary => vec![Line::styled(msg::binary_show(blob.size), dim)],
            BlobContent::TooLarge => vec![Line::styled(msg::too_large(blob.size), dim)],
            BlobContent::LfsPointer(pointer) => vec![
                Line::styled(describe_pointer(pointer), dim),
                Line::styled(msg::LFS_PULL_HINT.get(), dim),
            ],
            BlobContent::Text { lines, .. } => {
                let number_width = digits(lines.len());
                windowed(insight, lines.len(), height)
                    .map(|i| {
                        let mut spans =
                            vec![Span::styled(format!("{:>number_width$} ", i + 1), dim)];
                        spans.push(Span::raw(clip_scrolled(
                            &lines[i],
                            insight.h_offset,
                            width.saturating_sub(number_width + 1),
                        )));
                        styled_row(spans, i == insight.selected, insight.highlighted(i))
                    })
                    .collect()
            }
        },
        FileInsightData::Blame(blame) => match &blame.content {
            BlameContent::Binary => vec![Line::styled(msg::binary_blame(blame.size), dim)],
            BlameContent::TooLarge => vec![Line::styled(msg::too_large_blame(blame.size), dim)],
            BlameContent::LfsPointer(pointer) => vec![Line::styled(
                msg::nothing_to_blame(describe_pointer(pointer)),
                dim,
            )],
            BlameContent::Lines(lines) => {
                let gutter = SHA_WIDTH + 1 + AUTHOR_WIDTH + 1 + DATE_WIDTH + 1;
                windowed(insight, lines.len(), height)
                    .map(|i| {
                        let line = &lines[i];
                        let mut spans = if crate::wip_view::is_uncommitted(line.commit) {
                            // One span over the whole attribution columns,
                            // so the text still starts at the same column.
                            vec![Span::styled(
                                pad(crate::wip_view::NOT_COMMITTED.get(), gutter - 1),
                                dim,
                            )]
                        } else {
                            vec![
                                Span::styled(short_sha(line.commit), accent),
                                Span::raw(" "),
                                Span::styled(pad(&line.author_name, AUTHOR_WIDTH), dim),
                                Span::raw(" "),
                                Span::styled(ymd(line.timestamp), dim),
                            ]
                        };
                        spans.extend([
                            Span::raw(" "),
                            Span::raw(clip_scrolled(
                                &line.content,
                                insight.h_offset,
                                width.saturating_sub(gutter),
                            )),
                        ]);
                        styled_row(spans, i == insight.selected, insight.highlighted(i))
                    })
                    .collect()
            }
        },
    }
}

/// The text column of every visible row, as `text_select` segments --
/// the selection's twin of `insight_rows`, over the same window and the
/// same gutter widths. Row `i` of the result is screen row `i` of the
/// inner rect. Empty for the notices (loading, binary, too large), which
/// have no text to select.
pub(crate) fn insight_segs(insight: &FileInsight, width: usize, height: usize) -> Vec<RowSegs> {
    let gutter = match &insight.data {
        FileInsightData::Content(Blob {
            content: BlobContent::Text { lines, .. },
            ..
        }) => digits(lines.len()) + 1,
        FileInsightData::Blame(Blame {
            content: BlameContent::Lines(_),
            ..
        }) => SHA_WIDTH + 1 + AUTHOR_WIDTH + 1 + DATE_WIDTH + 1,
        _ => return Vec::new(),
    };
    let start = insight.h_offset.saturating_sub(1);
    // The Blame view lays its columns out itself (`blame_view::layout`).
    let (seg_x, seg_w) = if insight.blame_lines().is_some() {
        let (x, w) = insight.blame_text_cols(width as u16, insight.len());
        (x, w)
    } else {
        (gutter as u16, width.saturating_sub(gutter) as u16)
    };
    windowed(insight, insight.len(), height)
        .map(|i| {
            let len = insight.raw(i).map_or(0, |l| l.chars().count());
            [
                Some(Seg {
                    line: i,
                    start,
                    len: len.saturating_sub(start),
                    x: seg_x,
                    width: seg_w,
                    last: true,
                }),
                None,
            ]
        })
        .collect()
}

/// The text position under screen cell `(col, row)`, `body` being the
/// panel's outer (bordered) rect. The row is clamped into the drawn rows,
/// so a drag past the top or bottom edge keeps extending to the edge row.
pub(crate) fn text_pos_at(
    insight: &FileInsight,
    body: ratatui::layout::Rect,
    col: u16,
    row: u16,
) -> Option<text_select::TextPos> {
    let inner = body.inner(ratatui::layout::Margin::new(1, 1));
    if inner.height == 0 {
        return None;
    }
    let segs = insight_segs(insight, inner.width as usize, inner.height as usize);
    let r = row.clamp(inner.y, inner.y + inner.height - 1) - inner.y;
    text_select::pos_at(&segs, r as usize, 0, col.saturating_sub(inner.x), insight)
}

/// The selectable text of each row: the file's line, Blame's code, or
/// History's summary. Tabs are drawn as one space (`clip`).
impl LineSource for FileInsight {
    fn raw(&self, line: usize) -> Option<&str> {
        match &self.data {
            FileInsightData::Content(blob) => match &blob.content {
                BlobContent::Text { lines, .. } => lines.get(line).map(String::as_str),
                _ => None,
            },
            FileInsightData::Blame(blame) => match &blame.content {
                BlameContent::Lines(lines) => lines.get(line).map(|l| l.content.as_str()),
                _ => None,
            },
            FileInsightData::Loading | FileInsightData::Failed(_) => None,
        }
    }
    fn tabs(&self) -> Tabs {
        Tabs::OneSpace
    }
}

impl FileInsight {
    /// `y` / `Ctrl+C`: the text selection when there is one, else the
    /// selected row's text - with the words the status line names it by.
    pub(crate) fn yank(&self) -> Result<(&'static str, String), &'static str> {
        if let Some(sel) = self.text_sel {
            return Ok((
                gitpoulpi_i18n::msg::diff_view::WHAT_SELECTION.get(),
                text_select::selected_text(&sel, self),
            ));
        }
        self.copy_line()
    }

    /// The selected row's text, whole.
    pub(crate) fn copy_line(&self) -> Result<(&'static str, String), &'static str> {
        self.raw(self.selected)
            .map(|l| {
                (
                    gitpoulpi_i18n::msg::diff_view::WHAT_LINE.get(),
                    l.to_string(),
                )
            })
            .ok_or(msg::NO_TEXT_HERE.get())
    }
}

/// The indices actually on screen, given the current offset.
/// Where the file's own text sits in `inner` once the rows are drawn: right
/// of the line-number column, on the rows that exist. `None` while the view
/// shows a notice (loading, binary, too large) instead of text.
pub(crate) fn text_rect(
    insight: &FileInsight,
    inner: ratatui::layout::Rect,
) -> Option<ratatui::layout::Rect> {
    let FileInsightData::Content(blob) = &insight.data else {
        return None;
    };
    let BlobContent::Text { lines, .. } = &blob.content else {
        return None;
    };
    let gutter = (digits(lines.len()) + 1) as u16;
    let rows = windowed(insight, lines.len(), inner.height as usize).len() as u16;
    Some(ratatui::layout::Rect::new(
        inner.x + gutter.min(inner.width),
        inner.y,
        inner.width.saturating_sub(gutter),
        rows,
    ))
}

fn windowed(insight: &FileInsight, len: usize, height: usize) -> std::ops::Range<usize> {
    let start = insight.offset.min(len);
    let end = (start + height.max(1)).min(len);
    start..end
}

fn styled_row<'a>(spans: Vec<Span<'a>>, selected: bool, highlighted: bool) -> Line<'a> {
    let line = Line::from(spans);
    if highlighted {
        line.style(
            Style::default()
                .bg(th().diff.search_hit_bg)
                .add_modifier(Modifier::BOLD),
        )
    } else if selected {
        line.style(
            Style::default()
                .bg(th().ui.selection)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        line
    }
}

/// First 8 characters of an oid, the same abbreviation the Log table uses.
fn short_sha(oid: Oid) -> String {
    oid.to_hex()[..SHA_WIDTH].to_string()
}

/// The Blame/History date column, through `ui.date_format`
/// (`live_settings::date_short_pattern`; default `YYYY-MM-DD`, which is
/// what this hard-coded before the setting was wired).
///
/// Clipped and padded to `DATE_WIDTH` regardless of the pattern's own
/// length: the gutter arithmetic above (`SHA_WIDTH + 1 + DATE_WIDTH + 1
/// + ...`) is fixed, so a longer configured pattern is truncated rather
/// than shoving every column after it sideways - the same trade
/// `LogColumn::Date`'s width negotiation makes in the Log table.
fn ymd(timestamp: i64) -> String {
    pad(
        &live_settings::format_date(&live_settings::date_short_pattern(), timestamp),
        DATE_WIDTH,
    )
}

/// Truncates to `width` *characters* - never bytes, which would split a
/// multi-byte codepoint and panic.
fn clip(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    for (i, ch) in text.chars().enumerate() {
        if i >= width {
            break;
        }
        // A tab in source code would render as a variable-width jump and
        // desynchronize the gutter from the text beside it.
        out.push(if ch == '\t' { ' ' } else { ch });
    }
    out
}

/// Like `clip`, but starting at character column `h_offset` - the
/// horizontal scroll. When scrolled (`h_offset > 0`) the first column is
/// spent on `\u{ab}`, a cheap "there is more to the left" indicator; the
/// gutter that called this never scrolls, so the marker is the only sign a
/// row gives that it isn't showing column 0 anymore.
fn clip_scrolled(text: &str, h_offset: usize, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if h_offset == 0 {
        return clip(text, width);
    }
    let mut chars = text.chars();
    // Character (not byte) offset, same reasoning as `clip`.
    if chars.by_ref().nth(h_offset - 1).is_none() {
        // Scrolled past the end of this particular (shorter) line -- still
        // show the marker so the row doesn't look like an unrelated blank.
        return "\u{ab}".to_string();
    }
    let mut out = String::from('\u{ab}');
    for ch in chars.take(width - 1) {
        out.push(if ch == '\t' { ' ' } else { ch });
    }
    out
}

fn pad(text: &str, width: usize) -> String {
    let clipped = clip(text, width);
    let len = clipped.chars().count();
    format!("{clipped}{}", " ".repeat(width.saturating_sub(len)))
}

fn digits(n: usize) -> usize {
    n.max(1).to_string().len()
}

#[cfg(test)]
#[path = "file_view_tests/mod.rs"]
mod tests;
