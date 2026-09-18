use super::*;
use crate::chrome::CentralView;
use crate::daemon_link::{FileInsightKind, FileInsightRequest};
use crate::diff_view::DiffViewState;
use crate::file_view::FileInsight;
use gitpoulpi_core::domain::{FileDiff, FileStatus};
use std::path::PathBuf;

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn changed(paths: &[&str]) -> Vec<FileDiff> {
    paths
        .iter()
        .map(|p| FileDiff {
            path: PathBuf::from(p),
            status: FileStatus::Modified,
            hunks: Vec::new(),
        })
        .collect()
}

fn insight(kind: FileInsightKind, worktree: bool) -> FileInsight {
    FileInsight::loading(FileInsightRequest {
        path: PathBuf::from("a.txt"),
        at: oid(4),
        kind,
        worktree,
        staged: false,
    })
}

#[test]
fn the_choice_round_trips_and_an_unknown_value_is_automatic() {
    for pref in [Pref::Auto, Pref::On, Pref::Off] {
        assert_eq!(Pref::from_saved(pref.saved()), pref);
    }
    assert_eq!(Pref::from_saved(9), Pref::Auto);
}

#[test]
fn automatic_shows_from_140_columns_and_a_choice_overrides_the_width() {
    let mut col = CommitColumn::new(0, 0);
    assert!(!col.wanted(139));
    assert!(col.wanted(140));
    col.pref = Pref::On;
    assert!(col.wanted(80));
    col.pref = Pref::Off;
    assert!(!col.wanted(300));
}

#[test]
fn the_column_never_leaves_the_view_under_its_minimum() {
    let col = CommitColumn::new(0, 0);
    // 63 cells: the central column of a 140 column terminal.
    let (list, view) = col.split(Rect::new(10, 2, 63, 30)).unwrap();
    assert!(view.width >= MIN_VIEW_COLS, "{view:?}");
    assert_eq!(list.width + view.width, 63);
    assert_eq!(list.right(), view.x);
    // Too narrow for both: no column at all.
    assert!(col
        .split(Rect::new(0, 0, MIN_VIEW_COLS + MIN_COLS - 1, 10))
        .is_none());
    // A wide width asked for is cut to what the view leaves.
    let wide = CommitColumn::new(0, 90);
    let (list, view) = wide.split(Rect::new(0, 0, 100, 10)).unwrap();
    assert_eq!((list.width, view.width), (70, 30));
}

#[test]
fn the_dragged_width_is_kept_inside_its_bounds() {
    let mut col = CommitColumn::new(0, 0);
    col.column = Rect::new(20, 1, 100, 10);
    assert_eq!(col.cols_at(20 + 40), 40);
    assert_eq!(col.cols_at(21), MIN_COLS);
    assert_eq!(col.cols_at(250), max_cols(100));
    col.set_cols(5);
    assert_eq!(col.cols, MIN_COLS);
}

#[test]
fn the_offset_follows_the_selection_until_the_wheel_takes_over() {
    let mut col = CommitColumn::new(0, 0);
    col.settle(Some(25), 100, 10);
    assert_eq!(col.offset, 16);
    col.settle(Some(3), 100, 10);
    assert_eq!(col.offset, 3);
    col.len = 100;
    col.rows_area = Rect::new(0, 0, 20, 10);
    col.scroll_to(50);
    assert!(!col.follow);
    col.settle(Some(3), 100, 10);
    assert_eq!(col.offset, 50, "a scrolled list is not pulled back");
    col.scroll_to(500);
    assert_eq!(col.offset, 90);
}

#[test]
fn only_a_view_of_one_commits_file_has_a_commit() {
    let commit_diff = DiffViewState::for_commit(oid(3), 0, PathBuf::from("a.txt"));
    assert_eq!(
        viewed_commit(CentralView::Diff, Some(&commit_diff), None),
        Some(oid(3))
    );
    let wip = DiffViewState::for_working(
        crate::diff_view::DiffSource::Unstaged,
        PathBuf::from("a.txt"),
    );
    assert_eq!(viewed_commit(CentralView::Diff, Some(&wip), None), None);
    let content = insight(FileInsightKind::Content, false);
    assert_eq!(
        viewed_commit(CentralView::File, None, Some(&content)),
        Some(oid(4))
    );
    let blame = insight(FileInsightKind::Blame, false);
    assert_eq!(
        viewed_commit(CentralView::File, None, Some(&blame)),
        Some(oid(4))
    );
    let on_disk = insight(FileInsightKind::Content, true);
    assert_eq!(viewed_commit(CentralView::File, None, Some(&on_disk)), None);
    let history = insight(FileInsightKind::History, false);
    assert_eq!(viewed_commit(CentralView::File, None, Some(&history)), None);
    // The editor edits the working copy: no commit, so no column.
    assert_eq!(
        viewed_commit(CentralView::Editor, Some(&commit_diff), None),
        None
    );
    assert_eq!(
        viewed_commit(CentralView::Editor, Some(&commit_diff), Some(&content)),
        None
    );
    assert_eq!(
        viewed_commit(CentralView::Log, Some(&commit_diff), None),
        None
    );
}

