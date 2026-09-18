//! The Workflow page: the template picker and Detect, the base and topic
//! editors with their live previews, the hooks editor, and the two Save
//! buttons (shared file, repo-state override).

use super::super::list_editor::ListOp;
use super::*;
use crate::flow::test_support::view_of;
use crate::flow::view::{self, FlowView};
use gitpoulpi_core::workflow::{import_git_flow, WorkflowTemplate};
use gitpoulpi_session::{load_shared, workflow_path_for, LoadedWorkflow};
use workflow_page::{WfButton, WfKind, WfTarget};

const BRANCHES: &[&str] = &[
    "main",
    "develop",
    "feature/login",
    "feature/search",
    "hotfix/1.0.1",
    "wip-experiment",
];

/// The page on its General tab (the template, hooks, issue flow).
fn page(dir: &Path) -> PreferencesState {
    let mut st = state_at(dir, "Workflow");
    st.remember_sub_tab(PrefSection::Workflow, SubTab::General);
    st
}

fn row(st: &PreferencesState, target: WfTarget) -> usize {
    st.fields()
        .iter()
        .position(|f| f.target == Target::Workflow(target))
        .unwrap_or_else(|| panic!("no row {target:?}"))
}

/// Selects what the list would: a kind and an index within its group.
fn select(st: &mut PreferencesState, kind: WfKind, at: usize) {
    st.wf_draft();
    st.workflow.kind = kind;
    // A rule's rows are on the list tab, everything else on General.
    let tab = if kind == WfKind::General {
        SubTab::General
    } else {
        SubTab::List
    };
    st.remember_sub_tab(PrefSection::Workflow, tab);
    match kind {
        WfKind::General => {}
        WfKind::Base => st.workflow.base = at,
        WfKind::Topic => st.workflow.topic = at,
        WfKind::Finish => st.workflow.finish = at,
    }
}

fn op(st: &mut PreferencesState, op: ListOp) {
    st.apply_list_op(op);
}

fn button(st: &mut PreferencesState, b: WfButton) {
    st.run_action(PrefAction::Workflow(b));
}

fn pick(st: &mut PreferencesState, target: WfTarget, value: &str) {
    st.commit_combo(&Target::Workflow(target), value);
}

fn give_view(st: &mut PreferencesState, branches: &[&str]) {
    let v = view_of(WorkflowTemplate::Custom, branches, "main", &[]);
    st.set_workflow_view(Some(v), 7);
}

fn shared_path(dir: &Path) -> std::path::PathBuf {
    workflow_path_for(dir, ".gitpoulpi")
}

fn write_git_flow(dir: &Path) {
    gitpoulpi_session::save_shared(
        &shared_path(dir),
        &WorkflowTemplate::GitFlow.workflow(),
        true,
    )
    .unwrap();
}

#[test]
fn the_page_is_in_the_repository_group_and_lists_every_part_of_the_definition() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let mut st = page(dir.path());
    assert!(PrefSection::Workflow.is_repo_scoped());
    assert_eq!(PrefSection::Workflow.id(), "workflow");
    let mut labels: Vec<String> = Vec::new();
    for (kind, at) in [
        (WfKind::General, 0),
        (WfKind::Base, 0),
        (WfKind::Topic, 0),
        (WfKind::Finish, 0),
    ] {
        select(&mut st, kind, at);
        labels.extend(st.fields().iter().map(|f| f.label.clone()));
    }
    for want in [
        "Workflow template",
        "Template",
        "Detect from the branches",
        "Base branches",
        "Name",
        "Merge strategy",
        "Topic types",
        "Name pattern",
        "Finish rules",
        "Finish mode",
        "Hooks",
        "Issue flow (host-driven)",
        "No changes",
    ] {
        assert!(
            labels.iter().any(|l| l == want),
            "{want} missing: {labels:?}"
        );
    }
    select(&mut st, WfKind::General, 0);
    let screen = draw_screen(&mut st);
    assert!(
        screen.contains("Workflow"),
        "the sidebar lists the page:\n{screen}"
    );
    assert!(screen.contains("Template"), "{screen}");
}

