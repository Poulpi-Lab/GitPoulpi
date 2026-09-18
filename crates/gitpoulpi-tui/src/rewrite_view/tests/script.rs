//! The Script rule (slice S10, TUI): the editor overlay with its template,
//! the live check, the parameter rows, "Test on selection", the files, the
//! greyed kind of a build without scripts and the `ScriptFailed` refusal.
//! The daemon's answers are put in by hand.

use super::render::{paint, rect_of, text};
use super::*;
use crate::daemon_link::Reply;
use crate::rewrite_view::script::{self, CheckState, TestState};
use gitpoulpi_core::domain::rewrite::script::{parse_header, ScriptCheck, ScriptTestRow};
use gitpoulpi_core::domain::rewrite::{FieldChange, RewriteField, ScriptSource};
use std::time::Duration;

const W: u16 = 120;
const H: u16 = 36;

fn oid(n: u8) -> Oid {
    Oid([n; 20])
}

/// A workbench with a Script rule just added through the Add menu, so its
/// editor is open on the starter template, and two commits to test on.
pub(super) fn script_view() -> (RewriteView, RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = view();
    v.scope_ctx.test_commits = vec![oid(1), oid(2)];
    v.scope_ctx.test_from_selection = true;
    v.open_add_menu();
    let at = RuleTemplate::ALL
        .iter()
        .position(|k| *k == RuleTemplate::Script)
        .unwrap();
    v.popup_pick(at);
    assert!(v.script.editor.is_some(), "the editor opens with the rule");
    (v, q, t)
}

pub(super) fn script_text(v: &RewriteView) -> String {
    match &v.recipe.rules[v.rules.cursor].kind {
        RuleKind::Script(s) => match &s.source {
            ScriptSource::Inline { text } => text.clone(),
            ScriptSource::File { path } => path.clone(),
        },
        other => panic!("{other:?}"),
    }
}

pub(super) fn set_script(v: &mut RewriteView, text: &str) {
    v.set_script_text(text.to_string());
}

