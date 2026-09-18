//! The rasteriser's sizes: stroke, marker, ring and corner radii, and the
//! avatar circle and its left pad.

use super::*;

/// The stroke half-width of a lane, in bitmap pixels.
///
/// Tied to the cell rather than fixed: at a large font a 2 px lane is a
/// hairline, and at a small one a 3 px lane swallows the marker. The
/// clamp is in **assumed cell pixels**, so it is applied before
/// `opt.scale` - clamping afterwards would make the lane a hairline at
/// 2x, which is the whole failure mode supersampling exists to avoid.
pub(super) fn stroke_half_width(opt: &RasterOptions) -> f32 {
    (opt.cell.width as f32 / 7.0).clamp(STROKE_HALF_MIN, STROKE_HALF_MAX) * opt.s()
}

/// The thinnest and the thickest lane half-width, in assumed cell pixels.
///
/// **The ceiling is the width at the default zoom** (`9 / 7 = 1.29`): a
/// lane has to stay about as thin in *pixels* when the terminal is zoomed
/// and its cells grow, not stay a fraction of the cell. Only the floor
/// depends on the cell, so a tiny font keeps a visible line.
pub(super) const STROKE_HALF_MIN: f32 = 0.85;
pub(super) const STROKE_HALF_MAX: f32 = 9.0 / 7.0;

/// The commit marker's diameter, as a fraction of the cell's **height**.
///
/// This is the font-metric fraction itself: `○`/`●` ink at 0.513 em in a
/// 1.321 em line box (Cascadia Code, measured as in
/// `docs/developer/commit-graph.md`'s glyph table), i.e. **0.513 / 1.321 =
/// 0.3884** of the line box, which is the cell height. Deriving the
/// marker from height makes it track the glyph's real size at any aspect
/// ratio.
///
/// At a squarer cell (height/width below about 2.27) this is the smaller
/// of the two values [`node_radius`] takes the `min` of;
/// [`NODE_RADIUS_WIDTH_CEILING`] covers the taller case.
pub(super) const NODE_DIAMETER_OF_HEIGHT: f32 = 0.3884;

/// The width-relative ceiling on the marker radius: never more than this
/// fraction of the cell width, however tall the cell.
///
/// At the maintainer's own 9x21 cell (height/width = 2.33) the height fraction
/// alone would give `0.1942 * 2.33 = 0.452`, past what the connectivity
/// check allows, so **this ceiling is the number in force at that cell**.
/// 0.44 is the largest value that keeps
/// `a_bare_marker_never_reaches_its_cells_edges_on_its_own` green at every
/// cell size and supersample factor tested (0.46 already fails at 9x21,
/// scale 1): `0.44 * 9 = 3.96` px radius, 7.92 px across.
pub(super) const NODE_RADIUS_WIDTH_CEILING: f32 = 0.44;

/// The commit marker's radius.
///
/// **The smaller of [`NODE_DIAMETER_OF_HEIGHT`] of the cell height and
/// [`NODE_RADIUS_WIDTH_CEILING`] of the cell width**: a squarer cell is
/// governed by the height fraction, a taller one by the width ceiling.
///
/// **The ceiling is the connectivity check, not taste.** Once a marker
/// alone reaches its cell's edge, `every_node_is_connected_to_every_stroke_it_claims`
/// no longer proves a stroke was drawn;
/// `a_bare_marker_never_reaches_its_cells_edges_on_its_own` holds that line.
pub(super) fn node_radius(opt: &RasterOptions) -> f32 {
    let half = stroke_half_width(opt);
    let from_height = opt.cell.height as f32 * NODE_DIAMETER_OF_HEIGHT / 2.0 * opt.s();
    let width_ceiling = opt.cell.width as f32 * NODE_RADIUS_WIDTH_CEILING * opt.s();
    from_height.min(width_ceiling).max(half * 2.4)
}

/// How thick a commit marker's ring is drawn - deliberately **thinner
/// than the lane stroke**.
///
/// At the maintainer's 9 px cell a marker is about 7 px across, and a ring
/// drawn at the lane's own stroke width leaves a 0.7 px hole: hollow and
/// filled then look identical, which is the whole distinction between an
/// ordinary commit and one a ref points at.
///
/// **0.80 of the lane stroke**: a 2.1 px ring that still leaves a 3 px
/// hole at the maintainer's cell size, which is what keeps hollow and filled
/// apart. Going further closes the hole and an ordinary commit starts
/// reading as a ref.
pub(super) fn ring_half_width(opt: &RasterOptions) -> f32 {
    (stroke_half_width(opt) * 0.80).max(0.6 * opt.s())
}

/// The corner radius of an elbow's fillet - what makes `╭` an actual
/// curve here, rather than the right angle a `┬` is forced into in a
/// character grid.
pub(super) fn corner_radius(opt: &RasterOptions) -> f32 {
    (opt.cell.width as f32 / 2.0)
        .min(opt.cell.height as f32 / 2.0)
        .max(1.0)
        * 0.85
        * opt.s()
}

/// The radius of a commit circle holding an avatar, in bitmap pixels.
///
/// **As big as the row allows, not as big as a bare marker**: a bare
/// marker (about 8 px across at 9x21) would make the picture a smudge. The
/// circle is the row's height less one assumed pixel of air either side,
/// capped at a little over one cell width of radius - two cells across,
/// the lane spacing, so it spills half a cell into the spacer column on
/// each side and stays clear of the next lane's stroke. At 9x21 that is
/// 18.9 px across, about 15 px of it picture inside the lane-colored ring.
///
/// Strokes still stop at the bare marker's ring, well inside this
/// circle, so the picture always overlaps the lanes it joins.
pub(crate) fn avatar_radius(opt: &RasterOptions) -> f32 {
    let from_height = opt.cell.height as f32 / 2.0 - 1.0;
    let from_width = opt.cell.width as f32 * 1.05;
    (from_height.min(from_width) * opt.s()).max(node_radius(opt))
}

/// The side, in bitmap pixels, of the square a commit node's picture is
/// drawn into for a terminal with `cell` pixels per cell, rasterised at
/// `scale`: the avatar circle's diameter, rounded up. What
/// `avatar::AvatarService` resamples every picture to, so the picture is
/// (about) 1:1 with the pixels it lands on instead of being stretched.
pub(crate) fn avatar_picture_px(cell: CellSize, scale: u32) -> u32 {
    let opt = RasterOptions::new(cell, 1).at_scale(scale);
    (2.0 * avatar_radius(&opt)).ceil().max(2.0) as u32
}

/// How far right the whole graph is drawn with avatars on, in bitmap
/// pixels: enough that the first lane's avatar circle (centered half a
/// cell in) is not cut by the column's left edge, plus one pixel of air.
/// The Log's text cells, hit targets and row arithmetic are untouched --
/// only the ink moves. The last lanes lose the same few pixels on the
/// right, where the column already clips a graph wider than itself.
pub(crate) fn avatar_left_pad(opt: &RasterOptions) -> u32 {
    if !opt.avatars {
        return 0;
    }
    let half_cell = opt.cell.width as f32 * opt.s() / 2.0;
    (avatar_radius(opt) - half_cell + opt.s()).ceil().max(0.0) as u32
}

/// Which node kinds get the picture: an ordinary commit and one a ref
/// points at. A merge keeps its bullseye (merges are drawn as a
/// small plain dot, not a face - the merge is not "authored" work in
/// the way its parents are), and a WIP or stash row has no author.
pub(super) fn takes_avatar(kind: NodeKind) -> bool {
    matches!(kind, NodeKind::Commit | NodeKind::Ref)
}
