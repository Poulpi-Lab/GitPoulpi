//! The commit graph as **pixels**.
//!
//! This is the module that answers the question every text-based attempt
//! failed at: *how do you join a round commit marker to its lane with no
//! gap?* You do not. You stop drawing characters and draw the column as an
//! image, where the disc and the strokes are the same ink laid down by the
//! same rasteriser into the same bitmap. Contiguity is then not a property
//! of a font -- it is arithmetic, and it is testable.
//!
//! **This module is pure.** Lane layout and a cell size in, an RGBA bitmap
//! out. No terminal, no escape sequence, no I/O. [`super::inline_image`]
//! turns the bitmap into something a terminal displays; the two are
//! separate so the interesting half can be tested without a terminal at
//! all -- which is a better test than a cell dump, not a worse one.
//!
//! ## Why it can promise what glyphs could not
//!
//! A glyph is drawn inside side bearings, so `○` inks 0.351 em of its cell
//! while `│` inks 1.214 em -- four to six bare pixel rows above and below
//! every node, and no code point closes them (see
//! `docs/developer/commit-graph.md` for the font measurements and for the
//! three shapes of that idea that were tried and rejected). Here the
//! stroke leaving a node stops **inside** the ring band rather than at its
//! edge, so the two overlap by half a stroke width. That is what
//! `every_node_is_connected_to_every_stroke_it_claims` checks, by
//! flood-filling 8-connected from the node's center and requiring the fill
//! to reach the cell edge on every side the node declares.
//!
//! ## The text fallback is a different contract
//!
//! [`super::part::TreeCell::span`] still renders the graph as characters
//! for terminals with no image support and for the pty harness. It draws
//! the same `○` `●` `◎` and **keeps the 4-6 px gap**, deliberately: a thin
//! honest line with a small gap, not a tinted background ribbon. The 8-connectivity guarantee
//! below is a claim about **this rasteriser's bitmap only**. The fallback
//! cannot satisfy it and is not expected to. *Do not "fix" the fallback by
//! putting a cell background back.*

use ratatui::style::Color;

use super::part::{LaneBackground, NodeKind, TreeCell, TreePart};
pub(crate) use crate::terminal_image::{Bitmap, CellSize};

mod metrics;
mod shape;

use metrics::*;
pub(crate) use metrics::{avatar_left_pad, avatar_picture_px, avatar_radius};
use shape::*;

/// One terminal line of the Graph column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RasterRow {
    /// The same `TreeCell`s the text renderer would draw, so the two paths
    /// can never disagree about the topology - only about how it is
    /// inked.
    ///
    /// What is pending rather than history - the WIP marker and its
    /// connector down to HEAD - is said per cell, by
    /// [`TreeCell::dashed`] and [`NodeKind::Wip`], and drawn dashed and
    /// dimmed. The text fallback says the same thing with `◌` and `┊`.
    pub cells: Vec<TreeCell>,
    /// Paint this row's band in the selection color. The selection band
    /// crosses the Graph column, and an image composites *above* text, so
    /// the band has to be drawn into the image - otherwise the selected
    /// row is the one row where the highlight stops.
    pub selected: bool,
    /// The commit author's picture, drawn inside this row's commit
    /// circle (`ui.avatars = "graph"`). `None` until it is fetched, and
    /// always for WIP and stash rows: the ordinary marker is drawn.
    pub avatar: Option<NodeAvatar>,
    /// Draw the ref-to-commit connector on this row (`ui.ref_connector`,
    /// only on a commit that carries a ref pill): a one-pixel line in the
    /// node's lane color from the image's left edge - where the Ref
    /// column's `─` padding stops - to the node. Laid **before** the
    /// row's cells, so every lane it passes is inked over it and stays on
    /// top. See [`draw_ref_connector`].
    pub ref_connector: bool,
}