/// The daemon's check, answered for the script on screen.
pub(super) fn answer_check(v: &mut RewriteView, problems: &[&str]) {
    let text = script_text(v);
    let check = ScriptCheck {
        header: parse_header(&text).unwrap_or_default(),
        functions: if text.contains("rewrite_commit") {
            vec!["rewrite_commit".into()]
        } else {
            Vec::new()
        },
        problems: problems.iter().map(|p| p.to_string()).collect(),
        ..Default::default()
    };
    let key = v.script_key().unwrap();
    v.script.check.sent(key, Reply::ready(Ok(check)));
    v.poll(None);
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn change(field: RewriteField, old: &str, new: &str) -> FieldChange {
    FieldChange {
        field,
        old: old.into(),
        new: new.into(),
    }
}

fn tested(v: &mut RewriteView, rows: Vec<ScriptTestRow>) {
    let count = v.scope_ctx.test_commits.len();
    v.script.test = TestState::Running {
        reply: Reply::ready(Ok(rows)),
        since: Instant::now(),
        count,
    };
    v.poll(None);
}

const WITH_PARAMS: &str = "\
--! name: Rename
--! param who: text \"Name to replace\"
--! param limit: number \"How many commits\" = 5
--! param keep: bool \"Keep the committer\" = true
function rewrite_commit(c, ctx) end
";

// ----------------------------------------------------- template and check

#[test]
fn a_new_script_rule_opens_the_editor_on_a_starter_template() {
    let (mut v, _q, _t) = script_view();
    let starter = script_text(&v);
    assert!(starter.starts_with("--! name:"), "{starter}");
    assert!(starter.contains("function rewrite_commit(c, ctx)"));
    answer_check(&mut v, &[]);
    let s = text(&v);
    assert!(s.contains("OK: rewrite_commit found"), "{s}");
    assert!(s.contains("Load from file...") && s.contains("Save to file..."));
    assert!(s.contains("Test on selection"));
    assert!(
        s.contains("Text added to each message"),
        "the param row\n{s}"
    );
    assert!(!s.contains("Esc"), "no Esc hint is drawn\n{s}");
    insta::assert_snapshot!("editor_template", s);
}

#[test]
fn a_syntax_error_shows_its_line_under_the_script_and_blocks_the_test() {
    let (mut v, _q, _t) = script_view();
    set_script(
        &mut v,
        "function rewrite_commit(c, ctx)\n  return {\n  )\nend\n",
    );
    answer_check(&mut v, &["script:3: unexpected symbol near ')'"]);
    let s = text(&v);
    assert!(s.contains("script:3: unexpected symbol near ')'"), "{s}");
    assert!(!s.contains("OK: rewrite_commit"), "{s}");
    insta::assert_snapshot!("editor_problem", s);
    assert!(v.test_blocker().is_some());
    v.start_script_test();
    assert!(matches!(v.script.test, TestState::Idle), "nothing is sent");
    // The detail pane says the same once the editor is closed.
    v.close_script_editor();
    let s = text(&v);
    assert!(s.contains("script:3:"), "{s}");
}

#[test]
fn the_detail_pane_shows_the_check_when_the_editor_is_closed() {
    let (mut v, _q, _t) = script_view();
    answer_check(&mut v, &[]);
    v.close_script_editor();
    let s = text(&v);
    assert!(s.contains("OK: rewrite_commit found"), "{s}");
    assert!(s.contains("Enter on the script opens the editor"), "{s}");
    // Enter on the script field opens it again.
    let at = v
        .detail
        .fields
        .iter()
        .position(|f| f.id == FieldId::ScriptText)
        .unwrap();
    v.detail.cursor = at;
    v.focus = Focus::Detail;
    v.on_key(key(KeyCode::Enter));
    assert!(v.script.editor.is_some());
}

#[test]
fn typing_goes_into_the_rule_at_every_key() {
    let (mut v, _q, _t) = script_view();
    v.on_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));
    for c in "x = 1".chars() {
        v.on_key(key(KeyCode::Char(c)));
    }
    assert_eq!(script_text(&v), "x = 1");
    v.on_key(key(KeyCode::Enter));
    assert_eq!(script_text(&v), "x = 1\n");
    assert_eq!(v.rule_rows()[0].detail, "x = 1");
}

// ---------------------------------------------------------- debounce, stale

#[test]
fn the_check_waits_for_a_pause_and_runs_one_at_a_time() {
    let t0 = Instant::now();
    let ms = |n| t0 + Duration::from_millis(n);
    let mut c = CheckState::default();
    assert!(!c.step(Some(1), ms(0)));
    assert!(!c.step(Some(1), ms(200)));
    // Another key press moves the due time.
    assert!(!c.step(Some(2), ms(250)));
    assert!(!c.step(Some(2), ms(500)));
    assert_eq!(c.next_deadline(), Some(ms(250) + script::CHECK_DEBOUNCE));
    assert!(c.step(Some(2), ms(551)));
    assert!(!c.step(Some(2), ms(552)), "sent once");
    // A request is in flight: the next text waits for its answer.
    c.sent(2, Reply::pending());
    assert!(!c.step(Some(3), ms(600)));
    assert!(!c.step(Some(3), ms(1_000)));
    assert_eq!(c.next_deadline(), None, "the answer wakes the loop");
    assert!(c.busy());
    // No script under the cursor: nothing scheduled.
    assert!(!c.step(None, ms(1_100)));
    assert!(c.next_deadline().is_none());
}

#[test]
fn an_answer_for_an_older_text_is_never_shown() {
    let (mut v, _q, _t) = script_view();
    let old = v.script_key().unwrap();
    let check = ScriptCheck {
        header: Default::default(),
        functions: vec!["rewrite_commit".into()],
        problems: Vec::new(),
        ..Default::default()
    };
    set_script(&mut v, "-- changed\nfunction rewrite_commit(c, ctx) end\n");
    v.script.check.sent(old, Reply::ready(Ok(check)));
    v.poll(None);
    assert_eq!(v.check_view(), CheckView::Checking);
    assert!(text(&v).contains("Checking..."));
    // The answer for what is on screen is.
    answer_check(&mut v, &[]);
    assert_eq!(v.check_view(), CheckView::Ok("rewrite_commit".into()));
}

