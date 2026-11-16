//! The review view over the host's API: no clone, so no daemon and no
//! `Requester` at all. The fake host answers `pull_request_diff`,
//! `pull_request_commits`, `commit_diff` and `file_content`; the pending
//! review lives in `<config>/remote-reviews.toml`.

use super::*;
use gitpoulpi_session::RemoteReviews;

fn api_target() -> ReviewTarget {
    ReviewTarget {
        repo_root: None,
        ..target(std::path::Path::new("."))
    }
}

pub(super) fn open_api_in(config: tempfile::TempDir, caps: ReviewCapabilities, ai: bool) -> Rig {
    let calls = Arc::new(Calls::default());
    let submitted = Arc::new(Mutex::new(Vec::new()));
    let conv = review_conv();
    let factory = Arc::new(Factory {
        caps,
        calls: calls.clone(),
        submitted: submitted.clone(),
        conv: conv.clone(),
    });
    let mut view = ReviewView::open_api(
        api_target(),
        factory,
        ai,
        "s".into(),
        Some(config.path().to_path_buf()),
    );
    view.diff_opts.mode = crate::diff_view::DiffViewMode::Inline;
    view.diff_opts.hunks_only = true;
    view.diff_opts.show_whitespace = true;
    view.diff_opts.word_wrap = false;
    view.sug.fake_defaults = Some(options::Defaults {
        run_blocked: Some("no clone".into()),
        ..Default::default()
    });
    wait(&mut view, "the host data", |v| v.host.is_some());
    wait(&mut view, "the code", |v| v.git.is_some());
    wait(&mut view, "the conversation", |v| v.ov.loaded());
    Rig {
        conv,
        view,
        calls,
        submitted,
        dir: config,
        caps,
        _queue: None,
    }
}

fn api_rig() -> Rig {
    open_api_in(tempfile::tempdir().unwrap(), GITHUB_CAPS, false)
}

impl Rig {
    fn calls_made(&self) -> Vec<String> {
        self.calls.0.lock().unwrap().clone()
    }

    /// The Files page, with the open file's diff read from the host.
    fn api_files(&mut self) {
        self.key(alt('2'));
        wait(&mut self.view, "the file diff", |v| {
            v.diffs.contains_key("src/a.rs")
        });
        self.screen();
    }

    fn remote_file(&self) -> RemoteReviews {
        RemoteReviews::load(&RemoteReviews::file_in(self.dir.path()))
    }
}

#[test]
fn the_view_has_no_daemon_and_reads_the_head_and_base_from_the_detail() {
    let r = api_rig();
    assert!(r.view.source.is_api() && r.view.source.requester().is_none());
    let git = r.view.git.as_ref().unwrap();
    assert_eq!(git.head, Oid::from_hex(&"1".repeat(40)).unwrap());
    assert_eq!(git.diff_base, Oid::from_hex(&"2".repeat(40)).unwrap());
    assert!(!git.comparing);
}

#[test]
fn the_files_page_draws_a_thread_right_under_its_line_from_the_hosts_diff() {
    let mut r = api_rig();
    r.api_files();
    let s = r.screen();
    let code = row_of(&s, "added two");
    let lines: Vec<&str> = s.lines().collect();
    assert!(lines[code + 1].contains("tomas \u{b7} 2026-10-11"), "{s}");
    assert!(lines[code + 2].contains("Why a Vec here?"), "{s}");
    assert!(lines[code + 5].contains("tail"), "{s}");
}

#[test]
fn the_whole_file_is_read_from_the_host_at_the_requests_head() {
    let mut r = api_rig();
    r.api_files();
    r.view.diff_opts.hunks_only = false;
    r.view.revision += 1;
    let head = "1".repeat(40);
    wait(&mut r.view, "the file text", |v| {
        v.blobs
            .get(v.git.as_ref().unwrap().head, "src/a.rs")
            .is_some()
    });
    let s = r.screen();
    assert!(s.contains("plain 3") && s.contains("plain 14"), "{s}");
    assert!(
        r.calls_made()
            .contains(&format!("file_content src/a.rs {head}")),
        "{:?}",
        r.calls_made()
    );
}