#[test]
fn picking_a_template_fills_the_draft_and_save_writes_the_shared_file() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    // Nothing saved yet: an empty custom definition.
    assert!(st.wf_draft().bases.is_empty());
    pick(&mut st, WfTarget::Template, "git-flow");
    let labels: Vec<String> = st
        .fields()
        .iter()
        .filter_map(|f| match &f.control {
            Control::Combo { value, .. } if f.target == Target::Workflow(WfTarget::Template) => {
                Some(value.clone())
            }
            _ => None,
        })
        .collect();
    assert_eq!(labels, ["git-flow"]);
    assert!(!shared_path(dir.path()).exists(), "picking saves nothing");
    button(&mut st, WfButton::SaveShared);
    let saved = load_shared(&shared_path(dir.path()))
        .unwrap()
        .unwrap()
        .workflow;
    let preset = WorkflowTemplate::GitFlow.workflow();
    assert_eq!(saved.template, WorkflowTemplate::GitFlow);
    assert_eq!(saved.bases, preset.bases);
    assert_eq!(saved.topics, preset.topics);
    let text = std::fs::read_to_string(shared_path(dir.path())).unwrap();
    assert!(text.contains("template = \"git-flow\""), "{text}");
    assert!(text.contains("feature/{slug}"), "{text}");
    assert!(st
        .take_workflow_saved()
        .is_some_and(|m| m.contains("workflow.toml")));
}

