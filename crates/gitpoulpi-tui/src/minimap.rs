//! The optional minimap: a shrunken picture of the whole file beside the
//! vertical scrollbar, the visible region highlighted, a click jumping there
//! (`ui.minimap`, off by default; Preferences, `M` in the diff, and the
//! palette's "Toggle Minimap").
//!
//! One measurement, two renderers. [`sample`] turns the text into a
//! [`Density`] at *pixel* resolution ([`PX_PER_COL`] x [`PX_PER_ROW`] per
//! cell): how much ink each pixel has, and which change (if any) each pixel
//! row carries. Then either
//!
//! - the **pixel** path draws that as a bitmap through the terminal image
//!   protocol (`terminal_image`, the same reserve / finish / write dance the
//!   commit-detail avatar uses -- [`finish_frame`] and [`write_pending`] are
//!   called next to that one's), or
//! - the **braille** fallback pools it 4x4 into the dots of a braille cell
//!   (2x4 dots), when the terminal draws no images. Text only, so it works
//!   everywhere.
//!
//! The vertical mapping is shared, so a click lands on the same line in both:
//! one line is `px_per_line` pixel rows when the file is short (up to
//! [`MAX_PX_PER_LINE`], so a small file is not a thin sliver), else one pixel
//! row stands for `lines_per_px` lines ([`scale`]).

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::Frame;

use crate::avatar::DetailAvatarImage;
use crate::scroll_marks::{look, Mark};
use crate::terminal_image::{Bitmap, Capability};
use crate::theme::th;

/// Cells the minimap is wide.
pub(crate) const COLS: u16 = 10;
/// Narrower than this (inner width), a minimap would starve the text.
pub(crate) const MIN_INNER_WIDTH: u16 = 50;
const PX_PER_COL: usize = 8;
const PX_PER_ROW: usize = 16;

fn tuple_of(c: Color, fallback: (u8, u8, u8)) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => fallback,
    }
}

/// The minimap's ground, the lifted view band and the default ink, as RGB
/// (the bitmap maths needs numbers; a non-RGB theme color falls back to
/// the Dark values).
fn bg() -> (u8, u8, u8) {
    tuple_of(th().minimap.bg, (24, 26, 38))
}

fn view_bg() -> (u8, u8, u8) {
    tuple_of(th().minimap.view, (56, 62, 92))
}

fn ink() -> (u8, u8, u8) {
    tuple_of(th().minimap.ink, (150, 156, 184))
}

/// The minimap's rect inside a view's padded block: the `COLS` columns just
/// left of the border (where the scrollbar is), or `None` when the block is
/// too small to spare them.
pub(crate) fn rect_in(body: Rect) -> Option<Rect> {
    if body.width < MIN_INNER_WIDTH + COLS + 2 || body.height < 5 {
        return None;
    }
    Some(Rect::new(
        body.x + body.width - 1 - COLS,
        body.y + 1,
        COLS,
        body.height - 2,
    ))
}

/// Right padding a view gives its block so its text stops where the minimap
/// starts: `COLS` when the minimap is on and fits, else 0.
pub(crate) fn padding_for(body: Rect, enabled: bool) -> u16 {
    if enabled && rect_in(body).is_some() {
        COLS
    } else {
        0
    }
}

/// The tallest a line is drawn: three pixel rows.
const MAX_PX_PER_LINE: usize = 3;

/// `(pixel rows per line, lines per pixel row)`; at most one is above 1. A
/// file with fewer lines than the minimap has pixel rows is stretched (so the
/// picture is a picture), a longer one is squeezed to fit the whole file.
pub(crate) fn scale(total: usize, rect_height: u16) -> (usize, usize) {
    let px = (rect_height as usize * PX_PER_ROW).max(1);
    if total >= px {
        (1, total.div_ceil(px))
    } else {
        ((px / total.max(1)).clamp(1, MAX_PX_PER_LINE), 1)
    }
}

/// The document line under minimap row `row` (a screen row inside `rect`).
pub(crate) fn line_at(rect: Rect, total: usize, row: u16) -> usize {
    if total == 0 {
        return 0;
    }
    let cell = row
        .saturating_sub(rect.y)
        .min(rect.height.saturating_sub(1)) as usize;
    let px = cell * PX_PER_ROW + PX_PER_ROW / 2;
    let (ppl, lpp) = scale(total, rect.height);
    (px / ppl * lpp).min(total - 1)
}

