//! Image decoding shared by the author avatars and the image view.
//!
//! * [`avatar`]: the original PNG/JPEG-to-opaque-RGB decoder the avatars
//!   use. Always built (avatars never need the `image-view` feature).
//! * [`resample`]: area-average / bilinear resizing. Always built.
//! * With the `image-view` feature: [`decode_rgba`], the full decoder.
//!   PNG (every depth, palette, `tRNS`), JPEG, GIF (first frame), WebP
//!   (first frame), BMP and ICO/CUR (largest entry), all to a straight
//!   alpha [`Bitmap`](crate::terminal_image::Bitmap), behind a pixel cap
//!   checked from the header *before* anything is allocated, and inside
//!   `catch_unwind` so a malformed file is an error, never a crash.
//!
//! Everything is a pure function of bytes already in memory.

mod avatar;
mod resample;

pub(crate) use avatar::decode;
pub(crate) use resample::{downscale, sample_bilinear};

#[cfg(feature = "image-view")]
#[allow(dead_code)] // the pure engine; the image view slices wire it
mod bmp;
#[cfg(feature = "image-view")]
#[allow(dead_code)]
mod rgba;
#[cfg(feature = "image-view")]
#[cfg(test)]
mod rgba_tests;

#[cfg(feature = "image-view")]
#[allow(unused_imports)] // wired by the image view slices that follow
pub(crate) use rgba::{
    decode_guarded, decode_rgba, sniff_format, DecodeError, Decoded, ImageFormat, MAX_PIXELS,
};
