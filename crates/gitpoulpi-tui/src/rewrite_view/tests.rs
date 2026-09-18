//! The workbench model: rules list, detail fields, scope, questions, save
//! and load. Nothing here draws; the daemon's answers are put in by hand.

use super::*;
use crate::daemon_link::RawQueue;
use gitpoulpi_core::domain::rewrite::{
    ContentFind, Counts, PlanDigest, PushState, RefChange, RewriteWarning, RunId, SecretText,
};
use gitpoulpi_core::domain::Oid;

fn view_in(root: &std::path::Path, config: &std::path::Path) -> (RewriteView, RawQueue) {
    let queue = RawQueue::default();
    let view = RewriteView::open(
        queue.requester(),
        RecipeStore {
            repo_root: root.to_path_buf(),
            config_dir: Some(config.to_path_buf()),
        },
        ScopeContext {
            head_branch: Some("main".into()),
            branches: vec!["main".into(), "dev".into()],
            tags: vec!["v1".into()],
            ..ScopeContext::default()
        },
    );
    (view, queue)
}

fn view() -> (RewriteView, RawQueue, tempfile::TempDir) {
    let tmp = tempfile::tempdir().unwrap();
    let (v, q) = view_in(&tmp.path().join("repo"), &tmp.path().join("cfg"));
    (v, q, tmp)
}

fn plan(signed: u64, protected: &[&str], pushed: bool) -> RewritePlan {
    RewritePlan {
        run: RunId("run-1".into()),
        digest: PlanDigest("digest-1".into()),
        counts: Counts {
            commits_walked: 10,
            commits_changed: 4,
            signed_changed: signed,
            refs_moved: protected.len() as u32 + 1,
            ..Counts::default()
        },
        refs: protected
            .iter()
            .map(|name| RefChange {
                name: (*name).into(),
                old: Oid([1; 20]),
                new: Some(Oid([2; 20])),
                pushed: if pushed {
                    PushState::OnRemote {
                        remote: "origin".into(),
                        remote_oid: Oid([1; 20]),
                        rewritten_on_remote: 2,
                    }
                } else {
                    PushState::LocalOnly
                },
                protected: true,
                included: false,
                checked_out_in: None,
            })
            .collect(),
        warnings: Vec::<RewriteWarning>::new(),
        per_rule: Vec::new(),
        rows: Vec::new(),
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn chars(view: &mut RewriteView, text: &str) {
    for c in text.chars() {
        view.on_key(key(KeyCode::Char(c)));
    }
}

/// An identity rule filled in: old@example.com becomes new@example.org.
fn with_identity_rule(view: &mut RewriteView) {
    view.add_rule(RuleTemplate::Identity);
    view.edit_field(1, FieldInput::Text("old@example.com".into()));
    view.edit_field(4, FieldInput::Text("new@example.org".into()));
}

#[test]
fn a_new_workbench_has_no_rule_and_says_what_is_missing() {
    let (v, _q, _t) = view();
    assert!(v.recipe.rules.is_empty());
    assert!(matches!(
        v.validation(),
        Validation::Invalid { rule: None, .. }
    ));
    assert!(!v.can_apply());
    assert!(v.apply_blocker().is_some());
}

#[test]
fn adding_a_rule_lists_its_fields_and_focuses_nothing_else() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::Identity);
    assert_eq!(v.recipe.rules.len(), 1);
    let ids: Vec<_> = v.detail.fields.iter().map(|f| f.id).collect();
    assert_eq!(ids[0], FieldId::MatchName);
    assert!(ids.contains(&FieldId::SetEmail));
    assert_eq!(v.rule_rows()[0].number, 1);
}

