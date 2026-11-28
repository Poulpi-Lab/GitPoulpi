# The commit graph: glyphs, pixels, and the traps

Code: `crates/gitpoulpi-app/src/graph/` (the frontend-neutral model shared
with the desktop UI: `part.rs` vocabulary and `ColorRef`, `layout.rs` lane
fold, `stash.rs` stash placement and `LogRows`, `paging.rs` lazy loading) and
`crates/gitpoulpi-tui/src/graph/` (`part.rs` glyphs and colours, `layout.rs`
resolves the neutral colours and builds the lines,
`raster.rs` + `raster/` pixels, `inline_image.rs` the column as a picture,
`cache.rs` per-change layout cache,
`connector.rs` ref-to-commit line) and `terminal_image/` (protocol
detection, encoders, Kitty). Module headers carry the detail; this page is
the evidence and the decisions that are not guessable from the code. Read
`part.rs`'s header before changing a glyph.

## `TreePart`: connectivity is a checked property

A graph row is `Vec<TreeCell>`: a `TreePart`, a lane colour, `bold`, and
`dashed: Sides` (pending ink: changes the ink, never the topology). Lane
`j` owns column `2*j`; `2*j+1` is the connector cell. `TreeCell::span` is
the only place a part becomes characters; layout never writes a literal.

Each part declares the `Sides` it reaches, and `no_node_is_left_unconnected`
asserts the declaration is mutual across the whole grid (grid top/bottom
are boundaries, not breaks). Before this, each branch of the layout pushed
its own string and connectivity broke three times, each caught only by a
photograph:

- a fan-out to an *adjacent* lane drew no horizontal fill (range
  `(lo+1)..hi` is empty when `hi == lo+1`): the elbow floated one cell
  from the node;
- a run crossing a *dead* intermediate lane left a hole in the line;
- a run passing over another lane's *elbow* was cut in two (an elbow inks
  only the half its arms reach).

A second check renders through a real `TestBackend`
(`the_rendered_buffer_puts_something_on_every_side_a_node_claims`), and
`the_rendered_graph_column_shows_a_marker_on_every_commit_row` goes through
the real Log table (the graph can be right while the column is too narrow
to show lane 0). A test asserting on strings you generated is not evidence.

The layout is a **fold**: row `i` depends on `nodes[0..=i]` only. That
prefix stability is what lets lazy loading append a page and re-lay out
without moving a lane (`appending_a_page_does_not_disturb_the_rows_already_laid_out`);
a global two-pass lane allocator would break it. A freshly claimed lane
always gets a fresh palette colour.

## The glyphs, and the evidence

| part | text glyph |
|---|---|
| ordinary commit | `○` U+25CB |
| commit a ref/HEAD points at | `●` U+25CF |
| merge | `◎` U+25CE |
| WIP | `◌` U+25CC, dimmed |
| stash | `□` U+25A1 |
| pending lane / run | `┊` U+250A / `┄` U+2504, dimmed |
| everything else | Box Drawing U+2500..U+257F |

### Glyph table: why U+20DD sheared the column on Windows

Until 2026-09-22 a node was `│` + U+20DD COMBINING ENCLOSING CIRCLE.
`unicode-width` says one cell and it looked fine on Linux. On Windows every
node took two cells of ink and the column slid. Measured by walking each
font's `cmap` with WPF `GlyphTypeface.CharacterToGlyphMap` (advance widths
in em; MISSING = not in the font):

| font | U+20DD | `│` U+2502 | `○` U+25CB | `●` U+25CF | `◎` U+25CE |
|---|---|---|---|---|---|
| Cascadia Code | **MISSING** | 0.5859 | 0.5859 | 0.5859 | 0.5859 |
| Cascadia Mono | **MISSING** | 0.5859 | 0.5859 | 0.5859 | 0.5859 |
| CaskaydiaCove NFM | **MISSING** | 0.5859 | 0.5859 | 0.5859 | 0.5859 |
| JetBrainsMono NFM | **MISSING** | 0.6000 | 0.6000 | 0.6000 | 0.6000 |
| Consolas | 0.5498 | 0.5498 | 0.5498 | 0.5498 | **MISSING** |
| Segoe UI Emoji | **MISSING** | MISSING | 0.8613 | 0.8613 | 0.8613 |

