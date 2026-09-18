//! The path rules in the workbench (S20): the extract-mode question says what
//! each way does, a clone or submodule answer asks where the new repository
//! goes before anything is previewed, and the apply carries the folder.

use super::*;
use crate::folder_picker::PickerOutcome;

/// A view with an extraction of `docs` waiting for its mode.
fn extracting() -> (RewriteView, crate::daemon_link::RawQueue, tempfile::TempDir) {
    let (mut v, q, t) = view();
    v.add_rule(RuleTemplate::MovePaths);
    v.edit_field(0, FieldInput::Choice(2));
    v.edit_field(1, FieldInput::Text("docs".into()));
    v.preview();
    assert_eq!(v.question, Some(Question::ExtractMode { cursor: 0 }));
    (v, q, t)
}

#[test]
fn the_question_says_what_the_highlighted_way_does() {
    let (mut v, _q, _t) = extracting();
    let prompt = |v: &RewriteView| v.question_view().unwrap().prompt;
    assert!(prompt(&v).contains("This repository is not touched"));
    v.on_key(key(KeyCode::Down));
    assert!(prompt(&v).contains("rewritten to hold only this folder"));
    v.on_key(key(KeyCode::Down));
    assert!(prompt(&v).contains("submodule"));
    assert!(prompt(&v).contains("is not rewritten"));
    // The base question stays in front of every note.
    assert!(prompt(&v).starts_with("How should the folder be extracted?"));
}

#[test]
fn a_clone_answer_asks_where_before_anything_is_sent_and_escape_asks_again() {
    let (mut v, _q, _t) = extracting();
    v.on_key(key(KeyCode::Enter));
    assert!(v.has_target_picker(), "the folder picker is up");
    assert!(!v.busy(), "nothing was previewed yet");
    assert_eq!(v.choices.extract_mode, None);
    // The picker owns the keys: Esc closes it, nothing was answered.
    v.on_key(key(KeyCode::Esc));
    assert!(!v.has_target_picker());
    assert_eq!(v.choices.extract_mode, None);
    assert!(!v.busy());
    v.preview();
    assert_eq!(v.question, Some(Question::ExtractMode { cursor: 0 }));
}

#[test]
fn the_new_repository_is_a_folder_named_after_the_extracted_one() {
    let (mut v, _q, tmp) = extracting();
    v.on_key(key(KeyCode::Enter));
    let out = tmp.path().join("out");
    std::fs::create_dir_all(&out).unwrap();
    v.on_target_outcome(PickerOutcome::Choose(out.clone()));
    assert!(!v.has_target_picker());
    assert_eq!(v.choices.extract_mode, Some(ExtractMode::Clone));
    assert!(v.busy(), "the preview was sent with the answer");
    let summary = v.extract_summary();
    assert!(summary[0].contains("This repository is not changed"));
    assert!(summary[1].contains("docs"), "{summary:?}");
    assert_eq!(v.extract_target(), Some(out.join("docs")));

    // A folder that is taken and not empty gets the next free name.
    std::fs::create_dir_all(out.join("docs")).unwrap();
    std::fs::write(out.join("docs/x"), "x").unwrap();
    assert_eq!(
        super::super::paths::free_target(&out, "docs"),
        out.join("docs-2")
    );
    // An empty one is fine to use.
    std::fs::remove_file(out.join("docs/x")).unwrap();
    assert_eq!(
        super::super::paths::free_target(&out, "docs"),
        out.join("docs")
    );
}

#[test]
fn an_in_place_answer_asks_for_no_folder_and_sends_no_target() {
    let (mut v, _q, _t) = extracting();
    v.on_key(key(KeyCode::Down));
    v.on_key(key(KeyCode::Enter));
    assert!(!v.has_target_picker());
    assert_eq!(v.choices.extract_mode, Some(ExtractMode::InPlace));
    assert!(v.busy());
    assert_eq!(v.extract_target(), None);
}

#[test]
fn a_finished_clone_opens_its_folder_once_and_the_result_names_it() {
    let (mut v, _q, tmp) = extracting();
    v.on_key(key(KeyCode::Enter));
    let out = tmp.path().join("out");
    std::fs::create_dir_all(&out).unwrap();
    v.on_target_outcome(PickerOutcome::Choose(out.clone()));
    let _ = v.extract_target();
    let outcome = RewriteOutcome {
        run: RunId("run-1".into()),
        moved: Vec::new(),
        commits_changed: 2,
    };
    v.finish_apply(outcome.clone());
    assert_eq!(v.take_open_repo(), Some(out.join("docs")));
    assert_eq!(v.take_open_repo(), None, "once");
    let headline = v.result_headline(&outcome);
    assert!(headline.contains("docs"), "{headline}");
    assert!(headline.contains("was not touched"), "{headline}");
}

#[test]
fn a_collision_is_a_list_of_commits_and_paths() {
    let (mut v, _q, _t) = view();
    v.add_rule(RuleTemplate::MovePaths);
    let error = CoreError::RewriteRefused(
        RewriteRefusal::PathCollision,
        "the target path already exists in 2 commit(s); the commit and the path:\nabc1234: guide\ndef5678: guide"
            .into(),
    );
    v.on_error(&error);
    let refusal = v.refusal.as_ref().expect("a refusal list");
    assert_eq!(refusal.kind, RewriteRefusal::PathCollision);
    assert_eq!(refusal.lines, ["abc1234: guide", "def5678: guide"]);
}
