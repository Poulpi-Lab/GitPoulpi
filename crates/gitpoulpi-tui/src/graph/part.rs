//! `TreePart` -- the one and only vocabulary the commit graph draws with.
//!
//! Every cell of the Graph column is a [`TreeCell`]: one [`TreePart`] plus
//! the lane color it carries. Layout produces parts; [`TreeCell::span`] is
//! the *single* place a part becomes characters. No code outside this file
//! ever writes a graph glyph as a string literal.
//!
//! Why the indirection, when a `&'static str` per case would be shorter:
//! **"does this piece join the piece next to it?" becomes a property you can
//! enumerate**. Each part declares the [`Sides`] it connects to, and
//! `no_node_is_left_unconnected` walks the whole rendered grid asserting the
//! declaration is mutual -- if a cell reaches right, its right-hand
//! neighbor must reach left; if it reaches down, the cell below it must
//! reach up. Without it, connectivity would depend on whichever string
//! literal a branch of `build_graph_lines` happened to push.
//!
//! ---------------------------------------------------------------------
//! # The glyph decision, and the evidence for it
//!
//! Read this before "improving" the glyph set. Three sets have shipped; two
//! of them were wrong, and the reasons are not guessable from the code.
//!
//! ## What broke on Windows
//!
//! A rejected set drew each node as **`│` + U+20DD COMBINING ENCLOSING
//! CIRCLE**: a box-drawing vertical with a ring combined over it. It
//! measures one cell wide through `unicode-width` and rendered acceptably on
//! Linux, but on Windows it sheared the whole column: circles floated a cell
//! away from the lane, elbows landed beside their node. The cause, measured
//! by walking each font's `cmap` through WPF's
//! `GlyphTypeface.CharacterToGlyphMap`:
//!
//! ```text
//! font                      U+20DD   U+2502   U+25CB  U+25CF  U+25CE
//! Cascadia Code             MISSING  0.5859   0.5859  0.5859  0.5859
//! Cascadia Mono             MISSING  0.5859   0.5859  0.5859  0.5859
//! CaskaydiaCove NFM         MISSING  0.5859   0.5859  0.5859  0.5859
//! JetBrainsMono NFM         MISSING  0.6000   0.6000  0.6000  0.6000
//! Consolas                  0.5498   0.5498   0.5498  0.5498  MISSING
//! Segoe UI Emoji            MISSING  MISSING  0.8613  0.8613  0.8613
//! ```
//!
//! (advance widths in em; `MISSING` = the code point is not in the font.)
//!
//! **U+20DD is in none of the monospace fonts a Windows terminal actually
//! uses.** It exists only in Consolas, which is not in the font stack, so
//! the terminal resolved it through system-wide fallback -- and a fallback
//! that lands in a *different* font breaks the grapheme cluster: the ring
//! stops being a zero-width mark over the `│` and gets its own advance.
//! One cell of text becomes two cells of ink, every node, and everything
//! to its right slides. That is precisely the screenshot.
//!
//! So: **no combining marks in this file, ever.** `unicode-width` saying
//! zero is a statement about Unicode, not about the font the user has.
//!
//! ## What is drawn
//!
//! Three commit shapes, and no others:
//! - [`NodeKind::Commit`] `○` U+25CB WHITE CIRCLE -- an ordinary commit;
//! - [`NodeKind::Ref`] `●` U+25CF BLACK CIRCLE -- a commit some branch,
//!   tag or HEAD points at;
//! - [`NodeKind::Merge`] `◎` U+25CE BULLSEYE -- a merge, the double circle.
//!
//! Plus two that are not commits: the WIP row's `◌` and a stash's `□`
//! U+25A1 WHITE SQUARE. `□` was measured the same way: present
//! in Cascadia Code/Mono, CaskaydiaCove NFM, JetBrainsMono NFM *and*
//! Consolas, at `│`'s advance in each, and absent from `emoji-data.txt`
//! (unlike `▪`/`▫`/`◻`/`◼`). `▣` and `▢` were closer to a stash box
//! but are missing from JetBrainsMono NFM and Consolas.
//!
//! All three are present in **every** monospace font measured above except
//! Consolas (which has `○` and `●` but not `◎`), all three carry the *same
//! advance width as `│`* in each of those fonts, all three are a single
//! code point with no variation selector and no combining mark, and none of
//! them has an emoji presentation -- U+25CB/CF/CE are absent from
//! `emoji-data.txt` entirely, unlike `⬛`/`⚫`/`◼`, which flip to a
//! double-width color glyph and sheared this same column once before.
//!
//! ## The gap, and why no choice of glyph could ever close it
//!
//! The graph keeps a visible break where the lane meets the commit marker.
//! Pre-made glyphs have side margins, and the circles are wanted fully
//! joined to their lanes, which only the image path ([`super::raster`])
//! delivers.
//!
//! The break is real and it is a property of type, not of this code. A
//! circle cannot reach the top and bottom edges of its cell, because a
//! glyph is drawn inside side bearings. Measured in Cascadia Code, em
//! relative, from the same `GlyphTypeface` dump as the table above:
//!
//! ```text
//! │  U+2502  ink height 1.346 em  (overflows the em box deliberately:
//!                                  box-drawing exists to tile seamlessly)
//! ○  U+25CB  ink height 0.513 em
//! ●  U+25CF  ink height 0.513 em
//! ◎  U+25CE  ink height 0.586 em
//! ```
//!
//! The line box is 1.321 em. On a 9x21 px cell that is an 8 px circle with
//! **6 px of bare background above it and 6 px below**: the visible break.
//!
//! Shopping for a rounder, taller code point does not work and the search
//! is closed: `⬤` `⭘` `⌽` `Φ` `◉` `◯` were all measured, the best of them
//! only got the gap from 7 px to 4 px, and the ones with a stroke through
//! them (`⌽` U+233D, `⦶` U+29B6) are in none of the fonts above.
//!
//! ### Nor could we draw one, and this is the part worth keeping
//!
//! The obvious next move is to stop using a glyph and *compose* the marker
//! out of sub-cell pieces. It cannot be done, for a reason that is worth
//! writing down so nobody spends another day on it:
//!
//! **The only full-bleed raster a terminal has, in every font, is two
//! sub-rows per line.** Half blocks (`▀ ▄ ▌ ▐ █`) are the one family
//! present in every font measured -- Consolas and Lucida Console included
//! -- with zero side bearings and ink height exactly half or exactly all
//! of the line box. Quadrants (`▖▗▘▝▙▛▜▟`) are missing from Consolas and
//! Lucida Console and still give only two sub-rows. **Braille (U+2800) and
//! sextants (U+1FB00) -- the two families that would give three or four
//! sub-rows -- are missing from JetBrainsMono NFM, Consolas and Lucida
//! Console**; they exist only in the Cascadia family, so neither can be a
//! default. (Measured, same method; the table is in
//! `docs/developer/commit-graph.md`.)
//!
//! A closed curve needs at least three sub-rows: one for the top arc, one
//! for the sides, one for the bottom arc. With two, every "rounding" of a
//! shape removes half its height, so every composition collapses to a
//! rectangle or a bar -- all of them were rendered at 9x21 px and looked
//! at before this was written. Going taller does not rescue it either: a
//! sub-cell block is 0.293 em wide and 0.66 em tall, so a *circular*
//! region four sub-rows tall would have to be nine sub-columns -- four and
//! a half cells -- wide. A two-row node is a worse ellipse than a one-row
//! glyph, not a better circle.
//!
//! ### What is drawn instead: nothing, here
//!
//! **The gap is not closed in this module, and no longer pretends to be.**
//! Closing it needs pixels, and pixels are [`super::raster`]: the Graph
//! column is drawn as an image and sent to the terminal by
//! [`super::inline_image`], where the disc and the strokes are the same
//! ink in the same bitmap and contiguity is arithmetic rather than
//! typography.
//!
//! This module is what runs **when that is not available** -- a terminal
//! with no image protocol, and the pty test harness, which is a cell grid
//! and has no pixels to inspect. It draws `○` `●` `◎` and box drawing, and
//! it **keeps the 4-6 px gap**.
//!
//! That is a deliberate trade: a terminal without image support falls back
//! to this rendering, without a background.
//!
//! A per-cell background in the lane color would close the gap, but it reads
//! as a wide tinted ribbon behind the commits rather than as a line, so it is
//! not drawn. **Do not put it back**, and do not swap the glyph set to work
//! around the gap either. `ui.graph_lane_background = "row_tint"` is the
//! opt-in alternative: a faint band per commit row ([`row_tint_bg`]).