/// The first line to show so a click on `line` puts it mid-view.
pub(crate) fn top_for(line: usize, total: usize, viewport: usize) -> usize {
    line.saturating_sub(viewport / 2)
        .min(total.saturating_sub(viewport))
}

/// What the picture is drawn from.
#[derive(Debug, Default)]
pub(crate) struct Density {
    width: usize,
    height: usize,
    /// `width * height` ink amounts, 0 (blank) ..= 255.
    ink: Vec<u8>,
    /// Per pixel row, the change it carries.
    tint: Vec<Option<Mark>>,
    /// Per pixel row, an explicit color (the Blame view's age heat); wins
    /// over `tint`. Empty when the picture is not colored that way.
    rgb: Vec<Option<(u8, u8, u8)>>,
    ppl: usize,
    lpp: usize,
}

impl Density {
    fn ink_at(&self, x: usize, y: usize) -> u8 {
        self.ink.get(y * self.width + x).copied().unwrap_or(0)
    }

    /// The color a cell covering pixel rows `ys` is drawn in: the first
    /// explicit color, else the strongest change, else the plain ink.
    fn pixel_rows_color(&self, ys: std::ops::Range<usize>) -> (u8, u8, u8) {
        if let Some(c) = ys.clone().find_map(|y| self.rgb.get(y).copied().flatten()) {
            return c;
        }
        ys.filter_map(|y| self.tint.get(y).copied().flatten())
            .max()
            .map_or(ink(), |m| rgb(look(m).1))
    }

    /// The color pixel row `y` is drawn in: its explicit color, else its
    /// change's, else the plain ink.
    fn row_color(&self, y: usize) -> (u8, u8, u8) {
        if let Some(c) = self.rgb.get(y).copied().flatten() {
            return c;
        }
        self.tint
            .get(y)
            .copied()
            .flatten()
            .map_or(ink(), |m| rgb(look(m).1))
    }
}

/// Measures `total` lines. `line(i)` is line `i`'s text and the change it
/// carries, if any.
pub(crate) fn sample<'a>(
    total: usize,
    rect_height: u16,
    line: impl Fn(usize) -> (Cow<'a, str>, Option<Mark>),
) -> Density {
    sample_core(total, rect_height, |i| {
        let (text, mark) = line(i);
        (text, mark, None)
    })
}

/// [`sample`] with an explicit color per line instead of a change.
pub(crate) fn sample_colored<'a>(
    total: usize,
    rect_height: u16,
    line: impl Fn(usize) -> (Cow<'a, str>, Option<Color>),
) -> Density {
    sample_core(total, rect_height, |i| {
        let (text, color) = line(i);
        (text, None, color.map(|c| tuple_of(c, ink())))
    })
}

type LineLook<'a> = (Cow<'a, str>, Option<Mark>, Option<(u8, u8, u8)>);

fn sample_core<'a>(
    total: usize,
    rect_height: u16,
    line: impl Fn(usize) -> LineLook<'a>,
) -> Density {
    let width = COLS as usize * PX_PER_COL;
    let height = rect_height as usize * PX_PER_ROW;
    let (ppl, lpp) = scale(total, rect_height);
    let mut d = Density {
        width,
        height,
        ink: vec![0; width * height],
        tint: vec![None; height],
        rgb: vec![None; height],
        ppl,
        lpp,
    };
    let mut hits = vec![0u32; width];
    for y in 0..height {
        let first = y / ppl * lpp;
        if first >= total {
            break;
        }
        let last = (first + lpp).min(total);
        hits.iter_mut().for_each(|h| *h = 0);
        let mut tint: Option<Mark> = None;
        let mut color: Option<(u8, u8, u8)> = None;
        for i in first..last {
            let (text, mark, own) = line(i);
            color = color.or(own);
            let mut col = 0usize;
            // Columns are terminal cells, as in the diff: a CJK char inks
            // two, a combining mark none.
            for ch in text.as_ref().chars() {
                if col >= width {
                    break;
                }
                let span = if ch == '\t' {
                    4
                } else {
                    crate::cell_text::char_cells(ch)
                };
                if !ch.is_whitespace() {
                    for h in &mut hits[col..(col + span).min(width)] {
                        *h += 1;
                    }
                }
                col += span;
            }
            tint = match (tint, mark) {
                (Some(Mark::Add), Some(Mark::Del)) | (Some(Mark::Del), Some(Mark::Add)) => {
                    Some(Mark::Modified)
                }
                (t, None) => t,
                (None, m) => m,
                (Some(a), Some(b)) => Some(a.max(b)),
            };
        }
        let n = (last - first) as u32;
        for (x, h) in hits.iter().enumerate() {
            d.ink[y * width + x] = (h * 255 / n).min(255) as u8;
        }
        d.tint[y] = tint;
        d.rgb[y] = color;
    }
    d
}

