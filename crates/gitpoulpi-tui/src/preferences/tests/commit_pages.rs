//! The Commit, Merge options and Message template pages of the repository
//! group: their rows, what a toggle saves, the template file and the
//! `commit.template` request of the Save button.

use super::*;
use crate::preferences::repo_pages::{RpButton, RpTarget};
use crossterm::event::{KeyEvent, KeyModifiers};

fn saved(dir: &Path) -> gitpoulpi_session::RepoPreferences {
    RepoState::load(&RepoState::path_for(dir, ".gitpoulpi"))
        .unwrap()
        .preferences
}

fn select(st: &mut PreferencesState, label_start: &str) {
    st.focus = PrefFocus::Fields;
    st.field = st
        .fields()
        .iter()
        .position(|f| f.label.starts_with(label_start))
        .unwrap_or_else(|| {
            panic!(
                "no `{label_start}` in {:?}",
                st.fields().iter().map(|f| &f.label).collect::<Vec<_>>()
            )
        });
}

fn toggled(st: &PreferencesState, label_start: &str) -> bool {
    match st
        .fields()
        .iter()
        .find(|f| f.label.starts_with(label_start))
        .unwrap()
        .control
    {
        Control::Toggle(on) => on,
        ref other => panic!("{label_start} is {other:?}"),
    }
}

fn type_in_editor(st: &mut PreferencesState, text: &str) {
    for c in text.chars() {
        let code = if c == '\n' {
            KeyCode::Enter
        } else {
            KeyCode::Char(c)
        };
        st.on_key(KeyEvent::new(code, KeyModifiers::NONE));
    }
    st.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
}

#[test]
fn the_three_pages_are_in_the_repository_group_with_stable_ids() {
    for (section, id) in [
        (PrefSection::Commit, "commit"),
        (PrefSection::MergeBehavior, "merge-behavior"),
        (PrefSection::CommitTemplate, "commit-template"),
    ] {
        assert!(section.is_repo_scoped(), "{id}");
        assert_eq!(section.id(), id);
        let page = gitpoulpi_ipc::ui::SettingsPage::parse(id).unwrap();
        assert_eq!(PrefSection::from_page(page), section);
        assert!(section.is_built());
    }
}

#[test]
fn every_switch_starts_off_and_is_saved_in_the_repo_data() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "Commit");
    assert!(!toggled(&st, "Push on every commit"));
    assert!(!toggled(&st, "Bypass git hooks"));
    select(&mut st, "Push on every commit");
    st.activate();
    assert!(toggled(&st, "Push on every commit"));
    select(&mut st, "Bypass git hooks");
    st.activate();
    let on = saved(dir.path());
    assert_eq!(on.push_after_commit, Some(true));
    assert_eq!(on.skip_hooks_by_default, Some(true));
    // Back off: the file records the choice, not a missing key.
    select(&mut st, "Push on every commit");
    st.activate();
    assert_eq!(saved(dir.path()).push_after_commit, Some(false));

    go(&mut st, PrefSection::MergeBehavior);
    assert!(!toggled(&st, "Squash"));
    select(&mut st, "Squash");
    st.activate();
    assert_eq!(saved(dir.path()).squash_merge, Some(true));
    let help = st.fields()[0].help.clone().unwrap();
    assert!(help.contains("yourself"), "{help}");

    go(&mut st, PrefSection::CommitTemplate);
    assert!(!toggled(&st, "Prefill an empty"));
    assert!(!toggled(&st, "Drop comment lines"));
    select(&mut st, "Prefill an empty");
    st.activate();
    select(&mut st, "Drop comment lines");
    st.activate();
    let prefs = saved(dir.path());
    assert_eq!(prefs.template_apply, Some(true));
    assert_eq!(prefs.template_strip_comments, Some(true));
}

#[test]
fn the_summary_counts_down_from_72_and_goes_over() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "CommitTemplate");
    assert!(st.fields().iter().any(|f| f.label == "Summary (72 left)"));
    select(&mut st, "Summary");
    st.activate();
    for c in "fix: a thing".chars() {
        st.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }
    // Live while typing, before the value is committed.
    assert!(
        st.fields().iter().any(|f| f.label == "Summary (60 left)"),
        "{:?}",
        st.fields().iter().map(|f| &f.label).collect::<Vec<_>>()
    );
    st.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert!(st.fields().iter().any(|f| f.label == "Summary (60 left)"));
    // Longer than the limit: it says by how much.
    st.repo_pages.template.summary = "x".repeat(80);
    assert!(st.fields().iter().any(|f| f.label == "Summary (8 over)"));
}

