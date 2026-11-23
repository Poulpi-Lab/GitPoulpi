//! The column's Graph part: the lanes it draws as text, the rows its picture
//! is handed, the room the lanes get, and the strip that brings it back.

use std::collections::HashSet;

use gitpoulpi_core::domain::{CommitGraph, CommitNode, Oid, Stash, StashId};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

use super::picture::{plan_widths, plan_widths_with, raster_rows, ColumnPicture, MIN_TITLE};
use super::*;
use crate::graph::{
    self, GraphRender, GraphSettings, LayoutInputs, LogLayout, LogLayoutCache, NodeSet,
};

fn oid(i: usize) -> Oid {
    let mut b = [0u8; 20];
    b[..8].copy_from_slice(&(i as u64 + 1).to_be_bytes());
    Oid(b)
}

fn node(i: usize, parents: Vec<Oid>) -> CommitNode {
    CommitNode {
        id: oid(i),
        parents,
        summary: format!("commit {i}"),
        message: format!("commit {i}\n\nbody"),
        author_name: "Alice Doe".into(),
        author_email: "a@example.com".into(),
        timestamp: 1_700_000_000 - i as i64,
        refs: Vec::new(),
    }
}

/// Newest first, a side branch merged back every 7th commit.
fn history(n: usize) -> CommitGraph {
    CommitGraph {
        nodes: (0..n)
            .map(|i| {
                let mut parents = Vec::new();
                if i + 1 < n {
                    parents.push(oid(i + 1));
                }
                if i % 7 == 3 && i + 2 < n {
                    parents.push(oid(i + 2));
                }
                node(i, parents)
            })
            .collect(),
    }
}

/// `tips` branch tips that all sit on one root: `tips` lanes side by side.
fn fan(tips: usize) -> CommitGraph {
    let mut nodes: Vec<CommitNode> = (0..tips).map(|i| node(i, vec![oid(tips)])).collect();
    nodes.push(node(tips, Vec::new()));
    CommitGraph { nodes }
}

fn stash_on(base: usize) -> Stash {
    Stash {
        id: StashId(0),
        message: "saved".into(),
        branch_at_creation: "main".into(),
        timestamp: 1_700_000_000,
        commit: Some(oid(900)),
        base: Some(oid(base)),
    }
}

/// The Log's layout for `graph`, as the window builds it.
fn layout_cache(graph: CommitGraph, wip: bool, stashes: Vec<Stash>) -> LogLayoutCache {
    layout_cache_with(graph, wip, stashes, GraphSettings::default())
}

fn layout_cache_with(
    graph: CommitGraph,
    wip: bool,
    stashes: Vec<Stash>,
    settings: GraphSettings,
) -> LogLayoutCache {
    let log = Ok(graph);
    let refs = Ok(Vec::new());
    let stashes = Ok(stashes);
    let none = HashSet::new();
    let mut cache = LogLayoutCache::default();
    cache.update(
        &LayoutInputs {
            log: &log,
            log_gen: 1,
            refs: &refs,
            refs_gen: 1,
            stashes: &stashes,
            stashes_gen: 1,
            solo: &none,
            hidden: &none,
            has_wip: wip,
            show_stashes: true,
            settings: GraphSettings {
                render: GraphRender::Text,
                ..settings
            },
        },
        None,
    );
    cache
}

static NO_CHANGES: std::sync::LazyLock<
    std::collections::HashMap<Oid, gitpoulpi_core::domain::CommitStat>,
> = std::sync::LazyLock::new(Default::default);

fn paint_in<'a>(layout: &'a LogLayout, selected: Option<Oid>) -> ColumnPaint<'a> {
    ColumnPaint {
        changes: &NO_CHANGES,
        layout,
        selected,
        hovered: None,
        focused: false,
        more: false,
        node_set: NodeSet::default(),
    }
}

