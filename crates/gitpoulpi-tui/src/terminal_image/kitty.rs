//! The **Kitty graphics protocol**, placed with **Unicode placeholders**
//! -- the protocol that gets pictures through herdr.
//!
//! ## Why placeholders, not a placement at the cursor
//!
//! The protocol offers two ways to show an uploaded image. A *direct*
//! placement (`a=p` at the cursor, `c=`/`r=` cells) floats the picture
//! over the grid at a screen position: the terminal owns it, text drawn
//! over it does not remove it, and whoever sent it must delete it again
//! when the cells underneath change. A *virtual* placement (`U=1`) shows
//! nothing by itself; the picture appears wherever the grid holds the
//! placeholder character `U+10EEEE`, with the image id in the cell's
//! foreground color and the cell's row and column inside the picture in
//! two combining diacritics. The picture *is* those cells.
//!
//! Placeholders are chosen, for three reasons:
//!
//! 1. **It is what works in herdr.** herdr runs each pane on its own
//!    terminal emulator (libghostty) and re-emits the pane's images to the
//!    outer terminal. Its issue tracker has direct placements "wiped by
//!    Neovim's redraws" (herdr#494) and a run of fixes aimed specifically
//!    at virtual placements (herdr#136, #732, #1175). yazi -- the
//!    maintainer's proof that images work in his herdr -- uses exactly this:
//!    `a=T,U=1,...` then placeholder cells (`yazi-adapter/src/drivers/
//!    kgp.rs`).
//! 2. **No ghosts, by construction.** A placeholder is a character, so
//!    ratatui's own diff moves it, overwrites it and restores it. A popup
//!    over the graph replaces placeholder cells with its text and the
//!    picture disappears exactly there; the popup closes, ratatui writes
//!    the placeholders back, the picture is back. The iTerm2 path has to
//!    detect all of that itself and re-send the image (see
//!    `super::ImageReservation`); this one has nothing to detect.
//! 3. **Uploads are one-off.** An image is sent once, by id, and any
//!    number of cells can show it afterwards: a Log avatar that scrolls
//!    three rows up costs the placeholder cells, not a re-upload.
//!
//! What it costs: the terminal must implement placeholders. kitty,
//! Ghostty and herdr do; WezTerm draws `U+10EEEE` as tofu, which is one
//! of the reasons bare WezTerm stays on iTerm2 (`super::protocol_for`).
//!
//! ## Lifecycle
//!
//! - Every encoded picture gets a **fresh id** ([`allocate`]). Changing
//!   the picture never re-transmits under an id cells already point at:
//!   the new picture gets a new id, its cells carry the new color (so
//!   ratatui re-emits them), and the old id is deleted after the frame.
//!   Replacing in place is what herdr#1167 reports as blanking.
//! - Dropping a [`KittyImage`] **retires** its id; [`write_retired`],
//!   called after every frame, sends `a=d,d=I` for each so the terminal
//!   frees the pixels. By then the frame has overwritten every cell that
//!   still pointed at it.
//! - [`release_all`] deletes every id this thread ever uploaded, on the
//!   way out. `a=d` alone (delete all visible) does not reach virtual
//!   placements -- the protocol says so explicitly -- so it is by id.
//! - Every command carries `q=2`: no reply ever comes back on stdin,
//!   where it would arrive as keystrokes.
//!
//! Ids are 24 bits (the foreground color holds exactly that), with the
//! top byte taken from the process id so two GitPoulpi instances in two
//! panes of one herdr do not delete each other's pictures.

use std::cell::{Cell as StdCell, RefCell};
use std::collections::HashSet;
use std::fmt::Write as _;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

/// Base64 bytes per escape sequence. The protocol's limit, and a
/// multiple of four so every chunk but the last is whole base64 quanta.
pub(crate) const CHUNK: usize = 4096;

/// The placeholder character. Private use, one cell wide.
pub(crate) const PLACEHOLDER: char = '\u{10EEEE}';

