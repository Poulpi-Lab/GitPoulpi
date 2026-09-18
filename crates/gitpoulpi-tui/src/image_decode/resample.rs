//! Area-average and bilinear resampling of a [`Bitmap`], shared by the
//! avatars (`avatar::resolve`), the graph's picture sampling and the image view.

use crate::terminal_image::Bitmap;

/// The weights (source index, weight) one output pixel takes from a line
/// of `src` source pixels resampled to `dst`.
///
/// **Shrinking is an area average, growing is bilinear.** A picture
/// fetched much larger than its node (the source is deliberately fetched
/// at several times the node's size, see `resolve::Context::fetch_px`)
/// has to be *averaged* down: picking one source pixel per output pixel
/// (nearest-neighbor) throws away all but
/// `1/(f*f)` of the picture and turns every fine detail into a hard,
/// aliased speckle - the "heavily pixelated" avatars in the graph. An
/// output pixel covering `f` source pixels takes every one of them, the
/// two edge pixels in proportion to how much of them it covers.
fn axis_weights(src: u32, dst: u32, out: u32) -> Vec<(u32, f32)> {
    let ratio = src as f64 / dst as f64;
    if ratio > 1.0 {
        let (a, b) = (out as f64 * ratio, (out + 1) as f64 * ratio);
        let first = a.floor() as u32;
        let last = (b.ceil() as u32).min(src).max(first + 1);
        (first..last)
            .map(|k| {
                let overlap = b.min(k as f64 + 1.0) - a.max(k as f64);
                (k.min(src - 1), overlap.max(0.0) as f32)
            })
            .collect()
    } else {
        // Center-aligned linear interpolation between the two source
        // pixels the output pixel's center falls between.
        let center = ((out as f64 + 0.5) * ratio - 0.5).clamp(0.0, (src - 1) as f64);
        let k = center.floor() as u32;
        let t = (center - k as f64) as f32;
        vec![(k, 1.0 - t), ((k + 1).min(src - 1), t)]
    }
}

/// Resizes `bitmap` into exactly `target_w`x`target_h`: an area average
/// when shrinking, bilinear when growing (see [`axis_weights`]), done as
/// two separable passes so the cost is `O(source pixels + target pixels *
/// footprint)` rather than a 2-D kernel per pixel.
pub(crate) fn downscale(bitmap: &Bitmap, target_w: u32, target_h: u32) -> Bitmap {
    if bitmap.width == 0 || bitmap.height == 0 || target_w == 0 || target_h == 0 {
        return Bitmap::new(target_w.max(1), target_h.max(1), (0, 0, 0));
    }
    let (sw, sh) = (bitmap.width, bitmap.height);
    // Horizontal pass: `sh` rows of `target_w` pixels, kept as floats so
    // the second pass averages unrounded values.
    let x_weights: Vec<Vec<(u32, f32)>> = (0..target_w)
        .map(|x| axis_weights(sw, target_w, x))
        .collect();
    let mut mid = vec![0f32; (target_w * sh) as usize * 3];
    for y in 0..sh {
        for (x, taps) in x_weights.iter().enumerate() {
            let mut acc = [0f32; 3];
            let mut total = 0f32;
            for &(k, w) in taps {
                let (r, g, b) = bitmap.pixel(k, y).unwrap_or((0, 0, 0));
                acc[0] += f32::from(r) * w;
                acc[1] += f32::from(g) * w;
                acc[2] += f32::from(b) * w;
                total += w;
            }
            let at = ((y * target_w) as usize + x) * 3;
            for c in 0..3 {
                mid[at + c] = acc[c] / total.max(f32::EPSILON);
            }
        }
    }
    let mut out = Bitmap::new(target_w, target_h, (0, 0, 0));
    for y in 0..target_h {
        let taps = axis_weights(sh, target_h, y);
        for x in 0..target_w {
            let mut acc = [0f32; 3];
            let mut total = 0f32;
            for &(k, w) in &taps {
                let at = ((k * target_w + x) as usize) * 3;
                acc[0] += mid[at] * w;
                acc[1] += mid[at + 1] * w;
                acc[2] += mid[at + 2] * w;
                total += w;
            }
            let t = total.max(f32::EPSILON);
            out.set_pixel(
                x,
                y,
                (
                    (acc[0] / t).round().clamp(0.0, 255.0) as u8,
                    (acc[1] / t).round().clamp(0.0, 255.0) as u8,
                    (acc[2] / t).round().clamp(0.0, 255.0) as u8,
                ),
            );
        }
    }
    out
}

