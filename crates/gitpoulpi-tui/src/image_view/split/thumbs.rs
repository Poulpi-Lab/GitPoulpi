//! Thumbnails of the merge editor's binary conflict card (plan
//! the image-view design, slice S8): the pictures of stage 2
//! (A), stage 3 (B) and stage 1 (base) of a conflicted image, drawn inside the
//! card's side boxes through the same pipeline as the Diff view's panes
//! (`Slot`: bytes, decode, raster, half-block or a pixel protocol).
//!
//! Read-only and always Fit: no zoom, no pan, no hit rect. Choosing a side
//! stays the card's buttons. Looking at a conflict is not a mutation, so
//! there is no Undo story.
//!
//! The stage bytes come from the daemon as `BlobSource::Stage(n)`, through
//! the same `ImageBytes` request the Diff view uses.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::Instant;

use gitpoulpi_core::domain::Oid;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;

use super::{FileInsightKind, FileInsightOutcome, FileInsightPayload, FileInsightRequest, Slot};
use crate::avatar::DetailAvatarImage;
use crate::daemon_link::DiffContentSide;
use crate::image_view::halfblock;
use crate::image_view::pane::{
    has_image_extension, place, rasterise, rgb, view_key, Phase, RasterJob, SPINNER_AFTER,
};
use crate::image_view::raster::{self, Checker};
use crate::image_view::viewport::{Size, Viewport};
use crate::image_view::worker::Worker;
use crate::terminal_image::Capability;
use crate::{live_settings, th};
use gitpoulpi_i18n::msg::image_view as msg;

/// Which conflict the slots are about: the path and the blob ids of the three
/// stages (an id changes when the conflict is redone).
#[derive(Clone, PartialEq)]
struct Key {
    path: PathBuf,
    stages: Vec<(u8, Oid)>,
}

struct State {
    cap: Option<Capability>,
    key: Option<Key>,
    /// Stage 1 (base), 2 (A), 3 (B).
    slots: [Slot; 3],
    drawn: bool,
    outbox: Vec<FileInsightRequest>,
}

impl State {
    const fn new() -> State {
        State {
            cap: None,
            key: None,
            slots: [Slot::new(), Slot::new(), Slot::new()],
            drawn: false,
            outbox: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.key = None;
        for s in &mut self.slots {
            s.clear();
        }
        self.outbox.clear();
    }
}

thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State::new()) };
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

// --- lifecycle (called from the Diff view's, so the event loop is untouched) ---

pub(super) fn init(capability: Capability) {
    with(|s| s.cap = Some(capability));
}

/// A card that the previous frame did not draw is forgotten (frees bitmaps).
pub(super) fn begin_frame() {
    with(|s| {
        if !s.drawn && s.key.is_some() {
            s.reset();
        }
        s.drawn = false;
    });
}

pub(super) fn finish_frame(buf: &Buffer) {
    with(|s| {
        for slot in &mut s.slots {
            if let Some(i) = slot.image.as_mut() {
                i.finish_frame(buf);
            }
        }
    });
}

pub(super) fn write_pending<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
    with(|s| {
        for slot in &mut s.slots {
            if let Some(i) = slot.image.as_mut() {
                i.write_pending(out)?;
            }
        }
        Ok(())
    })
}

pub(super) fn take_requests() -> Vec<FileInsightRequest> {
    with(|s| std::mem::take(&mut s.outbox))
}

pub(super) fn animating() -> bool {
    with(|s| s.drawn && s.key.is_some() && s.slots.iter().any(Slot::pending))
}

/// Takes the answers to this card's reads; anything else is handed back.
pub(super) fn offer(outcome: FileInsightOutcome) -> Option<FileInsightOutcome> {
    if !matches!(
        outcome.request.kind,
        FileInsightKind::ImageBytes(DiffContentSide::Stage(_))
    ) {
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
            Err(e) => slot.phase = Phase::Card(vec![msg::read_failed(e.to_string())]),
        }
        None
    })
}

// --- the card's side ----------------------------------------------------------

/// Whether the card of `path` previews pictures.
pub(crate) fn wanted(path: &Path) -> bool {
    live_settings::image_view() && has_image_extension(path)
}

/// Declares the conflict being drawn (call every frame the card is drawn):
/// a different one starts the reads over.
pub(crate) fn follow(path: &Path, stages: &[(u8, Oid)]) {
    with(|s| {
        s.drawn = true;
        let key = Key {
            path: path.to_path_buf(),
            stages: stages.to_vec(),
        };
        if s.key.as_ref() == Some(&key) {
            return;
        }
        s.reset();
        for &(stage, _) in stages {
            let Some(slot) = s.slots.get_mut(usize::from(stage).wrapping_sub(1)) else {
                continue;
            };
            let request = FileInsightRequest {
                path: path.to_path_buf(),
                at: Oid([0u8; 20]),
                kind: FileInsightKind::ImageBytes(DiffContentSide::Stage(stage)),
                worktree: false,
                staged: false,
            };
            slot.present = true;
            slot.request = Some(request.clone());
            slot.phase = Phase::Fetching {
                since: Instant::now(),
            };
            s.outbox.push(request);
        }
        s.key = Some(key);
    });
}

