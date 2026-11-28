//! The image pane: a File View on a picture (plan
//! the image-view design, slice S4).
//!
//! The state lives in one thread-local singleton, like the minimap's
//! picture (`minimap.rs`): the draw closure, the key and mouse handlers and
//! the palette all reach it without a new field in each of their contexts,
//! and `finish_frame` / `write_pending` are called next to the minimap's in
//! `frame.rs`.
//!
//! The flow, none of it on the input thread (maintainer rule 9):
//!
//! 1. [`sync`] (every tick) sees a File View whose content is an image and
//!    asks the daemon for the exact bytes (`FileInsightKind::ImageBytes`);
//! 2. [`offer`] takes the answer off the File Insight channel and hands the
//!    bytes to the decode worker;
//! 3. the decoded bitmap is kept; [`render`] asks the raster worker for the
//!    visible crop at the pane's pixel size (latest wins) and keeps drawing
//!    the previous picture until the new one lands;
//! 4. a pixel protocol draws it through a `DetailAvatarImage`; without one
//!    it is drawn as half-block cells (`halfblock.rs`).
//!
//! Nothing here mutates the repository, so there is no Undo story: viewing a
//! file is not a mutation.

use crate::keymap::term::TermKeymap;
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use gitpoulpi_core::domain::{BlobBody, BlobBytes, BlobContent, ImageFormat};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::raster::{self, Checker};
use super::viewport::{Size, View, Viewport, Zoom, ZOOM_STEP};
use super::worker::Worker;
use super::{halfblock, header};
use crate::avatar::DetailAvatarImage;
use crate::chrome::HitTarget;
use crate::daemon_link::{
    DaemonLink, DiffContentSide, FileInsightKind, FileInsightOutcome, FileInsightPayload,
    FileInsightRequest,
};
use crate::file_view::{FileInsight, FileInsightData};
use crate::scrollbar;
use crate::terminal_image::{Bitmap, Capability};
use crate::ui_kit::ClippedRender;
use crate::{live_settings, th};
use gitpoulpi_i18n::msg::image_view as msg;

/// Past this, a pending picture shows its "Decoding..." line and spinner.
pub(super) const SPINNER_AFTER: Duration = Duration::from_millis(150);

/// A header button of the image view (`HitTarget::Image`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ImageHit {
    ZoomIn,
    ZoomOut,
    Fit,
    ActualSize,
    /// The inline Diff view's `Before | After` caption: show that side.
    Flip(super::layout::Side),
    /// The draggable divider between the before and after panes.
    Divider,
    /// The header's "Edit in" button (slice S7): opens the editor list.
    EditIn,
}

/// A zoom command, from a key, a button or the palette.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ZoomOp {
    In,
    Out,
    Fit,
    Actual,
}

impl ImageHit {
    /// The zoom this button runs, if it is a zoom button.
    pub(crate) fn op(self) -> Option<ZoomOp> {
        match self {
            ImageHit::ZoomIn => Some(ZoomOp::In),
            ImageHit::ZoomOut => Some(ZoomOp::Out),
            ImageHit::Fit => Some(ZoomOp::Fit),
            ImageHit::ActualSize => Some(ZoomOp::Actual),
            ImageHit::Flip(_) | ImageHit::Divider | ImageHit::EditIn => None,
        }
    }
}

/// A left click on a header button: a zoom, or the "Edit in" list.
pub(crate) fn click(hit: ImageHit) {
    match hit {
        ImageHit::ZoomIn => drop(run_zoom(ZoomOp::In)),
        ImageHit::ZoomOut => drop(run_zoom(ZoomOp::Out)),
        ImageHit::Fit => drop(run_zoom(ZoomOp::Fit)),
        ImageHit::ActualSize => drop(run_zoom(ZoomOp::Actual)),
        ImageHit::EditIn => super::edit_in::open(super::edit_in::Target::Pane),
        ImageHit::Flip(_) | ImageHit::Divider => {}
    }
}

// --- workers ---------------------------------------------------------------

/// Why a decode failed, in the pane's own terms (the decoder's error type
/// only exists with the `image-view` feature).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Failure {
    TooLarge { width: u32, height: u32 },
    Corrupt,
    NoPreview,
}

pub(super) struct DecodeJob {
    pub(super) gen: u64,
    pub(super) bytes: Vec<u8>,
}

pub(super) enum DecodeOut {
    Ok { gen: u64, bitmap: Bitmap },
    Err { gen: u64, failure: Failure },
}

pub(super) fn decode(job: DecodeJob) -> DecodeOut {
    use crate::image_decode::{decode_guarded, DecodeError};
    match decode_guarded(&job.bytes) {
        Ok(d) => DecodeOut::Ok {
            gen: job.gen,
            bitmap: d.bitmap,
        },
        Err(e) => DecodeOut::Err {
            gen: job.gen,
            failure: match e {
                DecodeError::TooLarge { width, height } => Failure::TooLarge { width, height },
                DecodeError::Unrecognized | DecodeError::Unsupported(_) => Failure::NoPreview,
                DecodeError::Corrupt | DecodeError::Panicked => Failure::Corrupt,
            },
        },
    }
}

pub(super) struct RasterJob {
    pub(super) key: u64,
    pub(super) src: Arc<Bitmap>,
    pub(super) view: View,
    pub(super) checker: Checker,
    pub(super) dw: u32,
    pub(super) dh: u32,
    /// The scale from which the picture is enlarged without smoothing
    /// (`ui.image_view_pixelated_from`).
    pub(super) pixelated_from: f32,
}