fn draw(layout: &LogLayout, state: &mut CommitColumn, width: u16, height: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|f| {
            let area = f.area();
            render(f, area, state, &paint_in(layout, None), None);
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

/// The focused column's frame is `ui.border_focused`, never `ui.accent`.
#[test]
fn a_focused_column_is_framed_in_border_focused() {
    let mut theme = crate::theme::Theme::dark();
    theme.ui.accent = ratatui::style::Color::Rgb(1, 2, 3);
    theme.ui.border_focused = ratatui::style::Color::Rgb(231, 223, 0);
    let _guard = crate::theme::override_theme(&theme);
    let cache = layout_cache(history(5), false, Vec::new());
    let layout = cache.layout();
    let mut state = CommitColumn::new(0, 0);
    let mut paint = paint_in(layout, None);
    paint.focused = true;
    let mut terminal = Terminal::new(TestBackend::new(60, 12)).unwrap();
    terminal
        .draw(|f| {
            let area = f.area();
            render(f, area, &mut state, &paint, None);
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    assert_eq!(buf[(0, 0)].fg, theme.ui.border_focused);
    assert_ne!(buf[(0, 0)].fg, theme.ui.accent);
}

fn row_text(buf: &Buffer, y: u16) -> String {
    (0..buf.area.width)
        .map(|x| buf[(x, y)].symbol().to_string())
        .collect()
}

fn lanes_text(layout: &LogLayout, commit: usize) -> String {
    graph::text_line(&layout.grid[commit], NodeSet::default())
        .spans
        .iter()
        .map(|s| s.content.to_string())
        .collect()
}

/// The lanes of a commit are drawn whole, as the Log's text graph draws
/// them, whatever their number: a graph of 16 lanes is not cut at 12.
/// Red when a cap on the lanes' cells comes back.
#[test]
fn the_text_lanes_are_whole_when_the_room_is_there() {
    let cache = layout_cache(fan(16), false, Vec::new());
    let layout = cache.layout();
    let needed = picture::lanes_needed(layout, 0, 17);
    assert!(needed > 12, "the fixture must be wider than the old cap");
    let mut state = CommitColumn::new(0, 0);
    let buf = draw(layout, &mut state, 90, 20);
    for commit in 0..17 {
        let lanes = lanes_text(layout, commit);
        let row = row_text(&buf, 2 + commit as u16);
        assert!(
            row.contains(lanes.as_str()),
            "row {commit} lost lanes: {row:?} does not hold {lanes:?}"
        );
    }
}

/// Plain history: each row starts with its grid row's glyphs, one line
/// per commit, so the lanes run unbroken from row to row.
#[test]
fn the_text_lanes_are_the_logs_text_graph_row_by_row() {
    let cache = layout_cache(history(30), true, vec![stash_on(5)]);
    let layout = cache.layout();
    let mut state = CommitColumn::new(0, 0);
    let buf = draw(layout, &mut state, 60, 20);
    let first = 2;
    for commit in 0..17 {
        let lanes = lanes_text(layout, commit);
        let row = row_text(&buf, first + commit as u16);
        // Just inside the left border.
        let inside: String = row.chars().skip(1).collect();
        assert!(
            inside.starts_with(lanes.as_str()),
            "row {commit}: {inside:?} should start with {lanes:?}"
        );
    }
}

#[test]
fn the_lanes_take_the_room_before_the_author_and_the_date_do() {
    // Roomy: everything.
    let w = plan_widths(60, 16);
    assert_eq!((w.graph, w.author, w.date), (16, 8, 5));
    // 16 lanes + air + the title floor + date: no author.
    let w = plan_widths(16 + 1 + MIN_TITLE + 6, 16);
    assert_eq!((w.graph, w.author, w.date), (16, 0, 5));
    // Then no date either.
    let w = plan_widths(16 + 1 + MIN_TITLE, 16);
    assert_eq!((w.graph, w.author, w.date), (16, 0, 0));
    // Too narrow even so: the lanes are cut, the title keeps its floor.
    let w = plan_widths(22, 16);
    assert_eq!(w.graph, 22 - MIN_TITLE - 1);
    assert!(w.author == 0 && w.date == 0);
    // No lanes, no air.
    assert_eq!(plan_widths(40, 0).graph, 0);
}

/// The picture's rows are the Log's rows for the same commits, whatever the
/// Log put between them (the WIP row, a stash row): the column skips those,
/// and the lanes stay the commits' own. Red when the walk goes through the
/// Log's display indices, or the WIP / stash rows are included.
#[test]
fn the_pictures_rows_are_the_logs_rows_for_the_same_commits() {
    let cache = layout_cache(history(30), true, vec![stash_on(5)]);
    let layout = cache.layout();
    assert!(layout.wip_row.is_some() && !layout.stash_rows.is_empty());
    let commits = layout.nodes.as_ref().unwrap().len();
    // The Log's own rows, from its top: WIP, stash rows and commits.
    let log_len = layout.order.len(commits) as u16;
    let log_rows = graph::visible_raster_rows(
        &layout.grid,
        layout.wip_row.as_ref(),
        &layout.stash_rows,
        &layout.order,
        &graph::ColumnWindow {
            rows: log_len,
            offset: 0,
            selected: None,
        },
    );
    assert_eq!(log_rows.len(), usize::from(log_len));
    for offset in [0usize, 4, 11] {
        let mine = raster_rows(layout, offset, 10, Some(offset + 2));
        assert_eq!(mine.len(), 10);
        for (k, row) in mine.iter().enumerate() {
            let commit = offset + k;
            let theirs = &log_rows[layout.order.commit_display_index(commit)];
            assert_eq!(row.cells, theirs.cells, "commit {commit} from {offset}");
            assert_eq!(row.selected, k == 2, "the selection is the column's own");
        }
    }
}

/// A row past the end of the commits is not invented.
#[test]
fn the_pictures_rows_stop_with_the_commits() {
    let cache = layout_cache(history(5), false, Vec::new());
    assert_eq!(raster_rows(cache.layout(), 3, 10, None).len(), 2);
    assert!(raster_rows(cache.layout(), 9, 10, None).is_empty());
}

/// Hidden but able to come back: the strip is painted at the view's left,
/// registered as one hit over its whole height, and its tab holds a chevron.
#[test]
fn the_strip_is_a_hit_over_its_whole_height() {
    let mut terminal = Terminal::new(TestBackend::new(40, 12)).unwrap();
    let mut state = CommitColumn::new(0, 0);
    let mut hits = Vec::new();
    terminal
        .draw(|f| {
            hits = render_strip(f, Rect::new(5, 2, 2, 8), &mut state, false);
        })
        .unwrap();
    assert_eq!(
        hits,
        vec![(
            Rect::new(6, 2, 1, 8),
            crate::chrome::HitTarget::CommitColumn(ColumnHit::Reopen)
        )]
    );
    let buf = terminal.backend().buffer();
    let tab: String = (2..10).map(|y| buf[(6, y)].symbol().to_string()).collect();
    assert!(tab.contains('\u{203A}'), "{tab:?}");
    // And forgotten when the frame has none.
    state.hide();
    assert!(state.strip.is_empty());
}

/// On a terminal with an image protocol the lanes are a picture, sent once
/// over blank cells, and not characters; without one, characters and no
/// picture. Red when the column ignores the image path (the glyphs show and
/// nothing is sent), or draws both.
#[test]
fn the_lanes_are_a_picture_where_the_log_draws_one() {
    use crate::avatar::AvatarService;
    use crate::terminal_image::{Capability, CellSize, Protocol};

    let cache = layout_cache(history(30), false, Vec::new());
    let layout = cache.layout();
    let dir = tempfile::tempdir().unwrap();
    let mut service = AvatarService::spawn(
        dir.path().to_path_buf(),
        16,
        Box::new(|| None),
        Box::new(|_| None),
    );
    let mut sent = |protocol: Protocol| {
        let mut image = graph::GraphImage::new(Capability {
            protocol,
            cell: CellSize::FALLBACK,
        });
        let mut state = CommitColumn::new(0, 0);
        let mut terminal = Terminal::new(TestBackend::new(60, 20)).unwrap();
        terminal
            .draw(|f| {
                let mut picture = ColumnPicture {
                    image: &mut image,
                    avatars: false,
                    service: &mut service,
                    lane_background: graph::LaneBackground::Off,
                    tint_strength: Default::default(),
                };
                let area = f.area();
                render(
                    f,
                    area,
                    &mut state,
                    &paint_in(layout, None),
                    Some(&mut picture),
                );
                image.finish_frame(f.buffer_mut());
            })
            .unwrap();
        let mut out = Vec::new();
        image.write_pending(&mut out).unwrap();
        let buf = terminal.backend().buffer().clone();
        let text: String = (1..19).map(|y| row_text(&buf, y)).collect();
        (String::from_utf8(out).unwrap(), text, state)
    };
    let lanes: String = (0..18)
        .map(|c| lanes_text(layout, c).trim().to_string())
        .max_by_key(|l| l.chars().count())
        .unwrap();
    assert!(lanes.chars().any(|c| c != ' '), "the fixture has lanes");

    let (written, text, state) = sent(Protocol::Iterm2);
    assert!(
        written.contains("\x1b]1337;File="),
        "the picture was not sent"
    );
    assert!(!text.contains(&lanes), "glyphs drawn under the picture");
    assert!(state.len > 0);

    let (written, text, _) = sent(Protocol::None);
    assert_eq!(written, "", "no picture without a protocol");
    assert!(text.contains(&lanes), "the text fallback is the Log's");
}

/// A part the user sized is kept, never given up for the title: what does
/// not fit makes the row wider than the room (it scrolls); what fits leaves
/// the title the rest. Red when the shrink order of the automatic widths
/// applies to a width the user chose.
#[test]
fn the_widths_the_user_chose_are_kept_and_the_row_scrolls_when_they_do_not_fit() {
    // Nothing chosen: the automatic rule, the row is as wide as the room.
    let auto = plan_widths_with(44, 4, [0; 7]);
    assert_eq!(auto, plan_widths(44, 4));
    assert_eq!(auto.content, 44);
    // Roomy: the title takes what is left, the row fits.
    let roomy = plan_widths_with(80, 4, [0, 20, 0, 0, 0, 0, 0]);
    assert_eq!((roomy.author, roomy.content), (20, 80));
    assert_eq!(roomy.title, 80 - (4 + 1) - (20 + 1) - (5 + 1));
    // Too narrow for the author asked for: the title keeps its floor, the
    // author is not hidden, and the row is wider than the room.
    let tight = plan_widths_with(44, 4, [0, 32, 0, 0, 0, 0, 0]);
    assert_eq!((tight.author, tight.title), (32, MIN_TITLE));
    assert!(tight.content > 44, "{tight:?}");
    // The parts are laid out left to right with one cell between.
    let spans = tight.spans();
    assert!(spans.windows(2).all(|w| w[0].1 + w[0].2 + 1 == w[1].1));
}

/// A row scrolled sideways shows the cells from the offset on, the header
/// with it, and the picture gives way to characters while it is cut.
#[test]
fn a_scrolled_row_and_its_header_start_at_the_offset() {
    let cache = layout_cache(history(10), false, Vec::new());
    let layout = cache.layout();
    let mut state = CommitColumn::new(0, 0);
    state.widths = [0, 30, 0, 0, 0, 0, 0];
    let first = draw(layout, &mut state, 40, 12);
    assert!(
        state.max_h_offset() > 0,
        "the author asked for does not fit"
    );
    assert!(row_text(&first, 1).contains("Message"));
    state.scroll_h(100);
    let by = state.h_offset;
    assert!(by > 0, "it scrolled");
    let moved = draw(layout, &mut state, 40, 12);
    let header = |b: &Buffer| row_text(b, 1);
    // In cells (the rules are three bytes each), not bytes.
    let at = |b: &Buffer, word: &str| {
        let line = header(b);
        line.find(word).map(|i| line[..i].chars().count())
    };
    assert_eq!(
        at(&first, "Author").unwrap(),
        at(&moved, "Author").unwrap() + by
    );
    assert!(
        at(&moved, "Message").is_none()
            || at(&moved, "Message").unwrap() < at(&first, "Message").unwrap()
    );
}

/// The Log's header grab: a part's boundary is the first letter of the next
/// part's name, no rule is drawn between the names, and the cell is the one
/// the hit is registered on (so the hover paint lights that letter).
#[test]
fn a_boundary_is_the_first_letter_of_the_next_name_with_no_rule() {
    // The `header` mode (the first letter of the next name is the grab).
    crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Header);
    let cache = layout_cache(history(10), false, Vec::new());
    let layout = cache.layout();
    let mut state = CommitColumn::new(0, 0);
    let buf = draw(layout, &mut state, 60, 12);
    let header = row_text(&buf, 1);
    // The two border columns are the only rules on that row.
    assert_eq!(header.matches('\u{2502}').count(), 2, "{header}");
    let (part, rect) = state
        .boundaries
        .iter()
        .find(|(p, _)| *p == Part::Title)
        .copied()
        .expect("the message | author boundary");
    assert_eq!(part, Part::Title);
    assert_eq!(
        buf[(rect.x, rect.y)].symbol(),
        "A",
        "the author's first letter"
    );
}

/// One frame of the column on a terminal with `protocol`, the second after
/// the row was scrolled `scroll` cells sideways: what the terminal was sent,
/// and the cells inside the border.
fn paint_picture(
    layout: &LogLayout,
    widths: [u16; 7],
    cols: u16,
    scroll: usize,
    avatars: bool,
) -> (String, Vec<String>) {
    use crate::avatar::AvatarService;
    use crate::terminal_image::{Bitmap, Capability, CellSize, Protocol};

    let dir = tempfile::tempdir().unwrap();
    let mut service = AvatarService::spawn(
        dir.path().to_path_buf(),
        16,
        Box::new(|| None),
        Box::new(|_| None),
    );
    // The author's picture is ready: no job, no network.
    service.seed_picture("a@example.com", Bitmap::new(42, 42, (200, 30, 30)));
    let mut image = graph::GraphImage::new(Capability {
        protocol: Protocol::Iterm2,
        cell: CellSize::FALLBACK,
    });
    let mut state = CommitColumn::new(0, 0);
    state.widths = widths;
    let mut terminal = Terminal::new(TestBackend::new(cols, 20)).unwrap();
    for frame in 0..=usize::from(scroll > 0) {
        if frame == 1 {
            state.scroll_h(scroll as isize);
        }
        terminal
            .draw(|f| {
                let mut picture = ColumnPicture {
                    image: &mut image,
                    avatars,
                    service: &mut service,
                    lane_background: graph::LaneBackground::Off,
                    tint_strength: Default::default(),
                };
                let area = f.area();
                render(
                    f,
                    area,
                    &mut state,
                    &paint_in(layout, None),
                    Some(&mut picture),
                );
                image.finish_frame(f.buffer_mut());
            })
            .unwrap();
        if frame == 0 && scroll > 0 {
            // Only the scrolled frame's bytes count.
            image.write_pending(&mut Vec::new()).unwrap();
        }
    }
    assert_eq!(state.h_offset, scroll, "the row scrolled");
    let mut out = Vec::new();
    image.write_pending(&mut out).unwrap();
    let buf = terminal.backend().buffer().clone();
    let cells = (2..19)
        .map(|y| {
            (1..cols - 1)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect()
        })
        .collect();
    (String::from_utf8(out).unwrap(), cells)
}

/// The lanes stay a picture, with the authors' faces in the circles, when the
/// row is wider than the pane: scrolled sideways (the Graph part cut at the
/// left) or a Graph part wider than the room (cut at the right). They were
/// characters (no avatar) as soon as the picture did not fit whole. Red when
/// `as_picture` goes back to needing the whole Graph part on screen from the
/// row's start.
#[test]
fn a_graph_cut_by_the_pane_stays_a_picture_with_its_avatars() {
    let cache = layout_cache(history(30), false, Vec::new());
    let layout = cache.layout();
    let glyphs = "\u{25cb}\u{25cf}\u{25ce}\u{256d}\u{256e}\u{256f}\u{2570}\u{2502}";
    // Scrolled two cells: the Graph part is cut at its left edge.
    // Graph part (60) wider than the pane (40): cut at its right edge.
    for (name, widths, scroll) in [
        ("scrolled", [0, 30, 0, 0, 0, 0, 0], 2usize),
        ("wider than the pane", [60, 0, 0, 0, 0, 0, 0], 0usize),
    ] {
        let (with, cells) = paint_picture(layout, widths, 40, scroll, true);
        assert!(
            with.contains("\x1b]1337;File="),
            "{name}: the picture was not sent"
        );
        for row in &cells {
            assert!(
                !row.chars().any(|c| glyphs.contains(c)),
                "{name}: lanes drawn as characters: {row:?}"
            );
        }
        let (without, _) = paint_picture(layout, widths, 40, scroll, false);
        assert!(without.contains("\x1b]1337;File="), "{name}");
        assert_ne!(with, without, "{name}: the avatars are not in the picture");
    }
}

/// The Graph part at its least width: one dot per row, no branch or merge
/// line, whatever the history; one cell wider the lanes come back. Both as
/// characters and as a picture. Red when the width stops deciding.
#[test]
fn at_its_least_width_the_graph_part_is_one_dot_per_row_and_one_cell_more_restores_the_lanes() {
    let cache = layout_cache(history(30), false, Vec::new());
    let layout = cache.layout();
    let strokes = "\u{256d}\u{256e}\u{256f}\u{2570}\u{2502}\u{2500}\u{253c}\u{251c}\u{2524}";
    let dots = "\u{25cb}\u{25cf}\u{25ce}";
    let graph_cells = |buf: &Buffer, y: u16, w: u16| -> String {
        (1..1 + w)
            .map(|x| buf[(x, y)].symbol().to_string())
            .collect()
    };
    let mut state = CommitColumn::new(0, 0);
    state.widths = [graph::COMPACT_W as u16, 0, 0, 0, 0, 0, 0];
    let buf = draw(layout, &mut state, 60, 20);
    assert_eq!(state.plan.graph, graph::COMPACT_W);
    for y in 2..19 {
        let cells = graph_cells(&buf, y, 2);
        assert!(
            cells.chars().next().is_some_and(|c| dots.contains(c)),
            "row {y}: {cells:?}"
        );
        assert!(
            !cells.chars().any(|c| strokes.contains(c)),
            "row {y} draws a line: {cells:?}"
        );
    }
    let mut state = CommitColumn::new(0, 0);
    state.widths = [graph::COMPACT_W as u16 + 1, 0, 0, 0, 0, 0, 0];
    let buf = draw(layout, &mut state, 60, 20);
    assert!(
        (2..19).any(|y| graph_cells(&buf, y, 3).chars().any(|c| strokes.contains(c))),
        "one cell wider has its lanes back"
    );

    // As a picture: the compact rows are one cell wide, the tree's are not.
    let widest = |compact: bool| {
        let mut rows = raster_rows(layout, 0, 17, None);
        if compact {
            graph::compact_raster_rows(&mut rows);
        }
        rows.iter().map(|r| r.cells.len()).max().unwrap()
    };
    assert_eq!(widest(true), 1);
    assert!(widest(false) > 2);
}

/// `line` mode: a tick rests in each separator of the header, the registered
/// grab is that separator down the header and the rows, and a lit boundary is
/// the thin line; `header` mode has neither tick nor tall grab.
#[test]
fn line_mode_ticks_the_header_and_grabs_the_separator_down_the_column() {
    use gitpoulpi_config::ColumnResize;
    let cache = layout_cache(history(10), false, Vec::new());
    let layout = cache.layout();

    crate::live_settings::install_column_resize(ColumnResize::Line);
    let mut state = CommitColumn::new(0, 0);
    let mut buf = draw(layout, &mut state, 60, 12);
    let header_y = state.rows_area.y - 1;
    let (_, rect) = state
        .boundaries
        .iter()
        .find(|(p, _)| *p == Part::Title)
        .copied()
        .expect("the message | author boundary");
    assert_eq!(rect.y, header_y);
    assert_eq!(rect.height, 1 + state.rows_area.height, "header and rows");
    assert_eq!(
        buf[(rect.x, header_y)].symbol(),
        "\u{2502}",
        "the resting tick"
    );
    // The separator in a body row is blank (nothing drawn until hovered).
    assert_eq!(buf[(rect.x, header_y + 1)].symbol(), " ");
    // Hovered: the thin line from the header to the last row.
    crate::column_resize::paint_grab(&mut buf, ColumnResize::Line, rect);
    for y in rect.y..rect.bottom() {
        assert_eq!(buf[(rect.x, y)].symbol(), "\u{2502}", "row {y}");
    }
    // The rows' hit rects come after it: the boundary wins on its cells.
    let mut hits_state = CommitColumn::new(0, 0);
    let mut terminal = Terminal::new(TestBackend::new(60, 12)).unwrap();
    let mut hits = Vec::new();
    terminal
        .draw(|f| {
            let area = f.area();
            hits = render(f, area, &mut hits_state, &paint_in(layout, None), None);
        })
        .unwrap();
    let at = (rect.x, header_y + 3);
    let first = hits.iter().find(|(r, _)| r.contains(at.into())).unwrap();
    assert_eq!(
        first.1,
        crate::HitTarget::CommitColumn(ColumnHit::Boundary(Part::Title))
    );

    crate::live_settings::install_column_resize(ColumnResize::Header);
    let mut state = CommitColumn::new(0, 0);
    let buf = draw(layout, &mut state, 60, 12);
    for x in 1..59 {
        assert_ne!(
            buf[(x, header_y)].symbol(),
            "\u{2502}",
            "no tick in header mode"
        );
    }
    let (_, rect) = state
        .boundaries
        .iter()
        .find(|(p, _)| *p == Part::Title)
        .copied()
        .unwrap();
    assert_eq!(rect.height, 1, "the header cell only");
}

/// The gap between two parts of a selected row carries the selection color,
/// so the row is one band in either mode.
#[test]
fn the_selection_runs_through_the_separators_of_the_column() {
    use gitpoulpi_config::ColumnResize;
    let cache = layout_cache(history(10), false, Vec::new());
    let layout = cache.layout();
    for mode in [ColumnResize::Line, ColumnResize::Header] {
        crate::live_settings::install_column_resize(mode);
        let mut state = CommitColumn::new(0, 0);
        let mut terminal = Terminal::new(TestBackend::new(60, 12)).unwrap();
        let first = layout.nodes.as_deref().unwrap()[0].id;
        terminal
            .draw(|f| {
                let area = f.area();
                let mut paint = paint_in(layout, None);
                paint.selected = Some(first);
                render(f, area, &mut state, &paint, None);
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        let y = state.rows_area.y;
        let (_, rect) = state
            .boundaries
            .iter()
            .find(|(p, _)| *p == Part::Title)
            .copied()
            .unwrap();
        let gap_x = match mode {
            ColumnResize::Line => rect.x,
            ColumnResize::Header => rect.x - 1,
        };
        assert_eq!(buf[(gap_x, y)].bg, crate::th().ui.selection, "{mode:?}");
        assert_eq!(buf[(gap_x + 1, y)].bg, crate::th().ui.selection, "{mode:?}");
    }
}

/// The Log and the commit column, one tiny history, the same row tint: the
/// cell right before the next part carries the tint (at rest) and the thin
/// line (on hover), with no plain cell between the band and that part. Red
/// when the column's tint stops one cell short, at the last lane cell, and
/// leaves the spacer plain.
#[test]
fn the_row_tint_runs_to_the_boundary_cell_in_the_log_and_in_the_commit_column() {
    use crate::log_table::{column_x_offsets, commit_row, LogColumn, LogColumnWidths};
    use gitpoulpi_config::ColumnResize;
    use ratatui::layout::Rect;
    use ratatui::style::Style;
    use ratatui::widgets::Cell;

    crate::live_settings::install_column_resize(ColumnResize::Line);
    let settings = GraphSettings {
        lane_background: graph::LaneBackground::RowTint,
        ..Default::default()
    };
    let hist = history(4);
    let nodes = hist.nodes.clone();
    let cache = layout_cache_with(hist, false, Vec::new(), settings);
    let layout = cache.layout();
    let plain = Style::default().bg;
    let tinted = |buf: &Buffer, x: u16, y: u16| {
        let bg = buf[(x, y)].bg;
        bg != ratatui::style::Color::Reset && Some(bg) != plain
    };

    // The Log: its real table, the Graph cell of every row.
    let log_area = Rect::new(0, 0, 100, 8);
    let widths = LogColumnWidths::for_area_width(
        log_area.width,
        &[LogColumn::Changes].into(),
        &std::collections::HashMap::new(),
    );
    let order: Vec<LogColumn> = LogColumn::ALL
        .into_iter()
        .filter(|c| *c != LogColumn::Changes)
        .collect();
    let offsets = column_x_offsets(0, &order, &widths);
    let (_, gx, gw) = *offsets
        .iter()
        .find(|(c, _, _)| *c == LogColumn::Graph)
        .unwrap();
    let rows: Vec<_> = nodes
        .iter()
        .zip(&layout.rows)
        .map(|(c, (line, color))| {
            commit_row(
                c,
                &[],
                line.clone(),
                *color,
                &widths,
                &order,
                false,
                false,
                None,
                false,
                crate::avatar::LogAvatarMode::Off,
                gitpoulpi_config::AvatarFallback::Initials,
                None,
            )
        })
        .collect();
    let mut terminal = Terminal::new(TestBackend::new(log_area.width, log_area.height)).unwrap();
    terminal
        .draw(|f| {
            let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
            f.render_widget(
                crate::log_table::log_table(rows, header, &order, &widths, Style::default()),
                log_area,
            );
        })
        .unwrap();
    let log_buf = terminal.backend().buffer().clone();
    let log_edge = gx + gw - 1;

    // The commit column.
    let mut state = CommitColumn::new(0, 0);
    let col_buf = draw(layout, &mut state, 60, 12);
    let (_, x0, x1) = state
        .parts
        .iter()
        .find(|(p, _, _)| *p == Part::Graph)
        .copied()
        .unwrap();
    let next = state
        .parts
        .iter()
        .find(|(_, a, _)| *a > x0)
        .map(|(_, a, _)| *a)
        .expect("a part after the Graph");
    let col_edge = next - 1;
    assert!(col_edge >= x1.saturating_sub(1));

    // At rest: some row of each has the tint on the cell before the next part
    // (the first rows' nodes are in the lanes), and in the column never a
    // plain cell between the band and the next part.
    let body = state.rows_area.y;
    for y in body..body + 4 {
        let log_y = y - body + 2;
        if tinted(&log_buf, log_edge, log_y) {
            assert!(
                tinted(&col_buf, col_edge, y),
                "row {y}: the Log's tint reaches its edge, the column's stops short"
            );
        }
        let lit: Vec<u16> = (x0..=col_edge)
            .filter(|x| tinted(&col_buf, *x, y))
            .collect();
        if let Some(&last) = lit.last() {
            assert_eq!(last, col_edge, "row {y}: a plain cell before the next part");
        }
    }
    assert!(
        (body..body + 4).any(|y| tinted(&col_buf, col_edge, y)),
        "the column's tint reaches the boundary cell"
    );

    // On hover the line replaces that cell, in both.
    let (_, rect) = state
        .boundaries
        .iter()
        .find(|(p, _)| *p == Part::Graph)
        .copied()
        .unwrap();
    assert_eq!(rect.x, col_edge, "the grab is the tint's end cell");
    let mut col_lit = col_buf.clone();
    crate::column_resize::paint_grab(&mut col_lit, ColumnResize::Line, rect);
    let log_rect = crate::pane_layout::log_boundary_grab_rect(log_area, gx + gw);
    assert_eq!(log_rect.x, log_edge);
    let mut log_lit = log_buf.clone();
    crate::column_resize::paint_grab(&mut log_lit, ColumnResize::Line, log_rect);
    for y in rect.y..rect.bottom().min(body + 4) {
        assert_eq!(col_lit[(col_edge, y)].symbol(), "\u{2502}", "row {y}");
    }
    assert_eq!(log_lit[(log_edge, 3)].symbol(), "\u{2502}");

    // `header` mode: the tint is the same, the grab is the next part's first
    // header cell, which is the cell right after the tint's end.
    crate::live_settings::install_column_resize(ColumnResize::Header);
    let mut state = CommitColumn::new(0, 0);
    let buf = draw(layout, &mut state, 60, 12);
    for y in body..body + 4 {
        assert!(tinted(&buf, col_edge, y), "header mode, row {y}");
    }
    let (_, rect) = state
        .boundaries
        .iter()
        .find(|(p, _)| *p == Part::Graph)
        .copied()
        .unwrap();
    assert_eq!((rect.x, rect.y), (next, state.rows_area.y - 1));
    crate::live_settings::install_column_resize(ColumnResize::Line);
}
