use super::*;
use gitpoulpi_core::domain::FileHistoryPage;

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

fn entry(n: u8, path: &str, change: FileChangeKind) -> FileHistoryEntry {
    FileHistoryEntry {
        commit: oid(n),
        parents: vec![oid(n.wrapping_add(100))],
        author_name: "Ada Lovelace".into(),
        author_email: "ada@example.com".into(),
        timestamp: 1_700_000_000,
        committer_timestamp: 1_700_000_000,
        summary: format!("commit {n}"),
        path: PathBuf::from(path),
        change,
        added: Some(3),
        removed: Some(1),
    }
}

fn request() -> FileInsightRequest {
    FileInsightRequest {
        path: PathBuf::from("new.txt"),
        at: oid(9),
        kind: FileInsightKind::History,
        worktree: false,
        staged: false,
    }
}

/// A view that holds `entries`, as after its first page.
fn view(entries: Vec<FileHistoryEntry>, more: bool) -> HistoryView {
    let mut v = HistoryView::promote(&request(), 0);
    v.first_loaded = true;
    v.entries = entries;
    v.more = more;
    v
}

fn page_outcome(skip: u32, entries: Vec<FileHistoryEntry>, more: bool) -> FileInsightOutcome {
    FileInsightOutcome {
        request: FileInsightRequest {
            kind: FileInsightKind::HistoryPage(skip),
            ..request()
        },
        result: Ok(FileInsightPayload::HistoryPage(FileHistoryPage {
            entries,
            more,
        })),
    }
}

fn three() -> Vec<FileHistoryEntry> {
    vec![
        entry(3, "new.txt", FileChangeKind::Modified),
        entry(
            2,
            "new.txt",
            FileChangeKind::Renamed {
                from: PathBuf::from("old.txt"),
            },
        ),
        entry(1, "old.txt", FileChangeKind::Added),
    ]
}

#[test]
fn the_first_page_is_asked_for_once_and_the_answer_fills_the_list() {
    let mut v = HistoryView::promote(&request(), 0);
    let first = v.next_request(10).expect("the first page");
    assert_eq!(first.kind, FileInsightKind::HistoryPage(0));
    assert_eq!(first.at, oid(9));
    assert!(v.next_request(10).is_none(), "one page in flight at a time");
    assert!(v.loading());
    assert!(v.offer(page_outcome(0, three(), false)).is_none());
    assert_eq!(v.entries.len(), 3);
    assert!(!v.loading());
    assert!(v.next_request(10).is_none(), "nothing more to ask");
}

#[test]
fn scrolling_near_the_end_asks_for_the_next_page_from_where_it_stopped() {
    let many: Vec<_> = (0..PAGE as u8)
        .map(|n| entry(n, "new.txt", FileChangeKind::Modified))
        .collect();
    let mut v = view(many, true);
    // Far from the end: nothing.
    assert!(v.next_request(10).is_none());
    v.set_cursor(PAGE - 5, false);
    let next = v.next_request(10).expect("near the end");
    assert_eq!(next.kind, FileInsightKind::HistoryPage(PAGE as u32));
    // A stale page (another skip) is dropped, the right one is taken.
    assert!(v.offer(page_outcome(7, three(), false)).is_none());
    assert_eq!(v.entries.len(), PAGE);
    assert!(v.offer(page_outcome(PAGE as u32, three(), false)).is_none());
    assert_eq!(v.entries.len(), PAGE + 3);
    assert!(!v.more);
}

#[test]
fn rows_count_the_loading_row_while_more_exist() {
    let v = view(three(), true);
    assert_eq!(v.rows(), 4);
    assert_eq!(view(three(), false).rows(), 3);
}

#[test]
fn a_single_row_asks_for_its_commit_against_its_parent_and_a_rename_names_both_paths() {
    let mut v = view(three(), false);
    let spec = v.diff_spec().unwrap();
    assert_eq!(spec.target, DiffTarget::CommitVsParent(oid(3)));
    assert_eq!(
        (spec.path.as_path(), spec.also),
        (std::path::Path::new("new.txt"), None)
    );
    v.set_cursor(1, false);
    let spec = v.diff_spec().unwrap();
    assert_eq!(spec.target, DiffTarget::CommitVsParent(oid(2)));
    assert_eq!(spec.path, PathBuf::from("new.txt"));
    assert_eq!(spec.also, Some(PathBuf::from("old.txt")));
    // The commit before the rename: the old path.
    v.set_cursor(2, false);
    let spec = v.diff_spec().unwrap();
    assert_eq!(spec.path, PathBuf::from("old.txt"));
    assert_eq!(spec.also, None);
}

