use super::*;
use crate::terminal_image::Protocol;
use crate::th;
use crate::wide_edge::Clear;
use ratatui::widgets::{Paragraph, Widget};

const SCREEN: Rect = Rect::new(0, 0, 40, 12);
/// The Author column, like the real one: under the Log's border and
/// header, a dozen cells wide.
const AUTHOR_X: u16 = 20;
const AUTHOR_W: u16 = 12;
const TOP: u16 = 2;
const RIGHT: u16 = 39;

fn images() -> LogAvatarImages {
    LogAvatarImages::new(Capability {
        protocol: Protocol::Iterm2,
        cell: CellSize::FALLBACK,
    })
}

fn face(fill: (u8, u8, u8)) -> Bitmap {
    Bitmap::new(42, 42, fill)
}

/// One visible row: which author (by name, standing in for the hash),
/// whether their picture is ready, and the row's tint.
#[derive(Clone)]
struct Row {
    author: &'static str,
    ready: bool,
    tint: RowTint,
}

fn row(author: &'static str) -> Row {
    Row {
        author,
        ready: true,
        tint: RowTint::Plain,
    }
}

/// The table's own drawing of the Author column: the two-letter initials
/// in the slot and the name after it, on every row. What the pictures go
/// over.
fn draw_table(buf: &mut Buffer, rows: &[Row]) {
    for (i, r) in rows.iter().enumerate() {
        let y = TOP + i as u16;
        buf.set_string(
            AUTHOR_X,
            y,
            format!("{:<2} {}", &r.author[..2], r.author),
            ratatui::style::Style::default(),
        );
    }
}

/// One whole frame in the draw loop's order: the table, the avatars over
/// it, whatever popup is open, `finish_frame`, then `write_pending`.
/// Returns the buffer and what was written after the flush.
fn frame(
    img: &mut LogAvatarImages,
    rows: &[Row],
    faces: &HashMap<&'static str, Bitmap>,
    screen: Rect,
    overlay: Option<Rect>,
) -> (Buffer, String) {
    let mut buf = Buffer::empty(screen);
    draw_table(&mut buf, rows);
    let slots: Vec<AvatarSlot> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.ready)
        .filter_map(|(i, r)| {
            Some(AvatarSlot {
                area: slot_area(AUTHOR_X, AUTHOR_W, TOP + i as u16, RIGHT)?,
                key: PictureKey {
                    author: r.author.to_string(),
                    tint: Some(r.tint),
                    dimmed: false,
                },
                bitmap: faces.get(r.author)?,
            })
        })
        .collect();
    img.render(&mut buf, &slots);
    if let Some(popup) = overlay {
        Clear.render(popup, &mut buf);
        Paragraph::new("a tooltip over the avatars").render(popup, &mut buf);
    }
    img.finish_frame(&buf);
    let mut out = Vec::new();
    img.write_pending(&mut out).unwrap();
    (buf, String::from_utf8(out).unwrap())
}

fn faces() -> HashMap<&'static str, Bitmap> {
    HashMap::from([
        ("ada", face((200, 40, 40))),
        ("bob", face((40, 200, 40))),
        ("cyd", face((40, 40, 200))),
    ])
}

fn pictures_sent(written: &str) -> usize {
    written.matches("]1337;File=").count()
}

/// Where each picture was sent: the row of every `MoveTo` crossterm
/// wrote just before a picture (`ESC [ row ; col H`, 1-based).
fn rows_sent(written: &str) -> Vec<u16> {
    written
        .split("\x1b[")
        .filter_map(|chunk| {
            let (pos, rest) = chunk.split_once('H')?;
            if !rest.starts_with("\x1b]1337;File=") {
                return None;
            }
            let (row, _) = pos.split_once(';')?;
            Some(row.parse::<u16>().ok()? - 1)
        })
        .collect()
}

// ---- placement ----

#[test]
fn the_slot_is_the_author_columns_two_leading_cells() {
    assert_eq!(
        slot_area(AUTHOR_X, AUTHOR_W, 5, RIGHT),
        Some(Rect::new(AUTHOR_X, 5, SLOT_COLS, 1))
    );
}

/// Red if the `+ 1` (gutter) is dropped from the width check: a two-cell
/// column gets a slot that butts into the next column.
#[test]
fn a_column_too_narrow_for_the_picture_and_its_gutter_gets_none() {
    assert_eq!(slot_area(AUTHOR_X, SLOT_COLS, 5, RIGHT), None);
    assert!(slot_area(AUTHOR_X, SLOT_COLS + 1, 5, RIGHT).is_some());
}

