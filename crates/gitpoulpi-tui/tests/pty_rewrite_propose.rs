//! An agent's rewrite proposal (`ui propose rewrite`) end to end: the real
//! binary and its embedded daemon in a pty, and a plain IPC client standing
//! in for the command (`common::ui_show`). The proposal opens the history
//! workbench with the recipe in it, marked "Proposed by"; nothing runs until
//! the user previews and applies it themselves.
//!
//! Every wait names the state it waits for.

mod common;

use std::time::{Duration, Instant};

use gitpoulpi_core::domain::rewrite::{
    IdentityMatch, IdentityRule, IdentitySet, Recipe, Rule, RuleKind, Scope, SignaturePolicy, Who,
};
use gitpoulpi_core::domain::RepoId;
use gitpoulpi_ipc::ui::{Proposal, ProposalStatus, UiIntent, UiOutcome};
use gitpoulpi_testkit::{FixtureRepo, FixtureRepoBuilder};

const STEP: Duration = Duration::from_secs(30);

fn fixture() -> FixtureRepo {
    FixtureRepoBuilder::new()
        .unwrap()
        .commit("base")
        .unwrap()
        .commit("middle")
        .unwrap()
        .commit("tip on top")
        .unwrap()
        .build()
}

fn authors(repo: &FixtureRepo) -> Vec<String> {
    repo.git(&["log", "--format=%ae"])
        .lines()
        .map(str::to_string)
        .collect()
}

fn recipe(from: &str) -> Recipe {
    let mut recipe = Recipe::new(
        "from the agent",
        Scope::default(),
        vec![Rule::new(RuleKind::Identity(IdentityRule {
            when: IdentityMatch::email(from),
            set: IdentitySet {
                name: None,
                email: Some("agent.picked@example.com".into()),
            },
            apply_to: Who::Author,
            include_taggers: false,
            map_trailers: false,
        }))],
    );
    recipe.options.signatures = Some(SignaturePolicy::Strip);
    recipe
}

fn status_of(dir: &std::path::Path, session: &str, id: &str) -> Option<ProposalStatus> {
    common::ui_state(dir, session)
        .proposals
        .into_iter()
        .find(|p| p.id == id)
        .map(|p| p.status)
}

fn wait_status(dir: &std::path::Path, session: &str, id: &str, want: ProposalStatus) {
    let deadline = Instant::now() + STEP;
    loop {
        let got = status_of(dir, session, id);
        if got == Some(want) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "{id}: wanted {want:?}, got {got:?}"
        );
        std::thread::sleep(Duration::from_millis(150));
    }
}

#[test]
fn a_rewrite_proposal_opens_the_workbench_and_only_the_users_apply_rewrites() {
    let repo = fixture();
    let from = authors(&repo)[0].clone();
    let session = "pty-propose-rewrite";
    let (mut term, config) = common::launch(repo.root())
        .session(session)
        .size(200, 50)
        .no_exit_pause()
        .ready("tip on top")
        .start();
    common::wait_for_viewer(config.path(), session);
    // Past the "the user just pressed a key" deferral window.
    std::thread::sleep(Duration::from_millis(1700));
    let dir = config.path();
    let proposal = Proposal::Rewrite {
        recipe: Box::new(recipe(&from)),
    };
    let send = |id: &str| {
        common::ui_show(
            dir,
            session,
            UiIntent::Propose {
                repo: RepoId::from_path(repo.root()),
                id: id.into(),
                proposal: proposal.clone(),
            },
        )
    };
    let before = authors(&repo);
    let head = repo.git(&["rev-parse", "HEAD"]);

    // Opened with the recipe in it, with the banner. Waiting changes
    // nothing; Esc closes it and the proposal is rejected.
    assert!(matches!(send("p-r1"), UiOutcome::Proposed { .. }));
    term.wait_for("the workbench, filled and marked", STEP, |s| {
        let t = s.text();
        t.contains("Proposed by claude-code") && t.contains("Replace an author")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(status_of(dir, session, "p-r1"), Some(ProposalStatus::Open));
    std::thread::sleep(Duration::from_millis(1500));
    assert_eq!(authors(&repo), before, "opening rewrites nothing");
    assert_eq!(repo.git(&["rev-parse", "HEAD"]), head);
    term.send_keys("Esc").unwrap();
    term.wait_for("the workbench closed", STEP, |s| {
        !s.text().contains("Proposed by claude-code")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    wait_status(dir, session, "p-r1", ProposalStatus::Rejected);
    assert_eq!(authors(&repo), before, "nothing was rewritten");

    // Again: the user previews, then applies with the workbench's own key.
    std::thread::sleep(Duration::from_millis(1700));
    assert!(matches!(send("p-r2"), UiOutcome::Proposed { .. }));
    term.wait_for("the workbench again", STEP, |s| {
        s.text().contains("Proposed by claude-code")
    })
    .unwrap_or_else(|e| panic!("{e}"));
    term.send_keys("F5").unwrap();
    term.wait_for("the preview", STEP, |s| s.text().contains("3 of 3"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(authors(&repo), before, "a preview rewrites nothing");
    term.send_keys("C-s").unwrap();
    let deadline = Instant::now() + STEP;
    while authors(&repo).contains(&from) {
        assert!(
            Instant::now() < deadline,
            "authors are {:?}, wanted none of {from}",
            authors(&repo)
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(authors(&repo)
        .iter()
        .all(|a| a == "agent.picked@example.com"));
    wait_status(dir, session, "p-r2", ProposalStatus::Accepted);
}
