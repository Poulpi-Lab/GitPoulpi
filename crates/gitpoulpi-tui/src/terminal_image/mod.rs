//! Putting **pixels** on a terminal, for anything in this TUI that needs
//! them.
//!
//! Three jobs, none of them specific to any one feature: work out whether
//! this terminal can display an image and how big one of its cells is;
//! turn an RGB bitmap into the escape sequence that protocol wants; and
//! park that sequence in the ratatui buffer so the backend emits it at
//! exactly the right cell.
//!
//! Its first caller is the commit graph ([`crate::graph`]), whose Graph
//! column is drawn as a picture because no arrangement of characters can
//! join a round commit marker to its lane -- see
//! `crate::graph::raster`'s module header. It is deliberately **not**
//! written into that feature: author avatars want the same capability
//! detection, the same cell metrics and the same encoder for a 16x16
//! Gravatar, and a layer that has to be re-derived for the second caller
//! is in the wrong shape.
//!
//! ## Why iTerm2's protocol, and why hand-rolled
//!
//! The maintainer's daily driver is **WezTerm on Windows**. WezTerm implements
//! the iTerm2 inline-image protocol and Sixel, and *not* the Kitty
//! protocol. Of those, iTerm2 is the one WezTerm's maintainer and the
//! `ratatui-image` compatibility matrix both call bug-free; WezTerm's
//! Sixel has known clearing glitches, and Kitty's Unicode-placeholder
//! fallback paints tofu here. It is also the only one with
//! `doNotMoveCursor=1` -- WezTerm's own extension -- which is what lets an
//! image sit on any row, including the last, without scrolling the
//! alternate screen out from under the TUI.
//!
//! `ratatui-image` was investigated first and does all of this well. It is
//! **not** used, for two concrete reasons rather than taste:
//!
//! 1. The versions that support `ratatui` 0.29 need rustc 1.86, and this
//!    workspace declares `rust-version = "1.80"`. Cargo resolves the
//!    constraint down to `ratatui-image` 3.0, which targets a `ratatui`
//!    several releases older than ours. Bumping the workspace MSRV to suit
//!    one column of one pane is not this change's call to make.
//! 2. Its bulk is *decoding and resizing* arbitrary images through the
//!    `image` crate. The graph's bitmap is generated at exactly the pixel
//!    size it will occupy, so none of that applies, and `image` plus
//!    `rand` would be carried for the ~150 lines below. (An avatar *is*
//!    decoded -- a PNG or JPEG off the network -- so if avatars land, that
//!    is the moment to revisit the trade, and the seam is
//!    [`Capability`] plus [`encode`], not this whole module.)
//!
//! What is borrowed from it, because it is the right design, is the
//! buffer-cell trick in [`ImageReservation`].
//!
//! ## And the Kitty graphics protocol, for herdr
//!
//! herdr does not forward `OSC 1337`, but with its `kitty_graphics`
//! setting on it implements the Kitty graphics protocol itself and
//! re-emits the pictures to the outer terminal -- which is how yazi shows
//! images there. So [`Protocol::Kitty`] exists, placed with Unicode
//! placeholders; the whole of it, and why placeholders rather than a
//! placement at the cursor, is in [`kitty`]. kitty and Ghostty get it
//! natively. **Bare WezTerm stays on iTerm2**: it is the proven path
//! there, `doNotMoveCursor=1` is WezTerm's own extension, and WezTerm
//! draws the placeholder character as tofu.
//!
//! ## Capability detection, and the thing it must never do
//!
//! **Detection never reads from stdin.** [`cell_size`] carries the full
//! story of why -- a query on the wire needs a read with a deadline, a
//! blocked read cannot be canceled, and the first version of this module
//! therefore swallowed every keystroke the pty test suite sent. What is
//! left is:
//!
//! - `GITPOULPI_TERMINAL_IMAGE=off` disables the whole path -- the escape
//!   hatch for a terminal that lies, and what the pty harness sets,
//!   because a pty inherits this process' environment and would otherwise
//!   believe it is the WezTerm the suite was launched from.
//! - A terminal is a candidate only if it identifies itself, through
//!   `WEZTERM_EXECUTABLE` or `TERM_PROGRAM`, as one known to implement the
//!   protocol. Deny by default: an unknown terminal gets the text
//!   rendering, which is correct everywhere, rather than a gamble that
//!   paints base64 across the pane when it loses.
//! - **And nothing may be sitting between this process and that
//!   terminal.** See [`inside_a_multiplexer`]; this is the half the first
//!   version got wrong, at the user's expense.
//! - `ui.graph_renderer` (`auto` | `image` | `text`) overrides the
//!   verdict in either direction, so a user is never stuck with a wrong
//!   guess, and `ui.image_protocol` (`auto` | `iterm2` | `kitty` |
//!   `none`) picks the protocol outright.
//! - **herdr** is the one multiplexer that can be let through: when its
//!   own `config.toml` turns `kitty_graphics` on and the outer terminal is
//!   one that draws Kitty images (herdr strips `KITTY_WINDOW_ID` from its
//!   panes, so kitty is recognized there by the `KITTY_PID` /
//!   `KITTY_INSTALLATION_DIR` / `KITTY_LISTEN_ON` leftovers, in this
//!   branch only), the verdict is [`Protocol::Kitty`]. That
//!   is a file read, once, at start-up -- not a query. The protocol's own
//!   query (`ESC _G a=q ...`) was considered and rejected for the reason
//!   [`cell_size`] gives: its reply comes back on stdin, and there is no
//!   read with a deadline here that does not eat a keystroke (on Windows
//!   crossterm reads console input records, where the reply is not even
//!   bytes).
//! - **herdr over SSH** (`herdr --remote`, or `ssh` then `herdr`) is text
//!   under `auto`. Measured on herdr 0.9.3: the remote pane carries
//!   `SSH_CONNECTION` and herdr's own `TERM`; the picture is drawn or not
//!   by the *local* client (its `kitty_graphics`, its terminal), neither of
//!   which this process can read. With the remote config saying `true` and
//!   kitty leftovers in the env but the local client's switch off, the old
//!   verdict was Kitty and the Graph column came out **blank**. The upload
//!   itself is fine remotely: it is always direct data (`t=d`, chunked),
//!   never a file or shared-memory reference. `ui.graph_renderer = "image"`
//!   / `ui.image_protocol = "kitty"` opt in; `"text"` is the workaround.
//! - The cell size comes from `TIOCGWINSZ` where that exists, and from a
//!   measured default otherwise. Getting it wrong costs resolution, not
//!   placement -- the image is sized in cells. It is also **supersampled**
//!   (see [`supersample`]), which is what keeps a wrong estimate from
//!   showing as pixelation when the user zooms.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