#[test]
fn a_slot_that_would_cross_the_panes_right_edge_gets_none() {
    assert_eq!(slot_area(RIGHT - 1, AUTHOR_W, 5, RIGHT), None);
    assert!(slot_area(RIGHT - 2, AUTHOR_W, 5, RIGHT).is_some());
}

// ---- mode / fallback ----

#[test]
fn the_log_shows_pictures_only_with_the_setting_on_and_an_image_terminal() {
    let image = Capability {
        protocol: Protocol::Iterm2,
        cell: CellSize::FALLBACK,
    };
    let text = Capability {
        protocol: Protocol::None,
        cell: CellSize::FALLBACK,
    };
    use gitpoulpi_config::AvatarPlacement::{Author, Graph, Off};
    assert_eq!(log_avatar_mode(Off, image), LogAvatarMode::Off);
    assert_eq!(log_avatar_mode(Off, text), LogAvatarMode::Off);
    assert_eq!(log_avatar_mode(Author, image), LogAvatarMode::Picture);
    // herdr, tmux, `ui.graph_renderer = "text"`: all `Protocol::None`.
    assert_eq!(log_avatar_mode(Author, text), LogAvatarMode::Glyph);
    // `graph`: the picture is in the commit circle, so the Author column
    // shows nothing -- unless the graph is text, which has no circle to
    // put it in, and the column's glyph stands in.
    assert_eq!(log_avatar_mode(Graph, image), LogAvatarMode::Off);
    assert_eq!(log_avatar_mode(Graph, text), LogAvatarMode::Glyph);
}

/// Regression: with `ui.avatars = "graph"` (the maintainer's
/// setting) the comparison cards showed no avatar at all, because they reused
/// the Log's Author-column verdict (`Off`: the picture lives in the graph).
/// The cards have no graph circle, so they show it in their own slot.
#[test]
fn the_cards_show_pictures_for_graph_and_author_but_not_off() {
    let image = Capability {
        protocol: Protocol::Kitty,
        cell: CellSize::FALLBACK,
    };
    let text = Capability {
        protocol: Protocol::None,
        cell: CellSize::FALLBACK,
    };
    use gitpoulpi_config::AvatarPlacement::{Author, Graph, Off};
    assert_eq!(card_avatar_mode(Graph, image), LogAvatarMode::Picture);
    assert_eq!(card_avatar_mode(Author, image), LogAvatarMode::Picture);
    assert_eq!(card_avatar_mode(Off, image), LogAvatarMode::Off);
    assert_eq!(card_avatar_mode(Graph, text), LogAvatarMode::Glyph);
    assert_eq!(card_avatar_mode(Author, text), LogAvatarMode::Glyph);
    assert_eq!(card_avatar_mode(Off, text), LogAvatarMode::Off);
}

/// A terminal without images reserves nothing: the table's initials are
/// what shows, and nothing is ever written after the frame.
#[test]
fn a_terminal_that_cannot_draw_images_keeps_the_initials() {
    let mut img = LogAvatarImages::new(Capability {
        protocol: Protocol::None,
        cell: CellSize::FALLBACK,
    });
    let rows = vec![row("ada"), row("bob")];
    let (buf, written) = frame(&mut img, &rows, &faces(), SCREEN, None);
    assert_eq!(written, "");
    assert_eq!(buf[(AUTHOR_X, TOP)].symbol(), "a");
    assert!(buf[(AUTHOR_X, TOP)].diff_option != ratatui::buffer::CellDiffOption::Skip);
}

/// Only the rows handed a picture are reserved; the others keep what the
/// table drew. ("Not cached" means `AvatarService::pictures` answers
/// `None`, pinned by
/// `a_cached_author_becomes_a_picture_without_blocking_the_frame`.) Red if
/// the helper's uncached row gets a slot anyway: the initials cells turn
/// `skip`.
#[test]
fn a_row_whose_picture_is_not_cached_keeps_its_initials_and_gets_no_picture() {
    let mut img = images();
    let rows = vec![
        row("ada"),
        Row {
            ready: false,
            ..row("bob")
        },
    ];
    let (buf, written) = frame(&mut img, &rows, &faces(), SCREEN, None);
    assert_eq!(rows_sent(&written), vec![TOP]);
    assert!(
        buf[(AUTHOR_X, TOP)].diff_option == ratatui::buffer::CellDiffOption::Skip,
        "ada's picture was not reserved"
    );
    assert!(buf[(AUTHOR_X, TOP + 1)].diff_option != ratatui::buffer::CellDiffOption::Skip);
    assert_eq!(buf[(AUTHOR_X, TOP + 1)].symbol(), "b");
}

