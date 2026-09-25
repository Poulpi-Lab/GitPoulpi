//! The zoom/scroll model (decision D4), pure arithmetic.
//!
//! A [`Viewport`] is `(zoom, center)` with the center in **normalized image
//! coordinates** (`0..=1` on each axis), so it does not depend on the pane
//! size. The pane size is an argument of every query, which makes the
//! re-adaptation rule free:
//!
//! * `Fit` stays `Fit` across any pane resize, layout switch or divider
//!   drag;
//! * `Scale(s)` keeps `s` and its center, so the point under the center
//!   stays under the center;
//! * after any change the center is clamped so no axis scrolls past an
//!   edge, and an axis that fits inside the pane is centered;
//! * scroll tracks are computed from `(scaled image, pane)` on demand and
//!   never cached.
//!
//! **Units.** Pane geometry is in pixels (cells x `Capability.cell`, or
//! cols x rows*2 for the half-block grid). `Scale(s)` is *screen pixels per
//! image pixel* (2.0 = 200 %, 1.0 = 1:1), clamped to
//! [`MIN_SCALE`]..=[`MAX_SCALE`]. (The plan words this as image px per
//! screen px; the header shows a percentage and `+` multiplies, so the
//! inverse reads naturally. The bounds are the same pair.)

pub(crate) const MIN_SCALE: f32 = 1.0 / 64.0;
pub(crate) const MAX_SCALE: f32 = 32.0;
/// `+` / `-` multiply / divide the scale by this.
pub(crate) const ZOOM_STEP: f32 = std::f32::consts::SQRT_2;

/// A size in pixels (image or pane).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Size {
    pub w: f32,
    pub h: f32,
}

impl Size {
    pub(crate) fn new(w: f32, h: f32) -> Size {
        Size { w, h }
    }

    pub(crate) fn px(w: u32, h: u32) -> Size {
        Size::new(w as f32, h as f32)
    }

    fn is_empty(self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Zoom {
    Fit,
    Scale(f32),
}

/// What to draw: the part of the image visible, and where in the pane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct View {
    pub scale: f32,
    /// Visible part of the image, in image pixels (x, y, w, h).
    pub src: (f32, f32, f32, f32),
    /// Where it lands in the pane, in pane pixels (x, y, w, h). An axis that
    /// fits is centered, so x/y can be positive.
    pub dst: (f32, f32, f32, f32),
}

/// One axis of a scroll track, in pane pixels: the scaled content length,
/// the visible length, and the scroll offset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Track {
    pub content: f32,
    pub view: f32,
    pub offset: f32,
}

impl Track {
    /// Whether this axis scrolls at all (a scrollbar is shown).
    pub(crate) fn scrolls(&self) -> bool {
        self.content > self.view + 0.5
    }