use gitpoulpi_config::ImageProtocol;

mod cell_query;
mod detect;
pub(crate) mod passthrough;

/// Whether a terminal multiplexer (tmux, screen, zellij, herdr) sits between
/// this process and the terminal: what does not pass through it (an image,
/// an `OSC 22` pointer shape) is not sent.
pub(crate) fn inside_multiplexer() -> bool {
    detect::inside_a_multiplexer(&|key: &str| std::env::var(key).ok())
}
pub(crate) mod kitty;
#[cfg(test)]
mod kitty_detect_tests;
pub(crate) mod sixel;
#[cfg(test)]
pub(crate) mod vt_sim;

pub(crate) use detect::detect;
#[cfg(test)]
use detect::*;
pub(crate) use kitty::{release_all, write_retired};

/// The terminal's cell size in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CellSize {
    pub width: u32,
    pub height: u32,
}

impl CellSize {
    /// What to use when the terminal answers no cell-size query. Cascadia
    /// Mono at the maintainer's size measures 9x21; this is deliberately that
    /// rather than a round number, because a wrong aspect ratio shows up
    /// as an oval commit marker, and a round one is the whole point.
    pub(crate) const FALLBACK: CellSize = CellSize {
        width: 9,
        height: 21,
    };

    /// Clamps a queried size into something drawable. A terminal answering
    /// `0` - or something absurd - must not make a rasteriser divide by
    /// zero or allocate a gigabyte.
    pub(crate) fn sanitised(width: u32, height: u32) -> CellSize {
        CellSize {
            width: width.clamp(4, 64),
            height: height.clamp(6, 128),
        }
    }
}

/// How many bitmap pixels are drawn per assumed cell pixel, per axis.
///
/// **This is the answer to zoom.** The image is sized in *cells*
/// (`width=N;height=M;preserveAspectRatio=0`), so the terminal scales
/// whatever bitmap it is handed into the cell box. The bitmap is
/// rasterised at the *assumed* cell size - and on Windows that assumption
/// is always [`CellSize::FALLBACK`], because `crossterm`'s `window_size()`
/// has no Windows implementation and returns an error. Press `Ctrl +` and
/// the cells get physically bigger while the bitmap does not, so the
/// terminal upscales a 9x21 tile into a 14x32 box and the disc turns into
/// stair steps. The user saw exactly that.
///
/// Rasterising at 2x makes the bitmap larger than any plausible cell, so
/// the terminal *down*samples - which is filtering, not blocking, and
/// stays smooth. It is robust against **any** wrong estimate, including
/// one this code can never make (a font the user changed after start-up),
/// and it needs no new I/O at all, which is what rules out the obvious
/// alternative: a `CSI 16 t` query has to read stdin, and a blocked read
/// here cannot be canceled (see [`cell_size`]).
///
/// 2 rather than 3, measured (release, full pane, 2026-09-23): scale 1
/// costs 3.1 ms and 22 KB per changed frame, scale 2 costs 6.8 ms and
/// 39 KB, scale 3 costs 12.9 ms and 56 KB. 2x is smooth at every zoom
/// tried and 3x looked identical for twice the price. The table and what
/// made those numbers affordable are in
/// `a_full_pane_of_graph_rasterises_and_encodes_inside_a_frame_budget`.
/// `GITPOULPI_TERMINAL_SUPERSAMPLE=1` turns it off, for a machine where
/// the CPU matters more than the edges.
pub(crate) fn supersample() -> u32 {
    supersample_from(
        std::env::var("GITPOULPI_TERMINAL_SUPERSAMPLE")
            .ok()
            .as_deref(),
    )
}

