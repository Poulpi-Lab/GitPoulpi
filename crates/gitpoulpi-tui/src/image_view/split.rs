//! The Diff view on a picture: before and after (plan
//! the image-view design, slice S5).
//!
//! Same shape as the File View's pane (`pane.rs`): one thread-local state the
//! draw, the key and mouse handlers and the palette all reach, nothing heavy
//! on the input thread (maintainer rule 9), and the hit rects come from the paint
//! (rule 6). The differences:
//!
//! * **Two pictures.** A [`Slot`] per side holds one picture's pipeline
//!   (bytes, decode, raster, the pixel-protocol image). The before side is
//!   read from the parent commit, `HEAD` or the index, the after side from
//!   the commit, the index or the disk (`DiffContentSide`); an added file has
//!   no before and a deleted one no after.
//! * **Split or inline.** `ui.diff_layout` and `m` pick it (the Diff view's
//!   own `DiffViewMode`); `Tab` flips the one pane of an inline view between
//!   the sides, keeping zoom and center. A split too narrow for two panes is
//!   drawn inline without touching the setting (`layout.rs`).
//! * **A resizable divider** (`pane_split.rs`, shared with the merge editor):
//!   a drag, `Alt+Right` / `Alt+Left` / `Alt+0`, a double click to reset.
//! * **One viewport for both panes** when `ui.image_view_sync` is on (always
//!   in an inline view): zoom and center are kept relative to the after
//!   picture, and each pane shows them through its own image size, so two
//!   pictures of different sizes cover the same part of their panes.
//!
//! Re-adaptation (plan D4) is the viewport's: Fit stays Fit across a resize,
//! a mode switch or a divider move, a zoomed view keeps its scale and center,
//! and the scrollbars are recomputed from the geometry every frame.
//!
//! Nothing here mutates the repository, so there is no Undo story: viewing a
//! diff is not a mutation.

pub(crate) mod thumbs;

use std::cell::RefCell;
use std::sync::Arc;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEventKind};
use gitpoulpi_core::domain::{
    sniff_image, BlobBody, BlobBytes, FileDiff, FileStatus, ImageInfo, Oid,
};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::layout::{self, Arrangement, Layout, PaneBox, Side};
use super::pane::{
    center_lines, decode, describe, has_image_extension, place, rasterise, rgb, view_key,
    DecodeJob, DecodeOut, Failure, ImageHit, Info, Phase, RasterJob, RasterOut, Shown, ZoomOp,
    SPINNER_AFTER,
};
use super::raster::{self, Checker};
use super::viewport::{Size, Track, Viewport, Zoom, ZOOM_STEP};
use super::worker::Worker;
use super::{halfblock, header};
use crate::avatar::DetailAvatarImage;
use crate::chrome::HitTarget;
use crate::daemon_link::{
    DiffContentSide, FileInsightKind, FileInsightOutcome, FileInsightPayload, FileInsightRequest,
};
use crate::diff_view::{DiffBarButton, DiffSource, DiffViewMode, DiffViewState};
use crate::terminal_image::{Bitmap, Capability};
use crate::ui_kit::{display_width, ClippedRender};
use crate::{live_settings, pane_split, scrollbar, th};
use gitpoulpi_i18n::msg::image_view as msg;

/// A press on the divider within this window of the previous one resets the
/// split.
const DOUBLE_CLICK: std::time::Duration = crate::pane_layout::DOUBLE_CLICK_WINDOW;

/// Which file of which diff the state is about.
#[derive(Clone, PartialEq)]
struct Key {
    source: DiffSource,
    commit: Oid,
    path: std::path::PathBuf,
    status: FileStatus,
}

/// The geometry of a picture's last frame, for keys and the mouse.
#[derive(Clone, Copy)]
struct Geo {
    picture: Rect,
    pane_px: Size,
    img: Size,
    /// Pane pixels per cell, horizontally and vertically.
    cell: (f32, f32),
}

#[derive(Clone, Copy, Default)]
struct Bars {
    v: Option<(scrollbar::Track, Track)>,
    h: Option<(scrollbar::Track, Track)>,
}

#[derive(Clone, Copy)]
enum Drag {
    Pan {
        side: Side,
        last: (u16, u16),
    },
    Thumb {
        side: Side,
        vertical: bool,
        grab: i32,
    },
    Divider,
}

/// One side's picture pipeline.
struct Slot {
    /// The file has this side at all (an added file has no before).
    present: bool,
    request: Option<FileInsightRequest>,
    phase: Phase,
    /// The bytes are not a picture: when every side is not, the Diff view
    /// shows today's notice.
    not_image: bool,
    info: Option<Info>,
    /// The exact bytes, kept for "Edit in" (a read-only copy for a side
    /// that is not the working-tree file).
    raw: Option<Arc<Vec<u8>>>,
    gen: u64,
    decoded: Option<(u64, Arc<Bitmap>)>,
    decoder: Option<Worker<DecodeJob, DecodeOut>>,
    rasterer: Option<Worker<RasterJob, RasterOut>>,
    requested: Option<u64>,
    shown: Option<Shown>,
    image: Option<DetailAvatarImage>,
    geo: Option<Geo>,
    bars: Bars,
}

impl Slot {
    const fn new() -> Slot {
        Slot {
            present: false,
            request: None,
            phase: Phase::Card(Vec::new()),
            not_image: false,
            info: None,
            raw: None,
            gen: 0,
            decoded: None,
            decoder: None,
            rasterer: None,
            requested: None,
            shown: None,
            image: None,
            geo: None,
            bars: Bars { v: None, h: None },
        }
    }

    /// Forgets the picture (keeps the workers and the pixel image, which
    /// are reused for the next file).
    fn clear(&mut self) {
        self.present = false;
        self.request = None;
        self.phase = Phase::Card(Vec::new());
        self.not_image = false;
        self.info = None;
        self.raw = None;
        self.decoded = None;
        self.requested = None;
        self.shown = None;
        self.geo = None;
        self.bars = Bars::default();
        // A running decode / raster is superseded by the next submit; the
        // generation makes a late answer harmless.
        self.gen += 1;
    }

    fn pending(&self) -> bool {
        self.present && matches!(self.phase, Phase::Fetching { .. } | Phase::Decoding { .. })
    }

    fn ready(&self) -> bool {
        self.present && matches!(self.phase, Phase::Ready) && self.decoded.is_some()
    }

    fn img_size(&self) -> Option<Size> {
        self.decoded
            .as_ref()
            .map(|(_, b)| Size::px(b.width, b.height))
    }

