//! Vector painter for the TUI's graph cells (`lanes.rs`). The geometry is a port of
//! `crates/gitpoulpi-tui/src/graph/raster/{metrics,shape}.rs` and of `shapes_for` in `raster.rs`:
//! same stroke width, node radius, ring thickness, elbow fillets, dash pattern and pending alpha,
//! with the cell size (9 x 22 px here) standing where the terminal's cell pixels stood. A lane takes
//! two cells (glyph + connector), exactly as in the TUI.

use std::sync::Arc;

use gpui_kit::{fill, point, px, size, Bounds, Hsla, PathBuilder, Pixels, Point, Window};

use super::lanes::{ColorRef, GraphRow, NodeKind, Sides, TreeCell, TreePart};

pub const CW: f32 = 9.0;
pub const CH: f32 = 22.0;

/// Colours of the active theme and lane palette.
#[derive(Clone)]
pub struct Palette {
    pub lanes: Arc<Vec<Hsla>>,
    pub wip: Hsla,
    pub stash: Hsla,
}

impl Palette {
    pub(super) fn color(&self, c: ColorRef) -> Hsla {
        match c {
            ColorRef::Lane(n) => self.lanes[n % self.lanes.len().max(1)],
            ColorRef::Wip => self.wip,
            ColorRef::Stash => self.stash,
        }
    }
}

fn hw() -> f32 {
    (CW / 7.0).clamp(0.85, 9.0 / 7.0)
}
fn node_r() -> f32 {
    let from_h = CH * 0.3884 / 2.0;
    let ceil_w = CW * 0.44;
    from_h.min(ceil_w).max(hw() * 2.4)
}
fn ring_hw() -> f32 {
    (hw() * 0.80).max(0.6)
}
fn corner_r() -> f32 {
    (CW / 2.0).min(CH / 2.0) * 0.85
}

type P = (f32, f32);

struct Ctx<'a> {
    ox: f32,
    oy: f32,
    /// absolute y of this row's top, in cell pixels, for a dash phase continuous across rows
    phase_y: f32,
    window: &'a mut Window,
}