#[test]
fn polling_sends_the_check_after_the_pause_not_before() {
    let (mut v, _q, _t) = script_view();
    let t0 = Instant::now();
    v.poll_script(t0);
    assert!(!v.script.check.busy() || v.script.check.next_deadline().is_some());
    // Within the debounce: still nothing in flight.
    v.poll_script(t0 + Duration::from_millis(100));
    assert!(v.script.check.next_deadline().is_some());
    // Past it: the request is in flight (no deadline left to wait for).
    v.poll_script(t0 + Duration::from_secs(5));
    assert!(v.script.check.next_deadline().is_none());
    assert!(v.script.check.busy());
}

// -------------------------------------------------------------- parameters

#[test]
fn parameters_are_typed_rows_with_their_labels_and_defaults() {
    let (mut v, _q, _t) = script_view();
    set_script(&mut v, WITH_PARAMS);
    answer_check(&mut v, &[]);
    let s = text(&v);
    for label in ["Name to replace", "How many commits", "Keep the committer"] {
        assert!(s.contains(label), "{label}\n{s}");
    }
    assert!(s.contains("[x]"), "the bool defaults to on\n{s}");
    assert!(s.contains('5'), "the number's default\n{s}");
    insta::assert_snapshot!("editor_params", s);
    // The text parameter has no default: it is required, and Preview says so.
    let who = v
        .detail
        .fields
        .iter()
        .find(|f| f.id == FieldId::ScriptParam(0))
        .unwrap();
    assert_eq!(who.error.as_deref(), Some("A value is needed."));
    assert!(matches!(v.validation(), Validation::Invalid { .. }));
    assert!(v.apply_blocker().unwrap().contains("who"));
}

#[test]
fn typing_a_parameter_stores_it_and_clearing_it_restores_the_default() {
    let (mut v, _q, _t) = script_view();
    set_script(&mut v, WITH_PARAMS);
    answer_check(&mut v, &[]);
    // Tab: editor -> who. A text parameter takes the keys at once.
    v.on_key(key(KeyCode::Tab));
    assert_eq!(
        v.script.editor.as_ref().unwrap().focus,
        ScriptFocus::Param(0)
    );
    for c in "Alex".chars() {
        v.on_key(key(KeyCode::Char(c)));
    }
    let RuleKind::Script(s) = &v.recipe.rules[0].kind else {
        panic!()
    };
    assert_eq!(s.params.get("who").map(String::as_str), Some("Alex"));
    assert!(v.validation().is_ok());
    // The number refuses a word and keeps the draft on screen.
    v.on_key(key(KeyCode::Tab));
    v.on_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));
    for c in "many".chars() {
        v.on_key(key(KeyCode::Char(c)));
    }
    let limit = v
        .detail
        .fields
        .iter()
        .find(|f| f.id == FieldId::ScriptParam(1))
        .unwrap();
    assert_eq!(limit.error.as_deref(), Some("Write a number."));
    // Emptied, it is the header's default again.
    v.on_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL));
    v.on_key(key(KeyCode::Backspace));
    let RuleKind::Script(s) = &v.recipe.rules[0].kind else {
        panic!()
    };
    assert!(!s.params.contains_key("limit"));
    // The bool flips with Space and is saved as true/false.
    v.on_key(key(KeyCode::Tab));
    assert_eq!(
        v.script.editor.as_ref().unwrap().focus,
        ScriptFocus::Param(2)
    );
    v.on_key(key(KeyCode::Char(' ')));
    let RuleKind::Script(s) = &v.recipe.rules[0].kind else {
        panic!()
    };
    assert_eq!(s.params.get("keep").map(String::as_str), Some("false"));
}