impl RasterRow {
    /// An unselected row. Only the tests build rows this way; the Log
    /// always goes through [`super::visible_raster_rows`], which knows
    /// which row is selected.
    #[cfg(test)]
    pub(crate) fn new(cells: Vec<TreeCell>) -> RasterRow {
        RasterRow {
            cells,
            selected: false,
            avatar: None,
            ref_connector: false,
        }
    }
}

/// An author's picture for a commit circle. Shared, not copied: one
/// author's bitmap is the same `Arc` on every row they appear in.
///
/// Compared and hashed by `key` alone (the author's Gravatar hash, which
/// is also the picture's identity in `avatar::AvatarService`), so a
/// frame's "did anything change" check never compares pixels.
#[derive(Clone)]
pub(crate) struct NodeAvatar {
    pub key: String,
    pub bitmap: std::sync::Arc<Bitmap>,
}

impl PartialEq for NodeAvatar {
    fn eq(&self, other: &NodeAvatar) -> bool {
        self.key == other.key
    }
}

impl Eq for NodeAvatar {}

impl std::fmt::Debug for NodeAvatar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NodeAvatar").field(&self.key).finish()
    }
}

/// Everything the rasteriser needs besides the rows themselves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RasterOptions {
    pub cell: CellSize,
    /// How many terminal cells wide the image is. Rows shorter than this
    /// are padded; rows longer are clipped, exactly as the text column is.
    pub columns: u16,
    /// `None` (the default): the picture is **transparent** wherever
    /// nothing is drawn, so the terminal's real background - its theme,
    /// its translucency - and the cell backgrounds the table paints
    /// underneath (the selection and hover highlights) show through.
    ///
    /// `Some`: bake an opaque backdrop and the selection band into the
    /// picture. Only for protocols whose inline images cannot be relied on
    /// to composite alpha (iTerm2's `File=inline`), where the reserved
    /// cells are `skip` and nothing underneath is repainted anyway.
    pub backdrop: Option<Backdrop>,
    /// Bitmap pixels drawn per assumed cell pixel, per axis --
    /// [`crate::terminal_image::supersample`]. `1` means "the bitmap is
    /// exactly `cell` pixels per cell"; `2` means every length below is
    /// doubled, so the *same* picture is drawn at twice the resolution and
    /// the terminal downsamples it into the cell box instead of upscaling
    /// a too-small tile. Every geometry constant in this module is
    /// therefore expressed in **assumed cell pixels** and multiplied by
    /// this exactly once, at the point it becomes a coordinate - a
    /// constant that forgets to scale is a stroke that gets thinner as the
    /// resolution goes up.
    pub scale: u32,
    /// **A test hook, zero in production.** Extra pixels of air
    /// deliberately left between a stroke and the node it is supposed to
    /// meet.
    ///
    /// It exists so the connectivity test can be falsified on demand
    /// rather than by hand-editing the geometry: at `0.0` every node is
    /// 8-connected to its strokes, and at `3.0` the same check must go
    /// red. A test that has never been seen to fail is not evidence --
    /// `docs/developer/visual-verification.md`, "falsify every test".
    pub stroke_gap: f32,
    /// `ui.avatars = "graph"`: commit circles may hold the author's
    /// picture ([`RasterRow::avatar`]), so the whole graph is shifted
    /// right by [`avatar_left_pad`] to give the first lane's - wider than
    /// a cell - picture room. Off, not one pixel moves.
    pub avatars: bool,
    /// `ui.graph_lane_background`: `RowTint` paints a band from each
    /// commit's node to the right edge. `Off` (the default) paints nothing
    /// behind the lanes.
    pub lane_background: LaneBackground,
    /// `ui.graph_row_tint_strength`: the `row_tint` band's alpha. The
    /// right-edge bar stays at full color.
    pub tint_strength: gitpoulpi_config::GraphRowTintStrength,
}

/// The opaque colors of a non-transparent picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Backdrop {
    /// The pane background the column is drawn on (the theme's `bg`).
    pub background: (u8, u8, u8),
    /// The selection highlight, for rows that carry it (the theme's `selection`).
    pub selection: (u8, u8, u8),
}