fn supersample_from(raw: Option<&str>) -> u32 {
    raw.and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or(2)
        .clamp(1, 4)
}

/// What this terminal can do with a picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Protocol {
    /// iTerm2 inline images (`OSC 1337 ; File=`), for terminals that are
    /// not known to composite the PNG's alpha: the picture carries its
    /// own opaque backdrop.
    Iterm2,
    /// The same escape, for a terminal verified to composite the PNG's
    /// alpha over the cell background (WezTerm; see [`blends_png_alpha`]):
    /// the picture is transparent where nothing is drawn, and the cells
    /// under it keep their background ([`BlankReservation`]).
    Iterm2Alpha,
    /// The Kitty graphics protocol, placed with Unicode placeholders
    /// ([`kitty`]).
    Kitty,
    /// Sixel (`DCS q`): Windows Terminal, foot, Konsole, mlterm and a VS
    /// Code with `terminal.integrated.enableImages`. No ids and no alpha;
    /// handled like [`Protocol::Iterm2Alpha`] (blank reserved cells, text
    /// rewritten under the picture), see [`sixel`].
    Sixel,
    /// No image protocol. Every caller must have a text rendering for
    /// this case; it is what the pty test harness gets, and what every
    /// terminal this code has not been taught about gets.
    None,
}

/// The result of detection: which protocol, and the cell size any
/// rasteriser has to match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Capability {
    pub protocol: Protocol,
    pub cell: CellSize,
}

impl Capability {
    pub(crate) fn draws_images(self) -> bool {
        self.protocol != Protocol::None
    }

    /// Whether the Graph column is drawn as a picture: with every image
    /// protocol, sixel included (a transparent picture over blank cells,
    /// like WezTerm's; see [`sixel`] for how it is kept cheap).
    pub(crate) fn draws_graph(self) -> bool {
        self.draws_images()
    }
}

/// The cell size, **without ever reading from stdin**.
///
/// The obvious implementation writes `CSI 16 t` and reads the reply back.
/// It was written, and it broke every pty test in the suite, in a way
/// worth recording because it is not obvious and it would be re-invented:
///
/// - reading stdin needs a deadline, and there is no portable
///   `read-with-timeout` here (no `libc::poll`, no `WaitForSingleObject`),
///   so the read has to go on a helper thread;
/// - a helper thread blocked in `read` **cannot be stopped**. It has
///   already consumed the next byte by the time it notices the channel is
///   gone - so every terminal that does not answer loses the user's next
///   keystroke, and one that answers late loses one anyway;
/// - a pty inherits the parent's environment. Under the test harness
///   `WEZTERM_EXECUTABLE` is set but the actual terminal is a ConPTY that
///   answers nothing, so the thread sat on stdin and swallowed every
///   keystroke the four `pty_ui` tests sent. All four timed out.
///
/// So the query is deliberately not made. What is used instead:
///
/// 1. `GITPOULPI_TERMINAL_CELL_PX=9x21`, when set; on Windows under sixel,
///    the size [`measure_cell_px`] asked the terminal for (a bounded wait
///    on the console handle with no helper thread, so the objection above
///    does not apply to it);
/// 2. `crossterm::terminal::window_size()`, which is `TIOCGWINSZ` on Unix
///    and costs no round trip (Windows has no equivalent and returns an
///    error, which is fine);
/// 3. [`CellSize::FALLBACK`].
///
/// **Being wrong here is cheap**, which is what makes this trade
/// reasonable rather than a shortcut. The image is placed and sized in
/// *cells* (`width=N;height=M` with `preserveAspectRatio=0`), so the
/// terminal scales the bitmap into exactly the cell box whatever its pixel
/// dimensions are. A wrong cell size changes the resolution the graph is
/// drawn at, and nothing about where it lands or how much of the pane it
/// covers.
pub(crate) fn cell_size() -> CellSize {
    if let Some(forced) = forced_cell_size() {
        return forced;
    }
    if let Some(asked) = cell_query::measured() {
        return asked;
    }
    if let Ok(size) = crossterm::terminal::window_size() {
        if size.width > 0 && size.height > 0 && size.columns > 0 && size.rows > 0 {
            return CellSize::sanitised(
                u32::from(size.width) / u32::from(size.columns),
                u32::from(size.height) / u32::from(size.rows),
            );
        }
    }
    CellSize::FALLBACK
}

/// Windows + sixel only: asks the terminal for its cell size in pixels
/// (`CSI 16 t`, bounded, see [`cell_query`]) so [`cell_size`] is right
/// from now on; `true` when the size changed or became known. Call from
/// the thread that reads input, never while another reader is active. A
/// no-op when the size is forced by `GITPOULPI_TERMINAL_CELL_PX`.
pub(crate) fn measure_cell_px(protocol: Protocol) -> bool {
    if protocol != Protocol::Sixel || forced_cell_size().is_some() {
        return false;
    }
    let before = cell_query::measured();
    cell_query::measure() && cell_query::measured() != before
}

