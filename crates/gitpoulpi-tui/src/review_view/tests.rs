//! The review view over a fake host: the data comes from memory, the diff
//! is put in by hand (no daemon), and everything else - keys, mouse, the
//! pending review on disk, the calls to the host - is the real code.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use gitpoulpi_core::domain::{DiffLine, FileDiff, FileStatus, Hunk, HunkId, LineOrigin, Oid};
use gitpoulpi_hosting::{
    ChecksState, DiffSide, HostedProject, HostedSshKey, HostingClient, Integration,
    IntegrationKind, MergeState, NewPullRequest, PullRequestDetail, PullRequestState,
    PullRequestSummary, ReviewCapabilities, ReviewComment, ReviewEvent, ReviewFile, ReviewOutcome,
    ReviewState, ReviewSubmission, ReviewThread, SubmittedReview,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::draw::Hit;
use super::popups::ComposeFor;
use super::*;
use crate::review_cmd::{ReviewCmd, Tab};

const AREA: Rect = Rect {
    x: 0,
    y: 0,
    width: 140,
    height: 44,
};

#[derive(Default)]
struct Calls(Mutex<Vec<String>>);

struct Fake {
    integration: Integration,
    caps: ReviewCapabilities,
    calls: Arc<Calls>,
    submitted: Arc<Mutex<Vec<ReviewSubmission>>>,
}

fn comment(id: &str, author: &str, body: &str, at: &str) -> ReviewComment {
    ReviewComment {
        id: id.into(),
        author: author.into(),
        body: body.into(),
        created_at: at.into(),
        in_reply_to: None,
        web_url: None,
    }
}

#[async_trait::async_trait]
impl HostingClient for Fake {
    fn integration(&self) -> &Integration {
        &self.integration
    }
    async fn list_projects(&self) -> gitpoulpi_hosting::Result<Vec<HostedProject>> {
        Ok(Vec::new())
    }
    async fn list_ssh_keys(&self) -> gitpoulpi_hosting::Result<Vec<HostedSshKey>> {
        Ok(Vec::new())
    }
    async fn upload_ssh_key(&self, _: &str, _: &str) -> gitpoulpi_hosting::Result<HostedSshKey> {
        Err(gitpoulpi_hosting::Error::Unsupported("upload".into()))
    }
    async fn create_pull_request(
        &self,
        _: &NewPullRequest,
    ) -> gitpoulpi_hosting::Result<gitpoulpi_hosting::CreatedPullRequest> {
        Err(gitpoulpi_hosting::Error::Unsupported("create".into()))
    }
    fn review_capabilities(&self) -> ReviewCapabilities {
        self.caps
    }
    async fn pull_request_detail(
        &self,
        _: &str,
        number: u64,
    ) -> gitpoulpi_hosting::Result<PullRequestDetail> {
        Ok(PullRequestDetail {
            patch_sets: Vec::new(),
            shown_patch_set: None,
            vote_labels: Vec::new(),
            can_submit: false,
            can_abandon: false,
            summary: PullRequestSummary {
                number,
                iid_label: format!("!{number}"),
                title: "Add moons".into(),
                state: PullRequestState::Open,
                source_branch: "feature/moons".into(),
                target_branch: "main".into(),
                source_project: None,
                author: "ines".into(),
                web_url: "https://gitlab.example.com/g/p/-/merge_requests/9".into(),
                updated_at: "2026-10-12T16:30:00Z".into(),
                review_requested: false,
                assignees: vec!["ines".into()],
                reviewers: vec!["mara".into()],
                checks: Some(ChecksState::Passing),
                has_conflicts: Some(false),
            },
            body: "Adds the **moons** column.\n\n- sorted\n- tested".into(),
            head_sha: "1".repeat(40),
            base_sha: "2".repeat(40),
            merge_state: MergeState::Clean,
            checks: Some(ChecksState::Passing),
            created_at: "2026-10-10T09:00:00Z".into(),
            // GitLab's request carries no line counts (the files list does).
            additions: self.caps.request_changes.then_some(22),
            deletions: self.caps.request_changes.then_some(3),
            changed_files: self.caps.request_changes.then_some(2),
            capabilities: self.caps,
        })
    }
    async fn pull_request_files(
        &self,
        _: &str,
        _: u64,
    ) -> gitpoulpi_hosting::Result<Vec<ReviewFile>> {
        Ok(vec![
            ReviewFile {
                path: "src/a.rs".into(),
                previous_path: None,
                status: gitpoulpi_hosting::FileStatus::Modified,
                additions: 2,
                deletions: 1,
            },
            ReviewFile {
                path: "docs/moons.md".into(),
                previous_path: None,
                status: gitpoulpi_hosting::FileStatus::Added,
                additions: 8,
                deletions: 0,
            },
        ])
    }
    async fn pull_request_threads(
        &self,
        _: &str,
        _: u64,
    ) -> gitpoulpi_hosting::Result<Vec<ReviewThread>> {
        Ok(vec![ReviewThread {
            id: "T1".into(),
            path: "src/a.rs".into(),
            line: Some(12),
            original_line: Some(12),
            side: DiffSide::New,
            start_line: None,
            resolved: false,
            outdated: false,
            resolvable: true,
            comments: vec![
                comment("1", "tomas", "Why a Vec here?", "2026-10-11T08:05:00Z"),
                comment(
                    "2",
                    "ines",
                    "Built once at start-up.",
                    "2026-10-11T09:05:00Z",
                ),
            ],
        }])
    }
    async fn pull_request_conversation(
        &self,
        _: &str,
        _: u64,
    ) -> gitpoulpi_hosting::Result<Vec<ReviewComment>> {
        Ok(vec![comment(
            "9",
            "tomas",
            "Looks promising.",
            "2026-10-11T08:00:00Z",
        )])
    }
    async fn pull_request_reviews(
        &self,
        _: &str,
        _: u64,
    ) -> gitpoulpi_hosting::Result<Vec<SubmittedReview>> {
        Ok(vec![
            SubmittedReview {
                id: "r1".into(),
                author: "mara".into(),
                state: ReviewState::ChangesRequested,
                body: "Please look at the inline note.".into(),
                submitted_at: "2026-10-12T10:00:00Z".into(),
            },
            // GitLab's approvals carry neither a text nor a date.
            SubmittedReview {
                id: "r2".into(),
                author: "nora".into(),
                state: ReviewState::Approved,
                body: String::new(),
                submitted_at: String::new(),
            },
        ])
    }
    async fn reply_to_thread(
        &self,
        _: &str,
        _: u64,
        thread_id: &str,
        body: &str,
    ) -> gitpoulpi_hosting::Result<ReviewComment> {
        self.calls
            .0
            .lock()
            .unwrap()
            .push(format!("reply {thread_id} {body}"));
        Ok(comment("3", "me", body, "2026-10-13T10:00:00Z"))
    }
    async fn resolve_thread(
        &self,
        _: &str,
        _: u64,
        thread_id: &str,
        resolved: bool,
    ) -> gitpoulpi_hosting::Result<()> {
        self.calls
            .0
            .lock()
            .unwrap()
            .push(format!("resolve {thread_id} {resolved}"));
        Ok(())
    }
    async fn add_comment(
        &self,
        _: &str,
        _: u64,
        body: &str,
    ) -> gitpoulpi_hosting::Result<ReviewComment> {
        self.calls.0.lock().unwrap().push(format!("comment {body}"));
        Ok(comment("4", "me", body, "2026-10-13T11:00:00Z"))
    }
    async fn pull_request_diff(
        &self,
        _: &str,
        _: u64,
        _: Option<u32>,
        _: Option<u32>,
    ) -> gitpoulpi_hosting::Result<Vec<gitpoulpi_hosting::review::ApiFileDiff>> {
        use gitpoulpi_hosting::review::ApiFileDiff;
        Ok(vec![
            ApiFileDiff {
                diff: a_rs(),
                previous_path: None,
                additions: 2,
                deletions: 1,
                omitted: None,
            },
            // The host left this file's patch out.
            ApiFileDiff {
                diff: FileDiff {
                    path: "docs/moons.md".into(),
                    status: FileStatus::Added,
                    hunks: Vec::new(),
                },
                previous_path: None,
                additions: 8,
                deletions: 0,
                omitted: Some(gitpoulpi_hosting::review::NOT_SHOWN_BY_THE_HOST.into()),
            },
        ])
    }
    async fn pull_request_commits(
        &self,
        _: &str,
        _: u64,
    ) -> gitpoulpi_hosting::Result<Vec<gitpoulpi_hosting::review::ApiCommit>> {
        use gitpoulpi_hosting::review::ApiCommit;
        Ok(vec![
            ApiCommit {
                sha: "3".repeat(40),
                parents: vec!["2".repeat(40)],
                author: "ines".into(),
                date: "2026-10-10T09:00:00Z".into(),
                summary: "Add moons".into(),
            },
            ApiCommit {
                sha: "4".repeat(40),
                parents: vec!["3".repeat(40)],
                author: "ines".into(),
                date: "2026-10-11T09:00:00Z".into(),
                summary: "Sort moons".into(),
            },
        ])
    }
    async fn commit_diff(
        &self,
        _: &str,
        sha: &str,
    ) -> gitpoulpi_hosting::Result<Vec<gitpoulpi_hosting::review::ApiFileDiff>> {
        self.calls
            .0
            .lock()
            .unwrap()
            .push(format!("commit_diff {sha}"));
        Ok(vec![gitpoulpi_hosting::review::ApiFileDiff {
            diff: a_rs(),
            previous_path: None,
            additions: 2,
            deletions: 1,
            omitted: None,
        }])
    }
    async fn file_content(
        &self,
        _: &str,
        path: &str,
        rev: Option<&str>,
    ) -> gitpoulpi_hosting::Result<Option<Vec<u8>>> {
        self.calls
            .0
            .lock()
            .unwrap()
            .push(format!("file_content {path} {}", rev.unwrap_or("-")));
        Ok(Some(a_rs_whole().join("\n").into_bytes()))
    }
    async fn submit_review(
        &self,
        _: &str,
        _: u64,
        review: &ReviewSubmission,
    ) -> gitpoulpi_hosting::Result<ReviewOutcome> {
        self.submitted.lock().unwrap().push(review.clone());
        Ok(ReviewOutcome {
            event: review.event,
            comments: review.comments.len(),
            review_id: None,
        })
    }
}

struct Factory {
    caps: ReviewCapabilities,
    calls: Arc<Calls>,
    submitted: Arc<Mutex<Vec<ReviewSubmission>>>,
}

impl ClientFactory for Factory {
    fn client(
        &self,
        integration: &Integration,
        _: &[Integration],
    ) -> gitpoulpi_hosting::Result<Box<dyn HostingClient>> {
        Ok(Box::new(Fake {
            integration: integration.clone(),
            caps: self.caps,
            calls: self.calls.clone(),
            submitted: self.submitted.clone(),
        }))
    }
}

const GITHUB_CAPS: ReviewCapabilities = ReviewCapabilities {
    review: true,
    approve: true,
    request_changes: true,
    line_ranges: true,
    resolve: true,
};

const GITLAB_CAPS: ReviewCapabilities = ReviewCapabilities {
    review: true,
    approve: true,
    request_changes: false,
    line_ranges: false,
    resolve: true,
};

struct Rig {
    view: ReviewView,
    calls: Arc<Calls>,
    submitted: Arc<Mutex<Vec<ReviewSubmission>>>,
    dir: tempfile::TempDir,
    caps: ReviewCapabilities,
    /// Keeps the daemon queue's receiver alive: nobody answers it.
    _queue: Option<crate::daemon_link::RawQueue>,
}

fn target(dir: &std::path::Path) -> ReviewTarget {
    let integration = Integration::new(
        "corp",
        IntegrationKind::GitlabSelfManaged,
        "https://gitlab.example.com",
    );
    ReviewTarget {
        all: vec![integration.clone()],
        integration,
        project: "g/p".into(),
        number: 9,
        iid_label: "!9".into(),
        title: "Add moons".into(),
        remote: "origin".into(),
        web_url: "https://gitlab.example.com/g/p/-/merge_requests/9".into(),
        source_branch: "feature/moons".into(),
        target_branch: "main".into(),
        patch_set: None,
        compare_patch_set: None,
        repo_root: Some(dir.to_path_buf()),
    }
}

fn open_in(dir: tempfile::TempDir, caps: ReviewCapabilities) -> Rig {
    open_with(dir, caps, false)
}

fn open_with(dir: tempfile::TempDir, caps: ReviewCapabilities, ai: bool) -> Rig {
    let calls = Arc::new(Calls::default());
    let submitted = Arc::new(Mutex::new(Vec::new()));
    let factory = Arc::new(Factory {
        caps,
        calls: calls.clone(),
        submitted: submitted.clone(),
    });
    let queue = crate::daemon_link::RawQueue::default();
    let requester = queue.requester();
    let mut view = ReviewView::open(target(dir.path()), factory, requester, ai, "s".into());
    // The toggles start from the user's last choices, a process-wide setting
    // other tests change: these tests start from the defaults.
    view.diff_opts.mode = crate::diff_view::DiffViewMode::Inline;
    view.diff_opts.hunks_only = true;
    view.diff_opts.show_whitespace = true;
    view.diff_opts.word_wrap = false;
    // The options dialog reads the user's files: not in a test.
    view.sug.fake_defaults = Some(options::Defaults {
        state_dir: "in_worktree".into(),
        ..Default::default()
    });
    wait(&mut view, "the host data", |v| v.host.is_some());
    Rig {
        view,
        calls,
        submitted,
        dir,
        caps,
        _queue: Some(queue),
    }
}

fn rig(caps: ReviewCapabilities) -> Rig {
    open_in(tempfile::tempdir().unwrap(), caps)
}

/// A review with AI on.
fn rig_ai(caps: ReviewCapabilities) -> Rig {
    open_with(tempfile::tempdir().unwrap(), caps, true)
}

fn wait(view: &mut ReviewView, what: &str, mut done: impl FnMut(&ReviewView) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        view.poll();
        if done(view) {
            return;
        }
        assert!(Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn line(origin: LineOrigin, text: &str) -> DiffLine {
    DiffLine {
        origin,
        content: format!("{text}\n"),
        staged: false,
    }
}

fn a_rs() -> FileDiff {
    FileDiff {
        path: "src/a.rs".into(),
        status: FileStatus::Modified,
        hunks: vec![Hunk {
            id: HunkId(0),
            old_start: 10,
            old_lines: 3,
            new_start: 10,
            new_lines: 4,
            lines: vec![
                line(LineOrigin::Context, "keep"),
                line(LineOrigin::Delete, "gone"),
                line(LineOrigin::Add, "added one"),
                line(LineOrigin::Add, "added two"),
                line(LineOrigin::Context, "tail"),
            ],
        }],
    }
}

impl Rig {
    /// The Files page with `src/a.rs`'s diff already in (no daemon here).
    fn on_files(&mut self) {
        self.view.git = Some(GitData {
            head: Oid::from_hex(&"1".repeat(40)).unwrap(),
            diff_base: Oid::from_hex(&"2".repeat(40)).unwrap(),
            comparing: false,
            commits: Vec::new(),
        });
        self.view.git_rx = None;
        self.view.diffs.insert("src/a.rs".into(), a_rs());
        self.key(alt('2'));
        self.screen();
    }

    fn key(&mut self, key: KeyEvent) -> ReviewEffect {
        self.view.on_key(key)
    }

    fn type_text(&mut self, text: &str) {
        for c in text.chars() {
            self.view.on_key(plain(KeyCode::Char(c)));
        }
    }

    fn screen(&mut self) -> String {
        let mut buf = Buffer::empty(AREA);
        self.view.render(AREA, &mut buf);
        crate::dialog_kit::buffer_text(&buf)
    }

    fn rect_of(&self, hit: Hit) -> Rect {
        self.view
            .hits
            .iter()
            .find(|(_, h)| *h == hit)
            .unwrap_or_else(|| panic!("no hit {hit:?}"))
            .0
    }

    fn click(&mut self, at: Rect) -> ReviewEffect {
        let (column, row) = (at.x + at.width / 2, at.y);
        self.view.on_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    fn row_with_in(&self, id: PaneId, text: &str) -> usize {
        let pane = match id {
            PaneId::Files => &self.view.files_pane,
            PaneId::Commits => &self.view.commits_pane,
        };
        (0..pane.len())
            .find(|i| pane.text(*i).contains(text))
            .unwrap_or_else(|| panic!("no row {text}"))
    }

    /// The first row of the Files pane showing `text`.
    fn row_with(&self, text: &str) -> usize {
        let pane = &self.view.files_pane;
        (0..pane.len())
            .find(|i| pane.text(*i).contains(text))
            .unwrap_or_else(|| panic!("no row {text}"))
    }

    /// Moves the diff cursor to the row showing `text`.
    fn cursor_on(&mut self, text: &str) {
        // Rows are rebuilt by the paint; a real key always follows one.
        self.screen();
        self.key(plain(KeyCode::Enter));
        let at = self.row_with(text);
        self.view.files_pane.jump(at);
    }
}

fn plain(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn alt(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn row_of(screen: &str, needle: &str) -> usize {
    screen
        .lines()
        .position(|l| l.contains(needle))
        .unwrap_or_else(|| panic!("no `{needle}` on:\n{screen}"))
}

#[test]
fn the_header_names_the_request_its_branches_checks_and_metadata() {
    let mut r = rig(GITHUB_CAPS);
    let s = r.screen();
    for want in [
        "!9 Add moons",
        "main \u{2190} feature/moons",
        "by ines",
        "checks passing",
        "no conflicts",
        "Assignees: ines",
        "Reviewers: mara",
        "Check out",
        "Finish review",
    ] {
        assert!(s.contains(want), "{want} missing on:\n{s}");
    }
}

#[test]
fn the_header_sums_the_size_from_the_files_when_the_host_gives_no_counts() {
    // GitLab: no additions / deletions on the request itself.
    let mut r = rig(GITLAB_CAPS);
    let s = r.screen();
    // src/a.rs +2 -1 and docs/moons.md +8 -0
    assert!(s.contains("+10 -1 in 2 files"), "size missing on:\n{s}");
    let mut r = rig(GITHUB_CAPS);
    assert!(r.screen().contains("+22 -3 in 2 files"));
}

#[test]
fn the_overview_lists_the_description_comments_and_submitted_reviews_with_their_verdict() {
    let mut r = rig(GITHUB_CAPS);
    let s = r.screen();
    for want in [
        "Description",
        "Adds the moons column.",
        "\u{2022} sorted",
        "\u{2022} tested",
        "Conversation (3)",
        "tomas \u{b7} 2026-10-11",
        "Looks promising.",
        "mara \u{b7} 2026-10-12",
        "requested changes",
        "Please look at the inline note.",
        "Write a comment... (posted at once)",
    ] {
        assert!(s.contains(want), "{want} missing on:\n{s}");
    }
    // Markdown is shown, not its markers.
    assert!(!s.contains("**moons**") && !s.contains("- sorted"), "{s}");
}

#[test]
fn a_review_without_a_date_shows_no_dangling_separator() {
    let mut r = rig(GITLAB_CAPS);
    let s = r.screen();
    let nora = s
        .lines()
        .find(|l| l.contains("nora"))
        .unwrap_or_else(|| panic!("no nora on:\n{s}"));
    assert!(nora.contains("approved"), "{nora}");
    assert!(
        !nora.contains("nora \u{b7}"),
        "no separator without a date: {nora}"
    );
}

#[test]
fn the_files_page_draws_the_thread_right_under_its_line_with_its_buttons() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    let s = r.screen();
    let code = row_of(&s, "added two");
    let lines: Vec<&str> = s.lines().collect();
    assert!(
        lines[code + 1].contains("tomas \u{b7} 2026-10-11"),
        "the thread head is the next row:\n{s}"
    );
    assert!(lines[code + 1].contains("Reply") && lines[code + 1].contains("Resolve"));
    assert!(lines[code + 2].contains("Why a Vec here?"));
    assert!(lines[code + 3].contains("ines \u{b7} 2026-10-11"));
    assert!(lines[code + 4].contains("Built once at start-up."));
    assert!(
        lines[code + 5].contains("tail"),
        "the code goes on after the thread:\n{s}"
    );
    // The file list beside it: both files, the open thread count.
    assert!(s.contains("src/a.rs") && s.contains("docs/moons.md"));
    assert!(s.contains("\u{25CF}1"), "one open thread on a.rs:\n{s}");
}

#[test]
fn a_resolved_thread_is_folded_to_one_row_and_a_key_unfolds_it() {
    let mut r = rig(GITHUB_CAPS);
    r.view.host.as_mut().unwrap().threads[0].resolved = true;
    r.view.collapsed.insert("T1".into());
    r.on_files();
    let s = r.screen();
    assert!(s.contains("tomas: Why a Vec here?"), "{s}");
    assert!(s.contains("resolved") && s.contains("Reopen"), "{s}");
    assert!(!s.contains("Built once at start-up."), "folded:\n{s}");
    r.cursor_on("tomas: Why a Vec here?");
    r.key(plain(KeyCode::Char('z')));
    let s = r.screen();
    assert!(s.contains("Built once at start-up."), "unfolded:\n{s}");
}

#[test]
fn an_outdated_thread_is_listed_under_the_diff() {
    let mut r = rig(GITHUB_CAPS);
    {
        let t = &mut r.view.host.as_mut().unwrap().threads[0];
        t.line = None;
        t.outdated = true;
    }
    r.on_files();
    let s = r.screen();
    let heading = row_of(&s, "Comments outside the lines shown");
    let lines: Vec<&str> = s.lines().collect();
    assert!(lines[heading + 1].contains("tomas") && lines[heading + 1].contains("outdated"));
    assert!(row_of(&s, "tail") < heading, "after the hunks:\n{s}");
}

#[test]
fn a_line_comment_is_pending_until_the_review_is_finished_and_survives_a_restart() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.cursor_on("added one");
    r.key(plain(KeyCode::Char('c')));
    let s = r.screen();
    assert!(
        s.contains("Comment on code") && s.contains("src/a.rs:11"),
        "{s}"
    );
    assert!(
        s.contains("added to your pending review"),
        "the box says it is held back:\n{s}"
    );
    r.type_text("rename this");
    r.key(ctrl('s'));
    assert_eq!(r.view.pending.len(), 1);
    let c = &r.view.pending[0];
    assert_eq!(
        (c.path.as_str(), c.line, c.side, c.body.as_str()),
        ("src/a.rs", 11, DiffSide::New, "rename this")
    );
    assert!(
        r.calls.0.lock().unwrap().is_empty() && r.view.call.is_none(),
        "nothing went to the host: {:?}",
        r.calls.0.lock().unwrap()
    );
    let s = r.screen();
    assert!(s.contains("1 pending comment"), "{s}");
    assert!(s.contains("Pending - line 11"), "{s}");
    assert!(s.contains("Finish review (1)"), "{s}");

    // Closed and reopened: the comment is still there.
    let Rig { view, dir, .. } = r;
    drop(view);
    let again = open_in(dir, GITHUB_CAPS);
    assert_eq!(again.view.pending.len(), 1);
    assert_eq!(again.view.pending[0].body, "rename this");
}

#[test]
fn a_context_line_comment_carries_the_other_number_for_gitlab() {
    let mut r = rig(GITLAB_CAPS);
    r.on_files();
    r.cursor_on("keep");
    r.key(plain(KeyCode::Char('c')));
    r.type_text("nit");
    r.key(ctrl('s'));
    let c = &r.view.pending[0];
    assert_eq!((c.line, c.other_line), (10, Some(10)));
}

#[test]
fn a_selection_makes_a_range_where_the_host_takes_ranges_and_not_on_gitlab() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.cursor_on("added one");
    r.key(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT));
    r.key(plain(KeyCode::Char('c')));
    r.type_text("both");
    r.key(ctrl('s'));
    let c = &r.view.pending[0];
    assert_eq!((c.start_line, c.line), (Some(11), 12));

    let mut r = rig(GITLAB_CAPS);
    r.on_files();
    r.cursor_on("added one");
    r.key(KeyEvent::new(KeyCode::Down, KeyModifiers::SHIFT));
    assert_eq!(r.view.files_pane.anchor, None, "one line per comment");
    let s = r.screen();
    assert!(
        s.contains("this host allows only one line per comment"),
        "{s}"
    );
}

#[test]
fn a_pending_comment_can_be_edited_and_deleted() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.cursor_on("added one");
    r.key(plain(KeyCode::Char('c')));
    r.type_text("first");
    r.key(ctrl('s'));
    r.cursor_on("added one");
    r.key(plain(KeyCode::Char('e')));
    r.type_text(" and more");
    r.key(ctrl('s'));
    assert_eq!(r.view.pending[0].body, "first and more");
    r.cursor_on("added one");
    r.key(plain(KeyCode::Char('d')));
    assert!(r.view.pending.is_empty());
    let draft = r.view.read_draft();
    assert!(draft.comments.is_empty(), "the file follows");
}

#[test]
fn viewed_ticks_are_kept_per_file_and_survive_a_restart() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    let tick = r.rect_of(Hit::Viewed(0));
    r.click(tick);
    assert!(r.view.viewed.contains("src/a.rs"));
    let s = r.screen();
    assert!(s.contains("1 viewed"), "{s}");
    let Rig { view, dir, .. } = r;
    drop(view);
    let again = open_in(dir, GITHUB_CAPS);
    assert!(again.view.viewed.contains("src/a.rs"));
}

