use super::*;

fn twin_refs() -> Vec<gitpoulpi_core::domain::Ref> {
    use crate::test_support::*;
    let mut local = local_branch_at("feat", oid(1));
    local.upstream = Some("origin/feat".into());
    vec![local, remote_branch_at("origin", "feat", oid(1))]
}

#[test]
fn a_branch_with_a_remote_twin_gets_the_three_delete_rows() {
    let refs = twin_refs();
    let (local_twin, remote_twin) = branch_twins(&refs, RefRowKind::Local, "feat");
    assert_eq!(
        (local_twin, remote_twin.as_deref()),
        (None, Some("origin/feat"))
    );
    let ctx = MenuContext {
        remote_twin: Some("origin/feat".into()),
        ..Default::default()
    };
    let labels: Vec<String> = delete_entries(RefRowKind::Local, "feat", &ctx)
        .iter()
        .map(|e| match e {
            context_menu::Entry::Action { label, .. } => label.clone(),
            _ => String::new(),
        })
        .collect();
    assert_eq!(labels.len(), 3);
    assert!(labels[0].ends_with("Delete feat..."));
    assert!(labels[1].ends_with("Delete origin/feat..."));
    assert!(labels[2].ends_with("Delete feat and origin/feat..."));
    let targets = delete_targets(&refs, RefRowKind::Local, "feat");
    assert_eq!(targets.local.as_deref(), Some("feat"));
    assert_eq!(
        targets.remote,
        Some(("origin".to_string(), "feat".to_string()))
    );
}

#[test]
fn a_remote_row_resolves_its_local_twin_and_the_checked_out_branch_is_refused() {
    let mut refs = twin_refs();
    let (local, _) = branch_twins(&refs, RefRowKind::Remote, "origin/feat");
    assert_eq!(local.as_deref(), Some("feat"));
    assert_eq!(checked_out_refusal(&refs, "feat"), None);
    refs[0].is_head = true;
    assert!(checked_out_refusal(&refs, "feat")
        .unwrap()
        .contains("it is checked out"));
}
