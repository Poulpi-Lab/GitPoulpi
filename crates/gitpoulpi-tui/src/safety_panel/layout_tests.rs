//! The history list as painted: translated labels, the undone entries above
//! the stack, day headings and the fixed icon / age / tag columns. Buffer
//! tests over `ratatui`'s buffer, no terminal and no daemon.

use crossterm::event::{KeyCode, KeyModifiers};
use gitpoulpi_core::domain::OperationKind as K;
use gitpoulpi_core::domain::{JournalListing, ListedEntry, RestoreOffer};
use gitpoulpi_i18n::Lang;

use super::render::Hit;
use super::tests::{column_of, entry, key, panel_with};
use super::*;

const DAY: i64 = 86_400;

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// `(id, kind, stored label, seconds ago)`.
type Spec<'a> = (u64, K, &'a str, i64);

fn panel_of(undo: &[Spec], redo: &[Spec]) -> SafetyPanel {
    let mut panel = panel_with(&[], &[]);
    let make = |specs: &[Spec]| -> Vec<ListedEntry> {
        specs
            .iter()
            .map(|(id, kind, label, ago)| {
                let mut e = entry(*id, label);
                e.summary.kind = *kind;
                e.summary.at = now() - ago;
                e
            })
            .collect()
    };
    panel.history.set(JournalListing {
        undo: make(undo),
        redo: make(redo),
        retired: Vec::new(),
    });
    panel
}

fn paint(panel: &mut SafetyPanel) -> ratatui::buffer::Buffer {
    let area = Rect::new(0, 0, 100, 34);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    panel.render(render::safety_rect(area), &mut buf);
    buf
}

fn row_text(buf: &ratatui::buffer::Buffer, y: u16) -> String {
    let (mut text, mut skip) = (String::new(), 0);
    for x in 0..buf.area.width {
        // The cell after a wide glyph is its blank continuation.
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let sym = buf[(x, y)].symbol();
        skip = crate::display_width(sym).saturating_sub(1);
        text.push_str(sym);
    }
    text
}

fn screen(buf: &ratatui::buffer::Buffer) -> String {
    (0..buf.area.height)
        .map(|y| row_text(buf, y))
        .collect::<Vec<_>>()
        .join("\n")
}

fn y_of(buf: &ratatui::buffer::Buffer, needle: &str) -> u16 {
    (0..buf.area.height)
        .find(|y| row_text(buf, *y).contains(needle))
        .unwrap_or_else(|| panic!("{needle:?} is not on screen:\n{}", screen(buf)))
}

// -- translated labels --------------------------------------------------------

#[test]
fn the_rows_say_what_the_action_was_in_the_ui_language() {
    let _lang = gitpoulpi_i18n::override_lang(Lang::Fr);
    let _terms = gitpoulpi_i18n::override_git_terms(true);
    let mut panel = panel_of(
        &[
            (1, K::Discard, "Discard tidepool/tables.txt", 60),
            (2, K::BranchCreate, "Create branch topic (abc1234)", 120),
            (3, K::TagDelete, "Delete tag v1", 180),
        ],
        &[],
    );
    let screen = screen(&paint(&mut panel));
    for want in [
        "Abandonner tidepool/tables.txt",
        "Créer la branche topic (abc1234)",
        "Supprimer l'étiquette v1",
    ] {
        assert!(screen.contains(want), "{want:?} missing:\n{screen}");
    }
    assert!(!screen.contains("Discard") && !screen.contains("Create branch"));
}

#[test]
fn an_unrecognised_stored_label_stays_as_recorded() {
    let _lang = gitpoulpi_i18n::override_lang(Lang::Fr);
    let mut panel = panel_of(&[(1, K::Commit, "Something an older build wrote", 60)], &[]);
    assert!(screen(&paint(&mut panel)).contains("Something an older build wrote"));
}

// -- the undone entries sit above the stack -----------------------------------