#[test]
fn a_keyboard_session_adds_edits_and_previews_the_rule() {
    let (mut v, _q, _t) = view();
    v.on_key(key(KeyCode::Char('a')));
    assert!(matches!(v.popup, Some(Popup::AddRule { .. })));
    v.on_key(key(KeyCode::Enter));
    assert_eq!(v.focus, Focus::Detail);
    assert_eq!(v.recipe.rules.len(), 1);
    // Field 1 is "Email is": Down, Enter starts typing, text goes in live.
    v.on_key(key(KeyCode::Down));
    v.on_key(key(KeyCode::Enter));
    chars(&mut v, "old@example.com");
    assert!(v.editing.is_some());
    v.on_key(key(KeyCode::Enter));
    assert!(v.editing.is_none());
    match &v.recipe.rules[0].kind {
        RuleKind::Identity(r) => assert_eq!(r.when.email.as_deref(), Some("old@example.com")),
        other => panic!("{other:?}"),
    }
    // The new identity is still empty: the footer says so and Preview stays unsent.
    v.on_key(key(KeyCode::F(5)));
    assert!(!v.busy());
    assert!(v.notice.as_ref().is_some_and(|n| n.error));
    v.edit_field(4, FieldInput::Text("new@example.org".into()));
    v.on_key(key(KeyCode::F(5)));
    assert!(v.busy(), "the preview was sent");
}

#[test]
fn toggle_reorder_delete_and_restore_work_from_the_keys() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::DateShift);
    v.add_rule(RuleTemplate::Trailer);
    v.focus = Focus::Rules;
    // Space switches the rule under the cursor off.
    v.on_key(key(KeyCode::Char(' ')));
    assert!(!v.recipe.rules[1].enabled);
    // Alt+Up moves it above the first.
    v.on_key(KeyEvent::new(KeyCode::Up, KeyModifiers::ALT));
    assert_eq!(v.recipe.rules[0].kind.name(), "trailer");
    assert_eq!(v.rules.cursor, 0);
    // d deletes, Ctrl+Z puts it back where it was.
    v.on_key(key(KeyCode::Char('d')));
    assert_eq!(v.recipe.rules.len(), 1);
    v.on_key(KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL));
    assert_eq!(v.recipe.rules.len(), 2);
    assert_eq!(v.recipe.rules[0].kind.name(), "trailer");
    assert!(!v.recipe.rules[0].enabled);
    // Ctrl+D duplicates.
    v.on_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL));
    assert_eq!(v.recipe.rules.len(), 3);
}

#[test]
fn a_field_that_does_not_parse_keeps_its_draft_and_the_rule_unchanged() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::DateShift);
    let before = v.recipe.rules[0].clone();
    v.edit_field(0, FieldInput::Text("2x".into()));
    assert_eq!(v.recipe.rules[0], before);
    let field = &v.detail.fields[0];
    assert_eq!(field.as_text(), Some("2x"));
    assert!(field.error.is_some());
    v.edit_field(0, FieldInput::Text("-1d 3h".into()));
    assert_eq!(
        v.recipe.rules[0].kind,
        RuleKind::DateShift {
            by_seconds: -(86_400 + 3 * 3_600),
            apply_to: Default::default()
        }
    );
    assert!(v.detail.fields[0].error.is_none());
}

#[test]
fn a_zero_shift_is_the_recipe_checks_to_report() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::DateShift);
    v.edit_field(0, FieldInput::Text("0s".into()));
    match v.validation() {
        Validation::Invalid { rule, text } => {
            assert_eq!(rule, Some(0));
            assert!(text.contains('1'), "{text}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_formats_round_trip() {
    use rules_form::*;
    for s in [3600, -86_400 - 7200, 61, 86_400 * 3] {
        assert_eq!(parse_duration(&format_duration(s)), Some(s));
    }
    assert_eq!(parse_duration("  +2h "), Some(7200));
    assert_eq!(parse_duration("2"), None);
    assert_eq!(parse_duration(""), None);
    assert_eq!(parse_offset("+02:00"), Some(120));
    assert_eq!(parse_offset("-0530"), Some(-330));
    assert_eq!(parse_offset("Z"), Some(0));
    assert_eq!(parse_offset("+99:00"), None);
    assert_eq!(format_offset(-330), "-05:30");
    assert_eq!(parse_size("5M"), Some(5 << 20));
    assert_eq!(format_size(5 << 20), "5 MiB");
    assert_eq!(parse_size("5 MiB"), Some(5 << 20));
    assert_eq!(parse_size("500 KiB"), Some(500 << 10));
    assert_eq!(parse_size("1gib"), Some(1 << 30));
    assert_eq!(parse_size(&format_size(1_500_000)), Some(1_500_000));
    assert_eq!(parse_size("12x"), None);
    let w = parse_windows("Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00").unwrap();
    assert_eq!(w.len(), 2);
    assert_eq!(w[0].days.len(), 5);
    assert_eq!(parse_windows(&format_windows(&w)), Some(w));
    assert_eq!(parse_windows("Funday 10:00-12:00"), None);
}

#[test]
fn a_choice_that_changes_the_fields_rebuilds_the_pane() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::ContentReplace);
    let secret_at = |v: &RewriteView| v.detail.fields.iter().position(|f| f.secret);
    assert!(secret_at(&v).is_none());
    let mode_at = v
        .detail
        .fields
        .iter()
        .position(|f| f.id == FieldId::ContentMode)
        .unwrap();
    v.edit_field(mode_at, FieldInput::Choice(1));
    assert!(secret_at(&v).is_some(), "the secret field appears");
    v.edit_field(mode_at, FieldInput::Choice(0));
    assert!(secret_at(&v).is_none());
}

