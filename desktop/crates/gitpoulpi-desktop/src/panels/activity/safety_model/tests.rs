use super::*;
use gitpoulpi_core::domain::{EntryState, EntrySummary, OperationKind, RestoreOffer};

fn entry(id: u64, at: i64) -> ListedEntry {
    ListedEntry {
        source: EntrySource::Gitpoulpi,
        outside: None,
        summary: EntrySummary {
            id,
            kind: OperationKind::Commit,
            label: format!("commit {id}"),
            at,
            state: EntryState::Done,
            origin: None,
        },
        changes: Vec::new(),
        restore: RestoreOffer::default(),
    }
}

#[test]
fn the_list_puts_undone_entries_above_the_divider_and_never_selects_it() {
    let mut model = HistoryModel::default();
    // The daemon lists oldest first: 1, 2, 3 undoable and 4, 5 undone (5 comes back first).
    model.set(JournalListing {
        undo: vec![entry(1, 100), entry(2, 101), entry(3, 102)],
        redo: vec![entry(5, 104), entry(4, 103)],
        retired: Vec::new(),
    });
    assert_eq!(model.undo_count, 3);
    assert!(model.lines.contains(&Line::Redo));
    // The newest undoable entry is selected first, never the divider.
    assert_eq!(model.selected().unwrap().entry.summary.id, 3);
    model.move_by(-1);
    assert!(
        model.selected().unwrap().redo,
        "up from the stack crosses the divider"
    );
    model.move_by(-1);
    model.move_by(-1);
    assert_eq!(
        model.selected_row(),
        Some(model.rows.len() - 1),
        "the top of the list stays put"
    );
}

#[test]
fn undo_up_to_here_covers_newest_first_and_redo_oldest_first() {
    let mut model = HistoryModel::default();
    model.set(JournalListing {
        undo: vec![entry(1, 100), entry(2, 101), entry(3, 102)],
        redo: vec![entry(5, 104), entry(4, 103)],
        retired: Vec::new(),
    });
    model.select_entry(1);
    let (dir, rows) = model.steps_for_selection().unwrap();
    assert_eq!(dir, gitpoulpi_core::domain::Direction::Undo);
    let ids: Vec<u64> = rows.iter().map(|r| r.entry.summary.id).collect();
    assert_eq!(ids, [3, 2, 1]);
    model.select_entry(5);
    let (dir, rows) = model.steps_for_selection().unwrap();
    assert_eq!(dir, gitpoulpi_core::domain::Direction::Redo);
    let ids: Vec<u64> = rows.iter().map(|r| r.entry.summary.id).collect();
    assert_eq!(
        ids,
        [4, 5],
        "the next one to redo first, up to the chosen one"
    );
}

#[test]
fn a_selection_survives_a_reload_by_entry_id() {
    let mut model = HistoryModel::default();
    model.set(JournalListing {
        undo: vec![entry(1, 100), entry(2, 101)],
        ..Default::default()
    });
    model.select_entry(1);
    model.set(JournalListing {
        undo: vec![entry(1, 100), entry(2, 101), entry(3, 102)],
        ..Default::default()
    });
    assert_eq!(model.selected().unwrap().entry.summary.id, 1);
}

#[test]
fn day_headings_appear_only_when_the_entries_span_days() {
    let same = [entry(1, 100), entry(2, 200)].map(|entry| HistoryRow { entry, redo: false });
    assert!(!build_lines(&same, 2)
        .iter()
        .any(|l| matches!(l, Line::Day(_))));
    let spread = [entry(1, 100), entry(2, 200_000)].map(|entry| HistoryRow { entry, redo: false });
    assert!(build_lines(&spread, 2)
        .iter()
        .any(|l| matches!(l, Line::Day(_))));
}

#[test]
fn a_blocked_top_entry_is_blocked_and_only_the_top_one() {
    let rows: Vec<HistoryRow> = [entry(1, 100), entry(2, 101)]
        .into_iter()
        .map(|entry| HistoryRow { entry, redo: false })
        .collect();
    let state = JournalState {
        blocked: Some(gitpoulpi_core::domain::UndoRefusal::IndexChanged),
        ..Default::default()
    };
    assert!(matches!(
        row_state(&rows[0], 0, &state),
        RowState::Blocked(_)
    ));
    assert_eq!(row_state(&rows[1], 1, &state), RowState::Undoable);
}

#[test]
fn the_reflog_restores_only_a_branch_or_head() {
    let model = |name: &str| ReflogModel {
        refname: name.to_string(),
        ..Default::default()
    };
    assert_eq!(model("HEAD").target().as_deref(), Some("HEAD"));
    assert_eq!(model("refs/heads/main").target().as_deref(), Some("main"));
    assert_eq!(model("refs/tags/v1").target(), None);
}