impl Backdrop {
    /// What the active theme paints: its panel color as the column's
    /// background and its selection color for selected rows.
    pub(crate) fn theme() -> Backdrop {
        Backdrop {
            background: crate::theme::pill_blend_target(),
            selection: crate::theme::backdrop_selection(),
        }
    }
}

impl RasterOptions {
    pub(crate) fn new(cell: CellSize, columns: u16) -> RasterOptions {
        RasterOptions {
            cell,
            columns,
            backdrop: None,
            scale: 1,
            stroke_gap: 0.0,
            avatars: false,
            lane_background: LaneBackground::Off,
            tint_strength: gitpoulpi_config::GraphRowTintStrength::default(),
        }
    }

    /// The same options, with the `ui.graph_row_tint_strength` band.
    pub(crate) fn with_tint_strength(
        self,
        tint_strength: gitpoulpi_config::GraphRowTintStrength,
    ) -> RasterOptions {
        RasterOptions {
            tint_strength,
            ..self
        }
    }

    /// The same options, with the `ui.graph_lane_background` rail.
    pub(crate) fn with_lane_background(self, lane_background: LaneBackground) -> RasterOptions {
        RasterOptions {
            lane_background,
            ..self
        }
    }

    /// The same options, drawn on an opaque [`Backdrop`] instead of
    /// transparent.
    pub(crate) fn with_backdrop(self, backdrop: Option<Backdrop>) -> RasterOptions {
        RasterOptions { backdrop, ..self }
    }

    /// The same options, with room made for avatars in the circles.
    pub(crate) fn with_avatars(self, avatars: bool) -> RasterOptions {
        RasterOptions { avatars, ..self }
    }

    /// The same options, rasterised at `scale` bitmap pixels per cell
    /// pixel per axis.
    pub(crate) fn at_scale(self, scale: u32) -> RasterOptions {
        RasterOptions {
            scale: scale.max(1),
            ..self
        }
    }

    fn s(&self) -> f32 {
        self.scale.max(1) as f32
    }
}

/// Sub-samples per bitmap pixel per axis, for a given supersample factor.
///
/// At `scale = 1`, 3x3 samples is what makes a 7 px disc read as round
/// (2x2 leaves stair steps on the shoulders). At `scale >= 2` the
/// resolution does that job, so 2x2 gives 16 effective samples per cell
/// pixel; keeping 3x3 would make a changed frame cost 49 ms instead of 22
/// (measured, release) for the same picture.
fn subsamples(scale: u32) -> u32 {
    if scale >= 2 {
        2
    } else {
        3
    }
}

