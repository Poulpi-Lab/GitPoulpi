//! The author filter's pure model, its popup and its two painted surfaces.

use super::*;
use crate::log_search;
use gitpoulpi_core::domain::{CommitNode, Oid};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::Terminal;

fn oid(n: u8) -> Oid {
    Oid::from_hex(&format!("{n:02x}").repeat(20)).unwrap()
}

fn commit(n: u8, author: &str, email: &str) -> CommitNode {
    CommitNode {
        id: oid(n),
        parents: Vec::new(),
        summary: format!("commit {n}"),
        message: format!("commit {n}"),
        author_name: author.to_string(),
        author_email: email.to_string(),
        timestamp: 0,
        refs: Vec::new(),
    }
}

/// Alice 3 commits, Bob 2, Carol 1: the three generic authors of the fixtures.
fn history() -> Vec<CommitNode> {
    vec![
        commit(1, "Bob Example", "bob@example.com"),
        commit(2, "Alice Example", "alice@example.com"),
        commit(3, "Alice Example", "ALICE@example.com"),
        commit(4, "Carol Example", "carol@example.com"),
        commit(5, "Alice Example", "alice@example.com"),
        commit(6, "Bob Example", "bob@example.com"),
    ]
}

fn chosen(keys: &[&str]) -> BTreeSet<String> {
    keys.iter().map(|k| k.to_string()).collect()
}

#[test]
fn the_key_is_the_lowercased_email_else_the_name() {
    assert_eq!(
        author_key("Alice Example", " Alice@Example.com "),
        "alice@example.com"
    );
    assert_eq!(author_key("Alice Example", ""), "alice example");
}

#[test]
fn authors_are_listed_by_commit_count_then_name() {
    let list = build_authors(&history(), &BTreeSet::new());
    let names: Vec<(&str, usize)> = list.iter().map(|e| (e.name.as_str(), e.commits)).collect();
    // The same email in two cases is one author; the most active first.
    assert_eq!(
        names,
        [
            ("Alice Example", 3),
            ("Bob Example", 2),
            ("Carol Example", 1)
        ]
    );
}

#[test]
fn an_equal_count_falls_back_to_the_name() {
    let nodes = vec![
        commit(1, "Zed Example", "zed@example.com"),
        commit(2, "Ann Example", "ann@example.com"),
    ];
    let list = build_authors(&nodes, &BTreeSet::new());
    assert_eq!(list[0].name, "Ann Example");
    assert_eq!(list[1].name, "Zed Example");
}

#[test]
fn a_chosen_author_with_no_loaded_commit_stays_in_the_list() {
    let list = build_authors(&history(), &chosen(&["dave@example.com"]));
    let dave = list.last().unwrap();
    assert_eq!((dave.key.as_str(), dave.commits), ("dave@example.com", 0));
}

#[test]
fn chosen_names_use_the_loaded_name_else_the_key() {
    let names = chosen_names(
        &history(),
        &chosen(&["bob@example.com", "dave@example.com"]),
    );
    assert_eq!(names, ["Bob Example", "dave@example.com"]);
}

#[test]
fn the_note_names_the_first_author_and_counts_the_rest() {
    assert_eq!(note_text(&[]), "");
    assert_eq!(
        note_text(&["Alice Example".into()]),
        "Author: Alice Example"
    );
    assert_eq!(
        note_text(&["Alice Example".into(), "Bob Example".into()]),
        "Authors: Alice Example +1"
    );
}

#[test]
fn the_filter_alone_makes_the_chosen_authors_the_hits() {
    let nodes = history();
    let hits = log_search::combine_hits(&nodes, None, &chosen(&["bob@example.com"]));
    assert_eq!(hits, [0, 5]);
}

#[test]
fn the_filter_and_a_text_search_both_apply() {
    let nodes = history();
    let refs = Vec::new();
    let alice = chosen(&["alice@example.com"]);
    // "commit 5" is by Alice, "commit 6" by Bob: the AND keeps one.
    assert_eq!(
        log_search::find_matches_with_authors(&nodes, &refs, "commit 5", &alice),
        [4]
    );
    assert!(log_search::find_matches_with_authors(&nodes, &refs, "commit 6", &alice).is_empty());
    // A blank query is the filter alone; no filter is the plain search.
    assert_eq!(
        log_search::find_matches_with_authors(&nodes, &refs, "  ", &alice),
        [1, 2, 4]
    );
    assert_eq!(
        log_search::find_matches_with_authors(&nodes, &refs, "commit 6", &BTreeSet::new()),
        [5]
    );
    assert!(log_search::find_matches_with_authors(&nodes, &refs, "", &BTreeSet::new()).is_empty());
}

