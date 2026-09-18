//! A small, dependency-free **sixel** encoder for the bitmaps this TUI
//! draws (avatars, a few hundred to a few thousand pixels).
//!
//! Sixel has three properties that shape everything below:
//!
//! - **No delete, no id.** A sixel is not an object the terminal keeps: it
//!   is pixels written into the cells under the cursor, and writing text
//!   over those cells replaces them (foot, Windows Terminal, Konsole). So
//!   the picture is handled exactly like a transparent iTerm2 one
//!   ([`super::Encoded::blends`]): blank reserved cells, text rewritten
//!   under it before every re-send, and a rewrite of the cells to hide it.
//! - **Exact pixel size.** There is no "fit to N cells" parameter; the
//!   picture covers what its pixels cover. [`encode`] therefore resamples
//!   to exactly `cols * cell.width` by `rows * cell.height`.
//! - **A palette, no alpha.** At most [`MAX_COLORS`] registers; a pixel is
//!   drawn or not. The header's `P2 = 1` leaves undrawn pixels
//!   transparent, so the cell background shows through them.
//!
//! The wire format: `ESC P 0;1;0 q`, raster attributes `"1;1;W;H`,
//! `#n;2;r;g;b` (percent) per register, then per band of six rows one line
//! per color used (`#n`, the data characters `?`..`~` with `!count`
//! run-length, `$` back to the line start), `-` to the next band, and
//! `ESC \` to finish.

use super::Bitmap;

/// Registers used when the image has more distinct colors than this.
/// Foot and xterm offer 256 or more, Windows Terminal 256; 128 keeps the
/// bytes down and is plenty for an avatar.
pub(super) const MAX_COLORS: usize = 128;

/// Pixels below this alpha are not drawn.
const ALPHA_CUTOFF: u8 = 128;

/// Sentinel palette index for an undrawn pixel.
const CLEAR: u8 = u8::MAX;

/// Encodes `bitmap` as a sixel sequence exactly `width` x `height` pixels
/// big. `None` for an empty target or bitmap.
pub(super) fn encode(bitmap: &Bitmap, width: u32, height: u32) -> Option<String> {
    if width == 0 || height == 0 || bitmap.width == 0 || bitmap.height == 0 {
        return None;
    }
    let pixels = resample(bitmap, width, height);
    let (palette, indices) = quantize(&pixels);
    Some(write(&palette, &indices, width as usize, height as usize))
}

/// Straight-alpha RGBA at the target size: nearest when enlarging, an area
/// average (alpha-weighted) when shrinking, which is what undoes the
/// supersampling the rasterisers apply.
fn resample(src: &Bitmap, width: u32, height: u32) -> Vec<[u8; 4]> {
    let (sw, sh) = (src.width as usize, src.height as usize);
    let (w, h) = (width as usize, height as usize);
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
        let y0 = y * sh / h;
        let y1 = ((y + 1) * sh / h).max(y0 + 1).min(sh);
        for x in 0..w {
            let x0 = x * sw / w;
            let x1 = ((x + 1) * sw / w).max(x0 + 1).min(sw);
            if x1 - x0 == 1 && y1 - y0 == 1 {
                let i = (y0 * sw + x0) * 4;
                out.push([
                    src.data[i],
                    src.data[i + 1],
                    src.data[i + 2],
                    src.data[i + 3],
                ]);
                continue;
            }
            let (mut r, mut g, mut b, mut a, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);
            for sy in y0..y1 {
                for sx in x0..x1 {
                    let i = (sy * sw + sx) * 4;
                    let pa = u32::from(src.data[i + 3]);
                    r += u32::from(src.data[i]) * pa;
                    g += u32::from(src.data[i + 1]) * pa;
                    b += u32::from(src.data[i + 2]) * pa;
                    a += pa;
                    n += 1;
                }
            }
            match (r.checked_div(a), g.checked_div(a), b.checked_div(a)) {
                (Some(r), Some(g), Some(b)) => out.push([r as u8, g as u8, b as u8, (a / n) as u8]),
                _ => out.push([0, 0, 0, 0]),
            }
        }
    }
    out
}

/// A palette and one index per pixel ([`CLEAR`] for an undrawn one).
fn quantize(pixels: &[[u8; 4]]) -> (Vec<[u8; 3]>, Vec<u8>) {
    // Exact colors first: flat art (a lane, an initials badge) stays exact.
    let mut exact: std::collections::HashMap<u32, u8> = std::collections::HashMap::new();
    let mut palette: Vec<[u8; 3]> = Vec::new();
    let mut indices = Vec::with_capacity(pixels.len());
    for p in pixels {
        if p[3] < ALPHA_CUTOFF {
            indices.push(CLEAR);
            continue;
        }
        let key = u32::from(p[0]) << 16 | u32::from(p[1]) << 8 | u32::from(p[2]);
        let index = match exact.get(&key) {
            Some(i) => *i,
            None => {
                if palette.len() >= MAX_COLORS {
                    return median_cut(pixels);
                }
                let i = palette.len() as u8;
                exact.insert(key, i);
                palette.push([p[0], p[1], p[2]]);
                i
            }
        };
        indices.push(index);
    }
    (palette, indices)
}