#[test]
fn finish_review_offers_only_the_verdicts_the_host_allows() {
    let mut r = rig(GITLAB_CAPS);
    r.key(plain(KeyCode::Char('f')));
    let dialog = r.view.finish.as_ref().expect("the finish dialog");
    assert_eq!(
        dialog.offered(),
        vec![ReviewEvent::Comment, ReviewEvent::Approve]
    );
    let mut r = rig(GITHUB_CAPS);
    r.key(plain(KeyCode::Char('f')));
    assert_eq!(r.view.finish.as_ref().unwrap().offered().len(), 3);
}

#[test]
fn finishing_with_approve_sends_the_pending_comments_and_the_verdict_in_one_call() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.cursor_on("added one");
    r.key(plain(KeyCode::Char('c')));
    r.type_text("rename this");
    r.key(ctrl('s'));
    r.key(plain(KeyCode::Char('f')));
    let s = r.screen();
    assert!(s.contains("1 pending comment will be sent."), "{s}");
    // The verdict dropdown: open it, go down once (Approve), choose it.
    r.key(plain(KeyCode::Tab));
    r.key(plain(KeyCode::Enter));
    r.key(plain(KeyCode::Down));
    r.key(plain(KeyCode::Enter));
    r.type_text("");
    r.key(ctrl('s'));
    wait(&mut r.view, "the review sent", |v| v.call.is_none());
    let sent = r.submitted.lock().unwrap().clone();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].event, ReviewEvent::Approve);
    assert_eq!(sent[0].comments.len(), 1);
    assert_eq!(sent[0].comments[0].body, "rename this");
    assert_eq!(sent[0].head_sha.as_deref(), Some("1".repeat(40).as_str()));
    assert!(
        r.view.pending.is_empty(),
        "sent comments are no longer pending"
    );
    assert!(r.view.finish.is_none());
    assert!(r.view.read_draft().comments.is_empty());
}

