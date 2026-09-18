//! The rasteriser's primitives: `Shape`, anti-aliased `draw`, and the builders
//! for boxes, dashed rings, dashed runs and elbows.

use super::*;

/// A shape, as the only thing the rasteriser needs from one: *is this
/// point inside you?* Everything below is a closure over that, which is
/// why anti-aliasing is one loop rather than one case per primitive.
pub(super) enum Shape {
    /// A capsule: every point within `half_width` of the segment.
    Segment {
        a: (f32, f32),
        b: (f32, f32),
        half_width: f32,
    },
    /// An annulus sector: within `half_width` of the circle of radius
    /// `radius`, and on the `(sx, sy)` side of the center.
    Arc {
        center: (f32, f32),
        radius: f32,
        half_width: f32,
        sx: f32,
        sy: f32,
    },
    /// A filled circle: the marker for a commit a ref points at.
    Disc { center: (f32, f32), radius: f32 },
    /// An annulus: the marker for an ordinary commit, and the outer half
    /// of a merge's double ring.
    Ring {
        center: (f32, f32),
        radius: f32,
        half_width: f32,
    },
    /// A piece of a ring between two angles, in radians, screen-oriented
    /// (0 is to the right, `pi/2` straight down): one dash of the WIP
    /// marker's dashed outline.
    RingArc {
        center: (f32, f32),
        radius: f32,
        half_width: f32,
        from: f32,
        to: f32,
    },
}

impl Shape {
    fn contains(&self, p: (f32, f32)) -> bool {
        match *self {
            Shape::Segment { a, b, half_width } => distance_to_segment(p, a, b) <= half_width,
            Shape::Arc {
                center,
                radius,
                half_width,
                sx,
                sy,
            } => {
                let dx = p.0 - center.0;
                let dy = p.1 - center.1;
                if dx * sx < 0.0 || dy * sy < 0.0 {
                    return false;
                }
                ((dx * dx + dy * dy).sqrt() - radius).abs() <= half_width
            }
            Shape::Disc { center, radius } => {
                let dx = p.0 - center.0;
                let dy = p.1 - center.1;
                dx * dx + dy * dy <= radius * radius
            }
            Shape::Ring {
                center,
                radius,
                half_width,
            } => {
                let dx = p.0 - center.0;
                let dy = p.1 - center.1;
                ((dx * dx + dy * dy).sqrt() - radius).abs() <= half_width
            }
            Shape::RingArc {
                center,
                radius,
                half_width,
                from,
                to,
            } => {
                let dx = p.0 - center.0;
                let dy = p.1 - center.1;
                if ((dx * dx + dy * dy).sqrt() - radius).abs() > half_width {
                    return false;
                }
                // The angle relative to `from`, wrapped into [0, tau), so
                // an arc straddling the +x axis needs no special case.
                let tau = std::f32::consts::TAU;
                let rel = (dy.atan2(dx) - from).rem_euclid(tau);
                rel <= (to - from).rem_euclid(tau)
            }
        }
    }

    /// The pixel rectangle that can possibly be touched, so a cell's
    /// shapes cost their own area rather than the whole bitmap's.
    fn bounds(&self) -> (f32, f32, f32, f32) {
        match *self {
            Shape::Segment { a, b, half_width } => (
                a.0.min(b.0) - half_width,
                a.1.min(b.1) - half_width,
                a.0.max(b.0) + half_width,
                a.1.max(b.1) + half_width,
            ),
            Shape::Arc {
                center,
                radius,
                half_width,
                ..
            }
            | Shape::Ring {
                center,
                radius,
                half_width,
            }
            | Shape::RingArc {
                center,
                radius,
                half_width,
                ..
            } => {
                let r = radius + half_width;
                (center.0 - r, center.1 - r, center.0 + r, center.1 + r)
            }
            Shape::Disc { center, radius } => (
                center.0 - radius,
                center.1 - radius,
                center.0 + radius,
                center.1 + radius,
            ),
        }
    }
}

