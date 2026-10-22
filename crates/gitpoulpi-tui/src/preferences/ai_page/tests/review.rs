//! The code review context rows (`[ai.review]`): global on the AI page, with
//! "use global" and Reset on the repository page, and in the Settings search.

use super::*;
use gitpoulpi_config::RunInRepo;

fn global_state(dir: &Path) -> PreferencesState {
    let mut state = state(dir);
    state.config.ai.enabled = true;
    state
}

#[test]
fn the_global_rows_write_config_toml() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = global_state(dir.path());
    // The combobox: Profile setting -> Yes.
    select_in(&mut state, "Code review", "Run the agent in the repository");
    state.choose_next();
    assert!(
        file(dir.path()).contains("run_in_repository = \"yes\""),
        "{}",
        file(dir.path())
    );
    // The checkboxes.
    select_in(&mut state, "Code review", "Include AGENTS.md");
    state.activate();
    assert!(file(dir.path()).contains("include_agents_md = false"));
    assert!(!state.config.ai.review.include_agents_md);
    assert!(state.config.ai.review.include_claude_md, "untouched");
    // The extra file, a text field.
    select_in(&mut state, "Code review", "Extra file");
    state.activate();
    type_text(&mut state, "docs/REVIEW.md");
    state.on_key(key(KeyCode::Enter));
    assert!(
        file(dir.path()).contains("extra_file = \"docs/REVIEW.md\""),
        "{}",
        file(dir.path())
    );
    assert_eq!(state.config.ai.review.run_in_repository, RunInRepo::Yes);
}

#[test]
fn the_repository_rows_inherit_show_the_global_value_and_reset() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = repo_state_on_page(dir.path());
    state.config.ai.review.include_agents_md = false;
    state.config.ai.review.extra_file = "global.md".into();
    select_in(&mut state, "Code review", "Include AGENTS.md");
    let fields = state.fields();
    match &fields[state.field].control {
        Control::Choice { options, index } => {
            assert_eq!(options[0], "Use global (Off)", "the greyed global value");
            assert_eq!(*index, 0);
        }
        other => panic!("{other:?}"),
    }
    let extra = fields.iter().find(|f| f.label == "Extra file").unwrap();
    assert_eq!(
        extra.placeholder.as_deref(),
        Some("global.md"),
        "an empty field shows the global value"
    );
    // Pick "On": the repository overrides the global Off.
    state.choose_next();
    assert_eq!(
        repo_state(dir.path())
            .preferences
            .ai_review
            .include_agents_md,
        Some(true)
    );
    assert!(
        !file(dir.path()).contains("include_agents_md"),
        "config.toml is untouched"
    );
    // A run choice and an extra file of its own.
    select_in(&mut state, "Code review", "Run the agent in the repository");
    state.choose_next(); // Profile setting
    state.choose_next(); // Yes
    select_in(&mut state, "Code review", "Extra file");
    state.activate();
    type_text(&mut state, "docs/R.md");
    state.on_key(key(KeyCode::Enter));
    let own = repo_state(dir.path()).preferences.ai_review;
    assert_eq!(own.run_in_repository, Some(RunInRepo::Yes));
    assert_eq!(own.extra_file.as_deref(), Some("docs/R.md"));
    // Reset to global clears all four.
    let reset = state
        .fields()
        .iter()
        .position(|f| {
            f.label == "Reset to global"
                && matches!(
                    f.target,
                    Target::Action(PrefAction::Ai(AiButton::ResetReview))
                )
        })
        .expect("the reset button of the block");
    state.field = reset;
    state.activate();
    assert!(repo_state(dir.path()).preferences.ai_review.is_empty());
    assert!(!state.fields().iter().any(|f| matches!(
        f.target,
        Target::Action(PrefAction::Ai(AiButton::ResetReview))
    )));
}

