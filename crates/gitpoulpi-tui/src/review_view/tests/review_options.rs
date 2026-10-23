//! The "✦ Review options" dialog: prefill, per-run values, saving the
//! defaults, the reasons a choice is disabled and the context problems.

use super::*;
use crate::review_view::options;
use gitpoulpi_config::{AiReviewContext, RepoAiReview, RunInRepo};
use gitpoulpi_session::{RemoteReviews, RepoState};

fn defaults() -> options::Defaults {
    options::Defaults {
        state_dir: "in_worktree".into(),
        ..Default::default()
    }
}

#[test]
fn s_opens_the_options_prefilled_with_the_repository_over_the_global_values() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.view.sug.fake_defaults = Some(options::Defaults {
        global: AiReviewContext {
            include_claude_md: true,
            include_agents_md: true,
            extra_file: "global.md".into(),
            ..Default::default()
        },
        own: RepoAiReview {
            include_agents_md: Some(false),
            extra_file: Some("docs/REPO.md".into()),
            run_in_repository: Some(RunInRepo::Yes),
            ..Default::default()
        },
        ..defaults()
    });
    r.key(plain(KeyCode::Char('s')));
    let s = r.screen();
    for want in [
        "Review options",
        "Run the agent in the repository",
        "Yes",
        "[x] CLAUDE.md",
        "[ ] AGENTS.md",
        "docs/REPO.md",
        "Save as this repository's defaults",
        "the agent reads it on its own in the repository",
    ] {
        assert!(s.contains(want), "{want}:\n{s}");
    }
    assert!(!s.contains("global.md"), "the repository value wins:\n{s}");
    // Nothing was asked yet.
    assert!(!r.view.sug.running());
}

#[test]
fn enter_runs_with_the_values_of_the_dialog_and_saves_nothing_by_default() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.key(plain(KeyCode::Char('s')));
    // Run combobox, CLAUDE.md, AGENTS.md: untick the last one.
    r.key(plain(KeyCode::Tab));
    r.key(plain(KeyCode::Tab));
    r.key(plain(KeyCode::Char(' ')));
    r.key(plain(KeyCode::Enter));
    assert!(r.view.options.is_none(), "the dialog closed");
    let per_run = &r.view.sug.per_run;
    assert_eq!(per_run.include_agents_md, Some(false));
    assert_eq!(per_run.include_claude_md, Some(true));
    assert_eq!(per_run.extra_file, None);
    assert_eq!(per_run.no_extra_file, Some(true), "an empty field is none");
    assert!(r.view.sug.running(), "queued: the code is not here yet");
    let state = RepoState::path_for(r.dir.path(), "in_worktree");
    assert!(!state.exists(), "no default was saved");
}

#[test]
fn the_extra_file_is_typed_and_goes_with_the_run() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.key(plain(KeyCode::Char('s')));
    for _ in 0..3 {
        r.key(plain(KeyCode::Tab));
    }
    r.type_text("docs/REVIEW.md");
    r.key(plain(KeyCode::Enter));
    assert_eq!(
        r.view.sug.per_run.extra_file.as_deref(),
        Some("docs/REVIEW.md")
    );
}

#[test]
fn saving_the_defaults_writes_the_repository_state_with_only_what_differs() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.key(plain(KeyCode::Char('s')));
    for _ in 0..2 {
        r.key(plain(KeyCode::Tab));
    }
    r.key(plain(KeyCode::Char(' '))); // AGENTS.md off
    for _ in 0..4 {
        r.key(plain(KeyCode::Tab)); // extra file, Browse, No extra file, "save"
    }
    r.key(plain(KeyCode::Char(' ')));
    r.key(plain(KeyCode::Enter));
    let saved = RepoState::load(&RepoState::path_for(r.dir.path(), "in_worktree"))
        .unwrap()
        .preferences
        .ai_review;
    assert_eq!(
        saved,
        RepoAiReview {
            include_agents_md: Some(false),
            ..Default::default()
        },
        "what agrees with the global value keeps following it"
    );
    // Opened again, the dialog starts from them.
    r.key(plain(KeyCode::Esc));
    assert!(!r.view.sug.running());
    r.view.sug.fake_defaults = Some(options::Defaults {
        own: saved,
        ..defaults()
    });
    r.key(plain(KeyCode::Char('s')));
    let s = r.screen();
    assert!(
        s.contains("[ ] AGENTS.md") && s.contains("[x] CLAUDE.md"),
        "{s}"
    );
}

