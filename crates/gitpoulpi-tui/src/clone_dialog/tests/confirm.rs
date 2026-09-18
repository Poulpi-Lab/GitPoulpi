//! The destination-exists popup, the off-frame destination probe, and the
//! sources' keyboard path.

use super::*;

fn dialog_for(tmp: &Path) -> CloneDialog {
    let mut dialog = CloneDialog::new(Some(tmp));
    typed(&mut dialog, "https://example.com/team/app.git");
    dialog
}

/// Presses Clone and lets the probe answer: the popup is up (or not).
fn press_clone(dialog: &mut CloneDialog) -> CloneOutcome {
    dialog.focus = CloneFocus::Clone;
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), CloneOutcome::Stay);
    settle(dialog)
}

#[test]
fn the_first_free_suffix_skips_the_taken_ones() {
    let tmp = tempfile::tempdir().unwrap();
    assert_eq!(dest::first_free_name(tmp.path(), "app"), "app-2");
    std::fs::create_dir(tmp.path().join("app-2")).unwrap();
    std::fs::write(tmp.path().join("app-3"), "a file counts too").unwrap();
    assert_eq!(dest::first_free_name(tmp.path(), "app"), "app-4");
}

#[test]
fn the_probe_tells_missing_empty_same_clone_and_other() {
    let tmp = tempfile::tempdir().unwrap();
    let app = tmp.path().join("app");
    let url = "https://example.com/team/app.git";
    assert_eq!(dest::inspect(&app, url), dest::DestKind::Missing);
    std::fs::create_dir(&app).unwrap();
    assert_eq!(dest::inspect(&app, url), dest::DestKind::Empty);
    std::fs::write(app.join("f.txt"), "x").unwrap();
    assert_eq!(
        dest::inspect(&app, url),
        dest::DestKind::NotEmpty {
            suggestion: "app-2".into()
        }
    );
    std::fs::create_dir(app.join(".git")).unwrap();
    std::fs::write(
        app.join(".git").join("config"),
        "[remote \"origin\"]\n\turl = https://EXAMPLE.com/team/app/\n",
    )
    .unwrap();
    assert_eq!(dest::inspect(&app, url), dest::DestKind::SameRepo);
    // Another remote is just a folder with files.
    assert!(matches!(
        dest::inspect(&app, "https://example.com/team/other.git"),
        dest::DestKind::NotEmpty { .. }
    ));
}

#[test]
fn painting_does_not_read_the_disk_it_says_it_is_checking() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("app")).unwrap();
    std::fs::write(tmp.path().join("app").join("f.txt"), "x").unwrap();
    let mut dialog = dialog_for(tmp.path());
    // No poll yet: no answer, so no warning from the disk, only "checking".
    let (text, _) = paint(&mut dialog, 100, 30);
    assert!(text.contains("checking the folder"), "{text}");
    assert!(!text.contains("not empty"), "{text}");
    assert_eq!(settle(&mut dialog), CloneOutcome::Stay);
    let (text, _) = paint(&mut dialog, 100, 30);
    assert!(text.contains("not empty"), "{text}");
    assert!(!text.contains("checking the folder"), "{text}");
}

#[test]
fn a_missing_destination_clones_without_a_popup() {
    let tmp = tempfile::tempdir().unwrap();
    let mut dialog = dialog_for(tmp.path());
    assert!(matches!(
        press_clone(&mut dialog),
        CloneOutcome::Submit { .. }
    ));
    assert!(dialog.confirm.is_none());
}

#[test]
fn an_empty_folder_asks_then_clones_into_it() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("app")).unwrap();
    let mut dialog = dialog_for(tmp.path());
    assert_eq!(press_clone(&mut dialog), CloneOutcome::Stay);
    let (text, _) = paint(&mut dialog, 120, 30);
    assert!(text.contains("The folder exists and is empty"), "{text}");
    assert!(
        text.contains("Clone into it") && text.contains("Back"),
        "{text}"
    );
    assert_eq!(
        dialog.on_key(key(KeyCode::Enter)),
        CloneOutcome::Submit {
            url: "https://example.com/team/app.git".into(),
            dest: tmp.path().join("app"),
            opts: CloneOptions::default(),
        }
    );
}

#[test]
fn a_folder_that_is_already_this_clone_opens_instead() {
    let tmp = tempfile::tempdir().unwrap();
    let app = tmp.path().join("app");
    std::fs::create_dir_all(app.join(".git")).unwrap();
    std::fs::write(
        app.join(".git").join("config"),
        "[remote \"origin\"]\n\turl = https://example.com/team/app.git\n",
    )
    .unwrap();
    let mut dialog = dialog_for(tmp.path());
    assert_eq!(press_clone(&mut dialog), CloneOutcome::Stay);
    let (text, _) = paint(&mut dialog, 120, 30);
    assert!(
        text.contains("Already cloned here") && text.contains("Open it"),
        "{text}"
    );
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), CloneOutcome::Open(app));
}