#[test]
fn the_no_extra_file_switch_and_a_file_of_the_repository_exclude_each_other() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = repo_state_on_page(dir.path());
    state.config.ai.review.extra_file = "global.md".into();
    select_in(&mut state, "Code review", "No extra file");
    state.activate();
    let own = repo_state(dir.path()).preferences.ai_review;
    assert_eq!(own.no_extra_file, Some(true));
    assert_eq!(own.extra_file, None, "an empty field still inherits");
    let fields = state.fields();
    let extra = fields.iter().find(|f| f.label == "Extra file").unwrap();
    assert_eq!(extra.placeholder.as_deref(), Some("(none)"));
    // Typing a file takes the switch off.
    select_in(&mut state, "Code review", "Extra file");
    state.activate();
    type_text(&mut state, "docs/R.md");
    state.on_key(key(KeyCode::Enter));
    let own = repo_state(dir.path()).preferences.ai_review;
    assert_eq!(own.extra_file.as_deref(), Some("docs/R.md"));
    assert_eq!(own.no_extra_file, None);
    // The switch back on drops the file.
    select_in(&mut state, "Code review", "No extra file");
    state.activate();
    let own = repo_state(dir.path()).preferences.ai_review;
    assert_eq!((own.extra_file, own.no_extra_file), (None, Some(true)));
    // Off again: follows the global file.
    state.activate();
    assert!(repo_state(dir.path()).preferences.ai_review.is_empty());
}

/// Presses the Browse button of `scope`, then picks the last file of the
/// folder shown (files come last) once the lister has answered.
fn browse_and_pick(state: &mut PreferencesState, scope: AiScope) {
    let at = state
        .fields()
        .iter()
        .position(|f| f.target == Target::Action(PrefAction::Ai(AiButton::BrowseExtra(scope))))
        .expect("a Browse button");
    state.field = at;
    state.activate();
    assert!(state.ai.browse_open(), "the picker is up");
    for _ in 0..300 {
        let listed = state.ai.browse.as_mut().is_some_and(|(_, browse)| {
            browse.pump();
            browse.picker().listing().is_some()
        });
        if !listed {
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }
        state.on_key(key(KeyCode::End));
        state.on_key(key(KeyCode::Enter));
        if !state.ai.browse_open() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("no file was picked");
}

#[test]
fn browse_fills_the_repository_row_with_a_path_relative_to_the_repository() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("REVIEW.md"), "rules").unwrap();
    let mut state = repo_state_on_page(dir.path());
    state.config.ai.review.extra_file = "global.md".into();
    browse_and_pick(&mut state, AiScope::Repo);
    let own = repo_state(dir.path()).preferences.ai_review;
    assert_eq!(own.extra_file.as_deref(), Some("REVIEW.md"));
}

#[test]
fn browse_on_the_global_page_stores_an_absolute_path() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = global_state(dir.path());
    state.config.ai.review.extra_file = dir.path().join("REVIEW.md").display().to_string();
    std::fs::write(dir.path().join("REVIEW.md"), "rules").unwrap();
    browse_and_pick(&mut state, AiScope::Global);
    let want = dir.path().join("REVIEW.md").display().to_string();
    assert_eq!(state.config.ai.review.extra_file, want);
    assert!(
        file(dir.path()).contains("extra_file"),
        "{}",
        file(dir.path())
    );
}

#[test]
fn escape_closes_the_picker_and_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut state = repo_state_on_page(dir.path());
    let at = state
        .fields()
        .iter()
        .position(|f| {
            f.target == Target::Action(PrefAction::Ai(AiButton::BrowseExtra(AiScope::Repo)))
        })
        .unwrap();
    state.field = at;
    state.activate();
    assert!(state.ai.browse_open());
    state.on_key(key(KeyCode::Esc));
    assert!(!state.ai.browse_open());
    assert!(repo_state(dir.path()).preferences.ai_review.is_empty());
}

#[test]
fn the_search_finds_the_rows_on_both_pages() {
    let dir = tempfile::tempdir().unwrap();
    let state = repo_state_on_page(dir.path());
    let hits = crate::preferences::search::matches("agents.md", &state.search_entries());
    let sections: Vec<PrefSection> = hits
        .iter()
        .filter(|h| h.label == "Include AGENTS.md")
        .map(|h| PrefSection::ALL[h.section])
        .collect();
    assert!(sections.contains(&PrefSection::Ai), "{sections:?}");
    assert!(sections.contains(&PrefSection::RepoAi), "{sections:?}");
}