/// `GITPOULPI_TERMINAL_CELL_PX=9x21` - an override for a terminal whose
/// metrics cannot be read, and the only way a test can pin the geometry
/// without a terminal at all.
fn forced_cell_size() -> Option<CellSize> {
    let spec = std::env::var("GITPOULPI_TERMINAL_CELL_PX").ok()?;
    let (w, h) = spec.split_once('x')?;
    Some(CellSize::sanitised(
        w.trim().parse().ok()?,
        h.trim().parse().ok()?,
    ))
}

/// An 8-bit RGBA bitmap, row-major, four bytes per pixel, **straight**
/// (non-premultiplied) alpha.
///
/// Straight alpha is what PNG stores and what the terminals' compositors
/// expect, and it is what makes anti-aliasing fringe-free: the edge pixel
/// of a lane keeps the *pure* lane color and only its alpha falls with
/// coverage, so it blends toward whatever the terminal really has behind
/// the image (a theme, a translucent window, the selection band) instead
/// of toward a color guessed in advance.
///
/// Opaque bitmaps (decoded avatars, the minimap) are simply bitmaps whose
/// alpha is 255 everywhere ([`Bitmap::new`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Bitmap {
    /// A bitmap of one flat, **opaque** color.
    ///
    /// Filled by **doubling**, not pixel by pixel: one pixel is written,
    /// then the buffer copies itself onto its own end until it is full, so
    /// the work is `log2(n)` memcpys instead of `n` four-byte pushes. At
    /// the supersampled sizes this module produces - a full pane is
    /// 270x2520 - the naive loop was a measurable slice of every changed
    /// frame, and a changed frame is every scroll step.
    pub(crate) fn new(width: u32, height: u32, fill: (u8, u8, u8)) -> Bitmap {
        Bitmap::filled(width, height, [fill.0, fill.1, fill.2, 255])
    }

    /// A bitmap with nothing drawn: every pixel fully transparent. Its
    /// color channels are zero, which is irrelevant at alpha 0 and
    /// compresses best.
    pub(crate) fn transparent(width: u32, height: u32) -> Bitmap {
        Bitmap::filled(width, height, [0, 0, 0, 0])
    }

    fn filled(width: u32, height: u32, px: [u8; 4]) -> Bitmap {
        let total = (width as usize) * (height as usize) * 4;
        let mut data = Vec::with_capacity(total);
        if total >= 4 {
            data.extend_from_slice(&px);
            while data.len() < total {
                let take = (total - data.len()).min(data.len());
                data.extend_from_within(..take);
            }
        }
        Bitmap {
            width,
            height,
            data,
        }
    }

    /// An opaque bitmap from packed RGB bytes (what the PNG/JPEG decoders
    /// produce). `rgb` must hold `width * height * 3` bytes.
    pub(crate) fn from_rgb(width: u32, height: u32, rgb: &[u8]) -> Bitmap {
        let mut data = Vec::with_capacity(rgb.len() / 3 * 4);
        for p in rgb.as_chunks::<3>().0 {
            data.extend_from_slice(&[p[0], p[1], p[2], 255]);
        }
        Bitmap {
            width,
            height,
            data,
        }
    }

    fn index(&self, x: u32, y: u32) -> usize {
        ((y as usize) * (self.width as usize) + x as usize) * 4
    }

    /// The color at `(x, y)` ignoring alpha, or `None` outside the
    /// bitmap. Used by `image_decode::downscale` and by the
    /// rasteriser's connectivity proofs.
    pub(crate) fn pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8)> {
        let (r, g, b, _) = self.rgba(x, y)?;
        Some((r, g, b))
    }

    /// The straight-alpha pixel at `(x, y)`, or `None` outside.
    pub(crate) fn rgba(&self, x: u32, y: u32) -> Option<(u8, u8, u8, u8)> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = self.index(x, y);
        Some((
            self.data[i],
            self.data[i + 1],
            self.data[i + 2],
            self.data[i + 3],
        ))
    }

    /// Writes a single opaque pixel outright (`blend` at full coverage).
    pub(crate) fn set_pixel(&mut self, x: u32, y: u32, color: (u8, u8, u8)) {
        self.blend(x, y, color, 1.0);
    }

    /// Composites `color` over the pixel at `coverage` opacity, with the
    /// Porter-Duff "over" operator on straight alpha:
    ///
    /// `a = src_a + dst_a * (1 - src_a)`, and the color is the
    /// alpha-weighted mean of source and destination. Over a transparent
    /// pixel that is the pure source color at `src_a` (no fringe); over
    /// an opaque pixel it is the plain lerp the opaque version did.
    pub(crate) fn blend(&mut self, x: u32, y: u32, color: (u8, u8, u8), coverage: f32) {
        if x >= self.width || y >= self.height || coverage <= 0.0 {
            return;
        }
        let a = coverage.min(1.0);
        let i = self.index(x, y);
        // The common case by a wide margin -- the interior of every
        // stroke and every disc -- and it needs no arithmetic at all.
        if a >= 1.0 {
            self.data[i] = color.0;
            self.data[i + 1] = color.1;
            self.data[i + 2] = color.2;
            self.data[i + 3] = 255;
            return;
        }
        let dst_a = f32::from(self.data[i + 3]) / 255.0;
        let out_a = a + dst_a * (1.0 - a);
        if out_a <= 0.0 {
            return;
        }
        for (k, c) in [color.0, color.1, color.2].into_iter().enumerate() {
            let d = f32::from(self.data[i + k]);
            let v = (f32::from(c) * a + d * dst_a * (1.0 - a)) / out_a;
            self.data[i + k] = v.round().clamp(0.0, 255.0) as u8;
        }
        self.data[i + 3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
    }

    /// Lays `color` at `alpha` over `[x0, x1) x [y0, y1)` (Porter-Duff
    /// over, so transparent stays faintly colored and opaque stays opaque).
    pub(crate) fn fill_rect_alpha(
        &mut self,
        x0: u32,
        y0: u32,
        x1: u32,
        y1: u32,
        color: (u8, u8, u8),
        alpha: f32,
    ) {
        for y in y0..y1.min(self.height) {
            for x in x0..x1.min(self.width) {
                self.blend(x, y, color, alpha);
            }
        }
    }

    /// Fills a horizontal band from column `x0` to the right edge, opaque.
    pub(crate) fn band(&mut self, x0: u32, y0: u32, y1: u32, color: (u8, u8, u8)) {
        for y in y0..y1.min(self.height) {
            for x in x0.min(self.width)..self.width {
                let i = self.index(x, y);
                self.data[i..i + 4].copy_from_slice(&[color.0, color.1, color.2, 255]);
            }
        }
    }
}