#[test]
fn the_picture_replaces_the_initials_once_it_arrives() {
    let mut img = images();
    let loading = vec![Row {
        ready: false,
        ..row("ada")
    }];
    let (_, written) = frame(&mut img, &loading, &faces(), SCREEN, None);
    assert_eq!(written, "");
    let (buf, written) = frame(&mut img, &[row("ada")], &faces(), SCREEN, None);
    assert_eq!(rows_sent(&written), vec![TOP]);
    assert!(buf[(AUTHOR_X, TOP)].diff_option == ratatui::buffer::CellDiffOption::Skip);
}

// ---- the re-emit decision ----

#[test]
fn the_first_frame_sends_every_visible_picture() {
    let rows = vec![row("ada"), row("bob"), row("cyd")];
    let (_, written) = frame(&mut images(), &rows, &faces(), SCREEN, None);
    assert_eq!(rows_sent(&written), vec![TOP, TOP + 1, TOP + 2]);
}

/// The property the whole route rests on: this TUI redraws about thirty
/// times a second, and an idle redraw must cost nothing. Red if
/// `finish_frame` pushes every drawn slot to `pending` regardless of
/// `on_screen`.
#[test]
fn an_unchanged_frame_sends_nothing() {
    let mut img = images();
    let rows = vec![row("ada"), row("bob"), row("cyd")];
    let (one, _) = frame(&mut img, &rows, &faces(), SCREEN, None);
    let (two, written) = frame(&mut img, &rows, &faces(), SCREEN, None);
    assert_eq!(written, "");
    assert!(one.diff(&two).is_empty(), "ratatui would re-emit cells");
}

/// Only the rows that changed go out: the selection moving from row 0
/// to row 1 re-sends those two rows and not the third.
#[test]
fn a_selection_move_sends_only_the_two_rows_it_changed() {
    let mut img = images();
    let mut rows = vec![row("ada"), row("bob"), row("cyd")];
    rows[0].tint = RowTint::Selected;
    let _ = frame(&mut img, &rows, &faces(), SCREEN, None);
    rows[0].tint = RowTint::Plain;
    rows[1].tint = RowTint::Selected;
    let (_, written) = frame(&mut img, &rows, &faces(), SCREEN, None);
    assert_eq!(rows_sent(&written), vec![TOP, TOP + 1]);
}

#[test]
fn a_hover_move_sends_only_the_rows_it_changed() {
    let mut img = images();
    let mut rows = vec![row("ada"), row("bob"), row("cyd")];
    let _ = frame(&mut img, &rows, &faces(), SCREEN, None);
    rows[2].tint = RowTint::Hovered;
    let (_, written) = frame(&mut img, &rows, &faces(), SCREEN, None);
    assert_eq!(rows_sent(&written), vec![TOP + 2]);
}

/// Scrolling by one: every row whose author at that screen line changed
/// is re-sent, and a line that shows the same author as before costs
/// nothing - a one-author stretch of history scrolls for free.
#[test]
fn a_scroll_sends_only_the_lines_whose_author_changed() {
    let mut img = images();
    let before = vec![row("ada"), row("ada"), row("bob"), row("cyd")];
    let _ = frame(&mut img, &before, &faces(), SCREEN, None);
    let after = vec![row("ada"), row("bob"), row("cyd"), row("cyd")];
    let (_, written) = frame(&mut img, &after, &faces(), SCREEN, None);
    assert_eq!(rows_sent(&written), vec![TOP + 1, TOP + 2]);
}

#[test]
fn a_resize_sends_every_picture_again() {
    let mut img = images();
    let rows = vec![row("ada"), row("bob")];
    let _ = frame(&mut img, &rows, &faces(), SCREEN, None);
    let (_, written) = frame(&mut img, &rows, &faces(), Rect::new(0, 0, 44, 14), None);
    assert_eq!(pictures_sent(&written), 2);
}

