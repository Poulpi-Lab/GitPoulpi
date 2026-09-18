//! The Settings close guard (X1) and the close cross of the editors inside
//! Settings: what asks, what each button does, and no "Esc" text beside a
//! cross.

use super::*;
use crate::preferences::repo_pages::RpTarget;
use crossterm::event::MouseButton;
use gitpoulpi_core::workflow::WorkflowTemplate;
use workflow_page::WfTarget;

fn press(st: &mut PreferencesState, code: KeyCode) -> bool {
    st.on_key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn click(st: &mut PreferencesState, col: u16, row: u16) -> bool {
    st.on_mouse(MouseEventKind::Down(MouseButton::Left), col, row)
}

fn dirty_workflow(dir: &Path) -> PreferencesState {
    gitpoulpi_session::save_shared(
        &gitpoulpi_session::workflow_path_for(dir, ".gitpoulpi"),
        &WorkflowTemplate::GitFlow.workflow(),
        true,
    )
    .unwrap();
    let mut st = state_at(dir, "Workflow");
    st.wf_commit_text(WfTarget::TopicLabel, "Feat");
    assert!(st.wf_dirty());
    st
}

#[test]
fn a_clean_settings_closes_at_once() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    assert!(!press(&mut st, KeyCode::Esc));
    let mut st = state(dir.path());
    assert!(!press(&mut st, KeyCode::Char('q')));
}

#[test]
fn an_unsaved_workflow_draft_asks_and_cancel_keeps_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = dirty_workflow(dir.path());
    assert!(press(&mut st, KeyCode::Esc), "still open");
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Close Settings?"), "{screen}");
    assert!(screen.contains("Discard and close"), "{screen}");
    assert!(screen.contains("Go to it"), "{screen}");
    // Esc is Cancel: the question goes, the draft stays.
    assert!(press(&mut st, KeyCode::Esc));
    assert!(st.close_ask.is_none());
    assert!(st.wf_dirty(), "Cancel keeps the draft");
    // `q`, then Discard and close (second button) closes.
    press(&mut st, KeyCode::Char('q'));
    assert!(st.close_ask.is_some());
    press(&mut st, KeyCode::Right);
    assert!(!press(&mut st, KeyCode::Enter), "Discard closes Settings");
}

#[test]
fn go_to_it_lands_on_the_workflow_page() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = dirty_workflow(dir.path());
    go(&mut st, PrefSection::General);
    assert!(press(&mut st, KeyCode::Esc));
    assert!(
        press(&mut st, KeyCode::Enter),
        "Go to it is the first button"
    );
    assert!(st.close_ask.is_none());
    assert_eq!(st.current_section(), PrefSection::Workflow);
    assert!(st.wf_dirty());
}

#[test]
fn the_cross_asks_too() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = dirty_workflow(dir.path());
    draw_screen(&mut st);
    let cross = st.close_rect.expect("a cross");
    assert!(click(&mut st, cross.x + 1, cross.y));
    assert!(st.close_ask.is_some(), "the cross asks");
    press(&mut st, KeyCode::Esc);
    assert!(st.close_ask.is_none());
}

#[test]
fn a_profile_without_an_email_asks_and_go_to_it_selects_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    st.config.profiles.push({
        let mut p = Profile::new("a", "Ada");
        p.author_email = "ada@example.com".into();
        p
    });
    st.config.profiles.push(Profile::new("b", "Bob"));
    assert!(press(&mut st, KeyCode::Esc));
    let screen = draw_screen(&mut st);
    assert!(
        screen.contains("Close anyway"),
        "a warning, not lost work:\n{screen}"
    );
    assert!(!screen.contains("Discard and close"), "{screen}");
    // Go to it is the focused button.
    press(&mut st, KeyCode::Enter);
    assert_eq!(st.current_section(), PrefSection::Profiles);
    assert_eq!(st.profile_index, 1, "the profile with no email");
    assert_eq!(st.focus, PrefFocus::List);
    // Close anyway (second button) closes.
    press(&mut st, KeyCode::Esc);
    press(&mut st, KeyCode::Right);
    assert!(!press(&mut st, KeyCode::Enter));
}

#[test]
fn an_unsaved_template_asks() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "CommitTemplate");
    st.repo_pages.template.dirty = true;
    assert!(press(&mut st, KeyCode::Esc));
    assert!(draw_screen(&mut st).contains("message template"));
}