pub(super) struct RasterOut {
    pub(super) key: u64,
    pub(super) bitmap: Bitmap,
    pub(super) dst: (f32, f32, f32, f32),
}

pub(super) fn rasterise(job: RasterJob) -> RasterOut {
    RasterOut {
        key: job.key,
        bitmap: raster::render_to_from(
            &job.src,
            &job.view,
            &job.checker,
            job.dw,
            job.dh,
            job.pixelated_from,
        ),
        dst: job.view.dst,
    }
}

// --- state -----------------------------------------------------------------

pub(super) enum Phase {
    /// The bytes are being fetched from the daemon.
    Fetching {
        since: Instant,
    },
    /// The bytes are being decoded.
    Decoding {
        since: Instant,
    },
    Ready,
    /// A card instead of a picture: why there is none.
    Card(Vec<String>),
}

pub(super) struct Info {
    pub(super) format: Option<ImageFormat>,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) size: u64,
    pub(super) lfs: bool,
    pub(super) frames: Option<u32>,
}

pub(super) struct Shown {
    pub(super) key: u64,
    pub(super) bitmap: Bitmap,
    pub(super) dst: (f32, f32, f32, f32),
}

/// The geometry of the last frame, for keys and the mouse.
#[derive(Clone, Copy)]
struct Geo {
    pane: Rect,
    pane_px: Size,
    img: Size,
    /// Pane pixels per cell, horizontally and vertically.
    cell: (f32, f32),
}

#[derive(Clone, Copy)]
struct Bars {
    v: Option<(scrollbar::Track, super::viewport::Track)>,
    h: Option<(scrollbar::Track, super::viewport::Track)>,
}

#[derive(Clone, Copy)]
enum Drag {
    Pan { last: (u16, u16) },
    Thumb { vertical: bool, grab: i32 },
}

/// Which load of which request the pane shows.
#[derive(Clone, PartialEq)]
struct Key {
    request: FileInsightRequest,
    load: u64,
}

struct State {
    cap: Option<Capability>,
    key: Option<Key>,
    phase: Phase,
    info: Option<Info>,
    name: String,
    vp: Viewport,
    gen: u64,
    decoded: Option<(u64, Arc<Bitmap>)>,
    decoder: Option<Worker<DecodeJob, DecodeOut>>,
    rasterer: Option<Worker<RasterJob, RasterOut>>,
    requested: Option<u64>,
    shown: Option<Shown>,
    geo: Option<Geo>,
    bars: Bars,
    drag: Option<Drag>,
    hovered: Option<ImageHit>,
    images: Option<(DetailAvatarImage, DetailAvatarImage)>,
    /// The exact bytes the picture was decoded from, for the temp copy an
    /// editor is handed (no second daemon read).
    raw: Option<Arc<Vec<u8>>>,
}

impl State {
    const fn new() -> State {
        State {
            cap: None,
            key: None,
            phase: Phase::Card(Vec::new()),
            info: None,
            name: String::new(),
            vp: Viewport {
                zoom: Zoom::Fit,
                center: (0.5, 0.5),
            },
            gen: 0,
            decoded: None,
            decoder: None,
            rasterer: None,
            requested: None,
            shown: None,
            geo: None,
            bars: Bars { v: None, h: None },
            drag: None,
            hovered: None,
            images: None,
            raw: None,
        }
    }

    fn reset(&mut self) {
        self.key = None;
        self.info = None;
        self.decoded = None;
        self.raw = None;
        self.requested = None;
        self.shown = None;
        self.geo = None;
        self.bars = Bars { v: None, h: None };
        self.drag = None;
        self.vp = Viewport::fit();
        // A running decode / raster is superseded by the next submit; the
        // generation makes a late answer harmless.
        self.gen += 1;
    }
}

thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State::new()) };
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

/// Tells the pane what the terminal can draw.
pub(crate) fn init(capability: Capability) {
    with(|s| {
        s.cap = Some(capability);
        s.images = Some((
            DetailAvatarImage::new(capability),
            DetailAvatarImage::new(capability),
        ));
    });
}

/// Call once per frame, after every popup is drawn.
pub(crate) fn finish_frame(buf: &Buffer) {
    with(|s| {
        if let Some((a, b)) = s.images.as_mut() {
            a.finish_frame(buf);
            b.finish_frame(buf);
        }
    });
}

/// Call after `Terminal::draw` returns.
pub(crate) fn write_pending<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
    with(|s| match s.images.as_mut() {
        Some((a, b)) => {
            a.write_pending(out)?;
            b.write_pending(out)
        }
        None => Ok(()),
    })
}

/// Whether an image (or its pending / card state) is what the File View
/// shows: the keys, the mouse and the palette consult this.
pub(crate) fn active() -> bool {
    with(|s| s.key.is_some())
}

/// Whether the pane is waiting on bytes or a decode (the loop keeps its
/// frame rate for the spinner).
pub(crate) fn animating() -> bool {
    with(|s| matches!(s.phase, Phase::Fetching { .. } | Phase::Decoding { .. }) && s.key.is_some())
}

/// Whether the pane owns the drawing of this File View.
pub(crate) fn shows(insight: &FileInsight) -> bool {
    with(|s| {
        s.key
            .as_ref()
            .is_some_and(|k| k.request == insight.request && k.load == insight.load_id)
    })
}

// --- picking an image out of a File View ------------------------------------