#[test]
fn a_script_from_a_file_gets_its_parameter_rows_from_the_check() {
    let (mut v, _q, _t) = script_view();
    v.close_script_editor();
    let source = v
        .detail
        .fields
        .iter()
        .position(|f| f.id == FieldId::ScriptSource)
        .unwrap();
    v.edit_field(source, FieldInput::Choice(1));
    let path = v
        .detail
        .fields
        .iter()
        .position(|f| f.id == FieldId::ScriptPath)
        .unwrap();
    v.edit_field(path, FieldInput::Text("tools/rename.lua".into()));
    assert!(
        v.param_fields().is_empty(),
        "nothing known before the check"
    );
    let key = v.script_key().unwrap();
    let check = ScriptCheck {
        header: parse_header(WITH_PARAMS).unwrap(),
        functions: vec!["rewrite_commit".into()],
        problems: Vec::new(),
        ..Default::default()
    };
    v.script.check.sent(key, Reply::ready(Ok(check)));
    v.poll(None);
    assert_eq!(v.param_fields().len(), 3);
    assert!(text(&v).contains("Name to replace"));
}

// -------------------------------------------------------------------- test

#[test]
fn test_on_selection_runs_on_the_selected_commits_and_shows_old_and_new() {
    let (mut v, _q, _t) = script_view();
    answer_check(&mut v, &[]);
    assert_eq!(v.test_target(), "2 selected commits");
    v.on_script_press(RewriteHit::ScriptTest, 0, 0);
    assert!(matches!(v.script.test, TestState::Running { count: 2, .. }));
    let s = text(&v);
    assert!(s.contains("Testing 2 commits..."), "{s}");
    assert!(s.contains("Cancel"), "{s}");
    tested(
        &mut v,
        vec![
            ScriptTestRow {
                commit: Oid([0xab; 20]),
                fields: vec![change(RewriteField::AuthorName, "alex", "ALEX")],
                log: vec!["renamed one".into()],
            },
            ScriptTestRow {
                commit: Oid([0xcd; 20]),
                fields: Vec::new(),
                log: Vec::new(),
            },
        ],
    );
    let s = text(&v);
    assert!(s.contains("2 commits tested, 1 would change."), "{s}");
    assert!(s.contains("abababa"), "the short id\n{s}");
    assert!(s.contains("author name: alex -> ALEX"), "{s}");
    assert!(s.contains("log: renamed one"), "{s}");
    assert!(s.contains("no change"), "{s}");
    insta::assert_snapshot!("test_results", s);
    // Nothing was changed in the recipe by testing.
    assert!(v.plan.is_none());
}

#[test]
fn without_a_selection_the_newest_commits_are_tested() {
    let (mut v, _q, _t) = script_view();
    v.scope_ctx.test_from_selection = false;
    v.scope_ctx.test_commits = (1..=20).map(oid).collect();
    assert_eq!(v.test_target(), "the 20 newest commits");
    v.scope_ctx.test_commits.clear();
    answer_check(&mut v, &[]);
    assert_eq!(
        v.test_blocker().as_deref(),
        Some("No commit to test: the Log shows none.")
    );
}

#[test]
fn a_script_that_fails_names_the_commit_and_the_reason() {
    let (mut v, _q, _t) = script_view();
    answer_check(&mut v, &[]);
    v.script.test = TestState::Running {
        reply: Reply::ready(Err(CoreError::RewriteRefused(
            RewriteRefusal::ScriptFailed,
            format!(
                "{}: script:2: attempt to index a nil value",
                oid(0xab).to_hex()
            ),
        ))),
        since: Instant::now(),
        count: 2,
    };
    v.poll(None);
    let s = text(&v);
    assert!(s.contains("Failed on commit abababa"), "{s}");
    assert!(
        s.contains("attempt to index") && s.contains("a nil value"),
        "{s}"
    );
}

