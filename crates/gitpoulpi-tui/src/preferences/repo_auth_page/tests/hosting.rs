//! The hosting rows of the SSH & credentials page: "Use for the hosting API"
//! of a token command, and the repository's Assignment block.

use super::*;

fn label_of(st: &PreferencesState, start: &str) -> String {
    labels(st)
        .into_iter()
        .find(|l| l.starts_with(start))
        .unwrap_or_else(|| panic!("no `{start}` in {:?}", labels(st)))
}

#[test]
fn use_for_hosting_is_sent_with_the_profile_and_asks_the_person() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, queue) = command_page(dir.path());
    let row = label_of(&st, "Use for the hosting API");
    select(&mut st, &row);
    st.activate();
    assert!(st.repo_auth.draft.use_for_hosting, "the toggle is on");
    press(&mut st, "Save");
    assert!(
        st.ra_overlay_open(),
        "a command still needs its confirmation"
    );
    st.on_key(key(KeyCode::Char('1')));
    let sent = queue.queued();
    assert_eq!(sent.len(), 1, "{sent:?}");
    let args: RepoAuthSetArgs = serde_json::from_value(sent[0].1.clone()).unwrap();
    assert!(args.confirm);
    assert!(matches!(
        args.profile.https,
        Some(HttpsAuth::TokenCommand {
            use_for_hosting: true,
            ..
        })
    ));
}

#[test]
fn the_row_only_shows_with_a_token_command() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, _queue) = page(dir.path());
    loaded(&mut st);
    assert!(!labels(&st)
        .iter()
        .any(|l| l.starts_with("Use for the hosting API")));
}

#[test]
fn assignment_is_a_choice_saved_at_once() {
    let dir = tempfile::tempdir().unwrap();
    let (mut st, _queue) = page(dir.path());
    loaded(&mut st);
    pick(&mut st, "Assignment", "Blocked");
    let path = st.repo_state_path.clone();
    assert!(
        RepoState::load(&path).unwrap().assignment_blocked,
        "written to the repository's state file"
    );
    let at = labels(&st).iter().position(|l| l == "Assignment").unwrap();
    let Control::Choice { index, .. } = st.fields()[at].control.clone() else {
        panic!("a choice");
    };
    assert_eq!(index, 1, "the row shows Blocked");
    pick(&mut st, "Assignment", "Allowed");
    assert!(!RepoState::load(&path).unwrap().assignment_blocked);
}
