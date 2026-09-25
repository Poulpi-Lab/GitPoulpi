//! BMP and the DIB payload of ICO/CUR, hand-rolled.
//!
//! Supported: BITMAPCOREHEADER, BITMAPINFOHEADER and its V4/V5 growth;
//! 1/4/8-bit palettes, 16/24/32-bit direct color; `BI_RGB` and
//! `BI_BITFIELDS`/`BI_ALPHABITFIELDS`; bottom-up and top-down rows.
//! RLE4/RLE8 are reported as [`DecodeError::Unsupported`].

use super::rgba::{check_pixels, DecodeError};

fn u16le(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?))
}
fn u32le(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}
fn i32le(b: &[u8], at: usize) -> Option<i32> {
    Some(i32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// A whole `.bmp` file.
pub(super) fn decode_file(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), DecodeError> {
    if bytes.get(..2) != Some(b"BM") {
        return Err(DecodeError::Corrupt);
    }
    let off = u32le(bytes, 10).ok_or(DecodeError::Corrupt)? as usize;
    decode_dib(&bytes[14..], Some(off.saturating_sub(14)), false)
}

/// A DIB (header onwards). `pixel_off` is the offset of the pixel array
/// from the DIB start when the container says so (a BMP file), else it
/// follows the header, masks and palette (ICO). `ico` halves the height
/// and applies the 1-bit AND mask that follows the color data.
pub(super) fn decode_dib(
    dib: &[u8],
    pixel_off: Option<usize>,
    ico: bool,
) -> Result<(u32, u32, Vec<u8>), DecodeError> {
    let corrupt = DecodeError::Corrupt;
    let hsize = u32le(dib, 0).ok_or(corrupt)? as usize;
    let (w, raw_h, bpp, comp, mut ncolors, pal_entry) = if hsize == 12 {
        (
            i32::from(u16le(dib, 4).ok_or(corrupt)?),
            i32::from(u16le(dib, 6).ok_or(corrupt)?),
            u16le(dib, 10).ok_or(corrupt)?,
            0u32,
            0usize,
            3usize,
        )
    } else if hsize >= 40 {
        (
            i32le(dib, 4).ok_or(corrupt)?,
            i32le(dib, 8).ok_or(corrupt)?,
            u16le(dib, 14).ok_or(corrupt)?,
            u32le(dib, 16).ok_or(corrupt)?,
            u32le(dib, 32).ok_or(corrupt)? as usize,
            4usize,
        )
    } else {
        return Err(corrupt);
    };
    if w <= 0 || raw_h == 0 {
        return Err(corrupt);
    }
    let top_down = raw_h < 0;
    let mut h = raw_h.unsigned_abs();
    if ico {
        h /= 2;
    }
    let w = w as u32;
    check_pixels(w, h)?;
    if comp == 1 || comp == 2 {
        return Err(DecodeError::Unsupported("RLE-compressed BMP")); // i18n-english
    }
    if !matches!(comp, 0 | 3 | 6) {
        return Err(DecodeError::Unsupported("BMP compression")); // i18n-english
    }
    if !matches!(bpp, 1 | 4 | 8 | 16 | 24 | 32) {
        return Err(corrupt);
    }
    // Channel masks: after a 40-byte header they sit right behind it;
    // V4/V5 carry them inside the header.
    let mut at = hsize;
    let mut masks: Option<[u32; 4]> = None;
    if comp == 3 || comp == 6 {
        let n = if comp == 6 { 4 } else { 3 };
        let base = if hsize == 40 || hsize == 52 {
            40.min(hsize)
        } else {
            40
        };
        let mut m = [0u32; 4];
        for (i, slot) in m.iter_mut().enumerate().take(n) {
            *slot = u32le(dib, base + i * 4).ok_or(corrupt)?;
        }
        if hsize == 40 {
            at += n * 4;
        }
        if hsize >= 108 && comp == 3 {
            m[3] = u32le(dib, 52).unwrap_or(0);
        }
        masks = Some(m);
    } else if hsize >= 108 && bpp == 32 {
        let m = [
            u32le(dib, 40).unwrap_or(0),
            u32le(dib, 44).unwrap_or(0),
            u32le(dib, 48).unwrap_or(0),
            u32le(dib, 52).unwrap_or(0),
        ];
        if m[0] != 0 && m[1] != 0 && m[2] != 0 {
            masks = Some(m);
        }
    }
    let mut palette: Vec<[u8; 3]> = Vec::new();
    if bpp <= 8 {
        if ncolors == 0 || ncolors > (1 << bpp) {
            ncolors = 1 << bpp;
        }
        let pal_at = at.max(hsize);
        for i in 0..ncolors {
            let e = dib
                .get(pal_at + i * pal_entry..pal_at + i * pal_entry + 3)
                .ok_or(corrupt)?;
            palette.push([e[2], e[1], e[0]]);
        }
        at = pal_at + ncolors * pal_entry;
    }
    let start = pixel_off.unwrap_or(at);
    let stride = ((w as usize * bpp as usize).div_ceil(32)) * 4;
    let need = stride.checked_mul(h as usize).ok_or(corrupt)?;
    let pixels = dib
        .get(start..start.checked_add(need).ok_or(corrupt)?)
        .ok_or(corrupt)?;
    let mut out = vec![0u8; w as usize * h as usize * 4];
    let mut any_alpha = false;
    let field = |v: u32, mask: u32| -> u8 {
        if mask == 0 {
            return 255;
        }
        let shift = mask.trailing_zeros();
        let max = mask >> shift;
        (((v & mask) >> shift) * 255 / max.max(1)) as u8
    };
    for row in 0..h as usize {
        let src_row = if top_down { row } else { h as usize - 1 - row };
        let line = &pixels[src_row * stride..src_row * stride + stride];
        for x in 0..w as usize {
            let px = match bpp {
                1 | 4 | 8 => {
                    let per = 8 / bpp as usize;
                    let byte = line[x / per];
                    let shift = 8 - bpp as usize * (x % per + 1);
                    let idx = (byte >> shift) as usize & ((1usize << bpp) - 1);
                    let c = palette.get(idx).copied().unwrap_or([0, 0, 0]);
                    [c[0], c[1], c[2], 255]
                }
                16 => {
                    let v = u32::from(u16::from_le_bytes([line[x * 2], line[x * 2 + 1]]));
                    match masks {
                        Some(m) => [
                            field(v, m[0]),
                            field(v, m[1]),
                            field(v, m[2]),
                            field(v, m[3]),
                        ],
                        None => [field(v, 0x7c00), field(v, 0x03e0), field(v, 0x001f), 255],
                    }
                }
                24 => [line[x * 3 + 2], line[x * 3 + 1], line[x * 3], 255],
                _ => {
                    let v = u32::from_le_bytes([
                        line[x * 4],
                        line[x * 4 + 1],
                        line[x * 4 + 2],
                        line[x * 4 + 3],
                    ]);
                    match masks {
                        Some(m) => [
                            field(v, m[0]),
                            field(v, m[1]),
                            field(v, m[2]),
                            field(v, m[3]),
                        ],
                        None => [
                            line[x * 4 + 2],
                            line[x * 4 + 1],
                            line[x * 4],
                            line[x * 4 + 3],
                        ],
                    }
                }
            };
            if px[3] != 0 {
                any_alpha = true;
            }
            let o = (row * w as usize + x) * 4;
            out[o..o + 4].copy_from_slice(&px);
        }
    }
    // A 32-bit `BI_RGB` file whose alpha bytes are all zero is opaque
    // (the fourth byte is padding there), not fully transparent.
    let alpha_is_real = match (bpp, masks) {
        (32, Some(m)) => m[3] != 0 && any_alpha,
        (32, None) => any_alpha,
        (16, Some(m)) => m[3] != 0 && any_alpha,
        _ => false,
    };
    if !alpha_is_real {
        for px in out.chunks_exact_mut(4) {
            px[3] = 255;
        }
    }
    if ico && !alpha_is_real {
        let mask_stride = (w as usize).div_ceil(32) * 4;
        let mask_at = start + need;
        if let Some(mask) = dib.get(mask_at..mask_at + mask_stride * h as usize) {
            for row in 0..h as usize {
                let src_row = if top_down { row } else { h as usize - 1 - row };
                let line = &mask[src_row * mask_stride..];
                for x in 0..w as usize {
                    if line[x / 8] & (0x80 >> (x % 8)) != 0 {
                        out[(row * w as usize + x) * 4 + 3] = 0;
                    }
                }
            }
        }
    }
    Ok((w, h, out))
}