/// The row/column diacritics, in order: the `n`th encodes `n`. Copied
/// from kitty's `rowcolumn-diacritics.txt` (297 entries); a picture taller
/// or wider than that in cells shows its first 297 rows/columns.
const DIACRITICS: [char; 297] = [
    '\u{0305}',
    '\u{030D}',
    '\u{030E}',
    '\u{0310}',
    '\u{0312}',
    '\u{033D}',
    '\u{033E}',
    '\u{033F}',
    '\u{0346}',
    '\u{034A}',
    '\u{034B}',
    '\u{034C}',
    '\u{0350}',
    '\u{0351}',
    '\u{0352}',
    '\u{0357}',
    '\u{035B}',
    '\u{0363}',
    '\u{0364}',
    '\u{0365}',
    '\u{0366}',
    '\u{0367}',
    '\u{0368}',
    '\u{0369}',
    '\u{036A}',
    '\u{036B}',
    '\u{036C}',
    '\u{036D}',
    '\u{036E}',
    '\u{036F}',
    '\u{0483}',
    '\u{0484}',
    '\u{0485}',
    '\u{0486}',
    '\u{0487}',
    '\u{0592}',
    '\u{0593}',
    '\u{0594}',
    '\u{0595}',
    '\u{0597}',
    '\u{0598}',
    '\u{0599}',
    '\u{059C}',
    '\u{059D}',
    '\u{059E}',
    '\u{059F}',
    '\u{05A0}',
    '\u{05A1}',
    '\u{05A8}',
    '\u{05A9}',
    '\u{05AB}',
    '\u{05AC}',
    '\u{05AF}',
    '\u{05C4}',
    '\u{0610}',
    '\u{0611}',
    '\u{0612}',
    '\u{0613}',
    '\u{0614}',
    '\u{0615}',
    '\u{0616}',
    '\u{0617}',
    '\u{0657}',
    '\u{0658}',
    '\u{0659}',
    '\u{065A}',
    '\u{065B}',
    '\u{065D}',
    '\u{065E}',
    '\u{06D6}',
    '\u{06D7}',
    '\u{06D8}',
    '\u{06D9}',
    '\u{06DA}',
    '\u{06DB}',
    '\u{06DC}',
    '\u{06DF}',
    '\u{06E0}',
    '\u{06E1}',
    '\u{06E2}',
    '\u{06E4}',
    '\u{06E7}',
    '\u{06E8}',
    '\u{06EB}',
    '\u{06EC}',
    '\u{0730}',
    '\u{0732}',
    '\u{0733}',
    '\u{0735}',
    '\u{0736}',
    '\u{073A}',
    '\u{073D}',
    '\u{073F}',
    '\u{0740}',
    '\u{0741}',
    '\u{0743}',
    '\u{0745}',
    '\u{0747}',
    '\u{0749}',
    '\u{074A}',
    '\u{07EB}',
    '\u{07EC}',
    '\u{07ED}',
    '\u{07EE}',
    '\u{07EF}',
    '\u{07F0}',
    '\u{07F1}',
    '\u{07F3}',
    '\u{0816}',
    '\u{0817}',
    '\u{0818}',
    '\u{0819}',
    '\u{081B}',
    '\u{081C}',
    '\u{081D}',
    '\u{081E}',
    '\u{081F}',
    '\u{0820}',
    '\u{0821}',
    '\u{0822}',
    '\u{0823}',
    '\u{0825}',
    '\u{0826}',
    '\u{0827}',
    '\u{0829}',
    '\u{082A}',
    '\u{082B}',
    '\u{082C}',
    '\u{082D}',
    '\u{0951}',
    '\u{0953}',
    '\u{0954}',
    '\u{0F82}',
    '\u{0F83}',
    '\u{0F86}',
    '\u{0F87}',
    '\u{135D}',
    '\u{135E}',
    '\u{135F}',
    '\u{17DD}',
    '\u{193A}',
    '\u{1A17}',
    '\u{1A75}',
    '\u{1A76}',
    '\u{1A77}',
    '\u{1A78}',
    '\u{1A79}',
    '\u{1A7A}',
    '\u{1A7B}',
    '\u{1A7C}',
    '\u{1B6B}',
    '\u{1B6D}',
    '\u{1B6E}',
    '\u{1B6F}',
    '\u{1B70}',
    '\u{1B71}',
    '\u{1B72}',
    '\u{1B73}',
    '\u{1CD0}',
    '\u{1CD1}',
    '\u{1CD2}',
    '\u{1CDA}',
    '\u{1CDB}',
    '\u{1CE0}',
    '\u{1DC0}',
    '\u{1DC1}',
    '\u{1DC3}',
    '\u{1DC4}',
    '\u{1DC5}',
    '\u{1DC6}',
    '\u{1DC7}',
    '\u{1DC8}',
    '\u{1DC9}',
    '\u{1DCB}',
    '\u{1DCC}',
    '\u{1DD1}',
    '\u{1DD2}',
    '\u{1DD3}',
    '\u{1DD4}',
    '\u{1DD5}',
    '\u{1DD6}',
    '\u{1DD7}',
    '\u{1DD8}',
    '\u{1DD9}',
    '\u{1DDA}',
    '\u{1DDB}',
    '\u{1DDC}',
    '\u{1DDD}',
    '\u{1DDE}',
    '\u{1DDF}',
    '\u{1DE0}',
    '\u{1DE1}',
    '\u{1DE2}',
    '\u{1DE3}',
    '\u{1DE4}',
    '\u{1DE5}',
    '\u{1DE6}',
    '\u{1DFE}',
    '\u{20D0}',
    '\u{20D1}',
    '\u{20D4}',
    '\u{20D5}',
    '\u{20D6}',
    '\u{20D7}',
    '\u{20DB}',
    '\u{20DC}',
    '\u{20E1}',
    '\u{20E7}',
    '\u{20E9}',
    '\u{20F0}',
    '\u{2CEF}',
    '\u{2CF0}',
    '\u{2CF1}',
    '\u{2DE0}',
    '\u{2DE1}',
    '\u{2DE2}',
    '\u{2DE3}',
    '\u{2DE4}',
    '\u{2DE5}',
    '\u{2DE6}',
    '\u{2DE7}',
    '\u{2DE8}',
    '\u{2DE9}',
    '\u{2DEA}',
    '\u{2DEB}',
    '\u{2DEC}',
    '\u{2DED}',
    '\u{2DEE}',
    '\u{2DEF}',
    '\u{2DF0}',
    '\u{2DF1}',
    '\u{2DF2}',
    '\u{2DF3}',
    '\u{2DF4}',
    '\u{2DF5}',
    '\u{2DF6}',
    '\u{2DF7}',
    '\u{2DF8}',
    '\u{2DF9}',
    '\u{2DFA}',
    '\u{2DFB}',
    '\u{2DFC}',
    '\u{2DFD}',
    '\u{2DFE}',
    '\u{2DFF}',
    '\u{A66F}',
    '\u{A67C}',
    '\u{A67D}',
    '\u{A6F0}',
    '\u{A6F1}',
    '\u{A8E0}',
    '\u{A8E1}',
    '\u{A8E2}',
    '\u{A8E3}',
    '\u{A8E4}',
    '\u{A8E5}',
    '\u{A8E6}',
    '\u{A8E7}',
    '\u{A8E8}',
    '\u{A8E9}',
    '\u{A8EA}',
    '\u{A8EB}',
    '\u{A8EC}',
    '\u{A8ED}',
    '\u{A8EE}',
    '\u{A8EF}',
    '\u{A8F0}',
    '\u{A8F1}',
    '\u{AAB0}',
    '\u{AAB2}',
    '\u{AAB3}',
    '\u{AAB7}',
    '\u{AAB8}',
    '\u{AABE}',
    '\u{AABF}',
    '\u{AAC1}',
    '\u{FE20}',
    '\u{FE21}',
    '\u{FE22}',
    '\u{FE23}',
    '\u{FE24}',
    '\u{FE25}',
    '\u{FE26}',
    '\u{10A0F}',
    '\u{10A38}',
    '\u{1D185}',
    '\u{1D186}',
    '\u{1D187}',
    '\u{1D188}',
    '\u{1D189}',
    '\u{1D1AA}',
    '\u{1D1AB}',
    '\u{1D1AC}',
    '\u{1D1AD}',
    '\u{1D242}',
    '\u{1D243}',
    '\u{1D244}',
];

