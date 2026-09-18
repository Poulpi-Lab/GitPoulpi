//! The proof that the drawn graph is actually joined up.
//!
//! **Scope, stated once so nobody widens it by accident.** Everything here
//! is a claim about the *bitmap* [`super::raster::rasterize`] produces. It
//! is deliberately not a claim about the text fallback in
//! [`super::part`], which draws `○` `●` `◎` with the four-to-six pixels of
//! side bearing every font gives them and therefore **cannot** be
//! 8-connected. That gap is an accepted trade on terminals that cannot
//! display an image (see `part`'s module header). Do not "fix" the
//! fallback to make a test here pass -- there is no test here that asks it
//! to.

use ratatui::style::Color;

use super::part::{NodeKind, Sides, TreeCell, TreePart};
use super::raster::{rasterize, Backdrop, Bitmap, CellSize, RasterOptions, RasterRow};
use crate::terminal_image::{encode, Protocol};

/// The pane color the pictures are judged against: what a terminal with
/// the TUI's own theme shows behind a transparent picture.
const BG: (u8, u8, u8) = (34, 36, 46);

/// What the eye sees at `(x, y)`: the straight-alpha pixel composited
/// over [`BG`]. The rasteriser's default output is transparent where it
/// draws nothing, so every "is this background / is this ink" question is
/// asked of the composite - the same answer an opaque render gave.
fn shown(bmp: &Bitmap, x: u32, y: u32) -> Option<(u8, u8, u8)> {
    let (r, g, b, a) = bmp.rgba(x, y)?;
    let over = |c: u8, d: u8| {
        let a = f32::from(a) / 255.0;
        (f32::from(c) * a + f32::from(d) * (1.0 - a)).round() as u8
    };
    Some((over(r, BG.0), over(g, BG.1), over(b, BG.2)))
}

const LANE: Color = Color::Rgb(64, 156, 255);

fn cell(part: TreePart) -> TreeCell {
    TreeCell::new(part, LANE)
}

fn node(kind: NodeKind, sides: Sides) -> TreeCell {
    cell(TreePart::Node { kind, sides })
}

/// A pixel counts as ink when it differs from the background it was
/// painted on. Deliberately *any* difference rather than a threshold: the
/// question is "did the rasteriser put something here", and an
/// anti-aliased shoulder at 5% coverage is still something.
fn is_ink(bmp: &Bitmap, x: u32, y: u32, background: (u8, u8, u8)) -> bool {
    shown(bmp, x, y).is_some_and(|p| p != background)
}

/// A pixel counts as **solid** ink when it is at least half-way from the
/// background to [`LANE`] - what the eye reads as part of a line, as
/// opposed to the anti-aliased fringe around one.
///
/// This is what the connectivity check floods over, and deliberately not
/// [`is_ink`]. With "any difference" as the criterion, a fringe pixel at a
/// few percent coverage bridges a gap a pixel or so wide, and a marker
/// whose soft edge grazes the cell border "reaches" that border whether or
/// not a stroke was ever drawn - both of which made the check pass on a
/// detached disc once the markers grew to the size of the `○` glyph
/// (2026-09-23). Measured on the channel where the lane color differs
/// most from the background, which is the one where coverage is least
/// ambiguous.
fn is_solid(bmp: &Bitmap, x: u32, y: u32, background: (u8, u8, u8)) -> bool {
    let Some(p) = shown(bmp, x, y) else {
        return false;
    };
    let channels = [
        (p.0, background.0, LANE_RGB.0),
        (p.1, background.1, LANE_RGB.1),
        (p.2, background.2, LANE_RGB.2),
    ];
    let (px, bg, lane) = channels
        .into_iter()
        .max_by_key(|(_, bg, lane)| (i16::from(*lane) - i16::from(*bg)).abs())
        .expect("three channels");
    let span = f32::from(lane) - f32::from(bg);
    (f32::from(px) - f32::from(bg)) / span >= 0.5
}

const LANE_RGB: (u8, u8, u8) = (64, 156, 255);

/// Every solid pixel 8-connected to `(sx, sy)`.
fn flood(bmp: &Bitmap, background: (u8, u8, u8), sx: u32, sy: u32) -> Vec<bool> {
    let mut seen = vec![false; (bmp.width * bmp.height) as usize];
    if !is_solid(bmp, sx, sy, background) {
        return seen;
    }
    let mut stack = vec![(sx, sy)];
    seen[(sy * bmp.width + sx) as usize] = true;
    while let Some((x, y)) = stack.pop() {
        for dy in -1i64..=1 {
            for dx in -1i64..=1 {
                let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                if nx < 0 || ny < 0 || nx >= i64::from(bmp.width) || ny >= i64::from(bmp.height) {
                    continue;
                }
                let (nx, ny) = (nx as u32, ny as u32);
                let i = (ny * bmp.width + nx) as usize;
                if !seen[i] && is_solid(bmp, nx, ny, background) {
                    seen[i] = true;
                    stack.push((nx, ny));
                }
            }
        }
    }
    seen
}

/// The check the whole image route exists to satisfy.
///
/// For every node in `rows`: flood-fill 8-connected over *solid* ink (see
/// [`is_solid`]) from one pixel of its marker, then require the fill to have reached the cell's edge on
/// **every side the node declares**, and to contain every other inked
/// pixel of that cell. If a single pixel row or column of background sat
/// between the marker and a stroke, the fill would stop there and the edge
/// pixel would not be reached.
///
/// The seed is deliberately *not* the cell's center: an ordinary commit's
/// marker is a **ring**, so its center is background by design, and
/// seeding there would make this check quietly vacuous on exactly the
/// commonest node kind. It is the first inked pixel found in the cell
/// instead, and the "every inked pixel is in the fill" half then makes the
/// choice of seed irrelevant - except for a merge, whose bullseye center
/// dot is a second component on purpose.
///
/// Returns the failures, named, so a red run says which junction broke
/// rather than just `false`.
fn unreached_sides(rows: &[RasterRow], opt: &RasterOptions) -> Vec<String> {
    let bmp = rasterize(rows, opt);
    // In **bitmap** pixels, which is cell pixels times the supersample
    // factor. Reading `opt.cell` raw here would silently check the
    // top-left quarter of every cell at scale 2.
    let scale = opt.scale.max(1);
    let (cw, ch) = (opt.cell.width * scale, opt.cell.height * scale);
    let mut broken = Vec::new();
    for (r, row) in rows.iter().enumerate() {
        // A selected row paints its own band, so "the background" there is
        // the selection color, not the pane color.
        let background = match (row.selected, opt.backdrop) {
            (true, Some(b)) => b.selection,
            _ => BG,
        };
        for (c, tree_cell) in row.cells.iter().enumerate() {
            let TreePart::Node { sides, kind } = tree_cell.part else {
                continue;
            };
            let (x0, y0) = (c as u32 * cw, r as u32 * ch);
            let (cx, cy) = (x0 + cw / 2, y0 + ch / 2);
            let Some(seed) = (y0..y0 + ch)
                .flat_map(|y| (x0..x0 + cw).map(move |x| (x, y)))
                .find(|&(x, y)| is_solid(&bmp, x, y, background))
            else {
                broken.push(format!("row {r} col {c}: the node drew nothing at all"));
                continue;
            };
            let seen = flood(&bmp, background, seed.0, seed.1);
            let at = |x: u32, y: u32| seen[(y * bmp.width + x) as usize];
            // A merge's marker is a bullseye, so its center dot is a second
            // component **by design** -- that is what makes it read as a
            // double ring. Every other kind must be a single piece.
            if kind != NodeKind::Merge {
                for y in y0..y0 + ch {
                    for x in x0..x0 + cw {
                        if is_solid(&bmp, x, y, background) && !at(x, y) {
                            broken.push(format!(
                                "row {r} col {c}: ink at ({x},{y}) is a separate \
                                 component from the marker"
                            ));
                        }
                    }
                }
            }
            for (claimed, name, (x, y)) in [
                (sides.up, "up", (cx, y0)),
                (sides.down, "down", (cx, y0 + ch - 1)),
                (sides.left, "left", (x0, cy)),
                (sides.right, "right", (x0 + cw - 1, cy)),
            ] {
                if claimed && !at(x, y) {
                    broken.push(format!("row {r} col {c}: {name} edge not reached"));
                }
            }
        }
    }
    broken
}

/// A grid exercising every part that can appear next to a node: a plain
/// commit with a lane through it, a ref, a merge fanning out both ways,
/// crossings, elbows and tees.
fn exercise_rows() -> Vec<RasterRow> {
    let v = || cell(TreePart::Vertical);
    let e = || TreeCell::empty();
    vec![
        RasterRow::new(vec![
            node(NodeKind::Ref, Sides::new(false, true, false, true)),
            cell(TreePart::Horizontal),
            cell(TreePart::BranchOutRight),
        ]),
        RasterRow::new(vec![v(), e(), v()]),
        RasterRow::new(vec![
            node(NodeKind::Merge, Sides::ALL),
            cell(TreePart::Horizontal),
            cell(TreePart::MergeInRight),
        ]),
        RasterRow::new(vec![v(), e(), e()]),
        RasterRow {
            cells: vec![node(NodeKind::Commit, Sides::VERTICAL), e(), e()],
            selected: true,
            avatar: None,
            ref_connector: false,
        },
        RasterRow::new(vec![
            // `right` too: a crossing is a horizontal lane running past,
            // and its round cap bleeds into this cell by a stroke's
            // half width. With a thin stroke and a big marker that cap no
            // longer touches the disc on its own, so the node claims the
            // lane it sits next to.
            node(NodeKind::Commit, Sides::ALL),
            cell(TreePart::Crossing),
            cell(TreePart::MergeInCrossed),
        ]),
        RasterRow::new(vec![node(
            NodeKind::Commit,
            Sides::new(true, false, false, false),
        )]),
        // A stash's box with a stroke down and to both sides: the layout
        // only ever gives it `down`, but the box has to meet a stroke from
        // any side exactly as a ring does. (Not `up`: the row above has no
        // `down` to meet it, and a stroke's round cap would bleed into
        // that commit's cell as a stray component.)
        RasterRow::new(vec![
            node(NodeKind::Stash, Sides::new(false, true, true, true)),
            e(),
            e(),
        ]),
    ]
}

