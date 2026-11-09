//! Image headers, read without decoding anything: format, pixel size and
//! (when the header says) frame count and alpha. Pure, dependency-free and
//! shared by the backends (`Blob::image`), the CLI/MCP and the TUI.
//!
//! [`sniff`] never panics: truncated or garbage input is `None`.

use serde::{Deserialize, Serialize};

/// The container format recognized from the magic bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
    Webp,
    /// ICO and CUR.
    Ico,
    Tiff,
    Avif,
    Heic,
}

/// What the header of an image file says.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageInfo {
    pub format: ImageFormat,
    /// Pixels; `0` when the format is recognized but its size is not read
    /// (AVIF, HEIC).
    pub width: u32,
    pub height: u32,
    /// Frame count when it is known from the header alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frames: Option<u32>,
    /// Whether the image has an alpha channel, when the header says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub has_alpha: Option<bool>,
}

fn info(format: ImageFormat, width: u32, height: u32) -> ImageInfo {
    ImageInfo {
        format,
        width,
        height,
        frames: None,
        has_alpha: None,
    }
}

fn be16(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from(u16::from_be_bytes(
        b.get(at..at + 2)?.try_into().ok()?,
    )))
}
fn le16(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from(u16::from_le_bytes(
        b.get(at..at + 2)?.try_into().ok()?,
    )))
}
fn be32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(b.get(at..at + 4)?.try_into().ok()?))
}
fn le32(b: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?))
}

/// The image `bytes` start, from its header alone. `None` when it is not a
/// recognized image or the header is truncated.
pub fn sniff(bytes: &[u8]) -> Option<ImageInfo> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return png(bytes);
    }
    if bytes.starts_with(&[0xFF, 0xD8]) {
        return jpeg(bytes);
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        let (w, h) = (le16(bytes, 6)?, le16(bytes, 8)?);
        return Some(info(ImageFormat::Gif, w, h));
    }
    if bytes.starts_with(b"BM") {
        return bmp(bytes);
    }
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        return webp(bytes);
    }
    if bytes.starts_with(&[0, 0, 1, 0]) || bytes.starts_with(&[0, 0, 2, 0]) {
        return ico(bytes);
    }
    if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        return tiff(bytes);
    }
    if bytes.get(4..8) == Some(b"ftyp") {
        let format = match bytes.get(8..12)? {
            b"avif" | b"avis" => ImageFormat::Avif,
            b"heic" | b"heix" | b"hevc" | b"heim" | b"heis" | b"mif1" | b"msf1" => {
                ImageFormat::Heic
            }
            _ => return None,
        };
        return Some(info(format, 0, 0));
    }
    None
}

fn png(b: &[u8]) -> Option<ImageInfo> {
    if b.get(12..16)? != b"IHDR" {
        return None;
    }
    let (w, h) = (be32(b, 16)?, be32(b, 20)?);
    let color = *b.get(25)?;
    let mut i = info(ImageFormat::Png, w, h);
    // Palette images may carry a tRNS chunk; the header cannot tell.
    i.has_alpha = match color {
        4 | 6 => Some(true),
        0 | 2 => Some(false),
        _ => None,
    };
    Some(i)
}

fn jpeg(b: &[u8]) -> Option<ImageInfo> {
    let mut at = 2;
    loop {
        // Skip fill bytes, then read the marker.
        while *b.get(at)? != 0xFF {
            at += 1;
        }
        while *b.get(at)? == 0xFF {
            at += 1;
        }
        let marker = *b.get(at)?;
        at += 1;
        match marker {
            0x01 | 0xD0..=0xD8 => continue,
            0xD9 | 0xDA => return None,
            _ => {}
        }
        let len = be16(b, at)? as usize;
        if matches!(marker, 0xC0..=0xCF) && !matches!(marker, 0xC4 | 0xC8 | 0xCC) {
            let (h, w) = (be16(b, at + 3)?, be16(b, at + 5)?);
            let mut i = info(ImageFormat::Jpeg, w, h);
            i.has_alpha = Some(false);
            return Some(i);
        }
        if len < 2 {
            return None;
        }
        at += len;
    }
}

fn bmp(b: &[u8]) -> Option<ImageInfo> {
    let dib = le32(b, 14)?;
    let (w, h, bpp) = if dib == 12 {
        (le16(b, 18)?, le16(b, 20)?, le16(b, 24)?)
    } else if dib >= 40 {
        (
            (le32(b, 18)? as i32).unsigned_abs(),
            (le32(b, 22)? as i32).unsigned_abs(),
            le16(b, 28)?,
        )
    } else {
        return None;
    };
    let mut i = info(ImageFormat::Bmp, w, h);
    // 32 bpp may or may not use its fourth byte as alpha.
    i.has_alpha = if bpp == 32 { None } else { Some(false) };
    Some(i)
}