/// Median cut over a 5-bit-per-channel histogram: boxes of histogram bins
/// split at the median of their widest channel until [`MAX_COLORS`] remain.
/// Every bin ends up in exactly one box, so mapping a pixel is one lookup.
fn median_cut(pixels: &[[u8; 4]]) -> (Vec<[u8; 3]>, Vec<u8>) {
    let bin = |p: &[u8; 4]| -> usize {
        (usize::from(p[0] >> 3) << 10) | (usize::from(p[1] >> 3) << 5) | usize::from(p[2] >> 3)
    };
    // bin -> [count, summed r, g, b]
    let mut hist: std::collections::HashMap<usize, [u64; 4]> = std::collections::HashMap::new();
    for p in pixels.iter().filter(|p| p[3] >= ALPHA_CUTOFF) {
        let e = hist.entry(bin(p)).or_insert([0; 4]);
        e[0] += 1;
        e[1] += u64::from(p[0]);
        e[2] += u64::from(p[1]);
        e[3] += u64::from(p[2]);
    }
    let mut bins: Vec<(usize, [u64; 4])> = hist.into_iter().collect();
    bins.sort_unstable_by_key(|(b, _)| *b);
    let channel = |b: usize, c: usize| -> usize {
        match c {
            0 => b >> 10,
            1 => (b >> 5) & 31,
            _ => b & 31,
        }
    };
    let mut boxes: Vec<Vec<(usize, [u64; 4])>> = vec![bins];
    while boxes.len() < MAX_COLORS {
        // Split the box with the most pixels that still has two bins.
        let Some(at) = (0..boxes.len())
            .filter(|i| boxes[*i].len() > 1)
            .max_by_key(|i| boxes[*i].iter().map(|(_, h)| h[0]).sum::<u64>())
        else {
            break;
        };
        let mut b = boxes.swap_remove(at);
        let widest = (0..3)
            .max_by_key(|c| {
                let lo = b.iter().map(|(k, _)| channel(*k, *c)).min().unwrap_or(0);
                let hi = b.iter().map(|(k, _)| channel(*k, *c)).max().unwrap_or(0);
                hi - lo
            })
            .unwrap_or(0);
        b.sort_unstable_by_key(|(k, _)| channel(*k, widest));
        let total: u64 = b.iter().map(|(_, h)| h[0]).sum();
        let (mut acc, mut cut) = (0, 1);
        for (i, (_, h)) in b.iter().enumerate() {
            acc += h[0];
            if acc * 2 >= total {
                cut = (i + 1).clamp(1, b.len() - 1);
                break;
            }
        }
        let tail = b.split_off(cut);
        boxes.push(b);
        boxes.push(tail);
    }
    let mut lut = vec![0u8; 1 << 15];
    let mut palette = Vec::with_capacity(boxes.len());
    for (i, b) in boxes.iter().enumerate() {
        let n: u64 = b.iter().map(|(_, h)| h[0]).sum::<u64>().max(1);
        let mean = |c: usize| (b.iter().map(|(_, h)| h[c]).sum::<u64>() / n) as u8;
        palette.push([mean(1), mean(2), mean(3)]);
        for (k, _) in b {
            lut[*k] = i as u8;
        }
    }
    let indices = pixels
        .iter()
        .map(|p| {
            if p[3] < ALPHA_CUTOFF {
                CLEAR
            } else {
                lut[bin(p)]
            }
        })
        .collect();
    (palette, indices)
}

fn write(palette: &[[u8; 3]], indices: &[u8], width: usize, height: usize) -> String {
    let mut out = String::with_capacity(64 + palette.len() * 16 + indices.len() / 2);
    out.push_str("\x1bP0;1;0q\"1;1;");
    out.push_str(&format!("{width};{height}"));
    for (i, c) in palette.iter().enumerate() {
        let pct = |v: u8| (u32::from(v) * 100 + 127) / 255;
        out.push_str(&format!("#{i};2;{};{};{}", pct(c[0]), pct(c[1]), pct(c[2])));
    }
    let bands = height.div_ceil(6);
    let mut masks: Vec<Vec<u8>> = vec![Vec::new(); palette.len()];
    for band in 0..bands {
        let mut used: Vec<usize> = Vec::new();
        for x in 0..width {
            for bit in 0..6 {
                let y = band * 6 + bit;
                if y >= height {
                    break;
                }
                let index = indices[y * width + x];
                if index == CLEAR {
                    continue;
                }
                let m = &mut masks[usize::from(index)];
                if m.is_empty() {
                    m.resize(width, 0);
                    used.push(usize::from(index));
                }
                m[x] |= 1 << bit;
            }
        }
        for (n, index) in used.iter().enumerate() {
            if n > 0 {
                out.push('$');
            }
            out.push_str(&format!("#{index}"));
            let m = std::mem::take(&mut masks[*index]);
            // Trailing blanks need not be written.
            let end = m.iter().rposition(|b| *b != 0).map_or(0, |p| p + 1);
            run_length(&mut out, &m[..end]);
        }
        if band + 1 < bands {
            out.push('-');
        }
    }
    out.push_str("\x1b\\");
    out
}