#[test]
fn a_range_compares_the_two_ends_with_the_path_each_had() {
    let mut v = view(three(), false);
    v.set_cursor(0, false);
    v.move_by(2, true);
    assert!(v.is_range());
    assert_eq!(v.range(), (0, 2));
    let spec = v.diff_spec().unwrap();
    assert_eq!(spec.target, DiffTarget::CommitRange(oid(1), oid(3)));
    assert_eq!(spec.path, PathBuf::from("new.txt"));
    assert_eq!(spec.also, Some(PathBuf::from("old.txt")));
    assert!(v.in_selection(1));
    // Moving without Shift drops the range.
    v.move_by(-1, false);
    assert!(!v.is_range());
}

#[test]
fn comparing_with_the_working_tree_is_dropped_by_moving() {
    let mut v = view(three(), false);
    assert!(v.toggle_worktree());
    assert_eq!(
        v.diff_spec().unwrap().target,
        DiffTarget::CommitVsWorkingTree(oid(3))
    );
    v.move_by(1, false);
    assert_eq!(
        v.diff_spec().unwrap().target,
        DiffTarget::CommitVsParent(oid(2))
    );
}

#[test]
fn the_diff_state_follows_the_selection_after_the_debounce_and_at_once_when_cached() {
    let mut v = view(three(), false);
    let mut state = Some(v.initial_state());
    let t0 = Instant::now();
    // The first selection is shown at once.
    assert!(v.sync_diff(&mut state, t0 + DEBOUNCE).is_none());
    assert_eq!(
        state.as_ref().unwrap().compare,
        Some(DiffTarget::CommitVsParent(oid(3)))
    );
    // A fresh move waits out the debounce...
    v.move_by(1, false);
    let now = Instant::now();
    let due = v.sync_diff(&mut state, now).expect("held back");
    assert!(due > now);
    assert_eq!(
        state.as_ref().unwrap().compare,
        Some(DiffTarget::CommitVsParent(oid(3))),
        "still the old diff while it settles"
    );
    // ... then applies, with the rename's second path.
    assert!(v.sync_diff(&mut state, due).is_none());
    let s = state.as_ref().unwrap();
    assert_eq!(s.compare, Some(DiffTarget::CommitVsParent(oid(2))));
    assert_eq!(s.also_path, Some(PathBuf::from("old.txt")));
    assert!(HistoryView::owns(s));
    // A row whose diff is cached skips the wait.
    let request =
        FileDiffRequest::file(DiffTarget::CommitVsParent(oid(3)), PathBuf::from("new.txt"));
    v.remember(&request, &[]);
    v.move_by(-1, false);
    assert!(v.sync_diff(&mut state, Instant::now()).is_none());
    assert_eq!(
        state.as_ref().unwrap().compare,
        Some(DiffTarget::CommitVsParent(oid(3)))
    );
}

#[test]
fn the_cache_hands_back_what_it_kept_and_never_keeps_the_working_tree() {
    let mut v = view(three(), false);
    let commit = FileDiffRequest::file(DiffTarget::CommitVsParent(oid(3)), PathBuf::from("a"));
    let live = FileDiffRequest::file(DiffTarget::CommitVsWorkingTree(oid(3)), PathBuf::from("a"));
    v.remember(&commit, &[]);
    v.remember(&live, &[]);
    assert!(v.cached(&commit).is_some());
    assert!(v.cached(&live).is_none());
    let other = FileDiffRequest::file(DiffTarget::CommitVsParent(oid(4)), PathBuf::from("a"));
    assert!(v.cached(&other).is_none());
}

#[test]
fn a_wanted_commit_is_selected_when_its_page_lands() {
    let mut v = HistoryView::promote(&request(), 0);
    v.want = Some(oid(1));
    let _ = v.next_request(10);
    assert!(v.offer(page_outcome(0, three(), false)).is_none());
    assert_eq!(v.cursor, 2);
    assert!(v.want.is_none());
}

#[test]
fn the_split_is_clamped_and_keeps_room_for_both_halves() {
    let mut v = view(three(), false);
    v.set_split(0);
    assert_eq!(v.split_pct, DEFAULT_SPLIT_PCT);
    v.set_split(5);
    assert_eq!(v.split_pct, SPLIT_PCT_RANGE.0);
    v.set_split(99);
    assert_eq!(v.split_pct, SPLIT_PCT_RANGE.1);
    let (list, diff) = v.split(Rect::new(0, 0, 100, 20));
    assert_eq!(list.width + diff.width, 100);
    assert_eq!(diff.x, list.right());
    let (list, diff) = v.split(Rect::new(0, 0, 20, 5));
    assert!(list.width >= MIN_COLS && list.width + diff.width == 20);
}

