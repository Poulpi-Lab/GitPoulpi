//! The Graph column's own use of [`crate::terminal_image`]: rasterise the
//! visible rows, encode them once, and let ratatui decide when to put
//! them back on the wire.
//!
//! Everything generic (capability detection, the cell-size query, the
//! PNG/base64/iTerm2 encoder, the buffer-cell widget) lives in
//! `crate::terminal_image`; what is here is only about *this* column.

use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::Rect;

use crate::terminal_image::{self, Capability, CellSize, Encoded};

use super::part::LaneBackground;
use super::raster::{rasterize, Backdrop, RasterOptions, RasterRow};

/// Detection, a one-entry cache of the last encode, and what is on screen.
///
/// One cache entry is enough because the input changes only when the
/// graph, the scroll position, the selection or the pane width does - and
/// when it does, the previous encode is worthless. A larger cache would
/// hold megabytes of base64 for frames that will never come back.
///
/// A frame goes through three calls, in this order:
///
/// 1. [`GraphImage::render`], inside the draw closure, where the table
///    draws the column - reserves the cells, so ratatui emits nothing
///    there;
/// 2. [`GraphImage::finish_frame`], at the very end of the same closure,
///    once every popup has been drawn - looks at what ended up over the
///    column and decides whether the picture has to go out again;
/// 3. [`GraphImage::write_pending`], after `Terminal::draw` has returned
///    - writes it, and the popup cells over it, if so.
///
/// The picture is re-sent when, and only when, what the terminal shows
/// would otherwise be wrong: the pixels changed, the screen was resized
/// (ratatui clears it), or the set of cells some popup drew over the
/// column changed - a popup appearing, moving or closing. An idle frame,
/// and a popup that sits still over an unchanged graph, cost nothing.
pub(crate) struct GraphImage {
    capability: Capability,
    /// Bitmap pixels per assumed cell pixel per axis. Read once, because
    /// it comes from the environment and cannot change under a running
    /// process - and because it is part of the cache key.
    scale: u32,
    /// `ui.avatars = "graph"`: rows may carry an author picture for their
    /// commit circle, and the graph is laid out with room for it
    /// (`RasterOptions::avatars`). Part of the cache key.
    avatars: bool,
    /// `ui.graph_lane_background`: part of the cache key.
    lane_background: LaneBackground,
    /// `ui.graph_row_tint_strength`: part of the cache key.
    tint_strength: gitpoulpi_config::GraphRowTintStrength,
    cached_key: Option<u64>,
    cached: Option<Encoded>,
    /// Set by `render`, consumed by `finish_frame`: where the picture was
    /// reserved this frame, and which picture.
    drawn: Option<(Rect, u64, Vec<Cell>)>,
    /// What the terminal is showing, as of the last frame written.
    on_screen: Option<OnScreen>,
    /// The picture's anchor and the cells to repaint over it, when the
    /// frame just drawn needs the picture sent.
    pending: Option<Pending>,
    /// Where the picture was reserved last frame: if this frame does not
    /// draw it (Settings, Diff, Home...), it has to be erased.
    shown: Option<Rect>,
    hide: Option<terminal_image::Hide>,
    /// An overlay that wants the Log and its picture visible behind it (the
    /// theme picker's live preview): the picture stays however much of it
    /// the overlay covers, and the overlay's cells are repainted over it.
    keep_under_overlay: bool,
}

/// A picture waiting to be written after the frame.
struct Pending {
    anchor: (u16, u16),
    /// `false` while an overlay hides the column: only `over` is written,
    /// to wipe whatever image fragments the terminal kept under it.
    picture: bool,
    /// Transparent iTerm2 only: the area's cells, written before the
    /// picture so no earlier picture shows through it.
    under: Vec<(u16, u16, Cell)>,
    /// The cells some popup drew over the picture, repainted after it.
    over: Vec<(u16, u16, Cell)>,
}

/// Everything that decides whether the terminal's copy of the picture is
/// still right.
#[derive(Debug, PartialEq, Eq)]
struct OnScreen {
    /// The whole frame: a resize makes ratatui clear the screen, which
    /// takes the picture with it even when the column itself did not move.
    screen: Rect,
    area: Rect,
    key: u64,
    /// The cells of `area` that something drew over this frame.
    covered: Vec<(u16, u16)>,
    /// An overlay covers at least [`HIDE_AT`] of the area, so the picture
    /// is not sent at all.
    hidden: bool,
    /// Transparent iTerm2 only: every cell of `area` as ratatui last wrote
    /// it. Any change means ratatui rewrote a cell and with it dropped
    /// the picture there, so the picture goes out again.
    cells: Vec<Cell>,
}

/// The share of the Graph column's cells an overlay must cover for the
/// picture to be hidden rather than laid under it: a tooltip or a menu
/// crossing the column keeps it, a full-screen page (Settings) or a
/// dialog over most of it drops it.
const HIDE_AT: (usize, usize) = (1, 2);

fn mostly_covered(covered: usize, total: usize) -> bool {
    total > 0 && covered * HIDE_AT.1 >= total * HIDE_AT.0
}

impl GraphImage {
    pub(crate) fn new(capability: Capability) -> GraphImage {
        GraphImage {
            capability,
            scale: terminal_image::supersample(),
            avatars: false,
            lane_background: LaneBackground::Off,
            tint_strength: gitpoulpi_config::GraphRowTintStrength::default(),
            cached_key: None,
            cached: None,
            drawn: None,
            on_screen: None,
            pending: None,
            shown: None,
            hide: None,
            keep_under_overlay: false,
        }
    }

    /// Set each frame before [`GraphImage::finish_frame`]: the overlay drawn
    /// this frame must not hide the picture, however large it is.
    pub(crate) fn keep_under_overlay(&mut self, keep: bool) {
        self.keep_under_overlay = keep;
    }