const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "webp", "ico", "cur", "tif", "tiff", "avif", "heic",
];

pub(super) fn has_image_extension(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| IMAGE_EXTENSIONS.iter().any(|x| x.eq_ignore_ascii_case(e)))
}

pub(super) fn format_name(f: ImageFormat) -> &'static str {
    match f {
        ImageFormat::Png => "PNG",
        ImageFormat::Jpeg => "JPEG",
        ImageFormat::Gif => "GIF",
        ImageFormat::Bmp => "BMP",
        ImageFormat::Webp => "WebP",
        ImageFormat::Ico => "ICO",
        ImageFormat::Tiff => "TIFF",
        ImageFormat::Avif => "AVIF",
        ImageFormat::Heic => "HEIC",
    }
}

/// What the File View's content is, as far as the image pane cares:
/// `Some((info, lfs))` for a picture to preview.
fn wanted(insight: &FileInsight) -> Option<Info> {
    if !live_settings::image_view() || insight.request.kind != FileInsightKind::Content {
        return None;
    }
    let FileInsightData::Content(blob) = &insight.data else {
        return None;
    };
    match &blob.content {
        BlobContent::Binary | BlobContent::TooLarge => blob.image.as_ref().map(|i| Info {
            format: Some(i.format),
            width: i.width,
            height: i.height,
            size: blob.size,
            lfs: false,
            frames: i.frames,
        }),
        BlobContent::LfsPointer(p) if has_image_extension(&insight.request.path) => Some(Info {
            format: None,
            width: 0,
            height: 0,
            size: p.size,
            lfs: true,
            frames: None,
        }),
        _ => None,
    }
}

fn side_of(request: &FileInsightRequest) -> DiffContentSide {
    if request.worktree {
        if request.staged {
            DiffContentSide::Index
        } else {
            DiffContentSide::WorkingTree
        }
    } else {
        DiffContentSide::Commit
    }
}

/// Follows the File View: when it shows a new picture, asks for its bytes;
/// when it stops showing one, forgets it. Called every tick.
pub(crate) fn sync(insight: Option<&FileInsight>, link: &DaemonLink) {
    with(|s| {
        let want = insight.and_then(|i| wanted(i).map(|info| (i, info)));
        match want {
            None => {
                if s.key.is_some() {
                    s.reset();
                }
            }
            Some((insight, info)) => {
                let key = Key {
                    request: insight.request.clone(),
                    load: insight.load_id,
                };
                if s.key.as_ref() == Some(&key) {
                    return;
                }
                s.reset();
                s.name = insight
                    .request
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                // The OS's apps for this picture's type, for its "Edit in" list.
                crate::open_with::warm(&s.name);
                s.key = Some(key);
                let cap = live_settings::image_view_max_bytes();
                if info.size > cap {
                    s.phase = Phase::Card(vec![msg::over_cap(gitpoulpi_core::domain::human_size(
                        info.size,
                    ))]);
                } else {
                    s.phase = Phase::Fetching {
                        since: Instant::now(),
                    };
                    link.request_file_insight(FileInsightRequest {
                        kind: FileInsightKind::ImageBytes(side_of(&insight.request)),
                        ..insight.request.clone()
                    });
                }
                s.info = Some(info);
            }
        }
    });
}

/// Takes the image bytes' answer off the File Insight channel; anything
/// else is handed back.
pub(crate) fn offer(outcome: FileInsightOutcome) -> Option<FileInsightOutcome> {
    if !matches!(outcome.request.kind, FileInsightKind::ImageBytes(_)) {
        return Some(outcome);
    }
    with(|s| {
        let mine = s.key.as_ref().is_some_and(|k| {
            let mut asked = k.request.clone();
            asked.kind = outcome.request.kind;
            asked == outcome.request
        });
        if !mine {
            return;
        }
        match outcome.result {
            Ok(FileInsightPayload::ImageBytes(bytes)) => s.bytes_arrived(bytes),
            Ok(_) => {}
            Err(e) => s.phase = Phase::Card(vec![msg::read_failed(e.to_string())]),
        }
    });
    None
}

impl State {
    fn bytes_arrived(&mut self, bytes: BlobBytes) {
        let data = match bytes.body {
            BlobBody::Bytes(b) => b,
            BlobBody::Lfs {
                object: Some(b), ..
            } => b,
            BlobBody::TooLarge => {
                self.phase = Phase::Card(vec![msg::over_cap(gitpoulpi_core::domain::human_size(
                    bytes.size,
                ))]);
                return;
            }
            // Not downloaded: the File View's own LFS card (with the pull
            // hint) is the right thing to show.
            BlobBody::Lfs { object: None, .. } => {
                self.reset();
                return;
            }
        };
        self.gen += 1;
        let gen = self.gen;
        self.phase = Phase::Decoding {
            since: Instant::now(),
        };
        let decoder = self
            .decoder
            .get_or_insert_with(|| Worker::spawn("image-decode", decode, crate::redraw::poke));
        self.raw = Some(Arc::new(data.clone()));
        decoder.submit(DecodeJob { gen, bytes: data });
    }