#[test]
fn the_undo_off_question_is_a_confirm_box_with_a_cross() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "General");
    st.focus = PrefFocus::Fields;
    st.field = st
        .fields()
        .iter()
        .position(|f| f.target == Target::Global("general.undo_enabled"))
        .expect("the undo row");
    st.activate();
    assert!(st.confirm_undo_off.is_some());
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Turn Undo off"), "{screen}");
    let title_row = screen
        .lines()
        .find(|l| l.contains("Turn Undo off"))
        .unwrap();
    assert!(
        title_row.contains('\u{2715}'),
        "a cross on the box: {title_row}"
    );
    press(&mut st, KeyCode::Esc);
    assert!(st.confirm_undo_off.is_none());
}

#[test]
fn the_hook_editor_has_a_cross_and_asks_before_discarding() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "Workflow");
    st.wf_open_editor(WfTarget::HookAfterStart);
    let screen = draw_screen(&mut st);
    let title = screen
        .lines()
        .find(|l| l.contains("After start") && l.contains('\u{2715}'))
        .unwrap_or_else(|| panic!("a second cross on the editor frame:\n{screen}"));
    assert!(!title.contains("Esc"), "{title}");
    assert!(
        !screen.contains("cancel"),
        "no cancel hint in the footer:\n{screen}"
    );
    // Untouched: Esc closes at once.
    press(&mut st, KeyCode::Esc);
    assert!(!st.wf_editor_open());
    // Edited: Esc asks, Cancel-side first.
    st.wf_open_editor(WfTarget::HookAfterStart);
    press(&mut st, KeyCode::Char('x'));
    press(&mut st, KeyCode::Esc);
    assert!(st.wf_editor_open(), "edited: asks first");
    press(&mut st, KeyCode::Right);
    press(&mut st, KeyCode::Enter);
    assert!(!st.wf_editor_open());
}

#[test]
fn the_paths_editor_closes_on_its_cross_and_swallows_an_outside_click_when_edited() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "SparseCheckout");
    st.rp_open_editor(RpTarget::SparsePaths);
    draw_screen(&mut st);
    let cross = st.rp_editor_close().unwrap().cross().expect("a cross");
    click(&mut st, cross.x + 1, cross.y);
    assert!(
        !st.rp_editor_open(),
        "a pristine editor closes on the cross"
    );

    st.rp_open_editor(RpTarget::SparsePaths);
    press(&mut st, KeyCode::Char('x'));
    draw_screen(&mut st);
    click(&mut st, 0, 0);
    assert!(
        st.rp_editor_open(),
        "an outside click never loses typed text"
    );
    let cross = st.rp_editor_close().unwrap().cross().expect("a cross");
    click(&mut st, cross.x + 1, cross.y);
    assert!(st.rp_editor_open(), "the cross asks");
    assert!(st.rp_editor_close().unwrap().asking());
}

#[test]
fn the_search_popup_has_a_cross_and_no_esc_text() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    st.on_key(KeyEvent::from(KeyCode::Char('/')));
    let screen = draw_screen(&mut st);
    let popup = st.search.as_ref().unwrap().popup;
    let row = screen.lines().nth(popup.y as usize).unwrap().to_string();
    assert!(
        row.contains('\u{2715}'),
        "a cross on the popup border: {row}"
    );
    assert!(!row.contains("Esc"), "{row}");
    let cross = st.search.as_ref().unwrap().close.expect("a cross rect");
    click(&mut st, cross.x + 1, cross.y);
    assert!(st.search.is_none(), "the cross closes the popup");
}

#[test]
#[cfg(feature = "ai")]
fn the_ai_prompt_editor_has_a_cross_and_no_esc_text() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "Ai");
    st.config.ai.enabled = true;
    st.focus = PrefFocus::Fields;
    st.set_sub_tab(SubTab::General);
    st.focus = PrefFocus::Fields;
    st.field = st
        .fields()
        .iter()
        .position(|f| f.label == "Custom instructions")
        .expect("the instructions row");
    st.activate();
    assert!(st.ai.prompt_open());
    let screen = draw_screen(&mut st);
    let frame_row = screen
        .lines()
        .find(|l| l.contains("Custom instructions") && l.contains('\u{2715}'))
        .unwrap_or_else(|| panic!("the editor's own cross next to the page's:\n{screen}"));
    assert!(!frame_row.contains("Esc"), "{frame_row}");
    assert!(
        !screen.contains("cancel"),
        "no cancel hint in the footer:\n{screen}"
    );
}
