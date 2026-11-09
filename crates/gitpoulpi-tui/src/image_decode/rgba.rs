//! The full decoder: any supported format to a straight-alpha RGBA
//! [`Bitmap`]. See the module doc of `image_decode`.

use std::io::Cursor;
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::terminal_image::Bitmap;

use super::bmp;

/// The pixel cap (`width * height`), checked from the header before any
/// pixel buffer is allocated: 64 MP, about 256 MiB of RGBA.
pub(crate) const MAX_PIXELS: u64 = 64_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    WebP,
    Bmp,
    Ico,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DecodeError {
    /// Not an image this module knows.
    Unrecognized,
    /// Over [`MAX_PIXELS`]; the dimensions are from the header.
    TooLarge { width: u32, height: u32 },
    /// Recognised, but the data is damaged or truncated.
    Corrupt,
    /// Recognised, but a variant this decoder does not handle.
    Unsupported(&'static str),
    /// A decoder panicked on hostile input; caught.
    Panicked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Decoded {
    pub format: ImageFormat,
    pub bitmap: Bitmap,
    /// Whether any pixel is not fully opaque.
    pub has_alpha: bool,
}

/// The format from the magic bytes, never from a name or a content type.
pub(crate) fn sniff_format(b: &[u8]) -> Option<ImageFormat> {
    if b.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        Some(ImageFormat::Png)
    } else if b.starts_with(&[0xff, 0xd8, 0xff]) {
        Some(ImageFormat::Jpeg)
    } else if b.starts_with(b"GIF87a") || b.starts_with(b"GIF89a") {
        Some(ImageFormat::Gif)
    } else if b.len() >= 12 && &b[..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        Some(ImageFormat::WebP)
    } else if b.starts_with(b"BM") {
        Some(ImageFormat::Bmp)
    } else if b.len() >= 6
        && b[0] == 0
        && b[1] == 0
        && (b[2] == 1 || b[2] == 2)
        && b[3] == 0
        && (b[4] != 0 || b[5] != 0)
    {
        Some(ImageFormat::Ico)
    } else {
        None
    }
}

pub(super) fn check_pixels(width: u32, height: u32) -> Result<(), DecodeError> {
    if width == 0 || height == 0 {
        return Err(DecodeError::Corrupt);
    }
    if u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(DecodeError::TooLarge { width, height });
    }
    Ok(())
}

/// [`decode_rgba`] inside `catch_unwind`: a decoder that panics on a
/// hostile file yields [`DecodeError::Panicked`] instead of taking the
/// worker (or the UI) down.
pub(crate) fn decode_guarded(bytes: &[u8]) -> Result<Decoded, DecodeError> {
    catch_unwind(AssertUnwindSafe(|| decode_rgba(bytes))).unwrap_or(Err(DecodeError::Panicked))
}

pub(crate) fn decode_rgba(bytes: &[u8]) -> Result<Decoded, DecodeError> {
    let format = sniff_format(bytes).ok_or(DecodeError::Unrecognized)?;
    let (w, h, data) = match format {
        ImageFormat::Png => decode_png(bytes)?,
        ImageFormat::Jpeg => decode_jpeg(bytes)?,
        ImageFormat::Gif => decode_gif(bytes)?,
        ImageFormat::WebP => decode_webp(bytes)?,
        ImageFormat::Bmp => bmp::decode_file(bytes)?,
        ImageFormat::Ico => decode_ico(bytes)?,
    };
    let has_alpha = data.as_chunks::<4>().0.iter().any(|p| p[3] != 255);
    Ok(Decoded {
        format,
        bitmap: Bitmap {
            width: w,
            height: h,
            data,
        },
        has_alpha,
    })
}

fn decode_png(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), DecodeError> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    decoder.set_limits(png::Limits {
        bytes: (MAX_PIXELS as usize) * 8,
    });
    let mut reader = decoder.read_info().map_err(|_| DecodeError::Corrupt)?;
    let (w, h) = {
        let i = reader.info();
        (i.width, i.height)
    };
    check_pixels(w, h)?;
    let mut buf = vec![0; reader.output_buffer_size().ok_or(DecodeError::Corrupt)?];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|_| DecodeError::Corrupt)?;
    let src = &buf[..info.buffer_size()];
    let n = w as usize * h as usize;
    let mut out = Vec::with_capacity(n * 4);
    match info.color_type {
        png::ColorType::Rgba if src.len() >= n * 4 => out.extend_from_slice(&src[..n * 4]),
        png::ColorType::Rgb if src.len() >= n * 3 => {
            for p in src.as_chunks::<3>().0.iter().take(n) {
                out.extend_from_slice(&[p[0], p[1], p[2], 255]);
            }
        }
        png::ColorType::Grayscale if src.len() >= n => {
            for &g in src.iter().take(n) {
                out.extend_from_slice(&[g, g, g, 255]);
            }
        }
        png::ColorType::GrayscaleAlpha if src.len() >= n * 2 => {
            for p in src.as_chunks::<2>().0.iter().take(n) {
                out.extend_from_slice(&[p[0], p[0], p[0], p[1]]);
            }
        }
        _ => return Err(DecodeError::Corrupt),
    }
    Ok((w, h, out))
}