#[test]
fn a_diff_or_file_view_shows_the_diff_when_the_commit_changed_the_file() {
    let files = changed(&["x.txt", "a.txt"]);
    let path = PathBuf::from("a.txt");
    for kind in [ViewKind::Diff, ViewKind::Content] {
        assert_eq!(
            plan(kind, &path, Some(&files)),
            Retarget::Diff {
                file_index: 1,
                path: path.clone()
            }
        );
    }
}

#[test]
fn an_untouched_file_is_shown_as_it_was_there() {
    let files = changed(&["x.txt"]);
    let path = PathBuf::from("a.txt");
    for kind in [ViewKind::Diff, ViewKind::Content] {
        assert_eq!(
            plan(kind, &path, Some(&files)),
            Retarget::Content { path: path.clone() }
        );
    }
    // A commit whose file list failed to load: the file itself.
    assert_eq!(
        plan(ViewKind::Diff, &path, None),
        Retarget::Content { path: path.clone() }
    );
}

#[test]
fn blame_stays_blame_and_remembers_whether_the_commit_changed_the_file() {
    let path = PathBuf::from("a.txt");
    assert_eq!(
        plan(ViewKind::Blame, &path, Some(&changed(&["a.txt"]))),
        Retarget::Blame {
            path: path.clone(),
            changed: Some(0)
        }
    );
    assert_eq!(
        plan(ViewKind::Blame, &path, Some(&changed(&["b.txt"]))),
        Retarget::Blame {
            path,
            changed: None
        }
    );
}

#[test]
fn the_short_author_is_one_word_and_the_date_drops_the_year() {
    assert_eq!(render::short_author("Ada Lovelace", 8), "Ada");
    assert_eq!(
        render::short_author("Bartholomew Simpson", 8),
        "Barthol\u{2026}"
    );
    assert_eq!(render::short_author("", 8), "");
    // 2026-10-27 12:00 UTC, whatever the zone: the month and the day.
    let date = render::short_date(1_793_102_400, 5);
    assert_eq!(date.chars().count(), 5, "{date}");
    assert!(date.starts_with("10-"), "{date}");
}

#[test]
fn the_list_round_trips_and_an_unknown_value_is_all_the_commits() {
    for mode in [ColumnMode::File, ColumnMode::All] {
        assert_eq!(ColumnMode::from_saved(mode.saved()), mode);
    }
    assert_eq!(ColumnMode::from_saved(7), ColumnMode::All);
    assert_eq!(ColumnMode::File.state(), ColumnState::File);
    assert_eq!(ColumnMode::All.state(), ColumnState::Graph);
}

#[test]
fn a_saved_session_gives_the_list_and_the_widths() {
    let col = CommitColumn::new(1, 0).with_saved(1, &[3, 12]);
    assert_eq!(col.mode, ColumnMode::File);
    assert_eq!(col.widths, [3, 12, 0, 0, 0, 0, 0]);
    // Nothing saved: all commits, automatic.
    let col = CommitColumn::new(0, 0).with_saved(0, &[]);
    assert_eq!((col.mode, col.widths), (ColumnMode::All, [0; 7]));
}

/// The rows scroll sideways inside what the row has beyond the room, and
/// not at all when it fits.
#[test]
fn the_sideways_scroll_stays_inside_the_row() {
    let mut col = CommitColumn::new(0, 0);
    col.content_w = 54;
    col.inner_w = 44;
    col.scroll_h(4);
    assert_eq!(col.h_offset, 4);
    col.scroll_h(100);
    assert_eq!(col.h_offset, 10, "the end of the row");
    col.scroll_h(-100);
    assert_eq!(col.h_offset, 0);
    col.content_w = 44;
    col.scroll_h(4);
    assert_eq!(col.h_offset, 0, "a row that fits does not scroll");
}

