use super::*;

/// root/ { a/, b/ { b1/, b2/ { deep/ } }, c/, .hid/ }
fn nested() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    for dir in ["a", "b/b1", "b/b2/deep", "c", ".hid"] {
        std::fs::create_dir_all(tmp.path().join(dir)).unwrap();
    }
    tmp
}

/// `(name, depth, open)` of the tree rows below `root`, root excluded.
fn below(picker: &FolderPicker, root: &Path) -> Vec<(String, usize, bool)> {
    let rows = picker.left_rows();
    let start = rows
        .iter()
        .position(|r| matches!(r, LeftRow::Tree { path, .. } if path == root))
        .expect("the root is on the open path");
    let LeftRow::Tree {
        depth: root_depth, ..
    } = &rows[start]
    else {
        unreachable!()
    };
    rows[start + 1..]
        .iter()
        .take_while(|r| matches!(r, LeftRow::Tree { depth, .. } if depth > root_depth))
        .map(|r| match r {
            LeftRow::Tree {
                name, depth, open, ..
            } => (name.clone(), *depth, *open),
            other => panic!("{other:?}"),
        })
        .collect()
}

fn open_at_b2() -> (tempfile::TempDir, FolderPicker) {
    let tmp = nested();
    let picker = picker_at(&tmp.path().join("b").join("b2"));
    (tmp, picker)
}

fn alt_s() -> KeyEvent {
    KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT)
}

#[test]
fn unchecked_the_tree_is_the_chain_and_the_current_folders_children() {
    let (tmp, picker) = open_at_b2();
    assert!(!picker.show_siblings);
    let d = tmp.path().components().count();
    assert_eq!(
        below(&picker, tmp.path()),
        [
            ("b".to_string(), d, true),
            ("b2".to_string(), d + 1, true),
            ("deep".to_string(), d + 2, false),
        ]
    );
}

#[test]
fn checked_every_open_level_lists_its_folders_with_the_path_expanded() {
    let (tmp, mut picker) = open_at_b2();
    picker.set_show_siblings(true);
    settle(&mut picker);
    let d = tmp.path().components().count();
    assert_eq!(
        below(&picker, tmp.path()),
        [
            ("a".to_string(), d, false),
            ("b".to_string(), d, true),
            ("b1".to_string(), d + 1, false),
            ("b2".to_string(), d + 1, true),
            ("deep".to_string(), d + 2, false),
            ("c".to_string(), d, false),
        ]
    );
    // Hidden folders follow the picker's rule.
    picker.show_hidden = true;
    assert!(below(&picker, tmp.path())
        .iter()
        .any(|(name, _, _)| name == ".hid"));
}

#[test]
fn the_ancestors_are_read_lazily_and_only_when_checked() {
    let (tmp, mut picker) = open_at_b2();
    assert!(picker.wants().is_empty(), "unchecked reads nothing extra");
    assert_eq!(
        picker.on_key(alt_s()),
        PickerOutcome::ShowSiblingsChanged(true)
    );
    let asked = picker.wants();
    assert!(asked.contains(&tmp.path().to_path_buf()));
    assert!(asked.contains(&tmp.path().join("b")));
    // Until the answers land, the chain alone is shown.
    assert!(!below(&picker, tmp.path()).iter().any(|(n, _, _)| n == "a"));
    for dir in asked {
        picker.deliver(&dir, read_folder(&dir, FOLDER_LIMIT));
    }
    assert!(below(&picker, tmp.path()).iter().any(|(n, _, _)| n == "a"));
    assert_eq!(
        picker.on_key(alt_s()),
        PickerOutcome::ShowSiblingsChanged(false)
    );
    assert!(!below(&picker, tmp.path()).iter().any(|(n, _, _)| n == "a"));
}

#[test]
fn the_checkbox_and_a_sibling_row_work_by_mouse_and_draw_the_fold_marks() {
    let (tmp, mut picker) = open_at_b2();
    draw(&mut picker, 120, 30);
    assert_eq!(
        click(&mut picker, PickerHit::ShowSiblings, false),
        PickerOutcome::ShowSiblingsChanged(true)
    );
    settle(&mut picker);
    let text = draw(&mut picker, 120, 30);
    assert!(text.contains("Show sibling folders"), "{text}");
    assert!(text.contains(&format!("{} a", crate::fold_mark::COLLAPSED)));
    assert!(text.contains(&format!("{} b", crate::fold_mark::EXPANDED)));
    let a = picker
        .left_rows()
        .iter()
        .position(|r| matches!(r, LeftRow::Tree { name, .. } if name == "a"))
        .unwrap();
    click(&mut picker, PickerHit::Left(a), false);
    assert_eq!(picker.dir, tmp.path().join("a"));
}