fn mix(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    (f(a.0, b.0), f(a.1, b.1), f(a.2, b.2))
}

fn rgb(c: Color) -> (u8, u8, u8) {
    tuple_of(c, ink())
}

/// The picture as a bitmap: ink on a dark ground, changed rows in their
/// color, the visible pixel rows `view` lifted.
pub(crate) fn bitmap(d: &Density, view: std::ops::Range<usize>) -> Bitmap {
    let (base, lifted) = (bg(), view_bg());
    let mut bmp = Bitmap::new(d.width as u32, d.height as u32, base);
    for y in 0..d.height {
        let in_view = view.contains(&y);
        let ground = if in_view { lifted } else { base };
        let ink = d.row_color(y);
        for x in 0..d.width {
            let a = d.ink_at(x, y) as f32 / 255.0;
            let c = mix(ground, ink, (a * 1.6).min(1.0) * 0.9);
            bmp.set_pixel(x as u32, y as u32, c);
        }
    }
    bmp
}

/// Braille bits of dot (`i` column 0..2, `j` row 0..4).
fn braille_bit(i: usize, j: usize) -> u32 {
    match (i, j) {
        (0, 0) => 0x01,
        (0, 1) => 0x02,
        (0, 2) => 0x04,
        (1, 0) => 0x08,
        (1, 1) => 0x10,
        (1, 2) => 0x20,
        (0, 3) => 0x40,
        _ => 0x80,
    }
}

/// The braille fallback, painted straight into `buf` over `rect`.
pub(crate) fn paint_braille(
    buf: &mut Buffer,
    rect: Rect,
    d: &Density,
    view: std::ops::Range<usize>,
) {
    let (base, lifted) = (bg(), view_bg());
    for cy in 0..rect.height as usize {
        let y0 = cy * PX_PER_ROW;
        let in_view = view.start < y0 + PX_PER_ROW && y0 < view.end;
        let bg = if in_view { lifted } else { base };
        let fg = d.pixel_rows_color(y0..y0 + PX_PER_ROW);
        for cx in 0..rect.width as usize {
            let mut mask = 0u32;
            for i in 0..2 {
                for j in 0..4 {
                    // A dot pools a 4x4 block of pixels.
                    let (px, py) = (cx * PX_PER_COL + i * 4, y0 + j * 4);
                    let sum: u32 = (0..4)
                        .flat_map(|dy| (0..4).map(move |dx| (dx, dy)))
                        .map(|(dx, dy)| d.ink_at(px + dx, py + dy) as u32)
                        .sum();
                    if sum > 16 * 24 {
                        mask |= braille_bit(i, j);
                    }
                }
            }
            let ch = char::from_u32(0x2800 + mask).unwrap_or(' ');
            let cell = &mut buf[(rect.x + cx as u16, rect.y + cy as u16)];
            cell.set_char(ch);
            cell.set_style(
                Style::default()
                    .fg(Color::Rgb(fg.0, fg.1, fg.2))
                    .bg(Color::Rgb(bg.0, bg.1, bg.2)),
            );
        }
    }
}

thread_local! {
    static IMAGE: RefCell<Option<DetailAvatarImage>> = const { RefCell::new(None) };
}

/// Tells the pixel path what the terminal can draw. Without it (or with no
/// image protocol) every minimap is the braille fallback.
pub(crate) fn init(capability: Capability) {
    IMAGE.with(|i| *i.borrow_mut() = Some(DetailAvatarImage::new(capability)));
}

/// Call once per frame after every popup is drawn (see `DetailAvatarImage`).
pub(crate) fn finish_frame(buf: &Buffer) {
    IMAGE.with(|i| {
        if let Some(img) = i.borrow_mut().as_mut() {
            img.finish_frame(buf);
        }
    });
}

