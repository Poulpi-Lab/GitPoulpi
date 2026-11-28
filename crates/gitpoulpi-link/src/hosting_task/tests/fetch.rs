use super::*;

#[test]
fn a_matching_profile_yields_lists_and_the_head_pr() {
    let mut factory = FakeFactory::new();
    factory.prs = vec![pr(53, "feature", PullRequestState::Open)];
    factory.issues = vec![issue(12)];
    factory.head = Some(pr(53, "feature", PullRequestState::Open));
    let snap = fetch_snapshot(
        &request("git@gitlab.example.com:g/p.git"),
        &factory,
        &runtime(),
        1000,
    );
    assert_eq!(snap.profile_id.as_deref(), Some("corp"));
    assert_eq!(snap.project, "g/p");
    assert_eq!(snap.prs.len(), 1);
    assert_eq!(snap.issues.len(), 1);
    assert_eq!(snap.head_pr.as_ref().unwrap().number, 53);
    assert!(snap.problem.is_none());
    assert!(snap.can_pulls && snap.can_issues);
    let seen = factory.seen.lock().unwrap().clone();
    assert!(seen.iter().any(|s| s.starts_with("head g/p feature")));
}

#[test]
fn an_unlinked_repository_is_silent_and_only_suggests() {
    let factory = FakeFactory::new();
    // The remote matches the `corp` profile: that is a proposal only.
    let snap = fetch_snapshot(
        &unlinked_request("https://gitlab.example.com/g/p.git"),
        &factory,
        &runtime(),
        1,
    );
    assert!(!snap.linked());
    assert!(snap.pulls_target.is_none() && snap.issues_target.is_none());
    assert!(!snap.can_pulls && snap.problem.is_none());
    assert!(factory.seen.lock().unwrap().is_empty(), "nothing fetched");
    assert_eq!(snap.suggestions.len(), 1);
    assert_eq!(snap.suggestions[0].profile.id, "corp");
    assert_eq!(snap.suggestions[0].project, "g/p");
    assert_eq!(snap.suggestions[0].remote, "origin");
    assert_eq!(snap.pull_profiles.len(), 1);
}

#[test]
fn a_repository_no_profile_matches_has_no_suggestion() {
    let factory = FakeFactory::new();
    let snap = fetch_snapshot(
        &unlinked_request("https://github.com/o/r.git"),
        &factory,
        &runtime(),
        1,
    );
    assert!(snap.suggestions.is_empty() && !snap.linked());
}

#[test]
fn a_missing_token_is_a_reconnect_problem_not_a_crash() {
    let mut factory = FakeFactory::new();
    factory.no_token = true;
    let snap = fetch_snapshot(
        &request("https://gitlab.example.com/g/p.git"),
        &factory,
        &runtime(),
        1,
    );
    assert!(snap.linked());
    assert_eq!(snap.problem, Some(Problem::NoToken));
    assert!(snap.problem.as_ref().unwrap().wants_reconnect());
}

#[test]
fn errors_are_classified_and_the_other_list_survives() {
    let mut factory = FakeFactory::new();
    factory.issues = vec![issue(1)];
    factory.fail = Some(|| HostingError::Unauthorized {
        what: "prs".into(),
        detail: "401".into(),
    });
    let snap = fetch_snapshot(
        &request("https://gitlab.example.com/g/p.git"),
        &factory,
        &runtime(),
        1,
    );
    assert!(matches!(snap.problem, Some(Problem::Reconnect(_))));
    assert_eq!(snap.issues.len(), 1);

    factory.fail = Some(|| HostingError::Http("connection refused".into()));
    let snap = fetch_snapshot(
        &request("https://gitlab.example.com/g/p.git"),
        &factory,
        &runtime(),
        1,
    );
    assert!(matches!(snap.problem, Some(Problem::Offline(_))));
    assert!(!snap.problem.unwrap().wants_reconnect());
}

#[test]
fn the_worker_answers_over_its_channel_and_coalesces() {
    let mut factory = FakeFactory::new();
    factory.prs = vec![pr(7, "x", PullRequestState::Open)];
    let worker = HostingWorker::spawn(Arc::new(factory));
    worker.request(request("https://gitlab.example.com/g/p.git"));
    let deadline = Instant::now() + Duration::from_secs(10);
    let snap = loop {
        if let Some(snap) = worker.try_recv() {
            break snap;
        }
        assert!(Instant::now() < deadline, "no answer");
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(snap.prs[0].number, 7);
}