    fn bytes_arrived(&mut self, bytes: BlobBytes) {
        let (data, lfs) = match bytes.body {
            BlobBody::Bytes(b) => (b, false),
            BlobBody::Lfs {
                object: Some(b), ..
            } => (b, true),
            BlobBody::TooLarge => {
                self.phase = Phase::Card(vec![msg::over_cap(gitpoulpi_core::domain::human_size(
                    bytes.size,
                ))]);
                return;
            }
            BlobBody::Lfs { object: None, .. } => {
                self.info = Some(Info {
                    format: None,
                    width: 0,
                    height: 0,
                    size: bytes.size,
                    lfs: true,
                    frames: None,
                });
                self.phase = Phase::Card(vec![msg::LFS_NOT_LOCAL.get().to_string()]);
                return;
            }
        };
        let Some(ImageInfo {
            format,
            width,
            height,
            frames,
            ..
        }) = sniff_image(&data)
        else {
            self.not_image = true;
            self.phase = Phase::Card(Vec::new());
            return;
        };
        self.info = Some(Info {
            format: Some(format),
            width,
            height,
            size: data.len() as u64,
            lfs,
            frames,
        });
        self.raw = Some(Arc::new(data.clone()));
        self.gen += 1;
        let gen = self.gen;
        self.phase = Phase::Decoding {
            since: Instant::now(),
        };
        let decoder = self
            .decoder
            .get_or_insert_with(|| Worker::spawn("image-decode", decode, crate::redraw::poke));
        decoder.submit(DecodeJob { gen, bytes: data });
    }

    fn poll_decode(&mut self) {
        let Some(done) = self.decoder.as_ref().and_then(Worker::try_recv) else {
            return;
        };
        match done.value {
            DecodeOut::Ok { gen, bitmap } if gen == self.gen => {
                self.decoded = Some((gen, Arc::new(bitmap)));
                self.phase = Phase::Ready;
            }
            DecodeOut::Err { gen, failure } if gen == self.gen => {
                self.phase = Phase::Card(vec![match failure {
                    Failure::TooLarge { width, height } => msg::too_large_pixels(width, height),
                    Failure::Corrupt => msg::CORRUPT.get().to_string(),
                    Failure::NoPreview => msg::NO_PREVIEW.get().to_string(),
                }]);
            }
            _ => {}
        }
    }

    fn poll_raster(&mut self) {
        let Some(done) = self.rasterer.as_ref().and_then(Worker::try_recv) else {
            return;
        };
        let out = done.value;
        if self.requested == Some(out.key) {
            self.requested = None;
        }
        self.shown = Some(Shown {
            key: out.key,
            bitmap: out.bitmap,
            dst: out.dst,
        });
    }
}

struct State {
    cap: Option<Capability>,
    key: Option<Key>,
    slots: [Slot; 2],
    /// The viewport(s): with sync, the one at `shared_idx` for both panes;
    /// without, one per side. Always stored in the units of the picture it
    /// belongs to, after the reference size (`ref_size`).
    vps: [Viewport; 2],
    /// The side an inline view shows.
    flip: Side,
    /// The split's shares, session-only (a ratio, so it survives a resize).
    weights: Option<[u16; 2]>,
    /// The widths of the two panes as painted (or as last changed).
    widths: Option<[u16; 2]>,
    layout: Option<Layout>,
    /// Set by every frame that draws the image diff (or decides to fall back
    /// to the text); cleared by `begin_frame`.
    drawn: bool,
    /// Every present side is no picture: the text diff is drawn instead.
    fallback: bool,
    /// This frame's panes share one viewport.
    synced: bool,
    /// The zoom percentage label of the pane the keys act on.
    percent: Option<u32>,
    drag: Option<Drag>,
    last_press: Option<Instant>,
    hovered: Option<ImageHit>,
    hovered_mode: Option<DiffViewMode>,
    /// Reads to send after the frame (`take_requests`).
    outbox: Vec<FileInsightRequest>,
    /// The disk file behind the working-tree side of an unstaged diff, to
    /// notice an external edit (`refresh_worktree`).
    watch: Option<Watch>,
}

/// What the working-tree side looked like when it was last read.
struct Watch {
    stamp: Option<(std::time::SystemTime, u64)>,
    checked: Instant,
}

/// How often the working-tree file is looked at (a `stat`, no read).
const WATCH_EVERY: std::time::Duration = std::time::Duration::from_millis(500);

impl State {
    const fn new() -> State {
        State {
            cap: None,
            key: None,
            slots: [Slot::new(), Slot::new()],
            vps: [Viewport {
                zoom: Zoom::Fit,
                center: (0.5, 0.5),
            }; 2],
            flip: Side::After,
            weights: None,
            widths: None,
            layout: None,
            drawn: false,
            fallback: false,
            synced: true,
            percent: None,
            drag: None,
            last_press: None,
            hovered: None,
            hovered_mode: None,
            outbox: Vec::new(),
            watch: None,
        }
    }

    /// Back to nothing open (the split's shares are kept: they are the
    /// user's, like the merge editor's).
    fn reset(&mut self) {
        self.key = None;
        for s in &mut self.slots {
            s.clear();
        }
        self.vps = [Viewport::fit(); 2];
        self.flip = Side::After;
        self.widths = None;
        self.layout = None;
        self.fallback = false;
        self.percent = None;
        self.drag = None;
        self.outbox.clear();
        self.watch = None;
    }
}

thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State::new()) };
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

// --- lifecycle ---------------------------------------------------------------

/// Tells the view what the terminal can draw.
pub(crate) fn init(capability: Capability) {
    thumbs::init(capability);
    with(|s| s.cap = Some(capability));
}

/// Call at the start of every draw: an image diff that was not drawn by the
/// previous frame (the Diff view closed, another file) is forgotten, which
/// frees its bitmaps.
pub(crate) fn begin_frame() {
    thumbs::begin_frame();
    with(|s| {
        if !s.drawn && s.key.is_some() {
            s.reset();
        }
        s.drawn = false;
    });
}

/// Call once per frame, after every popup is drawn.
pub(crate) fn finish_frame(buf: &ratatui::buffer::Buffer) {
    thumbs::finish_frame(buf);
    with(|s| {
        for slot in &mut s.slots {
            if let Some(i) = slot.image.as_mut() {
                i.finish_frame(buf);
            }
        }
    });
}