/// Lane layout in, bitmap out. The whole public surface of this module.
pub(crate) fn rasterize(rows: &[RasterRow], opt: &RasterOptions) -> Bitmap {
    let scale = opt.scale.max(1);
    let cw = opt.cell.width * scale;
    let ch = opt.cell.height * scale;
    let width = cw * u32::from(opt.columns.max(1));
    let height = ch * rows.len().max(1) as u32;
    let mut bmp = match opt.backdrop {
        Some(b) => Bitmap::new(width, height, b.background),
        None => Bitmap::transparent(width, height),
    };
    let samples = subsamples(scale);
    let pad = avatar_left_pad(opt);

    for (r, row) in rows.iter().enumerate() {
        let y0 = r as u32 * ch;
        if let (true, Some(b)) = (row.selected, opt.backdrop) {
            // From the commit node's cell to the right edge: the
            // highlight starts at the circle, the lanes to its left keep
            // the plain background. No node: the whole row.
            let from = super::connector::node_column(&row.cells)
                .map_or(0, |node| (node as u32 * cw).min(width));
            bmp.band(from, y0, y0 + ch, b.selection);
        }
        if opt.lane_background == LaneBackground::RowTint {
            if let Some(node) = super::connector::node_column(&row.cells) {
                if node < usize::from(opt.columns) {
                    // From the node's center to the right edge, in the
                    // commit's lane color at a low alpha: straight alpha,
                    // so the terminal's own background shows through it.
                    let x0 = pad + node as u32 * cw + cw / 2;
                    bmp.fill_rect_alpha(
                        x0,
                        y0,
                        bmp.width,
                        y0 + ch,
                        rgb_of(row.cells[node].color),
                        super::part::row_tint_mix(opt.tint_strength) as f32,
                    );
                    // The marker: a thin bar in the lane's color at the
                    // column's right edge, just left of the message, so
                    // the eye sees where the graph ends. Thin in
                    // *pixels* like the lanes, not a fraction of the cell.
                    let bar = ((ROW_BAR_PX * opt.s()).round() as u32).max(1);
                    bmp.fill_rect_alpha(
                        bmp.width.saturating_sub(bar),
                        y0,
                        bmp.width,
                        y0 + ch,
                        rgb_of(row.cells[node].color),
                        ROW_BAR_ALPHA,
                    );
                }
            }
        }
        // Under the cells, so every lane it passes is inked over it.
        if row.ref_connector {
            draw_ref_connector(&mut bmp, row, y0, opt, samples);
        }
        let mut avatar_node = None;
        for (c, cell) in row.cells.iter().enumerate() {
            if c >= usize::from(opt.columns) {
                break;
            }
            let color = rgb_of(cell.color);
            let x0 = pad + c as u32 * cw;
            if let (Some(avatar), TreePart::Node { kind, .. }) = (&row.avatar, cell.part) {
                if opt.avatars && takes_avatar(kind) {
                    avatar_node = Some((x0, color, avatar));
                }
            }
            for (shape, pending) in shapes_for(cell, x0, y0, opt) {
                let alpha = if pending { PENDING_ALPHA } else { 1.0 };
                draw(&mut bmp, &shape, color, alpha, samples);
            }
        }
        // After the whole row: the picture spills into the spacer cells
        // either side, and a stroke drawn later in the row (a merge's
        // run leaving to the right) would otherwise cut across the face.
        if let Some((x0, color, avatar)) = avatar_node {
            let center = (x0 as f32 + cw as f32 / 2.0, y0 as f32 + ch as f32 / 2.0);
            draw_avatar_node(&mut bmp, center, &avatar.bitmap, color, opt, samples);
        }
    }
    bmp
}

/// The row tint's right-edge bar: its width in assumed cell pixels (about
/// a lane's own thickness at the default zoom, and constant when the
/// terminal is zoomed) and its opacity.
const ROW_BAR_PX: f32 = 2.0;
const ROW_BAR_ALPHA: f32 = 0.9;

/// The ref-to-commit connector's thickness, in **assumed cell pixels**:
/// one. The lanes are [`stroke_half_width`] either side of their center --
/// about 2.6 px across at the maintainer's 9x21 cell - so this is visibly the
/// thinner line.
const REF_CONNECTOR_THICKNESS: f32 = 1.0;

