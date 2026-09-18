use super::*;

fn remote(name: &str, fetch: &str, push: Option<&str>) -> Remote {
    Remote {
        name: name.into(),
        fetch_url: fetch.into(),
        push_url: Some(push.unwrap_or(fetch).into()),
    }
}

#[test]
fn adding_a_remote_sends_add_then_a_push_url_only_when_it_differs() {
    let mut form = RemoteForm::add(vec!["origin".into()]);
    form.name = "fork".into();
    form.fetch_url = "https://host/me/r.git".into();
    assert_eq!(
        form.submit().unwrap().actions,
        vec![Action::RemoteAdd {
            name: "fork".into(),
            url: "https://host/me/r.git".into()
        }]
    );
    form.push_url = "git@host:me/r.git".into();
    assert_eq!(form.submit().unwrap().actions.len(), 2);
    form.name = "origin".into();
    assert!(form.submit().is_err(), "a taken name is refused");
}

#[test]
fn editing_a_remote_sends_only_what_changed_rename_first() {
    let r = remote("origin", "https://host/o/r.git", Some("git@host:o/r.git"));
    let mut form = RemoteForm::edit(&r, vec!["origin".into(), "fork".into()]);
    assert_eq!(form.push_url, "git@host:o/r.git");
    assert!(form.submit().unwrap().actions.is_empty());

    form.name = "upstream".into();
    form.fetch_url = "https://host/o/r2.git".into();
    form.push_url.clear();
    let submit = form.submit().unwrap();
    assert_eq!(
        submit.actions,
        vec![
            Action::RemoteRename {
                old_name: "origin".into(),
                new_name: "upstream".into()
            },
            Action::RemoteSetUrl {
                name: "upstream".into(),
                url: "https://host/o/r2.git".into()
            },
            // Cleared push field: the separate push URL is removed,
            // not rewritten to a copy of the fetch URL.
            Action::RemoteSetPushUrl {
                name: "upstream".into(),
                url: None
            },
        ]
    );
    assert_eq!(
        submit.renamed,
        Some(("origin".to_string(), "upstream".to_string()))
    );

    form.name = "fork".into();
    assert!(
        form.submit().is_err(),
        "renaming onto a taken name is refused"
    );
}

/// Found by the pty test: a remote whose URL is a local path could
/// not be edited at all, because the untouched fetch URL failed the
/// check meant for typed URLs.
#[test]
fn an_edit_does_not_judge_a_url_the_user_left_alone() {
    let r = remote("local", "C:/repos/thing.git", None);
    let mut form = RemoteForm::edit(&r, vec!["local".into()]);
    form.push_url = "https://host/me/r.git".into();
    assert_eq!(
        form.submit().unwrap().actions,
        vec![Action::RemoteSetPushUrl {
            name: "local".into(),
            url: Some("https://host/me/r.git".into())
        }]
    );
    form.fetch_url = "not a url".into();
    assert!(form.submit().is_err(), "a typed URL is still checked");
}

#[test]
fn a_remote_without_a_separate_push_url_edits_with_an_empty_push_field() {
    let r = remote("origin", "https://host/o/r.git", None);
    let form = RemoteForm::edit(&r, vec!["origin".into()]);
    assert_eq!(form.push_url, "");
}

#[test]
fn the_form_is_driven_by_the_keyboard_alone() {
    let key = |c| KeyEvent::new(c, KeyModifiers::NONE);
    let mut form = RemoteForm::add(Vec::new());
    for c in "gh".chars() {
        form.on_key(key(KeyCode::Char(c)));
    }
    assert_eq!(form.on_key(key(KeyCode::Enter)), FormOutcome::Stay);
    assert_eq!(form.focus, RemoteFormPart::FetchUrl);
    form.on_paste("git@github.com:me/r.git\n");
    form.on_key(key(KeyCode::Tab));
    match form.on_key(key(KeyCode::Enter)) {
        FormOutcome::Submit(s) => assert_eq!(
            s.actions,
            vec![Action::RemoteAdd {
                name: "gh".into(),
                url: "git@github.com:me/r.git".into()
            }]
        ),
        other => panic!("{other:?}"),
    }
    let mut bad = RemoteForm::add(Vec::new());
    bad.focus = RemoteFormPart::Submit;
    assert_eq!(bad.on_key(key(KeyCode::Enter)), FormOutcome::Stay);
    assert!(bad.error.is_some(), "an invalid form explains itself");
    assert_eq!(bad.on_key(key(KeyCode::Esc)), FormOutcome::Close);
}