/// Call after `Terminal::draw` returns.
pub(crate) fn write_pending<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
    thumbs::write_pending(out)?;
    with(|s| {
        for slot in &mut s.slots {
            if let Some(i) = slot.image.as_mut() {
                i.write_pending(out)?;
            }
        }
        Ok(())
    })
}

/// The reads the last frame asked for (sent off the draw closure).
pub(crate) fn take_requests() -> Vec<FileInsightRequest> {
    let mut out = thumbs::take_requests();
    out.extend(with(|s| std::mem::take(&mut s.outbox)));
    out
}

/// An unstaged diff reads its after side from the disk once; this notices
/// that the file changed since (an editor started with "Edit in", a tool, a
/// checkout) by its modification time and size, and reads it again, so the
/// picture follows. Call after each frame; it costs one `stat` every 500 ms
/// while such a diff is on screen.
pub(crate) fn refresh_worktree(repo_root: &std::path::Path) {
    with(|s| s.refresh_worktree(repo_root));
}

impl State {
    fn refresh_worktree(&mut self, repo_root: &std::path::Path) {
        let after = Side::After.index();
        let Some(key) = self.key.as_ref().filter(|k| {
            self.drawn && k.source == DiffSource::Unstaged && k.status != FileStatus::Deleted
        }) else {
            return;
        };
        if self
            .watch
            .as_ref()
            .is_some_and(|w| w.checked.elapsed() < WATCH_EVERY)
        {
            return;
        }
        let stamp = std::fs::metadata(repo_root.join(&key.path))
            .ok()
            .and_then(|m| Some((m.modified().ok()?, m.len())));
        let Some(watch) = self.watch.as_mut() else {
            // The first look is the baseline.
            self.watch = Some(Watch {
                stamp,
                checked: Instant::now(),
            });
            return;
        };
        watch.checked = Instant::now();
        if watch.stamp == stamp {
            return;
        }
        watch.stamp = stamp;
        let slot = &mut self.slots[after];
        // The picture stays until the new bytes arrive.
        if let Some(request) = slot.request.clone().filter(|_| slot.present) {
            self.outbox.push(request);
        }
    }
}

/// Whether the Diff view is showing before / after pictures: the keys, the
/// mouse and the palette consult this.
pub(crate) fn active() -> bool {
    with(|s| s.drawn && !s.fallback && s.key.is_some())
}

/// Whether a split is on screen (the divider keys and commands apply).
pub(crate) fn split_shown() -> bool {
    with(|s| {
        s.drawn
            && !s.fallback
            && s.layout
                .as_ref()
                .is_some_and(|l| l.arrangement == Arrangement::Split)
    })
}

/// Whether a picture is being fetched or decoded (the loop keeps its frame
/// rate for the spinner).
pub(crate) fn animating() -> bool {
    thumbs::animating() || with(|s| s.drawn && s.key.is_some() && s.slots.iter().any(Slot::pending))
}

/// Whether a mouse drag started in the view is still going.
pub(crate) fn dragging() -> bool {
    with(|s| s.drag.is_some())
}

/// The header button under the pointer, for its highlight.
pub(crate) fn set_hovered(target: Option<&HitTarget>) {
    with(|s| {
        s.hovered = match target {
            Some(HitTarget::Image(h)) => Some(*h),
            _ => None,
        };
        s.hovered_mode = match target {
            Some(HitTarget::DiffBarButton(DiffBarButton::Mode(m))) => Some(*m),
            _ => None,
        };
    });
}

// --- which file, which sides -------------------------------------------------

/// Whether the open diff of `file` is one to show as pictures: an image
/// extension and no text to show (a binary has no hunks; an LFS pointer's
/// text is the card the pictures replace).
pub(crate) fn wants(state: &DiffViewState, file: &FileDiff) -> bool {
    live_settings::image_view()
        && !state.lfs_raw
        // A comparison's pictures are not wired (the sides are two
        // arbitrary revisions): the text notice stands.
        && state.source != DiffSource::Compare
        && has_image_extension(&file.path)
        && (file.hunks.is_empty() || gitpoulpi_core::domain::lfs_change(file).is_some())
}

fn side_kind(source: DiffSource, side: Side) -> DiffContentSide {
    match (source, side) {
        (DiffSource::Commit | DiffSource::Compare, Side::Before) => DiffContentSide::Parent,
        (DiffSource::Commit | DiffSource::Compare, Side::After) => DiffContentSide::Commit,
        (DiffSource::Staged, Side::Before) => DiffContentSide::Head,
        (DiffSource::Staged, Side::After) => DiffContentSide::Index,
        (DiffSource::Unstaged, Side::Before) => DiffContentSide::Index,
        (DiffSource::Unstaged, Side::After) => DiffContentSide::WorkingTree,
    }
}

fn side_exists(status: FileStatus, side: Side) -> bool {
    match side {
        Side::Before => !matches!(status, FileStatus::Added | FileStatus::Untracked),
        Side::After => status != FileStatus::Deleted,
    }
}

/// "parent", "HEAD", "index", "working tree" or a short sha: where a side
/// comes from, for its caption.
fn where_from(key: &Key, side: Side) -> String {
    match (key.source, side) {
        (DiffSource::Commit | DiffSource::Compare, Side::Before) => {
            msg::WHERE_PARENT.get().to_string()
        }
        (DiffSource::Commit | DiffSource::Compare, Side::After) => {
            key.commit.to_hex().chars().take(7).collect()
        }
        (DiffSource::Staged, Side::Before) => msg::WHERE_HEAD.get().to_string(),
        (DiffSource::Staged, Side::After) | (DiffSource::Unstaged, Side::Before) => {
            msg::WHERE_INDEX.get().to_string()
        }
        (DiffSource::Unstaged, Side::After) => msg::WHERE_WORKTREE.get().to_string(),
    }
}

impl State {
    /// Follows the open diff: a different file starts the reads over.
    fn follow(&mut self, st: &DiffViewState, file: &FileDiff) {
        let key = Key {
            source: st.source,
            commit: st.commit,
            path: st.path.clone(),
            status: file.status,
        };
        if self.key.as_ref() == Some(&key) {
            return;
        }
        self.reset();
        // The OS's apps for this picture's type, for its "Edit in" list.
        if let Some(name) = st.path.file_name() {
            crate::open_with::warm(&name.to_string_lossy());
        }
        let at = match st.source {
            DiffSource::Commit => st.commit,
            _ => Oid([0u8; 20]),
        };
        for side in [Side::Before, Side::After] {
            let slot = &mut self.slots[side.index()];
            slot.present = side_exists(key.status, side);
            if !slot.present {
                continue;
            }
            let request = FileInsightRequest {
                path: st.path.clone(),
                at,
                kind: FileInsightKind::ImageBytes(side_kind(st.source, side)),
                worktree: false,
                staged: false,
            };
            slot.request = Some(request.clone());
            slot.phase = Phase::Fetching {
                since: Instant::now(),
            };
            self.outbox.push(request);
        }
        self.key = Some(key);
    }
}