#[test]
fn cancel_forgets_the_answer_when_it_comes() {
    let (mut v, _q, _t) = script_view();
    answer_check(&mut v, &[]);
    v.on_key(key(KeyCode::F(5)));
    assert!(matches!(v.script.test, TestState::Running { .. }));
    v.on_script_press(RewriteHit::ScriptCancel, 0, 0);
    assert!(matches!(v.script.test, TestState::Canceled));
    v.poll(None);
    assert!(text(&v).contains("Test canceled."));
}

#[test]
fn a_test_needs_its_text_parameters() {
    let (mut v, _q, _t) = script_view();
    set_script(&mut v, WITH_PARAMS);
    answer_check(&mut v, &[]);
    let why = v.test_blocker().unwrap();
    assert!(why.contains("Fill in the parameters first"), "{why}");
    assert!(why.contains("who"), "{why}");
}

// ------------------------------------------------------------- mouse parity

#[test]
fn every_click_has_its_key() {
    // Test: the button and F5.
    let (mut by_mouse, _q1, _t1) = script_view();
    answer_check(&mut by_mouse, &[]);
    let (mut by_key, _q2, _t2) = script_view();
    answer_check(&mut by_key, &[]);
    by_mouse.on_press(RewriteHit::ScriptTest, 0, 0);
    by_key.on_key(key(KeyCode::F(5)));
    assert!(matches!(by_mouse.script.test, TestState::Running { .. }));
    assert!(matches!(by_key.script.test, TestState::Running { .. }));
    // Close: the cross, an outside click and Esc.
    for how in 0..3 {
        let (mut v, _q, _t) = script_view();
        paint(&v, W, H, None);
        match how {
            0 => {
                v.on_press(RewriteHit::ScriptClose, 0, 0);
            }
            1 => {
                assert_eq!(v.outside_press(), RewriteKeyOutcome::Stay);
            }
            _ => {
                v.on_key(key(KeyCode::Esc));
            }
        }
        assert!(v.script.editor.is_none(), "way {how}");
        assert_eq!(v.recipe.rules.len(), 1, "the rule stays");
    }
    // Focus: a click on a part and Tab reach the same place.
    let (mut v, _q, _t) = script_view();
    v.on_press(RewriteHit::ScriptPath, 0, 0);
    assert_eq!(v.script.editor.as_ref().unwrap().focus, ScriptFocus::Path);
    let (mut w, _q, _t) = script_view();
    while w.script.editor.as_ref().unwrap().focus != ScriptFocus::Path {
        w.on_key(key(KeyCode::Tab));
    }
    // Load and Save: the buttons and Enter on them.
    for (hit, focus) in [
        (RewriteHit::ScriptLoad, ScriptFocus::Load),
        (RewriteHit::ScriptSave, ScriptFocus::Save),
    ] {
        let (mut by_mouse, _q, _t) = script_view();
        let (mut by_key, _q2, _t2) = script_view();
        by_mouse.on_press(hit, 0, 0);
        while by_key.script.editor.as_ref().unwrap().focus != focus {
            by_key.on_key(key(KeyCode::Tab));
        }
        by_key.on_key(key(KeyCode::Enter));
        assert_eq!(
            by_mouse.script.editor.as_ref().unwrap().note,
            by_key.script.editor.as_ref().unwrap().note,
            "{hit:?}"
        );
    }
}