/// A bitmap, wrapped in the escape sequence `protocol` wants, sized to
/// `cols` x `rows` terminal cells.
///
/// `width`/`height` are given in **cells** (a bare number means cells in
/// this protocol), `preserveAspectRatio=0` so the terminal does not
/// letterbox something that is already exactly the right shape, and
/// `doNotMoveCursor=1` so placing the image never scrolls the alternate
/// screen - which it otherwise does the moment the image reaches the last
/// row, taking the whole TUI with it.
///
/// For [`Protocol::Kitty`] the sequence is the upload only; the picture
/// is shown by the placeholder cells [`Encoded::reserve`] writes, and
/// `cols` x `rows` is the virtual placement's size.
pub(crate) fn encode_image(
    protocol: Protocol,
    bitmap: &Bitmap,
    cols: u16,
    rows: u16,
) -> Option<Encoded> {
    match protocol {
        Protocol::None => None,
        Protocol::Iterm2 | Protocol::Iterm2Alpha => {
            let png = encode_png(bitmap)?;
            let sequence = format!(
                "\x1b]1337;File=inline=1;size={};width={};height={};preserveAspectRatio=0;doNotMoveCursor=1:{}\x07",
                png.len(),
                cols,
                rows,
                base64(&png)
            );
            Some(if protocol == Protocol::Iterm2Alpha {
                Encoded::Iterm2Alpha(sequence)
            } else {
                Encoded::Iterm2(sequence)
            })
        }
        Protocol::Kitty => {
            let png = encode_png(bitmap)?;
            Some(Encoded::Kitty(kitty::KittyImage::new(&png, cols, rows)))
        }
        Protocol::Sixel => {
            // Sixel has no "fit to N cells": the pixels are the size.
            let cell = cell_size();
            sixel::encode(
                bitmap,
                u32::from(cols) * cell.width,
                u32::from(rows) * cell.height,
            )
            .map(Encoded::Sixel)
        }
    }
}

/// [`encode_image`]'s escape sequence alone, for tests that measure or
/// inspect the bytes.
#[cfg(test)]
pub(crate) fn encode(protocol: Protocol, bitmap: &Bitmap, cols: u16, rows: u16) -> Option<String> {
    encode_image(protocol, bitmap, cols, rows).map(|e| e.sequence().to_string())
}

/// A picture ready for the wire, in the protocol it was encoded for.
///
/// The two protocols put a picture on screen in opposite ways, and this
/// type is the one place the callers see that difference:
///
/// - **iTerm2**: the sequence *is* the picture, written after the frame
///   at the area's top-left; the area's cells are reserved (`skip`) so
///   ratatui leaves them alone, and whatever a popup drew there has to be
///   written again over it. The callers track that coverage.
/// - **Kitty**: the sequence only uploads the picture; the area's cells
///   become placeholder characters in the ratatui buffer, which ratatui's
///   diff emits, moves and restores like any text. After the frame only
///   the upload goes out, once per screen size
///   ([`Encoded::needs_upload`]); coverage needs no tracking at all
///   ([`Encoded::places_itself`]).
pub(crate) enum Encoded {
    Iterm2(String),
    /// iTerm2 escape, transparent picture ([`Protocol::Iterm2Alpha`]).
    Iterm2Alpha(String),
    Kitty(kitty::KittyImage),
    /// A sixel picture ([`Protocol::Sixel`]): transparent where undrawn.
    Sixel(String),
}