/// Takes an image-bytes answer that is one of this view's two reads; anything
/// else is handed back (the File View's pane takes its own next).
pub(crate) fn offer(outcome: FileInsightOutcome) -> Option<FileInsightOutcome> {
    let outcome = thumbs::offer(outcome)?;
    if !matches!(outcome.request.kind, FileInsightKind::ImageBytes(_)) {
        return Some(outcome);
    }
    with(|s| {
        if s.key.is_none() {
            return Some(outcome);
        }
        let Some(i) = s
            .slots
            .iter()
            .position(|slot| slot.request.as_ref() == Some(&outcome.request))
        else {
            return Some(outcome);
        };
        let slot = &mut s.slots[i];
        match outcome.result {
            Ok(FileInsightPayload::ImageBytes(bytes)) => slot.bytes_arrived(bytes),
            Ok(_) => {}
            // No earlier version (a root commit, an unborn HEAD): this is
            // a new file as far as the view can tell.
            Err(_) if i == Side::Before.index() => {
                slot.present = false;
                slot.phase = Phase::Card(Vec::new());
            }
            Err(e) => slot.phase = Phase::Card(vec![msg::read_failed(e.to_string())]),
        }
        None
    })
}

// --- viewport plumbing -------------------------------------------------------

/// `vp` is in the units of the reference picture; the same viewport in the
/// units of the picture of size `own`.
fn localise(vp: Viewport, own: Size, reference: Size) -> Viewport {
    match vp.zoom {
        Zoom::Fit => vp,
        Zoom::Scale(s) => Viewport {
            zoom: Zoom::Scale(if own.w > 0.0 {
                s * reference.w / own.w
            } else {
                s
            }),
            center: vp.center,
        },
    }
}

/// The inverse of [`localise`].
fn globalise(vp: Viewport, own: Size, reference: Size) -> Viewport {
    match vp.zoom {
        Zoom::Fit => vp,
        Zoom::Scale(s) => Viewport {
            zoom: Zoom::Scale(if reference.w > 0.0 {
                s * own.w / reference.w
            } else {
                s
            }),
            center: vp.center,
        },
    }
}

impl State {
    /// The slot whose viewport the panes share (the after picture, else the
    /// before one).
    fn shared_idx(&self) -> usize {
        if self.slots[Side::After.index()].present {
            Side::After.index()
        } else {
            Side::Before.index()
        }
    }

    /// Where `side`'s viewport is stored.
    fn vp_idx(&self, side: Side) -> usize {
        if self.synced {
            self.shared_idx()
        } else {
            side.index()
        }
    }

    /// The size the stored viewport's scale is relative to.
    fn ref_size(&self, own: Size) -> Size {
        if self.synced {
            self.slots[self.shared_idx()].img_size().unwrap_or(own)
        } else {
            own
        }
    }

    /// The side the keys act on: the after pane of a split, else the one
    /// pane shown.
    fn key_side(&self) -> Option<Side> {
        let l = self.layout.as_ref()?;
        let pick = if l.arrangement == Arrangement::Split {
            l.panes.iter().find(|p| p.side == Side::After)
        } else {
            l.panes.first()
        }?;
        self.slots[pick.side.index()]
            .geo
            .is_some()
            .then_some(pick.side)
    }

    /// Edits `side`'s viewport through `f(viewport, image size, pane size)`,
    /// all in that picture's own units.
    fn edit(&mut self, side: Side, f: impl FnOnce(&Viewport, Size, Size) -> Viewport) -> bool {
        let Some(g) = self.slots[side.index()].geo else {
            return false;
        };
        let idx = self.vp_idx(side);
        let reference = self.ref_size(g.img);
        let local = localise(self.vps[idx], g.img, reference);
        self.vps[idx] = globalise(f(&local, g.img, g.pane_px), g.img, reference);
        true
    }

    fn zoom(&mut self, side: Side, op: ZoomOp, anchor: Option<(f32, f32)>) -> bool {
        self.edit(side, |vp, img, pane| match op {
            ZoomOp::In => vp.zoom_by(ZOOM_STEP, img, pane, false, anchor),
            ZoomOp::Out => vp.zoom_by(1.0 / ZOOM_STEP, img, pane, false, anchor),
            ZoomOp::Fit => Viewport::fit(),
            ZoomOp::Actual => vp.zoom_to(1.0, img, pane, false, anchor),
        })
    }

    fn pan(&mut self, side: Side, dx: f32, dy: f32) -> bool {
        self.edit(side, |vp, img, pane| vp.pan_by((dx, dy), img, pane, false))
    }
}

// --- drawing -------------------------------------------------------------------

fn button_style(hovered: bool) -> Style {
    if hovered {
        Style::default()
            .bg(th().ui.button_hover)
            .fg(th().ui.on_button)
    } else {
        Style::default().bg(th().ui.button).fg(th().ui.on_button)
    }
}

fn active_style() -> Style {
    Style::default()
        .bg(th().ui.accent)
        .fg(th().ui.bg)
        .add_modifier(Modifier::BOLD)
}

type Hits = Vec<(Rect, HitTarget)>;

/// Paints the image diff of `file` inside the Diff view's bordered block
/// (`inner` = the block's inside) and returns the hit rects from that paint;
/// `None` when the bytes are no picture on any side, so the caller draws the
/// text diff (today's notice) instead.
pub(crate) fn render(
    frame: &mut Frame,
    inner: Rect,
    st: &DiffViewState,
    file: &FileDiff,
) -> Option<Hits> {
    with(|s| s.render(frame, inner, st, file))
}

