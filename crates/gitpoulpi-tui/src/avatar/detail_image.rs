//! The commit-detail pane's own avatar picture: **one** bitmap, not a
//! whole column's worth.
//!
//! Same mechanism as `graph::inline_image::GraphImage` -- reserve the
//! cells in the ratatui buffer during the draw closure
//! ([`DetailAvatarImage::render`]), see what a popup drew over them once
//! every popup is drawn ([`DetailAvatarImage::finish_frame`]), write the
//! picture (and whatever covered it) after `Terminal::draw` returns
//! ([`DetailAvatarImage::write_pending`]) -- generalised down to a single
//! picture instead of a per-row composite, because the commit-detail
//! pane shows exactly one author at a time. `graph::inline_image` is not
//! reused directly: its cache key is built from `RasterRow`s, which this
//! caller has none of, and threading a second, unrelated key type through
//! one struct would have made *that* module read like two features
//! sharing a badly-fitting shape rather than one well-scoped one. What is
//! shared is `crate::terminal_image` -- the actual encoder, capability
//! detection and buffer-cell trick -- exactly as that module's header
//! asked for.

use ratatui::buffer::{Buffer, Cell};
use ratatui::layout::Rect;

use crate::terminal_image::{self, Bitmap, Capability, CellSize, Encoded};

/// Rows the commit-detail avatar spans: the author block's two lines.
pub(crate) const DETAIL_ROWS: u16 = 2;

/// The commit-detail avatar's box in cells, `(cols, rows)`, chosen so the
/// picture is *square in pixels*: a cell is about twice as tall as wide
/// (9x21 measured, 8x16 typical), so two rows need about 4-5 columns.
/// Computed from the terminal's own cell size rather than hard-coded, so
/// the proportions hold on any font.
pub(crate) fn detail_size(cell: CellSize) -> (u16, u16) {
    let want_w = DETAIL_ROWS as u32 * cell.height;
    let cols = ((want_w + cell.width / 2) / cell.width.max(1)).clamp(2, 8);
    (cols as u16, DETAIL_ROWS)
}

pub(crate) struct DetailAvatarImage {
    capability: Capability,
    cached_key: Option<u64>,
    cached: Option<Encoded>,
    drawn: Option<(Rect, u64)>,
    on_screen: Option<OnScreen>,
    pending: Option<Pending>,
    /// Where the picture was reserved last frame, to erase it when this
    /// frame does not draw it.
    shown: Option<Rect>,
    hide: Option<terminal_image::Hide>,
}

struct Pending {
    anchor: (u16, u16),
    /// Cells written before the picture (transparent iTerm2 only).
    under: Vec<(u16, u16, Cell)>,
    over: Vec<(u16, u16, Cell)>,
}

#[derive(Debug, PartialEq, Eq)]
struct OnScreen {
    screen: Rect,
    area: Rect,
    key: u64,
    covered: Vec<(u16, u16)>,
    /// The area's cells, compared by value for a transparent picture,
    /// whose blank cells ratatui rewrites (and so wipes the picture).
    cells: Vec<Cell>,
}

impl DetailAvatarImage {
    pub(crate) fn new(capability: Capability) -> DetailAvatarImage {
        DetailAvatarImage {
            capability,
            cached_key: None,
            cached: None,
            drawn: None,
            on_screen: None,
            pending: None,
            shown: None,
            hide: None,
        }
    }

    /// Withdraws this frame's reservation: the picture is erased if it was
    /// on screen, and nothing is sent. For a caller that sees a popup over
    /// it in `finish_frame`.
    pub(crate) fn cancel(&mut self) {
        self.drawn = None;
    }

    /// This terminal draws pictures at all.
    pub(crate) fn can_draw(&self) -> bool {
        self.capability.draws_images()
    }

    fn encode_for(&mut self, bitmap: &Bitmap, area: Rect) -> Option<u64> {
        if !self.capability.draws_images() || area.is_empty() {
            return None;
        }
        let key = key_of(bitmap, area);
        if self.cached_key != Some(key) {
            // Sent as decoded, never downscaled here: the decode box is
            // already supersampled for the terminal's cell size, and
            // `image_decode::downscale` is RGB-only (it would turn a transparent
            // picture's corners opaque black).
            self.cached = terminal_image::encode_image(
                self.capability.protocol,
                bitmap,
                area.width,
                area.height,
            );
            self.cached_key = Some(key);
        }
        self.cached.is_some().then_some(key)
    }