/// A picture that left the screen (the Diff view took the central
/// column, say) and comes back must be sent again: ratatui painted its
/// cells in between.
#[test]
fn a_picture_that_was_not_drawn_for_a_frame_is_sent_again() {
    let mut img = images();
    let rows = vec![row("ada")];
    let _ = frame(&mut img, &rows, &faces(), SCREEN, None);
    let _ = frame(&mut img, &[], &faces(), SCREEN, None);
    let (_, written) = frame(&mut img, &rows, &faces(), SCREEN, None);
    assert_eq!(pictures_sent(&written), 1);
}

// ---- overlays ----

/// **The trace bug**, per row: a tooltip over an avatar prints into the
/// picture's cells. A slot a popup touches shows no picture at all (a
/// terminal that keeps the half the popup does not cover would draw a
/// half avatar beside it); when the popup closes the picture is sent
/// again, for the rows it crossed and not the others. Red if `covered`
/// is dropped from the `OnScreen` comparison.
#[test]
fn closing_a_popup_over_the_avatars_resends_the_rows_it_covered() {
    let mut img = images();
    let rows = vec![row("ada"), row("bob"), row("cyd")];
    let popup = Some(Rect::new(AUTHOR_X - 3, TOP + 1, 10, 1));
    let _ = frame(&mut img, &rows, &faces(), SCREEN, None);
    let (_, written) = frame(&mut img, &rows, &faces(), SCREEN, popup);
    assert_eq!(pictures_sent(&written), 0, "no picture under a popup");
    assert!(written.contains("oo"), "the popup is written: {written:?}");
    let (_, written) = frame(&mut img, &rows, &faces(), SCREEN, popup);
    assert_eq!(written, "", "a popup that sits still costs nothing");
    let (_, written) = frame(&mut img, &rows, &faces(), SCREEN, None);
    assert_eq!(rows_sent(&written), vec![TOP + 1]);
}

/// A row retinted under a popup that did not move: still no picture, and
/// the popup's cells are written again so nothing of the old picture is
/// left where it crosses.
#[test]
fn a_row_changed_under_a_still_popup_stays_hidden_and_writes_the_popup() {
    let mut img = images();
    let mut rows = vec![row("ada")];
    let popup = Some(Rect::new(AUTHOR_X + 1, TOP, 10, 1));
    let _ = frame(&mut img, &rows, &faces(), SCREEN, popup);
    rows[0].tint = RowTint::Selected;
    let (_, written) = frame(&mut img, &rows, &faces(), SCREEN, popup);
    assert_eq!(pictures_sent(&written), 0);
    // The popup's first character, `a`, sits on the slot's second cell.
    assert!(written.contains('a'), "{written:?}");
}

// ---- composition ----

fn composed(tint: RowTint) -> Bitmap {
    compose(&face((250, 0, 0)), Some(tint), false, CellSize::FALLBACK, 1)
}

#[test]
fn the_picture_is_exactly_the_slot_in_assumed_cell_pixels() {
    let bmp = compose(
        &face((1, 2, 3)),
        Some(RowTint::Plain),
        false,
        CellSize::FALLBACK,
        2,
    );
    assert_eq!(bmp.width, u32::from(SLOT_COLS) * 9 * 2);
    assert_eq!(bmp.height, 21 * 2);
}

/// Round-masked: the center is the face, the corners are the row's
/// background. Red if the inside-test is always true (a square picture).
#[test]
fn the_picture_is_round_with_the_rows_background_in_the_corners() {
    let bmp = composed(RowTint::Plain);
    assert_eq!(bmp.pixel(bmp.width / 2, bmp.height / 2), Some((250, 0, 0)));
    let bg = RowTint::Plain.rgb();
    for (x, y) in [
        (0, 0),
        (bmp.width - 1, 0),
        (0, bmp.height - 1),
        (bmp.width - 1, bmp.height - 1),
    ] {
        assert_eq!(bmp.pixel(x, y), Some(bg), "corner ({x},{y})");
    }
    // Round, not a square inset: the disc's bounding box corner is
    // outside the circle.
    let d = bmp.width.min(bmp.height) - 2;
    let left = (bmp.width - d) / 2;
    let top = (bmp.height - d) / 2;
    assert_eq!(bmp.pixel(left, top), Some(bg));
}