    fn poll_decode(&mut self) {
        let Some(done) = self.decoder.as_ref().and_then(Worker::try_recv) else {
            return;
        };
        match done.value {
            DecodeOut::Ok { gen, bitmap } if gen == self.gen => {
                if let Some(info) = self.info.as_mut() {
                    if info.width == 0 {
                        info.width = bitmap.width;
                        info.height = bitmap.height;
                    }
                }
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

    fn info_line(&self) -> String {
        self.info.as_ref().map(describe).unwrap_or_default()
    }
}

/// "PNG 512x512 . 34.2 KiB . LFS . first frame of 12".
pub(super) fn describe(i: &Info) -> String {
    let mut parts = Vec::new();
    let mut head = i.format.map(format_name).unwrap_or("").to_string();
    if i.width > 0 {
        if !head.is_empty() {
            head.push(' ');
        }
        head.push_str(&format!("{}\u{d7}{}", i.width, i.height));
    }
    if !head.is_empty() {
        parts.push(head);
    }
    parts.push(gitpoulpi_core::domain::human_size(i.size));
    if i.lfs {
        parts.push("LFS".to_string()); // i18n-english
    }
    if let Some(n) = i.frames.filter(|n| *n > 1) {
        parts.push(msg::first_frame_of(n));
    }
    parts.join(" \u{b7} ")
}

// --- drawing -----------------------------------------------------------------

pub(super) fn rgb(c: Color, fallback: (u8, u8, u8)) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => fallback,
    }
}

fn hash_key(parts: &[u32]) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    parts.hash(&mut h);
    h.finish()
}

pub(super) fn view_key(
    gen: u64,
    view: &View,
    dw: u32,
    dh: u32,
    mode: u32,
    checker: &Checker,
) -> u64 {
    let f = |v: f32| v.to_bits();
    hash_key(&[
        gen as u32,
        f(view.scale),
        f(view.src.0),
        f(view.src.1),
        f(view.src.2),
        f(view.src.3),
        f(view.dst.0),
        f(view.dst.1),
        dw,
        dh,
        mode,
        u32::from(checker.light.0),
        u32::from(checker.dark.0),
    ])
}

/// What painting the pane produced: the header's hit rects.
pub(crate) struct Painted {
    pub(crate) hits: Vec<(Rect, HitTarget)>,
}

/// Paints the image pane inside the File View's bordered `body`
/// (`inner` = the block's inside): the header line, then the picture, a card
/// or the decoding line, and the scrollbars on the border.
pub(crate) fn render(frame: &mut Frame, body: Rect, inner: Rect) -> Painted {
    with(|s| s.render(frame, body, inner))
}

/// The header button under the pointer, for its highlight (set by the draw
/// before the File View is painted).
pub(crate) fn set_hovered(target: Option<&HitTarget>) {
    with(|s| {
        s.hovered = match target {
            Some(HitTarget::Image(h)) => Some(*h),
            _ => None,
        }
    });
}

impl State {
    fn render(&mut self, frame: &mut Frame, body: Rect, inner: Rect) -> Painted {
        self.poll_decode();
        self.poll_raster();
        let mut hits = Vec::new();
        self.bars = Bars { v: None, h: None };
        self.geo = None;
        if inner.height < 2 || inner.width == 0 {
            return Painted { hits };
        }
        let row = Rect::new(inner.x, inner.y, inner.width, 1);
        let pane = Rect::new(inner.x, inner.y + 1, inner.width, inner.height - 1);
        let hovered_hit = self.hovered;
        let ready = matches!(self.phase, Phase::Ready) && self.decoded.is_some();
        let scale_percent = if ready {
            self.decoded.as_ref().map(|(_, bmp)| {
                let (img, pane_px, _) = self.measure(pane, bmp);
                (self.vp.scale(img, pane_px, false) * 100.0).round() as u32
            })
        } else {
            None
        };
        let head = header::layout(
            row,
            &self.info_line(),
            scale_percent,
            msg::FIT.get(),
            hovered_hit,
        );
        frame.render_clipped(Paragraph::new(head.line.clone()), row);
        hits.extend(head.buttons.iter().map(|(r, h)| (*r, HitTarget::Image(*h))));
        let slot = header::edit_slot(row);
        if let Some(rect) = slot.rect {
            super::edit_in::render_button(
                frame.buffer_mut(),
                rect,
                hovered_hit == Some(ImageHit::EditIn),
            );
        }
        hits.extend(slot.hits);
        // A menu's "Edit in…" on a picture that was not loaded yet.
        if super::edit_in::pending()
            && matches!(self.phase, Phase::Ready | Phase::Card(_))
            && super::edit_in::take_pending()
        {
            super::edit_in::open_for(super::edit_in::Target::Pane, self.name.clone());
        }

        match &self.phase {
            Phase::Fetching { since } | Phase::Decoding { since } => {
                if since.elapsed() >= SPINNER_AFTER {
                    let line = format!(
                        "{} {}",
                        crate::activity::spinner_frame(since.elapsed()),
                        msg::decoding(&self.name)
                    );
                    center_lines(frame, pane, &[line]);
                }
            }
            Phase::Card(lines) => center_lines(frame, pane, lines),
            Phase::Ready => self.render_ready(frame, body, pane),
        }
        Painted { hits }
    }

    /// `(image size, pane size in pixels, cell size in pixels)`.
    fn measure(&self, pane: Rect, bmp: &Bitmap) -> (Size, Size, (f32, f32)) {
        let cell = match self.cap {
            Some(c) if c.draws_images() => (c.cell.width as f32, c.cell.height as f32),
            _ => (1.0, 2.0),
        };
        (
            Size::px(bmp.width, bmp.height),
            Size::new(
                f32::from(pane.width) * cell.0,
                f32::from(pane.height) * cell.1,
            ),
            cell,
        )
    }