#[test]
fn a_folder_with_other_files_offers_the_next_free_name_and_clones_there() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("app")).unwrap();
    std::fs::write(tmp.path().join("app").join("f.txt"), "x").unwrap();
    std::fs::create_dir(tmp.path().join("app-2")).unwrap();
    std::fs::write(tmp.path().join("app-2").join("f.txt"), "x").unwrap();
    let mut dialog = dialog_for(tmp.path());
    assert_eq!(press_clone(&mut dialog), CloneOutcome::Stay);
    let (text, _) = paint(&mut dialog, 120, 30);
    assert!(text.contains("The folder is not empty"), "{text}");
    assert!(text.contains("Use \"app-3\""), "{text}");
    assert_eq!(
        dialog.on_key(key(KeyCode::Enter)),
        CloneOutcome::Submit {
            url: "https://example.com/team/app.git".into(),
            dest: tmp.path().join("app-3"),
            opts: CloneOptions::default(),
        }
    );
    assert_eq!(dialog.name, "app-3", "the Name field shows what was used");
}

#[test]
fn back_and_escape_close_the_popup_and_clone_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("app")).unwrap();
    let mut dialog = dialog_for(tmp.path());
    press_clone(&mut dialog);
    assert_eq!(dialog.on_key(key(KeyCode::Esc)), CloneOutcome::Stay);
    assert!(dialog.confirm.is_none(), "Esc is Back, not Cancel");
    press_clone(&mut dialog);
    // Tab moves to Back, Enter presses it.
    dialog.on_key(key(KeyCode::Tab));
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), CloneOutcome::Stay);
    assert!(dialog.confirm.is_none());
}

#[test]
fn the_popup_buttons_click_and_the_popup_owns_the_pointer() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join("app")).unwrap();
    std::fs::write(tmp.path().join("app").join("f.txt"), "x").unwrap();
    let mut dialog = dialog_for(tmp.path());
    press_clone(&mut dialog);
    paint(&mut dialog, 120, 30);
    assert!(
        dialog
            .hits
            .iter()
            .all(|(_, h)| matches!(h, CloneHit::Confirm(_))),
        "nothing behind the popup is live: {:?}",
        dialog.hits
    );
    // A click outside the popup is Back (a click on the form's own Cancel
    // button, behind the popup, does nothing else).
    assert_eq!(dialog.on_mouse(click(1, 1)), CloneOutcome::Stay);
    assert!(dialog.confirm.is_none(), "outside the popup = Back");
    // So is its cross.
    press_clone(&mut dialog);
    paint(&mut dialog, 120, 30);
    let cross = dialog.confirm_close.rect().expect("the popup has a cross");
    assert_eq!(
        dialog.on_mouse(click(cross.x + 1, cross.y)),
        CloneOutcome::Stay
    );
    assert!(dialog.confirm.is_none(), "the cross = Back");
    press_clone(&mut dialog);
    paint(&mut dialog, 120, 30);
    let back = dialog.hit_rect(CloneHit::Confirm(ConfirmButton::Back));
    let go = dialog.hit_rect(CloneHit::Confirm(ConfirmButton::Go));
    // Hover lands on the popup's button.
    dialog.on_mouse(MouseEvent {
        kind: MouseEventKind::Moved,
        column: go.x + 1,
        row: go.y,
        modifiers: KeyModifiers::NONE,
    });
    assert_eq!(dialog.hover, Some(CloneHit::Confirm(ConfirmButton::Go)));
    // Back closes it.
    assert_eq!(
        dialog.on_mouse(click(back.x + 1, back.y)),
        CloneOutcome::Stay
    );
    assert!(dialog.confirm.is_none());
    // Reopen, click the main button: the next free name is cloned.
    press_clone(&mut dialog);
    paint(&mut dialog, 120, 30);
    let go = dialog.hit_rect(CloneHit::Confirm(ConfirmButton::Go));
    assert_eq!(
        dialog.on_mouse(click(go.x + 1, go.y)),
        CloneOutcome::Submit {
            url: "https://example.com/team/app.git".into(),
            dest: tmp.path().join("app-2"),
            opts: CloneOptions::default(),
        }
    );
}

#[test]
fn clone_pressed_before_the_probe_answers_carries_on_when_it_does() {
    let tmp = tempfile::tempdir().unwrap();
    let mut dialog = dialog_for(tmp.path());
    dialog.focus = CloneFocus::Clone;
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), CloneOutcome::Stay);
    assert!(dialog.submit_after_probe);
    assert!(matches!(settle(&mut dialog), CloneOutcome::Submit { .. }));
}