impl State {
    fn render(
        &mut self,
        frame: &mut Frame,
        inner: Rect,
        st: &DiffViewState,
        file: &FileDiff,
    ) -> Option<Hits> {
        self.drawn = true;
        self.follow(st, file);
        for slot in &mut self.slots {
            slot.poll_decode();
            slot.poll_raster();
            slot.geo = None;
            slot.bars = Bars::default();
        }
        let present: Vec<&Slot> = self.slots.iter().filter(|s| s.present).collect();
        self.fallback = !present.is_empty() && present.iter().all(|s| s.not_image);
        if self.fallback {
            self.layout = None;
            return None;
        }
        let key = self.key.clone()?;
        let has = [self.slots[0].present, self.slots[1].present];
        let want_split = st.mode == DiffViewMode::Split;
        let lay = layout::compute(inner, want_split, has, self.flip, self.weights);
        self.synced = live_settings::image_view_sync() || lay.arrangement != Arrangement::Split;
        self.widths = (lay.arrangement == Arrangement::Split)
            .then(|| [lay.panes[0].picture.width, lay.panes[1].picture.width]);
        let mut hits: Hits = Vec::new();
        self.percent = None;

        // Pictures first (they decide the geometry the header reports).
        for pb in &lay.panes {
            self.paint_pane(frame, pb, &lay, &key, &mut hits);
        }
        self.paint_divider(frame, &lay, &mut hits);
        self.paint_header(frame, &lay, st, &mut hits);
        self.layout = Some(lay);
        Some(hits)
    }

    fn paint_header(
        &mut self,
        frame: &mut Frame,
        lay: &Layout,
        st: &DiffViewState,
        hits: &mut Hits,
    ) {
        let row = lay.header;
        if row.is_empty() {
            return;
        }
        // The layout toggle sits at the right end: `Split | Inline`, or the
        // note that the split does not fit. Hidden for a one-sided file.
        let mut toggle_w = 0u16;
        if lay.arrangement != Arrangement::Single {
            let split_label = format!(" {} ", msg::SPLIT.get());
            let inline_label = format!(" {} ", msg::INLINE.get());
            if lay.narrow {
                let text = format!(" {} ", msg::SPLIT_NARROW.get());
                toggle_w = display_width(&text) as u16;
                if toggle_w < row.width {
                    frame.render_clipped(
                        Paragraph::new(Line::from(Span::styled(
                            text,
                            Style::default().fg(th().ui.muted),
                        ))),
                        Rect::new(row.right() - toggle_w, row.y, toggle_w, 1),
                    );
                }
            } else {
                let (sw, iw) = (
                    display_width(&split_label) as u16,
                    display_width(&inline_label) as u16,
                );
                toggle_w = sw + iw;
                if toggle_w < row.width {
                    let x = row.right() - toggle_w;
                    let split_on = st.mode == DiffViewMode::Split;
                    let style = |on: bool, mode: DiffViewMode| {
                        if on {
                            active_style()
                        } else {
                            button_style(self.hovered_mode == Some(mode))
                        }
                    };
                    let spans = vec![
                        Span::styled(split_label, style(split_on, DiffViewMode::Split)),
                        Span::styled(inline_label, style(!split_on, DiffViewMode::Inline)),
                    ];
                    frame.render_clipped(
                        Paragraph::new(Line::from(spans)),
                        Rect::new(x, row.y, toggle_w, 1),
                    );
                    hits.push((
                        Rect::new(x, row.y, sw, 1),
                        HitTarget::DiffBarButton(DiffBarButton::Mode(DiffViewMode::Split)),
                    ));
                    hits.push((
                        Rect::new(x + sw, row.y, iw, 1),
                        HitTarget::DiffBarButton(DiffBarButton::Mode(DiffViewMode::Inline)),
                    ));
                }
            }
        }
        let room = row.width.saturating_sub(toggle_w + 1);
        let head_row = Rect::new(row.x, row.y, room, 1);
        let info = self
            .slots
            .iter()
            .filter(|s| s.present)
            .filter_map(|s| s.info.as_ref().map(describe))
            .collect::<Vec<_>>()
            .join(" \u{2192} ");
        let head = header::layout(head_row, &info, self.percent, msg::FIT.get(), self.hovered);
        frame.render_clipped(Paragraph::new(head.line.clone()), head_row);
        hits.extend(head.buttons.iter().map(|(r, h)| (*r, HitTarget::Image(*h))));
        let slot = header::edit_slot(head_row);
        if let Some(rect) = slot.rect {
            super::edit_in::render_button(
                frame.buffer_mut(),
                rect,
                self.hovered == Some(ImageHit::EditIn),
            );
        }
        hits.extend(slot.hits);
    }

    fn paint_divider(&mut self, frame: &mut Frame, lay: &Layout, hits: &mut Hits) {
        let Some(div) = lay.divider else {
            return;
        };
        let lit =
            self.hovered == Some(ImageHit::Divider) || matches!(self.drag, Some(Drag::Divider));
        let style = if lit {
            Style::default().fg(th().ui.accent)
        } else {
            Style::default().fg(th().ui.muted)
        };
        let glyph = if lit { "\u{2503}" } else { "\u{2502}" };
        let buf = frame.buffer_mut();
        for y in div.y..div.bottom() {
            if let Some(cell) = buf.cell_mut((div.x, y)) {
                cell.set_symbol(glyph).set_style(style);
            }
        }
        hits.push((div, HitTarget::Image(ImageHit::Divider)));
    }

