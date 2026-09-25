//! Tests of the full decoder, on tiny generated images. PNG, BMP and ICO
//! are built here; the GIF, WebP and JPEG ones are byte constants made
//! once with ImageMagick (the fixtures' pixel values are in their docs).

use super::*;

/// 2x2, RGBA: red, green / blue, transparent. `magick a.pam a.gif`.
const GIF_2X2: [u8; 50] = [
    0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x02, 0x00, 0x02, 0x00, 0xf1, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xff, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0xff, 0x21, 0xf9, 0x04, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x2c, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0x02, 0x03, 0x8c, 0x06, 0x05,
    0x00, 0x3b,
];
/// The same pixels, lossless WebP with alpha.
const WEBP_RGBA_2X2: [u8; 50] = [
    0x52, 0x49, 0x46, 0x46, 0x2a, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x56, 0x50, 0x38, 0x4c,
    0x1e, 0x00, 0x00, 0x00, 0x2f, 0x01, 0x40, 0x00, 0x10, 0x1f, 0x20, 0x10, 0x48, 0xda, 0x1f, 0x7a,
    0x8d, 0xf9, 0x17, 0x10, 0x14, 0xf9, 0x3f, 0xda, 0xfc, 0x07, 0xf0, 0xa5, 0x60, 0x17, 0x22, 0xfa,
    0x1f, 0x01,
];
/// Lossless WebP, no alpha: red, green / blue, (10,20,30).
const WEBP_RGB_2X2: [u8; 58] = [
    0x52, 0x49, 0x46, 0x46, 0x32, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x56, 0x50, 0x38, 0x4c,
    0x26, 0x00, 0x00, 0x00, 0x2f, 0x01, 0x40, 0x00, 0x00, 0x1f, 0x20, 0x10, 0x20, 0x78, 0xf0, 0xbf,
    0x6a, 0x43, 0x48, 0x90, 0xd0, 0xfd, 0xbf, 0x57, 0x81, 0x00, 0xc1, 0x88, 0xff, 0x4b, 0xf3, 0x1f,
    0xf8, 0xe4, 0xa8, 0xe0, 0x06, 0x0c, 0x11, 0xfd, 0x0f, 0x01,
];
/// An 8x8 baseline JPEG of one flat color, (200, 100, 50).
const JPEG_FLAT_8X8: [u8; 285] = [
    0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
    0x00, 0x01, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43, 0x00, 0x02, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02,
    0x01, 0x01, 0x01, 0x02, 0x02, 0x02, 0x02, 0x02, 0x04, 0x03, 0x02, 0x02, 0x02, 0x02, 0x05, 0x04,
    0x04, 0x03, 0x04, 0x06, 0x05, 0x06, 0x06, 0x06, 0x05, 0x06, 0x06, 0x06, 0x07, 0x09, 0x08, 0x06,
    0x07, 0x09, 0x07, 0x06, 0x06, 0x08, 0x0b, 0x08, 0x09, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x06, 0x08,
    0x0b, 0x0c, 0x0b, 0x0a, 0x0c, 0x09, 0x0a, 0x0a, 0x0a, 0xff, 0xdb, 0x00, 0x43, 0x01, 0x02, 0x02,
    0x02, 0x02, 0x02, 0x02, 0x05, 0x03, 0x03, 0x05, 0x0a, 0x07, 0x06, 0x07, 0x0a, 0x0a, 0x0a, 0x0a,
    0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a,
    0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a,
    0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0x0a, 0xff, 0xc0,
    0x00, 0x11, 0x08, 0x00, 0x08, 0x00, 0x08, 0x03, 0x01, 0x11, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11,
    0x01, 0xff, 0xc4, 0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0xff, 0xc4, 0x00, 0x14, 0x10, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xc4, 0x00,
    0x14, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x08, 0xff, 0xc4, 0x00, 0x14, 0x11, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xda, 0x00, 0x0c, 0x03, 0x01, 0x00,
    0x02, 0x11, 0x03, 0x11, 0x00, 0x3f, 0x00, 0x3c, 0x57, 0x36, 0x1f, 0xff, 0xd9,
];

fn png_bytes(w: u32, h: u32, color: png::ColorType, depth: png::BitDepth, data: &[u8]) -> Vec<u8> {
    png_with(w, h, color, depth, data, |_| {})
}