/// Without a backdrop of its own the picture is transparent outside the
/// disc and opaque in its middle, so the cell background shows through.
#[test]
fn a_transparent_picture_has_clear_corners_and_a_solid_face() {
    let bmp = compose(&face((250, 0, 0)), None, false, CellSize::FALLBACK, 1);
    assert_eq!(
        bmp.rgba(bmp.width / 2, bmp.height / 2),
        Some((250, 0, 0, 255))
    );
    for (x, y) in [(0, 0), (bmp.width - 1, bmp.height - 1)] {
        assert_eq!(bmp.rgba(x, y).map(|p| p.3), Some(0), "corner ({x},{y})");
    }
    let dimmed = compose(&face((250, 0, 0)), None, true, CellSize::FALLBACK, 1);
    let a = dimmed.rgba(dimmed.width / 2, dimmed.height / 2).unwrap().3;
    assert!(a < 255 && a > 0, "dimmed fades toward transparent: {a}");
}

/// The selection band crosses the Author column like every other one:
/// a selected row's picture sits on the selection color.
#[test]
fn a_selected_rows_picture_carries_the_selection_background() {
    let bmp = composed(RowTint::Selected);
    assert_eq!(bmp.pixel(0, 0), Some(rgb_of(th().ui.selection)));
    let bmp = composed(RowTint::Hovered);
    assert_eq!(bmp.pixel(0, 0), Some(rgb_of(th().ui.hover)));
}

#[test]
fn a_dimmed_picture_is_faded_toward_the_background() {
    let bmp = compose(
        &face((250, 0, 0)),
        Some(RowTint::Plain),
        true,
        CellSize::FALLBACK,
        1,
    );
    let (r, _, _) = bmp.pixel(bmp.width / 2, bmp.height / 2).unwrap();
    assert!(r < 250 && r > RowTint::Plain.rgb().0, "{r}");
}

#[test]
fn the_edge_is_anti_aliased_not_stair_stepped() {
    let bmp = compose(
        &face((250, 250, 250)),
        Some(RowTint::Plain),
        false,
        CellSize::FALLBACK,
        2,
    );
    let bg = RowTint::Plain.rgb();
    let partial = (0..bmp.height)
        .flat_map(|y| (0..bmp.width).map(move |x| (x, y)))
        .filter_map(|(x, y)| bmp.pixel(x, y))
        .filter(|&p| p != bg && p != (250, 250, 250))
        .count();
    assert!(partial > 0, "no blended edge pixels at all");
}

#[test]
fn an_empty_source_bitmap_draws_the_background_rather_than_panicking() {
    let bmp = compose(
        &Bitmap::new(0, 0, (0, 0, 0)),
        Some(RowTint::Plain),
        false,
        CellSize::FALLBACK,
        1,
    );
    assert_eq!(
        bmp.pixel(bmp.width / 2, bmp.height / 2),
        Some(RowTint::Plain.rgb())
    );
}

/// The frame budget: a full screen of distinct authors composed and
/// encoded from scratch (the worst case: scrolling never re-encodes a row it
/// has encoded before). See the module header of `log_images` and
/// `commit-graph.md`.
#[test]
fn a_full_screen_of_new_authors_encodes_inside_a_frame_budget() {
    let started = std::time::Instant::now();
    let mut bytes = 0;
    for i in 0..60u8 {
        let mut src = Bitmap::new(42, 42, (i, 255 - i, i / 2));
        for y in 0..42 {
            for x in 0..42 {
                src.set_pixel(x, y, (x as u8 * 6, y as u8 * 6, i.wrapping_mul(7)));
            }
        }
        let bmp = compose(&src, Some(RowTint::Plain), false, CellSize::FALLBACK, 2);
        bytes += terminal_image::encode(Protocol::Iterm2, &bmp, SLOT_COLS, 1)
            .unwrap()
            .len();
    }
    let took = started.elapsed();
    // Generous: a debug build on a loaded CI machine. The point is to
    // catch an order-of-magnitude regression, not to benchmark.
    assert!(took < std::time::Duration::from_secs(2), "{took:?}");
    assert!(bytes < 60 * 16 * 1024, "{bytes} bytes");
}