#[test]
fn save_writes_the_template_file_and_points_commit_template_at_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "CommitTemplate");
    select(&mut st, "Summary");
    st.activate();
    for c in "feat: title".chars() {
        st.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE));
    }
    st.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    st.rp_open_editor(RpTarget::TemplateDescription);
    assert!(st.rp_editor_open());
    type_in_editor(&mut st, "why\n# a comment");
    assert!(!st.rp_editor_open());
    assert!(st.repo_pages.template.dirty);

    st.run_action(PrefAction::RepoPage(RpButton::SaveTemplate));
    let file = dir.path().join(".gitpoulpi").join("commit-template.txt");
    assert_eq!(
        std::fs::read_to_string(&file).unwrap(),
        "feat: title\n\nwhy\n# a comment\n"
    );
    assert!(!st.repo_pages.template.dirty);
    assert_eq!(
        st.take_repo_config_requests(),
        vec![(
            "commit.template".to_string(),
            Some(file.to_string_lossy().into_owned())
        )]
    );
    assert!(st.take_message().unwrap().contains("commit-template.txt"));
    // Own file: no "points elsewhere" row.
    assert!(!st
        .fields()
        .iter()
        .any(|f| f.label.contains("this page edits that file")));
}

#[test]
fn a_commit_template_that_points_elsewhere_is_shown_edited_and_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let elsewhere = dir.path().join("team-template.txt");
    std::fs::write(&elsewhere, "team: summary\n\nteam body\n").unwrap();
    let mut st = state_at(dir.path(), "CommitTemplate");
    let mut values = std::collections::BTreeMap::new();
    values.insert(
        "commit.template".to_string(),
        Some(elsewhere.to_string_lossy().into_owned()),
    );
    st.set_repo_config(&values, 1);
    let fields = st.fields();
    assert!(fields
        .iter()
        .any(|f| f.label.contains("team-template.txt") && f.label.contains("edits that file")));
    let summary = fields
        .iter()
        .find(|f| f.label.starts_with("Summary"))
        .unwrap();
    assert_eq!(summary.control, Control::Text("team: summary".into()));
    assert_eq!(
        fields
            .iter()
            .find(|f| f.label == "Description")
            .unwrap()
            .control,
        Control::Area("team body".into())
    );

    st.rp_open_editor(RpTarget::TemplateDescription);
    type_in_editor(&mut st, "\nmore");
    st.run_action(PrefAction::RepoPage(RpButton::SaveTemplate));
    assert_eq!(
        std::fs::read_to_string(&elsewhere).unwrap(),
        "team: summary\n\nteam body\nmore\n"
    );
    assert!(
        st.take_repo_config_requests().is_empty(),
        "commit.template already names a file: untouched"
    );
    assert!(!dir
        .path()
        .join(".gitpoulpi")
        .join("commit-template.txt")
        .exists());
}

#[test]
fn unsaved_edits_survive_a_config_refresh_but_a_new_file_loads() {
    let dir = tempfile::tempdir().unwrap();
    let other = dir.path().join("other.txt");
    std::fs::write(&other, "other\n").unwrap();
    let mut st = state_at(dir.path(), "CommitTemplate");
    st.rp_open_editor(RpTarget::TemplateDescription);
    type_in_editor(&mut st, "draft");
    let mut values = std::collections::BTreeMap::new();
    values.insert(
        "commit.template".to_string(),
        Some(other.to_string_lossy().into_owned()),
    );
    st.set_repo_config(&values, 1);
    assert_eq!(st.repo_pages.template.description, "draft");
    st.template_reload(true);
    assert_eq!(st.repo_pages.template.summary, "other");
}

#[test]
fn the_pages_draw_their_rows() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "Commit");
    let text = draw_screen(&mut st);
    assert!(text.contains("Push on every commit"), "{text}");
    assert!(text.contains("Bypass git hooks"), "{text}");
    go(&mut st, PrefSection::CommitTemplate);
    let text = draw_screen(&mut st);
    assert!(text.contains("Summary (72 left)"), "{text}");
    assert!(text.contains("Save template"), "{text}");
}