#[test]
fn the_wheel_moves_the_view_not_the_cursor_and_a_key_brings_it_back() {
    let many: Vec<_> = (0..50u8)
        .map(|n| entry(n, "new.txt", FileChangeKind::Modified))
        .collect();
    let mut v = view(many, false);
    v.scroll_to(20, 10);
    assert_eq!((v.offset, v.cursor), (20, 0));
    v.clamp(10);
    assert_eq!(v.offset, 20, "the wheel's offset is not dragged back");
    v.move_by(1, false);
    v.clamp(10);
    assert_eq!(v.offset, 1);
}

mod rows {
    use super::*;
    use crate::avatar::LogAvatarMode;
    use crate::history_view::render::{row_lines, RowLook};

    fn text(line: &ratatui::text::Line<'_>) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    fn paint(mode: LogAvatarMode) -> ListPaint {
        ListPaint {
            close_hot: false,
            fallback: gitpoulpi_config::AvatarFallback::Initials,
            mode,
            hovered_row: None,
            divider_hot: false,
            now: 1_700_000_000 + 3 * 86_400,
        }
    }

    const LOOK: RowLook = RowLook {
        cursor: false,
        in_range: false,
        hot: false,
    };

    fn lines(e: &FileHistoryEntry, width: usize, mode: LogAvatarMode) -> (String, String) {
        let [a, b] = row_lines(e, width, LOOK, &paint(mode));
        (text(&a), text(&b))
    }

    #[test]
    fn a_row_is_two_lines_summary_and_stats_over_author_date_and_sha() {
        let e = entry(1, "new.txt", FileChangeKind::Modified);
        let (one, two) = lines(&e, 60, LogAvatarMode::Glyph);
        assert!(
            one.starts_with("M ") && one.contains("commit 1") && one.contains("+3 -1"),
            "{one:?}"
        );
        assert_eq!(crate::ui_kit::display_width(&one), 60, "{one:?}");
        for part in ["Ada Lovelace", "ago", "0101010"] {
            assert!(two.contains(part), "{part:?} in {two:?}");
        }
        assert!(two.trim_start().starts_with('A'), "initials first: {two:?}");
    }

    #[test]
    fn a_narrow_row_still_shows_the_author_and_the_date() {
        let e = entry(1, "new.txt", FileChangeKind::Modified);
        for width in [34, 28, 24, 20] {
            let (one, two) = lines(&e, width, LogAvatarMode::Picture);
            assert!(one.contains("commit"), "{width}: {one:?}");
            assert!(two.contains("ago"), "the date at {width}: {two:?}");
            assert!(
                two.contains("Ada") || two.contains("A\u{2026}"),
                "the author at {width}: {two:?}"
            );
            assert!(
                crate::ui_kit::display_width(&two) <= width,
                "{width}: {two:?}"
            );
        }
    }

    #[test]
    fn a_rename_shows_old_and_new_on_the_second_line_when_there_is_room() {
        let e = entry(
            2,
            "new.txt",
            FileChangeKind::Renamed {
                from: PathBuf::from("old.txt"),
            },
        );
        let (one, two) = lines(&e, 90, LogAvatarMode::Off);
        assert!(one.starts_with("R "), "{one:?}");
        assert!(two.contains("old.txt \u{2192} new.txt"), "{two:?}");
        let (_, narrow) = lines(&e, 30, LogAvatarMode::Off);
        assert!(
            !narrow.contains("old.txt") && narrow.contains("ago"),
            "{narrow:?}"
        );
    }

    #[test]
    fn avatars_off_leaves_no_avatar_gutter() {
        let e = entry(1, "new.txt", FileChangeKind::Added);
        let (on1, on2) = lines(&e, 60, LogAvatarMode::Picture);
        let (off1, off2) = lines(&e, 60, LogAvatarMode::Off);
        assert!(off1.starts_with("A commit"), "{off1:?}");
        assert!(on1.starts_with("A  commit"), "{on1:?}");
        assert!(off2.starts_with("  Ada Lovelace"), "{off2:?}");
        assert!(on2.starts_with("AL Ada Lovelace"), "{on2:?}");
    }

    #[test]
    fn rows_are_two_cells_tall_in_every_conversion() {
        let mut v = view(three(), false);
        v.rows_area = Rect::new(5, 10, 40, 9);
        assert_eq!(v.viewport_rows(), 4, "an odd height shows four whole rows");
        v.offset = 1;
        assert_eq!(v.row_y(1), 10);
        assert_eq!(v.row_y(2), 12);
    }
}