#[test]
fn every_node_is_connected_to_every_stroke_it_claims() {
    let rows = exercise_rows();
    for cell_size in [
        CellSize::FALLBACK,
        CellSize::sanitised(7, 15),
        CellSize::sanitised(12, 28),
        CellSize::sanitised(20, 40),
    ] {
        // Every supersample factor, not just 1: the whole geometry is
        // multiplied by `scale`, and a constant that forgot to be would
        // show up first as a stroke that no longer reaches its marker.
        for scale in [1, 2, 3] {
            let opt = RasterOptions::new(cell_size, 3).at_scale(scale);
            let broken = unreached_sides(&rows, &opt);
            assert!(
                broken.is_empty(),
                "at cell {}x{} scale {scale} the disc is not 8-connected to: {broken:#?}",
                cell_size.width,
                cell_size.height
            );
        }
    }
}

/// **What keeps the connectivity check meaningful as the markers grow.**
///
/// `every_node_is_connected_to_every_stroke_it_claims` asks whether a
/// node's fill reaches the edge of its cell on each side it claims. That
/// only proves a stroke was drawn if the marker *on its own* does not
/// reach the edge - otherwise the answer is "yes" with or without the
/// stroke. So: a bare marker, of every kind, at every cell size and
/// supersample factor, must leave the center pixel of all four cell edges
/// short of solid ink.
///
/// This is the real ceiling on [`super::raster`]'s marker radius. The
/// markers were raised to 0.425 of the cell width on 2026-09-23, to
/// match the 7.6 px `○` of the maintainer's Cascadia Code 11pt they replace;
/// falsified by raising it to 0.5, where the marker fills the edge column
/// and this goes red.
#[test]
fn a_bare_marker_never_reaches_its_cells_edges_on_its_own() {
    for cell_size in [
        CellSize::FALLBACK,
        CellSize::sanitised(7, 15),
        CellSize::sanitised(12, 28),
        CellSize::sanitised(20, 40),
    ] {
        for scale in [1, 2, 3] {
            for kind in [
                NodeKind::Commit,
                NodeKind::Ref,
                NodeKind::Merge,
                NodeKind::Stash,
            ] {
                let opt = RasterOptions::new(cell_size, 1).at_scale(scale);
                let bmp = rasterize(
                    &[RasterRow::new(vec![node(
                        kind,
                        Sides::new(false, false, false, false),
                    )])],
                    &opt,
                );
                let (w, h) = (bmp.width, bmp.height);
                for (name, (x, y)) in [
                    ("top", (w / 2, 0)),
                    ("bottom", (w / 2, h - 1)),
                    ("left", (0, h / 2)),
                    ("right", (w - 1, h / 2)),
                ] {
                    assert!(
                        !is_solid(&bmp, x, y, BG),
                        "at cell {}x{} scale {scale} a bare {kind:?} marker reaches the {name} edge by itself, so a missing stroke on that side would go unnoticed",
                        cell_size.width,
                        cell_size.height
                    );
                }
            }
        }
    }
}

/// **Supersampling must change the resolution and nothing else.**
///
/// The image is sized in *cells*, so a bitmap rasterised at 2x is scaled
/// back down into the same box - which only helps if the picture inside
/// it is the same picture. Every length in [`super::raster`] is expressed
/// in assumed cell pixels and multiplied by `scale` exactly once; miss one
/// and that feature gets relatively thinner as the factor goes up, which
/// no screenshot at a single factor can catch.
///
/// The measure is the inked *fraction* of the bitmap, which is
/// scale-invariant by construction. Falsified by dropping the `* opt.s()`
/// from `stroke_half_width`: the fraction at scale 2 then falls from 0.19
/// to 0.11 and this goes red.
#[test]
fn supersampling_changes_the_resolution_and_not_the_picture() {
    let rows = exercise_rows();
    let inked_fraction = |scale: u32| {
        let opt = RasterOptions::new(CellSize::FALLBACK, 3).at_scale(scale);
        let bmp = rasterize(&rows, &opt);
        let inked = (0..bmp.height)
            .flat_map(|y| (0..bmp.width).map(move |x| (x, y)))
            .filter(|&(x, y)| is_ink(&bmp, x, y, BG))
            .count();
        inked as f64 / (bmp.width * bmp.height) as f64
    };
    let one = inked_fraction(1);
    for scale in [2, 3] {
        let scaled = inked_fraction(scale);
        assert!(
            (scaled - one).abs() < 0.03,
            "at scale {scale} the graph inks {scaled:.3} of the column against \
             {one:.3} at scale 1 - some length is not being multiplied by the \
             supersample factor, so the picture changes shape with the resolution"
        );
    }
}

#[test]
fn a_supersampled_bitmap_is_bigger_but_still_covers_the_same_cells() {
    let rows: Vec<RasterRow> = (0..4).map(|_| RasterRow::new(vec![])).collect();
    let base = RasterOptions::new(CellSize::FALLBACK, 5);
    let one = rasterize(&rows, &base);
    let two = rasterize(&rows, &base.at_scale(2));
    assert_eq!((one.width, one.height), (9 * 5, 21 * 4));
    assert_eq!(
        (two.width, two.height),
        (9 * 5 * 2, 21 * 4 * 2),
        "the bitmap grows with the factor - the cell box it is scaled into \
         does not, which is the entire point"
    );
}

/// **The falsification, made permanent.**
///
/// `RasterOptions::stroke_gap` exists for exactly this: it pushes the
/// point where a stroke stops back *outside* the marker's outer edge,
/// which is precisely the defect every glyph-based attempt had - a few
/// pixels of bare background between the circle and its lane. The test
/// above must go red when that happens, or it is asserting nothing.
///
/// This is how the check was falsified when it was written: run with
/// `stroke_gap: 3.0` and `every_node_is_connected_to_every_stroke_it_claims`
/// reports `row 0 col 0: down edge not reached` and five more. It is kept
/// as a test rather than described in a comment so it keeps being true.
#[test]
fn a_deliberately_detached_disc_fails_the_connectivity_check() {
    let rows = exercise_rows();
    for scale in [1, 2, 3] {
        let opt = RasterOptions {
            stroke_gap: 3.0,
            ..RasterOptions::new(CellSize::FALLBACK, 3).at_scale(scale)
        };
        let broken = unreached_sides(&rows, &opt);
        assert!(
            !broken.is_empty(),
            "at scale {scale} a 3px gap between every stroke and its node went \
             undetected - the connectivity check is vacuous"
        );
    }
}

/// The gap has to be detected *because it is a gap*, not because the
/// shifted stroke happens to fall off the cell. Checks the geometry is
/// still sane at `stroke_gap: 3.0`: the strokes are drawn, there is simply
/// background between them and the disc.
#[test]
fn the_detached_variant_still_draws_both_the_disc_and_the_strokes() {
    let rows = vec![RasterRow::new(vec![node(
        NodeKind::Commit,
        Sides::VERTICAL,
    )])];
    let opt = RasterOptions {
        stroke_gap: 3.0,
        ..RasterOptions::new(CellSize::FALLBACK, 1)
    };
    let bmp = rasterize(&rows, &opt);
    let cw = CellSize::FALLBACK.width;
    let ch = CellSize::FALLBACK.height;
    assert!(
        is_ink(&bmp, cw / 2, 0, BG),
        "the stroke leaving the top of the cell is missing entirely"
    );
    assert!(
        is_ink(&bmp, cw / 2, ch / 2 - 3, BG),
        "the ring itself is missing"
    );
}

#[test]
fn a_ref_node_is_filled_and_a_commit_node_is_hollow() {
    let opt = RasterOptions::new(CellSize::sanitised(16, 32), 1);
    let filled = rasterize(
        &[RasterRow::new(vec![node(NodeKind::Ref, Sides::VERTICAL)])],
        &opt,
    );
    let hollow = rasterize(
        &[RasterRow::new(vec![node(
            NodeKind::Commit,
            Sides::VERTICAL,
        )])],
        &opt,
    );
    let center = (8u32, 16u32);
    assert_eq!(
        shown(&filled, center.0, center.1),
        Some((64, 156, 255)),
        "a ref's marker must be a solid disc"
    );
    assert_eq!(
        shown(&hollow, center.0, center.1),
        Some(BG),
        "an ordinary commit's marker must be a ring, i.e. hollow at the center"
    );
}

#[test]
fn a_merge_node_draws_two_concentric_rings() {
    // Walk the horizontal scan line through the marker's center and count
    // the runs of ink: ring, dot, ring. A plain ring would give two and a
    // filled disc one, so the count is exactly the thing that tells the
    // three kinds apart.
    let opt = RasterOptions::new(CellSize::sanitised(24, 48), 1);
    let bmp = rasterize(
        &[RasterRow::new(vec![node(
            NodeKind::Merge,
            Sides::new(false, false, false, false),
        )])],
        &opt,
    );
    let y = 24;
    let mut runs = 0;
    let mut previous = false;
    for x in 0..bmp.width {
        let ink = is_ink(&bmp, x, y, BG);
        if ink && !previous {
            runs += 1;
        }
        previous = ink;
    }
    assert_eq!(
        runs, 3,
        "a scan line through a bullseye crosses three runs of ink - the \
         ring on the left, the center dot, the ring on the right. Two \
         would mean the dot is missing, so a merge reads as an ordinary \
         commit; one would mean the dot has grown into the ring, so it \
         reads as a ref. Got {runs}"
    );
}

fn two_rows_second_selected() -> Vec<RasterRow> {
    vec![
        RasterRow::new(vec![TreeCell::empty()]),
        RasterRow {
            cells: vec![TreeCell::empty()],
            selected: true,
            avatar: None,
            ref_connector: false,
        },
    ]
}

/// The opaque (iTerm2) route still bakes the band: its reserved cells are
/// `skip`, so nothing underneath would show the highlight.
#[test]
fn an_opaque_backdrop_carries_the_selection_band_across_the_whole_column() {
    // No node on the row: nothing to start the band at, so all of it.
    let opt = RasterOptions::new(CellSize::FALLBACK, 4).with_backdrop(Some(Backdrop::theme()));
    let bmp = rasterize(&two_rows_second_selected(), &opt);
    let ch = CellSize::FALLBACK.height;
    for x in 0..bmp.width {
        assert_eq!(
            bmp.rgba(x, ch + ch / 2),
            Some((52, 58, 90, 255)),
            "the selection band must cover the Graph column at x={x}"
        );
        assert_eq!(bmp.rgba(x, ch / 2), Some((34, 36, 46, 255)));
    }
}