#[test]
fn the_undone_entries_are_above_the_newest_undo_one_and_the_selection_is_on_it() {
    let _lang = gitpoulpi_i18n::override_lang(Lang::En);
    let mut panel = panel_of(
        &[
            (1, K::Commit, "Commit \"old-undo\"", 300),
            (2, K::Commit, "Commit \"new-undo\"", 100),
        ],
        // Redo stack, the next one to redo last: 4 is redone first.
        &[
            (3, K::Commit, "Commit \"far-redo\"", 50),
            (4, K::Commit, "Commit \"near-redo\"", 200),
        ],
    );
    let buf = paint(&mut panel);
    let far = y_of(&buf, "far-redo");
    let near = y_of(&buf, "near-redo");
    let divider = y_of(&buf, gitpoulpi_i18n::msg::undo_label::DIVIDER_REDO.get());
    let new = y_of(&buf, "new-undo");
    let old = y_of(&buf, "old-undo");
    assert!(
        far < near && near < divider && divider < new && new < old,
        "{far} {near} {divider} {new} {old}"
    );
    let selected = panel.selected_history().expect("a selection");
    assert_eq!(selected.entry.summary.id, 2, "the newest undo entry");
    assert!(!selected.redo);
    // The undone rows keep their own action: Redo up to here.
    let state = panel.journal_state();
    panel.on_key(key(KeyCode::Up));
    assert_eq!(panel.actions(&state), vec![Act::RedoTo]);
    assert_eq!(panel.selected_history().unwrap().entry.summary.id, 4);
}

#[test]
fn home_and_end_reach_the_first_and_last_entry() {
    let mut panel = panel_of(
        &[(1, K::Commit, "Commit \"a\"", 100)],
        &[(2, K::Commit, "Commit \"b\"", 50)],
    );
    panel.on_key(key(KeyCode::Home));
    assert_eq!(panel.selected_history().unwrap().entry.summary.id, 2);
    panel.on_key(key(KeyCode::End));
    assert_eq!(panel.selected_history().unwrap().entry.summary.id, 1);
}

// -- day separators -----------------------------------------------------------

#[test]
fn days_are_headed_only_when_the_list_spans_several() {
    let _lang = gitpoulpi_i18n::override_lang(Lang::En);
    let mut one_day = panel_of(
        &[
            (2, K::Commit, "Commit \"b\"", 90),
            (1, K::Commit, "Commit \"a\"", 60),
        ],
        &[],
    );
    let plain = screen(&paint(&mut one_day));
    assert!(!plain.contains("Today") && !plain.contains("Yesterday"));

    let mut several = panel_of(
        &[
            (3, K::Commit, "Commit \"older\"", 5 * DAY),
            (2, K::Commit, "Commit \"yesterday\"", DAY + 60),
            (1, K::Commit, "Commit \"today\"", 60),
        ],
        &[],
    );
    let buf = paint(&mut several);
    let today = y_of(&buf, "Today");
    let yesterday = y_of(&buf, "Yesterday");
    assert!(today < y_of(&buf, "today"));
    assert!(y_of(&buf, "today") < yesterday && yesterday < y_of(&buf, "yesterday"));
    assert!(y_of(&buf, "yesterday") < y_of(&buf, "older"));
}

#[test]
fn a_separator_is_never_selected_nor_clickable() {
    let mut panel = panel_of(
        &[
            (3, K::Commit, "Commit \"older\"", 5 * DAY),
            (2, K::Commit, "Commit \"yesterday\"", DAY + 60),
            (1, K::Commit, "Commit \"today\"", 60),
        ],
        &[],
    );
    let _ = paint(&mut panel);
    // Down moves over the heading, never onto it.
    for want in [2, 3, 3] {
        panel.on_key(key(KeyCode::Down));
        assert_eq!(panel.selected_history().unwrap().entry.summary.id, want);
    }
    for want in [2, 1, 1] {
        panel.on_key(key(KeyCode::Up));
        assert_eq!(panel.selected_history().unwrap().entry.summary.id, want);
    }
    // Hit rects: one per entry, none on a heading.
    let _ = paint(&mut panel);
    let rows: Vec<usize> = panel
        .hits
        .iter()
        .filter_map(|(_, hit)| match hit {
            Hit::Row(line) => Some(*line),
            _ => None,
        })
        .collect();
    assert_eq!(rows.len(), 3);
    assert!(rows
        .iter()
        .all(|line| panel.history.row_at(*line).is_some()));
}

