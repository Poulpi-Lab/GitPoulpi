//! Account-wide rows: pure, no terminal and no network.

use super::*;
use crate::hosting_task::{IssuesTarget, ProfileRef};
use crate::radar::model::{Board, BoardInput};
use crate::radar::worker::RepoAnswer;
use gitpoulpi_hosting::account::{AccountIssue, AccountPr};
use gitpoulpi_hosting::{IssueSummary, PullRequestSummary};
use gitpoulpi_session::RadarState;

fn pr(n: u64, author: &str, state: PullRequestState) -> PullRequestSummary {
    PullRequestSummary {
        number: n,
        iid_label: format!("#{n}"),
        title: format!("Change {n}"),
        state,
        source_branch: format!("feat-{n}"),
        target_branch: "main".into(),
        source_project: None,
        author: author.into(),
        web_url: format!("https://github.com/acme/app/pull/{n}"),
        updated_at: format!("2026-10-{:02}T10:00:00Z", 10 + n),
        review_requested: true,
        assignees: vec![],
        reviewers: vec![],
        checks: None,
        has_conflicts: None,
    }
}

fn row(project: &str, pr: PullRequestSummary) -> AccountPr {
    AccountPr {
        project: project.into(),
        clone_url: format!("https://github.com/{project}.git"),
        pr,
    }
}

fn answer(prs: Vec<AccountPr>, issues: Vec<AccountIssue>) -> AccountAnswer {
    AccountAnswer {
        round: 1,
        id: "gh".into(),
        label: "GitHub".into(),
        kind: IntegrationKind::Github,
        host: "github.com".into(),
        me: "kai".into(),
        items: AccountItems {
            prs,
            issues,
            truncated: false,
            notes: Vec::new(),
        },
        fetched_at: 1_000,
        cached: false,
        problem: None,
        paused: None,
    }
}

/// A scope answer listing request `n` of `acme/app` cloned at `/repos/app`.
fn scope_answer(project: &str, n: u64) -> RepoAnswer {
    let mut snap = crate::radar::worker::empty_snapshot();
    snap.profile_id = Some("gh".into());
    snap.kind = Some(IntegrationKind::Github);
    snap.can_pulls = true;
    snap.project = project.into();
    snap.prs = vec![pr(n, "ann", PullRequestState::Open)];
    snap.issues_target = Some(IssuesTarget {
        profile: ProfileRef {
            id: "gh".into(),
            label: "GitHub".into(),
            kind: IntegrationKind::Github,
        },
        project: project.into(),
        chosen: true,
    });
    RepoAnswer {
        path: PathBuf::from("/repos/app"),
        name: "app".into(),
        host: "github.com".into(),
        me: "kai".into(),
        snapshot: snap,
        cached: false,
        round: 1,
        paused: None,
    }
}

fn build(
    scope: &[RepoAnswer],
    accounts: &[AccountAnswer],
    clones: &CloneIndex,
    state: &RadarState,
) -> Board {
    Board::build(&BoardInput {
        answers: scope,
        accounts,
        clones,
        wips: &[],
        state,
        now: 0,
    })
}

fn titles(board: &Board) -> Vec<String> {
    let mut t: Vec<String> = board.items.iter().map(|i| i.title.clone()).collect();
    t.sort();
    t
}

#[test]
fn the_scope_row_wins_over_the_account_row() {
    let scope = [scope_answer("acme/app", 1)];
    // The same request as the host spells it (another case), and a second one.
    let accounts = [answer(
        vec![
            row("Acme/App", pr(1, "ann", PullRequestState::Open)),
            row("Acme/App", pr(2, "ann", PullRequestState::Open)),
        ],
        vec![],
    )];
    let board = build(
        &scope,
        &accounts,
        &CloneIndex::default(),
        &RadarState::default(),
    );
    assert_eq!(board.items.len(), 2, "{:?}", titles(&board));
    let first = board.items.iter().find(|i| i.number == 1).unwrap();
    assert_eq!(first.repo.as_deref(), Some(Path::new("/repos/app")));
    assert!(
        first.integration.is_empty(),
        "the scope's row is the one kept"
    );
    let second = board.items.iter().find(|i| i.number == 2).unwrap();
    assert_eq!(second.integration, "gh");
}

#[test]
fn a_known_clone_gives_its_path_and_other_rows_are_not_cloned() {
    let accounts = [answer(
        vec![
            row("acme/tool", pr(3, "bob", PullRequestState::Open)),
            row("acme/other", pr(4, "bob", PullRequestState::Open)),
        ],
        vec![],
    )];
    let clones = CloneIndex::of(&[("github.com", "ACME/Tool", Path::new("/work/tool"))]);
    let board = build(&[], &accounts, &clones, &RadarState::default());
    let tool = board.items.iter().find(|i| i.number == 3).unwrap();
    assert_eq!(tool.repo.as_deref(), Some(Path::new("/work/tool")));
    assert_eq!(tool.repo_name, "tool");
    let other = board.items.iter().find(|i| i.number == 4).unwrap();
    assert_eq!(other.repo, None);
    assert_eq!(other.repo_name, "acme/other");
    assert_eq!(other.project, "acme/other");
    assert_eq!(other.clone_url, "https://github.com/acme/other.git");
    assert_eq!(other.group, Group::NeedsReview);
}