fn decode_jpeg(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), DecodeError> {
    let mut d = jpeg_decoder::Decoder::new(bytes);
    d.read_info().map_err(|_| DecodeError::Corrupt)?;
    let info = d.info().ok_or(DecodeError::Corrupt)?;
    let (w, h) = (u32::from(info.width), u32::from(info.height));
    check_pixels(w, h)?;
    let px = d.decode().map_err(|_| DecodeError::Corrupt)?;
    let mut out = Vec::with_capacity(w as usize * h as usize * 4);
    match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => {
            for p in px.as_chunks::<3>().0 {
                out.extend_from_slice(&[p[0], p[1], p[2], 255]);
            }
        }
        jpeg_decoder::PixelFormat::L8 => {
            for &g in &px {
                out.extend_from_slice(&[g, g, g, 255]);
            }
        }
        jpeg_decoder::PixelFormat::CMYK32 => {
            // Adobe CMYK JPEGs store inverted values: r = c * k / 255.
            for p in px.as_chunks::<4>().0 {
                let k = u32::from(p[3]);
                let f = |c: u8| (u32::from(c) * k / 255) as u8;
                out.extend_from_slice(&[f(p[0]), f(p[1]), f(p[2]), 255]);
            }
        }
        jpeg_decoder::PixelFormat::L16 => return Err(DecodeError::Unsupported("16-bit JPEG")),
    }
    if out.len() != w as usize * h as usize * 4 {
        return Err(DecodeError::Corrupt);
    }
    Ok((w, h, out))
}

/// The first frame, placed on a transparent logical-screen canvas.
fn decode_gif(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), DecodeError> {
    // The logical screen is in the fixed header: check it before the
    // decoder gets a chance to allocate for it.
    if bytes.len() >= 10 {
        let w = u32::from(u16::from_le_bytes([bytes[6], bytes[7]]));
        let h = u32::from(u16::from_le_bytes([bytes[8], bytes[9]]));
        check_pixels(w, h)?;
    }
    let mut opts = gif::DecodeOptions::new();
    opts.set_color_output(gif::ColorOutput::RGBA);
    opts.set_memory_limit(gif::MemoryLimit::Bytes(
        std::num::NonZeroU64::new(MAX_PIXELS * 4).unwrap_or(std::num::NonZeroU64::MIN),
    ));
    let mut d = opts
        .read_info(Cursor::new(bytes))
        .map_err(|_| DecodeError::Corrupt)?;
    let (w, h) = (u32::from(d.width()), u32::from(d.height()));
    check_pixels(w, h)?;
    let frame = d
        .read_next_frame()
        .map_err(|_| DecodeError::Corrupt)?
        .ok_or(DecodeError::Corrupt)?;
    let mut out = vec![0u8; w as usize * h as usize * 4];
    let (fl, ft) = (usize::from(frame.left), usize::from(frame.top));
    let (fw, fh) = (usize::from(frame.width), usize::from(frame.height));
    if frame.buffer.len() < fw * fh * 4 {
        return Err(DecodeError::Corrupt);
    }
    if fl >= w as usize || ft >= h as usize {
        return Ok((w, h, out));
    }
    for y in 0..fh.min(h as usize - ft) {
        let n = fw.min((w as usize).saturating_sub(fl));
        let dst = ((ft + y) * w as usize + fl) * 4;
        out[dst..dst + n * 4].copy_from_slice(&frame.buffer[y * fw * 4..y * fw * 4 + n * 4]);
    }
    Ok((w, h, out))
}

fn decode_webp(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), DecodeError> {
    let mut d =
        image_webp::WebPDecoder::new(Cursor::new(bytes)).map_err(|_| DecodeError::Corrupt)?;
    let (w, h) = d.dimensions();
    check_pixels(w, h)?;
    let alpha = d.has_alpha();
    let size = d.output_buffer_size().ok_or(DecodeError::Corrupt)?;
    let mut buf = vec![0u8; size];
    d.read_image(&mut buf).map_err(|_| DecodeError::Corrupt)?;
    let n = w as usize * h as usize;
    if alpha {
        if buf.len() < n * 4 {
            return Err(DecodeError::Corrupt);
        }
        buf.truncate(n * 4);
        return Ok((w, h, buf));
    }
    if buf.len() < n * 3 {
        return Err(DecodeError::Corrupt);
    }
    let mut out = Vec::with_capacity(n * 4);
    for p in buf.as_chunks::<3>().0.iter().take(n) {
        out.extend_from_slice(&[p[0], p[1], p[2], 255]);
    }
    Ok((w, h, out))
}

/// The largest entry (by area, then bit depth): a PNG or a DIB.
fn decode_ico(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), DecodeError> {
    let count = usize::from(u16::from_le_bytes([bytes[4], bytes[5]]));
    let mut best: Option<(u32, u16, usize, usize)> = None;
    for i in 0..count {
        let e = bytes
            .get(6 + i * 16..6 + i * 16 + 16)
            .ok_or(DecodeError::Corrupt)?;
        let w = if e[0] == 0 { 256 } else { u32::from(e[0]) };
        let h = if e[1] == 0 { 256 } else { u32::from(e[1]) };
        let bpp = u16::from_le_bytes([e[6], e[7]]);
        let size = u32::from_le_bytes([e[8], e[9], e[10], e[11]]) as usize;
        let off = u32::from_le_bytes([e[12], e[13], e[14], e[15]]) as usize;
        let area = w * h;
        if best.is_none_or(|(a, b, _, _)| (area, bpp) > (a, b)) {
            best = Some((area, bpp, off, size));
        }
    }
    let (_, _, off, size) = best.ok_or(DecodeError::Corrupt)?;
    let data = bytes
        .get(off..off.saturating_add(size).min(bytes.len()))
        .ok_or(DecodeError::Corrupt)?;
    if data.starts_with(&[0x89, b'P', b'N', b'G']) {
        decode_png(data)
    } else {
        bmp::decode_dib(data, None, true)
    }
}