    pub(crate) fn capability(&self) -> Capability {
        self.capability
    }

    /// The terminal's cell size in pixels changed (a zoom): from now on
    /// the picture is drawn for it. Part of the cache key, so the next
    /// frame rasterises again.
    pub(crate) fn set_cell(&mut self, cell: CellSize) {
        self.capability.cell = cell;
    }

    /// Whether commit circles hold author pictures (`ui.avatars =
    /// "graph"`); live, re-read when Preferences closes.
    pub(crate) fn set_avatars(&mut self, avatars: bool) {
        self.avatars = avatars;
    }

    /// `ui.graph_lane_background`, live like the other settings.
    pub(crate) fn set_lane_background(&mut self, lane_background: LaneBackground) {
        self.lane_background = lane_background;
    }

    /// `ui.graph_row_tint_strength`, live like the other settings.
    pub(crate) fn set_tint_strength(
        &mut self,
        tint_strength: gitpoulpi_config::GraphRowTintStrength,
    ) {
        self.tint_strength = tint_strength;
    }

    /// Rasterises and encodes `rows` for `area`, reusing the last result
    /// when nothing that affects the pixels has changed. Returns the
    /// picture's key.
    fn encode_for(&mut self, rows: &[RasterRow], area: Rect) -> Option<u64> {
        if !self.capability.draws_images() || area.is_empty() || rows.is_empty() {
            return None;
        }
        let key = key_of(
            rows,
            area,
            self.capability.cell,
            self.scale,
            self.avatars,
            backdrop_for(self.capability.protocol).is_some(),
            self.lane_background,
            self.tint_strength,
        );
        if self.cached_key != Some(key) {
            let options = RasterOptions::new(self.capability.cell, area.width)
                .at_scale(self.scale)
                .with_avatars(self.avatars)
                .with_lane_background(self.lane_background)
                .with_tint_strength(self.tint_strength)
                .with_backdrop(backdrop_for(self.capability.protocol));
            let bitmap = rasterize(rows, &options);
            self.cached = terminal_image::encode_image(
                self.capability.protocol,
                &bitmap,
                area.width,
                rows.len() as u16,
            );
            self.cached_key = Some(key);
        }
        self.cached.is_some().then_some(key)
    }

    /// Reserves the Graph column in `area`. Returns `false` when this
    /// terminal has no image support, in which case the caller's text
    /// rendering is what the user sees and nothing here has touched the
    /// buffer.
    pub(crate) fn render(&mut self, buf: &mut Buffer, area: Rect, rows: &[RasterRow]) -> bool {
        // `area.height` is what the caller asked for; `rows` is what there
        // is. Drawing the shorter of the two keeps the image from running
        // past the pane's bottom border into the status bar.
        let area = Rect {
            height: area.height.min(rows.len() as u16),
            ..area
        };
        let Some(key) = self.encode_for(rows, area) else {
            return false;
        };
        let mut expected = Vec::new();
        if let Some(encoded) = &self.cached {
            encoded.reserve(area, buf);
            if encoded.blends() {
                expected = terminal_image::cells_of(buf, area)
                    .into_iter()
                    .map(|(_, _, cell)| cell)
                    .collect();
            }
        }
        self.drawn = Some((area, key, expected));
        true
    }