// -- B2: the sources ---------------------------------------------------

#[cfg(feature = "hosting")]
#[test]
fn the_sources_are_the_first_tab_stop_and_alt_s_reaches_them() {
    let mut dialog = with_hosted();
    assert_eq!(dialog.order()[0], CloneFocus::Sources);
    dialog.focus = CloneFocus::Where;
    dialog.on_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
    assert_eq!(
        dialog.focus,
        CloneFocus::Sources,
        "Shift+Tab from the first field"
    );
    dialog.focus = CloneFocus::Shallow;
    dialog.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT));
    assert_eq!(dialog.focus, CloneFocus::Sources, "Alt+S from anywhere");
    // The footer names the key.
    let (text, _) = paint(&mut dialog, 140, 34);
    assert!(text.contains("Alt+S") && text.contains("sources"), "{text}");
}

#[cfg(feature = "hosting")]
#[test]
fn alt_s_works_from_a_text_field_too() {
    let mut dialog = with_hosted();
    dialog.focus = CloneFocus::Url;
    dialog.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT));
    assert_eq!(dialog.focus, CloneFocus::Sources);
    assert_eq!(dialog.url, "", "the key typed nothing");
}

#[cfg(feature = "hosting")]
#[test]
fn walking_three_rows_asks_for_one_list_only_once_the_highlight_rests() {
    let mut dialog = with_hosted();
    dialog.focus = CloneFocus::Sources;
    dialog.on_key(key(KeyCode::Down));
    dialog.on_key(key(KeyCode::Down));
    dialog.on_key(key(KeyCode::Up));
    dialog.on_key(key(KeyCode::Down));
    assert_eq!(dialog.source, 2);
    assert!(dialog.next_deadline().is_some(), "the loop wakes for it");
    assert_eq!(dialog.poll(), CloneOutcome::Stay, "not yet");
    rest(&mut dialog);
    assert_eq!(dialog.poll(), CloneOutcome::LoadProjects("hub".into()));
    assert!(!dialog.projects.contains_key("corp"), "the row passed over");
    assert!(dialog.next_deadline().is_none());
}

#[cfg(feature = "hosting")]
#[test]
fn enter_right_and_tab_leave_the_sources_and_ask_at_once() {
    for leave in [KeyCode::Enter, KeyCode::Right, KeyCode::Tab] {
        let mut dialog = with_hosted();
        dialog.focus = CloneFocus::Sources;
        dialog.on_key(key(KeyCode::Down));
        assert_eq!(
            dialog.on_key(key(leave)),
            CloneOutcome::LoadProjects("corp".into()),
            "{leave:?}"
        );
        assert_eq!(dialog.focus, CloneFocus::Search, "{leave:?}");
        assert!(dialog.next_deadline().is_none());
    }
    // On "Clone with URL" they go to the URL field.
    let mut dialog = with_hosted();
    dialog.focus = CloneFocus::Sources;
    assert_eq!(dialog.on_key(key(KeyCode::Enter)), CloneOutcome::Stay);
    assert_eq!(dialog.focus, CloneFocus::Url);
}

#[cfg(feature = "hosting")]
#[test]
fn home_and_end_jump_between_the_first_and_last_source() {
    let mut dialog = with_hosted();
    dialog.focus = CloneFocus::Sources;
    dialog.on_key(key(KeyCode::End));
    assert_eq!(dialog.source, 2);
    dialog.on_key(key(KeyCode::Home));
    assert_eq!(dialog.source, 0);
}

#[cfg(feature = "hosting")]
#[test]
fn without_a_profile_the_connect_note_is_a_link_to_integrations() {
    let mut dialog = CloneDialog::new(Some(&abs("clones")));
    assert_eq!(dialog.order()[0], CloneFocus::Connect);
    dialog.focus = CloneFocus::Where;
    dialog.on_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
    assert_eq!(dialog.focus, CloneFocus::Connect);
    assert_eq!(
        dialog.on_key(key(KeyCode::Enter)),
        CloneOutcome::OpenIntegrations
    );
    // And a click on the note does the same.
    paint(&mut dialog, 120, 30);
    let note = dialog.hit_rect(CloneHit::Part(CloneFocus::Connect));
    dialog.focus = CloneFocus::Url;
    assert_eq!(
        dialog.on_mouse(click(note.x + 1, note.y)),
        CloneOutcome::OpenIntegrations
    );
    dialog.focus = CloneFocus::Shallow;
    dialog.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT));
    assert_eq!(dialog.focus, CloneFocus::Connect, "Alt+S reaches the note");
}