#[test]
fn without_a_clone_the_defaults_go_to_remote_reviews_and_the_run_choice_says_why_not() {
    let config = tempfile::tempdir().unwrap();
    let mut r = api::open_api_in(config, GITHUB_CAPS, true);
    r.key(plain(KeyCode::Char('s')));
    let s = r.screen();
    assert!(s.contains("no clone"), "the reason is shown:\n{s}");
    assert!(
        r.view.options.as_ref().unwrap().hit_of("run").is_none(),
        "the choice is not clickable"
    );
    // The first Tab goes straight to CLAUDE.md: untick it, save, run.
    r.key(plain(KeyCode::Char(' ')));
    for _ in 0..5 {
        r.key(plain(KeyCode::Tab));
    }
    r.key(plain(KeyCode::Char(' ')));
    r.key(plain(KeyCode::Enter));
    assert_eq!(r.view.sug.per_run.run_in_repository, None);
    assert_eq!(r.view.sug.per_run.include_claude_md, Some(false));
    let key = r.view.target.remote_key();
    let file = RemoteReviews::load(&RemoteReviews::file_in(r.dir.path()));
    assert_eq!(
        file.ai_review.get(&key),
        Some(&RepoAiReview {
            include_claude_md: Some(false),
            ..Default::default()
        })
    );
}

#[test]
fn an_http_profile_disables_the_run_choice_with_its_reason() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.view.sug.fake_defaults = Some(options::Defaults {
        run_blocked: Some("this profile has no working folder".into()),
        ..defaults()
    });
    r.key(plain(KeyCode::Char('s')));
    let s = r.screen();
    assert!(s.contains("this profile has no working folder"), "{s}");
    r.key(plain(KeyCode::Enter));
    assert_eq!(
        r.view.sug.per_run.run_in_repository, None,
        "left to the profile"
    );
}

#[test]
fn the_cross_and_escape_close_the_dialog_without_running() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.key(plain(KeyCode::Char('s')));
    r.screen();
    r.key(plain(KeyCode::Esc));
    assert!(r.view.options.is_none() && !r.view.sug.running());
    r.key(plain(KeyCode::Char('s')));
    r.screen();
    let cross = r.view.options.as_ref().unwrap().cross().expect("a cross");
    r.click(cross);
    assert!(r.view.options.is_none() && !r.view.sug.running());
    // Cancel is a button too.
    r.key(plain(KeyCode::Char('s')));
    r.screen();
    let cancel = r.view.options.as_ref().unwrap().hit_of("cancel").unwrap();
    r.click(cancel);
    assert!(r.view.options.is_none() && !r.view.sug.running());
}

#[test]
fn the_buttons_run_and_the_checkboxes_toggle_by_click() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.key(plain(KeyCode::Char('s')));
    r.screen();
    let agents = r.view.options.as_ref().unwrap().hit_of("agents").unwrap();
    r.click(agents);
    r.screen();
    let go = r.view.options.as_ref().unwrap().hit_of("go").unwrap();
    r.click(go);
    assert_eq!(r.view.sug.per_run.include_agents_md, Some(false));
}

#[test]
fn context_problems_and_cut_notes_show_on_the_draft_card() {
    let mut r = rig_ai(GITHUB_CAPS);
    r.view.on_suggestions(super::super::ai::Done {
        items: vec![item(11, Sev::Info, "x")],
        summary: "s".into(),
        body: "A body.".into(),
        verdict: ReviewEvent::Comment,
        cut: 0,
        left_out: 0,
        context_notes: vec![
            "Context not given to the model: docs/R.md: not found".into(),
            "CLAUDE.md was shortened to fit the context limit.".into(),
        ],
    });
    let s = r.screen();
    assert!(s.contains("docs/R.md: not found"), "{s}");
    assert!(s.contains("CLAUDE.md was shortened"), "{s}");
}