    /// Call at the very end of the draw closure, after every popup.
    ///
    /// A popup drawn over the column shows up here as a cell of the
    /// reserved area that is no longer `skip`: every widget that paints a
    /// popup `Clear`s it first, and `Clear` resets the cell.
    pub(crate) fn finish_frame(&mut self, buf: &Buffer) {
        self.pending = None;
        self.hide = None;
        let Some((area, key, expected)) = self.drawn.take() else {
            // Not drawn this frame: whatever comes back later must be
            // sent afresh, and what is on screen now has to be erased
            // (ratatui's diff skips cells that did not change).
            self.on_screen = None;
            if let Some(area) = self.shown.take() {
                let mut hide = terminal_image::Hide::default();
                let id = self.cached.as_ref().and_then(Encoded::kitty_id);
                hide.add(buf, area, id);
                if let Some(encoded) = &self.cached {
                    encoded.forget_upload();
                }
                self.hide = Some(hide);
            }
            return;
        };
        if let Some(encoded) = self.cached.as_ref().filter(|e| e.places_itself()) {
            // Kitty: the placeholder cells are the picture, and ratatui
            // has just written them. Only the upload may be owed.
            self.on_screen = None;
            if !self.keep_under_overlay && encoded.mostly_overdrawn(buf, area) {
                // A full-pane page (Settings) or a big dialog covers it:
                // delete the picture once, send it again when it is gone.
                if self.shown.take().is_some() {
                    let mut hide = terminal_image::Hide::default();
                    hide.add(buf, area, encoded.kitty_id());
                    encoded.forget_upload();
                    self.hide = Some(hide);
                }
                return;
            }
            self.shown = Some(area);
            if encoded.needs_upload(buf.area) {
                self.pending = Some(Pending {
                    anchor: (area.x, area.y),
                    picture: true,
                    under: Vec::new(),
                    over: Vec::new(),
                });
            }
            return;
        }
        self.shown = Some(area);
        if self.cached.as_ref().is_some_and(Encoded::blends) {
            // Transparent iTerm2: the cells are real, so the whole area is
            // the comparison. What differs from the plain reservation is a
            // popup, repainted over the picture.
            let under = terminal_image::cells_of(buf, area);
            let cells: Vec<Cell> = under.iter().map(|(_, _, c)| c.clone()).collect();
            let over: Vec<_> = under
                .iter()
                .zip(&expected)
                .filter(|((_, _, now), plain)| now != *plain)
                .map(|(cell, _)| cell.clone())
                .collect();
            let hidden = !self.keep_under_overlay && mostly_covered(over.len(), under.len());
            let now = OnScreen {
                screen: buf.area,
                area,
                key,
                covered: Vec::new(),
                hidden,
                cells,
            };
            if self.on_screen.as_ref() != Some(&now) {
                self.pending = Some(if hidden {
                    Pending {
                        anchor: (area.x, area.y),
                        picture: false,
                        under: Vec::new(),
                        over: under,
                    }
                } else {
                    Pending {
                        anchor: (area.x, area.y),
                        picture: true,
                        under,
                        over,
                    }
                });
            }
            self.on_screen = Some(now);
            return;
        }
        let mut covered = Vec::new();
        let mut over = Vec::new();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = &buf[(x, y)];
                if !cell.skip {
                    covered.push((x, y));
                    over.push((x, y, cell.clone()));
                }
            }
        }
        let hidden = !self.keep_under_overlay
            && mostly_covered(
                covered.len(),
                usize::from(area.width) * usize::from(area.height),
            );
        let now = OnScreen {
            screen: buf.area,
            area,
            key,
            covered,
            hidden,
            cells: Vec::new(),
        };
        if self.on_screen.as_ref() != Some(&now) {
            self.pending = Some(Pending {
                anchor: (area.x, area.y),
                picture: !hidden,
                under: Vec::new(),
                over,
            });
        }
        self.on_screen = Some(now);
    }

    /// Call after `Terminal::draw` has returned: sends the picture if the
    /// frame just drawn needs it, and costs nothing otherwise.
    pub(crate) fn write_pending<W: std::io::Write>(&mut self, out: &mut W) -> std::io::Result<()> {
        if let Some(hide) = self.hide.take() {
            terminal_image::write_hide(out, &hide)?;
        }
        let (
            Some(Pending {
                anchor,
                picture,
                under,
                over,
            }),
            Some(sequence),
        ) = (
            self.pending.take(),
            self.cached.as_ref().map(Encoded::sequence),
        )
        else {
            return Ok(());
        };
        let pictures: &[((u16, u16), &str)] = if picture { &[(anchor, sequence)] } else { &[] };
        let written = terminal_image::write_images_layered(out, &under, pictures, &over);
        if written.is_err() {
            // Whatever reached the terminal is unknown; make the next
            // frame send it again rather than trust it.
            self.on_screen = None;
            if let Some(encoded) = &self.cached {
                encoded.forget_upload();
            }
        }
        written
    }
}

/// Kitty composites the picture's alpha over the cells' own background
/// (selection and hover included), so the picture is transparent. iTerm2
/// inline images are written over `skip` cells that nothing repaints, and
/// not every implementation composites alpha, so that picture carries its
/// own opaque backdrop and selection band.
fn backdrop_for(protocol: terminal_image::Protocol) -> Option<Backdrop> {
    match protocol {
        terminal_image::Protocol::Kitty
        | terminal_image::Protocol::Iterm2Alpha
        | terminal_image::Protocol::Sixel => None,
        _ => Some(Backdrop::theme()),
    }
}