pub(super) fn distance_to_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
    let len2 = vx * vx + vy * vy;
    let t = if len2 <= f32::EPSILON {
        0.0
    } else {
        (((p.0 - a.0) * vx + (p.1 - a.1) * vy) / len2).clamp(0.0, 1.0)
    };
    let (dx, dy) = (p.0 - (a.0 + t * vx), p.1 - (a.1 + t * vy));
    (dx * dx + dy * dy).sqrt()
}

/// A stash's box: a square outline of four round-capped strokes, `h` from
/// the center to each side's center line.
pub(super) fn push_box(out: &mut Vec<Shape>, (cx, cy): (f32, f32), h: f32, half_width: f32) {
    let corners = [
        (cx - h, cy - h),
        (cx + h, cy - h),
        (cx + h, cy + h),
        (cx - h, cy + h),
    ];
    for i in 0..4 {
        out.push(Shape::Segment {
            a: corners[i],
            b: corners[(i + 1) % 4],
            half_width,
        });
    }
}

/// How many dashes the WIP marker's ring is cut into. Eight reads as
/// "dashed" rather than "broken" at the maintainer's 9 px cell and stays
/// clearly a circle; each dash covers [`RING_DASH_DUTY`] of its arc.
pub(super) const RING_DASHES: usize = 8;
/// The share of each dash's arc that is inked; the rest is the gap.
pub(super) const RING_DASH_DUTY: f32 = 0.55;

/// The WIP marker: a ring cut into [`RING_DASHES`] arcs. The first dash
/// is centered on the ring's bottom, so the connector leaving downwards
/// always meets ink rather than a gap.
pub(super) fn push_dashed_ring(
    out: &mut Vec<Shape>,
    center: (f32, f32),
    radius: f32,
    half_width: f32,
) {
    let step = std::f32::consts::TAU / RING_DASHES as f32;
    let half_dash = step * RING_DASH_DUTY / 2.0;
    // Screen y grows downwards, so "bottom" is +pi/2.
    let bottom = std::f32::consts::FRAC_PI_2;
    for k in 0..RING_DASHES {
        let mid = bottom + step * k as f32;
        out.push(Shape::RingArc {
            center,
            radius,
            half_width,
            from: mid - half_dash,
            to: mid + half_dash,
        });
    }
}

/// A dash of the WIP stem, as the eye sees it: cap to cap, in assumed
/// cell pixels.
pub(super) const DASH_VISIBLE: f32 = 4.0;
/// The clear background between two dashes, in assumed cell pixels.
pub(super) const DASH_GAP: f32 = 3.5;