impl Encoded {
    /// What [`write_image`] sends for this picture.
    pub(crate) fn sequence(&self) -> &str {
        match self {
            Encoded::Iterm2(sequence)
            | Encoded::Iterm2Alpha(sequence)
            | Encoded::Sixel(sequence) => sequence,
            Encoded::Kitty(image) => &image.upload,
        }
    }

    /// Kitty: the cells themselves carry the picture.
    pub(crate) fn places_itself(&self) -> bool {
        matches!(self, Encoded::Kitty(_))
    }

    /// iTerm2 with a transparent picture: the cells under it are real
    /// (blank, row background kept) and the picture is laid over them
    /// after the frame, so a change to any of them must re-send it.
    pub(crate) fn blends(&self) -> bool {
        matches!(self, Encoded::Iterm2Alpha(_) | Encoded::Sixel(_))
    }

    /// Kitty only: the id the terminal stores this picture under.
    pub(crate) fn kitty_id(&self) -> Option<u32> {
        match self {
            Encoded::Kitty(image) => Some(image.id),
            _ => None,
        }
    }

    /// Whether something drawn this frame covers at least half of the
    /// picture at `area` (Settings, a dialog): such a picture is hidden
    /// rather than laid under it. Kitty detects it from the placeholders
    /// that are gone, the others from the cells that stopped being `skip`.
    pub(crate) fn mostly_overdrawn(&self, buf: &Buffer, area: Rect) -> bool {
        let area = area.intersection(buf.area);
        let total = usize::from(area.width) * usize::from(area.height);
        let covered = if self.places_itself() {
            kitty::overdrawn_cells(buf, area)
        } else {
            let blends = self.blends();
            let mut n = 0;
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    let cell = &buf[(x, y)];
                    // A blending picture's cells are real blanks, told from
                    // a page's by the mark `BlankReservation` leaves.
                    let held = if blends {
                        cell.modifier.contains(BLANK_RESERVED)
                    } else {
                        cell.diff_option == ratatui::buffer::CellDiffOption::Skip
                    };
                    if !held {
                        n += 1;
                    }
                }
            }
            n
        };
        total > 0 && covered * 2 >= total
    }

    /// Claims `area` of the buffer for the picture: `skip` cells for
    /// iTerm2 ([`ImageReservation`]), placeholder cells for Kitty.
    pub(crate) fn reserve(&self, area: Rect, buf: &mut Buffer) {
        match self {
            Encoded::Iterm2(_) => ImageReservation.render(area, buf),
            Encoded::Iterm2Alpha(_) | Encoded::Sixel(_) => BlankReservation.render(area, buf),
            Encoded::Kitty(image) => kitty::paint_placeholders(image.id, area, buf),
        }
    }

    /// Kitty only: whether the upload has to go out with a frame of size
    /// `screen` (see [`kitty::KittyImage::needs_upload`]). Always `false`
    /// for iTerm2, whose re-sends are decided from what covers it.
    pub(crate) fn needs_upload(&self, screen: Rect) -> bool {
        match self {
            Encoded::Iterm2(_) | Encoded::Iterm2Alpha(_) | Encoded::Sixel(_) => false,
            Encoded::Kitty(image) => image.needs_upload(screen),
        }
    }

    /// A write failed: send the upload again next time.
    pub(crate) fn forget_upload(&self) {
        if let Encoded::Kitty(image) = self {
            image.forget_upload();
        }
    }
}

/// A bitmap as PNG bytes (the shortcut icons use it too).
pub(crate) fn png_bytes(bitmap: &Bitmap) -> Option<Vec<u8>> {
    encode_png(bitmap)
}

fn encode_png(bitmap: &Bitmap) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, bitmap.width, bitmap.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        // Large flat areas of two or three colors, so the cheap level
        // gets within a few per cent of the expensive one at a fraction
        // of the per-change cost.
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&bitmap.data).ok()?;
    }
    Some(out)
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding (RFC 4648). Twenty lines rather than a
/// dependency, and pinned by `base64_matches_rfc_4648_test_vectors`.
pub(crate) fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            B64[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// The widget that **reserves** an image's cells in the ratatui buffer.
///
/// Every cell of the area is blanked and marked `skip`, so ratatui never
/// emits anything there and nothing else in the TUI can paint over the
/// picture by accident. The picture itself is **not** in the buffer: it is
/// written after the frame by [`write_image`].
///
/// Why the picture is not the symbol of an anchor cell, left to ratatui's
/// diff:
///
/// - `Buffer::diff` measures a symbol's **display width** to know how many
///   following cells a wide character has overwritten, and a base64 string
///   is tens of thousands of "columns" wide. Every non-skip cell after the
///   anchor would be re-emitted **on every frame**.
/// - A popup drawn over the column (a tooltip, a menu, the palette) writes
///   characters into cells the image owns, and the terminal replaces the
///   picture there. When the popup closes those cells go back to `skip`
///   and an unchanged anchor re-sends nothing, so the popup's text would
///   stay printed over the graph. Re-sending needs to know what covered
///   the column, which only the whole frame knows.
pub(crate) struct ImageReservation;

impl Widget for ImageReservation {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = &mut buf[(x, y)];
                cell.reset();
                cell.set_diff_option(ratatui::buffer::CellDiffOption::Skip);
            }
        }
    }
}