/// The selection band starts at the commit's node; the
/// lane cells to its left keep the plain background.
#[test]
fn the_baked_selection_band_starts_at_the_node_not_at_the_column_edge() {
    let opt = RasterOptions::new(CellSize::FALLBACK, 4).with_backdrop(Some(Backdrop::theme()));
    let rows = vec![RasterRow {
        cells: vec![
            TreeCell::empty(),
            TreeCell::empty(),
            node(NodeKind::Commit, Sides::NONE),
            TreeCell::empty(),
        ],
        selected: true,
        avatar: None,
        ref_connector: false,
    }];
    let bmp = rasterize(&rows, &opt);
    let cw = CellSize::FALLBACK.width;
    let y = CellSize::FALLBACK.height - 1;
    assert_eq!(bmp.rgba(cw, y), Some((34, 36, 46, 255)), "left of the node");
    assert_eq!(bmp.rgba(2 * cw, y), Some((52, 58, 90, 255)), "at the node");
    assert_eq!(bmp.rgba(bmp.width - 1, y), Some((52, 58, 90, 255)));
}

/// The default (Kitty) route: nothing is drawn where nothing is drawn, on
/// a selected row too - the table's cell background is the highlight.
#[test]
fn a_transparent_picture_has_no_selection_band_and_no_background() {
    let opt = RasterOptions::new(CellSize::FALLBACK, 4);
    let bmp = rasterize(&two_rows_second_selected(), &opt);
    for y in 0..bmp.height {
        for x in 0..bmp.width {
            assert_eq!(
                bmp.rgba(x, y).map(|p| p.3),
                Some(0),
                "untouched pixel ({x},{y}) must be fully transparent"
            );
        }
    }
}

/// Anti-aliased edges keep the **pure** lane color and let alpha carry
/// the coverage - no dark or light fringe on any background - and the
/// interior of a stroke is fully opaque.
#[test]
fn anti_aliased_edges_keep_the_pure_lane_color_with_partial_alpha() {
    for scale in [1, 2] {
        let opt = RasterOptions::new(CellSize::FALLBACK, 1).at_scale(scale);
        let bmp = rasterize(&[RasterRow::new(vec![cell(TreePart::Vertical)])], &opt);
        let (mut partial, mut opaque) = (0, 0);
        for y in 0..bmp.height {
            for x in 0..bmp.width {
                let (r, g, b, a) = bmp.rgba(x, y).unwrap();
                match a {
                    0 => {}
                    255 => {
                        opaque += 1;
                        assert_eq!((r, g, b), LANE_RGB, "interior at ({x},{y})");
                    }
                    _ => {
                        partial += 1;
                        assert_eq!(
                            (r, g, b),
                            LANE_RGB,
                            "edge pixel ({x},{y}) alpha {a} must be the pure lane color"
                        );
                    }
                }
            }
        }
        assert!(
            partial > 0,
            "a stroke has an anti-aliased edge (scale {scale})"
        );
        assert!(opaque > 0, "a stroke has a solid core (scale {scale})");
    }
}

/// Where two shapes overlap the result is composited, not overwritten:
/// an edge pixel under a second shape must not become transparent again.
#[test]
fn overlapping_shapes_composite_instead_of_clobbering() {
    let mut bmp = Bitmap::transparent(1, 1);
    bmp.blend(0, 0, LANE_RGB, 0.5);
    bmp.blend(0, 0, LANE_RGB, 0.5);
    let (r, g, b, a) = bmp.rgba(0, 0).unwrap();
    assert_eq!((r, g, b), LANE_RGB);
    assert!(
        (191..=192).contains(&a),
        "1 - 0.5 * 0.5 = 0.75 of 255, got {a}"
    );
    bmp.blend(0, 0, (255, 255, 255), 0.5);
    let (_, _, _, a2) = bmp.rgba(0, 0).unwrap();
    assert!(a2 > a);
}

#[test]
fn the_bitmap_is_exactly_the_pixel_size_of_the_cells_it_covers() {
    let opt = RasterOptions::new(CellSize::sanitised(11, 23), 7);
    let rows: Vec<RasterRow> = (0..5).map(|_| RasterRow::new(vec![])).collect();
    let bmp = rasterize(&rows, &opt);
    // Off by one cell here and the whole column is drawn shifted, which is
    // the one defect a unit test can catch and a screenshot argues about.
    assert_eq!((bmp.width, bmp.height), (11 * 7, 23 * 5));
    assert_eq!(bmp.data.len(), (11 * 7 * 23 * 5 * 4) as usize);
}

