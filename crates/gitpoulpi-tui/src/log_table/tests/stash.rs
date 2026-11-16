//! Stash rows in the Log: selection arithmetic, detail and drawing.

use super::*;
use super::{changes_off, line, shown_order};
use crate::test_support::*;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

// ---- Stash rows --------------------------------------------------------

/// `stash@{0}`, made on commit 2 after commit 1 was written: its row
/// goes between commits 1 and 2.
fn stash_on_2() -> gitpoulpi_core::domain::Stash {
    gitpoulpi_core::domain::Stash {
        id: gitpoulpi_core::domain::StashId(0),
        message: "WIP on main: 0202020 commit 2".into(),
        branch_at_creation: "main".into(),
        timestamp: 15,
        commit: Some(oid(200)),
        base: Some(oid(2)),
    }
}

fn dated_history() -> Vec<CommitNode> {
    let mut nodes = vec![commit(1, &[2]), commit(2, &[3]), commit(3, &[])];
    for (n, t) in nodes.iter_mut().zip([20, 10, 5]) {
        n.timestamp = t;
    }
    nodes
}

/// **The Log's selection arithmetic with a stash row between two
/// commits**, on a dirty tree: display 0 is the WIP row, 1 commit 1,
/// 2 the stash, 3 and 4 commits 2 and 3. A selection on the stash row
/// is no commit (nothing to cherry-pick or reset onto) but is that
/// stash; one below it is the commit actually drawn there; and
/// selecting a commit by id lands on its row, not one above.
#[test]
fn a_stash_row_shifts_the_commits_below_it_and_is_its_own_selection() {
    let log = Ok(gitpoulpi_core::domain::CommitGraph {
        nodes: dated_history(),
    });
    let dirty = Ok(WorkingTree {
        files: vec![gitpoulpi_core::domain::StatusEntry {
            renamed_from: None,
            path: PathBuf::from("wip.rs"),
            index: None,
            worktree: Some(gitpoulpi_core::domain::FileStatus::Modified),
        }],
        ..WorkingTree::default()
    });
    let stashes = Ok(vec![stash_on_2()]);
    let rows = log_rows_now(
        &log,
        &dirty,
        &stashes,
        true,
        &Default::default(),
        &Ok(Vec::new()),
    );
    assert_eq!(rows.len(3), 5);

    let mut state = TableState::default();
    let at = |state: &mut TableState, i| {
        state.select(Some(i));
        (
            selected_log_commit(&log, state, &rows),
            selected_log_stash(&stashes, state, &rows).map(|s| s.id.0),
        )
    };
    assert_eq!(at(&mut state, 0), (None, None), "the WIP row");
    assert_eq!(at(&mut state, 1), (Some(oid(1)), None));
    assert_eq!(at(&mut state, 2), (None, Some(0)), "the stash row");
    assert_eq!(at(&mut state, 3), (Some(oid(2)), None));
    assert_eq!(at(&mut state, 4), (Some(oid(3)), None));

    assert!(select_log_commit(&log, &rows, &mut state, oid(2)));
    assert_eq!(state.selected(), Some(3));

    // The setting off, or soloing: no stash row, nothing shifts.
    let off = log_rows_now(
        &log,
        &dirty,
        &stashes,
        false,
        &Default::default(),
        &Ok(Vec::new()),
    );
    assert_eq!(off.len(3), 4);
    state.select(Some(2));
    assert_eq!(selected_log_commit(&log, &state, &off), Some(oid(2)));
    let solo: HashSet<String> = ["refs/heads/main".to_string()].into_iter().collect();
    assert!(!stash_rows_shown(true, &solo));
    assert!(stash_rows_shown(true, &HashSet::new()));
    assert!(!stash_rows_shown(false, &HashSet::new()));
}

/// A stash detailed in the right pane is its own commit against its
/// base - exactly what it changed - named by its handle rather than
/// an author with an empty address.
#[test]
fn a_stash_is_detailed_as_its_own_commit_on_its_base() {
    let node = stash_as_commit(&stash_on_2()).expect("a stash with a commit");
    assert_eq!(node.id, oid(200));
    assert_eq!(node.parents, vec![oid(2)]);
    assert_eq!(node.summary, "WIP on main: 0202020 commit 2");
    assert_eq!(
        author_label(&node.author_name, &node.author_email),
        "stash@{0}"
    );
    assert_eq!(author_label("A", "a@example.com"), "A <a@example.com>");
}

/// Through the real table: the stash row is one line between commits
/// 1 and 2, its Graph cell the `□` box, its Message the stash's own
/// message, and the commits keep their own rows around it.
#[test]
fn the_stash_row_is_drawn_between_the_commits_it_sits_between() {
    let nodes = dated_history();
    let stash = stash_on_2();
    let drawable = crate::graph::GraphStash::from_stashes(std::slice::from_ref(&stash));
    let graph = crate::graph::build_graph_lines_with(
        &nodes,
        &[],
        None,
        &drawable,
        crate::graph::GraphSettings::default(),
    );
    let order = graph.grid.log_rows();
    let log_area = Rect::new(0, 0, 100, 10);
    let widths = LogColumnWidths::for_area_width(
        log_area.width,
        &changes_off(),
        &std::collections::HashMap::new(),
    );
    let columns = shown_order();
    let mut commit_lines = graph.rows.into_iter();
    let mut stash_lines = graph.stashes.into_iter();
    let rows: Vec<Row> = (0..order.len(nodes.len()))
        .map(|d| match order.entry(d) {
            crate::graph::LogEntry::Commit(i) => {
                let (line, color) = commit_lines.next().unwrap();
                commit_row(
                    &nodes[i],
                    &[],
                    line,
                    color,
                    &widths,
                    &columns,
                    false,
                    false,
                    None,
                    false,
                    avatar::LogAvatarMode::Off,
                    gitpoulpi_config::AvatarFallback::Initials,
                    None,
                )
            }
            crate::graph::LogEntry::Stash(_) => stash_log_row(
                &stash,
                stash_lines.next().unwrap(),
                None,
                &widths,
                &columns,
                false,
                false,
            ),
            crate::graph::LogEntry::Wip => unreachable!("a clean tree"),
        })
        .collect();
    let mut terminal = Terminal::new(TestBackend::new(log_area.width, log_area.height)).unwrap();
    let mut state = TableState::default();
    terminal
        .draw(|frame| {
            let header: Vec<Cell> = columns.iter().map(|&c| Cell::from(c.title())).collect();
            frame.render_stateful_clipped(
                log_table(rows, header, &columns, &widths, Style::default()),
                log_area,
                &mut state,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let top = log_top_y(log_area);
    assert!(line(&buf, top).contains("commit 1"));
    let stash_line = line(&buf, top + 1);
    assert!(stash_line.contains('\u{25A1}'), "the box: {stash_line:?}");
    assert!(
        stash_line.contains("WIP on main"),
        "the message: {stash_line:?}"
    );
    assert!(
        stash_line.contains("stash@{0}"),
        "the handle: {stash_line:?}"
    );
    assert!(line(&buf, top + 2).contains("commit 2"));
    assert!(line(&buf, top + 3).contains("commit 3"));
    // And a click on each line resolves to what is drawn there.
    assert_eq!(
        order.entry(log_row_to_index(log_area, 0, top + 1, 4).unwrap()),
        crate::graph::LogEntry::Stash(0)
    );
    assert_eq!(
        order.entry(log_row_to_index(log_area, 0, top + 2, 4).unwrap()),
        crate::graph::LogEntry::Commit(1)
    );
}