    /// The offset a drag of the thumb to `thumb_top` (0..=1 of the track)
    /// means.
    pub(crate) fn offset_for_fraction(&self, fraction: f32) -> f32 {
        (fraction.clamp(0.0, 1.0) * (self.content - self.view).max(0.0)).max(0.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Viewport {
    pub zoom: Zoom,
    pub center: (f32, f32),
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport::fit()
    }
}

/// The scale that fits `img` inside `pane`; with `upscale` off a small
/// image is not enlarged past 1:1 (it is centered instead).
pub(crate) fn fit_scale(img: Size, pane: Size, upscale: bool) -> f32 {
    if img.is_empty() || pane.is_empty() {
        return 1.0;
    }
    let s = (pane.w / img.w).min(pane.h / img.h);
    let s = if upscale { s } else { s.min(1.0) };
    s.clamp(MIN_SCALE, MAX_SCALE)
}

fn clamp_scale(s: f32) -> f32 {
    s.clamp(MIN_SCALE, MAX_SCALE)
}

/// The center (normalized) clamped on one axis: an axis that fits in the
/// pane is centered, otherwise the visible span stays inside the image.
fn clamp_axis(center: f32, img: f32, pane: f32, scale: f32) -> f32 {
    let scaled = img * scale;
    if scaled <= pane || img <= 0.0 {
        return 0.5;
    }
    let half = pane / (2.0 * scale) / img; // half the visible span, normalized
    center.clamp(half, 1.0 - half)
}

impl Viewport {
    pub(crate) fn fit() -> Viewport {
        Viewport {
            zoom: Zoom::Fit,
            center: (0.5, 0.5),
        }
    }

    /// 1:1, centered.
    pub(crate) fn actual_size() -> Viewport {
        Viewport {
            zoom: Zoom::Scale(1.0),
            center: (0.5, 0.5),
        }
    }

    pub(crate) fn is_fit(&self) -> bool {
        self.zoom == Zoom::Fit
    }

    /// The scale actually used for `img` in `pane`.
    pub(crate) fn scale(&self, img: Size, pane: Size, fit_upscale: bool) -> f32 {
        match self.zoom {
            Zoom::Fit => fit_scale(img, pane, fit_upscale),
            Zoom::Scale(s) => clamp_scale(s),
        }
    }

    /// The same viewport with the center clamped for this geometry. Fit
    /// always centers.
    pub(crate) fn clamped(&self, img: Size, pane: Size, fit_upscale: bool) -> Viewport {
        let s = self.scale(img, pane, fit_upscale);
        let center = match self.zoom {
            Zoom::Fit => (0.5, 0.5),
            Zoom::Scale(_) => (
                clamp_axis(self.center.0, img.w, pane.w, s),
                clamp_axis(self.center.1, img.h, pane.h, s),
            ),
        };
        Viewport {
            zoom: self.zoom,
            center,
        }
    }

    /// The visible crop and its placement.
    pub(crate) fn view(&self, img: Size, pane: Size, fit_upscale: bool) -> View {
        let scale = self.scale(img, pane, fit_upscale);
        let c = self.clamped(img, pane, fit_upscale).center;
        let axis = |center: f32, img: f32, pane: f32| -> (f32, f32, f32, f32) {
            let scaled = img * scale;
            if scaled <= pane {
                (0.0, img, (pane - scaled) / 2.0, scaled)
            } else {
                let span = pane / scale;
                let start = (center * img - span / 2.0).clamp(0.0, img - span);
                (start, span, 0.0, pane)
            }
        };
        let (sx, sw, dx, dw) = axis(c.0, img.w, pane.w);
        let (sy, sh, dy, dh) = axis(c.1, img.h, pane.h);
        View {
            scale,
            src: (sx, sy, sw, sh),
            dst: (dx, dy, dw, dh),
        }
    }

    /// The horizontal and vertical scroll tracks for the current geometry.
    pub(crate) fn tracks(&self, img: Size, pane: Size, fit_upscale: bool) -> (Track, Track) {
        let v = self.view(img, pane, fit_upscale);
        (
            Track {
                content: img.w * v.scale,
                view: v.dst.2.min(pane.w),
                offset: v.src.0 * v.scale,
            },
            Track {
                content: img.h * v.scale,
                view: v.dst.3.min(pane.h),
                offset: v.src.1 * v.scale,
            },
        )
    }

    /// The image point (image pixels) under the pane point `p`. Points in
    /// the margin around a centered image map outside `0..img`.
    pub(crate) fn image_at(
        &self,
        img: Size,
        pane: Size,
        fit_upscale: bool,
        p: (f32, f32),
    ) -> (f32, f32) {
        let v = self.view(img, pane, fit_upscale);
        (
            v.src.0 + (p.0 - v.dst.0) / v.scale,
            v.src.1 + (p.1 - v.dst.1) / v.scale,
        )
    }

    /// Zooms by `factor`, keeping the image point under `anchor` (pane
    /// pixels) where it is; `None` anchors on the pane center (keyboard
    /// zoom). Leaves Fit for an explicit scale.
    pub(crate) fn zoom_by(
        &self,
        factor: f32,
        img: Size,
        pane: Size,
        fit_upscale: bool,
        anchor: Option<(f32, f32)>,
    ) -> Viewport {
        let cur = self.scale(img, pane, fit_upscale);
        let new = clamp_scale(cur * factor);
        self.rescaled(new, img, pane, fit_upscale, anchor)
    }

    /// An explicit scale (e.g. 1:1), anchored like [`Viewport::zoom_by`].
    pub(crate) fn zoom_to(
        &self,
        scale: f32,
        img: Size,
        pane: Size,
        fit_upscale: bool,
        anchor: Option<(f32, f32)>,
    ) -> Viewport {
        self.rescaled(clamp_scale(scale), img, pane, fit_upscale, anchor)
    }

    fn rescaled(
        &self,
        new: f32,
        img: Size,
        pane: Size,
        fit_upscale: bool,
        anchor: Option<(f32, f32)>,
    ) -> Viewport {
        let anchor = anchor.unwrap_or((pane.w / 2.0, pane.h / 2.0));
        let at = self.image_at(img, pane, fit_upscale, anchor);
        // The new center puts `at` back under `anchor`:
        //   at = center*img + (anchor - pane/2)/new
        let center = (
            (at.0 - (anchor.0 - pane.w / 2.0) / new) / img.w.max(f32::EPSILON),
            (at.1 - (anchor.1 - pane.h / 2.0) / new) / img.h.max(f32::EPSILON),
        );
        Viewport {
            zoom: Zoom::Scale(new),
            center,
        }
        .clamped(img, pane, fit_upscale)
    }

    /// Pans by `delta` pane pixels (positive = towards the right / bottom
    /// of the image). Panning from Fit leaves Fit only if something scrolls.
    pub(crate) fn pan_by(
        &self,
        delta: (f32, f32),
        img: Size,
        pane: Size,
        fit_upscale: bool,
    ) -> Viewport {
        let s = self.scale(img, pane, fit_upscale);
        let c = self.clamped(img, pane, fit_upscale).center;
        let moved = Viewport {
            zoom: Zoom::Scale(s),
            center: (
                c.0 + delta.0 / s / img.w.max(f32::EPSILON),
                c.1 + delta.1 / s / img.h.max(f32::EPSILON),
            ),
        }
        .clamped(img, pane, fit_upscale);
        if self.is_fit() && moved.center == (0.5, 0.5) {
            *self
        } else {
            moved
        }
    }

    /// Sets the scroll offset of one axis outright (scrollbar drag), in
    /// scaled pane pixels from the start of the content.
    pub(crate) fn scroll_to(
        &self,
        axis_x: bool,
        offset: f32,
        img: Size,
        pane: Size,
        fit_upscale: bool,
    ) -> Viewport {
        let s = self.scale(img, pane, fit_upscale);
        let mut c = self.clamped(img, pane, fit_upscale).center;
        if axis_x {
            c.0 = (offset + pane.w / 2.0) / s / img.w.max(f32::EPSILON);
        } else {
            c.1 = (offset + pane.h / 2.0) / s / img.h.max(f32::EPSILON);
        }
        Viewport {
            zoom: Zoom::Scale(s),
            center: c,
        }
        .clamped(img, pane, fit_upscale)
    }

    /// The view of a second image that shares this viewport (sync on).
    ///
    /// `Fit` fits each image on its own. A `Scale(s)` is defined against
    /// `reference`: an image of the same size uses the same scale, one of a
    /// different size uses `s * reference.w / img.w`, so both cover the same
    /// fraction of the pane and the shared normalized center points at the
    /// same relative spot in each.
    pub(crate) fn view_synced(
        &self,
        img: Size,
        reference: Size,
        pane: Size,
        fit_upscale: bool,
    ) -> View {
        let own = match self.zoom {
            Zoom::Fit => *self,
            Zoom::Scale(s) => {
                let rel = if img.w > 0.0 {
                    reference.w / img.w
                } else {
                    1.0
                };
                Viewport {
                    zoom: Zoom::Scale(clamp_scale(s * rel)),
                    center: self.center,
                }
            }
        };
        own.view(img, pane, fit_upscale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IMG: Size = Size {
        w: 1000.0,
        h: 500.0,
    };
    const PANE: Size = Size { w: 200.0, h: 100.0 };

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn fit_fills_the_pane_and_never_upscales_a_small_image() {
        assert!(close(fit_scale(IMG, PANE, false), 0.2));
        let icon = Size::new(16.0, 16.0);
        assert!(close(fit_scale(icon, PANE, false), 1.0));
        assert!(close(fit_scale(icon, PANE, true), 100.0 / 16.0));
        // The small icon is centered, not stretched.
        let v = Viewport::fit().view(icon, PANE, false);
        assert_eq!(v.src, (0.0, 0.0, 16.0, 16.0));
        assert_eq!(v.dst, (92.0, 42.0, 16.0, 16.0));
    }

    #[test]
    fn fit_is_kept_across_any_resize_and_refits() {
        let vp = Viewport::fit();
        for pane in [PANE, Size::new(400.0, 50.0), Size::new(30.0, 300.0)] {
            let after = vp.clamped(IMG, pane, false);
            assert!(after.is_fit());
            let v = after.view(IMG, pane, false);
            // The whole image is visible, scaled to fit one axis exactly.
            assert_eq!(v.src, (0.0, 0.0, 1000.0, 500.0));
            let fits_w = close(v.dst.2, pane.w);
            let fits_h = close(v.dst.3, pane.h);
            assert!(fits_w || fits_h, "{pane:?} -> {v:?}");
        }
    }

    #[test]
    fn a_scale_keeps_its_value_and_its_center_across_a_resize() {
        let vp = Viewport {
            zoom: Zoom::Scale(1.0),
            center: (0.3, 0.6),
        };
        let bigger = Size::new(300.0, 150.0);
        let after = vp.clamped(IMG, bigger, false);
        assert_eq!(after.zoom, Zoom::Scale(1.0));
        assert!(close(after.center.0, 0.3) && close(after.center.1, 0.6));
        let v = after.view(IMG, bigger, false);
        // The point under the pane center is the same image point.
        let before_pt = vp.image_at(IMG, PANE, false, (100.0, 50.0));
        let after_pt = after.image_at(IMG, bigger, false, (150.0, 75.0));
        assert!(close(before_pt.0, after_pt.0) && close(before_pt.1, after_pt.1));
        assert!(close(v.src.2, 300.0));
    }

    #[test]
    fn the_center_is_clamped_at_every_edge() {
        let at = |c: (f32, f32)| {
            Viewport {
                zoom: Zoom::Scale(1.0),
                center: c,
            }
            .view(IMG, PANE, false)
            .src
        };
        assert_eq!(at((-5.0, -5.0)), (0.0, 0.0, 200.0, 100.0));
        assert_eq!(at((9.0, 9.0)), (800.0, 400.0, 200.0, 100.0));
        // An axis that fits is centered whatever the center says.
        let wide = Size::new(2000.0, 100.0);
        let v = Viewport {
            zoom: Zoom::Scale(1.0),
            center: (0.5, 0.9),
        }
        .view(IMG, wide, false);
        assert_eq!(v.dst.0, (2000.0 - 1000.0) / 2.0);
        assert_eq!(v.src.0, 0.0);
    }

    #[test]
    fn zooming_on_the_pane_center_keeps_the_center() {
        let vp = Viewport {
            zoom: Zoom::Scale(1.0),
            center: (0.4, 0.45),
        };
        let z = vp.zoom_by(ZOOM_STEP, IMG, PANE, false, None);
        assert!(close(z.center.0, 0.4) && close(z.center.1, 0.45));
        assert!(matches!(z.zoom, Zoom::Scale(s) if close(s, ZOOM_STEP)));
        // From Fit the first step starts from the fit scale.
        let z = Viewport::fit().zoom_by(2.0, IMG, PANE, false, None);
        assert!(matches!(z.zoom, Zoom::Scale(s) if close(s, 0.4)));
    }

    #[test]
    fn pointer_anchored_zoom_maps_the_point_back_exactly() {
        let vp = Viewport {
            zoom: Zoom::Scale(1.0),
            center: (0.5, 0.5),
        };
        let anchor = (150.0, 30.0);
        let before = vp.image_at(IMG, PANE, false, anchor);
        for factor in [ZOOM_STEP, 2.0, 0.5, 1.0 / ZOOM_STEP] {
            let z = vp.zoom_by(factor, IMG, PANE, false, Some(anchor));
            let after = z.image_at(IMG, PANE, false, anchor);
            assert!(
                close(before.0, after.0) && close(before.1, after.1),
                "factor {factor}: {before:?} vs {after:?}"
            );
        }
    }

    #[test]
    fn zoom_is_clamped_to_its_bounds() {
        let z = Viewport::actual_size().zoom_to(1e9, IMG, PANE, false, None);
        assert_eq!(z.zoom, Zoom::Scale(MAX_SCALE));
        let z = Viewport::actual_size().zoom_to(1e-9, IMG, PANE, false, None);
        assert_eq!(z.zoom, Zoom::Scale(MIN_SCALE));
    }

    #[test]
    fn zooming_out_until_the_image_fits_recenters_it() {
        let vp = Viewport {
            zoom: Zoom::Scale(2.0),
            center: (0.9, 0.9),
        };
        let z = vp.zoom_to(0.1, IMG, PANE, false, Some((10.0, 10.0)));
        assert_eq!(z.center, (0.5, 0.5));
    }

    #[test]
    fn panning_moves_by_pane_pixels_and_stops_at_the_edges() {
        let vp = Viewport::actual_size();
        let p = vp.pan_by((100.0, 0.0), IMG, PANE, false);
        let v = p.view(IMG, PANE, false);
        assert!(close(v.src.0, 400.0 + 100.0), "{v:?}");
        let far = vp
            .pan_by((1e6, 1e6), IMG, PANE, false)
            .view(IMG, PANE, false);
        assert_eq!((far.src.0, far.src.1), (800.0, 400.0));
        // Nothing to scroll in Fit: it stays Fit.
        assert!(Viewport::fit()
            .pan_by((50.0, 50.0), IMG, PANE, false)
            .is_fit());
    }

    #[test]
    fn tracks_are_recomputed_from_the_geometry_every_time() {
        let vp = Viewport {
            zoom: Zoom::Scale(1.0),
            center: (0.5, 0.5),
        };
        let (x, y) = vp.tracks(IMG, PANE, false);
        assert_eq!((x.content, x.view, x.offset), (1000.0, 200.0, 400.0));
        assert_eq!((y.content, y.view, y.offset), (500.0, 100.0, 200.0));
        assert!(x.scrolls() && y.scrolls());
        // Widen the pane (a divider drag): the track follows, the same
        // viewport still describes the same scale and center.
        let (x2, _) = vp.tracks(IMG, Size::new(600.0, 100.0), false);
        assert_eq!((x2.content, x2.view, x2.offset), (1000.0, 600.0, 200.0));
        // Fit shows no scrollbar.
        let (fx, fy) = Viewport::fit().tracks(IMG, PANE, false);
        assert!(!fx.scrolls() && !fy.scrolls());
        // Dragging the thumb to the end is the last offset.
        assert_eq!(x.offset_for_fraction(1.0), 800.0);
        let moved = vp.scroll_to(true, 800.0, IMG, PANE, false);
        assert_eq!(moved.view(IMG, PANE, false).src.0, 800.0);
    }

    #[test]
    fn sync_uses_one_scale_for_same_sized_images_and_a_relative_one_otherwise() {
        let vp = Viewport {
            zoom: Zoom::Scale(1.0),
            center: (0.25, 0.75),
        };
        let a = vp.view_synced(IMG, IMG, PANE, false);
        let same = vp.view_synced(IMG, IMG, PANE, false);
        assert_eq!(a, same);
        assert!(close(a.scale, 1.0));
        // The other image is half the size: it is drawn at twice the scale,
        // so both cover the same fraction of the pane.
        let small = Size::new(500.0, 250.0);
        let b = vp.view_synced(small, IMG, PANE, false);
        assert!(close(b.scale, 2.0));
        assert!(close(a.src.2 / IMG.w, b.src.2 / small.w));
        assert!(close(a.src.0 / IMG.w, b.src.0 / small.w), "{a:?} {b:?}");
        // Fit fits each on its own.
        let f = Viewport::fit().view_synced(small, IMG, PANE, false);
        assert!(close(f.scale, 0.4));
    }

    #[test]
    fn a_degenerate_geometry_does_not_divide_by_zero() {
        let z = Viewport::fit().view(Size::new(0.0, 0.0), PANE, false);
        assert!(z.scale.is_finite());
        let z = Viewport::actual_size().view(IMG, Size::new(0.0, 0.0), false);
        assert!(z.src.2.is_finite() && z.dst.2.is_finite());
    }
}
