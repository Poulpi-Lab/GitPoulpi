//! The source the dialog opens on (`repos.clone_start_source`).
use super::*;

fn idx(setting: &str, last: Option<&str>) -> usize {
    start_source_index(setting, last, &hosted())
}

#[test]
fn last_reopens_the_remembered_profile_and_url_ignores_it() {
    assert_eq!(idx("last", Some("hub")), 2);
    assert_eq!(idx("", Some("corp")), 1);
    assert_eq!(idx("last", None), 0);
    assert_eq!(idx("url", Some("hub")), 0);
}

#[test]
fn a_named_profile_wins_over_the_remembered_one() {
    assert_eq!(idx("corp", Some("hub")), 1);
    assert_eq!(idx("hub", None), 2);
}

#[test]
fn a_missing_or_invalid_profile_is_url() {
    assert_eq!(idx("last", Some("gone")), 0);
    assert_eq!(idx("gone", Some("hub")), 0);
    // Not among the sources (disabled / not connected / not a repo host).
    assert_eq!(start_source_index("jira", None, &hosted()), 0);
}

#[cfg(all(feature = "github", feature = "gitlab"))]
#[test]
fn the_dialog_opens_on_the_resolved_source_and_asks_for_its_list() {
    let mut dialog = with_hosted().with_start_source("last", Some("hub"));
    assert_eq!(dialog.source, 2);
    assert_eq!(dialog.take_source_change(), None, "opening is not a choice");
    assert_eq!(dialog.poll(), CloneOutcome::LoadProjects("hub".into()));
}

#[cfg(all(feature = "github", feature = "gitlab"))]
#[test]
fn a_source_the_caller_picks_is_not_overridden() {
    // The Radar's path picks with `with_target` and never applies the
    // start source, so its choice stands.
    let dialog = with_hosted().with_target("corp", "a/b", "note".into());
    assert_eq!(dialog.source, 1);
    let dialog = with_hosted().with_start_source("url", Some("hub"));
    assert_eq!(dialog.source, 0);
}

#[cfg(all(feature = "github", feature = "gitlab"))]
#[test]
fn moving_to_another_source_is_reported_once() {
    let mut dialog = with_hosted();
    dialog.focus = CloneFocus::Sources;
    dialog.on_key(key(KeyCode::Down));
    assert_eq!(dialog.take_source_change(), Some(Some("corp".into())));
    assert_eq!(dialog.take_source_change(), None);
    dialog.on_key(key(KeyCode::Up));
    assert_eq!(dialog.take_source_change(), Some(None));
}