/// The reservation for a transparent iTerm2 picture: blank cells that keep
/// the background the table gave them (selection, hover), so that is what
/// shows through the picture's transparent pixels. Not `skip`: ratatui
/// writes them, which also wipes whatever picture or text was there.
pub(crate) struct BlankReservation;

/// An otherwise unused modifier bit [`BlankReservation`] sets on its cells,
/// and that `Clear` (hence any popup or full-screen page) drops. The
/// picture's own cells are blank, so a page that is blank there too (Settings
/// under its last section) is indistinguishable by value from "nothing drew
/// here": without a mark, the picture would be judged uncovered and stay
/// under the page. ratatui's backend ignores bits it has no attribute for,
/// so the mark never reaches the terminal.
pub(crate) const BLANK_RESERVED: ratatui::style::Modifier =
    ratatui::style::Modifier::from_bits_retain(1 << 15);

impl Widget for BlankReservation {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let cell = &mut buf[(x, y)];
                let bg = cell.bg;
                cell.reset();
                cell.set_symbol(" ");
                cell.set_bg(bg);
                cell.modifier = BLANK_RESERVED;
            }
        }
    }
}

/// A picture whose screen is gone (Settings, the Diff view, a management
/// tab replaced the view it belonged to): what has to reach the terminal
/// so nothing of it stays visible. ratatui's diff alone is not enough --
/// a cell the new screen leaves blank equals the blank the transparent
/// picture sat on, so ratatui does not rewrite it and the terminal keeps
/// the picture there. So the area is rewritten cell by cell from the
/// finished frame, and a Kitty picture is deleted outright.
#[derive(Default)]
pub(crate) struct Hide {
    /// Kitty image ids to delete (`a=d,d=I`).
    pub(crate) kitty_ids: Vec<u32>,
    /// The cells to write again, taken from the finished frame.
    pub(crate) cells: Vec<(u16, u16, ratatui::buffer::Cell)>,
}

impl Hide {
    /// Adds `area` of `buf` (the frame just finished) and, for Kitty, the
    /// picture's id.
    pub(crate) fn add(&mut self, buf: &Buffer, area: Rect, id: Option<u32>) {
        self.kitty_ids.extend(id);
        let area = area.intersection(buf.area);
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                if buf[(x, y)].diff_option != ratatui::buffer::CellDiffOption::Skip {
                    self.cells.push(painted_cell(buf, area, x, y));
                }
            }
        }
    }
}

/// Sends a [`Hide`]: the Kitty deletes, then the cells again.
pub(crate) fn write_hide<W: std::io::Write>(out: &mut W, hide: &Hide) -> std::io::Result<()> {
    for id in &hide.kitty_ids {
        out.write_all(passthrough::passthrough(kitty::delete_command(*id)).as_bytes())?;
    }
    write_images_layered(out, &[], &[], &hide.cells)
}

/// What the finished frame does to every cell after the popups
/// (`draw_frame`: the theme's own background and text color for a cell
/// that has none, then `ui.color_depth`), for a cell the image layer
/// writes itself. Those cells are cloned inside the draw closure, before
/// that pass, so written as they are a `Reset` background reaches the
/// terminal as its own default one: on a light theme, a dark rectangle
/// over the popup or page that covers the Graph column.
pub(crate) fn as_painted(cell: &ratatui::buffer::Cell) -> ratatui::buffer::Cell {
    let mut cell = cell.clone();
    paint_cell(&mut cell);
    crate::color_depth::apply_cell(&mut cell);
    cell
}

/// The theme's background and text color for a cell with none (a no-op
/// on Dark, whose `ui.bg` is `Reset`). Kitty placeholder cells carry their
/// image id in the foreground and keep it.
pub(crate) fn paint_cell(cell: &mut ratatui::buffer::Cell) {
    let (paint, ink) = (crate::th().ui.bg, crate::th().ui.text);
    if paint != ratatui::style::Color::Reset && cell.bg == ratatui::style::Color::Reset {
        cell.bg = paint;
    }
    if paint != ratatui::style::Color::Reset
        && cell.fg == ratatui::style::Color::Reset
        && cell.symbol() != "\u{10EEEE}"
    {
        cell.fg = ink;
    }
}

/// The cells of `area`, row by row, with their coordinates.
pub(crate) fn cells_of(buf: &Buffer, area: Rect) -> Vec<(u16, u16, ratatui::buffer::Cell)> {
    let mut out = Vec::with_capacity(usize::from(area.width) * usize::from(area.height));
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            out.push(painted_cell(buf, area, x, y));
        }
    }
    out
}

