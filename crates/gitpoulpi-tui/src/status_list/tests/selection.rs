use super::*;

fn states() -> StatusStates {
    StatusStates::default()
}

fn lines_of(tree: &WorkingTree, mode: FileListMode) -> Vec<(StatusSection, Vec<StatusPaneLine>)> {
    StatusSection::ORDER
        .iter()
        .map(|&s| {
            (
                s,
                build_status_section_lines(tree, s, mode, &HashSet::new(), ""),
            )
        })
        .collect()
}

// The wheel scrolls a working-changes sub-pane's
// viewport, never its selection.

/// A long `Unstaged Files` list, tall enough that a wheel notch and a
/// visible area of 5 rows genuinely leaves rows off screen.
fn many_unstaged(n: usize) -> WorkingTree {
    let names: Vec<String> = (0..n).map(|i| format!("f{i}.rs")).collect();
    let entries: Vec<(&str, bool, bool)> =
        names.iter().map(|n| (n.as_str(), false, true)).collect();
    tree(&entries)
}

/// The two halves are separate sub-panes, so neither carries a header
/// row - the header is the sub-pane's block title. A row index is an
/// index into that ONE section, never into a combined list.
#[test]
fn each_sub_pane_holds_only_its_own_files_and_no_header_row() {
    let t = tree(&[("a.rs", false, true), ("b.rs", true, false)]);
    let all = lines_of(&t, FileListMode::Path);
    let unstaged = section_lines(&all, StatusSection::Unstaged);
    let staged = section_lines(&all, StatusSection::Staged);
    assert_eq!(unstaged.len(), 1);
    assert_eq!(staged.len(), 1);
    assert_eq!(unstaged[0].file().unwrap().1, &PathBuf::from("a.rs"));
    assert_eq!(staged[0].file().unwrap().1, &PathBuf::from("b.rs"));
}

#[test]
fn an_empty_sub_pane_simply_has_no_rows() {
    let t = tree(&[("a.rs", false, true)]);
    let all = lines_of(&t, FileListMode::Path);
    assert_eq!(section_lines(&all, StatusSection::Staged).len(), 0);
}

/// `Ctrl+G s`/`u` resolve the selected row to a path. Each acts on its
/// own sub-pane's rows, so index 0 means a different file in each.
#[test]
fn staging_the_selected_row_only_acts_on_the_matching_section() {
    let t = tree(&[("a.rs", false, true), ("b.rs", true, false)]);
    let all = lines_of(&t, FileListMode::Path);
    assert_eq!(
        selected_status_path(section_lines(&all, StatusSection::Unstaged), Some(0), false),
        Some(PathBuf::from("a.rs"))
    );
    // The unstaged sub-pane's row can't be unstaged.
    assert_eq!(
        selected_status_path(section_lines(&all, StatusSection::Unstaged), Some(0), true),
        None
    );
    assert_eq!(
        selected_status_path(section_lines(&all, StatusSection::Staged), Some(0), true),
        Some(PathBuf::from("b.rs"))
    );
    assert_eq!(
        selected_status_path(section_lines(&all, StatusSection::Unstaged), None, false),
        None
    );
}

/// Staging a file empties one sub-pane and fills the other. A
/// selection left past the new end must resolve to nothing rather than
/// acting on an unrelated row.
#[test]
fn a_selection_left_past_the_end_after_staging_resolves_to_nothing() {
    let after = tree(&[("a.rs", true, false)]);
    let all = lines_of(&after, FileListMode::Path);
    let unstaged = section_lines(&all, StatusSection::Unstaged);
    assert!(unstaged.is_empty());
    assert_eq!(selected_status_path(unstaged, Some(0), false), None);
    assert_eq!(move_index(Some(3), unstaged.len(), 0), None);
}