#[test]
fn the_popup_lists_clear_first_then_every_author() {
    let popup = AuthorFilterPopup::open(&history(), &BTreeSet::new(), true);
    let rows = popup.rows();
    assert_eq!(rows[0], AuthorRow::Clear);
    assert_eq!(rows.len(), 4);
    // Opens on the first author, not on Clear.
    assert_eq!(popup.selected, 1);
}

#[test]
fn typing_narrows_the_list_fuzzily_and_keeps_the_clear_row() {
    let mut popup = AuthorFilterPopup::open(&history(), &BTreeSet::new(), true);
    popup.draft = "bob".into();
    let rows = popup.rows();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], AuthorRow::Clear);
    let AuthorRow::Author(i) = rows[1] else {
        panic!("an author row")
    };
    assert_eq!(popup.entry(i).unwrap().name, "Bob Example");
    // The email matches too, and a subsequence does (the palette's scorer).
    popup.draft = "crl".into();
    assert_eq!(popup.rows().len(), 2);
    popup.draft = "nobody".into();
    assert_eq!(popup.rows(), [AuthorRow::Clear]);
    popup.field_edited();
    assert_eq!(popup.selected, 0);
}

#[test]
fn enter_toggles_an_author_and_the_clear_row_drops_everything() {
    let dir = tempfile::tempdir().unwrap();
    let mut set = BTreeSet::new();
    let mut popup = AuthorFilterPopup::open(&history(), &set, false);
    // Row 1 is Alice, the most active.
    let (msg, close) = activate(&mut popup, dir.path(), &mut set);
    assert!(!close, "choosing an author keeps the popup open");
    assert_eq!(msg.unwrap(), "Author filter: 1 chosen");
    assert_eq!(set, chosen(&["alice@example.com"]));
    assert!(popup.chosen.contains("alice@example.com"));
    // A second author: multi-select.
    popup.selected = 2;
    activate(&mut popup, dir.path(), &mut set);
    assert_eq!(set.len(), 2);
    // Toggling again removes.
    popup.selected = 1;
    activate(&mut popup, dir.path(), &mut set);
    assert_eq!(set, chosen(&["bob@example.com"]));
    // Persisted per repository.
    assert_eq!(load(dir.path()), set);
    // The Clear row.
    popup.selected = 0;
    let (msg, close) = activate(&mut popup, dir.path(), &mut set);
    assert!(close);
    assert_eq!(msg.unwrap(), "Author filter cleared");
    assert!(set.is_empty() && popup.chosen.is_empty());
    assert!(load(dir.path()).is_empty());
}

#[test]
fn the_funnel_follows_the_title_and_hides_when_the_column_is_narrow() {
    // "Author" is 6 cells: room after it in a wide column.
    assert_eq!(funnel_layout(20, 6), Some((6, 7)));
    // Exactly title, space, glyph and the gutter cell.
    assert_eq!(funnel_layout(9, 6), Some((6, 7)));
    // The title is never cut for the funnel: none when they do not both fit
    // (the Log's top border says the filter is on).
    assert_eq!(funnel_layout(8, 6), None);
    assert_eq!(funnel_layout(4, 6), None);
    let r = funnel_rect(30, 20, 6, 2).unwrap();
    assert_eq!((r.x, r.y, r.width), (36, 2, 2));
}

fn header_text(buf: &Buffer, width: u16) -> String {
    (0..width)
        .map(|x| buf[(x, 0)].symbol().to_string())
        .collect()
}

