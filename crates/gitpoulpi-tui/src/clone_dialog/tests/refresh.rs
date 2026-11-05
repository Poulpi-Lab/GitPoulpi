//! A saved repository list, refreshed in the background, the view kept in
//! place.

use super::*;

fn team(range: std::ops::Range<u32>) -> Vec<HostedProject> {
    range.map(|i| project(&format!("team/r{i:02}"))).collect()
}

fn review_dialog(project: &str) -> CloneDialog {
    CloneDialog::new(Some(&abs("clones")))
        .with_hosted(hosted())
        .with_url("https://gitlab.example.com/acme/app.git")
        .with_target("corp", project, "note".into())
}

/// The identity of the first painted row (the header counts as a row).
fn top_row(dialog: &CloneDialog) -> String {
    let rows = crate::clone_dialog::anchor::row_keys(&dialog.grouped());
    format!("{:?}", rows[dialog.repo_offset])
}

fn cursor_name(dialog: &CloneDialog) -> String {
    dialog.filtered()[dialog.repo_cursor].full_name.clone()
}

/// A source on the saved copy r05..r29, scrolled so the first row is r09,
/// with r15 chosen.
fn cached_dialog() -> CloneDialog {
    let mut dialog = with_hosted();
    dialog.source = 1;
    dialog.set_cached_projects("corp", team(5..30));
    paint(&mut dialog, 130, 34);
    let at = dialog
        .filtered()
        .iter()
        .position(|p| p.full_name == "team/r15")
        .unwrap();
    dialog.choose_repo(at);
    dialog.repo_offset = 10;
    paint(&mut dialog, 130, 34);
    dialog
}

#[test]
fn a_fresh_list_keeps_the_choice_the_cursor_and_the_viewport_while_rows_come_and_go() {
    let mut dialog = cached_dialog();
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(text.contains("refreshing..."), "{text}");
    let (top, cursor) = (top_row(&dialog), cursor_name(&dialog));
    assert_eq!(cursor, "team/r15");
    // Five repositories appear above, one inside the view goes away.
    let mut fresh = team(0..30);
    fresh.retain(|p| p.full_name != "team/r12");
    dialog.set_projects_answer("corp", Ok(fresh), false);
    assert_eq!(dialog.chosen.as_deref(), Some("team/r15"));
    assert_eq!(cursor_name(&dialog), "team/r15");
    assert_eq!(top_row(&dialog), top, "the viewport stays on the same rows");
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(!text.contains("refreshing..."), "{text}");
    assert_eq!(top_row(&dialog), top, "still the same after a paint");
}

#[test]
fn a_chosen_project_that_left_the_host_is_cleared_with_a_note() {
    let mut dialog = cached_dialog();
    assert!(dialog.hosted_url.is_some());
    let mut fresh = team(5..30);
    fresh.retain(|p| p.full_name != "team/r15");
    dialog.set_projects_answer("corp", Ok(fresh), false);
    assert_eq!(dialog.chosen, None);
    assert_eq!(dialog.hosted_url, None);
    assert_eq!(dialog.name, "");
    assert_eq!(
        dialog.message.as_deref(),
        Some("team/r15 is no longer on the host")
    );
    assert!(dialog.dest().is_none());
}

#[test]
fn a_chosen_projects_url_follows_the_fresh_row() {
    let mut dialog = cached_dialog();
    let mut fresh = team(5..30);
    for p in &mut fresh {
        if p.full_name == "team/r15" {
            p.https_url = Some("https://gitlab.example.com/moved/r15.git".into());
        }
    }
    dialog.set_projects_answer("corp", Ok(fresh), false);
    assert_eq!(
        dialog.hosted_url.as_deref(),
        Some("https://gitlab.example.com/moved/r15.git")
    );
}

#[test]
fn a_failed_refresh_keeps_the_saved_list_but_a_rejected_token_does_not() {
    let mut dialog = cached_dialog();
    dialog.set_projects_answer("corp", Err("timeout".into()), false);
    assert!(matches!(
        dialog.projects.get("corp"),
        Some(ProjectsState::Ready(_))
    ));
    assert_eq!(dialog.chosen.as_deref(), Some("team/r15"));
    assert_eq!(dialog.message, None, "the list stays usable, no error line");
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(text.contains("saved list (refresh failed"), "{text}");
    assert!(!text.contains("refreshing..."), "{text}");

    let mut dialog = cached_dialog();
    dialog.set_projects_answer("corp", Err("401".into()), true);
    assert!(matches!(
        dialog.projects.get("corp"),
        Some(ProjectsState::Failed(_))
    ));
    let (text, _) = paint(&mut dialog, 130, 34);
    assert!(text.contains("could not list repositories: 401"), "{text}");
}

#[test]
fn a_fresh_list_waits_for_a_held_scrollbar_thumb() {
    let mut dialog = cached_dialog();
    let track = dialog.list_bar.expect("the list overflows");
    let down = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: track.rect.x,
        row: track.rect.y + 3,
        modifiers: KeyModifiers::NONE,
    };
    dialog.on_mouse(down);
    assert!(dialog.bar_drag.held());
    dialog.set_projects_answer("corp", Ok(team(0..30)), false);
    assert_eq!(dialog.filtered().len(), 25, "the swap is held back");
    assert_eq!(dialog.poll(), CloneOutcome::Stay);
    assert_eq!(dialog.filtered().len(), 25, "still held");
    dialog.on_mouse(MouseEvent {
        kind: MouseEventKind::Up(MouseButton::Left),
        ..down
    });
    dialog.poll();
    assert_eq!(dialog.filtered().len(), 30, "applied after the release");
    assert_eq!(cursor_name(&dialog), "team/r15");
}

#[test]
fn a_review_target_missing_from_the_saved_copy_is_selected_from_the_fresh_list() {
    let mut dialog = review_dialog("acme/app");
    assert_eq!(dialog.poll(), CloneOutcome::LoadProjects("corp".into()));
    dialog.set_cached_projects("corp", vec![project("acme/other")]);
    assert_eq!(
        dialog.chosen, None,
        "the copy does not have it: not used up"
    );
    assert_eq!(dialog.search, "");
    dialog.set_projects_answer(
        "corp",
        Ok(vec![project("acme/other"), project("acme/app")]),
        false,
    );
    assert_eq!(dialog.chosen.as_deref(), Some("acme/app"));
    assert_eq!(dialog.message.as_deref(), Some("note"));
}

#[test]
fn a_late_saved_copy_never_replaces_a_list_that_is_already_showing() {
    let mut dialog = with_hosted();
    dialog.source = 1;
    dialog.set_projects("corp", Ok(team(0..3)));
    dialog.set_cached_projects("corp", team(0..2));
    assert_eq!(dialog.filtered().len(), 3);
    assert!(!dialog.refreshing.contains("corp"));
}