#[test]
fn a_template_is_picked_through_the_dropdown_with_the_keyboard() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    st.focus = PrefFocus::Fields;
    st.field = row(&st, WfTarget::Template);
    st.activate();
    assert!(st.choice.is_some(), "a combo, never click-to-cycle");
    let screen = draw_screen(&mut st);
    for want in ["Git Flow", "GitHub Flow", "GitLab Flow", "Trunk-based"] {
        assert!(screen.contains(want), "{want}:\n{screen}");
    }
    // The list opens on the current template (Custom, the last one): Home,
    // then Enter picks the first, Git Flow.
    let k = |c| KeyEvent::new(c, KeyModifiers::NONE);
    st.on_key(k(KeyCode::Home));
    st.on_key(k(KeyCode::Enter));
    select(&mut st, WfKind::Base, 0);
    let bases: Vec<String> = st
        .fields()
        .iter()
        .filter_map(|f| match (&f.control, &f.target) {
            (Control::Text(v), Target::Workflow(WfTarget::BaseName)) => Some(v.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(bases, ["main"]);
}

#[test]
fn detect_shows_the_score_and_the_names_it_found_and_use_adopts_them() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    // No branches read yet: it says so.
    button(&mut st, WfButton::Detect);
    assert!(draw_screen(&mut st).contains("No branches were read yet"));
    give_view(&mut st, BRANCHES);
    button(&mut st, WfButton::Detect);
    let rows: Vec<(String, String)> = st
        .fields()
        .iter()
        .filter_map(|f| match &f.control {
            Control::Info(t) => Some((f.label.clone(), t.clone())),
            _ => None,
        })
        .collect();
    let find = |label: &str| {
        rows.iter()
            .find(|(l, _)| l == label)
            .map(|(_, t)| t.clone())
            .unwrap_or_else(|| panic!("{label}: {rows:?}"))
    };
    let best = find("Best match");
    assert!(
        best.starts_with("Closest template: Git Flow (score "),
        "{best}"
    );
    assert_eq!(find("Base branches found"), "main, develop");
    let topics: Vec<&String> = rows
        .iter()
        .filter(|(l, _)| l == "Topic types found")
        .map(|(_, t)| t)
        .collect();
    assert!(
        topics
            .iter()
            .any(|t| t.starts_with("feature: 2 branches (feature/login")),
        "{topics:?}"
    );
    assert!(
        topics.iter().any(|t| t.starts_with("hotfix: 1 branch")),
        "{topics:?}"
    );
    assert!(find("Scores").contains("GitHub Flow"));
    // Use it: the draft is the detected workflow, and nothing is saved.
    button(&mut st, WfButton::UseDetected);
    assert_eq!(st.wf_draft().template, WorkflowTemplate::GitFlow);
    assert!(!shared_path(dir.path()).exists());
}

#[test]
fn a_git_flow_config_in_the_repository_is_offered_for_import() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = page(dir.path());
    assert!(!st.fields().iter().any(|f| f.label == "Import it"));
    let entries: Vec<(String, String)> = [
        ("gitflow.branch.master", "trunk"),
        ("gitflow.branch.develop", "dev"),
        ("gitflow.prefix.feature", "feat/"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    let imported = import_git_flow(&entries).expect("git-flow config");
    let mut report = view_of(WorkflowTemplate::Custom, BRANCHES, "main", &[])
        .report
        .clone();
    report.imports = vec![imported.clone()];
    st.set_workflow_view(Some(std::sync::Arc::new(FlowView::new(report))), 9);
    let labels: Vec<String> = st.fields().iter().map(|f| f.label.clone()).collect();
    assert!(
        labels.contains(&"Found in this repository".to_string()),
        "{labels:?}"
    );
    assert!(labels.contains(&"Import it".to_string()));
    button(&mut st, WfButton::Import);
    assert_eq!(st.wf_draft().bases[0].name, "trunk");
    assert_eq!(st.wf_draft().bases, imported.workflow.bases);
}

#[test]
fn a_pattern_previews_the_branches_it_matches_and_a_name_it_would_create() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let mut st = page(dir.path());
    give_view(&mut st, BRANCHES);
    // The first topic type is Feature.
    select(&mut st, WfKind::Topic, 0);
    let info = |st: &PreferencesState, label: &str| -> Option<String> {
        st.fields().iter().find_map(|f| match &f.control {
            Control::Info(t) if f.label.trim() == label => Some(t.clone()),
            _ => None,
        })
    };
    assert_eq!(
        info(&st, "Matches now").as_deref(),
        Some("feature/login, feature/search")
    );
    assert_eq!(
        info(&st, "Would create").as_deref(),
        Some("feature/add-login")
    );
    // Editing the pattern re-previews at once.
    assert!(st.wf_commit_text(WfTarget::TopicPattern, "hotfix/{slug}"));
    assert_eq!(
        info(&st, "Would create").as_deref(),
        Some("hotfix/add-login")
    );
    // A broken pattern is kept (nothing typed is lost) and says why, inline.
    assert!(st.wf_commit_text(WfTarget::TopicPattern, "feature/{slug"));
    let f = st.fields();
    let at = f
        .iter()
        .position(|f| f.target == Target::Workflow(WfTarget::TopicPattern))
        .unwrap();
    let warning = f[at].warning.clone().expect("an inline error");
    assert!(warning.contains("unclosed"), "{warning}");
    assert!(info(&st, "Would create").is_none());
}

#[test]
fn editing_bases_and_topics_keeps_the_references_between_them() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let mut st = page(dir.path());
    // Rename `main`: develop's parent and the finish targets follow.
    assert!(st.wf_commit_text(WfTarget::BaseName, "trunk"));
    let wf = st.wf_draft().clone();
    let develop = wf.bases.iter().find(|b| b.name == "develop").unwrap();
    assert_eq!(develop.parent.as_deref(), Some("trunk"));
    assert!(wf
        .topics
        .iter()
        .flat_map(|t| &t.finish)
        .all(|f| f.into != "main"));
    // An empty or a taken name is refused with a message.
    assert!(!st.wf_commit_text(WfTarget::BaseName, ""));
    assert!(st.message.is_some());
    assert!(!st.wf_commit_text(WfTarget::BaseName, "develop"));
    // Remove the selected base: nobody keeps a parent that is gone.
    op(&mut st, ListOp::Remove(1));
    let wf = st.wf_draft();
    assert!(wf
        .bases
        .iter()
        .all(|b| b.parent.as_deref() != Some("trunk")));
    // Add a topic type, a finish target, move it.
    let before = wf.topics.len();
    op(&mut st, ListOp::Add(Some(1)));
    assert_eq!(st.wf_draft().topics.len(), before + 1);
    assert_eq!(st.workflow.topic, before, "the new type is selected");
    op(&mut st, ListOp::Add(Some(2)));
    assert_eq!(st.wf_draft().topics[before].finish.len(), 1);
    // Back on the topic type, move it up within its group.
    select(&mut st, WfKind::Topic, before);
    let at = st.wf_list_selected();
    op(
        &mut st,
        ListOp::Move {
            from: at,
            to: at - 1,
        },
    );
    assert_eq!(st.workflow.topic, before - 1);
    assert_eq!(st.wf_draft().topics[before - 1].id, "topic");
}

