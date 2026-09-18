//! The WIP row: its layout in the real Log table, its inline message field, its counters, and what selecting it shows.

use super::*;
use super::{changes_off, line, shown_order};
use crate::test_support::*;
use gitpoulpi_core::domain::{FileStatus, StatusEntry};
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn entry(path: &str, index: Option<FileStatus>, worktree: Option<FileStatus>) -> StatusEntry {
    StatusEntry {
        path: PathBuf::from(path),
        index,
        worktree,
    }
}

fn tree(files: Vec<StatusEntry>) -> WorkingTree {
    WorkingTree { files }
}

/// The Log table with a WIP row on top of `nodes`, drawn through the
/// same `log_table` and `wip_row` the event loop uses.
fn draw_log(summary: &str, editing: bool, counts: WipCounts) -> (ratatui::buffer::Buffer, Rect) {
    let nodes = vec![commit(1, &[2]), commit(2, &[])];
    let refs = vec![{
        let mut main = local_branch_at("main", oid(1));
        main.is_head = true;
        main
    }];
    let graph = crate::graph::build_graph_lines_with(
        &nodes,
        &refs,
        Some(crate::graph::WipAnchor { head: Some(oid(1)) }),
        &[],
        crate::graph::GraphSettings::default(),
    );
    let log_area = Rect::new(0, 0, 100, 10);
    let widths = LogColumnWidths::for_area_width(
        log_area.width,
        &changes_off(),
        &std::collections::HashMap::new(),
    );
    let order = shown_order();
    let mut rows = vec![wip_row(
        graph.wip.clone().expect("a WIP line"),
        None,
        &widths,
        &order,
        summary,
        editing,
        None,
        counts,
        wip_counts_at(&order, true),
        false,
        false,
    )];
    for (c, (line, color)) in nodes.iter().zip(graph.rows) {
        rows.push(commit_row(
            c,
            &refs,
            line,
            color,
            &widths,
            &order,
            false,
            false,
            None,
            false,
            avatar::LogAvatarMode::Off,
            gitpoulpi_config::AvatarFallback::Initials,
            None,
        ));
    }
    let mut terminal = Terminal::new(TestBackend::new(log_area.width, log_area.height)).unwrap();
    let mut state = TableState::default();
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = order.iter().map(|&c| Cell::from(c.title())).collect();
            frame.render_stateful_clipped(
                log_table(rows, header, &order, &widths, Style::default()),
                log_area,
                &mut state,
            );
        })
        .unwrap();
    (terminal.backend().buffer().clone(), log_area)
}

/// **One row, like a commit.** Through the real table: the WIP row is
/// the first body line, carrying the dotted marker and `// WIP`, and
/// the first commit is on the very next line - not two lines down, as
/// with the old two-line block.
/// Renders just the WIP row over `hidden` columns; returns the Message
/// and Changes cells' text.
fn wip_cells(hidden: &std::collections::HashSet<LogColumn>, at: WipCountsAt) -> (String, String) {
    let widths = LogColumnWidths::for_area_width(200, hidden, &std::collections::HashMap::new());
    let order: Vec<LogColumn> = LogColumn::ALL
        .into_iter()
        .filter(|c| !hidden.contains(c))
        .collect();
    let counts = WipCounts {
        modified: 29,
        added: 9,
        deleted: 1,
    };
    let row = wip_row(
        Line::default(),
        None,
        &widths,
        &order,
        "",
        false,
        None,
        counts,
        at,
        false,
        false,
    );
    let area = Rect::new(0, 0, 200, 1);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    let table = Table::new(
        vec![row],
        order
            .iter()
            .map(|c| Constraint::Length(widths.width_of(*c)))
            .collect::<Vec<_>>(),
    )
    .column_spacing(0);
    ratatui::widgets::Widget::render(table, area, &mut buf);
    let mut x = 0u16;
    let (mut message, mut changes) = (String::new(), String::new());
    for c in &order {
        let w = widths.width_of(*c);
        let text: String = (x..x + w)
            .map(|i| buf[(i, 0)].symbol().to_string())
            .collect();
        match c {
            LogColumn::Message => message = text,
            LogColumn::Changes => changes = text,
            _ => {}
        }
        x += w;
    }
    (message, changes)
}