fn png_with(
    w: u32,
    h: u32,
    color: png::ColorType,
    depth: png::BitDepth,
    data: &[u8],
    tweak: impl FnOnce(&mut png::Encoder<&mut Vec<u8>>),
) -> Vec<u8> {
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(color);
    enc.set_depth(depth);
    tweak(&mut enc);
    let mut wr = enc.write_header().unwrap();
    wr.write_image_data(data).unwrap();
    wr.finish().unwrap();
    out
}

fn px(d: &Decoded, x: u32, y: u32) -> (u8, u8, u8, u8) {
    d.bitmap.rgba(x, y).unwrap()
}

/// A BMP file: `dib` is the header onwards, `pixels` the pixel array.
fn bmp_file(dib: &[u8], pixels: &[u8]) -> Vec<u8> {
    let off = 14 + dib.len();
    let mut f = b"BM".to_vec();
    f.extend(((off + pixels.len()) as u32).to_le_bytes());
    f.extend([0u8; 4]);
    f.extend((off as u32).to_le_bytes());
    f.extend(dib);
    f.extend(pixels);
    f
}

/// A BITMAPINFOHEADER.
fn info_header(w: i32, h: i32, bpp: u16, comp: u32, ncolors: u32) -> Vec<u8> {
    let mut d = 40u32.to_le_bytes().to_vec();
    d.extend(w.to_le_bytes());
    d.extend(h.to_le_bytes());
    d.extend(1u16.to_le_bytes());
    d.extend(bpp.to_le_bytes());
    d.extend(comp.to_le_bytes());
    d.extend([0u8; 12]);
    d.extend(ncolors.to_le_bytes());
    d.extend([0u8; 4]);
    d
}

#[test]
fn every_format_is_recognized_by_its_magic_bytes_only() {
    assert_eq!(sniff_format(&GIF_2X2), Some(ImageFormat::Gif));
    assert_eq!(sniff_format(&WEBP_RGBA_2X2), Some(ImageFormat::WebP));
    assert_eq!(sniff_format(&JPEG_FLAT_8X8), Some(ImageFormat::Jpeg));
    let png = png_bytes(1, 1, png::ColorType::Grayscale, png::BitDepth::Eight, &[0]);
    assert_eq!(sniff_format(&png), Some(ImageFormat::Png));
    assert_eq!(sniff_format(b"BM\0\0"), Some(ImageFormat::Bmp));
    assert_eq!(sniff_format(&[0, 0, 1, 0, 1, 0]), Some(ImageFormat::Ico));
    assert_eq!(sniff_format(&[0, 0, 2, 0, 1, 0]), Some(ImageFormat::Ico));
    assert_eq!(
        sniff_format(&[0, 0, 1, 0, 0, 0]),
        None,
        "an empty ICO directory"
    );
    assert_eq!(sniff_format(b"<html>"), None);
    assert_eq!(sniff_format(&[]), None);
    assert_eq!(decode_rgba(b"not an image"), Err(DecodeError::Unrecognized));
}

#[test]
fn png_rgba_keeps_straight_alpha() {
    let b = png_bytes(
        2,
        1,
        png::ColorType::Rgba,
        png::BitDepth::Eight,
        &[1, 2, 3, 255, 9, 8, 7, 100],
    );
    let d = decode_rgba(&b).unwrap();
    assert_eq!(
        (d.format, d.bitmap.width, d.bitmap.height),
        (ImageFormat::Png, 2, 1)
    );
    assert_eq!(px(&d, 0, 0), (1, 2, 3, 255));
    assert_eq!(px(&d, 1, 0), (9, 8, 7, 100));
    assert!(d.has_alpha);
}

#[test]
fn png_palette_with_a_transparency_chunk_expands() {
    let b = png_with(
        2,
        1,
        png::ColorType::Indexed,
        png::BitDepth::Eight,
        &[0, 1],
        |e| {
            e.set_palette(vec![10, 20, 30, 40, 50, 60]);
            e.set_trns(vec![255, 0]);
        },
    );
    let d = decode_rgba(&b).unwrap();
    assert_eq!(px(&d, 0, 0), (10, 20, 30, 255));
    assert_eq!(px(&d, 1, 0), (40, 50, 60, 0));
}