/// Merge options > Fast-forward: (inherited) writes nothing, a pick is saved
/// in the repo data in `MergeFastForward`'s own spelling, and git's own
/// `merge.ff`, when the repository sets it, is shown beside the choice.
#[test]
fn the_fast_forward_choice_is_saved_and_git_merge_ff_is_shown() {
    use gitpoulpi_core::domain::MergeFastForward;
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "MergeBehavior");
    let row = |st: &PreferencesState| {
        st.fields()
            .into_iter()
            .find(|f| f.label == "Fast-forward")
            .unwrap()
    };
    let Control::Choice { options, index } = row(&st).control else {
        panic!("a choice");
    };
    assert_eq!(index, 0, "inherited until chosen");
    assert_eq!(options, ["(inherited)", "never", "allowed", "only"]);
    assert_eq!(saved_ff(dir.path()), None);
    assert!(
        !st.fields().iter().any(|f| f.label.contains("merge.ff")),
        "nothing about git's setting while it is unset"
    );

    for (pick, want) in [(2, "allowed"), (3, "only"), (1, "never")] {
        st.commit_choice(&Target::Repo(RepoField::MergeFastForward), pick);
        assert_eq!(saved_ff(dir.path()).as_deref(), Some(want));
        assert_eq!(
            MergeFastForward::parse(want),
            Some(MergeFastForward::ALL[pick - 1])
        );
        assert!(matches!(row(&st).control, Control::Choice { index, .. } if index == pick));
    }
    st.commit_choice(&Target::Repo(RepoField::MergeFastForward), 0);
    assert_eq!(saved_ff(dir.path()), None, "back to inherited");

    let mut values = std::collections::BTreeMap::new();
    values.insert("merge.ff".to_string(), Some("false".to_string()));
    st.set_repo_config(&values, 1);
    assert!(
        st.fields()
            .iter()
            .any(|f| f.label.contains("merge.ff") && f.label.contains("false")),
        "{:?}",
        st.fields().iter().map(|f| &f.label).collect::<Vec<_>>()
    );
}

fn saved_ff(dir: &Path) -> Option<String> {
    saved(dir).merge_fast_forward
}

/// The global row on the Git page defaults to `never` and writes the config key.
#[test]
fn the_global_fast_forward_row_defaults_to_never() {
    let dir = tempfile::tempdir().unwrap();
    let st = state_at(dir.path(), "GitAndEncoding");
    let row = st
        .fields()
        .into_iter()
        .find(|f| matches!(f.target, Target::Global("git.merge_fast_forward")))
        .expect("the global row");
    assert!(matches!(row.control, Control::Choice { index: 0, .. }));
}

#[test]
fn the_git_page_has_a_checkbox_per_remote_for_new_branch_push_targets() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "GitAndEncoding");
    // No remote: a heading and a note, nothing to tick.
    let none = st.fields();
    assert!(none
        .iter()
        .any(|f| f.label == "Extra push targets for new branches"));
    assert!(none.iter().any(|f| f.label == "(no remote configured)"));

    st.set_remote_names(&["origin".into(), "gitlab".into(), "backup".into()]);
    let labels: Vec<String> = st.fields().iter().map(|f| f.label.clone()).collect();
    let heading = labels
        .iter()
        .position(|l| l == "Extra push targets for new branches")
        .unwrap();
    assert_eq!(
        &labels[heading + 1..heading + 4],
        ["origin", "gitlab", "backup"]
    );
    assert!(!toggled(&st, "gitlab"));

    // Ticking saves into the repository's own data, in the remotes' order.
    select(&mut st, "backup");
    st.activate();
    select(&mut st, "gitlab");
    st.activate();
    assert!(toggled(&st, "gitlab") && toggled(&st, "backup") && !toggled(&st, "origin"));
    assert_eq!(saved(dir.path()).default_push_targets, ["gitlab", "backup"]);
    // Unticking removes it; an empty list is not written at all.
    select(&mut st, "gitlab");
    st.activate();
    select(&mut st, "backup");
    st.activate();
    assert!(saved(dir.path()).default_push_targets.is_empty());
    let raw = std::fs::read_to_string(RepoState::path_for(dir.path(), ".gitpoulpi")).unwrap();
    assert!(!raw.contains("default_push_targets"), "{raw}");
}

#[test]
fn the_group_preflight_row_is_a_global_toggle_on_by_default_and_saved_to_config() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state_at(dir.path(), "GitAndEncoding");
    assert!(toggled(&st, "Check every remote before a push"));
    let help = st
        .fields()
        .iter()
        .find(|f| f.label.starts_with("Check every remote"))
        .and_then(|f| f.help.clone())
        .unwrap();
    assert!(help.contains("third remote"), "{help}");
    select(&mut st, "Check every remote");
    st.activate();
    assert!(!toggled(&st, "Check every remote before a push"));
    let raw = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(raw.contains("push_group_preflight = false"), "{raw}");
}