#[test]
fn cells_wider_than_the_column_are_clipped_not_wrapped() {
    let opt = RasterOptions::new(CellSize::FALLBACK, 1);
    let bmp = rasterize(
        &[RasterRow::new(vec![
            TreeCell::empty(),
            cell(TreePart::Vertical),
        ])],
        &opt,
    );
    assert_eq!(bmp.width, CellSize::FALLBACK.width);
    for x in 0..bmp.width {
        for y in 0..bmp.height {
            assert_eq!(
                shown(&bmp, x, y),
                Some(BG),
                "lane 1 is outside a one-cell-wide column and must not be drawn"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Which rows the picture covers
// ---------------------------------------------------------------------------
//
// The Log's row arithmetic is the convention in this repo that has caused
// the most real bugs, and an image drawn one row out is worse than no
// image: it lands on the wrong commits and looks deliberate. These pin
// `visible_raster_rows` against the same facts `log_index_to_row` uses.

/// Grid row `i` is made identifiable by its *width*: `i + 1` cells. That
/// is how [`rows_shown`] can say which grid row a screen row is showing
/// without the fixture needing real history behind it.
fn grid_of(rows: usize) -> Vec<super::layout::GraphRow> {
    (0..rows)
        .map(|i| {
            let mut cells = vec![TreeCell::empty(); i];
            cells.push(cell(TreePart::Node {
                kind: NodeKind::Commit,
                sides: Sides::VERTICAL,
            }));
            super::layout::GraphRow {
                cells,
                lane: i,
                color: LANE,
            }
        })
        .collect()
}

/// A WIP row, recognisable by its marker.
fn wip_row() -> super::layout::GraphRow {
    super::layout::GraphRow {
        cells: vec![cell(TreePart::Node {
            kind: NodeKind::Wip,
            sides: Sides::new(false, true, false, false),
        })],
        lane: 0,
        color: LANE,
    }
}

/// Which grid row each screen row shows - `None` for the WIP row, which
/// has no grid row behind it.
fn rows_shown(
    window: &super::ColumnWindow,
    grid: &[super::layout::GraphRow],
    wip: Option<&super::layout::GraphRow>,
) -> Vec<Option<usize>> {
    super::visible_raster_rows(
        grid,
        wip,
        &[],
        &super::LogRows::new(wip.is_some(), Vec::new()),
        window,
    )
    .iter()
    .map(|r| {
        let is_wip = r.cells.iter().any(|c| {
            matches!(
                c.part,
                TreePart::Node {
                    kind: NodeKind::Wip,
                    ..
                }
            )
        });
        (!is_wip).then(|| r.cells.len() - 1)
    })
    .collect()
}

#[test]
fn with_no_wip_row_screen_row_n_shows_grid_row_offset_plus_n() {
    let grid = grid_of(10);
    let window = super::ColumnWindow {
        rows: 4,
        offset: 3,
        selected: None,
    };
    assert_eq!(
        rows_shown(&window, &grid, None),
        vec![Some(3), Some(4), Some(5), Some(6)]
    );
}

/// **The WIP row is one screen row, like a commit.** It used to be two
/// (marker, then a stem), which shifted every commit below it by one line
/// and made the Log the one table in this TUI whose rows were not all the
/// same height - the source of most of its off-by-one clicks.
#[test]
fn the_wip_row_takes_exactly_one_screen_row() {
    let grid = grid_of(10);
    let wip = wip_row();
    let window = super::ColumnWindow {
        rows: 3,
        offset: 0,
        selected: None,
    };
    assert_eq!(
        rows_shown(&window, &grid, Some(&wip)),
        vec![None, Some(0), Some(1)]
    );
}

#[test]
fn once_the_table_has_scrolled_the_wip_row_is_not_drawn_at_all() {
    // Past `offset == 0` the WIP row is above the window, so the picture
    // must not reserve a line for it either.
    let grid = grid_of(10);
    let wip = wip_row();
    let window = super::ColumnWindow {
        rows: 3,
        offset: 2,
        selected: None,
    };
    assert_eq!(
        rows_shown(&window, &grid, Some(&wip)),
        vec![Some(1), Some(2), Some(3)],
        "display index 2 with a WIP row at 0 is grid row 1"
    );
}

#[test]
fn the_selection_lands_on_the_selected_display_index_not_the_grid_index() {
    let grid = grid_of(10);
    let wip = wip_row();
    let window = super::ColumnWindow {
        rows: 4,
        offset: 0,
        selected: Some(2),
    };
    let selected: Vec<bool> = super::visible_raster_rows(
        &grid,
        Some(&wip),
        &[],
        &super::LogRows::new(true, Vec::new()),
        &window,
    )
    .iter()
    .map(|r| r.selected)
    .collect();
    // The WIP row, then display 1 (grid 0), then display 2 (grid 1).
    assert_eq!(selected, vec![false, false, true, false]);
}

#[test]
fn selecting_the_wip_row_bands_the_wip_row() {
    let grid = grid_of(10);
    let wip = wip_row();
    let window = super::ColumnWindow {
        rows: 3,
        offset: 0,
        selected: Some(0),
    };
    let selected: Vec<bool> = super::visible_raster_rows(
        &grid,
        Some(&wip),
        &[],
        &super::LogRows::new(true, Vec::new()),
        &window,
    )
    .iter()
    .map(|r| r.selected)
    .collect();
    assert_eq!(selected, vec![true, false, false]);
}

#[test]
fn a_window_taller_than_the_history_stops_at_the_last_commit() {
    let grid = grid_of(2);
    let window = super::ColumnWindow {
        rows: 40,
        offset: 0,
        selected: None,
    };
    // Not 40 blank rows: the image is sized to what there is, so it never
    // paints an empty band under the last commit.
    assert_eq!(
        super::visible_raster_rows(&grid, None, &[], &super::LogRows::default(), &window).len(),
        2
    );
}

/// **The cost of drawing the column as pixels, as a number rather than an
/// assumption.**
///
/// The standing objection to this whole route is "the Log scrolls, so the
/// image has to be re-transmitted on every keypress". It does not, twice
/// over - [`super::inline_image::GraphImage`] caches the encode against a
/// hash of exactly what is drawn, and ratatui's own diffing declines to
/// re-emit an unchanged cell - but even the uncached path has to be
/// affordable, because a scroll genuinely does change every row.
///
/// So this measures the worst realistic frame: a full-height pane, eight
/// lanes wide, rasterised *and* PNG-encoded *and* base64'd from scratch,
/// at **the supersample factor production actually uses** (2 - see
/// [`crate::terminal_image::supersample`]). Only a frame whose pixels
/// actually changed pays it at all; an unchanged column costs a hash.
///
/// Measured on a development machine, release, 2026-09-23, after the
/// supersampling batch:
///
/// | scale | bitmap | rasterise | encode | total | bytes on the wire |
/// |---|---|---|---|---|---|
/// | 1 | 135x1260 | 2.7 ms | 0.4 ms | **3.1 ms** | 22 KB |
/// | 2 | 270x2520 | 5.7 ms | 1.0 ms | **6.8 ms** | 39 KB |
/// | 3 | 405x3780 | 11.2 ms | 1.7 ms | **12.9 ms** | 56 KB |
///
/// The starting point that morning was **12.7 ms for scale 1 alone**, and
/// a naive 2x came out at 24.4 ms. Three changes closed that, and they
/// are worth knowing because each one is a thing to not undo:
///
/// - `Bitmap::new` fills by doubling instead of pushing three bytes two
///   million times (the single biggest item);
/// - `Bitmap::blend` writes straight through at full coverage, which is
///   most of the ink;
/// - the per-pixel subsample grid drops from 3x3 to 2x2 once `scale >= 2`,
///   where the resolution is already doing that job
///   ([`super::raster::subsamples`]).
///
/// So the production path - supersampled 2x, which is what makes the
/// markers survive a `Ctrl +` - is now **cheaper than the un-supersampled
/// path was before**, and 3x remains available at roughly double.
///
/// The ceiling is deliberately loose, because a CI runner under load is
/// not a performance regression and a test that flakes gets deleted. What
/// it catches is an order of magnitude - somebody making the rasteriser
/// quadratic in the pane height, or turning the PNG compression up to
/// `Best`.
#[test]
fn a_full_pane_of_graph_rasterises_and_encodes_inside_a_frame_budget() {
    let lanes: Vec<TreeCell> = (0..15)
        .map(|i| {
            if i % 2 == 0 {
                cell(TreePart::Vertical)
            } else {
                TreeCell::empty()
            }
        })
        .collect();
    let mut rows: Vec<RasterRow> = (0..60).map(|_| RasterRow::new(lanes.clone())).collect();
    rows[30].cells[0] = node(NodeKind::Merge, Sides::ALL);
    rows[30].selected = true;
    rows[10].cells[4] = node(NodeKind::Ref, Sides::VERTICAL);

    for scale in [1, 2, 3] {
        let opt = RasterOptions::new(CellSize::FALLBACK, 15).at_scale(scale);
        // The fastest of five runs: the ceiling guards the cost of the code,
        // and one wall-clock sample on a machine running the whole workspace
        // suite measures the neighbours as much as the code (431 ms against
        // a 400 ms ceiling, for a pane that takes 140 ms alone).
        let mut best: Option<(std::time::Duration, std::time::Duration, _, _)> = None;
        for _ in 0..5 {
            let started = std::time::Instant::now();
            let bitmap = rasterize(&rows, &opt);
            let rasterised = started.elapsed();
            let sequence = encode(Protocol::Iterm2, &bitmap, 15, 60).expect("should encode");
            let elapsed = started.elapsed();
            if best.as_ref().is_none_or(|(e, ..)| elapsed < *e) {
                best = Some((elapsed, rasterised, bitmap, sequence));
            }
            if elapsed < std::time::Duration::from_millis(100) {
                break;
            }
        }
        let (elapsed, rasterised, bitmap, sequence) = best.expect("at least one run");

        let encoded = elapsed - rasterised;
        println!(
            "MEASURED scale {scale}: {}x{} bitmap, rasterize {rasterised:?} + encode \
             {encoded:?} = {elapsed:?}, {} bytes on the wire",
            bitmap.width,
            bitmap.height,
            sequence.len()
        );
        // The ceilings are per factor. Scale 1 keeps the bounds this test
        // was written with (400 ms, 64 KB), unchanged: it is the same
        // frame it always was. Scale 2 -- production -- gets the same ~7x
        // headroom over its own debug-build cost (139 ms alone on the
        // a development machine, 2026-09-23), because this test is known to run
        // slow when the whole workspace suite is competing for the CPU,
        // and a test that flakes gets deleted. What either catches is an
        // order of magnitude: a rasteriser gone quadratic in the pane
        // height, or PNG compression turned up to `Best`.
        let (ceiling, max_bytes) = match scale {
            1 => (400, 64 * 1024),
            2 => (1_000, 160 * 1024),
            _ => continue,
        };
        assert!(
            elapsed < std::time::Duration::from_millis(ceiling),
            "a full pane took {elapsed:?} to rasterise and encode at scale {scale}; at that cost a scroll would be visibly slow"
        );
        assert!(
            sequence.len() < max_bytes,
            "the encoded column is {} bytes at scale {scale}; something has defeated the PNG compression",
            sequence.len()
        );
    }
}

#[test]
fn a_queried_cell_size_is_clamped_into_something_drawable() {
    // A terminal answering 0 would divide by zero; one answering 9999
    // would allocate hundreds of megabytes per frame.
    assert_eq!(
        CellSize::sanitised(0, 0),
        CellSize {
            width: 4,
            height: 6
        }
    );
    assert_eq!(
        CellSize::sanitised(9999, 9999),
        CellSize {
            width: 64,
            height: 128
        }
    );
}

#[test]
fn the_encoded_sequence_is_a_complete_iterm2_file_escape() {
    let opt = RasterOptions::new(CellSize::FALLBACK, 2);
    let bmp = rasterize(&exercise_rows(), &opt);
    let seq = encode(Protocol::Iterm2, &bmp, 2, 7).expect("the bitmap should encode");
    assert!(seq.starts_with("\u{1b}]1337;File="), "{}", &seq[..40]);
    assert!(seq.ends_with('\u{7}'), "the sequence must be terminated");
    // `doNotMoveCursor=1` is WezTerm's extension and the reason an image
    // can sit on the last row of the alternate screen without scrolling
    // the whole TUI. Losing it is silent until someone selects the bottom
    // commit, which is exactly the kind of thing to pin.
    assert!(seq.contains(";doNotMoveCursor=1:"));
    assert!(seq.contains(";width=2;height=7;"));
}

#[test]
fn base64_round_trips_through_a_reference_decoder() {
    // The encoder is 20 lines written by hand rather than a dependency,
    // so it gets a test with known answers from RFC 4648.
    let seq = encode(
        Protocol::Iterm2,
        &rasterize(
            &[RasterRow::new(vec![TreeCell::empty()])],
            &RasterOptions::new(CellSize::sanitised(4, 6), 1),
        ),
        1,
        1,
    )
    .unwrap();
    let payload = seq.rsplit_once(':').unwrap().1.trim_end_matches('\u{7}');
    assert!(
        payload.len() % 4 == 0,
        "base64 output must be padded to a multiple of four"
    );
    assert!(
        payload.starts_with("iVBORw0KGgo"),
        "the payload must be a PNG"
    );
}

/// **The WIP stem must read as dashed**, not as a slightly grainy solid
/// line - which is what it did until 2026-09-23. Each dash was a
/// round-capped segment, and the caps add half a stroke width at *both*
/// ends: "3 px of ink, 3 px of air" was really 5.6 px of ink and 0.4 px of
/// air at a 9 px cell, which anti-aliasing then filled in.
///
/// Measured the way an eye would: walk down the stroke's center line and
/// look for runs of **pure background**, not merely lighter pixels. Every
/// gap must be at least two cell pixels of nothing, at every supersample
/// factor.
#[test]
fn a_wip_stem_is_visibly_dashed_with_real_gaps() {
    let rows = vec![RasterRow::new(vec![
        cell(TreePart::Vertical).with_dashed(Sides::VERTICAL)
    ])];
    for cell_size in [CellSize::FALLBACK, CellSize::sanitised(12, 28)] {
        for scale in [1, 2, 3] {
            let opt = RasterOptions::new(cell_size, 1).at_scale(scale);
            let bmp = rasterize(&rows, &opt);
            let x = bmp.width / 2;
            // Runs along the center line: (is_ink, length).
            let mut runs: Vec<(bool, u32)> = Vec::new();
            for y in 0..bmp.height {
                let ink = is_ink(&bmp, x, y, BG);
                match runs.last_mut() {
                    Some((kind, len)) if *kind == ink => *len += 1,
                    _ => runs.push((ink, 1)),
                }
            }
            // Only the gaps *between* two dashes count; the ends of the
            // cell may be cut anywhere in the pattern.
            let inner_gaps: Vec<u32> = runs
                .windows(3)
                .filter(|w| w[0].0 && !w[1].0 && w[2].0)
                .map(|w| w[1].1)
                .collect();
            assert!(
                inner_gaps.len() >= 2,
                "at cell {}x{} scale {scale} the WIP stem has {} gap(s) \
                 between dashes - it reads as a solid line: {runs:?}",
                cell_size.width,
                cell_size.height,
                inner_gaps.len()
            );
            assert!(
                inner_gaps.iter().all(|&g| g >= 2 * scale),
                "at cell {}x{} scale {scale} a gap in the WIP stem is under two \
                 cell pixels of clear background: {runs:?}",
                cell_size.width,
                cell_size.height
            );
        }
    }
}

// ---- The WIP row in pixels ---------------------------------------------

/// Pixels along the circle of radius `radius` around `(cx, cy)`, sampled
/// every half degree: true where there is ink.
fn around(bmp: &Bitmap, bg: (u8, u8, u8), (cx, cy): (f32, f32), radius: f32) -> Vec<bool> {
    (0..720)
        .map(|i| {
            let a = (i as f32 / 2.0).to_radians();
            let x = (cx + radius * a.cos()).round() as u32;
            let y = (cy + radius * a.sin()).round() as u32;
            is_ink(bmp, x, y, bg)
        })
        .collect()
}

/// How many separate gaps a closed walk has.
fn gaps(walk: &[bool]) -> usize {
    (0..walk.len())
        .filter(|&i| walk[i] && !walk[(i + 1) % walk.len()])
        .count()
}

/// **The WIP marker is a dashed ring**,
/// and an ordinary commit's ring is not. Walked around the ring's own
/// center line (found on the commit ring, which is the same size), at the
/// production supersample factor.
#[test]
fn the_wip_marker_is_a_dashed_ring_and_a_commit_ring_is_not() {
    let opt = RasterOptions::new(CellSize::FALLBACK, 1).at_scale(2);
    let bg = BG;
    let lone = |kind| rasterize(&[RasterRow::new(vec![node(kind, Sides::NONE)])], &opt);
    let commit = lone(NodeKind::Commit);
    let wip = lone(NodeKind::Wip);
    let (cx, cy) = (commit.width as f32 / 2.0, commit.height as f32 / 2.0);
    // The ring's center line: halfway through the ink met walking right
    // from the center of the commit ring.
    let inked: Vec<u32> = (cx as u32..commit.width)
        .filter(|&x| is_ink(&commit, x, cy as u32, bg))
        .collect();
    let radius = (inked[0] + inked[inked.len() - 1]) as f32 / 2.0 - cx + 0.5;

    let commit_gaps = gaps(&around(&commit, bg, (cx, cy), radius));
    let wip_gaps = gaps(&around(&wip, bg, (cx, cy), radius));
    assert_eq!(commit_gaps, 0, "a commit's ring must be closed");
    assert!(
        wip_gaps >= 6,
        "the WIP ring must read as dashed, it has {wip_gaps} gap(s)"
    );
}

/// **The WIP connector is dashed on both sides of the row boundary and
/// reaches both markers.** The WIP row is one line now, so the connector
/// is two half-cells - the marker's `down`, HEAD's `up` - in two
/// different rows; they must both be inked, and dashed rather than solid.
#[test]
fn the_wip_connector_to_head_is_dashed_across_the_row_boundary() {
    let down = Sides::new(false, true, false, false);
    let up = Sides::new(true, false, false, false);
    for cell_size in [CellSize::FALLBACK, CellSize::sanitised(12, 28)] {
        for scale in [1, 2] {
            let opt = RasterOptions::new(cell_size, 1).at_scale(scale);
            let bg = BG;
            let rows = vec![
                RasterRow::new(vec![node(NodeKind::Wip, down).with_dashed(down)]),
                RasterRow::new(vec![node(
                    NodeKind::Ref,
                    Sides::new(true, true, false, false),
                )
                .with_dashed(up)]),
            ];
            let bmp = rasterize(&rows, &opt);
            let ch = cell_size.height * scale;
            let x = bmp.width / 2;
            // Between the two markers: from just under the WIP ring to
            // just above HEAD's disc. The markers' outer radius is
            // measured, not assumed -- on a lone disc of the same size,
            // because in `bmp` a dash may touch the marker and walking out
            // of it would run on along the dash.
            let lone = rasterize(
                &[RasterRow::new(vec![node(NodeKind::Ref, Sides::NONE)])],
                &opt,
            );
            let outer = (ch / 2..ch)
                .find(|&y| !is_ink(&lone, x, y, bg))
                .expect("the disc has an edge")
                - ch / 2;
            let head_cy = ch + ch / 2;
            let disc_top = head_cy - outer;
            let ring_bottom = ch / 2 + outer;
            let band: Vec<bool> = (ring_bottom..=disc_top)
                .map(|y| is_ink(&bmp, x, y, bg))
                .collect();
            let upper = (ring_bottom..ch).any(|y| is_ink(&bmp, x, y, bg));
            let lower = (ch..=disc_top).any(|y| is_ink(&bmp, x, y, bg));
            assert!(
                upper && lower,
                "at {}x{} scale {scale}: the connector must be inked in both rows \
                 (WIP half {upper}, HEAD half {lower})",
                cell_size.width,
                cell_size.height
            );
            // A real gap: at least two cell pixels of pure background
            // between two inked stretches.
            let mut runs: Vec<(bool, u32)> = Vec::new();
            for ink in band {
                match runs.last_mut() {
                    Some((kind, len)) if *kind == ink => *len += 1,
                    _ => runs.push((ink, 1)),
                }
            }
            let inner_gap = runs
                .windows(3)
                .any(|w| w[0].0 && !w[1].0 && w[2].0 && w[1].1 >= 2 * scale);
            assert!(
                inner_gap,
                "at {}x{} scale {scale}: the connector reads as solid: {runs:?}",
                cell_size.width, cell_size.height
            );
        }
    }
}

/// Pending ink is dimmed: the WIP marker never reaches the full lane
/// color a commit's ring does.
#[test]
fn the_wip_marker_is_dimmer_than_a_commit() {
    let opt = RasterOptions::new(CellSize::FALLBACK, 1).at_scale(2);
    let peak = |kind| {
        let bmp = rasterize(&[RasterRow::new(vec![node(kind, Sides::NONE)])], &opt);
        (0..bmp.height)
            .flat_map(|y| (0..bmp.width).map(move |x| (x, y)))
            .filter_map(|(x, y)| shown(&bmp, x, y))
            .map(|p| i32::from(p.2) - i32::from(BG.2))
            .max()
            .unwrap_or(0)
    };
    let (commit, wip) = (peak(NodeKind::Commit), peak(NodeKind::Wip));
    assert!(
        (wip as f32) < commit as f32 * 0.8,
        "WIP peak {wip} vs commit peak {commit}: pending ink must be dimmed"
    );
}

// ---- Stashes in pixels -------------------------------------------------

/// The farthest point along the bitmap's main diagonal (top-left to
/// bottom-right of the marker's own cell) that reads as solid ink,
/// measured as pixels out from the center. Data-driven rather than a
/// hand-computed pixel offset, so it stays correct across any future
/// change to `node_radius`, `ring_half_width` or the box's own geometry --
/// the previous version of this test hard-coded a corner offset for one
/// specific radius and had to be re-derived by hand each time that radius
/// changed (twice, both times this same batch).
fn diagonal_reach(bmp: &Bitmap, background: (u8, u8, u8)) -> u32 {
    let (cx, cy) = (bmp.width / 2, bmp.height / 2);
    let max_k = cx.min(cy);
    (0..max_k)
        .rev()
        .find(|&k| is_solid(bmp, cx + k, cy + k, background))
        .unwrap_or(0)
}

/// **A stash is a little box**, not another
/// circle: a square's corner sits farther from the center, on the
/// diagonal, than any point of the circle inscribed in that same square --
/// so the box's ink must reach farther out along the diagonal than the
/// commit ring's does, at the same cell size and radius.
#[test]
fn a_stash_marker_is_a_box_and_a_commit_ring_is_not() {
    for scale in [1, 2] {
        let opt = RasterOptions::new(CellSize::sanitised(20, 40), 1).at_scale(scale);
        let bg = BG;
        let lone = |kind| rasterize(&[RasterRow::new(vec![node(kind, Sides::NONE)])], &opt);
        let (commit, stash) = (lone(NodeKind::Commit), lone(NodeKind::Stash));
        let (ring_reach, box_reach) = (diagonal_reach(&commit, bg), diagonal_reach(&stash, bg));
        assert!(
            box_reach > ring_reach,
            "scale {scale}: the stash box reaches {box_reach} px out on the diagonal, \
             the commit ring {ring_reach} px - the box's corner should clear the ring \
             by a full stroke width or more, since a square's corner sits outside the \
             circle inscribed in it"
        );
        assert!(
            box_reach as f32 - ring_reach as f32 >= scale as f32,
            "scale {scale}: the box only clears the ring by {} px on the diagonal, \
             too close to be a reliable margin",
            box_reach - ring_reach
        );
    }
}

/// A stash is recorded work, not pending: its box is drawn at the full
/// lane strength a commit's ring is, unlike the WIP marker.
#[test]
fn a_stash_box_is_drawn_at_full_strength() {
    let opt = RasterOptions::new(CellSize::FALLBACK, 1).at_scale(2);
    let peak = |kind| {
        let bmp = rasterize(&[RasterRow::new(vec![node(kind, Sides::NONE)])], &opt);
        (0..bmp.height)
            .flat_map(|y| (0..bmp.width).map(move |x| (x, y)))
            .filter_map(|(x, y)| shown(&bmp, x, y))
            .map(|p| i32::from(p.2) - i32::from(BG.2))
            .max()
            .unwrap_or(0)
    };
    let (commit, stash) = (peak(NodeKind::Commit), peak(NodeKind::Stash));
    assert!(
        (stash as f32) > commit as f32 * 0.95,
        "stash peak {stash} vs commit peak {commit}: a stash is not dimmed"
    );
}

/// Runs of ink / background along a line of pixels.
fn runs_of(ink: impl Iterator<Item = bool>) -> Vec<(bool, u32)> {
    let mut runs: Vec<(bool, u32)> = Vec::new();
    for i in ink {
        match runs.last_mut() {
            Some((kind, len)) if *kind == i => *len += 1,
            _ => runs.push((i, 1)),
        }
    }
    runs
}

/// **A stash's connector is dashed all the way into its base**: down its
/// own lane across a row boundary, then through the elbow and along the
/// run into the base commit's marker - both stretches carry real gaps of
/// clear background, and both are inked in every cell they cross.
///
/// The three rows the layout produces for a stash above a commit that
/// is not its base, then the base: the box (dashed `down`), the lane
/// passing a commit (dashed `│`), and the base's row - its marker with a
/// dashed right arm, a dashed `─`, and the dashed `╯`.
#[test]
fn a_stash_connector_is_dashed_down_its_lane_and_into_its_base() {
    let down = Sides::new(false, true, false, false);
    let right = Sides::new(false, false, false, true);
    let e = TreeCell::empty;
    for cell_size in [CellSize::FALLBACK, CellSize::sanitised(12, 28)] {
        for scale in [1, 2] {
            let opt = RasterOptions::new(cell_size, 3).at_scale(scale);
            let bg = BG;
            let rows = vec![
                RasterRow::new(vec![
                    e(),
                    e(),
                    node(NodeKind::Stash, down).with_dashed(down),
                ]),
                RasterRow::new(vec![
                    cell(TreePart::Vertical),
                    e(),
                    cell(TreePart::Vertical).with_dashed(Sides::VERTICAL),
                ]),
                RasterRow::new(vec![
                    node(NodeKind::Commit, Sides::new(true, false, false, true)).with_dashed(right),
                    cell(TreePart::Horizontal).with_dashed(Sides::HORIZONTAL),
                    cell(TreePart::MergeInRight).with_dashed(Sides::new(true, false, true, false)),
                ]),
            ];
            let bmp = rasterize(&rows, &opt);
            let (cw, ch) = (cell_size.width * scale, cell_size.height * scale);
            let what = format!("{}x{} scale {scale}", cell_size.width, cell_size.height);

            // Down the stash's lane, from under the box to the elbow's
            // corner: rows 0 (below center), 1, and 2 (above center).
            let x = 2 * cw + cw / 2;
            let vertical =
                runs_of((ch / 2 + ch / 3..2 * ch + ch / 4).map(|y| is_ink(&bmp, x, y, bg)));
            let inked_rows: Vec<bool> = (0..3)
                .map(|r| (r * ch..(r + 1) * ch).any(|y| is_ink(&bmp, x, y, bg)))
                .collect();
            assert_eq!(
                inked_rows,
                vec![true, true, true],
                "{what}: the lane is inked in every row"
            );
            assert!(
                vertical
                    .windows(3)
                    .any(|w| w[0].0 && !w[1].0 && w[2].0 && w[1].1 >= 2 * scale),
                "{what}: the stash's lane reads as solid: {vertical:?}"
            );

            // Along the run into the base, from just right of the base's
            // marker to the elbow.
            let y = 2 * ch + ch / 2;
            let horizontal =
                runs_of((cw / 2 + cw / 2..2 * cw + cw / 4).map(|x| is_ink(&bmp, x, y, bg)));
            assert!(
                horizontal.iter().any(|r| r.0),
                "{what}: the run into the base is inked"
            );
            assert!(
                horizontal.iter().any(|r| !r.0 && r.1 >= 2 * scale),
                "{what}: the run into the base reads as solid: {horizontal:?}"
            );

            // The elbow itself is pending ink too: too short to carry a
            // dash of its own, it is laid dimmed, never at the full lane
            // strength of history. (Where the fillet overlaps an arm the
            // two dimmed layers stack to ~86 %, hence 0.95 and not 0.62.)
            let full = i32::from(LANE_RGB.2) - i32::from(bg.2);
            let elbow_peak = (2 * ch..3 * ch)
                .flat_map(|y| (2 * cw..3 * cw).map(move |x| (x, y)))
                .filter_map(|(x, y)| shown(&bmp, x, y))
                .map(|p| i32::from(p.2) - i32::from(bg.2))
                .max()
                .unwrap_or(0);
            assert!(
                elbow_peak > 0 && (elbow_peak as f32) < full as f32 * 0.95,
                "{what}: the elbow is drawn at {elbow_peak} of {full}: it must be dimmed"
            );
        }
    }
}

// ---- Marker size tracks the cell, not a pixel count -------------------

/// The inked vertical extent of a bare marker (its outer diameter, read
/// off the bitmap rather than computed from `node_radius`'s private
/// constants), at the given cell size.
fn inked_diameter(kind: NodeKind, cell: CellSize) -> u32 {
    let opt = RasterOptions::new(cell, 1);
    let bmp = rasterize(&[RasterRow::new(vec![node(kind, Sides::NONE)])], &opt);
    let cx = bmp.width / 2;
    let ys: Vec<u32> = (0..bmp.height)
        .filter(|&y| is_solid(&bmp, cx, y, BG))
        .collect();
    let (top, bottom) = (
        *ys.first().expect("a bare marker inks something"),
        *ys.last().expect("a bare marker inks something"),
    );
    bottom - top + 1
}

/// **The node scales with the cell instead of drawing a fixed pixel
/// count.** Doubling the cell size (same aspect ratio, so the same branch
/// of `node_radius`'s `min` wins at both) must double the marker's inked
/// diameter and leave the diameter's *fraction* of the cell height
/// unchanged. A hard-coded pixel radius would leave the diameter constant
/// in pixels while the cell doubled around it, halving that fraction --
/// which is exactly what this test would catch, and exactly the defect
/// the PNG markers had before `node_radius` read the cell at all.
///
/// Falsified by hard-coding `node_radius` to return a fixed pixel value
/// (e.g. `4.0 * opt.s()`): the small cell's fraction comes out about
/// double the big cell's, and the `(small - big).abs() < 0.03` assertion
/// goes red.
#[test]
fn the_commit_markers_inked_diameter_scales_with_the_cell_not_a_pixel_count() {
    // Twice the fallback cell, not the fallback cell itself: the diameter
    // is measured to the nearest bitmap pixel, and that +/-1 px rounding
    // is a much bigger share of a ~4 px radius than of an ~8 px one.
    // Starting bigger keeps the *raw* pixel-count ratio reliably close to
    // 2.0 as well as the (tighter) fraction check. `CellSize::sanitised`
    // clamps width to 64 and height to 128 -- doubling past the fallback
    // cell's own x4 would silently clip one axis and break the very
    // aspect ratio this test depends on staying fixed, so this stays at
    // x2/x4 rather than x4/x8.
    let small = CellSize::sanitised(CellSize::FALLBACK.width * 2, CellSize::FALLBACK.height * 2);
    let big = CellSize::sanitised(small.width * 2, small.height * 2);
    for kind in [NodeKind::Commit, NodeKind::Ref, NodeKind::Merge] {
        let d_small = inked_diameter(kind, small);
        let d_big = inked_diameter(kind, big);
        let ratio = d_big as f32 / d_small as f32;
        assert!(
            (ratio - 2.0).abs() < 0.1,
            "{kind:?}: diameter went from {d_small} px to {d_big} px when the cell \
             doubled - ratio {ratio:.2}, expected close to 2.0"
        );
        let (fraction_small, fraction_big) = (
            d_small as f32 / small.height as f32,
            d_big as f32 / big.height as f32,
        );
        assert!(
            (fraction_small - fraction_big).abs() < 0.03,
            "{kind:?}: diameter is {:.3} of the cell height at {}x{} but {:.3} at \
             {}x{} - the marker is not tracking the cell size",
            fraction_small,
            small.width,
            small.height,
            fraction_big,
            big.width,
            big.height
        );
    }
}

/// **A bigger node still fits inside its own row.** Bumping the marker
/// size is exactly the kind of change that could
/// make a node's own ink - the ring, the dashed WIP outline, the stash
/// box - spill into the row above or below if its radius grew past half
/// the cell height. The node is bare (no connecting sides): a node's
/// *connectors* are deliberately allowed to bleed a stroke-width's worth
/// of round cap past the cell edge - that overlap is what lets a lane
/// join seamlessly across the row boundary in the first place, and
/// `every_node_is_connected_to_every_stroke_it_claims` already covers
/// that seam. This test is about the marker's own shape, sandwiched
/// between two empty rows: neither neighbor may carry a speck of ink.
#[test]
fn a_bigger_node_still_fits_inside_its_own_row() {
    for cell_size in [CellSize::FALLBACK, CellSize::sanitised(20, 40)] {
        for scale in [1, 2] {
            for kind in [
                NodeKind::Commit,
                NodeKind::Ref,
                NodeKind::Merge,
                NodeKind::Wip,
                NodeKind::Stash,
            ] {
                let opt = RasterOptions::new(cell_size, 1).at_scale(scale);
                let rows = vec![
                    RasterRow::new(vec![TreeCell::empty()]),
                    RasterRow::new(vec![node(kind, Sides::NONE)]),
                    RasterRow::new(vec![TreeCell::empty()]),
                ];
                let bmp = rasterize(&rows, &opt);
                let ch = cell_size.height * scale;
                for (label, range) in [("above", 0..ch), ("below", 2 * ch..3 * ch)] {
                    for y in range {
                        for x in 0..bmp.width {
                            assert!(
                                !is_ink(&bmp, x, y, BG),
                                "cell {}x{} scale {scale}: a {kind:?} node inked the \
                                 row {label} its own at ({x},{y})",
                                cell_size.width,
                                cell_size.height
                            );
                        }
                    }
                }
            }
        }
    }
}

// ---- `ui.avatars = "graph"`: the author's picture in the commit circle ----

const FACE: (u8, u8, u8) = (220, 30, 40);

fn face() -> super::raster::NodeAvatar {
    super::raster::NodeAvatar {
        key: "author".into(),
        bitmap: std::sync::Arc::new(Bitmap::new(32, 32, FACE)),
    }
}

/// One row, `lanes` lanes wide, the commit on lane `on` holding a face.
/// Returns the bitmap, the options and the circle's center x.
fn face_row(kind: NodeKind, lanes: usize, on: usize) -> (Bitmap, RasterOptions, u32) {
    let columns = (lanes * 2) as u16;
    let opt = RasterOptions::new(CellSize::FALLBACK, columns).with_avatars(true);
    let cells = (0..lanes * 2)
        .map(|c| {
            if c == on * 2 {
                node(kind, Sides::VERTICAL)
            } else if c % 2 == 0 {
                cell(TreePart::Vertical)
            } else {
                TreeCell::empty()
            }
        })
        .collect();
    let row = RasterRow {
        cells,
        selected: false,
        avatar: Some(face()),
        ref_connector: false,
    };
    let bmp = rasterize(&[row], &opt);
    let pad = super::raster::avatar_left_pad(&opt);
    let cx = pad + (on as u32 * 2) * CellSize::FALLBACK.width + CellSize::FALLBACK.width / 2;
    (bmp, opt, cx)
}

fn near(a: (u8, u8, u8), b: (u8, u8, u8)) -> bool {
    let d = |x: u8, y: u8| (i16::from(x) - i16::from(y)).abs();
    d(a.0, b.0) <= 12 && d(a.1, b.1) <= 12 && d(a.2, b.2) <= 12
}

/// The face fills the circle, a ring in the lane's
/// color borders it, and the circle is much bigger than a bare marker.
///
/// Falsified: skipped `draw_avatar_node` (the center stayed the hollow
/// ring's background, red), and drew the face without the ring (the
/// border pixel was face, red); restored.
#[test]
fn the_face_fills_the_commit_circle_inside_a_lane_colored_ring() {
    let (bmp, _opt, cx) = face_row(NodeKind::Commit, 3, 1);
    let cy = CellSize::FALLBACK.height / 2;
    assert!(near(shown(&bmp, cx, cy).unwrap(), FACE));
    // Well off the center but inside the ring: still face -- a bare
    // marker's hole, or its ring, would be here.
    assert!(near(shown(&bmp, cx + 5, cy).unwrap(), FACE));
    assert!(near(shown(&bmp, cx, cy - 5).unwrap(), FACE));
    // The ring: just inside the circle's edge (radius ~9.45 at 9x21).
    assert!(
        near(shown(&bmp, cx + 8, cy).unwrap(), LANE_RGB),
        "{:?}",
        shown(&bmp, cx + 8, cy)
    );
    assert!(near(shown(&bmp, cx, cy + 8).unwrap(), LANE_RGB));
    // Past the edge on the spacer side: background again.
    assert_eq!(shown(&bmp, cx + 11, cy), Some(BG));
}

/// Falsified: widened the disc test to a square in `draw_avatar_node`
/// (the corner went face, red); restored.
#[test]
fn the_face_is_clipped_to_a_disc_not_a_square() {
    let (bmp, _opt, cx) = face_row(NodeKind::Ref, 3, 1);
    let cy = CellSize::FALLBACK.height / 2;
    // 7 px out on both axes is 9.9 px from the center: outside the 9.45
    // px circle, inside its bounding square.
    for (dx, dy) in [(7i32, 7i32), (-7, 7), (7, -7), (-7, -7)] {
        let p = bmp
            .pixel((cx as i32 + dx) as u32, (cy as i32 + dy) as u32)
            .unwrap();
        assert!(!near(p, FACE), "corner ({dx},{dy}) is face: {p:?}");
    }
    assert_eq!(shown(&bmp, cx + 8, cy + 8), Some(BG));
}

/// The scaling path behind "heavily pixelated avatars": a picture much
/// bigger than the node, made of one-pixel stripes, resampled the way the
/// service does and drawn. Its inside must be the mid-gray the eye sees;
/// the old nearest-neighbor path left hard black/white speckle.
///
/// Falsified: put `downscale` back to picking one source pixel per output
/// pixel; the inside came back pure black and white and this went red.
#[test]
fn a_big_fine_picture_is_averaged_into_the_node_not_speckled() {
    let cell = CellSize::FALLBACK;
    let scale = 2;
    let side = super::raster::avatar_picture_px(cell, scale);
    let mut big = Bitmap::new(512, 512, (0, 0, 0));
    for y in 0..512 {
        for x in 0..512 {
            if (x + y) % 2 == 0 {
                big.set_pixel(x, y, (255, 255, 255));
            }
        }
    }
    let picture = crate::image_decode::downscale(&big, side, side);
    let opt = RasterOptions::new(cell, 6)
        .with_avatars(true)
        .at_scale(scale);
    let row = RasterRow {
        cells: (0..6)
            .map(|c| {
                if c == 2 {
                    node(NodeKind::Commit, Sides::VERTICAL)
                } else {
                    TreeCell::empty()
                }
            })
            .collect(),
        selected: false,
        avatar: Some(super::raster::NodeAvatar {
            key: "stripes".into(),
            bitmap: std::sync::Arc::new(picture),
        }),
        ref_connector: false,
    };
    let bmp = rasterize(&[row], &opt);
    let cx = super::raster::avatar_left_pad(&opt) + 2 * cell.width * scale + cell.width * scale / 2;
    let cy = cell.height * scale / 2;
    let mut checked = 0;
    for dy in -6i32..=6 {
        for dx in -6i32..=6 {
            let p = bmp
                .pixel((cx as i32 + dx) as u32, (cy as i32 + dy) as u32)
                .unwrap();
            assert!(
                (120..=136).contains(&p.0) && p.0 == p.1 && p.1 == p.2,
                "({dx},{dy}) = {p:?}: a fine pattern must average to gray inside the node"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 169);
}

/// A lane's thickness in screen pixels, measured across a vertical lane.
fn lane_thickness_px(cell: CellSize, scale: u32) -> f32 {
    let rows = vec![RasterRow::new(vec![self::cell(TreePart::Vertical)])];
    let opt = RasterOptions::new(cell, 1).at_scale(scale);
    let bmp = rasterize(&rows, &opt);
    let y = bmp.height / 4;
    let coverage: f32 = (0..bmp.width)
        .map(|x| f32::from(bmp.rgba(x, y).map_or(0, |p| p.3)) / 255.0)
        .sum();
    coverage / scale as f32
}

/// **Line width follows the zoom.** A terminal zoom grows the cell in
/// pixels; a lane must stay about as thin in pixels as at the default
/// 9x21 (owner: lanes became fat bars at a big zoom), not stay a
/// fraction of the cell.
///
/// Falsified: with the old ceiling (`cell.width / 7` clamped to 2.2) the
/// big-cell lanes were about 4.4 px against 2.6 at 9x21 (red); restored.
#[test]
fn a_lane_stays_thin_in_pixels_when_the_cell_grows_with_the_zoom() {
    for scale in [1, 2] {
        let normal = lane_thickness_px(CellSize::FALLBACK, scale);
        for zoom in [2, 3, 4] {
            let big = lane_thickness_px(CellSize::sanitised(9 * zoom, 21 * zoom), scale);
            assert!(
                big <= normal * 1.25 && big >= normal * 0.8,
                "zoom {zoom}x scale {scale}: lane {big:.2}px against {normal:.2}px at the default"
            );
        }
        assert!(
            normal < 3.2,
            "scale {scale}: the default lane is {normal:.2}px"
        );
    }
}

/// The avatar's decode box is the on-screen circle: it grows with the
/// cell, so a zoomed-in terminal is not handed a small picture to stretch.
#[test]
fn the_avatar_box_grows_with_the_zoom() {
    let small = super::raster::avatar_picture_px(CellSize::FALLBACK, 1);
    let big = super::raster::avatar_picture_px(CellSize::sanitised(18, 42), 1);
    assert!(big >= small * 2 - 2, "{small}px at 9x21, {big}px at 18x42");
}

/// The picture box the service resamples to is the circle the rasteriser
/// draws (its diameter, rounded up), at every supersample factor.
#[test]
fn the_picture_box_is_the_avatar_circles_diameter() {
    let cell = CellSize::FALLBACK;
    for scale in 1..=3u32 {
        let px = super::raster::avatar_picture_px(cell, scale);
        let opt = RasterOptions::new(cell, 1).at_scale(scale);
        let diameter = 2.0 * super::raster::avatar_radius(&opt);
        assert!(
            (px as f32 - diameter).abs() < 1.0 && px as f32 >= diameter,
            "scale {scale}: box {px}, circle {diameter}"
        );
    }
    // Twice the supersample, twice the box.
    assert!(
        super::raster::avatar_picture_px(cell, 2)
            >= 2 * super::raster::avatar_picture_px(cell, 1) - 1
    );
}

/// The first lane's circle is wider than its cell; the graph is shifted
/// right so it is not cut by the column's left edge.
///
/// Falsified: made `avatar_left_pad` return 0 (the first column of
/// pixels was ring, red); restored.
#[test]
fn the_first_lanes_face_is_not_cut_by_the_columns_edge() {
    let (bmp, _opt, cx) = face_row(NodeKind::Commit, 1, 0);
    let cy = CellSize::FALLBACK.height / 2;
    assert!(near(shown(&bmp, cx, cy).unwrap(), FACE));
    for y in 0..bmp.height {
        assert_eq!(shown(&bmp, 0, y), Some(BG), "x=0 inked at y={y}");
    }
}

/// A merge keeps its bullseye, and a row with no picture yet keeps the
/// ordinary hollow marker - the fallback until the face arrives.
#[test]
fn merges_and_rows_without_a_picture_keep_their_marker() {
    let (bmp, _, cx) = face_row(NodeKind::Merge, 1, 0);
    let cy = CellSize::FALLBACK.height / 2;
    assert!(!near(shown(&bmp, cx, cy).unwrap(), FACE));
    assert!(!near(shown(&bmp, cx + 2, cy).unwrap(), FACE));

    let opt = RasterOptions::new(CellSize::FALLBACK, 2).with_avatars(true);
    let bare = rasterize(
        &[RasterRow::new(vec![node(
            NodeKind::Commit,
            Sides::VERTICAL,
        )])],
        &opt,
    );
    let cx = super::raster::avatar_left_pad(&opt) + CellSize::FALLBACK.width / 2;
    // The hollow ring's hole: background at the center.
    assert_eq!(shown(&bare, cx, cy), Some(BG));
}

/// With `avatars` off (the default), a row that happens to carry a
/// picture draws exactly what it drew before this feature existed.
#[test]
fn with_avatars_off_nothing_moves_and_no_face_is_drawn() {
    let opt = RasterOptions::new(CellSize::FALLBACK, 2);
    let cells = vec![node(NodeKind::Commit, Sides::VERTICAL), TreeCell::empty()];
    let plain = rasterize(&[RasterRow::new(cells.clone())], &opt);
    let with_face = rasterize(
        &[RasterRow {
            cells,
            selected: false,
            avatar: Some(face()),
            ref_connector: false,
        }],
        &opt,
    );
    assert_eq!(plain, with_face);
    assert_eq!(super::raster::avatar_left_pad(&opt), 0);
}

// ---------------------------------------------------------------------
// The ref-to-commit connector (`ui.ref_connector`).
// ---------------------------------------------------------------------

/// The crossed lanes' color: *not* [`LANE`], so a connector inked over
/// them (rather than under) would show on their center pixel.
const OTHER_LANE: Color = Color::Rgb(255, 171, 0);

/// Lane 2's commit, with lanes 0 and 1 live beside it: `│ │ ○`.
fn connector_row(on: bool) -> RasterRow {
    RasterRow {
        cells: vec![
            TreeCell::new(TreePart::Vertical, OTHER_LANE),
            TreeCell::empty(),
            TreeCell::new(TreePart::Vertical, OTHER_LANE),
            TreeCell::empty(),
            node(NodeKind::Commit, Sides::VERTICAL),
        ],
        selected: false,
        avatar: None,
        ref_connector: on,
    }
}

/// **The connector in pixels**: one pixel row, in the lane's color, from
/// the image's left edge to the node's ring - thinner than the lanes, and
/// drawn *under* them.
///
/// - every pixel the connector changes lies on that one row, left of the
///   node's center, so nothing else in the picture moved;
/// - it is solid lane color at the column's left edge and in both blank
///   spacer cells;
/// - it is exactly one pixel thick where a lane is about three: measured
///   across the spacer cell and across a lane, same bitmap;
/// - at each crossed lane's center the pixel is the lane's own, unchanged
///   - the lane is on top;
/// - it stops at the ring: the hollow marker's hole is still background.
///
/// Falsified: dropping the `if row.ref_connector` draw fails the "one
/// pixel row" assertion; drawing it *after* the row's cells (over the
/// lanes) fails "lane on top", the crossed lanes being another color.
#[test]
fn the_connector_is_one_pixel_under_the_lanes_up_to_the_ring() {
    let opt = RasterOptions::new(CellSize::FALLBACK, 5);
    let (cw, ch) = (CellSize::FALLBACK.width, CellSize::FALLBACK.height);
    let bg = BG;
    let plain = rasterize(&[connector_row(false)], &opt);
    let with = rasterize(&[connector_row(true)], &opt);

    // Which pixel rows changed at all.
    let node_cx = 4 * cw + cw / 2;
    let mut changed_rows = std::collections::BTreeSet::new();
    for y in 0..with.height {
        for x in 0..with.width {
            if shown(&plain, x, y) != shown(&with, x, y) {
                changed_rows.insert(y);
                assert!(x < node_cx, "changed pixel ({x}, {y}) right of the node");
            }
        }
    }
    assert_eq!(changed_rows.len(), 1, "one pixel row: {changed_rows:?}");
    let y = *changed_rows.iter().next().unwrap();
    assert!(y.abs_diff(ch / 2) <= 1, "on the row's middle: y = {y}");

    // Solid lane color at the left edge and across both spacer cells.
    for x in [0, cw + cw / 2, 3 * cw + cw / 2] {
        assert_eq!(shown(&with, x, y), Some(LANE_RGB), "x = {x}");
    }

    // One pixel thick in the spacer cell, against the lane's width.
    let spacer_x = cw + cw / 2;
    let thick = (0..ch)
        .filter(|&yy| is_ink(&with, spacer_x, yy, bg))
        .count();
    let lane_row = 2; // well above the node and the connector
    let lane_width = (0..cw).filter(|&x| is_ink(&with, x, lane_row, bg)).count();
    assert_eq!(thick, 1);
    assert!(
        lane_width > thick,
        "lane {lane_width} px vs connector {thick} px"
    );

    // The lanes stay on top: their center pixel on that row is untouched.
    for lane_cell in [0, 2] {
        let cx = lane_cell * cw + cw / 2;
        assert_eq!(
            shown(&with, cx, y),
            shown(&plain, cx, y),
            "lane {lane_cell}"
        );
    }

    // The hollow marker's hole is still background.
    let (hole_x, hole_y) = (node_cx, ch / 2);
    assert_eq!(shown(&with, hole_x, hole_y), shown(&plain, hole_x, hole_y));
    assert_eq!(shown(&with, hole_x, hole_y), Some(bg));
}

/// At the 2x supersample the Log actually uses, the line scales with the
/// picture: two bitmap pixels, i.e. still one assumed cell pixel - a
/// constant that forgot to scale would be a hairline at 2x.
#[test]
fn the_connector_scales_with_the_supersample() {
    let opt = RasterOptions::new(CellSize::FALLBACK, 5).at_scale(2);
    let (cw, ch) = (CellSize::FALLBACK.width * 2, CellSize::FALLBACK.height * 2);
    let with = rasterize(&[connector_row(true)], &opt);
    let spacer_x = cw + cw / 2;
    let thick: Vec<u32> = (0..ch)
        .filter(|&y| is_ink(&with, spacer_x, y, BG))
        .collect();
    assert_eq!(thick.len(), 2, "rows {thick:?}");
    for y in thick {
        assert_eq!(shown(&with, spacer_x, y), Some(LANE_RGB));
    }
}

/// A row without the flag - every row but the ones carrying a pill --
/// is byte-for-byte what it was.
#[test]
fn a_row_without_the_connector_is_untouched() {
    let opt = RasterOptions::new(CellSize::FALLBACK, 5);
    let before = rasterize(&[RasterRow::new(connector_row(false).cells)], &opt);
    assert_eq!(before, rasterize(&[connector_row(false)], &opt));
    let spacer = (CellSize::FALLBACK.width + 4, CellSize::FALLBACK.height / 2);
    assert_eq!(shown(&before, spacer.0, spacer.1), Some(BG));
}

/// `ui.graph_lane_background`: off paints nothing but the lane's ink (a
/// corner of the cell is fully transparent); row_tint paints a band from a
/// commit's node to the right edge.
#[test]
fn the_lane_background_setting_decides_whether_a_rail_is_painted() {
    use crate::graph::LaneBackground;
    let rows = vec![RasterRow::new(vec![TreeCell::new(
        TreePart::Vertical,
        Color::Rgb(64, 156, 255),
    )])];
    let base = RasterOptions::new(CellSize::FALLBACK, 1);
    let alpha_at = |bmp: &Bitmap, x: u32, y: u32| bmp.data[((y * bmp.width + x) * 4 + 3) as usize];
    let off = rasterize(&rows, &base);
    assert_eq!(alpha_at(&off, 0, 0), 0, "off: the corner is clear");
    // A row tint needs a node: a bare lane row gets no band.
    let tint = rasterize(&rows, &base.with_lane_background(LaneBackground::RowTint));
    assert_eq!(alpha_at(&tint, 0, 0), 0);
    // With a node in column 0 of a 3-cell column: nothing left of the
    // node's center, a faint translucent band from there to the right edge.
    let node_row = vec![RasterRow::new(vec![
        TreeCell::new(
            TreePart::Node {
                kind: NodeKind::Commit,
                sides: Sides::VERTICAL,
            },
            Color::Rgb(64, 156, 255),
        ),
        TreeCell::empty(),
        TreeCell::empty(),
    ])];
    let wide = RasterOptions::new(CellSize::FALLBACK, 3);
    let band = rasterize(
        &node_row,
        &wide.with_lane_background(LaneBackground::RowTint),
    );
    let plain = rasterize(&node_row, &wide);
    let last = band.width - 1;
    let y = band.height - 1;
    // Just left of the right-edge bar: the faint band.
    let a = alpha_at(&band, last - 3, y);
    assert!(a > 0 && a < 80, "faint, translucent: {a}");
    assert_eq!(alpha_at(&plain, last - 3, y), 0, "off: nothing there");
    // The bar itself: strong, in the lane's color, and thin.
    assert!(
        alpha_at(&band, last, y) > 200,
        "the right-edge bar is solid"
    );
    assert_eq!(
        &band.data[((y * band.width + last) * 4) as usize..][..3],
        &[64, 156, 255]
    );
    let bar_px = (0..band.width)
        .filter(|&x| alpha_at(&band, x, y) > 200)
        .count();
    assert!((1..=3).contains(&bar_px), "thin bar, {bar_px} px wide");
    assert_eq!(alpha_at(&plain, last, y), 0, "off: nothing there");
    assert_eq!(
        alpha_at(&band, 0, y),
        alpha_at(&plain, 0, y),
        "left of the node untouched"
    );
}

/// `ui.graph_row_tint_strength`: the band's alpha follows the level
/// (strong is 0.22), the right-edge bar stays solid.
#[test]
fn the_row_tint_strength_fades_the_band_but_not_the_bar() {
    use crate::graph::LaneBackground;
    use gitpoulpi_config::GraphRowTintStrength as S;
    let color = Color::Rgb(64, 156, 255);
    let node_row = vec![RasterRow::new(vec![
        TreeCell::new(
            TreePart::Node {
                kind: NodeKind::Commit,
                sides: Sides::VERTICAL,
            },
            color,
        ),
        TreeCell::empty(),
        TreeCell::empty(),
    ])];
    let base =
        RasterOptions::new(CellSize::FALLBACK, 3).with_lane_background(LaneBackground::RowTint);
    let alpha_at = |bmp: &Bitmap, x: u32, y: u32| bmp.data[((y * bmp.width + x) * 4 + 3) as usize];
    let at = |s: S| {
        let b = rasterize(&node_row, &base.with_tint_strength(s));
        let (x, y) = (b.width - 4, b.height - 1);
        (alpha_at(&b, x, y), alpha_at(&b, b.width - 1, y))
    };
    let (strong, strong_bar) = at(S::Strong);
    let (medium, medium_bar) = at(S::Medium);
    let (subtle, subtle_bar) = at(S::Subtle);
    assert_eq!(strong, (0.22f32 * 255.0).round() as u8);
    assert!(
        strong > medium && medium > subtle && subtle > 0,
        "{strong} {medium} {subtle}"
    );
    // The bar is composited over the band, so it differs by a few
    // levels at most; it stays solid at every strength.
    for bar in [strong_bar, medium_bar, subtle_bar] {
        assert!(bar > 200, "{bar}");
    }
    // The default is medium.
    let d = rasterize(&node_row, &base);
    assert_eq!(alpha_at(&d, d.width - 4, d.height - 1), medium);
}