/// The ids this thread has uploaded and not yet deleted, and the ones
/// waiting for their delete to go out.
///
/// Thread-local, not global: every picture is encoded, drawn and dropped
/// on the TUI's draw thread, and each unit test runs on a thread of its
/// own - a global queue would let one test's writes carry another's
/// deletes.
struct Registry {
    next: u32,
    live: HashSet<u32>,
    retired: Vec<u32>,
}

thread_local! {
    static REGISTRY: RefCell<Registry> = RefCell::new(Registry {
        next: 0,
        live: HashSet::new(),
        retired: Vec::new(),
    });
}

/// The top byte of every id this process allocates: never zero (an id of
/// zero means "none" in the protocol).
fn id_base() -> u32 {
    ((std::process::id() % 255) + 1) << 16
}

/// A fresh image id, not currently live on this thread.
pub(crate) fn allocate() -> u32 {
    REGISTRY.with(|r| {
        let mut r = r.borrow_mut();
        let base = id_base();
        loop {
            r.next = (r.next + 1) & 0xFFFF;
            let id = base | r.next;
            if r.live.insert(id) {
                return id;
            }
        }
    })
}

fn retire(id: u32) {
    // `try_with`: a picture dropped while the thread is being torn down
    // has nowhere to be recorded, and must not panic about it.
    let _ = REGISTRY.try_with(|r| {
        let mut r = r.borrow_mut();
        if r.live.remove(&id) {
            r.retired.push(id);
        }
    });
}