U+20DD is in none of the monospace fonts a Windows terminal uses, so it
resolved through system fallback into a *different* font, which breaks the
grapheme cluster: the ring gets its own advance.

**Rules this leaves, for the whole TUI** (the tab bar's `⌂`, the pane
chevrons `›`/`‹` and every marker follow them):

- **No combining marks, ever.** `unicode-width` returning 0 is about
  Unicode, not the user's font. One code point per part
  (`every_tree_part_is_one_cell_and_one_code_point`).
- **No emoji-presentation code points and no VS16 (U+FE0F).** Check
  `emoji-data.txt`: `⬛` `⚫` `◼` `▪` `▫` `◻` and U+1F4C1-style pictographs
  flip to a double-width colour glyph and have sheared this column before.
  U+25CB/CF/CE/CC/A1 are absent from it.
- **Measure before adopting a glyph**: present, at `│`'s advance, in
  Cascadia Code/Mono, CaskaydiaCove NFM, JetBrainsMono NFM and Consolas.
  All of the table above pass except `◎` in Consolas. `□`, `◌`, `┊`, `┄`
  were measured the same way and pass everywhere. `▣` `▢` (closer to
  the stash icon) are missing from JetBrainsMono NFM and Consolas;
  `⊟` `⊡` `☐` from most of the set.

Reproduce with WPF from PowerShell: `[Windows.Media.Fonts]::SystemFontFamilies`,
`GetTypefaces()`, `TryGetGlyphTypeface`, then `CharacterToGlyphMap` /
`AdvanceWidths` / `TopSideBearings` / `BottomSideBearings`. It answers "is
this code point in this font, at what width" without rendering, which no
screenshot can.

### Font measurements: why a glyph circle always has a gap

A glyph sits inside side bearings, so it cannot reach its cell's top and
bottom. Cascadia Code, same dump:

```
│  U+2502  ink height 1.346 em  (overflows the em box: box drawing tiles)
○  U+25CB  ink height 0.513 em
●  U+25CF  ink height 0.513 em
◎  U+25CE  ink height 0.586 em
line box   1.321 em
```

At the maintainer's 9x21 px cell: an 8 px circle with 6 px of bare background
above and below. The maintainer's requirement (2026-09-22): circles joined to
their lanes completely, "pas de négociation".

Dead ends, closed:

- **Another code point.** `⬤` `⭘` `⌽` `Φ` `◉` `◯` measured; the best cut
  the gap from 7 px to 4 px; the ones with a stroke through (`⌽` U+233D,
  `⦶` U+29B6) are in none of the fonts. Box Drawing is the only
  edge-to-edge family and has no round members.
- **Composing from sub-cell blocks.** The only full-bleed raster in every
  font is two sub-rows per line:

  | family | Cascadia Code/Mono, CaskaydiaCove NFM | JetBrainsMono NFM | Consolas | Lucida Console |
  |---|---|---|---|---|
  | half blocks `▀ ▄ ▌ ▐ █` | present, lsb/rsb 0 | present, lsb/rsb 0 | present | present |
  | quadrants `▖▗▘▝▙▛▜▟` | present | present | MISSING | MISSING |
  | Braille U+2800.. | present | MISSING | MISSING | MISSING |
  | sextants U+1FB00.. | present | MISSING | MISSING | MISSING |

  A closed curve needs three sub-rows; with two, every rounding collapses
  to a bar (a dozen were rendered at 9x21 and looked at). Taller rows do
  not help: a sub-cell block is 0.293 em x 0.66 em, so a circle four
  sub-rows tall is 4.5 cells wide, and it halves visible history.
- **A cell background behind the lane** closed the gap and was rejected on
  sight ("c'est quoi ce background derrière les commits").