#[test]
fn the_commits_page_lists_the_hosts_commits_and_reads_one_commits_diff() {
    let mut r = api_rig();
    r.key(alt('3'));
    let s = r.screen();
    // Newest first, as with a clone.
    assert!(
        row_of(&s, "4444444 Sort moons") < row_of(&s, "3333333 Add moons"),
        "{s}"
    );
    wait(&mut r.view, "the commit diff", |v| {
        !v.commit_diffs.is_empty()
    });
    let s = r.screen();
    assert!(s.contains("added two"), "{s}");
    assert_eq!(
        r.calls_made(),
        vec![format!("commit_diff {}", "4".repeat(40))]
    );
    // The whole file of a commit is read at that commit.
    r.view.diff_opts.hunks_only = false;
    r.view.revision += 1;
    let at = Oid::from_hex(&"4".repeat(40)).unwrap();
    wait(&mut r.view, "the file text", |v| {
        v.blobs.get(at, "src/a.rs").is_some()
    });
    assert!(r
        .calls_made()
        .contains(&format!("file_content src/a.rs {}", "4".repeat(40))));
}

#[test]
fn a_file_the_host_sent_no_patch_for_says_so() {
    let mut r = api_rig();
    r.key(alt('2'));
    r.view.file_sel = 1;
    wait(&mut r.view, "the file diff", |v| {
        v.diffs.contains_key("docs/moons.md")
    });
    let s = r.screen();
    assert!(s.contains("not shown by the host"), "{s}");
}

#[test]
fn a_pending_comment_is_kept_in_remote_reviews_and_comes_back_on_reopen() {
    let mut r = api_rig();
    r.api_files();
    r.cursor_on("added one");
    r.key(plain(KeyCode::Char('c')));
    r.type_text("rename this");
    r.key(ctrl('s'));
    assert_eq!(r.view.pending.len(), 1);
    assert_eq!(
        r.view.pending[0].line, 11,
        "the comment lands on a real line"
    );

    let file = r.remote_file();
    let draft = file
        .draft("gitlab.example.com/g/p", &r.view.target.draft_key())
        .expect("the draft is filed under host/project");
    assert_eq!(draft.comments[0].body, "rename this");
    assert_eq!(draft.head_sha, "1".repeat(40));

    let Rig { view, dir, .. } = r;
    drop(view);
    let again = open_api_in(dir, GITHUB_CAPS, false);
    assert_eq!(again.view.pending.len(), 1);
    assert_eq!(again.view.pending[0].body, "rename this");
}

#[test]
fn submitting_sends_the_head_sha_and_empties_the_stored_draft() {
    let mut r = api_rig();
    r.api_files();
    r.cursor_on("added one");
    r.key(plain(KeyCode::Char('c')));
    r.type_text("rename this");
    r.key(ctrl('s'));
    r.key(plain(KeyCode::Char('f')));
    r.key(ctrl('s'));
    wait(&mut r.view, "the review sent", |v| v.call.is_none());
    let sent = r.submitted.lock().unwrap().clone();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].head_sha.as_deref(), Some("1".repeat(40).as_str()));
    assert_eq!(sent[0].comments[0].line, 11);
    assert!(r.remote_file().drafts.is_empty(), "nothing left to keep");
}

#[test]
fn the_encoding_is_refused_without_a_clone() {
    let mut r = api_rig();
    r.api_files();
    r.key(plain(KeyCode::Char('E')));
    assert!(r.view.bar_menu.is_none());
    assert_eq!(
        r.view.message.as_deref(),
        Some("Not available without a clone")
    );
    // The bar's button says the same.
    r.view.message = None;
    let button = r.rect_of(Hit::Bar(crate::diff_view::DiffBarButton::Encoding));
    r.click(button);
    assert!(r.view.bar_menu.is_none());
    assert_eq!(
        r.view.message.as_deref(),
        Some("Not available without a clone")
    );
}