/// The same screenful as sixel (a photographic 42x42 face quantized, at 2x
/// then resampled to the slot's pixels): a first scroll over unseen authors
/// stays well inside a frame, and a slot's bytes stay small.
#[test]
fn a_full_screen_of_new_authors_encodes_as_sixel_inside_a_frame_budget() {
    let started = std::time::Instant::now();
    let mut bytes = 0;
    for i in 0..60u8 {
        let mut src = Bitmap::new(42, 42, (i, 255 - i, i / 2));
        for y in 0..42 {
            for x in 0..42 {
                src.set_pixel(x, y, (x as u8 * 6, y as u8 * 6, i.wrapping_mul(7)));
            }
        }
        let bmp = compose(&src, None, false, CellSize::FALLBACK, 2);
        bytes += terminal_image::encode(Protocol::Sixel, &bmp, SLOT_COLS, 1)
            .unwrap()
            .len();
    }
    let took = started.elapsed();
    assert!(took < std::time::Duration::from_secs(2), "{took:?}");
    assert!(bytes < 60 * 24 * 1024, "{bytes} bytes");
}

// ---- Kitty ----

fn kitty_images() -> LogAvatarImages {
    LogAvatarImages::new(Capability {
        protocol: Protocol::Kitty,
        cell: CellSize::FALLBACK,
    })
}

fn kitty_uploads(written: &str) -> usize {
    written.matches("\x1b_Ga=T,U=1,").count()
}

/// Under Kitty a picture is uploaded once, whatever number of rows show it,
/// and a scroll moves placeholder cells rather than pictures (the iTerm2
/// test above re-sends two rows for the same scroll). Red if `needs_upload`
/// always answers `true`.
#[test]
fn kitty_uploads_each_author_once_and_a_scroll_uploads_nothing() {
    let mut img = kitty_images();
    let before = vec![row("ada"), row("ada"), row("bob"), row("cyd")];
    let (buf, written) = frame(&mut img, &before, &faces(), SCREEN, None);
    assert_eq!(kitty_uploads(&written), 3, "one per author, not per row");
    assert!(
        buf[(AUTHOR_X, TOP)].symbol().starts_with('\u{10EEEE}'),
        "the slot holds placeholder cells"
    );
    let after = vec![row("ada"), row("bob"), row("cyd"), row("cyd")];
    let (_, written) = frame(&mut img, &after, &faces(), SCREEN, None);
    assert_eq!(written, "");
}

/// A popup over a Kitty avatar needs nothing from this module: ratatui
/// wrote its text over the placeholders and writes them back when it
/// closes.
#[test]
fn kitty_avatars_under_a_popup_cost_no_upload_either_way() {
    let mut img = kitty_images();
    let rows = vec![row("ada"), row("bob")];
    let _ = frame(&mut img, &rows, &faces(), SCREEN, None);
    let popup = Some(Rect::new(18, 2, 10, 2));
    let (_, opened) = frame(&mut img, &rows, &faces(), SCREEN, popup);
    assert_eq!(opened, "");
    let (buf, closed) = frame(&mut img, &rows, &faces(), SCREEN, None);
    assert_eq!(closed, "");
    assert!(buf[(AUTHOR_X, TOP)].symbol().starts_with('\u{10EEEE}'));
}

#[test]
fn a_resize_uploads_the_kitty_pictures_again() {
    let mut img = kitty_images();
    let rows = vec![row("ada"), row("bob")];
    let _ = frame(&mut img, &rows, &faces(), SCREEN, None);
    let (_, written) = frame(&mut img, &rows, &faces(), Rect::new(0, 0, 44, 14), None);
    assert_eq!(kitty_uploads(&written), 2);
}

/// With the Log gone from the screen (Settings' page, the Diff view, Home)
/// the avatars are erased: Kitty pictures deleted, iTerm2 cells rewritten,
/// and nothing more is written while it stays away.
#[test]
fn leaving_the_log_erases_the_avatars_once() {
    for kitty in [false, true] {
        let mut img = if kitty { kitty_images() } else { images() };
        let rows: Vec<Row> = (0..8).map(|_| row("ada")).collect();
        let _ = frame(&mut img, &rows, &faces(), SCREEN, None);
        let (_, gone) = frame(&mut img, &[], &faces(), SCREEN, None);
        assert!(!gone.contains("]1337;File="), "{gone:?}");
        if kitty {
            assert!(gone.contains("a=d,d=I"), "{gone:?}");
        } else {
            assert!(gone.contains("\x1b["), "the slots were not rewritten");
        }
        let (_, still) = frame(&mut img, &[], &faces(), SCREEN, None);
        assert_eq!(still, "");
        // Back: the pictures are sent again.
        let (_, back) = frame(&mut img, &rows, &faces(), SCREEN, None);
        assert!(back.contains("]1337;File=") || back.contains("a=T,U=1"));
    }
}