#[test]
fn a_partially_staged_file_appears_in_both_sub_panes() {
    let t = tree(&[("both.rs", true, true)]);
    let all = lines_of(&t, FileListMode::Path);
    assert_eq!(
        selected_status_path(section_lines(&all, StatusSection::Unstaged), Some(0), false),
        Some(PathBuf::from("both.rs"))
    );
    assert_eq!(
        selected_status_path(section_lines(&all, StatusSection::Staged), Some(0), true),
        Some(PathBuf::from("both.rs"))
    );
}

#[test]
fn tree_mode_puts_folder_rows_before_their_files_inside_a_sub_pane() {
    let t = tree(&[("crates/tui/a.rs", false, true)]);
    let all = lines_of(&t, FileListMode::Tree);
    let rows = section_lines(&all, StatusSection::Unstaged);
    // crates/, tui/, a.rs -- and no header row.
    assert_eq!(rows.len(), 3);
    assert!(matches!(rows[0], StatusPaneLine::Folder { .. }));
    assert!(matches!(rows[1], StatusPaneLine::Folder { .. }));
    // A folder row is not a file row: staging must skip it.
    assert_eq!(selected_status_path(rows, Some(0), false), None);
    assert_eq!(
        selected_status_path(rows, Some(2), false),
        Some(PathBuf::from("crates/tui/a.rs"))
    );
}

/// Stepping off the bottom of `Unstaged` lands on `Staged`'s first
/// row instead of sticking - the edge-crossing the sidebar's sections
/// got, so pane 1 and pane 3 navigate alike.
#[test]
fn moving_down_past_the_last_unstaged_row_crosses_into_the_staged_sub_pane() {
    let t = tree(&[("a.rs", false, true), ("b.rs", true, false)]);
    let all = lines_of(&t, FileListMode::Path);
    let mut st = states();
    let mut active = StatusSection::Unstaged;
    st.state_mut(StatusSection::Unstaged).select(Some(0));

    move_status_selection(&mut st, &all, &HashSet::new(), &mut active, 1);
    assert_eq!(active, StatusSection::Staged);
    assert_eq!(st.state(StatusSection::Staged).selected(), Some(0));

    // ...and back up again, landing on the last unstaged row.
    move_status_selection(&mut st, &all, &HashSet::new(), &mut active, -1);
    assert_eq!(active, StatusSection::Unstaged);
    assert_eq!(st.state(StatusSection::Unstaged).selected(), Some(0));
}

#[test]
fn navigation_skips_a_collapsed_sub_pane() {
    let t = tree(&[("a.rs", false, true), ("b.rs", true, false)]);
    let all = lines_of(&t, FileListMode::Path);
    let mut collapsed = HashSet::new();
    collapsed.insert(StatusSection::Staged);
    let mut st = states();
    let mut active = StatusSection::Unstaged;
    st.state_mut(StatusSection::Unstaged).select(Some(0));
    // Staged is collapsed, so there is nowhere to cross to: stay put.
    move_status_selection(&mut st, &all, &collapsed, &mut active, 1);
    assert_eq!(active, StatusSection::Unstaged);
    assert_eq!(st.state(StatusSection::Unstaged).selected(), Some(0));
}

#[test]
fn navigation_lands_on_a_usable_sub_pane_when_the_active_one_is_empty() {
    // Nothing unstaged at all, so starting there must not dead-end.
    let t = tree(&[("b.rs", true, false)]);
    let all = lines_of(&t, FileListMode::Path);
    let mut st = states();
    let mut active = StatusSection::Unstaged;
    move_status_selection(&mut st, &all, &HashSet::new(), &mut active, 1);
    assert_eq!(active, StatusSection::Staged);
    assert_eq!(st.state(StatusSection::Staged).selected(), Some(0));
}