#[test]
fn the_overlay_owns_the_pointer_and_every_target_has_a_glyph() {
    let (mut v, _q, _t) = script_view();
    answer_check(&mut v, &[]);
    tested(
        &mut v,
        vec![ScriptTestRow {
            commit: oid(0xab),
            fields: vec![change(RewriteField::Message, "a", "b")],
            log: Vec::new(),
        }],
    );
    for (w, h) in [(W, H), (76, 30)] {
        let (_, hits, buf) = paint(&v, w, h, None);
        // The cross and the parts come first, then the panel and the shield;
        // the workbench behind them is registered after, so unreachable.
        assert_eq!(hits.first().unwrap().1, RewriteHit::ScriptClose);
        let shield = hits
            .iter()
            .position(|(_, h)| *h == RewriteHit::ScriptShield)
            .unwrap();
        assert_eq!(hits[shield - 1].1, RewriteHit::ScriptPanel);
        assert!(
            hits[..=shield].iter().all(|(_, h)| !matches!(
                h,
                RewriteHit::Preview | RewriteHit::Apply | RewriteHit::Field(_)
            )),
            "{hits:?}"
        );
        let hits: Vec<_> = hits.into_iter().take(shield + 1).collect();
        for (r, hit) in &hits {
            if matches!(hit, RewriteHit::ScriptPanel | RewriteHit::ScriptShield) {
                continue;
            }
            let blank = (r.x..r.right()).all(|x| {
                buf[(x, r.y)].symbol().trim().is_empty() && buf[(x, r.y)].bg == crate::th().ui.panel
            });
            assert!(
                !blank,
                "{hit:?} at {r:?} has nothing drawn under it ({w}x{h})"
            );
        }
        for hit in [
            RewriteHit::ScriptEditor,
            RewriteHit::ScriptPath,
            RewriteHit::ScriptLoad,
            RewriteHit::ScriptSave,
            RewriteHit::ScriptTest,
            RewriteHit::ScriptRow(0),
        ] {
            rect_of(&hits, hit);
        }
    }
}

#[test]
fn a_click_in_the_text_places_the_caret() {
    let (mut v, _q, _t) = script_view();
    set_script(&mut v, "one\ntwo\nthree\n");
    let (_, hits, _) = paint(&v, W, H, None);
    let area = rect_of(&hits, RewriteHit::ScriptEditor);
    // Second line, on the "o": the caret goes before it.
    v.on_press(RewriteHit::ScriptEditor, area.x + 2, area.y + 1);
    for c in "X".chars() {
        v.on_key(key(KeyCode::Char(c)));
    }
    assert_eq!(script_text(&v), "one\ntwXo\nthree\n");
}

// ------------------------------------------------------------------- files

#[test]
fn load_and_save_stay_inside_the_repository() {
    let (mut v, _q, tmp) = script_view();
    let root = tmp.path().join("repo");
    std::fs::create_dir_all(root.join("tools")).unwrap();
    std::fs::write(root.join("tools/rename.lua"), "-- from a file\n").unwrap();
    v.script.editor.as_mut().unwrap().path = "tools/rename.lua".into();
    v.load_script_file();
    assert_eq!(script_text(&v), "-- from a file\n");
    assert!(v
        .script
        .editor
        .as_ref()
        .unwrap()
        .note
        .as_ref()
        .is_some_and(|n| !n.error && n.text.contains("Loaded tools/rename.lua")));
    // Save writes the editor's text, making the folders.
    set_script(&mut v, "-- saved\n");
    v.script.editor.as_mut().unwrap().path = "out/dir/script.lua".into();
    v.save_script_file();
    assert_eq!(
        std::fs::read_to_string(root.join("out/dir/script.lua")).unwrap(),
        "-- saved\n"
    );
    // A path that leaves the repository is refused, both ways.
    for bad in ["../outside.lua", "/etc/passwd", "", "a/../../b.lua"] {
        v.script.editor.as_mut().unwrap().path = bad.into();
        v.save_script_file();
        let note = v.script.editor.as_ref().unwrap().note.clone().unwrap();
        assert!(
            note.error && note.text.contains("inside the repository"),
            "{bad}"
        );
        v.load_script_file();
        assert!(
            v.script
                .editor
                .as_ref()
                .unwrap()
                .note
                .as_ref()
                .unwrap()
                .error
        );
    }
    assert!(!tmp.path().join("outside.lua").exists());
    // A file that is not there says why.
    v.script.editor.as_mut().unwrap().path = "missing.lua".into();
    v.load_script_file();
    assert!(
        v.script
            .editor
            .as_ref()
            .unwrap()
            .note
            .as_ref()
            .unwrap()
            .error
    );
}

// -------------------------------------------------------- feature off