    /// Reserves `area` for `bitmap`. `false` when this terminal cannot
    /// draw it - the caller's initials badge is what the user sees, and
    /// nothing here has touched the buffer.
    pub(crate) fn render(&mut self, buf: &mut Buffer, area: Rect, bitmap: &Bitmap) -> bool {
        let Some(key) = self.encode_for(bitmap, area) else {
            return false;
        };
        if let Some(encoded) = &self.cached {
            encoded.reserve(area, buf);
        }
        self.drawn = Some((area, key));
        true
    }

    /// Call once, after every popup has been drawn.
    pub(crate) fn finish_frame(&mut self, buf: &Buffer) {
        self.pending = None;
        self.hide = None;
        let Some((area, key)) = self.drawn.take() else {
            self.on_screen = None;
            if let Some(area) = self.shown.take() {
                let mut hide = terminal_image::Hide::default();
                hide.add(buf, area, self.cached.as_ref().and_then(Encoded::kitty_id));
                if let Some(encoded) = &self.cached {
                    encoded.forget_upload();
                }
                self.hide = Some(hide);
            }
            return;
        };
        self.shown = Some(area);
        if let Some(encoded) = self.cached.as_ref().filter(|e| e.places_itself()) {
            // Kitty: the placeholder cells are the picture, and ratatui
            // has just written them. Only the upload may be owed.
            self.on_screen = None;
            if encoded.needs_upload(buf.area) {
                self.pending = Some(Pending {
                    anchor: (area.x, area.y),
                    under: Vec::new(),
                    over: Vec::new(),
                });
            }
            return;
        }
        let blends = self.cached.as_ref().is_some_and(Encoded::blends);
        let mut covered = Vec::new();
        let mut over = Vec::new();
        let mut cells = Vec::new();
        let mut under = Vec::new();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = &buf[(x, y)];
                if blends {
                    // A transparent picture's cells are real blanks
                    // (`BlankReservation`, not `skip`): they go out
                    // *before* the picture, and only what a popup drew
                    // over them is written on top. Writing them after
                    // would wipe the picture.
                    cells.push(cell.clone());
                    under.push((x, y, cell.clone()));
                    if !cell.modifier.contains(terminal_image::BLANK_RESERVED) {
                        covered.push((x, y));
                        over.push((x, y, cell.clone()));
                    }
                } else if cell.diff_option != ratatui::buffer::CellDiffOption::Skip {
                    covered.push((x, y));
                    over.push((x, y, cell.clone()));
                }
            }
        }
        let now = OnScreen {
            screen: buf.area,
            area,
            key,
            covered,
            cells,
        };
        if self.on_screen.as_ref() != Some(&now) {
            self.pending = Some(Pending {
                anchor: (area.x, area.y),
                under,
                over,
            });
        }
        self.on_screen = Some(now);
    }

    /// Call after `Terminal::draw` returns.
    pub(crate) fn write_pending<W: std::io::Write>(&mut self, out: &mut W) -> std::io::Result<()> {
        if let Some(hide) = self.hide.take() {
            terminal_image::write_hide(out, &hide)?;
        }
        let (
            Some(Pending {
                anchor,
                under,
                over,
            }),
            Some(sequence),
        ) = (
            self.pending.take(),
            self.cached.as_ref().map(Encoded::sequence),
        )
        else {
            return Ok(());
        };
        let written =
            terminal_image::write_images_layered(out, &under, &[(anchor, sequence)], &over);
        if written.is_err() {
            self.on_screen = None;
            if let Some(encoded) = &self.cached {
                encoded.forget_upload();
            }
        }
        written
    }
}