#[test]
fn discarding_the_pending_review_asks_first() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.cursor_on("added one");
    r.key(plain(KeyCode::Char('c')));
    r.type_text("x");
    r.key(ctrl('s'));
    r.view.run(ReviewCmd::DiscardReview);
    let s = r.screen();
    assert!(s.contains("Discard 1 pending comment?"), "{s}");
    // Esc keeps it.
    r.key(plain(KeyCode::Esc));
    assert_eq!(r.view.pending.len(), 1);
    r.view.run(ReviewCmd::DiscardReview);
    let yes = r.view.confirm.as_ref().map(|_| ());
    assert!(yes.is_some());
    r.key(plain(KeyCode::Tab));
    r.key(plain(KeyCode::Enter));
    assert!(r.view.pending.is_empty());
}

#[test]
fn replying_resolving_and_commenting_go_to_the_host_at_once_and_say_so() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.cursor_on("Built once at start-up.");
    r.key(plain(KeyCode::Char('r')));
    let s = r.screen();
    assert!(
        s.contains("Reply to thread") && s.contains("posted at once"),
        "{s}"
    );
    r.type_text("thanks");
    r.key(ctrl('s'));
    wait(&mut r.view, "the reply", |v| v.call.is_none());
    assert_eq!(r.calls.0.lock().unwrap()[0], "reply T1 thanks");
    assert_eq!(r.view.host.as_ref().unwrap().threads[0].comments.len(), 3);

    r.cursor_on("Built once at start-up.");
    r.key(plain(KeyCode::Char('x')));
    wait(&mut r.view, "the resolve", |v| v.call.is_none());
    assert_eq!(r.calls.0.lock().unwrap()[1], "resolve T1 true");
    assert!(r.view.host.as_ref().unwrap().threads[0].resolved);
    assert!(r.screen().contains("Reopen"), "folded with a Reopen button");

    r.key(alt('1'));
    r.key(plain(KeyCode::Char('c')));
    r.type_text("overall fine");
    r.key(ctrl('s'));
    wait(&mut r.view, "the comment", |v| v.call.is_none());
    assert_eq!(r.calls.0.lock().unwrap()[2], "comment overall fine");
    assert!(r.screen().contains("overall fine"));
}

#[test]
fn a_hosts_long_refusal_in_the_finish_form_wraps_instead_of_being_cut() {
    let mut r = rig(GITHUB_CAPS);
    r.key(plain(KeyCode::Char('f')));
    r.view.finish.as_mut().unwrap().message = Some(
        "the host refused the request (Unprocessable Entity (Review Can not approve your own pull request))"
            .to_string(),
    );
    let s = r.screen();
    assert!(
        s.contains("own pull request))"),
        "the end of the refusal is cut:\n{s}"
    );
    assert!(!s.contains('\u{2026}'), "{s}");
}

#[test]
fn a_refused_reply_gives_the_words_back() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.view.start_call(
        Call::Reply {
            thread_id: "T1".into(),
            body: "lost?".into(),
        },
        false,
        Some((
            ComposeFor::Reply {
                thread_id: "T1".into(),
            },
            "lost?".into(),
        )),
    );
    // The fake accepts, so simulate the refusal the way poll sees it.
    let (tx, rx) = std::sync::mpsc::channel();
    tx.send(Err("host said no".to_string())).unwrap();
    r.view.call.as_mut().unwrap().rx = rx;
    r.view.poll();
    let c = r.view.compose.as_ref().expect("the box is back");
    assert_eq!(c.text(), "lost?");
    assert!(r.screen().contains("could not send: host said no"));
}

#[test]
fn the_tabs_switch_by_key_and_by_click() {
    let mut r = rig(GITHUB_CAPS);
    r.key(alt('3'));
    assert_eq!(r.view.tab, Tab::Commits);
    r.key(alt('2'));
    assert_eq!(r.view.tab, Tab::Files);
    r.screen();
    let overview = r.rect_of(Hit::Tab(Tab::Overview));
    r.click(overview);
    assert_eq!(r.view.tab, Tab::Overview);
}

#[test]
fn escape_closes_the_view_but_first_leaves_a_popup_or_a_selection() {
    let mut r = rig(GITHUB_CAPS);
    r.key(plain(KeyCode::Char('f')));
    assert_eq!(r.key(plain(KeyCode::Esc)), ReviewEffect::None);
    assert!(r.view.finish.is_none());
    assert_eq!(r.key(plain(KeyCode::Esc)), ReviewEffect::Close);
}

#[test]
fn a_click_on_the_plus_of_a_line_opens_the_comment_box() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.cursor_on("added one");
    r.screen();
    let at = r.row_with("added one");
    let plus = r.rect_of(Hit::Add(PaneId::Files, at));
    r.click(plus);
    assert!(r.view.compose.is_some());
}

#[test]
fn the_palette_rows_say_why_they_do_nothing() {
    let mut r = rig(GITHUB_CAPS);
    let avail = Some(r.view.avail());
    assert!(
        ReviewCmd::AddComment.unavailable(avail).is_some(),
        "overview"
    );
    assert!(ReviewCmd::FinishReview.unavailable(avail).is_none());
    assert!(ReviewCmd::DiscardReview.unavailable(avail).is_some());
    assert!(ReviewCmd::Close.unavailable(None).is_some());
    r.on_files();
    r.cursor_on("added one");
    let avail = Some(r.view.avail());
    assert!(ReviewCmd::AddComment.unavailable(avail).is_none());
    assert!(
        ReviewCmd::Reply.unavailable(avail).is_some(),
        "no thread there"
    );
    r.cursor_on("Why a Vec here?");
    let avail = Some(r.view.avail());
    assert!(ReviewCmd::Reply.unavailable(avail).is_none());
    assert!(ReviewCmd::ToggleResolve.unavailable(avail).is_none());
}

#[test]
fn the_review_is_pending_scoped_per_request() {
    let r = rig(GITHUB_CAPS);
    assert_eq!(r.view.target.draft_key(), "corp|g/p#9");
    let _ = &r.dir;
    let _ = r.caps;
}