/// The first resize fixes every shown part at what it has (the message
/// included), so sizing one moves nothing else; the keys never take a part
/// under its least, and reset puts everything back to automatic.
#[test]
fn sizing_a_part_pins_the_others_and_a_reset_unpins_them() {
    use super::picture::{plan_widths_with, AUTHOR_W, DATE_W, MIN_AUTHOR};
    let mut col = CommitColumn::new(0, 0);
    col.plan = plan_widths_with(60, 6, [0; 7]);
    col.part = Part::Author;
    col.resize_part(2);
    let message = col.plan.title as u16;
    assert_eq!(
        col.widths,
        [6, AUTHOR_W as u16 + 2, DATE_W as u16, 0, message, 0, 0],
        "refs is hidden: it stays automatic"
    );
    col.resize_part(-100);
    assert_eq!(col.widths[1], MIN_AUTHOR as u16);
    col.reset_widths();
    assert_eq!(col.widths, [0; 7]);
    assert_eq!(col.h_offset, 0);
}

/// Dragging a boundary (the right edge of a part) puts the next part's first
/// letter under the pointer, and nothing else moves in width: the message is
/// sized like the others.
#[test]
fn a_boundary_drag_puts_the_next_first_letter_under_the_pointer() {
    // The `header` mode (the first letter of the next name is the grab).
    crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Header);
    use super::picture::plan_widths_with;
    let mut col = CommitColumn::new(0, 0);
    col.rows_area = Rect::new(10, 5, 44, 10);
    col.inner_w = 44;
    col.plan = plan_widths_with(44, 4, [0; 7]);
    // Lanes | message: the message's first letter is at 9 cells.
    col.drag_boundary(Part::Graph, 10 + 9);
    assert_eq!(col.widths[0], 8, "9 minus the cell between");
    let author = col.widths[1];
    let message_before = col.widths[4];
    // Message | author: the message ends before the pointer.
    col.plan = plan_widths_with(44, 4, col.widths);
    let start = col.plan.spans()[1].1;
    col.drag_boundary(Part::Title, 10 + 30);
    assert_eq!(usize::from(col.widths[4]), 30 - start - 1);
    assert_eq!(col.widths[1], author, "the author keeps its width");
    assert_ne!(col.widths[4], message_before);
    // The boundary cell is that first letter, before and after the paint.
    col.plan = plan_widths_with(44, 4, col.widths);
    let rect = col.boundary_rect(Part::Title).unwrap();
    assert_eq!((rect.x, rect.y), (10 + 30, 4));
}

/// The Refs part is an option: the saved names give the shown parts, the
/// message is always shown, nothing saved is the default, and the shown
/// parts are saved back as the Log's own column names.
#[test]
fn the_shown_parts_round_trip_through_their_saved_names() {
    let names = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let col = CommitColumn::new(0, 0).with_visible(&[]);
    assert_eq!(col.shown, DEFAULT_SHOWN);
    assert!(col.visible_names().is_empty());
    let col = CommitColumn::new(0, 0).with_visible(&names(&["ref", "author"]));
    assert!(col.is_shown(Part::Refs) && col.is_shown(Part::Author));
    assert!(!col.is_shown(Part::Graph) && !col.is_shown(Part::Date));
    assert!(col.is_shown(Part::Title), "the message cannot be hidden");
    let again = CommitColumn::new(0, 0).with_visible(&col.visible_names());
    assert_eq!(again.shown, col.shown);
    let mut col = CommitColumn::new(0, 0);
    assert!(!col.toggle_shown(Part::Title));
    assert!(col.toggle_shown(Part::Refs));
    assert!(col.is_shown(Part::Refs));
}

/// The widest the column may be is a share of the central column, not a flat
/// 100 cells.
#[test]
fn the_column_may_take_most_of_a_wide_central_column() {
    let mut col = CommitColumn::new(0, 400);
    let (list, view) = col.split(Rect::new(0, 0, 200, 30)).unwrap();
    assert_eq!(list.width, 140, "70% of 200");
    assert_eq!(view.width, 60);
    col.column = Rect::new(0, 0, 200, 30);
    assert_eq!(col.cols_at(190), 140);
    // A narrow one still leaves the view its floor.
    let (list, view) = col.split(Rect::new(0, 0, 60, 30)).unwrap();
    assert_eq!((list.width, view.width), (30, 30));
}

