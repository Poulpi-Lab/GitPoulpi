//! Inline buttons (`Control::Buttons`): drawn in the value column, no
//! `-> run`, destructive ones ask first and Cancel changes nothing.

use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn press(st: &mut PreferencesState, code: KeyCode) {
    st.on_key(KeyEvent::new(code, KeyModifiers::NONE));
}

/// Selects the SSH page's "Forget all usernames and passwords" row.
fn select_forget(st: &mut PreferencesState) {
    go(st, PrefSection::Ssh);
    st.focus = PrefFocus::Fields;
    st.field = st
        .fields()
        .iter()
        .position(|f| f.target == Target::Action(PrefAction::ForgetCredentials))
        .unwrap();
}

#[test]
fn no_page_prints_an_arrow_run_any_more() {
    for section in PrefSection::ALL {
        if !section.is_built() {
            continue;
        }
        let dir = tempfile::tempdir().unwrap();
        let mut st = state(dir.path());
        go(&mut st, *section);
        let screen = draw_screen(&mut st);
        assert!(
            !screen.contains("\u{2192} run") && !screen.contains("\u{2192} lancer"),
            "{section:?}:\n{screen}"
        );
    }
}

#[test]
fn a_button_row_draws_its_button_and_records_its_rect() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    go(&mut st, PrefSection::Ssh);
    let screen = draw_screen(&mut st);
    let line = screen
        .lines()
        .find(|l| l.contains("Forget all usernames and passwords"))
        .unwrap_or_else(|| panic!("no forget button:\n{screen}"));
    assert!(!line.contains('\u{2192}'), "{line}");
    assert!(
        !st.btn_rects.is_empty(),
        "the paint walk recorded the rects"
    );
}

#[test]
fn forget_credentials_asks_and_cancel_keeps_the_tokens() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    add_profile(&mut st, IntegrationKind::Gitlab);
    let token_ref = st.config.integrations[0].token_ref.clone();
    st.tokens.store(&token_ref, "secret").unwrap();
    select_forget(&mut st);

    press(&mut st, KeyCode::Enter);
    assert!(st.confirm_action.is_some(), "a destructive button asks");
    assert!(st.tokens.has(&token_ref), "nothing is forgotten yet");
    let screen = draw_screen(&mut st);
    assert!(screen.contains("undone"), "{screen}");

    // Cancel is focused: Enter keeps the tokens.
    press(&mut st, KeyCode::Enter);
    assert!(st.confirm_action.is_none());
    assert!(st.tokens.has(&token_ref), "Cancel keeps the token");

    // Ask again and confirm.
    press(&mut st, KeyCode::Enter);
    press(&mut st, KeyCode::Right);
    press(&mut st, KeyCode::Enter);
    assert!(st.confirm_action.is_none());
    assert!(!st.tokens.has(&token_ref), "the token was forgotten");
}

#[test]
fn escape_closes_the_confirmation_without_running_the_action() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    add_profile(&mut st, IntegrationKind::Gitlab);
    let token_ref = st.config.integrations[0].token_ref.clone();
    st.tokens.store(&token_ref, "secret").unwrap();
    select_forget(&mut st);
    press(&mut st, KeyCode::Enter);
    press(&mut st, KeyCode::Esc);
    assert!(st.confirm_action.is_none());
    assert!(st.tokens.has(&token_ref));
}

#[test]
fn a_non_destructive_button_runs_at_once() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    st.run_action(PrefAction::InitializeLfs);
    assert!(st.lfs_init_requested);
    assert!(crate::preferences::buttons::is_destructive(
        PrefAction::ForgetCredentials
    ));
    assert!(!crate::preferences::buttons::is_destructive(
        PrefAction::InitializeLfs
    ));
}