/// The ref-to-commit connector, on the row whose band starts at `y0`: a
/// horizontal line [`REF_CONNECTOR_THICKNESS`] thick, in the node's lane
/// color, from the bitmap's left edge to the node's ring.
///
/// Drawn before the row's cells, so lanes crossing it are inked on top --
/// in pixels there is no need to skip a lane's cell the way the text
/// renderer does (`super::connector`), the lane simply covers the line.
/// It stops inside the ring band (never in a hollow marker's hole), and
/// its band is snapped to whole bitmap pixels so it is one crisp pixel row
/// at scale 1 rather than two half-covered ones.
fn draw_ref_connector(
    bmp: &mut Bitmap,
    row: &RasterRow,
    y0: u32,
    opt: &RasterOptions,
    samples: u32,
) {
    // A node past the column's right edge (a graph wider than its column)
    // still gets its line, clipped at the edge by `draw` -- exactly as the
    // text renderer's connector runs to the last visible cell.
    let Some(node) = super::connector::node_column(&row.cells) else {
        return;
    };
    let s = opt.s();
    let (cw, ch) = (opt.cell.width as f32 * s, opt.cell.height as f32 * s);
    let thickness = REF_CONNECTOR_THICKNESS * s;
    let top = y0 as f32 + ((ch - thickness) / 2.0).floor();
    let cy = top + thickness / 2.0;
    let pad = avatar_left_pad(opt) as f32;
    let node_cx = pad + node as f32 * cw + cw / 2.0;
    let end = node_cx - node_radius(opt) + ring_half_width(opt);
    if end <= 0.0 {
        return;
    }
    let color = rgb_of(row.cells[node].color);
    // From x = 0: the capsule's rounded left cap then falls off the
    // bitmap, so the line is square and full-strength at the column edge,
    // where it meets the Ref column's `─`. The right cap lands inside the
    // ring band, under the marker.
    draw(
        bmp,
        &Shape::Segment {
            a: (0.0, cy),
            b: (end, cy),
            half_width: thickness / 2.0,
        },
        color,
        1.0,
        samples,
    );
}

/// A commit circle holding a picture: the picture clipped to a disc of
/// [`avatar_radius`], anti-aliased the same way as every other shape
/// here, then a ring in the lane's color over its edge - a
/// colored border, which is what keeps the lane readable through the
/// node.
fn draw_avatar_node(
    bmp: &mut Bitmap,
    center: (f32, f32),
    picture: &Bitmap,
    lane: (u8, u8, u8),
    opt: &RasterOptions,
    samples: u32,
) {
    let radius = avatar_radius(opt);
    if picture.width > 0 && picture.height > 0 {
        let (left, top) = (center.0 - radius, center.1 - radius);
        let diameter = 2.0 * radius;
        let x0 = left.floor().max(0.0) as u32;
        let y0 = top.floor().max(0.0) as u32;
        let x1 = ((center.0 + radius).ceil().max(0.0) as u32 + 1).min(bmp.width);
        let y1 = ((center.1 + radius).ceil().max(0.0) as u32 + 1).min(bmp.height);
        let n = samples as f32;
        for y in y0..y1 {
            for x in x0..x1 {
                let mut inside = 0u32;
                let mut sum = [0u32; 3];
                for sy in 0..samples {
                    for sx in 0..samples {
                        let px = x as f32 + (sx as f32 + 0.5) / n;
                        let py = y as f32 + (sy as f32 + 0.5) / n;
                        let (dx, dy) = (px - center.0, py - center.1);
                        if dx * dx + dy * dy > radius * radius {
                            continue;
                        }
                        inside += 1;
                        // Bilinear, not nearest: the picture is (about)
                        // the circle's own size, so a fractional offset
                        // must blend neighbors, not snap to one.
                        let u = (px - left) / diameter * picture.width as f32;
                        let v = (py - top) / diameter * picture.height as f32;
                        let (r, g, b) = crate::image_decode::sample_bilinear(picture, u, v);
                        sum[0] += u32::from(r);
                        sum[1] += u32::from(g);
                        sum[2] += u32::from(b);
                    }
                }
                if let (Some(r), Some(g), Some(b)) = (
                    sum[0].checked_div(inside),
                    sum[1].checked_div(inside),
                    sum[2].checked_div(inside),
                ) {
                    let color = (r as u8, g as u8, b as u8);
                    bmp.blend(x, y, color, inside as f32 / (n * n));
                }
            }
        }
    }
    let rhw = ring_half_width(opt);
    draw(
        bmp,
        &Shape::Ring {
            center,
            radius: radius - rhw,
            half_width: rhw,
        },
        lane,
        1.0,
        samples,
    );
}