/// The delete command for one image, pixels included (`d=I`).
pub(crate) fn delete_command(id: u32) -> String {
    format!("\x1b_Ga=d,d=I,i={id},q=2\x1b\\")
}

/// Sends the deletes for every picture dropped since the last call. Call
/// after each frame, once ratatui has overwritten their cells.
pub(crate) fn write_retired<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
    let retired = REGISTRY
        .try_with(|r| std::mem::take(&mut r.borrow_mut().retired))
        .unwrap_or_default();
    if retired.is_empty() {
        return Ok(());
    }
    for id in retired {
        out.write_all(delete_command(id).as_bytes())?;
    }
    out.flush()
}

/// Deletes every picture this thread uploaded, live or retired. On the
/// way out of the TUI; harmless on a terminal that never saw one.
pub(crate) fn release_all<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
    let ids: Vec<u32> = REGISTRY
        .try_with(|r| {
            let mut r = r.borrow_mut();
            let mut ids = std::mem::take(&mut r.retired);
            ids.extend(r.live.iter().copied());
            ids
        })
        .unwrap_or_default();
    if ids.is_empty() {
        return Ok(());
    }
    for id in ids {
        out.write_all(delete_command(id).as_bytes())?;
    }
    out.flush()
}

/// The transmit-and-place command for a PNG: `a=T` (transmit) with
/// `U=1` (a virtual placement for placeholders) of `cols` x `rows` cells,
/// base64 split into [`CHUNK`]-byte escapes (`m=1` on all but the last).
pub(crate) fn upload_command(id: u32, png: &[u8], cols: u16, rows: u16) -> String {
    let payload = super::base64(png);
    let chunks: Vec<&str> = if payload.is_empty() {
        vec![""]
    } else {
        // Base64 is ASCII, so byte offsets are char boundaries.
        payload
            .as_bytes()
            .chunks(CHUNK)
            .map(|c| std::str::from_utf8(c).unwrap_or_default())
            .collect()
    };
    let mut out = String::with_capacity(payload.len() + chunks.len() * 24 + 64);
    let last = chunks.len() - 1;
    for (i, chunk) in chunks.iter().enumerate() {
        let more = u8::from(i < last);
        if i == 0 {
            let _ = write!(
                out,
                "\x1b_Ga=T,U=1,f=100,t=d,i={id},c={cols},r={rows},C=1,q=2,m={more};{chunk}\x1b\\"
            );
        } else {
            let _ = write!(out, "\x1b_Gm={more},q=2;{chunk}\x1b\\");
        }
    }
    out
}

/// The foreground color that names image `id` to the terminal.
pub(crate) fn id_color(id: u32) -> Color {
    Color::Rgb((id >> 16) as u8, (id >> 8) as u8, id as u8)
}

/// The placeholder cell for `row`, `col` inside a picture: `U+10EEEE`,
/// then the row diacritic, then the column one. `None` past the table.
pub(crate) fn placeholder_symbol(row: usize, col: usize) -> Option<String> {
    let (r, c) = (DIACRITICS.get(row)?, DIACRITICS.get(col)?);
    let mut s = String::with_capacity(12);
    s.push(PLACEHOLDER);
    s.push(*r);
    s.push(*c);
    Some(s)
}