/// The color at fractional pixel coordinates (`x`, `y` in *pixel units*,
/// where pixel `k` is centered at `k + 0.5`), interpolated bilinearly
/// between the four pixels around it and clamped at the edges. What a
/// picture that is about the size of its destination is sampled with, so
/// a fractional offset blends neighbors instead of snapping to one.
pub(crate) fn sample_bilinear(bitmap: &Bitmap, x: f32, y: f32) -> (u8, u8, u8) {
    if bitmap.width == 0 || bitmap.height == 0 {
        return (0, 0, 0);
    }
    let fx = (x - 0.5).clamp(0.0, (bitmap.width - 1) as f32);
    let fy = (y - 0.5).clamp(0.0, (bitmap.height - 1) as f32);
    let (x0, y0) = (fx.floor() as u32, fy.floor() as u32);
    let (x1, y1) = (
        (x0 + 1).min(bitmap.width - 1),
        (y0 + 1).min(bitmap.height - 1),
    );
    let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
    let at = |px: u32, py: u32| bitmap.pixel(px, py).unwrap_or((0, 0, 0));
    let (a, b, c, d) = (at(x0, y0), at(x1, y0), at(x0, y1), at(x1, y1));
    let mix = |p: u8, q: u8, r: u8, s: u8| {
        let top = f32::from(p) * (1.0 - tx) + f32::from(q) * tx;
        let bottom = f32::from(r) * (1.0 - tx) + f32::from(s) * tx;
        (top * (1.0 - ty) + bottom * ty).round().clamp(0.0, 255.0) as u8
    };
    (
        mix(a.0, b.0, c.0, d.0),
        mix(a.1, b.1, c.1, d.1),
        mix(a.2, b.2, c.2, d.2),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn downscaling_a_flat_color_stays_that_color() {
        let big = Bitmap::new(64, 64, (200, 100, 50));
        let small = downscale(&big, 8, 8);
        assert_eq!(small.width, 8);
        assert_eq!(small.height, 8);
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(small.pixel(x, y), Some((200, 100, 50)));
            }
        }
    }

    #[test]
    fn downscaling_picks_up_every_quadrant_of_a_split_image() {
        // Left half red, right half blue: a resize that always samples
        // column 0 would return all-red, which is exactly the kind of
        // silently-wrong index this test exists to catch.
        let mut src = Bitmap::new(4, 2, (0, 0, 0));
        for y in 0..2 {
            src.set_pixel(0, y, (255, 0, 0));
            src.set_pixel(1, y, (255, 0, 0));
            src.set_pixel(2, y, (0, 0, 255));
            src.set_pixel(3, y, (0, 0, 255));
        }
        let small = downscale(&src, 2, 1);
        assert_eq!(small.pixel(0, 0), Some((255, 0, 0)));
        assert_eq!(small.pixel(1, 0), Some((0, 0, 255)));
    }

    #[test]
    fn an_empty_source_downscales_to_a_black_placeholder_rather_than_panicking() {
        let empty = Bitmap::from_rgb(0, 0, &[]);
        let out = downscale(&empty, 4, 4);
        assert_eq!(out.width, 4);
        assert_eq!(out.height, 4);
    }

    /// The regression behind "heavily pixelated avatars": a fine
    /// checkerboard shrunk 8x. Nearest-neighbor returned pure black or
    /// pure white pixels (whichever source pixel it landed on); an area
    /// average returns the gray the eye sees.
    #[test]
    fn shrinking_a_fine_pattern_averages_it_instead_of_picking_pixels() {
        let mut src = Bitmap::new(64, 64, (0, 0, 0));
        for y in 0..64 {
            for x in 0..64 {
                if (x + y) % 2 == 0 {
                    src.set_pixel(x, y, (255, 255, 255));
                }
            }
        }
        let small = downscale(&src, 8, 8);
        for y in 0..8 {
            for x in 0..8 {
                let (r, g, b) = small.pixel(x, y).unwrap();
                assert!(
                    (126..=129).contains(&r) && r == g && g == b,
                    "({x},{y}) = {r},{g},{b}: a checkerboard must average to gray"
                );
            }
        }
    }

    #[test]
    fn a_non_integer_shrink_weights_the_edge_pixels_by_coverage() {
        // 3 source pixels -> 2 output pixels: each covers 1.5 source
        // pixels. Source 0,90,180 gives (0*1 + 90*.5)/1.5 = 30 and
        // (90*.5 + 180*1)/1.5 = 150.
        let mut src = Bitmap::new(3, 1, (0, 0, 0));
        src.set_pixel(1, 0, (90, 90, 90));
        src.set_pixel(2, 0, (180, 180, 180));
        let out = downscale(&src, 2, 1);
        assert_eq!(out.pixel(0, 0), Some((30, 30, 30)));
        assert_eq!(out.pixel(1, 0), Some((150, 150, 150)));
    }

    #[test]
    fn growing_a_picture_interpolates_smoothly_between_its_pixels() {
        let mut src = Bitmap::new(2, 1, (0, 0, 0));
        src.set_pixel(1, 0, (200, 200, 200));
        let big = downscale(&src, 8, 1);
        let ramp: Vec<u8> = (0..8).map(|x| big.pixel(x, 0).unwrap().0).collect();
        assert!(ramp.windows(2).all(|w| w[0] <= w[1]), "{ramp:?}");
        assert!(
            ramp[3] > 0 && ramp[3] < 200 && ramp[4] > ramp[3],
            "{ramp:?}"
        );
        assert_eq!(ramp[0], 0);
        assert_eq!(ramp[7], 200);
    }

    #[test]
    fn bilinear_sampling_blends_the_four_neighbors_and_clamps_at_the_edges() {
        let mut src = Bitmap::new(2, 2, (0, 0, 0));
        src.set_pixel(1, 0, (100, 0, 0));
        src.set_pixel(0, 1, (100, 0, 0));
        src.set_pixel(1, 1, (200, 0, 0));
        // The exact center of the 2x2 image is the mean of all four.
        assert_eq!(sample_bilinear(&src, 1.0, 1.0), (100, 0, 0));
        // A pixel's own center is that pixel.
        assert_eq!(sample_bilinear(&src, 1.5, 1.5), (200, 0, 0));
        // Outside the picture clamps to its edge pixel.
        assert_eq!(sample_bilinear(&src, -5.0, -5.0), (0, 0, 0));
        assert_eq!(sample_bilinear(&src, 9.0, 9.0), (200, 0, 0));
    }
}