impl Ctx<'_> {
    fn pt(&self, p: P) -> Point<Pixels> {
        point(px(self.ox + p.0), px(self.oy + p.1))
    }

    fn stroke(
        &mut self,
        color: Hsla,
        width: f32,
        f: impl FnOnce(&mut PathBuilder, &dyn Fn(P) -> Point<Pixels>),
    ) {
        let (ox, oy) = (self.ox, self.oy);
        let map = move |p: P| point(px(ox + p.0), px(oy + p.1));
        let mut b = PathBuilder::stroke(px(width));
        f(&mut b, &map);
        if let Ok(path) = b.build() {
            self.window.paint_path(path, color);
        }
    }

    fn segment(&mut self, color: Hsla, a: P, b: P) {
        self.stroke(color, hw() * 2.0, |pb, m| {
            pb.move_to(m(a));
            pb.line_to(m(b));
        });
    }

    /// The same dash pattern as `push_dashes`: 4 px visible, 3.5 px gap, phase tied to the absolute
    /// position so consecutive rows continue the pattern.
    fn dashes(&mut self, color: Hsla, a: P, b: P) {
        let vertical = (b.1 - a.1).abs() >= (b.0 - a.0).abs();
        let (lo, hi) = if vertical {
            (a.1.min(b.1), a.1.max(b.1))
        } else {
            (a.0.min(b.0), a.0.max(b.0))
        };
        if hi - lo <= f32::EPSILON {
            return;
        }
        let off = if vertical { self.phase_y } else { 0.0 };
        let at = |u: f32| if vertical { (a.0, u) } else { (u, a.1) };
        let visible = 4.0;
        let period = visible + 3.5;
        let half = hw();
        let cap = half.min(visible / 2.0);
        let core = (visible - 2.0 * half).max(0.0);
        let mut k = ((lo + off) / period).floor() - 1.0;
        loop {
            let core_start = k * period + cap - off;
            let core_end = core_start + core;
            if core_start - cap > hi {
                break;
            }
            let (s, e) = (core_start.max(lo), core_end.min(hi));
            if s <= e && core_end + cap >= lo {
                self.segment(color, at(s), at(e));
            }
            k += 1.0;
        }
    }

    fn arc_cubic(
        pb: &mut PathBuilder,
        m: &dyn Fn(P) -> Point<Pixels>,
        center: P,
        r: f32,
        a0: f32,
        a1: f32,
    ) {
        // split into <= 90 degree pieces
        let n = (((a1 - a0).abs() / std::f32::consts::FRAC_PI_2).ceil()).max(1.0) as usize;
        let step = (a1 - a0) / n as f32;
        let k = 4.0 / 3.0 * (step / 4.0).tan();
        for i in 0..n {
            let t0 = a0 + step * i as f32;
            let t1 = t0 + step;
            let p0 = (center.0 + r * t0.cos(), center.1 + r * t0.sin());
            let p3 = (center.0 + r * t1.cos(), center.1 + r * t1.sin());
            let c1 = (p0.0 - k * r * t0.sin(), p0.1 + k * r * t0.cos());
            let c2 = (p3.0 + k * r * t1.sin(), p3.1 - k * r * t1.cos());
            if i == 0 {
                pb.move_to(m(p0));
            }
            pb.cubic_bezier_to(m(p3), m(c1), m(c2));
        }
    }

    fn ring(&mut self, color: Hsla, center: P, r: f32, half_w: f32) {
        self.stroke(color, half_w * 2.0, |pb, m| {
            Self::arc_cubic(pb, m, center, r, 0.0, std::f32::consts::TAU);
            pb.close();
        });
    }

    fn ring_arc(&mut self, color: Hsla, center: P, r: f32, half_w: f32, from: f32, to: f32) {
        self.stroke(color, half_w * 2.0, |pb, m| {
            Self::arc_cubic(pb, m, center, r, from, to)
        });
    }

    fn disc(&mut self, color: Hsla, center: P, r: f32) {
        let b = Bounds::new(
            self.pt((center.0 - r, center.1 - r)),
            size(px(r * 2.0), px(r * 2.0)),
        );
        self.window.paint_quad(fill(b, color).corner_radii(px(r)));
    }

    /// A corner: a vertical arm, a quarter-circle fillet, a horizontal arm. `cell` is the cell's origin
    /// and size, `(sx, sy)` the direction the arms point, `(dash_v, dash_h)` which arm is dashed.
    fn elbow(
        &mut self,
        color: Hsla,
        center: P,
        cell: (P, P),
        (sx, sy): P,
        (dash_v, dash_h): (bool, bool),
    ) {
        let (cx, cy) = center;
        let ((x0, y0), (cw, ch)) = cell;
        let rr = corner_r();
        let v_end = (cx, if sy > 0.0 { y0 + ch } else { y0 });
        let h_end = (if sx > 0.0 { x0 + cw } else { x0 }, cy);
        // vertical arm, fillet, horizontal arm as one stroked path when solid
        if !dash_v && !dash_h {
            self.stroke(color, hw() * 2.0, |pb, m| {
                pb.move_to(m(v_end));
                pb.line_to(m((cx, cy + sy * rr)));
                // quarter circle centred diagonally opposite the corner
                let start = (cx, cy + sy * rr);
                let end = (cx + sx * rr, cy);
                let k = 0.5523 * rr;
                let c1 = (start.0, start.1 - sy * k);
                let c2 = (end.0 - sx * k, end.1);
                pb.cubic_bezier_to(m(end), m(c1), m(c2));
                pb.line_to(m(h_end));
            });
        } else {
            if dash_v {
                self.dashes(color, (cx, cy + sy * rr), v_end);
            } else {
                self.segment(color, (cx, cy + sy * rr), v_end);
            }
            if dash_h {
                self.dashes(color, (cx + sx * rr, cy), h_end);
            } else {
                self.segment(color, (cx + sx * rr, cy), h_end);
            }
            let start = (cx, cy + sy * rr);
            let end = (cx + sx * rr, cy);
            let k = 0.5523 * rr;
            let c1 = (start.0, start.1 - sy * k);
            let c2 = (end.0 - sx * k, end.1);
            self.stroke(color, hw() * 2.0, |pb, m| {
                pb.move_to(m(start));
                pb.cubic_bezier_to(m(end), m(c1), m(c2));
            });
        }
    }
}

fn dim(c: Hsla) -> Hsla {
    c.opacity(0.62)
}

