use super::*;

fn actions(entries: Vec<Entry<RemoteUiAction>>) -> Vec<RemoteUiAction> {
    entries
        .into_iter()
        .filter_map(|e| match e {
            Entry::Action { action, .. } => Some(action),
            _ => None,
        })
        .collect()
}

#[test]
fn a_remote_tracking_namespace_with_no_remote_is_a_leftover_once_the_list_is_read() {
    let refs = vec![
        remote_branch_at("origin", "main", oid(1)),
        remote_branch_at("cm6", "main", oid(1)),
        remote_branch_at("cm6", "topic", oid(2)),
    ];
    let configured = vec!["origin".to_string()];
    let got: Vec<String> = leftover_names(&refs, &configured, true)
        .into_iter()
        .collect();
    assert_eq!(got, vec!["cm6".to_string()]);
    assert!(
        leftover_names(&refs, &configured, false).is_empty(),
        "before the remote list is read, nothing is a leftover"
    );
}

#[test]
fn a_leftovers_menu_offers_neither_edit_nor_fetch_but_remove_and_add() {
    let acts = actions(leftover_menu_entries("cm6", false));
    assert!(acts.contains(&RemoteUiAction::RemoveLeftover("cm6".into())));
    assert!(acts.contains(&RemoteUiAction::AddAsRemote("cm6".into())));
    assert!(!acts.contains(&RemoteUiAction::EditRemote("cm6".into())));
    assert!(!acts.contains(&RemoteUiAction::FetchRemote("cm6".into())));
    assert!(!acts.contains(&RemoteUiAction::SetDefaultRemote("cm6".into())));
}

#[test]
fn the_add_form_for_a_leftover_carries_its_name() {
    let mut ui = RemoteUi::load(std::path::Path::new("/nonexistent-leftover-test"));
    ui.loaded = true;
    let form = RemoteForm::add_named(ui.names(), "cm6");
    assert_eq!(form.name, "cm6");
    assert!(ui.is_leftover("cm6"));
}
