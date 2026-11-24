//! The Profile filter: rows narrowed to one integration profile, the
//! remembered choice, a profile that no longer exists, narrow windows.

use super::*;
use crate::radar::account::{AccountAnswer, CloneIndex};
use crate::radar::worker::Setup;
use gitpoulpi_config::{Config, Integration, IntegrationKind};
use gitpoulpi_hosting::account::{AccountItems, AccountPr};

fn write_profiles(cfg: &std::path::Path, profiles: &[(&str, IntegrationKind, &str)]) {
    let mut config = Config::default();
    for (id, kind, name) in profiles {
        let mut i = Integration::new(*id, *kind, "https://example.com");
        i.name = (*name).to_string();
        config.integrations.push(i);
    }
    config.save(&cfg.join("config.toml")).unwrap();
}

fn far(n: u64, id: &str) -> AccountAnswer {
    let mut p = pr(n, "ann", "2026-10-12T10:00:00Z");
    p.title = format!("Far change {n}");
    p.review_requested = true;
    AccountAnswer {
        round: 0,
        id: id.into(),
        label: id.into(),
        kind: IntegrationKind::GitlabSelfManaged,
        host: "git.example.com".into(),
        me: "kai".into(),
        items: AccountItems {
            prs: vec![AccountPr {
                project: "acme/far".into(),
                clone_url: "https://git.example.com/acme/far.git".into(),
                pr: p,
            }],
            ..AccountItems::default()
        },
        fetched_at: 0,
        cached: false,
        problem: None,
        paused: None,
    }
}

/// Two cloned repositories listed through the profile `gh`, and one far
/// request asked once for the profile `gl`.
fn two_profiles(dir: &std::path::Path) -> RadarUi {
    let (ui, ..) = radar(dir);
    drop(ui);
    write_profiles(
        &dir.join("cfg"),
        &[
            ("gh", IntegrationKind::Github, "personal"),
            ("gl", IntegrationKind::GitlabSelfManaged, "work"),
        ],
    );
    let (mut ui, ..) = radar(dir);
    ui.apply_setup(Setup {
        round: ui.round,
        active: vec!["gl".into()],
        offers: vec![],
        clones: CloneIndex::default(),
    });
    ui.apply_account(far(20, "gl"));
    ui.rebuild();
    ui.fill_combos();
    ui
}

fn titles(ui: &RadarUi) -> Vec<String> {
    ui.rows
        .iter()
        .filter_map(|r| match r {
            Row::Item(i) => Some(ui.board.items[*i].title.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_combobox_lists_all_profiles_then_each_profile_by_name() {
    let dir = tempfile::tempdir().unwrap();
    let ui = two_profiles(dir.path());
    let labels: Vec<String> = ui.profile_combo_labels();
    assert_eq!(labels, vec!["All profiles", "personal", "work"]);
}

#[test]
fn choosing_a_profile_narrows_scope_rows_and_account_rows() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = two_profiles(dir.path());
    assert!(titles(&ui).contains(&"Change 1".to_string()));
    assert!(titles(&ui).contains(&"Far change 20".to_string()));
    ui.picked(Hit::Profile, "gl");
    assert_eq!(titles(&ui), vec!["Far change 20".to_string()]);
    ui.picked(Hit::Profile, "gh");
    let t = titles(&ui);
    assert!(t.contains(&"Change 1".to_string()) && t.contains(&"Bug 7".to_string()));
    assert!(!t.contains(&"Far change 20".to_string()), "{t:?}");
    ui.picked(Hit::Profile, "");
    assert!(titles(&ui).contains(&"Far change 20".to_string()));
}

#[test]
fn the_key_opens_the_combobox_and_a_pick_filters() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = two_profiles(dir.path());
    press(&mut ui, 'P');
    assert!(ui.profile_combo.is_open());
    // Down to "personal", Down to "work", Enter.
    ui.on_key(key(KeyCode::Down));
    ui.on_key(key(KeyCode::Down));
    ui.on_key(key(KeyCode::Enter));
    assert_eq!(ui.profile, "gl");
    assert_eq!(titles(&ui), vec!["Far change 20".to_string()]);
}

#[test]
fn the_choice_is_remembered_and_a_removed_profile_falls_back_to_all() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = two_profiles(dir.path());
    let cfg = dir.path().join("cfg");
    ui.picked(Hit::Profile, "gl");
    let saved = RadarState::load(&RadarState::file_in(&cfg));
    assert_eq!(saved.profile.as_deref(), Some("gl"));
    let again = RadarUi::with_parts("s", Some(cfg.clone()), None, clock);
    assert_eq!(again.profile, "gl");
    // The profile is deleted in Settings: the next open lists all of them.
    write_profiles(&cfg, &[("gh", IntegrationKind::Github, "personal")]);
    let again = RadarUi::with_parts("s", Some(cfg.clone()), None, clock);
    assert_eq!(again.profile, "");
    // And one removed while the Radar is open falls back when it is next used.
    ui.open_combo(Hit::Profile);
    assert_eq!(ui.profile, "");
}

#[test]
fn an_empty_list_says_the_profile_filter_hides_rows() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = two_profiles(dir.path());
    write_profiles(
        &dir.path().join("cfg"),
        &[
            ("gh", IntegrationKind::Github, "personal"),
            ("gl", IntegrationKind::GitlabSelfManaged, "work"),
            ("gt", IntegrationKind::Gitea, "empty"),
        ],
    );
    ui.fill_combos();
    ui.picked(Hit::Profile, "gt");
    let text = screen(&mut ui, 100, 20);
    assert!(text.contains("for this profile"), "{text}");
}

#[test]
fn the_profile_combobox_shrinks_in_a_narrow_window_and_never_vanishes() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = two_profiles(dir.path());
    for width in [120u16, 80, 60, 40, 30] {
        let _ = screen(&mut ui, width, 12);
        assert!(
            ui.hits
                .iter()
                .any(|(r, h)| *h == Hit::Profile && r.right() <= width && r.width >= 3),
            "width {width}: {:?}",
            ui.hits
        );
        for which in [Hit::Scope, Hit::View, Hit::Who] {
            assert!(ui.hits.iter().any(|(_, h)| *h == which), "width {width}");
        }
    }
    let wide = screen(&mut ui, 120, 12);
    assert!(wide.lines().nth(2).unwrap().contains("Profile"), "{wide}");
}

#[test]
fn the_badge_order_puts_profile_after_show() {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = two_profiles(dir.path());
    let _ = screen(&mut ui, 120, 20);
    let spots: Vec<Spot> = ui.nav_targets().into_iter().map(|(_, s)| s).collect();
    let who = spots.iter().position(|s| *s == Spot::Who).unwrap();
    assert_eq!(spots[who + 1], Spot::Profile);
    ui.nav_focus(Spot::Profile);
    assert!(ui.profile_combo.is_open());
}