/// How strongly pending ink (the WIP marker and its connector) is laid
/// down: dimmed, because that work is not history yet.
const PENDING_ALPHA: f32 = 0.62;

/// The shapes one cell contributes, in bitmap pixel coordinates, each
/// flagged with whether it is pending (dimmed) ink.
///
/// **This is where contiguity is decided.** Every stroke leaving a node
/// stops at `radius - half_width/2`, i.e. *inside* the ring band rather
/// than at its outer edge, so the two overlap by half a stroke width and
/// no rounding can slip a blank pixel between them.
///
/// A stroke on one of the cell's [`TreeCell::dashed`] sides is dashed
/// instead of solid. The dashes are laid on a pattern anchored to the
/// **bitmap**, not to the stroke (see [`push_dashes`]), so the WIP
/// connector's pieces in consecutive rows - the marker's `down`, any
/// lane cells, HEAD's `up` - read as one evenly dashed line.
fn shapes_for(cell: &TreeCell, x0: u32, y0: u32, opt: &RasterOptions) -> Vec<(Shape, bool)> {
    let s = opt.s();
    let (cw, ch) = (opt.cell.width as f32 * s, opt.cell.height as f32 * s);
    let (x0, y0) = (x0 as f32, y0 as f32);
    let (cx, cy) = (x0 + cw / 2.0, y0 + ch / 2.0);
    let hw = stroke_half_width(opt);
    let mut shapes: Vec<Shape> = Vec::new();
    let mut pending: Vec<Shape> = Vec::new();
    let dashed = cell.dashed;

    let segment = |a: (f32, f32),
                   b: (f32, f32),
                   dash: bool,
                   shapes: &mut Vec<Shape>,
                   pending: &mut Vec<Shape>| {
        if dash {
            push_dashes(pending, a, b, hw, s);
        } else {
            shapes.push(Shape::Segment {
                a,
                b,
                half_width: hw,
            });
        }
    };
    let vertical_dashed = dashed.up || dashed.down;

    match cell.part {
        TreePart::Empty => {}
        TreePart::Vertical => segment(
            (cx, y0),
            (cx, y0 + ch),
            vertical_dashed,
            &mut shapes,
            &mut pending,
        ),
        TreePart::Horizontal => segment(
            (x0, cy),
            (x0 + cw, cy),
            dashed.left || dashed.right,
            &mut shapes,
            &mut pending,
        ),
        TreePart::Crossing => {
            segment(
                (cx, y0),
                (cx, y0 + ch),
                vertical_dashed,
                &mut shapes,
                &mut pending,
            );
            segment(
                (x0, cy),
                (x0 + cw, cy),
                dashed.left || dashed.right,
                &mut shapes,
                &mut pending,
            );
        }
        TreePart::BranchOutCrossed => {
            segment(
                (x0, cy),
                (x0 + cw, cy),
                dashed.left || dashed.right,
                &mut shapes,
                &mut pending,
            );
            segment(
                (cx, cy),
                (cx, y0 + ch),
                dashed.down,
                &mut shapes,
                &mut pending,
            );
        }
        TreePart::MergeInCrossed => {
            segment(
                (x0, cy),
                (x0 + cw, cy),
                dashed.left || dashed.right,
                &mut shapes,
                &mut pending,
            );
            segment((cx, y0), (cx, cy), dashed.up, &mut shapes, &mut pending);
        }
        TreePart::BranchOutLeft
        | TreePart::BranchOutRight
        | TreePart::MergeInLeft
        | TreePart::MergeInRight => {
            let sides = cell.part.sides();
            let sx = if sides.right { 1.0 } else { -1.0 };
            let sy = if sides.down { 1.0 } else { -1.0 };
            // A stash's connector landing in its base is dashed like the
            // rest of it: the two straight arms on the global dash
            // pattern, and the short fillet between them laid as pending
            // (dimmed) ink -- a dashed quarter-arc of two pixels would
            // read as noise, not as a dash.
            let dash_v = dashed.up || dashed.down;
            let dash_h = dashed.left || dashed.right;
            if dash_v || dash_h {
                let mut arms = Vec::new();
                elbow(&mut arms, (cx, cy), (x0, y0), (cw, ch), opt, sx, sy);
                for (i, arm) in arms.into_iter().enumerate() {
                    // `elbow` pushes the vertical arm, the horizontal arm,
                    // then the fillet, in that order.
                    match (i, arm) {
                        (0, Shape::Segment { a, b, .. }) if dash_v => {
                            push_dashes(&mut pending, a, b, hw, s)
                        }
                        (1, Shape::Segment { a, b, .. }) if dash_h => {
                            push_dashes(&mut pending, a, b, hw, s)
                        }
                        (_, shape) => pending.push(shape),
                    }
                }
            } else {
                elbow(&mut shapes, (cx, cy), (x0, y0), (cw, ch), opt, sx, sy);
            }
        }
        TreePart::Node { kind, sides } => {
            let r = node_radius(opt);
            let rhw = ring_half_width(opt);
            // Where a stroke stops: *inside* the ring band [r - 2*rhw, r]
            // rather than at its outer edge, so the marker and the lane
            // share ink and no rounding can slip a blank pixel between
            // them. `stroke_gap` pushes it back out past `r`, which is how
            // the connectivity test is falsified.
            let stop = r - rhw * 0.5 + opt.stroke_gap * s;
            if sides.up {
                segment(
                    (cx, y0),
                    (cx, cy - stop),
                    dashed.up,
                    &mut shapes,
                    &mut pending,
                );
            }
            if sides.down {
                segment(
                    (cx, cy + stop),
                    (cx, y0 + ch),
                    dashed.down,
                    &mut shapes,
                    &mut pending,
                );
            }
            if sides.left {
                segment(
                    (x0, cy),
                    (cx - stop, cy),
                    dashed.left,
                    &mut shapes,
                    &mut pending,
                );
            }
            if sides.right {
                segment(
                    (cx + stop, cy),
                    (x0 + cw, cy),
                    dashed.right,
                    &mut shapes,
                    &mut pending,
                );
            }
            let ring = Shape::Ring {
                center: (cx, cy),
                radius: r - rhw,
                half_width: rhw,
            };
            match kind {
                NodeKind::Commit => shapes.push(ring),
                NodeKind::Ref => shapes.push(Shape::Disc {
                    center: (cx, cy),
                    radius: r,
                }),
                NodeKind::Merge => {
                    // A bullseye: the same ring, plus a center dot sized
                    // to leave visible air inside the ring rather than
                    // growing into it.
                    shapes.push(ring);
                    shapes.push(Shape::Disc {
                        center: (cx, cy),
                        radius: ((r - rhw * 2.0) * 0.55).max(0.6),
                    });
                }
                // The WIP marker: the commit ring's own size and weight,
                // cut into dashes -- a dashed-outline circle.
                NodeKind::Wip => push_dashed_ring(&mut pending, (cx, cy), r - rhw, rhw),
                // A stash: a little box, drawn rather than typed.
                // The ring's own extent and weight, squared off -- the
                // outline's center line at `r - rhw`, its band `rhw` either
                // side -- so the strokes stop inside the band exactly as
                // they do for a ring, and the marker is no bigger than a
                // commit's.
                NodeKind::Stash => push_box(&mut shapes, (cx, cy), r - rhw, rhw),
            }
        }
    }
    shapes
        .into_iter()
        .map(|shape| (shape, false))
        .chain(pending.into_iter().map(|shape| (shape, true)))
        .collect()
}

/// A lane color as RGB.
///
/// `LANE_PALETTE` is explicit RGB, so the fallback is never taken in
/// practice; a palette-relative color has no pixel value this module
/// could know, and guessing one would be worse than drawing the lane in
/// the foreground gray.
fn rgb_of(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (150, 154, 172),
    }
}