/// A dashed run: the WIP connector, which is provisional rather than
/// recorded history. Only ever axis-aligned (a lane is vertical, a run
/// horizontal), which is what lets the pattern be anchored globally.
///
/// **The lengths are measured cap to cap.** A dash is a round-capped
/// segment, each cap extending it by `half_width` past its end points, so
/// the core segment is shortened by a cap at each end, down to a round dot
/// when the stroke is thicker than the dash is long. Spacing the end
/// points instead leaves 0.4 px of air at a 9 px cell, which
/// anti-aliasing fills in and the stem reads as solid.
///
/// **The pattern is anchored to the bitmap, not to the segment.** Dash
/// `k` covers `[k * period, k * period + visible)` in absolute bitmap
/// pixels along the axis, and each segment draws only the part of that
/// pattern it spans. The WIP connector is several short pieces in
/// different cells (the marker's `down`, lane cells, HEAD's `up`); a
/// pattern restarting at each piece would put a dash on both sides of
/// every cell edge. Anchored globally, the pieces read as one evenly
/// dashed line.
pub(super) fn push_dashes(
    out: &mut Vec<Shape>,
    a: (f32, f32),
    b: (f32, f32),
    half_width: f32,
    scale: f32,
) {
    let vertical = (b.1 - a.1).abs() >= (b.0 - a.0).abs();
    let (lo, hi) = if vertical {
        (a.1.min(b.1), a.1.max(b.1))
    } else {
        (a.0.min(b.0), a.0.max(b.0))
    };
    if hi - lo <= f32::EPSILON {
        return;
    }
    let at = |u: f32| if vertical { (a.0, u) } else { (u, a.1) };
    let visible = DASH_VISIBLE * scale;
    let period = visible + DASH_GAP * scale;
    let cap = half_width.min(visible / 2.0);
    let core = (visible - 2.0 * half_width).max(0.0);
    // One period early, so a dash that starts before `lo` and reaches
    // into this segment is not skipped.
    let mut k = (lo / period).floor() - 1.0;
    loop {
        let core_start = k * period + cap;
        let core_end = core_start + core;
        if core_start - cap > hi {
            break;
        }
        // Clip the dash's *core* to the segment. The cap then overhangs
        // the segment's end by at most `half_width` -- into the next
        // cell, where the same dash continues, or into the marker the
        // segment runs to.
        let (s, e) = (core_start.max(lo), core_end.min(hi));
        if s <= e && core_end + cap >= lo {
            out.push(Shape::Segment {
                a: at(s),
                b: at(e),
                half_width,
            });
        }
        k += 1.0;
    }
}

/// A rounded corner: two straight arms and the quarter-arc fillet joining
/// them, all three in one cell.
///
/// `sx` is +1 when the horizontal arm leaves to the right, `sy` is +1 when
/// the vertical arm leaves downwards - so the four combinations are
/// exactly `╭ ╮ ╰ ╯`.
pub(super) fn elbow(
    out: &mut Vec<Shape>,
    (cx, cy): (f32, f32),
    (x0, y0): (f32, f32),
    (cw, ch): (f32, f32),
    opt: &RasterOptions,
    sx: f32,
    sy: f32,
) {
    let hw = stroke_half_width(opt);
    let rr = corner_radius(opt);
    // Tangent points: the arc meets the vertical arm at (cx, cy + sy*rr)
    // and the horizontal arm at (cx + sx*rr, cy).
    out.push(Shape::Segment {
        a: (cx, cy + sy * rr),
        b: (cx, if sy > 0.0 { y0 + ch } else { y0 }),
        half_width: hw,
    });
    out.push(Shape::Segment {
        a: (cx + sx * rr, cy),
        b: (if sx > 0.0 { x0 + cw } else { x0 }, cy),
        half_width: hw,
    });
    // The fillet's center is diagonally opposite the corner, so the arc
    // bulges towards the corner and is tangent to both arms.
    out.push(Shape::Arc {
        center: (cx + sx * rr, cy + sy * rr),
        radius: rr,
        half_width: hw,
        sx: -sx,
        sy: -sy,
    });
}

pub(super) fn draw(bmp: &mut Bitmap, shape: &Shape, color: (u8, u8, u8), alpha: f32, samples: u32) {
    let (lx, ly, hx, hy) = shape.bounds();
    let x0 = lx.floor().max(0.0) as u32;
    let y0 = ly.floor().max(0.0) as u32;
    let x1 = (hx.ceil().max(0.0) as u32 + 1).min(bmp.width);
    let y1 = (hy.ceil().max(0.0) as u32 + 1).min(bmp.height);
    let n = samples as f32;
    for y in y0..y1 {
        for x in x0..x1 {
            let mut hits = 0u32;
            for sy in 0..samples {
                for sx in 0..samples {
                    let p = (
                        x as f32 + (sx as f32 + 0.5) / n,
                        y as f32 + (sy as f32 + 0.5) / n,
                    );
                    if shape.contains(p) {
                        hits += 1;
                    }
                }
            }
            if hits > 0 {
                bmp.blend(x, y, color, alpha * hits as f32 / (n * n));
            }
        }
    }
}
