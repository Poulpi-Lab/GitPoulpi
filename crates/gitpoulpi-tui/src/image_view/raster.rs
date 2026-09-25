//! Rasterising only what is visible (decision D5): crop the source to the
//! [`View`], resample it to the destination pixel size and composite alpha
//! over a checkerboard, so every protocol (and the half-block fallback)
//! shows transparency the same way.
//!
//! * Shrinking (`scale < 1`): area average over the crop. Up to 8 source
//!   pixels per axis are taken per output pixel (exact coverage weights);
//!   beyond that 8 evenly spaced ones are, which bounds the cost at 64
//!   taps per output pixel however far the image is zoomed out.
//! * `1 <= scale < 2`: bilinear.
//! * `scale >= 2`: nearest neighbor, so pixels stay pixels (icons, pixel
//!   art).
//!
//! Large outputs are split into row bands over a few scoped threads; the
//! per-pixel work is a fixed-size gather, so the cost is linear in the
//! *output* size, not the source size.

use crate::terminal_image::Bitmap;

use super::viewport::View;

/// Above this many output pixels a pane is rasterised smaller and the
/// terminal upscales it (decision D5).
pub(crate) const MAX_OUTPUT_PIXELS: u64 = 2_500_000;
/// Source pixels read per output pixel and axis, at most.
const MAX_TAPS: usize = 8;
/// Rows are split over threads from this many output pixels up.
const PARALLEL_FROM: u64 = 150_000;

/// The two grays of the transparency checkerboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Checker {
    pub light: (u8, u8, u8),
    pub dark: (u8, u8, u8),
    /// Square size in pixels.
    pub cell: u32,
}

impl Checker {
    /// Grays derived from the theme's background so the checkerboard sits
    /// quietly in any theme: both steps move away from the background
    /// towards the side with more room.
    pub(crate) fn from_background(bg: (u8, u8, u8)) -> Checker {
        let lum = (u32::from(bg.0) * 299 + u32::from(bg.1) * 587 + u32::from(bg.2) * 114) / 1000;
        let shift = |c: (u8, u8, u8), d: i32| {
            let f = |v: u8| (i32::from(v) + d).clamp(0, 255) as u8;
            (f(c.0), f(c.1), f(c.2))
        };
        let sign = if lum < 128 { 1 } else { -1 };
        Checker {
            light: shift(bg, sign * 22),
            dark: shift(bg, sign * 46),
            cell: 8,
        }
    }

    fn at(&self, x: u32, y: u32) -> (u8, u8, u8) {
        if ((x / self.cell) + (y / self.cell)) % 2 == 0 {
            self.light
        } else {
            self.dark
        }
    }
}

/// `(w, h)` shrunk, keeping the aspect, so `w * h <= max_px`.
pub(crate) fn cap_output(w: u32, h: u32, max_px: u64) -> (u32, u32) {
    let px = u64::from(w) * u64::from(h);
    if px <= max_px || px == 0 {
        return (w, h);
    }
    let f = (max_px as f64 / px as f64).sqrt();
    (
        ((f64::from(w) * f).floor() as u32).max(1),
        ((f64::from(h) * f).floor() as u32).max(1),
    )
}

/// The destination size in whole pixels for a view.
pub(crate) fn dst_size(view: &View) -> (u32, u32) {
    (
        (view.dst.2.round() as u32).max(1),
        (view.dst.3.round() as u32).max(1),
    )
}

#[derive(Clone, Copy)]
struct Taps {
    n: usize,
    idx: [u32; MAX_TAPS],
    w: [f32; MAX_TAPS],
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Area,
    Bilinear,
    Nearest,
}

/// The source pixels (and weights) every output pixel of one axis reads.
fn axis_taps(start: f32, len: f32, dst: u32, src_len: u32, mode: Mode) -> Vec<Taps> {
    let ratio = len / dst as f32;
    let last = src_len.saturating_sub(1);
    (0..dst)
        .map(|i| {
            let mut t = Taps {
                n: 0,
                idx: [0; MAX_TAPS],
                w: [0.0; MAX_TAPS],
            };
            let mut push = |k: i64, w: f32| {
                t.idx[t.n] = k.clamp(0, i64::from(last)) as u32;
                t.w[t.n] = w;
                t.n += 1;
            };
            match mode {
                Mode::Nearest => push((start + (i as f32 + 0.5) * ratio).floor() as i64, 1.0),
                Mode::Bilinear => {
                    let c = (start + (i as f32 + 0.5) * ratio - 0.5).clamp(0.0, last as f32);
                    let k = c.floor();
                    let f = c - k;
                    push(k as i64, 1.0 - f);
                    push(k as i64 + 1, f);
                }
                Mode::Area => {
                    let (a, b) = (start + i as f32 * ratio, start + (i as f32 + 1.0) * ratio);
                    let first = a.floor() as i64;
                    let end = (b.ceil() as i64).max(first + 1);
                    if (end - first) as usize <= MAX_TAPS {
                        let mut total = 0.0;
                        let mut ws = [0.0f32; MAX_TAPS];
                        for (j, k) in (first..end).enumerate() {
                            ws[j] = (b.min(k as f32 + 1.0) - a.max(k as f32)).max(0.0);
                            total += ws[j];
                        }
                        let total = total.max(f32::EPSILON);
                        for (j, k) in (first..end).enumerate() {
                            push(k, ws[j] / total);
                        }
                    } else {
                        for j in 0..MAX_TAPS {
                            let p = a + (j as f32 + 0.5) * ratio / MAX_TAPS as f32;
                            push(p.floor() as i64, 1.0 / MAX_TAPS as f32);
                        }
                    }
                }
            }
            t
        })
        .collect()
}

