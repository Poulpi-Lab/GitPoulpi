//! The Log's author avatars as pixels: one small round picture per
//! visible commit row, in the Author column's two leading cells.
//!
//! ## One picture per row, not one strip for the column
//!
//! The Graph column is one image because every pixel of it is computed from
//! the same data at the same moment. The avatars are not: each row's picture
//! arrives on its own schedule (`service`) and each row's background changes
//! on its own (selection, hover). So each row is its **own** picture, and a
//! frame sends only the rows whose picture is not already on screen:
//!
//! - an idle frame sends nothing;
//! - the selection or hover moving sends the two rows it left and entered,
//!   where a strip would re-send the whole column;
//! - an avatar arriving sends the rows showing that author;
//! - a scroll sends the rows whose author (or background) at that screen
//!   line changed.
//!
//! And **no row is ever re-encoded twice**: the encoded escape sequence is
//! cached per (author, background, dimmed), the only inputs to the pixels
//! (the cell size is fixed for the process), so scrolling is bytes on the
//! wire and zero PNG work.
//!
//! ## The frame protocol
//!
//! The same three calls as `graph::GraphImage`, per row:
//! [`LogAvatarImages::render`] reserves the rows' cells after the table
//! has drawn them (a row whose picture is not ready is left alone -- the
//! table's initials badge is what shows); [`LogAvatarImages::finish_frame`],
//! after every popup, compares each row with what the terminal already
//! shows; [`LogAvatarImages::write_pending`], after `Terminal::draw`,
//! sends the rows that differ and then repaints whatever popup cells lie
//! over them. A row is re-sent when its picture, its position, the cells
//! some popup covers in it, or the screen size changed -- the Graph
//! column's rule, which is what keeps a closed tooltip from leaving its
//! text printed over an avatar.

use crate::th;
use std::collections::HashMap;

use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::Rect;
use ratatui::style::Color;

use gitpoulpi_config::AvatarPlacement;

use crate::terminal_image::{self, Bitmap, Capability, CellSize, Encoded};

/// Terminal cells one Log avatar is wide. One row tall. Two cells of a
/// 9x21 font are 18x21 pixels: the closest a one-row slot gets to square.
pub(crate) const SLOT_COLS: u16 = 2;

/// Encoded pictures kept before the cache is dropped and rebuilt. Every
/// entry is one author on one background - a few kilobytes - so this
/// is about a megabyte at worst, and a screen needs well under a hundred.
const MAX_ENCODED: usize = 256;

/// How the Log's Author cell shows the author's avatar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LogAvatarMode {
    /// `ui.avatars = "off"`, or `"graph"` with the picture in the commit
    /// circle: the name alone.
    Off,
    /// No image protocol (a text terminal, a pane multiplexer such as tmux or
    /// a herdr without Kitty graphics, `ui.graph_renderer = "text"`): a
    /// one-cell colored initial.
    Glyph,
    /// The terminal draws images: a [`SLOT_COLS`]-wide slot holding the
    /// picture once it is ready, and the two-letter initials until then.
    Picture,
}

/// Which of the three the Log uses. Same verdict as the Graph column's
/// renderer: the capability comes from `terminal_image::detect`, which
/// denies a multiplexer and honours `ui.graph_renderer`.
///
/// `ui.avatars = "graph"` puts the picture in the Graph column's commit
/// circle instead (`graph::raster`), so the Author column shows nothing,
/// except on a text graph, which has no circle to hold a picture: there the
/// Author column's glyph stands in, as for `author`.
pub(crate) fn log_avatar_mode(placement: AvatarPlacement, capability: Capability) -> LogAvatarMode {
    match placement {
        AvatarPlacement::Off => LogAvatarMode::Off,
        _ if !capability.draws_images() => LogAvatarMode::Glyph,
        AvatarPlacement::Author => LogAvatarMode::Picture,
        AvatarPlacement::Graph if !capability.draws_graph() => LogAvatarMode::Picture,
        AvatarPlacement::Graph => LogAvatarMode::Off,
    }
}

/// Which of the three the comparison cards use. Unlike the Log, the cards
/// have no graph circle to hold the picture, so `ui.avatars = "graph"` still
/// gives them the picture in their own slot (the Log's Author column stays
/// bare: it would show the same face twice in one row). Only `off` hides it.
pub(crate) fn card_avatar_mode(
    placement: AvatarPlacement,
    capability: Capability,
) -> LogAvatarMode {
    match placement {
        AvatarPlacement::Graph => log_avatar_mode(AvatarPlacement::Author, capability),
        other => log_avatar_mode(other, capability),
    }
}