/// Call after `Terminal::draw` returns.
pub(crate) fn write_pending<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
    IMAGE.with(|i| match i.borrow_mut().as_mut() {
        Some(img) => img.write_pending(out),
        None => Ok(()),
    })
}

/// One view's minimap: caches the measurement while the text does not change.
#[derive(Default)]
pub(crate) struct Minimap {
    cache: Option<(u64, usize, u16, Rc<Density>)>,
    /// Keeps whatever `key` was derived from (an address) alive, so a new
    /// text can never be handed the old one's key by the allocator.
    owner: Option<Rc<dyn std::any::Any>>,
}

impl Minimap {
    /// Pins `owner` for as long as it is the text on show.
    pub(crate) fn pin(&mut self, owner: Rc<dyn std::any::Any>) {
        self.owner = Some(owner);
    }

    /// Draws the minimap in `rect`. `key` names the text (it changes when
    /// the text does); `top..top + viewport` is the visible line range.
    pub(crate) fn draw<'a>(
        &mut self,
        frame: &mut Frame,
        rect: Rect,
        key: u64,
        total: usize,
        (top, viewport): (usize, usize),
        line: impl Fn(usize) -> (Cow<'a, str>, Option<Mark>),
    ) {
        self.draw_buf(frame.buffer_mut(), rect, key, total, (top, viewport), line);
    }

    /// [`Minimap::draw`] on a bare buffer (a page painted without a `Frame`,
    /// the review's diff pane).
    pub(crate) fn draw_buf<'a>(
        &mut self,
        buf: &mut Buffer,
        rect: Rect,
        key: u64,
        total: usize,
        (top, viewport): (usize, usize),
        line: impl Fn(usize) -> (Cow<'a, str>, Option<Mark>),
    ) {
        self.draw_with(buf, rect, key, total, (top, viewport), || {
            sample(total, rect.height, line)
        });
    }

    /// [`Minimap::draw`] with a color per line (the Blame view's age heat)
    /// instead of a change.
    pub(crate) fn draw_colored<'a>(
        &mut self,
        frame: &mut Frame,
        rect: Rect,
        key: u64,
        total: usize,
        (top, viewport): (usize, usize),
        line: impl Fn(usize) -> (Cow<'a, str>, Option<Color>),
    ) {
        self.draw_with(
            frame.buffer_mut(),
            rect,
            key,
            total,
            (top, viewport),
            || sample_colored(total, rect.height, line),
        );
    }

