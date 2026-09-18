//! The trailer rule's form, the identity rule's trailer option, the message
//! hash option and what a preview row says about a trailer change (S17).

use super::render::{previewed, text};
use super::*;
use gitpoulpi_core::domain::rewrite::{
    CommitChange, FieldChange, RewriteField, TrailerAction, TrailerIfExists,
};

fn field_at(v: &RewriteView, id: FieldId) -> usize {
    v.detail
        .fields
        .iter()
        .position(|f| f.id == id)
        .unwrap_or_else(|| panic!("no {id:?} in {:?}", v.detail.fields))
}

fn ids(v: &RewriteView) -> Vec<FieldId> {
    v.detail.fields.iter().map(|f| f.id).collect()
}

fn trailer(v: &RewriteView) -> &gitpoulpi_core::domain::rewrite::TrailerRule {
    match &v.recipe.rules[v.rules.cursor].kind {
        RuleKind::Trailer(r) => r,
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_add_form_has_labelled_fields_and_a_dropdown_for_the_action() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::Trailer);
    assert_eq!(
        ids(&v),
        [
            FieldId::TrailerAction,
            FieldId::TrailerKey,
            FieldId::TrailerValue,
            FieldId::TrailerIfExists,
            FieldId::UpdateHashes,
        ]
    );
    let at = field_at(&v, FieldId::TrailerValue);
    v.edit_field(at, FieldInput::Text("Sam Roe <sam@example.com>".into()));
    let s = text(&v);
    for label in [
        "Action",
        "Trailer",
        "Value",
        "If it already exists",
        "Update commit ids",
    ] {
        assert!(s.contains(label), "{label}\n{s}");
    }
    // Choices are dropdowns (a closed list with the arrow), never a cycle.
    assert!(s.contains("Add (unless identical)"), "{s}");
    insta::assert_snapshot!("form_trailer_add", s);
}

#[test]
fn the_action_chooses_the_fields_remove_and_map() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::Trailer);
    v.edit_field(0, FieldInput::Choice(1));
    assert_eq!(
        ids(&v),
        [
            FieldId::TrailerAction,
            FieldId::TrailerKey,
            FieldId::TrailerFind,
            FieldId::UpdateHashes,
        ]
    );
    let s = text(&v);
    assert!(s.contains("Value contains"), "{s}");
    assert!(s.contains("any value"), "{s}");
    insta::assert_snapshot!("form_trailer_remove", s);

    v.edit_field(0, FieldInput::Choice(2));
    assert_eq!(trailer(&v).action, TrailerAction::Map);
    assert_eq!(
        ids(&v),
        [
            FieldId::TrailerAction,
            FieldId::TrailerKey,
            FieldId::TrailerFind,
            FieldId::With,
            FieldId::UpdateHashes,
        ]
    );
    let find = field_at(&v, FieldId::TrailerFind);
    v.edit_field(find, FieldInput::Text("@old.org".into()));
    let with = field_at(&v, FieldId::With);
    v.edit_field(with, FieldInput::Text("@new.org".into()));
    let rule = trailer(&v).clone();
    assert_eq!(rule.find.as_ref().unwrap().pattern, "@old.org");
    assert_eq!(rule.with, "@new.org");
    insta::assert_snapshot!("form_trailer_map", text(&v));
}

#[test]
fn the_if_exists_dropdown_sets_the_rule() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::Trailer);
    let at = field_at(&v, FieldId::TrailerIfExists);
    assert_eq!(trailer(&v).if_exists, TrailerIfExists::Add);
    v.edit_field(at, FieldInput::Choice(2));
    assert_eq!(trailer(&v).if_exists, TrailerIfExists::Skip);
    v.edit_field(at, FieldInput::Choice(1));
    assert_eq!(trailer(&v).if_exists, TrailerIfExists::Replace);
}

#[test]
fn the_message_hash_option_belongs_to_the_recipe_not_the_rule() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::Trailer);
    assert!(!v.recipe.options.update_message_hashes, "off by default");
    let at = field_at(&v, FieldId::UpdateHashes);
    v.edit_field(at, FieldInput::Toggle(true));
    assert!(v.recipe.options.update_message_hashes);
    // The rule itself did not change, and another message rule shows it on.
    v.add_rule(RuleTemplate::MessageReplace);
    let at = field_at(&v, FieldId::UpdateHashes);
    assert_eq!(v.detail.fields[at].value, FieldValue::Toggle(true));
    v.edit_field(at, FieldInput::Toggle(false));
    assert!(!v.recipe.options.update_message_hashes);
    // A rule that does not edit messages does not carry the option.
    v.add_rule(RuleTemplate::DateShift);
    assert!(!ids(&v).contains(&FieldId::UpdateHashes));
}

#[test]
fn the_option_makes_the_plan_out_of_date_like_any_edit() {
    let (mut v, _q, _t) = previewed();
    v.add_rule(RuleTemplate::Trailer);
    v.plan_state = PlanState::Current;
    let at = field_at(&v, FieldId::UpdateHashes);
    v.edit_field(at, FieldInput::Toggle(true));
    assert_eq!(v.plan_state, PlanState::OutOfDate);
}

#[test]
fn an_identity_rule_can_also_map_trailers_default_off() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::Identity);
    let at = field_at(&v, FieldId::MapTrailers);
    match &v.recipe.rules[0].kind {
        RuleKind::Identity(r) => assert!(!r.map_trailers),
        other => panic!("{other:?}"),
    }
    assert!(text(&v).contains("Also map trailers"));
    v.edit_field(at, FieldInput::Toggle(true));
    match &v.recipe.rules[0].kind {
        RuleKind::Identity(r) => assert!(r.map_trailers),
        other => panic!("{other:?}"),
    }
}

fn message_row(old: &str, new: &str) -> CommitChange {
    CommitChange {
        old: Oid([1; 20]),
        new: Some(Oid([2; 20])),
        fields: vec![FieldChange {
            field: RewriteField::Message,
            old: old.into(),
            new: new.into(),
        }],
        paths: Vec::new(),
        pushed: false,
    }
}

#[test]
fn a_row_shows_the_trailers_added_and_removed_not_the_unchanged_subject() {
    let (mut v, _q, _t) = previewed();
    v.rows.loaded = vec![
        message_row(
            "subject\n\nbody\n",
            "subject\n\nbody\n\nSigned-off-by: Sam Roe <sam@example.com>\n",
        ),
        message_row(
            "subject\n\nSigned-off-by: Al <al@old.org>\nFoo: 1\n",
            "subject\n\nSigned-off-by: Al <al@new.org>\nFoo: 1\n",
        ),
    ];
    v.tab = PreviewTab::Commits;
    let s = text(&v);
    assert!(
        s.contains("message: +Signed-off-by: Sam Roe <sam@example.com>"),
        "{s}"
    );
    assert!(
        s.contains("message: +Signed-off-by: Al <al@new.org> -Signed-off-by: Al <al@old.org>"),
        "{s}"
    );
    assert!(!s.contains("subject \u{2192} subject"), "{s}");
}

#[test]
fn the_summary_counts_the_ids_that_followed_and_those_left() {
    let (mut v, _q, _t) = previewed();
    if let Some(plan) = v.plan.as_mut() {
        plan.counts.message_hashes_updated = 5;
        plan.counts.message_hashes_left = 1;
    }
    v.tab = PreviewTab::Summary;
    let s = text(&v);
    assert!(s.contains("5 commit ids in messages were updated."), "{s}");
    assert!(
        s.contains("1 commit id in messages was left as it is (ambiguous or unknown)."),
        "{s}"
    );
}