/// The background a row's picture is drawn on, **when it has to carry
/// its own** (iTerm2: an image composites above text and nothing
/// repaints the cells under it). Under Kitty the picture is transparent
/// and the cells' own background shows through, so the key's tint is
/// `None` there and every row of one author shares one encode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RowTint {
    Plain,
    Hovered,
    Selected,
}

impl RowTint {
    /// Selection wins over hover, as in `log_table::commit_row`'s
    /// `sel_bg`.
    pub(crate) fn of(selected: bool, hovered: bool) -> RowTint {
        if selected {
            RowTint::Selected
        } else if hovered {
            RowTint::Hovered
        } else {
            RowTint::Plain
        }
    }

    fn rgb(self) -> (u8, u8, u8) {
        match self {
            // The Log's own background as the Graph column's picture
            // assumes it (`graph::raster::RasterOptions::new`).
            RowTint::Plain => (34, 36, 46),
            RowTint::Hovered => rgb_of(th().ui.hover),
            RowTint::Selected => rgb_of(th().ui.selection),
        }
    }
}

fn rgb_of(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (34, 36, 46),
    }
}

/// Everything one row's pixels depend on. The cell size and the
/// supersample factor are fixed for the process, so they are not here.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PictureKey {
    /// The author's Gravatar hash: the picture's identity.
    pub(crate) author: String,
    pub(crate) tint: Option<RowTint>,
    /// A search is running and this row is not a hit: the table dims the
    /// row's text, so the picture is faded toward the background too.
    pub(crate) dimmed: bool,
}

/// One row's picture for this frame.
pub(crate) struct AvatarSlot<'a> {
    pub(crate) area: Rect,
    pub(crate) key: PictureKey,
    pub(crate) bitmap: &'a Bitmap,
}

/// The slot for the Author column at `column_x` (`column_width` wide) on
/// screen line `y`: its [`SLOT_COLS`] leading cells. `None` when the
/// column is too narrow to keep one cell of gutter after the picture, or
/// when the slot would cross `right` (the pane's inner right edge) --
/// a picture must never run over the border.
pub(crate) fn slot_area(column_x: u16, column_width: u16, y: u16, right: u16) -> Option<Rect> {
    if column_width < SLOT_COLS + 1 || column_x.checked_add(SLOT_COLS)? > right {
        return None;
    }
    Some(Rect::new(column_x, y, SLOT_COLS, 1))
}

/// Draws `avatar` round-masked into a [`SLOT_COLS`] x 1 cell box, on
/// `tint`'s background. Pure: bitmap in, bitmap out.
///
/// The disc is as wide as the slot's shorter side less one assumed cell
/// pixel of air on each side, centered. Each output pixel is 3x3
/// supersampled: the fraction of samples inside the circle is its coverage
/// (the anti-aliased edge) and the average of the source pixels under those
/// samples is its color, which also downscales the fetched picture with a
/// box filter rather than nearest-neighbor.
pub(crate) fn compose(
    avatar: &Bitmap,
    tint: Option<RowTint>,
    dimmed: bool,
    cell: CellSize,
    scale: u32,
) -> Bitmap {
    let scale = scale.max(1);
    let width = u32::from(SLOT_COLS) * cell.width * scale;
    let height = cell.height * scale;
    // The color a dimmed picture fades toward when it has a backdrop of
    // its own; without one it fades toward transparency instead.
    let bg = tint.map_or((34, 36, 46), RowTint::rgb);
    let mut out = match tint {
        Some(t) => Bitmap::new(width, height, t.rgb()),
        None => Bitmap::transparent(width, height),
    };
    if avatar.width == 0 || avatar.height == 0 {
        return out;
    }
    let margin = scale as f32;
    let diameter = (width.min(height) as f32 - 2.0 * margin).max(1.0);
    let radius = diameter / 2.0;
    let (cx, cy) = (width as f32 / 2.0, height as f32 / 2.0);
    let (left, top) = (cx - radius, cy - radius);
    const SUB: u32 = 3;
    let x0 = left.floor().max(0.0) as u32;
    let x1 = ((cx + radius).ceil() as u32).min(width);
    let y0 = top.floor().max(0.0) as u32;
    let y1 = ((cy + radius).ceil() as u32).min(height);
    for y in y0..y1 {
        for x in x0..x1 {
            let mut inside = 0u32;
            let mut sum = [0u32; 3];
            for sy in 0..SUB {
                for sx in 0..SUB {
                    let px = x as f32 + (sx as f32 + 0.5) / SUB as f32;
                    let py = y as f32 + (sy as f32 + 0.5) / SUB as f32;
                    let (dx, dy) = (px - cx, py - cy);
                    if dx * dx + dy * dy > radius * radius {
                        continue;
                    }
                    inside += 1;
                    let u = (((px - left) / diameter) * avatar.width as f32) as u32;
                    let v = (((py - top) / diameter) * avatar.height as f32) as u32;
                    let (r, g, b) = avatar
                        .pixel(u.min(avatar.width - 1), v.min(avatar.height - 1))
                        .unwrap_or(bg);
                    sum[0] += u32::from(r);
                    sum[1] += u32::from(g);
                    sum[2] += u32::from(b);
                }
            }
            if inside == 0 {
                continue;
            }
            let mut color = (
                (sum[0] / inside) as u8,
                (sum[1] / inside) as u8,
                (sum[2] / inside) as u8,
            );
            if dimmed && tint.is_some() {
                color = (
                    halfway(color.0, bg.0),
                    halfway(color.1, bg.1),
                    halfway(color.2, bg.2),
                );
            }
            let coverage = inside as f32 / (SUB * SUB) as f32;
            let coverage = if dimmed && tint.is_none() {
                coverage * 0.5
            } else {
                coverage
            };
            out.blend(x, y, color, coverage);
        }
    }
    out
}