// -- the ✦ suggestions ---------------------------------------------------------------

use super::model::{Item, Place, Sev};

fn item(line: u32, sev: Sev, body: &str) -> Item {
    Item {
        id: 0,
        path: "src/a.rs".into(),
        line,
        severity: sev,
        body: body.into(),
        anchored: true,
        place: Some(Place {
            side: DiffSide::New,
            line,
            other_line: None,
            start_line: None,
        }),
        prefix: String::new(),
    }
}

impl Rig {
    /// Puts suggestions in as if a run had finished.
    fn suggest(&mut self, items: Vec<Item>, summary: &str) {
        self.view.on_suggestions(super::ai::Done {
            items,
            summary: summary.into(),
            body: summary.into(),
            verdict: ReviewEvent::Comment,
            cut: 0,
            left_out: 0,
            context_notes: Vec::new(),
        });
    }

    /// A finished run with a full draft.
    fn draft(&mut self, items: Vec<Item>, body: &str, verdict: ReviewEvent) {
        self.view.on_suggestions(super::ai::Done {
            items,
            summary: "A tidy change.".into(),
            body: body.into(),
            verdict,
            cut: 0,
            left_out: 0,
            context_notes: Vec::new(),
        });
    }
}

#[test]
fn nothing_of_the_ai_shows_while_ai_is_off() {
    let mut r = rig(GITHUB_CAPS);
    let s = r.screen();
    assert!(
        !s.contains("Draft review") && !s.contains('\u{2726}'),
        "{s}"
    );
    // The key says why, and the palette rows are hidden by `is_ai`.
    r.key(plain(KeyCode::Char('s')));
    assert!(r
        .view
        .message
        .as_deref()
        .unwrap()
        .contains("AI is turned off"));
    assert!(ReviewCmd::SuggestReview.is_ai());
    assert!(!r.view.sug.running());
}

#[test]
fn the_suggest_button_exists_with_ai_on_and_waits_for_the_code() {
    let mut r = rig_ai(GITHUB_CAPS);
    let s = r.screen();
    assert!(s.contains("\u{2726} Draft review..."), "{s}");
    // The code of the request is not here yet (no daemon in this rig): the
    // ask is held until it is, and Esc drops it.
    r.key(plain(KeyCode::Char('s')));
    assert!(r.view.options.is_some(), "the key opens the options first");
    assert!(!r.view.sug.running(), "nothing runs before Enter");
    r.key(plain(KeyCode::Enter));
    assert!(
        r.view.message.as_deref().unwrap().contains("fetching"),
        "{:?}",
        r.view.message
    );
    assert!(
        r.view.sug.running() && r.view.sug.run.is_none(),
        "queued, not asked"
    );
    r.key(plain(KeyCode::Esc));
    assert!(!r.view.sug.running(), "canceled before it started");
    // Once the code is here the same key asks at once.
    r.view.git_err = Some("no fetch".into());
    r.key(plain(KeyCode::Char('s')));
    assert!(r.view.message.as_deref().unwrap().contains("not ready"));
    assert!(r.view.options.is_none(), "no dialog when it cannot run");
}

#[test]
fn suggestions_are_ghost_comments_under_their_line_with_their_own_buttons() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.suggest(
        vec![item(11, Sev::Warning, "Name this value.")],
        "Fine overall.",
    );
    r.on_files();
    let s = r.screen();
    let code = row_of(&s, "added one");
    let lines: Vec<&str> = s.lines().collect();
    assert!(
        lines[code + 1].contains('\u{2726}')
            && lines[code + 1].contains("warning - line 11")
            && lines[code + 1].contains("Accept")
            && lines[code + 1].contains("Dismiss"),
        "the ghost head is the next row:\n{s}"
    );
    assert!(lines[code + 2].contains("Name this value."));
    assert!(
        lines[code + 3].contains("added two"),
        "the code goes on:\n{s}"
    );
}

#[test]
fn accepting_a_suggestion_makes_one_pending_comment_and_sends_nothing() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.suggest(
        vec![
            item(11, Sev::Info, "First remark."),
            item(12, Sev::Issue, "Second remark."),
        ],
        "",
    );
    r.on_files();
    r.screen();
    r.cursor_on("added one");
    r.key(plain(KeyCode::Char('y')));
    assert_eq!(r.view.pending.len(), 1);
    assert_eq!(
        (r.view.pending[0].line, r.view.pending[0].body.as_str()),
        (11, "First remark.")
    );
    assert_eq!(r.view.sug.items.len(), 1, "the accepted one is gone");
    let s = r.screen();
    assert!(
        s.contains("1 pending comment") && s.contains("Pending - line 11"),
        "{s}"
    );
    assert!(
        r.calls.0.lock().unwrap().is_empty() && r.submitted.lock().unwrap().is_empty(),
        "nothing reached the host"
    );
    // It was saved like any pending comment.
    assert!(r
        .view
        .read_draft()
        .comments
        .iter()
        .any(|c| c.body == "First remark."));
}

#[test]
fn dismissing_drops_the_suggestion_and_adds_no_comment() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.suggest(
        vec![item(11, Sev::Info, "Nope."), item(12, Sev::Info, "Keep.")],
        "",
    );
    r.on_files();
    r.screen();
    r.cursor_on("added one");
    r.key(plain(KeyCode::Char('n')));
    assert!(r.view.pending.is_empty());
    assert_eq!(r.view.sug.items.len(), 1);
    assert_eq!(r.view.sug.items[0].body, "Keep.");
}

#[test]
fn edit_then_accept_sends_the_edited_words_to_the_pending_list() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.suggest(vec![item(11, Sev::Info, "Draft remark")], "");
    r.on_files();
    r.screen();
    r.cursor_on("added one");
    r.key(plain(KeyCode::Char('w')));
    let s = r.screen();
    assert!(
        s.contains("Edit suggested comment") && s.contains("Draft remark"),
        "{s}"
    );
    r.type_text(" - better");
    r.key(ctrl('s'));
    assert_eq!(r.view.pending.len(), 1);
    assert_eq!(r.view.pending[0].body, "Draft remark - better");
    assert!(r.view.sug.items.is_empty());
}

#[test]
fn accept_all_goes_to_pending_only_and_the_buttons_do_the_same_as_the_keys() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.suggest(
        vec![item(11, Sev::Info, "One."), item(12, Sev::Warning, "Two.")],
        "Looks fine.",
    );
    // The Overview lists them with the buttons of the list.
    let s = r.screen();
    for want in [
        "AI review",
        "Suggestions (2)",
        "Looks fine.",
        "src/a.rs:11",
        "One.",
        "Accept all",
    ] {
        assert!(s.contains(want), "{want} missing on:\n{s}");
    }
    let rect = r.rect_of(Hit::SugAcceptAll);
    r.click(rect);
    assert_eq!(r.view.pending.len(), 2);
    assert!(r.view.sug.items.is_empty());
    assert!(r.submitted.lock().unwrap().is_empty());
    assert!(r.calls.0.lock().unwrap().is_empty());
}

#[test]
fn the_first_suggestion_block_is_lit_and_no_triangle_marks_it() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.suggest(
        vec![item(11, Sev::Info, "One."), item(12, Sev::Warning, "Two.")],
        "Looks fine.",
    );
    let mut buf = Buffer::empty(AREA);
    r.view.render(AREA, &mut buf);
    let text = crate::dialog_kit::buffer_text(&buf);
    assert!(!text.contains('\u{25B8}'), "no marker drawn:\n{text}");
    let lit = crate::dialog_kit::row_bg(true, false);
    let probe = |needle: &str| {
        let y = text
            .lines()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("{needle} missing"));
        let line = text.lines().nth(y).unwrap();
        let x = line[..line.find(needle).unwrap()].chars().count();
        buf[(x as u16, y as u16)].bg
    };
    for needle in ["src/a.rs:11", "One."] {
        assert_eq!(probe(needle), lit, "{needle} row is lit");
    }
    for needle in ["src/a.rs:12", "Two."] {
        assert_ne!(probe(needle), lit, "{needle} row is not lit");
    }
}

#[test]
fn a_suggestion_on_a_line_the_diff_does_not_show_is_listed_for_the_file_and_never_dropped() {
    let mut r = rig_ai(GITHUB_CAPS);
    let mut far = item(400, Sev::Warning, "Far from the change.");
    far.anchored = false;
    far.place = Some(Place {
        side: DiffSide::Old,
        line: 11,
        other_line: None,
        start_line: None,
    });
    far.prefix = gitpoulpi_i18n::msg::review_ai::about_prefix(400);
    r.suggest(vec![far], "");
    r.on_files();
    let s = r.screen();
    let heading = row_of(&s, "AI suggestions about the whole file");
    let lines: Vec<&str> = s.lines().collect();
    assert!(lines[heading + 1].contains("about line 400"), "{s}");
    assert!(lines[heading + 2].contains("Far from the change."));
    r.cursor_on("Far from the change.");
    r.key(plain(KeyCode::Char('y')));
    assert_eq!(r.view.pending.len(), 1);
    assert_eq!(
        r.view.pending[0].line, 11,
        "on the file's first changed line"
    );
    assert_eq!(r.view.pending[0].side, DiffSide::Old);
    assert_eq!(
        r.view.pending[0].body,
        "(about line 400) Far from the change."
    );
}

#[test]
fn a_suggestion_whose_file_is_not_in_the_diff_cannot_be_accepted_but_stays() {
    let mut r = rig_ai(GITHUB_CAPS);
    let mut ghost = item(3, Sev::Info, "Elsewhere.");
    ghost.path = "not/in/diff.rs".into();
    ghost.anchored = false;
    ghost.place = None;
    r.suggest(vec![ghost], "");
    r.key(plain(KeyCode::Char('y')));
    assert!(r.view.pending.is_empty());
    assert_eq!(r.view.sug.items.len(), 1, "still listed");
    assert!(r.view.message.as_deref().unwrap().contains("nowhere to go"));
}

#[test]
fn the_summary_goes_into_the_finish_text_only_by_an_explicit_button() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.suggest(vec![], "A tidy change; one risk in the loader.");
    r.key(plain(KeyCode::Char('f')));
    let s = r.screen();
    assert!(s.contains("Use AI summary"), "{s}");
    assert!(
        !s.contains("one risk in the loader"),
        "not inserted on its own:\n{s}"
    );
    let rect = r
        .view
        .finish
        .as_ref()
        .unwrap()
        .summary_button()
        .expect("the summary button");
    r.click(rect);
    let s = r.screen();
    assert!(s.contains("one risk in the loader"), "{s}");
    assert!(r.submitted.lock().unwrap().is_empty(), "nothing sent");
}

