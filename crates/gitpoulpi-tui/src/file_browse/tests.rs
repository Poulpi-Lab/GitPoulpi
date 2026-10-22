use super::*;
use crossterm::event::{KeyCode, KeyModifiers};

/// root/ { docs/ { R.md }, NOTES.md }
fn tree() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("docs")).unwrap();
    std::fs::write(tmp.path().join("docs").join("R.md"), "rules").unwrap();
    std::fs::write(tmp.path().join("NOTES.md"), "notes").unwrap();
    tmp
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Pumps the real lister (threads) until the folder on screen is listed.
fn listed(browse: &mut FileBrowse, name: &str) {
    for _ in 0..200 {
        browse.pump();
        let mut buf = Buffer::empty(Rect::new(0, 0, 110, 30));
        browse.render(buf.area, &mut buf);
        if crate::dialog_kit::buffer_text(&buf).contains(name) {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("{name} never listed");
}

#[test]
fn a_file_inside_the_repository_is_stored_relative_with_slashes() {
    let tmp = tree();
    let root = tmp.path();
    assert_eq!(stored_path(&root.join("NOTES.md"), Some(root)), "NOTES.md");
    assert_eq!(
        stored_path(&root.join("docs").join("R.md"), Some(root)),
        "docs/R.md"
    );
}

#[test]
fn a_file_outside_the_repository_or_without_one_is_stored_absolute() {
    let tmp = tree();
    let other = tree();
    let outside = other.path().join("NOTES.md");
    assert_eq!(
        stored_path(&outside, Some(tmp.path())),
        outside.display().to_string()
    );
    assert_eq!(
        stored_path(&outside, None),
        outside.display().to_string(),
        "no clone: always absolute"
    );
}

#[test]
fn the_picker_starts_at_the_root_or_beside_the_current_file() {
    let tmp = tree();
    let root = tmp.path();
    assert_eq!(start_dir(Some(root), ""), root);
    assert_eq!(start_dir(Some(root), "docs/R.md"), root.join("docs"));
    assert_eq!(
        start_dir(Some(root), "missing/x.md"),
        root,
        "no such folder"
    );
    assert_eq!(
        start_dir(Some(root), "https://example.org/a.md"),
        root,
        "a link is not a place"
    );
    let abs = root.join("docs").join("R.md");
    assert_eq!(
        start_dir(None, &abs.display().to_string()),
        root.join("docs")
    );
}

#[test]
fn choosing_a_listed_file_answers_with_the_stored_path() {
    let tmp = tree();
    let mut browse = FileBrowse::open("Extra file", "Use", Some(tmp.path()), "");
    listed(&mut browse, "NOTES.md");
    // Files come last in the folder's content.
    assert_eq!(browse.on_key(key(KeyCode::End)), BrowseOutcome::Stay);
    assert_eq!(
        browse.on_key(key(KeyCode::Enter)),
        BrowseOutcome::Chosen("NOTES.md".into())
    );
}

#[test]
fn a_file_picker_never_chooses_the_folder_shown() {
    let tmp = tree();
    let mut browse = FileBrowse::open("Extra file", "Use", Some(tmp.path()), "");
    listed(&mut browse, "NOTES.md");
    browse.picker().content_sel = None;
    assert_eq!(browse.on_key(key(KeyCode::Enter)), BrowseOutcome::Stay);
    let ctrl_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    assert_eq!(browse.on_key(ctrl_s), BrowseOutcome::Stay);
    assert_eq!(browse.on_key(key(KeyCode::Esc)), BrowseOutcome::Cancel);
}

#[test]
fn it_starts_beside_the_current_file_and_stores_a_nested_path() {
    let tmp = tree();
    let mut browse = FileBrowse::open("Extra file", "Use", Some(tmp.path()), "docs/R.md");
    listed(&mut browse, "R.md");
    browse.on_key(key(KeyCode::End));
    assert_eq!(
        browse.on_key(key(KeyCode::Enter)),
        BrowseOutcome::Chosen("docs/R.md".into())
    );
}

#[test]
fn a_file_outside_the_clone_comes_back_absolute() {
    let tmp = tree();
    let other = tree();
    let outside = other.path().join("NOTES.md");
    let mut browse = FileBrowse::open(
        "Extra file",
        "Use",
        Some(tmp.path()),
        &outside.display().to_string(),
    );
    listed(&mut browse, "NOTES.md");
    browse.on_key(key(KeyCode::End));
    assert_eq!(
        browse.on_key(key(KeyCode::Enter)),
        BrowseOutcome::Chosen(outside.display().to_string())
    );
}
