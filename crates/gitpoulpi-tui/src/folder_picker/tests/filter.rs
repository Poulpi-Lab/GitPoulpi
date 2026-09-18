use super::*;

fn type_text(picker: &mut FolderPicker, text: &str) {
    for c in text.chars() {
        picker.on_key(key(KeyCode::Char(c)));
    }
}

fn names(picker: &FolderPicker) -> Vec<String> {
    let listing = picker.listing().expect("listed");
    picker
        .content_rows()
        .iter()
        .map(|row| match row {
            ContentRow::Parent(_) => "..".to_string(),
            ContentRow::Dir(i) => listing.dirs[*i].name.clone(),
            ContentRow::File(i) => listing.files[*i].name.clone(),
        })
        .collect()
}

#[test]
fn slash_opens_the_filter_and_typing_narrows_the_list_keeping_the_parent_row() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    assert_eq!(picker.zone, Zone::Content);
    picker.on_key(key(KeyCode::Char('/')));
    assert_eq!(picker.zone, Zone::Filter);
    type_text(&mut picker, "BET");
    assert_eq!(picker.filter, "BET", "the keys went into the field");
    assert_eq!(names(&picker), ["..", "beta"], "case-insensitive substring");
    let drawn = draw(&mut picker, 120, 30);
    assert!(drawn.contains("1 match of 3"), "{drawn}");
    assert!(!drawn.contains("notes.txt"));
}

#[test]
fn a_substring_not_a_prefix_matches() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.on_key(KeyEvent::new(KeyCode::Char('f'), KeyModifiers::CONTROL));
    assert_eq!(picker.zone, Zone::Filter);
    type_text(&mut picker, "otes");
    assert_eq!(names(&picker), ["..", "notes.txt"]);
}

#[test]
fn the_filter_clears_when_the_folder_changes() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.focus(Zone::Filter);
    type_text(&mut picker, "alp");
    picker.navigate(tmp.path().join("beta"));
    assert_eq!(picker.filter, "");
    settle(&mut picker);
    assert_eq!(names(&picker), ["..", "inner"]);
}

#[test]
fn slash_is_a_path_character_in_the_path_field() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.focus(Zone::Path);
    type_text(&mut picker, "a/b");
    assert_eq!(picker.zone, Zone::Path);
    assert_eq!(picker.path_draft, "a/b");
    assert_eq!(picker.filter, "");
}

#[test]
fn escape_clears_the_filter_then_leaves_it_then_cancels() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.focus(Zone::Filter);
    type_text(&mut picker, "al");
    assert_eq!(picker.on_key(key(KeyCode::Esc)), PickerOutcome::Stay);
    assert_eq!(picker.filter, "");
    assert_eq!(picker.zone, Zone::Filter);
    assert_eq!(picker.on_key(key(KeyCode::Esc)), PickerOutcome::Stay);
    assert_eq!(picker.zone, Zone::Content);
    assert_eq!(picker.on_key(key(KeyCode::Esc)), PickerOutcome::Cancel);
}

#[test]
fn enter_in_the_filter_selects_the_first_match() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.focus(Zone::Filter);
    type_text(&mut picker, "e");
    picker.on_key(key(KeyCode::Enter));
    assert_eq!(picker.zone, Zone::Content);
    assert_eq!(picker.chosen(), tmp.path().join("beta"));
}

#[test]
fn tab_reaches_the_filter_between_the_path_and_the_left_column() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.focus(Zone::Path);
    picker.on_key(key(KeyCode::Tab));
    assert_eq!(picker.zone, Zone::Filter);
    picker.on_key(key(KeyCode::Tab));
    assert_eq!(picker.zone, Zone::Left);
}

#[test]
fn a_click_on_the_filter_field_focuses_it() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    draw(&mut picker, 120, 30);
    click(&mut picker, PickerHit::FilterField, false);
    assert_eq!(picker.zone, Zone::Filter);
}

#[test]
fn a_filter_matching_nothing_says_so() {
    let tmp = tree();
    let mut picker = picker_at(tmp.path());
    picker.focus(Zone::Filter);
    type_text(&mut picker, "zzz");
    let drawn = draw(&mut picker, 120, 30);
    assert!(drawn.contains("no entry matches the filter"), "{drawn}");
    assert!(drawn.contains("0 matches of 3"), "{drawn}");
}