use gitpoulpi_config::GraphRowTintStrength;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

// The topology vocabulary is frontend-neutral and lives in `gitpoulpi-app`;
// this file is the terminal's reading of it (glyphs, colours, spans).
pub(crate) use gitpoulpi_app::graph::{ColorRef, NodeKind, Sides, TreePart};

/// Which marker family the graph draws commits with. Only the circles exist
/// today; the type stays so [`super::GraphSettings`] has one place to choose it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NodeSet {
    /// `○` `●` `◎` - round. A few pixels of air above and below the marker;
    /// see this module's header for the measurements.
    #[default]
    Circles,
}

/// `ui.graph_lane_background`, `Off` by default: whether each commit row gets
/// a tint band behind its lane marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LaneBackground {
    /// No background at all. The lane is a line and the marker has a few
    /// pixels of air around it.
    #[default]
    Off,
    /// Each commit row is tinted ([`row_tint_bg`]), from its node to the Graph column's right edge.
    RowTint,
}

/// Box Drawing, U+2500..U+257F. Mandatory for a terminal font - Cascadia
/// Code/Mono on Windows, Menlo/SF Mono on macOS, DejaVu Sans Mono and every
/// Nerd Font on Linux cover the block completely - so no fallback font is
/// ever consulted for these and the metrics are the primary font's
/// everywhere.
const LANE: &str = "\u{2502}"; // │
const HORIZONTAL: &str = "\u{2500}"; // ─
const CROSSING: &str = "\u{253C}"; // ┼
const CORNER_OUT_LEFT: &str = "\u{256D}"; // ╭ right + down
const CORNER_OUT_RIGHT: &str = "\u{256E}"; // ╮ left + down
const CORNER_IN_LEFT: &str = "\u{2570}"; // ╰ up + right
const CORNER_IN_RIGHT: &str = "\u{256F}"; // ╯ up + left
const TEE_DOWN: &str = "\u{252C}"; // ┬ left + right + down
const TEE_UP: &str = "\u{2534}"; // ┴ left + right + up
/// A lane that is not history yet: the WIP row's connector down to HEAD.
/// Same block, so the same guarantee: present in Cascadia Code/Mono,
/// CaskaydiaCove NFM, JetBrainsMono NFM and Consolas, at `│`'s own advance
/// width.
const LANE_DASHED: &str = "\u{250A}"; // ┊
/// The same, horizontal: the stretch of a stash's connector that runs across
/// to its base. Same block, same fonts, same width.
const HORIZONTAL_DASHED: &str = "\u{2504}"; // ┄