    fn render_ready(&mut self, frame: &mut Frame, body: Rect, pane: Rect) {
        let Some((gen, bmp)) = self.decoded.clone() else {
            return;
        };
        if pane.is_empty() {
            return;
        }
        let pixel = self.cap.is_some_and(Capability::draws_images);
        if !pixel && live_settings::image_card_only() {
            center_lines(frame, pane, &[msg::TEXT_OFF.get().to_string()]);
            return;
        }
        let (img, pane_px, cell) = self.measure(pane, &bmp);
        self.geo = Some(Geo {
            pane,
            pane_px,
            img,
            cell,
        });
        self.vp = self.vp.clamped(img, pane_px, false);
        let view = self.vp.view(img, pane_px, false);
        let (mut dw, mut dh) = raster::dst_size(&view);
        if pixel {
            (dw, dh) = raster::cap_output(dw, dh, raster::MAX_OUTPUT_PIXELS);
        }
        let checker = Checker::from_background(rgb(th().ui.bg, (24, 26, 38)));
        let mode = u32::from(pixel)
            ^ live_settings::image_pixelated_from()
                .to_bits()
                .rotate_left(1);
        let key = view_key(gen, &view, dw, dh, mode, &checker);
        if self.shown.as_ref().map(|s| s.key) != Some(key) && self.requested != Some(key) {
            self.requested = Some(key);
            let worker = self.rasterer.get_or_insert_with(|| {
                Worker::spawn("image-raster", rasterise, crate::redraw::poke)
            });
            worker.submit(RasterJob {
                key,
                src: bmp.clone(),
                view,
                checker,
                dw,
                dh,
                pixelated_from: live_settings::image_pixelated_from(),
            });
        }
        if let Some(shown) = self.shown.as_ref() {
            let area = place(pane, shown.dst, cell, pixel);
            let drawn = pixel
                && self
                    .images
                    .as_mut()
                    .is_some_and(|(a, _)| a.render(frame.buffer_mut(), area, &shown.bitmap));
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
        // Scrollbars on the border, recomputed from the geometry every frame.
        let (th_track, tv_track) = self.vp.tracks(img, pane_px, false);
        let vp_rows = usize::from(body.height.saturating_sub(2)).max(1);
        let vp_cols = usize::from(body.width.saturating_sub(2)).max(1);
        let cells = |t: &super::viewport::Track, vp: usize| -> (usize, usize) {
            let k = vp as f32 / t.view.max(1.0);
            (
                (t.content * k).round() as usize,
                (t.offset * k).round() as usize,
            )
        };
        if tv_track.scrolls() {
            let (content, offset) = cells(&tv_track, vp_rows);
            if let Some(track) = scrollbar::Track::vertical(body, content, offset) {
                scrollbar::render(frame, track, true, false);
                self.bars.v = Some((track, tv_track));
            }
        }
        if th_track.scrolls() {
            let (content, offset) = cells(&th_track, vp_cols);
            if let Some(track) = scrollbar::Track::horizontal(body, content, offset) {
                scrollbar::render(frame, track, false, false);
                self.bars.h = Some((track, th_track));
            }
        }
    }
}

/// Where the rendered crop lands, in cells, clipped to the pane.
pub(super) fn place(pane: Rect, dst: (f32, f32, f32, f32), cell: (f32, f32), pixel: bool) -> Rect {
    let (x, y, w, h) = if pixel {
        (
            (dst.0 / cell.0).round(),
            (dst.1 / cell.1).round(),
            (dst.2 / cell.0).round().max(1.0),
            (dst.3 / cell.1).round().max(1.0),
        )
    } else {
        (
            dst.0.floor(),
            (dst.1 / 2.0).floor(),
            dst.2.ceil().max(1.0),
            (dst.3 / 2.0).ceil().max(1.0),
        )
    };
    Rect::new(
        pane.x.saturating_add(x as u16),
        pane.y.saturating_add(y as u16),
        w as u16,
        h as u16,
    )
    .intersection(pane)
}

pub(super) fn center_lines(frame: &mut Frame, pane: Rect, lines: &[String]) {
    if pane.is_empty() || lines.is_empty() {
        return;
    }
    let top = pane.y + pane.height.saturating_sub(lines.len() as u16) / 2;
    for (i, line) in lines.iter().enumerate() {
        let y = top + i as u16;
        if y >= pane.bottom() {
            break;
        }
        frame.render_clipped(
            Paragraph::new(Line::from(Span::styled(
                line.clone(),
                Style::default().fg(th().ui.muted),
            )))
            .alignment(ratatui::layout::Alignment::Center),
            Rect::new(pane.x, y, pane.width, 1),
        );
    }
}

// --- keys, palette and mouse -------------------------------------------------

/// Runs a zoom command. `anchor` is a pane-pixel point for the pointer;
/// `None` anchors on the pane center. Returns whether an image was there.
fn zoom(op: ZoomOp, anchor: Option<(f32, f32)>) -> bool {
    with(|s| {
        let Some(g) = s.geo else {
            return false;
        };
        s.vp = match op {
            ZoomOp::In => s.vp.zoom_by(ZOOM_STEP, g.img, g.pane_px, false, anchor),
            ZoomOp::Out => {
                s.vp.zoom_by(1.0 / ZOOM_STEP, g.img, g.pane_px, false, anchor)
            }
            ZoomOp::Fit => Viewport::fit(),
            ZoomOp::Actual => s.vp.zoom_to(1.0, g.img, g.pane_px, false, anchor),
        };
        true
    })
}

/// The File View picture's file name.
pub(crate) fn current_name() -> Option<String> {
    with(|s| s.key.as_ref().map(|_| s.name.clone()))
}

/// The file "Edit in" hands to an editor, and the status notice when it is a
/// copy: the real file for a working-tree image (the editor saves in place),
/// else a read-only temp copy written from the bytes already in memory.
pub(crate) fn edit_file(
    repo_root: &std::path::Path,
) -> Result<(std::path::PathBuf, Option<String>), String> {
    use gitpoulpi_i18n::msg::image_edit_in as t;
    with(|s| {
        let key = s
            .key
            .as_ref()
            .ok_or_else(|| t::NO_IMAGE.get().to_string())?;
        let request = &key.request;
        if request.worktree && !request.staged {
            return Ok((repo_root.join(&request.path), None));
        }
        let raw = s
            .raw
            .clone()
            .ok_or_else(|| t::NOT_LOADED.get().to_string())?;
        let rev = if request.worktree {
            "index".to_string()
        } else {
            gitpoulpi_core::domain::short(request.at)
        };
        let path = crate::image_editor::materialise(&raw, &rev, &s.name)
            .map_err(|e| t::copy_failed(e.to_string()))?;
        Ok((path, Some(t::copy_notice(&s.name, &rev))))
    })
}

pub(crate) fn run_zoom(op: ZoomOp) -> bool {
    zoom(op, None)
}

fn pan(dx: f32, dy: f32) -> bool {
    with(|s| {
        let Some(g) = s.geo else {
            return false;
        };
        s.vp = s.vp.pan_by((dx, dy), g.img, g.pane_px, false);
        true
    })
}

/// A key the image pane takes while it is shown: the four zoom actions
/// (`Ctx::Image`) and panning on the scroll keys. `true` when consumed.
pub(crate) fn on_key(km: &crate::keymap::Keymap, key: &KeyEvent) -> bool {
    use crate::keymap::Action as A;
    if km.is(A::ImageZoomIn, key) {
        return zoom(ZoomOp::In, None);
    }
    if km.is(A::ImageZoomOut, key) {
        return zoom(ZoomOp::Out, None);
    }
    if km.is(A::ImageFit, key) {
        return zoom(ZoomOp::Fit, None);
    }
    if km.is(A::ImageActualSize, key) {
        return zoom(ZoomOp::Actual, None);
    }
    let Some(g) = with(|s| s.geo) else {
        return false;
    };
    let big = key.modifiers.contains(KeyModifiers::SHIFT);
    let (w, h) = (g.pane_px.w, g.pane_px.h);
    let (sx, sy) = if big { (w, h) } else { (w / 10.0, h / 10.0) };
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => pan(-sx, 0.0),
        KeyCode::Right | KeyCode::Char('l') => pan(sx, 0.0),
        KeyCode::Up | KeyCode::Char('k') => pan(0.0, -sy),
        KeyCode::Down | KeyCode::Char('j') => pan(0.0, sy),
        KeyCode::PageUp => pan(0.0, -h),
        KeyCode::PageDown => pan(0.0, h),
        KeyCode::Home => pan(-w * 1e4, -h * 1e4),
        KeyCode::End => pan(w * 1e4, h * 1e4),
        _ => false,
    }
}