/// How many cells of `area` no longer hold a placeholder: something (a
/// popup, a full-pane page) was drawn over the picture.
pub(crate) fn overdrawn_cells(buf: &Buffer, area: Rect) -> usize {
    let area = area.intersection(buf.area);
    let mut n = 0;
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if !buf[(x, y)].symbol().starts_with(PLACEHOLDER) {
                n += 1;
            }
        }
    }
    n
}

/// Writes image `id`'s placeholder cells over `area`. Every diacritic is
/// explicit, never inherited from the cell to the left: a popup that
/// covers the left half of a row must not change which part of the
/// picture the right half shows.
pub(crate) fn paint_placeholders(id: u32, area: Rect, buf: &mut Buffer) {
    let area = area.intersection(buf.area);
    let color = id_color(id);
    for (row, y) in (area.top()..area.bottom()).enumerate() {
        for (col, x) in (area.left()..area.right()).enumerate() {
            let cell = &mut buf[(x, y)];
            // The picture is transparent where nothing is drawn, so the
            // cell's own background (the table's selection or hover
            // highlight) is what shows through: keep it, reset the rest.
            let bg = cell.bg;
            cell.reset();
            cell.set_bg(bg);
            if let Some(symbol) = placeholder_symbol(row, col) {
                cell.set_symbol(&symbol);
                cell.set_fg(color);
            }
        }
    }
}

/// One uploaded picture: its id, the command that uploads it, and the
/// screen it was last sent to.
pub(crate) struct KittyImage {
    pub(crate) id: u32,
    pub(crate) upload: String,
    /// The frame size the upload last went out for. A resize makes
    /// ratatui clear the screen, and whether a clear takes virtual
    /// placements with it is up to the terminal (kitty keeps them; herdr's
    /// libghostty is not documented either way), so it is sent again.
    sent_on: StdCell<Option<Rect>>,
}

impl KittyImage {
    pub(crate) fn new(png: &[u8], cols: u16, rows: u16) -> KittyImage {
        let id = allocate();
        KittyImage {
            id,
            upload: upload_command(id, png, cols, rows),
            sent_on: StdCell::new(None),
        }
    }

    /// Whether the upload has to go out for a frame of size `screen`;
    /// marks it as sent when it does, so a second slot showing the same
    /// picture in the same frame does not send it twice.
    pub(crate) fn needs_upload(&self, screen: Rect) -> bool {
        if self.sent_on.get() == Some(screen) {
            return false;
        }
        self.sent_on.set(Some(screen));
        true
    }

    /// The write failed: whatever reached the terminal is unknown.
    pub(crate) fn forget_upload(&self) {
        self.sent_on.set(None);
    }
}

