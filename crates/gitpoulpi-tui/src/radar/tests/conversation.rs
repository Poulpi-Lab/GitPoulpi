//! The conversation of an item whose repository is not cloned: the Radar
//! hands the profile and the project to the event loop (no checkout), offers
//! it in the row menu, and hosts the view over the list.

use super::*;
use crate::radar::account::{AccountAnswer, CloneIndex};
use crate::radar::worker::Setup;
use gitpoulpi_hosting::account::{AccountIssue, AccountItems, AccountPr};

fn with_far(dir: &std::path::Path) -> RadarUi {
    let (mut ui, ..) = radar(dir);
    std::fs::write(
        dir.join("cfg").join("config.toml"),
        "[[integrations]]\nid = \"gh\"\nkind = \"github\"\nurl = \"https://github.com\"\ntoken_ref = \"integration:gh\"\n",
    )
    .unwrap();
    ui.apply_setup(Setup {
        round: ui.round,
        active: vec!["gh".into()],
        offers: vec![],
        clones: CloneIndex::default(),
    });
    let mut p = pr(20, "ann", "2026-10-12T10:00:00Z");
    p.title = "Far change 20".into();
    ui.apply_account(AccountAnswer {
        round: 0,
        id: "gh".into(),
        label: "GitHub".into(),
        kind: gitpoulpi_config::IntegrationKind::Github,
        host: "github.com".into(),
        me: "kai".into(),
        items: AccountItems {
            prs: vec![AccountPr {
                project: "acme/far".into(),
                clone_url: String::new(),
                pr: p,
            }],
            issues: vec![AccountIssue {
                project: "acme/far".into(),
                clone_url: String::new(),
                issue: IssueSummary {
                    number: 30,
                    iid_label: "#30".into(),
                    reference: "#30".into(),
                    title: "Far bug".into(),
                    web_url: "https://github.com/acme/far/issues/30".into(),
                    ..IssueSummary::default()
                },
            }],
            ..AccountItems::default()
        },
        fetched_at: 0,
        cached: false,
        problem: None,
        paused: None,
    });
    ui.rebuild();
    ui
}

fn request(out: Outcome) -> ConversationRequest {
    match out {
        Outcome::ApiConversation(r) => *r,
        other => panic!("not a conversation request: {other:?}"),
    }
}

#[test]
fn i_on_a_not_cloned_issue_asks_for_its_conversation_not_a_clone() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path());
    select(&mut ui, "Far bug");
    let r = request(press(&mut ui, 'i'));
    assert_eq!(
        (
            r.integration.as_str(),
            r.project.as_str(),
            r.number,
            r.pr,
            r.label.as_str()
        ),
        ("gh", "acme/far", 30, false, "#30")
    );
    assert_eq!(r.web_url, "https://github.com/acme/far/issues/30");
}

#[test]
fn a_not_cloned_request_has_its_conversation_too() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path());
    select(&mut ui, "Far change 20");
    let r = request(press(&mut ui, 'i'));
    assert_eq!((r.project.as_str(), r.number, r.pr), ("acme/far", 20, true));
}

#[test]
fn the_row_menu_offers_it_for_both_kinds() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path());
    for title in ["Far bug", "Far change 20"] {
        select(&mut ui, title);
        let index = ui.selected_item().unwrap();
        let acts: Vec<Act> = ui
            .item_entries(index)
            .iter()
            .filter_map(|e| match e {
                Entry::Action { action, .. } => Some(*action),
                _ => None,
            })
            .collect();
        assert!(acts.contains(&Act::Conversation), "{title}: {acts:?}");
        // What needs a checkout stays out.
        assert!(!acts.contains(&Act::Checkout), "{title}: {acts:?}");
    }
}

#[test]
fn the_palette_command_runs_the_same_action() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = with_far(dir.path());
    select(&mut ui, "Far bug");
    assert!(matches!(
        ui.command(RadarCommand::Act(Act::Conversation)),
        Outcome::ApiConversation(_)
    ));
}