#[test]
fn secret_values_are_masked_never_shown_in_the_row_and_never_saved() {
    let (mut v, _q, tmp) = view();
    v.add_rule(RuleTemplate::ContentReplace);
    let mode_at = v
        .detail
        .fields
        .iter()
        .position(|f| f.id == FieldId::ContentMode)
        .unwrap();
    v.edit_field(mode_at, FieldInput::Choice(1));
    let at = v
        .detail
        .fields
        .iter()
        .position(|f| f.id == FieldId::SecretValues)
        .unwrap();
    assert!(v.detail.fields[at].secret);
    v.edit_field(at, FieldInput::Text("hunter2\nswordfish".into()));
    match &v.recipe.rules[0].kind {
        RuleKind::ContentReplace(c) => match &c.find {
            ContentFind::Secrets { literals, .. } => {
                assert_eq!(
                    literals,
                    &[SecretText("hunter2".into()), SecretText("swordfish".into())]
                );
            }
            other => panic!("{other:?}"),
        },
        other => panic!("{other:?}"),
    }
    assert!(v.recipe.holds_secrets());
    assert!(!v.rule_rows()[0].detail.contains("hunter2"));
    v.set_name("Remove passwords");
    v.save(RecipePlace::Shared);
    let path = tmp
        .path()
        .join("repo/.gitpoulpi/rewrite/remove-passwords.toml");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        !text.contains("hunter2") && !text.contains("swordfish"),
        "{text}"
    );
    // The workbench keeps them in memory for the run, and says they were not saved.
    assert!(v.recipe.holds_secrets());
    assert!(v.notice.as_ref().is_some_and(|n| !n.error));
}

#[test]
fn saving_shared_and_only_for_me_go_to_different_folders() {
    let (mut v, _q, tmp) = view();
    with_identity_rule(&mut v);
    v.save(RecipePlace::Shared);
    assert!(
        v.notice.as_ref().is_some_and(|n| n.error),
        "a recipe needs a name first"
    );
    v.set_name("Fix work email");
    v.save(RecipePlace::Shared);
    v.save(RecipePlace::Mine);
    let shared = tmp
        .path()
        .join("repo/.gitpoulpi/rewrite/fix-work-email.toml");
    let mine = tmp.path().join("cfg/rewrite/fix-work-email.toml");
    assert!(shared.exists() && mine.exists());
    let saved = v.saved_recipes();
    assert_eq!(saved.len(), 2);
    assert_eq!(saved[0].place, RecipePlace::Shared);
    assert_eq!(saved[1].place, RecipePlace::Mine);
}

#[test]
fn a_saved_recipe_loads_back_and_forgets_the_answers_of_a_run() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.set_name("Fix work email");
    v.choices.signatures = Some(SignaturePolicy::Strip);
    v.recipe.options.signatures = Some(SignaturePolicy::Strip);
    v.save(RecipePlace::Mine);
    let saved = v.saved_recipes().remove(0);
    let text = std::fs::read_to_string(&saved.path).unwrap();
    assert!(
        !text.contains("strip"),
        "the signature choice is asked each time: {text}"
    );
    v.delete_rule();
    assert!(v.recipe.rules.is_empty());
    v.load(&saved);
    assert_eq!(v.recipe.rules.len(), 1);
    assert_eq!(v.recipe.name, "Fix work email");
    assert!(v.recipe.options.signatures.is_none());
    assert_eq!(v.choices, Choices::default());
    assert!(
        !v.rules.can_restore(),
        "the removed rule belonged to the old recipe"
    );
}

