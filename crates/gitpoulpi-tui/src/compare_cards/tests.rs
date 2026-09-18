//! Buffer tests of the comparison cards (initials avatars: no picture is
//! ever laid over them here), drawn the way the right pane does, as lines in
//! a `TestBackend` buffer.

use super::*;
use crate::compare::{Card, CardKind, CompareSpec, EndMark};
use crate::test_support::oid;
use gitpoulpi_core::domain::DiffTarget;
use ratatui::backend::TestBackend;
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

const NOW: i64 = 1_800_000_000;
const WIDTH: usize = 44;

fn commit(row: usize, id: u8, who: &str, summary: &str, age: i64) -> Card {
    Card {
        row,
        kind: CardKind::Commit,
        id: Some(oid(id)),
        author_name: who.into(),
        author_email: format!("{}@example.com", who.to_lowercase().replace(' ', ".")),
        timestamp: NOW - age,
        summary: summary.into(),
        mark: None,
    }
}

fn wip(row: usize) -> Card {
    Card {
        row,
        kind: CardKind::Wip,
        id: None,
        author_name: String::new(),
        author_email: String::new(),
        timestamp: 0,
        summary: String::new(),
        mark: None,
    }
}

fn stash(row: usize, id: u8, handle: &str, message: &str) -> Card {
    Card {
        row,
        kind: CardKind::Stash,
        id: Some(oid(id)),
        author_name: handle.into(),
        author_email: String::new(),
        timestamp: NOW - 7200,
        summary: message.into(),
        mark: None,
    }
}

fn spec(mut cards: Vec<Card>, swapped: bool) -> CompareSpec {
    let n = cards.len();
    let (from, to) = if swapped { (0, n - 1) } else { (n - 1, 0) };
    cards[from].mark = Some(EndMark::From);
    cards[to].mark = Some(EndMark::To);
    CompareSpec {
        id: oid(99),
        target: DiffTarget::CommitRange(oid(1), oid(2)),
        from_label: String::new(),
        to_label: String::new(),
        rows: n,
        can_swap: n == 2 && cards[0].kind != CardKind::Wip,
        swapped,
        cards,
    }
}

fn look(hovered: Option<usize>, cursor_row: Option<usize>) -> CardLook {
    CardLook {
        width: WIDTH,
        now: NOW,
        avatars: LogAvatarMode::Glyph,
        fallback: AvatarFallback::Initials,
        hovered,
        cursor_row,
    }
}