/// Paints stage `stage`'s picture, Fit, inside `area`.
pub(crate) fn paint(buf: &mut Buffer, area: Rect, stage: u8) {
    with(|s| s.paint(buf, area, stage));
}

fn say(buf: &mut Buffer, area: Rect, text: &str) {
    if area.is_empty() {
        return;
    }
    let w = crate::ui_kit::display_width(text) as u16;
    let x = area.x + area.width.saturating_sub(w) / 2;
    crate::dialog_kit::put(
        buf,
        x,
        area.y + area.height / 2,
        text,
        area.width,
        Style::default().fg(th().ui.muted),
    );
}

impl State {
    fn paint(&mut self, buf: &mut Buffer, area: Rect, stage: u8) {
        let Some(i) = usize::from(stage).checked_sub(1).filter(|i| *i < 3) else {
            return;
        };
        let slot = &mut self.slots[i];
        slot.poll_decode();
        slot.poll_raster();
        if !slot.present || area.is_empty() {
            return;
        }
        if slot.not_image {
            say(buf, area, msg::NOT_AN_IMAGE.get());
            return;
        }
        match &slot.phase {
            Phase::Fetching { since } | Phase::Decoding { since } => {
                if since.elapsed() >= SPINNER_AFTER {
                    say(
                        buf,
                        area,
                        &crate::activity::spinner_frame(since.elapsed()).to_string(),
                    );
                }
                return;
            }
            Phase::Card(lines) => {
                say(buf, area, lines.first().map_or("", String::as_str));
                return;
            }
            Phase::Ready => {}
        }
        let Some((gen, bmp)) = slot.decoded.clone() else {
            return;
        };
        let pixel = self.cap.is_some_and(Capability::draws_images);
        if !pixel && live_settings::image_card_only() {
            say(buf, area, msg::TEXT_OFF.get());
            return;
        }
        let cell = match self.cap {
            Some(c) if c.draws_images() => (c.cell.width as f32, c.cell.height as f32),
            _ => (1.0, 2.0),
        };
        let img = Size::px(bmp.width, bmp.height);
        let pane_px = Size::new(
            f32::from(area.width) * cell.0,
            f32::from(area.height) * cell.1,
        );
        let view = Viewport::fit().view(img, pane_px, false);
        let (mut dw, mut dh) = raster::dst_size(&view);
        if pixel {
            (dw, dh) = raster::cap_output(dw, dh, raster::MAX_OUTPUT_PIXELS);
        }
        let pixelated_from = live_settings::image_pixelated_from();
        let checker = Checker::from_background(rgb(th().ui.bg, (24, 26, 38)));
        let mode = u32::from(pixel) ^ pixelated_from.to_bits().rotate_left(1);
        let rkey = view_key(gen, &view, dw, dh, mode, &checker);
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
            let at = place(area, shown.dst, cell, pixel);
            if pixel && slot.image.is_none() {
                if let Some(cap) = self.cap {
                    slot.image = Some(DetailAvatarImage::new(cap));
                }
            }
            let drawn = pixel
                && slot
                    .image
                    .as_mut()
                    .is_some_and(|im| im.render(buf, at, &shown.bitmap));
            if !drawn {
                halfblock::render(
                    buf,
                    at,
                    &shown.bitmap,
                    crate::color_depth::Depth::True,
                    rgb(th().ui.bg, (24, 26, 38)),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stages() -> Vec<(u8, Oid)> {
        vec![(1, Oid([1; 20])), (2, Oid([2; 20])), (3, Oid([3; 20]))]
    }

    #[test]
    fn following_a_conflict_asks_for_each_stage_once() {
        with(State::reset);
        follow(Path::new("a.png"), &stages());
        let reqs = take_requests();
        assert_eq!(reqs.len(), 3);
        assert!(matches!(
            reqs[1].kind,
            FileInsightKind::ImageBytes(DiffContentSide::Stage(2))
        ));
        // Drawn again, same conflict: no new reads.
        follow(Path::new("a.png"), &stages());
        assert!(take_requests().is_empty());
        // Another path restarts them.
        follow(Path::new("b.png"), &stages());
        assert_eq!(take_requests().len(), 3);
        with(State::reset);
    }

    #[test]
    fn a_card_that_is_not_drawn_is_forgotten() {
        with(State::reset);
        follow(Path::new("a.png"), &stages()[1..]);
        assert_eq!(take_requests().len(), 2);
        begin_frame(); // drawn was set by follow: kept
        assert!(with(|s| s.key.is_some()));
        begin_frame(); // nothing drew it in between
        assert!(with(|s| s.key.is_none()));
    }

    #[test]
    fn answers_for_other_reads_are_handed_back() {
        let other = FileInsightOutcome {
            request: FileInsightRequest {
                path: "a.png".into(),
                at: Oid([0; 20]),
                kind: FileInsightKind::ImageBytes(DiffContentSide::Commit),
                worktree: false,
                staged: false,
            },
            result: Err(gitpoulpi_core::Error::Backend("x".into())),
        };
        assert!(offer(other).is_some());
    }
}