    fn caption(&self, lay: &Layout, pb: &PaneBox, key: &Key) -> Vec<Span<'static>> {
        let muted = Style::default().fg(th().ui.muted);
        let single = lay.arrangement == Arrangement::Single;
        if single {
            let text = if pb.side == Side::After {
                msg::NEW_FILE.get()
            } else {
                msg::DELETED_FILE.get()
            };
            return vec![Span::styled(
                format!(" {text}"),
                muted.add_modifier(Modifier::BOLD),
            )];
        }
        let label = |side: Side| match side {
            Side::Before => msg::BEFORE.get(),
            Side::After => msg::AFTER.get(),
        };
        if lay.arrangement == Arrangement::Inline {
            // `Before | After`: the shown one lit, both clickable.
            let mut spans = vec![Span::raw(" ")];
            for side in [Side::Before, Side::After] {
                let on = side == pb.side;
                let lit = self.hovered == Some(ImageHit::Flip(side));
                let style = if on {
                    active_style()
                } else {
                    button_style(lit)
                };
                spans.push(Span::styled(format!(" {} ", label(side)), style));
            }
            spans.push(Span::styled(
                format!(" \u{b7} {}", where_from(key, pb.side)),
                muted,
            ));
            return spans;
        }
        vec![Span::styled(
            format!(
                " {}",
                msg::caption(label(pb.side), where_from(key, pb.side))
            ),
            muted.add_modifier(Modifier::BOLD),
        )]
    }

    fn paint_pane(
        &mut self,
        frame: &mut Frame,
        pb: &PaneBox,
        lay: &Layout,
        key: &Key,
        hits: &mut Hits,
    ) {
        // Caption, with the inline flip's two segments as hits.
        let spans = self.caption(lay, pb, key);
        if lay.arrangement == Arrangement::Inline {
            let mut x = pb.caption.x + 1;
            for (i, side) in [Side::Before, Side::After].into_iter().enumerate() {
                let w = display_width(&spans[1 + i].content) as u16;
                if x + w <= pb.caption.right() {
                    hits.push((
                        Rect::new(x, pb.caption.y, w, 1),
                        HitTarget::Image(ImageHit::Flip(side)),
                    ));
                }
                x += w;
            }
        }
        frame.render_clipped(Paragraph::new(Line::from(spans)), pb.caption);

        let i = pb.side.index();
        let picture = pb.picture;
        if picture.is_empty() {
            return;
        }
        let slot = &self.slots[i];
        if slot.not_image {
            center_lines(frame, picture, &[msg::NOT_AN_IMAGE.get().to_string()]);
            return;
        }
        match &slot.phase {
            Phase::Fetching { since } | Phase::Decoding { since } => {
                if since.elapsed() >= SPINNER_AFTER {
                    let line = format!(
                        "{} {}",
                        crate::activity::spinner_frame(since.elapsed()),
                        msg::decoding(
                            key.path
                                .file_name()
                                .map(|n| n.to_string_lossy().into_owned())
                                .unwrap_or_default()
                        )
                    );
                    center_lines(frame, picture, &[line]);
                }
            }
            Phase::Card(lines) => center_lines(frame, picture, lines),
            Phase::Ready => {
                // The zoom label is the pane the keys act on: the after pane
                // of a split, the one pane otherwise.
                let keyed = lay.arrangement != Arrangement::Split || pb.side == Side::After;
                self.paint_picture(frame, pb, keyed)
            }
        }
    }

    fn paint_picture(&mut self, frame: &mut Frame, pb: &PaneBox, keyed: bool) {
        let i = pb.side.index();
        let Some((gen, bmp)) = self.slots[i].decoded.clone() else {
            return;
        };
        let picture = pb.picture;
        let pixel = self.cap.is_some_and(Capability::draws_images);
        if !pixel && live_settings::image_card_only() {
            center_lines(frame, picture, &[msg::TEXT_OFF.get().to_string()]);
            return;
        }
        let cell = match self.cap {
            Some(c) if c.draws_images() => (c.cell.width as f32, c.cell.height as f32),
            _ => (1.0, 2.0),
        };
        let img = Size::px(bmp.width, bmp.height);
        let pane_px = Size::new(
            f32::from(picture.width) * cell.0,
            f32::from(picture.height) * cell.1,
        );
        self.slots[i].geo = Some(Geo {
            picture,
            pane_px,
            img,
            cell,
        });
        let idx = self.vp_idx(pb.side);
        let reference = self.ref_size(img);
        let local = localise(self.vps[idx], img, reference);
        let view = local.view(img, pane_px, false);
        if keyed {
            self.percent = Some((view.scale * 100.0).round() as u32);
        }
        let (mut dw, mut dh) = raster::dst_size(&view);
        if pixel {
            (dw, dh) = raster::cap_output(dw, dh, raster::MAX_OUTPUT_PIXELS);
        }
        let pixelated_from = live_settings::image_pixelated_from();
        let checker = Checker::from_background(rgb(th().ui.bg, (24, 26, 38)));
        let mode = u32::from(pixel) ^ pixelated_from.to_bits().rotate_left(1);
        let rkey = view_key(gen, &view, dw, dh, mode, &checker);
        let slot = &mut self.slots[i];
        if slot.shown.as_ref().map(|s| s.key) != Some(rkey) && slot.requested != Some(rkey) {
            slot.requested = Some(rkey);
            let worker = slot.rasterer.get_or_insert_with(|| {
                Worker::spawn("image-raster", rasterise, crate::redraw::poke)
            });
            worker.submit(RasterJob {
                key: rkey,
                src: bmp.clone(),
                view,
                checker,
                dw,
                dh,
                pixelated_from,
            });
        }
        if let Some(shown) = slot.shown.as_ref() {
            let area = place(picture, shown.dst, cell, pixel);
            if pixel && slot.image.is_none() {
                if let Some(cap) = self.cap {
                    slot.image = Some(DetailAvatarImage::new(cap));
                }
            }
            let drawn = pixel
                && slot
                    .image
                    .as_mut()
                    .is_some_and(|im| im.render(frame.buffer_mut(), area, &shown.bitmap));
            if !drawn {
                halfblock::render(
                    frame.buffer_mut(),
                    area,
                    &shown.bitmap,
                    crate::color_depth::Depth::True,
                    rgb(th().ui.bg, (24, 26, 38)),
                );
            }
        }
        // Scrollbars on the ring around the picture, recomputed from this
        // frame's geometry (never cached across a change of it).
        let tracks = (
            Track {
                content: img.w * view.scale,
                view: view.dst.2.min(pane_px.w),
                offset: view.src.0 * view.scale,
            },
            Track {
                content: img.h * view.scale,
                view: view.dst.3.min(pane_px.h),
                offset: view.src.1 * view.scale,
            },
        );
        let frame_rect = pb.scroll_frame();
        let cells = |t: &Track, vp: usize| -> (usize, usize) {
            let k = vp as f32 / t.view.max(1.0);
            (
                (t.content * k).round() as usize,
                (t.offset * k).round() as usize,
            )
        };
        if tracks.1.scrolls() {
            let (content, offset) = cells(&tracks.1, usize::from(picture.height).max(1));
            if let Some(track) = scrollbar::Track::vertical(frame_rect, content, offset) {
                scrollbar::render(frame, track, true, false);
                self.slots[i].bars.v = Some((track, tracks.1));
            }
        }
        if tracks.0.scrolls() {
            let (content, offset) = cells(&tracks.0, usize::from(picture.width).max(1));
            if let Some(track) = scrollbar::Track::horizontal(frame_rect, content, offset) {
                scrollbar::render(frame, track, false, false);
                self.slots[i].bars.h = Some((track, tracks.0));
            }
        }
    }
}

// --- commands and keys ---------------------------------------------------------

/// Runs a zoom command on the pane the keys act on. `true` when a picture
/// was there.
pub(crate) fn run_zoom(op: ZoomOp) -> bool {
    with(|s| match s.key_side() {
        Some(side) => s.zoom(side, op, None),
        None => false,
    })
}

