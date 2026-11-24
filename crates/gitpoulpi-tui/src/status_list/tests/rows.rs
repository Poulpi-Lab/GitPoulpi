use super::*;

/// 2,000 files: only the paint window's rows are built.
#[test]
fn status_rows_built_equal_the_visible_rows_not_the_file_count() {
    use gitpoulpi_core::domain::{FileStatus, StatusEntry};
    let tree = WorkingTree {
        files: (0..2000)
            .map(|i| StatusEntry {
                renamed_from: None,
                path: PathBuf::from(format!("dir{}/file{i}.rs", i % 40)),
                index: None,
                worktree: Some(FileStatus::Modified),
            })
            .collect(),
        ..WorkingTree::default()
    };
    let lines = build_status_section_lines(
        &tree,
        StatusSection::Unstaged,
        FileListMode::Tree,
        &HashSet::new(),
        "",
    );
    assert!(lines.len() >= 2000);
    let lfs = crate::lfs_ui::LfsUi::default();
    let area = Rect::new(0, 5, 40, 12);
    let mut state = ListState::default();
    *state.offset_mut() = 300;
    state.select(Some(305));
    for follow in [false, true] {
        STATUS_ROWS_BUILT.with(|n| n.set(0));
        let mut hits = Vec::new();
        let items = status_row_items(
            &lines,
            StatusSection::Unstaged,
            None,
            &state,
            follow,
            area,
            &lfs,
            &FileSelection::default(),
            0,
            &mut hits,
        );
        assert_eq!(items.len(), lines.len(), "the list keeps one item per row");
        assert_eq!(STATUS_ROWS_BUILT.with(|n| n.get()), 12);
    }
    // A followed selection far below widens the window to reach it.
    state.select(Some(900));
    assert_eq!(status_paint_window(lines.len(), &state, true, 12), 300..901);
    // Hidden (wheel-scrolled away) selection does not.
    assert_eq!(
        status_paint_window(lines.len(), &state, false, 12),
        300..312
    );
}

/// Closes the `HitTarget::StatusFolderRow` hover gap
/// (`chrome.rs`'s `hover_coverage`): a folder row's own name span
/// must lift its background on hover, distinct from (and in
/// addition to) the Stage/Unstage folder button `hovered` already
/// reveals.
#[test]
fn a_hovered_folder_row_lifts_its_name_spans_background() {
    use ratatui::backend::TestBackend;

    let line = StatusPaneLine::Folder {
        path: PathBuf::from("src"),
        fold_key: PathBuf::from("<unstaged>/src"),
        depth: 0,
        label: "src".to_string(),
        collapsed: false,
    };
    let list_area = Rect::new(0, 0, 30, 1);
    for hovered in [false, true] {
        let mut hits = Vec::new();
        let item = status_row_item(
            &line,
            StatusSection::Unstaged,
            hovered,
            true,
            list_area,
            0,
            LfsMark::default(),
            false,
            0,
            &mut hits,
        );
        let mut terminal = Terminal::new(TestBackend::new(30, 1)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_clipped(List::new(vec![item]), Rect::new(0, 0, 30, 1));
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        // The chevron cell, always part of the name span (never the
        // revealed button, which sits at the row's right end).
        let lifted = buf[(0, 0)].bg == th().ui.hover;
        assert_eq!(
            lifted, hovered,
            "folder name background must lift on hover and only on hover"
        );
    }
}

/// A file stored with Git LFS says so on its row (and only then).
/// Falsified by deleting the `if lfs` span in `status_row_item`.
#[test]
fn an_lfs_file_row_carries_an_lfs_tag_and_a_plain_row_does_not() {
    use ratatui::backend::TestBackend;

    let line = StatusPaneLine::File {
        path: PathBuf::from("assets/hero.psd"),
        depth: 0,
        section: StatusSection::Unstaged,
        label: "assets/hero.psd".to_string(),
        status: Some(gitpoulpi_core::domain::FileStatus::Modified),
    };
    let row_text = |lfs: bool, lock: Option<&str>| -> String {
        let mut hits = Vec::new();
        let item = status_row_item(
            &line,
            StatusSection::Unstaged,
            false,
            true,
            Rect::new(0, 0, 40, 1),
            0,
            LfsMark {
                is_lfs: lfs,
                locked_by: lock,
            },
            false,
            0,
            &mut hits,
        );
        let mut terminal = Terminal::new(TestBackend::new(40, 1)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_clipped(List::new(vec![item]), Rect::new(0, 0, 40, 1));
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        (0..40).map(|x| buf[(x, 0)].symbol().to_string()).collect()
    };
    assert!(
        row_text(true, None).contains("hero.psd LFS"),
        "{:?}",
        row_text(true, None)
    );
    assert!(
        !row_text(false, None).contains("LFS"),
        "{:?}",
        row_text(false, None)
    );
    // A locked file says who holds the lock.
    let locked = row_text(true, Some("bob"));
    assert!(locked.contains("LFS locked by bob"), "{locked:?}");
    assert!(!row_text(true, None).contains("locked"));
}

/// A row scrolled sideways shows its name from the offset, and the hover
/// button stays whole at the row's right end whatever the offset.
///
/// Falsified: pushing the button before `cut_spans` scrolls it out (red at
/// the button), and ignoring `h_off` leaves the head (red at the text).
#[test]
fn a_scrolled_row_keeps_its_hover_button_at_the_right_end() {
    use ratatui::backend::TestBackend;

    let line = StatusPaneLine::File {
        path: PathBuf::from("0123456789abcdefghij.rs"),
        depth: 0,
        section: StatusSection::Unstaged,
        label: "0123456789abcdefghij.rs".to_string(),
        status: Some(gitpoulpi_core::domain::FileStatus::Modified),
    };
    let list_area = Rect::new(0, 0, 30, 1);
    let text = |hovered: bool, h_off: usize| -> String {
        let mut hits = Vec::new();
        let item = status_row_item(
            &line,
            StatusSection::Unstaged,
            hovered,
            true,
            list_area,
            0,
            LfsMark::default(),
            false,
            h_off,
            &mut hits,
        );
        let mut terminal = Terminal::new(TestBackend::new(30, 1)).unwrap();
        terminal
            .draw(|frame| {
                frame.render_clipped(List::new(vec![item]), list_area);
            })
            .unwrap();
        let buf = terminal.backend().buffer().clone();
        (0..30).map(|x| buf[(x, 0)].symbol().to_string()).collect()
    };
    // The 2-cell marker is scrolled off by 4, with two cells of the name.
    assert!(
        text(false, 4).starts_with("23456789abcdefghij.rs"),
        "{}",
        text(false, 4)
    );
    let hovered = text(true, 4);
    assert!(hovered.starts_with("23456789abcdef"), "{hovered}");
    assert!(hovered.trim_end().ends_with("Stage file"), "{hovered}");
}
