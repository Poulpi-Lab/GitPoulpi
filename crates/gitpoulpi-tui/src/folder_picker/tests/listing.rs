use super::*;

#[test]
fn it_lists_folders_marks_repositories_and_dims_files() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    let text = draw(&mut picker, 110, 30);
    assert!(text.contains("alpha"), "{text}");
    assert!(text.contains("git repository"), "{text}");
    assert!(text.contains("beta"), "{text}");
    assert!(text.contains("notes.txt"), "{text}");
    assert!(
        text.contains("QUICK ACCESS") || text.contains("FOLDERS"),
        "{text}"
    );
    // Files are drawn but never clickable.
    let files: Vec<_> = picker
        .hits()
        .iter()
        .filter(|(_, h)| matches!(h, PickerHit::Content(_)))
        .collect();
    assert_eq!(files.len(), 3, "the parent, alpha and beta only");
}

#[test]
fn before_the_listing_arrives_it_says_loading_and_a_stale_answer_is_ignored() {
    let tmp = tree();
    let mut picker = FolderPicker::new("t", "Open", tmp.path().to_path_buf());
    let asked = picker.wants();
    assert_eq!(asked, [tmp.path().to_path_buf()]);
    assert!(draw(&mut picker, 100, 30).contains("loading"));
    // An answer for another folder does not replace "loading".
    picker.deliver(&tmp.path().join("beta"), FolderListing::default());
    assert!(picker.listing().is_none());
    picker.deliver(tmp.path(), read_folder(tmp.path(), FOLDER_LIMIT));
    assert!(!draw(&mut picker, 100, 30).contains("loading"));
}

/// "Show hidden folders" is a pure render-time filter: flipping it needs no re-read, `Alt+H`
/// and the checkbox both
/// report the change so the owner can persist `repos.show_hidden`.
#[test]
fn show_hidden_reveals_dot_folders_dimmed_without_a_reread() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join(".hidden")).unwrap();
    std::fs::create_dir(tmp.path().join("Plain")).unwrap();
    let mut picker = picker_at(tmp.path());
    assert!(!picker.content_rows().iter().any(
        |r| matches!(r, ContentRow::Dir(i) if picker.listing().unwrap().dirs[*i].name == ".hidden")
    ));

    let outcome = picker.on_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::ALT));
    assert_eq!(outcome, PickerOutcome::ShowHiddenChanged(true));
    assert!(picker.show_hidden);
    // No new read was queued -- the entry was already there, just
    // filtered out.
    assert!(picker.wants().is_empty());
    assert!(picker.content_rows().iter().any(
        |r| matches!(r, ContentRow::Dir(i) if picker.listing().unwrap().dirs[*i].name == ".hidden")
    ));

    // The checkbox does the same thing on a click.
    draw(&mut picker, 110, 30);
    assert_eq!(
        click(&mut picker, PickerHit::ShowHidden, false),
        PickerOutcome::ShowHiddenChanged(false)
    );
    assert!(!picker.show_hidden);
}

/// Windows' `\\?\` verbatim prefix - which
/// `canonicalize` adds and a `RepoRef::root` or a path pasted from
/// another app can carry - must never reach the path field, the
/// "Selected:" line or the FOLDERS tree's root.
#[test]
fn navigating_to_a_verbatim_prefixed_path_strips_it_everywhere_shown() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    let verbatim = PathBuf::from(format!(r"\\?\{}", tmp.path().display()));
    picker.navigate(verbatim);
    assert_eq!(picker.dir, tmp.path());
    assert!(
        !picker.path_draft.contains(r"\\?\"),
        "path field: {}",
        picker.path_draft
    );
    settle(&mut picker);
    let text = draw(&mut picker, 110, 30);
    assert!(!text.contains(r"\\?\"), "{text}");

    // A picker started straight on a verbatim path (the Browse
    // button's start, or a repository's canonicalized root) is
    // stripped too, not only a later navigation.
    let verbatim_start = PathBuf::from(format!(r"\\?\{}", tmp.path().display()));
    let started = FolderPicker::new("t", "Open", verbatim_start);
    assert_eq!(started.dir, tmp.path());
    assert!(!started.path_draft.contains(r"\\?\"));
}