/// Everything the picture depends on, hashed. Note what is *not* here:
/// the commit ids, the author names, anything the column does not draw.
/// Hashing those would re-encode on every daemon poll for no visible
/// change.
#[allow(clippy::too_many_arguments)]
fn key_of(
    rows: &[RasterRow],
    area: Rect,
    cell: CellSize,
    scale: u32,
    avatars: bool,
    selection_baked: bool,
    lane_background: LaneBackground,
    tint_strength: gitpoulpi_config::GraphRowTintStrength,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    (lane_background as u8).hash(&mut hasher);
    (tint_strength as u8).hash(&mut hasher);
    // Colors baked into the picture (lane palette, backdrop) follow the theme.
    crate::theme_generation().hash(&mut hasher);
    area.width.hash(&mut hasher);
    area.height.hash(&mut hasher);
    cell.hash(&mut hasher);
    scale.hash(&mut hasher);
    avatars.hash(&mut hasher);
    for row in rows {
        // Only a baked selection band changes the pixels; a transparent
        // picture is the same one whichever row is selected.
        if selection_baked {
            row.selected.hash(&mut hasher);
        }
        // An author picture arriving is a different picture; its key is
        // the author, so the pixels are never hashed.
        row.avatar
            .as_ref()
            .map(|a| a.key.as_str())
            .hash(&mut hasher);
        for c in &row.cells {
            c.part.hash(&mut hasher);
            c.color.hash(&mut hasher);
            // The WIP connector: same part, dashed or not, is a
            // different picture.
            c.dashed.hash(&mut hasher);
        }
        // A separator, so two different row splits cannot hash alike.
        u16::MAX.hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{NodeKind, Sides, TreeCell, TreePart};
    use crate::terminal_image::Protocol;
    use crate::wide_edge::Clear;
    use ratatui::style::Color;
    use ratatui::widgets::{Paragraph, Widget};

    const SCREEN: Rect = Rect::new(0, 0, 24, 10);
    /// The Graph column: four cells wide, six rows tall, one cell in from
    /// the left edge and one row down, like the real one under the Log's
    /// border and header.
    const GRAPH: Rect = Rect::new(1, 1, 4, 6);

    fn image() -> GraphImage {
        GraphImage::new(Capability {
            protocol: Protocol::Iterm2,
            cell: CellSize::FALLBACK,
        })
    }

    fn rows(selected: usize) -> Vec<RasterRow> {
        (0..GRAPH.height as usize)
            .map(|r| RasterRow {
                cells: vec![TreeCell::new(
                    TreePart::Node {
                        kind: NodeKind::Commit,
                        sides: Sides::VERTICAL,
                    },
                    Color::Rgb(64, 156, 255),
                )],
                selected: r == selected,
                avatar: None,
                ref_connector: false,
            })
            .collect()
    }

    /// One whole frame, in the order the real draw loop does it: the
    /// graph, then whatever popup is open on top of it, then
    /// `finish_frame`, then - after ratatui's flush - `write_pending`.
    /// Returns the buffer ratatui would diff and the bytes written after
    /// it.
    fn frame(img: &mut GraphImage, rows: &[RasterRow], overlay: Option<Rect>) -> (Buffer, String) {
        let mut buf = Buffer::empty(SCREEN);
        assert!(img.render(&mut buf, GRAPH, rows));
        if let Some(popup) = overlay {
            Clear.render(popup, &mut buf);
            Paragraph::new("a tooltip over the graph").render(popup, &mut buf);
        }
        img.finish_frame(&buf);
        let mut out = Vec::new();
        img.write_pending(&mut out).unwrap();
        (buf, String::from_utf8(out).unwrap())
    }

    /// What ratatui would write for `next`, given `prev` is on screen:
    /// its own diff, which is exactly what `Terminal` hands the backend.
    fn emitted(prev: &Buffer, next: &Buffer) -> Vec<(u16, u16)> {
        prev.diff(next)
            .into_iter()
            .map(|(x, y, _)| (x, y))
            .collect()
    }

    fn sends_the_image(written: &str) -> bool {
        written.contains("\x1b]1337;File=") || written.contains("\x1bP0;1;0q")
    }

    /// Red without `row.avatar` in `key_of`: an arrived picture is never sent.
    #[test]
    fn an_avatar_arriving_in_a_commit_circle_sends_the_graph_again() {
        let mut img = image();
        img.set_avatars(true);
        let _ = frame(&mut img, &rows(0), None);
        let (_, idle) = frame(&mut img, &rows(0), None);
        assert_eq!(idle, "");
        let mut with_face = rows(0);
        with_face[2].avatar = Some(crate::graph::raster::NodeAvatar {
            key: "author-hash".into(),
            bitmap: std::sync::Arc::new(crate::terminal_image::Bitmap::new(4, 4, (200, 0, 0))),
        });
        let (_, written) = frame(&mut img, &with_face, None);
        assert!(sends_the_image(&written));
        let (_, again) = frame(&mut img, &with_face, None);
        assert_eq!(again, "", "an unchanged picture was sent again");
    }

    #[test]
    fn the_first_frame_sends_the_image() {
        let (_, written) = frame(&mut image(), &rows(0), None);
        assert!(sends_the_image(&written), "{written:?}");
    }

    /// The property this route rests on: an idle redraw (about thirty a
    /// second) writes nothing at all. The sequence lives in a cell's symbol,
    /// and `Buffer::diff` would take its tens of thousands of base64
    /// characters for a character that wide and re-emit every cell after it.
    #[test]
    fn an_unchanged_graph_costs_nothing_on_the_wire() {
        let mut img = image();
        let (one, _) = frame(&mut img, &rows(0), None);
        let (two, written) = frame(&mut img, &rows(0), None);
        assert_eq!(emitted(&one, &two), Vec::<(u16, u16)>::new());
        assert_eq!(written, "", "the unchanged image was sent again");
    }

    #[test]
    fn a_changed_graph_is_sent_again() {
        let mut img = image();
        let _ = frame(&mut img, &rows(0), None);
        let (_, written) = frame(&mut img, &rows(3), None);
        assert!(sends_the_image(&written));
    }

    #[test]
    fn a_resize_sends_the_image_again_even_when_the_column_did_not_move() {
        // ratatui clears the whole screen on a resize, picture included.
        let mut img = image();
        let _ = frame(&mut img, &rows(0), None);
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 12));
        assert!(img.render(&mut buf, GRAPH, &rows(0)));
        img.finish_frame(&buf);
        let mut out = Vec::new();
        img.write_pending(&mut out).unwrap();
        assert!(sends_the_image(&String::from_utf8(out).unwrap()));
    }

    /// An overlay over most of the column (Settings, a dialog) hides the
    /// picture: it is not sent, only the covering cells are rewritten
    /// once, and closing it sends the picture again.
    /// Red if `hidden` is always false: the picture is sent under the overlay.
    #[test]
    fn an_overlay_over_most_of_the_column_hides_the_picture_and_closing_it_brings_it_back() {
        let page = Some(Rect::new(0, 0, 24, 10));
        let mut img = image();
        let _ = frame(&mut img, &rows(0), None);
        let (_, covered) = frame(&mut img, &rows(0), page);
        assert!(!sends_the_image(&covered), "sent under an overlay");
        let (_, idle) = frame(&mut img, &rows(0), page);
        assert_eq!(idle, "", "a hidden picture costs nothing");
        let (_, back) = frame(&mut img, &rows(0), None);
        assert!(sends_the_image(&back), "not redrawn after the overlay");
    }

    /// A page that paints its text in one corner and leaves the rest of its
    /// `Clear`ed area blank (Settings under its last section) still covers
    /// the whole column. On the transparent iTerm2 path (WezTerm) a blank
    /// cell equals the plain reservation, so the cell comparison alone sees
    /// "nothing drew here" and lays the picture under the page.
    /// Red before the fix: the picture is sent under the blank page.
    #[test]
    fn a_mostly_blank_page_over_the_graph_hides_the_transparent_picture() {
        let mut img = GraphImage::new(Capability {
            protocol: Protocol::Iterm2Alpha,
            cell: CellSize::FALLBACK,
        });
        let page = |img: &mut GraphImage, with_page: bool| {
            let mut buf = Buffer::empty(SCREEN);
            assert!(img.render(&mut buf, GRAPH, &rows(0)));
            if with_page {
                Clear.render(SCREEN, &mut buf);
                Paragraph::new("Settings").render(Rect::new(10, 0, 12, 1), &mut buf);
            }
            img.finish_frame(&buf);
            let mut out = Vec::new();
            img.write_pending(&mut out).unwrap();
            String::from_utf8(out).unwrap()
        };
        assert!(sends_the_image(&page(&mut img, false)));
        let opened = page(&mut img, true);
        assert!(!sends_the_image(&opened), "sent under a blank page");
        assert!(!opened.is_empty(), "the covered cells were never rewritten");
        assert_eq!(page(&mut img, true), "", "a hidden picture costs nothing");
        assert!(sends_the_image(&page(&mut img, false)), "not back");
    }

    #[test]
    fn a_popup_that_stays_put_over_an_unchanged_graph_costs_nothing_either() {
        let popup = Some(Rect::new(2, 3, 12, 2));
        let mut img = image();
        let _ = frame(&mut img, &rows(0), None);
        let (one, _) = frame(&mut img, &rows(0), popup);
        let (two, written) = frame(&mut img, &rows(0), popup);
        assert_eq!(emitted(&one, &two), Vec::<(u16, u16)>::new());
        assert_eq!(written, "");
    }

    /// **The trace bug.** A popup over the Graph column writes text into
    /// cells the image owns, and the terminal replaces those pixels with the
    /// popup's characters. When it closes, every one of those cells is `skip`
    /// again and ratatui writes nothing there, so unless the picture is sent
    /// again the popup's text stays painted over the column.
    #[test]
    fn closing_a_popup_over_the_graph_sends_the_image_again() {
        let popup = Some(Rect::new(2, 3, 12, 2));
        let mut img = image();
        let _ = frame(&mut img, &rows(0), None);
        let _ = frame(&mut img, &rows(0), popup);
        let (_, written) = frame(&mut img, &rows(0), None);
        assert!(
            sends_the_image(&written),
            "the popup left the graph and the picture was not re-sent, so its text stays printed over the column"
        );
    }

    #[test]
    fn moving_a_popup_over_the_graph_sends_the_image_again() {
        let mut img = image();
        let _ = frame(&mut img, &rows(0), None);
        let _ = frame(&mut img, &rows(0), Some(Rect::new(2, 3, 12, 2)));
        let (_, written) = frame(&mut img, &rows(0), Some(Rect::new(2, 5, 12, 2)));
        assert!(
            sends_the_image(&written),
            "row 3 was uncovered by the move and nothing redrew the graph there"
        );
    }

    /// The other direction: the picture is re-sent under a popup that did
    /// not move (the selection changed, say). It lands over the popup's
    /// cells, and ratatui will not write them again because they did not
    /// change - so they are written after the picture, or the popup is
    /// erased wherever it crosses the graph.
    #[test]
    fn a_graph_redrawn_under_a_still_popup_repaints_the_popup_over_it() {
        let popup = Rect::new(2, 3, 12, 2);
        let mut img = image();
        let _ = frame(&mut img, &rows(0), Some(popup));
        let (_, written) = frame(&mut img, &rows(4), Some(popup));
        let image_at = written
            .find("\x1b]1337;File=")
            .expect("the changed graph was sent");
        let text_at = written.rfind("a t").expect("the popup was not repainted");
        assert!(image_at < text_at, "the popup went out before the picture");
    }
    /// A modelled terminal (writing a cell drops its picture tile): a popup
    /// crossing the column leaves no tile under its own cells, whatever the
    /// protocol, and closing it brings every tile back.
    #[test]
    fn on_a_terminal_a_popup_crossing_the_graph_has_no_picture_under_it() {
        use crate::terminal_image::vt_sim::Vt;
        for protocol in [Protocol::Iterm2, Protocol::Iterm2Alpha, Protocol::Sixel] {
            let mut img = GraphImage::new(Capability {
                protocol,
                cell: CellSize::FALLBACK,
            });
            let mut vt = Vt::new(SCREEN);
            let mut prev = Buffer::empty(SCREEN);
            let mut show = |img: &mut GraphImage, popup: Option<Rect>| {
                let (buf, written) = frame(img, &rows(0), popup);
                vt.draw(&prev, &buf);
                vt.feed(written.as_bytes());
                prev = buf;
                (0..SCREEN.height)
                    .flat_map(|y| (0..SCREEN.width).map(move |x| (x, y)))
                    .filter(|&(x, y)| vt.tile(x, y).is_some())
                    .collect::<Vec<_>>()
            };
            assert_eq!(show(&mut img, None).len(), 24, "{protocol:?}");
            for popup in [
                Rect::new(3, 2, 8, 1),
                Rect::new(3, 2, 8, 2),
                Rect::new(0, 4, 3, 2),
            ] {
                let tiles = show(&mut img, Some(popup));
                assert!(
                    tiles.iter().all(|&(x, y)| !popup.contains((x, y).into())),
                    "{protocol:?}: a picture over the popup {popup:?}: {tiles:?}"
                );
                assert!(
                    !tiles.is_empty(),
                    "{protocol:?}: the rest of the graph stays"
                );
            }
            assert_eq!(show(&mut img, None).len(), 24, "{protocol:?}: back");
        }
    }

    // ---- Kitty ----

    fn kitty_image() -> GraphImage {
        GraphImage::new(Capability {
            protocol: Protocol::Kitty,
            cell: CellSize::FALLBACK,
        })
    }

    fn uploads(written: &str) -> usize {
        written.matches("\x1b_Ga=T,U=1,").count()
    }

    /// The image id the Graph column's placeholder cells name.
    fn id_on_screen(buf: &Buffer) -> Color {
        buf[(GRAPH.x, GRAPH.y)].fg
    }

    /// Under Kitty a popup over the graph, and its closing, cost nothing
    /// here: the placeholders are cells, and ratatui restores them.
    /// Compare `closing_a_popup_over_the_graph_sends_the_image_again`.
    #[test]
    fn kitty_uploads_the_graph_once_and_popups_need_no_resend() {
        let popup = Some(Rect::new(2, 3, 12, 2));
        let mut img = kitty_image();
        let (first_buf, first) = frame(&mut img, &rows(0), None);
        assert_eq!(uploads(&first), 1, "{first:?}");
        let (_, idle) = frame(&mut img, &rows(0), None);
        assert_eq!(idle, "");
        let (_, opened) = frame(&mut img, &rows(0), popup);
        assert_eq!(opened, "");
        let (closed_buf, closed) = frame(&mut img, &rows(0), None);
        assert_eq!(closed, "");
        assert_eq!(
            emitted(&first_buf, &closed_buf),
            Vec::<(u16, u16)>::new(),
            "the placeholders are back exactly as they were"
        );
    }

    /// The selection lives in the cells' own background, not in the
    /// picture, so moving it re-encodes and re-uploads nothing.
    #[test]
    fn selection_does_not_change_a_transparent_picture() {
        let mut img = kitty_image();
        let (_, first) = frame(&mut img, &rows(0), None);
        assert_eq!(uploads(&first), 1);
        let (_, moved) = frame(&mut img, &rows(3), None);
        assert_eq!(moved, "", "{moved:?}");
    }

    /// A changed graph is a new picture under a new id - never a
    /// re-upload under an id cells already point at (herdr#1167) - and
    /// the old id is deleted once the frame has overwritten its cells.
    ///
    /// Red if `kitty::allocate` hands out one constant id.
    #[test]
    fn a_changed_kitty_graph_gets_a_new_id_and_the_old_one_is_deleted() {
        let mut img = kitty_image();
        let (one, _) = frame(&mut img, &rows(0), None);
        let mut drained = Vec::new();
        terminal_image::write_retired(&mut drained).unwrap();
        // Not the selection: a transparent picture is the same one
        // whichever row is selected.
        let mut changed = rows(0);
        changed[3].cells[0].color = Color::Rgb(255, 0, 0);
        let (two, written) = frame(&mut img, &changed, None);
        assert_eq!(uploads(&written), 1);
        let (Color::Rgb(r, g, b), new) = (id_on_screen(&one), id_on_screen(&two)) else {
            panic!("the placeholder carries no true-color id");
        };
        assert_ne!(Color::Rgb(r, g, b), new, "same id for a different picture");
        let old_id = (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b);
        let mut deletes = Vec::new();
        terminal_image::write_retired(&mut deletes).unwrap();
        assert_eq!(
            String::from_utf8(deletes).unwrap(),
            terminal_image::kitty::delete_command(old_id)
        );
    }

    fn alpha_image() -> GraphImage {
        GraphImage::new(Capability {
            protocol: Protocol::Iterm2Alpha,
            cell: CellSize::FALLBACK,
        })
    }

    /// WezTerm: the reserved cells are real (not `skip`), the picture goes
    /// out after the cells it sits on, and an idle redraw costs nothing.
    #[test]
    fn a_transparent_picture_sits_on_real_cells_and_idles_for_free() {
        let mut img = alpha_image();
        let (buf, written) = frame(&mut img, &rows(0), None);
        assert!(!buf[(GRAPH.x, GRAPH.y)].skip);
        assert!(sends_the_image(&written));
        // The cells under the picture are rewritten before it.
        let image_at = written.find("]1337;File=").unwrap();
        assert!(image_at > 20, "{written:?}");
        let (_, idle) = frame(&mut img, &rows(0), None);
        assert_eq!(idle, "");
    }

    /// A cell the table repainted (selection moved) drops the picture
    /// there, so the picture is sent again; the selection band is the
    /// cell's own background, never baked into the picture.
    #[test]
    fn a_repainted_cell_under_a_transparent_picture_sends_it_again() {
        let mut img = alpha_image();
        let mut first = Buffer::empty(SCREEN);
        assert!(img.render(&mut first, GRAPH, &rows(0)));
        img.finish_frame(&first);
        let mut sink = Vec::new();
        img.write_pending(&mut sink).unwrap();

        let mut second = Buffer::empty(SCREEN);
        second.set_style(
            Rect::new(GRAPH.x, GRAPH.y + 2, GRAPH.width, 1),
            ratatui::style::Style::default().bg(Color::Rgb(52, 58, 90)),
        );
        assert!(img.render(&mut second, GRAPH, &rows(0)));
        assert_eq!(second[(GRAPH.x, GRAPH.y + 2)].bg, Color::Rgb(52, 58, 90));
        img.finish_frame(&second);
        let mut out = Vec::new();
        img.write_pending(&mut out).unwrap();
        assert!(sends_the_image(&String::from_utf8(out).unwrap()));
    }

    fn sixel_image() -> GraphImage {
        GraphImage::new(Capability {
            protocol: Protocol::Sixel,
            cell: CellSize::FALLBACK,
        })
    }

    /// Sixel: the whole column is one picture exactly `cols * cell.width`
    /// by `rows * cell.height` pixels (no "fit to cells" exists), sent after
    /// the blank cells under it, and an idle redraw costs nothing.
    /// Red if `draws_graph` is false for sixel (nothing is sent) or the
    /// pixel size is not the cells' size.
    #[test]
    fn a_sixel_graph_is_one_picture_of_exactly_its_cells_and_idles_for_free() {
        let mut img = sixel_image();
        let (buf, written) = frame(&mut img, &rows(0), None);
        assert!(!buf[(GRAPH.x, GRAPH.y)].skip, "cells stay real blanks");
        let cell = terminal_image::cell_size();
        let raster = format!(
            "\"1;1;{};{}",
            u32::from(GRAPH.width) * cell.width,
            u32::from(GRAPH.height) * cell.height
        );
        assert!(written.contains(&raster), "{raster} not in {written:.80}");
        assert!(written.find("\x1bP0;1;0q").unwrap() > 20, "cells first");
        let (_, idle) = frame(&mut img, &rows(0), None);
        assert_eq!(idle, "");
    }

    /// Scrolling (different rows) re-sends; a repainted cell (selection)
    /// re-sends, because writing a cell erases the sixel pixels under it.
    #[test]
    fn a_sixel_graph_is_sent_again_when_rows_or_cells_change() {
        let mut img = sixel_image();
        let _ = frame(&mut img, &rows(0), None);
        // A transparent picture is the same whichever row is selected.
        let mut other = rows(0);
        other[3].cells[0].color = Color::Rgb(255, 0, 0);
        let (_, scrolled) = frame(&mut img, &other, None);
        assert!(sends_the_image(&scrolled), "{scrolled:?}");
        let mut second = Buffer::empty(SCREEN);
        second.set_style(
            Rect::new(GRAPH.x, GRAPH.y + 2, GRAPH.width, 1),
            ratatui::style::Style::default().bg(Color::Rgb(52, 58, 90)),
        );
        assert!(img.render(&mut second, GRAPH, &other));
        img.finish_frame(&second);
        let mut out = Vec::new();
        img.write_pending(&mut out).unwrap();
        assert!(sends_the_image(&String::from_utf8(out).unwrap()));
    }

    /// The speed rule: a full pane (50 rows, 14 cells wide, a face in
    /// every node) rasterises and encodes as sixel well inside a frame
    /// budget, and the bytes stay modest. Generous bounds (debug build):
    /// it catches an order-of-magnitude regression, not a benchmark.
    #[test]
    fn a_full_pane_of_sixel_graph_encodes_inside_a_frame_budget() {
        let cell = CellSize::sanitised(10, 20);
        let mut img = GraphImage::new(Capability {
            protocol: Protocol::Sixel,
            cell,
        });
        img.set_avatars(true);
        let mut face = crate::terminal_image::Bitmap::new(42, 42, (0, 0, 0));
        for y in 0..42 {
            for x in 0..42 {
                face.set_pixel(x, y, (x as u8 * 6, y as u8 * 6, 90));
            }
        }
        let face = std::sync::Arc::new(face);
        let mut full = rows(0);
        full.clear();
        for r in 0..50 {
            full.push(RasterRow {
                cells: vec![TreeCell::new(
                    TreePart::Node {
                        kind: NodeKind::Commit,
                        sides: Sides::VERTICAL,
                    },
                    Color::Rgb(64, 156, 255),
                )],
                selected: r == 3,
                avatar: Some(crate::graph::raster::NodeAvatar {
                    key: format!("a{}", r % 7),
                    bitmap: std::sync::Arc::clone(&face),
                }),
                ref_connector: false,
            });
        }
        let area = Rect::new(1, 1, 14, 50);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 60));
        let started = std::time::Instant::now();
        assert!(img.render(&mut buf, area, &full));
        let took = started.elapsed();
        let bytes = img.cached.as_ref().unwrap().sequence().len();
        eprintln!("sixel full pane: {took:?}, {bytes} bytes");
        assert!(took < std::time::Duration::from_secs(3), "{took:?}");
        assert!(bytes < 600 * 1024, "{bytes} bytes");
    }

    #[test]
    fn only_kitty_and_blending_iterm2_get_a_transparent_graph() {
        use crate::terminal_image::Protocol as P;
        assert!(backdrop_for(P::Kitty).is_none());
        assert!(backdrop_for(P::Iterm2Alpha).is_none());
        assert!(backdrop_for(P::Iterm2).is_some());
    }

    /// A frame that does not draw the Graph at all (Settings' own page,
    /// the Diff view, Home): returns what `write_pending` wrote.
    fn frame_without_graph(img: &mut GraphImage) -> String {
        let mut buf = Buffer::empty(SCREEN);
        // The new screen leaves the old graph's area blank.
        Paragraph::new("").render(SCREEN, &mut buf);
        img.finish_frame(&buf);
        let mut out = Vec::new();
        img.write_pending(&mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    /// ratatui's diff skips a blank cell that was blank before, so a
    /// transparent picture would stay; the area is rewritten instead.
    #[test]
    fn leaving_the_log_rewrites_the_area_a_transparent_picture_sat_on() {
        for mut img in [alpha_image(), image()] {
            let _ = frame(&mut img, &rows(0), None);
            let written = frame_without_graph(&mut img);
            assert!(!sends_the_image(&written));
            // One cursor move per rewritten row at least.
            assert!(
                written.matches("\x1b[").count() >= usize::from(GRAPH.height),
                "{written:?}"
            );
            // Once: the next frame without the Log costs nothing.
            assert_eq!(frame_without_graph(&mut img), "");
        }
    }

    /// A Kitty picture is deleted when the Log is not drawn, and uploaded
    /// again when it comes back.
    #[test]
    fn leaving_the_log_deletes_a_kitty_picture_and_it_comes_back_uploaded() {
        let mut img = kitty_image();
        let (buf, first) = frame(&mut img, &rows(0), None);
        assert_eq!(uploads(&first), 1);
        let id = match id_on_screen(&buf) {
            Color::Rgb(r, g, b) => (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b),
            other => panic!("{other:?}"),
        };
        let written = frame_without_graph(&mut img);
        assert!(written.contains(&format!("a=d,d=I,i={id}")), "{written:?}");
        let (_, back) = frame(&mut img, &rows(0), None);
        assert_eq!(uploads(&back), 1);
    }

    /// A full-pane page drawn over the Log (Settings) deletes a Kitty
    /// picture once, and sends nothing while it stays covered.
    #[test]
    fn a_page_over_the_log_deletes_a_kitty_picture_once() {
        let mut img = kitty_image();
        let _ = frame(&mut img, &rows(0), None);
        let (_, covered) = frame(&mut img, &rows(0), Some(SCREEN));
        assert!(covered.contains("a=d,d=I"), "{covered:?}");
        let (_, again) = frame(&mut img, &rows(0), Some(SCREEN));
        assert_eq!(again, "");
        let (_, closed) = frame(&mut img, &rows(0), None);
        assert_eq!(uploads(&closed), 1);
    }

    /// The cells the image layer rewrites over a hidden picture are cloned
    /// inside the draw closure, before the frame's own pass gives a cell
    /// with no background the theme's. Written as they were, a `Reset`
    /// background reaches the terminal as *its* default: a dark rectangle
    /// the width of the Graph column over a light theme's page or popup.
    /// Red before `as_painted`: the bytes carry `49m` and no theme color.
    #[test]
    fn the_cells_rewritten_over_a_hidden_picture_carry_the_themes_background() {
        use crate::theme::{override_theme, Theme};
        let light = Theme::from(&gitpoulpi_config::theme::preset("light").unwrap());
        let _g = override_theme(&light);
        let Color::Rgb(r, g, b) = light.ui.bg else {
            panic!("the Light theme paints a true-color background");
        };
        for (name, make) in [
            ("kitty", kitty_image as fn() -> GraphImage),
            ("iterm2", image),
            ("wezterm", alpha_image),
        ] {
            let mut img = make();
            let _ = frame(&mut img, &rows(0), None);
            let (_, written) = frame(&mut img, &rows(0), Some(Rect::new(0, 0, 24, 10)));
            assert!(!sends_the_image(&written), "{name}: sent under the page");
            assert!(
                written.contains(&format!("48;2;{r};{g};{b}")),
                "{name}: the theme background never reached the rewritten cells: {written:?}"
            );
            // ratatui's closing reset is not a cell.
            let cells = written.replace("\x1b[39m\x1b[49m\x1b[59m\x1b[0m", "");
            assert!(
                !cells.contains("49m"),
                "{name}: a cell was written on the terminal's own background: {written:?}"
            );
        }
    }

    /// The theme picker previews on the Log, so its popup must not hide the
    /// graph however much of the column it covers: no delete, the picture is
    /// still there, and the popup is repainted over it where the picture is
    /// sent. Red without `keep_under_overlay`: the picture is hidden.
    #[test]
    fn an_overlay_that_keeps_the_graph_never_hides_it() {
        let page = Some(Rect::new(0, 0, 24, 10));
        // Kitty: nothing is deleted and nothing is owed.
        let mut img = kitty_image();
        let _ = frame(&mut img, &rows(0), None);
        img.keep_under_overlay(true);
        let (buf, opened) = frame(&mut img, &rows(0), page);
        assert!(
            !opened.contains("a=d"),
            "deleted under the popup: {opened:?}"
        );
        assert!(img.shown.is_some(), "the picture was dropped");
        assert_eq!(buf[(GRAPH.x, GRAPH.y)].symbol(), " ", "the popup is drawn");
        // The popup closing needs no resend either.
        img.keep_under_overlay(true);
        let (_, closed) = frame(&mut img, &rows(0), None);
        assert_eq!(closed, "");
        // iTerm2: the picture goes out again under the popup, which is
        // then repainted over it.
        let mut img = image();
        let _ = frame(&mut img, &rows(0), None);
        img.keep_under_overlay(true);
        let (_, opened) = frame(&mut img, &rows(0), page);
        assert!(sends_the_image(&opened), "iterm2: hidden: {opened:?}");
        assert!(
            opened.find("]1337;File=") < opened.find("a tooltip").or(Some(usize::MAX)),
            "iterm2: the popup went out before the picture"
        );
    }
}

#[cfg(test)]
mod theme_tests {
    use super::*;
    use crate::theme::{override_theme, Theme};

    fn key() -> u64 {
        key_of(
            &[],
            Rect::new(0, 0, 4, 4),
            CellSize::FALLBACK,
            1,
            false,
            false,
            LaneBackground::Off,
            gitpoulpi_config::GraphRowTintStrength::default(),
        )
    }

    /// A theme switch changes the picture key, so a baked color is re-encoded.
    #[test]
    fn the_picture_key_follows_the_theme_generation() {
        let dark = Theme::dark();
        let _a = override_theme(&dark);
        let before = key();
        assert_eq!(before, key());
        let _b = override_theme(&dark);
        assert_ne!(before, key());
    }

    /// The non-alpha picture's backdrop is the theme's: Dark keeps the old
    /// panel and selection colors, a painted-background theme brings its own.
    #[test]
    fn the_backdrop_follows_the_theme() {
        let dark = Theme::dark();
        let _g = override_theme(&dark);
        assert_eq!(
            (Backdrop::theme().background, Backdrop::theme().selection),
            ((34, 36, 46), (52, 58, 90))
        );
        let nord = Theme::from(&gitpoulpi_config::theme::preset("nord").unwrap());
        let _n = override_theme(&nord);
        assert_eq!(Backdrop::theme().background, (0x2e, 0x34, 0x40));
        assert_eq!(Backdrop::theme().selection, (0x43, 0x4c, 0x5e));
    }
}
