use super::*;
use crate::preferences::ai_page::prompt_vars::{completion_at, matching};

fn open_editor(dir: &Path) -> PreferencesState {
    let mut state = state(dir);
    select(&mut state, "Custom instructions");
    state.activate();
    assert!(state.ai.prompt_open());
    state
}

fn text(state: &PreferencesState) -> String {
    state.ai.prompt.as_ref().unwrap().text.clone()
}

#[test]
fn completion_is_found_only_inside_an_open_variable() {
    let c = completion_at("say ${re", 8).unwrap();
    assert_eq!((c.start, c.prefix.as_str()), (4, "re"));
    assert_eq!(completion_at("${", 2).unwrap().prefix, "");
    assert!(completion_at("$${re", 5).is_none(), "escaped");
    assert!(completion_at("{re", 3).is_none());
    assert!(completion_at("${repo} x", 9).is_none(), "closed variable");
    assert!(completion_at("${a b", 5).is_none());
}

#[test]
fn matching_puts_prefix_matches_first_then_contains() {
    assert_eq!(matching("re"), ["repo", "ref", "remote"]);
    let all = matching("");
    assert_eq!(all.len(), gitpoulpi_ai::vars::VARIABLES.len());
    assert_eq!(matching("UIL"), ["uilang"]);
    assert!(matching("zzz").is_empty());
    // "commit" completes `commit` and `commit_subject`; "mit" only contains.
    assert_eq!(matching("commit"), ["commit", "commit_subject"]);
    assert_eq!(matching("mit"), ["commit", "commit_subject"]);
}

#[test]
fn typing_dollar_brace_offers_variables_and_enter_inserts_one() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = open_editor(dir.path());
    type_text(&mut state, "Repo: ${re");
    let (c, m) = state.ai.prompt.as_ref().unwrap().completion().unwrap();
    assert_eq!(c.prefix, "re");
    assert_eq!(m[0], "repo");
    let screen = draw(&mut state, 120, 40);
    assert!(screen.contains("${repo}"), "{screen}");
    state.on_key(key(KeyCode::Enter));
    assert_eq!(text(&state), "Repo: ${repo}");
    assert!(state.ai.prompt.as_ref().unwrap().completion().is_none());
    // Enter now is a newline again.
    state.on_key(key(KeyCode::Enter));
    assert_eq!(text(&state), "Repo: ${repo}\n");
}

#[test]
fn down_moves_tab_accepts_and_a_typed_closing_brace_is_not_doubled() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = open_editor(dir.path());
    type_text(&mut state, "${re");
    state.on_key(key(KeyCode::Down));
    state.on_key(key(KeyCode::Tab));
    assert_eq!(text(&state), "${ref}");
    // A `}` already after the caret is absorbed.
    let mut state = open_editor(dir.path());
    type_text(&mut state, "${}");
    state.on_key(key(KeyCode::Left));
    type_text(&mut state, "ui");
    state.on_key(key(KeyCode::Enter));
    assert_eq!(text(&state), "${uilang}");
}

#[test]
fn escape_closes_the_popup_not_the_editor_until_the_next_variable() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = open_editor(dir.path());
    type_text(&mut state, "${re");
    state.on_key(key(KeyCode::Esc));
    assert!(state.ai.prompt_open(), "Esc only closed the popup");
    assert!(state.ai.prompt.as_ref().unwrap().completion().is_none());
    type_text(&mut state, "p");
    assert!(state.ai.prompt.as_ref().unwrap().completion().is_none());
    state.on_key(key(KeyCode::Esc));
    assert!(
        state.ai.prompt_open(),
        "a second Esc asks about the typed text"
    );
    state.on_key(key(KeyCode::Right));
    state.on_key(key(KeyCode::Enter));
    assert!(!state.ai.prompt_open(), "Discard cancels the editor");
}

#[test]
fn the_variables_dropdown_inserts_at_the_caret() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = open_editor(dir.path());
    type_text(&mut state, "Answer in ");
    state.on_key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL));
    assert!(state.ai.prompt.as_ref().unwrap().vars.combo.is_open());
    let screen = draw(&mut state, 120, 40);
    assert!(screen.contains("${uilang}"), "{screen}");
    state.on_key(key(KeyCode::Enter)); // the first entry
    assert_eq!(text(&state), "Answer in ${uilang}");
    assert!(!state.ai.prompt.as_ref().unwrap().vars.combo.is_open());
}

#[test]
fn the_dropdown_works_with_the_mouse_and_its_rows_describe_themselves() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = open_editor(dir.path());
    draw(&mut state, 120, 40);
    let field = state.ai.prompt.as_ref().unwrap().vars.combo.field_rect();
    assert!(field.width > 0);
    state.on_mouse(
        MouseEventKind::Down(MouseButton::Left),
        field.x + 1,
        field.y,
    );
    assert!(state.ai.prompt.as_ref().unwrap().vars.combo.is_open());
    draw(&mut state, 120, 40);
    let (band, row) = state.ai.prompt.as_ref().unwrap().vars.combo.item_rects()[1];
    assert_eq!(row, 1);
    state.on_mouse(MouseEventKind::Moved, band.x + 2, band.y);
    let screen = draw(&mut state, 120, 40);
    assert!(screen.contains("Repository folder name"), "{screen}");
    // A searchable popup: one click selects, the second (double click) picks.
    state.on_mouse(MouseEventKind::Down(MouseButton::Left), band.x + 2, band.y);
    assert_eq!(text(&state), "", "a single click only selects");
    state.on_mouse(MouseEventKind::Down(MouseButton::Left), band.x + 2, band.y);
    assert_eq!(text(&state), "${repo}");
    assert!(state.ai.prompt_open(), "the editor stays open");
}

#[test]
fn clicking_an_autocomplete_row_inserts_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = open_editor(dir.path());
    type_text(&mut state, "${");
    draw(&mut state, 120, 40);
    let hits = state.ai.prompt.as_ref().unwrap().vars.ac_hits.clone();
    let (band, i) = hits[2];
    assert_eq!(i, 2);
    state.on_mouse(MouseEventKind::Down(MouseButton::Left), band.x + 1, band.y);
    assert_eq!(text(&state), "${branch}");
}

#[test]
fn an_unknown_variable_is_warned_about_and_saved_as_written() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = open_editor(dir.path());
    type_text(&mut state, "Hi ${nope} ${repo}");
    assert_eq!(state.ai.prompt.as_ref().unwrap().unknown(), ["nope"]);
    let screen = draw(&mut state, 120, 40);
    assert!(screen.contains("Unknown variable: ${nope}"), "{screen}");
    state.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert_eq!(state.config.ai.global_instructions, "Hi ${nope} ${repo}");
}