#[test]
fn the_palette_rows_of_the_suggestions_say_why_they_do_nothing() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.screen();
    let avail = Some(r.view.avail());
    assert_eq!(
        ReviewCmd::AcceptAllSuggestions.unavailable(avail),
        Some("there is no suggestion to act on here")
    );
    assert_eq!(
        ReviewCmd::UseSummary.unavailable(avail),
        Some("the AI gave no summary")
    );
    assert_eq!(
        ReviewCmd::SuggestReview.unavailable(avail),
        None,
        "it queues while the code is coming"
    );
    r.view.git_err = Some("no fetch".into());
    assert!(ReviewCmd::SuggestReview
        .unavailable(Some(r.view.avail()))
        .is_some());
    r.view.git_err = None;
    r.suggest(vec![item(11, Sev::Info, "x")], "s");
    let avail = Some(r.view.avail());
    assert_eq!(ReviewCmd::AcceptAllSuggestions.unavailable(avail), None);
    assert_eq!(ReviewCmd::UseSummary.unavailable(avail), None);
    // Off: every one of them says AI is off.
    let mut off = rig(GITHUB_CAPS);
    off.screen();
    let avail = Some(off.view.avail());
    for cmd in [
        ReviewCmd::SuggestReview,
        ReviewCmd::AcceptSuggestion,
        ReviewCmd::UseSummary,
    ] {
        assert!(cmd.unavailable(avail).unwrap().contains("AI is turned off"));
    }
}

// -- the full ✦ review draft ------------------------------------------------------------

#[test]
fn the_overview_shows_the_draft_card_with_verdict_text_and_buttons() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.draft(
        vec![item(11, Sev::Issue, "Off by one")],
        "Please fix the loop bound.",
        ReviewEvent::RequestChanges,
    );
    let s = r.screen();
    assert!(s.contains("AI review draft"), "{s}");
    assert!(s.contains("Request changes"), "the verdict chip:\n{s}");
    assert!(s.contains("Please fix the loop bound."), "{s}");
    assert!(
        s.contains("Use draft...") && s.contains("Edit text..."),
        "{s}"
    );
    assert!(s.contains("Discard draft"), "{s}");
    assert!(!s.contains("changed since this draft"), "not stale:\n{s}");
    // Off: nothing of it.
    let mut off = rig(GITHUB_CAPS);
    assert!(!off.screen().contains("AI review draft"));
}

#[test]
fn use_draft_accepts_the_open_suggestions_and_fills_finish_review_and_cancel_keeps_the_draft() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.draft(
        vec![
            item(11, Sev::Issue, "Off by one"),
            item(12, Sev::Info, "Name it"),
        ],
        "Please fix the loop bound.",
        ReviewEvent::RequestChanges,
    );
    r.screen();
    r.key(plain(KeyCode::Char('u')));
    assert_eq!(r.view.pending.len(), 2, "both suggestions are pending");
    assert!(r.view.sug.items.is_empty());
    let (event, body) = r.view.finish.as_ref().expect("Finish review").filled();
    assert_eq!(event, ReviewEvent::RequestChanges);
    assert_eq!(body, "Please fix the loop bound.");
    let s = r.screen();
    assert!(s.contains("2 pending comments will be sent."), "{s}");
    assert!(r.submitted.lock().unwrap().is_empty(), "nothing is sent");
    // Cancel: the draft stays, on screen and on disk.
    r.key(plain(KeyCode::Esc));
    assert!(r.view.finish.is_none());
    assert!(r.view.has_draft());
    assert!(r.screen().contains("Please fix the loop bound."));
    let kept = r.view.read_draft();
    let ai = kept.ai.expect("the draft is kept");
    assert_eq!(
        (ai.body.as_str(), ai.verdict.as_str()),
        ("Please fix the loop bound.", "request-changes")
    );
    // Submitting spends it.
    r.key(plain(KeyCode::Char('u')));
    r.key(ctrl('s'));
    wait(&mut r.view, "the review sent", |v| v.call.is_none());
    let sent = r.submitted.lock().unwrap().clone();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].event, ReviewEvent::RequestChanges);
    assert_eq!(sent[0].comments.len(), 2);
    assert_eq!(sent[0].body, "Please fix the loop bound.");
    assert!(!r.view.has_draft());
    assert!(r.view.read_draft().ai.is_none());
}

#[test]
fn a_host_without_request_changes_gets_a_comment_with_a_note() {
    let mut r = rig_ai(GITLAB_CAPS);
    r.draft(vec![], "Needs work.", ReviewEvent::RequestChanges);
    r.key(plain(KeyCode::Char('u')));
    let dialog = r.view.finish.as_ref().expect("Finish review");
    assert_eq!(
        dialog.filled(),
        (ReviewEvent::Comment, "Needs work.".to_string())
    );
    let s = r.screen();
    assert!(s.contains("no Request changes"), "{s}");
}

#[test]
fn gerrit_gets_a_plus_or_minus_one_vote_within_the_permitted_values() {
    use gitpoulpi_hosting::{VoteLabel, VoteValue};
    let label = |values: &[i32]| VoteLabel {
        name: "Code-Review".into(),
        values: values
            .iter()
            .map(|v| VoteValue {
                value: *v,
                text: String::new(),
            })
            .collect(),
        current: 0,
        default: 0,
    };
    let caps = ReviewCapabilities {
        review: true,
        ..Default::default()
    };
    let mut d = super::popups::FinishDialog::new(caps, 0, None, &[label(&[-2, -1, 0, 1, 2])]);
    assert!(d.prefill("ok", ReviewEvent::Approve).is_none());
    assert_eq!(d.vote_changes(), vec![("Code-Review".to_string(), 1)]);
    let mut d = super::popups::FinishDialog::new(caps, 0, None, &[label(&[-2, -1, 0, 1, 2])]);
    assert!(d.prefill("no", ReviewEvent::RequestChanges).is_none());
    assert_eq!(d.vote_changes(), vec![("Code-Review".to_string(), -1)]);
    // The user may not vote -1 here: no vote is set, and the dialog says so.
    let mut d = super::popups::FinishDialog::new(caps, 0, None, &[label(&[0, 1])]);
    assert!(d.prefill("no", ReviewEvent::RequestChanges).is_some());
    assert!(d.vote_changes().is_empty());
    // A comment sets no vote.
    let mut d = super::popups::FinishDialog::new(caps, 0, None, &[label(&[-1, 0, 1])]);
    assert!(d.prefill("fine", ReviewEvent::Comment).is_none());
    assert!(d.vote_changes().is_empty());
}

#[test]
fn edit_text_changes_the_draft_text_and_discard_throws_the_whole_draft_away() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.draft(
        vec![item(11, Sev::Info, "Name it")],
        "First words.",
        ReviewEvent::Approve,
    );
    r.screen();
    r.key(plain(KeyCode::Char('t')));
    let s = r.screen();
    assert!(
        s.contains("Edit the draft text") && s.contains("First words."),
        "{s}"
    );
    r.type_text(" More.");
    r.key(ctrl('s'));
    assert_eq!(r.view.sug.body, "First words. More.");
    assert_eq!(r.view.read_draft().ai.unwrap().body, "First words. More.");
    // The buttons do what the keys do.
    r.screen();
    let rect = r.rect_of(Hit::DiscardDraft);
    r.click(rect);
    assert!(!r.view.has_draft());
    assert!(r.view.sug.items.is_empty() && r.view.sug.body.is_empty());
    assert!(r.view.read_draft().ai.is_none());
    assert!(!r.screen().contains("AI review draft"));
    // `N` does it from the keyboard.
    r.draft(
        vec![item(11, Sev::Info, "x")],
        "Words.",
        ReviewEvent::Comment,
    );
    r.key(plain(KeyCode::Char('N')));
    assert!(!r.view.has_draft());
    assert!(r.view.pending.is_empty(), "nothing was accepted");
}

#[test]
fn the_draft_comes_back_on_reopen_and_is_marked_stale_when_the_head_moved() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.draft(
        vec![item(11, Sev::Warning, "Name it")],
        "Words.",
        ReviewEvent::Approve,
    );
    let Rig { view, dir, .. } = r;
    drop(view);
    let mut again = open_with(dir, GITHUB_CAPS, true);
    assert!(again.view.has_draft());
    assert_eq!(again.view.sug.verdict, ReviewEvent::Approve);
    assert_eq!(again.view.sug.items.len(), 1);
    let s = again.screen();
    assert!(s.contains("Words.") && s.contains("Name it"), "{s}");
    assert!(!s.contains("changed since this draft"), "{s}");
    // The head moved since.
    again.view.sug.head_sha = "9".repeat(40);
    assert!(again.screen().contains("changed since this draft"));
}

#[test]
fn the_draft_rows_of_the_palette_say_why_they_do_nothing() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.screen();
    let avail = Some(r.view.avail());
    for cmd in [
        ReviewCmd::UseDraft,
        ReviewCmd::EditDraftText,
        ReviewCmd::DiscardDraft,
    ] {
        assert!(cmd.is_ai());
        assert_eq!(
            cmd.unavailable(avail),
            Some("there is no AI draft yet (ask for one with Draft review)")
        );
    }
    r.draft(vec![], "Words.", ReviewEvent::Comment);
    let avail = Some(r.view.avail());
    assert_eq!(ReviewCmd::UseDraft.unavailable(avail), None);
    let mut off = rig(GITHUB_CAPS);
    off.screen();
    assert!(ReviewCmd::UseDraft
        .unavailable(Some(off.view.avail()))
        .unwrap()
        .contains("AI is turned off"));
}

// -- the Diff view's options on the review's diff pane --------------------------

/// The whole of `src/a.rs` the diff above is a part of: 14 lines.
fn a_rs_whole() -> Vec<String> {
    (1..=14)
        .map(|n| match n {
            10 => "keep".to_string(),
            11 => "added one".to_string(),
            12 => "added two".to_string(),
            13 => "tail".to_string(),
            n => format!("plain {n}"),
        })
        .collect()
}

impl Rig {
    fn mode(&mut self, mode: crate::diff_view::DiffViewMode) {
        self.view.diff_opts.mode = mode;
        self.view.revision += 1;
    }

    /// The whole file instead of the hunks, its text already read.
    fn whole_file(&mut self) {
        let head = self.view.git.as_ref().unwrap().head;
        self.view.blobs.insert(head, "src/a.rs", a_rs_whole());
        self.view.diff_opts.hunks_only = false;
        self.view.revision += 1;
    }

    /// The glyphs painted in the files pane's scrollbar column, on a window
    /// short enough for the diff to scroll.
    fn ruler(&mut self) -> String {
        let area = Rect::new(0, 0, 140, 22);
        let mut buf = Buffer::empty(area);
        self.view.render(area, &mut buf);
        let bar = self.view.files_pane.bar.expect("a scrollbar").rect;
        (bar.y..bar.bottom())
            .map(|y| buf[(bar.x, y)].symbol().to_string())
            .collect()
    }
}

impl Rig {
    /// The screen column of the Split separator on the Files pane.
    fn split_column(&self) -> usize {
        let pane = &self.view.files_pane;
        let split_x = pane.render.as_ref().and_then(|r| r.split_x).expect("Split");
        (pane.area.x + split_x - 1) as usize
    }
}

fn col_of(line: &str, needle: &str) -> usize {
    let byte = line
        .find(needle)
        .unwrap_or_else(|| panic!("no `{needle}` in {line}"));
    line[..byte].chars().count()
}

#[test]
fn the_diff_pane_has_the_diff_views_bar_with_every_toggle_and_the_encoding() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    let s = r.screen();
    let bar = s.lines().nth(row_of(&s, "Inline")).unwrap();
    for label in [
        "Inline", "Split", "Hunks", "WS", "Wrap", "Map", "Auto", "\u{2191}", "\u{2193}",
    ] {
        assert!(bar.contains(label), "{label} on the bar: {bar}");
    }
}

