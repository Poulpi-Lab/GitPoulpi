//! The toolbar chip beside the branch.

use super::*;

#[test]
fn the_linked_chip_names_the_provider_and_follows_the_state() {
    crate::live_settings::HIDE_FINISHED_PR_CHIP_TEST.with(|c| c.set(Some(false)));
    let (ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![]));
    let chip = ui.chip().unwrap();
    assert_eq!(chip.text, "GL !53");
    assert_eq!(chip.bg, Some(th().refs.local));
    assert_eq!(chip.hit, HostingHit::LinkedPrChip);

    let mut merged = snapshot(vec![pr(9, "feature", PullRequestState::Merged)], vec![]);
    merged.kind = Some(IntegrationKind::Github);
    let (ui, _d) = ui_with(merged);
    let chip = ui.chip().unwrap();
    assert_eq!(chip.text, "GH !9");
    assert_eq!(chip.bg, Some(th().hosting.merged));

    let tip = ui.chip_tooltip().join("\n");
    assert!(tip.contains("Change number 9"), "{tip}");
    assert!(tip.contains("merged"), "{tip}");
    assert!(tip.contains("feature \u{2192} main"), "{tip}");
}

#[test]
fn a_pushed_branch_without_a_pull_request_offers_the_dim_new_pr_chip() {
    let mut snap = snapshot(vec![], vec![]);
    snap.head_pr = None;
    let (mut ui, _d) = ui_with(snap);
    let chip = ui.chip().unwrap();
    assert_eq!(chip.text, "+ PR");
    assert_eq!(chip.bg, None);
    assert_eq!(chip.hit, HostingHit::NewPrChip);
    let mut clip = crate::clipboard::Clipboard::new();
    assert_eq!(
        ui.click_chip(&HostingHit::NewPrChip, &mut clip),
        HostingEffect::StartPullRequest("feature".into())
    );
    // Not pushed: silent.
    ui.inputs.as_mut().unwrap().head_pushed = false;
    assert!(ui.chip().is_none());
    // A failing host: silent too (we do not know there is none).
    ui.inputs.as_mut().unwrap().head_pushed = true;
    let mut failing = snapshot(vec![], vec![]);
    failing.head_pr = None;
    failing.problem = Some(Problem::Offline("x".into()));
    ui.apply(failing);
    assert!(ui.chip().is_none());
}

#[test]
fn no_integration_means_no_chip_and_no_sections() {
    let dir = tempfile::tempdir().unwrap();
    let ui = HostingUi::with_parts(dir.path(), None, None, profiles, None);
    assert!(ui.chip().is_none());
    assert_eq!(ui.sections_visible(), (false, false));
}

#[test]
fn a_chip_from_another_branch_is_hidden_until_the_refresh_lands() {
    let (mut ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![]));
    assert!(ui.chip().is_some());
    ui.inputs.as_mut().unwrap().head_branch = Some("other".into());
    assert!(ui.chip().is_none());
}

#[test]
fn the_chip_decision_by_state_and_setting() {
    use crate::hosting_ui::chip_visible;
    for (state, hidden_when_on) in [
        (PullRequestState::Open, false),
        (PullRequestState::Draft, false),
        (PullRequestState::Closed, true),
        (PullRequestState::Merged, true),
    ] {
        assert_eq!(chip_visible(state, true), !hidden_when_on, "{state:?} on");
        assert!(chip_visible(state, false), "{state:?} off");
    }
}

#[test]
fn a_finished_request_hides_its_chip_and_offers_a_new_one() {
    let set = |v| crate::live_settings::HIDE_FINISHED_PR_CHIP_TEST.with(|c| c.set(Some(v)));
    for state in [PullRequestState::Merged, PullRequestState::Closed] {
        let (ui, _d) = ui_with(snapshot(vec![pr(9, "feature", state)], vec![]));
        set(true);
        let chip = ui.chip().unwrap();
        assert_eq!(chip.hit, HostingHit::NewPrChip, "{state:?}");
        assert!(chip.text.starts_with("+ "), "{}", chip.text);
        assert!(
            ui.chip_tooltip().join("\n").contains("No "),
            "{:?}",
            ui.chip_tooltip()
        );
        set(false);
        assert_eq!(ui.chip().unwrap().hit, HostingHit::LinkedPrChip);
    }
    set(true);
    let (ui, _d) = ui_with(snapshot(vec![open_pr(53)], vec![]));
    assert_eq!(ui.chip().unwrap().hit, HostingHit::LinkedPrChip);
}