/// The ref-to-commit connector (`ui.ref_connector`): the thin line from a
/// ref pill to its commit's node. The same `─` as a run, but it is **not**
/// a [`TreePart`] - it is a decoration laid over blank cells, carries no
/// topology, and is never part of the grid the connectivity tests walk.
/// Drawn dimmed, so it reads as lighter than the lanes it passes.
const REF_CONNECTOR: &str = "\u{2500}"; // ─

/// Geometric Shapes, U+25A0..U+25FF. See the module header for the font
/// coverage measurements and for why these three and not others.
const CIRCLE_HOLLOW: &str = "\u{25CB}"; // ○
const CIRCLE_FILLED: &str = "\u{25CF}"; // ●
const CIRCLE_DOUBLE: &str = "\u{25CE}"; // ◎
/// The WIP row's marker: a dotted outline, "not a commit yet". Same fonts
/// and advance as `LANE_DASHED`/`○`, single code point, absent from
/// `emoji-data.txt`.
const CIRCLE_DOTTED: &str = "\u{25CC}"; // ◌
/// A stash: a box. See the module header for the measurements.
const STASH_BOX: &str = "\u{25A1}"; // □

/// The color row tints are blended towards: `th().ui.blend_bg`, the
/// background the user actually has (Dark: the assumed terminal background
/// `Rgb(29, 32, 34)`, since the Log pane paints none; a theme that paints its
/// own `bg` blends toward that).
fn panel_bg() -> (u8, u8, u8) {
    crate::theme::blend_rgb()
}

