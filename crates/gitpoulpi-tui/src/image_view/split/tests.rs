use super::*;
use gitpoulpi_core::domain::FileDiff;

fn state(source: DiffSource) -> DiffViewState {
    match source {
        DiffSource::Commit => DiffViewState::for_commit(Oid([7; 20]), 0, "pic.png".into()),
        _ => DiffViewState::for_working(source, "pic.png".into()),
    }
}

fn file(path: &str, status: FileStatus) -> FileDiff {
    FileDiff {
        renamed_from: None,
        similarity: None,
        path: path.into(),
        status,
        hunks: Vec::new(),
    }
}

#[test]
fn a_binary_with_an_image_extension_is_shown_as_pictures_and_other_files_are_not() {
    let st = state(DiffSource::Commit);
    assert!(wants(&st, &file("pic.png", FileStatus::Modified)));
    assert!(wants(&st, &file("DIR/Photo.JPG", FileStatus::Added)));
    assert!(!wants(&st, &file("data.bin", FileStatus::Modified)));
    assert!(!wants(&st, &file("notes.txt", FileStatus::Modified)));
}

#[test]
fn the_raw_lfs_pointer_view_keeps_the_text_diff() {
    let mut st = state(DiffSource::Commit);
    st.lfs_raw = true;
    assert!(!wants(&st, &file("pic.png", FileStatus::Modified)));
}

#[test]
fn each_side_reads_from_the_place_the_plan_names() {
    use DiffContentSide as D;
    assert_eq!(side_kind(DiffSource::Commit, Side::Before), D::Parent);
    assert_eq!(side_kind(DiffSource::Commit, Side::After), D::Commit);
    assert_eq!(side_kind(DiffSource::Staged, Side::Before), D::Head);
    assert_eq!(side_kind(DiffSource::Staged, Side::After), D::Index);
    assert_eq!(side_kind(DiffSource::Unstaged, Side::Before), D::Index);
    assert_eq!(side_kind(DiffSource::Unstaged, Side::After), D::WorkingTree);
}

#[test]
fn an_added_file_has_no_before_and_a_deleted_one_no_after() {
    assert!(!side_exists(FileStatus::Added, Side::Before));
    assert!(side_exists(FileStatus::Added, Side::After));
    assert!(side_exists(FileStatus::Deleted, Side::Before));
    assert!(!side_exists(FileStatus::Deleted, Side::After));
    assert!(side_exists(FileStatus::Modified, Side::Before));
    assert!(side_exists(FileStatus::Modified, Side::After));
}

#[test]
fn following_a_diff_asks_for_each_side_once_and_a_new_file_asks_again() {
    let mut s = State::new();
    let st = state(DiffSource::Commit);
    s.follow(&st, &file("pic.png", FileStatus::Modified));
    assert_eq!(s.outbox.len(), 2);
    assert_eq!(
        s.outbox[0].kind,
        FileInsightKind::ImageBytes(DiffContentSide::Parent)
    );
    s.outbox.clear();
    s.follow(&st, &file("pic.png", FileStatus::Modified));
    assert!(s.outbox.is_empty(), "the same file is not asked again");
    s.follow(&st, &file("pic.png", FileStatus::Added));
    assert_eq!(s.outbox.len(), 1, "an added file has the after side only");
    assert!(!s.slots[0].present && s.slots[1].present);
}

#[test]
fn a_viewport_survives_the_round_trip_to_another_pictures_units() {
    let after = Size::new(640.0, 480.0);
    let before = Size::new(320.0, 240.0);
    let vp = Viewport {
        zoom: Zoom::Scale(2.0),
        center: (0.25, 0.75),
    };
    // Half the size, so twice the scale to cover the same part of the pane.
    let local = localise(vp, before, after);
    assert_eq!(local.zoom, Zoom::Scale(4.0));
    assert_eq!(local.center, (0.25, 0.75));
    assert_eq!(globalise(local, before, after), vp);
    // Fit stays Fit whatever the sizes.
    assert!(localise(Viewport::fit(), before, after).is_fit());
}

#[test]
fn the_captions_name_where_each_side_comes_from() {
    let key = |source| Key {
        source,
        commit: Oid([0xab; 20]),
        path: "pic.png".into(),
        status: FileStatus::Modified,
    };
    assert_eq!(where_from(&key(DiffSource::Commit), Side::After), "abababa");
    assert_eq!(
        where_from(&key(DiffSource::Commit), Side::Before),
        msg::WHERE_PARENT.get()
    );
    assert_eq!(
        where_from(&key(DiffSource::Staged), Side::Before),
        msg::WHERE_HEAD.get()
    );
    assert_eq!(
        where_from(&key(DiffSource::Unstaged), Side::After),
        msg::WHERE_WORKTREE.get()
    );
}

#[test]
fn a_no_image_answer_for_every_side_falls_back_to_the_text_diff() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut s = State::new();
    let st = state(DiffSource::Commit);
    let f = file("pic.png", FileStatus::Modified);
    s.follow(&st, &f);
    for slot in &mut s.slots {
        slot.not_image = true;
        slot.phase = Phase::Card(Vec::new());
    }
    let mut term = Terminal::new(TestBackend::new(80, 20)).unwrap();
    let mut drew = None;
    term.draw(|frame| {
        drew = s.render(frame, Rect::new(1, 1, 78, 18), &st, &f);
    })
    .unwrap();
    assert!(drew.is_none());
    assert!(s.fallback);
}