#[test]
fn png_sixteen_bit_and_gray_alpha_decode() {
    // 16-bit RGB: the high byte of each sample wins.
    let b = png_bytes(
        1,
        1,
        png::ColorType::Rgb,
        png::BitDepth::Sixteen,
        &[0x80, 0x00, 0x40, 0xff, 0xff, 0xff],
    );
    let d = decode_rgba(&b).unwrap();
    assert_eq!(px(&d, 0, 0), (0x80, 0x40, 0xff, 255));
    let b = png_bytes(
        1,
        1,
        png::ColorType::GrayscaleAlpha,
        png::BitDepth::Eight,
        &[77, 128],
    );
    assert_eq!(px(&decode_rgba(&b).unwrap(), 0, 0), (77, 77, 77, 128));
    // 1-bit gray: the bits expand to 0 / 255.
    let b = png_bytes(
        2,
        1,
        png::ColorType::Grayscale,
        png::BitDepth::One,
        &[0b0100_0000],
    );
    let d = decode_rgba(&b).unwrap();
    assert_eq!(
        (px(&d, 0, 0), px(&d, 1, 0)),
        ((0, 0, 0, 255), (255, 255, 255, 255))
    );
}

#[test]
fn jpeg_decodes_to_opaque_rgb() {
    let d = decode_rgba(&JPEG_FLAT_8X8).unwrap();
    assert_eq!(
        (d.format, d.bitmap.width, d.bitmap.height),
        (ImageFormat::Jpeg, 8, 8)
    );
    let (r, g, b, a) = px(&d, 3, 4);
    assert!(
        r.abs_diff(200) < 6 && g.abs_diff(100) < 6 && b.abs_diff(50) < 6,
        "{r},{g},{b}"
    );
    assert_eq!(a, 255);
    assert!(!d.has_alpha);
}

#[test]
fn gif_first_frame_with_a_transparent_index() {
    let d = decode_rgba(&GIF_2X2).unwrap();
    assert_eq!(
        (d.format, d.bitmap.width, d.bitmap.height),
        (ImageFormat::Gif, 2, 2)
    );
    assert_eq!(px(&d, 0, 0), (255, 0, 0, 255));
    assert_eq!(px(&d, 1, 0), (0, 255, 0, 255));
    assert_eq!(px(&d, 0, 1), (0, 0, 255, 255));
    assert_eq!(px(&d, 1, 1).3, 0);
    assert!(d.has_alpha);
}

#[test]
fn webp_lossless_with_and_without_alpha() {
    let d = decode_rgba(&WEBP_RGBA_2X2).unwrap();
    assert_eq!(
        (d.format, d.bitmap.width, d.bitmap.height),
        (ImageFormat::WebP, 2, 2)
    );
    assert_eq!(px(&d, 0, 0), (255, 0, 0, 255));
    assert_eq!(px(&d, 1, 0), (0, 255, 0, 255));
    assert_eq!(px(&d, 0, 1), (0, 0, 255, 255));
    assert_eq!(px(&d, 1, 1).3, 0);
    let d = decode_rgba(&WEBP_RGB_2X2).unwrap();
    assert_eq!(px(&d, 1, 1), (10, 20, 30, 255));
    assert!(!d.has_alpha);
}

#[test]
fn bmp_24_bit_bottom_up_and_top_down_agree() {
    // Rows are BGR, padded to 4 bytes. Picture (top row first):
    //   red  green
    //   blue white
    let top = [0, 0, 255, 0, 255, 0, 0, 0];
    let bottom = [255, 0, 0, 255, 255, 255, 0, 0];
    let up = bmp_file(&info_header(2, 2, 24, 0, 0), &[bottom, top].concat());
    let down = bmp_file(&info_header(2, -2, 24, 0, 0), &[top, bottom].concat());
    for f in [up, down] {
        let d = decode_rgba(&f).unwrap();
        assert_eq!(d.format, ImageFormat::Bmp);
        assert_eq!(px(&d, 0, 0), (255, 0, 0, 255));
        assert_eq!(px(&d, 1, 0), (0, 255, 0, 255));
        assert_eq!(px(&d, 0, 1), (0, 0, 255, 255));
        assert_eq!(px(&d, 1, 1), (255, 255, 255, 255));
    }
}

