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