fn key_of(bitmap: &Bitmap, area: Rect) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    area.width.hash(&mut hasher);
    area.height.hash(&mut hasher);
    bitmap.width.hash(&mut hasher);
    bitmap.height.hash(&mut hasher);
    bitmap.data.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_image::{CellSize, Protocol};
    use crate::wide_edge::Clear;
    use ratatui::widgets::{Paragraph, Widget};

    const SCREEN: Rect = Rect::new(0, 0, 24, 10);
    const AVATAR: Rect = Rect::new(2, 1, 2, 1);

    fn image() -> DetailAvatarImage {
        DetailAvatarImage::new(Capability {
            protocol: Protocol::Iterm2,
            cell: CellSize::FALLBACK,
        })
    }

    fn bitmap(fill: (u8, u8, u8)) -> Bitmap {
        Bitmap::new(4, 4, fill)
    }

    fn frame(
        img: &mut DetailAvatarImage,
        bitmap: &Bitmap,
        overlay: Option<Rect>,
    ) -> (Buffer, String) {
        let mut buf = Buffer::empty(SCREEN);
        assert!(img.render(&mut buf, AVATAR, bitmap));
        if let Some(popup) = overlay {
            Clear.render(popup, &mut buf);
            Paragraph::new("x").render(popup, &mut buf);
        }
        img.finish_frame(&buf);
        let mut out = Vec::new();
        img.write_pending(&mut out).unwrap();
        (buf, String::from_utf8(out).unwrap())
    }

    fn sends_the_image(written: &str) -> bool {
        written.contains("]1337;File=")
    }

    /// The box is square in *pixels*, whatever the font: 2 rows of a 1:2.3
    /// cell take about as many pixels across as down.
    #[test]
    fn the_detail_avatar_box_is_square_in_pixels_for_any_cell_size() {
        for (w, h) in [(9u32, 21u32), (8, 16), (10, 20), (7, 14), (12, 24)] {
            let (cols, rows) = detail_size(CellSize {
                width: w,
                height: h,
            });
            let (px_w, px_h) = (u32::from(cols) * w, u32::from(rows) * h);
            let ratio = px_w as f32 / px_h as f32;
            assert!(
                (0.85..=1.2).contains(&ratio),
                "{w}x{h} cell: {cols}x{rows} cells is {px_w}x{px_h}px"
            );
        }
        // 2 rows by 4 columns on an 8x16 cell.
        assert_eq!(
            detail_size(CellSize {
                width: 8,
                height: 16
            }),
            (4, 2)
        );
    }

    #[test]
    fn the_first_frame_sends_the_image() {
        let (_, written) = frame(&mut image(), &bitmap((10, 20, 30)), None);
        assert!(sends_the_image(&written), "{written:?}");
    }

    /// Falsified: skipped the `on_screen` comparison and always cleared
    /// `pending` to `None` regardless of `key`, watched a *changed*
    /// avatar stop sending on the second frame, restored the comparison.
    #[test]
    fn an_unchanged_avatar_costs_nothing_on_the_wire() {
        let mut img = image();
        let _ = frame(&mut img, &bitmap((10, 20, 30)), None);
        let (_, written) = frame(&mut img, &bitmap((10, 20, 30)), None);
        assert_eq!(written, "");
    }

    #[test]
    fn a_different_authors_avatar_is_sent_again() {
        let mut img = image();
        let _ = frame(&mut img, &bitmap((10, 20, 30)), None);
        let (_, written) = frame(&mut img, &bitmap((200, 20, 30)), None);
        assert!(sends_the_image(&written));
    }

    #[test]
    fn a_terminal_that_cannot_draw_images_is_never_reserved() {
        let mut img = DetailAvatarImage::new(Capability {
            protocol: Protocol::None,
            cell: CellSize::FALLBACK,
        });
        let mut buf = Buffer::empty(SCREEN);
        assert!(!img.render(&mut buf, AVATAR, &bitmap((1, 2, 3))));
        // Untouched -- not reserved (`skip`) -- so the caller's initials
        // badge renders into these cells through the ordinary ratatui
        // path instead.
        assert!(buf[(2, 1)].diff_option != ratatui::buffer::CellDiffOption::Skip);
    }

    #[test]
    fn closing_a_popup_over_the_avatar_sends_the_image_again() {
        let popup = Some(Rect::new(1, 0, 10, 2));
        let mut img = image();
        let _ = frame(&mut img, &bitmap((10, 20, 30)), None);
        let _ = frame(&mut img, &bitmap((10, 20, 30)), popup);
        let (_, written) = frame(&mut img, &bitmap((10, 20, 30)), None);
        assert!(sends_the_image(&written));
    }

    fn kitty() -> DetailAvatarImage {
        DetailAvatarImage::new(Capability {
            protocol: Protocol::Kitty,
            cell: CellSize::FALLBACK,
        })
    }

    fn unbase64(text: &str) -> Vec<u8> {
        const A: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let (mut out, mut acc, mut bits) = (Vec::new(), 0u32, 0);
        for c in text.chars().filter(|c| *c != '=') {
            acc = (acc << 6) | A.find(c).unwrap() as u32;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                out.push((acc >> bits) as u8);
                acc &= (1 << bits) - 1;
            }
        }
        out
    }

    /// The picture goes out at the decode size, alpha intact.
    #[test]
    fn the_picture_is_sent_as_decoded_with_its_transparency() {
        let mut img = kitty();
        let mut buf = Buffer::empty(SCREEN);
        let big = Bitmap::transparent(90, 90);
        assert!(img.render(&mut buf, Rect::new(2, 1, 5, 2), &big));
        let encoded = img.cached.as_ref().unwrap().sequence().to_string();
        let b64 = encoded
            .split(';')
            .nth(1)
            .unwrap()
            .split('\x1b')
            .next()
            .unwrap();
        let png = unbase64(b64);
        let w = u32::from_be_bytes(png[16..20].try_into().unwrap());
        let h = u32::from_be_bytes(png[20..24].try_into().unwrap());
        assert_eq!((w, h), (90, 90));
    }

    /// WezTerm (transparent iTerm2): the area's blank cells must reach the
    /// terminal before the picture, never after it - written after, they
    /// erase it and the box stays blank. Red before the fix.
    #[test]
    fn a_transparent_picture_is_the_last_thing_written() {
        let mut img = DetailAvatarImage::new(Capability {
            protocol: Protocol::Iterm2Alpha,
            cell: CellSize::FALLBACK,
        });
        let (_, written) = frame(&mut img, &bitmap((10, 20, 30)), None);
        let end = written.rfind('\x07').expect("picture sequence");
        let tail = &written[end + 1..];
        assert!(
            !tail.contains(' '),
            "cells written over the picture: {tail:?}"
        );
        let (_, again) = frame(&mut img, &bitmap((10, 20, 30)), None);
        assert_eq!(again, "");
    }

    /// Sixel: no id and no delete, so like the transparent iTerm2 picture
    /// the cells go first and the picture is last, and an unchanged avatar
    /// is not sent again.
    #[test]
    fn a_sixel_picture_is_the_last_thing_written_and_sent_once() {
        let mut img = DetailAvatarImage::new(Capability {
            protocol: Protocol::Sixel,
            cell: CellSize::FALLBACK,
        });
        let (_, written) = frame(&mut img, &bitmap((10, 20, 30)), None);
        assert!(written.contains("\x1bP0;1;0q"), "{written:?}");
        let end = written.rfind("\x1b\\").expect("sixel terminator");
        assert!(!written[end + 2..].contains(' '), "{written:?}");
        let (_, again) = frame(&mut img, &bitmap((10, 20, 30)), None);
        assert_eq!(again, "");
        // A popup over one cell: the picture goes again, and the popup's
        // cell is written after it, so no picture shows through it.
        let (_, covered) = frame(&mut img, &bitmap((10, 20, 30)), Some(Rect::new(3, 1, 1, 1)));
        let end = covered.rfind("\x1b\\").expect("sixel terminator");
        assert!(covered[end..].contains('x'), "{covered:?}");
    }

    /// What a Kitty terminal holds after the writes: ids uploaded and not
    /// deleted since.
    fn apply(terminal: &mut std::collections::HashSet<u32>, written: &str) {
        for esc in written.split("\x1b_G").skip(1) {
            let ctl = esc.split(';').next().unwrap_or("");
            let id = ctl
                .split(',')
                .find_map(|kv| kv.strip_prefix("i="))
                .and_then(|v| v.parse::<u32>().ok());
            let Some(id) = id else { continue };
            if ctl.starts_with("a=T") {
                terminal.insert(id);
            } else if ctl.starts_with("a=d") {
                terminal.remove(&id);
            }
        }
    }

    /// Every picture the screen names must exist on the terminal.
    #[test]
    fn a_placeholder_on_screen_always_has_its_picture_uploaded() {
        let authors = [
            bitmap((10, 20, 30)),
            bitmap((200, 20, 30)),
            bitmap((5, 90, 30)),
        ];
        let mut img = kitty();
        let mut terminal = std::collections::HashSet::new();
        let steps = [
            Some(0),
            Some(0),
            Some(1),
            None,
            Some(1),
            Some(0),
            None,
            None,
            Some(0),
            Some(2),
            Some(2),
            None,
            Some(2),
            Some(1),
        ];
        for (n, step) in steps.into_iter().enumerate() {
            let mut buf = Buffer::empty(SCREEN);
            if let Some(a) = step {
                assert!(img.render(&mut buf, Rect::new(2, 1, 4, 2), &authors[a]));
            }
            img.finish_frame(&buf);
            let mut out = Vec::new();
            img.write_pending(&mut out).unwrap();
            crate::terminal_image::write_retired(&mut out).unwrap();
            apply(&mut terminal, &String::from_utf8(out).unwrap());
            if step.is_some() {
                let color = buf[(2, 1)].fg;
                let ids: Vec<u32> = terminal.iter().copied().collect();
                assert!(
                    ids.iter()
                        .any(|id| crate::terminal_image::kitty::id_color(*id) == color),
                    "step {n}: placeholder {color:?} but terminal holds {ids:?}"
                );
            }
        }
    }
}