#[test]
fn the_ai_draft_reads_the_hosts_files_and_is_kept_in_remote_reviews() {
    let mut r = open_api_in(tempfile::tempdir().unwrap(), GITHUB_CAPS, true);
    // The model is a fake: it reports what the worker read for it.
    r.view.sug.fake = Some(Arc::new(|files| super::super::ai::Done {
        items: vec![item(11, Sev::Warning, "Name it")],
        summary: format!(
            "{} files: {}",
            files.len(),
            files
                .iter()
                .map(|f| f.path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        body: "Please rename.".into(),
        verdict: ReviewEvent::RequestChanges,
        cut: 0,
        left_out: 0,
        context_notes: Vec::new(),
    }));
    r.view.start_suggest(Default::default());
    wait(&mut r.view, "the draft", |v| !v.sug.running());
    assert_eq!(r.view.sug.summary, "2 files: src/a.rs, docs/moons.md");
    assert_eq!(r.view.sug.verdict, ReviewEvent::RequestChanges);
    assert_eq!(r.view.sug.items.len(), 1);
    let file = r.remote_file();
    let kept = file
        .draft("gitlab.example.com/g/p", &r.view.target.draft_key())
        .and_then(|d| d.ai.as_ref())
        .expect("the draft is stored with the request");
    assert_eq!(kept.body, "Please rename.");
}

#[cfg(feature = "ai")]
#[test]
fn the_context_files_are_read_from_the_host_at_the_requests_base() {
    use gitpoulpi_ai::review_context::{gather, EffectiveReviewContext};
    let r = api_rig();
    let source = crate::review_view::ai_context::ViewContext {
        source: std::sync::Mutex::new(r.view.source.clone()),
        base: r.view.git.as_ref().unwrap().diff_base,
        cancel: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        integrations: Vec::new(),
    };
    let ctx = EffectiveReviewContext {
        run_in_repository: gitpoulpi_config::RunInRepo::Profile,
        include_claude_md: true,
        include_agents_md: false,
        extra_file: "docs/REVIEW.md".into(),
    };
    let files = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(gather(&ctx, &source));
    assert_eq!(
        files.iter().map(|f| f.label.as_str()).collect::<Vec<_>>(),
        ["CLAUDE.md", "docs/REVIEW.md"]
    );
    let base = "2".repeat(40);
    assert_eq!(
        r.calls_made(),
        vec![
            format!("file_content CLAUDE.md {base}"),
            format!("file_content docs/REVIEW.md {base}")
        ]
    );
}

#[cfg(feature = "ai")]
#[test]
fn the_repositorys_review_values_come_from_its_state_or_the_projects_entry() {
    use crate::review_view::ai_context::{effective, own_values};
    use gitpoulpi_config::{AiReviewContext, RepoAiReview, RunInRepo};
    let dir = tempfile::tempdir().unwrap();
    // A clone: its repo-state.toml.
    let root = dir.path().join("clone");
    let mut state = gitpoulpi_session::RepoState::default();
    state.preferences.ai_review.include_claude_md = Some(false);
    state.preferences.ai_review.run_in_repository = Some(RunInRepo::Yes);
    state
        .save(&gitpoulpi_session::RepoState::path_for(
            &root,
            "in_worktree",
        ))
        .unwrap();
    let own = own_values(Some(&root), None, "h/p", "in_worktree");
    let global = AiReviewContext::default();
    let eff = effective(&global, &own, &RepoAiReview::default());
    assert!(!eff.include_claude_md);
    assert!(eff.include_agents_md, "inherited");
    assert_eq!(eff.run_in_repository, RunInRepo::Yes);
    // No clone: the project's entry of remote-reviews.toml.
    let file = gitpoulpi_session::RemoteReviews::file_in(dir.path());
    let mut all = gitpoulpi_session::RemoteReviews::default();
    all.ai_review.insert(
        "h/p".into(),
        RepoAiReview {
            extra_file: Some("docs/R.md".into()),
            ..RepoAiReview::default()
        },
    );
    all.save(&file).unwrap();
    let own = own_values(None, Some(&file), "h/p", "in_worktree");
    assert_eq!(
        effective(&global, &own, &RepoAiReview::default()).extra_file,
        "docs/R.md"
    );
    assert_eq!(
        own_values(None, Some(&file), "h/other", "in_worktree"),
        RepoAiReview::default()
    );
}