#[test]
fn a_recipe_edit_makes_the_preview_out_of_date_and_blocks_apply() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.run = Run::Previewing {
        reply: Reply::ready(Ok(plan(0, &[], false))),
        since: Instant::now(),
        job: None,
        progress: None,
    };
    v.poll(None);
    assert_eq!(v.plan_state, PlanState::Current);
    assert!(v.can_apply());
    v.toggle_tags();
    assert_eq!(v.plan_state, PlanState::OutOfDate);
    assert!(!v.can_apply());
    assert!(v.apply_blocker().is_some());
}

#[test]
fn a_preview_that_changes_nothing_cannot_be_applied() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    let mut empty = plan(0, &[], false);
    empty.counts.commits_changed = 0;
    v.run = Run::Previewing {
        reply: Reply::ready(Ok(empty)),
        since: Instant::now(),
        job: None,
        progress: None,
    };
    v.poll(None);
    assert!(!v.can_apply());
}

#[test]
fn the_signature_choice_is_required_and_asked_not_defaulted() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    // The daemon refuses a preview while signed commits change and no choice was made.
    let refusal = CoreError::RewriteRefused(
        RewriteRefusal::SignatureChoiceNeeded,
        "3 signed commits would be rewritten; choose what happens to their signatures".into(),
    );
    v.run = Run::Previewing {
        reply: Reply::ready(Err(refusal)),
        since: Instant::now(),
        job: None,
        progress: None,
    };
    v.poll(None);
    assert!(!v.busy());
    assert_eq!(
        v.question,
        Some(Question::Signatures {
            count: Some(3),
            cursor: 0
        })
    );
    assert!(v.choices.signatures.is_none(), "no silent default");
    let shown = v.question_view().unwrap();
    // The three answers and the "Check signing" row.
    assert_eq!(shown.options.len(), 4);
    assert!(shown.prompt.contains('3'));
    // Esc leaves it unanswered: nothing was sent.
    v.on_key(key(KeyCode::Esc));
    assert!(v.question.is_none() && !v.busy());
    // Answering sends the preview again with the choice in the recipe.
    v.question = Some(Question::Signatures {
        count: None,
        cursor: 0,
    });
    v.on_key(key(KeyCode::Char('2')));
    assert_eq!(v.choices.signatures, Some(SignaturePolicy::ResignSigned));
    assert!(v.busy());
    assert_eq!(
        v.request_recipe().options.signatures,
        Some(SignaturePolicy::ResignSigned)
    );
    assert!(
        v.recipe.options.signatures.is_none(),
        "the recipe itself is untouched"
    );
}

#[test]
fn an_extract_rule_asks_how_before_anything_is_sent() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::MovePaths);
    v.edit_field(0, FieldInput::Choice(2));
    v.edit_field(1, FieldInput::Text("docs".into()));
    assert_eq!(v.validation(), Validation::NeedsExtractMode);
    assert!(v.can_preview());
    v.preview();
    assert!(!v.busy());
    assert_eq!(v.question, Some(Question::ExtractMode { cursor: 0 }));
    v.on_key(key(KeyCode::Down));
    v.on_key(key(KeyCode::Enter));
    assert_eq!(v.choices.extract_mode, Some(ExtractMode::InPlace));
    assert!(v.busy());
    let sent = v.request_recipe();
    match &sent.rules[0].kind {
        RuleKind::MovePaths(PathMove::ExtractSubdir { mode, .. }) => {
            assert_eq!(*mode, Some(ExtractMode::InPlace))
        }
        other => panic!("{other:?}"),
    }
    // Saved, the mode is gone again: it is asked each time.
    match &v.saved_recipe().rules[0].kind {
        RuleKind::MovePaths(PathMove::ExtractSubdir { mode, .. }) => assert_eq!(*mode, None),
        other => panic!("{other:?}"),
    }
}