/// The Author header cell, painted as the Log paints it.
fn header_buffer(width: u16, on: bool) -> Buffer {
    use crate::log_table::{log_header_cells, LogColumn, LogColumnWidths};
    use ratatui::layout::Constraint;
    use ratatui::widgets::{Row, Table, Widget};
    let order = [LogColumn::Author];
    let mut overrides = std::collections::HashMap::new();
    overrides.insert(LogColumn::Author, width);
    let widths = LogColumnWidths::for_area_width(width + 62, &Default::default(), &overrides);
    let cells = log_header_cells(&order, &widths, None, on);
    let area = ratatui::layout::Rect::new(0, 0, width, 1);
    let mut buf = Buffer::empty(area);
    Widget::render(
        Table::new(
            Vec::<Row>::new(),
            [Constraint::Length(widths.width_of(LogColumn::Author))],
        )
        .header(Row::new(cells)),
        area,
        &mut buf,
    );
    buf
}

#[test]
fn the_header_funnel_is_outline_at_rest_and_filled_with_the_accent_when_on() {
    let off = header_buffer(16, false);
    let on = header_buffer(16, true);
    assert!(header_text(&off, 16).starts_with(&format!("Author {FUNNEL}")));
    assert!(header_text(&on, 16).starts_with(&format!("Author {FUNNEL_ACTIVE}")));
    let accent = crate::th().ui.accent;
    assert_eq!(on[(7, 0)].fg, accent);
    assert_ne!(off[(7, 0)].fg, accent);
}

#[test]
fn a_narrow_author_column_draws_no_funnel() {
    let buf = header_buffer(5, true);
    let text = header_text(&buf, 5);
    assert!(
        !text.contains(FUNNEL_ACTIVE) && !text.contains(FUNNEL),
        "{text}"
    );
}

fn draw(popup: &AuthorFilterPopup) -> (Buffer, crate::chrome::PalettePopup) {
    let mut terminal = Terminal::new(TestBackend::new(90, 24)).unwrap();
    let editor = crate::text_edit::FieldEditor::default();
    let mut rects = None;
    terminal
        .draw(|f| rects = Some(render(f, popup, &editor)))
        .unwrap();
    (terminal.backend().buffer().clone(), rects.unwrap())
}