fn webp(b: &[u8]) -> Option<ImageInfo> {
    match b.get(12..16)? {
        b"VP8 " => {
            if b.get(23..26)? != [0x9d, 0x01, 0x2a] {
                return None;
            }
            let mut i = info(
                ImageFormat::Webp,
                le16(b, 26)? & 0x3fff,
                le16(b, 28)? & 0x3fff,
            );
            i.has_alpha = Some(false);
            Some(i)
        }
        b"VP8L" => {
            if *b.get(20)? != 0x2f {
                return None;
            }
            let bits = le32(b, 21)?;
            let mut i = info(
                ImageFormat::Webp,
                (bits & 0x3fff) + 1,
                ((bits >> 14) & 0x3fff) + 1,
            );
            i.has_alpha = Some((bits >> 28) & 1 == 1);
            Some(i)
        }
        b"VP8X" => {
            let flags = *b.get(20)?;
            let dim = |at: usize| -> Option<u32> {
                let s = b.get(at..at + 3)?;
                Some((u32::from(s[0]) | u32::from(s[1]) << 8 | u32::from(s[2]) << 16) + 1)
            };
            let mut i = info(ImageFormat::Webp, dim(24)?, dim(27)?);
            i.has_alpha = Some(flags & 0x10 != 0);
            if flags & 0x02 == 0 {
                i.frames = Some(1);
            }
            Some(i)
        }
        _ => None,
    }
}

fn ico(b: &[u8]) -> Option<ImageInfo> {
    let count = le16(b, 4)? as usize;
    if count == 0 {
        return None;
    }
    let mut best: Option<(u32, u32)> = None;
    for n in 0..count {
        let at = 6 + n * 16;
        let (Some(&w), Some(&h)) = (b.get(at), b.get(at + 1)) else {
            break;
        };
        let (w, h) = (if w == 0 { 256 } else { u32::from(w) }, {
            if h == 0 {
                256
            } else {
                u32::from(h)
            }
        });
        if best.is_none_or(|(bw, bh)| w * h > bw * bh) {
            best = Some((w, h));
        }
    }
    let (w, h) = best?;
    let mut i = info(ImageFormat::Ico, w, h);
    i.frames = Some(count as u32);
    Some(i)
}