impl Drop for KittyImage {
    fn drop(&mut self) {
        retire(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The picture is transparent where nothing is drawn, so the cell's
    /// own background (the table's selection or hover highlight) is what
    /// the terminal shows through it: placeholders must not reset it.
    #[test]
    fn placeholders_keep_the_cell_background_the_table_painted() {
        let area = Rect::new(0, 0, 3, 2);
        let mut buf = Buffer::empty(area);
        let hl = ratatui::style::Color::Rgb(52, 58, 90);
        buf.set_style(area, ratatui::style::Style::default().bg(hl));
        paint_placeholders(7, area, &mut buf);
        for y in 0..2 {
            for x in 0..3 {
                assert_eq!(buf[(x, y)].bg, hl, "({x},{y})");
            }
        }
    }

    #[test]
    fn a_small_image_is_one_complete_apc_escape() {
        let seq = upload_command(0x01_0203, b"hello", 3, 2);
        assert_eq!(
            seq,
            "\x1b_Ga=T,U=1,f=100,t=d,i=66051,c=3,r=2,C=1,q=2,m=0;aGVsbG8=\x1b\\"
        );
    }

    /// Falsified: chunked at 4095 (the first escape then carried 4095
    /// base64 bytes, red); restored.
    #[test]
    fn a_large_image_is_chunked_at_4096_base64_bytes() {
        // 3 * 4096 raw bytes is exactly 4 * 4096 base64 bytes: four
        // chunks, the last one full.
        let png = vec![0u8; 3 * 4096];
        let seq = upload_command(7, &png, 1, 1);
        let escapes: Vec<&str> = seq.split("\x1b\\").filter(|s| !s.is_empty()).collect();
        assert_eq!(escapes.len(), 4, "{} escapes", escapes.len());
        for (i, escape) in escapes.iter().enumerate() {
            assert!(escape.starts_with("\x1b_G"), "escape {i} is not APC G");
            let (control, data) = escape[3..].split_once(';').unwrap();
            assert_eq!(data.len(), CHUNK, "chunk {i}");
            let more = if i < 3 { "m=1" } else { "m=0" };
            assert!(control.contains(more), "chunk {i}: {control}");
            if i == 0 {
                assert!(control.starts_with("a=T,U=1,f=100,"), "{control}");
                assert!(control.contains("i=7"), "{control}");
            } else {
                // Only `m` (and `q`) on continuation chunks.
                assert_eq!(control, format!("{more},q=2"));
            }
        }
        let rejoined: String = escapes
            .iter()
            .map(|e| e.split_once(';').unwrap().1)
            .collect();
        assert_eq!(rejoined, super::super::base64(&png));
    }

    #[test]
    fn the_id_is_the_foreground_color_byte_for_byte() {
        assert_eq!(id_color(0x12_3456), Color::Rgb(0x12, 0x34, 0x56));
    }

    #[test]
    fn a_placeholder_cell_names_its_row_then_its_column() {
        assert_eq!(
            placeholder_symbol(0, 0).unwrap(),
            "\u{10EEEE}\u{0305}\u{0305}"
        );
        // The protocol's own example: row 1 column 0 is 030D 0305.
        assert_eq!(
            placeholder_symbol(1, 0).unwrap(),
            "\u{10EEEE}\u{030D}\u{0305}"
        );
        assert_eq!(
            placeholder_symbol(0, 2).unwrap(),
            "\u{10EEEE}\u{0305}\u{030E}"
        );
        assert_eq!(DIACRITICS.len(), 297);
        assert_eq!(DIACRITICS[296], '\u{1D244}');
        assert!(placeholder_symbol(297, 0).is_none());
    }

    #[test]
    fn a_placeholder_is_one_cell_wide() {
        // ratatui measures a cell's symbol with this same width, and a
        // placeholder it took for zero or two columns would shift every
        // cell after it.
        for (r, c) in [(0, 0), (5, 9), (296, 296)] {
            let span = ratatui::text::Span::raw(placeholder_symbol(r, c).unwrap());
            assert_eq!(span.width(), 1);
        }
    }

    #[test]
    fn ids_are_24_bit_nonzero_and_unique_while_live() {
        let a = KittyImage::new(b"x", 1, 1);
        let b = KittyImage::new(b"x", 1, 1);
        assert_ne!(a.id, b.id);
        for id in [a.id, b.id] {
            assert!(id > 0 && id < (1 << 24), "{id:#x}");
        }
    }

    /// Falsified: emptied `Drop` (no delete ever went out, red); restored.
    #[test]
    fn a_dropped_picture_is_deleted_after_the_frame_and_only_once() {
        let image = KittyImage::new(b"x", 1, 1);
        let id = image.id;
        let mut out = Vec::new();
        write_retired(&mut out).unwrap();
        assert!(out.is_empty(), "a live picture was deleted");
        drop(image);
        write_retired(&mut out).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), delete_command(id));
        let mut again = Vec::new();
        write_retired(&mut again).unwrap();
        assert!(again.is_empty());
    }

    #[test]
    fn leaving_deletes_every_picture_still_uploaded() {
        let kept = KittyImage::new(b"x", 1, 1);
        let dropped = KittyImage::new(b"y", 1, 1);
        let dropped_id = dropped.id;
        drop(dropped);
        let mut out = Vec::new();
        release_all(&mut out).unwrap();
        let out = String::from_utf8(out).unwrap();
        assert!(out.contains(&format!("i={},", kept.id)), "{out:?}");
        assert!(out.contains(&format!("i={dropped_id},")), "{out:?}");
        assert!(out.contains("a=d,d=I,"));
    }

    #[test]
    fn an_upload_goes_out_once_per_screen_size() {
        let image = KittyImage::new(b"x", 1, 1);
        let screen = Rect::new(0, 0, 80, 24);
        assert!(image.needs_upload(screen));
        assert!(!image.needs_upload(screen), "sent twice for one screen");
        assert!(image.needs_upload(Rect::new(0, 0, 100, 30)), "resize");
        image.forget_upload();
        assert!(image.needs_upload(Rect::new(0, 0, 100, 30)));
    }
}