/// The wheel must move the sub-pane under the pointer, not whichever
/// one happens to be active - this is what the shared (section, area)
/// list buys.
#[test]
fn the_wheel_scrolls_the_sub_pane_under_the_cursor() {
    // The wheel scrolls whichever sub-pane the
    // pointer is over, but only its *viewport* -- selection is
    // untouched and only moves on a click (see the
    // `wheel_over_a_sub_pane_*` tests below for that half).
    let t = many_unstaged(20);
    let all = lines_of(&t, FileListMode::Path);
    let areas = vec![
        (StatusSection::Unstaged, Rect::new(0, 0, 40, 5)),
        (StatusSection::Staged, Rect::new(0, 5, 40, 5)),
    ];
    let mut st = states();
    st.select(StatusSection::Unstaged, Some(0));
    // Pointer over the unstaged half (the only one with any rows here).
    scroll_status_pane(&areas, &mut st, &all, 1, 1, 1);
    assert_eq!(
        st.state(StatusSection::Unstaged).offset(),
        1,
        "the sub-pane under the pointer must scroll"
    );
    assert_eq!(
        st.state(StatusSection::Unstaged).selected(),
        Some(0),
        "but its selection must not move"
    );
    assert_eq!(
        st.state(StatusSection::Staged).offset(),
        0,
        "the other sub-pane must not have moved"
    );
    // Pointer outside both: nothing moves.
    scroll_status_pane(&areas, &mut st, &all, 1, 99, 1);
    assert_eq!(st.state(StatusSection::Unstaged).offset(), 1);
}

#[test]
fn collapse_all_covers_folders_from_both_sub_panes() {
    let t = tree(&[("a/one.rs", false, true), ("b/two.rs", true, false)]);
    let all = lines_of(&t, FileListMode::Tree);
    let folders = all_status_fold_keys(&all);
    assert!(folders.contains(&fold_key(StatusSection::Unstaged, Path::new("a"))));
    assert!(folders.contains(&fold_key(StatusSection::Staged, Path::new("b"))));
}

/// The maintainer's report: folding `src/` in Staged Files folded it in
/// Unstaged Files too, and back. Each sub-pane folds on its own.
#[test]
fn folding_a_folder_in_one_sub_pane_leaves_the_other_open() {
    let t = tree(&[("src/a.rs", true, true)]);
    let all = lines_of(&t, FileListMode::Tree);
    let StatusPaneLine::Folder { fold_key, .. } = &section_lines(&all, StatusSection::Staged)[0]
    else {
        panic!("a folder row first");
    };
    let collapsed: HashSet<PathBuf> = [fold_key.clone()].into();
    let rows =
        |section| build_status_section_lines(&t, section, FileListMode::Tree, &collapsed, "");
    assert_eq!(rows(StatusSection::Staged).len(), 1, "Staged: src/ folded");
    assert_eq!(
        rows(StatusSection::Unstaged).len(),
        2,
        "Unstaged: src/ still open"
    );
}