/// `ui.graph_lane_background = "row_tint"`: the faint band from a commit's
/// node to the Graph column's right edge - the commit's
/// lane color mixed into the pane background at [`row_tint_mix`]. A fixed mix rather than a luminance
/// step: the band is wide, so it must stay far below the text's contrast on
/// every lane color, and the bright ones are the ones that would shout.
pub(crate) fn row_tint_bg(lane: Color, strength: GraphRowTintStrength) -> Color {
    let Color::Rgb(r, g, b) = lane else {
        return lane;
    };
    let k = row_tint_mix(strength);
    let mix = |c: u8, toward: u8| (f64::from(c) * k + f64::from(toward) * (1.0 - k)).round() as u8;
    let panel = panel_bg();
    Color::Rgb(mix(r, panel.0), mix(g, panel.1), mix(b, panel.2))
}

/// How much of the lane color survives in a row tint
/// (`ui.graph_row_tint_strength`); the image renderer uses the same number
/// as the band's alpha, so both renderers agree.
///
/// The row color is a flat lane color at roughly 6-10% over
/// its dark pane; the maintainer wanted the bands to stand out more than that
/// (2026-10-09), above all on light themes where dark lanes over a pale
/// pane read faint, so the levels are 0.22 / 0.14 / 0.09 (were 0.16 / 0.10
/// / 0.06). On the 29/32/34 pane a saturated blue lane color moves its
/// dominant channel by about 31 (medium) and 20 (subtle) levels against
/// strong's 49.
pub(crate) fn row_tint_mix(strength: GraphRowTintStrength) -> f64 {
    let k = match strength {
        GraphRowTintStrength::Strong => 0.22,
        GraphRowTintStrength::Medium => 0.14,
        GraphRowTintStrength::Subtle => 0.09,
    };
    // A light theme's lanes are dark: mixed into a pale pane they only
    // muddy it, so the band takes more of the lane to read as a color.
    if crate::theme::th().light {
        k * LIGHT_TINT_BOOST
    } else {
        k
    }
}

/// How much more of the lane a row tint takes on a light theme.
const LIGHT_TINT_BOOST: f64 = 2.0;

/// The terminal's spelling of a [`TreePart`].
pub(crate) trait TreePartGlyph {
    /// The characters this part paints, under `set`. Exactly one cell wide
    /// and exactly one code point for every variant - pinned by
    /// `every_tree_part_is_one_cell_and_one_code_point`.
    fn glyph(self, set: NodeSet) -> &'static str;
}

impl TreePartGlyph for TreePart {
    fn glyph(self, set: NodeSet) -> &'static str {
        match self {
            TreePart::Empty => " ",
            TreePart::Vertical => LANE,
            TreePart::Horizontal => HORIZONTAL,
            TreePart::Crossing => CROSSING,
            TreePart::Node { kind, .. } => match (set, kind) {
                (NodeSet::Circles, NodeKind::Commit) => CIRCLE_HOLLOW,
                (NodeSet::Circles, NodeKind::Ref) => CIRCLE_FILLED,
                (NodeSet::Circles, NodeKind::Merge) => CIRCLE_DOUBLE,
                (NodeSet::Circles, NodeKind::Wip) => CIRCLE_DOTTED,
                (NodeSet::Circles, NodeKind::Stash) => STASH_BOX,
            },
            TreePart::BranchOutLeft => CORNER_OUT_LEFT,
            TreePart::BranchOutRight => CORNER_OUT_RIGHT,
            TreePart::MergeInLeft => CORNER_IN_LEFT,
            TreePart::MergeInRight => CORNER_IN_RIGHT,
            TreePart::BranchOutCrossed => TEE_DOWN,
            TreePart::MergeInCrossed => TEE_UP,
        }
    }
}