fn halfway(a: u8, b: u8) -> u8 {
    ((u16::from(a) + u16::from(b)) / 2) as u8
}

/// How tall a column of avatar slots must be for "all of them covered" to
/// mean a full-pane page (Settings) rather than a popup.
const PAGE_ROWS: u16 = 8;

/// What the terminal shows at one slot.
#[derive(Debug, PartialEq, Eq)]
struct OnScreen {
    key: PictureKey,
    /// The cells of the slot some popup drew over.
    covered: Vec<(u16, u16)>,
    /// A popup covers part of the slot, so no picture is on screen there:
    /// the table's own cells are (see `finish_frame`).
    hidden: bool,
    /// Transparent iTerm2 only: the slot's cells as last written (any
    /// change drops the picture there, so it is sent again).
    cells: Vec<Cell>,
}

/// One slot reserved by `render`.
struct Drawn {
    area: Rect,
    key: PictureKey,
    /// Blending iTerm2 only: the cells as `render` left them (blank,
    /// the row's background).
    expected: Vec<Cell>,
    /// The cells as the table drew them (the initials), before the
    /// picture claimed them: what a slot shows while it is hidden.
    table: Vec<Cell>,
}

/// The Log's per-row avatar pictures: an encode cache, what is on
/// screen, and what the frame just drawn needs sent. See the module
/// header for the frame protocol and why it is one picture per row.
pub(crate) struct LogAvatarImages {
    capability: Capability,
    scale: u32,
    /// One picture per key. Under Kitty each holds an image id the
    /// terminal keeps until the entry is dropped, so a picture that
    /// scrolls to another row costs only its placeholder cells.
    encoded: HashMap<PictureKey, Option<Encoded>>,
    /// Set by `render`, consumed by `finish_frame`.
    drawn: Vec<Drawn>,
    /// Keyed by slot: what the terminal is showing there.
    on_screen: HashMap<Rect, OnScreen>,
    /// The frame size `on_screen` is valid for - a resize clears the
    /// terminal, pictures included.
    screen: Option<Rect>,
    pending: Vec<(Rect, PictureKey)>,
    pending_over: Vec<(u16, u16, Cell)>,
    /// Cells written before the pictures (transparent iTerm2).
    pending_under: Vec<(u16, u16, Cell)>,
    /// The slots drawn last frame (and their Kitty ids): when a frame
    /// draws none (Settings, Diff, Home...) they are erased.
    shown: Vec<(Rect, Option<u32>)>,
    hide: Option<terminal_image::Hide>,
    /// A dialog over the live Log (not a full page): the pictures stay under
    /// it however many slots it covers; the cells it painted cover them.
    keep_under_overlay: bool,
}

impl LogAvatarImages {
    pub(crate) fn keep_under_overlay(&mut self, keep: bool) {
        self.keep_under_overlay = keep;
    }