#[test]
fn hiding_not_cloned_keeps_the_known_clones() {
    let accounts = [answer(
        vec![
            row("acme/tool", pr(3, "bob", PullRequestState::Open)),
            row("acme/other", pr(4, "bob", PullRequestState::Open)),
        ],
        vec![],
    )];
    let clones = CloneIndex::of(&[("github.com", "acme/tool", Path::new("/work/tool"))]);
    let state = RadarState {
        show_not_cloned: false,
        ..RadarState::default()
    };
    let board = build(&[], &accounts, &clones, &state);
    assert_eq!(titles(&board), vec!["Change 3".to_string()]);
}

#[test]
fn drafts_of_the_account_rows_are_hidden_until_asked() {
    let accounts = [answer(
        vec![
            row("acme/a", pr(1, "bob", PullRequestState::Open)),
            row("acme/a", pr(2, "bob", PullRequestState::Draft)),
        ],
        vec![],
    )];
    let hidden = build(
        &[],
        &accounts,
        &CloneIndex::default(),
        &RadarState::default(),
    );
    assert_eq!(titles(&hidden), vec!["Change 1".to_string()]);
    let state = RadarState {
        show_drafts: true,
        ..RadarState::default()
    };
    let shown = build(&[], &accounts, &CloneIndex::default(), &state);
    let draft = shown.items.iter().find(|i| i.number == 2).unwrap();
    assert!(draft.draft);
}

#[test]
fn my_own_request_is_mine_and_an_issue_is_assigned() {
    let mut own = pr(5, "kai", PullRequestState::Open);
    own.review_requested = false;
    let issue = AccountIssue {
        project: "acme/a".into(),
        clone_url: String::new(),
        issue: IssueSummary {
            number: 9,
            iid_label: "#9".into(),
            reference: "acme/a#9".into(),
            title: "Bug".into(),
            ..IssueSummary::default()
        },
    };
    let accounts = [answer(vec![row("acme/a", own)], vec![issue])];
    let board = build(
        &[],
        &accounts,
        &CloneIndex::default(),
        &RadarState::default(),
    );
    let mine = board.items.iter().find(|i| i.number == 5).unwrap();
    assert_eq!(mine.group, Group::Mine);
    let bug = board.items.iter().find(|i| i.number == 9).unwrap();
    assert_eq!(
        (bug.group, bug.kind),
        (Group::IssueAssigned, ItemKind::Issue)
    );
    assert_eq!(bug.label, "acme/a#9");
}

#[test]
fn a_pin_keeps_a_not_cloned_row_on_top() {
    let accounts = [answer(
        vec![row("acme/a", pr(1, "bob", PullRequestState::Open))],
        vec![],
    )];
    let mut state = RadarState::default();
    let key = ItemKey::new("github.com", "acme/a", KIND_PR, 1);
    state.toggle_pin(&key);
    let board = build(&[], &accounts, &CloneIndex::default(), &state);
    assert_eq!(board.pinned.len(), 1);
}

#[test]
fn clone_index_reads_every_remote_spelling() {
    let mut index = CloneIndex::default();
    index.add_remote("git@github.com:Acme/App.git", Path::new("/a"));
    index.add_remote(
        "https://gitlab.example.com:8443/g/sub/p.git",
        Path::new("/b"),
    );
    index.add_remote("ssh://git@host.example/team/x", Path::new("/c"));
    index.add_remote("not a url", Path::new("/d"));
    assert_eq!(
        index.find("github.com", "acme/app"),
        Some(&PathBuf::from("/a"))
    );
    // The integration's host may carry the port the remote does not.
    assert_eq!(
        index.find("gitlab.example.com:8443", "g/sub/p"),
        Some(&PathBuf::from("/b"))
    );
    assert_eq!(
        index.find("host.example", "team/x"),
        Some(&PathBuf::from("/c"))
    );
    assert_eq!(index.find("github.com", "acme/missing"), None);
    // The first clone of a project is kept.
    index.add_remote("https://github.com/acme/app", Path::new("/later"));
    assert_eq!(
        index.find("github.com", "acme/app"),
        Some(&PathBuf::from("/a"))
    );
}

#[test]
fn notes_say_what_failed_what_paused_and_what_was_cut() {
    let mut cut = answer(vec![], vec![]);
    cut.items.notes = vec!["12 more pull/merge requests are not listed (limit 300)".into()];
    let mut rejected = answer(vec![], vec![]);
    rejected.label = "GitLab".into();
    rejected.id = "gl".into();
    rejected.problem = Some(Problem::Reconnect("401".into()));
    let mut paused = answer(vec![], vec![]);
    paused.label = "Gitea".into();
    paused.paused = Some("rate limited".into());
    let notes = notes(&[cut, rejected, paused], 2_000);
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].name, "GitHub");
    assert!(notes[0].text.contains("12 more"));
    assert!(notes[0].act.is_none());
    assert_eq!(notes[1].act, Some((Act::Reconnect, Some("gl".to_string()))));
    assert!(notes[2].text.contains("rate limited"));
    assert!(notes.iter().all(|n| n.path.as_os_str().is_empty()));
}