#[test]
fn every_toggle_has_a_button_a_key_and_a_palette_command_and_is_remembered() {
    use crate::diff_view::{DiffBarButton as B, DiffViewMode};
    let _lock = crate::live_settings::DIFF_VIEW_PREFS_TEST_LOCK
        .lock()
        .unwrap();
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.screen();

    // The button, the key and the palette command do the same thing.
    let split = r.rect_of(Hit::Bar(B::Mode(DiffViewMode::Split)));
    r.click(split);
    assert_eq!(r.view.diff_opts.mode, DiffViewMode::Split);
    assert_eq!(
        crate::live_settings::diff_view_prefs().layout,
        gitpoulpi_config::DiffLayout::Split,
        "the next diff opens the way this one was left"
    );
    r.key(plain(KeyCode::Char('V')));
    assert_eq!(r.view.diff_opts.mode, DiffViewMode::Inline);
    r.view.run(ReviewCmd::ToggleLayout);
    assert_eq!(r.view.diff_opts.mode, DiffViewMode::Split);
    r.screen();
    let inline = r.rect_of(Hit::Bar(B::Mode(DiffViewMode::Inline)));
    r.click(inline);
    assert_eq!(r.view.diff_opts.mode, DiffViewMode::Inline);

    r.screen();
    let hunks = r.rect_of(Hit::Bar(B::ToggleHunksOnly));
    r.click(hunks);
    assert!(!r.view.diff_opts.hunks_only);
    r.key(plain(KeyCode::Char('H')));
    assert!(r.view.diff_opts.hunks_only);
    r.view.run(ReviewCmd::ToggleHunksOnly);
    assert!(!r.view.diff_opts.hunks_only);
    r.view.run(ReviewCmd::ToggleHunksOnly);

    r.screen();
    let ws = r.rect_of(Hit::Bar(B::ToggleWhitespace));
    r.click(ws);
    assert!(!r.view.diff_opts.show_whitespace);
    r.key(plain(KeyCode::Char('S')));
    assert!(r.view.diff_opts.show_whitespace);

    r.screen();
    let wrap = r.rect_of(Hit::Bar(B::ToggleWordWrap));
    r.click(wrap);
    assert!(r.view.diff_opts.word_wrap);
    r.key(plain(KeyCode::Char('W')));
    assert!(!r.view.diff_opts.word_wrap);
    let prefs = crate::live_settings::diff_view_prefs();
    assert_eq!(
        (prefs.hunks_only, prefs.show_whitespace, prefs.word_wrap),
        (true, true, false)
    );

    // Not on the Overview.
    r.key(alt('1'));
    assert!(ReviewCmd::ToggleLayout
        .unavailable(Some(r.view.avail()))
        .is_some());
    crate::live_settings::install_diff_view_prefs(
        crate::live_settings::DiffViewPrefs::const_default(),
    );
}

#[test]
fn the_hunk_keys_walk_the_hunks_and_say_when_there_is_no_other() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.screen();
    r.key(plain(KeyCode::Enter));
    r.view.files_pane.jump(0);
    r.key(plain(KeyCode::Char('K')));
    assert!(
        r.view
            .message
            .as_deref()
            .unwrap_or("")
            .contains("first hunk"),
        "{:?}",
        r.view.message
    );
    r.key(plain(KeyCode::Char('J')));
    assert!(
        r.view
            .message
            .as_deref()
            .unwrap_or("")
            .contains("no further hunk"),
        "{:?}",
        r.view.message
    );
}

#[test]
fn split_puts_a_thread_under_the_half_of_its_side_with_its_buttons() {
    use crate::diff_view::DiffViewMode;
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.mode(DiffViewMode::Split);
    let s = r.screen();
    let code = row_of(&s, "added two");
    let lines: Vec<&str> = s.lines().collect();
    let sep = r.split_column();
    // T1 is on the new side: right half, under its line, buttons included.
    let head = lines[code + 1];
    assert!(col_of(head, "tomas") > sep, "right half:\n{s}");
    assert!(head.contains("Reply") && head.contains("Resolve"), "{head}");
    assert!(lines[code + 2].contains("Why a Vec here?"));
    assert!(lines[code + 4].contains("Built once at start-up."));
    assert!(
        lines[code + 5].contains("tail"),
        "the code goes on after the thread:\n{s}"
    );
    let reply_row = r.row_with("tomas");
    let reply = r.rect_of(Hit::ThreadReply(reply_row));
    assert!(reply.x as usize > sep);
    // The separator column goes on through the thread.
    assert_eq!(lines[code + 2].chars().nth(sep), Some('\u{2502}'), "{s}");
}

#[test]
fn a_comment_in_split_goes_on_the_side_of_the_half_that_was_clicked() {
    use crate::diff_view::DiffViewMode;
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.mode(DiffViewMode::Split);
    r.screen();
    // "gone" (old 11) and "added one" (new 11) are one Split row.
    let row = r.row_with("gone");
    let sep = r.split_column() as u16;

    for (column, side) in [(sep - 6, DiffSide::Old), (sep + 6, DiffSide::New)] {
        r.screen();
        let area = r.view.files_pane.area;
        r.view.on_mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row: area.y + (row - r.view.files_pane.offset) as u16,
            modifiers: KeyModifiers::NONE,
        });
        r.key(plain(KeyCode::Char('c')));
        r.type_text("here");
        r.key(ctrl('s'));
        let c = r.view.pending.last().unwrap();
        assert_eq!((c.side, c.line), (side, 11), "column {column}");
    }
    // Both pending comments are drawn, each in its own half, under the pair.
    let s = r.screen();
    let heads: Vec<usize> = s
        .lines()
        .filter(|l| l.contains("Pending - line 11"))
        .map(|l| col_of(l, "Pending"))
        .collect();
    assert_eq!(heads.len(), 2, "{s}");
    assert!(heads.iter().any(|c| *c < sep as usize) && heads.iter().any(|c| *c > sep as usize));
}

#[test]
fn the_whole_file_shows_a_thread_outside_the_hunks_under_its_line_in_both_layouts() {
    use crate::diff_view::DiffViewMode;
    let mut r = rig(GITHUB_CAPS);
    r.view.host.as_mut().unwrap().threads[0].line = Some(3);
    r.on_files();
    for mode in [DiffViewMode::Inline, DiffViewMode::Split] {
        r.mode(mode);
        r.view.diff_opts.hunks_only = true;
        let s = r.screen();
        assert!(
            s.contains("Comments outside the lines shown"),
            "{mode:?} hunks only:\n{s}"
        );

        r.whole_file();
        let s = r.screen();
        assert!(
            !s.contains("Comments outside the lines shown"),
            "{mode:?}:\n{s}"
        );
        let lines: Vec<&str> = s.lines().collect();
        let three = row_of(&s, "plain 3");
        assert!(
            lines[three + 1].contains("tomas") && lines[three + 1].contains("Reply"),
            "{mode:?} right under line 3:\n{s}"
        );
    }
}

#[test]
fn the_ruler_marks_the_threads_apart_from_the_changes() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    // Hunks only: the Diff view draws no change marks there, the threads still.
    let ruler = r.ruler();
    assert!(ruler.contains('\u{25cf}'), "a thread dot: {ruler:?}");
    assert!(
        !ruler.contains('\u{2590}'),
        "no change marks in hunks only: {ruler:?}"
    );
    r.whole_file();
    let ruler = r.ruler();
    assert!(
        ruler.contains('\u{25cf}') && ruler.contains('\u{2590}'),
        "{ruler:?}"
    );
}

#[test]
fn the_whole_file_waits_for_its_text_without_hiding_the_hunks() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.view.diff_opts.hunks_only = false;
    r.view.revision += 1;
    let s = r.screen();
    assert!(s.contains("loading the whole file"), "{s}");
    assert!(s.contains("added two") && !s.contains("plain 3"), "{s}");
}

#[test]
fn the_commits_page_uses_the_same_bar_layouts_and_whole_file() {
    use crate::diff_view::DiffViewMode;
    let mut r = rig(GITHUB_CAPS);
    let commit = gitpoulpi_core::domain::CommitNode {
        id: Oid::from_hex(&"3".repeat(40)).unwrap(),
        parents: Vec::new(),
        summary: "Add moons".into(),
        message: "Add moons".into(),
        author_name: "ines".into(),
        author_email: "ines@example.com".into(),
        timestamp: 0,
        refs: Vec::new(),
    };
    r.view.git = Some(GitData {
        head: Oid::from_hex(&"1".repeat(40)).unwrap(),
        diff_base: Oid::from_hex(&"2".repeat(40)).unwrap(),
        comparing: false,
        commits: vec![commit.clone()],
    });
    r.view.git_rx = None;
    r.view.commit_diffs.insert(commit.id, vec![a_rs()]);
    r.key(alt('3'));
    let s = r.screen();
    for label in ["Inline", "Split", "Hunks", "WS", "Wrap", "Auto"] {
        assert!(s.contains(label), "{label} on the Commits page:\n{s}");
    }
    let lines: Vec<&str> = s.lines().collect();
    let name = row_of(&s, "src/a.rs");
    assert!(lines[name + 1].contains("@@ -10,3 +10,4 @@"), "{s}");

    r.view.diff_opts.mode = DiffViewMode::Split;
    r.view.revision += 1;
    let s = r.screen();
    let at = row_of(&s, "gone");
    assert!(s.lines().nth(at).unwrap().contains("added one"), "{s}");

    // The whole file of a commit is read at that commit.
    r.view.blobs.insert(commit.id, "src/a.rs", a_rs_whole());
    r.view.diff_opts.hunks_only = false;
    r.view.revision += 1;
    let s = r.screen();
    assert!(s.contains("plain 3") && s.contains("plain 14"), "{s}");
    // ...and the hunk keys work there too.
    r.view.focus = Focus::Diff;
    r.key(plain(KeyCode::Char('J')));
    assert_eq!(
        r.view.commits_pane.cursor,
        r.row_with_in(PaneId::Commits, "@@")
    );
}

// -- the Diff view's other parts on the review's pane ---------------------------------

/// A one-hunk diff of `n` context lines with a very long added line near the
/// top, for the scroll and the minimap.
fn tall_and_wide(n: usize) -> FileDiff {
    let mut lines: Vec<DiffLine> = (0..n)
        .map(|i| line(LineOrigin::Context, &format!("context {i}")))
        .collect();
    lines.insert(
        2,
        line(
            LineOrigin::Add,
            &format!("{}END_OF_LONG_LINE", "long ".repeat(60)),
        ),
    );
    FileDiff {
        path: "src/a.rs".into(),
        status: FileStatus::Modified,
        hunks: vec![Hunk {
            id: HunkId(0),
            old_start: 1,
            old_lines: n as u32,
            new_start: 1,
            new_lines: n as u32 + 1,
            lines,
        }],
    }
}

impl Rig {
    fn screen_at(&mut self, area: Rect) -> String {
        let mut buf = Buffer::empty(area);
        self.view.render(area, &mut buf);
        crate::dialog_kit::buffer_text(&buf)
    }