/// Paints the cards the way the message block does and returns the rows as
/// text.
fn paint(spec: &CompareSpec, look: &CardLook, height: u16, scroll: usize) -> Vec<String> {
    let rows = build(spec, look);
    let lines: Vec<Line> = rows
        .iter()
        .skip(scroll)
        .take(height as usize)
        .map(|r| r.line.clone())
        .collect();
    let mut terminal = Terminal::new(TestBackend::new(WIDTH as u16, height)).unwrap();
    terminal
        .draw(|f| f.render_widget(Paragraph::new(lines), f.area()))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    (0..height)
        .map(|y| {
            (0..WIDTH as u16)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

#[test]
fn two_commits_are_two_cards_with_author_age_sha_summary_and_the_ends() {
    let s = spec(
        vec![
            commit(1, 4, "Jane Doe", "Fix the parser", 3 * 86_400),
            commit(3, 2, "Sam Lee", "Add the lexer", 30 * 60),
        ],
        false,
    );
    let text = paint(&s, &look(None, None), 4, 0);
    assert_eq!(text.len(), 4);
    // Newest first; the initials badge, then the name, the age and the sha.
    assert!(text[0].contains("J  Jane Doe"), "{text:#?}");
    assert!(text[0].contains("3 days ago"), "{text:#?}");
    assert!(text[0].contains(&short_oid(oid(4))), "{text:#?}");
    assert!(
        text[1].contains("Fix the parser") && text[1].trim_end().ends_with("To"),
        "the newer card is the To end: {text:#?}"
    );
    assert!(text[2].contains("S  Sam Lee") && text[2].contains("30 minutes ago"));
    assert!(
        text[3].contains("Add the lexer") && text[3].trim_end().ends_with("From"),
        "the older card is the From end: {text:#?}"
    );
}

#[test]
fn swapping_moves_the_from_and_to_marks() {
    let cards = || {
        vec![
            commit(1, 4, "Jane Doe", "Fix the parser", 86_400),
            commit(3, 2, "Sam Lee", "Add the lexer", 2 * 86_400),
        ]
    };
    let plain = paint(&spec(cards(), false), &look(None, None), 4, 0);
    let swapped = paint(&spec(cards(), true), &look(None, None), 4, 0);
    assert!(plain[1].trim_end().ends_with("To") && plain[3].trim_end().ends_with("From"));
    assert!(
        swapped[1].trim_end().ends_with("From") && swapped[3].trim_end().ends_with("To"),
        "{swapped:#?}"
    );
}

#[test]
fn the_wip_row_is_the_working_tree_and_a_stash_keeps_its_glyph_and_message() {
    let s = spec(
        vec![
            wip(0),
            stash(1, 7, "stash@{0}", "WIP on main: tidy"),
            commit(2, 3, "Jane Doe", "Fix", 86_400),
        ],
        false,
    );
    let text = paint(&s, &look(None, None), 7, 0);
    // Three rows: the range note comes first.
    assert!(text[0].contains("3 selected"), "{text:#?}");
    assert!(text[1].contains("Working tree"), "{text:#?}");
    assert!(text[2].contains("Uncommitted changes") && text[2].trim_end().ends_with("To"));
    assert!(
        text[3].contains(STASH_ICON) && text[3].contains("stash@{0}"),
        "{text:#?}"
    );
    assert!(text[4].contains("WIP on main: tidy"), "{text:#?}");
    assert!(text[5].contains("Jane Doe") && text[6].trim_end().ends_with("From"));
}

#[test]
fn a_narrow_pane_drops_the_date_then_the_sha_and_ellipsizes_the_summary() {
    let s = spec(
        vec![
            commit(
                1,
                4,
                "Jane Doe",
                "A very long summary that cannot possibly fit",
                60,
            ),
            commit(3, 2, "Sam Lee", "Short", 60),
        ],
        false,
    );
    let mut narrow = look(None, None);
    narrow.width = 24;
    let rows = build(&s, &narrow);
    let first: String = rows[0]
        .line
        .spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect();
    assert!(!first.contains("minute"), "the date goes first: {first}");
    let second: String = rows[1]
        .line
        .spans
        .iter()
        .map(|s| s.content.as_ref())
        .collect();
    assert!(second.contains('\u{2026}'), "{second}");
    assert!(display_width(&second) <= 24, "{second}");
}

#[test]
fn five_rows_scroll_inside_a_capped_block() {
    let cards: Vec<Card> = (0..5)
        .map(|i| {
            commit(
                i,
                10 - i as u8,
                &format!("Author {i}"),
                &format!("subject {i}"),
                60,
            )
        })
        .collect();
    let s = spec(cards, false);
    assert_eq!(line_count(&s), 11, "5 cards x 2 lines + the range note");
    // The block shows 6 of the 11 lines: the cap leaves the files the room.
    assert_eq!(crate::commit_detail::compare_card_rows(30, 11, 0), 10);
    assert_eq!(crate::commit_detail::compare_card_rows(60, 11, 0), 11);
    assert_eq!(
        crate::commit_detail::compare_card_rows(60, 11, 5),
        5,
        "a dragged splitter wins"
    );
    let top = paint(&s, &look(None, None), 6, 0);
    assert!(
        top[0].contains("5 selected") && top[1].contains("Author 0"),
        "{top:#?}"
    );
    let bottom = paint(&s, &look(None, None), 6, 5);
    assert!(
        bottom[5].trim_end().ends_with("From") && bottom[5].contains("subject 4"),
        "{bottom:#?}"
    );
    assert!(
        bottom.iter().all(|l| !l.contains("Author 0")),
        "{bottom:#?}"
    );
}

#[test]
fn the_hovered_and_cursor_cards_are_tinted_and_the_others_are_not() {
    let s = spec(
        vec![
            commit(1, 4, "Jane Doe", "Fix", 60),
            commit(3, 2, "Sam Lee", "Add", 60),
        ],
        false,
    );
    let hovered = build(&s, &look(Some(1), None));
    let bg_of = |r: &CardRow| r.line.spans[1].style.bg;
    assert_eq!(bg_of(&hovered[0]), None);
    assert_eq!(bg_of(&hovered[2]), Some(th().ui.hover));
    let cursor = build(&s, &look(None, Some(1)));
    assert_eq!(bg_of(&cursor[0]), Some(th().ui.selection));
    assert_eq!(bg_of(&cursor[2]), None);
}

fn picture_look() -> CardLook {
    CardLook {
        avatars: LogAvatarMode::Picture,
        ..look(None, None)
    }
}

#[test]
fn picture_mode_reserves_two_cells_of_initials_under_each_picture() {
    let s = spec(
        vec![
            commit(1, 4, "Jane Doe", "Fix the parser", 3 * 86_400),
            commit(3, 2, "Sam Lee", "Add the lexer", 30 * 60),
        ],
        false,
    );
    let text = paint(&s, &picture_look(), 4, 0);
    // The initials show until the picture is laid over them.
    assert!(text[0].contains("JD Jane Doe"), "{text:#?}");
    assert!(text[2].contains("SL Sam Lee"), "{text:#?}");
}

#[test]
fn every_commit_card_gets_an_image_placement_when_images_are_on() {
    use crate::avatar::{AvatarSlot, LogAvatarImages, PictureKey, RowTint};
    use crate::terminal_image::{Capability, CellSize, Protocol};
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Style;

    let s = spec(
        vec![
            commit(1, 4, "Jane Doe", "Fix the parser", 3 * 86_400),
            stash(2, 3, "stash@{0}", "WIP"),
            commit(3, 2, "Sam Lee", "Add the lexer", 30 * 60),
        ],
        false,
    );
    let rows = build(&s, &picture_look());
    let inner = Rect::new(2, 3, WIDTH as u16, rows.len() as u16);
    let bitmap = crate::terminal_image::Bitmap::new(42, 42, (200, 100, 50));
    let slots: Vec<AvatarSlot> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.first)
        .filter(|(_, r)| s.cards[r.card.unwrap()].kind == CardKind::Commit)
        .map(|(line, r)| AvatarSlot {
            area: slot_area(inner, line as u16).expect("the slot fits"),
            key: PictureKey {
                author: s.cards[r.card.unwrap()].author_email.clone(),
                tint: Some(RowTint::Plain),
                dimmed: false,
            },
            bitmap: &bitmap,
        })
        .collect();
    assert_eq!(slots.len(), 2, "the stash gets no picture");
    let mut buf = Buffer::empty(Rect::new(0, 0, 60, 20));
    buf.set_style(buf.area, Style::default());
    let mut images = LogAvatarImages::new(Capability {
        protocol: Protocol::Iterm2,
        cell: CellSize::FALLBACK,
    });
    assert_eq!(images.render(&mut buf, &slots), 2);
    // No image support: nothing is placed and the initials stay.
    let mut text_only = LogAvatarImages::new(Capability {
        protocol: Protocol::None,
        cell: CellSize::FALLBACK,
    });
    assert_eq!(text_only.render(&mut buf, &slots), 0);
}