    pub(crate) fn new(capability: Capability) -> LogAvatarImages {
        LogAvatarImages {
            capability,
            scale: terminal_image::supersample(),
            encoded: HashMap::new(),
            drawn: Vec::new(),
            on_screen: HashMap::new(),
            screen: None,
            pending: Vec::new(),
            pending_over: Vec::new(),
            pending_under: Vec::new(),
            shown: Vec::new(),
            hide: None,
            keep_under_overlay: false,
        }
    }

    /// Reserves each slot whose picture can be drawn, after the table has
    /// drawn the rows. A slot that cannot (no image protocol, an encode
    /// failure, off screen) is left as the table drew it - the initials.
    /// Returns how many slots were reserved.
    pub(crate) fn render(&mut self, buf: &mut Buffer, slots: &[AvatarSlot<'_>]) -> usize {
        if !self.capability.draws_images() {
            return 0;
        }
        if self.encoded.len() > MAX_ENCODED {
            self.encoded.clear();
        }
        let mut reserved = 0;
        for slot in slots {
            let area = slot.area.intersection(buf.area);
            if area != slot.area || area.is_empty() {
                continue;
            }
            let (capability, scale) = (self.capability, self.scale);
            // Kitty composites alpha over the cell background, so the
            // highlight is not part of the picture (see `RowTint`).
            let mut key = slot.key.clone();
            if matches!(
                capability.protocol,
                terminal_image::Protocol::Kitty
                    | terminal_image::Protocol::Iterm2Alpha
                    | terminal_image::Protocol::Sixel
            ) {
                key.tint = None;
            }
            let table: Vec<Cell> = terminal_image::cells_of(buf, area)
                .into_iter()
                .map(|(_, _, c)| c)
                .collect();
            let encoded = self.encoded.entry(key.clone()).or_insert_with(|| {
                let bitmap = compose(slot.bitmap, key.tint, key.dimmed, capability.cell, scale);
                terminal_image::encode_image(capability.protocol, &bitmap, area.width, area.height)
            });
            let Some(encoded) = encoded else {
                continue;
            };
            encoded.reserve(area, buf);
            let expected = if encoded.blends() {
                terminal_image::cells_of(buf, area)
                    .into_iter()
                    .map(|(_, _, c)| c)
                    .collect()
            } else {
                Vec::new()
            };
            self.drawn.push(Drawn {
                area,
                key,
                expected,
                table,
            });
            reserved += 1;
        }
        reserved
    }

    /// The slots drawn this frame span at least [`PAGE_ROWS`] rows.
    fn spans_a_page(&self) -> bool {
        let top = self.drawn.iter().map(|d| d.area.top()).min();
        let bottom = self.drawn.iter().map(|d| d.area.bottom()).max();
        matches!((top, bottom), (Some(t), Some(b)) if b - t >= PAGE_ROWS)
    }

    /// Call once, after every popup has been drawn.
    pub(crate) fn finish_frame(&mut self, buf: &Buffer) {
        self.pending.clear();
        self.pending_over.clear();
        self.pending_under.clear();
        self.hide = None;
        // Nothing drawn (Settings' page, the Diff view, Home...), or every
        // slot covered by a full-pane page: erase what was shown and send
        // nothing until a slot is visible again.
        // A popup or tooltip cannot cover a tall column of slots; a page
        // can. Only a column spanning `PAGE_ROWS` rows counts.
        let all_covered = !self.keep_under_overlay
            && self.spans_a_page()
            && (self.drawn.iter()).all(|d| {
                matches!(self.encoded.get(&d.key), Some(Some(e)) if e.mostly_overdrawn(buf, d.area))
            });
        if self.drawn.is_empty() || all_covered {
            if !self.shown.is_empty() {
                let mut hide = terminal_image::Hide::default();
                for (area, id) in std::mem::take(&mut self.shown) {
                    hide.add(buf, area, id);
                }
                for encoded in self.encoded.values().flatten() {
                    encoded.forget_upload();
                }
                self.hide = Some(hide);
            }
            self.drawn.clear();
            self.on_screen.clear();
            return;
        }
        self.shown = (self.drawn.iter())
            .map(|d| {
                let id = self
                    .encoded
                    .get(&d.key)
                    .and_then(|e| e.as_ref()?.kitty_id());
                (d.area, id)
            })
            .collect();
        if self.screen != Some(buf.area) {
            self.on_screen.clear();
            self.screen = Some(buf.area);
        }
        let mut now = HashMap::with_capacity(self.drawn.len());
        for Drawn {
            area,
            key,
            expected,
            table,
        } in std::mem::take(&mut self.drawn)
        {
            if let Some(Some(encoded)) = self.encoded.get(&key) {
                if encoded.places_itself() {
                    // Kitty: the placeholder cells are the picture; only
                    // the upload may be owed, once per picture.
                    if encoded.needs_upload(buf.area) {
                        self.pending.push((area, key));
                    }
                    continue;
                }
            }
            if matches!(self.encoded.get(&key), Some(Some(e)) if e.blends()) {
                let under = terminal_image::cells_of(buf, area);
                let cells: Vec<Cell> = under.iter().map(|(_, _, c)| c.clone()).collect();
                let overdrawn: Vec<bool> = under
                    .iter()
                    .zip(&expected)
                    .map(|(cell, plain)| &cell.2 != plain)
                    .collect();
                let hidden = overdrawn.contains(&true);
                let state = OnScreen {
                    key,
                    covered: Vec::new(),
                    hidden,
                    cells,
                };
                if self.on_screen.get(&area) != Some(&state) {
                    if hidden {
                        self.pending_over
                            .extend(hidden_cells(&under, &overdrawn, &table));
                    } else {
                        self.pending_under.extend(under);
                        self.pending.push((area, state.key.clone()));
                    }
                }
                now.insert(area, state);
                continue;
            }
            let under = terminal_image::cells_of(buf, area);
            let overdrawn: Vec<bool> = under
                .iter()
                .map(|(_, _, c)| c.diff_option != ratatui::buffer::CellDiffOption::Skip)
                .collect();
            let covered: Vec<(u16, u16)> = (under.iter().zip(&overdrawn))
                .filter(|(_, over)| **over)
                .map(|((x, y, _), _)| (*x, *y))
                .collect();
            let hidden = !covered.is_empty();
            let state = OnScreen {
                key,
                covered,
                hidden,
                cells: Vec::new(),
            };
            if self.on_screen.get(&area) != Some(&state) {
                if hidden {
                    self.pending_over
                        .extend(hidden_cells(&under, &overdrawn, &table));
                } else {
                    self.pending.push((area, state.key.clone()));
                }
            }
            now.insert(area, state);
        }
        // A slot not drawn this frame is gone: ratatui repaints its cells
        // itself, and a picture coming back there is sent afresh.
        self.on_screen = now;
    }

    /// Call after `Terminal::draw` returns: sends the rows the frame
    /// needs, and nothing at all otherwise.
    pub(crate) fn write_pending<W: std::io::Write>(&mut self, out: &mut W) -> std::io::Result<()> {
        if let Some(hide) = self.hide.take() {
            terminal_image::write_hide(out, &hide)?;
        }
        if self.pending.is_empty() && self.pending_over.is_empty() {
            return Ok(());
        }
        let pending = std::mem::take(&mut self.pending);
        let over = std::mem::take(&mut self.pending_over);
        let under = std::mem::take(&mut self.pending_under);
        let pictures: Vec<((u16, u16), &str)> = pending
            .iter()
            .filter_map(|(area, key)| {
                let sequence = self.encoded.get(key)?.as_ref()?.sequence();
                Some(((area.x, area.y), sequence))
            })
            .collect();
        let written = terminal_image::write_images_layered(out, &under, &pictures, &over);
        if written.is_err() {
            // Unknown what reached the terminal: send everything again.
            self.on_screen.clear();
            for encoded in self.encoded.values().flatten() {
                encoded.forget_upload();
            }
        }
        written
    }
}

/// What a slot shows while a popup covers part of it: the popup's cells
/// where it drew, the table's own cells (the initials) elsewhere. Writing
/// text into a cell is what drops the picture it carried, so every cell of
/// the slot is written - the ones ratatui leaves alone (`skip`) included --
/// and a half picture is never left beside the popup.
fn hidden_cells(
    under: &[(u16, u16, Cell)],
    overdrawn: &[bool],
    table: &[Cell],
) -> Vec<(u16, u16, Cell)> {
    (under.iter().zip(overdrawn).zip(table))
        .map(|(((x, y, now), over), plain)| (*x, *y, if *over { now } else { plain }.clone()))
        .collect()
}

#[cfg(test)]
#[path = "log_images_tests.rs"]
mod tests;