    fn mouse(&mut self, kind: MouseEventKind, column: u16, row: u16) {
        self.view.on_mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        });
    }

    /// The screen cell of the `n`th character of the code on pane row `row`.
    fn code_cell(&self, row: usize, n: u16) -> (u16, u16) {
        let pane = &self.view.files_pane;
        let seg = pane.render.as_ref().unwrap().segs[row][0].expect("a code row");
        (
            pane.area.x + seg.x + n,
            pane.area.y + (row - pane.offset) as u16,
        )
    }
}

#[test]
fn the_encoding_dropdown_reads_the_file_again_and_is_remembered_per_path() {
    use crate::diff_view::DiffBarButton as B;
    use crate::editor_charset::Charset;
    let _lock = crate::live_settings::DIFF_VIEW_PREFS_TEST_LOCK
        .lock()
        .unwrap();
    let path = std::path::Path::new("src/a.rs");
    crate::diff_recode::remember(path, None);
    let mut r = rig(GITHUB_CAPS);
    r.on_files();

    // The button opens the list, Auto-detect first; Esc leaves without a choice.
    let button = r.rect_of(Hit::Bar(B::Encoding));
    r.click(button);
    assert!(r.view.bar_menu.is_some());
    let s = r.screen();
    assert!(
        s.contains("Auto-detect") && s.contains("Windows-1252"),
        "{s}"
    );
    r.key(plain(KeyCode::Esc));
    assert!(r.view.bar_menu.is_none());
    assert_eq!(crate::diff_recode::remembered(path), None);

    // The key opens the same list; typing narrows it, Enter picks.
    r.key(plain(KeyCode::Char('E')));
    assert!(r.view.bar_menu.is_some());
    r.type_text("1252");
    r.key(plain(KeyCode::Enter));
    assert!(r.view.bar_menu.is_none());
    assert_eq!(
        crate::diff_recode::remembered(path),
        Some(Charset::Windows1252)
    );
    let s = r.screen();
    assert!(
        s.contains("Windows-1252 \u{25be}"),
        "the bar names it:\n{s}"
    );
    assert!(
        r.view
            .message
            .as_deref()
            .unwrap_or("")
            .contains("Windows-1252"),
        "{:?}",
        r.view.message
    );

    // The daemon is asked for the bytes of both sides; its answer is the diff.
    r.view.poll();
    assert!(r.view.recodes.busy(), "the read is out");
    let sides = r.view.sides_of(None).unwrap();
    let (diff, new_lines) = crate::diff_recode::recode(
        path,
        Some(b"keep\ngone\ntail\n"),
        Some(b"keep\nd\xe9j\xe0 vu\ntail\n"),
        Charset::Windows1252,
    )
    .unwrap();
    r.view.recodes.insert(
        super::recode::Key::new(
            "src/a.rs",
            FileStatus::Modified,
            sides,
            Charset::Windows1252,
        ),
        super::recode::Done::Ready(Arc::new(crate::diff_recode::Recoded {
            diff,
            new_lines: Arc::new(new_lines),
        })),
    );
    r.view.revision += 1;
    let s = r.screen();
    assert!(
        s.contains("d\u{e9}j\u{e0} vu") && !s.contains("added one"),
        "{s}"
    );

    // Back to Auto by the palette's command: the file shows as the backend read it.
    r.view.run(ReviewCmd::ChooseEncoding);
    r.key(plain(KeyCode::Home));
    r.key(plain(KeyCode::Enter));
    assert_eq!(crate::diff_recode::remembered(path), None);
    let s = r.screen();
    assert!(
        s.contains("added one") && !s.contains("d\u{e9}j\u{e0}"),
        "{s}"
    );

    // Not on the Overview, with the reason.
    r.key(alt('1'));
    assert!(ReviewCmd::ChooseEncoding
        .unavailable(Some(r.view.avail()))
        .is_some());
}

#[test]
fn a_failed_reread_says_why_above_the_diff_it_keeps() {
    use crate::editor_charset::Charset;
    let _lock = crate::live_settings::DIFF_VIEW_PREFS_TEST_LOCK
        .lock()
        .unwrap();
    let path = std::path::Path::new("src/a.rs");
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    crate::diff_recode::remember(path, Some(Charset::Utf16Le));
    let sides = r.view.sides_of(None).unwrap();
    r.view.recodes.insert(
        super::recode::Key::new("src/a.rs", FileStatus::Modified, sides, Charset::Utf16Le),
        super::recode::Done::Failed("not valid UTF-16 LE".into()),
    );
    r.view.revision += 1;
    let s = r.screen();
    crate::diff_recode::remember(path, None);
    assert!(
        s.contains("cannot read as UTF-16 LE: not valid UTF-16 LE") && s.contains("added one"),
        "{s}"
    );
}

#[test]
fn the_text_scrolls_sideways_by_key_wheel_bar_and_palette_and_left_at_the_edge_leaves_the_diff() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.view.diffs.insert("src/a.rs".into(), tall_and_wide(5));
    r.view.revision += 1;
    let s = r.screen();
    assert!(!s.contains("END_OF_LONG_LINE"), "cut at the edge:\n{s}");
    assert!(r.view.files_pane.h_max > 0);
    assert!(r.view.files_pane.h_bar.is_some(), "a sideways scrollbar");

    r.key(plain(KeyCode::Enter));
    assert_eq!(r.view.focus, Focus::Diff);
    // Right: the keyboard's way.
    r.key(plain(KeyCode::Right));
    assert_eq!(r.view.diff_opts.h_offset, 4);
    let s = r.screen();
    assert!(s.contains('\u{ab}'), "the scrolled-row marker:\n{s}");
    // The palette's twins.
    r.view.run(ReviewCmd::ScrollRight);
    assert_eq!(r.view.diff_opts.h_offset, 8);
    r.view.run(ReviewCmd::ScrollLeft);
    assert_eq!(r.view.diff_opts.h_offset, 4);
    // The sideways wheel and shift + the wheel.
    r.screen();
    let (c, y) = (r.view.files_pane.area.x + 5, r.view.files_pane.area.y + 2);
    r.mouse(MouseEventKind::ScrollRight, c, y);
    assert_eq!(r.view.diff_opts.h_offset, 8);
    r.view.on_mouse(MouseEvent {
        kind: MouseEventKind::ScrollUp,
        column: c,
        row: y,
        modifiers: KeyModifiers::SHIFT,
    });
    assert_eq!(r.view.diff_opts.h_offset, 4);
    // The bar's far end shows the end of the line.
    r.screen();
    let track = r.view.files_pane.h_bar.unwrap().rect;
    r.mouse(
        MouseEventKind::Down(MouseButton::Left),
        track.right() - 3,
        track.y,
    );
    r.mouse(
        MouseEventKind::Up(MouseButton::Left),
        track.right() - 3,
        track.y,
    );
    assert_eq!(r.view.diff_opts.h_offset, r.view.files_pane.h_max);
    let s = r.screen();
    assert!(s.contains("END_OF_LONG_LINE"), "{s}");
    // Left walks back; at the left edge it is the way back to the list.
    for _ in 0..100 {
        if r.view.diff_opts.h_offset == 0 {
            break;
        }
        r.key(plain(KeyCode::Left));
    }
    assert_eq!((r.view.diff_opts.h_offset, r.view.focus), (0, Focus::Diff));
    r.key(plain(KeyCode::Left));
    assert_eq!(r.view.focus, Focus::List);
    // Word wrap has nothing to scroll.
    r.view.run(ReviewCmd::ToggleWrap);
    r.view.run(ReviewCmd::ScrollRight);
    assert_eq!(r.view.diff_opts.h_offset, 0);
    r.view.run(ReviewCmd::ToggleWrap);
}

#[test]
fn a_drag_in_the_code_selects_text_to_copy_and_a_drag_in_the_gutter_still_selects_lines() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.screen();
    let from = r.row_with("added one");
    let (x, y) = r.code_cell(from, 0);
    // In the code: text.
    r.mouse(MouseEventKind::Down(MouseButton::Left), x + 1, y);
    r.mouse(MouseEventKind::Drag(MouseButton::Left), x + 4, y);
    r.mouse(MouseEventKind::Up(MouseButton::Left), x + 4, y);
    let sel = r.view.files_pane.text_sel.expect("a text selection");
    assert_ne!(sel.anchor, sel.head);
    assert_eq!(r.view.files_pane.anchor, None, "not a line selection");
    // It is painted, and Ctrl+C copies exactly the characters.
    let s = r.screen();
    assert!(s.contains("added one"), "{s}");
    r.key(ctrl('c'));
    assert!(
        r.view.message.as_deref().unwrap_or("").contains("(dded)"),
        "{:?}",
        r.view.message
    );
    // The same copy by the palette's command, with the selection gone: the line.
    r.view.files_pane.text_sel = None;
    r.view.run(ReviewCmd::CopyText);
    assert!(
        r.view
            .message
            .as_deref()
            .unwrap_or("")
            .contains("added one"),
        "{:?}",
        r.view.message
    );

    // In the gutter: the lines, as before.
    let (gx, gy) = {
        let pane = &r.view.files_pane;
        (pane.area.x + 7, pane.area.y + (from - pane.offset) as u16)
    };
    r.mouse(MouseEventKind::Down(MouseButton::Left), gx, gy);
    r.mouse(MouseEventKind::Drag(MouseButton::Left), gx, gy + 1);
    r.mouse(MouseEventKind::Up(MouseButton::Left), gx, gy + 1);
    assert_eq!(r.view.files_pane.text_sel, None);
    assert_eq!(r.view.files_pane.anchor, Some(from));
    assert_eq!(r.view.files_pane.cursor, from + 1);

    // A plain click in the gutter drops the text selection it replaces.
    let (x, y) = r.code_cell(from, 0);
    r.mouse(MouseEventKind::Down(MouseButton::Left), x + 1, y);
    r.mouse(MouseEventKind::Drag(MouseButton::Left), x + 3, y);
    r.mouse(MouseEventKind::Up(MouseButton::Left), x + 3, y);
    assert!(r.view.files_pane.text_sel.is_some());
    r.mouse(MouseEventKind::Down(MouseButton::Left), gx, gy);
    r.mouse(MouseEventKind::Up(MouseButton::Left), gx, gy);
    assert_eq!(r.view.files_pane.text_sel, None);
}

#[test]
fn the_minimap_sits_beside_the_text_and_a_click_on_it_scrolls() {
    let _lock = crate::live_settings::DIFF_VIEW_PREFS_TEST_LOCK
        .lock()
        .unwrap();
    crate::live_settings::install_minimap(true);
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    r.view.diffs.insert("src/a.rs".into(), tall_and_wide(120));
    r.view.revision += 1;
    r.screen();
    let map = r.view.files_pane.map.expect("the minimap");
    let area = r.view.files_pane.area;
    assert!(map.x >= area.right(), "beside the text, not under it");
    assert_eq!(r.view.files_pane.offset, 0);
    r.mouse(
        MouseEventKind::Down(MouseButton::Left),
        map.x + 2,
        map.bottom() - 1,
    );
    r.mouse(
        MouseEventKind::Up(MouseButton::Left),
        map.x + 2,
        map.bottom() - 1,
    );
    assert!(
        r.view.files_pane.offset > 50,
        "{}",
        r.view.files_pane.offset
    );
    // The key switches it off, and the text has its room back.
    r.screen();
    r.key(plain(KeyCode::Char('M')));
    assert!(!crate::live_settings::minimap());
    r.screen();
    assert!(r.view.files_pane.map.is_none());
    assert!(r.view.files_pane.area.width > area.width);
}