#[test]
fn protected_branches_are_unchecked_by_default_and_check_all_and_uncheck_all_work() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.run = Run::Previewing {
        reply: Reply::ready(Ok(plan(
            0,
            &["refs/heads/main", "refs/heads/release"],
            true,
        ))),
        since: Instant::now(),
        job: None,
        progress: None,
    };
    v.poll(None);
    let entries = v.protected();
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().all(|e| !e.included && e.pushed));
    // One checkbox: Space on the Refs tab.
    v.focus = Focus::Preview;
    v.tab = PreviewTab::Refs;
    v.on_key(key(KeyCode::Char(' ')));
    assert_eq!(
        v.recipe.scope_options.include_protected,
        ["refs/heads/main".to_string()]
    );
    assert_eq!(
        v.plan_state,
        PlanState::OutOfDate,
        "the plan no longer matches"
    );
    // Check all, then uncheck all.
    v.on_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT));
    assert!(v.protected().iter().all(|e| e.included));
    v.on_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::ALT));
    assert!(v.protected().iter().all(|e| !e.included));
    assert!(v.recipe.scope_options.include_protected.is_empty());
}

#[test]
fn applying_a_pushed_plan_asks_for_confirmation_then_sends_it() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.run = Run::Previewing {
        reply: Reply::ready(Ok(plan(0, &[], true))),
        since: Instant::now(),
        job: None,
        progress: None,
    };
    v.poll(None);
    v.apply();
    assert!(matches!(v.run, Run::Applying { .. }));
    // The daemon answers that pushed refs need a go-ahead.
    v.run = Run::Applying {
        reply: Reply::ready(Err(CoreError::NeedsConfirmation(
            "2 refs are on origin".into(),
        ))),
        since: Instant::now(),
    };
    v.poll(None);
    assert!(!v.busy());
    assert_eq!(
        v.question,
        Some(Question::ConfirmPushed {
            prompt: "2 refs are on origin".into()
        })
    );
    // "Leave it" sends nothing; "Rewrite anyway" sends the apply.
    v.on_key(key(KeyCode::Char('2')));
    assert!(!v.busy());
    v.question = Some(Question::ConfirmPushed { prompt: "p".into() });
    v.on_key(key(KeyCode::Char('1')));
    assert!(matches!(v.run, Run::Applying { .. }));
}

#[test]
fn a_stale_refusal_marks_the_plan_stale() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.run = Run::Previewing {
        reply: Reply::ready(Ok(plan(0, &[], false))),
        since: Instant::now(),
        job: None,
        progress: None,
    };
    v.poll(None);
    v.run = Run::Applying {
        reply: Reply::ready(Err(CoreError::RewriteRefused(
            RewriteRefusal::Stale,
            String::new(),
        ))),
        since: Instant::now(),
    };
    v.poll(None);
    assert_eq!(v.plan_state, PlanState::Stale);
    assert!(!v.can_apply());
}

#[test]
fn a_finished_apply_shows_the_result_and_a_later_edit_leaves_it() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.run = Run::Applying {
        reply: Reply::ready(Ok(RewriteOutcome {
            run: RunId("run-1".into()),
            moved: Vec::new(),
            commits_changed: 4,
        })),
        since: Instant::now(),
    };
    v.choices.signatures = Some(SignaturePolicy::Strip);
    v.poll(None);
    assert!(matches!(v.run, Run::Done(_)));
    assert_eq!(
        v.choices,
        Choices::default(),
        "answers do not outlive the run"
    );
    v.toggle_tags();
    assert!(matches!(v.run, Run::Idle));
}

#[test]
fn a_canceled_preview_is_a_note_not_an_error() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.run = Run::Previewing {
        reply: Reply::ready(Err(CoreError::Canceled { job: 4 })),
        since: Instant::now(),
        job: Some(4),
        progress: None,
    };
    v.poll(None);
    assert!(v.notice.as_ref().is_some_and(|n| !n.error));
    assert!(v.plan.is_none());
}

#[test]
fn progress_reaches_the_running_preview() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.run = Run::Previewing {
        reply: Reply::pending(),
        since: Instant::now(),
        job: None,
        progress: None,
    };
    let report = RewriteProgress {
        phase: gitpoulpi_core::domain::rewrite::RewritePhase::Walking,
        done: 6,
        total: 10,
    };
    v.poll(Some((9, report)));
    match &v.run {
        Run::Previewing { job, progress, .. } => {
            assert_eq!(*job, Some(9));
            assert_eq!(*progress, Some(report));
        }
        _ => panic!("still previewing"),
    }
}