#[test]
fn choices_are_dropdowns_not_click_to_cycle() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let st = page(dir.path());
    let mut st = st;
    for (target, kind) in [
        (WfTarget::BaseParent, WfKind::Base),
        (WfTarget::BaseMerge, WfKind::Base),
        (WfTarget::TopicParent, WfKind::Topic),
        (WfTarget::FinishMerge, WfKind::Finish),
        (WfTarget::TopicFinishMode, WfKind::Topic),
        (WfTarget::IssueMode, WfKind::General),
        (WfTarget::Template, WfKind::General),
    ] {
        select(&mut st, kind, 0);
        let f = &st.fields()[row(&st, target)];
        assert!(
            matches!(f.control, Control::Combo { .. }),
            "{target:?} is a combobox"
        );
    }
}

#[test]
fn hooks_are_edited_in_a_multi_line_editor_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let mut st = page(dir.path());
    let hook_row = &st.fields()[row(&st, WfTarget::HookBeforeFinish)];
    assert!(matches!(hook_row.control, Control::Area(_)));
    assert!(hook_row
        .warning
        .as_deref()
        .is_some_and(|w| w.contains("shell commands")));
    st.focus = PrefFocus::Fields;
    st.field = row(&st, WfTarget::HookBeforeFinish);
    st.activate();
    assert!(st.wf_editor_open());
    let screen = draw_screen(&mut st);
    assert!(screen.contains("Before finish"), "{screen}");
    assert!(screen.contains("run shell commands"), "{screen}");
    let k = |c| KeyEvent::new(c, KeyModifiers::NONE);
    for c in "cargo test".chars() {
        st.on_key(k(KeyCode::Char(c)));
    }
    st.on_key(k(KeyCode::Enter));
    for c in "cargo fmt --check".chars() {
        st.on_key(k(KeyCode::Char(c)));
    }
    st.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL));
    assert!(!st.wf_editor_open());
    assert_eq!(
        st.wf_draft().hooks.before_finish,
        ["cargo test", "cargo fmt --check"]
    );
    // Esc discards.
    st.activate();
    st.on_key(k(KeyCode::Char('x')));
    st.on_key(k(KeyCode::Esc));
    assert_eq!(st.wf_draft().hooks.before_finish.len(), 2);
}

#[test]
fn only_for_me_writes_the_difference_to_the_repo_state_and_leaves_the_shared_file() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let before = std::fs::read_to_string(shared_path(dir.path())).unwrap();
    let mut st = page(dir.path());
    st.focus = PrefFocus::Fields;
    // Protect `develop` and change a topic's label, for me only.
    select(&mut st, WfKind::Base, 1);
    st.wf_commit_bool(WfTarget::BaseProtected, true);
    st.wf_commit_text(WfTarget::TopicLabel, "Feat");
    button(&mut st, WfButton::SaveMine);
    assert_eq!(
        std::fs::read_to_string(shared_path(dir.path())).unwrap(),
        before,
        "the shared file is untouched"
    );
    let loaded = LoadedWorkflow::load(dir.path(), ".gitpoulpi").unwrap();
    let ovr = loaded.user.clone().expect("an override was written");
    assert_eq!(ovr.bases.len(), 1);
    assert_eq!(ovr.bases[0].name, "develop");
    assert_eq!(ovr.topics.len(), 1);
    let eff = loaded.effective();
    assert!(
        eff.bases
            .iter()
            .find(|b| b.name == "develop")
            .unwrap()
            .protected
    );
    assert_eq!(eff.topics[0].label, "Feat");
    // And the teammates' view (the shared file alone) is the preset.
    assert_eq!(
        loaded.shared.unwrap().workflow,
        WorkflowTemplate::GitFlow.workflow()
    );
}