fn screen(buf: &Buffer) -> String {
    let w = buf.area.width;
    (0..buf.area.height)
        .map(|y| {
            (0..w)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_popup_paints_one_avatar_per_author_and_ticks_the_chosen() {
    set_anchor(Some(ratatui::layout::Rect::new(40, 3, 20, 1)));
    let mut popup = AuthorFilterPopup::open(&history(), &chosen(&["bob@example.com"]), true);
    popup.chosen = chosen(&["bob@example.com"]);
    let (buf, rects) = draw(&popup);
    let text = screen(&buf);
    assert!(text.contains("Clear filter"), "{text}");
    assert!(text.contains("filter by author"), "{text}");
    for who in ["Alice Example", "Bob Example", "Carol Example"] {
        assert_eq!(text.matches(who).count(), 1, "{who} once:\n{text}");
    }
    // One avatar badge per author: the initials sit before each name, and the
    // email carries none of its own.
    for badge in ["AE", "BE", "CE"] {
        assert_eq!(text.matches(badge).count(), 1, "{badge}:\n{text}");
    }
    // Hangs under the Author header: the popup starts one row below it.
    assert_eq!(rects.popup.y, 4);
    assert_eq!(rects.count, 4);
    let bob_row = text
        .lines()
        .find(|l| l.contains("Bob Example"))
        .expect("Bob's row");
    assert!(bob_row.contains('\u{2713}'), "{bob_row}");
    let alice_row = text.lines().find(|l| l.contains("Alice Example")).unwrap();
    assert!(!alice_row.contains('\u{2713}'), "{alice_row}");
    set_anchor(None);
}

#[test]
fn with_avatars_off_the_popup_paints_no_badge() {
    set_anchor(Some(ratatui::layout::Rect::new(40, 3, 20, 1)));
    let popup = AuthorFilterPopup::open(&history(), &BTreeSet::new(), false);
    let text = screen(&draw(&popup).0);
    for badge in ["AE", "BE", "CE"] {
        assert!(!text.contains(badge), "{badge}:\n{text}");
    }
    set_anchor(None);
}

#[test]
fn the_tooltip_lists_the_chosen_authors() {
    set_tip(Vec::new());
    assert_eq!(tip_lines(), ["Filter the Log by author"]);
    set_tip(vec!["Alice Example".into(), "Bob Example".into()]);
    assert_eq!(
        tip_lines(),
        [
            "Author filter (2):".to_string(),
            "  Alice Example".into(),
            "  Bob Example".into()
        ]
    );
    set_tip(Vec::new());
}

mod pictures_tests {
    use super::*;
    use crate::avatar::{
        card_avatar_mode, AvatarSlot, LogAvatarImages, LogAvatarMode, PictureKey, RowTint,
    };
    use crate::terminal_image::{Bitmap, Capability, CellSize, Protocol};
    use gitpoulpi_config::AvatarPlacement;

    fn kitty() -> Capability {
        Capability {
            protocol: Protocol::Kitty,
            cell: CellSize::FALLBACK,
        }
    }

    fn open() -> AuthorFilterPopup {
        AuthorFilterPopup::open(&history(), &BTreeSet::new(), true)
    }

    #[test]
    fn a_picture_slot_sits_on_each_badge_and_only_with_picture_mode() {
        set_anchor(Some(ratatui::layout::Rect::new(40, 3, 20, 1)));
        let popup = open();
        let (buf, rects) = draw(&popup);
        let slots = pictures::slots(
            &popup,
            rects.list,
            rects.window_start,
            LogAvatarMode::Picture,
        );
        assert_eq!(slots.len(), 3);
        // Each slot is exactly the two cells of the initials badge.
        for (area, e, _) in &slots {
            let name = &popup.entry(*e).unwrap().name;
            let badge: String = (area.x..area.right())
                .map(|x| buf[(x, area.y)].symbol().to_string())
                .collect();
            let want: String = name
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .collect();
            assert_eq!(badge, want, "{name}");
        }
        // Text terminal or avatars off: no picture, the badge stays.
        for mode in [LogAvatarMode::Glyph, LogAvatarMode::Off] {
            assert!(pictures::slots(&popup, rects.list, rects.window_start, mode).is_empty());
        }
        let off = AuthorFilterPopup::open(&history(), &BTreeSet::new(), false);
        assert!(
            pictures::slots(&off, rects.list, rects.window_start, LogAvatarMode::Picture)
                .is_empty()
        );
        set_anchor(None);
    }

    #[test]
    fn the_mode_follows_the_log_rules() {
        let text = Capability {
            protocol: Protocol::None,
            cell: CellSize::FALLBACK,
        };
        assert_eq!(
            card_avatar_mode(AvatarPlacement::Author, kitty()),
            LogAvatarMode::Picture
        );
        assert_eq!(
            card_avatar_mode(AvatarPlacement::Author, text),
            LogAvatarMode::Glyph
        );
        assert_eq!(
            card_avatar_mode(AvatarPlacement::Off, kitty()),
            LogAvatarMode::Off
        );
    }

    /// Only the author whose picture is ready is drawn as an image; the
    /// others keep their initials; a frame with no slot withdraws it.
    #[test]
    fn a_ready_picture_replaces_the_badge_and_the_rest_keep_initials() {
        set_anchor(Some(ratatui::layout::Rect::new(40, 3, 20, 1)));
        let popup = open();
        let (mut buf, rects) = draw(&popup);
        let slots = pictures::slots(
            &popup,
            rects.list,
            rects.window_start,
            LogAvatarMode::Picture,
        );
        let face = Bitmap::new(42, 42, (200, 30, 30));
        let first = slots[0];
        let draw_slots = [AvatarSlot {
            area: first.0,
            key: PictureKey {
                author: "alice".into(),
                tint: Some(RowTint::Plain),
                dimmed: false,
            },
            bitmap: &face,
        }];
        let mut images = LogAvatarImages::new(kitty());
        assert_eq!(images.render(&mut buf, &draw_slots), 1);
        let cell = |x, y| buf[(x, y)].symbol().to_string();
        assert!(
            cell(first.0.x, first.0.y).contains('\u{10EEEE}'),
            "the picture claims its cells"
        );
        let second = slots[1].0;
        assert!(!cell(second.x, second.y).contains('\u{10EEEE}'));
        images.finish_frame(&buf);
        let mut out = Vec::new();
        images.write_pending(&mut out).unwrap();
        // The popup closed: no slot is drawn, the picture is erased.
        let gone = Buffer::empty(buf.area);
        images.finish_frame(&gone);
        let mut out = Vec::new();
        images.write_pending(&mut out).unwrap();
        assert!(!out.is_empty(), "closing sends the erase");
        set_anchor(None);
    }
}