/// The selector is as wide in every state: the segments are the same two
/// words and the rule, lit or not.
#[test]
fn the_selector_keeps_its_width_in_every_state() {
    let width_in = |state: ColumnState, mode: ColumnMode| -> usize {
        note_painted(state);
        note_mode(mode);
        let (spans, hits) = selector(None);
        let drawn: usize = spans
            .iter()
            .map(|s| crate::ui_kit::display_width(&s.content))
            .sum();
        let hit_end = hits
            .iter()
            .map(|(dx, w, _)| usize::from(dx + w))
            .max()
            .unwrap();
        assert_eq!(drawn, hit_end);
        drawn
    };
    let widths: Vec<usize> = [ColumnState::Off, ColumnState::File, ColumnState::Graph]
        .into_iter()
        .flat_map(|state| {
            [ColumnMode::File, ColumnMode::All]
                .into_iter()
                .map(move |mode| (state, mode))
        })
        .map(|(state, mode)| width_in(state, mode))
        .collect();
    assert!(widths.windows(2).all(|w| w[0] == w[1]), "{widths:?}");
    note_painted(ColumnState::Off);
    note_mode(ColumnMode::All);
}

/// The saved order: unknown names and repeats are ignored, the parts it
/// leaves out follow in default order, and the default order is saved as
/// nothing.
#[test]
fn the_saved_order_is_completed_and_the_default_is_saved_as_nothing() {
    let names = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let col = CommitColumn::new(0, 0).with_order(&[]);
    assert_eq!(col.order, Part::ORDER);
    assert!(col.order_names().is_empty());
    let col = CommitColumn::new(0, 0).with_order(&names(&["sha", "bogus", "date", "sha"]));
    assert_eq!(
        col.order,
        [
            Part::Sha,
            Part::Date,
            Part::Refs,
            Part::Graph,
            Part::Title,
            Part::Author,
            Part::Changes
        ]
    );
    let again = CommitColumn::new(0, 0).with_order(&col.order_names());
    assert_eq!(again.order, col.order);
}

/// Moving a part among the shown ones: a hidden part is stepped over, the
/// ends refuse, and a drag to the right lands after the target.
#[test]
fn a_part_moves_among_the_shown_ones_and_stops_at_the_ends() {
    let mut col = CommitColumn::new(0, 0);
    // Shown by default: Graph, Message, Author, Date (Refs is hidden).
    assert!(col.move_part(Part::Title, Side::Right));
    assert_eq!(
        &col.order[..4],
        [Part::Refs, Part::Graph, Part::Author, Part::Title]
    );
    assert!(col.move_part(Part::Title, Side::Right));
    assert!(!col.move_part(Part::Title, Side::Right), "Date was last");
    assert!(
        !col.move_part(Part::Refs, Side::Left),
        "hidden: not movable"
    );
    assert!(!col.move_part(Part::Graph, Side::Left), "first shown part");
    // The Log's drag, to the right: after the target.
    let mut col = CommitColumn::new(0, 0);
    assert!(col.move_onto(Part::Graph, Part::Author));
    assert_eq!(
        &col.order[..4],
        [Part::Refs, Part::Title, Part::Author, Part::Graph]
    );
    assert!(!col.move_onto(Part::Graph, Part::Graph));
}

/// A drag reads the row as it is now, and a part does not flip back under a
/// still pointer: to the right it waits for the pointer to be where the
/// dragged part would then begin.
#[test]
fn a_header_drag_moves_the_part_once_and_does_not_flip_back() {
    use super::picture::plan_widths_with;
    let mut col = CommitColumn::new(0, 0);
    col.rows_area = Rect::new(10, 5, 44, 10);
    col.inner_w = 44;
    col.plan = plan_widths_with(44, 4, [0; PARTS]);
    // Lanes 4 | message 24 | author 8 | date 5. Drag the lanes right onto the
    // message: not before the pointer is as far as the lanes would then start
    // (the message's width + 1 cells in).
    col.drag_header(Part::Graph, 10 + 6, 6);
    assert!(
        col.header_drop.is_none(),
        "not before the pointer is far enough"
    );
    col.drag_header(Part::Graph, 10 + 26, 6);
    let (target, (bar_x, bar_y)) = col.header_drop.expect("a drop target mid-drag");
    assert_eq!(target, Part::Title);
    assert_eq!(bar_y, 4, "the bar sits on the header row");
    assert!(bar_x >= 10);
    // Nothing moved while the button is down.
    assert_eq!(&col.order[..3], [Part::Refs, Part::Graph, Part::Title]);
    col.dragging_header = Some(Part::Graph);
    assert!(col.drop_header());
    assert_eq!(&col.order[..3], [Part::Refs, Part::Title, Part::Graph]);
    // And back to the left, then Esc leaves it as it is.
    col.dragging_header = Some(Part::Graph);
    col.drag_header(Part::Graph, 10 + 2, 6);
    assert!(col.cancel_header_drag());
    assert!(!col.drop_header());
    assert_eq!(&col.order[..3], [Part::Refs, Part::Title, Part::Graph]);
}