fn paint_cell(ctx: &mut Ctx, cell: &TreeCell, x0: f32, color: Hsla) {
    let (cw, ch) = (CW, CH);
    let y0 = 0.0;
    let (cx, cy) = (x0 + cw / 2.0, y0 + ch / 2.0);
    let d = cell.dashed;
    let pend = dim(color);
    let vert_dashed = d.up || d.down;

    let seg = |ctx: &mut Ctx, a: P, b: P, dash: bool| {
        if dash {
            ctx.dashes(pend, a, b);
        } else {
            ctx.segment(color, a, b);
        }
    };

    match cell.part {
        TreePart::Empty => {}
        TreePart::Vertical => seg(ctx, (cx, y0), (cx, y0 + ch), vert_dashed),
        TreePart::Horizontal => seg(ctx, (x0, cy), (x0 + cw, cy), d.left || d.right),
        TreePart::Crossing => {
            seg(ctx, (cx, y0), (cx, y0 + ch), vert_dashed);
            seg(ctx, (x0, cy), (x0 + cw, cy), d.left || d.right);
        }
        TreePart::BranchOutCrossed => {
            seg(ctx, (x0, cy), (x0 + cw, cy), d.left || d.right);
            seg(ctx, (cx, cy), (cx, y0 + ch), d.down);
        }
        TreePart::MergeInCrossed => {
            seg(ctx, (x0, cy), (x0 + cw, cy), d.left || d.right);
            seg(ctx, (cx, y0), (cx, cy), d.up);
        }
        TreePart::BranchOutLeft
        | TreePart::BranchOutRight
        | TreePart::MergeInLeft
        | TreePart::MergeInRight => {
            let s: Sides = cell.part.sides();
            let sx = if s.right { 1.0 } else { -1.0 };
            let sy = if s.down { 1.0 } else { -1.0 };
            let dv = d.up || d.down;
            let dh = d.left || d.right;
            let col = if dv || dh { pend } else { color };
            ctx.elbow(col, (cx, cy), ((x0, y0), (cw, ch)), (sx, sy), (dv, dh));
        }
        TreePart::Node { kind, sides } => {
            let r = node_r();
            let rhw = ring_hw();
            let stop = r - rhw * 0.5;
            if sides.up {
                seg(ctx, (cx, y0), (cx, cy - stop), d.up);
            }
            if sides.down {
                seg(ctx, (cx, cy + stop), (cx, y0 + ch), d.down);
            }
            if sides.left {
                seg(ctx, (x0, cy), (cx - stop, cy), d.left);
            }
            if sides.right {
                seg(ctx, (cx + stop, cy), (x0 + cw, cy), d.right);
            }
            match kind {
                NodeKind::Commit => ctx.ring(color, (cx, cy), r - rhw, rhw),
                NodeKind::Ref => ctx.disc(color, (cx, cy), r),
                NodeKind::Merge => {
                    ctx.ring(color, (cx, cy), r - rhw, rhw);
                    ctx.disc(color, (cx, cy), ((r - rhw * 2.0) * 0.55).max(0.6));
                }
                NodeKind::Wip => {
                    let step = std::f32::consts::TAU / 8.0;
                    let half = step * 0.55 / 2.0;
                    for k in 0..8 {
                        let mid = std::f32::consts::FRAC_PI_2 + step * k as f32;
                        ctx.ring_arc(pend, (cx, cy), r - rhw, rhw, mid - half, mid + half);
                    }
                }
                NodeKind::Stash => {
                    let h = r - rhw;
                    let c = [
                        (cx - h, cy - h),
                        (cx + h, cy - h),
                        (cx + h, cy + h),
                        (cx - h, cy + h),
                    ];
                    ctx.stroke(color, rhw * 2.0, |pb, m| {
                        pb.move_to(m(c[0]));
                        pb.line_to(m(c[1]));
                        pb.line_to(m(c[2]));
                        pb.line_to(m(c[3]));
                        pb.close();
                    });
                }
            }
        }
    }
}

/// Paints one row's graph into `bounds` (clipped by the caller's overflow). With `connector`, the
/// ref-to-commit connector is laid first: a one-pixel line in the node's lane colour from the left
/// edge (where the Ref column's pill ends) to the node, as `draw_ref_connector` does.
pub(super) fn paint_row(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    row: &GraphRow,
    row_index: usize,
    connector: bool,
    pal: &Palette,
) {
    let mut ctx = Ctx {
        ox: bounds.origin.x.as_f32(),
        oy: bounds.origin.y.as_f32(),
        phase_y: row_index as f32 * CH,
        window,
    };
    if connector {
        let node = row.lane * 2;
        if let Some(cell) = row.cells.get(node) {
            let cx = node as f32 * CW + CW / 2.0;
            let end = cx - node_r() + ring_hw();
            let cy = (CH / 2.0).floor() + 0.5;
            if end > 0.0 {
                let color = pal.color(cell.color);
                ctx.stroke(color, 1.0, |pb, m| {
                    pb.move_to(m((0.0, cy)));
                    pb.line_to(m((end, cy)));
                });
            }
        }
    }
    for (c, cell) in row.cells.iter().enumerate() {
        if cell.part == TreePart::Empty {
            continue;
        }
        let x0 = c as f32 * CW;
        if x0 > bounds.size.width.as_f32() {
            break;
        }
        paint_cell(&mut ctx, cell, x0, pal.color(cell.color));
    }
}