/// The scale from which the picture is enlarged as square pixels by
/// default (`ui.image_view_pixelated_from = "2x"`).
pub(crate) const DEFAULT_PIXELATED_FROM: f32 = 2.0;

fn mode_for(scale: f32, pixelated_from: f32) -> Mode {
    if scale < 1.0 {
        Mode::Area
    } else if scale < pixelated_from {
        Mode::Bilinear
    } else {
        Mode::Nearest
    }
}

/// Renders `view` of `src` into an opaque bitmap of [`dst_size`] pixels.
/// The caller places it at `view.dst`'s origin; the margin around a
/// centered small image is the pane's own background, not drawn here.
pub(crate) fn render(src: &Bitmap, view: &View, checker: &Checker) -> Bitmap {
    let (dw, dh) = dst_size(view);
    render_to(src, view, checker, dw, dh)
}

/// As [`render`], at an explicit (typically [`cap_output`]-reduced) size.
pub(crate) fn render_to(src: &Bitmap, view: &View, checker: &Checker, dw: u32, dh: u32) -> Bitmap {
    render_to_from(src, view, checker, dw, dh, DEFAULT_PIXELATED_FROM)
}

/// As [`render_to`], enlarging as square pixels from `pixelated_from` (a
/// scale: `1.0`, `2.0`, or infinity for never) and smoothing below it.
pub(crate) fn render_to_from(
    src: &Bitmap,
    view: &View,
    checker: &Checker,
    dw: u32,
    dh: u32,
    pixelated_from: f32,
) -> Bitmap {
    let mut out = Bitmap::new(dw, dh, (0, 0, 0));
    if src.width == 0
        || src.height == 0
        || src.data.len() < (src.width as usize) * (src.height as usize) * 4
    {
        return out;
    }
    let (sx, sy, sw, sh) = view.src;
    let mode = mode_for(view.scale, pixelated_from);
    let xt = axis_taps(sx, sw, dw, src.width, mode);
    let yt = axis_taps(sy, sh, dh, src.height, mode);
    let row_bytes = dw as usize * 4;
    let total = u64::from(dw) * u64::from(dh);
    let threads = if total >= PARALLEL_FROM {
        std::thread::available_parallelism()
            .map_or(1, |n| n.get())
            .min(8)
    } else {
        1
    };
    let rows_per = (dh as usize).div_ceil(threads).max(1);
    let fill = |first_row: usize, band: &mut [u8]| {
        for (r, line) in band.chunks_exact_mut(row_bytes).enumerate() {
            let y = first_row + r;
            let ty = &yt[y];
            for (x, px) in line.chunks_exact_mut(4).enumerate() {
                let tx = &xt[x];
                let (mut ar, mut ag, mut ab, mut aa) = (0f32, 0f32, 0f32, 0f32);
                for j in 0..ty.n {
                    let row = ty.idx[j] as usize * src.width as usize;
                    for i in 0..tx.n {
                        let w = ty.w[j] * tx.w[i];
                        let o = (row + tx.idx[i] as usize) * 4;
                        let p = &src.data[o..o + 4];
                        let a = if p[3] == 255 {
                            1.0
                        } else {
                            f32::from(p[3]) / 255.0
                        };
                        let wa = w * a;
                        ar += wa * f32::from(p[0]);
                        ag += wa * f32::from(p[1]);
                        ab += wa * f32::from(p[2]);
                        aa += wa;
                    }
                }
                let rest = (1.0 - aa).clamp(0.0, 1.0);
                let (cr, cg, cb) = if rest > 0.0 {
                    checker.at(x as u32, y as u32)
                } else {
                    (0, 0, 0)
                };
                px[0] = (ar + rest * f32::from(cr)).round().clamp(0.0, 255.0) as u8;
                px[1] = (ag + rest * f32::from(cg)).round().clamp(0.0, 255.0) as u8;
                px[2] = (ab + rest * f32::from(cb)).round().clamp(0.0, 255.0) as u8;
                px[3] = 255;
            }
        }
    };
    if threads <= 1 {
        fill(0, &mut out.data);
    } else {
        std::thread::scope(|s| {
            for (n, band) in out.data.chunks_mut(rows_per * row_bytes).enumerate() {
                let fill = &fill;
                s.spawn(move || fill(n * rows_per, band));
            }
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::viewport::{Size, Viewport, Zoom};
    use super::*;

    fn bmp(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Bitmap {
        let mut data = Vec::new();
        for y in 0..h {
            for x in 0..w {
                data.extend(f(x, y));
            }
        }
        Bitmap {
            width: w,
            height: h,
            data,
        }
    }

    const CK: Checker = Checker {
        light: (100, 100, 100),
        dark: (60, 60, 60),
        cell: 8,
    };

    fn view_of(img: &Bitmap, zoom: Zoom, pane: (f32, f32)) -> View {
        Viewport {
            zoom,
            center: (0.5, 0.5),
        }
        .view(
            Size::px(img.width, img.height),
            Size::new(pane.0, pane.1),
            true,
        )
    }

    #[test]
    fn nearest_from_two_times_keeps_pixels_square() {
        let img = bmp(2, 1, |x, _| {
            if x == 0 {
                [255, 0, 0, 255]
            } else {
                [0, 0, 255, 255]
            }
        });
        let v = view_of(&img, Zoom::Scale(4.0), (8.0, 4.0));
        let out = render(&img, &v, &CK);
        assert_eq!((out.width, out.height), (8, 4));
        for y in 0..4 {
            for x in 0..8 {
                let want = if x < 4 { (255, 0, 0) } else { (0, 0, 255) };
                assert_eq!(out.pixel(x, y), Some(want), "({x},{y})");
            }
        }
    }

    #[test]
    fn between_one_and_two_times_it_is_bilinear() {
        let img = bmp(2, 1, |x, _| {
            if x == 0 {
                [0, 0, 0, 255]
            } else {
                [200, 200, 200, 255]
            }
        });
        let v = view_of(&img, Zoom::Scale(1.5), (3.0, 1.5));
        let out = render(&img, &v, &CK);
        let mid = out.pixel(1, 0).unwrap().0;
        assert!(mid > 0 && mid < 200, "middle pixel blends: {mid}");
    }

    #[test]
    fn shrinking_averages_instead_of_picking() {
        let img = bmp(64, 64, |x, y| {
            if (x + y) % 2 == 0 {
                [255, 255, 255, 255]
            } else {
                [0, 0, 0, 255]
            }
        });
        let v = view_of(&img, Zoom::Fit, (8.0, 8.0));
        let out = render(&img, &v, &CK);
        for y in 0..8 {
            for x in 0..8 {
                let (r, g, b) = out.pixel(x, y).unwrap();
                assert!(
                    (126..=129).contains(&r) && r == g && g == b,
                    "({x},{y}) {r}"
                );
            }
        }
    }

    #[test]
    fn only_the_visible_crop_is_drawn() {
        // 100x10: left half red, right half blue; a 10x10 pane at 1:1
        // centered on the right half shows only blue.
        let img = bmp(100, 10, |x, _| {
            if x < 50 {
                [255, 0, 0, 255]
            } else {
                [0, 0, 255, 255]
            }
        });
        let v = Viewport {
            zoom: Zoom::Scale(1.0),
            center: (0.9, 0.5),
        }
        .view(Size::px(100, 10), Size::new(10.0, 10.0), true);
        let out = render(&img, &v, &CK);
        assert!((0..10).all(|x| out.pixel(x, 5) == Some((0, 0, 255))));
        let v = Viewport {
            zoom: Zoom::Scale(1.0),
            center: (0.1, 0.5),
        }
        .view(Size::px(100, 10), Size::new(10.0, 10.0), true);
        assert_eq!(render(&img, &v, &CK).pixel(0, 0), Some((255, 0, 0)));
    }

    #[test]
    fn transparency_shows_the_checkerboard() {
        let img = bmp(16, 16, |_, _| [255, 0, 0, 0]);
        let v = view_of(&img, Zoom::Scale(1.0), (16.0, 16.0));
        let out = render(&img, &v, &CK);
        assert_eq!(out.pixel(0, 0), Some((100, 100, 100)));
        assert_eq!(out.pixel(8, 0), Some((60, 60, 60)));
        assert_eq!(out.pixel(0, 8), Some((60, 60, 60)));
        assert_eq!(out.pixel(8, 8), Some((100, 100, 100)));
        // Half transparent red over the light gray: the mean.
        let half = bmp(16, 16, |_, _| [200, 0, 0, 128]);
        let out = render(&half, &v, &CK);
        let (r, g, _) = out.pixel(0, 0).unwrap();
        assert!(r > 140 && r < 160 && g > 45 && g < 55, "{r},{g}");
        // Opaque pixels hide it entirely.
        let solid = bmp(16, 16, |_, _| [1, 2, 3, 255]);
        assert_eq!(render(&solid, &v, &CK).pixel(9, 9), Some((1, 2, 3)));
    }

    #[test]
    fn a_translucent_edge_keeps_the_pure_color_when_averaged() {
        // Straight alpha: a transparent pixel's color must not bleed into
        // a neighbor when they are averaged (premultiplied accumulation).
        let img = bmp(2, 1, |x, _| {
            if x == 0 {
                [255, 0, 0, 255]
            } else {
                [0, 255, 0, 0]
            }
        });
        let v = view_of(&img, Zoom::Fit, (1.0, 1.0));
        let v = View { scale: 0.5, ..v };
        let out = render_to(&img, &v, &CK, 1, 1);
        let (r, g, _) = out.pixel(0, 0).unwrap();
        assert!(g < 70 && r > 120, "no green bleed: {r},{g}");
    }

    #[test]
    fn the_output_cap_keeps_the_aspect() {
        assert_eq!(cap_output(1000, 500, 2_500_000), (1000, 500));
        let (w, h) = cap_output(4000, 2000, 2_500_000);
        assert!(u64::from(w) * u64::from(h) <= 2_500_000);
        assert!((w as f64 / h as f64 - 2.0).abs() < 0.01);
    }

    #[test]
    fn checker_grays_follow_the_background() {
        let dark = Checker::from_background((20, 20, 30));
        assert!(dark.light.0 > 20 && dark.dark.0 > dark.light.0);
        let light = Checker::from_background((250, 250, 250));
        assert!(light.light.0 < 250 && light.dark.0 < light.light.0);
    }

    #[test]
    fn a_threaded_render_equals_the_single_threaded_one() {
        let img = bmp(700, 500, |x, y| {
            [(x % 256) as u8, (y % 256) as u8, ((x * y) % 256) as u8, 255]
        });
        // 600x400 = 240k px: above the threading threshold.
        let v = View {
            scale: 600.0 / 700.0,
            src: (0.0, 0.0, 700.0, 500.0),
            dst: (0.0, 0.0, 600.0, 400.0),
        };
        let threaded = render(&img, &v, &CK);
        // The same rows, rendered one band at a time through a tiny crop.
        let mut band_data = Vec::new();
        for y in [0u32, 137, 399] {
            let band = View {
                scale: v.scale,
                src: (0.0, 500.0 * y as f32 / 400.0, 700.0, 500.0 / 400.0),
                dst: (0.0, 0.0, 600.0, 1.0),
            };
            band_data.push((y, render_to(&img, &band, &CK, 600, 1)));
        }
        for (y, b) in band_data {
            for x in 0..600 {
                assert_eq!(threaded.pixel(x, y), b.pixel(x, 0), "({x},{y})");
            }
        }
    }

    /// Budget: a 24 MP source into a 2.5 MP pane (a whole-image Fit), and a
    /// 2.5 MP 1:1 crop of it, each inside 60 ms. Release only: a debug
    /// build is an order of magnitude slower and says nothing about it.
    #[test]
    #[cfg_attr(debug_assertions, ignore = "timing budget: release builds only")]
    fn a_big_image_rasterises_inside_a_frame_budget() {
        let img = Bitmap {
            width: 6000,
            height: 4000,
            data: (0..6000u64 * 4000 * 4).map(|i| (i % 251) as u8).collect(),
        };
        let fit = View {
            scale: 2000.0 / 6000.0,
            src: (0.0, 0.0, 6000.0, 4000.0),
            dst: (0.0, 0.0, 1936.0, 1291.0),
        };
        let crop = View {
            scale: 1.0,
            src: (1000.0, 1000.0, 2000.0, 1250.0),
            dst: (0.0, 0.0, 2000.0, 1250.0),
        };
        for (name, v) in [("fit", fit), ("crop", crop)] {
            let t = std::time::Instant::now();
            let out = render(&img, &v, &CK);
            let ms = t.elapsed().as_millis();
            assert_eq!(out.width, dst_size(&v).0);
            assert!(ms < 60, "{name}: {ms} ms");
        }
    }
}