#[test]
fn wip_counts_go_in_the_changes_column_when_it_is_visible_not_in_the_message() {
    let order = LogColumn::ALL.to_vec();
    let at = wip_counts_at(&order, true);
    assert_eq!(at, WipCountsAt::ChangesColumn);
    let (message, changes) = wip_cells(&std::collections::HashSet::new(), at);
    assert!(changes.contains("\u{270E} 29"), "{changes:?}");
    assert!(
        changes.contains("+ 9") && changes.contains("\u{2212} 1"),
        "{changes:?}"
    );
    assert!(
        !message.contains('\u{270E}') && !message.contains("+ 9"),
        "{message:?}"
    );
    // The setting does not matter while the column is visible.
    assert_eq!(wip_counts_at(&order, false), WipCountsAt::ChangesColumn);
}

#[test]
fn wip_counts_ride_in_the_message_when_the_column_is_hidden_and_the_setting_is_on() {
    let order = shown_order();
    let at = wip_counts_at(&order, true);
    assert_eq!(at, WipCountsAt::Message);
    let (message, changes) = wip_cells(&changes_off(), at);
    assert!(message.contains("\u{270E} 29"), "{message:?}");
    assert!(changes.is_empty());
}

#[test]
fn wip_counts_show_nowhere_when_the_column_is_hidden_and_the_setting_is_off() {
    let order = shown_order();
    let at = wip_counts_at(&order, false);
    assert_eq!(at, WipCountsAt::Nowhere);
    let (message, changes) = wip_cells(&changes_off(), at);
    assert!(
        !message.contains('\u{270E}') && !message.contains("+ 9"),
        "{message:?}"
    );
    assert!(changes.is_empty());
}

#[test]
fn the_wip_row_is_one_table_line_with_the_first_commit_right_under_it() {
    let (buf, area) = draw_log("", false, WipCounts::default());
    let top = log_top_y(area);
    let wip = line(&buf, top);
    assert!(wip.contains('\u{25CC}'), "the WIP marker: {wip:?}");
    assert!(wip.contains(WIP_PLACEHOLDER), "the placeholder: {wip:?}");
    let next = line(&buf, top + 1);
    assert!(
        next.contains("commit 1"),
        "the first commit must be on the line right under the WIP row: {next:?}"
    );
    // And the row arithmetic agrees with what was drawn.
    assert_eq!(log_row_to_index(area, 0, top, 3), Some(0));
    assert_eq!(log_row_to_index(area, 0, top + 1, 3), Some(1));
}

/// The field shows what is typed, the placeholder only while empty,
/// and a caret only while it has the keyboard.
#[test]
fn the_message_field_shows_the_summary_the_placeholder_and_the_caret() {
    let text = |l: &Line| {
        l.spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect::<String>()
    };
    let empty = text(&wip_message_line("", false, None, WipCounts::default(), 40));
    assert!(empty.starts_with(WIP_PLACEHOLDER), "{empty:?}");
    let typed = text(&wip_message_line(
        "fix login",
        false,
        None,
        WipCounts::default(),
        40,
    ));
    assert!(typed.starts_with("fix login"), "{typed:?}");
    assert!(!typed.contains(WIP_PLACEHOLDER), "{typed:?}");
    let editing = text(&wip_message_line(
        "fix",
        true,
        None,
        WipCounts::default(),
        40,
    ));
    assert!(
        editing.starts_with("fix_"),
        "the caret follows the text: {editing:?}"
    );
    let editing_empty = text(&wip_message_line("", true, None, WipCounts::default(), 40));
    assert!(
        editing_empty.starts_with('_') && !editing_empty.contains(WIP_PLACEHOLDER),
        "no placeholder under the caret: {editing_empty:?}"
    );
}