#[test]
fn saving_shared_folds_the_users_override_in_and_drops_it() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let mut st = page(dir.path());
    st.wf_commit_text(WfTarget::TopicLabel, "Feat");
    button(&mut st, WfButton::SaveMine);
    let mut st = page(dir.path());
    assert_eq!(st.wf_draft().topics[0].label, "Feat", "the override shows");
    button(&mut st, WfButton::SaveShared);
    let loaded = LoadedWorkflow::load(dir.path(), ".gitpoulpi").unwrap();
    assert!(
        loaded.user.is_none(),
        "the override is folded into the file"
    );
    assert_eq!(loaded.shared.unwrap().workflow.topics[0].label, "Feat");
}

#[test]
fn discard_reads_the_saved_workflow_again() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let mut st = page(dir.path());
    st.wf_commit_text(WfTarget::TopicLabel, "Changed");
    assert_eq!(st.wf_draft().topics[0].label, "Changed");
    button(&mut st, WfButton::Discard);
    assert_eq!(st.wf_draft().topics[0].label, "Feature");
}

#[test]
fn the_page_is_reachable_by_its_ipc_id_and_the_flow_menu() {
    let dir = tempfile::tempdir().unwrap();
    let mut st = state(dir.path());
    st.show_workflow();
    assert_eq!(st.current_section(), PrefSection::Workflow);
    assert_eq!(st.focus, PrefFocus::Fields);
    assert!(gitpoulpi_ipc::ui::SettingsPage::parse("workflow").is_some());
    assert!(view::version() < u64::MAX);
}

fn draw_small(st: &mut PreferencesState, height: u16) -> Vec<String> {
    use ratatui::backend::TestBackend;
    let mut terminal = ratatui::Terminal::new(TestBackend::new(110, height)).unwrap();
    terminal
        .draw(|frame| st.render(frame, frame.area()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| {
            (0..110)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect()
}

#[test]
fn the_list_groups_bases_topic_types_and_finish_rules() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let mut st = page(dir.path());
    st.remember_sub_tab(PrefSection::Workflow, SubTab::List);
    st.sync_list();
    let items = &st.list.spec.items;
    let wf = WorkflowTemplate::GitFlow.workflow();
    let finish = wf.topics[0].finish.len();
    assert_eq!(items.len(), wf.bases.len() + wf.topics.len() + finish);
    assert!(
        items.iter().all(|i| i.group.as_deref() != Some("General")),
        "General is the other tab, not a list row"
    );
    let groups: Vec<&str> = items
        .iter()
        .filter_map(|i| i.group.as_deref())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    assert_eq!(groups, ["Base branches", "Finish rules", "Topic types"]);
    let screen = draw_small(&mut st, 40).join("\n");
    for header in ["Base branches", "Topic types", "Finish rules"] {
        assert!(screen.contains(header), "{header}:\n{screen}");
    }
    // Selecting a group's item swaps the detail rows.
    st.wf_select_index(1);
    assert_eq!(st.wf_kind(), WfKind::Base);
    assert!(st.fields().iter().any(|f| f.label == "Other names"));
}

#[test]
fn reordering_stays_inside_a_group() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let mut st = page(dir.path());
    st.remember_sub_tab(PrefSection::Workflow, SubTab::List);
    let nb = st.wf_draft().bases.len();
    st.sync_list();
    // The last base: down would cross into the topic types.
    st.list.select(nb - 1);
    assert!(st.list.can_move_up());
    assert!(!st.list.can_move_down(), "never across a group");
    // The first base: up would leave the group.
    st.list.select(0);
    assert!(!st.list.can_move_up());
    assert!(st.list.can_remove(), "a rule can be removed");
    // A forged cross-group move changes nothing.
    let before = st.wf_draft().clone();
    op(
        &mut st,
        ListOp::Move {
            from: nb - 1,
            to: nb,
        },
    );
    assert_eq!(*st.wf_draft(), before);
}

#[test]
fn the_save_bar_shows_the_dirty_mark_and_clears_after_a_save() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let mut st = page(dir.path());
    let bar = |st: &PreferencesState| st.fields().last().map(|f| f.label.clone()).unwrap();
    assert_eq!(bar(&st), "No changes");
    assert!(!st.wf_dirty());
    st.wf_commit_text(WfTarget::TopicLabel, "Feat");
    assert!(st.wf_dirty());
    assert_eq!(bar(&st), "\u{25CF} Unsaved");
    button(&mut st, WfButton::SaveMine);
    assert!(!st.wf_dirty(), "saved: the bar is clean again");
    assert_eq!(bar(&st), "No changes");
}

