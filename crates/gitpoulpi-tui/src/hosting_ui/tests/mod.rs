use super::*;
use crate::hosting_task::fake::{issue, pr, FakeFactory};
use crate::hosting_task::{IssuesTarget, ProfileRef};
use crate::test_support::{local_branch_at, oid, remote_branch_at};
use gitpoulpi_config::IntegrationKind;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

mod actions;
mod chip;
mod i18n;
mod picker;
mod providers;
mod rows;
mod sync;

pub(super) fn profiles() -> Vec<Integration> {
    vec![Integration::new(
        "corp",
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example.com",
    )]
}

pub(super) fn snapshot(prs: Vec<PullRequestSummary>, issues: Vec<IssueSummary>) -> HostingSnapshot {
    HostingSnapshot {
        prs_mentioned: Vec::new(),
        issues_mentioned: Vec::new(),
        profile_id: Some("corp".into()),
        profile_label: "corp".into(),
        kind: Some(IntegrationKind::GitlabSelfManaged),
        remote: "origin".into(),
        project: "g/p".into(),
        can_pulls: true,
        can_issues: true,
        head_pr: prs.first().cloned(),
        prs,
        issues,
        head_branch: Some("feature".into()),
        pr_view: PrView::Open,
        issue_view: IssueView::Open,
        me: "kai".into(),
        issue_profiles: vec![ProfileRef {
            id: "corp".into(),
            label: "corp".into(),
            kind: IntegrationKind::GitlabSelfManaged,
        }],
        issues_target: Some(IssuesTarget {
            profile: ProfileRef {
                id: "corp".into(),
                label: "corp".into(),
                kind: IntegrationKind::GitlabSelfManaged,
            },
            project: "g/p".into(),
            chosen: false,
        }),
        pull_profiles: vec![ProfileRef {
            id: "corp".into(),
            label: "corp".into(),
            kind: IntegrationKind::GitlabSelfManaged,
        }],
        pulls_target: Some(IssuesTarget {
            profile: ProfileRef {
                id: "corp".into(),
                label: "corp".into(),
                kind: IntegrationKind::GitlabSelfManaged,
            },
            project: "g/p".into(),
            chosen: true,
        }),
        suggestions: Vec::new(),
        issues_problem: None,
        ambiguous: false,
        fetched_at: 1_000,
        problem: None,
    }
}

pub(super) fn ui_with(snap: HostingSnapshot) -> (HostingUi, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let mut ui = HostingUi::with_parts(dir.path(), None, None, profiles, None);
    ui.inputs = Some(Inputs {
        remotes: vec![ResolveRemote::new(
            "origin",
            "git@gitlab.example.com:g/p.git",
        )],
        head_branch: Some("feature".into()),
        head_pushed: true,
    });
    ui.profiles = vec![
        ("corp".into(), "corp".into()),
        ("other".into(), "other".into()),
    ];
    ui.set_clock(|| 1_000);
    ui.apply(snap);
    (ui, dir)
}

pub(super) fn open_pr(n: u64) -> PullRequestSummary {
    pr(n, "feature", PullRequestState::Open)
}

pub(super) fn draw_section(
    ui: &HostingUi,
    list: HostingList,
    w: u16,
    h: u16,
    hovered: Option<HitTarget>,
) -> (String, Vec<(Rect, HitTarget)>, Vec<String>) {
    let mut hits = Vec::new();
    let mut states = SidebarStates::default();
    let mut terminal = Terminal::new(TestBackend::new(w, h)).unwrap();
    terminal
        .draw(|f| {
            ui.render_section(
                f,
                Rect::new(0, 0, w, h),
                list,
                true,
                true,
                hovered.as_ref(),
                &mut states,
                &mut hits,
            );
        })
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let rows: Vec<String> = (0..h)
        .map(|y| (0..w).map(|x| buf[(x, y)].symbol()).collect::<String>())
        .collect();
    (rows.join("\n"), hits, rows)
}

pub(super) fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// The first pull-request row.
pub(super) fn first_pull(ui: &HostingUi) -> usize {
    ui.rows(HostingList::Pulls)
        .iter()
        .position(|r| matches!(r, HostingRow::Pull(..)))
        .expect("a pull request row")
}

/// The index of the first row of a kind.
pub(super) fn row_of(ui: &HostingUi, list: HostingList, want: HostingRow) -> usize {
    ui.rows(list)
        .iter()
        .position(|r| *r == want)
        .unwrap_or_else(|| panic!("{want:?} not in {:?}", ui.rows(list)))
}