/// Whether a mouse drag started in the pane is still going: the move and the
/// release are the pane's wherever the pointer is.
pub(crate) fn dragging() -> bool {
    with(|s| s.drag.is_some())
}

/// The pane's mouse handling: the wheel (Shift = horizontal, Ctrl = zoom at
/// the pointer), a left drag that pans, the scrollbars. `true` when consumed.
pub(crate) fn on_mouse(kind: MouseEventKind, col: u16, row: u16, mods: KeyModifiers) -> bool {
    with(|s| s.on_mouse(kind, col, row, mods))
}

impl State {
    fn anchor(&self, g: &Geo, col: u16, row: u16) -> (f32, f32) {
        (
            (f32::from(col) - f32::from(g.pane.x) + 0.5) * g.cell.0,
            (f32::from(row) - f32::from(g.pane.y) + 0.5) * g.cell.1,
        )
    }

    fn on_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16, mods: KeyModifiers) -> bool {
        let Some(g) = self.geo else {
            self.drag = None;
            return false;
        };
        let (img, pane_px) = (g.img, g.pane_px);
        let step = 3.0;
        let ctrl = mods.contains(KeyModifiers::CONTROL);
        let shift = mods.contains(KeyModifiers::SHIFT);
        match kind {
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown if ctrl => {
                let factor = if kind == MouseEventKind::ScrollUp {
                    ZOOM_STEP
                } else {
                    1.0 / ZOOM_STEP
                };
                let anchor = self.anchor(&g, col, row);
                self.vp = self.vp.zoom_by(factor, img, pane_px, false, Some(anchor));
            }
            MouseEventKind::ScrollUp
            | MouseEventKind::ScrollDown
            | MouseEventKind::ScrollLeft
            | MouseEventKind::ScrollRight => {
                let sign = match kind {
                    MouseEventKind::ScrollUp | MouseEventKind::ScrollLeft => -1.0,
                    _ => 1.0,
                };
                let horizontal = shift
                    || matches!(
                        kind,
                        MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight
                    );
                let d = if horizontal {
                    (sign * step * g.cell.0, 0.0)
                } else {
                    (0.0, sign * step * g.cell.1)
                };
                self.vp = self.vp.pan_by(d, img, pane_px, false);
            }
            MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                let at = ratatui::layout::Position::new(col, row);
                if let Some((track, px)) = self.bars.v.filter(|(t, _)| t.rect.contains(at)) {
                    let pos = row.saturating_sub(track.rect.y);
                    self.bar_down(true, track, px, pos, &g);
                } else if let Some((track, px)) = self.bars.h.filter(|(t, _)| t.rect.contains(at)) {
                    let pos = col.saturating_sub(track.rect.x);
                    self.bar_down(false, track, px, pos, &g);
                } else if g.pane.contains(at) {
                    self.drag = Some(Drag::Pan { last: (col, row) });
                } else {
                    return false;
                }
            }
            MouseEventKind::Drag(crossterm::event::MouseButton::Left) => match self.drag {
                Some(Drag::Pan { last }) => {
                    let dx = f32::from(col) - f32::from(last.0);
                    let dy = f32::from(row) - f32::from(last.1);
                    self.vp = self
                        .vp
                        .pan_by((-dx * g.cell.0, -dy * g.cell.1), img, pane_px, false);
                    self.drag = Some(Drag::Pan { last: (col, row) });
                }
                Some(Drag::Thumb { vertical, grab }) => {
                    let bar = if vertical { self.bars.v } else { self.bars.h };
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
                        self.vp = self.vp.scroll_to(
                            !vertical,
                            px.offset_for_fraction(f),
                            img,
                            pane_px,
                            false,
                        );
                    }
                }
                None => return false,
            },
            MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                if self.drag.take().is_none() {
                    return false;
                }
            }
            _ => return false,
        }
        true
    }

    fn bar_down(
        &mut self,
        vertical: bool,
        track: scrollbar::Track,
        px: super::viewport::Track,
        pos: u16,
        g: &Geo,
    ) {
        let pane = if vertical { g.pane_px.h } else { g.pane_px.w };
        let signed = |d: f32| if vertical { (0.0, d) } else { (d, 0.0) };
        match track.press(pos) {
            scrollbar::TrackPress::Arrow(arrow) => {
                let d = arrow as f32 * pane / 10.0;
                self.vp = self.vp.pan_by(signed(d), g.img, g.pane_px, false);
            }
            scrollbar::TrackPress::Grab { grab, jump } => {
                self.drag = Some(Drag::Thumb { vertical, grab });
                if jump {
                    let room = f32::from(track.inner_len().saturating_sub(track.thumb_len));
                    let f = if room > 0.0 {
                        ((i32::from(pos) - grab - i32::from(track.arrow)) as f32 / room)
                            .clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    self.vp = self.vp.scroll_to(
                        !vertical,
                        px.offset_for_fraction(f),
                        g.img,
                        g.pane_px,
                        false,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_crop_is_placed_in_cells_clipped_to_the_pane() {
        let pane = Rect::new(10, 4, 20, 10);
        // Half-block: a 4x4 pixel image centered in a 20x20 pixel grid.
        let r = place(pane, (8.0, 8.0, 4.0, 4.0), (1.0, 2.0), false);
        assert_eq!(r, Rect::new(18, 8, 4, 2));
        // Never leaves the pane.
        let r = place(pane, (18.0, 18.0, 10.0, 10.0), (1.0, 2.0), false);
        assert!(pane.contains(ratatui::layout::Position::new(r.x, r.y)));
        assert!(r.right() <= pane.right() && r.bottom() <= pane.bottom());
        // A pixel protocol: cells of 10x20 pixels.
        let r = place(pane, (50.0, 40.0, 100.0, 80.0), (10.0, 20.0), true);
        assert_eq!(r, Rect::new(15, 6, 10, 4));
    }

    // --- a forced pixel-protocol capability -----------------------------

    use crate::terminal_image::{CellSize, Protocol};
    use crate::wide_edge::Clear;
    use ratatui::backend::TestBackend;
    use ratatui::widgets::Widget;
    use ratatui::Terminal;

    const BODY: Rect = Rect::new(0, 0, 40, 14);
    const INNER: Rect = Rect::new(1, 1, 38, 12);

    /// The thread-local pane, shown with a 32x32 picture on an iTerm2
    /// terminal. (Each test runs on its own thread, so each has its own.)
    fn ready_iterm2() -> Terminal<TestBackend> {
        init(Capability {
            protocol: Protocol::Iterm2,
            cell: CellSize {
                width: 8,
                height: 16,
            },
        });
        with(|s| {
            s.key = Some(Key {
                request: FileInsightRequest {
                    path: "pic.png".into(),
                    at: gitpoulpi_core::domain::Oid([1; 20]),
                    kind: FileInsightKind::Content,
                    worktree: false,
                    staged: false,
                },
                load: 1,
            });
            s.gen = 1;
            s.decoded = Some((1, Arc::new(Bitmap::new(32, 32, (200, 30, 30)))));
            s.phase = Phase::Ready;
        });
        Terminal::new(TestBackend::new(40, 14)).unwrap()
    }

    /// Draws one frame (the picture, then `popup` over it, then the
    /// `finish_frame` the event loop calls) and returns what is written to
    /// the terminal after it.
    fn frame(term: &mut Terminal<TestBackend>, popup: Option<Rect>) -> String {
        term.draw(|f| {
            render(f, BODY, INNER);
            if let Some(p) = popup {
                Clear.render(p, f.buffer_mut());
            }
            finish_frame(f.buffer_mut());
        })
        .unwrap();
        let mut out = Vec::new();
        write_pending(&mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    /// Draws until the raster worker's picture is on screen.
    fn until_sent(term: &mut Terminal<TestBackend>) -> String {
        let end = Instant::now() + Duration::from_secs(10);
        loop {
            let written = frame(term, None);
            if written.contains("]1337;File=") {
                return written;
            }
            assert!(Instant::now() < end, "the picture was never written");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// The escape sequence goes out after the frame, never inside it, and an
    /// open popup over the picture keeps it from being drawn on top of the
    /// popup; closing the popup sends it again.
    #[test]
    fn a_pixel_protocol_picture_is_written_after_the_frame_and_hidden_under_a_popup() {
        let mut term = ready_iterm2();
        let written = until_sent(&mut term);
        assert!(!written.is_empty());
        // Nothing changed: nothing is sent again.
        assert_eq!(frame(&mut term, None), "");
        // A palette-like popup over the whole pane: whatever the picture
        // sequence paints is repainted by the popup's own cells right after
        // it, so nothing of the picture shows through.
        let popup = Some(Rect::new(0, 0, 40, 14));
        let covered = frame(&mut term, popup);
        if let Some(end) = covered.rfind('\x07') {
            assert!(
                covered[end + 1..].contains("    "),
                "the popup's cells were not rewritten over the picture: {covered:?}"
            );
        }
        // Closed again: the picture comes back.
        let back = frame(&mut term, None);
        assert!(back.contains("]1337;File="), "{back:?}");
    }

    /// Zoom works from the geometry of the last frame: `+` changes the
    /// picture that is sent, `Fit` restores the first one.
    #[test]
    fn zooming_resends_a_different_picture_and_fit_restores_the_first() {
        let mut term = ready_iterm2();
        let first = until_sent(&mut term);
        assert!(run_zoom(ZoomOp::In));
        let zoomed = until_sent(&mut term);
        assert_ne!(first, zoomed);
        assert!(run_zoom(ZoomOp::Fit));
        let fit = until_sent(&mut term);
        assert_eq!(first, fit);
    }

    /// The wheel pans (Shift: sideways), Ctrl+wheel zooms at the pointer, a
    /// left drag pans, and the release ends it.
    #[test]
    fn the_wheel_and_a_drag_pan_and_ctrl_wheel_zooms() {
        use crossterm::event::MouseButton;
        let mut term = ready_iterm2();
        let _ = until_sent(&mut term);
        for _ in 0..8 {
            assert!(run_zoom(ZoomOp::In));
        }
        let _ = frame(&mut term, None);
        let center = || with(|s| s.vp.center);
        let none = KeyModifiers::NONE;
        let start = center();
        assert!(on_mouse(MouseEventKind::ScrollDown, 20, 6, none));
        assert!(center().1 > start.1, "down moves the view down");
        let after_wheel = center();
        assert!(on_mouse(
            MouseEventKind::ScrollDown,
            20,
            6,
            KeyModifiers::SHIFT
        ));
        assert!(center().0 > after_wheel.0, "Shift+wheel moves sideways");
        // A drag to the right moves the view to the left of the image.
        let before = center();
        assert!(on_mouse(
            MouseEventKind::Down(MouseButton::Left),
            20,
            6,
            none
        ));
        assert!(dragging());
        assert!(on_mouse(
            MouseEventKind::Drag(MouseButton::Left),
            24,
            6,
            none
        ));
        assert!(center().0 < before.0, "dragging right pans left");
        assert!(on_mouse(MouseEventKind::Up(MouseButton::Left), 24, 6, none));
        assert!(!dragging());
        // Ctrl+wheel up zooms in under the pointer.
        let scale = || {
            with(|s| {
                let g = s.geo.unwrap();
                s.vp.scale(g.img, g.pane_px, false)
            })
        };
        let before = scale();
        assert!(on_mouse(
            MouseEventKind::ScrollUp,
            10,
            4,
            KeyModifiers::CONTROL
        ));
        assert!(scale() > before);
    }

    /// A pending picture says nothing for the first 150 ms, then shows the
    /// decoding line with a spinner (maintainer rule 9).
    #[test]
    fn a_slow_decode_shows_its_line_only_after_150_ms() {
        let mut term = ready_iterm2();
        with(|s| {
            s.name = "pic.png".into();
            s.decoded = None;
            s.phase = Phase::Decoding {
                since: Instant::now(),
            };
        });
        let text = |term: &mut Terminal<TestBackend>| {
            frame(term, None);
            let buf = term.backend().buffer().clone();
            (0..buf.area.height)
                .map(|y| {
                    (0..buf.area.width)
                        .map(|x| buf[(x, y)].symbol().to_string())
                        .collect::<String>()
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert!(!text(&mut term).contains("Decoding"));
        with(|s| {
            s.phase = Phase::Decoding {
                since: Instant::now() - Duration::from_millis(400),
            }
        });
        assert!(text(&mut term).contains("Decoding pic.png"));
        assert!(animating());
    }

    #[test]
    fn image_extensions_are_matched_without_regard_to_case() {
        assert!(has_image_extension(std::path::Path::new("a/B.PNG")));
        assert!(has_image_extension(std::path::Path::new("x.webp")));
        assert!(!has_image_extension(std::path::Path::new("x.rs")));
        assert!(!has_image_extension(std::path::Path::new("png")));
    }
}