#[test]
fn the_save_bar_stays_visible_when_the_page_scrolls() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let mut st = page(dir.path());
    st.wf_commit_text(WfTarget::TopicLabel, "Feat");
    // A short window: the General rows do not fit, so the page scrolls.
    let top = draw_small(&mut st, 16);
    let bar_row = |rows: &[String]| {
        rows.iter()
            .position(|r| r.contains("Unsaved") && r.contains("Save to the shared file"))
    };
    let at = bar_row(&top).unwrap_or_else(|| panic!("bar at the top:\n{}", top.join("\n")));
    st.scroll = 30;
    let scrolled = draw_small(&mut st, 16);
    assert_ne!(top, scrolled, "the body really scrolled");
    assert_eq!(
        bar_row(&scrolled),
        Some(at),
        "the bar stays on its row:\n{}",
        scrolled.join("\n")
    );
    assert!(at >= 9, "pinned at the bottom of the page: row {at}");
}

#[test]
fn only_for_me_keeps_a_reorder_of_topic_types_and_finish_rules_across_a_reload() {
    let dir = tempfile::tempdir().unwrap();
    write_git_flow(dir.path());
    let shared = WorkflowTemplate::GitFlow.workflow();
    let mut st = page(dir.path());
    // Topic types: move the first one to the end. Finish rules: reverse the
    // first topic type's two rules (added here, so there are two).
    let n = st.wf_draft().topics.len();
    let nb = st.wf_draft().bases.len();
    select(&mut st, WfKind::Topic, 0);
    op(&mut st, ListOp::Add(Some(2)));
    op(&mut st, ListOp::Add(Some(2)));
    let first = st.wf_draft().topics[0].finish.clone();
    assert!(first.len() >= 2);
    let finish_at = nb + n;
    op(
        &mut st,
        ListOp::Move {
            from: finish_at,
            to: finish_at + 1,
        },
    );
    op(
        &mut st,
        ListOp::Move {
            from: nb,
            to: nb + n - 1,
        },
    );
    let want = st.wf_draft().clone();
    assert_eq!(want.topics.last().unwrap().id, shared.topics[0].id);
    button(&mut st, WfButton::SaveMine);
    let reloaded = page(dir.path());
    assert_eq!(*reloaded.wf_draft(), want, "the order survives a reload");
    let ids: Vec<&str> = reloaded
        .wf_draft()
        .topics
        .iter()
        .map(|t| t.id.as_str())
        .collect();
    assert_ne!(
        ids,
        shared
            .topics
            .iter()
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>()
    );
}

#[test]
fn an_unchanged_order_stores_no_order_in_the_override() {
    let shared = WorkflowTemplate::GitFlow.workflow();
    let mut draft = shared.clone();
    draft.topics[0].label = "Feat".into();
    let ovr = workflow_page::override_from(&shared, &draft);
    assert!(ovr.topic_order.is_empty() && ovr.base_order.is_empty());
}