#[test]
fn esc_cancels_a_running_preview_and_a_second_esc_closes() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Close);
    v.run = Run::Previewing {
        reply: Reply::pending(),
        since: Instant::now(),
        job: None,
        progress: None,
    };
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Stay);
    v.run = Run::Idle;
    assert_eq!(v.on_key(key(KeyCode::Esc)), RewriteKeyOutcome::Close);
}

#[test]
fn esc_in_a_text_field_puts_the_old_value_back() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::DateShift);
    v.focus = Focus::Detail;
    v.on_key(key(KeyCode::Enter));
    for _ in 0..3 {
        v.on_key(key(KeyCode::Backspace));
    }
    chars(&mut v, "9h");
    assert_eq!(
        v.recipe.rules[0].kind,
        RuleKind::DateShift {
            by_seconds: 9 * 3600,
            apply_to: Default::default()
        }
    );
    v.on_key(key(KeyCode::Esc));
    assert!(v.editing.is_none());
    assert_eq!(
        v.recipe.rules[0].kind,
        RuleKind::DateShift {
            by_seconds: 3600,
            apply_to: Default::default()
        }
    );
}

#[test]
fn scope_keys_change_the_kind_and_the_tags() {
    let (mut v, _q, _t) = view();
    v.focus = Focus::Scope;
    assert_eq!(v.scope_kind(), ScopeKind::CurrentBranch);
    v.on_key(key(KeyCode::Right));
    assert_eq!(v.scope_kind(), ScopeKind::AllLocal);
    v.on_key(key(KeyCode::Left));
    assert_eq!(v.scope_kind(), ScopeKind::CurrentBranch);
    assert!(v.recipe.scope_options.include_tags);
    v.on_key(key(KeyCode::Char(' ')));
    assert!(!v.recipe.scope_options.include_tags);
}

#[test]
fn tab_cycles_the_panes_and_skips_the_detail_when_there_is_no_rule() {
    let (mut v, _q, _t) = view();
    v.focus = Focus::Scope;
    v.on_key(key(KeyCode::Tab));
    assert_eq!(v.focus, Focus::Rules);
    v.on_key(key(KeyCode::Tab));
    assert_eq!(v.focus, Focus::Preview, "no rule, no detail");
    v.add_rule(RuleTemplate::DateShift);
    v.focus = Focus::Rules;
    v.on_key(key(KeyCode::Tab));
    assert_eq!(v.focus, Focus::Detail);
}

#[test]
fn a_rule_needing_a_missing_tool_is_greyed_with_its_hint() {
    use gitpoulpi_core::domain::rewrite::{OsFamily, RewriteTools, Tool, ToolCheck, ToolStatus};
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::LfsImport);
    v.add_rule(RuleTemplate::DateShift);
    v.tools = ToolsState::Known(RewriteTools {
        external_scripts: None,
        signing_format: Default::default(),
        tools: vec![ToolCheck::new(
            Tool::GitLfs,
            ToolStatus::Missing,
            OsFamily::current(),
        )],
    });
    assert_eq!(v.missing_tools(0).len(), 1);
    assert!(v.missing_tools(1).is_empty());
    v.edit_field(0, FieldInput::Text("*.bin".into()));
    v.rules.cursor = 0;
    v.preview();
    assert!(
        !v.busy(),
        "a missing tool stops the preview before it is sent"
    );
    assert!(v.notice.as_ref().is_some_and(|n| n.error));
}

#[test]
fn rows_are_asked_page_by_page_and_only_for_the_current_run() {
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.run = Run::Previewing {
        reply: Reply::ready(Ok(plan(0, &[], false))),
        since: Instant::now(),
        job: None,
        progress: None,
    };
    v.poll(None);
    // The first page was asked for when the plan arrived.
    assert!(v.rows.pending.is_some());
    let row = |n: u8| gitpoulpi_core::domain::rewrite::CommitChange {
        old: Oid([n; 20]),
        new: Some(Oid([n + 1; 20])),
        fields: Vec::new(),
        paths: Vec::new(),
        pushed: false,
    };
    // A page of another run is dropped.
    v.rows.pending = Some(Reply::ready(Ok(RowsPage {
        run: RunId("other".into()),
        filter: RowFilter::Changed,
        offset: 0,
        total: 2,
        rows: vec![row(1)],
    })));
    v.poll(None);
    assert!(v.rows.loaded.is_empty());
    v.rows.pending = Some(Reply::ready(Ok(RowsPage {
        run: RunId("run-1".into()),
        filter: RowFilter::Changed,
        offset: 0,
        total: 2,
        rows: vec![row(1), row(2)],
    })));
    v.poll(None);
    assert_eq!(v.rows.loaded.len(), 2);
    assert_eq!(v.rows.total, 2);
    // Everything is loaded: nothing more is asked.
    v.want_rows(1);
    assert!(v.rows.pending.is_none());
}