    fn draw_with(
        &mut self,
        buf: &mut Buffer,
        rect: Rect,
        key: u64,
        total: usize,
        (top, viewport): (usize, usize),
        measure: impl FnOnce() -> Density,
    ) {
        let fresh = matches!(&self.cache, Some((k, t, h, _)) if *k == key && *t == total && *h == rect.height);
        if !fresh {
            self.cache = Some((key, total, rect.height, Rc::new(measure())));
        }
        let Some((_, _, _, d)) = &self.cache else {
            return;
        };
        let view = top * d.ppl / d.lpp..((top + viewport) * d.ppl).div_ceil(d.lpp);
        let bmp = bitmap(d, view.clone());
        let drawn = IMAGE.with(|i| {
            i.borrow_mut()
                .as_mut()
                .is_some_and(|img| img.render(buf, rect, &bmp))
        });
        if !drawn {
            paint_braille(buf, rect, d, view);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    fn text_lines(n: usize) -> Vec<String> {
        (0..n)
            .map(|i| format!("    let value_{i} = compute({i});"))
            .collect()
    }

    #[test]
    fn a_click_maps_to_the_line_at_that_height_and_the_view_centers_on_it() {
        let rect = Rect::new(0, 1, COLS, 20);
        // 20 cells * 16 = 320 pixel rows: a 100-line file gets 3 per line.
        assert_eq!(scale(100, 20), (3, 1));
        assert_eq!(line_at(rect, 100, rect.y), 2, "px row 8 is in line 2");
        // 3200 lines -> 10 lines per pixel row.
        assert_eq!(scale(3200, 20), (1, 10));
        let mid = line_at(rect, 3200, rect.y + 10);
        assert!((1600..1700).contains(&mid), "{mid}");
        assert_eq!(top_for(mid, 3200, 40), mid - 20);
        assert_eq!(top_for(2, 3200, 40), 0, "near the top it clamps");
        assert_eq!(top_for(3199, 3200, 40), 3160, "near the end it clamps");
    }

    #[test]
    fn sampling_finds_ink_where_the_text_is_and_the_change_color_on_its_rows() {
        let lines = text_lines(10);
        let d = sample(10, 5, |i| {
            (
                Cow::Borrowed(lines[i].as_str()),
                (i == 4).then_some(Mark::Add),
            )
        });
        assert!(d.ink_at(6, 0) > 0, "code has ink");
        assert_eq!(d.ink_at(2, 0), 0, "the indent is blank");
        assert_eq!(d.tint[12], Some(Mark::Add), "line 4 is pixel rows 12..15");
        assert_eq!(d.tint[9], None);
        assert_eq!(d.ink_at(6, 30), 0, "past the last line nothing is drawn");
    }

    #[test]
    fn a_wide_char_inks_two_columns_and_a_combining_mark_none() {
        // "日本" is four cells, then a space, then "x": ink at 0..4 and 5.
        let d = sample(1, 1, |_| (Cow::Borrowed("日本 x"), None));
        for x in 0..4 {
            assert!(d.ink_at(x, 0) > 0, "cell {x} of a wide char has ink");
        }
        assert_eq!(d.ink_at(4, 0), 0, "the space");
        assert!(d.ink_at(5, 0) > 0, "x lands after two cells per wide char");
        // "e" + COMBINING ACUTE is one cell: "y" is at column 1.
        let d = sample(1, 1, |_| (Cow::Borrowed("e\u{301}y"), None));
        assert!(d.ink_at(1, 0) > 0);
        assert_eq!(d.ink_at(2, 0), 0, "the mark takes no column");
    }

    #[test]
    fn the_bitmap_lifts_the_visible_rows_and_tints_the_changed_ones() {
        let lines = text_lines(10);
        let d = sample(10, 5, |i| {
            (
                Cow::Borrowed(lines[i].as_str()),
                (i == 4).then_some(Mark::Del),
            )
        });
        let bmp = bitmap(&d, 0..2);
        assert_eq!((bmp.width, bmp.height), (80, 80));
        assert_eq!(
            bmp.pixel(1, 0),
            Some(view_bg()),
            "visible, blank: lifted ground"
        );
        assert_eq!(bmp.pixel(1, 40), Some(bg()));
        let red = bmp.pixel(6, 12).unwrap();
        assert!(red.0 > red.1, "a deleted line reads red: {red:?}");
    }

    #[test]
    fn the_braille_fallback_lights_dots_for_text_and_highlights_the_view() {
        let lines = text_lines(40);
        let rect = Rect::new(2, 1, COLS, 6);
        let d = sample(40, 6, |i| (Cow::Borrowed(lines[i].as_str()), None));
        let mut terminal = Terminal::new(TestBackend::new(20, 10)).unwrap();
        terminal
            .draw(|f| paint_braille(f.buffer_mut(), rect, &d, 0..PX_PER_ROW))
            .unwrap();
        let buf = terminal.backend().buffer();
        let first = &buf[(rect.x + 1, rect.y)];
        let ch = first.symbol().chars().next().unwrap();
        assert!(
            ('\u{2801}'..='\u{28ff}').contains(&ch),
            "text has dots: {ch:?}"
        );
        assert_eq!(first.bg, Color::Rgb(view_bg().0, view_bg().1, view_bg().2));
        let below = &buf[(rect.x + 1, rect.y + 3)];
        assert_eq!(
            below.bg,
            Color::Rgb(bg().0, bg().1, bg().2),
            "outside the view"
        );
    }

    #[test]
    fn the_rect_needs_room_and_the_padding_follows_it() {
        assert!(rect_in(Rect::new(0, 0, 30, 20)).is_none());
        let body = Rect::new(0, 0, 100, 20);
        let r = rect_in(body).unwrap();
        assert_eq!(
            (r.x + r.width, r.height),
            (99, 18),
            "just inside the border"
        );
        assert_eq!(padding_for(body, true), COLS);
        assert_eq!(padding_for(body, false), 0);
        assert_eq!(padding_for(Rect::new(0, 0, 30, 20), true), 0);
    }
}