/// A part plus the terminal colour of its lane: [`gitpoulpi_app::graph::TreeCell`]
/// with the [`ColorRef`] resolved against the palette in force.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TreeCell {
    pub part: TreePart,
    pub color: Color,
    /// See [`gitpoulpi_app::graph::TreeCell::bold`].
    pub bold: bool,
    /// See [`gitpoulpi_app::graph::TreeCell::dashed`].
    pub dashed: Sides,
}

impl TreeCell {
    #[cfg(test)]
    pub(crate) fn new(part: TreePart, color: Color) -> TreeCell {
        TreeCell {
            part,
            color,
            bold: false,
            dashed: Sides::NONE,
        }
    }

    /// The same cell with `sides` drawn dashed.
    #[cfg(test)]
    pub(crate) fn with_dashed(mut self, sides: Sides) -> TreeCell {
        self.dashed = sides;
        self
    }

    pub(crate) const fn empty() -> TreeCell {
        TreeCell {
            part: TreePart::Empty,
            color: Color::Reset,
            bold: false,
            dashed: Sides::NONE,
        }
    }

    /// The neutral cell with its colour resolved by `color_of`.
    pub(crate) fn resolve(
        cell: gitpoulpi_app::graph::TreeCell,
        color_of: impl Fn(ColorRef) -> Color,
    ) -> TreeCell {
        TreeCell {
            part: cell.part,
            color: color_of(cell.color),
            bold: cell.bold,
            dashed: cell.dashed,
        }
    }

    /// [`gitpoulpi_app::graph::TreeCell::is_pending`], for the terminal cell.
    pub(crate) fn is_pending(self) -> bool {
        let mut neutral = gitpoulpi_app::graph::TreeCell::new(self.part, ColorRef::None);
        neutral.dashed = self.dashed;
        neutral.is_pending()
    }

    /// The single place a `TreePart` becomes something a terminal draws.
    pub(crate) fn span(self, set: NodeSet) -> Span<'static> {
        if self.part == TreePart::Empty {
            // Deliberately unstyled: an empty cell carries no lane, so
            // giving it a color would put a background/modifier on blank
            // space for no reason.
            return Span::raw(" ");
        }
        let mut style = Style::default().fg(self.color);
        if self.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        // Pending work reads as pending in characters too: dimmed, and a
        // lane cell carrying only a dashed connector is `┊`, not `│` (`┄`,
        // not `─`, for a run). A crossing on that lane stays `┼`, and a
        // stash connector's elbow stays a solid, dimmed `╯` -- the
        // box-drawing block has no dashed cross or corner, and a solid one
        // is the honest fallback.
        if self.is_pending() {
            style = style.add_modifier(Modifier::DIM);
            match self.part {
                TreePart::Vertical => return Span::styled(LANE_DASHED, style),
                TreePart::Horizontal => return Span::styled(HORIZONTAL_DASHED, style),
                _ => {}
            }
        }
        Span::styled(self.part.glyph(set), style)
    }
}

/// `cells` cells of the ref-to-commit connector, in `lane`'s color:
/// `─`, dimmed. The single place the connector becomes characters - both the
/// Ref column's padding after a pill (`log_pills`) and the blank Graph
/// cells up to the node (`super::connector`) call this.
///
/// Dimmed rather than a different glyph: box drawing's light stroke is
/// already the lightest line a font has, and the dashed variants (`┄`,
/// `╌`) already mean "pending" in this graph (the WIP and stash
/// connectors). DIM is what makes it read as thinner than a lane.
pub(crate) fn ref_connector_span(lane: Color, cells: usize) -> Span<'static> {
    Span::styled(
        REF_CONNECTOR.repeat(cells),
        Style::default().fg(lane).add_modifier(Modifier::DIM),
    )
}