mod content;
mod host;
mod identities;
mod paths;
mod purge;
mod push;
mod render;
mod rules_ai;
mod script;
mod script_ai;
mod script_external;
mod secrets;
mod size;

#[test]
fn check_signing_shows_the_time_the_estimate_and_why_it_failed() {
    use gitpoulpi_core::domain::rewrite::{SigningCheck, SigningFormat, Tool};
    let (mut v, _q, _t) = view();
    v.question = Some(Question::Signatures {
        count: Some(40),
        cursor: 0,
    });
    // Nothing asked yet: the prompt is the plain question.
    assert!(!v.question_view().unwrap().prompt.contains("Signing works"));

    // A working setup: the measured time and 40 signatures of it.
    v.sign_check = Some(SigningCheck {
        format: SigningFormat::Ssh,
        tool: Tool::SshKeygen,
        hint_key: "tool_hint_ssh_keygen_linux".into(),
        reason: None,
        detail: String::new(),
        millis: 250,
    });
    let prompt = v.question_view().unwrap().prompt;
    assert!(
        prompt.contains("Signing works with ssh-keygen: 250 ms"),
        "{prompt}"
    );
    assert!(prompt.contains("about 10.0 s"), "{prompt}");

    // A missing tool names the problem and carries the install hint; a
    // broken key only says what the program said.
    v.sign_check = Some(SigningCheck {
        format: SigningFormat::Ssh,
        tool: Tool::SshKeygen,
        hint_key: "tool_hint_ssh_keygen_linux".into(),
        reason: Some(RewriteRefusal::ToolMissing),
        detail: "ssh-keygen was not found".into(),
        millis: 0,
    });
    let prompt = v.question_view().unwrap().prompt;
    assert!(
        prompt.contains("Signing does not work: ssh-keygen was not found"),
        "{prompt}"
    );
    assert!(prompt.len() > "Signing does not work: ssh-keygen was not found".len() + 60);
    v.sign_check = Some(SigningCheck {
        format: SigningFormat::Ssh,
        tool: Tool::SshKeygen,
        hint_key: String::new(),
        reason: Some(RewriteRefusal::SigningFailed),
        detail: "Load key: No such file".into(),
        millis: 0,
    });
    let prompt = v.question_view().unwrap().prompt;
    assert!(
        prompt.ends_with("Signing does not work: Load key: No such file"),
        "{prompt}"
    );
}

mod lfs;
mod trailers;

/// A repository that signs with ssh keys needs `ssh-keygen` to re-sign, not
/// `gpg`: the view takes the format from the tools answer.
#[test]
fn re_signing_in_an_ssh_repository_asks_for_ssh_keygen_not_gpg() {
    use gitpoulpi_core::domain::rewrite::{
        OsFamily, RewriteTools, SigningFormat, Tool, ToolCheck, ToolStatus,
    };
    let (mut v, _q, _t) = view();
    with_identity_rule(&mut v);
    v.choices.signatures = Some(SignaturePolicy::ResignAll);
    let os = OsFamily::current();
    v.tools_wait = Some(Reply::ready(Ok(RewriteTools {
        external_scripts: None,
        signing_format: SigningFormat::Ssh,
        tools: vec![
            ToolCheck::new(Tool::Gpg, ToolStatus::Missing, os),
            ToolCheck::new(
                Tool::SshKeygen,
                ToolStatus::Installed {
                    version: String::new(),
                },
                os,
            ),
        ],
    })));
    v.poll(None);
    assert!(v.missing_tools_for_run().is_empty());
}