/// `!n c` for runs of four or more, the characters themselves otherwise.
fn run_length(out: &mut String, data: &[u8]) {
    let mut i = 0;
    while i < data.len() {
        let c = data[i];
        let mut run = 1;
        while i + run < data.len() && data[i + run] == c {
            run += 1;
        }
        let ch = char::from(63 + c);
        if run >= 4 {
            out.push_str(&format!("!{run}{ch}"));
        } else {
            for _ in 0..run {
                out.push(ch);
            }
        }
        i += run;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, rgba: [u8; 4]) -> Bitmap {
        let mut b = Bitmap::transparent(w, h);
        for px in b.data.chunks_exact_mut(4) {
            px.copy_from_slice(&rgba);
        }
        b
    }

    #[test]
    fn header_raster_palette_and_terminator() {
        let s = encode(&solid(2, 6, [255, 0, 0, 255]), 2, 6).unwrap();
        assert!(s.starts_with("\x1bP0;1;0q\"1;1;2;6#0;2;100;0;0"), "{s:?}");
        assert!(s.ends_with("\x1b\\"));
    }

    #[test]
    fn a_full_band_is_all_six_bits_set_and_runs_are_compressed() {
        // 6 rows: every bit set = 63 + 63 = '~'. Width 10 -> a run.
        let s = encode(&solid(10, 6, [0, 255, 0, 255]), 10, 6).unwrap();
        assert!(s.contains("#0!10~"), "{s:?}");
        // A short run is written out.
        let s = encode(&solid(3, 6, [0, 255, 0, 255]), 3, 6).unwrap();
        assert!(s.contains("#0~~~"), "{s:?}");
    }

    #[test]
    fn bands_are_six_rows_and_separated_by_a_dash() {
        let s = encode(&solid(2, 13, [0, 0, 255, 255]), 2, 13).unwrap();
        assert_eq!(s.matches('-').count(), 2, "13 rows = 3 bands: {s:?}");
        // The last band has one row: bit 0 only = '@'.
        assert!(s.contains("#0@@\x1b\\"), "{s:?}");
    }

    #[test]
    fn colors_of_one_band_share_it_with_a_carriage_return() {
        let mut b = solid(2, 6, [255, 0, 0, 255]);
        b.data[4..8].copy_from_slice(&[0, 0, 255, 255]);
        let s = encode(&b, 2, 6).unwrap();
        assert!(s.contains("#0;2;100;0;0#1;2;0;0;100"), "{s:?}");
        // Red: column 0 whole, column 1 minus its top row; blue: that top pixel.
        assert!(s.contains("#0~}$#1?@"), "{s:?}");
    }

    #[test]
    fn transparent_pixels_are_not_drawn_and_trailing_blanks_are_trimmed() {
        let mut b = solid(4, 6, [255, 255, 255, 255]);
        for y in 0..6 {
            for x in 1..4 {
                b.data[(y * 4 + x) * 4 + 3] = 0;
            }
        }
        let s = encode(&b, 4, 6).unwrap();
        assert!(s.contains("#0~\x1b\\"), "{s:?}");
        let none = encode(&solid(4, 6, [0, 0, 0, 0]), 4, 6).unwrap();
        assert!(!none.contains("#0~"), "{none:?}");
    }

    #[test]
    fn resampling_hits_the_exact_pixel_size() {
        let s = encode(&solid(40, 80, [9, 9, 9, 255]), 10, 21).unwrap();
        assert!(s.contains("\"1;1;10;21"), "{s:?}");
        assert_eq!(s.matches('-').count(), 3, "21 rows = 4 bands");
    }

    #[test]
    fn many_colors_are_cut_down_to_the_palette_limit() {
        let mut b = Bitmap::transparent(64, 64);
        for y in 0..64u32 {
            for x in 0..64u32 {
                b.set_pixel(x, y, ((x * 4) as u8, (y * 4) as u8, ((x ^ y) * 4) as u8));
            }
        }
        let s = encode(&b, 64, 64).unwrap();
        let registers = s.matches(";2;").count();
        assert!(registers <= MAX_COLORS && registers > 16, "{registers}");
    }

    #[test]
    fn an_empty_target_encodes_to_nothing() {
        assert!(encode(&solid(2, 2, [1, 2, 3, 255]), 0, 4).is_none());
    }
}