/// The counters sit at the same column however long the summary gets:
/// the field's width is chrome, not content.
#[test]
fn the_counters_do_not_move_as_the_summary_grows() {
    let counts = WipCounts {
        modified: 4,
        added: 8,
        deleted: 0,
    };
    let text = |s: &str, editing| {
        wip_message_line(s, editing, None, counts, 40)
            .spans
            .iter()
            .map(|s| s.content.to_string())
            .collect::<String>()
    };
    let at = |t: &str| display_width(&t[..t.find('\u{270E}').expect("the counter")]);
    for editing in [false, true] {
        let short = text("x", editing);
        let long = text(&"a very long summary indeed ".repeat(4), editing);
        assert_eq!(
            at(&short),
            at(&long),
            "editing={editing}: {short:?} / {long:?}"
        );
        assert!(
            long.contains("\u{270E} 4") && long.contains("+ 8"),
            "{long:?}"
        );
        assert!(
            !long.contains('\u{2212}'),
            "no deleted counter at zero: {long:?}"
        );
    }
}

/// Counted per file, as follows: a file staged and
/// further edited is one modification; a deletion on either side is a
/// deletion; new and untracked files are additions.
#[test]
fn change_counts_are_per_file() {
    use FileStatus::*;
    let counts = wip_change_counts(&tree(vec![
        entry("m", None, Some(Modified)),
        entry("both", Some(Staged), Some(Modified)),
        entry("renamed", Some(Renamed), None),
        entry("conflict", None, Some(Conflicted)),
        entry("new", Some(Added), None),
        entry("untracked", None, Some(Untracked)),
        entry("gone", None, Some(Deleted)),
        entry("staged-gone", Some(Deleted), None),
    ]));
    assert_eq!(
        counts,
        WipCounts {
            modified: 4,
            added: 2,
            deleted: 2,
        }
    );
}

/// **Selecting the WIP row shows the working-changes view, never the
/// commit detail** - whatever path set `CommitDetail`.
#[test]
fn selecting_the_wip_row_always_shows_the_working_changes_pane() {
    assert_eq!(
        right_panel_for_log_selection(RightPanel::CommitDetail, true),
        RightPanel::Status
    );
    assert_eq!(
        right_panel_for_log_selection(RightPanel::Status, true),
        RightPanel::Status
    );
    // A real commit keeps whatever the handler chose.
    assert_eq!(
        right_panel_for_log_selection(RightPanel::CommitDetail, false),
        RightPanel::CommitDetail
    );

    let dirty: gitpoulpi_core::Result<WorkingTree> =
        Ok(tree(vec![entry("a", None, Some(FileStatus::Modified))]));
    let clean: gitpoulpi_core::Result<WorkingTree> = Ok(tree(Vec::new()));
    let mut state = TableState::default();
    state.select(Some(0));
    assert!(wip_row_selected(&dirty, &state));
    assert!(
        !wip_row_selected(&clean, &state),
        "no WIP row on a clean tree"
    );
    state.select(Some(1));
    assert!(!wip_row_selected(&dirty, &state));
}