#[test]
fn a_build_without_scripts_greys_the_kind_and_says_why() {
    script::set_scripts_built(false);
    let (mut v, _q, _t) = view();
    v.open_add_menu();
    let rows = v.popup_rows();
    let at = RuleTemplate::ALL
        .iter()
        .position(|k| *k == RuleTemplate::Script)
        .unwrap();
    v.popup_set_cursor(at);
    assert!(rows[at].disabled.is_some());
    assert!(rows
        .iter()
        .enumerate()
        .all(|(i, r)| i == at || r.disabled.is_none()));
    let (s, _, _) = paint(&v, W, H, None);
    assert!(s.contains("not built into this version"), "{s}");
    // Picking it adds nothing and says why.
    v.popup_pick(at);
    assert!(v.recipe.rules.is_empty());
    assert!(v.popup.is_none());
    let n = v.notice.clone().unwrap();
    assert!(
        n.error && n.text.contains("not built into this version"),
        "{n:?}"
    );
    // The palette row opens the workbench without the rule.
    let v = view().0.with_rule(RuleTemplate::Script);
    assert!(v.recipe.rules.is_empty());
    assert!(v.notice.is_some());
    // A script rule from a loaded recipe cannot open its editor.
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::Script);
    v.open_script_editor();
    assert!(v.script.editor.is_none());
    assert!(text(&v).contains("Scripts are not built"), "{}", text(&v));
    script::set_scripts_built(true);
}

// ----------------------------------------------------------------- refusal

#[test]
fn a_script_failed_refusal_names_the_commit_and_the_reason() {
    let (mut v, _q, _t) = script_view();
    v.close_script_editor();
    super::content::refused(
        &mut v,
        RewriteRefusal::ScriptFailed,
        &format!(
            "{}: script:4: attempt to perform arithmetic on a nil value",
            oid(0xab).to_hex()
        ),
    );
    let s = text(&v);
    assert!(s.contains("a script failed"), "{s}");
    assert!(
        s.contains("abababa: script:4: attempt to perform arithmetic on a nil value"),
        "{s}"
    );
    assert!(s.contains("Fix the script"), "{s}");
    assert!(
        !s.contains(&oid(0xab).to_hex()),
        "the full id is not shown\n{s}"
    );
    insta::assert_snapshot!("refusal_script", s);
    // A script that does not compile has no commit: its reason is the line.
    super::content::refused(
        &mut v,
        RewriteRefusal::ScriptFailed,
        "script:3: '=' expected",
    );
    let s = text(&v);
    assert!(s.contains("script:3: '=' expected"), "{s}");
    // The footer notice says it too, with the short id.
    super::content::refused(
        &mut v,
        RewriteRefusal::ScriptFailed,
        &format!("{}: boom", oid(0xcd).to_hex()),
    );
    assert!(v.notice.as_ref().unwrap().text.contains("cdcdcdc: boom"));
}

#[test]
fn a_preview_with_a_missing_parameter_is_refused_before_it_is_sent() {
    let (mut v, _q, _t) = script_view();
    set_script(&mut v, WITH_PARAMS);
    v.close_script_editor();
    v.preview();
    assert!(!v.busy());
    let n = v.notice.clone().unwrap();
    assert!(n.error && n.text.contains("who"), "{n:?}");
}

#[test]
fn a_small_window_never_paints_past_the_overlay() {
    let (mut v, _q, _t) = script_view();
    set_script(&mut v, WITH_PARAMS);
    answer_check(
        &mut v,
        &[
            "syntax error: script:2: first problem that is long enough to need wrapping",
            "script:3: second",
            "script:4: third",
            "script:5: fourth",
        ],
    );
    for (w, h) in [(58, 17), (60, 18), (76, 24), (100, 20), (120, 17), (58, 40)] {
        let (_, _, buf) = paint(&v, w, h, None);
        // The overlay's bottom border is still the panel's corner.
        assert_eq!(
            buf[(2, h - 2)].symbol(),
            "\u{2570}",
            "{w}x{h}\n{}",
            crate::dialog_kit::buffer_text(&buf)
        );
    }
}