fn tiff(b: &[u8]) -> Option<ImageInfo> {
    let little = b.starts_with(b"II");
    let u16_at = |at: usize| if little { le16(b, at) } else { be16(b, at) };
    let u32_at = |at: usize| if little { le32(b, at) } else { be32(b, at) };
    let ifd = u32_at(4)? as usize;
    let n = u16_at(ifd)? as usize;
    let (mut w, mut h) = (None, None);
    for k in 0..n.min(256) {
        let e = ifd + 2 + k * 12;
        let tag = u16_at(e)?;
        let ty = u16_at(e + 2)?;
        let value = match ty {
            3 => u16_at(e + 8)?,
            4 => u32_at(e + 8)?,
            _ => continue,
        };
        match tag {
            256 => w = Some(value),
            257 => h = Some(value),
            _ => {}
        }
    }
    Some(info(ImageFormat::Tiff, w?, h?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_bytes(w: u32, h: u32, color: u8) -> Vec<u8> {
        let mut v = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        v.extend(w.to_be_bytes());
        v.extend(h.to_be_bytes());
        v.extend([8, color, 0, 0, 0]);
        v
    }

    #[test]
    fn png_header() {
        let i = sniff(&png_bytes(640, 480, 6)).unwrap();
        assert_eq!((i.format, i.width, i.height), (ImageFormat::Png, 640, 480));
        assert_eq!(i.has_alpha, Some(true));
        assert_eq!(sniff(&png_bytes(1, 2, 2)).unwrap().has_alpha, Some(false));
        assert_eq!(sniff(&png_bytes(1, 2, 3)).unwrap().has_alpha, None);
    }

    #[test]
    fn jpeg_skips_app_segments_to_the_sof() {
        let mut v = vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 4, 1, 2];
        v.extend([0xFF, 0xC2, 0, 11, 8, 0x01, 0x00, 0x02, 0x00, 3, 0, 0, 0]);
        let i = sniff(&v).unwrap();
        assert_eq!((i.format, i.width, i.height), (ImageFormat::Jpeg, 512, 256));
    }

    #[test]
    fn gif_header() {
        let mut v = b"GIF89a".to_vec();
        v.extend([10, 0, 20, 0]);
        let i = sniff(&v).unwrap();
        assert_eq!((i.format, i.width, i.height), (ImageFormat::Gif, 10, 20));
    }

    #[test]
    fn bmp_info_header_and_top_down_negative_height() {
        let mut v = b"BM".to_vec();
        v.extend([0; 12]);
        v.extend(40u32.to_le_bytes());
        v.extend(30i32.to_le_bytes());
        v.extend((-20i32).to_le_bytes());
        v.extend([1, 0, 24, 0]);
        let i = sniff(&v).unwrap();
        assert_eq!((i.format, i.width, i.height), (ImageFormat::Bmp, 30, 20));
        assert_eq!(i.has_alpha, Some(false));
    }

    #[test]
    fn webp_lossy_lossless_and_extended() {
        let mut lossy = b"RIFF\0\0\0\0WEBPVP8 \0\0\0\0\0\0\0\x9d\x01\x2a".to_vec();
        lossy.extend([0x40, 0x01, 0xF0, 0x00]);
        let i = sniff(&lossy).unwrap();
        assert_eq!((i.format, i.width, i.height), (ImageFormat::Webp, 320, 240));

        let mut ll = b"RIFF\0\0\0\0WEBPVP8L\0\0\0\0\x2f".to_vec();
        let bits: u32 = 99 | (49 << 14) | (1 << 28);
        ll.extend(bits.to_le_bytes());
        let i = sniff(&ll).unwrap();
        assert_eq!((i.width, i.height, i.has_alpha), (100, 50, Some(true)));

        let mut x = b"RIFF\0\0\0\0WEBPVP8X\0\0\0\0".to_vec();
        x.extend([0x10, 0, 0, 0, 0xFF, 0, 0, 0x7F, 0, 0]);
        let i = sniff(&x).unwrap();
        assert_eq!((i.width, i.height, i.has_alpha), (256, 128, Some(true)));
    }

    #[test]
    fn ico_takes_the_largest_entry_and_zero_means_256() {
        let mut v = vec![0, 0, 1, 0, 2, 0];
        v.extend([16, 16, 0, 0, 1, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        v.extend([0, 0, 0, 0, 1, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let i = sniff(&v).unwrap();
        assert_eq!((i.format, i.width, i.height), (ImageFormat::Ico, 256, 256));
        assert_eq!(i.frames, Some(2));
    }

    #[test]
    fn tiff_ifd0_both_byte_orders() {
        let mut le = b"II*\0".to_vec();
        le.extend(8u32.to_le_bytes());
        le.extend(2u16.to_le_bytes());
        for (tag, val) in [(256u16, 70u16), (257, 80)] {
            le.extend(tag.to_le_bytes());
            le.extend(3u16.to_le_bytes());
            le.extend(1u32.to_le_bytes());
            le.extend(u32::from(val).to_le_bytes());
        }
        let i = sniff(&le).unwrap();
        assert_eq!((i.format, i.width, i.height), (ImageFormat::Tiff, 70, 80));

        let mut be = b"MM\0*".to_vec();
        be.extend(8u32.to_be_bytes());
        be.extend(2u16.to_be_bytes());
        for (tag, val) in [(256u16, 5u32), (257, 6)] {
            be.extend(tag.to_be_bytes());
            be.extend(4u16.to_be_bytes());
            be.extend(1u32.to_be_bytes());
            be.extend(val.to_be_bytes());
        }
        let i = sniff(&be).unwrap();
        assert_eq!((i.width, i.height), (5, 6));
    }

    #[test]
    fn avif_and_heic_report_the_format_only() {
        let avif = [
            0, 0, 0, 0x1c, b'f', b't', b'y', b'p', b'a', b'v', b'i', b'f',
        ];
        let i = sniff(&avif).unwrap();
        assert_eq!((i.format, i.width, i.height), (ImageFormat::Avif, 0, 0));
        let heic = [
            0, 0, 0, 0x18, b'f', b't', b'y', b'p', b'h', b'e', b'i', b'c',
        ];
        assert_eq!(sniff(&heic).unwrap().format, ImageFormat::Heic);
        let mp4 = [
            0, 0, 0, 0x18, b'f', b't', b'y', b'p', b'i', b's', b'o', b'm',
        ];
        assert!(sniff(&mp4).is_none());
    }

    #[test]
    fn truncated_and_garbage_input_is_none_for_every_prefix() {
        assert!(sniff(b"").is_none());
        assert!(sniff(b"hello world, not an image").is_none());
        assert!(sniff(&[0u8; 64]).is_none());
        let samples = [png_bytes(3, 4, 6), vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 4, 1, 2]];
        for s in samples {
            for n in 0..s.len() - 1 {
                // Must not panic; most prefixes are None.
                let _ = sniff(&s[..n]);
            }
        }
        assert!(sniff(&png_bytes(3, 4, 6)[..20]).is_none());
    }
}