/// The last part has its own boundary, the cell right after it, which the
/// pointer then covers; its light stays on the column's border when the part
/// is dragged past the room.
#[test]
fn the_last_part_is_sized_from_the_cell_after_it() {
    use super::picture::plan_widths_with;
    let mut col = CommitColumn::new(0, 0);
    col.rows_area = Rect::new(10, 5, 44, 10);
    col.inner_w = 44;
    col.plan = plan_widths_with(44, 4, [0; PARTS]);
    // Date is last, five cells wide, ending at the room's edge: the cell
    // after it is the border (44 cells in).
    let rect = col.boundary_rect(Part::Date).unwrap();
    assert_eq!((rect.x, rect.y), (10 + 44, 4));
    col.drag_boundary(Part::Date, 10 + 47);
    assert_eq!(col.widths[Part::Date.slot()], 8);
    col.plan = plan_widths_with(44, 4, col.widths);
    let rect = col.boundary_rect(Part::Date).unwrap();
    assert_eq!(rect.x, 10 + 44, "the light stays on the border");
    // Narrowing: the pointer inside the room gives the width up to it.
    col.drag_boundary(Part::Date, 10 + 44);
    assert_eq!(col.widths[Part::Date.slot()], 5);
}

/// The Changes and SHA parts are options that are kept (the row scrolls
/// sideways) rather than given up for room, like the Refs part.
#[test]
fn changes_and_sha_are_kept_when_the_room_is_short() {
    use super::picture::{plan_shown, CHANGES_W, SHA_W};
    let mut shown = DEFAULT_SHOWN;
    shown[Part::Changes.slot()] = true;
    shown[Part::Sha.slot()] = true;
    let plan = plan_shown(44, 4, shown, Part::ORDER);
    assert_eq!((plan.changes, plan.sha), (CHANGES_W, SHA_W));
    assert_eq!(plan.author + plan.date, 0, "the optional extras give way");
    assert_eq!(plan.spans().last().map(|s| s.0), Some(Part::Sha));
    // In another order the spans follow it.
    let mut order = Part::ORDER;
    order.swap(0, 6);
    let plan = plan_shown(60, 4, shown, order);
    assert_eq!(plan.spans().first().map(|s| s.0), Some(Part::Sha));
}

/// `line` mode: the pointer holds the separator cell, one before the next
/// part's first letter, and gets the same widths as a drag by that letter; the
/// grab runs from the header down every row.
#[test]
fn in_line_mode_a_boundary_drag_by_the_separator_is_the_same_drag() {
    use super::picture::plan_widths_with;
    crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Line);
    let mut by_gap = CommitColumn::new(0, 0);
    by_gap.rows_area = Rect::new(10, 5, 44, 10);
    by_gap.inner_w = 44;
    by_gap.plan = plan_widths_with(44, 4, [0; 7]);
    let mut by_letter = CommitColumn::new(0, 0);
    by_letter.rows_area = by_gap.rows_area;
    by_letter.inner_w = 44;
    by_letter.plan = by_gap.plan;
    by_gap.drag_boundary(Part::Graph, 10 + 8);
    crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Header);
    by_letter.drag_boundary(Part::Graph, 10 + 9);
    assert_eq!(by_gap.widths, by_letter.widths);
    assert_eq!(by_gap.widths[0], 8);

    crate::live_settings::install_column_resize(gitpoulpi_config::ColumnResize::Line);
    by_gap.plan = plan_widths_with(44, 4, by_gap.widths);
    let rect = by_gap.boundary_rect(Part::Graph).unwrap();
    assert_eq!(
        (rect.x, rect.y),
        (10 + 8, 4),
        "the separator, from the header"
    );
    assert_eq!(rect.height, 1 + 10, "the header and every row");
}