/// The Diff view picture's file name.
pub(crate) fn current_name() -> Option<String> {
    with(|s| {
        s.key.as_ref().map(|k| {
            k.path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
    })
}

/// The file "Edit in" hands to an editor for the side shown (the after pane of
/// a split, the flipped side of an inline view, the one side of an added or
/// deleted file), and the status notice when it is a copy: the real file for
/// the working-tree side of an unstaged diff, else a read-only temp copy
/// written from the bytes already in memory.
pub(crate) fn edit_file(
    repo_root: &std::path::Path,
) -> Result<(std::path::PathBuf, Option<String>), String> {
    use gitpoulpi_i18n::msg::image_edit_in as t;
    with(|s| {
        let key = s.key.clone().ok_or_else(|| t::NO_IMAGE.get().to_string())?;
        let lay = s
            .layout
            .as_ref()
            .ok_or_else(|| t::NO_IMAGE.get().to_string())?;
        let side = if lay.arrangement == Arrangement::Split {
            Side::After
        } else {
            lay.panes.first().map_or(Side::After, |p| p.side)
        };
        if side == Side::After && key.source == DiffSource::Unstaged {
            return Ok((repo_root.join(&key.path), None));
        }
        let raw = s.slots[side.index()]
            .raw
            .clone()
            .ok_or_else(|| t::NOT_LOADED.get().to_string())?;
        let short = gitpoulpi_core::domain::short(key.commit);
        let rev = match (key.source, side) {
            (DiffSource::Commit, Side::After) => short,
            (DiffSource::Commit, Side::Before) => format!("{short}^"),
            (DiffSource::Staged, Side::Before) => "HEAD".to_string(),
            _ => "index".to_string(),
        };
        let name = key
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let path = crate::image_editor::materialise(&raw, &rev, &name)
            .map_err(|e| t::copy_failed(e.to_string()))?;
        Ok((path, Some(t::copy_notice(&name, &rev))))
    })
}

/// `Tab`: the other side of an inline view, keeping zoom and center.
pub(crate) fn flip() -> bool {
    with(|s| {
        let inline = s
            .layout
            .as_ref()
            .is_some_and(|l| l.arrangement == Arrangement::Inline);
        if inline {
            s.flip = s.flip.other();
        }
        inline
    })
}

/// A click on the inline caption's `Before` / `After`.
pub(crate) fn show_side(side: Side) {
    with(|s| s.flip = side);
}

/// `Alt+Right` (`+1`: the before pane takes room) / `Alt+Left` (`-1`).
pub(crate) fn widen(direction: i32) -> bool {
    with(|s| {
        if !s
            .layout
            .as_ref()
            .is_some_and(|l| l.arrangement == Arrangement::Split)
        {
            return false;
        }
        let Some(cur) = s.widths else {
            return false;
        };
        let step = pane_split::key_step(cur[0] + cur[1]);
        s.set_split(cur, i32::from(cur[0]) + direction * step);
        true
    })
}

/// `Alt+0`: the equal share again.
pub(crate) fn reset_split() -> bool {
    with(|s| {
        let split = s
            .layout
            .as_ref()
            .is_some_and(|l| l.arrangement == Arrangement::Split);
        if split {
            s.weights = None;
            s.widths = None;
        }
        split
    })
}

impl State {
    /// Moves the divider so the before pane is `left` columns wide.
    fn set_split(&mut self, current: [u16; 2], left: i32) {
        if let Some(w) = layout::drag_to(current, left) {
            self.weights = Some(w);
            // Keys that arrive in one burst each step from the last result.
            let usable = current[0] + current[1];
            let next = pane_split::widths(Some(&w[..]), usable, 2);
            self.widths = Some([next[0], next[1]]);
        }
    }
}

/// The keys of an image diff: the four zoom actions, flip and the divider
/// (`Ctx::Image`), and panning on the scroll keys. `true` when consumed.
pub(crate) fn on_key(km: &crate::keymap::Keymap, key: &KeyEvent) -> bool {
    use crate::keymap::Action as A;
    // The Alt chords first: `Alt+0` must not be read as `0` (Fit).
    if km.is(A::ImageFlipSide, key) {
        return flip();
    }
    if km.is(A::ImageWidenBefore, key) {
        return widen(1);
    }
    if km.is(A::ImageWidenAfter, key) {
        return widen(-1);
    }
    if km.is(A::ImageResetSplit, key) {
        return reset_split();
    }
    if km.is(A::ImageZoomIn, key) {
        return run_zoom(ZoomOp::In);
    }
    if km.is(A::ImageZoomOut, key) {
        return run_zoom(ZoomOp::Out);
    }
    if km.is(A::ImageFit, key) {
        return run_zoom(ZoomOp::Fit);
    }
    if km.is(A::ImageActualSize, key) {
        return run_zoom(ZoomOp::Actual);
    }
    let Some((side, g)) = with(|s| {
        s.key_side()
            .and_then(|side| s.slots[side.index()].geo.map(|g| (side, g)))
    }) else {
        return false;
    };
    let big = key.modifiers.contains(KeyModifiers::SHIFT);
    let (w, h) = (g.pane_px.w, g.pane_px.h);
    let (sx, sy) = if big { (w, h) } else { (w / 10.0, h / 10.0) };
    let pan = |dx: f32, dy: f32| with(|s| s.pan(side, dx, dy));
    match key.code {
        KeyCode::Left | KeyCode::Char('h') if key.modifiers.is_empty() || big => pan(-sx, 0.0),
        KeyCode::Right | KeyCode::Char('l') if key.modifiers.is_empty() || big => pan(sx, 0.0),
        KeyCode::Up | KeyCode::Char('k') => pan(0.0, -sy),
        KeyCode::Down | KeyCode::Char('j') => pan(0.0, sy),
        KeyCode::PageUp => pan(0.0, -h),
        KeyCode::PageDown => pan(0.0, h),
        KeyCode::Home => pan(-w * 1e4, -h * 1e4),
        KeyCode::End => pan(w * 1e4, h * 1e4),
        _ => false,
    }
}

// --- the mouse -------------------------------------------------------------------

/// The pane's mouse handling: the wheel (Shift = horizontal, Ctrl = zoom at
/// the pointer), a left drag that pans, the scrollbars and the divider.
/// `true` when consumed.
pub(crate) fn on_mouse(kind: MouseEventKind, col: u16, row: u16, mods: KeyModifiers) -> bool {
    with(|s| s.on_mouse(kind, col, row, mods))
}

impl State {
    /// The side whose picture contains the cell.
    fn side_at(&self, col: u16, row: u16) -> Option<Side> {
        let at = ratatui::layout::Position::new(col, row);
        [Side::Before, Side::After].into_iter().find(|side| {
            self.slots[side.index()]
                .geo
                .is_some_and(|g| g.picture.contains(at))
        })
    }

    fn on_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16, mods: KeyModifiers) -> bool {
        let ctrl = mods.contains(KeyModifiers::CONTROL);
        let shift = mods.contains(KeyModifiers::SHIFT);
        match kind {
            MouseEventKind::ScrollUp
            | MouseEventKind::ScrollDown
            | MouseEventKind::ScrollLeft
            | MouseEventKind::ScrollRight => {
                let Some(side) = self.side_at(col, row) else {
                    return false;
                };
                let g = self.slots[side.index()].geo.expect("side_at has geometry");
                if ctrl && matches!(kind, MouseEventKind::ScrollUp | MouseEventKind::ScrollDown) {
                    let factor = if kind == MouseEventKind::ScrollUp {
                        ZOOM_STEP
                    } else {
                        1.0 / ZOOM_STEP
                    };
                    let anchor = (
                        (f32::from(col) - f32::from(g.picture.x) + 0.5) * g.cell.0,
                        (f32::from(row) - f32::from(g.picture.y) + 0.5) * g.cell.1,
                    );
                    return self.edit(side, |vp, img, pane| {
                        vp.zoom_by(factor, img, pane, false, Some(anchor))
                    });
                }
                let sign = match kind {
                    MouseEventKind::ScrollUp | MouseEventKind::ScrollLeft => -1.0,
                    _ => 1.0,
                };
                let horizontal = shift
                    || matches!(
                        kind,
                        MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight
                    );
                let step = 3.0;
                let (dx, dy) = if horizontal {
                    (sign * step * g.cell.0, 0.0)
                } else {
                    (0.0, sign * step * g.cell.1)
                };
                self.pan(side, dx, dy)
            }
            MouseEventKind::Down(MouseButton::Left) => self.press(col, row),
            MouseEventKind::Drag(MouseButton::Left) => self.drag_to(col, row),
            MouseEventKind::Up(MouseButton::Left) => self.drag.take().is_some(),
            _ => false,
        }
    }

    fn press(&mut self, col: u16, row: u16) -> bool {
        let at = ratatui::layout::Position::new(col, row);
        // The scrollbars first: they sit on the ring around a picture.
        for side in [Side::Before, Side::After] {
            let bars = self.slots[side.index()].bars;
            if let Some((track, px)) = bars.v.filter(|(t, _)| t.rect.contains(at)) {
                self.bar_down(side, true, track, px, row.saturating_sub(track.rect.y));
                return true;
            }
            if let Some((track, px)) = bars.h.filter(|(t, _)| t.rect.contains(at)) {
                self.bar_down(side, false, track, px, col.saturating_sub(track.rect.x));
                return true;
            }
        }
        if self
            .layout
            .as_ref()
            .and_then(|l| l.divider)
            .is_some_and(|d| d.contains(at))
        {
            // A second press within the window is a double click: back to
            // the equal share, no drag.
            let now = Instant::now();
            let double = self
                .last_press
                .is_some_and(|t| now.duration_since(t) < DOUBLE_CLICK);
            if double {
                self.last_press = None;
                self.weights = None;
                self.widths = None;
            } else {
                self.last_press = Some(now);
                self.drag = Some(Drag::Divider);
            }
            return true;
        }
        if let Some(side) = self.side_at(col, row) {
            self.drag = Some(Drag::Pan {
                side,
                last: (col, row),
            });
            return true;
        }
        false
    }

    fn drag_to(&mut self, col: u16, row: u16) -> bool {
        match self.drag {
            Some(Drag::Divider) => {
                let (Some(cur), Some(first)) = (
                    self.widths,
                    self.layout
                        .as_ref()
                        .and_then(|l| l.panes.first())
                        .map(|p| p.picture.x),
                ) else {
                    return true;
                };
                // The divider column sits one gutter column right of the
                // before pane: the pane is as wide as the pointer is far.
                self.set_split(cur, i32::from(col) - i32::from(first) - 1);
                true
            }
            Some(Drag::Pan { side, last }) => {
                let Some(g) = self.slots[side.index()].geo else {
                    return true;
                };
                let dx = f32::from(col) - f32::from(last.0);
                let dy = f32::from(row) - f32::from(last.1);
                self.pan(side, -dx * g.cell.0, -dy * g.cell.1);
                self.drag = Some(Drag::Pan {
                    side,
                    last: (col, row),
                });
                true
            }
            Some(Drag::Thumb {
                side,
                vertical,
                grab,
            }) => {
                let bars = self.slots[side.index()].bars;
                let bar = if vertical { bars.v } else { bars.h };
                if let Some((track, px)) = bar {
                    let pos = if vertical {
                        i32::from(row) - i32::from(track.rect.y)
                    } else {
                        i32::from(col) - i32::from(track.rect.x)
                    };
                    let room = f32::from(track.inner_len().saturating_sub(track.thumb_len));
                    let f = if room > 0.0 {
                        ((pos - grab - i32::from(track.arrow)) as f32 / room).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let offset = px.offset_for_fraction(f);
                    self.edit(side, |vp, img, pane| {
                        vp.scroll_to(!vertical, offset, img, pane, false)
                    });
                }
                true
            }
            None => false,
        }
    }

    fn bar_down(
        &mut self,
        side: Side,
        vertical: bool,
        track: scrollbar::Track,
        px: Track,
        pos: u16,
    ) {
        let Some(g) = self.slots[side.index()].geo else {
            return;
        };
        let pane = if vertical { g.pane_px.h } else { g.pane_px.w };
        match track.press(pos) {
            scrollbar::TrackPress::Arrow(arrow) => {
                let d = arrow as f32 * pane / 10.0;
                if vertical {
                    self.pan(side, 0.0, d);
                } else {
                    self.pan(side, d, 0.0);
                }
            }
            scrollbar::TrackPress::Grab { grab, jump } => {
                self.drag = Some(Drag::Thumb {
                    side,
                    vertical,
                    grab,
                });
                if jump {
                    let room = f32::from(track.inner_len().saturating_sub(track.thumb_len));
                    let f = if room > 0.0 {
                        ((i32::from(pos) - grab - i32::from(track.arrow)) as f32 / room)
                            .clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let offset = px.offset_for_fraction(f);
                    self.edit(side, |vp, img, pane| {
                        vp.scroll_to(!vertical, offset, img, pane, false)
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