#[test]
fn a_click_on_a_row_selects_the_entry_not_the_line_number() {
    let mut panel = panel_of(
        &[
            (2, K::Commit, "Commit \"yesterday\"", DAY + 60),
            (1, K::Commit, "Commit \"today\"", 60),
        ],
        &[],
    );
    let buf = paint(&mut panel);
    let y = y_of(&buf, "yesterday");
    let rect = panel
        .hits
        .iter()
        .find(|(r, hit)| matches!(hit, Hit::Row(_)) && r.y == y)
        .map(|(r, _)| *r)
        .expect("a hit rect on the painted row");
    panel.on_mouse(crossterm::event::MouseEvent {
        kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
        column: rect.x + 5,
        row: y,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(panel.selected_history().unwrap().entry.summary.id, 2);
}

// -- fixed columns ------------------------------------------------------------

#[test]
fn icons_labels_ages_and_tags_stay_in_their_columns() {
    for lang in [Lang::En, Lang::Ja, Lang::Fr] {
        let _lang = gitpoulpi_i18n::override_lang(lang);
        // (needle on the row, family of its icon, seconds ago)
        let rows_wanted = [
            ("alpha", label::Family::Commit, 60),
            ("x.txt", label::Family::Discard, 61),
            ("beta", label::Family::Tag, DAY + 60),
            ("\u{65E5}\u{672C}\u{8A9E}", label::Family::Commit, DAY + 61),
            ("stage.txt", label::Family::Staging, 3 * DAY),
            ("abc1234", label::Family::Move, 30),
            ("mystery", label::Family::Other, 5 * DAY),
        ];
        let mut panel = panel_of(
            &[
                (1, K::Commit, "Commit \"alpha\"", 60),
                (2, K::Discard, "Discard x.txt", 61),
                (3, K::TagCreate, "Create tag beta", DAY + 60),
                (
                    4,
                    K::Commit,
                    "Commit \"\u{65E5}\u{672C}\u{8A9E}\"",
                    DAY + 61,
                ),
                (5, K::StageFile, "Stage stage.txt", 3 * DAY),
                (7, K::Status, "mystery", 5 * DAY),
            ],
            &[(6, K::Reset, "Reset (hard) to abc1234", 30)],
        );
        // One entry carries a tag, the others none.
        panel.history.rows[1].entry.restore = RestoreOffer {
            items: vec![gitpoulpi_core::domain::RestoreItem {
                kind: gitpoulpi_core::domain::RestoreItemKind::File,
                label: "x.txt".into(),
            }],
            not_restorable: None,
        };
        let buf = paint(&mut panel);
        let (mut icon_xs, mut label_xs, mut age_ends) = (Vec::new(), Vec::new(), Vec::new());
        for (needle, family, ago) in rows_wanted {
            let y = y_of(&buf, needle);
            let icon = label::glyph(family).trim_end();
            let at = column_of(&buf, y, icon)
                .unwrap_or_else(|| panic!("{lang:?}: no {icon:?} on {needle:?}'s row"));
            icon_xs.push(at);
            // The label starts after the two-cell icon column and a space.
            assert_eq!(buf[(at + 2, y)].symbol(), " ", "{lang:?} {needle}");
            label_xs.push(buf[(at + 3, y)].symbol() != " ");
            let age = rows::age(now() - ago);
            let end = column_of(&buf, y, &age)
                .unwrap_or_else(|| panic!("{lang:?}: no age {age:?} on {needle:?}'s row"))
                + crate::display_width(&age) as u16;
            age_ends.push(end);
        }
        assert!(
            icon_xs.windows(2).all(|w| w[0] == w[1]),
            "{lang:?}: icons at {icon_xs:?}\n{}",
            screen(&buf)
        );
        assert!(label_xs.iter().all(|started| *started), "{lang:?}");
        assert!(
            age_ends.windows(2).all(|w| w[0] == w[1]),
            "{lang:?}: ages end at {age_ends:?}\n{}",
            screen(&buf)
        );
    }
}

#[test]
fn every_family_has_a_one_icon_two_cell_column() {
    use label::Family as F;
    for family in [
        F::Commit,
        F::Branch,
        F::Tag,
        F::Stash,
        F::Move,
        F::Merge,
        F::Discard,
        F::Edit,
        F::Staging,
        F::Config,
        F::Outside,
        F::Other,
    ] {
        let glyph = label::glyph(family);
        assert_eq!(crate::display_width(glyph), 2, "{family:?} {glyph:?}");
        assert!(
            !glyph.contains('\u{FE0F}') && !glyph.contains('\u{FE0E}'),
            "{family:?} needs no variation selector"
        );
    }
}