#[test]
fn bmp_palette_depths_1_4_and_8() {
    // Palette entries are BGRA.
    let pal: Vec<u8> = [[0, 0, 255, 0], [255, 0, 0, 0], [0, 255, 0, 0], [9, 9, 9, 0]].concat();
    // 1 bpp, 8 px wide: 0b10100000 -> idx 1,0,1,0,0,0,0,0 (colors 0 = red, 1 = blue).
    let mut dib = info_header(8, 1, 1, 0, 2);
    dib.extend(&pal[..8]);
    let d = decode_rgba(&bmp_file(&dib, &[0b1010_0000, 0, 0, 0])).unwrap();
    assert_eq!(
        (px(&d, 0, 0), px(&d, 1, 0)),
        ((0, 0, 255, 255), (255, 0, 0, 255))
    );
    // 4 bpp: nibbles 2,1,3,0.
    let mut dib = info_header(4, 1, 4, 0, 4);
    dib.extend(&pal);
    let d = decode_rgba(&bmp_file(&dib, &[0x21, 0x30, 0, 0])).unwrap();
    assert_eq!(px(&d, 0, 0), (0, 255, 0, 255));
    assert_eq!(px(&d, 1, 0), (0, 0, 255, 255));
    assert_eq!(px(&d, 2, 0), (9, 9, 9, 255));
    assert_eq!(px(&d, 3, 0), (255, 0, 0, 255));
    // 8 bpp, ncolors = 0 means 256 entries.
    let mut dib = info_header(2, 1, 8, 0, 0);
    let mut full = vec![0u8; 256 * 4];
    full[4..8].copy_from_slice(&[1, 2, 3, 0]); // index 1 = rgb(3,2,1)
    dib.extend(full);
    let d = decode_rgba(&bmp_file(&dib, &[1, 0, 0, 0])).unwrap();
    assert_eq!(px(&d, 0, 0), (3, 2, 1, 255));
}

#[test]
fn bmp_32_bit_alpha_is_only_real_when_something_sets_it() {
    // BI_RGB: a zero fourth byte everywhere is padding -> opaque.
    let f = bmp_file(&info_header(1, 1, 32, 0, 0), &[30, 20, 10, 0]);
    assert_eq!(px(&decode_rgba(&f).unwrap(), 0, 0), (10, 20, 30, 255));
    // A non-zero one is alpha.
    let f = bmp_file(
        &info_header(2, 1, 32, 0, 0),
        &[30, 20, 10, 128, 1, 2, 3, 255],
    );
    let d = decode_rgba(&f).unwrap();
    assert_eq!(px(&d, 0, 0), (10, 20, 30, 128));
    assert!(d.has_alpha);
}

#[test]
fn bmp_bitfields_16_and_32_bit() {
    // 16-bit RGB565 through BI_BITFIELDS (masks follow the 40-byte header).
    let mut dib = info_header(1, 1, 16, 3, 0);
    for m in [0xf800u32, 0x07e0, 0x001f] {
        dib.extend(m.to_le_bytes());
    }
    let v: u16 = (31 << 11) | 31; // magenta
    let mut row = v.to_le_bytes().to_vec();
    row.extend([0, 0]);
    let d = decode_rgba(&bmp_file(&dib, &row)).unwrap();
    assert_eq!(px(&d, 0, 0), (255, 0, 255, 255));
    // 16-bit default (no masks) is RGB555.
    let v: u16 = 31 << 5; // green
    let mut row = v.to_le_bytes().to_vec();
    row.extend([0, 0]);
    let d = decode_rgba(&bmp_file(&info_header(1, 1, 16, 0, 0), &row)).unwrap();
    assert_eq!(px(&d, 0, 0), (0, 255, 0, 255));
    // 32-bit ARGB with an alpha mask (BI_ALPHABITFIELDS).
    let mut dib = info_header(1, 1, 32, 6, 0);
    for m in [0x00ff0000u32, 0x0000ff00, 0x000000ff, 0xff000000] {
        dib.extend(m.to_le_bytes());
    }
    let v: u32 = (0x40 << 24) | (0x11 << 16) | (0x22 << 8) | 0x33;
    let d = decode_rgba(&bmp_file(&dib, &v.to_le_bytes())).unwrap();
    assert_eq!(px(&d, 0, 0), (0x11, 0x22, 0x33, 0x40));
}

#[test]
fn bmp_rle_is_reported_not_guessed() {
    let f = bmp_file(&info_header(1, 1, 8, 1, 0), &[0; 8]);
    assert!(matches!(decode_rgba(&f), Err(DecodeError::Unsupported(_))));
}

fn ico_dir(entries: &[(u8, u8, u16, Vec<u8>)]) -> Vec<u8> {
    let mut out = vec![0, 0, 1, 0];
    out.extend((entries.len() as u16).to_le_bytes());
    let mut off = 6 + 16 * entries.len();
    for (w, h, bpp, data) in entries {
        out.extend([*w, *h, 0, 0, 1, 0]);
        out.extend(bpp.to_le_bytes());
        out.extend((data.len() as u32).to_le_bytes());
        out.extend((off as u32).to_le_bytes());
        off += data.len();
    }
    for (_, _, _, data) in entries {
        out.extend(data);
    }
    out
}