So the column is **drawn as pixels** (below). The text renderer stays as
the fallback and **keeps the 4-6 px gap**, deliberately: do not put a
background back as the default and do not swap the glyph set to hide it.
`ui.graph_lane_background = "row_tint"` (default off) is a
row band from the node to the column's edge, a fixed mix of the lane
colour (`ui.graph_row_tint_strength`), not a rail behind the lane.

## Image renderer

`ui.graph_renderer` = `auto` | `image` | `text`. Under image mode the text
cells are still produced as blanks of the same width, so the Log table's
column arithmetic and every hit target are identical in both modes.

### Drawing (`raster.rs`, pure: rows + cell size in, RGBA bitmap out)

- Anti-aliased by supersampled inside-tests per shape: a segment is a
  capsule, an arc an annulus sector, an elbow two arms plus a quarter-arc
  fillet (a real curve, not `┬`'s right angle).
- Commit = ring at 0.62 of the lane stroke width (at a ~7 px marker a ring
  at full lane width leaves a 0.7 px hole: hollow and filled become
  indistinguishable); ref = filled disc; merge = ring plus a centre dot
  that leaves visible air.
- **Contiguity:** a stroke leaving a node stops at
  `radius - ring_half_width/2`, inside the ring band, so the two overlap by
  half a stroke and no rounding can slip a blank pixel between them.
- **Marker size tracks the glyph's height, not the cell width.** The
  glyph inks 0.513 em of a 1.321 em line box, and the line box is the cell
  height, so the diameter is 0.3884 of the height, capped at 0.44 of the
  width (in force at 9x21: 7.92 px). A first width-relative sizing matched
  only one aspect ratio and the maintainer still saw markers smaller than the
  glyphs. The cap is the largest value keeping
  `a_bare_marker_never_reaches_its_cells_edges_on_its_own` green: a bare
  marker touching its cell edge would make the connectivity proof vacuous.
- Lane width is clamped in assumed cell pixels before scaling (else a
  hairline at 2x). Details: `raster/metrics.rs`.

### The connectivity proof (`raster_tests.rs`)

`every_node_is_connected_to_every_stroke_it_claims`: flood-fill
8-connected from the first inked pixel of each node's cell and require it
to reach the cell edge on every declared side and cover every inked pixel
of the cell; run at cell sizes 7x15 to 20x40. Falsified permanently by
`RasterOptions::stroke_gap` (test-only, pushes strokes outside the marker:
`a_deliberately_detached_disc_fails_the_connectivity_check`,
`the_detached_variant_still_draws_both_the_disc_and_the_strokes`). Traps
hit while writing it: seeding at the cell **centre** is background for a
hollow ring and silently asserted nothing; flooding over "any non-background"
let the anti-aliased fringe bridge a detached stroke once the marker grew,
so it floods only over solid ink (at least half-way to the lane colour).
The guarantee covers the bitmap only; the text fallback cannot be
8-connected and no test asks it to be.

### Supersampling: the answer to zoom

The image is placed and sized in **cells**, so a wrong pixel size costs
resolution, never placement. Cell size: `GITPOULPI_TERMINAL_CELL_PX`, then
`TIOCGWINSZ` (Unix), else the measured 9x21 (`CellSize::FALLBACK`; a wrong
aspect makes ovals). On Windows it is always the fallback (crossterm's
`window_size()` is unimplemented there), so `Ctrl +` made the terminal
upscale a 9x21 tile: "quand on zoom ça pixelize". Fix: rasterise at 2x
(`RasterOptions::scale`, `GITPOULPI_TERMINAL_SUPERSAMPLE`, clamp 1-4) so
the terminal downsamples. No new I/O, robust against any wrong estimate.
Every geometry constant is in assumed cell pixels, multiplied by `scale`
exactly once (`supersampling_changes_the_resolution_and_not_the_picture`).
2x rather than 3x: 3x looked identical for twice the cost (numbers on
`terminal_image::supersample`).

### Protocols: iTerm2 and Kitty

Full rationale: `terminal_image/mod.rs`, `detect.rs` and `kitty.rs` headers.

- **iTerm2 (`OSC 1337`)** for bare WezTerm (a common Windows terminal):
  it has no Kitty placeholders (draws U+10EEEE as tofu), and
  `doNotMoveCursor=1` lets a picture sit on the last row without scrolling
  the alternate screen. Opaque picture, selection band baked in.
- **Kitty graphics with Unicode placeholders** for kitty, Ghostty, and
  herdr with `kitty_graphics` on (its `config.toml`, read once). Being
  characters, placeholders are moved and restored by ratatui's diff: no
  ghosts. Traps: a changed picture needs a **fresh id** (re-sending under a
  live id blanks in herdr); delete by id (`a=d` misses virtual
  placements); `q=2` everywhere (a reply would arrive as keystrokes); id
  top byte from the PID (two instances in one herdr). Transparent picture;
  the table cell background is the selection.
- `ui.image_protocol` = `auto` | `iterm2` | `kitty` | `none`;
  `GITPOULPI_TERMINAL_IMAGE=off` overrides everything.

**Detection never reads stdin.** A `CSI 16 t` query on a helper thread
could not be cancelled, ate the next byte, and made the pty harness
(which inherits `WEZTERM_EXECUTABLE`) swallow every keystroke.
`tests/pty_ui.rs` sets `GITPOULPI_TERMINAL_IMAGE=off`; any `xtask tui`
script must too. Kitty's `a=q` query is rejected for the same reason.

**Multiplexers deny by default.** herdr inside WezTerm keeps
`WEZTERM_EXECUTABLE`/`TERM_PROGRAM=WezTerm` and passes `TERM` through, but
forwards cells, not OSC 1337: the column filled with chewed base64. Deny
list (`HERDR_*` measured on a live pane, `TMUX`, `ZELLIJ`, `STY`, `TERM`
`screen*`/`tmux*`) in `detect.rs`; only the herdr Kitty route gets through.
herdr over SSH is text under `auto`: the local client decides whether
pictures draw, and guessing gave a blank column.

### iTerm2 path: write after the frame, never inside a cell

The first version put the whole escape in the top-left cell's symbol
(`ratatui-image`'s trick). Two bugs: `Buffer::diff` measured the base64 as
tens of thousands of columns wide and re-emitted every cell after it on
every frame (~30 fps idle); and a popup over the column replaced pixels
that were never resent when it closed (tooltip traces). Now
`ImageReservation` blanks the area and marks it `skip`; `finish_frame`
compares screen size, picture key and the set of popup-covered cells with
the previous frame; `write_pending` (after `Terminal::draw`) sends the
picture then repaints the popup cells over it, inside a synchronized
update (mode 2026). Tests in `inline_image.rs`: idle frame writes nothing,
popup close/move and resize resend, popup repainted after the picture.

`ratatui-image` is not used (MSRV and `image`-crate bulk; see
`terminal_image/mod.rs`).

Scroll cost: encode only when the hash of what is drawn changes; the input
loop drains queued events before redrawing, so a fast wheel coalesces.
`ui.mouse_scroll_lines` (default 3) is the wheel step.

### Which rows the picture covers

`graph::visible_raster_rows` takes the same scroll offset and selected
index as `log_index_to_row` / `log_row_to_index`, and `graph::LogRows` is
the one mapping from display index to WIP / commit `i` / stash `k`. Never
re-derive the row arithmetic: an image one row out lands on the wrong
commits and looks deliberate.

## The WIP row

One Log row on HEAD's lane, a dashed ring joined to HEAD by a dashed line
(it was a two-line block until 2026-09-24).

- Laid out as the newest pseudo-commit whose parent is HEAD
  (`WipAnchor { head }`): it takes lane 0 and colour 0, then the ordinary
  fold does the rest; only the ink of that lane from WIP to HEAD is
  `dashed`. HEAD on top: commit rows are exactly those of a clean tree.
  HEAD further down: newer commits' lanes shift right by one.
- No HEAD to hang from (unborn, detached: the refs carry no `is_head`,
  solo-filtered, not loaded): a lone marker that reserves no lane/colour.
- Kept out of `GraphGrid::rows` (`GraphGrid::wip`), so grid index ==
  `nodes` index; display index is +1. Folded first, so prefix stability
  holds.
- Pixels: the ring cut into 8 dashes with one centred at the bottom (the
  connector always meets ink); pending ink at `PENDING_ALPHA` 0.62. **The
  dash pattern is anchored to the bitmap, not per segment**: the connector
  spans several short pieces across rows, and restarting per piece put a
  dash on both sides of each cell edge. Dashes are measured cap to cap:
  round caps add half a stroke per end, so "3 on, 3 off" read as solid
  (`a_wip_stem_is_visibly_dashed_with_real_gaps`).
- Text: `◌` then, when HEAD is directly below, **no connector glyph at
  all** (no half cell to draw a stub in); `┊` on lane cells; crossings
  stay solid `┼` (no dashed cross exists). HEAD's marker is never dimmed.

## Stash rows

Each stash is a row at its date, a box on its own lane right of its
**base** (first parent), joined by a dashed connector curving into it.
`ui.graph_show_stashes` (default on); hidden while soloing.

- Folded as a pseudo-commit whose only parent is the base, with three
  differences so the box sits *beside* the base: a stash lane
  (`stash_lanes`) is never the lane a commit takes as "expecting me"; it
  goes to the first free lane right of where the base will land (a lane
  already expecting the base, else one whose first-parent chain reaches
  it); when none does (base is a tip below the stash) a lane is
  *reserved* for the base, drawn as nothing.
- Placement (`place_stashes`): right above the first commit, from the top
  down to the base, not newer than the stash. Several stashes on one
  commit keep `stash@{0}` first. A stash whose base is not loaded is not
  drawn until lazy loading brings the base in. Reads `nodes[..=base]` only
  (prefix stable).
- Colour: always `layout::stash_color()` (the theme's `graph.stash`), never a palette slot; taking
  one would recolour every branch tip below.
- Ink: box solid; connector dashed wherever no history shares the cell (a
  run that is also a merge's stays solid). Pixels: square outline at the
  ring's extent and weight, so the connectivity proof covers it; an elbow's
  dashed arms on the global pattern and its fillet as dimmed ink (a dashed
  2 px arc reads as noise). Text: `□` undimmed, `┊` / `┄` dimmed, elbow a
  solid dimmed `╯`/`╰` (no dashed corner exists).
- Rows are in `GraphGrid::stashes` / `stash_slots`, not `rows`.

## Ref-to-commit connector

`ui.ref_connector` (default on): a dimmed line from each ref pill to its
node. **Not a `TreePart`**: no topology, invisible to the connectivity
tests, fills blanks only, so centring and hit targets stay put. Lanes it
passes are skipped, never drawn as `┼` (that is `TreePart::Crossing` and
would read as history). Only while the Ref column sits immediately left of
the Graph column (`ref_connector_applies`).

## Author avatars in the Log

`avatar/log_images.rs`, `ui.show_avatars`: a round picture in the Author
column's two leading cells, reusing detection, multiplexer denial,
supersampling and the post-frame write. **One picture per row, not one
strip**: avatars arrive on their own schedule and each row's background
(selection, hover) changes on its own; a strip would be re-encoded whole.
The encode is cached per (author hash, row tint, dimmed), so a scroll
never encodes; only rows whose key changed are sent, in one
`terminal_image::write_images` call. The row background is drawn into the
picture's corners (an image covers text). Worst case measured
(2026-09-24): 60 distinct authors from scratch, 5.5-6.8 ms and 85 KB at 2x,
paid once; synthetic faces, real photos up to ~2x the bytes. Guarded by
`a_full_screen_of_new_authors_encodes_inside_a_frame_budget`.

## How much history is loaded

`general.initial_commits_in_graph` default 5000, floor 500 (more
than any window, bounded on huge repos), then lazy pages
(`gitpoulpi-app` `graph/paging.rs`, one request in flight, fire-and-forget off the input
loop) up to `general.max_commits_in_graph`. It was once a hard-coded 50,
which left the pane half empty with nothing to scroll to.
`a_full_page_of_history_lays_out_quickly` catches a quadratic lane walk.