/// A full-pane page drawn over a tall column of slots hides them too.
#[test]
fn a_page_over_the_log_erases_the_avatars_and_sends_nothing_under_it() {
    let mut img = images();
    let rows: Vec<Row> = (0..9).map(|_| row("ada")).collect();
    let _ = frame(&mut img, &rows, &faces(), SCREEN, None);
    let (_, opened) = frame(&mut img, &rows, &faces(), SCREEN, Some(SCREEN));
    assert!(!opened.contains("]1337;File="), "{opened:?}");
    let (_, steady) = frame(&mut img, &rows, &faces(), SCREEN, Some(SCREEN));
    assert_eq!(steady, "");
    let (_, closed) = frame(&mut img, &rows, &faces(), SCREEN, None);
    assert!(closed.contains("]1337;File="));
}

/// A dialog over the live Log (the Activity Log, the Undo history) is not a
/// page: with `keep_under_overlay` the pictures are not erased however many
/// slots it covers. Kitty: no delete. iTerm2: no picture is written inside
/// the dialog (its cells are rewritten over them) and none outside is sent
/// again. Red without the flag: the whole column is erased.
#[test]
fn a_dialog_over_the_log_keeps_the_avatars_it_does_not_cover() {
    for (kitty, rows_covered) in [(false, 7), (true, 7), (false, 12), (true, 12)] {
        let mut img = if kitty { kitty_images() } else { images() };
        img.keep_under_overlay(true);
        let rows: Vec<Row> = (0..9).map(|_| row("ada")).collect();
        let _ = frame(&mut img, &rows, &faces(), SCREEN, None);
        // The dialog covers all but the last rows (7), or every row (12).
        let dialog = Rect::new(AUTHOR_X - 4, 0, 20, rows_covered);
        let (buf, opened) = frame(&mut img, &rows, &faces(), SCREEN, Some(dialog));
        assert!(!opened.contains("a=d"), "kitty={kitty}: deleted {opened:?}");
        assert!(
            !opened.contains("]1337;File="),
            "kitty={kitty}: a picture was sent under the dialog {opened:?}"
        );
        for y in dialog.top()..dialog.bottom() {
            for x in dialog.left()..dialog.right() {
                assert!(
                    !buf[(x, y)].symbol().starts_with('\u{10EEEE}'),
                    "kitty={kitty}: a placeholder inside the dialog at {x},{y}"
                );
            }
        }
        if rows_covered >= SCREEN.height {
            continue;
        }
        // The rows below the dialog still hold their picture.
        let kept = if kitty {
            buf[(AUTHOR_X, TOP + 8)].symbol().starts_with('\u{10EEEE}')
        } else {
            buf[(AUTHOR_X, TOP + 8)].diff_option == ratatui::buffer::CellDiffOption::Skip
        };
        assert!(kept, "kitty={kitty}: the picture outside the dialog is gone");
    }
}

/// The transparent iTerm2 path (WezTerm): a page whose text sits in one
/// corner and which is blank over the avatars (Settings under its last
/// section) must still erase them. A blank cell equals the blank the
/// picture sat on, so only the reservation's mark tells it was covered.
/// Red before the mark: nothing at all is written and the avatars stay.
#[test]
fn a_mostly_blank_page_over_the_log_erases_transparent_avatars() {
    let mut img = LogAvatarImages::new(Capability {
        protocol: Protocol::Iterm2Alpha,
        cell: CellSize::FALLBACK,
    });
    let rows: Vec<Row> = (0..9).map(|_| row("ada")).collect();
    let _ = frame(&mut img, &rows, &faces(), SCREEN, None);
    // The helper's overlay writes one line of text on the page's first row.
    let (_, opened) = frame(&mut img, &rows, &faces(), SCREEN, Some(SCREEN));
    assert!(!opened.contains("]1337;File="), "{opened:?}");
    assert!(
        opened.contains("["),
        "the avatars' cells were not rewritten"
    );
    let (_, steady) = frame(&mut img, &rows, &faces(), SCREEN, Some(SCREEN));
    assert_eq!(steady, "");
    let (_, closed) = frame(&mut img, &rows, &faces(), SCREEN, None);
    assert!(closed.contains("]1337;File="));
}

// ---- a modelled terminal: no picture ever shows over a popup ----

mod on_a_terminal {
    use super::*;
    use crate::terminal_image::vt_sim::Vt;

    fn images_for(protocol: Protocol) -> LogAvatarImages {
        LogAvatarImages::new(Capability {
            protocol,
            cell: CellSize::FALLBACK,
        })
    }