/// **The WIP field and the composer's Summary are one buffer.** Typed
/// into from either side - `WipSummary` or `Summary` focus, which
/// are the one `MainField::Summary` - and both renderings show it on
/// the next frame.
#[test]
fn typing_in_either_field_shows_in_both() {
    let mut summary = String::new();
    let description = String::new();
    let mut editor = text_edit::FieldEditor::<MainField>::default();
    let field_of = |focus| MainField::active(&InputMode::Normal, Some(focus), false).unwrap();
    assert_eq!(field_of(ComposerField::WipSummary), MainField::Summary);
    assert_eq!(field_of(ComposerField::Summary), MainField::Summary);
    let type_in =
        |editor: &mut text_edit::FieldEditor<MainField>, summary: &mut String, focus, s: &str| {
            for c in s.chars() {
                editor.on_key(
                    field_of(focus),
                    summary,
                    &KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE),
                    false,
                );
            }
        };
    type_in(
        &mut editor,
        &mut summary,
        ComposerField::WipSummary,
        "from log",
    );
    type_in(
        &mut editor,
        &mut summary,
        ComposerField::Summary,
        " and pane",
    );
    assert_eq!(summary, "from log and pane");
    assert!(
        description.is_empty(),
        "neither field writes the description"
    );

    // The Log's row...
    let (buf, area) = draw_log(&summary, false, WipCounts::default());
    let wip = line(&buf, log_top_y(area));
    assert!(wip.contains("from log and pane"), "{wip:?}");
    // ...and the composer under Changed files. Row 3 at
    // `COMPOSER_MIN_HEIGHT`: sep(0), amend(1), the box's top
    // border(2), Summary(3) -- there's no "Summary:" label prefix
    // any more, the bordered box itself is the field.
    let mut terminal = Terminal::new(TestBackend::new(60, COMPOSER_MIN_HEIGHT)).unwrap();
    terminal
        .draw(|frame| {
            let mut hits: Vec<(Rect, HitTarget)> = Vec::new();
            render_commit_composer(
                frame,
                Rect::new(0, 0, 60, COMPOSER_MIN_HEIGHT),
                &mut hits,
                &summary,
                &description,
                false,
                Some(ComposerField::WipSummary),
                1,
                false,
                ComposerHover::default(),
                &text_edit::FieldEditor::default(),
            );
        })
        .unwrap();
    let pane = line(terminal.backend().buffer(), 3);
    assert!(pane.contains("from log and pane"), "{pane:?}");

    // Backspace from the Log edits the pane's text too.
    editor.on_key(
        field_of(ComposerField::WipSummary),
        &mut summary,
        &KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
        false,
    );
    assert_eq!(summary, "from log and pan");
}

/// Enter from the WIP field goes on to the Description, exactly as
/// Enter from the composer's own Summary does; Enter in the
/// Description is a new line, and the commit chord
/// (`Ctrl+Enter`) commits from any of them.
#[test]
fn enter_moves_from_either_summary_to_the_description_then_breaks_lines() {
    assert_eq!(
        composer_enter(ComposerField::WipSummary, false),
        ComposerEnter::Next(ComposerField::Description)
    );
    assert_eq!(
        composer_enter(ComposerField::Summary, false),
        ComposerEnter::Next(ComposerField::Description)
    );
    assert_eq!(
        composer_enter(ComposerField::Description, false),
        ComposerEnter::Newline
    );
    for field in [
        ComposerField::Summary,
        ComposerField::WipSummary,
        ComposerField::Description,
    ] {
        assert_eq!(composer_enter(field, true), ComposerEnter::Commit);
    }
    let chord = |code, mods| is_commit_chord(&KeyEvent::new(code, mods));
    assert!(chord(KeyCode::Enter, KeyModifiers::CONTROL));
    assert!(chord(KeyCode::Enter, KeyModifiers::ALT));
    assert!(chord(KeyCode::Char('j'), KeyModifiers::CONTROL));
    assert!(!chord(KeyCode::Enter, KeyModifiers::NONE));
    assert!(!chord(KeyCode::Enter, KeyModifiers::SHIFT));
    assert!(!chord(KeyCode::Char('j'), KeyModifiers::NONE));
}

/// The keyboard twin of hovering the WIP row: true only when the
/// selection is on it, never on a commit or a stash row - what tells
/// the menu key to open the WIP row's own context menu instead of the
/// per-commit one.
#[test]
fn selected_log_is_wip_is_true_only_on_the_pinned_wip_row() {
    let dirty = graph::LogRows::new(true, Vec::new());
    let clean = graph::LogRows::default();

    let mut state = TableState::default();
    state.select(Some(0));
    assert!(
        selected_log_is_wip(&state, &dirty),
        "row 0 on a dirty tree is the WIP row"
    );
    assert!(
        !selected_log_is_wip(&state, &clean),
        "row 0 on a clean tree is the first commit, not WIP"
    );

    state.select(Some(1));
    assert!(
        !selected_log_is_wip(&state, &dirty),
        "row 1 on a dirty tree is the first commit"
    );

    state.select(None);
    assert!(!selected_log_is_wip(&state, &dirty), "no selection at all");
}
