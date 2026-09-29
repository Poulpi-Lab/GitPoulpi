use super::*;

#[test]
fn views_map_to_filters_and_cycle_through_every_choice() {
    assert_eq!(PrView::Open.filter().state, StateFilter::Open);
    assert_eq!(PrView::Closed.filter().state, StateFilter::Closed);
    assert_eq!(PrView::All.filter().state, StateFilter::All);
    assert!(IssueView::Assigned.filter().assigned_to_me);
    assert_eq!(IssueView::Closed.filter().state, StateFilter::Closed);
    // Every choice has its own label: the dropdown lists them all.
    let labels: std::collections::HashSet<&str> = PrView::ALL.iter().map(|v| v.label()).collect();
    assert_eq!(labels.len(), PrView::ALL.len());
    let labels: std::collections::HashSet<&str> =
        IssueView::ALL.iter().map(|v| v.label()).collect();
    assert_eq!(labels.len(), IssueView::ALL.len());
    // A cache written before the groups existed still loads.
    assert_eq!(
        serde_json::from_str::<PrView>("\"review_requested\"").unwrap(),
        PrView::Open
    );
}

#[test]
fn the_policy_refreshes_on_open_head_change_remote_move_interval_and_demand() {
    let t0 = Instant::now();
    let mut p = RefreshPolicy::new(Some(Duration::from_secs(300)));
    let observed = p.observe(Some("main"), 1);
    assert_eq!(p.should_send(t0, observed), Some(Reason::Opened));

    // Nothing changed, not due yet.
    let observed = p.observe(Some("main"), 1);
    assert_eq!(p.should_send(t0 + Duration::from_secs(10), observed), None);

    // HEAD moved to another branch.
    let observed = p.observe(Some("feature"), 1);
    assert_eq!(
        p.should_send(t0 + Duration::from_secs(20), observed),
        Some(Reason::HeadChanged)
    );

    // A push or fetch moved a remote-tracking ref.
    let observed = p.observe(Some("feature"), 2);
    assert_eq!(
        p.should_send(t0 + Duration::from_secs(40), observed),
        Some(Reason::RemotesChanged)
    );

    // The periodic timer.
    let observed = p.observe(Some("feature"), 2);
    assert_eq!(
        p.should_send(t0 + Duration::from_secs(400), observed),
        Some(Reason::Interval)
    );

    // On demand, even inside the debounce gap.
    p.demand();
    let observed = p.observe(Some("feature"), 2);
    assert_eq!(
        p.should_send(t0 + Duration::from_secs(401), observed),
        Some(Reason::Demand)
    );
}

#[test]
fn zero_minutes_turns_the_periodic_refresh_off_but_not_the_rest() {
    let t0 = Instant::now();
    let mut p = RefreshPolicy::new(None);
    let observed = p.observe(Some("main"), 1);
    assert_eq!(p.should_send(t0, observed), Some(Reason::Opened));
    let observed = p.observe(Some("main"), 1);
    assert_eq!(
        p.should_send(t0 + Duration::from_secs(86_400), observed),
        None
    );
    p.demand();
    let observed = p.observe(Some("main"), 1);
    assert_eq!(
        p.should_send(t0 + Duration::from_secs(86_401), observed),
        Some(Reason::Demand)
    );
}

#[test]
fn a_burst_of_ref_changes_is_debounced_then_retried() {
    let t0 = Instant::now();
    let mut p = RefreshPolicy::new(None);
    let observed = p.observe(Some("main"), 1);
    assert!(p.should_send(t0, observed).is_some());
    // Changed again a moment later: held back...
    let observed = p.observe(Some("main"), 2);
    assert_eq!(
        p.should_send(t0 + Duration::from_millis(300), observed),
        None
    );
    // ...but noticed again on the next observation after the gap.
    let observed = p.observe(Some("main"), 2);
    assert_eq!(
        p.should_send(t0 + Duration::from_secs(3), observed),
        Some(Reason::RemotesChanged)
    );
}

#[test]
fn the_signature_follows_head_and_remote_tracking_refs_only() {
    use crate::test_support::{local_branch_at, oid, remote_branch_at};
    let local = |name: &str, head: bool, n: u8| {
        let mut r = local_branch_at(name, oid(n));
        r.is_head = head;
        r
    };
    let remote = |name: &str, n: u8| remote_branch_at("origin", name, oid(n));
    let a = refs_signature(&[local("main", true, 1), remote("main", 5)]);
    // A local commit (tip moved) is not a hosting-relevant change.
    let b = refs_signature(&[local("main", true, 2), remote("main", 5)]);
    assert_eq!(a, b);
    // A push (remote-tracking ref moved) is.
    let c = refs_signature(&[local("main", true, 1), remote("main", 6)]);
    assert_ne!(a, c);
    // So is checking out another branch.
    let d = refs_signature(&[
        local("main", false, 1),
        local("dev", true, 1),
        remote("main", 5),
    ]);
    assert_ne!(a, d);
}

#[test]
fn ages_read_naturally() {
    assert_eq!(age_label(100, 100), "just now");
    assert_eq!(age_label(400, 100), "5m");
    assert_eq!(age_label(100 + 3 * 3600, 100), "3h");
    assert_eq!(age_label(100 + 2 * 86_400, 100), "2d");
    assert_eq!(age_label(100, 500), "just now");
}