    /// A screen: the real draw order, flushed into a modelled terminal.
    struct Screen {
        img: LogAvatarImages,
        vt: Vt,
        prev: Buffer,
        rows: Vec<Row>,
    }

    impl Screen {
        fn new(protocol: Protocol, rows: Vec<Row>) -> Screen {
            Screen {
                img: images_for(protocol),
                vt: Vt::new(SCREEN),
                prev: Buffer::empty(SCREEN),
                rows,
            }
        }

        fn show(&mut self, popup: Option<Rect>) {
            let rows = self.rows.clone();
            let (buf, written) = frame(&mut self.img, &rows, &faces(), SCREEN, popup);
            self.vt.draw(&self.prev, &buf);
            self.vt.feed(written.as_bytes());
            self.prev = buf;
        }

        fn slot(&self, i: u16) -> Rect {
            Rect::new(AUTHOR_X, TOP + i, SLOT_COLS, 1)
        }

        /// No tile on a cell the popup covers, nor on a cell of a slot
        /// the popup touches (a half picture is as wrong as a whole one).
        fn assert_clean_under(&self, popup: Rect) {
            for i in 0..self.rows.len() as u16 {
                let slot = self.slot(i);
                if slot.intersects(popup) {
                    assert_eq!(
                        self.vt.tiles_in(slot),
                        Vec::<(u16, u16)>::new(),
                        "slot {i} touched by the popup still carries a picture"
                    );
                }
            }
        }
    }

    const PROTOCOLS: [Protocol; 3] = [Protocol::Iterm2, Protocol::Iterm2Alpha, Protocol::Sixel];

    /// The About window's left border crossing the right cell of a slot.
    #[test]
    fn a_slot_a_popup_crosses_shows_no_picture_and_comes_back_on_close() {
        for protocol in PROTOCOLS {
            let mut s = Screen::new(protocol, vec![row("ada"), row("bob"), row("cyd")]);
            s.show(None);
            assert_eq!(s.vt.tiles_in(s.slot(1)).len(), 2, "{protocol:?}");
            // Covers the second cell of bob's slot and everything right of it.
            let popup = Rect::new(AUTHOR_X + 1, TOP + 1, 10, 1);
            s.show(Some(popup));
            s.assert_clean_under(popup);
            assert_eq!(s.vt.tiles_in(s.slot(0)).len(), 2, "{protocol:?}: neighbor");
            assert_eq!(s.vt.tiles_in(s.slot(2)).len(), 2, "{protocol:?}: neighbor");
            s.show(Some(popup));
            s.assert_clean_under(popup);
            s.show(None);
            assert_eq!(s.vt.tiles_in(s.slot(1)).len(), 2, "{protocol:?}: restored");
        }
    }

    /// The popup opens over a slot that was already there, and then moves.
    #[test]
    fn a_popup_that_moves_across_the_slots_leaves_no_picture_under_it() {
        for protocol in PROTOCOLS {
            let mut s = Screen::new(protocol, vec![row("ada"), row("bob"), row("cyd")]);
            s.show(None);
            for popup in [
                Rect::new(AUTHOR_X - 4, TOP, 5, 1),
                Rect::new(AUTHOR_X - 4, TOP, 5, 2),
                Rect::new(AUTHOR_X + 1, TOP + 1, 6, 2),
                Rect::new(AUTHOR_X + 2, TOP, 6, 3),
            ] {
                s.show(Some(popup));
                s.assert_clean_under(popup);
            }
            s.show(None);
            for i in 0..3 {
                assert_eq!(s.vt.tiles_in(s.slot(i)).len(), 2, "{protocol:?} slot {i}");
            }
        }
    }

    /// A scroll under a standing popup.
    #[test]
    fn scrolling_under_a_popup_keeps_it_clean() {
        for protocol in PROTOCOLS {
            let mut s = Screen::new(protocol, vec![row("ada"), row("bob"), row("cyd")]);
            let popup = Rect::new(AUTHOR_X + 1, TOP + 1, 10, 2);
            s.show(None);
            s.show(Some(popup));
            s.rows = vec![row("cyd"), row("ada"), row("bob")];
            s.show(Some(popup));
            s.assert_clean_under(popup);
            s.rows = vec![row("bob"), row("cyd"), row("ada")];
            s.show(Some(popup));
            s.assert_clean_under(popup);
        }
    }
}