/// The cell `(x, y)` of `buf` as a repaint of `area` writes it. Where `area`
/// starts on the blank second half of a double-width glyph, that glyph
/// (one cell to the left, outside `area`) takes its place in the list, so
/// the repaint redraws it whole: writing the blank alone would erase it.
/// The list keeps one entry per cell, so lists taken at different times
/// still line up.
fn painted_cell(buf: &Buffer, area: Rect, x: u16, y: u16) -> (u16, u16, ratatui::buffer::Cell) {
    if x == area.left() && x > buf.area.left() && is_wide(&buf[(x - 1, y)]) {
        return (x - 1, y, buf[(x - 1, y)].clone());
    }
    (x, y, buf[(x, y)].clone())
}

/// Whether `cell` is a double-width glyph (a CJK character): the cell
/// after it, in the buffer, is its blank second half.
fn is_wide(cell: &ratatui::buffer::Cell) -> bool {
    ratatui::text::Span::raw(cell.symbol()).width() > 1
}

/// `cells` without the blank second half of every double-width glyph.
///
/// The buffer holds that second half as a plain `" "` next to the glyph.
/// Written out it is a real space: the terminal already advanced two cells
/// for the glyph, so the space lands on the next cell and pushes the rest of
/// the row one column right per wide character (CJK text "spaced out", the
/// popup's border shoved past its edge). Kitty and the terminals that
/// forward its picture protocol (WezTerm, the multiplexers in front of
/// them) are where this repaint runs, so it only showed there.
fn without_continuations(
    cells: &[(u16, u16, ratatui::buffer::Cell)],
) -> Vec<(u16, u16, &ratatui::buffer::Cell)> {
    let mut out = Vec::with_capacity(cells.len());
    let mut prev: Option<(u16, u16, bool)> = None;
    for (x, y, cell) in cells {
        let second_half = matches!(prev, Some((px, py, true)) if py == *y && px + 1 == *x);
        prev = Some((*x, *y, !second_half && is_wide(cell)));
        if !second_half {
            out.push((*x, *y, cell));
        }
    }
    out
}

/// Writes an encoded image at `anchor`, then `over` on top of it.
///
/// Called **after** ratatui has flushed the frame. `over` is whatever the
/// frame drew into the image's area - a popup's cells - which the
/// picture has just been laid over and which must therefore be written
/// again, or a popup that did not change would be erased wherever it
/// crosses the image.
///
/// The cursor is saved and restored around the whole thing (`DECSC` /
/// `DECRC`), so a visible text cursor that ratatui has just positioned
/// stays exactly where it was.
#[cfg(test)]
pub(crate) fn write_image<W: std::io::Write>(
    out: &mut W,
    anchor: (u16, u16),
    sequence: &str,
    over: &[(u16, u16, ratatui::buffer::Cell)],
) -> std::io::Result<()> {
    write_images(out, &[(anchor, sequence)], over)
}

/// [`write_image`] for several pictures at once: every picture, in order,
/// then `over` on top of all of them, inside one cursor save/restore and
/// one flush. The Log's per-row author avatars
/// (`avatar::LogAvatarImages`) send only the rows that changed, and a
/// scroll can change most of them in one frame.
#[cfg(test)]
pub(crate) fn write_images<W: std::io::Write>(
    out: &mut W,
    pictures: &[((u16, u16), &str)],
    over: &[(u16, u16, ratatui::buffer::Cell)],
) -> std::io::Result<()> {
    write_images_layered(out, &[], pictures, over)
}

/// [`write_images`] with `under`: cells written **before** the pictures.
/// A transparent picture ([`Protocol::Iterm2Alpha`]) is composited over
/// whatever the cell holds, and a picture already there would show through
/// the new one's transparent pixels. Writing text into a cell drops its
/// images, so rewriting the cells first guarantees a clean background.
pub(crate) fn write_images_layered<W: std::io::Write>(
    out: &mut W,
    under: &[(u16, u16, ratatui::buffer::Cell)],
    pictures: &[((u16, u16), &str)],
    over: &[(u16, u16, ratatui::buffer::Cell)],
) -> std::io::Result<()> {
    use crossterm::cursor::{MoveTo, RestorePosition, SavePosition};
    use ratatui::backend::Backend;
    crossterm::queue!(out, SavePosition)?;
    if !under.is_empty() {
        let under: Vec<_> = without_continuations(under)
            .into_iter()
            .map(|(x, y, c)| (x, y, as_painted(c)))
            .collect();
        let mut backend = ratatui::backend::CrosstermBackend::new(&mut *out);
        backend.draw(under.iter().map(|(x, y, cell)| (*x, *y, cell)))?;
    }
    for (anchor, sequence) in pictures {
        crossterm::queue!(out, MoveTo(anchor.0, anchor.1))?;
        out.write_all(sequence.as_bytes())?;
    }
    if !over.is_empty() {
        let over: Vec<_> = without_continuations(over)
            .into_iter()
            .map(|(x, y, c)| (x, y, as_painted(c)))
            .collect();
        let mut backend = ratatui::backend::CrosstermBackend::new(&mut *out);
        backend.draw(over.iter().map(|(x, y, cell)| (*x, *y, cell)))?;
    }
    crossterm::queue!(out, RestorePosition)?;
    out.flush()
}

#[cfg(test)]
mod tests;