#[test]
fn ico_takes_the_largest_entry_whether_png_or_dib() {
    let small = png_bytes(
        1,
        1,
        png::ColorType::Rgba,
        png::BitDepth::Eight,
        &[1, 1, 1, 255],
    );
    let big = png_bytes(
        2,
        2,
        png::ColorType::Rgba,
        png::BitDepth::Eight,
        &[7, 7, 7, 255, 7, 7, 7, 255, 7, 7, 7, 255, 7, 7, 7, 255],
    );
    let ico = ico_dir(&[(1, 1, 32, small.clone()), (2, 2, 32, big)]);
    let d = decode_rgba(&ico).unwrap();
    assert_eq!(
        (d.format, d.bitmap.width, px(&d, 1, 1)),
        (ImageFormat::Ico, 2, (7, 7, 7, 255))
    );
    // A DIB entry: 2x2 at 24 bpp (height doubled in the header), bottom-up
    // rows, then the 1-bit AND mask (set = transparent): only the top-left
    // pixel is masked out.
    let mut dib = info_header(2, 4, 24, 0, 0);
    dib.extend([1, 2, 3, 4, 5, 6, 0, 0]); // bottom row
    dib.extend([9, 9, 9, 9, 9, 9, 0, 0]); // top row
    dib.extend([0, 0, 0, 0]); // mask bottom row
    dib.extend([0b1000_0000, 0, 0, 0]); // mask top row
    let d = decode_rgba(&ico_dir(&[(2, 2, 24, dib)])).unwrap();
    assert_eq!((d.bitmap.width, d.bitmap.height), (2, 2));
    assert_eq!(px(&d, 0, 0).3, 0);
    assert_eq!(px(&d, 1, 0), (9, 9, 9, 255));
    assert_eq!(px(&d, 0, 1), (3, 2, 1, 255));
}

#[test]
fn the_pixel_cap_is_checked_from_the_header_before_decoding() {
    // 8001 x 8001 one-bit gray: tiny once deflated, 64.016 MP declared.
    let data = vec![0u8; 1001 * 8001];
    let b = png_bytes(
        8001,
        8001,
        png::ColorType::Grayscale,
        png::BitDepth::One,
        &data,
    );
    assert!(b.len() < 200_000);
    assert_eq!(
        decode_rgba(&b),
        Err(DecodeError::TooLarge {
            width: 8001,
            height: 8001
        })
    );
    // A GIF screen descriptor claiming 20000 x 20000 and nothing else.
    let mut g = b"GIF89a".to_vec();
    g.extend(20000u16.to_le_bytes());
    g.extend(20000u16.to_le_bytes());
    g.extend([0, 0, 0]);
    assert_eq!(
        decode_rgba(&g),
        Err(DecodeError::TooLarge {
            width: 20000,
            height: 20000
        })
    );
    // A BMP header claiming 100000 x 100000.
    let f = bmp_file(&info_header(100_000, 100_000, 24, 0, 0), &[]);
    assert!(matches!(decode_rgba(&f), Err(DecodeError::TooLarge { .. })));
    assert_eq!(MAX_PIXELS, 64_000_000);
}

#[test]
fn damaged_files_are_errors_never_panics() {
    let png = png_bytes(3, 3, png::ColorType::Rgba, png::BitDepth::Eight, &[200; 36]);
    let bmp = bmp_file(&info_header(2, 2, 24, 0, 0), &[0; 16]);
    let ico = ico_dir(&[(2, 2, 24, info_header(2, 4, 24, 0, 0))]);
    for (name, f) in [
        ("png", png.as_slice()),
        ("jpeg", &JPEG_FLAT_8X8),
        ("gif", &GIF_2X2),
        ("webp", &WEBP_RGBA_2X2),
        ("bmp", &bmp),
        ("ico", &ico),
    ] {
        for len in 0..f.len() {
            // Not asserting Ok/Err: only that nothing panics or hangs.
            let _ = decode_guarded(&f[..len]);
        }
        // Every single-byte corruption too.
        for i in 0..f.len().min(120) {
            let mut c = f.to_vec();
            c[i] ^= 0xa5;
            let r = decode_guarded(&c);
            assert_ne!(r, Err(DecodeError::Panicked), "{name} byte {i} panicked");
        }
    }
    assert_eq!(
        decode_guarded(&png[..png.len() / 2]),
        Err(DecodeError::Corrupt)
    );
    assert_eq!(decode_guarded(b"garbage"), Err(DecodeError::Unrecognized));
}