/// `section_paths` pins a conflicted file to the Unstaged half, and
/// Tree mode inserts a folder row above every file - so resolving the
/// selection against the wrong row model is how "Take theirs" lands on
/// a file the user never pointed at.
#[test]
fn the_selected_conflicted_path_survives_tree_mode_folder_rows() {
    use gitpoulpi_core::domain::{ConflictEntry, FileStatus, Oid, StageEntry, StatusEntry};

    let entry = |path: &str| StatusEntry {
        path: PathBuf::from(path),
        index: Some(FileStatus::Conflicted),
        worktree: Some(FileStatus::Conflicted),
    };
    let tree = Ok(WorkingTree {
        files: vec![
            entry("src/a.rs"),
            StatusEntry {
                path: PathBuf::from("src/b.rs"),
                index: None,
                worktree: Some(FileStatus::Modified),
            },
        ],
    });
    let stage = |n: u8| {
        Some(StageEntry {
            mode: 0o100644,
            oid: Oid([n; 20]),
        })
    };
    let conflicts =
        vec![
            ConflictEntry::from_stages(PathBuf::from("src/a.rs"), stage(1), stage(2), stage(3))
                .unwrap(),
        ];
    let collapsed = HashSet::new();

    // Path mode: row 0 is `src/a.rs` itself.
    let mut sel = states();
    sel.state_mut(StatusSection::Unstaged).select(Some(0));
    assert_eq!(
        selected_conflicted_path(&tree, &sel, FileListMode::Path, &collapsed, &conflicts),
        Some(PathBuf::from("src/a.rs"))
    );

    // Tree mode: row 0 is the `src` folder, row 1 is `a.rs`. Selecting
    // the folder finds nothing; selecting the file finds the conflict.
    let mut sel = states();
    sel.state_mut(StatusSection::Unstaged).select(Some(0));
    assert_eq!(
        selected_conflicted_path(&tree, &sel, FileListMode::Tree, &collapsed, &conflicts),
        None,
        "a folder row is not a conflicted file"
    );
    sel.state_mut(StatusSection::Unstaged).select(Some(1));
    assert_eq!(
        selected_conflicted_path(&tree, &sel, FileListMode::Tree, &collapsed, &conflicts),
        Some(PathBuf::from("src/a.rs"))
    );

    // And a non-conflicted row in the same list is not offered the
    // resolutions.
    sel.state_mut(StatusSection::Unstaged).select(Some(2));
    assert_eq!(
        selected_conflicted_path(&tree, &sel, FileListMode::Tree, &collapsed, &conflicts),
        None
    );
}

#[test]
fn wheel_over_a_sub_pane_moves_its_offset_and_leaves_the_selection_alone() {
    let t = many_unstaged(20);
    let all = lines_of(&t, FileListMode::Path);
    let mut states = states();
    states.select(StatusSection::Unstaged, Some(2));
    let areas = [
        (StatusSection::Unstaged, Rect::new(0, 0, 30, 7)), // 5 content rows
        (StatusSection::Staged, Rect::new(0, 7, 30, 7)),
    ];

    scroll_status_pane(&areas, &mut states, &all, 3, 3, 3);

    assert_eq!(
        states.state(StatusSection::Unstaged).selected(),
        Some(2),
        "the wheel must never move the selection"
    );
    assert_eq!(
        states.state(StatusSection::Unstaged).offset(),
        3,
        "but it must move the offset"
    );
    assert!(
        !states.follow(StatusSection::Unstaged),
        "a wheel-scrolled sub-pane stops following its selection until \
         something else (keyboard, click) touches it again"
    );
}

#[test]
fn wheel_over_a_sub_pane_cannot_scroll_the_offset_past_its_own_content() {
    let t = many_unstaged(6); // fewer rows than the visible area below
    let all = lines_of(&t, FileListMode::Path);
    let mut states = states();
    let areas = [(StatusSection::Unstaged, Rect::new(0, 0, 30, 10))];

    scroll_status_pane(&areas, &mut states, &all, 3, 3, 50);

    assert_eq!(
        states.state(StatusSection::Unstaged).offset(),
        0,
        "everything already fits - there is nothing to scroll to"
    );
}

#[test]
fn a_keyboard_move_after_a_wheel_scroll_snaps_the_pane_back_to_following() {
    let t = many_unstaged(20);
    let all = lines_of(&t, FileListMode::Path);
    let mut states = states();
    states.select(StatusSection::Unstaged, Some(0));
    let areas = [(StatusSection::Unstaged, Rect::new(0, 0, 30, 7))];
    scroll_status_pane(&areas, &mut states, &all, 3, 3, 10);
    assert!(!states.follow(StatusSection::Unstaged));

    let mut active = StatusSection::Unstaged;
    move_status_selection(&mut states, &all, &HashSet::new(), &mut active, 1);

    assert!(
        states.follow(StatusSection::Unstaged),
        "a keyboard move (through move_status_selection, which itself \
         goes through StatusStates::select) must re-arm following"
    );
}
