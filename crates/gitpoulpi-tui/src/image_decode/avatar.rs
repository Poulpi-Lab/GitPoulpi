//! Turning the bytes Gravatar answers with into a [`Bitmap`] sized for the
//! avatar's cell box.
//!
//! Two formats, because Gravatar serves both depending on what the user
//! uploaded: PNG (via the `png` crate, already a dependency for the
//! commit graph's *encoder* -- decoding is the same crate's other half)
//! and JPEG (via `jpeg-decoder`, `default-features = false` so the
//! `rayon` parallel decoder is not pulled in for one small image).
//! Detection is by magic bytes, not by trusting a `Content-Type` header a
//! misconfigured host could get wrong.
//!
//! Everything here is a pure function of bytes already in memory --
//! nothing in this file performs I/O -- so it is testable with small,
//! synthetic images built in the tests below, with no network and no
//! filesystem.

use crate::terminal_image::Bitmap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageFormat {
    Png,
    Jpeg,
}

fn sniff(bytes: &[u8]) -> Option<ImageFormat> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        Some(ImageFormat::Png)
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some(ImageFormat::Jpeg)
    } else {
        None
    }
}

/// Decodes `bytes` into an RGB [`Bitmap`], `None` for anything that is
/// not a recognisable PNG or JPEG (including a host's HTML error page,
/// which is what a misconfigured or down Gravatar can answer with even
/// on a 200).
pub(crate) fn decode(bytes: &[u8]) -> Option<Bitmap> {
    match sniff(bytes)? {
        ImageFormat::Png => decode_png(bytes),
        ImageFormat::Jpeg => decode_jpeg(bytes),
    }
}

fn decode_png(bytes: &[u8]) -> Option<Bitmap> {
    // `png::Decoder` wants `BufRead + Seek`; a plain `&[u8]` is neither,
    // `std::io::Cursor` makes it both without a copy.
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let bytes = &buf[..info.buffer_size()];
    let rgb = to_rgb(
        bytes,
        info.color_type,
        info.bit_depth,
        info.width,
        info.height,
    )?;
    Some(Bitmap::from_rgb(info.width, info.height, &rgb))
}

/// Normalises whatever PNG color type/depth the encoder used into plain
/// 8-bit RGB, dropping alpha by compositing over white - an avatar with
/// transparency composited over the Log's own background would need to
/// know that background at decode time, and Gravatar avatars are
/// practically always opaque photographs; white is the least surprising
/// choice for the rare exception.
fn to_rgb(
    bytes: &[u8],
    color_type: png::ColorType,
    bit_depth: png::BitDepth,
    width: u32,
    height: u32,
) -> Option<Vec<u8>> {
    if bit_depth != png::BitDepth::Eight {
        // 16-bit and palette/1-4-bit depths are rare for a Gravatar photo
        // and not worth the extra conversion code; treated as
        // undecodable so the caller falls back to initials rather than
        // guessing at a conversion this module cannot test in isolation.
        return None;
    }
    let pixels = (width as usize) * (height as usize);
    let mut out = Vec::with_capacity(pixels * 3);
    match color_type {
        png::ColorType::Rgb => {
            if bytes.len() < pixels * 3 {
                return None;
            }
            out.extend_from_slice(&bytes[..pixels * 3]);
        }
        png::ColorType::Rgba => {
            if bytes.len() < pixels * 4 {
                return None;
            }
            for px in bytes.chunks_exact(4).take(pixels) {
                let a = f32::from(px[3]) / 255.0;
                for c in &px[..3] {
                    let v = f32::from(*c) * a + 255.0 * (1.0 - a);
                    out.push(v.round() as u8);
                }
            }
        }
        png::ColorType::Grayscale => {
            if bytes.len() < pixels {
                return None;
            }
            for &g in bytes.iter().take(pixels) {
                out.extend_from_slice(&[g, g, g]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            if bytes.len() < pixels * 2 {
                return None;
            }
            for px in bytes.chunks_exact(2).take(pixels) {
                let a = f32::from(px[1]) / 255.0;
                let v = (f32::from(px[0]) * a + 255.0 * (1.0 - a)).round() as u8;
                out.extend_from_slice(&[v, v, v]);
            }
        }
        png::ColorType::Indexed => return None,
    }
    Some(out)
}

fn decode_jpeg(bytes: &[u8]) -> Option<Bitmap> {
    let mut decoder = jpeg_decoder::Decoder::new(bytes);
    let pixels = decoder.decode().ok()?;
    let info = decoder.info()?;
    let (width, height) = (u32::from(info.width), u32::from(info.height));
    let rgb = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => pixels,
        jpeg_decoder::PixelFormat::L8 => pixels.into_iter().flat_map(|g| [g, g, g]).collect(),
        // CMYK avatars are not a case worth writing a converter for; the
        // caller falls back to initials.
        jpeg_decoder::PixelFormat::CMYK32 => return None,
        jpeg_decoder::PixelFormat::L16 => return None,
    };
    Some(Bitmap::from_rgb(width, height, &rgb))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_png(bitmap: &Bitmap) -> Vec<u8> {
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, bitmap.width, bitmap.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&bitmap.data).unwrap();
        drop(writer);
        out
    }

    #[test]
    fn an_unknown_byte_stream_is_not_a_recognized_image() {
        assert_eq!(decode(b"not an image"), None);
        assert_eq!(decode(b"<html>404</html>"), None);
        assert_eq!(decode(&[]), None);
    }

    #[test]
    fn a_round_tripped_png_decodes_back_to_the_same_pixels() {
        let original = Bitmap::new(4, 3, (10, 20, 30));
        let bytes = encode_png(&original);
        let decoded = decode(&bytes).expect("a PNG this module wrote must decode");
        assert_eq!(decoded.width, 4);
        assert_eq!(decoded.height, 3);
        assert_eq!(decoded.pixel(0, 0), Some((10, 20, 30)));
        assert_eq!(decoded.pixel(3, 2), Some((10, 20, 30)));
    }

    /// Falsified: changed the RGBA compositing formula to ignore alpha
    /// (always keep the color channel verbatim), watched a half-transparent
    /// pixel come back as the wrong color, restored the real formula.
    #[test]
    fn a_png_with_alpha_is_composited_over_white() {
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, 1, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        // Fully transparent red must come back as white, not red.
        writer.write_image_data(&[255, 0, 0, 0]).unwrap();
        drop(writer);
        let decoded = decode(&out).unwrap();
        assert_eq!(decoded.pixel(0, 0), Some((255, 255, 255)));
    }
}
