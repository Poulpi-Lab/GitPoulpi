//! The image view's pure engine (see the image-view design,
//! slice S2): viewport maths, the cropping rasteriser, half-block cells
//! and the latest-wins worker. No UI wiring lives here; the Diff and File
//! views use it from the slices that follow, which is why the engine is
//! `allow(dead_code)` until then.
//!
//! * [`viewport`]: zoom, center, clamp, pointer-anchored zoom, scroll
//!   tracks and the two-pane sync (decision D4).
//! * [`raster`]: crop + resample + checkerboard (D5).
//! * [`halfblock`]: `▀` cells from a bitmap, color-depth aware (D5).
//! * [`layout`], [`split`]: the Diff view's before / after panes (slice S5).
//! * [`worker`]: a background thread where a newer request supersedes
//!   the queued ones (D5).

#![allow(dead_code)]

pub(crate) mod edit_in;
pub(crate) mod halfblock;
pub(crate) mod header;
pub(crate) mod layout;
pub(crate) mod pane;
pub(crate) mod raster;
pub(crate) mod split;
pub(crate) mod viewport;
pub(crate) mod worker;