#[test]
fn buttons_that_do_not_fit_go_behind_a_more_button_and_run_from_its_list() {
    let _lock = crate::live_settings::DIFF_VIEW_PREFS_TEST_LOCK
        .lock()
        .unwrap();
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    let narrow = Rect::new(0, 0, 96, 44);
    let s = r.screen_at(narrow);
    assert!(s.contains('\u{22ef}'), "a more button:\n{s}");
    let (more, hidden) = r.view.bar_slots.more.clone().expect("the hidden buttons");
    assert!(!hidden.is_empty());
    let at = r
        .view
        .hits
        .iter()
        .find(|(_, h)| *h == Hit::BarMore)
        .unwrap()
        .0;
    assert_eq!(at, more);
    r.click(at);
    assert!(r.view.bar_menu.is_some());
    // Its list names every hidden button.
    let s = r.screen_at(narrow);
    for b in &hidden {
        let label = b.label_for(&r.view.diff_opts);
        assert!(s.contains(&label), "{label} in the list:\n{s}");
    }
    // Nothing behind it takes the keys while it is open.
    assert!(!r.view.takes_palette());
    // Enter runs the first one as if it had been clicked in place.
    let first = hidden[0];
    let toggles = |v: &ReviewView| {
        (
            v.diff_opts.mode,
            v.diff_opts.hunks_only,
            v.diff_opts.show_whitespace,
            v.diff_opts.word_wrap,
        )
    };
    let before = toggles(&r.view);
    r.key(plain(KeyCode::Enter));
    assert!(
        r.view.bar_menu.is_none() || first == crate::diff_view::DiffBarButton::Encoding,
        "the list closed"
    );
    match first {
        crate::diff_view::DiffBarButton::Encoding => assert!(r.view.bar_menu.is_some()),
        crate::diff_view::DiffBarButton::ToggleMinimap => {}
        crate::diff_view::DiffBarButton::NextHunk | crate::diff_view::DiffBarButton::PrevHunk => {}
        _ => assert_ne!(before, toggles(&r.view), "{first:?} ran"),
    }
    crate::live_settings::install_diff_view_prefs(
        crate::live_settings::DiffViewPrefs::const_default(),
    );
}

#[test]
fn the_hint_line_offers_resolve_only_where_the_host_can() {
    let mut r = rig(GITHUB_CAPS);
    r.on_files();
    let s = r.screen();
    assert!(s.contains("x resolve"), "{s}");
    let mut r = rig(ReviewCapabilities {
        resolve: false,
        ..GITHUB_CAPS
    });
    r.on_files();
    let s = r.screen();
    assert!(!s.contains("x resolve"), "{s}");
    assert!(s.contains("r reply"), "the other hints stay:\n{s}");
}

#[test]
fn no_diff_is_asked_while_the_refs_are_read_again() {
    let mut r = rig(GITHUB_CAPS);
    r.view.git = Some(GitData {
        head: Oid::from_hex(&"1".repeat(40)).unwrap(),
        diff_base: Oid::from_hex(&"2".repeat(40)).unwrap(),
        comparing: false,
        commits: Vec::new(),
    });
    r.view.git_rx = None;
    r.key(alt('2'));
    // A compare base was just picked: the old reading is still in `git`.
    let (_tx, rx) = std::sync::mpsc::channel();
    r.view.git_rx = Some(rx);
    r.view.poll();
    assert!(
        r.view.diff_wait.is_none(),
        "a diff against the old base would be cached for the new one"
    );
    r.view.git_rx = None;
    r.view.poll();
    assert!(r.view.diff_wait.is_some(), "asked once the reading is in");
}

#[test]
fn the_submit_and_abandon_confirmations_are_titled_by_their_verb() {
    let mut r = rig(GITHUB_CAPS);
    r.view.ask_change(super::ConfirmFor::AbandonChange);
    let s = r.screen();
    assert!(s.contains("Abandon !9 on the server?"), "{s}");
    assert!(!s.contains("Discard review"), "no wrong title:\n{s}");
    r.key(plain(KeyCode::Esc));
    r.view.ask_change(super::ConfirmFor::SubmitChange);
    let s = r.screen();
    assert!(s.contains("Submit !9 on the server?"), "{s}");
    assert!(!s.contains("Discard review"), "no wrong title:\n{s}");
}

mod api;
mod review_options;

/// The three small review dialogs carry the shared close cross on their top
/// border, and a click on it answers like Esc (Cancel / Cancel / No).
/// Falsified by dropping the `close.hit` arm of each `on_mouse`.
#[test]
fn the_small_review_dialogs_close_on_their_cross() {
    use super::popups::{ComposeBox, ComposeOutcome, ConfirmOutcome, FinishOutcome};
    let area = Rect::new(0, 0, 120, 40);
    let down = |c: u16, r: u16| MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: c,
        row: r,
        modifiers: KeyModifiers::NONE,
    };
    let cross_of = |buf: &Buffer| -> (u16, u16) {
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                if buf[(x, y)].symbol() == "\u{2715}" {
                    return (x, y);
                }
            }
        }
        panic!("no close cross painted");
    };

    let mut compose = ComposeBox::new(
        ComposeFor::Conversation,
        "Comment".into(),
        "about".into(),
        String::new(),
    );
    let mut buf = Buffer::empty(area);
    compose.render(area, &mut buf);
    let (c, r) = cross_of(&buf);
    assert_eq!(compose.on_mouse(down(c, r)), ComposeOutcome::Cancel);

    let caps = ReviewCapabilities {
        review: true,
        ..Default::default()
    };
    let mut finish = super::popups::FinishDialog::new(caps, 0, None, &[]);
    let mut buf = Buffer::empty(area);
    finish.render(area, &mut buf, ' ');
    let (c, r) = cross_of(&buf);
    assert!(matches!(finish.on_mouse(down(c, r)), FinishOutcome::Cancel));

    let mut confirm = super::popups::Confirm::new("Pick", "Sure?".into(), "Yes");
    let mut buf = Buffer::empty(area);
    confirm.render(area, &mut buf);
    let (c, r) = cross_of(&buf);
    assert_eq!(confirm.on_mouse(down(c, r)), ConfirmOutcome::No);
}

mod md_composer {
    //! The Markdown composer inside the review's two text dialogs: the
    //! Write / Preview modes, the discard question and the send key.

    use super::*;
    use crate::review_view::popups::{ComposeBox, ComposeOutcome, FinishOutcome};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn alt(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT)
    }

    fn type_text(compose: &mut ComposeBox, text: &str) {
        for c in text.chars() {
            compose.on_key(key(KeyCode::Char(c)));
        }
    }

    fn screen(buf: &Buffer) -> String {
        (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn compose() -> ComposeBox {
        ComposeBox::new(
            ComposeFor::Conversation,
            "Comment".into(),
            "about".into(),
            String::new(),
        )
    }

    #[test]
    fn the_comment_box_has_write_and_preview_and_renders_the_markdown() {
        let area = Rect::new(0, 0, 120, 40);
        let mut c = compose();
        type_text(&mut c, "**bold** words");
        let mut buf = Buffer::empty(area);
        c.render(area, &mut buf);
        let s = screen(&buf);
        for want in ["Write", "Preview", "Split", "**bold** words"] {
            assert!(s.contains(want), "{want:?}:\n{s}");
        }
        // Alt+P: the same text, rendered.
        c.on_key(alt('p'));
        let mut buf = Buffer::empty(area);
        c.render(area, &mut buf);
        let s = screen(&buf);
        assert!(s.contains("bold words"), "{s}");
        assert!(!s.contains("**bold**"), "{s}");
        // Back to Write: the text is still there.
        c.on_key(alt('p'));
        assert_eq!(c.text(), "**bold** words");
    }

    #[test]
    fn a_comment_that_was_typed_asks_before_it_is_thrown_away() {
        let area = Rect::new(0, 0, 120, 40);
        let mut c = compose();
        type_text(&mut c, "half a thought");
        // Esc asks; Keep editing goes back to the text.
        assert_eq!(c.on_key(key(KeyCode::Esc)), ComposeOutcome::Stay);
        let mut buf = Buffer::empty(area);
        c.render(area, &mut buf);
        let s = screen(&buf);
        assert!(s.contains("Discard this text?"), "{s}");
        assert_eq!(c.on_key(key(KeyCode::Esc)), ComposeOutcome::Stay);
        assert_eq!(c.text(), "half a thought");
        // Again, then Discard (the second button) closes.
        c.on_key(key(KeyCode::Esc));
        c.on_key(key(KeyCode::Right));
        assert_eq!(c.on_key(key(KeyCode::Enter)), ComposeOutcome::Cancel);
    }

    #[test]
    fn an_untouched_comment_box_closes_without_asking() {
        let mut c = compose();
        assert_eq!(c.on_key(key(KeyCode::Esc)), ComposeOutcome::Cancel);
        let mut edit = ComposeBox::new(
            ComposeFor::EditPending(0),
            "Edit".into(),
            "a:1".into(),
            "existing words".into(),
        );
        assert_eq!(edit.on_key(key(KeyCode::Esc)), ComposeOutcome::Cancel);
        type_text(&mut edit, "x");
        assert_eq!(edit.on_key(key(KeyCode::Esc)), ComposeOutcome::Stay);
    }

    #[test]
    fn ctrl_enter_and_the_post_button_send_the_trimmed_text() {
        let mut c = compose();
        assert_eq!(
            c.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)),
            ComposeOutcome::Stay,
            "nothing to send yet"
        );
        type_text(&mut c, " hi ");
        c.on_key(key(KeyCode::Enter));
        type_text(&mut c, "there");
        assert_eq!(
            c.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::CONTROL)),
            ComposeOutcome::Submit("hi \nthere".into())
        );
    }

    #[test]
    fn the_finish_summary_is_a_markdown_editor_too() {
        let area = Rect::new(0, 0, 120, 40);
        let caps = ReviewCapabilities {
            review: true,
            ..Default::default()
        };
        let mut d = super::super::popups::FinishDialog::new(caps, 0, None, &[]);
        for c in "# Verdict".chars() {
            d.on_key(key(KeyCode::Char(c)));
        }
        let mut buf = Buffer::empty(area);
        d.render(area, &mut buf, ' ');
        let s = screen(&buf);
        assert!(s.contains("Write") && s.contains("Preview"), "{s}");
        d.on_key(alt('p'));
        let mut buf = Buffer::empty(area);
        d.render(area, &mut buf, ' ');
        let s = screen(&buf);
        assert!(!s.contains("# Verdict"), "{s}");
        assert!(s.contains("Verdict"), "{s}");
        // Typed text: Esc asks.
        d.on_key(alt('p'));
        assert!(matches!(d.on_key(key(KeyCode::Esc)), FinishOutcome::Stay));
        let mut buf = Buffer::empty(area);
        d.render(area, &mut buf, ' ');
        assert!(screen(&buf).contains("Discard this text?"));
    }
}
